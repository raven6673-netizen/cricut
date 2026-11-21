//! Pictures: loading photos and drawings, removing the background, and turning them into outlines.

use crate::geometry::{self, Contour};

/// A loaded picture, scaled down so tracing stays quick.
#[derive(Clone)]
pub struct Picture {
    pub w: usize,
    pub h: usize,
    pub rgba: Vec<u8>,
}

/// How to separate the design from its background.
#[derive(Clone, PartialEq)]
pub struct Cleanup {
    /// Pixels darker than this (0-255) are kept, when not using the background colour.
    pub threshold: u8,
    /// Remove pixels close to the colour in the corners instead of using brightness.
    pub remove_background: bool,
    /// How close (0-255) a pixel must be to the background colour to be removed.
    pub tolerance: u8,
    pub invert: bool,
    /// Outline smoothing, in pixels.
    pub smooth: f32,
}

impl Default for Cleanup {
    fn default() -> Self {
        Self { threshold: 160, remove_background: true, tolerance: 60, invert: false, smooth: 1.2 }
    }
}

pub fn load(bytes: &[u8]) -> Result<Picture, String> {
    let img = image::load_from_memory(bytes).map_err(|e| format!("Couldn't read that picture: {e}"))?;
    // enough detail to check edges up close in the upload window, still quick to trace
    let max = 1400u32;
    let img = if img.width() > max || img.height() > max { img.thumbnail(max, max) } else { img };
    let rgba = img.to_rgba8();
    Ok(Picture { w: rgba.width() as usize, h: rgba.height() as usize, rgba: rgba.into_raw() })
}

fn background_colour(p: &Picture) -> [u8; 3] {
    let corners = [(0, 0), (p.w - 1, 0), (0, p.h - 1), (p.w - 1, p.h - 1)];
    let mut acc = [0u32; 3];
    for (x, y) in corners {
        let i = (y * p.w + x) * 4;
        for c in 0..3 {
            acc[c] += p.rgba[i + c] as u32;
        }
    }
    [(acc[0] / 4) as u8, (acc[1] / 4) as u8, (acc[2] / 4) as u8]
}

/// Which pixels belong to the design.
pub fn mask(p: &Picture, c: &Cleanup) -> Vec<bool> {
    let bg = background_colour(p);
    let mut m = vec![false; p.w * p.h];
    for (i, px) in p.rgba.chunks(4).enumerate() {
        let keep = if px[3] < 40 {
            false
        } else if c.remove_background {
            let d = (px[0] as i32 - bg[0] as i32).abs().max((px[1] as i32 - bg[1] as i32).abs()).max((px[2] as i32 - bg[2] as i32).abs());
            d > c.tolerance as i32
        } else {
            let lum = (px[0] as u32 * 299 + px[1] as u32 * 587 + px[2] as u32 * 114) / 1000;
            lum < c.threshold as u32
        };
        m[i] = keep != c.invert;
    }
    m
}

/// The picture with removed parts made see-through (for print then cut).
pub fn cleaned_rgba(p: &Picture, m: &[bool]) -> Vec<u8> {
    let mut out = p.rgba.clone();
    for (i, keep) in m.iter().enumerate() {
        if !keep {
            out[i * 4 + 3] = 0;
        }
    }
    out
}

/// Outlines of the kept parts, in pixel units.
pub fn trace(p: &Picture, m: &[bool], smooth: f32) -> Vec<Contour> {
    geometry::trace_mask(m, p.w, p.h, smooth.max(0.3))
}

/// An outline around the whole picture's kept parts, a little outside them (the print-then-cut line).
pub fn print_outline(p: &Picture, m: &[bool]) -> Vec<Contour> {
    let traced = trace(p, m, 1.5);
    let grown = geometry::offset_contours(&traced, 2.0);
    // keep only outer edges: no cutting inside a printed picture
    let mut outer: Vec<Contour> = Vec::new();
    for c in grown {
        if area(&c) > 0.0 {
            outer.push(c);
        }
    }
    if outer.is_empty() {
        vec![vec![
            egui::pos2(0.0, 0.0),
            egui::pos2(p.w as f32, 0.0),
            egui::pos2(p.w as f32, p.h as f32),
            egui::pos2(0.0, p.h as f32),
            egui::pos2(0.0, 0.0),
        ]]
    } else {
        outer
    }
}

/// Signed area (positive for the outer rings that geo writes counter-clockwise in y-up terms).
fn area(c: &Contour) -> f32 {
    let mut a = 0.0;
    for w in c.windows(2) {
        a += w[0].x * w[1].y - w[1].x * w[0].y;
    }
    -a / 2.0
}

