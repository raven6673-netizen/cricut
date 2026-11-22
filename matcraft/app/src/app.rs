//! The window: a design canvas with the editing tools, a "prepare mats" screen, and the machine panel.

use crate::fonts;
use crate::geometry::{self, Combine, Contour, Operation, ShapeKind, TextAlign, TextOpts};
use crate::imaging::{self, Cleanup, Picture};
use crate::machine::{self, Cmd, Event, Link, MachineHandle, Request};
use crate::materials::{self, MATERIALS};
use egui::epaint::Shadow;
use egui::{
    pos2, vec2, Align, Align2, Color32, CornerRadius, FontId, Key, Layout, Margin, Mesh, Pos2, Rect, RichText,
    Sense, Shape, Stroke, StrokeKind, TextStyle, TextureHandle, Ui, Vec2,
};
use std::collections::{HashMap, HashSet};
use std::sync::atomic::{AtomicBool, AtomicU64, Ordering};

// ---------------------------------------------------------------- look

const HEADER: Color32 = Color32::from_rgb(0x15, 0x1d, 0x26);
const HEADER_2: Color32 = Color32::from_rgb(0x24, 0x30, 0x3c);
const ACCENT: Color32 = Color32::from_rgb(0x10, 0xa3, 0x6b);

// ---------------------------------------------------------------- light and dark themes

/// Deep green for text and icons on light backgrounds; a light green in dark mode so it stays readable.
#[allow(non_snake_case)]
fn ACCENT_DARK() -> Color32 {
    themed(Color32::from_rgb(0x0b, 0x7d, 0x52), Color32::from_rgb(0x5e, 0xe0, 0xa8))
}

static DARK: AtomicBool = AtomicBool::new(false);

/// Whether the app is in dark mode.
pub fn dark() -> bool {
    DARK.load(Ordering::Relaxed)
}

/// Pick the light or dark version of a colour.
fn themed(light: Color32, dark_c: Color32) -> Color32 {
    if dark() {
        dark_c
    } else {
        light
    }
}

#[allow(non_snake_case)]
fn BG() -> Color32 {
    themed(Color32::from_rgb(0xec, 0xef, 0xf3), Color32::from_rgb(0x12, 0x16, 0x1b))
}
#[allow(non_snake_case)]
fn SURFACE() -> Color32 {
    themed(Color32::WHITE, Color32::from_rgb(0x1c, 0x22, 0x29))
}
#[allow(non_snake_case)]
fn SUBTLE() -> Color32 {
    themed(Color32::from_rgb(0xf5, 0xf7, 0xfa), Color32::from_rgb(0x25, 0x2c, 0x34))
}
#[allow(non_snake_case)]
fn BORDER() -> Color32 {
    themed(Color32::from_rgb(0xe1, 0xe6, 0xec), Color32::from_rgb(0x34, 0x3d, 0x47))
}
#[allow(non_snake_case)]
pub fn INK() -> Color32 {
    themed(Color32::from_rgb(0x1f, 0x29, 0x33), Color32::from_rgb(0xe6, 0xea, 0xef))
}
#[allow(non_snake_case)]
pub fn MUTED() -> Color32 {
    themed(Color32::from_rgb(0x6b, 0x77, 0x85), Color32::from_rgb(0x9a, 0xa5, 0xb1))
}
#[allow(non_snake_case)]
fn ACCENT_SOFT() -> Color32 {
    themed(Color32::from_rgb(0xe3, 0xf6, 0xee), Color32::from_rgb(0x13, 0x3a, 0x2b))
}
#[allow(non_snake_case)]
fn SELECT_SOFT() -> Color32 {
    themed(Color32::from_rgb(0xe8, 0xf0, 0xfe), Color32::from_rgb(0x1a, 0x2c, 0x4a))
}
/// Background of the choice pills.
#[allow(non_snake_case)]
fn SEG_BG() -> Color32 {
    themed(Color32::from_rgb(0xe9, 0xed, 0xf2), Color32::from_rgb(0x2a, 0x32, 0x3b))
}
/// The second tool bar (text bar).
#[allow(non_snake_case)]
fn BAR_ALT() -> Color32 {
    themed(Color32::from_rgb(0xfa, 0xfb, 0xfd), Color32::from_rgb(0x20, 0x27, 0x2f))
}
#[allow(non_snake_case)]
fn RAIL_FG() -> Color32 {
    themed(Color32::from_rgb(0x4a, 0x55, 0x61), Color32::from_rgb(0xb4, 0xbe, 0xc9))
}
#[allow(non_snake_case)]
fn DISABLED() -> Color32 {
    themed(Color32::from_gray(190), Color32::from_gray(85))
}
#[allow(non_snake_case)]
fn PRESSED() -> Color32 {
    themed(Color32::from_rgb(0xcf, 0xee, 0xe0), Color32::from_rgb(0x1a, 0x4d, 0x39))
}
/// The cutting area on the canvas and its grid.
#[allow(non_snake_case)]
fn SHEET() -> Color32 {
    themed(Color32::WHITE, Color32::from_rgb(0x22, 0x29, 0x31))
}
#[allow(non_snake_case)]
fn GRID_MAJOR() -> Color32 {
    themed(Color32::from_rgb(0xe3, 0xe8, 0xee), Color32::from_rgb(0x34, 0x3e, 0x49))
}
#[allow(non_snake_case)]
fn GRID_MINOR() -> Color32 {
    themed(Color32::from_rgb(0xf1, 0xf4, 0xf7), Color32::from_rgb(0x2a, 0x32, 0x3b))
}
#[allow(non_snake_case)]
fn SHEET_EDGE() -> Color32 {
    themed(Color32::from_rgb(0xd5, 0xdc, 0xe4), Color32::from_rgb(0x41, 0x4c, 0x58))
}
#[allow(non_snake_case)]
fn DOTS() -> Color32 {
    themed(Color32::from_rgb(0xcf, 0xd6, 0xde), Color32::from_rgb(0x30, 0x38, 0x42))
}
#[allow(non_snake_case)]
fn TICK() -> Color32 {
    themed(Color32::from_rgb(0xb7, 0xc0, 0xca), Color32::from_rgb(0x56, 0x61, 0x6d))
}
/// Striped rows in tables.
#[allow(non_snake_case)]
pub fn STRIPE() -> Color32 {
    themed(Color32::from_rgb(0xf4, 0xf6, 0xf8), Color32::from_rgb(0x23, 0x2a, 0x32))
}
const SELECT: Color32 = Color32::from_rgb(0x3b, 0x82, 0xf6);
const DANGER: Color32 = Color32::from_rgb(0xdc, 0x4c, 0x4c);
const MAT_BLUE: Color32 = Color32::from_rgb(0x7f, 0xb8, 0xe0);
const MAT_STICKY: Color32 = Color32::from_rgb(0xc4, 0xe0, 0xf3);
const MAT_PRINT: Color32 = Color32::from_rgb(0xf3, 0xf4, 0xf6);

const PALETTE: [Color32; 8] = [
    Color32::from_rgb(0xec, 0x5f, 0x7a),
    Color32::from_rgb(0x3f, 0x8c, 0xdc),
    Color32::from_rgb(0xf3, 0xb2, 0x36),
    Color32::from_rgb(0x2f, 0xaa, 0x6d),
    Color32::from_rgb(0x8e, 0x5c, 0xd6),
    Color32::from_rgb(0x33, 0x3a, 0x44),
    Color32::from_rgb(0xf0, 0x7b, 0x3f),
    Color32::from_rgb(0x3f, 0xb8, 0xc2),
];

/// Space kept between pieces and from the mat's edge, in inches.
const MAT_GAP: f32 = 0.25;
/// Screen pixels per design inch at 100 % zoom.
const BASE_PPI: f32 = 48.0;
/// Width of the rulers along the canvas edges, in pixels.
const RULER: f32 = 24.0;
/// Where a project's design area starts on the canvas (inches): a quarter inch in from the
/// mat's corner, where it lands on the mat when the layout is kept.
const DESIGN_AREA_ORIGIN: Pos2 = pos2(0.25, 0.25);
/// Largest printable area for print then cut, in inches (letter paper with margins; estimated).


fn op_color(op: Operation) -> Color32 {
    match op {
        Operation::Cut => ACCENT,
        Operation::Draw => SELECT,
        Operation::Score => Color32::from_rgb(0xd9, 0x8b, 0x10),
        Operation::Engrave => Color32::from_rgb(0x8e, 0x5c, 0xd6),
        Operation::Deboss => Color32::from_rgb(0x6d, 0x4c, 0x41),
        Operation::Wave => Color32::from_rgb(0x00, 0x97, 0xa7),
        Operation::Perforate => Color32::from_rgb(0xc2, 0x18, 0x5b),
        Operation::Print => Color32::from_rgb(0xe6, 0x51, 0x00),
        Operation::Guide => MUTED(),
    }
}

fn shadow(blur: u8, alpha: u8) -> Shadow {
    Shadow { offset: [0, 2], blur, spread: 0, color: Color32::from_black_alpha(alpha) }
}

static NEXT_UID: AtomicU64 = AtomicU64::new(1);
fn new_uid() -> u64 {
    NEXT_UID.fetch_add(1, Ordering::Relaxed)
}

// ---------------------------------------------------------------- model

#[derive(Clone, serde::Serialize, serde::Deserialize)]
pub struct Item {
    pub name: String,
    #[serde(with = "contours_serde")]
    pub unit: Vec<Contour>,
    pub pos: [f32; 2],
    pub size: [f32; 2],
    pub rot: f32,
    pub color: [u8; 3],
    pub op: Operation,
    pub visible: bool,
    pub text: Option<String>,
    #[serde(default)]
    pub text_opts: Option<TextOpts>,
    /// Natural size of the laid-out text, so a change of font keeps the letter height.
    #[serde(default)]
    pub natural: [f32; 2],
    #[serde(default)]
    pub group: Option<u32>,
    #[serde(default)]
    pub attach: Option<u32>,
    /// Outlines hidden with the Contour tool (not cut).
    #[serde(default)]
    pub hidden: Vec<bool>,
    /// A picture printed for print then cut, as base64 PNG, and where it sits in the unit box.
    #[serde(default)]
    pub image_png: Option<String>,
    #[serde(default)]
    pub img_rect: [f32; 4],
    #[serde(default)]
    pub img_flip: [bool; 2],
    /// How an Engrave layer is engraved.
    #[serde(default)]
    pub engrave: EngraveOpts,
    #[serde(skip)]
    pub uid: u64,
}

/// How an Engrave layer is engraved: along its outline, filled with a line pattern, or (for
/// pictures) as rows of dots and dashes following the picture's light and dark.
#[derive(Clone, Copy, PartialEq, Eq, Debug, serde::Serialize, serde::Deserialize, Default)]
pub enum EngraveStyle {
    #[default]
    Outline,
    Fill,
    Photo,
}

impl EngraveStyle {
    pub fn label(self) -> &'static str {
        match self {
            EngraveStyle::Outline => "Outline",
            EngraveStyle::Fill => "Fill",
            EngraveStyle::Photo => "Photo",
        }
    }
}

/// Defaults from the original's fill tool (mstoker-vector-fill): spacing 0.01 in, kerf 0.01 in,
/// non-zero fill rule.
pub(crate) const ENGRAVE_DEFAULT_SPACING: f32 = 0.01;
pub(crate) const ENGRAVE_DEFAULT_MARGIN: f32 = 0.01;

#[derive(Clone, Copy, PartialEq, Debug, serde::Serialize, serde::Deserialize)]
#[serde(default)]
pub struct EngraveOpts {
    pub style: EngraveStyle,
    pub pattern: geometry::FillPattern,
    /// Distance between engraved lines, in inches.
    pub spacing: f32,
    /// Space left clear inside the edges, in inches.
    pub margin: f32,
    pub even_odd: bool,
    /// Photo: lighten (+) or darken (-), -1..1.
    pub brightness: f32,
    /// Photo: engrave the light parts instead of the dark ones.
    pub invert: bool,
}

impl Default for EngraveOpts {
    fn default() -> Self {
        Self {
            style: EngraveStyle::Outline,
            pattern: geometry::FillPattern::HorizontalLines,
            spacing: ENGRAVE_DEFAULT_SPACING,
            margin: ENGRAVE_DEFAULT_MARGIN,
            even_odd: false,
            brightness: 0.0,
            invert: false,
        }
    }
}

mod contours_serde {
    use super::Contour;
    use serde::{Deserialize, Deserializer, Serialize, Serializer};
    pub fn serialize<S: Serializer>(c: &[Contour], s: S) -> Result<S::Ok, S::Error> {
        let v: Vec<Vec<[f32; 2]>> = c.iter().map(|c| c.iter().map(|p| [p.x, p.y]).collect()).collect();
        v.serialize(s)
    }
    pub fn deserialize<'de, D: Deserializer<'de>>(d: D) -> Result<Vec<Contour>, D::Error> {
        let v: Vec<Vec<[f32; 2]>> = Vec::deserialize(d)?;
        Ok(v.into_iter().map(|c| c.into_iter().map(|p| egui::pos2(p[0], p[1])).collect()).collect())
    }
}

impl Item {
    fn new(name: &str, unit: Vec<Contour>, pos: Pos2, size: Vec2, color: Color32) -> Self {
        Self {
            name: name.into(),
            unit,
            pos: [pos.x, pos.y],
            size: [size.x, size.y],
            rot: 0.0,
            color: [color.r(), color.g(), color.b()],
            op: Operation::Cut,
            visible: true,
            text: None,
            text_opts: None,
            natural: [0.0, 0.0],
            group: None,
            attach: None,
            hidden: Vec::new(),
            image_png: None,
            img_rect: [0.0, 0.0, 1.0, 1.0],
            img_flip: [false, false],
            engrave: EngraveOpts::default(),
            uid: new_uid(),
        }
    }
    /// A new item from outlines already in design inches (rotation baked in).
    fn from_world(name: &str, contours: Vec<Contour>, color: [u8; 3], op: Operation) -> Option<Self> {
        if contours.is_empty() {
            return None;
        }
        let (min, max) = geometry::bounds(contours.iter().flatten().copied());
        let size = (max - min).max(Vec2::splat(1e-4));
        let unit = contours.iter().map(|c| c.iter().map(|p| pos2((p.x - min.x) / size.x, (p.y - min.y) / size.y)).collect()).collect();
        let mut it = Item::new(name, unit, min, size, Color32::from_rgb(color[0], color[1], color[2]));
        it.op = op;
        Some(it)
    }
    fn color32(&self) -> Color32 {
        Color32::from_rgb(self.color[0], self.color[1], self.color[2])
    }
    fn center(&self) -> Pos2 {
        pos2(self.pos[0] + self.size[0] / 2.0, self.pos[1] + self.size[1] / 2.0)
    }
    fn set_center(&mut self, c: Pos2) {
        self.pos = [c.x - self.size[0] / 2.0, c.y - self.size[1] / 2.0];
    }
    /// Unit-box point to design inches (scale, then rotate around the centre).
    fn to_world(&self, u: Pos2) -> Pos2 {
        let c = self.center();
        let p = vec2((u.x - 0.5) * self.size[0], (u.y - 0.5) * self.size[1]);
        let (s, co) = self.rot.to_radians().sin_cos();
        c + vec2(p.x * co - p.y * s, p.x * s + p.y * co)
    }
    fn from_world_pt(&self, w: Pos2) -> Pos2 {
        let c = self.center();
        let d = w - c;
        let (s, co) = (-self.rot).to_radians().sin_cos();
        let p = vec2(d.x * co - d.y * s, d.x * s + d.y * co);
        pos2(p.x / self.size[0].max(1e-4) + 0.5, p.y / self.size[1].max(1e-4) + 0.5)
    }
    fn is_hidden(&self, i: usize) -> bool {
        self.hidden.get(i).copied().unwrap_or(false)
    }
    /// Outlines in design inches, without the ones hidden with Contour.
    pub fn world_contours(&self) -> Vec<Contour> {
        self.unit
            .iter()
            .enumerate()
            .filter(|(i, _)| !self.is_hidden(*i))
            .map(|(_, c)| c.iter().map(|p| self.to_world(*p)).collect())
            .collect()
    }
    fn all_world_contours(&self) -> Vec<Contour> {
        self.unit.iter().map(|c| c.iter().map(|p| self.to_world(*p)).collect()).collect()
    }
    fn world_bounds(&self) -> Rect {
        if self.unit.iter().all(|c| c.is_empty()) {
            let pts = [pos2(0., 0.), pos2(1., 0.), pos2(1., 1.), pos2(0., 1.)].map(|u| self.to_world(u));
            let (a, b) = geometry::bounds(pts.into_iter());
            return Rect::from_min_max(a, b);
        }
        let (a, b) = geometry::bounds(self.all_world_contours().into_iter().flatten());
        Rect::from_min_max(a, b)
    }
    fn is_text(&self) -> bool {
        self.text.is_some()
    }
}

/// Lay a text item out again after its words or text settings changed, keeping its letter height.
fn regen_text(it: &mut Item) {
    let Some(text) = it.text.clone() else { return };
    let opts = it.text_opts.clone().unwrap_or_default();
    let fd = fonts::font_data(&opts.font, opts.bold, opts.italic);
    let all: Vec<Contour> = geometry::layout_text(&text, &fd.0, fd.1, &opts).into_iter().flatten().collect();
    // keep the top-left corner where it is, so typing grows the text to the right like a word processor
    let anchor = it.to_world(pos2(0.0, 0.0));
    if all.is_empty() {
        // nothing to draw yet (empty, or only spaces): keep the box so typing can continue
        it.unit.clear();
        it.hidden.clear();
        return;
    }
    let (min, max) = geometry::bounds(all.iter().flatten().copied());
    let nat = (max - min).max(Vec2::splat(1e-4));
    let k = if it.natural[1] > 1e-4 { it.size[1] / it.natural[1] } else { 1.0 };
    it.unit = all.iter().map(|c| c.iter().map(|p| pos2((p.x - min.x) / nat.x, (p.y - min.y) / nat.y)).collect()).collect();
    it.natural = [nat.x, nat.y];
    it.size = [nat.x * k, nat.y * k];
    let fix = anchor - it.to_world(pos2(0.0, 0.0));
    it.pos[0] += fix.x;
    it.pos[1] += fix.y;
    it.hidden.clear();
}

/// A decoded picture: width, height, RGBA pixels.
type Pixels = (usize, usize, Vec<u8>);

/// The engraving lines of a layer, in the layer's own inches (x across its width, y down its
/// height, before rotation). None when it's engraved along its outline (or not engraved).
fn engrave_lines_local(it: &Item, picture: Option<&Pixels>) -> Option<Vec<Contour>> {
    if it.op != Operation::Engrave {
        return None;
    }
    let e = it.engrave;
    let sz = vec2(it.size[0].max(1e-4), it.size[1].max(1e-4));
    match (e.style, picture) {
        (EngraveStyle::Outline, _) => None,
        (EngraveStyle::Photo, Some((w, h, rgba))) => {
            let r = it.img_rect;
            let img_w_in = ((r[2] - r[0]) * sz.x).abs().max(1e-4);
            let px_per_in = *w as f32 / img_w_in;
            let cell = e.spacing.max(0.003) * px_per_in;
            let lines = imaging::photo_engrave_lines(rgba, *w, *h, cell, e.brightness, e.invert);
            let (ox, oy) = (r[0] * sz.x, r[1] * sz.y);
            let (sx, sy) = ((r[2] - r[0]) * sz.x / *w as f32, (r[3] - r[1]) * sz.y / *h as f32);
            let (fx, fy) = (it.img_flip[0], it.img_flip[1]);
            Some(
                lines
                    .into_iter()
                    .map(|l| {
                        l.into_iter()
                            .map(|p| {
                                let px = if fx { *w as f32 - p.x } else { p.x };
                                let py = if fy { *h as f32 - p.y } else { p.y };
                                pos2(ox + px * sx, oy + py * sy)
                            })
                            .collect()
                    })
                    .collect(),
            )
        }
        _ => {
            let local: Vec<Contour> = it
                .unit
                .iter()
                .enumerate()
                .filter(|(i, _)| !it.is_hidden(*i))
                .map(|(_, c)| c.iter().map(|p| pos2(p.x * sz.x, p.y * sz.y)).collect())
                .collect();
            Some(geometry::hatch(&local, e.pattern, e.spacing, e.margin, e.even_odd))
        }
    }
}

/// Engraving lines from a layer's own inches to design inches.
fn engrave_to_world(it: &Item, local: &[Contour]) -> Vec<Contour> {
    let sz = vec2(it.size[0].max(1e-4), it.size[1].max(1e-4));
    local.iter().map(|l| l.iter().map(|p| it.to_world(pos2(p.x / sz.x, p.y / sz.y))).collect()).collect()
}

/// Where the typing cursor of a text layer sits for every character position, in design inches
/// (top and bottom of the cursor line).
fn text_carets_world(it: &Item) -> Vec<(Pos2, Pos2)> {
    let Some(text) = &it.text else { return Vec::new() };
    let opts = it.text_opts.clone().unwrap_or_default();
    let fd = fonts::font_data(&opts.font, opts.bold, opts.italic);
    let (letters, carets) = geometry::layout_text_full(text, &fd.0, fd.1, &opts);
    let all: Vec<Pos2> = letters.iter().flatten().flatten().copied().collect();
    if all.is_empty() {
        // no letters yet: put the cursor at the left of the box, one line per new line typed
        let h = it.size[1].max(0.05);
        return carets
            .iter()
            .map(|(a, _)| {
                let line = (a.y + 1.05) / 1.05;
                let top = it.to_world(pos2(0.0, 0.0)) + vec2(0.0, line * h * 1.2);
                (top, top + vec2(0.0, h))
            })
            .collect();
    }
    let (min, max) = geometry::bounds(all.into_iter());
    let nat = (max - min).max(Vec2::splat(1e-4));
    let to_unit = |p: Pos2| pos2((p.x - min.x) / nat.x, (p.y - min.y) / nat.y);
    carets.iter().map(|(a, b)| (it.to_world(to_unit(*a)), it.to_world(to_unit(*b)))).collect()
}

/// The font list opened from the text bar: arrow keys move through it and the text changes live.
struct FontPicker {
    filter: String,
    /// Place in the filtered list (0 is the built-in typeface).
    highlight: usize,
    /// The font before the list was opened, for Esc.
    original: String,
    just_opened: bool,
    scroll_to: bool,
    anchor: Rect,
}

/// Typing straight into a text layer on the canvas.
struct TextEditing {
    uid: u64,
    /// Cursor position, counted in characters.
    caret: usize,
    /// When the cursor last moved, so it stays visible while typing.
    since: f64,
}

#[derive(Clone, Copy, PartialEq)]
enum Screen {
    Home,
    MyStuff,
    Design,
    Make,
}

mod home;
pub(crate) mod projects;
mod settings_ui;
use projects::{DesignArea, GridMode, ProjectType, Prefs};

/// The "What do you want to make?" window's choices.
struct NewProject {
    /// None shows every project type's templates.
    kind: Option<ProjectType>,
    template: usize,
    size: usize,
    w: f32,
    h: f32,
    show_needs: bool,
}

/// A saved project found in My Stuff.
#[derive(Clone)]
struct SavedProject {
    path: std::path::PathBuf,
    name: String,
    modified: Option<std::time::SystemTime>,
    kind: ProjectType,
    items: Vec<Item>,
}

#[derive(Clone, Copy, PartialEq)]
enum Drawer {
    None,
    Shapes,
    Text,
    Upload,
    Projects,
}

#[derive(Clone, Copy, PartialEq)]
enum Drag {
    None,
    Move,
    Resize,
    Rotate,
    Pan,
    Marquee,
}

#[derive(Clone, Copy, PartialEq)]
enum AlignHow {
    Left,
    CenterH,
    Right,
    Top,
    CenterV,
    Bottom,
    Center,
    DistributeH,
    DistributeV,
}

#[derive(Clone, Copy, PartialEq)]
enum Arrange {
    Back,
    Backward,
    Forward,
    Front,
}

/// Everything the menus and buttons can ask for; applied after the frame's UI is drawn.
#[derive(Clone, Copy, PartialEq)]
enum Act {
    Undo,
    Redo,
    Cut,
    Copy,
    Paste,
    Duplicate,
    Delete,
    SelectAll,
    Deselect,
    Group,
    Ungroup,
    Align(AlignHow),
    Arrange(Arrange),
    FlipH,
    FlipV,
    Weld,
    Combine(Combine),
    Slice,
    Attach,
    Detach,
    Flatten,
    Unflatten,
    Offset,
    Contour,
    UngroupLetters,
    Nudge(f32, f32),
}

#[derive(Clone, Copy, PartialEq, serde::Serialize, serde::Deserialize)]
enum LinkKind {
    Bluetooth,
    Usb,
    Device,
    Demo,
}

#[derive(Clone, serde::Serialize, serde::Deserialize)]
struct MachineSettings {
    link: LinkKind,
    mac: String,
    channel: u8,
    device_path: String,
    /// USB port; empty finds the machine by its USB id.
    #[serde(default)]
    usb_port: String,
    key_hex: String,
    /// Cut pressure sent in SetMachineState (0x30), in the machine's Z encoder counts. Used as a raw
    /// fallback only when the machine's calibration hasn't been read. 0 means "don't set pressure".
    #[serde(default)]
    pressure: u16,
    /// Cut pressure in grams. Converted to Z encoder counts with the machine's own calibration table
    /// (read at connect), so the count stays in the firmware's stable range. 0 means "don't cut".
    #[serde(default = "default_grams")]
    pressure_grams: u16,
}

fn default_grams() -> u16 {
    150
}

impl Default for MachineSettings {
    fn default() -> Self {
        Self {
            link: if !machine::find_usb_machines().is_empty() {
                LinkKind::Usb
            } else if cfg!(target_os = "linux") {
                LinkKind::Bluetooth
            } else {
                LinkKind::Demo
            },
            mac: machine::DEFAULT_MAC.into(),
            channel: 1,
            device_path: if cfg!(windows) { r"\\.\COM5".into() } else { "/dev/rfcomm0".into() },
            usb_port: String::new(),
            key_hex: String::new(),
            pressure: 150,
            pressure_grams: default_grams(),
        }
    }
}

pub(crate) fn config_dir() -> std::path::PathBuf {
    if let Ok(x) = std::env::var("XDG_CONFIG_HOME") {
        return std::path::PathBuf::from(x).join("matcraft");
    }
    if let Ok(a) = std::env::var("APPDATA") {
        return std::path::PathBuf::from(a).join("matcraft");
    }
    let home = std::env::var("HOME").unwrap_or_else(|_| ".".into());
    std::path::PathBuf::from(home).join(".config").join("matcraft")
}

fn documents_dir() -> std::path::PathBuf {
    let home = std::env::var("HOME").or_else(|_| std::env::var("USERPROFILE")).unwrap_or_else(|_| ".".into());
    std::path::PathBuf::from(home).join("Documents").join("MatCraft")
}

#[derive(Clone, Copy, PartialEq)]
enum MatKind {
    Cut,
    Print,
}

/// A mat to send: its colour and the pieces placed on it (offset from design inches to mat inches).
struct Mat {
    kind: MatKind,
    color: Color32,
    pieces: Vec<(usize, Vec2)>,
    overflow: bool,
}

/// The photo/drawing upload window.
struct ImageImport {
    name: String,
    pic: Picture,
    cleanup: Cleanup,
    print: bool,
    /// Engrave the picture (photo engraving) instead of cutting or printing it.
    engrave: bool,
    lines_for_engrave: bool,
    width_in: f32,
    mask: Vec<bool>,
    orig_tex: Option<TextureHandle>,
    preview_tex: Option<TextureHandle>,
    dirty: bool,
    /// The lines that will be cut, in picture pixels (recomputed when settings change).
    lines: Vec<Contour>,
    lines_for_print: bool,
    /// Preview view: 1.0 fits the picture; pan is in screen pixels from the centred position.
    zoom: f32,
    pan: Vec2,
    view: PreviewView,
    show_lines: bool,
}

#[derive(Clone, Copy, PartialEq)]
enum PreviewView {
    SideBySide,
    Result,
    Original,
}

struct OffsetTool {
    distance: f32,
}

pub struct App {
    screen: Screen,
    drawer: Drawer,
    items: Vec<Item>,
    selection: Vec<usize>,
    clipboard: Vec<Item>,
    next_group: u32,
    undo: Vec<Vec<Item>>,
    redo: Vec<Vec<Item>>,
    zoom: f32,
    pan: Vec2,
    drag: Drag,
    drag_start: Pos2,
    drag_orig: Vec<(usize, Item)>,
    drag_bounds: Rect,
    marquee_end: Pos2,
    edit_snapshot_taken: bool,
    lock_aspect: bool,
    text_input: String,
    svg_input: String,
    project_path: String,
    status: String,
    project_name: String,
    textures: HashMap<u64, TextureHandle>,
    image_import: Option<ImageImport>,
    offset_tool: Option<OffsetTool>,
    contour_item: Option<usize>,
    text_edit: Option<TextEditing>,
    font_picker: Option<FontPicker>,
    upload_queue: Vec<String>,
    svg_note: String,
    // Prepare mats
    mat_long: bool,
    mat_index: usize,
    material: usize,
    mirror: bool,
    // Machine
    machine: MachineHandle,
    settings: MachineSettings,
    connected: Option<String>,
    #[cfg(target_os = "linux")]
    pairing: bool,
    #[cfg(target_os = "linux")]
    pairing_result: Option<String>,
    log: Vec<String>,
    show_machine: bool,
    show_compare: bool,
    compare_filter: Option<crate::compare::Status>,
    compare_tab: u8,
    mech_report: Option<crate::mechanics_check::Report>,
    theta: f32,
    last_status: Option<(Cmd, Option<u8>)>,
    /// Drag-blade pressure calibration read from the machine: (grams, counts), sorted by force.
    calibration: Vec<(u16, u16)>,
    // projects, settings, engraving
    prefs: Prefs,
    project_type: ProjectType,
    design_area: Option<DesignArea>,
    new_project: Option<NewProject>,
    show_settings: bool,
    settings_tab: u8,
    custom_draft: crate::materials::CustomMaterial,
    engrave_cache: HashMap<u64, (String, Vec<Contour>)>,
    pic_cache: HashMap<u64, Pixels>,
    snap_guides: Vec<(bool, f32)>,
    last_autosave: f64,
    last_saved_json: String,
    my_stuff: Option<Vec<SavedProject>>,
    home_search: String,
    autoconnect_tried: bool,
}

