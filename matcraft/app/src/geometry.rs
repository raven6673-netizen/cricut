//! Shapes, outlines and simple SVG reading/writing. All sizes are in inches.

use egui::{pos2, Pos2, Vec2};

/// One closed or open outline made of straight segments (curves are flattened).
pub type Contour = Vec<Pos2>;

/// What the machine does with a layer.
#[derive(Clone, Copy, PartialEq, Eq, Debug, serde::Serialize, serde::Deserialize)]
pub enum Operation {
    Cut,
    Draw,
    Score,
    Engrave,
    Deboss,
    Wave,
    Perforate,
    /// Print then cut: the design is printed with its fill, then its outline is cut.
    Print,
    /// Shown on the canvas only; never sent to the machine.
    Guide,
}

impl Operation {
    pub const ALL: [Operation; 9] = [
        Operation::Cut,
        Operation::Draw,
        Operation::Score,
        Operation::Engrave,
        Operation::Deboss,
        Operation::Wave,
        Operation::Perforate,
        Operation::Print,
        Operation::Guide,
    ];
    pub fn label(self) -> &'static str {
        match self {
            Operation::Cut => "Cut",
            Operation::Draw => "Draw",
            Operation::Score => "Score",
            Operation::Engrave => "Engrave",
            Operation::Deboss => "Deboss",
            Operation::Wave => "Wave",
            Operation::Perforate => "Perforate",
            Operation::Print => "Print then cut",
            Operation::Guide => "Guide",
        }
    }
    /// Operations drawn as filled shapes on the canvas.
    pub fn filled(self) -> bool {
        matches!(self, Operation::Cut | Operation::Print | Operation::Wave | Operation::Perforate)
    }
}

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum ShapeKind {
    Square,
    Circle,
    Triangle,
    Star,
    Heart,
    Hexagon,
    Pentagon,
    Octagon,
    Diamond,
    Rectangle,
    RoundedSquare,
    Oval,
    Plus,
    HalfCircle,
    Shield,
    Line,
}

impl ShapeKind {
    pub const ALL: [ShapeKind; 16] = [
        ShapeKind::Square,
        ShapeKind::Circle,
        ShapeKind::Triangle,
        ShapeKind::Star,
        ShapeKind::Heart,
        ShapeKind::Hexagon,
        ShapeKind::Pentagon,
        ShapeKind::Octagon,
        ShapeKind::Diamond,
        ShapeKind::Rectangle,
        ShapeKind::RoundedSquare,
        ShapeKind::Oval,
        ShapeKind::Plus,
        ShapeKind::HalfCircle,
        ShapeKind::Shield,
        ShapeKind::Line,
    ];

    /// The size a new shape starts at, in inches.
    pub fn default_size(self) -> Vec2 {
        match self {
            ShapeKind::Rectangle | ShapeKind::Oval => egui::vec2(3.0, 1.5),
            ShapeKind::HalfCircle => egui::vec2(1.0, 2.0),
            ShapeKind::Shield => egui::vec2(1.6, 2.0),
            ShapeKind::Line => egui::vec2(3.0, 0.0001),
            _ => egui::vec2(2.0, 2.0),
        }
    }
    pub fn label(self) -> &'static str {
        match self {
            ShapeKind::Square => "Square",
            ShapeKind::Circle => "Circle",
            ShapeKind::Triangle => "Triangle",
            ShapeKind::Star => "Star",
            ShapeKind::Heart => "Heart",
            ShapeKind::Hexagon => "Hexagon",
            ShapeKind::Pentagon => "Pentagon",
            ShapeKind::Octagon => "Octagon",
            ShapeKind::Diamond => "Diamond",
            ShapeKind::Rectangle => "Rectangle",
            ShapeKind::RoundedSquare => "Rounded square",
            ShapeKind::Oval => "Oval",
            ShapeKind::Plus => "Plus",
            ShapeKind::HalfCircle => "Half circle",
            ShapeKind::Shield => "Shield",
            ShapeKind::Line => "Score line",
        }
    }

    /// Outline in a unit box (0..1 on both axes, y down).
    pub fn unit_contours(self) -> Vec<Contour> {
        match self {
            ShapeKind::Square => vec![vec![pos2(0., 0.), pos2(1., 0.), pos2(1., 1.), pos2(0., 1.), pos2(0., 0.)]],
            ShapeKind::Circle => vec![regular(64, 0.0)],
            ShapeKind::Triangle => vec![vec![pos2(0.5, 0.), pos2(1., 1.), pos2(0., 1.), pos2(0.5, 0.)]],
            ShapeKind::Hexagon => vec![regular(6, 0.0)],
            ShapeKind::Pentagon => vec![regular(5, 0.0)],
            ShapeKind::Octagon => vec![regular(8, std::f32::consts::PI / 8.0)],
            ShapeKind::Diamond => vec![vec![pos2(0.5, 0.), pos2(1., 0.5), pos2(0.5, 1.), pos2(0., 0.5), pos2(0.5, 0.)]],
            ShapeKind::Rectangle => vec![vec![pos2(0., 0.), pos2(1., 0.), pos2(1., 1.), pos2(0., 1.), pos2(0., 0.)]],
            ShapeKind::Oval => vec![regular(64, 0.0)],
            ShapeKind::RoundedSquare => {
                // corners of radius 0.15
                let r = 0.15;
                let mut c = Vec::new();
                for (cx, cy, a0) in [(1.0 - r, r, -90.0f32), (1.0 - r, 1.0 - r, 0.0), (r, 1.0 - r, 90.0), (r, r, 180.0)] {
                    for k in 0..=8 {
                        let a = (a0 + k as f32 * 90.0 / 8.0).to_radians();
                        c.push(pos2(cx + r * a.cos(), cy + r * a.sin()));
                    }
                }
                c.push(c[0]);
                vec![c]
            }
            ShapeKind::Plus => {
                let (a, b) = (0.34, 0.66);
                vec![vec![
                    pos2(a, 0.), pos2(b, 0.), pos2(b, a), pos2(1., a), pos2(1., b), pos2(b, b), pos2(b, 1.), pos2(a, 1.), pos2(a, b), pos2(0., b),
                    pos2(0., a), pos2(a, a), pos2(a, 0.),
                ]]
            }
            ShapeKind::HalfCircle => {
                // the right half of a circle, flat side on the left
                let mut c: Contour = (0..=32)
                    .map(|k| {
                        let a = (-90.0 + k as f32 * 180.0 / 32.0f32).to_radians();
                        pos2(a.cos(), 0.5 + 0.5 * a.sin())
                    })
                    .collect();
                c.push(c[0]);
                vec![c]
            }
            ShapeKind::Shield => {
                // flat top, straight sides, rounded bottom
                let mut c = vec![pos2(0., 0.), pos2(1., 0.), pos2(1., 0.55)];
                for k in 1..=16 {
                    let a = (k as f32 * 180.0 / 16.0f32).to_radians();
                    c.push(pos2(0.5 + 0.5 * a.cos(), 0.55 + 0.45 * a.sin()));
                }
                c.push(pos2(0., 0.));
                vec![c]
            }
            ShapeKind::Star => {
                let mut c = Vec::new();
                for i in 0..=10 {
                    let a = -std::f32::consts::FRAC_PI_2 + i as f32 * std::f32::consts::PI / 5.0;
                    let r = if i % 2 == 0 { 0.5 } else { 0.2 };
                    c.push(pos2(0.5 + r * a.cos(), 0.52 + r * a.sin()));
                }
                vec![c]
            }
            ShapeKind::Heart => {
                let mut c = Vec::new();
                for i in 0..=96 {
                    let t = i as f32 / 96.0 * std::f32::consts::TAU;
                    let x = 16.0 * t.sin().powi(3);
                    let y = 13.0 * t.cos() - 5.0 * (2.0 * t).cos() - 2.0 * (3.0 * t).cos() - (4.0 * t).cos();
                    c.push(pos2(0.5 + x / 34.0, 0.45 - y / 34.0));
                }
                vec![c]
            }
            ShapeKind::Line => vec![vec![pos2(0., 0.5), pos2(1., 0.5)]],
        }
    }
}