// ---------------------------------------------------------------- SVG converter

/// Write outlines (in inches) as a cut-ready SVG in millimetres: one path, even-odd fill so holes
/// stay holes. With a picture, it's placed underneath for print then cut.
pub fn svg_document(contours: &[Contour], color: [u8; 3], picture: Option<(&str, [f32; 4])>) -> String {
    let (min, max) = geometry::bounds(contours.iter().flatten().copied());
    let (min, max) = match picture {
        Some((_, r)) => (min.min(egui::pos2(r[0], r[1])), max.max(egui::pos2(r[2], r[3]))),
        None => (min, max),
    };
    let size = (max - min).max(egui::vec2(0.01, 0.01));
    let mm = 25.4;
    let mut s = format!(
        "<?xml version=\"1.0\" encoding=\"UTF-8\"?>\n<svg xmlns=\"http://www.w3.org/2000/svg\" width=\"{w:.3}mm\" height=\"{h:.3}mm\" viewBox=\"0 0 {w:.3} {h:.3}\">\n",
        w = size.x * mm,
        h = size.y * mm
    );
    if let Some((png, r)) = picture {
        s.push_str(&format!(
            "  <image x=\"{:.3}\" y=\"{:.3}\" width=\"{:.3}\" height=\"{:.3}\" href=\"data:image/png;base64,{}\"/>\n",
            (r[0] - min.x) * mm,
            (r[1] - min.y) * mm,
            (r[2] - r[0]) * mm,
            (r[3] - r[1]) * mm,
            png
        ));
    }
    let shifted: Vec<Contour> = contours.iter().map(|c| c.iter().map(|p| egui::pos2(p.x - min.x, p.y - min.y)).collect()).collect();
    let d: Vec<String> = shifted.iter().filter(|c| c.len() > 1).map(|c| geometry::contour_to_d(c, mm)).collect();
    let fill = if picture.is_some() { "none".to_string() } else { format!("#{:02x}{:02x}{:02x}", color[0], color[1], color[2]) };
    s.push_str(&format!(
        "  <path fill=\"{}\" fill-rule=\"evenodd\" stroke=\"#{:02x}{:02x}{:02x}\" stroke-width=\"0.1\" d=\"{}\"/>\n</svg>\n",
        fill,
        color[0],
        color[1],
        color[2],
        d.join(" ")
    ));
    s
}

/// The average colour of the kept pixels.
pub fn kept_colour(p: &Picture, m: &[bool]) -> [u8; 3] {
    let (mut r, mut g, mut b, mut n) = (0u64, 0u64, 0u64, 0u64);
    for (i, k) in m.iter().enumerate() {
        if *k {
            r += p.rgba[i * 4] as u64;
            g += p.rgba[i * 4 + 1] as u64;
            b += p.rgba[i * 4 + 2] as u64;
            n += 1;
        }
    }
    if n > 0 {
        [(r / n) as u8, (g / n) as u8, (b / n) as u8]
    } else {
        [0x33, 0x3a, 0x44]
    }
}

/// Turn a picture into SVG text at the given width in inches: traced outlines (cut image), or the
/// cleaned picture with a cut line around it (print then cut).
pub fn picture_to_svg(p: &Picture, c: &Cleanup, width_in: f32, print: bool) -> Result<String, String> {
    let m = mask(p, c);
    let scale = width_in / p.w.max(1) as f32;
    if print {
        let outline = print_outline(p, &m);
        let world: Vec<Contour> = outline.iter().map(|c| c.iter().map(|q| egui::pos2(q.x * scale, q.y * scale)).collect()).collect();
        let png = b64_encode(&encode_png(p.w, p.h, &cleaned_rgba(p, &m)));
        Ok(svg_document(&world, [0x30, 0x30, 0x30], Some((&png, [0.0, 0.0, p.w as f32 * scale, p.h as f32 * scale]))))
    } else {
        let traced = trace(p, &m, c.smooth);
        if traced.is_empty() {
            return Err("Nothing was left to cut with these settings.".into());
        }
        let world: Vec<Contour> = traced.iter().map(|c| c.iter().map(|q| egui::pos2(q.x * scale, q.y * scale)).collect()).collect();
        Ok(svg_document(&world, kept_colour(p, &m), None))
    }
}