/// Set the look for the current theme (light or dark). Called at start and when the theme changes.
fn apply_style(ctx: &egui::Context) {
    let is_dark = dark();
    ctx.set_theme(if is_dark { egui::Theme::Dark } else { egui::Theme::Light });
    ctx.all_styles_mut(|style| {
        style.visuals = if is_dark { egui::Visuals::dark() } else { egui::Visuals::light() };
        use egui::FontFamily::Proportional;
        style.text_styles.insert(TextStyle::Body, FontId::new(14.0, Proportional));
        style.text_styles.insert(TextStyle::Button, FontId::new(14.0, Proportional));
        style.text_styles.insert(TextStyle::Small, FontId::new(11.5, Proportional));
        style.text_styles.insert(TextStyle::Heading, FontId::new(20.0, Proportional));
        style.spacing.item_spacing = vec2(8.0, 6.0);
        style.spacing.button_padding = vec2(10.0, 6.0);
        style.spacing.interact_size.y = 30.0;
        style.spacing.window_margin = Margin::same(16);

        let v = &mut style.visuals;
        v.panel_fill = SURFACE();
        v.window_fill = SURFACE();
        v.window_stroke = Stroke::new(1.0, BORDER());
        v.window_corner_radius = CornerRadius::same(14);
        v.window_shadow = Shadow { offset: [0, 8], blur: 28, spread: 0, color: Color32::from_black_alpha(40) };
        v.popup_shadow = Shadow { offset: [0, 4], blur: 14, spread: 0, color: Color32::from_black_alpha(30) };
        v.menu_corner_radius = CornerRadius::same(10);
        v.extreme_bg_color = SUBTLE();
        v.faint_bg_color = SUBTLE();
        v.selection.bg_fill = SELECT;
        v.selection.stroke = Stroke::new(1.0, Color32::WHITE);
        v.hyperlink_color = SELECT;

        let r = CornerRadius::same(8);
        v.widgets.noninteractive.bg_stroke = Stroke::new(1.0, BORDER());
        v.widgets.noninteractive.fg_stroke = Stroke::new(1.0, INK());
        v.widgets.noninteractive.corner_radius = r;
        v.widgets.inactive.bg_fill = SUBTLE();
        v.widgets.inactive.weak_bg_fill = SUBTLE();
        v.widgets.inactive.bg_stroke = Stroke::new(1.0, BORDER());
        v.widgets.inactive.fg_stroke = Stroke::new(1.0, INK());
        v.widgets.inactive.corner_radius = r;
        v.widgets.hovered.bg_fill = ACCENT_SOFT();
        v.widgets.hovered.weak_bg_fill = ACCENT_SOFT();
        v.widgets.hovered.bg_stroke = Stroke::new(1.0, ACCENT);
        v.widgets.hovered.fg_stroke = Stroke::new(1.5, themed(ACCENT_DARK(), Color32::from_rgb(0x5e, 0xe0, 0xa8)));
        v.widgets.hovered.corner_radius = r;
        v.widgets.hovered.expansion = 0.0;
        v.widgets.active.bg_fill = PRESSED();
        v.widgets.active.weak_bg_fill = PRESSED();
        v.widgets.active.bg_stroke = Stroke::new(1.0, ACCENT_DARK());
        v.widgets.active.fg_stroke = Stroke::new(1.5, themed(ACCENT_DARK(), Color32::from_rgb(0x8c, 0xf0, 0xc4)));
        v.override_text_color = None;
        if is_dark {
            v.window_shadow.color = Color32::from_black_alpha(120);
            v.popup_shadow.color = Color32::from_black_alpha(110);
            v.text_edit_bg_color = Some(Color32::from_rgb(0x16, 0x1b, 0x21));
        }
        v.widgets.active.corner_radius = r;
        v.widgets.active.expansion = 0.0;
        v.widgets.open.corner_radius = r;
        v.widgets.open.weak_bg_fill = ACCENT_SOFT();
    });
}

impl App {
    pub fn new(cc: &eframe::CreationContext<'_>) -> Self {
        // dark or light: what was chosen last time, else what the computer is set to
        let saved: Option<bool> = std::fs::read_to_string(config_dir().join("look.json"))
            .ok()
            .and_then(|s| serde_json::from_str::<serde_json::Value>(&s).ok())
            .and_then(|v| v.get("dark").and_then(|d| d.as_bool()));
        let start_dark = if std::env::args().any(|a| a == "--dark") {
            true
        } else if std::env::args().any(|a| a == "--light") {
            false
        } else {
            saved.unwrap_or(cc.egui_ctx.system_theme() == Some(egui::Theme::Dark))
        };
        DARK.store(start_dark, Ordering::Relaxed);
        apply_style(&cc.egui_ctx);
        fonts::start_loading();

        let settings = std::fs::read_to_string(config_dir().join("machine.json"))
            .ok()
            .and_then(|s| serde_json::from_str(&s).ok())
            .unwrap_or_default();

        let mut app = Self {
            screen: Screen::Design,
            drawer: Drawer::Shapes,
            items: Vec::new(),
            selection: Vec::new(),
            clipboard: Vec::new(),
            next_group: 1,
            undo: Vec::new(),
            redo: Vec::new(),
            zoom: 1.0,
            pan: vec2(RULER + 36.0, RULER + 28.0),
            drag: Drag::None,
            drag_start: Pos2::ZERO,
            drag_orig: Vec::new(),
            drag_bounds: Rect::NOTHING,
            marquee_end: Pos2::ZERO,
            edit_snapshot_taken: false,
            lock_aspect: true,
            text_input: "Hello".into(),
            svg_input: String::new(),
            project_path: documents_dir().join("project.matcraft.json").display().to_string(),
            status: String::new(),
            project_name: "Untitled project".into(),
            textures: HashMap::new(),
            image_import: None,
            offset_tool: None,
            contour_item: None,
            text_edit: None,
            font_picker: None,
            upload_queue: Vec::new(),
            svg_note: String::new(),
            mat_long: false,
            mat_index: 0,
            material: 1,
            mirror: false,
            machine: machine::spawn(cc.egui_ctx.clone()),
            settings,
            connected: None,
            #[cfg(target_os = "linux")]
            pairing: false,
            #[cfg(target_os = "linux")]
            pairing_result: None,
            log: Vec::new(),
            show_machine: false,
            show_compare: false,
            compare_filter: None,
            compare_tab: 0,
            mech_report: None,
            theta: 90.0,
            calibration: Vec::new(),
            last_status: None,
            prefs: std::fs::read_to_string(config_dir().join("prefs.json")).ok().and_then(|s| serde_json::from_str(&s).ok()).unwrap_or_default(),
            project_type: ProjectType::Blank,
            design_area: None,
            new_project: None,
            show_settings: false,
            settings_tab: 0,
            custom_draft: crate::materials::CustomMaterial { name: "My material".into(), tool: crate::materials::Tool::FineBlade, pressure: 4, passes: 1 },
            engrave_cache: HashMap::new(),
            pic_cache: HashMap::new(),
            snap_guides: Vec::new(),
            last_autosave: 0.0,
            last_saved_json: String::new(),
            my_stuff: None,
            home_search: String::new(),
            autoconnect_tried: false,
        };
        app.add_sample();
        app.selection = vec![1];
        // the original opens on its home page; the canvas options of the command line start on the canvas
        let canvas_flags = ["--select", "--edit", "--drawer", "--upload", "--canvas", "--compare", "--make", "--engrave-sample", "--machine"];
        if !std::env::args().any(|a| canvas_flags.contains(&a.as_str())) {
            app.screen = Screen::Home;
        }
        app
    }

    fn save_prefs(&self) {
        let dir = config_dir();
        let _ = std::fs::create_dir_all(&dir);
        if let Ok(s) = serde_json::to_string_pretty(&self.prefs) {
            let _ = std::fs::write(dir.join("prefs.json"), s);
        }
    }

    /// The paper area for print then cut, from the chosen page size.
    fn print_area(&self) -> Vec2 {
        self.prefs.page.print_area()
    }

    /// Decode pictures and work out engraving lines for layers that need them (only when
    /// something about the layer changed; moving or rotating it doesn't).
    fn ensure_engrave(&mut self) {
        let live: HashSet<u64> = self.items.iter().map(|i| i.uid).collect();
        self.engrave_cache.retain(|k, _| live.contains(k));
        self.pic_cache.retain(|k, _| live.contains(k));
        for i in 0..self.items.len() {
            let it = &self.items[i];
            if it.op != Operation::Engrave || it.engrave.style == EngraveStyle::Outline {
                continue;
            }
            if it.engrave.style == EngraveStyle::Photo && it.image_png.is_some() && !self.pic_cache.contains_key(&it.uid) {
                if let Some(b64) = &it.image_png {
                    if let Ok(img) = image::load_from_memory(&imaging::b64_decode(b64)) {
                        let rgba = img.to_rgba8();
                        self.pic_cache.insert(it.uid, (rgba.width() as usize, rgba.height() as usize, rgba.into_raw()));
                    }
                }
            }
            let shape_sum: f32 = it.unit.iter().flatten().map(|p| p.x * 1.3 + p.y * 0.7).sum();
            let sig = format!("{:?}|{:?}|{}|{}|{:?}|{:?}|{:?}", it.engrave, it.size, it.unit.len(), shape_sum, it.hidden, it.img_rect, it.img_flip);
            if self.engrave_cache.get(&it.uid).map(|(s, _)| s == &sig).unwrap_or(false) {
                continue;
            }
            let pic = if it.engrave.style == EngraveStyle::Photo { self.pic_cache.get(&it.uid) } else { None };
            if let Some(lines) = engrave_lines_local(it, pic) {
                self.engrave_cache.insert(it.uid, (sig, lines));
            }
        }
        self.engrave_cache.retain(|k, _| self.items.iter().any(|it| it.uid == *k && it.op == Operation::Engrave && it.engrave.style != EngraveStyle::Outline));
    }

    /// A length for display: inches, or millimetres in metric.
    fn fmt_len(&self, inches: f32) -> String {
        if self.prefs.metric {
            format!("{:.1} mm", inches * 25.4)
        } else {
            format!("{inches:.2} in")
        }
    }

    fn fmt_size(&self, w: f32, h: f32) -> String {
        if self.prefs.metric {
            format!("{:.1} x {:.1} mm", w * 25.4, h * 25.4)
        } else {
            format!("{w:.2} x {h:.2} in")
        }
    }

    /// What new shapes, text and pictures are set to.
    fn default_op(&self) -> Operation {
        self.project_type.default_op()
    }

    /// Where new things go: inside the design area when there is one.
    fn place_point(&self) -> Pos2 {
        match &self.design_area {
            Some(_) => pos2(DESIGN_AREA_ORIGIN.x + 0.5, DESIGN_AREA_ORIGIN.y + 0.5),
            None => pos2(2.0, 2.0),
        }
    }

    /// Save automatically to My Stuff every minute when something changed.
    fn autosave(&mut self, now: f64) {
        if !self.prefs.autosave || self.screen == Screen::Home || now - self.last_autosave < 60.0 || self.items.is_empty() {
            return;
        }
        self.last_autosave = now;
        let json = self.project_json();
        if json != self.last_saved_json {
            let p = documents_dir().join("Autosave.matcraft.json");
            let _ = std::fs::create_dir_all(documents_dir());
            if std::fs::write(&p, &json).is_ok() {
                self.last_saved_json = json;
            }
        }
    }

    fn project_json(&self) -> String {
        let f = projects::ProjectFile {
            version: 2,
            name: self.project_name.clone(),
            project_type: self.project_type,
            design_area: self.design_area.clone(),
            items: self.items.clone(),
        };
        serde_json::to_string(&f).unwrap_or_default()
    }

    /// Switch between light and dark mode and remember the choice.
    fn toggle_dark(&mut self, ctx: &egui::Context) {
        let now = !dark();
        DARK.store(now, Ordering::Relaxed);
        apply_style(ctx);
        let dir = config_dir();
        let _ = std::fs::create_dir_all(&dir);
        let _ = std::fs::write(dir.join("look.json"), serde_json::json!({ "dark": now }).to_string());
    }

    /// Open the comparison window (used by `--compare`, and `--behaviour` for its machine tab).
    pub fn open_compare(&mut self) {
        self.show_compare = true;
        if std::env::args().any(|a| a == "--behaviour") {
            self.compare_tab = 1;
        }
    }

    /// Select one layer by its place in the list (used by `--select N`, for checking the bars).
    pub fn select_index(&mut self, i: usize) {
        if i < self.items.len() {
            self.selection = vec![i];
        }
    }

    /// Start typing into a text layer (used by `--edit N`, for checking the typing view).
    pub fn edit_index(&mut self, i: usize) {
        self.start_text_edit(i, 0.0);
        if let Some(t) = self.text_edit.as_mut() {
            t.caret = 3; // inside the word, so the picture shows the cursor between letters
        }
    }

    /// Upload a file as if it had been dropped (used by `--upload <file>`).
    pub fn upload_path(&mut self, path: &str) {
        self.drawer = Drawer::Upload;
        self.import_file(path);
    }

    /// Open a side panel by name (used by `--drawer upload|text|projects|shapes`).
    pub fn open_drawer(&mut self, name: &str) {
        self.drawer = match name {
            "upload" => Drawer::Upload,
            "text" => Drawer::Text,
            "projects" => Drawer::Projects,
            "shapes" => Drawer::Shapes,
            _ => Drawer::None,
        };
    }

    /// Open the Machine panel (used by `--machine`).
    pub fn open_machine_panel(&mut self) {
        self.show_machine = true;
    }

    /// Open Settings on a tab (used by `--settings N`).
    pub fn open_settings(&mut self, tab: u8) {
        self.show_settings = true;
        self.settings_tab = tab;
    }

    /// Open the "what do you want to make" picker on engraving (used by `--new-project`).
    pub fn open_picker(&mut self) {
        self.open_new_project(Some(projects::ProjectType::Engrave));
    }

    /// Start on the prepare-mats screen (used by `--make`).
    pub fn open_make(&mut self) {
        self.screen = Screen::Make;
        self.mat_index = 0;
    }

    fn add_sample(&mut self) {
        let heart = Item::new("Heart", ShapeKind::Heart.unit_contours(), pos2(1.0, 1.0), vec2(3.0, 2.7), PALETTE[0]);
        let mut star = Item::new("Star", ShapeKind::Star.unit_contours(), pos2(4.8, 1.2), vec2(2.6, 2.5), PALETTE[2]);
        star.rot = 8.0;
        self.items.push(heart);
        self.items.push(star);
        self.add_text_item("Hello", pos2(1.2, 4.6), PALETTE[1]);
    }

    // ------------------------------------------------ selection helpers

    fn sel_one(&self) -> Option<usize> {
        if self.selection.len() == 1 && self.selection[0] < self.items.len() {
            Some(self.selection[0])
        } else {
            None
        }
    }

    /// Add the other members of any group a selected item belongs to.
    fn with_groups(&self, mut set: Vec<usize>) -> Vec<usize> {
        let groups: HashSet<u32> = set.iter().filter_map(|&i| self.items.get(i).and_then(|it| it.group)).collect();
        for (i, it) in self.items.iter().enumerate() {
            if it.group.map(|g| groups.contains(&g)).unwrap_or(false) && !set.contains(&i) {
                set.push(i);
            }
        }
        set.retain(|i| *i < self.items.len());
        set.sort_unstable();
        set.dedup();
        set
    }

    fn select(&mut self, i: usize, add: bool) {
        let members = self.with_groups(vec![i]);
        if add {
            if members.iter().all(|m| self.selection.contains(m)) {
                self.selection.retain(|s| !members.contains(s));
            } else {
                self.selection.extend(members);
                self.selection.sort_unstable();
                self.selection.dedup();
            }
        } else {
            self.selection = members;
        }
    }

    fn selection_bounds(&self) -> Rect {
        self.selection.iter().filter_map(|&i| self.items.get(i)).fold(Rect::NOTHING, |r, it| r.union(it.world_bounds()))
    }

    fn selected_uids(&self) -> HashSet<u64> {
        self.selection.iter().filter_map(|&i| self.items.get(i)).map(|it| it.uid).collect()
    }

    fn reselect_uids(&mut self, uids: &HashSet<u64>) {
        self.selection = (0..self.items.len()).filter(|&i| uids.contains(&self.items[i].uid)).collect();
    }

    fn snapshot(&mut self) {
        self.undo.push(self.items.clone());
        if self.undo.len() > 100 {
            self.undo.remove(0);
        }
        self.redo.clear();
    }

    fn next_color(&self) -> Color32 {
        PALETTE[self.items.len() % PALETTE.len()]
    }

    fn push_and_select(&mut self, it: Item) {
        self.items.push(it);
        self.selection = vec![self.items.len() - 1];
    }

    fn add_shape(&mut self, kind: ShapeKind) {
        self.snapshot();
        let size = kind.default_size();
        let mut it = Item::new(kind.label(), kind.unit_contours(), self.place_point(), size, self.next_color());
        // the project decides (an Engrave project engraves everything you add)
        it.op = self.default_op();
        if kind == ShapeKind::Line && it.op != Operation::Engrave {
            it.op = Operation::Score;
            it.color = [0x55, 0x55, 0x55];
        }
        self.push_and_select(it);
    }

    fn add_text_item(&mut self, text: &str, at: Pos2, color: Color32) {
        let mut it = Item::new(&format!("Text: {}", text.lines().next().unwrap_or("")), Vec::new(), at, vec2(1.0, 1.0), color);
        it.text = Some(text.into());
        it.text_opts = Some(TextOpts::default());
        regen_text(&mut it);
        if it.unit.is_empty() {
            self.status = "That text has no letters the typeface can draw.".into();
            return;
        }
        // one inch tall to start, top-left at `at`
        let k = 1.0 / it.natural[1].max(1e-4);
        it.size = [it.natural[0] * k, it.natural[1] * k];
        it.pos = [at.x, at.y];
        it.op = self.default_op();
        self.push_and_select(it);
    }

    /// Bring a file in: SVG or DXF become outlines; photos and drawings open the upload window.
    fn import_file(&mut self, input: &str) {
        let input = input.trim();
        if input.starts_with('<') {
            self.import_outlines("Uploaded image", geometry::parse_svg(input), geometry::svg_inches_per_unit(input).unwrap_or(0.0));
            return;
        }
        let path = input.trim_matches('"');
        let lower = path.to_lowercase();
        let bytes = match std::fs::read(path) {
            Ok(b) => b,
            Err(e) => {
                self.status = format!("Couldn't read that file: {e}");
                return;
            }
        };
        let name = std::path::Path::new(path).file_stem().map(|s| s.to_string_lossy().to_string()).unwrap_or_else(|| "Image".into());
        if lower.ends_with(".svg") {
            let text = String::from_utf8_lossy(&bytes);
            self.import_outlines(&name, geometry::parse_svg(&text), geometry::svg_inches_per_unit(&text).unwrap_or(0.0));
        } else if lower.ends_with(".dxf") {
            // DXF drawings are usually in millimetres
            self.import_outlines(&name, geometry::parse_dxf(&String::from_utf8_lossy(&bytes)), 1.0 / 25.4);
        } else {
            match imaging::load(&bytes) {
                Ok(pic) => {
                    let cleanup = Cleanup::default();
                    let mask = imaging::mask(&pic, &cleanup);
                    let width_in = 4.0;
                    self.image_import = Some(ImageImport {
                        name,
                        pic,
                        cleanup,
                        print: false,
                        engrave: self.project_type == ProjectType::Engrave,
                        lines_for_engrave: false,
                        width_in,
                        mask,
                        orig_tex: None,
                        preview_tex: None,
                        dirty: true,
                        lines: Vec::new(),
                        lines_for_print: false,
                        zoom: 1.0,
                        pan: Vec2::ZERO,
                        view: PreviewView::SideBySide,
                        show_lines: true,
                    });
                    self.status.clear();
                }
                Err(e) => self.status = e,
            }
        }
    }

    /// Open the computer's file picker for uploads (several files at once).
    fn browse_uploads(&mut self) {
        let picked = rfd::FileDialog::new()
            .set_title("Upload images")
            .add_filter("Images and drawings", &["svg", "dxf", "png", "jpg", "jpeg", "bmp", "gif", "webp"])
            .add_filter("All files", &["*"])
            .pick_files();
        if let Some(files) = picked {
            for f in files {
                self.upload_queue.push(f.display().to_string());
            }
            self.next_upload();
        }
    }

    /// Bring in queued uploads until one needs the picture window.
    fn next_upload(&mut self) {
        while self.image_import.is_none() && !self.upload_queue.is_empty() {
            let f = self.upload_queue.remove(0);
            self.import_file(&f);
        }
    }

    /// The SVG converter: pick files, write an SVG next to each one.
    fn convert_files_to_svg(&mut self) {
        let Some(files) = rfd::FileDialog::new()
            .set_title("Convert to SVG")
            .add_filter("Images and drawings", &["png", "jpg", "jpeg", "bmp", "gif", "webp", "dxf"])
            .pick_files()
        else {
            return;
        };
        let mut done = Vec::new();
        let mut failed = Vec::new();
        let mut folder: Option<std::path::PathBuf> = None;
        for f in files {
            let name = f.file_name().map(|n| n.to_string_lossy().to_string()).unwrap_or_default();
            match imaging::file_to_svg(&f) {
                Ok(svg) => {
                    let mut out = f.with_extension("svg");
                    if std::fs::write(&out, &svg).is_err() {
                        // the picture's folder can't be written to: ask once where to save
                        if folder.is_none() {
                            folder = rfd::FileDialog::new().set_title("Where should the SVG files go?").pick_folder();
                        }
                        match &folder {
                            Some(dir) => out = dir.join(out.file_name().unwrap_or_default()),
                            None => {
                                failed.push(format!("{name} (not saved)"));
                                continue;
                            }
                        }
                        if std::fs::write(&out, &svg).is_err() {
                            failed.push(format!("{name} (not saved)"));
                            continue;
                        }
                    }
                    done.push(out.display().to_string());
                }
                Err(e) => failed.push(format!("{name}: {e}")),
            }
        }
        self.status = match (done.len(), failed.is_empty()) {
            (0, _) => format!("Nothing converted. {}", failed.join("; ")),
            (n, true) => format!("Converted {n} file(s) to SVG: {}", done.join(", ")),
            (n, false) => format!("Converted {n} file(s). Not converted: {}", failed.join("; ")),
        };
    }

    /// Save the whole design (visible layers) as one SVG file.
    fn export_design_svg(&mut self) {
        let Some(path) = rfd::FileDialog::new()
            .set_title("Export design as SVG")
            .set_file_name(format!("{}.svg", self.project_name.replace(['/', '\\'], "_")))
            .add_filter("SVG", &["svg"])
            .save_file()
        else {
            return;
        };
        let size = vec2(12.0, 12.0).max(self.items.iter().filter(|i| i.visible).fold(Rect::NOTHING, |r, i| r.union(i.world_bounds())).max.to_vec2());
        let mut s = format!(
            "<?xml version=\"1.0\" encoding=\"UTF-8\"?>\n<svg xmlns=\"http://www.w3.org/2000/svg\" width=\"{w:.3}mm\" height=\"{h:.3}mm\" viewBox=\"0 0 {w:.3} {h:.3}\">\n",
            w = size.x * 25.4,
            h = size.y * 25.4
        );
        for it in self.items.iter().filter(|i| i.visible && i.op != Operation::Guide) {
            let c = it.color32();
            if let Some(png) = &it.image_png {
                let r = it.img_rect;
                let a = it.to_world(pos2(r[0], r[1]));
                s.push_str(&format!(
                    "  <image x=\"{:.3}\" y=\"{:.3}\" width=\"{:.3}\" height=\"{:.3}\" href=\"data:image/png;base64,{}\"/>\n",
                    a.x * 25.4,
                    a.y * 25.4,
                    it.size[0] * (r[2] - r[0]) * 25.4,
                    it.size[1] * (r[3] - r[1]) * 25.4,
                    png
                ));
            }
            let d: Vec<String> = it.world_contours().iter().map(|c| geometry::contour_to_d(c, 25.4)).collect();
            let fill = if it.op.filled() && it.image_png.is_none() { format!("#{:02x}{:02x}{:02x}", c.r(), c.g(), c.b()) } else { "none".into() };
            s.push_str(&format!(
                "  <path data-name=\"{}\" data-op=\"{}\" fill=\"{}\" fill-rule=\"evenodd\" stroke=\"#{:02x}{:02x}{:02x}\" stroke-width=\"0.2\" d=\"{}\"/>\n",
                it.name.replace('"', "'").replace('<', "").replace('&', "and"),
                it.op.label().to_lowercase().replace(' ', "-"),
                fill,
                c.r(),
                c.g(),
                c.b(),
                d.join(" ")
            ));
        }
        s.push_str("</svg>\n");
        self.status = match std::fs::write(&path, s) {
            Ok(()) => format!("Saved {}", path.display()),
            Err(e) => format!("Couldn't save: {e}"),
        };
    }

    /// Add outlines from a file. `inches_per_unit` is the file's real scale (0 when unknown);
    /// a drawing that would come in tinier than 0.1 in or bigger than 24 in is fitted to 4 in.
    fn import_outlines(&mut self, name: &str, contours: Vec<Contour>, inches_per_unit: f32) {
        if contours.is_empty() {
            self.status = "No shapes found in that file.".into();
            return;
        }
        self.snapshot();
        let (min, max) = geometry::bounds(contours.iter().flatten().copied());
        let ext = (max - min).max(Vec2::splat(1e-4));
        let real = ext.x.max(ext.y) * inches_per_unit;
        let s = if inches_per_unit > 0.0 && (0.1..=24.0).contains(&real) { inches_per_unit } else { 4.0 / ext.x.max(ext.y) };
        let place = self.place_point();
        let world: Vec<Contour> = contours.iter().map(|c| c.iter().map(|p| place + (*p - min) * s).collect()).collect();
        let c = self.next_color();
        if let Some(it) = Item::from_world(name, world, [c.r(), c.g(), c.b()], self.default_op()) {
            self.push_and_select(it);
        }
        self.status = format!("Imported {} outline(s).", contours.len());
    }

    fn finish_image_import(&mut self) {
        let Some(imp) = self.image_import.take() else { return };
        let scale = imp.width_in / imp.pic.w as f32;
        let place = self.place_point();
        self.snapshot();
        if imp.engrave {
            // photo engraving: the picture with its removed parts see-through, inside its frame
            let (w, h) = (imp.pic.w as f32, imp.pic.h as f32);
            let frame = vec![vec![pos2(0.0, 0.0), pos2(w, 0.0), pos2(w, h), pos2(0.0, h), pos2(0.0, 0.0)]];
            let world: Vec<Contour> = frame.iter().map(|c| c.iter().map(|p| place + p.to_vec2() * scale).collect()).collect();
            if let Some(mut it) = Item::from_world(&imp.name, world, [0x30, 0x34, 0x3a], Operation::Engrave) {
                let png = imaging::encode_png(imp.pic.w, imp.pic.h, &imaging::cleaned_rgba(&imp.pic, &imp.mask));
                it.image_png = Some(imaging::b64_encode(&png));
                it.img_rect = [0.0, 0.0, 1.0, 1.0];
                it.engrave.style = EngraveStyle::Photo;
                self.push_and_select(it);
            }
            return;
        }
        if imp.print {
            let outline = imaging::print_outline(&imp.pic, &imp.mask);
            let (min, max) = geometry::bounds(outline.iter().flatten().copied());
            let sz = (max - min).max(Vec2::splat(1e-3));
            let world: Vec<Contour> = outline.iter().map(|c| c.iter().map(|p| place + (*p - min) * scale).collect()).collect();
            if let Some(mut it) = Item::from_world(&imp.name, world, [0x30, 0x30, 0x30], Operation::Print) {
                let png = imaging::encode_png(imp.pic.w, imp.pic.h, &imaging::cleaned_rgba(&imp.pic, &imp.mask));
                it.image_png = Some(imaging::b64_encode(&png));
                it.img_rect = [(0.0 - min.x) / sz.x, (0.0 - min.y) / sz.y, (imp.pic.w as f32 - min.x) / sz.x, (imp.pic.h as f32 - min.y) / sz.y];
                self.push_and_select(it);
            }
        } else {
            let traced = imaging::trace(&imp.pic, &imp.mask, imp.cleanup.smooth);
            if traced.is_empty() {
                self.status = "Nothing was left to cut: try another background setting.".into();
                return;
            }
            // colour: the average of the kept pixels
            let (mut r, mut g, mut b, mut n) = (0u64, 0u64, 0u64, 0u64);
            for (i, k) in imp.mask.iter().enumerate() {
                if *k {
                    r += imp.pic.rgba[i * 4] as u64;
                    g += imp.pic.rgba[i * 4 + 1] as u64;
                    b += imp.pic.rgba[i * 4 + 2] as u64;
                    n += 1;
                }
            }
            let col = if n > 0 { [(r / n) as u8, (g / n) as u8, (b / n) as u8] } else { [0x33, 0x3a, 0x44] };
            let world: Vec<Contour> = traced.iter().map(|c| c.iter().map(|p| place + p.to_vec2() * scale).collect()).collect();
            if let Some(it) = Item::from_world(&imp.name, world, col, Operation::Cut) {
                self.push_and_select(it);
            }
        }
    }

