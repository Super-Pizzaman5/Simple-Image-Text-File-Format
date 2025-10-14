use image::{GenericImageView, Rgba, RgbaImage};
use rayon::prelude::*;
use regex::Regex;
use std::env;
use std::fs::{self, File};
use std::io::Write;

/// Parse a color token (hex, shorthand, or grayscale %a/b)
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

/// Try to represent an RGB triple as a grayscale token if possible.
/// Returns Some("%num/den") or None if not grayscale.
fn rgb_to_grayscale_token(r: u8, g: u8, b: u8) -> Option<String> {
    if r == g && g == b && !(r == 0 || r == 255) {
        // Represent with denominator 1000 for decent precision
        let denom = 1000u32;
        let num = ((r as f32 / 255.0) * denom as f32).round() as u32;
        Some(format!("%{}/{}", num, denom))
    } else {
        None
    }
}

/// Convert PNG -> SITF (pixel-perfect)
fn png_to_sitf(input: &str, output: &str, metadata: &str) -> Result<(), Box<dyn std::error::Error>> {
    let img = image::open(input)?.to_rgba8();
    let (width, height) = img.dimensions();

    // Build rows in parallel
    let rows: Vec<String> = (0..height)
        .into_par_iter()
        .map(|y| {
            let mut parts = Vec::new();
            let mut x = 0u32;
            while x < width {
                let [r, g, b, a] = img.get_pixel(x, y).0;

                // transparency token
                let trans = if a == 255 {
                    "+".to_string()
                } else {
                    let pct = (100.0 * (1.0 - (a as f32 / 255.0))).round() as i32;
                    format!("-{}", pct)
                };

                // color token (try grayscale first)
                let color_token = if r == g && g == b {
                    if r == 255 {
                        "!F".to_string()
                    } else if r == 0 {
                        "!0".to_string()
                    } else {
                        // fraction representation
                        rgb_to_grayscale_token(r, g, b).unwrap_or_else(|| format!("%{}/{}", r, 255))
                    }
                } else {
                    match (r, g, b) {
                        (255, 0, 0) => "!R".to_string(),
                        (0, 255, 0) => "!G".to_string(),
                        (0, 0, 255) => "!B".to_string(),
                        _ => format!("#{:02X}{:02X}{:02X}", r, g, b),
                    }
                };

                // detect horizontal identical-run (range compression)
                let mut run_end = x;
                while run_end + 1 < width {
                    let next = img.get_pixel(run_end + 1, y);
                    if next.0 == [r, g, b, a] {
                        run_end += 1;
                    } else {
                        break;
                    }
                }

                if run_end > x {
                    parts.push(format!("{}-{}:{}{}{}", x + 1, run_end + 1, y + 1, trans, color_token));
                    x = run_end + 1;
                } else {
                    parts.push(format!("{}:{}{}{}", x + 1, y + 1, trans, color_token));
                    x += 1;
                }
            }
            parts.join(",")
        })
        .collect();

    let mut out = File::create(output)?;
    // write metadata section ($...@). If metadata is empty, write an empty metadata header.
    writeln!(out, "${}@", metadata)?;
for row in rows {
    writeln!(out, "{}", row)?;
}
    println!("Converted PNG -> SITF ({}x{}) => {}", width, height, output);
    Ok(())
}

/// Convert SITF -> PNG (pixel-perfect, 1:1)
fn sitf_to_png(input: &str, output: &str) -> Result<(), Box<dyn std::error::Error>> {
    let data = fs::read_to_string(input)?;
    // Split metadata and pixel data; if no '@' treat whole file as pixel data
    let pixels_str = data.split_once('@').map(|(_, p)| p).unwrap_or(&data);

    // regex covers either a range (x1-x2) or single x, then :y, then transparency, then color token
   let re = Regex::new(
    r"(?:(\d+)-(\d+)|(\d+)):(\d+)([+\-]\d*|[+])(!F|!0|!R|!G|!B|#[0-9A-Fa-f]{6}|%[0-9]+/[0-9]+)"
).unwrap();



    // collect pixels in parallel (flatten ranges)
    let mut pixels: Vec<(u32, u32, Rgba<u8>)> = re
        .captures_iter(pixels_str)
        .par_bridge()
        .flat_map(|cap| {
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

            (x_start..=x_end).map(move |x| (x, y, Rgba([r, g, b, alpha]))).collect::<Vec<_>>()
        })
        .collect();

    // Determine image dimensions
    let max_x = pixels.iter().map(|(x, _, _)| *x).max().unwrap_or(1);
    let max_y = pixels.iter().map(|(_, y, _)| *y).max().unwrap_or(1);

    // Create image and write pixels (pixel-perfect: no scaling)
    let mut img = RgbaImage::new(max_x, max_y);
    for (x, y, px) in pixels.drain(..) {
        img.put_pixel(x - 1, y - 1, px);
    }

    img.save(output)?;
    println!("Converted SITF -> PNG ({}x{}) => {}", max_x, max_y, output);
    Ok(())
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let args: Vec<String> = env::args().collect();
    if args.len() < 4 {
        eprintln!("Usage:\n  {} to-sitf <input.png> <output.sitf> [metadata]\n  {} to-png <input.sitf> <output.png>", args[0], args[0]);
        std::process::exit(1);
    }

    match args[1].as_str() {
        "to-sitf" => {
            let metadata = if args.len() > 4 { &args[4] } else { "" };
            png_to_sitf(&args[2], &args[3], metadata)?
        }
        "to-png" => sitf_to_png(&args[2], &args[3])?,
        _ => {
            eprintln!("Unknown mode: {}. Use 'to-sitf' or 'to-png'.", args[1]);
            std::process::exit(1);
        }
    }

    Ok(())
}