/// Convert any supported file (photo, drawing or DXF) to SVG text with the default settings.
pub fn file_to_svg(path: &std::path::Path) -> Result<String, String> {
    let bytes = std::fs::read(path).map_err(|e| format!("couldn't read it ({e})"))?;
    let ext = path.extension().map(|e| e.to_string_lossy().to_lowercase()).unwrap_or_default();
    match ext.as_str() {
        "svg" => Err("it's already an SVG".into()),
        "dxf" => {
            let c = geometry::parse_dxf(&String::from_utf8_lossy(&bytes));
            if c.is_empty() {
                return Err("no shapes found in it".into());
            }
            // DXF drawings are usually in millimetres
            let inch: Vec<Contour> = c.iter().map(|c| c.iter().map(|p| egui::pos2(p.x / 25.4, p.y / 25.4)).collect()).collect();
            Ok(svg_document(&inch, [0x33, 0x3a, 0x44], None))
        }
        _ => {
            let p = load(&bytes)?;
            picture_to_svg(&p, &Cleanup::default(), 4.0, false)
        }
    }
}

/// Photo engraving: rows of short lines, more of them where the picture is darker (ordered
/// dithering). `cell` is the row spacing in picture pixels; `brightness` (-1..1) lightens or
/// darkens. Transparent parts are never engraved. Lines are in picture pixels.
pub fn photo_engrave_lines(rgba: &[u8], w: usize, h: usize, cell: f32, brightness: f32, invert: bool) -> Vec<Contour> {
    const BAYER: [[f32; 4]; 4] = [[0., 8., 2., 10.], [12., 4., 14., 6.], [3., 11., 1., 9.], [15., 7., 13., 5.]];
    let cell = cell.max(0.5);
    let cols = ((w as f32) / cell).floor().max(1.0) as usize;
    let rows = ((h as f32) / cell).floor().max(1.0) as usize;
    let mut out = Vec::new();
    for r in 0..rows {
        let y = (r as f32 + 0.5) * cell;
        let mut start: Option<usize> = None;
        let mut runs: Vec<(usize, usize)> = Vec::new();
        for c in 0..=cols {
            let dark = if c == cols {
                false
            } else {
                // average the cell
                let (x0, y0) = ((c as f32 * cell) as usize, (r as f32 * cell) as usize);
                let x1 = ((((c + 1) as f32) * cell) as usize).min(w).max(x0 + 1);
                let y1 = ((((r + 1) as f32) * cell) as usize).min(h).max(y0 + 1);
                let (mut sum, mut n, mut alpha) = (0.0f32, 0.0f32, 0.0f32);
                for yy in y0..y1.min(h) {
                    for xx in x0..x1.min(w) {
                        let i = (yy * w + xx) * 4;
                        let a = rgba[i + 3] as f32 / 255.0;
                        let lum = (rgba[i] as f32 * 0.299 + rgba[i + 1] as f32 * 0.587 + rgba[i + 2] as f32 * 0.114) / 255.0;
                        // transparent counts as white
                        sum += lum * a + (1.0 - a);
                        alpha += a;
                        n += 1.0;
                    }
                }
                if n == 0.0 || alpha / n < 0.1 {
                    false
                } else {
                    let mut darkness = 1.0 - sum / n;
                    if invert {
                        darkness = 1.0 - darkness;
                    }
                    darkness = (darkness - brightness * 0.5).clamp(0.0, 1.0);
                    let threshold = (BAYER[r % 4][c % 4] + 0.5) / 16.0;
                    darkness > threshold
                }
            };
            match (dark, start) {
                (true, None) => start = Some(c),
                (false, Some(s)) => {
                    runs.push((s, c));
                    start = None;
                }
                _ => {}
            }
        }
        // alternate direction row by row so the tool doesn't travel back each time
        if r % 2 == 1 {
            runs.reverse();
        }
        for (s, e) in runs {
            let (a, b) = (egui::pos2(s as f32 * cell, y), egui::pos2(e as f32 * cell, y));
            out.push(if r % 2 == 1 { vec![b, a] } else { vec![a, b] });
        }
    }
    out
}

pub fn encode_png(w: usize, h: usize, rgba: &[u8]) -> Vec<u8> {
    let mut out = Vec::new();
    if let Some(img) = image::RgbaImage::from_raw(w as u32, h as u32, rgba.to_vec()) {
        let _ = img.write_to(&mut std::io::Cursor::new(&mut out), image::ImageFormat::Png);
    }
    out
}

const B64: &[u8; 64] = b"ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789+/";

pub fn b64_encode(data: &[u8]) -> String {
    let mut s = String::with_capacity(data.len() * 4 / 3 + 4);
    for chunk in data.chunks(3) {
        let b = [chunk[0], *chunk.get(1).unwrap_or(&0), *chunk.get(2).unwrap_or(&0)];
        let n = (b[0] as u32) << 16 | (b[1] as u32) << 8 | b[2] as u32;
        s.push(B64[(n >> 18) as usize & 63] as char);
        s.push(B64[(n >> 12) as usize & 63] as char);
        s.push(if chunk.len() > 1 { B64[(n >> 6) as usize & 63] as char } else { '=' });
        s.push(if chunk.len() > 2 { B64[n as usize & 63] as char } else { '=' });
    }
    s
}