    fn save_settings(&self) {
        let dir = config_dir();
        let _ = std::fs::create_dir_all(&dir);
        if let Ok(s) = serde_json::to_string_pretty(&self.settings) {
            let _ = std::fs::write(dir.join("machine.json"), s);
        }
    }

    fn mat_size(&self) -> Vec2 {
        // Settings > Load Type (12 x 12 and 12 x 24 are MAT_SIZES)
        self.prefs.load.size()
    }

    fn ensure_textures(&mut self, ctx: &egui::Context) {
        let live: HashSet<u64> = self.items.iter().map(|i| i.uid).collect();
        self.textures.retain(|k, _| live.contains(k));
        for it in &self.items {
            if self.textures.contains_key(&it.uid) {
                continue;
            }
            let Some(b64) = &it.image_png else { continue };
            let bytes = imaging::b64_decode(b64);
            if let Ok(img) = image::load_from_memory(&bytes) {
                let rgba = img.to_rgba8();
                let ci = egui::ColorImage::from_rgba_unmultiplied([rgba.width() as usize, rgba.height() as usize], rgba.as_raw());
                let tex = ctx.load_texture(format!("pic-{}", it.uid), ci, egui::TextureOptions::LINEAR);
                self.textures.insert(it.uid, tex);
            }
        }
    }

    // ------------------------------------------------ actions

    fn apply(&mut self, act: Act) {
        match act {
            Act::Undo => {
                if let Some(prev) = self.undo.pop() {
                    self.redo.push(std::mem::replace(&mut self.items, prev));
                    self.selection.clear();
                }
            }
            Act::Redo => {
                if let Some(next) = self.redo.pop() {
                    self.undo.push(std::mem::replace(&mut self.items, next));
                    self.selection.clear();
                }
            }
            Act::Copy | Act::Cut => {
                if self.selection.is_empty() {
                    return;
                }
                self.clipboard = self.selection.iter().map(|&i| self.items[i].clone()).collect();
                if act == Act::Cut {
                    self.apply(Act::Delete);
                }
            }
            Act::Paste => {
                if self.clipboard.is_empty() {
                    return;
                }
                self.snapshot();
                let mut regroup: HashMap<u32, u32> = HashMap::new();
                let start = self.items.len();
                for c in self.clipboard.clone() {
                    let mut it = c;
                    it.uid = new_uid();
                    it.pos[0] += 0.25;
                    it.pos[1] += 0.25;
                    if let Some(g) = it.group {
                        let ng = *regroup.entry(g).or_insert_with(|| {
                            self.next_group += 1;
                            self.next_group
                        });
                        it.group = Some(ng);
                    }
                    it.attach = None;
                    self.items.push(it);
                }
                // next paste lands a little further along
                for c in &mut self.clipboard {
                    c.pos[0] += 0.25;
                    c.pos[1] += 0.25;
                }
                self.selection = (start..self.items.len()).collect();
            }
            Act::Duplicate => {
                let saved = std::mem::take(&mut self.clipboard);
                self.apply(Act::Copy);
                self.apply(Act::Paste);
                self.clipboard = saved;
            }
            Act::Delete => {
                if self.selection.is_empty() {
                    return;
                }
                self.snapshot();
                let mut sel = self.selection.clone();
                sel.sort_unstable_by(|a, b| b.cmp(a));
                for i in sel {
                    if i < self.items.len() {
                        self.items.remove(i);
                    }
                }
                self.selection.clear();
            }
            Act::SelectAll => self.selection = (0..self.items.len()).collect(),
            Act::Deselect => self.selection.clear(),
            Act::Group => {
                if self.selection.len() < 2 {
                    return;
                }
                self.snapshot();
                self.next_group += 1;
                let g = self.next_group;
                for &i in &self.selection {
                    self.items[i].group = Some(g);
                }
            }
            Act::Ungroup => {
                self.snapshot();
                for &i in &self.selection {
                    self.items[i].group = None;
                }
            }
            Act::Align(how) => self.align(how),
            Act::Arrange(how) => self.arrange(how),
            Act::FlipH | Act::FlipV => self.flip(act == Act::FlipH),
            Act::Weld | Act::Combine(_) => {
                if self.selection.len() < 2 {
                    self.status = "Select two or more layers first.".into();
                    return;
                }
                self.snapshot();
                let sel = self.selection.clone(); // already bottom-to-top
                let base = &self.items[sel[0]];
                let (color, op) = (if act == Act::Weld { self.items[*sel.last().unwrap()].color } else { base.color }, base.op);
                let parts: Vec<Vec<Contour>> = sel.iter().map(|&i| self.items[i].world_contours()).collect();
                let result = match act {
                    Act::Weld | Act::Combine(Combine::Unite) => geometry::weld(&parts),
                    Act::Combine(how) => parts[1..].iter().fold(parts[0].clone(), |acc, p| geometry::combine(&acc, p, how)),
                    _ => unreachable!(),
                };
                let name = match act {
                    Act::Weld => "Weld",
                    Act::Combine(Combine::Unite) => "Unite",
                    Act::Combine(Combine::Subtract) => "Subtract",
                    Act::Combine(Combine::Intersect) => "Intersect",
                    _ => "Exclude",
                };
                let at = sel[0];
                for &i in sel.iter().rev() {
                    self.items.remove(i);
                }
                match Item::from_world(name, result, color, op) {
                    Some(it) => {
                        self.items.insert(at, it);
                        self.selection = vec![at];
                    }
                    None => {
                        self.selection.clear();
                        self.status = "Those shapes don't overlap, so nothing was left.".into();
                    }
                }
            }
            Act::Slice => {
                if self.selection.len() != 2 {
                    self.status = "Slice needs exactly two layers selected.".into();
                    return;
                }
                self.snapshot();
                let (a, b) = (self.selection[0], self.selection[1]);
                let (ia, ib) = (self.items[a].clone(), self.items[b].clone());
                let pieces = geometry::slice(&ia.world_contours(), &ib.world_contours());
                self.items.remove(b);
                self.items.remove(a);
                let mid = [
                    ((ia.color[0] as u16 + ib.color[0] as u16) / 2) as u8,
                    ((ia.color[1] as u16 + ib.color[1] as u16) / 2) as u8,
                    ((ia.color[2] as u16 + ib.color[2] as u16) / 2) as u8,
                ];
                let mut new_sel = Vec::new();
                for (n, (cs, col)) in pieces.into_iter().zip([ia.color, mid, ib.color]).enumerate() {
                    if let Some(it) = Item::from_world(&format!("Slice result {}", n + 1), cs, col, ia.op) {
                        self.items.insert(a.min(self.items.len()), it);
                        new_sel.push(a.min(self.items.len() - 1));
                    }
                }
                self.selection.clear();
            }
            Act::Attach | Act::Flatten => {
                if self.selection.len() < 2 && act == Act::Attach {
                    self.status = "Select two or more layers to attach.".into();
                    return;
                }
                self.snapshot();
                self.next_group += 1;
                let g = self.next_group;
                for &i in &self.selection {
                    self.items[i].attach = Some(g);
                    if act == Act::Flatten {
                        self.items[i].op = Operation::Print;
                    }
                }
            }
            Act::Detach | Act::Unflatten => {
                self.snapshot();
                for &i in &self.selection {
                    self.items[i].attach = None;
                    if act == Act::Unflatten && self.items[i].op == Operation::Print && self.items[i].image_png.is_none() {
                        self.items[i].op = Operation::Cut;
                    }
                }
            }
            Act::Offset => {
                if self.selection.is_empty() {
                    return;
                }
                self.offset_tool = Some(OffsetTool { distance: 0.125 });
            }
            Act::Contour => self.contour_item = self.sel_one(),
            Act::UngroupLetters => self.ungroup_letters(),
            Act::Nudge(dx, dy) => {
                if self.selection.is_empty() {
                    return;
                }
                self.snapshot();
                for &i in &self.selection {
                    self.items[i].pos[0] += dx;
                    self.items[i].pos[1] += dy;
                }
            }
        }
    }

    fn align(&mut self, how: AlignHow) {
        if self.selection.len() < 2 {
            self.status = "Select two or more layers to align them.".into();
            return;
        }
        self.snapshot();
        let all = self.selection_bounds();
        let sel = self.selection.clone();
        let bounds: Vec<Rect> = sel.iter().map(|&i| self.items[i].world_bounds()).collect();
        let mut shifts: Vec<Vec2> = vec![Vec2::ZERO; sel.len()];
        match how {
            AlignHow::Left => bounds.iter().zip(shifts.iter_mut()).for_each(|(b, s)| s.x = all.left() - b.left()),
            AlignHow::Right => bounds.iter().zip(shifts.iter_mut()).for_each(|(b, s)| s.x = all.right() - b.right()),
            AlignHow::CenterH => bounds.iter().zip(shifts.iter_mut()).for_each(|(b, s)| s.x = all.center().x - b.center().x),
            AlignHow::Top => bounds.iter().zip(shifts.iter_mut()).for_each(|(b, s)| s.y = all.top() - b.top()),
            AlignHow::Bottom => bounds.iter().zip(shifts.iter_mut()).for_each(|(b, s)| s.y = all.bottom() - b.bottom()),
            AlignHow::CenterV => bounds.iter().zip(shifts.iter_mut()).for_each(|(b, s)| s.y = all.center().y - b.center().y),
            AlignHow::Center => bounds.iter().zip(shifts.iter_mut()).for_each(|(b, s)| *s = all.center() - b.center()),
            AlignHow::DistributeH | AlignHow::DistributeV => {
                let horiz = how == AlignHow::DistributeH;
                let mut order: Vec<usize> = (0..sel.len()).collect();
                order.sort_by(|&a, &b| {
                    let (ka, kb) = if horiz { (bounds[a].center().x, bounds[b].center().x) } else { (bounds[a].center().y, bounds[b].center().y) };
                    ka.partial_cmp(&kb).unwrap_or(std::cmp::Ordering::Equal)
                });
                let total: f32 = bounds.iter().map(|b| if horiz { b.width() } else { b.height() }).sum();
                let span = if horiz { all.width() } else { all.height() };
                let gap = (span - total) / (sel.len() as f32 - 1.0).max(1.0);
                let mut cursor = if horiz { all.left() } else { all.top() };
                for &k in &order {
                    let b = bounds[k];
                    if horiz {
                        shifts[k].x = cursor - b.left();
                        cursor += b.width() + gap;
                    } else {
                        shifts[k].y = cursor - b.top();
                        cursor += b.height() + gap;
                    }
                }
            }
        }
        for (k, &i) in sel.iter().enumerate() {
            self.items[i].pos[0] += shifts[k].x;
            self.items[i].pos[1] += shifts[k].y;
        }
    }

    fn arrange(&mut self, how: Arrange) {
        if self.selection.is_empty() {
            return;
        }
        self.snapshot();
        let uids = self.selected_uids();
        let is_sel = |it: &Item| uids.contains(&it.uid);
        match how {
            Arrange::Front | Arrange::Back => {
                let (sel, rest): (Vec<Item>, Vec<Item>) = std::mem::take(&mut self.items).into_iter().partition(|it| is_sel(it));
                self.items = if how == Arrange::Front { rest.into_iter().chain(sel).collect() } else { sel.into_iter().chain(rest).collect() };
            }
            Arrange::Forward => {
                for i in (0..self.items.len().saturating_sub(1)).rev() {
                    if is_sel(&self.items[i]) && !is_sel(&self.items[i + 1]) {
                        self.items.swap(i, i + 1);
                    }
                }
            }
            Arrange::Backward => {
                for i in 1..self.items.len() {
                    if is_sel(&self.items[i]) && !is_sel(&self.items[i - 1]) {
                        self.items.swap(i, i - 1);
                    }
                }
            }
        }
        self.reselect_uids(&uids);
    }

    fn flip(&mut self, horizontal: bool) {
        if self.selection.is_empty() {
            return;
        }
        self.snapshot();
        let c = self.selection_bounds().center();
        let multi = self.selection.len() > 1;
        for &i in &self.selection.clone() {
            let it = &mut self.items[i];
            for cont in &mut it.unit {
                for p in cont.iter_mut() {
                    if horizontal {
                        p.x = 1.0 - p.x;
                    } else {
                        p.y = 1.0 - p.y;
                    }
                }
            }
            let r = it.img_rect;
            if horizontal {
                it.img_rect = [1.0 - r[2], r[1], 1.0 - r[0], r[3]];
                it.img_flip[0] = !it.img_flip[0];
            } else {
                it.img_rect = [r[0], 1.0 - r[3], r[2], 1.0 - r[1]];
                it.img_flip[1] = !it.img_flip[1];
            }
            it.rot = -it.rot;
            if multi {
                let mut ic = it.center();
                if horizontal {
                    ic.x = 2.0 * c.x - ic.x;
                } else {
                    ic.y = 2.0 * c.y - ic.y;
                }
                it.set_center(ic);
            }
        }
    }

    /// Keep the canvas view on the design: the 12 x 12 in area plus anything placed outside it,
    /// with an inch of room around. When that's smaller than the view it can't leave the view;
    /// when it's bigger, the view can't leave it.
    fn clamp_pan(&mut self, view: Vec2) {
        let ppi = BASE_PPI * self.zoom;
        let content = self
            .items
            .iter()
            .filter(|i| i.visible)
            .fold(Rect::from_min_max(Pos2::ZERO, pos2(12.0, 12.0)), |r, it| r.union(it.world_bounds()))
            .expand(1.0);
        self.pan = clamped_pan(self.pan, content, vec2(RULER, RULER), view, ppi);
    }

    // ------------------------------------------------ typing on the canvas

    fn editing_index(&self) -> Option<usize> {
        let uid = self.text_edit.as_ref()?.uid;
        self.items.iter().position(|it| it.uid == uid)
    }

    /// Start typing into a text layer, with the cursor at the end of its text.
    fn start_text_edit(&mut self, i: usize, now: f64) {
        if !self.items.get(i).map(|it| it.is_text()).unwrap_or(false) {
            return;
        }
        self.finish_text_edit();
        self.snapshot(); // the whole typing session is one undo step
        let caret = self.items[i].text.as_deref().map(|t| t.chars().count()).unwrap_or(0);
        self.text_edit = Some(TextEditing { uid: self.items[i].uid, caret, since: now });
        self.selection = vec![i];
        self.drawer = if self.drawer == Drawer::Text { Drawer::None } else { self.drawer };
    }

    /// Stop typing; a text layer left empty is removed.
    fn finish_text_edit(&mut self) {
        let Some(i) = self.editing_index() else {
            self.text_edit = None;
            return;
        };
        self.text_edit = None;
        if self.items[i].text.as_deref().map(|t| t.trim().is_empty()).unwrap_or(false) {
            self.items.remove(i);
            self.selection.clear();
        }
    }

    /// Apply this frame's keys and typed characters to the text being edited.
    fn type_into_text(&mut self, ctx: &egui::Context) {
        let Some(i) = self.editing_index() else {
            self.text_edit = None;
            return;
        };
        let now = ctx.input(|inp| inp.time);
        let events = ctx.input(|inp| inp.events.clone());
        let mut chars: Vec<char> = self.items[i].text.clone().unwrap_or_default().chars().collect();
        let mut caret = self.text_edit.as_ref().map(|t| t.caret).unwrap_or(0).min(chars.len());
        let before = chars.clone();
        let (moved, finish) = apply_typing(&mut chars, &mut caret, &events);
        if chars != before {
            let text: String = chars.iter().collect();
            let it = &mut self.items[i];
            it.name = format!("Text: {}", text.lines().next().unwrap_or(""));
            it.text = Some(text);
            regen_text(it);
        }
        if chars != before || moved {
            if let Some(t) = self.text_edit.as_mut() {
                t.since = now;
            }
        }
        if let Some(t) = self.text_edit.as_mut() {
            t.caret = caret;
        }
        if finish {
            self.finish_text_edit();
        }
    }
}

/// Lay pieces (their sizes, in inches) out on a mat in rows from the top-left corner, keeping
/// `gap` between pieces and from the edges. Returns each piece's top-left corner, and whether
/// anything doesn't fit inside `limit`.
pub(crate) fn row_layout(sizes: &[Vec2], limit: Vec2, gap: f32) -> (Vec<Pos2>, bool) {
    let (mut x, mut y, mut row_h) = (gap, gap, 0.0f32);
    let mut overflow = false;
    let mut out = Vec::with_capacity(sizes.len());
    for s in sizes {
        if x + s.x > limit.x - gap && x > gap {
            x = gap;
            y += row_h + gap;
            row_h = 0.0;
        }
        if y + s.y > limit.y - gap || s.x > limit.x - 2.0 * gap {
            overflow = true;
        }
        out.push(pos2(x, y));
        x += s.x + gap;
        row_h = row_h.max(s.y);
    }
    (out, overflow)
}

/// Mat sizes offered, in inches.
pub(crate) const MAT_SIZES: [Vec2; 2] = [vec2(12.0, 12.0), vec2(12.0, 24.0)];

/// Pieces kept this far apart on a mat (inches), and screen pixels per inch on the canvas at 100 %.
pub(crate) const LAYOUT_GAP: f32 = MAT_GAP;
pub(crate) const CANVAS_PPI: f32 = BASE_PPI;

/// The pan that keeps `content` (design inches) and the view region `lo..hi` (pixels from the
/// canvas corner) overlapping as much as they can.
fn clamped_pan(pan: Vec2, content: Rect, lo: Vec2, hi: Vec2, ppi: f32) -> Vec2 {
    let mut out = pan;
    for axis in 0..2 {
        let a = lo[axis] - content.min[axis] * ppi; // content start lines up with the view start
        let b = hi[axis] - content.max[axis] * ppi; // content end lines up with the view end
        let (mn, mx) = if a < b { (a, b) } else { (b, a) };
        out[axis] = out[axis].clamp(mn, mx);
    }
    out
}

/// The typing rules for text on the canvas: characters, paste, Backspace (Ctrl for a word),
/// Delete, Enter, arrows, Home/End, Ctrl+A to the end, Esc to finish.
/// Returns (cursor moved, finished).
fn apply_typing(chars: &mut Vec<char>, caret_io: &mut usize, events: &[egui::Event]) -> (bool, bool) {
    let mut caret = (*caret_io).min(chars.len());
    let mut moved = false;
    let mut finish = false;
    let line_start = |chars: &[char], c: usize| chars[..c].iter().rposition(|ch| *ch == '\n').map(|p| p + 1).unwrap_or(0);
    let line_end = |chars: &[char], c: usize| chars[c..].iter().position(|ch| *ch == '\n').map(|p| c + p).unwrap_or(chars.len());
    {
        for e in events {
            match e {
                egui::Event::Text(t) => {
                    for ch in t.chars().filter(|c| !c.is_control()) {
                        chars.insert(caret, ch);
                        caret += 1;
                    }
                }
                egui::Event::Paste(t) => {
                    for ch in t.chars().filter(|c| *c == '\n' || !c.is_control()) {
                        chars.insert(caret, ch);
                        caret += 1;
                    }
                }
                egui::Event::Key { key, pressed: true, modifiers, .. } => match key {
                    Key::Backspace => {
                        if modifiers.command {
                            while caret > 0 && chars[caret - 1] == ' ' {
                                caret -= 1;
                                chars.remove(caret);
                            }
                            while caret > 0 && !chars[caret - 1].is_whitespace() {
                                caret -= 1;
                                chars.remove(caret);
                            }
                        } else if caret > 0 {
                            caret -= 1;
                            chars.remove(caret);
                        }
                    }
                    Key::Delete => {
                        if caret < chars.len() {
                            chars.remove(caret);
                        }
                    }
                    Key::Enter => {
                        chars.insert(caret, '\n');
                        caret += 1;
                    }
                    Key::ArrowLeft => {
                        caret = caret.saturating_sub(1);
                        moved = true;
                    }
                    Key::ArrowRight => {
                        caret = (caret + 1).min(chars.len());
                        moved = true;
                    }
                    Key::Home => {
                        caret = line_start(&chars[..], caret);
                        moved = true;
                    }
                    Key::End => {
                        caret = line_end(&chars[..], caret);
                        moved = true;
                    }
                    Key::ArrowUp | Key::ArrowDown => {
                        let col = caret - line_start(&chars[..], caret);
                        if *key == Key::ArrowUp {
                            let ls = line_start(&chars[..], caret);
                            if ls > 0 {
                                let prev_start = line_start(&chars[..], ls - 1);
                                caret = (prev_start + col).min(ls - 1);
                            } else {
                                caret = 0;
                            }
                        } else {
                            let le = line_end(&chars[..], caret);
                            if le < chars.len() {
                                let next_start = le + 1;
                                caret = (next_start + col).min(line_end(&chars[..], next_start));
                            } else {
                                caret = chars.len();
                            }
                        }
                        moved = true;
                    }
                    Key::Escape => finish = true,
                    Key::A if modifiers.command => {
                        caret = chars.len();
                        moved = true;
                    }
                    _ => {}
                },
                _ => {}
            }
        }
    }
    *caret_io = caret;
    (moved, finish)
}

impl App {
    fn ungroup_letters(&mut self) {
        let Some(i) = self.sel_one() else { return };
        let it = self.items[i].clone();
        let Some(text) = &it.text else { return };
        let opts = it.text_opts.clone().unwrap_or_default();
        let fd = fonts::font_data(&opts.font, opts.bold, opts.italic);
        let glyphs = geometry::layout_text(text, &fd.0, fd.1, &opts);
        let all: Vec<Contour> = glyphs.iter().flatten().cloned().collect();
        if all.is_empty() {
            return;
        }
        let (min, max) = geometry::bounds(all.iter().flatten().copied());
        let nat = (max - min).max(Vec2::splat(1e-4));
        self.snapshot();
        self.items.remove(i);
        let letters: Vec<char> = text.chars().filter(|c| !c.is_whitespace()).collect();
        let start = self.items.len();
        for (n, g) in glyphs.into_iter().enumerate() {
            let world: Vec<Contour> = g.iter().map(|c| c.iter().map(|p| it.to_world(pos2((p.x - min.x) / nat.x, (p.y - min.y) / nat.y))).collect()).collect();
            let label = letters.get(n).map(|c| format!("Letter {c}")).unwrap_or_else(|| format!("Letter {}", n + 1));
            if let Some(mut li) = Item::from_world(&label, world, it.color, it.op) {
                li.group = None;
                self.items.push(li);
            }
        }
        self.selection = (start..self.items.len()).collect();
    }

    // ------------------------------------------------ mats

    /// Group visible pieces into mats: one per cut colour, plus one for print then cut. Attached
    /// layers keep their places relative to each other; the rest is laid out in rows.
    fn build_mats(&self) -> Vec<Mat> {
        #[derive(PartialEq, Clone, Copy)]
        enum Key {
            Cut(Color32),
            Print,
        }
        let size = self.mat_size();
        let live = |i: usize| self.items[i].visible && self.items[i].op != Operation::Guide;
        // blocks of items that move together
        let mut blocks: Vec<(Option<Key>, Vec<usize>)> = Vec::new();
        let mut seen_attach: HashMap<u32, usize> = HashMap::new();
        for i in (0..self.items.len()).filter(|&i| live(i)) {
            let it = &self.items[i];
            let key = match it.op {
                Operation::Cut => Some(Key::Cut(it.color32())),
                Operation::Print => Some(Key::Print),
                _ => None,
            };
            if let Some(a) = it.attach {
                if let Some(&b) = seen_attach.get(&a) {
                    blocks[b].1.push(i);
                    if blocks[b].0.is_none() || key == Some(Key::Print) {
                        blocks[b].0 = key.or(blocks[b].0);
                    }
                    continue;
                }
                seen_attach.insert(a, blocks.len());
            }
            blocks.push((key, vec![i]));
        }
        let mut keys: Vec<Key> = Vec::new();
        for (k, _) in &blocks {
            if let Some(k) = k {
                if !keys.contains(k) {
                    keys.push(*k);
                }
            }
        }
        let first = keys.first().copied();
        let mut mats: Vec<Mat> = Vec::new();
        for key in keys.iter().copied() {
            let (kind, color, limit) = match key {
                Key::Cut(c) => (MatKind::Cut, c, size),
                Key::Print => (MatKind::Print, Color32::from_rgb(0xe6, 0x51, 0x00), self.print_area()),
            };
            let mut mat = Mat { kind, color, pieces: Vec::new(), overflow: false };
            let mine: Vec<&Vec<usize>> = blocks
                .iter()
                .filter(|(bk, _)| match bk {
                    Some(k) => *k == key,
                    None => Some(key) == first,
                })
                .map(|(_, ids)| ids)
                .collect();
            let bounds: Vec<Rect> = mine.iter().map(|ids| ids.iter().fold(Rect::NOTHING, |r, &i| r.union(self.items[i].world_bounds()))).collect();
            let sizes: Vec<Vec2> = bounds.iter().map(|b| b.size()).collect();
            if self.project_type.keeps_layout() {
                // projects with a design area keep every layer where it sits on the canvas, so
                // the material can be placed on the mat to match the screen (Engrave, cards...)
                for (ids, b) in mine.iter().zip(&bounds) {
                    if b.left() < 0.0 || b.top() < 0.0 || b.right() > limit.x || b.bottom() > limit.y {
                        mat.overflow = true;
                    }
                    for &i in ids.iter() {
                        mat.pieces.push((i, Vec2::ZERO));
                    }
                }
            } else {
                let (spots, overflow) = row_layout(&sizes, limit, MAT_GAP);
                mat.overflow = overflow;
                for ((ids, b), spot) in mine.iter().zip(&bounds).zip(spots) {
                    let off = spot.to_vec2() - b.min.to_vec2();
                    for &i in ids.iter() {
                        mat.pieces.push((i, off));
                    }
                }
            }
            mats.push(mat);
        }
        if mats.is_empty() && blocks.iter().any(|(k, _)| k.is_none()) {
            let mut mat = Mat { kind: MatKind::Cut, color: Color32::from_gray(200), pieces: Vec::new(), overflow: false };
            for (_, ids) in &blocks {
                for &i in ids {
                    mat.pieces.push((i, Vec2::ZERO));
                }
            }
            mats.push(mat);
        }
        mats
    }

    /// The paths of one mat in millimetres, as SVG path data: the same form the original hands its
    /// machine process (one path_data per mat).
    fn mat_svg(&self, mat: &Mat) -> String {
        let size = if mat.kind == MatKind::Print { self.print_area() } else { self.mat_size() };
        let mut s = format!(
            "<svg xmlns=\"http://www.w3.org/2000/svg\" width=\"{w}mm\" height=\"{h}mm\" viewBox=\"0 0 {w} {h}\">\n",
            w = size.x * 25.4,
            h = size.y * 25.4
        );
        for (i, off) in &mat.pieces {
            let it = &self.items[*i];
            let c = it.color32();
            if let (Some(png), MatKind::Print) = (&it.image_png, mat.kind) {
                let r = it.img_rect;
                let a = it.to_world(pos2(r[0], r[1])) + *off;
                let w = it.size[0] * (r[2] - r[0]) * 25.4;
                let h = it.size[1] * (r[3] - r[1]) * 25.4;
                s.push_str(&format!(
                    "  <image x=\"{:.3}\" y=\"{:.3}\" width=\"{:.3}\" height=\"{:.3}\" href=\"data:image/png;base64,{}\"/>\n",
                    a.x * 25.4,
                    a.y * 25.4,
                    w,
                    h,
                    png
                ));
            }
            // filled and photo engraving: the engraved lines, as one path
            if let (Operation::Engrave, Some((_, lines))) = (it.op, self.engrave_cache.get(&it.uid)) {
                let mut d = String::new();
                for mut l in engrave_to_world(it, lines) {
                    for p in l.iter_mut() {
                        *p += *off;
                        if self.mirror {
                            p.x = size.x - p.x;
                        }
                    }
                    d.push_str(&geometry::contour_to_d(&l, 25.4));
                    d.push(' ');
                }
                s.push_str(&format!(
                    "  <path data-op=\"engrave-{}\" fill=\"none\" stroke=\"#{:02x}{:02x}{:02x}\" stroke-width=\"0.1\" d=\"{}\"/>\n",
                    it.engrave.style.label().to_lowercase(),
                    c.r(),
                    c.g(),
                    c.b(),
                    d.trim_end()
                ));
                if it.engrave.style == EngraveStyle::Photo {
                    continue; // a photo's frame isn't engraved
                }
            }
            for mut contour in it.world_contours() {
                for p in contour.iter_mut() {
                    *p += *off;
                    if self.mirror {
                        p.x = size.x - p.x;
                    }
                }
                let fill = if it.op == Operation::Print && it.image_png.is_none() {
                    format!("#{:02x}{:02x}{:02x}", c.r(), c.g(), c.b())
                } else {
                    "none".into()
                };
                s.push_str(&format!(
                    "  <path data-op=\"{}\" fill=\"{}\" stroke=\"#{:02x}{:02x}{:02x}\" stroke-width=\"0.2\" d=\"{}\"/>\n",
                    it.op.label().to_lowercase().replace(' ', "-"),
                    fill,
                    c.r(),
                    c.g(),
                    c.b(),
                    geometry::contour_to_d(&contour, 25.4)
                ));
            }
        }
        s.push_str("</svg>\n");
        s
    }

