use minifb::{Key, Window, WindowOptions};
use regex::Regex;
use std::fs;

/// Parse SITF color tokens (reuse your existing parser)
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

/// Load SITF file into a pixel buffer for display
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
            let pixel = ((alpha as u32) << 24) | ((r as u32) << 16) | ((g as u32) << 8) | b as u32;
            pixel_vec.push((x - 1, y - 1, pixel));
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

/// Public function to view a SITF image
pub fn view_sitf(file: &str) {
    let (buffer, width, height) = load_sitf(file);

    let mut window = Window::new(
        "SITF Viewer",
        width,
        height,
        WindowOptions::default(),
    ).unwrap();

    while window.is_open() && !window.is_key_down(Key::Escape) {
        window.update_with_buffer(&buffer, width, height).unwrap();
    }
}