pub fn b64_decode(s: &str) -> Vec<u8> {
    let mut out = Vec::with_capacity(s.len() * 3 / 4);
    let mut buf = 0u32;
    let mut bits = 0;
    for ch in s.bytes() {
        let v = match ch {
            b'A'..=b'Z' => ch - b'A',
            b'a'..=b'z' => ch - b'a' + 26,
            b'0'..=b'9' => ch - b'0' + 52,
            b'+' => 62,
            b'/' => 63,
            _ => continue,
        } as u32;
        buf = buf << 6 | v;
        bits += 6;
        if bits >= 8 {
            bits -= 8;
            out.push((buf >> bits) as u8);
        }
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn converter_turns_a_png_file_into_svg() {
        // a white picture with a dark ring (a shape with a hole)
        let (w, h) = (60u32, 60u32);
        let img = image::RgbaImage::from_fn(w, h, |x, y| {
            let d = ((x as f32 - 30.0).powi(2) + (y as f32 - 30.0).powi(2)).sqrt();
            if (12.0..24.0).contains(&d) { image::Rgba([20, 20, 120, 255]) } else { image::Rgba([255, 255, 255, 255]) }
        });
        let dir = std::env::temp_dir().join(format!("matcraft-test-{}", std::process::id()));
        std::fs::create_dir_all(&dir).unwrap();
        let png = dir.join("ring.png");
        img.save(&png).unwrap();
        let svg = file_to_svg(&png).unwrap();
        let back = geometry::parse_svg(&svg);
        assert_eq!(back.len(), 2, "outer edge and the hole");
        // it says its real size: 4 inches wide by default
        let k = geometry::svg_inches_per_unit(&svg).unwrap();
        let (a, b) = geometry::bounds(back.iter().flatten().copied());
        assert!(((b.x - a.x) * k - 4.0 * 48.0 / 60.0).abs() < 0.3);
        assert!(file_to_svg(&dir.join("missing.png")).is_err());
        let _ = std::fs::remove_dir_all(&dir);
    }

    #[test]
    fn photo_engraving_follows_darkness() {
        // left half black, right half white, 40 x 40
        let (w, h) = (40usize, 40usize);
        let mut rgba = vec![255u8; w * h * 4];
        for y in 0..h {
            for x in 0..w / 2 {
                let i = (y * w + x) * 4;
                rgba[i] = 0;
                rgba[i + 1] = 0;
                rgba[i + 2] = 0;
            }
        }
        let lines = photo_engrave_lines(&rgba, w, h, 2.0, 0.0, false);
        assert!(!lines.is_empty());
        // every line stays in the black half
        for l in &lines {
            assert!(l[0].x <= 20.0 + 0.01 && l[1].x <= 20.0 + 0.01);
        }
        // swapping light and dark moves them to the other half
        let inv = photo_engrave_lines(&rgba, w, h, 2.0, 0.0, true);
        assert!(inv.iter().all(|l| l[0].x >= 19.99 && l[1].x >= 19.99));
    }

    #[test]
    fn base64_roundtrip() {
        let d = b"print then cut!";
        assert_eq!(b64_decode(&b64_encode(d)), d.to_vec());
    }
    #[test]
    fn mask_removes_white_background() {
        let mut rgba = vec![255u8; 10 * 10 * 4];
        for y in 3..7 {
            for x in 3..7 {
                let i = (y * 10 + x) * 4;
                rgba[i] = 0;
                rgba[i + 1] = 0;
                rgba[i + 2] = 0;
            }
        }
        let p = Picture { w: 10, h: 10, rgba };
        let m = mask(&p, &Cleanup::default());
        assert_eq!(m.iter().filter(|k| **k).count(), 16);
        assert_eq!(trace(&p, &m, 0.5).len(), 1);
        // the SVG converter turns it into a path whose parser reads it back
        let svg = picture_to_svg(&p, &Cleanup { smooth: 0.5, ..Cleanup::default() }, 2.0, false).unwrap();
        assert!(svg.contains("<path") && svg.contains("evenodd"));
        assert_eq!(geometry::parse_svg(&svg).len(), 1);
        let printed = picture_to_svg(&p, &Cleanup::default(), 2.0, true).unwrap();
        assert!(printed.contains("<image"));
    }
}