    /// Add a line to the in-app machine log and also append it to a file on disk, so a cut's
    /// traffic can be read back without copying from the (non-selectable) log panel. The file is
    /// `<config>/machine-log.txt`; its path is shown under the log in the Machine panel.
    fn log_machine(&mut self, line: String) {
        let secs = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map(|d| d.as_secs())
            .unwrap_or(0)
            % 86_400;
        let stamp = format!("{:02}:{:02}:{:02}", secs / 3600, (secs % 3600) / 60, secs % 60);
        use std::io::Write as _;
        if let Ok(mut f) = std::fs::OpenOptions::new().create(true).append(true).open(config_dir().join("machine-log.txt")) {
            let _ = writeln!(f, "{stamp} {line}");
        }
        self.log.push(line);
    }

    fn pump_machine(&mut self) {
        while let Ok(ev) = self.machine.rx.try_recv() {
            match ev {
                Event::Log(s) => self.log_machine(s),
                #[cfg(target_os = "linux")]
                Event::PairFinished(message) => {
                    self.pairing = false;
                    self.log_machine(message.clone());
                    self.pairing_result = Some(message);
                }
                Event::Connected(n) => {
                    self.log_machine(format!("Connected to {n}."));
                    self.connected = Some(n);
                    // Read the machine's pressure calibration so grams map to stable counts.
                    let _ = self.machine.tx.send(Request::ReadCalibration);
                }
                Event::Calibration { rows } => {
                    self.calibration = rows;
                }
                Event::Disconnected(why) => {
                    self.log_machine(why);
                    self.connected = None;
                }
                Event::Reply { cmd, status, .. } => self.last_status = Some((cmd, status)),
            }
        }
        if self.log.len() > 400 {
            let n = self.log.len() - 400;
            self.log.drain(..n);
        }
    }

    fn connect(&mut self) {
        #[cfg(target_os = "linux")]
        if self.pairing {
            return;
        }
        let link = match self.settings.link {
            LinkKind::Bluetooth => Link::Rfcomm { mac: self.settings.mac.clone(), channel: self.settings.channel },
            LinkKind::Usb => Link::Usb { port: self.settings.usb_port.clone() },
            LinkKind::Device => Link::Device { path: self.settings.device_path.clone() },
            LinkKind::Demo => Link::Demo,
        };
        let key = machine::parse_key(&self.settings.key_hex);
        if !self.settings.key_hex.trim().is_empty() && key.is_none() {
            self.log.push("The session key should be 64 hex characters; sending unencrypted instead.".into());
        }
        self.save_settings();
        let _ = self.machine.tx.send(Request::Connect { link, key });
    }

    fn send(&mut self, cmd: Cmd) {
        let _ = self.machine.tx.send(Request::Send { cmd, theta: self.theta });
    }

    /// The design's tool paths as machine draw records (inches on the mat): a travel move to each
    /// contour's start, then cut lines along it. Visible items only; Guides are never sent, and
    /// filled ops are closed back to their start.
    fn machine_cut_records(&self) -> Vec<machine::DrawRecord> {
        let mut records = Vec::new();
        for it in self.items.iter().filter(|i| i.visible && i.op != Operation::Guide) {
            let contours: Vec<Vec<(f32, f32)>> = it
                .world_contours()
                .iter()
                .map(|c| c.iter().map(|p| (p.x, p.y)).collect())
                .collect();
            records.extend(machine::contours_to_records(&contours, it.op.filled()));
        }
        records
    }

    fn cut_on_machine(&mut self) {
        let records = self.machine_cut_records();
        if records.is_empty() {
            self.log.push("Nothing to cut: add a visible Cut, Draw or Score item first.".into());
            return;
        }
        // Work area = the mat size the original sends (its job size, default 12×12 in), not the
        // design's extent — the firmware uses this for the cut, and 2×2 was a parity mismatch.
        let width_in = 12u8;
        let height_in = 12u8;
        // Prefer grams via the machine's calibration; fall back to the raw count if it's unread.
        let pressure = if self.calibration.is_empty() {
            self.settings.pressure
        } else {
            machine::grams_to_counts(&self.calibration, self.settings.pressure_grams).unwrap_or(0)
        };
        let setup = machine::CutSetup { width_in, height_in, pressure };
        let _ = self.machine.tx.send(Request::Cut { records, setup });
    }
}

// ---------------------------------------------------------------- drawing helpers

/// Fill outlines with the even-odd rule (holes stay open) by scanning rows of the screen.
fn fill_even_odd(painter: &egui::Painter, contours: &[Vec<Pos2>], color: Color32) {
    let (min, max) = geometry::bounds(contours.iter().flatten().copied());
    if max.y - min.y < 0.5 {
        return;
    }
    let clip = painter.clip_rect();
    let mut mesh = Mesh::default();
    let mut y = min.y.max(clip.top() - 1.0).floor() + 0.5;
    let y_end = max.y.min(clip.bottom() + 1.0);
    let mut xs: Vec<f32> = Vec::new();
    while y < y_end {
        xs.clear();
        for c in contours {
            let n = c.len();
            if n < 2 {
                continue;
            }
            for k in 0..n {
                let a = c[k];
                let b = c[(k + 1) % n];
                if (a.y <= y && b.y > y) || (b.y <= y && a.y > y) {
                    xs.push(a.x + (y - a.y) / (b.y - a.y) * (b.x - a.x));
                }
            }
        }
        xs.sort_by(|a, b| a.partial_cmp(b).unwrap_or(std::cmp::Ordering::Equal));
        for pair in xs.chunks(2) {
            if pair.len() == 2 && pair[1] > pair[0] {
                mesh.add_colored_rect(Rect::from_min_max(pos2(pair[0], y - 0.5), pos2(pair[1], y + 0.5)), color);
            }
        }
        y += 1.0;
    }
    painter.add(Shape::mesh(mesh));
}

fn draw_item(
    painter: &egui::Painter,
    it: &Item,
    to_screen: &dyn Fn(Pos2) -> Pos2,
    offset: Vec2,
    mirror_w: Option<f32>,
    tex: Option<egui::TextureId>,
    engrave: Option<&[Contour]>,
) {
    let map = |p: Pos2| {
        let mut q = p + offset;
        if let Some(w) = mirror_w {
            q.x = w - q.x;
        }
        to_screen(q)
    };
    let contours: Vec<Vec<Pos2>> = it.world_contours().into_iter().map(|c| c.into_iter().map(map).collect()).collect();
    let color = it.color32();
    // filled or photo engraving: the engraved lines themselves, inside a faint outline
    if let (Operation::Engrave, Some(lines)) = (it.op, engrave) {
        let clip = painter.clip_rect().expand(4.0);
        let stroke = Stroke::new(1.0, color);
        for l in engrave_to_world(it, lines) {
            let pts: Vec<Pos2> = l.into_iter().map(map).collect();
            if pts.len() >= 2 && (clip.contains(pts[0]) || clip.contains(pts[pts.len() - 1])) {
                if pts.len() == 2 {
                    painter.line_segment([pts[0], pts[1]], stroke);
                } else {
                    painter.add(Shape::line(pts, stroke));
                }
            }
        }
        for c in &contours {
            painter.add(Shape::line(c.clone(), Stroke::new(0.8, color.gamma_multiply(0.35))));
        }
        return;
    }
    // a printed picture is drawn under its cut line
    if let (Some(t), Some(_)) = (tex, &it.image_png) {
        let r = it.img_rect;
        let corners = [pos2(r[0], r[1]), pos2(r[2], r[1]), pos2(r[2], r[3]), pos2(r[0], r[3])].map(|u| map(it.to_world(u)));
        let (fx, fy) = (it.img_flip[0], it.img_flip[1]);
        let uv = |u: f32, v: f32| pos2(if fx { 1.0 - u } else { u }, if fy { 1.0 - v } else { v });
        let uvs = [uv(0.0, 0.0), uv(1.0, 0.0), uv(1.0, 1.0), uv(0.0, 1.0)];
        let mut mesh = Mesh::with_texture(t);
        for k in 0..4 {
            mesh.vertices.push(egui::epaint::Vertex { pos: corners[k], uv: uvs[k], color: Color32::WHITE });
        }
        mesh.add_triangle(0, 1, 2);
        mesh.add_triangle(0, 2, 3);
        painter.add(Shape::mesh(mesh));
        for c in &contours {
            painter.add(Shape::line(c.clone(), Stroke::new(1.0, Color32::from_gray(90))));
        }
        return;
    }
    match it.op {
        Operation::Cut | Operation::Print => {
            fill_even_odd(painter, &contours, color);
            for c in &contours {
                painter.add(Shape::line(c.clone(), Stroke::new(1.0, color.gamma_multiply(0.55))));
            }
        }
        Operation::Wave | Operation::Perforate => {
            fill_even_odd(painter, &contours, color.gamma_multiply(0.85));
            for c in &contours {
                let (dash, gap) = if it.op == Operation::Perforate { (3.0, 3.0) } else { (8.0, 2.0) };
                painter.extend(Shape::dashed_line(c, Stroke::new(1.4, color.gamma_multiply(0.5)), dash, gap));
            }
        }
        Operation::Draw | Operation::Engrave | Operation::Deboss => {
            let w = if it.op == Operation::Deboss { 2.4 } else { 1.6 };
            for c in &contours {
                painter.add(Shape::line(c.clone(), Stroke::new(w, color)));
            }
        }
        Operation::Score => {
            for c in &contours {
                painter.extend(Shape::dashed_line(c, Stroke::new(1.6, color), 6.0, 4.0));
            }
        }
        Operation::Guide => {
            for c in &contours {
                painter.extend(Shape::dashed_line(c, Stroke::new(1.0, color.gamma_multiply(0.7)), 3.0, 3.0));
            }
        }
    }
}

/// A small line-drawn icon.
fn draw_icon(p: &egui::Painter, c: Pos2, which: &str, col: Color32) {
    let s = Stroke::new(1.8, col);
    match which {
        "New" => {
            p.rect_stroke(Rect::from_center_size(c, vec2(14.0, 18.0)), 2.0, s, StrokeKind::Middle);
            p.line_segment([c - vec2(4.0, 0.0), c + vec2(4.0, 0.0)], s);
            p.line_segment([c - vec2(0.0, 4.0), c + vec2(0.0, 4.0)], s);
        }
        "Shapes" => {
            p.rect_stroke(Rect::from_center_size(c + vec2(-3.0, 3.0), vec2(11.0, 11.0)), 2.0, s, StrokeKind::Middle);
            p.circle_stroke(c + vec2(4.0, -3.0), 5.5, s);
        }
        "Text" => {
            p.line_segment([c + vec2(-7.0, -7.0), c + vec2(7.0, -7.0)], s);
            p.line_segment([c + vec2(0.0, -7.0), c + vec2(0.0, 8.0)], s);
            p.line_segment([c + vec2(-3.0, 8.0), c + vec2(3.0, 8.0)], s);
        }
        "Upload" => {
            p.line_segment([c + vec2(0.0, 5.0), c + vec2(0.0, -8.0)], s);
            p.line_segment([c + vec2(-5.0, -3.0), c + vec2(0.0, -8.0)], s);
            p.line_segment([c + vec2(5.0, -3.0), c + vec2(0.0, -8.0)], s);
            p.line_segment([c + vec2(-8.0, 4.0), c + vec2(-8.0, 8.0)], s);
            p.line_segment([c + vec2(-8.0, 8.0), c + vec2(8.0, 8.0)], s);
            p.line_segment([c + vec2(8.0, 8.0), c + vec2(8.0, 4.0)], s);
        }
        "Projects" => {
            p.rect_stroke(Rect::from_center_size(c + vec2(0.0, 1.5), vec2(17.0, 12.0)), 2.0, s, StrokeKind::Middle);
            p.line_segment([c + vec2(-8.5, -4.5), c + vec2(-2.0, -4.5)], s);
            p.line_segment([c + vec2(-2.0, -4.5), c + vec2(0.0, -2.5)], s);
        }
        "Compare" => {
            p.rect_stroke(Rect::from_center_size(c + vec2(-5.0, 0.0), vec2(7.0, 16.0)), 1.5, s, StrokeKind::Middle);
            p.rect_stroke(Rect::from_center_size(c + vec2(5.0, 0.0), vec2(7.0, 16.0)), 1.5, s, StrokeKind::Middle);
        }
        "Undo" | "Redo" => {
            let dir = if which == "Undo" { -1.0 } else { 1.0 };
            let arc: Vec<Pos2> = (0..=10)
                .map(|k| {
                    let a = std::f32::consts::PI * (k as f32 / 10.0);
                    c + vec2(-dir * a.cos() * 6.0, -a.sin() * 5.0 + 3.0)
                })
                .collect();
            let tip = *arc.last().unwrap();
            p.add(Shape::line(arc, s));
            p.line_segment([tip, tip + vec2(-3.5, -4.0)], s);
            p.line_segment([tip, tip + vec2(3.5, -4.0)], s);
        }
        _ => {
            p.rect_stroke(Rect::from_center_size(c + vec2(0.0, -1.0), vec2(20.0, 10.0)), 3.0, s, StrokeKind::Middle);
            p.line_segment([c + vec2(-10.0, 7.0), c + vec2(10.0, 7.0)], s);
            p.rect_filled(Rect::from_center_size(c + vec2(3.0, -1.0), vec2(5.0, 5.0)), 1.0, col);
        }
    }
}

fn rail_button(ui: &mut Ui, label: &str, active: bool) -> egui::Response {
    let (rect, resp) = ui.allocate_exact_size(vec2(72.0, 58.0), Sense::click());
    let tile = rect.shrink2(vec2(6.0, 3.0));
    let (bg, fg) = if active {
        (ACCENT_SOFT(), ACCENT_DARK())
    } else if resp.hovered() {
        (SUBTLE(), INK())
    } else {
        (SURFACE(), RAIL_FG())
    };
    ui.painter().rect_filled(tile, 10.0, bg);
    if active {
        ui.painter().rect_filled(Rect::from_min_size(pos2(rect.left(), tile.top() + 12.0), vec2(3.0, tile.height() - 24.0)), 2.0, ACCENT);
    }
    draw_icon(ui.painter(), tile.center() - vec2(0.0, 8.0), label, fg);
    ui.painter().text(tile.center() + vec2(0.0, 16.0), Align2::CENTER_CENTER, label, FontId::proportional(11.0), fg);
    resp.on_hover_cursor(egui::CursorIcon::PointingHand)
}

fn primary_button(ui: &mut Ui, text: &str) -> egui::Response {
    ui.add(
        egui::Button::new(RichText::new(text).color(Color32::WHITE).strong().size(14.5))
            .fill(ACCENT)
            .stroke(Stroke::NONE)
            .corner_radius(CornerRadius::same(10))
            .min_size(vec2(100.0, 34.0)),
    )
    .on_hover_cursor(egui::CursorIcon::PointingHand)
}

fn secondary_button(ui: &mut Ui, text: &str) -> egui::Response {
    ui.add(
        egui::Button::new(RichText::new(text).color(INK()).size(13.5))
            .fill(SURFACE())
            .stroke(Stroke::new(1.0, BORDER()))
            .corner_radius(CornerRadius::same(10))
            .min_size(vec2(0.0, 32.0)),
    )
}

fn tool_chip(ui: &mut Ui, text: &str, enabled: bool) -> egui::Response {
    ui.add_enabled(
        enabled,
        egui::Button::new(RichText::new(text).color(INK()).size(12.5))
            .fill(SUBTLE())
            .stroke(Stroke::new(1.0, BORDER()))
            .corner_radius(CornerRadius::same(8))
            .min_size(vec2(0.0, 28.0)),
    )
}

fn header_button(ui: &mut Ui, text: &str) -> egui::Response {
    ui.add(
        egui::Button::new(RichText::new(text).color(Color32::from_gray(225)).size(13.0))
            .fill(HEADER_2)
            .stroke(Stroke::NONE)
            .corner_radius(CornerRadius::same(16))
            .min_size(vec2(0.0, 30.0)),
    )
}

fn icon_only(ui: &mut Ui, which: &str, enabled: bool, tip: &str) -> egui::Response {
    let (rect, resp) = ui.allocate_exact_size(vec2(32.0, 30.0), if enabled { Sense::click() } else { Sense::hover() });
    if enabled && resp.hovered() {
        ui.painter().rect_filled(rect, 8.0, SUBTLE());
    }
    draw_icon(ui.painter(), rect.center(), which, if enabled { INK() } else { DISABLED() });
    resp.on_hover_text(tip)
}

/// A number field for a length kept in inches, shown in inches or millimetres.
fn len_drag<'a>(v: &'a mut f32, prefix: &'static str, metric: bool) -> egui::DragValue<'a> {
    egui::DragValue::new(v)
        .speed(0.02)
        .custom_formatter(move |x, _| if metric { format!("{prefix}{:.1} mm", x * 25.4) } else { format!("{prefix}{x:.2} in") })
        .custom_parser(move |s| {
            let t: String = s.chars().filter(|c| c.is_ascii_digit() || *c == '.' || *c == '-').collect();
            t.parse::<f64>().ok().map(|n| if metric { n / 25.4 } else { n })
        })
}

/// A pill of choices where one is picked; returns true when the choice changed.
fn segmented<T: PartialEq + Copy>(ui: &mut Ui, value: &mut T, options: &[(T, &str)]) -> bool {
    let mut changed = false;
    egui::Frame::NONE.fill(SEG_BG()).corner_radius(9).inner_margin(3).show(ui, |ui| {
        ui.horizontal(|ui| {
            ui.spacing_mut().item_spacing.x = 2.0;
            for (v, label) in options {
                let sel = *value == *v;
                let text = RichText::new(*label).size(13.0).color(if sel { INK() } else { MUTED() });
                let b = egui::Button::new(if sel { text.strong() } else { text })
                    .fill(if sel { SURFACE() } else { Color32::TRANSPARENT })
                    .stroke(Stroke::NONE)
                    .corner_radius(CornerRadius::same(7))
                    .min_size(vec2(0.0, 26.0));
                if ui.add(b).clicked() && !sel {
                    *value = *v;
                    changed = true;
                }
            }
        });
    });
    changed
}

/// A soft rounded group of controls with a small caption in front.
fn group(ui: &mut Ui, caption: &str, add: impl FnOnce(&mut Ui)) {
    egui::Frame::NONE.fill(SUBTLE()).stroke(Stroke::new(1.0, BORDER())).corner_radius(10).inner_margin(Margin::symmetric(8, 3)).show(ui, |ui| {
        ui.horizontal_centered(|ui| {
            ui.label(RichText::new(caption).size(11.5).color(MUTED()));
            add(ui);
        });
    });
}

fn card(ui: &mut Ui, add: impl FnOnce(&mut Ui)) {
    egui::Frame::NONE.fill(SURFACE()).stroke(Stroke::new(1.0, BORDER())).corner_radius(12).inner_margin(14).shadow(shadow(8, 10)).show(ui, |ui| {
        ui.set_width(ui.available_width());
        add(ui);
    });
}

fn section_title(ui: &mut Ui, title: &str, subtitle: &str) {
    ui.label(RichText::new(title).size(17.0).strong().color(INK()));
    if !subtitle.is_empty() {
        ui.label(RichText::new(subtitle).size(12.5).color(MUTED()));
    }
    ui.add_space(6.0);
}

fn badge(ui: &mut Ui, text: &str, color: Color32) {
    egui::Frame::NONE.fill(color.gamma_multiply(0.14)).corner_radius(6).inner_margin(Margin::symmetric(6, 1)).show(ui, |ui| {
        ui.label(RichText::new(text).size(11.0).color(color).strong());
    });
}

fn menu_item(ui: &mut Ui, text: &str, shortcut: &str, enabled: bool) -> bool {
    let r = ui.add_enabled(enabled, egui::Button::new(text).shortcut_text(shortcut).min_size(vec2(190.0, 26.0)).fill(Color32::TRANSPARENT).stroke(Stroke::NONE));
    if r.clicked() {
        ui.close();
        true
    } else {
        false
    }
}

// ---------------------------------------------------------------- the app

impl eframe::App for App {
    fn ui(&mut self, ui: &mut Ui, _frame: &mut eframe::Frame) {
        self.pump_machine();
        let ctx = ui.ctx().clone();
        self.ensure_textures(&ctx);
        self.ensure_engrave();
        let now = ctx.input(|i| i.time);
        self.autosave(now);
        // connect on start when asked to (Settings > Machines > Autoconnect)
        if !self.autoconnect_tried {
            self.autoconnect_tried = true;
            if self.prefs.autoconnect && self.settings.link != LinkKind::Demo {
                self.connect();
            }
        }
        if fonts::families().is_none() {
            ctx.request_repaint_after(std::time::Duration::from_millis(200));
        }

        // files dropped onto the window are uploaded
        let dropped: Vec<String> = ctx.input(|i| i.raw.dropped_files.iter().map(|f| f.path().display().to_string()).filter(|p| !p.is_empty()).collect());
        if !dropped.is_empty() {
            self.screen = Screen::Design;
            self.drawer = Drawer::Upload;
            self.upload_queue.extend(dropped);
            self.next_upload();
        }
        if ctx.input(|i| !i.raw.hovered_files.is_empty()) {
            let p = ctx.layer_painter(egui::LayerId::new(egui::Order::Foreground, egui::Id::unique("drop-overlay")));
            let r = ctx.content_rect();
            p.rect_filled(r, 0.0, Color32::from_rgba_unmultiplied(16, 163, 107, 40));
            let card_r = Rect::from_center_size(r.center(), vec2(360.0, 120.0));
            p.add(shadow(24, 50).as_shape(card_r, CornerRadius::same(16)));
            p.rect_filled(card_r, 16.0, SURFACE());
            p.rect_stroke(card_r, 16.0, Stroke::new(2.0, ACCENT), StrokeKind::Inside);
            draw_icon(&p, card_r.center() - vec2(0.0, 20.0), "Upload", ACCENT_DARK());
            p.text(card_r.center() + vec2(0.0, 12.0), Align2::CENTER_CENTER, "Drop to upload", FontId::proportional(18.0), INK());
            p.text(card_r.center() + vec2(0.0, 36.0), Align2::CENTER_CENTER, "SVG, DXF, PNG, JPG, BMP, GIF, WebP", FontId::proportional(12.0), MUTED());
        }

        let mut acts: Vec<Act> = Vec::new();
        if self.screen != Screen::Design && self.text_edit.is_some() {
            self.finish_text_edit();
        }
        let typing = self.text_edit.is_some() && !ctx.egui_wants_keyboard_input();
        if typing {
            self.type_into_text(&ctx);
            // keep the cursor blinking
            ctx.request_repaint_after(std::time::Duration::from_millis(250));
        }
        if self.screen == Screen::Design && !ctx.egui_wants_keyboard_input() && !typing {
            ctx.input(|i| {
                let cmd = i.modifiers.command;
                for e in &i.events {
                    match e {
                        egui::Event::Copy => acts.push(Act::Copy),
                        egui::Event::Cut => acts.push(Act::Cut),
                        egui::Event::Paste(_) => acts.push(Act::Paste),
                        _ => {}
                    }
                }
                if i.key_pressed(Key::Delete) || i.key_pressed(Key::Backspace) {
                    acts.push(Act::Delete);
                }
                if cmd && i.key_pressed(Key::Z) {
                    acts.push(if i.modifiers.shift { Act::Redo } else { Act::Undo });
                }
                if cmd && i.key_pressed(Key::Y) {
                    acts.push(Act::Redo);
                }
                if cmd && i.key_pressed(Key::D) {
                    acts.push(Act::Duplicate);
                }
                if cmd && i.key_pressed(Key::A) {
                    acts.push(Act::SelectAll);
                }
                if cmd && i.key_pressed(Key::G) {
                    acts.push(if i.modifiers.shift { Act::Ungroup } else { Act::Group });
                }
                if i.key_pressed(Key::Escape) {
                    acts.push(Act::Deselect);
                }
                let step = if i.modifiers.shift { 1.0 } else { 0.1 };
                for (k, d) in [(Key::ArrowLeft, (-step, 0.0)), (Key::ArrowRight, (step, 0.0)), (Key::ArrowUp, (0.0, -step)), (Key::ArrowDown, (0.0, step))] {
                    if i.key_pressed(k) {
                        acts.push(Act::Nudge(d.0, d.1));
                    }
                }
            });
        }

        self.header(ui);
        match self.screen {
            Screen::Design => {
                self.edit_bar(ui, &mut acts);
                if self.sel_one().map(|i| self.items[i].is_text()).unwrap_or(false) {
                    self.text_bar(ui, &mut acts);
                } else {
                    self.font_picker = None;
                }
                self.engrave_bar(ui);
                self.left_bar(ui);
                self.layers_panel(ui, &mut acts);
                if self.drawer != Drawer::None {
                    self.drawer_panel(ui);
                }
                egui::CentralPanel::no_frame().frame(egui::Frame::NONE.fill(BG())).show(ui, |ui| self.canvas(ui, &mut acts));
            }
            Screen::Make => self.make_screen(ui),
            Screen::Home => self.home_screen(ui),
            Screen::MyStuff => self.my_stuff_screen(ui),
        }
        self.new_project_window(&ctx);
        self.settings_window(&ctx);
        if self.show_machine {
            self.machine_window(&ctx);
        }
        if self.show_compare {
            let mut open = true;
            egui::Window::new(RichText::new("Compared with the original app").strong())
                .open(&mut open)
                .default_width(880.0)
                .default_height(700.0)
                .default_pos(ctx.content_rect().center() + vec2(-440.0, -360.0))
                .show(&ctx, |ui| {
                    segmented(ui, &mut self.compare_tab, &[(0u8, "Features"), (1u8, "Machine behaviour")]);
                    ui.add_space(6.0);
                    if self.compare_tab == 0 {
                        crate::compare::show(ui, &mut self.compare_filter);
                    } else {
                        if self.mech_report.is_none() {
                            let original = crate::mechanics_check::default_original_path();
                            self.mech_report = Some(crate::mechanics_check::run(original.as_deref()));
                        }
                        if let Some(r) = &self.mech_report {
                            if crate::mechanics_check::show(ui, r) {
                                self.mech_report = None; // measured again next frame
                            }
                        }
                    }
                });
            if !open {
                self.show_compare = false;
            }
        }
        self.image_window(&ctx);
        self.offset_window(&ctx);
        self.contour_window(&ctx);

        for a in acts {
            self.apply(a);
        }
    }
}

impl App {
    fn header(&mut self, ui: &mut Ui) {
        egui::Panel::top("header")
            .exact_size(58.0)
            .frame(egui::Frame::NONE.fill(HEADER).inner_margin(Margin::symmetric(18, 8)))
            .show(ui, |ui| {
                let full = ui.max_rect();
                ui.horizontal_centered(|ui| {
                    let (r, _) = ui.allocate_exact_size(vec2(32.0, 32.0), Sense::hover());
                    ui.painter().rect_filled(r, 9.0, ACCENT);
                    ui.painter().line_segment([r.center() + vec2(-7.0, 6.0), r.center() + vec2(7.0, -6.0)], Stroke::new(2.6, Color32::WHITE));
                    ui.painter().circle_filled(r.center() + vec2(-7.0, 6.0), 2.4, Color32::WHITE);
                    ui.add_space(2.0);
                    ui.vertical(|ui| {
                        ui.add_space(1.0);
                        ui.label(RichText::new("MatCraft").color(Color32::WHITE).size(16.5).strong());
                        ui.add(
                            egui::TextEdit::singleline(&mut self.project_name)
                                .desired_width(170.0)
                                .font(FontId::proportional(12.0))
                                .text_color(Color32::from_gray(160))
                                .background_color(HEADER)
                                .frame(egui::Frame::NONE),
                        )
                        .on_hover_text("Click to rename the project");
                    });
                    ui.add_space(10.0);
                    // Home, and the project's kind
                    if header_button(ui, "Home").on_hover_text("Home page and My Stuff").clicked() {
                        self.finish_text_edit();
                        self.my_stuff = None;
                        self.screen = Screen::Home;
                    }
                    if matches!(self.screen, Screen::Design | Screen::Make) && self.project_type != ProjectType::Blank {
                        badge(ui, self.project_type.label(), Color32::from_rgb(0x8f, 0xe3, 0xb8));
                    }
                    ui.with_layout(Layout::right_to_left(Align::Center), |ui| {
                        match self.screen {
                            Screen::Design => {
                                if primary_button(ui, "Make it  >").clicked() {
                                    self.finish_text_edit();
                                    self.screen = Screen::Make;
                                    self.mat_index = 0;
                                    self.status.clear();
                                }
                                if header_button(ui, "Save").on_hover_text("Save to My Stuff").clicked() {
                                    self.save_project();
                                }
                            }
                            Screen::Make => {
                                if header_button(ui, "<  Back to canvas").clicked() {
                                    self.screen = Screen::Design;
                                }
                            }
                            Screen::Home | Screen::MyStuff => {
                                if primary_button(ui, "+ New Project").clicked() {
                                    self.open_new_project(None);
                                }
                            }
                        }
                        ui.add_space(8.0);
                        if header_button(ui, "Settings").clicked() {
                            self.show_settings = !self.show_settings;
                        }
                        let (dot, txt) = match &self.connected {
                            Some(n) => (Color32::from_rgb(0x34, 0xd3, 0x99), n.to_string()),
                            None => (Color32::from_rgb(0xf0, 0x6b, 0x6b), "Machine not connected".to_string()),
                        };
                        let resp = header_button(ui, &format!("     {txt}"));
                        ui.painter().circle_filled(resp.rect.left_center() + vec2(15.0, 0.0), 4.5, dot);
                        if resp.clicked() {
                            self.show_machine = !self.show_machine;
                        }
                        ui.add_space(4.0);
                        // light / dark switch: a moon in light mode, a sun in dark mode
                        let (r, tresp) = ui.allocate_exact_size(vec2(34.0, 30.0), Sense::click());
                        let hover = tresp.hovered();
                        ui.painter().rect_filled(r, 15.0, if hover { Color32::from_rgb(0x30, 0x3e, 0x4b) } else { HEADER_2 });
                        let c = r.center();
                        let fg = Color32::from_gray(230);
                        if dark() {
                            ui.painter().circle_filled(c, 4.5, Color32::from_rgb(0xff, 0xd5, 0x5a));
                            for k in 0..8 {
                                let a = k as f32 * std::f32::consts::TAU / 8.0;
                                let d = vec2(a.cos(), a.sin());
                                ui.painter().line_segment([c + d * 7.0, c + d * 9.5], Stroke::new(1.6, Color32::from_rgb(0xff, 0xd5, 0x5a)));
                            }
                        } else {
                            ui.painter().circle_filled(c, 7.0, fg);
                            ui.painter().circle_filled(c + vec2(3.5, -2.5), 6.0, if hover { Color32::from_rgb(0x30, 0x3e, 0x4b) } else { HEADER_2 });
                        }
                        let tip = if dark() { "Switch to light mode" } else { "Switch to dark mode" };
                        if tresp.on_hover_text(tip).on_hover_cursor(egui::CursorIcon::PointingHand).clicked() {
                            let ctx = ui.ctx().clone();
                            self.toggle_dark(&ctx);
                        }
                    });
                });
                let steps = [("1", "Design", self.screen == Screen::Design), ("2", "Prepare mats", self.screen == Screen::Make), ("3", "Cut", false)];
                let w = 380.0;
                let mut x = full.center().x - w / 2.0;
                let y = full.center().y;
                let p = ui.painter();
                for (n, (num, label, active)) in steps.iter().enumerate() {
                    let done = n == 0 && self.screen == Screen::Make;
                    let circle = if *active { ACCENT } else if done { ACCENT_DARK() } else { HEADER_2 };
                    let cc = pos2(x + 11.0, y);
                    p.circle_filled(cc, 11.0, circle);
                    if done {
                        let st = Stroke::new(2.0, Color32::WHITE);
                        p.line_segment([cc + vec2(-4.5, 0.5), cc + vec2(-1.5, 3.5)], st);
                        p.line_segment([cc + vec2(-1.5, 3.5), cc + vec2(4.5, -3.5)], st);
                    } else {
                        p.text(cc, Align2::CENTER_CENTER, *num, FontId::proportional(12.0), Color32::WHITE);
                    }
                    let tc = if *active { Color32::WHITE } else { Color32::from_gray(150) };
                    let g = p.layout_no_wrap(label.to_string(), FontId::proportional(13.0), tc);
                    let tw = g.size().x;
                    p.galley(pos2(x + 28.0, y - g.size().y / 2.0), g, tc);
                    x += 28.0 + tw + 14.0;
                    if n < 2 {
                        p.line_segment([pos2(x, y), pos2(x + 22.0, y)], Stroke::new(1.5, HEADER_2));
                        x += 36.0;
                    }
                }
            });
    }