fn regular(n: usize, phase: f32) -> Contour {
    (0..=n)
        .map(|i| {
            let a = phase - std::f32::consts::FRAC_PI_2 + i as f32 * std::f32::consts::TAU / n as f32;
            pos2(0.5 + 0.5 * a.cos(), 0.5 + 0.5 * a.sin())
        })
        .collect()
}

/// Normalise contours into the unit box; returns (unit contours, natural size).
pub fn normalise(contours: &[Contour]) -> (Vec<Contour>, Vec2) {
    let (min, max) = bounds(contours.iter().flatten().copied());
    let size = (max - min).max(Vec2::splat(1e-4));
    let unit = contours
        .iter()
        .map(|c| c.iter().map(|p| pos2((p.x - min.x) / size.x, (p.y - min.y) / size.y)).collect())
        .collect();
    (unit, size)
}

pub fn bounds(points: impl Iterator<Item = Pos2>) -> (Pos2, Pos2) {
    let mut min = pos2(f32::MAX, f32::MAX);
    let mut max = pos2(f32::MIN, f32::MIN);
    let mut any = false;
    for p in points {
        any = true;
        min = min.min(p);
        max = max.max(p);
    }
    if !any {
        return (Pos2::ZERO, Pos2::ZERO);
    }
    (min, max)
}

/// Flatten a cubic Bézier into points (excluding the start point).
pub fn cubic(out: &mut Contour, p0: Pos2, p1: Pos2, p2: Pos2, p3: Pos2, steps: usize) {
    for i in 1..=steps {
        let t = i as f32 / steps as f32;
        let u = 1.0 - t;
        let p = p0.to_vec2() * (u * u * u)
            + p1.to_vec2() * (3.0 * u * u * t)
            + p2.to_vec2() * (3.0 * u * t * t)
            + p3.to_vec2() * (t * t * t);
        out.push(p.to_pos2());
    }
}

pub fn quad(out: &mut Contour, p0: Pos2, p1: Pos2, p2: Pos2, steps: usize) {
    for i in 1..=steps {
        let t = i as f32 / steps as f32;
        let u = 1.0 - t;
        let p = p0.to_vec2() * (u * u) + p1.to_vec2() * (2.0 * u * t) + p2.to_vec2() * (t * t);
        out.push(p.to_pos2());
    }
}

// ---------------------------------------------------------------- SVG reading

