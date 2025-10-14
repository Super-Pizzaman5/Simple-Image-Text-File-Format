# SITF (Structured Image Text Format)

SITF is a compact and human-readable text format for storing image pixel data using structured syntax rules.

---

## 📘 Format Overview

Each SITF file consists of **metadata** and **pixel data**.

### 1. Metadata Section

- Starts with a **`$`** symbol.
- Any text after `$` is treated as metadata (e.g., author, creation date, description).
- Ends when an **`@`** symbol appears, which signals the start of pixel data.

Example:
```
$Author:Name, Date:2025-10-14@
```

---

### 2. Pixel Data Section

Pixel data is written as a sequence of pixel definitions separated by commas (`,`).
Each line (row of pixels) ends with a semicolon (`;`).

#### Basic Pixel Format
```
X:Y+VALUE
```

- `X` = horizontal coordiName (starting from 1)
- `Y` = vertical coordiName (starting from 1)
- `+` indicates full opacity
- `-N` indicates partial transparency (where `N` is the transparency percentage, e.g., `-50` = 50% transparent)
- `VALUE` defines the color or light value

#### Color Codes

| Symbol | Meaning | Example |
|:--|:--|:--|
| `#RRGGBB` | Standard hex color | `#FF0000` (red) |
| `!F` | Pure white | `!F` |
| `!0` | Pure black | `!0` |
| `!R` | Red | `!R` |
| `!G` | Green | `!G` |
| `!B` | Blue | `!B` |
| `%A/B` | Grayscale value (A ÷ B) | `%1/2` (medium gray) |

---

### 3. Range Compression

If multiple consecutive pixels on the **same row** share identical values, use a hyphen (`-`) to compress the range.

Example:
```
88-100:100+!F
```
→ Pixels 88 through 100 on line 100 are all pure white.

---

### 4. Line Separation

Each line (representing a row of pixels) ends with a semicolon (`;`).

Example:
```
1:1+!F,2:1+!R,3:1+!G;1:2+!B,2:2+%1/2;
```

---

### ✅ Example SITF File

```
$Author:Name,Date:2025-10-14,Description:Sample Rainbow Image@
1:1+!R,2:1+!G,3:1+!B;
1:2+%1/2,2:2+%3/4,3:2+%1/1;
88-100:100+!F;
```

---

## 🧩 Notes

- All coordiNames start at **1,1** (top-left).
- Files should end with a semicolon to mark the final line.
- Designed for compactness and human readability.
- Intended for pixel-perfect image reconstruction.

---

## ⚙️ Example Converter Usage

This repository includes a Rust program (`sitf_converter.rs`) that can convert `.png` images into `.sitf` files.

Run it using:
```bash
cargo run -- input.png output.sitf
```
