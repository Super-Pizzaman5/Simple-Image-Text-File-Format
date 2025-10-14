use minifb::{Window, WindowOptions, Key};
use regex::Regex;
use std::fs;

/// Parse SITF color tokens
fn parse_color_token(token: &str) -> (u8, u8, u8) {
    match token {
        "!F" => (255, 255, 255),
        "!0" => (0, 0, 0),
        "!R" => (255, 0, 0),
        "!G" => (0, 255, 0),
        "!B" => (0, 0, 255),
        _ if token.starts_with('#') && token.len() == 7 => {
            let r = u8::from_str_radix(&token[1..3], 16).unwrap_or(0);
            let g = u8::from_str_radix(&token[3..5], 16).unwrap_or(0);
            let b = u8::from_str_radix(&token[5..7], 16).unwrap_or(0);
            (r, g, b)
        }
        _ if token.starts_with('%') => {
            if let Some((num, den)) = token[1..].split_once('/') {
                let num: f32 = num.parse().unwrap_or(0.0);
                let den: f32 = den.parse().unwrap_or(1.0);
                let frac = (num / den).clamp(0.0, 1.0);
                let v = (frac * 255.0).round() as u8;
                (v, v, v)
            } else {
                (0, 0, 0)
            }
        }
        _ => (0, 0, 0),
    }
}

/// Load SITF file into a pixel buffer
fn load_sitf(input: &str) -> (Vec<u32>, usize, usize) {
    let data = fs::read_to_string(input).expect("Failed to read file");
    let pixels_str = data.split_once('@').map(|(_, p)| p).unwrap_or(&data);

    let re = Regex::new(
        r"(?:(\d+)-(\d+)|(\d+)):(\d+)([+\-]\d*|[+])(!F|!0|!R|!G|!B|#[0-9A-Fa-f]{6}|%[0-9]+/[0-9]+)"
    ).unwrap();

    let mut pixel_vec = Vec::new();
    let mut max_x = 0;
    let mut max_y = 0;

    for cap in re.captures_iter(pixels_str) {
        let x_start: u32 = cap.get(1).map_or_else(
            || cap.get(3).unwrap().as_str().parse().unwrap_or(1),
            |m| m.as_str().parse().unwrap_or(1),
        );
        let x_end: u32 = cap.get(2).map_or(x_start, |m| m.as_str().parse().unwrap_or(x_start));
        let y: u32 = cap[4].parse().unwrap_or(1);
        let trans = cap.get(5).unwrap().as_str();
        let color_token = cap.get(6).unwrap().as_str();

        let alpha = if trans.starts_with('-') {
            let pct: u8 = trans[1..].parse().unwrap_or(0);
            ((100u32.saturating_sub(pct as u32)) as f32 / 100.0 * 255.0).round() as u8
        } else {
            255u8
        };

        let (r, g, b) = parse_color_token(color_token);

        for x in x_start..=x_end {
            pixel_vec.push((x - 1, y - 1, ((alpha as u32) << 24) | ((r as u32) << 16) | ((g as u32) << 8) | b as u32));
            if x > max_x { max_x = x; }
        }
        if y > max_y { max_y = y; }
    }

    let mut buffer = vec![0; (max_x * max_y) as usize];
    for (x, y, px) in pixel_vec {
        let idx = (y * max_x + x) as usize;
        buffer[idx] = px;
    }

    (buffer, max_x as usize, max_y as usize)
}

/// Display a SITF image with scaling
pub fn view_sitf(file: &str) {
    let (buffer, width, height) = load_sitf(file);

    let mut window = Window::new(
        "SITF Viewer",
        width,
        height,
        WindowOptions {
            resize: true,
            ..Default::default()
        },
    ).unwrap();

    while window.is_open() && !window.is_key_down(Key::Escape) {
        let (win_w, win_h) = window.get_size();
        let mut scaled_buffer = vec![0u32; (win_w * win_h) as usize];

        for y in 0..win_h {
            let src_y = (y as f32) * (height as f32) / (win_h as f32);
            let y0 = src_y.floor().clamp(0.0, (height - 1) as f32) as usize;
            let y1 = (y0 + 1).min(height - 1);
            let fy = src_y - y0 as f32;

            for x in 0..win_w {
                let src_x = (x as f32) * (width as f32) / (win_w as f32);
                let x0 = src_x.floor().clamp(0.0, (width - 1) as f32) as usize;
                let x1 = (x0 + 1).min(width - 1);
                let fx = src_x - x0 as f32;

                let c00 = buffer[y0 * width + x0];
                let c10 = buffer[y0 * width + x1];
                let c01 = buffer[y1 * width + x0];
                let c11 = buffer[y1 * width + x1];

                let lerp = |a: u8, b: u8, t: f32| -> u8 {
                    ((a as f32) * (1.0 - t) + (b as f32) * t).round() as u8
                };

                let a = lerp(lerp((c00 >> 24) as u8, (c10 >> 24) as u8, fx),
                             lerp((c01 >> 24) as u8, (c11 >> 24) as u8, fx), fy);
                let r = lerp(lerp((c00 >> 16 & 0xFF) as u8, (c10 >> 16 & 0xFF) as u8, fx),
                             lerp((c01 >> 16 & 0xFF) as u8, (c11 >> 16 & 0xFF) as u8, fx), fy);
                let g = lerp(lerp((c00 >> 8 & 0xFF) as u8, (c10 >> 8 & 0xFF) as u8, fx),
                             lerp((c01 >> 8 & 0xFF) as u8, (c11 >> 8 & 0xFF) as u8, fx), fy);
                let b = lerp(lerp((c00 & 0xFF) as u8, (c10 & 0xFF) as u8, fx),
                             lerp((c01 & 0xFF) as u8, (c11 & 0xFF) as u8, fx), fy);

                scaled_buffer[y * win_w + x] = ((a as u32) << 24) | ((r as u32) << 16) | ((g as u32) << 8) | b as u32;
            }
        }

        window.update_with_buffer(&scaled_buffer, win_w, win_h).unwrap();
    }
}