/// Parse an SVG path `d` attribute into flattened contours (in SVG user units).
pub fn parse_path_d(d: &str) -> Vec<Contour> {
    let mut toks: Vec<Tok> = Vec::new();
    let b = d.as_bytes();
    let mut i = 0;
    while i < b.len() {
        let c = b[i] as char;
        if c.is_ascii_alphabetic() && c != 'e' && c != 'E' {
            toks.push(Tok::Cmd(c));
            i += 1;
        } else if c == '-' || c == '+' || c == '.' || c.is_ascii_digit() {
            let start = i;
            i += 1;
            let mut seen_dot = c == '.';
            while i < b.len() {
                let ch = b[i] as char;
                if ch.is_ascii_digit() {
                    i += 1;
                } else if ch == '.' && !seen_dot {
                    seen_dot = true;
                    i += 1;
                } else if (ch == 'e' || ch == 'E') && i + 1 < b.len() {
                    i += 1;
                    if b[i] == b'-' || b[i] == b'+' {
                        i += 1;
                    }
                } else {
                    break;
                }
            }
            if let Ok(v) = d[start..i].parse::<f32>() {
                toks.push(Tok::Num(v));
            }
        } else {
            i += 1;
        }
    }

    let mut out: Vec<Contour> = Vec::new();
    let mut cur: Contour = Vec::new();
    let mut pen = Pos2::ZERO;
    let mut start = Pos2::ZERO;
    let mut last_ctrl: Option<Pos2> = None;
    let mut cmd = 'M';
    let mut k = 0;
    let num = |k: &mut usize| -> Option<f32> {
        if let Some(Tok::Num(v)) = toks.get(*k) {
            *k += 1;
            Some(*v)
        } else {
            None
        }
    };
    while k < toks.len() {
        if let Tok::Cmd(c) = toks[k] {
            cmd = c;
            k += 1;
            if c == 'Z' || c == 'z' {
                if !cur.is_empty() {
                    cur.push(start);
                    out.push(std::mem::take(&mut cur));
                }
                pen = start;
                last_ctrl = None;
                continue;
            }
        }
        let rel = cmd.is_ascii_lowercase();
        let base = if rel { pen.to_vec2() } else { Vec2::ZERO };
        match cmd.to_ascii_uppercase() {
            'M' => {
                let (Some(x), Some(y)) = (num(&mut k), num(&mut k)) else { break };
                if cur.len() > 1 {
                    out.push(std::mem::take(&mut cur));
                }
                cur.clear();
                pen = pos2(x, y) + base;
                start = pen;
                cur.push(pen);
                cmd = if rel { 'l' } else { 'L' };
                last_ctrl = None;
            }
            'L' => {
                let (Some(x), Some(y)) = (num(&mut k), num(&mut k)) else { break };
                pen = pos2(x, y) + base;
                cur.push(pen);
                last_ctrl = None;
            }
            'H' => {
                let Some(x) = num(&mut k) else { break };
                pen.x = if rel { pen.x + x } else { x };
                cur.push(pen);
                last_ctrl = None;
            }
            'V' => {
                let Some(y) = num(&mut k) else { break };
                pen.y = if rel { pen.y + y } else { y };
                cur.push(pen);
                last_ctrl = None;
            }
            'C' => {
                let v: Vec<f32> = (0..6).filter_map(|_| num(&mut k)).collect();
                if v.len() < 6 {
                    break;
                }
                let p1 = pos2(v[0], v[1]) + base;
                let p2 = pos2(v[2], v[3]) + base;
                let p3 = pos2(v[4], v[5]) + base;
                if cur.is_empty() {
                    cur.push(pen);
                }
                cubic(&mut cur, pen, p1, p2, p3, 16);
                last_ctrl = Some(p2);
                pen = p3;
            }
            'S' => {
                let v: Vec<f32> = (0..4).filter_map(|_| num(&mut k)).collect();
                if v.len() < 4 {
                    break;
                }
                let p1 = last_ctrl.map(|c| pen + (pen - c)).unwrap_or(pen);
                let p2 = pos2(v[0], v[1]) + base;
                let p3 = pos2(v[2], v[3]) + base;
                if cur.is_empty() {
                    cur.push(pen);
                }
                cubic(&mut cur, pen, p1, p2, p3, 16);
                last_ctrl = Some(p2);
                pen = p3;
            }
            'Q' => {
                let v: Vec<f32> = (0..4).filter_map(|_| num(&mut k)).collect();
                if v.len() < 4 {
                    break;
                }
                let p1 = pos2(v[0], v[1]) + base;
                let p2 = pos2(v[2], v[3]) + base;
                if cur.is_empty() {
                    cur.push(pen);
                }
                quad(&mut cur, pen, p1, p2, 12);
                last_ctrl = Some(p1);
                pen = p2;
            }
            'T' => {
                let v: Vec<f32> = (0..2).filter_map(|_| num(&mut k)).collect();
                if v.len() < 2 {
                    break;
                }
                let p1 = last_ctrl.map(|c| pen + (pen - c)).unwrap_or(pen);
                let p2 = pos2(v[0], v[1]) + base;
                if cur.is_empty() {
                    cur.push(pen);
                }
                quad(&mut cur, pen, p1, p2, 12);
                last_ctrl = Some(p1);
                pen = p2;
            }
            'A' => {
                // Arcs are approximated by a straight segment to the end point.
                let v: Vec<f32> = (0..7).filter_map(|_| num(&mut k)).collect();
                if v.len() < 7 {
                    break;
                }
                pen = pos2(v[5], v[6]) + base;
                cur.push(pen);
                last_ctrl = None;
            }
            _ => {
                k += 1;
            }
        }
    }
    if cur.len() > 1 {
        out.push(cur);
    }
    out
}

enum Tok {
    Cmd(char),
    Num(f32),
}

fn attr<'a>(tag: &'a str, name: &str) -> Option<&'a str> {
    let mut search = 0;
    while let Some(pos) = tag[search..].find(name) {
        let at = search + pos;
        let before_ok = at == 0 || tag.as_bytes()[at - 1].is_ascii_whitespace();
        let rest = &tag[at + name.len()..];
        let rest_trim = rest.trim_start();
        if before_ok && rest_trim.starts_with('=') {
            let after = rest_trim[1..].trim_start();
            let q = after.chars().next()?;
            if q == '"' || q == '\'' {
                let end = after[1..].find(q)?;
                return Some(&after[1..1 + end]);
            }
        }
        search = at + name.len();
    }
    None
}

fn num_attr(tag: &str, name: &str) -> f32 {
    attr(tag, name)
        .map(|v| v.trim_end_matches(|c: char| c.is_ascii_alphabetic() || c == '%'))
        .and_then(|v| v.trim().parse().ok())
        .unwrap_or(0.0)
}

/// How many inches one SVG user unit is, from the root element's width and viewBox.
/// Plain numbers and px count as 96 per inch (the CSS rule).
pub fn svg_inches_per_unit(text: &str) -> Option<f32> {
    let start = text.find("<svg")?;
    let tag = &text[start..start + text[start..].find('>')?];
    let to_in = |v: &str| -> Option<f32> {
        let v = v.trim();
        let num_end = v.find(|c: char| !(c.is_ascii_digit() || c == '.' || c == '-' || c == '+' || c == 'e' || c == 'E')).unwrap_or(v.len());
        let n: f32 = v[..num_end].parse().ok()?;
        let unit = v[num_end..].trim();
        Some(match unit {
            "mm" => n / 25.4,
            "cm" => n / 2.54,
            "in" => n,
            "pt" => n / 72.0,
            "pc" => n / 6.0,
            "" | "px" => n / 96.0,
            _ => return None, // percentages and the like say nothing about real size
        })
    };
    let width_in = attr(tag, "width").and_then(to_in);
    let vb_w = attr(tag, "viewBox").and_then(|vb| {
        let v: Vec<f32> = vb.split(|c: char| c == ',' || c.is_whitespace()).filter_map(|s| s.parse().ok()).collect();
        v.get(2).copied()
    });
    match (width_in, vb_w) {
        (Some(w), Some(vw)) if vw > 0.0 => Some(w / vw),
        (Some(w), None) => {
            let raw: f32 = attr(tag, "width")?.trim_end_matches(|c: char| c.is_ascii_alphabetic()).trim().parse().ok()?;
            if raw > 0.0 {
                Some(w / raw)
            } else {
                None
            }
        }
        (None, _) => Some(1.0 / 96.0),
        _ => None,
    }
}