    fn left_bar(&mut self, ui: &mut Ui) {
        egui::Panel::left("tools")
            .exact_size(80.0)
            .resizable(false)
            .show_separator_line(false)
            .frame(egui::Frame::NONE.fill(SURFACE()).stroke(Stroke::new(1.0, BORDER())).inner_margin(Margin::symmetric(2, 8)))
            .show(ui, |ui| {
                ui.vertical_centered(|ui| {
                    ui.spacing_mut().item_spacing.y = 2.0;
                    let items = [("New", Drawer::None), ("Shapes", Drawer::Shapes), ("Text", Drawer::Text), ("Upload", Drawer::Upload), ("Projects", Drawer::Projects)];
                    for (l, d) in items {
                        let active = d != Drawer::None && self.drawer == d;
                        if rail_button(ui, l, active).clicked() {
                            if l == "New" {
                                self.snapshot();
                                self.items.clear();
                                self.selection.clear();
                                self.project_name = "Untitled project".into();
                                self.drawer = Drawer::None;
                            } else {
                                self.drawer = if self.drawer == d { Drawer::None } else { d };
                                self.status.clear();
                            }
                        }
                    }
                    ui.add_space(8.0);
                    let (r, _) = ui.allocate_exact_size(vec2(48.0, 1.0), Sense::hover());
                    ui.painter().rect_filled(r, 0.0, BORDER());
                    ui.add_space(8.0);
                    if rail_button(ui, "Machine", self.show_machine).clicked() {
                        self.show_machine = !self.show_machine;
                    }
                    if rail_button(ui, "Compare", self.show_compare).clicked() {
                        self.show_compare = !self.show_compare;
                    }
                });
            });
    }

    fn drawer_panel(&mut self, ui: &mut Ui) {
        egui::Panel::left("drawer")
            .exact_size(250.0)
            .resizable(false)
            .show_separator_line(false)
            .frame(egui::Frame::NONE.fill(SURFACE()).stroke(Stroke::new(1.0, BORDER())).inner_margin(Margin::symmetric(16, 16)))
            .show(ui, |ui| match self.drawer {
                Drawer::Shapes => {
                    section_title(ui, "Shapes", "Click a shape to add it to your design.");
                    egui::ScrollArea::vertical().show(ui, |ui| {
                        egui::Grid::new("shapes").spacing(vec2(10.0, 10.0)).show(ui, |ui| {
                            for (n, k) in ShapeKind::ALL.iter().enumerate() {
                                let (r, resp) = ui.allocate_exact_size(vec2(104.0, 86.0), Sense::click());
                                let hovered = resp.hovered();
                                if hovered {
                                    ui.painter().add(shadow(10, 22).as_shape(r, CornerRadius::same(12)));
                                }
                                ui.painter().rect_filled(r, 12.0, if hovered { SURFACE() } else { SUBTLE() });
                                ui.painter().rect_stroke(r, 12.0, Stroke::new(1.0, if hovered { ACCENT } else { BORDER() }), StrokeKind::Inside);
                                let inner = Rect::from_center_size(r.center() - vec2(0.0, 9.0), vec2(40.0, 40.0));
                                let cs: Vec<Vec<Pos2>> = k.unit_contours().into_iter().map(|c| c.into_iter().map(|p| inner.lerp_inside(p.to_vec2())).collect()).collect();
                                if *k == ShapeKind::Line {
                                    for c in &cs {
                                        ui.painter().extend(Shape::dashed_line(c, Stroke::new(2.2, INK()), 5.0, 3.0));
                                    }
                                } else {
                                    fill_even_odd(ui.painter(), &cs, PALETTE[n % PALETTE.len()]);
                                }
                                ui.painter().text(r.center_bottom() - vec2(0.0, 12.0), Align2::CENTER_CENTER, k.label(), FontId::proportional(12.0), INK());
                                if resp.on_hover_cursor(egui::CursorIcon::PointingHand).clicked() {
                                    self.add_shape(*k);
                                }
                                if n % 2 == 1 {
                                    ui.end_row();
                                }
                            }
                        });
                    });
                }
                Drawer::Text => {
                    section_title(ui, "Text", "Type, then press Add. Change font, spacing and curve in the text bar.");
                    ui.add(egui::TextEdit::multiline(&mut self.text_input).desired_rows(3).desired_width(f32::INFINITY).font(FontId::proportional(18.0)).margin(Margin::same(10)));
                    ui.add_space(6.0);
                    if primary_button(ui, "Add text").clicked() {
                        let t = self.text_input.clone();
                        self.snapshot();
                        let c = self.next_color();
                        let at = self.place_point();
                        self.add_text_item(&t, at, c);
                        // go straight to typing on the canvas
                        if let Some(i) = self.sel_one() {
                            let now = ui.input(|i| i.time);
                            self.start_text_edit(i, now);
                            self.undo.pop(); // adding and typing are one undo step
                        }
                    }
                    ui.label(RichText::new("Tip: double-click any text on the canvas to type into it.").size(12.0).color(MUTED()));
                    ui.add_space(8.0);
                    let n = fonts::families().map(|f| f.len()).unwrap_or(0);
                    ui.label(
                        RichText::new(if n > 0 { format!("{n} fonts found on this computer.") } else { "Looking for fonts on this computer...".into() })
                            .size(12.0)
                            .color(MUTED()),
                    );
                    if !self.status.is_empty() {
                        ui.label(RichText::new(&self.status).color(MUTED()));
                    }
                }
                Drawer::Upload => {
                    section_title(ui, "Upload", "SVG, DXF, PNG, JPG, BMP, GIF or WebP.");
                    // a drop zone that also opens the file picker
                    let (zone, zresp) = ui.allocate_exact_size(vec2(ui.available_width(), 120.0), Sense::click());
                    let hot = zresp.hovered() || ui.ctx().input(|i| !i.raw.hovered_files.is_empty());
                    ui.painter().rect_filled(zone, 12.0, if hot { ACCENT_SOFT() } else { SUBTLE() });
                    let border: Vec<Pos2> = [zone.left_top(), zone.right_top(), zone.right_bottom(), zone.left_bottom(), zone.left_top()]
                        .into_iter()
                        .map(|p| p + (zone.center() - p).normalized() * 1.0)
                        .collect();
                    ui.painter().extend(Shape::dashed_line(&border, Stroke::new(1.5, if hot { ACCENT } else { DISABLED() }), 7.0, 4.0));
                    draw_icon(ui.painter(), zone.center() - vec2(0.0, 22.0), "Upload", if hot { ACCENT_DARK() } else { MUTED() });
                    ui.painter().text(zone.center() + vec2(0.0, 6.0), Align2::CENTER_CENTER, "Drop files here", FontId::proportional(14.0), INK());
                    ui.painter().text(zone.center() + vec2(0.0, 26.0), Align2::CENTER_CENTER, "or click to browse", FontId::proportional(12.0), MUTED());
                    if zresp.on_hover_cursor(egui::CursorIcon::PointingHand).clicked() {
                        self.browse_uploads();
                    }
                    ui.add_space(6.0);
                    if primary_button(ui, "Choose files...").clicked() {
                        self.browse_uploads();
                    }
                    ui.add_space(10.0);
                    ui.label(RichText::new("Or type a file's path, or paste SVG text:").size(12.0).color(MUTED()));
                    ui.add(
                        egui::TextEdit::multiline(&mut self.svg_input)
                            .desired_rows(2)
                            .desired_width(f32::INFINITY)
                            .hint_text("/home/you/Pictures/flower.png")
                            .margin(Margin::same(8)),
                    );
                    if secondary_button(ui, "Upload this").clicked() {
                        let input = self.svg_input.clone();
                        self.import_file(&input);
                    }
                    ui.add_space(10.0);
                    let (r, _) = ui.allocate_exact_size(vec2(ui.available_width(), 1.0), Sense::hover());
                    ui.painter().rect_filled(r, 0.0, BORDER());
                    ui.add_space(6.0);
                    ui.label(RichText::new("SVG converter").size(14.0).strong().color(INK()));
                    ui.label(
                        RichText::new("Turn photos, drawings and DXF files into cut-ready SVG files saved next to them.").size(12.0).color(MUTED()),
                    );
                    if secondary_button(ui, "Convert files to SVG...").clicked() {
                        self.convert_files_to_svg();
                    }
                    if !self.status.is_empty() {
                        ui.add_space(4.0);
                        ui.label(RichText::new(&self.status).size(12.5).color(MUTED()));
                    }
                }
                Drawer::Projects => {
                    section_title(ui, "Projects", "Save your design, or open one you saved.");
                    ui.label(RichText::new("File").size(12.0).color(MUTED()));
                    ui.add(egui::TextEdit::singleline(&mut self.project_path).desired_width(f32::INFINITY).margin(Margin::same(8)));
                    ui.add_space(6.0);
                    ui.horizontal(|ui| {
                        if primary_button(ui, "Save").clicked() {
                            let p = std::path::PathBuf::from(&self.project_path);
                            if let Some(d) = p.parent() {
                                let _ = std::fs::create_dir_all(d);
                            }
                            self.status = match serde_json::to_string(&self.items)
                                .map_err(|e| e.to_string())
                                .and_then(|s| std::fs::write(&p, s).map_err(|e| e.to_string()))
                            {
                                Ok(()) => format!("Saved {}", p.display()),
                                Err(e) => format!("Couldn't save: {e}"),
                            };
                        }
                        if secondary_button(ui, "Open").clicked() {
                            match std::fs::read_to_string(&self.project_path)
                                .map_err(|e| e.to_string())
                                .and_then(|s| serde_json::from_str::<Vec<Item>>(&s).map_err(|e| e.to_string()))
                            {
                                Ok(mut items) => {
                                    self.snapshot();
                                    for it in &mut items {
                                        it.uid = new_uid();
                                        self.next_group = self.next_group.max(it.group.unwrap_or(0)).max(it.attach.unwrap_or(0));
                                    }
                                    self.items = items;
                                    self.selection.clear();
                                    self.status = "Opened.".into();
                                }
                                Err(e) => self.status = format!("Couldn't open: {e}"),
                            }
                        }
                        if secondary_button(ui, "Browse...").on_hover_text("Pick the project file").clicked() {
                            if let Some(p) = rfd::FileDialog::new().add_filter("MatCraft project", &["json"]).set_title("Project file").pick_file() {
                                self.project_path = p.display().to_string();
                            }
                        }
                    });
                    ui.add_space(10.0);
                    if secondary_button(ui, "Export design as SVG...").on_hover_text("One SVG with every visible layer, in millimetres").clicked() {
                        self.export_design_svg();
                    }
                    if !self.status.is_empty() {
                        ui.label(RichText::new(&self.status).color(MUTED()));
                    }
                }
                Drawer::None => {}
            });
    }

    fn edit_bar(&mut self, ui: &mut Ui, acts: &mut Vec<Act>) {
        egui::Panel::top("editbar")
            .exact_size(50.0)
            .show_separator_line(false)
            .frame(egui::Frame::NONE.fill(SURFACE()).stroke(Stroke::new(1.0, BORDER())).inner_margin(Margin::symmetric(12, 8)))
            .show(ui, |ui| {
                ui.horizontal_centered(|ui| {
                    ui.spacing_mut().item_spacing.x = 5.0;
                    if icon_only(ui, "Undo", !self.undo.is_empty(), "Undo (Ctrl+Z)").clicked() {
                        acts.push(Act::Undo);
                    }
                    if icon_only(ui, "Redo", !self.redo.is_empty(), "Redo (Ctrl+Y)").clicked() {
                        acts.push(Act::Redo);
                    }
                    let (r, _) = ui.allocate_exact_size(vec2(1.0, 26.0), Sense::hover());
                    ui.painter().rect_filled(r, 0.0, BORDER());

                    let has = !self.selection.is_empty();
                    let many = self.selection.len() > 1;
                    // Edit menu
                    ui.menu_button("Edit", |ui| {
                        for (t, k, a, en) in [
                            ("Cut", "Ctrl+X", Act::Cut, has),
                            ("Copy", "Ctrl+C", Act::Copy, has),
                            ("Paste", "Ctrl+V", Act::Paste, !self.clipboard.is_empty()),
                            ("Duplicate", "Ctrl+D", Act::Duplicate, has),
                            ("Delete", "Del", Act::Delete, has),
                            ("Select all", "Ctrl+A", Act::SelectAll, !self.items.is_empty()),
                            ("Deselect", "Esc", Act::Deselect, has),
                        ] {
                            if menu_item(ui, t, k, en) {
                                acts.push(a);
                            }
                        }
                    });
                    ui.menu_button("Align", |ui| {
                        for (t, a) in [
                            ("Align left", AlignHow::Left),
                            ("Center horizontally", AlignHow::CenterH),
                            ("Align right", AlignHow::Right),
                            ("Align top", AlignHow::Top),
                            ("Center vertically", AlignHow::CenterV),
                            ("Align bottom", AlignHow::Bottom),
                            ("Center", AlignHow::Center),
                        ] {
                            if menu_item(ui, t, "", many) {
                                acts.push(Act::Align(a));
                            }
                        }
                        ui.separator();
                        let three = self.selection.len() > 2;
                        if menu_item(ui, "Distribute horizontally", "", three) {
                            acts.push(Act::Align(AlignHow::DistributeH));
                        }
                        if menu_item(ui, "Distribute vertically", "", three) {
                            acts.push(Act::Align(AlignHow::DistributeV));
                        }
                    });
                    ui.menu_button("Arrange", |ui| {
                        for (t, a) in [("Send to back", Arrange::Back), ("Move backward", Arrange::Backward), ("Move forward", Arrange::Forward), ("Send to front", Arrange::Front)] {
                            if menu_item(ui, t, "", has) {
                                acts.push(Act::Arrange(a));
                            }
                        }
                    });
                    ui.menu_button("Flip", |ui| {
                        if menu_item(ui, "Flip horizontal", "", has) {
                            acts.push(Act::FlipH);
                        }
                        if menu_item(ui, "Flip vertical", "", has) {
                            acts.push(Act::FlipV);
                        }
                    });
                    if tool_chip(ui, "Offset", has).on_hover_text("Make an outline around the selection").clicked() {
                        acts.push(Act::Offset);
                    }
                    let (r, _) = ui.allocate_exact_size(vec2(1.0, 26.0), Sense::hover());
                    ui.painter().rect_filled(r, 0.0, BORDER());

                    if !has {
                        ui.label(RichText::new("Select something on the canvas to edit it. Drag on empty space to select several.").color(MUTED()));
                        return;
                    }

                    // operation and colour apply to every selected layer
                    let first = self.selection[0];
                    let mut op = self.items[first].op;
                    let mut col = self.items[first].color32();
                    let before_op = op;
                    let before_col = col;
                    let metric = self.prefs.metric;
                    // Settings > Canvas > Operation Type decides whether this menu shows
                    if self.prefs.show_operation {
                        egui::ComboBox::from_id_salt("op")
                            .selected_text(RichText::new(op.label()).color(op_color(op)).strong())
                            .width(118.0)
                            .show_ui(ui, |ui| {
                                for o in Operation::ALL {
                                    ui.selectable_value(&mut op, o, RichText::new(o.label()).color(op_color(o)));
                                }
                            });
                    }
                    ui.color_edit_button_srgba(&mut col).on_hover_text("Colour (one mat per colour)");
                    if op != before_op || col != before_col {
                        self.snapshot();
                        for &i in &self.selection {
                            self.items[i].op = op;
                            if col != before_col {
                                self.items[i].color = [col.r(), col.g(), col.b()];
                            }
                        }
                    }

                    if let Some(s) = self.sel_one() {
                        let before = self.items[s].clone();
                        let lock = self.lock_aspect;
                        let mut toggle_lock = false;
                        {
                            let it = &mut self.items[s];
                            group(ui, "Size", |ui| {
                                let (ow, oh) = (it.size[0], it.size[1]);
                                ui.add(len_drag(&mut it.size[0], "W ", metric).range(0.05..=24.0));
                                let lt = RichText::new(if lock { "linked" } else { "free" }).size(11.5).color(if lock { ACCENT_DARK() } else { MUTED() });
                                if ui.add(egui::Button::new(lt).fill(Color32::TRANSPARENT).stroke(Stroke::NONE)).on_hover_text("Keep proportions").clicked() {
                                    toggle_lock = true;
                                }
                                ui.add(len_drag(&mut it.size[1], "H ", metric).range(0.0001..=24.0));
                                if lock && ow > 1e-3 && oh > 1e-3 {
                                    if it.size[0] != ow {
                                        it.size[1] = oh * it.size[0] / ow;
                                    } else if it.size[1] != oh {
                                        it.size[0] = ow * it.size[1] / oh;
                                    }
                                }
                            });
                            group(ui, "Rotate", |ui| {
                                ui.add(egui::DragValue::new(&mut it.rot).speed(0.5).range(-360.0..=360.0).suffix(" deg").fixed_decimals(0));
                            });
                            group(ui, "Position", |ui| {
                                ui.add(len_drag(&mut it.pos[0], "X ", metric));
                                ui.add(len_drag(&mut it.pos[1], "Y ", metric));
                            });
                        }
                        if toggle_lock {
                            self.lock_aspect = !self.lock_aspect;
                        }
                        let it = &self.items[s];
                        let changed = it.size != before.size || it.rot != before.rot || it.pos != before.pos;
                        let editing = ui.ctx().egui_is_using_pointer() || ui.ctx().egui_wants_keyboard_input();
                        if changed && !self.edit_snapshot_taken {
                            let mut snap = self.items.clone();
                            snap[s] = before;
                            self.undo.push(snap);
                            self.redo.clear();
                            self.edit_snapshot_taken = editing;
                        }
                        if !editing {
                            self.edit_snapshot_taken = false;
                        }
                    } else {
                        // several layers: size and position of the whole selection
                        let b = self.selection_bounds();
                        let (mut w, mut x, mut y) = (b.width(), b.left(), b.top());
                        let h_label = self.fmt_len(b.height());
                        group(ui, "Selection", |ui| {
                            ui.add(len_drag(&mut w, "W ", metric).range(0.05..=48.0));
                            ui.label(RichText::new(format!("H {}", h_label)).color(MUTED()));
                            ui.add(len_drag(&mut x, "X ", metric));
                            ui.add(len_drag(&mut y, "Y ", metric));
                        });
                        if (w - b.width()).abs() > 1e-4 && b.width() > 1e-4 {
                            let k = w / b.width();
                            for &i in &self.selection.clone() {
                                let it = &mut self.items[i];
                                let c = b.min + (it.center() - b.min) * k;
                                it.size = [it.size[0] * k, it.size[1] * k];
                                it.set_center(c);
                            }
                        }
                        let d = vec2(x - b.left(), y - b.top());
                        if d.length() > 1e-5 {
                            for &i in &self.selection.clone() {
                                self.items[i].pos[0] += d.x;
                                self.items[i].pos[1] += d.y;
                            }
                        }
                        ui.label(RichText::new(format!("{} layers", self.selection.len())).size(12.0).color(MUTED()));
                    }
                });
            });
    }

    /// The engrave bar: how the selected Engrave layers are engraved.
    fn engrave_bar(&mut self, ui: &mut Ui) {
        let sel: Vec<usize> = self.selection.iter().copied().filter(|&i| i < self.items.len() && self.items[i].op == Operation::Engrave).collect();
        if sel.is_empty() || sel.len() != self.selection.len() {
            return;
        }
        let has_picture = sel.iter().any(|&i| self.items[i].image_png.is_some());
        let mut e = self.items[sel[0]].engrave;
        let before = e;
        let metric = self.prefs.metric;
        let tool = if self.prefs.model.engraving_tool() { "Engraving Tool" } else { "Engrave Tip" };
        egui::Panel::top("engravebar")
            .exact_size(48.0)
            .show_separator_line(false)
            .frame(egui::Frame::NONE.fill(BAR_ALT()).stroke(Stroke::new(1.0, BORDER())).inner_margin(Margin::symmetric(12, 7)))
            .show(ui, |ui| {
                ui.horizontal_centered(|ui| {
                    ui.spacing_mut().item_spacing.x = 6.0;
                    badge(ui, "Engrave", op_color(Operation::Engrave));
                    let mut styles: Vec<(EngraveStyle, &str)> = vec![(EngraveStyle::Outline, "Outline"), (EngraveStyle::Fill, "Fill")];
                    if has_picture {
                        styles.push((EngraveStyle::Photo, "Photo"));
                    }
                    segmented(ui, &mut e.style, &styles);
                    match e.style {
                        EngraveStyle::Outline => {
                            ui.label(RichText::new("Engraves along the outline. Choose Fill to engrave the whole shape.").color(MUTED()));
                        }
                        EngraveStyle::Fill => {
                            egui::ComboBox::from_id_salt("engrave-pattern").selected_text(e.pattern.label()).width(130.0).show_ui(ui, |ui| {
                                for p in geometry::FillPattern::ALL {
                                    ui.selectable_value(&mut e.pattern, p, p.label());
                                }
                            });
                            group(ui, "Spacing", |ui| {
                                ui.add(len_drag(&mut e.spacing, "", metric).speed(0.002).range(0.005..=0.5));
                            });
                            group(ui, "Edge margin", |ui| {
                                ui.add(len_drag(&mut e.margin, "", metric).speed(0.002).range(0.0..=0.5));
                            });
                            egui::ComboBox::from_id_salt("engrave-rule")
                                .selected_text(if e.even_odd { "Even-odd" } else { "Non-zero" })
                                .width(90.0)
                                .show_ui(ui, |ui| {
                                    ui.selectable_value(&mut e.even_odd, false, "Non-zero");
                                    ui.selectable_value(&mut e.even_odd, true, "Even-odd");
                                })
                                .response
                                .on_hover_text("Which parts count as inside where outlines overlap");
                        }
                        EngraveStyle::Photo => {
                            group(ui, "Line spacing", |ui| {
                                ui.add(len_drag(&mut e.spacing, "", metric).speed(0.002).range(0.005..=0.2));
                            });
                            group(ui, "Brightness", |ui| {
                                ui.add(egui::Slider::new(&mut e.brightness, -1.0..=1.0).show_value(false));
                            });
                            ui.checkbox(&mut e.invert, "Swap light and dark");
                        }
                    }
                    ui.with_layout(Layout::right_to_left(Align::Center), |ui| {
                        ui.label(RichText::new(format!("Tool: {tool}")).size(12.5).color(MUTED()));
                    });
                });
            });
        if e != before {
            if !self.edit_snapshot_taken {
                self.snapshot();
                self.edit_snapshot_taken = true;
            }
            for &i in &sel {
                let pic = self.items[i].image_png.is_some();
                let mut ne = e;
                if ne.style == EngraveStyle::Photo && !pic {
                    ne.style = EngraveStyle::Fill;
                }
                self.items[i].engrave = ne;
            }
        } else if !ui.ctx().egui_is_using_pointer() && !ui.ctx().egui_wants_keyboard_input() && self.text_edit.is_none() {
            self.edit_snapshot_taken = false;
        }
    }

