use std::fs::File;
use std::io::{BufRead, BufReader};
use image::{RgbaImage, Rgba};

#[derive(Clone)]
struct Pixel {
    red: u8,
    green: u8,
    blue: u8,
    alpha: f32, // 0.0 - 1.0
}

fn parse_sitf(path: &str) -> Vec<Vec<Pixel>> {
    let file = File::open(path).expect("Cannot open SITF file");
    let reader = BufReader::new(file);

    let mut pixels: Vec<Vec<Pixel>> = Vec::new();
    let mut reading_pixels = false;

    for line in reader.lines() {
        let line = line.unwrap().trim().to_string();

        if line.starts_with('$') && !reading_pixels {
            continue; // metadata
        }

        if line.starts_with('@') {
            reading_pixels = true;
            continue;
        }

        if reading_pixels {
            let row = parse_pixel_line(&line);
            pixels.push(row);
        }
    }

    pixels
}

fn parse_pixel_line(line: &str) -> Vec<Pixel> {
    let mut row = Vec::new();
    let entries: Vec<&str> = line.split(',').collect();

    for entry in entries {
        let entry = entry.trim();
        if entry.is_empty() { continue; }

        let parts: Vec<&str> = entry.split(':').collect();
        if parts.len() != 2 { continue; }

        let _x_part = parts[0];
        let rest = parts[1];

        let (alpha, color_token) = if rest.starts_with('+') {
            (1.0, &rest[1..])
        } else if rest.starts_with('-') {
            let mut split = rest[1..].split(|c| c == '!' || c == '#' || c == '%');
            let perc_str = split.next().unwrap_or("100");
            let alpha = perc_str.parse::<f32>().unwrap_or(100.0)/100.0;
            let color_token = &rest[1 + perc_str.len()..];
            (alpha, color_token)
        } else {
            (1.0, rest)
        };

        let pixel = match color_token {
            "!F" => Pixel { red:255, green:255, blue:255, alpha },
            "!0" => Pixel { red:0, green:0, blue:0, alpha },
            "!R" => Pixel { red:255, green:0, blue:0, alpha },
            "!G" => Pixel { red:0, green:255, blue:0, alpha },
            "!B" => Pixel { red:0, green:0, blue:255, alpha },
            _ if color_token.starts_with('#') && color_token.len()==7 => {
                let r = u8::from_str_radix(&color_token[1..3],16).unwrap_or(0);
                let g = u8::from_str_radix(&color_token[3..5],16).unwrap_or(0);
                let b = u8::from_str_radix(&color_token[5..7],16).unwrap_or(0);
                Pixel{red:r,green:g,blue:b,alpha}
            },
            _ if color_token.starts_with('%') => {
                let frac = &color_token[1..];
                let parts: Vec<&str> = frac.split('/').collect();
                if parts.len()==2 {
                    let num = parts[0].parse::<f32>().unwrap_or(0.0);
                    let denom = parts[1].parse::<f32>().unwrap_or(1.0);
                    let val = ((num/denom)*255.0).round() as u8;
                    Pixel{red:val,green:val,blue:val,alpha}
                } else {
                    Pixel{red:0,green:0,blue:0,alpha}
                }
            },
            _ => Pixel{red:0,green:0,blue:0,alpha},
        };

        // For now, just push one pixel per entry (ignore ranges)
        row.push(pixel);
    }

    row
}

fn main() {
    let args: Vec<String> = std::env::args().collect();
    if args.len()<3 {
        eprintln!("Usage: {} <input.sitf> <output.png>", args[0]);
        return;
    }

    let sitf_file = &args[1];
    let output_file = &args[2];

    let pixels = parse_sitf(sitf_file);

    if pixels.is_empty() {
        eprintln!("SITF file is empty or could not be parsed.");
        return;
    }

    let height = pixels.len() as u32;
    let width = pixels[0].len() as u32;

    let mut img = RgbaImage::new(width, height);

    for (y, row) in pixels.iter().enumerate() {
        for (x, px) in row.iter().enumerate() {
            img.put_pixel(
                x as u32,
                y as u32,
                Rgba([px.red, px.green, px.blue, (px.alpha*255.0).round() as u8])
            );
        }
    }

    img.save(output_file).expect("Failed to save PNG");
    println!("Saved PNG to {}", output_file);
}