/// Read the outlines of a simple SVG document (paths, rects, circles, ellipses, polygons, polylines, lines).
pub fn parse_svg(text: &str) -> Vec<Contour> {
    let mut out = Vec::new();
    let mut rest = text;
    while let Some(lt) = rest.find('<') {
        rest = &rest[lt + 1..];
        let Some(gt) = rest.find('>') else { break };
        let tag = &rest[..gt];
        rest = &rest[gt + 1..];
        let name: String = tag.chars().take_while(|c| c.is_ascii_alphanumeric()).collect();
        match name.as_str() {
            "path" => {
                if let Some(d) = attr(tag, "d") {
                    out.extend(parse_path_d(d));
                }
            }
            "rect" => {
                let (x, y, w, h) = (num_attr(tag, "x"), num_attr(tag, "y"), num_attr(tag, "width"), num_attr(tag, "height"));
                if w > 0.0 && h > 0.0 {
                    out.push(vec![pos2(x, y), pos2(x + w, y), pos2(x + w, y + h), pos2(x, y + h), pos2(x, y)]);
                }
            }
            "circle" | "ellipse" => {
                let (cx, cy) = (num_attr(tag, "cx"), num_attr(tag, "cy"));
                let (rx, ry) = if name == "circle" {
                    let r = num_attr(tag, "r");
                    (r, r)
                } else {
                    (num_attr(tag, "rx"), num_attr(tag, "ry"))
                };
                if rx > 0.0 && ry > 0.0 {
                    out.push(
                        (0..=64)
                            .map(|i| {
                                let a = i as f32 / 64.0 * std::f32::consts::TAU;
                                pos2(cx + rx * a.cos(), cy + ry * a.sin())
                            })
                            .collect(),
                    );
                }
            }
            "polygon" | "polyline" => {
                if let Some(pts) = attr(tag, "points") {
                    let v: Vec<f32> = pts
                        .split(|c: char| c == ',' || c.is_whitespace())
                        .filter_map(|s| s.parse().ok())
                        .collect();
                    let mut c: Contour = v.chunks(2).filter(|p| p.len() == 2).map(|p| pos2(p[0], p[1])).collect();
                    if name == "polygon" {
                        if let Some(f) = c.first().copied() {
                            c.push(f);
                        }
                    }
                    if c.len() > 1 {
                        out.push(c);
                    }
                }
            }
            "line" => {
                out.push(vec![
                    pos2(num_attr(tag, "x1"), num_attr(tag, "y1")),
                    pos2(num_attr(tag, "x2"), num_attr(tag, "y2")),
                ]);
            }
            _ => {}
        }
    }
    out
}

/// One contour as SVG path data, in the given unit scale.
pub fn contour_to_d(c: &Contour, scale: f32) -> String {
    let mut s = String::new();
    for (i, p) in c.iter().enumerate() {
        s.push_str(if i == 0 { "M" } else { " L" });
        s.push_str(&format!("{:.3} {:.3}", p.x * scale, p.y * scale));
    }
    if c.len() > 2 && c.first().map(|f| (*f - *c.last().unwrap()).length() < 1e-4).unwrap_or(false) {
        s.push_str(" Z");
    }
    s
}

// ---------------------------------------------------------------- Text outlines

struct OutlineSink {
    contours: Vec<Contour>,
    cur: Contour,
    pen: Pos2,
    origin_x: f32,
    scale: f32,
}

impl OutlineSink {
    fn p(&self, x: f32, y: f32) -> Pos2 {
        pos2(self.origin_x + x * self.scale, -y * self.scale)
    }
}

impl ttf_parser::OutlineBuilder for OutlineSink {
    fn move_to(&mut self, x: f32, y: f32) {
        if self.cur.len() > 1 {
            self.contours.push(std::mem::take(&mut self.cur));
        }
        self.cur.clear();
        self.pen = self.p(x, y);
        self.cur.push(self.pen);
    }
    fn line_to(&mut self, x: f32, y: f32) {
        self.pen = self.p(x, y);
        self.cur.push(self.pen);
    }
    fn quad_to(&mut self, x1: f32, y1: f32, x: f32, y: f32) {
        let (c, e) = (self.p(x1, y1), self.p(x, y));
        quad(&mut self.cur, self.pen, c, e, 8);
        self.pen = e;
    }
    fn curve_to(&mut self, x1: f32, y1: f32, x2: f32, y2: f32, x: f32, y: f32) {
        let (c1, c2, e) = (self.p(x1, y1), self.p(x2, y2), self.p(x, y));
        cubic(&mut self.cur, self.pen, c1, c2, e, 10);
        self.pen = e;
    }
    fn close(&mut self) {
        if let Some(f) = self.cur.first().copied() {
            self.cur.push(f);
        }
        if self.cur.len() > 1 {
            self.contours.push(std::mem::take(&mut self.cur));
        }
    }
}

/// Turn text into cut-ready outlines using the built-in typeface. Height of a capital ≈ 1 unit.
#[allow(dead_code)]
pub fn text_contours(text: &str) -> Vec<Contour> {
    layout_text(text, epaint_default_fonts::UBUNTU_LIGHT, 0, &TextOpts::default()).into_iter().flatten().collect()
}

#[derive(Clone, Copy, PartialEq, Eq, Debug, serde::Serialize, serde::Deserialize, Default)]
pub enum TextAlign {
    #[default]
    Left,
    Center,
    Right,
}

/// How a piece of text is laid out (the controls of the text bar).
#[derive(Clone, PartialEq, Debug, serde::Serialize, serde::Deserialize)]
#[serde(default)]
pub struct TextOpts {
    /// Font family name from the computer, or empty for the built-in typeface.
    pub font: String,
    pub bold: bool,
    pub italic: bool,
    /// Extra space between letters, in fractions of the capital height.
    pub letter_spacing: f32,
    /// Distance between lines, as a multiple of the font's normal line height.
    pub line_spacing: f32,
    pub align: TextAlign,
    /// Bend along an arc: degrees of arc the first line spans (positive bends up, negative down).
    pub curve: f32,
}

impl Default for TextOpts {
    fn default() -> Self {
        Self { font: String::new(), bold: false, italic: false, letter_spacing: 0.0, line_spacing: 1.0, align: TextAlign::Left, curve: 0.0 }
    }
}

/// The lines of a text: split on new lines, keeping empty ones (so a caret can sit on them).
pub fn text_lines(text: &str) -> Vec<&str> {
    text.split('\n').map(|l| l.strip_suffix('\r').unwrap_or(l)).collect()
}