    fn text_bar(&mut self, ui: &mut Ui, acts: &mut Vec<Act>) {
        let Some(s) = self.sel_one() else { return };
        egui::Panel::top("textbar")
            .exact_size(48.0)
            .show_separator_line(false)
            .frame(egui::Frame::NONE.fill(BAR_ALT()).stroke(Stroke::new(1.0, BORDER())).inner_margin(Margin::symmetric(12, 7)))
            .show(ui, |ui| {
                ui.horizontal_centered(|ui| {
                    ui.spacing_mut().item_spacing.x = 6.0;
                    badge(ui, "Text", SELECT);
                    let before_text = self.items[s].text.clone();
                    let before_opts = self.items[s].text_opts.clone().unwrap_or_default();
                    let mut text = before_text.clone().unwrap_or_default();
                    let mut o = before_opts.clone();
                    ui.add(egui::TextEdit::singleline(&mut text).desired_width(150.0).hint_text("Your text"));

                    // font list: arrows move through it and the text on the canvas changes as you go
                    let shown = if o.font.is_empty() { "Built-in (Ubuntu Light)".to_string() } else { o.font.clone() };
                    let mut shown_short = shown.clone();
                    if shown_short.chars().count() > 24 {
                        shown_short = shown_short.chars().take(23).collect::<String>() + "…";
                    }
                    let open = self.font_picker.is_some();
                    let btn = ui
                        .add(
                            egui::Button::new(RichText::new(format!("{shown_short}      ")).color(INK()))
                                .fill(if open { ACCENT_SOFT() } else { SURFACE() })
                                .stroke(Stroke::new(1.0, if open { ACCENT } else { BORDER() }))
                                .corner_radius(CornerRadius::same(8))
                                .min_size(vec2(200.0, 30.0)),
                        )
                        .on_hover_text("Choose a font. Use the arrow keys to try fonts one by one.");
                    // a small drop-down arrow, drawn (the font has no arrow character)
                    let tip = btn.rect.right_center() - vec2(14.0, -2.0);
                    ui.painter().add(Shape::convex_polygon(
                        vec![tip + vec2(-4.5, -4.0), tip + vec2(4.5, -4.0), tip + vec2(0.0, 1.5)],
                        MUTED(),
                        Stroke::NONE,
                    ));
                    if btn.clicked() {
                        if open {
                            self.font_picker = None;
                        } else {
                            self.font_picker = Some(FontPicker {
                                filter: String::new(),
                                highlight: 0,
                                original: o.font.clone(),
                                just_opened: true,
                                scroll_to: true,
                                anchor: btn.rect,
                            });
                        }
                    }
                    if let Some(fp) = self.font_picker.as_mut() {
                        fp.anchor = btn.rect;
                        let f = fp.filter.to_lowercase();
                        let mut list: Vec<String> = Vec::new();
                        if f.is_empty() || "built-in ubuntu light".contains(&f) {
                            list.push(String::new());
                        }
                        if let Some(fams) = fonts::families() {
                            list.extend(fams.iter().filter(|n| f.is_empty() || n.to_lowercase().contains(&f)).cloned());
                        }
                        if fp.just_opened {
                            fp.highlight = list.iter().position(|n| *n == o.font).unwrap_or(0);
                        }
                        // keys: up/down (Page Up/Down jump 10) try fonts live; Enter keeps; Esc goes back
                        let (down, up, pdown, pup, enter, esc) = ui.ctx().input_mut(|i| {
                            (
                                i.consume_key(egui::Modifiers::NONE, Key::ArrowDown),
                                i.consume_key(egui::Modifiers::NONE, Key::ArrowUp),
                                i.consume_key(egui::Modifiers::NONE, Key::PageDown),
                                i.consume_key(egui::Modifiers::NONE, Key::PageUp),
                                i.consume_key(egui::Modifiers::NONE, Key::Enter),
                                i.consume_key(egui::Modifiers::NONE, Key::Escape),
                            )
                        });
                        let n = list.len();
                        let mut moved = false;
                        if n > 0 {
                            let h = fp.highlight.min(n - 1);
                            let new_h = if down {
                                (h + 1).min(n - 1)
                            } else if up {
                                h.saturating_sub(1)
                            } else if pdown {
                                (h + 10).min(n - 1)
                            } else if pup {
                                h.saturating_sub(10)
                            } else {
                                h
                            };
                            if new_h != fp.highlight || down || up || pdown || pup {
                                moved = true;
                            }
                            fp.highlight = new_h;
                            if moved {
                                o.font = list[new_h].clone();
                                fp.scroll_to = true;
                            }
                        }
                        let mut close = enter;
                        if esc {
                            o.font = fp.original.clone();
                            close = true;
                        }
                        let mut picked: Option<String> = None;
                        let area = egui::Area::new(egui::Id::unique("font-picker"))
                            .order(egui::Order::Foreground)
                            .fixed_pos(fp.anchor.left_bottom() + vec2(0.0, 6.0))
                            .show(ui.ctx(), |ui| {
                                egui::Frame::NONE
                                    .fill(SURFACE())
                                    .stroke(Stroke::new(1.0, BORDER()))
                                    .corner_radius(12)
                                    .inner_margin(10)
                                    .shadow(shadow(18, 40))
                                    .show(ui, |ui| {
                                        ui.set_width(280.0);
                                        let search = ui.add(
                                            egui::TextEdit::singleline(&mut fp.filter)
                                                .hint_text("Search fonts")
                                                .desired_width(f32::INFINITY)
                                                .margin(Margin::same(6)),
                                        );
                                        if fp.just_opened {
                                            search.request_focus();
                                        }
                                        if search.changed() {
                                            fp.highlight = 0;
                                            fp.scroll_to = true;
                                        }
                                        ui.label(RichText::new("Up/Down to try fonts, Enter to keep, Esc to go back").size(11.0).color(MUTED()));
                                        if fonts::families().is_none() {
                                            ui.label(RichText::new("Looking for fonts on this computer...").color(MUTED()));
                                        }
                                        egui::ScrollArea::vertical().max_height(340.0).auto_shrink([false, true]).show(ui, |ui| {
                                            for (k, name) in list.iter().enumerate() {
                                                let label = if name.is_empty() { "Built-in (Ubuntu Light)" } else { name.as_str() };
                                                let sel = k == fp.highlight;
                                                let r = ui.add_sized(
                                                    vec2(ui.available_width(), 26.0),
                                                    egui::Button::new(RichText::new(label).color(if sel { Color32::WHITE } else { INK() }))
                                                        .fill(if sel { SELECT } else { Color32::TRANSPARENT })
                                                        .stroke(Stroke::NONE)
                                                        .corner_radius(CornerRadius::same(6)),
                                                );
                                                if sel && fp.scroll_to {
                                                    r.scroll_to_me(Some(Align::Center));
                                                }
                                                if r.clicked() {
                                                    picked = Some(name.clone());
                                                }
                                            }
                                            if list.is_empty() {
                                                ui.label(RichText::new("No font matches.").color(MUTED()));
                                            }
                                        });
                                    });
                            });
                        fp.scroll_to = false;
                        if let Some(name) = picked {
                            o.font = name;
                            close = true;
                        }
                        // a click anywhere outside the list closes it
                        let outside = ui.ctx().input(|i| {
                            i.pointer.any_pressed()
                                && i.pointer.interact_pos().map(|p| !area.response.rect.contains(p) && !fp.anchor.contains(p)).unwrap_or(false)
                        });
                        fp.just_opened = false;
                        if close || outside {
                            self.font_picker = None;
                        }
                    }
                    let bold_real = fonts::has_style(&o.font, true, o.italic);
                    if ui
                        .selectable_label(o.bold, RichText::new("B").strong())
                        .on_hover_text(if bold_real || o.font.is_empty() { "Bold" } else { "Bold (made thicker; this font has no bold of its own)" })
                        .clicked()
                    {
                        o.bold = !o.bold;
                    }
                    if ui.selectable_label(o.italic, RichText::new("I").italics()).on_hover_text("Italic").clicked() {
                        o.italic = !o.italic;
                    }
                    group(ui, "Letter", |ui| {
                        ui.add(egui::DragValue::new(&mut o.letter_spacing).speed(0.01).range(-0.5..=3.0).fixed_decimals(2));
                    });
                    group(ui, "Line", |ui| {
                        ui.add(egui::DragValue::new(&mut o.line_spacing).speed(0.01).range(0.3..=4.0).fixed_decimals(2));
                    });
                    segmented(ui, &mut o.align, &[(TextAlign::Left, "Left"), (TextAlign::Center, "Center"), (TextAlign::Right, "Right")]);
                    group(ui, "Curve", |ui| {
                        ui.add(egui::Slider::new(&mut o.curve, -360.0..=360.0).show_value(true).suffix(" deg").fixed_decimals(0));
                    });
                    if tool_chip(ui, "Ungroup letters", true).on_hover_text("Make every letter its own layer").clicked() {
                        acts.push(Act::UngroupLetters);
                    }

                    let text_changed = Some(text.clone()) != before_text;
                    if text_changed || o != before_opts {
                        if !self.edit_snapshot_taken {
                            self.snapshot();
                            self.edit_snapshot_taken = true;
                        }
                        let it = &mut self.items[s];
                        it.text = Some(text.clone());
                        it.text_opts = Some(o);
                        if text_changed {
                            it.name = format!("Text: {}", text.lines().next().unwrap_or(""));
                        }
                        regen_text(it);
                    } else if !ui.ctx().egui_is_using_pointer() && !ui.ctx().egui_wants_keyboard_input() {
                        self.edit_snapshot_taken = false;
                    }
                });
            });
    }

    fn layers_panel(&mut self, ui: &mut Ui, acts: &mut Vec<Act>) {
        egui::Panel::right("layers")
            .exact_size(280.0)
            .resizable(false)
            .show_separator_line(false)
            .frame(egui::Frame::NONE.fill(SURFACE()).stroke(Stroke::new(1.0, BORDER())).inner_margin(Margin::symmetric(14, 14)))
            .show(ui, |ui| {
                ui.horizontal(|ui| {
                    ui.label(RichText::new("Layers").size(17.0).strong().color(INK()));
                    badge(ui, &self.items.len().to_string(), MUTED());
                    ui.with_layout(Layout::right_to_left(Align::Center), |ui| {
                        let n = self.selection.len();
                        if tool_chip(ui, "Ungroup", n > 0 && self.selection.iter().any(|&i| self.items[i].group.is_some())).clicked() {
                            acts.push(Act::Ungroup);
                        }
                        if tool_chip(ui, "Group", n > 1).clicked() {
                            acts.push(Act::Group);
                        }
                    });
                });
                ui.add_space(6.0);
                let footer_h = 118.0;
                egui::ScrollArea::vertical().max_height((ui.available_height() - footer_h).max(80.0)).show(ui, |ui| {
                    ui.spacing_mut().item_spacing.y = 5.0;
                    for i in (0..self.items.len()).rev() {
                        let sel = self.selection.contains(&i);
                        let (r, resp) = ui.allocate_exact_size(vec2(ui.available_width(), 52.0), Sense::click());
                        let bg = if sel { SELECT_SOFT() } else if resp.hovered() { SUBTLE() } else { SURFACE() };
                        ui.painter().rect_filled(r, 10.0, bg);
                        ui.painter().rect_stroke(r, 10.0, Stroke::new(1.0, if sel { SELECT } else { BORDER() }), StrokeKind::Inside);
                        let it = &self.items[i];
                        if let Some(g) = it.group {
                            let gc = PALETTE[(g as usize) % PALETTE.len()];
                            ui.painter().rect_filled(Rect::from_min_size(r.min + vec2(2.0, 8.0), vec2(3.0, r.height() - 16.0)), 2.0, gc);
                        }
                        let thumb = Rect::from_min_size(r.min + vec2(9.0, 7.0), vec2(38.0, 38.0));
                        ui.painter().rect_filled(thumb, 8.0, if sel { SURFACE() } else { SUBTLE() });
                        let k = 28.0 / it.size[0].max(it.size[1]).max(1e-3);
                        let to_t = |p: Pos2| thumb.center() + vec2((p.x - 0.5) * it.size[0] * k, (p.y - 0.5) * it.size[1] * k);
                        if let (Some(t), Some(_)) = (self.textures.get(&it.uid), &it.image_png) {
                            let rr = it.img_rect;
                            ui.painter().image(t.id(), Rect::from_two_pos(to_t(pos2(rr[0], rr[1])), to_t(pos2(rr[2], rr[3]))), Rect::from_min_max(pos2(0.0, 0.0), pos2(1.0, 1.0)), Color32::WHITE);
                        } else {
                            let cs: Vec<Vec<Pos2>> = it.unit.iter().map(|c| c.iter().map(|p| to_t(*p)).collect()).collect();
                            if it.op.filled() {
                                fill_even_odd(ui.painter(), &cs, if it.visible { it.color32() } else { it.color32().gamma_multiply(0.35) });
                            } else {
                                for c in &cs {
                                    ui.painter().add(Shape::line(c.clone(), Stroke::new(1.2, it.color32())));
                                }
                            }
                        }
                        let name_color = if it.visible { INK() } else { MUTED() };
                        let mut name = it.name.clone();
                        if name.chars().count() > 20 {
                            name = name.chars().take(19).collect::<String>() + "…";
                        }
                        ui.painter().text(r.min + vec2(56.0, 17.0), Align2::LEFT_CENTER, name, FontId::proportional(13.5), name_color);
                        let mut bx = r.min.x + 56.0;
                        let mut chip = |text: String, c: Color32| {
                            let g = ui.painter().layout_no_wrap(text, FontId::proportional(10.5), c);
                            let br = Rect::from_min_size(pos2(bx, r.min.y + 29.0), g.size() + vec2(10.0, 4.0));
                            ui.painter().rect_filled(br, 5.0, c.gamma_multiply(0.14));
                            ui.painter().galley(br.min + vec2(5.0, 2.0), g, c);
                            bx = br.right() + 4.0;
                        };
                        chip(it.op.label().to_string(), op_color(it.op));
                        if let Some(g) = it.group {
                            chip(format!("Group {g}"), PALETTE[(g as usize) % PALETTE.len()]);
                        }
                        if it.attach.is_some() {
                            chip("Attached".into(), MUTED());
                        }
                        if it.hidden.iter().any(|h| *h) {
                            chip("Contoured".into(), MUTED());
                        }
                        let eye = pos2(r.right() - 20.0, r.center().y);
                        let es = Stroke::new(1.4, MUTED());
                        let lens: Vec<Pos2> = (0..=16)
                            .map(|k| {
                                let a = k as f32 / 16.0 * std::f32::consts::TAU;
                                eye + vec2(a.cos() * 8.0, a.sin() * 4.5)
                            })
                            .collect();
                        ui.painter().add(Shape::line(lens, es));
                        if it.visible {
                            ui.painter().circle_filled(eye, 2.6, MUTED());
                        } else {
                            ui.painter().line_segment([eye + vec2(-8.0, 6.0), eye + vec2(8.0, -6.0)], es);
                        }
                        if resp.clicked() {
                            if resp.interact_pointer_pos().map(|p| (p - eye).length() < 13.0).unwrap_or(false) {
                                self.items[i].visible = !self.items[i].visible;
                            } else {
                                let add = ui.input(|inp| inp.modifiers.shift || inp.modifiers.command);
                                self.select(i, add);
                            }
                        }
                    }
                    if self.items.is_empty() {
                        ui.add_space(30.0);
                        ui.vertical_centered(|ui| {
                            ui.label(RichText::new("Your design is empty").strong().color(INK()));
                            ui.label(RichText::new("Add a shape, text or an upload from the left.").color(MUTED()));
                        });
                    }
                });
                // the layer tools along the bottom, as in the original's layers panel
                ui.with_layout(Layout::bottom_up(Align::Min), |ui| {
                    let n = self.selection.len();
                    let attached = self.selection.iter().any(|&i| self.items[i].attach.is_some());
                    let flattened = self.selection.iter().any(|&i| self.items[i].attach.is_some() && self.items[i].op == Operation::Print);
                    ui.horizontal_wrapped(|ui| {
                        ui.spacing_mut().item_spacing = vec2(4.0, 4.0);
                        if tool_chip(ui, "Duplicate", n > 0).clicked() {
                            acts.push(Act::Duplicate);
                        }
                        if tool_chip(ui, "Delete", n > 0).clicked() {
                            acts.push(Act::Delete);
                        }
                        if tool_chip(ui, "Contour", n == 1).on_hover_text("Hide or show the shapes inside a layer").clicked() {
                            acts.push(Act::Contour);
                        }
                    });
                    ui.horizontal_wrapped(|ui| {
                        ui.spacing_mut().item_spacing = vec2(4.0, 4.0);
                        if tool_chip(ui, "Slice", n == 2).on_hover_text("Cut two overlapping layers into their pieces").clicked() {
                            acts.push(Act::Slice);
                        }
                        ui.add_enabled_ui(n > 1, |ui| {
                            ui.menu_button(RichText::new("Combine").size(12.5), |ui| {
                                if menu_item(ui, "Weld", "", true) {
                                    acts.push(Act::Weld);
                                }
                                for (t, c) in [("Unite", Combine::Unite), ("Subtract", Combine::Subtract), ("Intersect", Combine::Intersect), ("Exclude", Combine::Exclude)] {
                                    if menu_item(ui, t, "", true) {
                                        acts.push(Act::Combine(c));
                                    }
                                }
                            });
                        });
                        if attached && !flattened {
                            if tool_chip(ui, "Detach", true).clicked() {
                                acts.push(Act::Detach);
                            }
                        } else if tool_chip(ui, "Attach", n > 1).on_hover_text("Keep these layers in place together on the mat").clicked() {
                            acts.push(Act::Attach);
                        }
                        if flattened {
                            if tool_chip(ui, "Unflatten", true).clicked() {
                                acts.push(Act::Unflatten);
                            }
                        } else if tool_chip(ui, "Flatten", n > 0).on_hover_text("Turn into one printed image for print then cut").clicked() {
                            acts.push(Act::Flatten);
                        }
                    });
                    ui.add_space(2.0);
                    let (r, _) = ui.allocate_exact_size(vec2(ui.available_width(), 1.0), Sense::hover());
                    ui.painter().rect_filled(r, 0.0, BORDER());
                });
            });
    }

    fn canvas(&mut self, ui: &mut Ui, acts: &mut Vec<Act>) {
        let (full, resp) = ui.allocate_exact_size(ui.available_size(), Sense::click_and_drag());
        let painter = ui.painter_at(full);
        let ppi = BASE_PPI * self.zoom;
        let origin = full.min + self.pan;
        let to_screen = |p: Pos2| origin + p.to_vec2() * ppi;
        let to_world = |s: Pos2| ((s - origin) / ppi).to_pos2();
        let work = Rect::from_min_max(full.min + vec2(RULER, RULER), full.max);

        painter.rect_filled(full, 0.0, BG());
        let w0 = to_world(work.min);
        let w1 = to_world(work.max);
        let dot = DOTS();
        let mut gx = w0.x.floor();
        while gx <= w1.x {
            let mut gy = w0.y.floor();
            while gy <= w1.y {
                let sp = to_screen(pos2(gx, gy));
                if work.contains(sp) {
                    painter.circle_filled(sp, 1.1, dot);
                }
                gy += 1.0;
            }
            gx += 1.0;
        }
        let area = Rect::from_min_max(to_screen(Pos2::ZERO), to_screen(pos2(12.0, 12.0)));
        painter.add(Shadow { offset: [0, 4], blur: 18, spread: 0, color: Color32::from_black_alpha(28) }.as_shape(area, CornerRadius::same(4)));
        painter.rect_filled(area, 4.0, SHEET());
        // grid (Settings > Canvas): full = major lines plus halves, partial = major only;
        // inches, or centimetres in metric
        let unit: f32 = if self.prefs.metric { 1.0 / 2.54 } else { 1.0 };
        if self.prefs.grid != GridMode::None {
            let steps = (12.0 / unit * 2.0).ceil() as i32;
            for i in 1..steps {
                let f = i as f32 * unit * 0.5;
                if f >= 12.0 {
                    break;
                }
                let major = i % 2 == 0;
                if !major && self.prefs.grid == GridMode::Partial {
                    continue;
                }
                let col = if major { GRID_MAJOR() } else { GRID_MINOR() };
                let x = to_screen(pos2(f, 0.0)).x;
                let y = to_screen(pos2(0.0, f)).y;
                painter.line_segment([pos2(x, area.top()), pos2(x, area.bottom())], Stroke::new(1.0, col));
                painter.line_segment([pos2(area.left(), y), pos2(area.right(), y)], Stroke::new(1.0, col));
            }
        }
        painter.rect_stroke(area, 4.0, Stroke::new(1.0, SHEET_EDGE()), StrokeKind::Outside);
        let sheet_label = if self.prefs.metric { "30.5 x 30.5 cm cutting area" } else { "12 x 12 in cutting area" };
        painter.text(area.left_bottom() + vec2(0.0, 8.0), Align2::LEFT_TOP, sheet_label, FontId::proportional(11.0), MUTED());
        // the project's design area: a dashed frame with its name and size
        if let Some(da) = &self.design_area {
            let r = Rect::from_min_size(to_screen(DESIGN_AREA_ORIGIN), vec2(da.size[0], da.size[1]) * ppi);
            painter.rect_filled(r, 6.0, ACCENT.gamma_multiply(0.05));
            let pts = [r.left_top(), r.right_top(), r.right_bottom(), r.left_bottom(), r.left_top()];
            painter.extend(Shape::dashed_line(&pts, Stroke::new(1.5, ACCENT), 8.0, 5.0));
            let label = format!("{}  ·  {}", da.name, self.fmt_size(da.size[0], da.size[1]));
            let g = painter.layout_no_wrap(label, FontId::proportional(11.5), Color32::WHITE);
            let pill = Rect::from_min_size(r.left_top() + vec2(0.0, -22.0), g.size() + vec2(14.0, 6.0));
            painter.rect_filled(pill, 8.0, ACCENT);
            painter.galley(pill.min + vec2(7.0, 3.0), g, Color32::WHITE);
        }

        for it in self.items.iter().filter(|i| i.visible) {
            draw_item(&painter, it, &to_screen, Vec2::ZERO, None, self.textures.get(&it.uid).map(|t| t.id()), self.engrave_cache.get(&it.uid).map(|e| e.1.as_slice()));
        }

        // hidden (contoured) outlines of selected layers, faintly
        for &i in &self.selection {
            let it = &self.items[i];
            for (k, c) in it.unit.iter().enumerate() {
                if it.is_hidden(k) {
                    let pts: Vec<Pos2> = c.iter().map(|p| to_screen(it.to_world(*p))).collect();
                    painter.extend(Shape::dashed_line(&pts, Stroke::new(1.0, MUTED()), 3.0, 3.0));
                }
            }
        }

        // offset preview
        if let Some(t) = &self.offset_tool {
            let parts: Vec<Vec<Contour>> = self.selection.iter().map(|&i| self.items[i].world_contours()).collect();
            let welded = geometry::weld(&parts);
            for c in geometry::offset_contours(&welded, t.distance) {
                let pts: Vec<Pos2> = c.iter().map(|p| to_screen(*p)).collect();
                painter.extend(Shape::dashed_line(&pts, Stroke::new(1.6, ACCENT_DARK()), 6.0, 3.0));
            }
        }

        // selection: a rotated box for one layer, a box around everything for several
        let mut handles: Option<[Pos2; 4]> = None;
        if !self.selection.is_empty() {
            let corners: [Pos2; 5] = if let Some(s) = self.sel_one() {
                let it = &self.items[s];
                [pos2(0., 0.), pos2(1., 0.), pos2(1., 1.), pos2(0., 1.), pos2(0., 0.)].map(|u| to_screen(it.to_world(u)))
            } else {
                for &i in &self.selection {
                    let b = self.items[i].world_bounds();
                    painter.rect_stroke(Rect::from_min_max(to_screen(b.min), to_screen(b.max)), 2.0, Stroke::new(1.0, SELECT.gamma_multiply(0.45)), StrokeKind::Middle);
                }
                let b = self.selection_bounds();
                [b.left_top(), b.right_top(), b.right_bottom(), b.left_bottom(), b.left_top()].map(to_screen)
            };
            painter.extend(Shape::dashed_line(&corners, Stroke::new(1.4, SELECT), 6.0, 3.0));
            let hs = [corners[0], corners[1], corners[2], corners[3]];
            for (n, p) in hs.iter().enumerate() {
                let col = if n == 0 { DANGER } else if n == 3 { MUTED() } else { SELECT };
                painter.add(shadow(6, 40).as_shape(Rect::from_center_size(*p, vec2(24.0, 24.0)), CornerRadius::same(12)));
                painter.circle_filled(*p, 12.0, SURFACE());
                let st = Stroke::new(1.7, col);
                match n {
                    0 => {
                        painter.line_segment([*p + vec2(-4.0, -4.0), *p + vec2(4.0, 4.0)], st);
                        painter.line_segment([*p + vec2(-4.0, 4.0), *p + vec2(4.0, -4.0)], st);
                    }
                    1 => {
                        let arc: Vec<Pos2> = (0..=12)
                            .map(|k| {
                                let a = -2.6 + k as f32 * 0.42;
                                *p + vec2(a.cos(), a.sin()) * 5.0
                            })
                            .collect();
                        let end = *arc.last().unwrap();
                        painter.add(Shape::line(arc, st));
                        painter.line_segment([end, end + vec2(-3.5, -0.5)], st);
                    }
                    2 => {
                        painter.line_segment([*p + vec2(-4.5, -4.5), *p + vec2(4.5, 4.5)], st);
                        painter.line_segment([*p + vec2(4.5, 4.5), *p + vec2(0.5, 4.5)], st);
                        painter.line_segment([*p + vec2(4.5, 4.5), *p + vec2(4.5, 0.5)], st);
                    }
                    _ => {
                        let lc = if self.lock_aspect { ACCENT_DARK() } else { MUTED() };
                        painter.rect_filled(Rect::from_center_size(*p + vec2(0.0, 2.0), vec2(9.0, 6.5)), 1.5, lc);
                        let shackle: Vec<Pos2> = (0..=8)
                            .map(|k| {
                                let a = std::f32::consts::PI + k as f32 / 8.0 * std::f32::consts::PI;
                                *p + vec2(a.cos() * 3.0, -1.0 + a.sin() * 3.5)
                            })
                            .collect();
                        painter.add(Shape::line(shackle, Stroke::new(1.6, lc)));
                    }
                }
            }
            let (w, h) = match self.sel_one() {
                Some(s) => (self.items[s].size[0], self.items[s].size[1]),
                None => {
                    let b = self.selection_bounds();
                    (b.width(), b.height())
                }
            };
            let anchor = pos2((corners[2].x + corners[3].x) / 2.0, (corners[2].y + corners[3].y) / 2.0) + vec2(0.0, 26.0);
            let g = painter.layout_no_wrap(format!("{w:.2} in  x  {h:.2} in"), FontId::proportional(11.5), Color32::WHITE);
            let pill = Rect::from_center_size(anchor, g.size() + vec2(16.0, 8.0));
            painter.rect_filled(pill, 10.0, SELECT);
            painter.galley(pill.min + vec2(8.0, 4.0), g, Color32::WHITE);
            handles = Some(hs);
        }

        // typing into a text layer: a soft box and a blinking cursor
        let mut edit_carets: Vec<(Pos2, Pos2)> = Vec::new();
        if let Some(ei) = self.editing_index() {
            let it = &self.items[ei];
            let b = it.world_bounds();
            let box_r = Rect::from_min_max(to_screen(b.min), to_screen(b.max)).expand(6.0);
            painter.rect_filled(box_r, 6.0, SELECT.gamma_multiply(0.06));
            edit_carets = text_carets_world(it).into_iter().map(|(a, c)| (to_screen(a), to_screen(c))).collect();
            if let Some(te) = &self.text_edit {
                let now = ui.input(|i| i.time);
                let visible = ((now - te.since) % 1.0) < 0.6;
                if let Some((a, c)) = edit_carets.get(te.caret.min(edit_carets.len().saturating_sub(1))) {
                    if visible {
                        painter.line_segment([*a, *c], Stroke::new(3.5, Color32::WHITE));
                        painter.line_segment([*a, *c], Stroke::new(2.0, INK()));
                    }
                }
            }
            let hint = "Typing: Enter for a new line, Esc or click outside to finish";
            let g = painter.layout_no_wrap(hint.into(), FontId::proportional(12.0), Color32::WHITE);
            let pill = Rect::from_min_size(pos2(work.center().x - g.size().x / 2.0 - 12.0, work.top() + 12.0), g.size() + vec2(24.0, 10.0));
            painter.add(shadow(10, 30).as_shape(pill, CornerRadius::same(14)));
            painter.rect_filled(pill, 14.0, INK());
            painter.galley(pill.min + vec2(12.0, 5.0), g, Color32::WHITE);
            if resp.hovered() {
                if let Some(hp) = ui.input(|i| i.pointer.hover_pos()) {
                    if box_r.contains(hp) {
                        ui.ctx().set_cursor_icon(egui::CursorIcon::Text);
                    }
                }
            }
        }
        // the cursor position nearest a point on the screen
        let nearest_caret = |carets: &[(Pos2, Pos2)], p: Pos2| -> Option<usize> {
            carets
                .iter()
                .enumerate()
                .min_by(|(_, x), (_, y)| {
                    let dx = (p - pos2(x.0.x, (x.0.y + x.1.y) / 2.0)).length() + (p.y - (x.0.y + x.1.y) / 2.0).abs() * 2.0;
                    let dy = (p - pos2(y.0.x, (y.0.y + y.1.y) / 2.0)).length() + (p.y - (y.0.y + y.1.y) / 2.0).abs() * 2.0;
                    dx.partial_cmp(&dy).unwrap_or(std::cmp::Ordering::Equal)
                })
                .map(|(i, _)| i)
        };

        // snap guides while moving
        if self.drag == Drag::Move {
            let guide = Stroke::new(1.0, Color32::from_rgb(0xe8, 0x3e, 0x8c));
            for (vertical, at) in &self.snap_guides {
                if *vertical {
                    let x = to_screen(pos2(*at, 0.0)).x;
                    painter.line_segment([pos2(x, work.top()), pos2(x, work.bottom())], guide);
                } else {
                    let y = to_screen(pos2(0.0, *at)).y;
                    painter.line_segment([pos2(work.left(), y), pos2(work.right(), y)], guide);
                }
            }
        }

        // selection rectangle while dragging on empty space
        if self.drag == Drag::Marquee {
            let r = Rect::from_two_pos(self.drag_start, self.marquee_end);
            painter.rect_filled(r, 2.0, SELECT.gamma_multiply(0.08));
            painter.rect_stroke(r, 2.0, Stroke::new(1.0, SELECT), StrokeKind::Middle);
        }

        // rulers
        let top = Rect::from_min_max(full.min, pos2(full.right(), full.top() + RULER));
        let left = Rect::from_min_max(full.min, pos2(full.left() + RULER, full.bottom()));
        painter.rect_filled(top, 0.0, SURFACE());
        painter.rect_filled(left, 0.0, SURFACE());
        painter.line_segment([top.left_bottom(), top.right_bottom()], Stroke::new(1.0, BORDER()));
        painter.line_segment([left.right_top(), left.right_bottom()], Stroke::new(1.0, BORDER()));
        let tick = Stroke::new(1.0, TICK());
        // ruler marks in quarter units (inches, or centimetres in metric), numbered every unit
        let unit: f32 = if self.prefs.metric { 1.0 / 2.54 } else { 1.0 };
        let mut t = (to_world(pos2(work.left(), 0.0)).x / unit * 4.0).floor() / 4.0;
        while to_screen(pos2(t * unit, 0.0)).x <= full.right() {
            let x = to_screen(pos2(t * unit, 0.0)).x;
            if x > work.left() {
                let whole = (t - t.round()).abs() < 1e-3;
                let half = ((t * 2.0) - (t * 2.0).round()).abs() < 1e-3;
                let h = if whole { 10.0 } else if half { 6.0 } else { 3.5 };
                painter.line_segment([pos2(x, top.bottom() - h), pos2(x, top.bottom())], tick);
                if whole {
                    painter.text(pos2(x + 3.0, top.top() + 3.0), Align2::LEFT_TOP, format!("{}", t.round() as i32), FontId::proportional(10.0), MUTED());
                }
            }
            t += 0.25;
        }
        let mut t = (to_world(pos2(0.0, work.top())).y / unit * 4.0).floor() / 4.0;
        while to_screen(pos2(0.0, t * unit)).y <= full.bottom() {
            let y = to_screen(pos2(0.0, t * unit)).y;
            if y > work.top() {
                let whole = (t - t.round()).abs() < 1e-3;
                let half = ((t * 2.0) - (t * 2.0).round()).abs() < 1e-3;
                let w = if whole { 10.0 } else if half { 6.0 } else { 3.5 };
                painter.line_segment([pos2(left.right() - w, y), pos2(left.right(), y)], tick);
                if whole {
                    painter.text(pos2(left.left() + 3.0, y + 2.0), Align2::LEFT_TOP, format!("{}", t.round() as i32), FontId::proportional(10.0), MUTED());
                }
            }
            t += 0.25;
        }
        painter.rect_filled(Rect::from_min_size(full.min, vec2(RULER, RULER)), 0.0, SURFACE());
        painter.text(full.min + vec2(RULER / 2.0, RULER / 2.0), Align2::CENTER_CENTER, if self.prefs.metric { "cm" } else { "in" }, FontId::proportional(9.5), MUTED());

        // scroll pans; Ctrl+scroll (or pinch) zooms around the pointer. Either way the view stays on
        // the design: you can't scroll off into empty space.
        if resp.hovered() && self.font_picker.is_none() {
            let (scroll, zoom_d) = ui.input(|i| (i.smooth_scroll_delta, i.zoom_delta()));
            if zoom_d != 1.0 {
                if let Some(mp) = ui.input(|i| i.pointer.hover_pos()) {
                    let before = to_world(mp);
                    self.zoom = (self.zoom * zoom_d).clamp(0.25, 6.0);
                    self.pan = mp - full.min - before.to_vec2() * (BASE_PPI * self.zoom);
                }
            } else if scroll != Vec2::ZERO {
                self.pan += scroll;
            }
        }
        self.clamp_pan(full.size());

        let pointer = resp.interact_pointer_pos();
        let near = |b: Pos2| pointer.map(|a| (a - b).length() < 14.0).unwrap_or(false);
        let hit_at = |items: &[Item], w: Pos2| {
            (0..items.len()).rev().find(|&i| {
                let it = &items[i];
                let u = it.from_world_pt(w);
                it.visible && (-0.05..=1.05).contains(&u.x) && (-0.05..=1.05).contains(&u.y)
            })
        };
        let add_mod = ui.input(|i| i.modifiers.shift || i.modifiers.command);

        let now = ui.input(|i| i.time);
        if resp.clicked() {
            // while typing, a click inside the text moves the cursor; anywhere else finishes
            if let (Some(ei), Some(p)) = (self.editing_index(), pointer) {
                let b = self.items[ei].world_bounds();
                let box_r = Rect::from_min_max(to_screen(b.min), to_screen(b.max)).expand(6.0);
                if box_r.contains(p) {
                    if let (Some(c), Some(te)) = (nearest_caret(&edit_carets, p), self.text_edit.as_mut()) {
                        te.caret = c;
                        te.since = now;
                    }
                    return;
                }
                self.finish_text_edit();
            }
            if let Some([del, _, _, lock]) = handles {
                if near(del) {
                    acts.push(Act::Delete);
                    return;
                }
                if near(lock) {
                    self.lock_aspect = !self.lock_aspect;
                    return;
                }
            }
            if let Some(p) = pointer {
                match hit_at(&self.items, to_world(p)) {
                    Some(i) => self.select(i, add_mod),
                    None if !add_mod => self.selection.clear(),
                    None => {}
                }
            }
        }
        if resp.double_clicked() {
            // double-click a text layer to type straight into it, with the cursor where you clicked
            if let Some(p) = pointer {
                if let Some(i) = hit_at(&self.items, to_world(p)).filter(|&i| self.items[i].is_text()) {
                    if self.editing_index() != Some(i) {
                        self.start_text_edit(i, now);
                    }
                    let carets: Vec<(Pos2, Pos2)> =
                        text_carets_world(&self.items[i]).into_iter().map(|(a, c)| (to_screen(a), to_screen(c))).collect();
                    if let (Some(c), Some(te)) = (nearest_caret(&carets, p), self.text_edit.as_mut()) {
                        te.caret = c;
                        te.since = now;
                    }
                }
            }
        }

        if resp.drag_started() {
            let p = pointer.unwrap_or_default();
            let pan_mode = ui.input(|i| i.pointer.middle_down() || i.pointer.secondary_down() || i.key_down(Key::Space));
            self.drag = Drag::None;
            if pan_mode {
                self.drag = Drag::Pan;
            } else if let Some([_, rot, size, _]) = handles {
                if near(rot) {
                    self.drag = Drag::Rotate;
                } else if near(size) {
                    self.drag = Drag::Resize;
                }
            }
            if self.drag == Drag::None {
                let w = to_world(p);
                let in_sel_box = self.selection.len() > 1 && self.selection_bounds().expand(0.05).contains(w);
                if in_sel_box {
                    self.drag = Drag::Move;
                } else {
                    match hit_at(&self.items, w) {
                        Some(i) => {
                            if !self.selection.contains(&i) {
                                self.select(i, add_mod);
                            }
                            self.drag = Drag::Move;
                        }
                        None => {
                            self.drag = Drag::Marquee;
                            self.marquee_end = p;
                        }
                    }
                }
            }
            if matches!(self.drag, Drag::Move | Drag::Resize | Drag::Rotate) {
                self.snapshot();
                self.drag_orig = self.selection.iter().map(|&i| (i, self.items[i].clone())).collect();
                self.drag_bounds = self.selection_bounds();
            }
            self.drag_start = p;
        }

        if resp.dragged() {
            let delta = resp.drag_delta();
            let p = pointer.unwrap_or(self.drag_start);
            match self.drag {
                Drag::Pan => self.pan += delta,
                Drag::Marquee => self.marquee_end = p,
                Drag::Move => {
                    // move from where the drag began, so snapping never builds up
                    let total = (p - self.drag_start) / ppi;
                    let mut shift = total;
                    self.snap_guides.clear();
                    if self.prefs.snap && !ui.input(|i| i.modifiers.alt) {
                        let moved = self.drag_bounds.translate(total);
                        let mut targets_x: Vec<f32> = vec![0.0, 6.0, 12.0];
                        let mut targets_y: Vec<f32> = vec![0.0, 6.0, 12.0];
                        if let Some(da) = &self.design_area {
                            let r = Rect::from_min_size(DESIGN_AREA_ORIGIN, vec2(da.size[0], da.size[1]));
                            targets_x.extend([r.left(), r.center().x, r.right()]);
                            targets_y.extend([r.top(), r.center().y, r.bottom()]);
                        }
                        for (k, it) in self.items.iter().enumerate() {
                            if self.selection.contains(&k) || !it.visible {
                                continue;
                            }
                            let b = it.world_bounds();
                            targets_x.extend([b.left(), b.center().x, b.right()]);
                            targets_y.extend([b.top(), b.center().y, b.bottom()]);
                        }
                        let reach = 6.0 / ppi;
                        let best = |mine: [f32; 3], targets: &[f32]| -> Option<(f32, f32)> {
                            let mut found: Option<(f32, f32)> = None;
                            for m in mine {
                                for t in targets {
                                    let d = t - m;
                                    if d.abs() <= reach && found.map(|(fd, _)| d.abs() < fd.abs()).unwrap_or(true) {
                                        found = Some((d, *t));
                                    }
                                }
                            }
                            found
                        };
                        if let Some((d, line)) = best([moved.left(), moved.center().x, moved.right()], &targets_x) {
                            shift.x += d;
                            self.snap_guides.push((true, line));
                        }
                        if let Some((d, line)) = best([moved.top(), moved.center().y, moved.bottom()], &targets_y) {
                            shift.y += d;
                            self.snap_guides.push((false, line));
                        }
                    }
                    for (i, orig) in &self.drag_orig {
                        self.items[*i].pos[0] = orig.pos[0] + shift.x;
                        self.items[*i].pos[1] = orig.pos[1] + shift.y;
                    }
                }
                Drag::Resize => {
                    if self.drag_orig.len() == 1 {
                        let (s, orig) = self.drag_orig[0].clone();
                        let anchor = orig.to_world(pos2(0.0, 0.0));
                        let d = to_world(p) - anchor;
                        let (sn, co) = (-orig.rot).to_radians().sin_cos();
                        let local = vec2(d.x * co - d.y * sn, d.x * sn + d.y * co);
                        let mut nw = local.x.max(0.1);
                        let thin = orig.size[1] < 0.001;
                        let mut nh = if thin { orig.size[1] } else { local.y.max(0.1) };
                        if self.lock_aspect && !thin {
                            let k = (nw / orig.size[0]).max(nh / orig.size[1]);
                            nw = orig.size[0] * k;
                            nh = orig.size[1] * k;
                        }
                        let it = &mut self.items[s];
                        it.size = [nw, nh];
                        it.pos = orig.pos;
                        let fix = anchor - it.to_world(pos2(0.0, 0.0));
                        it.pos[0] += fix.x;
                        it.pos[1] += fix.y;
                    } else {
                        // several layers scale together, keeping proportions
                        let b = self.drag_bounds;
                        let w = to_world(p);
                        let k = ((w.x - b.left()) / b.width().max(1e-4)).max((w.y - b.top()) / b.height().max(1e-4)).max(0.05);
                        for (i, orig) in &self.drag_orig {
                            let it = &mut self.items[*i];
                            it.size = [orig.size[0] * k, orig.size[1] * k];
                            it.set_center(b.min + (orig.center() - b.min) * k);
                        }
                    }
                }
                Drag::Rotate => {
                    let c = self.drag_bounds.center();
                    let cs = to_screen(c);
                    let mut a = ((p - cs).angle() - (self.drag_start - cs).angle()).to_degrees();
                    if ui.input(|i| i.modifiers.shift) {
                        a = (a / 15.0).round() * 15.0;
                    }
                    let (sn, co) = a.to_radians().sin_cos();
                    for (i, orig) in &self.drag_orig {
                        let it = &mut self.items[*i];
                        let d = orig.center() - c;
                        it.rot = orig.rot + a;
                        it.size = orig.size;
                        it.set_center(c + vec2(d.x * co - d.y * sn, d.x * sn + d.y * co));
                    }
                }
                Drag::None => {}
            }
        }
        if resp.drag_stopped() {
            self.snap_guides.clear();
            if self.drag == Drag::Marquee {
                let r = Rect::from_two_pos(to_world(self.drag_start), to_world(self.marquee_end));
                let inside: Vec<usize> = (0..self.items.len()).filter(|&i| self.items[i].visible && r.contains_rect(self.items[i].world_bounds())).collect();
                let inside = self.with_groups(inside);
                if add_mod {
                    self.selection.extend(inside);
                    self.selection.sort_unstable();
                    self.selection.dedup();
                } else {
                    self.selection = inside;
                }
            }
            self.drag = Drag::None;
            self.drag_orig.clear();
        }

        // floating zoom control
        let zr = Rect::from_min_size(pos2(work.left() + 16.0, full.bottom() - 54.0), vec2(196.0, 38.0));
        ui.painter().add(shadow(12, 30).as_shape(zr, CornerRadius::same(19)));
        ui.painter().rect_filled(zr, 19.0, SURFACE());
        let mut zui = ui.new_child(egui::UiBuilder::new().max_rect(zr.shrink2(vec2(8.0, 4.0))));
        zui.horizontal_centered(|ui| {
            ui.spacing_mut().item_spacing.x = 4.0;
            let small = |t: &str| egui::Button::new(RichText::new(t).size(16.0).color(INK())).fill(Color32::TRANSPARENT).stroke(Stroke::NONE).min_size(vec2(28.0, 28.0));
            if ui.add(small("−")).on_hover_text("Zoom out (Ctrl+scroll)").clicked() {
                self.zoom = (self.zoom / 1.2).max(0.25);
            }
            ui.add_sized(vec2(48.0, 28.0), egui::Label::new(RichText::new(format!("{:.0}%", self.zoom * 100.0)).color(INK())));
            if ui.add(small("+")).on_hover_text("Zoom in").clicked() {
                self.zoom = (self.zoom * 1.2).min(6.0);
            }
            if ui
                .add(egui::Button::new(RichText::new("Fit").size(12.5).color(ACCENT_DARK())).fill(ACCENT_SOFT()).stroke(Stroke::NONE).corner_radius(CornerRadius::same(12)))
                .on_hover_text("Show the whole cutting area")
                .clicked()
            {
                let room = work.size() - vec2(80.0, 90.0);
                self.zoom = (room.x.min(room.y) / (12.0 * BASE_PPI)).clamp(0.25, 6.0);
                self.pan = vec2(RULER + 36.0, RULER + 28.0);
            }
        });
    }

    fn image_window(&mut self, ctx: &egui::Context) {
        let Some(imp) = self.image_import.as_mut() else { return };
        if imp.orig_tex.is_none() {
            let ci = egui::ColorImage::from_rgba_unmultiplied([imp.pic.w, imp.pic.h], &imp.pic.rgba);
            imp.orig_tex = Some(ctx.load_texture("upload-original", ci, egui::TextureOptions::LINEAR));
        }
        if imp.dirty || imp.lines_for_print != imp.print || imp.lines_for_engrave != imp.engrave {
            if imp.dirty {
                imp.mask = imaging::mask(&imp.pic, &imp.cleanup);
                // kept pixels in colour, removed ones as a light checkerboard
                let mut rgba = imp.pic.rgba.clone();
                for (i, keep) in imp.mask.iter().enumerate() {
                    if !keep {
                        let (x, y) = (i % imp.pic.w, i / imp.pic.w);
                        let v = if ((x / 8) + (y / 8)) % 2 == 0 { 236 } else { 250 };
                        rgba[i * 4..i * 4 + 4].copy_from_slice(&[v, v, v, 255]);
                    }
                }
                let ci = egui::ColorImage::from_rgba_unmultiplied([imp.pic.w, imp.pic.h], &rgba);
                imp.preview_tex = Some(ctx.load_texture("upload-preview", ci, egui::TextureOptions::LINEAR));
            }
            // the cut lines are worked out once per change, not every frame
            imp.lines = if imp.engrave {
                // preview of the engraving rows at the default spacing for this width
                let cell = ENGRAVE_DEFAULT_SPACING.max(0.02) * imp.pic.w as f32 / imp.width_in.max(0.1);
                imaging::photo_engrave_lines(&imaging::cleaned_rgba(&imp.pic, &imp.mask), imp.pic.w, imp.pic.h, cell, 0.0, false)
            } else if imp.print {
                imaging::print_outline(&imp.pic, &imp.mask)
            } else {
                imaging::trace(&imp.pic, &imp.mask, imp.cleanup.smooth)
            };
            imp.lines_for_print = imp.print;
            imp.lines_for_engrave = imp.engrave;
            imp.dirty = false;
        }
        let mut open = true;
        let mut apply = false;
        let mut save_svg = false;
        let svg_note = self.svg_note.clone();
        let print_area = self.prefs.page.print_area();
        // opens at about 90 % of the app window, centred, and can be resized
        let screen = ctx.content_rect();
        let win_size = vec2((screen.width() * 0.9).max(700.0), (screen.height() * 0.88).max(520.0));
        egui::Window::new(RichText::new(format!("Upload: {}", imp.name)).strong())
            .id(egui::Id::unique("upload-window"))
            .open(&mut open)
            .default_rect(Rect::from_center_size(screen.center(), win_size))
            .min_size(vec2(640.0, 420.0))
            .resizable(true)
            .collapsible(false)
            .show(ctx, |ui| {
                let full = ui.available_rect_before_wrap();
                let side_w = 290.0;
                // ---- settings column on the left
                let settings_rect = Rect::from_min_size(full.min, vec2(side_w, full.height()));
                let mut sui = ui.new_child(egui::UiBuilder::new().max_rect(settings_rect).layout(Layout::top_down(Align::Min)));
                {
                    let ui = &mut sui;
                    ui.spacing_mut().slider_width = 150.0;
                    ui.label(RichText::new("How should this picture be made?").size(13.0).strong().color(INK()));
                    let mut mode: u8 = if imp.engrave { 2 } else if imp.print { 1 } else { 0 };
                    if segmented(ui, &mut mode, &[(0u8, "Cut image"), (1u8, "Print then cut"), (2u8, "Engrave photo")]) {
                        imp.print = mode == 1;
                        imp.engrave = mode == 2;
                    }
                    ui.label(
                        RichText::new(if imp.engrave {
                            "Engraves the picture as rows of lines, closer where it's darker. Removed parts aren't engraved."
                        } else if imp.print {
                            "Prints the picture on your printer, then cuts around it."
                        } else {
                            "Only the outline is cut. Removed parts are not cut."
                        })
                        .size(12.0)
                        .color(MUTED()),
                    );
                    ui.add_space(10.0);
                    ui.label(RichText::new("Clean up").size(13.0).strong().color(INK()));
                    let before = imp.cleanup.clone();
                    ui.checkbox(&mut imp.cleanup.remove_background, "Remove the background colour");
                    if imp.cleanup.remove_background {
                        ui.add(egui::Slider::new(&mut imp.cleanup.tolerance, 5..=200).text("how much"));
                    } else {
                        ui.add(egui::Slider::new(&mut imp.cleanup.threshold, 10..=250).text("darkness kept"));
                    }
                    ui.checkbox(&mut imp.cleanup.invert, "Swap kept and removed");
                    if !imp.print {
                        ui.add(egui::Slider::new(&mut imp.cleanup.smooth, 0.3..=5.0).text("smoothing"));
                    }
                    if imp.cleanup != before {
                        imp.dirty = true;
                    }
                    ui.add_space(10.0);
                    ui.label(RichText::new("Size").size(13.0).strong().color(INK()));
                    ui.horizontal(|ui| {
                        ui.label(RichText::new("Width").color(MUTED()));
                        ui.add(egui::DragValue::new(&mut imp.width_in).speed(0.05).range(0.25..=23.0).suffix(" in"));
                        let h = imp.width_in * imp.pic.h as f32 / imp.pic.w.max(1) as f32;
                        ui.label(RichText::new(format!("x {h:.2} in")).color(MUTED()));
                    });
                    ui.add_space(6.0);
                    let summary = if imp.engrave {
                        format!("{} engraving lines in this preview. Change spacing and brightness on the canvas.", imp.lines.len())
                    } else if imp.print {
                        format!("Largest print size {:.2} x {:.2} in.", print_area.x, print_area.y)
                    } else {
                        format!("{} outline(s) will be cut.", imp.lines.len())
                    };
                    ui.label(RichText::new(summary).size(12.0).color(MUTED()));
                    ui.label(RichText::new(format!("Picture: {} x {} pixels", imp.pic.w, imp.pic.h)).size(11.5).color(MUTED()));
                    ui.add_space(12.0);
                    if primary_button(ui, "Add to canvas").clicked() {
                        apply = true;
                    }
                    ui.add_space(4.0);
                    if secondary_button(ui, "Save as SVG...").on_hover_text("Keep a cut-ready SVG copy of this picture").clicked() {
                        save_svg = true;
                    }
                    if !svg_note.is_empty() {
                        ui.label(RichText::new(svg_note.as_str()).size(12.0).color(MUTED()));
                    }
                }

                // ---- big preview on the right
                let area = Rect::from_min_max(pos2(full.left() + side_w + 14.0, full.top()), full.max);
                let bar = Rect::from_min_size(area.min, vec2(area.width(), 34.0));
                let mut bui = ui.new_child(egui::UiBuilder::new().max_rect(bar).layout(Layout::left_to_right(Align::Center)));
                {
                    let ui = &mut bui;
                    segmented(ui, &mut imp.view, &[(PreviewView::SideBySide, "Side by side"), (PreviewView::Result, "Result"), (PreviewView::Original, "Original")]);
                    ui.checkbox(&mut imp.show_lines, "Show cut lines");
                    ui.with_layout(Layout::right_to_left(Align::Center), |ui| {
                        if tool_chip(ui, "Fit", true).on_hover_text("Show the whole picture").clicked() {
                            imp.zoom = 1.0;
                            imp.pan = Vec2::ZERO;
                        }
                        if tool_chip(ui, "1:1", true).on_hover_text("One picture pixel per screen pixel").clicked() {
                            imp.zoom = -1.0; // worked out below once the pane size is known
                        }
                        if tool_chip(ui, "+", true).on_hover_text("Zoom in").clicked() {
                            imp.zoom = (imp.zoom * 1.25).min(40.0);
                        }
                        ui.label(RichText::new(format!("{:.0}%", imp.zoom.max(0.0) * 100.0)).color(INK()));
                        if tool_chip(ui, "−", true).on_hover_text("Zoom out").clicked() {
                            imp.zoom = (imp.zoom / 1.25).max(1.0);
                            if imp.zoom <= 1.0 {
                                imp.pan = Vec2::ZERO;
                            }
                        }
                        ui.label(RichText::new("Wheel to zoom, drag to move").size(11.5).color(MUTED()));
                    });
                }
                let panes_rect = Rect::from_min_max(pos2(area.left(), bar.bottom() + 8.0), area.max);
                let result_label = if imp.print { "Printed part" } else { "What will be cut" };
                let panes: Vec<(Rect, bool, &str)> = match imp.view {
                    PreviewView::SideBySide => {
                        let w = (panes_rect.width() - 10.0) / 2.0;
                        vec![
                            (Rect::from_min_size(panes_rect.min, vec2(w, panes_rect.height())), false, "Original"),
                            (Rect::from_min_size(panes_rect.min + vec2(w + 10.0, 0.0), vec2(w, panes_rect.height())), true, result_label),
                        ]
                    }
                    PreviewView::Result => vec![(panes_rect, true, result_label)],
                    PreviewView::Original => vec![(panes_rect, false, "Original")],
                };
                let img = vec2(imp.pic.w as f32, imp.pic.h as f32);
                let label_h = 20.0;
                let fit_of = |r: Rect| ((r.width() - 8.0) / img.x).min((r.height() - label_h - 8.0) / img.y).max(0.01);
                if imp.zoom < 0.0 {
                    // 1:1 - one picture pixel per screen pixel
                    imp.zoom = (1.0 / fit_of(panes[0].0)).clamp(1.0, 40.0);
                    imp.pan = Vec2::ZERO;
                }
                for (r, result, label) in &panes {
                    let resp = ui.interact(*r, egui::Id::unique(("upload-pane", *result)), Sense::drag());
                    let p = ui.painter_at(*r);
                    p.rect_filled(*r, 10.0, SUBTLE());
                    p.rect_stroke(*r, 10.0, Stroke::new(1.0, BORDER()), StrokeKind::Inside);
                    p.text(r.left_top() + vec2(10.0, 6.0), Align2::LEFT_TOP, *label, FontId::proportional(12.0), MUTED());
                    let view = Rect::from_min_max(r.min + vec2(4.0, label_h + 4.0), r.max - vec2(4.0, 4.0));
                    let s = fit_of(*r) * imp.zoom;
                    let tl = view.center() - img * s / 2.0 + imp.pan;
                    let pv = ui.painter_at(view);
                    let tex = if *result { &imp.preview_tex } else { &imp.orig_tex };
                    if let Some(t) = tex {
                        pv.image(t.id(), Rect::from_min_size(tl, img * s), Rect::from_min_max(pos2(0.0, 0.0), pos2(1.0, 1.0)), Color32::WHITE);
                    }
                    if *result && imp.show_lines {
                        let w = if s > 3.0 { 2.0 } else { 1.4 };
                        for c in &imp.lines {
                            let pts: Vec<Pos2> = c.iter().map(|q| tl + q.to_vec2() * s).collect();
                            pv.add(Shape::line(pts, Stroke::new(w, DANGER)));
                        }
                    }
                    // wheel zooms around the pointer; dragging moves the view (both panes together)
                    if resp.hovered() {
                        let (scroll, zd, hover) = ui.input(|i| (i.smooth_scroll_delta.y, i.zoom_delta(), i.pointer.hover_pos()));
                        let factor = if zd != 1.0 { zd } else { (scroll * 0.0025).exp() };
                        if factor != 1.0 {
                            if let Some(mp) = hover {
                                let q = (mp - tl) / s; // picture point under the pointer
                                let new_zoom = (imp.zoom * factor).clamp(1.0, 40.0);
                                let s2 = fit_of(*r) * new_zoom;
                                let tl2 = mp - q * s2;
                                imp.pan = tl2 - (view.center() - img * s2 / 2.0);
                                imp.zoom = new_zoom;
                            }
                        }
                        if imp.zoom > 1.0 {
                            ui.ctx().set_cursor_icon(if resp.dragged() { egui::CursorIcon::Grabbing } else { egui::CursorIcon::Grab });
                        }
                    }
                    if resp.dragged() {
                        imp.pan += resp.drag_delta();
                    }
                    // keep some of the picture in view
                    let size = img * fit_of(*r) * imp.zoom;
                    let limit = ((size - view.size()) / 2.0).max(Vec2::ZERO) + view.size() * 0.25;
                    imp.pan = imp.pan.clamp(-limit, limit);
                    if imp.zoom <= 1.0 {
                        imp.pan = Vec2::ZERO;
                    }
                }
                ui.allocate_rect(full, Sense::hover());
            });
        if save_svg {
            match imaging::picture_to_svg(&imp.pic, &imp.cleanup, imp.width_in, imp.print) {
                Ok(svg) => {
                    let picked = rfd::FileDialog::new()
                        .set_title("Save as SVG")
                        .set_file_name(format!("{}.svg", imp.name))
                        .add_filter("SVG", &["svg"])
                        .save_file();
                    if let Some(path) = picked {
                        self.svg_note = match std::fs::write(&path, svg) {
                            Ok(()) => format!("Saved {}", path.display()),
                            Err(e) => format!("Couldn't save: {e}"),
                        };
                    }
                }
                Err(e) => self.svg_note = e,
            }
        }
        if apply {
            self.svg_note.clear();
            self.finish_image_import();
            self.next_upload();
        } else if !open {
            self.svg_note.clear();
            self.image_import = None;
            self.next_upload();
        }
    }

    fn offset_window(&mut self, ctx: &egui::Context) {
        let Some(t) = self.offset_tool.as_mut() else { return };
        let mut open = true;
        let mut apply = false;
        egui::Window::new(RichText::new("Offset").strong())
            .open(&mut open)
            .default_width(300.0)
            .default_pos(ctx.content_rect().center() + vec2(-150.0, -120.0))
            .collapsible(false)
            .show(ctx, |ui| {
                ui.label(RichText::new("An outline around (or inside) the selection, shown dashed on the canvas.").size(12.5).color(MUTED()));
                ui.add(egui::Slider::new(&mut t.distance, -1.0..=1.0).text("distance (in)").fixed_decimals(3));
                ui.label(RichText::new("Corners are rounded.").size(12.0).color(MUTED()));
                if primary_button(ui, "Apply").clicked() {
                    apply = true;
                }
            });
        if apply {
            let d = t.distance;
            self.offset_tool = None;
            if self.selection.is_empty() {
                return;
            }
            let parts: Vec<Vec<Contour>> = self.selection.iter().map(|&i| self.items[i].world_contours()).collect();
            let welded = geometry::weld(&parts);
            let out = geometry::offset_contours(&welded, d);
            let c = self.next_color();
            if let Some(it) = Item::from_world("Offset", out, [c.r(), c.g(), c.b()], Operation::Cut) {
                self.snapshot();
                let at = *self.selection.iter().min().unwrap_or(&0);
                self.items.insert(at, it);
                self.selection = vec![at];
            }
        } else if !open {
            self.offset_tool = None;
        }
    }

    fn contour_window(&mut self, ctx: &egui::Context) {
        let Some(idx) = self.contour_item else { return };
        if idx >= self.items.len() {
            self.contour_item = None;
            return;
        }
        let mut open = true;
        let mut changed: Option<Vec<bool>> = None;
        egui::Window::new(RichText::new("Contour").strong())
            .open(&mut open)
            .default_width(320.0)
            .default_pos(ctx.content_rect().right_top() + vec2(-640.0, 140.0))
            .collapsible(false)
            .show(ctx, |ui| {
                let it = &self.items[idx];
                let n = it.unit.len();
                let mut hidden: Vec<bool> = (0..n).map(|i| it.is_hidden(i)).collect();
                ui.label(RichText::new("Untick a shape to leave it out of the cut. Hidden shapes show dashed.").size(12.5).color(MUTED()));
                ui.horizontal(|ui| {
                    if secondary_button(ui, "Show all").clicked() {
                        hidden = vec![false; n];
                    }
                    if secondary_button(ui, "Hide inner shapes").clicked() {
                        // keep the shape(s) not inside any other shape's bounds
                        let bounds: Vec<Rect> = it.unit.iter().map(|c| {
                            let (a, b) = geometry::bounds(c.iter().copied());
                            Rect::from_min_max(a, b)
                        }).collect();
                        for i in 0..n {
                            hidden[i] = (0..n).any(|j| j != i && bounds[j].contains_rect(bounds[i]) && bounds[j] != bounds[i]);
                        }
                    }
                });
                egui::ScrollArea::vertical().max_height(320.0).show(ui, |ui| {
                    egui::Grid::new("contours").num_columns(4).spacing(vec2(6.0, 6.0)).show(ui, |ui| {
                        for i in 0..n {
                            ui.vertical(|ui| {
                                let (r, resp) = ui.allocate_exact_size(vec2(58.0, 58.0), Sense::click());
                                ui.painter().rect_filled(r, 8.0, if hidden[i] { SUBTLE() } else { ACCENT_SOFT() });
                                ui.painter().rect_stroke(r, 8.0, Stroke::new(1.0, if hidden[i] { BORDER() } else { ACCENT }), StrokeKind::Inside);
                                let (a, b) = geometry::bounds(it.unit[i].iter().copied());
                                let ext = (b - a).max(Vec2::splat(1e-4));
                                let k = 42.0 / (ext.x * it.size[0]).max(ext.y * it.size[1]).max(1e-4);
                                let pts: Vec<Pos2> = it.unit[i]
                                    .iter()
                                    .map(|p| r.center() + vec2((p.x - (a.x + b.x) / 2.0) * it.size[0] * k, (p.y - (a.y + b.y) / 2.0) * it.size[1] * k))
                                    .collect();
                                if hidden[i] {
                                    ui.painter().extend(Shape::dashed_line(&pts, Stroke::new(1.2, MUTED()), 3.0, 2.0));
                                } else {
                                    ui.painter().add(Shape::line(pts, Stroke::new(1.4, it.color32())));
                                }
                                if resp.clicked() {
                                    hidden[i] = !hidden[i];
                                }
                            });
                            if i % 4 == 3 {
                                ui.end_row();
                            }
                        }
                    });
                });
                if hidden != (0..n).map(|i| it.is_hidden(i)).collect::<Vec<_>>() {
                    changed = Some(hidden);
                }
            });
        if let Some(h) = changed {
            self.snapshot();
            self.items[idx].hidden = h;
        }
        if !open {
            self.contour_item = None;
        }
    }