/// Lay out text into outlines, one group of contours per letter (spaces give none).
/// A capital letter is 1 unit tall; y grows downward; the first line's baseline is at y = 0.
pub fn layout_text(text: &str, font_data: &[u8], face_index: u32, o: &TextOpts) -> Vec<Vec<Contour>> {
    layout_text_full(text, font_data, face_index, o).0
}

/// Like [`layout_text`], and also where the typing cursor goes before each character (and after
/// the last): one (top, bottom) pair per character position, counting new lines as characters.
pub fn layout_text_full(text: &str, font_data: &[u8], face_index: u32, o: &TextOpts) -> (Vec<Vec<Contour>>, Vec<(Pos2, Pos2)>) {
    let face = match ttf_parser::Face::parse(font_data, face_index) {
        Ok(f) => f,
        Err(_) => match ttf_parser::Face::parse(epaint_default_fonts::UBUNTU_LIGHT, 0) {
            Ok(f) => f,
            Err(_) => return (Vec::new(), Vec::new()),
        },
    };
    let cap = face.capital_height().filter(|c| *c > 0).unwrap_or((face.units_per_em() as f32 * 0.7) as i16) as f32;
    let scale = 1.0 / cap.max(1.0);
    let line_h = (face.height() as f32 * scale).max(1.2) * o.line_spacing.max(0.1);
    let slant = if o.italic && !face.is_italic() { 0.2 } else { 0.0 };
    let descent = (-(face.descender() as f32) * scale).clamp(0.15, 0.4);

    let mut letters: Vec<(usize, Vec<Contour>)> = Vec::new(); // (line index, contours)
    let mut carets: Vec<(usize, f32)> = Vec::new(); // (line index, x)
    let mut line_widths: Vec<f32> = Vec::new();
    for (li, line) in text_lines(text).into_iter().enumerate() {
        let mut x = 0.0f32;
        let mut prev: Option<ttf_parser::GlyphId> = None;
        for ch in line.chars() {
            let Some(gid) = face.glyph_index(ch) else {
                carets.push((li, x));
                x += 0.5 + o.letter_spacing;
                continue;
            };
            if let (Some(p), Some(kern)) = (prev, face.tables().kern) {
                for sub in kern.subtables {
                    if let Some(k) = sub.glyphs_kerning(p, gid) {
                        x += k as f32 * scale;
                        break;
                    }
                }
            }
            carets.push((li, x));
            let mut sink = OutlineSink { contours: Vec::new(), cur: Vec::new(), pen: Pos2::ZERO, origin_x: x, scale };
            let _ = face.outline_glyph(gid, &mut sink);
            if sink.cur.len() > 1 {
                let c = std::mem::take(&mut sink.cur);
                sink.contours.push(c);
            }
            let mut glyph = sink.contours;
            if slant != 0.0 {
                for c in &mut glyph {
                    for p in c.iter_mut() {
                        p.x -= p.y * slant;
                    }
                }
            }
            if o.bold && face.weight().to_number() < 600 {
                glyph = offset_contours(&glyph, 0.025);
            }
            for c in &mut glyph {
                for p in c.iter_mut() {
                    p.y += li as f32 * line_h;
                }
            }
            if !glyph.is_empty() {
                letters.push((li, glyph));
            }
            x += face.glyph_hor_advance(gid).unwrap_or(500) as f32 * scale + o.letter_spacing;
            prev = Some(gid);
        }
        let end = if line.is_empty() { 0.0 } else { (x - o.letter_spacing).max(0.0) };
        carets.push((li, end)); // after the last character (the new line's position)
        line_widths.push(end);
    }
    // alignment within the widest line
    let widest = line_widths.iter().cloned().fold(0.0f32, f32::max);
    let shift_of = |li: usize| match o.align {
        TextAlign::Left => 0.0,
        TextAlign::Center => (widest - line_widths[li]) / 2.0,
        TextAlign::Right => widest - line_widths[li],
    };
    for (li, g) in &mut letters {
        let shift = shift_of(*li);
        for c in g.iter_mut() {
            for p in c.iter_mut() {
                p.x += shift;
            }
        }
    }
    let mut caret_pts: Vec<(Pos2, Pos2)> = carets
        .iter()
        .map(|&(li, x)| {
            let base = li as f32 * line_h;
            let xx = x + shift_of(li);
            // slanted like the letters when the italic is made by slanting
            (pos2(xx + 1.05 * slant, base - 1.05), pos2(xx - descent * slant, base + descent))
        })
        .collect();
    // bend along an arc
    if o.curve.abs() > 0.5 && widest > 0.0 {
        let angle = o.curve.to_radians();
        let r = widest / angle; // signed radius; centre sits below the text for positive curves
        let cx = widest / 2.0;
        let bend = |p: Pos2| {
            let phi = (p.x - cx) / r;
            let dist = r - p.y;
            pos2(cx + phi.sin() * dist, r - phi.cos() * dist)
        };
        for (_, g) in &mut letters {
            for c in g.iter_mut() {
                for p in c.iter_mut() {
                    *p = bend(*p);
                }
            }
        }
        for (a, b) in caret_pts.iter_mut() {
            *a = bend(*a);
            *b = bend(*b);
        }
    }
    (letters.into_iter().map(|(_, g)| g).collect(), caret_pts)
}

// ---------------------------------------------------------------- shape tools (weld, slice, offset)

fn to_multi(contours: &[Contour]) -> geo::MultiPolygon<f64> {
    use geo::{Coord, LineString, Polygon};
    geo::MultiPolygon(
        contours
            .iter()
            .filter(|c| c.len() >= 3)
            .map(|c| {
                let mut pts: Vec<Coord<f64>> = c.iter().map(|p| Coord { x: p.x as f64, y: p.y as f64 }).collect();
                if pts.first() != pts.last() {
                    pts.push(pts[0]);
                }
                Polygon::new(LineString(pts), vec![])
            })
            .collect(),
    )
}

fn from_multi(m: &geo::MultiPolygon<f64>) -> Vec<Contour> {
    let mut out = Vec::new();
    for poly in &m.0 {
        for ring in std::iter::once(poly.exterior()).chain(poly.interiors().iter()) {
            let c: Contour = ring.0.iter().map(|p| pos2(p.x as f32, p.y as f32)).collect();
            if c.len() >= 3 {
                out.push(c);
            }
        }
    }
    out
}

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum Combine {
    Unite,
    Subtract,
    Intersect,
    Exclude,
}