    fn make_screen(&mut self, ui: &mut Ui) {
        let mats = self.build_mats();
        if self.mat_index >= mats.len() {
            self.mat_index = 0;
        }
        egui::Panel::left("mats")
            .exact_size(210.0)
            .resizable(false)
            .show_separator_line(false)
            .frame(egui::Frame::NONE.fill(SURFACE()).stroke(Stroke::new(1.0, BORDER())).inner_margin(Margin::symmetric(14, 16)))
            .show(ui, |ui| {
                ui.horizontal(|ui| {
                    ui.label(RichText::new("Mats").size(17.0).strong().color(INK()));
                    badge(ui, &mats.len().to_string(), MUTED());
                });
                ui.label(RichText::new("One mat for each cut colour; print then cut on its own.").size(12.5).color(MUTED()));
                ui.add_space(8.0);
                egui::ScrollArea::vertical().show(ui, |ui| {
                    ui.spacing_mut().item_spacing.y = 10.0;
                    for (n, m) in mats.iter().enumerate() {
                        let ms = if m.kind == MatKind::Print { self.print_area() } else { self.mat_size() };
                        let thumb_h = 120.0 * (ms.y / ms.x).min(1.6);
                        let (r, resp) = ui.allocate_exact_size(vec2(ui.available_width(), thumb_h + 42.0), Sense::click());
                        let sel = n == self.mat_index;
                        if sel || resp.hovered() {
                            ui.painter().add(shadow(10, 20).as_shape(r, CornerRadius::same(12)));
                        }
                        ui.painter().rect_filled(r, 12.0, SURFACE());
                        ui.painter().rect_stroke(r, 12.0, Stroke::new(if sel { 2.0 } else { 1.0 }, if sel { ACCENT } else { BORDER() }), StrokeKind::Inside);
                        let k = (thumb_h / ms.y).min(150.0 / ms.x);
                        let mr = Rect::from_center_size(pos2(r.center().x, r.top() + 10.0 + thumb_h / 2.0), ms * k);
                        ui.painter().rect_filled(mr, 3.0, if m.kind == MatKind::Print { MAT_PRINT } else { MAT_STICKY });
                        let to_s = |p: Pos2| mr.min + p.to_vec2() * k;
                        for (i, off) in &m.pieces {
                            let it = &self.items[*i];
                            draw_item(ui.painter(), it, &to_s, *off, None, self.textures.get(&it.uid).map(|t| t.id()), self.engrave_cache.get(&it.uid).map(|e| e.1.as_slice()));
                        }
                        let y = r.bottom() - 16.0;
                        ui.painter().circle_filled(pos2(r.left() + 18.0, y), 6.0, m.color);
                        let title = if m.kind == MatKind::Print { format!("Mat {}  print", n + 1) } else { format!("Mat {}", n + 1) };
                        ui.painter().text(pos2(r.left() + 30.0, y), Align2::LEFT_CENTER, title, FontId::proportional(13.0), INK());
                        ui.painter().text(
                            pos2(r.right() - 12.0, y),
                            Align2::RIGHT_CENTER,
                            format!("{} piece(s)", m.pieces.len()),
                            FontId::proportional(11.0),
                            if m.overflow { DANGER } else { MUTED() },
                        );
                        if resp.on_hover_cursor(egui::CursorIcon::PointingHand).clicked() {
                            self.mat_index = n;
                        }
                    }
                });
            });

        egui::Panel::right("make_settings")
            .exact_size(320.0)
            .resizable(false)
            .show_separator_line(false)
            .frame(egui::Frame::NONE.fill(BG()).inner_margin(Margin::symmetric(14, 16)))
            .show(ui, |ui| {
                egui::ScrollArea::vertical().show(ui, |ui| {
                    ui.spacing_mut().item_spacing.y = 12.0;
                    let all = materials::all_materials(&self.prefs.custom_materials);
                    if self.material >= all.len() {
                        self.material = 0;
                    }
                    let m = all[self.material].clone();
                    card(ui, |ui| {
                        ui.label(RichText::new("Material").size(15.0).strong().color(INK()));
                        egui::ComboBox::from_id_salt("material").selected_text(m.name.as_str()).width(ui.available_width()).show_ui(ui, |ui| {
                            for (i, mm) in all.iter().enumerate() {
                                let label = if mm.custom { format!("{} (yours)", mm.name) } else { mm.name.clone() };
                                ui.selectable_value(&mut self.material, i, label);
                            }
                        });
                        let m = all[self.material].clone();
                        ui.add_space(4.0);
                        ui.horizontal(|ui| {
                            ui.label(RichText::new("Pressure").size(12.5).color(MUTED()));
                            let (r, _) = ui.allocate_exact_size(vec2(140.0, 10.0), Sense::hover());
                            for k in 0..7 {
                                let seg = Rect::from_min_size(r.min + vec2(k as f32 * 20.0, 0.0), vec2(17.0, 8.0));
                                ui.painter().rect_filled(seg, 3.0, if (k as u8) < m.pressure { ACCENT } else { BORDER() });
                            }
                        });
                        ui.horizontal(|ui| {
                            ui.label(RichText::new("Passes").size(12.5).color(MUTED()));
                            badge(ui, &m.passes.to_string(), INK());
                        });
                    });
                    let m = all[self.material].clone();
                    let engraving = mats.get(self.mat_index).map(|mt| mt.pieces.iter().any(|(i, _)| self.items[*i].op == Operation::Engrave)).unwrap_or(false);
                    if engraving {
                        // the original's three engraving notes, in our own words
                        card(ui, |ui| {
                            ui.label(RichText::new("Before you engrave").size(15.0).strong().color(INK()));
                            for (t, d) in [
                                ("Use the right supplies", "Engravable material such as aluminum, acrylic or leather."),
                                (
                                    if self.prefs.model.engraving_tool() { "Load the Engraving Tool" } else { "Load the Engrave Tip" },
                                    if self.prefs.model.engraving_tool() {
                                        "Your machine takes the separate Engraving Tool."
                                    } else {
                                        "It goes on a QuickSwap housing in clamp B."
                                    },
                                ),
                                ("Secure your material", "Place it on the mat where it sits on screen and tape all four sides so it can't shift."),
                            ] {
                                ui.label(RichText::new(t).strong().color(INK()));
                                ui.label(RichText::new(d).size(12.5).color(MUTED()));
                            }
                        });
                    }
                    card(ui, |ui| {
                        ui.label(RichText::new("Load these tools").size(15.0).strong().color(INK()));
                        let mut tools: Vec<(&'static str, Color32)> = Vec::new();
                        if let Some(mat) = mats.get(self.mat_index) {
                            for op in [
                                Operation::Draw,
                                Operation::Score,
                                Operation::Engrave,
                                Operation::Deboss,
                                Operation::Perforate,
                                Operation::Wave,
                                Operation::Cut,
                                Operation::Print,
                            ] {
                                if mat.pieces.iter().any(|(i, _)| self.items[*i].op == op) {
                                    let t = materials::tool_for_op(op, m.tool, self.prefs.model.engraving_tool()).label();
                                    if !tools.iter().any(|(x, _)| *x == t) {
                                        tools.push((t, op_color(op)));
                                    }
                                }
                            }
                        }
                        if tools.is_empty() {
                            tools.push((m.tool.label(), ACCENT));
                        }
                        for (n, (t, c)) in tools.iter().enumerate() {
                            ui.horizontal(|ui| {
                                let (r, _) = ui.allocate_exact_size(vec2(22.0, 22.0), Sense::hover());
                                ui.painter().circle_filled(r.center(), 11.0, c.gamma_multiply(0.15));
                                ui.painter().text(r.center(), Align2::CENTER_CENTER, format!("{}", n + 1), FontId::proportional(11.5), *c);
                                ui.label(RichText::new(*t).color(INK()));
                            });
                        }
                        if mats.get(self.mat_index).map(|m| m.kind == MatKind::Print).unwrap_or(false) {
                            ui.label(RichText::new("Print this mat first (save it as SVG and print at 100 %), then put the paper on the mat.").size(12.0).color(MUTED()));
                        }
                    });
                    card(ui, |ui| {
                        ui.label(RichText::new("Mat").size(15.0).strong().color(INK()));
                        let before = self.prefs.load;
                        let mut options: Vec<(projects::LoadType, &str)> = vec![
                            (projects::LoadType::Mat12, "12 x 12 in"),
                            (projects::LoadType::Mat24, "12 x 24 in"),
                            (projects::LoadType::CardMat, "Card mat"),
                        ];
                        if self.prefs.model.matless() {
                            options.push((projects::LoadType::None, "No mat"));
                        }
                        segmented(ui, &mut self.prefs.load, &options);
                        if self.prefs.load != before {
                            self.save_prefs();
                        }
                        ui.checkbox(&mut self.mirror, "Mirror the design (for iron-on)");
                        if self.project_type.keeps_layout() {
                            ui.label(RichText::new("This project keeps every layer where you placed it.").size(12.0).color(MUTED()));
                        }
                    });
                    card(ui, |ui| {
                        ui.label(RichText::new("Send to machine").size(15.0).strong().color(INK()));
                        let ready = self.connected.is_some() && !mats.is_empty();
                        let go = ui.add_enabled(
                            ready,
                            egui::Button::new(RichText::new("Go").color(Color32::WHITE).strong().size(16.0))
                                .fill(ACCENT)
                                .stroke(Stroke::NONE)
                                .corner_radius(CornerRadius::same(12))
                                .min_size(vec2(ui.available_width(), 42.0)),
                        );
                        if go.clicked() {
                            self.log.push("Go: wake the machine, clear errors and home the blade. Streaming the cut paths themselves is not decoded yet.".into());
                            self.send(Cmd::SendZeros);
                            self.send(Cmd::ClearError);
                            self.send(Cmd::HomeTheta);
                            self.show_machine = true;
                        }
                        if self.connected.is_none() && secondary_button(ui, "Connect machine").clicked() {
                            self.show_machine = true;
                        }
                        if secondary_button(ui, "Save mats as SVG files").clicked() {
                            let dir = documents_dir();
                            let _ = std::fs::create_dir_all(&dir);
                            let mut saved = 0;
                            for (n, m) in mats.iter().enumerate() {
                                let p = dir.join(format!("{}-mat{}.svg", self.project_name.replace(['/', '\\'], "_"), n + 1));
                                if std::fs::write(&p, self.mat_svg(m)).is_ok() {
                                    saved += 1;
                                }
                            }
                            self.status = format!("Saved {saved} mat file(s) to {}", dir.display());
                        }
                        let note = if !self.status.is_empty() {
                            self.status.clone()
                        } else if self.connected.is_none() {
                            "Connect your machine to enable Go.".to_string()
                        } else {
                            "Ready.".to_string()
                        };
                        ui.label(RichText::new(note).size(12.5).color(MUTED()));
                    });
                });
            });

        egui::CentralPanel::no_frame().frame(egui::Frame::NONE.fill(BG()).inner_margin(24)).show(ui, |ui| {
            let avail = ui.available_rect_before_wrap();
            let print = mats.get(self.mat_index).map(|m| m.kind == MatKind::Print).unwrap_or(false);
            let ms = if print { self.print_area() } else { self.mat_size() };
            let title_h = 34.0;
            let k = ((avail.height() - title_h - 40.0) / (ms.y + 1.0)).min((avail.width() - 20.0) / (ms.x + 1.0)).max(4.0);
            let mat_rect = Rect::from_center_size(avail.center() + vec2(0.0, title_h / 2.0), (ms + vec2(1.0, 1.0)) * k);
            let p = ui.painter();
            if let Some(m) = mats.get(self.mat_index) {
                p.circle_filled(pos2(mat_rect.left() + 7.0, mat_rect.top() - 20.0), 6.0, m.color);
                let what = if print { "Print then cut page".to_string() } else { MATERIALS[self.material].name.to_string() };
                p.text(
                    pos2(mat_rect.left() + 20.0, mat_rect.top() - 20.0),
                    Align2::LEFT_CENTER,
                    format!("Mat {} of {}  ·  {}", self.mat_index + 1, mats.len(), what),
                    FontId::proportional(15.0),
                    INK(),
                );
            }
            p.add(Shadow { offset: [0, 6], blur: 24, spread: 0, color: Color32::from_black_alpha(35) }.as_shape(mat_rect, CornerRadius::same(10)));
            p.rect_filled(mat_rect, 10.0, if print { SURFACE() } else { MAT_BLUE });
            let sticky = Rect::from_min_size(mat_rect.min + vec2(0.5, 0.5) * k, ms * k);
            p.rect_filled(sticky, 2.0, if print { MAT_PRINT } else { MAT_STICKY });
            let grid = Stroke::new(1.0, if print { BORDER() } else { Color32::from_rgba_unmultiplied(255, 255, 255, 150) });
            let label_col = if print { MUTED() } else { Color32::WHITE };
            for i in 0..=(ms.x as i32) {
                let x = sticky.left() + i as f32 * k;
                p.line_segment([pos2(x, sticky.top()), pos2(x, sticky.bottom())], grid);
                if i < ms.x as i32 {
                    p.text(pos2(x + k / 2.0, mat_rect.top() + k * 0.25), Align2::CENTER_CENTER, format!("{}", i + 1), FontId::proportional(10.0), label_col);
                }
            }
            for j in 0..=(ms.y as i32) {
                let y = sticky.top() + j as f32 * k;
                p.line_segment([pos2(sticky.left(), y), pos2(sticky.right(), y)], grid);
                if j < ms.y as i32 {
                    p.text(pos2(mat_rect.left() + k * 0.25, y + k / 2.0), Align2::CENTER_CENTER, format!("{}", j + 1), FontId::proportional(10.0), label_col);
                }
            }
            if print {
                // registration marks the machine's sensor looks for (drawn as corner brackets here)
                let st = Stroke::new(3.0, INK());
                let a = sticky.shrink(4.0);
                for (c, dx, dy) in [(a.left_top(), 1.0, 1.0), (a.right_top(), -1.0, 1.0), (a.left_bottom(), 1.0, -1.0)] {
                    p.line_segment([c, c + vec2(dx * k * 0.6, 0.0)], st);
                    p.line_segment([c, c + vec2(0.0, dy * k * 0.6)], st);
                }
            }
            match mats.get(self.mat_index) {
                Some(m) => {
                    let to_s = |q: Pos2| sticky.min + q.to_vec2() * k;
                    let mirror = if self.mirror { Some(ms.x) } else { None };
                    for (i, off) in &m.pieces {
                        let it = &self.items[*i];
                        draw_item(p, it, &to_s, *off, mirror, self.textures.get(&it.uid).map(|t| t.id()), self.engrave_cache.get(&it.uid).map(|e| e.1.as_slice()));
                    }
                    if m.overflow {
                        let msg = "Some pieces don't fit on this mat. Try the 12 x 24 in mat or make them smaller.";
                        let g = p.layout_no_wrap(msg.into(), FontId::proportional(13.0), Color32::WHITE);
                        let pill = Rect::from_center_size(mat_rect.center_bottom() + vec2(0.0, 22.0), g.size() + vec2(20.0, 10.0));
                        p.rect_filled(pill, 12.0, DANGER);
                        p.galley(pill.min + vec2(10.0, 5.0), g, Color32::WHITE);
                    }
                }
                None => {
                    p.text(mat_rect.center(), Align2::CENTER_CENTER, "Nothing to cut yet", FontId::proportional(18.0), Color32::WHITE);
                }
            }
        });
    }

    fn machine_window(&mut self, ctx: &egui::Context) {
        let mut open = self.show_machine;
        egui::Window::new(RichText::new("Machine").strong())
            .open(&mut open)
            .default_width(470.0)
            .default_pos(ctx.content_rect().right_top() + vec2(-510.0, 74.0))
            .show(ctx, |ui| {
                let (dot, title, sub) = match &self.connected {
                    Some(n) => (ACCENT, format!("Connected to {n}"), "Ready for commands.".to_string()),
                    None => (DANGER, "Not connected".to_string(), "Turn the machine on, then connect.".to_string()),
                };
                egui::Frame::NONE.fill(dot.gamma_multiply(0.10)).corner_radius(10).inner_margin(10).show(ui, |ui| {
                    ui.set_width(ui.available_width());
                    ui.horizontal(|ui| {
                        let (r, _) = ui.allocate_exact_size(vec2(14.0, 14.0), Sense::hover());
                        ui.painter().circle_filled(r.center(), 6.0, dot);
                        ui.vertical(|ui| {
                            ui.label(RichText::new(title).strong().color(INK()));
                            ui.label(RichText::new(sub).size(12.0).color(MUTED()));
                        });
                    });
                });
                ui.add_space(4.0);
                segmented(ui, &mut self.settings.link, &[(LinkKind::Bluetooth, "Bluetooth (Linux)"), (LinkKind::Usb, "USB cable"), (LinkKind::Device, "Serial device"), (LinkKind::Demo, "Demo machine")]);
                ui.add_space(2.0);
                match self.settings.link {
                    LinkKind::Bluetooth => {
                        egui::Grid::new("bt").num_columns(2).spacing(vec2(12.0, 8.0)).show(ui, |ui| {
                            ui.label(RichText::new("Address").color(MUTED()));
                            ui.add(egui::TextEdit::singleline(&mut self.settings.mac).desired_width(200.0).font(egui::TextStyle::Monospace));
                            ui.end_row();
                            ui.label(RichText::new("Channel").color(MUTED()));
                            ui.add(egui::DragValue::new(&mut self.settings.channel).range(1..=30));
                            ui.end_row();
                        });
                        if let Some(name) = machine::bluetooth_name(&self.settings.mac) {
                            ui.label(RichText::new(format!("In the Bluetooth list it shows up as \"{name}\" (a Microchip RN42 module).")).size(12.0).color(MUTED()));
                        }
                        egui::Frame::NONE.fill(SUBTLE()).corner_radius(8).inner_margin(10).show(ui, |ui| {
                            ui.set_width(ui.available_width());
                            ui.label(RichText::new("First time on Bazzite").size(12.0).strong().color(INK()));
                            ui.label(RichText::new("Turn on Bluetooth and the machine, then select Pair and trust now.").size(11.5).color(MUTED()));
                        });
                        #[cfg(target_os = "linux")]
                        if self.pairing {
                            ui.horizontal(|ui| {
                                ui.spinner();
                                ui.label("Pairing with the machine…");
                            });
                        } else {
                            if secondary_button(ui, "Pair and trust now").clicked() {
                                self.pairing_result = None;
                                self.pairing = self.machine.tx.send(Request::Pair {
                                    mac: self.settings.mac.clone(),
                                }).is_ok();
                                if !self.pairing {
                                    self.pairing_result = Some("Restart MatCraft to try pairing again.".into());
                                }
                            }
                            if let Some(message) = &self.pairing_result {
                                ui.label(message);
                            }
                        }
                    }
                    LinkKind::Usb => {
                        ui.horizontal(|ui| {
                            ui.label(RichText::new("Port").color(MUTED()));
                            let shown = if self.settings.usb_port.is_empty() { "Find automatically".to_string() } else { self.settings.usb_port.clone() };
                            egui::ComboBox::from_id_salt("usb_port").selected_text(shown).width(220.0).show_ui(ui, |ui| {
                                ui.selectable_value(&mut self.settings.usb_port, String::new(), "Find automatically");
                                for m in machine::find_usb_machines() {
                                    let label = format!("{} (machine {:04X})", m.port, m.product_id);
                                    ui.selectable_value(&mut self.settings.usb_port, m.port.clone(), label);
                                }
                            });
                            if secondary_button(ui, "Test USB").clicked() {
                                let lines = machine::probe_usb(&self.settings.usb_port, std::time::Duration::from_secs(5));
                                self.log.extend(lines);
                            }
                        });
                        let found = machine::find_usb_machines();
                        ui.label(
                            RichText::new(if found.is_empty() {
                                "No machine on USB right now. Plug in the cable and turn the machine on.".to_string()
                            } else {
                                format!("Machine plugged in on {}.", found.iter().map(|m| m.port.as_str()).collect::<Vec<_>>().join(", "))
                            })
                            .size(12.0)
                            .color(MUTED()),
                        );
                        #[cfg(target_os = "linux")]
                        ui.label(RichText::new("On Bazzite, if the port won't open, add your account to the dialout group, then log out and back in.").size(11.5).color(MUTED()));
                    }
                    LinkKind::Device => {
                        ui.horizontal(|ui| {
                            ui.label(RichText::new("Device").color(MUTED()));
                            ui.add(egui::TextEdit::singleline(&mut self.settings.device_path).font(egui::TextStyle::Monospace));
                        });
                        ui.label(RichText::new(format!("On Linux: sudo rfcomm bind 0 {} {}  (then use /dev/rfcomm0)", self.settings.mac, self.settings.channel)).monospace().size(11.0).color(MUTED()));
                    }
                    LinkKind::Demo => {
                        ui.label(RichText::new("A pretend machine that answers every command, to try the app.").color(MUTED()));
                    }
                }
                ui.collapsing("Encryption", |ui| {
                    ui.label(
                        RichText::new("The machine can switch to encrypted messages (AES-256). If yours does, paste the 64-character session key here; leave it empty to talk unencrypted.")
                            .size(12.5)
                            .color(MUTED()),
                    );
                    ui.add(egui::TextEdit::singleline(&mut self.settings.key_hex).password(true).desired_width(f32::INFINITY));
                });
                ui.collapsing("Serial number & activation", |ui| {
                    let usb = machine::find_usb_machines();
                    match usb.iter().find(|m| !m.serial.is_empty()) {
                        Some(m) => {
                            ui.horizontal(|ui| {
                                ui.label(RichText::new("Machine serial").color(MUTED()));
                                ui.label(RichText::new(&m.serial).monospace().strong().color(INK()));
                            });
                        }
                        None => {
                            ui.label(RichText::new("Plug the machine in by USB to read its serial number.").size(12.0).color(MUTED()));
                        }
                    }
                    ui.label(
                        RichText::new(
                            "The maker's own software keeps a list of serial numbers it has switched off (for example after a warranty swap) and refuses those machines. That check lives in the software on the computer, not in the machine. MatCraft has no such list: it talks to whatever machine is on the link, so a machine the maker marked off still works here.",
                        )
                        .size(12.0)
                        .color(MUTED()),
                    );
                    ui.label(
                        RichText::new(
                            "MatCraft never pretends to be another machine. Using someone else's serial number would stop their machine working, so it isn't offered.",
                        )
                        .size(11.5)
                        .italics()
                        .color(MUTED()),
                    );
                });
                ui.horizontal(|ui| {
                    if self.connected.is_some() {
                        if secondary_button(ui, "Disconnect").clicked() {
                            let _ = self.machine.tx.send(Request::Disconnect);
                        }
                    } else if primary_button(ui, "Connect").clicked() {
                        self.connect();
                    }
                    if let Some((cmd, st)) = self.last_status {
                        ui.label(RichText::new(format!("Last reply: {} -> {}", cmd.label(), st.map(|s| s.to_string()).unwrap_or_else(|| "-".into()))).size(12.0).color(MUTED()));
                    }
                });
                ui.add_space(4.0);
                ui.horizontal(|ui| {
                    ui.label(RichText::new("Cut pressure").size(12.0).color(INK()));
                    if self.calibration.is_empty() {
                        if ui.add(egui::DragValue::new(&mut self.settings.pressure).range(0..=3000).speed(5.0)).changed() {
                            self.save_settings();
                        }
                        ui.label(
                            RichText::new("raw machine counts — calibration not read; connect to read it, or tune by hand (0 = move only)")
                                .size(11.0)
                                .color(MUTED()),
                        );
                    } else {
                        let lo = self.calibration.first().map(|r| r.0).unwrap_or(0);
                        let hi = self.calibration.last().map(|r| r.0).unwrap_or(400);
                        if ui.add(egui::DragValue::new(&mut self.settings.pressure_grams).range(lo..=hi).speed(2.0).suffix(" g")).changed() {
                            self.save_settings();
                        }
                        let counts = machine::grams_to_counts(&self.calibration, self.settings.pressure_grams).unwrap_or(0);
                        ui.label(
                            RichText::new(format!("grams (machine range {lo}–{hi}) → {counts} counts, from the machine's own calibration"))
                                .size(11.0)
                                .color(MUTED()),
                        );
                    }
                });
                ui.add_enabled_ui(self.connected.is_some(), |ui| {
                    ui.horizontal(|ui| {
                        if primary_button(ui, "Cut design on machine").clicked() {
                            self.cut_on_machine();
                        }
                        if secondary_button(ui, "Test encrypted handshake").clicked() {
                            let _ = self.machine.tx.send(Request::TestHandshake);
                        }
                        let n = self.machine_cut_records().len();
                        ui.label(
                            RichText::new(format!(
                                "{n} moves · press Go on the machine when it asks, then watch the log below"
                            ))
                            .size(12.0)
                            .color(MUTED()),
                        );
                    });
                });
                ui.add_space(4.0);
                ui.label(RichText::new("Commands").size(13.0).strong().color(INK()));
                ui.add_enabled_ui(self.connected.is_some(), |ui| {
                    ui.horizontal_wrapped(|ui| {
                        for cmd in [Cmd::SendZeros, Cmd::GetInterfaceState, Cmd::ReadErrorSeverity, Cmd::ClearError, Cmd::HomeTheta, Cmd::ScanForEdge, Cmd::GetPersistentValueCount, Cmd::NewAes] {
                            if secondary_button(ui, cmd.label()).clicked() {
                                self.send(cmd);
                            }
                        }
                    });
                    ui.horizontal(|ui| {
                        ui.add(egui::DragValue::new(&mut self.theta).range(-360.0..=360.0).suffix(" deg"));
                        if secondary_button(ui, Cmd::Move4D.label()).clicked() {
                            self.send(Cmd::Move4D);
                        }
                        if ui
                            .add(
                                egui::Button::new(RichText::new(Cmd::AbortCutting.label()).color(Color32::WHITE).strong())
                                    .fill(DANGER)
                                    .stroke(Stroke::NONE)
                                    .corner_radius(CornerRadius::same(10))
                                    .min_size(vec2(0.0, 32.0)),
                            )
                            .clicked()
                        {
                            self.send(Cmd::AbortCutting);
                        }
                    });
                });
                ui.add_space(4.0);
                egui::Frame::NONE.fill(HEADER).corner_radius(10).inner_margin(10).show(ui, |ui| {
                    ui.set_width(ui.available_width());
                    egui::ScrollArea::vertical().max_height(160.0).stick_to_bottom(true).show(ui, |ui| {
                        if self.log.is_empty() {
                            ui.label(RichText::new("Messages to and from the machine appear here, and are saved to machine-log.txt in the settings folder.").monospace().size(11.0).color(Color32::from_gray(140)));
                        }
                        for l in &self.log {
                            let col = if l.starts_with('→') {
                                Color32::from_rgb(0x9a, 0xd8, 0xff)
                            } else if l.starts_with('←') {
                                Color32::from_rgb(0x8f, 0xe3, 0xb8)
                            } else {
                                Color32::from_gray(205)
                            };
                            ui.label(RichText::new(l).monospace().size(11.0).color(col));
                        }
                    });
                });
            });
        if !open {
            self.show_machine = false;
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn key(k: Key) -> egui::Event {
        egui::Event::Key { key: k, physical_key: None, pressed: true, repeat: false, modifiers: egui::Modifiers::NONE }
    }

    #[test]
    fn scrolling_stays_on_the_design() {
        let content = Rect::from_min_max(pos2(-1.0, -1.0), pos2(13.0, 13.0)); // 14 in square
        let lo = vec2(24.0, 24.0);
        // zoomed in: content (14 in x 100 px = 1400 px) is bigger than an 800 px view
        let far = clamped_pan(vec2(5000.0, -9000.0), content, lo, vec2(800.0, 800.0), 100.0);
        assert_eq!(far.x, 24.0 + 100.0); // can't scroll past the content's left edge
        assert_eq!(far.y, 800.0 - 1300.0); // or below its bottom edge
        // zoomed out: content (14 in x 20 px = 280 px) fits, so it can't leave the view
        let small = clamped_pan(vec2(5000.0, -9000.0), content, lo, vec2(800.0, 800.0), 20.0);
        assert_eq!(small.x, 800.0 - 13.0 * 20.0);
        assert_eq!(small.y, 24.0 + 20.0);
    }

    #[test]
    fn typing_on_the_canvas() {
        let mut chars: Vec<char> = "Hllo".chars().collect();
        let mut caret = 1;
        apply_typing(&mut chars, &mut caret, &[egui::Event::Text("e".into())]);
        assert_eq!(chars.iter().collect::<String>(), "Hello");
        assert_eq!(caret, 2);
        // to the end, new line, more text
        apply_typing(&mut chars, &mut caret, &[key(Key::End), key(Key::Enter), egui::Event::Text("you".into())]);
        assert_eq!(chars.iter().collect::<String>(), "Hello\nyou");
        // up keeps the column, backspace removes before the cursor
        apply_typing(&mut chars, &mut caret, &[key(Key::ArrowUp), key(Key::Backspace)]);
        assert_eq!(chars.iter().collect::<String>(), "Helo\nyou");
        assert_eq!(caret, 2);
        // delete after the cursor, home, escape finishes
        let (_, finish) = apply_typing(&mut chars, &mut caret, &[key(Key::Delete), key(Key::Home), key(Key::Escape)]);
        assert_eq!(chars.iter().collect::<String>(), "Heo\nyou");
        assert_eq!(caret, 0);
        assert!(finish);
        // paste keeps new lines
        let mut caret = 3;
        apply_typing(&mut chars, &mut caret, &[egui::Event::Paste("!\n".into())]);
        assert_eq!(chars.iter().collect::<String>(), "Heo!\n\nyou");
    }
}