/// Combine outlines (each list read with the even-odd rule, so holes stay holes).
pub fn combine(a: &[Contour], b: &[Contour], how: Combine) -> Vec<Contour> {
    use geo::BooleanOps;
    let (ma, mb) = (to_multi(a), to_multi(b));
    let r = match how {
        Combine::Unite => ma.union(&mb),
        Combine::Subtract => ma.difference(&mb),
        Combine::Intersect => ma.intersection(&mb),
        Combine::Exclude => ma.xor(&mb),
    };
    from_multi(&r)
}

/// Weld: merge everything into one outline with the overlaps removed.
pub fn weld(parts: &[Vec<Contour>]) -> Vec<Contour> {
    let mut acc: Vec<Contour> = Vec::new();
    for p in parts {
        acc = if acc.is_empty() { combine(p, &[], Combine::Unite) } else { combine(&acc, p, Combine::Unite) };
    }
    acc
}

/// Slice two shapes into the pieces they make: A without B, the overlap, B without A.
pub fn slice(a: &[Contour], b: &[Contour]) -> [Vec<Contour>; 3] {
    [combine(a, b, Combine::Subtract), combine(a, b, Combine::Intersect), combine(b, a, Combine::Subtract)]
}

/// Grow (positive) or shrink (negative) outlines by a distance, with rounded corners.
pub fn offset_contours(contours: &[Contour], distance: f32) -> Vec<Contour> {
    use geo::Buffer;
    if contours.is_empty() {
        return Vec::new();
    }
    // tidy overlapping parts first so the offset follows the visible outline
    let clean = to_multi(&combine(contours, &[], Combine::Unite));
    from_multi(&clean.buffer(distance as f64))
}

/// Douglas-Peucker simplification of an open or closed polyline.
pub fn simplify(c: &Contour, eps: f32) -> Contour {
    if c.len() < 3 {
        return c.clone();
    }
    let mut keep = vec![false; c.len()];
    keep[0] = true;
    keep[c.len() - 1] = true;
    let mut stack = vec![(0usize, c.len() - 1)];
    while let Some((a, b)) = stack.pop() {
        let (pa, pb) = (c[a], c[b]);
        let d = pb - pa;
        let len = d.length().max(1e-9);
        let mut best = (0.0f32, 0usize);
        for i in a + 1..b {
            let v = c[i] - pa;
            let dist = if len < 1e-6 { v.length() } else { (v.x * d.y - v.y * d.x).abs() / len };
            if dist > best.0 {
                best = (dist, i);
            }
        }
        if best.0 > eps {
            keep[best.1] = true;
            stack.push((a, best.1));
            stack.push((best.1, b));
        }
    }
    c.iter().zip(keep).filter(|(_, k)| *k).map(|(p, _)| *p).collect()
}

// ---------------------------------------------------------------- DXF reading

/// Read lines, polylines, circles, arcs and spline control points from a DXF drawing.
pub fn parse_dxf(text: &str) -> Vec<Contour> {
    let lines: Vec<&str> = text.lines().map(|l| l.trim()).collect();
    let mut pairs: Vec<(i32, &str)> = Vec::new();
    let mut i = 0;
    while i + 1 < lines.len() {
        if let Ok(code) = lines[i].parse::<i32>() {
            pairs.push((code, lines[i + 1]));
        }
        i += 2;
    }
    let mut out: Vec<Contour> = Vec::new();
    let mut k = 0;
    let num = |s: &str| s.parse::<f32>().unwrap_or(0.0);
    let mut poly: Option<(Contour, bool)> = None; // old-style POLYLINE ... VERTEX ... SEQEND
    while k < pairs.len() {
        if pairs[k].0 != 0 {
            k += 1;
            continue;
        }
        let kind = pairs[k].1;
        let mut j = k + 1;
        let mut fields: Vec<(i32, &str)> = Vec::new();
        while j < pairs.len() && pairs[j].0 != 0 {
            fields.push(pairs[j]);
            j += 1;
        }
        let get = |c: i32| fields.iter().find(|f| f.0 == c).map(|f| num(f.1));
        match kind {
            "LINE" => {
                if let (Some(x1), Some(y1), Some(x2), Some(y2)) = (get(10), get(20), get(11), get(21)) {
                    out.push(vec![pos2(x1, -y1), pos2(x2, -y2)]);
                }
            }
            "LWPOLYLINE" | "SPLINE" => {
                let mut c: Contour = Vec::new();
                let mut x = None;
                for f in &fields {
                    match f.0 {
                        10 => x = Some(num(f.1)),
                        20 => {
                            if let Some(xv) = x.take() {
                                c.push(pos2(xv, -num(f.1)));
                            }
                        }
                        _ => {}
                    }
                }
                let closed = get(70).map(|f| (f as i32) & 1 == 1).unwrap_or(false);
                if closed {
                    if let Some(f) = c.first().copied() {
                        c.push(f);
                    }
                }
                if c.len() > 1 {
                    out.push(c);
                }
            }
            "CIRCLE" | "ARC" => {
                if let (Some(cx), Some(cy), Some(r)) = (get(10), get(20), get(40)) {
                    let (a0, a1) = if kind == "ARC" {
                        let s = get(50).unwrap_or(0.0);
                        let mut e = get(51).unwrap_or(360.0);
                        if e < s {
                            e += 360.0;
                        }
                        (s, e)
                    } else {
                        (0.0, 360.0)
                    };
                    let steps = (((a1 - a0) / 6.0).ceil() as usize).max(4);
                    out.push(
                        (0..=steps)
                            .map(|n| {
                                let a = (a0 + (a1 - a0) * n as f32 / steps as f32).to_radians();
                                pos2(cx + r * a.cos(), -(cy + r * a.sin()))
                            })
                            .collect(),
                    );
                }
            }
            "POLYLINE" => {
                let closed = get(70).map(|f| (f as i32) & 1 == 1).unwrap_or(false);
                poly = Some((Vec::new(), closed));
            }
            "VERTEX" => {
                if let (Some((c, _)), Some(x), Some(y)) = (poly.as_mut(), get(10), get(20)) {
                    c.push(pos2(x, -y));
                }
            }
            "SEQEND" => {
                if let Some((mut c, closed)) = poly.take() {
                    if closed {
                        if let Some(f) = c.first().copied() {
                            c.push(f);
                        }
                    }
                    if c.len() > 1 {
                        out.push(c);
                    }
                }
            }
            _ => {}
        }
        k = j;
    }
    out
}

// ---------------------------------------------------------------- engraving fills

/// The fill patterns of the original's "Vector Fill" tool (mstoker-vector-fill).
#[derive(Clone, Copy, PartialEq, Eq, Debug, serde::Serialize, serde::Deserialize, Default)]
pub enum FillPattern {
    #[default]
    HorizontalLines,
    VerticalLines,
    Grid,
    ForwardSlash,
    BackwardSlash,
    CrossHatch,
    Concentric,
}

impl FillPattern {
    pub const ALL: [FillPattern; 7] = [
        FillPattern::HorizontalLines,
        FillPattern::VerticalLines,
        FillPattern::Grid,
        FillPattern::ForwardSlash,
        FillPattern::BackwardSlash,
        FillPattern::CrossHatch,
        FillPattern::Concentric,
    ];
    pub fn label(self) -> &'static str {
        match self {
            FillPattern::HorizontalLines => "Horizontal lines",
            FillPattern::VerticalLines => "Vertical lines",
            FillPattern::Grid => "Grid",
            FillPattern::ForwardSlash => "Forward slash",
            FillPattern::BackwardSlash => "Backward slash",
            FillPattern::CrossHatch => "Cross-hatch",
            FillPattern::Concentric => "Concentric",
        }
    }
    /// Line angles in degrees (empty for concentric).
    fn angles(self) -> &'static [f32] {
        match self {
            FillPattern::HorizontalLines => &[0.0],
            FillPattern::VerticalLines => &[90.0],
            FillPattern::Grid => &[0.0, 90.0],
            FillPattern::ForwardSlash => &[-45.0],
            FillPattern::BackwardSlash => &[45.0],
            FillPattern::CrossHatch => &[45.0, -45.0],
            FillPattern::Concentric => &[],
        }
    }
}

/// Fill outlines with engraving lines: parallel lines at `spacing` (in the outlines' units) for
/// the line patterns, or rings stepping inward for concentric. `inset` keeps that much clear
/// inside the edges (the "kerf"). `even_odd` picks the fill rule (holes stay empty with either
/// for ordinary artwork; they differ only where outlines overlap themselves).
/// Lines run back and forth so the tool travels as little as possible.
pub fn hatch(contours: &[Contour], pattern: FillPattern, spacing: f32, inset: f32, even_odd: bool) -> Vec<Contour> {
    let spacing = spacing.max(0.005);
    let closed: Vec<Contour> = contours.iter().filter(|c| c.len() >= 3).cloned().collect();
    if closed.is_empty() {
        return Vec::new();
    }
    // tidy overlaps first so the fill follows what's visible
    let base = if even_odd { closed.clone() } else { combine(&closed, &[], Combine::Unite) };
    let area = if inset > 0.0 { offset_contours(&base, -inset) } else { base };
    if area.is_empty() {
        return Vec::new();
    }
    if pattern == FillPattern::Concentric {
        let mut out = Vec::new();
        let mut ring = area.clone();
        for _ in 0..2000 {
            if ring.is_empty() {
                break;
            }
            out.extend(ring.iter().cloned());
            ring = offset_contours(&ring, -spacing);
        }
        return out;
    }
    let mut out = Vec::new();
    for &deg in pattern.angles() {
        let (s, c) = (-deg).to_radians().sin_cos();
        let rot = |p: Pos2| pos2(p.x * c - p.y * s, p.x * s + p.y * c);
        let (s2, c2) = deg.to_radians().sin_cos();
        let back = |p: Pos2| pos2(p.x * c2 - p.y * s2, p.x * s2 + p.y * c2);
        let turned: Vec<Contour> = area.iter().map(|ct| ct.iter().map(|p| rot(*p)).collect()).collect();
        let (min, max) = bounds(turned.iter().flatten().copied());
        let mut y = min.y + spacing / 2.0;
        let mut row = 0usize;
        let mut xs: Vec<f32> = Vec::new();
        while y < max.y && row < 20000 {
            xs.clear();
            for ct in &turned {
                let n = ct.len();
                for k in 0..n {
                    let a = ct[k];
                    let b = ct[(k + 1) % n];
                    if (a.y <= y && b.y > y) || (b.y <= y && a.y > y) {
                        xs.push(a.x + (y - a.y) / (b.y - a.y) * (b.x - a.x));
                    }
                }
            }
            xs.sort_by(|a, b| a.partial_cmp(b).unwrap_or(std::cmp::Ordering::Equal));
            let mut segs: Vec<Contour> = xs
                .chunks(2)
                .filter(|p| p.len() == 2 && p[1] - p[0] > 1e-6)
                .map(|p| vec![back(pos2(p[0], y)), back(pos2(p[1], y))])
                .collect();
            if row % 2 == 1 {
                segs.reverse();
                for sgm in segs.iter_mut() {
                    sgm.reverse();
                }
            }
            out.extend(segs);
            y += spacing;
            row += 1;
        }
    }
    out
}

// ---------------------------------------------------------------- tracing pictures

/// Trace the edges of a filled mask (true = keep) into closed outlines, in pixel units.
/// Marching squares, then simplified.
pub fn trace_mask(mask: &[bool], w: usize, h: usize, simplify_px: f32) -> Vec<Contour> {
    use std::collections::HashMap;
    let at = |x: i64, y: i64| -> bool { x >= 0 && y >= 0 && (x as usize) < w && (y as usize) < h && mask[y as usize * w + x as usize] };
    // points live on cell-edge midpoints, stored doubled so they stay integers
    let mut next: HashMap<(i64, i64), Vec<(i64, i64)>> = HashMap::new();
    let mut add = |a: (i64, i64), b: (i64, i64)| next.entry(a).or_default().push(b);
    for y in -1..h as i64 {
        for x in -1..w as i64 {
            let tl = at(x, y);
            let tr = at(x + 1, y);
            let br = at(x + 1, y + 1);
            let bl = at(x, y + 1);
            let case = (tl as u8) << 3 | (tr as u8) << 2 | (br as u8) << 1 | bl as u8;
            let top = (2 * x + 1, 2 * y);
            let right = (2 * x + 2, 2 * y + 1);
            let bottom = (2 * x + 1, 2 * y + 2);
            let left = (2 * x, 2 * y + 1);
            // segments are oriented so the kept side is on the left
            match case {
                0 | 15 => {}
                1 => add(left, bottom),
                2 => add(bottom, right),
                3 => add(left, right),
                4 => add(right, top),
                5 => {
                    add(left, top);
                    add(right, bottom);
                }
                6 => add(bottom, top),
                7 => add(left, top),
                8 => add(top, left),
                9 => add(top, bottom),
                10 => {
                    add(top, right);
                    add(bottom, left);
                }
                11 => add(top, right),
                12 => add(right, left),
                13 => add(right, bottom),
                14 => add(bottom, left),
                _ => {}
            }
        }
    }
    let mut out = Vec::new();
    let starts: Vec<(i64, i64)> = next.keys().copied().collect();
    for s in starts {
        loop {
            let Some(first) = next.get_mut(&s).and_then(|v| v.pop()) else { break };
            let mut c: Contour = vec![pos2(s.0 as f32 / 2.0, s.1 as f32 / 2.0)];
            let mut cur = first;
            let mut guard = 0;
            while cur != s && guard < 4_000_000 {
                guard += 1;
                c.push(pos2(cur.0 as f32 / 2.0, cur.1 as f32 / 2.0));
                match next.get_mut(&cur).and_then(|v| v.pop()) {
                    Some(n) => cur = n,
                    None => break,
                }
            }
            c.push(c[0]);
            if c.len() > 4 {
                let s = simplify(&c, simplify_px);
                if s.len() >= 4 {
                    out.push(s);
                }
            }
        }
    }
    // drop specks
    out.retain(|c| {
        let (a, b) = bounds(c.iter().copied());
        (b - a).length() > simplify_px * 3.0
    });
    out
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn path_parsing() {
        let c = parse_path_d("M0 0 L10 0 L10 10 Z m 20 0 h5 v5 z");
        assert_eq!(c.len(), 2);
        assert_eq!(c[0].len(), 4);
    }
    #[test]
    fn svg_shapes() {
        let c = parse_svg("<svg><rect x='1' y='1' width='2' height='3'/><circle cx=\"5\" cy=\"5\" r=\"1\"/></svg>");
        assert_eq!(c.len(), 2);
    }
    #[test]
    fn text_has_outlines() {
        assert!(!text_contours("Hi").is_empty());
        let curved = layout_text("Hello", epaint_default_fonts::UBUNTU_LIGHT, 0, &TextOpts { curve: 90.0, ..Default::default() });
        assert_eq!(curved.len(), 5);
        // one caret position per character plus the end, counting the new line
        let (_, carets) = layout_text_full("Hi\nyo", epaint_default_fonts::UBUNTU_LIGHT, 0, &TextOpts::default());
        assert_eq!(carets.len(), 6);
        assert!(carets[1].0.x > carets[0].0.x);
        assert!(carets[3].0.y > carets[2].0.y); // second line sits lower
        let (_, empty) = layout_text_full("a\n", epaint_default_fonts::UBUNTU_LIGHT, 0, &TextOpts::default());
        assert_eq!(empty.len(), 3);
    }
    #[test]
    fn weld_two_squares() {
        let a = vec![vec![pos2(0., 0.), pos2(2., 0.), pos2(2., 2.), pos2(0., 2.), pos2(0., 0.)]];
        let b = vec![vec![pos2(1., 1.), pos2(3., 1.), pos2(3., 3.), pos2(1., 3.), pos2(1., 1.)]];
        let w = weld(&[a.clone(), b.clone()]);
        assert_eq!(w.len(), 1);
        let [left, mid, right] = slice(&a, &b);
        assert!(!left.is_empty() && !mid.is_empty() && !right.is_empty());
        assert!(!offset_contours(&a, 0.25).is_empty());
    }
    #[test]
    fn trace_square_mask() {
        let (w, h) = (20, 20);
        let mut m = vec![false; w * h];
        for y in 5..15 {
            for x in 5..15 {
                m[y * w + x] = true;
            }
        }
        let c = trace_mask(&m, w, h, 0.5);
        assert_eq!(c.len(), 1);
    }
    #[test]
    fn engrave_fills() {
        // a 1 x 1 square with a 0.5 x 0.5 hole
        let outer = vec![pos2(0., 0.), pos2(1., 0.), pos2(1., 1.), pos2(0., 1.), pos2(0., 0.)];
        let hole = vec![pos2(0.25, 0.25), pos2(0.75, 0.25), pos2(0.75, 0.75), pos2(0.25, 0.75), pos2(0.25, 0.25)];
        let shape = vec![outer, hole];
        let lines = hatch(&shape, FillPattern::HorizontalLines, 0.1, 0.0, true);
        // 10 rows; the 5 rows through the hole split in two
        assert_eq!(lines.len(), 15);
        // nothing is drawn inside the hole
        for l in &lines {
            let mid = pos2((l[0].x + l[1].x) / 2.0, l[0].y);
            assert!(!(mid.x > 0.26 && mid.x < 0.74 && mid.y > 0.26 && mid.y < 0.74));
        }
        // a margin shortens the lines
        let inset = hatch(&shape, FillPattern::HorizontalLines, 0.1, 0.05, true);
        let longest = |ls: &Vec<Contour>| ls.iter().map(|l| (l[1] - l[0]).length()).fold(0.0f32, f32::max);
        assert!(longest(&inset) < longest(&lines));
        // cross-hatch has two directions, concentric gives rings
        assert!(hatch(&shape, FillPattern::CrossHatch, 0.1, 0.0, true).len() > lines.len());
        assert!(hatch(&shape, FillPattern::Concentric, 0.1, 0.0, true).len() >= 2);
    }
    #[test]
    fn svg_real_size() {
        let k = svg_inches_per_unit("<svg width=\"50.8mm\" height=\"10mm\" viewBox=\"0 0 50.8 10\">").unwrap();
        assert!((k - 1.0 / 25.4).abs() < 1e-6);
        assert!((svg_inches_per_unit("<svg viewBox='0 0 10 10'>").unwrap() - 1.0 / 96.0).abs() < 1e-6);
        assert!((svg_inches_per_unit("<svg width=\"2in\" viewBox=\"0 0 200 100\">").unwrap() - 0.01).abs() < 1e-6);
    }
    #[test]
    fn dxf_circle_and_line() {
        let d = "0\nSECTION\n2\nENTITIES\n0\nLINE\n10\n0\n20\n0\n11\n5\n21\n5\n0\nCIRCLE\n10\n1\n20\n1\n40\n2\n0\nENDSEC\n0\nEOF\n";
        assert_eq!(parse_dxf(d).len(), 2);
    }
}
