//! The home page, the "What do you want to make?" project picker, and My Stuff.

use super::projects::{self, Icon, ProjectType, Template};
use super::*;

/// A simple line drawing for a project type or template tile.
pub(super) fn draw_template_icon(p: &egui::Painter, r: Rect, icon: Icon, col: Color32) {
    let c = r.center();
    let s = r.width().min(r.height()) / 2.0;
    let st = Stroke::new((s * 0.07).clamp(1.2, 2.6), col);
    let pt = |x: f32, y: f32| c + vec2(x * s, y * s);
    let poly = |pts: &[(f32, f32)]| Shape::line(pts.iter().map(|(x, y)| pt(*x, *y)).collect(), st);
    let rr = |x0: f32, y0: f32, x1: f32, y1: f32, rad: f32| {
        Shape::rect_stroke(Rect::from_min_max(pt(x0, y0), pt(x1, y1)), rad * s, st, StrokeKind::Middle)
    };
    match icon {
        Icon::Bottle => {
            p.add(rr(-0.35, -0.35, 0.35, 0.85, 0.15));
            p.add(rr(-0.18, -0.6, 0.18, -0.35, 0.05));
            p.add(rr(-0.22, -0.82, 0.22, -0.6, 0.06));
            p.add(poly(&[(-0.2, 0.15), (0.0, 0.35), (0.2, 0.15)]));
        }
        Icon::Notebook => {
            p.add(rr(-0.55, -0.75, 0.55, 0.75, 0.06));
            for k in 0..6 {
                p.circle_stroke(pt(-0.55, -0.6 + k as f32 * 0.24), s * 0.06, st);
            }
            p.add(poly(&[(-0.2, -0.3), (0.3, -0.3)]));
        }
        Icon::Mug => {
            p.add(rr(-0.55, -0.55, 0.35, 0.65, 0.08));
            let arc: Vec<Pos2> = (0..=12).map(|k| {
                let a = -std::f32::consts::FRAC_PI_2 + k as f32 / 12.0 * std::f32::consts::PI;
                pt(0.35 + a.cos() * 0.28, 0.05 + a.sin() * 0.3)
            }).collect();
            p.add(Shape::line(arc, st));
        }
        Icon::Car => {
            p.add(poly(&[(-0.85, 0.25), (-0.85, 0.0), (-0.5, -0.05), (-0.25, -0.4), (0.35, -0.4), (0.6, -0.05), (0.85, 0.0), (0.85, 0.25), (-0.85, 0.25)]));
            p.circle_stroke(pt(-0.45, 0.3), s * 0.15, st);
            p.circle_stroke(pt(0.45, 0.3), s * 0.15, st);
        }
        Icon::Wineglass => {
            let bowl: Vec<Pos2> = (0..=12).map(|k| {
                let a = k as f32 / 12.0 * std::f32::consts::PI;
                pt(-a.cos() * 0.4, -0.75 + a.sin() * 0.55)
            }).collect();
            p.add(Shape::line(bowl, st));
            p.add(poly(&[(-0.4, -0.75), (0.4, -0.75)]));
            p.add(poly(&[(0.0, -0.2), (0.0, 0.65)]));
            p.add(poly(&[(-0.3, 0.75), (0.3, 0.75)]));
        }
        Icon::Phone => {
            p.add(rr(-0.4, -0.8, 0.4, 0.8, 0.12));
            p.circle_stroke(pt(-0.2, -0.6), s * 0.07, st);
        }
        Icon::Glass => { let _ = p.add(poly(&[(-0.45, -0.75), (0.45, -0.75), (0.35, 0.8), (-0.35, 0.8), (-0.45, -0.75)])); }
        Icon::Tumbler => {
            p.add(poly(&[(-0.42, -0.55), (0.42, -0.55), (0.3, 0.85), (-0.3, 0.85), (-0.42, -0.55)]));
            p.add(rr(-0.48, -0.72, 0.48, -0.55, 0.05));
            p.add(poly(&[(0.12, -0.72), (0.25, -0.98)]));
        }
        Icon::Shirt => { let _ = p.add(poly(&[
            (-0.3, -0.75), (-0.85, -0.45), (-0.65, -0.1), (-0.45, -0.2), (-0.45, 0.8), (0.45, 0.8), (0.45, -0.2), (0.65, -0.1), (0.85, -0.45), (0.3, -0.75),
            (0.15, -0.6), (-0.15, -0.6), (-0.3, -0.75),
        ])); }
        Icon::Bodysuit => { let _ = p.add(poly(&[
            (-0.25, -0.75), (-0.7, -0.45), (-0.55, -0.15), (-0.4, -0.25), (-0.4, 0.45), (-0.15, 0.8), (0.15, 0.8), (0.4, 0.45), (0.4, -0.25), (0.55, -0.15), (0.7, -0.45),
            (0.25, -0.75), (0.1, -0.62), (-0.1, -0.62), (-0.25, -0.75),
        ])); }
        Icon::Card => {
            p.add(rr(-0.6, -0.75, 0.6, 0.75, 0.04));
            p.add(poly(&[(-0.6, -0.75), (-0.35, -0.6), (-0.35, 0.9), (-0.6, 0.75)]));
            p.add(poly(&[(-0.1, -0.1), (0.35, -0.1)]));
            p.add(poly(&[(-0.1, 0.1), (0.25, 0.1)]));
        }
        Icon::Tag => {
            p.add(poly(&[(-0.35, -0.5), (0.0, -0.85), (0.35, -0.5), (0.35, 0.85), (-0.35, 0.85), (-0.35, -0.5)]));
            p.circle_stroke(pt(0.0, -0.45), s * 0.08, st);
        }
        Icon::Bookmark => { let _ = p.add(poly(&[(-0.3, -0.85), (0.3, -0.85), (0.3, 0.85), (0.0, 0.6), (-0.3, 0.85), (-0.3, -0.85)])); }
        Icon::Holder => {
            let heart: Vec<Pos2> = (0..=40).map(|k| {
                let t = k as f32 / 40.0 * std::f32::consts::TAU;
                let x = 16.0 * t.sin().powi(3);
                let y = 13.0 * t.cos() - 5.0 * (2.0 * t).cos() - 2.0 * (3.0 * t).cos() - (4.0 * t).cos();
                pt(x / 30.0, -0.25 - y / 30.0)
            }).collect();
            p.add(Shape::line(heart, st));
            p.add(poly(&[(0.0, 0.25), (0.0, 0.9)]));
        }
        Icon::Sheet => {
            p.add(rr(-0.6, -0.8, 0.6, 0.8, 0.04));
            for (x, y, rad) in [(-0.25, -0.4, 0.2), (0.25, -0.35, 0.15), (-0.2, 0.2, 0.15), (0.25, 0.3, 0.22)] {
                p.circle_stroke(pt(x, y), s * rad, st);
            }
        }
        Icon::Labels => {
            p.add(rr(-0.6, -0.8, 0.6, 0.8, 0.04));
            for row in 0..4 {
                for col in 0..2 {
                    let (x, y) = (-0.45 + col as f32 * 0.5, -0.65 + row as f32 * 0.38);
                    p.add(rr(x, y, x + 0.4, y + 0.26, 0.06));
                }
            }
        }
        Icon::Hat => {
            let dome: Vec<Pos2> = (0..=16).map(|k| {
                let a = std::f32::consts::PI + k as f32 / 16.0 * std::f32::consts::PI;
                pt(a.cos() * 0.55, 0.25 + a.sin() * 0.6)
            }).collect();
            p.add(Shape::line(dome, st));
            p.add(poly(&[(-0.55, 0.25), (0.9, 0.25), (0.95, 0.4), (0.4, 0.4)]));
        }
        Icon::Pillow => { let _ = p.add(poly(&[(-0.75, -0.65), (0.0, -0.55), (0.75, -0.65), (0.65, 0.0), (0.75, 0.65), (0.0, 0.55), (-0.75, 0.65), (-0.65, 0.0), (-0.75, -0.65)])); }
        Icon::Bag => {
            p.add(poly(&[(-0.6, -0.25), (0.6, -0.25), (0.7, 0.85), (-0.7, 0.85), (-0.6, -0.25)]));
            let handle: Vec<Pos2> = (0..=12).map(|k| {
                let a = std::f32::consts::PI + k as f32 / 12.0 * std::f32::consts::PI;
                pt(a.cos() * 0.3, -0.25 + a.sin() * 0.45)
            }).collect();
            p.add(Shape::line(handle, st));
        }
        Icon::Plate => {
            p.add(rr(-0.75, -0.55, 0.75, 0.55, 0.12));
            let thin = Stroke::new(st.width * 0.6, col);
            for k in 0..7 {
                let x = -0.5 + k as f32 * 0.17;
                p.line_segment([pt(x, 0.35), pt(x + 0.25, -0.35)], thin);
            }
        }
        Icon::Plus => {
            p.circle_stroke(c, s * 0.55, st);
            p.add(poly(&[(-0.25, 0.0), (0.25, 0.0)]));
            p.add(poly(&[(0.0, -0.25), (0.0, 0.25)]));
        }
    }
}

pub(super) fn type_icon(t: ProjectType) -> Icon {
    match t {
        ProjectType::VinylDecal => Icon::Bottle,
        ProjectType::IronOn => Icon::Shirt,
        ProjectType::Card => Icon::Card,
        ProjectType::CardstockCutout => Icon::Tag,
        ProjectType::Stickers => Icon::Sheet,
        ProjectType::Engrave => Icon::Plate,
        ProjectType::IronOnAccessories => Icon::Hat,
        ProjectType::Dtf => Icon::Labels,
        ProjectType::Blank => Icon::Plus,
    }
}

impl App {
    /// Open the "What do you want to make?" window, optionally on one project type.
    pub(super) fn open_new_project(&mut self, kind: Option<ProjectType>) {
        let (w, h) = kind
            .and_then(|k| projects::templates(k).first().map(|t| (t.sizes[0].1, t.sizes[0].2)))
            .unwrap_or((12.0, 12.0));
        self.new_project = Some(NewProject { kind, template: 0, size: 0, w, h, show_needs: false });
    }

    /// Start a new project of a kind, from a template (or a blank canvas).
    pub(super) fn start_project(&mut self, kind: ProjectType, template: Option<&str>, size: Vec2) {
        self.finish_text_edit();
        self.snapshot();
        self.items.clear();
        self.selection.clear();
        self.project_type = kind;
        self.design_area = template.map(|t| DesignArea { name: t.to_string(), size: [size.x, size.y] });
        self.project_name = template.map(|t| format!("{t} - {}", kind.label())).unwrap_or_else(|| "Blank Canvas".into());
        self.mirror = kind.mirrors();
        self.screen = Screen::Design;
        self.drawer = Drawer::Shapes;
        self.zoom = 1.0;
        self.pan = vec2(RULER + 36.0, RULER + 28.0);
        self.mat_index = 0;
        self.project_path = documents_dir().join(format!("{}.matcraft.json", safe_name(&self.project_name))).display().to_string();
        // a material that suits the project
        let want = match kind {
            ProjectType::Engrave => "Aluminum sheet (engrave)",
            ProjectType::VinylDecal => "Adhesive vinyl",
            ProjectType::IronOn | ProjectType::IronOnAccessories => "Iron-on (heat transfer)",
            ProjectType::Stickers => "Sticker paper",
            ProjectType::Card | ProjectType::CardstockCutout => "Cardstock (medium)",
            _ => "",
        };
        if let Some(i) = crate::materials::all_materials(&self.prefs.custom_materials).iter().position(|m| m.name == want) {
            self.material = i;
        }
        self.status.clear();
    }

    /// An engrave project with a filled heart, an outlined plate and engraved text (used by
    /// `--engrave-sample`, for checking the engraving views).
    pub fn engrave_sample(&mut self) {
        self.start_project(ProjectType::Engrave, Some("Engrave"), projects::ENGRAVE_AREA);
        let mut plate = Item::new("Plate", ShapeKind::RoundedSquare.unit_contours(), pos2(1.0, 1.0), vec2(6.0, 4.0), PALETTE[3]);
        plate.op = Operation::Engrave;
        let mut heart = Item::new("Heart", ShapeKind::Heart.unit_contours(), pos2(1.6, 1.6), vec2(2.4, 2.2), PALETTE[0]);
        heart.op = Operation::Engrave;
        heart.engrave.style = super::EngraveStyle::Fill;
        heart.engrave.spacing = 0.06;
        let mut shield = Item::new("Shield", ShapeKind::Shield.unit_contours(), pos2(4.4, 1.5), vec2(2.0, 2.5), PALETTE[2]);
        shield.op = Operation::Engrave;
        shield.engrave.style = super::EngraveStyle::Fill;
        shield.engrave.pattern = crate::geometry::FillPattern::CrossHatch;
        shield.engrave.spacing = 0.08;
        self.items.extend([plate, heart, shield]);
        self.add_text_item("Engraved", pos2(1.4, 5.4), PALETTE[1]);
        if let Some(t) = self.items.last_mut() {
            t.op = Operation::Engrave;
        }
        self.selection = vec![1];
    }

    /// Save the project into My Stuff (or where it was opened from).
    pub(super) fn save_project(&mut self) {
        let p = std::path::PathBuf::from(&self.project_path);
        if let Some(d) = p.parent() {
            let _ = std::fs::create_dir_all(d);
        }
        let json = self.project_json();
        self.status = match std::fs::write(&p, &json) {
            Ok(()) => {
                self.last_saved_json = json;
                self.my_stuff = None;
                format!("Saved to My Stuff: {}", p.display())
            }
            Err(e) => format!("Couldn't save: {e}"),
        };
    }

    /// Open a project file (new format, or the earlier list of layers).
    pub(super) fn open_project_file(&mut self, path: &std::path::Path) -> Result<(), String> {
        let text = std::fs::read_to_string(path).map_err(|e| e.to_string())?;
        let (items, kind, area, name) = match serde_json::from_str::<projects::ProjectFile>(&text) {
            Ok(f) => (f.items, f.project_type, f.design_area, f.name),
            Err(_) => {
                let items: Vec<Item> = serde_json::from_str(&text).map_err(|e| e.to_string())?;
                let name = path.file_stem().map(|s| s.to_string_lossy().replace(".matcraft", "")).unwrap_or_default();
                (items, ProjectType::Blank, None, name)
            }
        };
        self.finish_text_edit();
        self.snapshot();
        self.items = items;
        for it in &mut self.items {
            it.uid = new_uid();
            self.next_group = self.next_group.max(it.group.unwrap_or(0)).max(it.attach.unwrap_or(0));
        }
        self.project_type = kind;
        self.design_area = area;
        self.project_name = name;
        self.mirror = kind.mirrors();
        self.project_path = path.display().to_string();
        self.selection.clear();
        self.screen = Screen::Design;
        self.last_saved_json = self.project_json();
        Ok(())
    }

    fn scan_my_stuff(&mut self) {
        let mut list = Vec::new();
        if let Ok(rd) = std::fs::read_dir(documents_dir()) {
            for e in rd.flatten() {
                let path = e.path();
                let fname = path.file_name().map(|n| n.to_string_lossy().to_string()).unwrap_or_default();
                if !fname.ends_with(".matcraft.json") {
                    continue;
                }
                let modified = e.metadata().ok().and_then(|m| m.modified().ok());
                let text = std::fs::read_to_string(&path).unwrap_or_default();
                let (items, kind, name) = match serde_json::from_str::<projects::ProjectFile>(&text) {
                    Ok(f) => (f.items, f.project_type, f.name),
                    Err(_) => (
                        serde_json::from_str::<Vec<Item>>(&text).unwrap_or_default(),
                        ProjectType::Blank,
                        fname.trim_end_matches(".matcraft.json").to_string(),
                    ),
                };
                list.push(SavedProject { path, name, modified, kind, items });
            }
        }
        list.sort_by(|a, b| b.modified.cmp(&a.modified));
        self.my_stuff = Some(list);
    }

    /// The left navigation of the home page and My Stuff.
    fn home_nav(&mut self, ui: &mut Ui) {
        egui::Panel::left("home-nav")
            .exact_size(210.0)
            .resizable(false)
            .show_separator_line(false)
            .frame(egui::Frame::NONE.fill(SURFACE()).stroke(Stroke::new(1.0, BORDER())).inner_margin(Margin::symmetric(12, 16)))
            .show(ui, |ui| {
                ui.spacing_mut().item_spacing.y = 4.0;
                let mut nav = |ui: &mut Ui, label: &str, active: bool| -> bool {
                    let (r, resp) = ui.allocate_exact_size(vec2(ui.available_width(), 38.0), Sense::click());
                    if active {
                        ui.painter().rect_filled(r, 10.0, ACCENT_SOFT());
                    } else if resp.hovered() {
                        ui.painter().rect_filled(r, 10.0, SUBTLE());
                    }
                    ui.painter().text(r.left_center() + vec2(14.0, 0.0), Align2::LEFT_CENTER, label, FontId::proportional(14.0), if active { ACCENT_DARK() } else { INK() });
                    resp.on_hover_cursor(egui::CursorIcon::PointingHand).clicked()
                };
                if nav(ui, "Home", self.screen == Screen::Home) {
                    self.screen = Screen::Home;
                }
                if nav(ui, "My Stuff", self.screen == Screen::MyStuff) {
                    self.my_stuff = None;
                    self.screen = Screen::MyStuff;
                }
                if nav(ui, "Canvas", false) {
                    self.screen = Screen::Design;
                }
                ui.add_space(8.0);
                let (r, _) = ui.allocate_exact_size(vec2(ui.available_width(), 1.0), Sense::hover());
                ui.painter().rect_filled(r, 0.0, BORDER());
                ui.add_space(8.0);
                if nav(ui, "Machine", self.show_machine) {
                    self.show_machine = !self.show_machine;
                }
                if nav(ui, "Settings", self.show_settings) {
                    self.show_settings = !self.show_settings;
                }
                if nav(ui, "Compare", self.show_compare) {
                    self.show_compare = !self.show_compare;
                }
            });
    }

    pub(super) fn home_screen(&mut self, ui: &mut Ui) {
        self.home_nav(ui);
        if self.my_stuff.is_none() {
            self.scan_my_stuff();
        }
        egui::CentralPanel::no_frame().frame(egui::Frame::NONE.fill(BG())).show(ui, |ui| {
            egui::ScrollArea::vertical().show(ui, |ui| {
                let full_w = ui.available_width();
                // the welcome band
                let (band, _) = ui.allocate_exact_size(vec2(full_w, 250.0), Sense::hover());
                let p = ui.painter();
                let mut mesh = Mesh::default();
                let top = Color32::from_rgb(0x0d, 0x8a, 0x5b);
                let bottom = themed(Color32::from_rgb(0xd8, 0xf2, 0xe6), Color32::from_rgb(0x15, 0x30, 0x26));
                mesh.colored_vertex(band.left_top(), top);
                mesh.colored_vertex(band.right_top(), top);
                mesh.colored_vertex(band.right_bottom(), bottom);
                mesh.colored_vertex(band.left_bottom(), bottom);
                mesh.add_triangle(0, 1, 2);
                mesh.add_triangle(0, 2, 3);
                p.add(Shape::mesh(mesh));
                // a soft wave across the middle
                let wave: Vec<Pos2> = (0..=60)
                    .map(|k| {
                        let t = k as f32 / 60.0;
                        pos2(band.left() + t * band.width(), band.top() + 118.0 + (t * std::f32::consts::TAU).sin() * 10.0)
                    })
                    .chain([band.right_bottom(), band.left_bottom()])
                    .collect();
                p.add(Shape::convex_polygon(vec![band.left_bottom(), band.right_bottom()], Color32::TRANSPARENT, Stroke::NONE));
                let mut wm = Mesh::default();
                let base = wm.vertices.len() as u32;
                for w in wave.iter().take(61) {
                    wm.colored_vertex(*w, bottom);
                    wm.colored_vertex(pos2(w.x, band.bottom()), bottom);
                }
                for k in 0..60u32 {
                    let a = base + k * 2;
                    wm.add_triangle(a, a + 1, a + 2);
                    wm.add_triangle(a + 1, a + 3, a + 2);
                }
                p.add(Shape::mesh(wm));
                p.text(band.center_top() + vec2(0.0, 40.0), Align2::CENTER_CENTER, "What do you want to make?", FontId::proportional(24.0), Color32::WHITE);
                // search box
                let sr = Rect::from_center_size(band.center_top() + vec2(0.0, 84.0), vec2(460.0, 38.0));
                let mut sui = ui.new_child(egui::UiBuilder::new().max_rect(sr));
                sui.add(
                    egui::TextEdit::singleline(&mut self.home_search)
                        .hint_text("Search project types...")
                        .desired_width(460.0)
                        .margin(Margin::symmetric(14, 9))
                        .font(FontId::proportional(14.0)),
                );
                // project type buttons
                let n = ProjectType::ALL.len() as f32;
                let spacing = 100.0;
                let start_x = band.center().x - (n - 1.0) * spacing / 2.0;
                let mut open: Option<ProjectType> = None;
                for (k, t) in ProjectType::ALL.iter().enumerate() {
                    let c = pos2(start_x + k as f32 * spacing, band.top() + 168.0);
                    let r = Rect::from_center_size(c + vec2(0.0, 14.0), vec2(92.0, 92.0));
                    let resp = ui.interact(r, egui::Id::unique(("home-type", k)), Sense::click());
                    let hovered = resp.hovered();
                    ui.painter().circle_filled(c, 24.0, if hovered { ACCENT_DARK() } else { ACCENT });
                    draw_template_icon(ui.painter(), Rect::from_center_size(c, vec2(26.0, 26.0)), type_icon(*t), Color32::WHITE);
                    let g = ui.painter().layout(t.label().to_string(), FontId::proportional(12.0), INK(), 90.0);
                    let gw = g.size().x;
                    ui.painter().galley(pos2(c.x - gw / 2.0, c.y + 30.0), g, INK());
                    if resp.on_hover_cursor(egui::CursorIcon::PointingHand).clicked() {
                        open = Some(*t);
                    }
                }
                if let Some(t) = open {
                    if t == ProjectType::Blank {
                        self.start_project(ProjectType::Blank, None, Vec2::ZERO);
                    } else {
                        self.open_new_project(Some(t));
                    }
                }

                ui.add_space(18.0);
                let inner = |ui: &mut Ui, add: &mut dyn FnMut(&mut Ui)| {
                    egui::Frame::NONE.inner_margin(Margin::symmetric(40, 0)).show(ui, |ui| {
                        ui.set_width(ui.available_width());
                        add(ui)
                    });
                };
                // search results across every template
                let q = self.home_search.trim().to_lowercase();
                let mut start: Option<(ProjectType, String, Vec2)> = None;
                if !q.is_empty() {
                    inner(ui, &mut |ui: &mut Ui| {
                        ui.label(RichText::new("Matching projects").size(17.0).strong().color(INK()));
                        ui.add_space(4.0);
                        ui.horizontal_wrapped(|ui| {
                            let mut any = false;
                            for t in ProjectType::ALL {
                                for tpl in projects::templates(t) {
                                    if tpl.name.to_lowercase().contains(&q) || t.label().to_lowercase().contains(&q) {
                                        any = true;
                                        if template_tile(ui, &tpl, false, 120.0).clicked() {
                                            start = Some((t, tpl.name.to_string(), vec2(tpl.sizes[0].1, tpl.sizes[0].2)));
                                        }
                                    }
                                }
                            }
                            if !any {
                                ui.label(RichText::new("Nothing matches. Try \"mug\", \"shirt\" or \"engrave\".").color(MUTED()));
                            }
                        });
                    });
                    ui.add_space(14.0);
                }
                if let Some((t, name, size)) = start {
                    self.start_project(t, Some(&name), size);
                    return;
                }

                // continue the current design
                let current = self.items.len();
                let mut go_canvas = false;
                let mut open_path: Option<std::path::PathBuf> = None;
                let mut see_all = false;
                inner(ui, &mut |ui: &mut Ui| {
                    if current > 0 {
                        card(ui, |ui| {
                            ui.horizontal(|ui| {
                                ui.vertical(|ui| {
                                    ui.label(RichText::new("Continue designing").size(16.0).strong().color(INK()));
                                    ui.label(RichText::new(format!("{}  ·  {} layer(s)", self.project_name, current)).color(MUTED()));
                                });
                                ui.with_layout(Layout::right_to_left(Align::Center), |ui| {
                                    if primary_button(ui, "Open canvas").clicked() {
                                        go_canvas = true;
                                    }
                                });
                            });
                        });
                        ui.add_space(14.0);
                    }
                    ui.horizontal(|ui| {
                        ui.label(RichText::new("Your projects").size(17.0).strong().color(INK()));
                        ui.with_layout(Layout::right_to_left(Align::Center), |ui| {
                            if secondary_button(ui, "See all in My Stuff").clicked() {
                                see_all = true;
                            }
                        });
                    });
                    ui.add_space(6.0);
                    let list = self.my_stuff.clone().unwrap_or_default();
                    if list.is_empty() {
                        ui.label(RichText::new("Projects you save appear here. Start one above, or open the canvas.").color(MUTED()));
                    }
                    ui.horizontal_wrapped(|ui| {
                        for sp in list.iter().take(8) {
                            if project_tile(ui, sp).clicked() {
                                open_path = Some(sp.path.clone());
                            }
                        }
                    });
                    ui.add_space(20.0);
                });
                if go_canvas {
                    self.screen = Screen::Design;
                }
                if see_all {
                    self.screen = Screen::MyStuff;
                }
                if let Some(p) = open_path {
                    if let Err(e) = self.open_project_file(&p) {
                        self.status = format!("Couldn't open: {e}");
                    }
                }
            });
        });
    }

    pub(super) fn my_stuff_screen(&mut self, ui: &mut Ui) {
        self.home_nav(ui);
        if self.my_stuff.is_none() {
            self.scan_my_stuff();
        }
        let mut open_path: Option<std::path::PathBuf> = None;
        let mut refresh = false;
        egui::CentralPanel::no_frame().frame(egui::Frame::NONE.fill(BG()).inner_margin(Margin::symmetric(28, 20))).show(ui, |ui| {
            ui.horizontal(|ui| {
                ui.label(RichText::new("Projects").size(22.0).strong().color(INK()));
                ui.with_layout(Layout::right_to_left(Align::Center), |ui| {
                    if primary_button(ui, "+ New Project").clicked() {
                        self.open_new_project(None);
                    }
                    if secondary_button(ui, "Open a file...").clicked() {
                        if let Some(p) = rfd::FileDialog::new().add_filter("MatCraft project", &["json"]).set_title("Open project").pick_file() {
                            open_path = Some(p);
                        }
                    }
                    if secondary_button(ui, "Refresh").clicked() {
                        refresh = true;
                    }
                    ui.add(egui::TextEdit::singleline(&mut self.home_search).hint_text("Search my projects").desired_width(220.0));
                });
            });
            ui.label(RichText::new(format!("Saved in {}", documents_dir().display())).size(12.0).color(MUTED()));
            ui.add_space(10.0);
            let q = self.home_search.trim().to_lowercase();
            let list: Vec<SavedProject> = self.my_stuff.clone().unwrap_or_default().into_iter().filter(|p| q.is_empty() || p.name.to_lowercase().contains(&q)).collect();
            if list.is_empty() {
                ui.add_space(40.0);
                ui.vertical_centered(|ui| {
                    ui.label(RichText::new("No saved projects yet").size(16.0).strong().color(INK()));
                    ui.label(RichText::new("Use Save on the canvas and your projects collect here.").color(MUTED()));
                });
            }
            egui::ScrollArea::vertical().show(ui, |ui| {
                ui.horizontal_wrapped(|ui| {
                    for sp in &list {
                        if project_tile(ui, sp).clicked() {
                            open_path = Some(sp.path.clone());
                        }
                    }
                });
            });
        });
        if refresh {
            self.my_stuff = None;
        }
        if let Some(p) = open_path {
            if let Err(e) = self.open_project_file(&p) {
                self.status = format!("Couldn't open: {e}");
            }
        }
    }

    pub(super) fn new_project_window(&mut self, ctx: &egui::Context) {
        let Some(np) = self.new_project.as_mut() else { return };
        let mut open = true;
        let mut start: Option<(ProjectType, Option<String>, Vec2)> = None;
        let mut cancel = false;
        let screen = ctx.content_rect();
        egui::Window::new(RichText::new("What do you want to make?").strong().size(18.0))
            .id(egui::Id::unique("new-project"))
            .open(&mut open)
            .collapsible(false)
            .resizable(true)
            .default_rect(Rect::from_center_size(screen.center(), vec2((screen.width() * 0.72).max(760.0), (screen.height() * 0.78).max(520.0))))
            .show(ctx, |ui| {
                ui.label(RichText::new("Pick a project type, or start from scratch.").color(MUTED()));
                ui.add_space(6.0);
                let full = ui.available_rect_before_wrap();
                let foot = 48.0;
                let side_w = 190.0;
                // left list of project types
                let left = Rect::from_min_size(full.min, vec2(side_w, full.height() - foot));
                let mut lui = ui.new_child(egui::UiBuilder::new().max_rect(left).layout(Layout::top_down(Align::Min)));
                {
                    let ui = &mut lui;
                    ui.spacing_mut().item_spacing.y = 2.0;
                    ui.label(RichText::new("Project Types").size(12.5).strong().color(MUTED()));
                    let mut row = |ui: &mut Ui, label: &str, icon: Icon, sel: bool| -> bool {
                        let (r, resp) = ui.allocate_exact_size(vec2(side_w - 8.0, 30.0), Sense::click());
                        if sel {
                            ui.painter().rect_filled(r, 8.0, ACCENT_SOFT());
                        } else if resp.hovered() {
                            ui.painter().rect_filled(r, 8.0, SUBTLE());
                        }
                        draw_template_icon(ui.painter(), Rect::from_center_size(r.left_center() + vec2(16.0, 0.0), vec2(16.0, 16.0)), icon, if sel { ACCENT_DARK() } else { INK() });
                        ui.painter().text(r.left_center() + vec2(32.0, 0.0), Align2::LEFT_CENTER, label, FontId::proportional(13.0), if sel { ACCENT_DARK() } else { INK() });
                        resp.on_hover_cursor(egui::CursorIcon::PointingHand).clicked()
                    };
                    if row(ui, "All Project Types", Icon::Labels, np.kind.is_none()) {
                        np.kind = None;
                        np.template = 0;
                    }
                    for t in ProjectType::ALL.iter().filter(|t| **t != ProjectType::Blank) {
                        if row(ui, t.label(), type_icon(*t), np.kind == Some(*t)) {
                            np.kind = Some(*t);
                            np.template = 0;
                            np.size = 0;
                            if let Some(tpl) = projects::templates(*t).first() {
                                np.w = tpl.sizes[0].1;
                                np.h = tpl.sizes[0].2;
                            }
                        }
                    }
                    ui.add_space(8.0);
                    ui.label(RichText::new("Design From Scratch").size(12.5).strong().color(MUTED()));
                    if row(ui, "Blank Canvas", Icon::Plus, np.kind == Some(ProjectType::Blank)) {
                        np.kind = Some(ProjectType::Blank);
                    }
                }
                // right side
                let right = Rect::from_min_max(pos2(left.right() + 14.0, full.top()), pos2(full.right(), full.bottom() - foot));
                let mut rui = ui.new_child(egui::UiBuilder::new().max_rect(right).layout(Layout::top_down(Align::Min)));
                {
                    let ui = &mut rui;
                    match np.kind {
                        None => {
                            egui::ScrollArea::vertical().id_salt("all-templates").show(ui, |ui| {
                                ui.horizontal_wrapped(|ui| {
                                    for t in ProjectType::ALL {
                                        for tpl in projects::templates(t) {
                                            if template_tile(ui, &tpl, false, 128.0).clicked() {
                                                np.kind = Some(t);
                                                np.template = projects::templates(t).iter().position(|x| x.name == tpl.name).unwrap_or(0);
                                                np.size = 0;
                                                np.w = tpl.sizes[0].1;
                                                np.h = tpl.sizes[0].2;
                                            }
                                        }
                                    }
                                });
                            });
                        }
                        Some(ProjectType::Blank) => {
                            card(ui, |ui| {
                                ui.add_space(20.0);
                                ui.label(RichText::new("Ready to design from scratch?").size(20.0).strong().color(INK()));
                                ui.add_space(40.0);
                                ui.label(RichText::new("A blank canvas is yours to shape. Press Start Designing to begin.").size(15.0).color(INK()));
                                ui.label(RichText::new("New to this? Starting from a project type sets up the size and settings for you.").color(MUTED()));
                                ui.add_space(20.0);
                            });
                        }
                        Some(ProjectType::Engrave) => {
                            ui.vertical_centered(|ui| {
                                ui.label(RichText::new("Things to know about engraving").size(18.0).strong().color(INK()));
                            });
                            ui.add_space(10.0);
                            let tips = [
                                (Icon::Plate, "Use the right supplies", "Engrave metal, acrylic, leather and other engravable blanks with the Engrave Tip or the Engraving Tool."),
                                (Icon::Holder, "Ready to engrave", "Shapes, text and pictures you add are set to Engrave straight away, so you can start designing."),
                                (Icon::Sheet, "Secure your material", "Place the material on the mat where it sits on screen, and tape it down on all sides so it can't shift."),
                            ];
                            ui.columns(3, |cols| {
                                for (k, (icon, title, text)) in tips.iter().enumerate() {
                                    let ui = &mut cols[k];
                                    let (r, _) = ui.allocate_exact_size(vec2(ui.available_width(), 130.0), Sense::hover());
                                    ui.painter().rect_filled(r, 12.0, SUBTLE());
                                    draw_template_icon(ui.painter(), Rect::from_center_size(r.center(), vec2(80.0, 80.0)), *icon, MUTED());
                                    ui.add_space(6.0);
                                    ui.label(RichText::new(*title).strong().color(INK()));
                                    ui.label(RichText::new(*text).size(12.5).color(MUTED()));
                                }
                            });
                            ui.add_space(8.0);
                            ui.label(
                                RichText::new(format!(
                                    "Design area {} x {} in. Your design is engraved exactly where it sits in that area.",
                                    projects::ENGRAVE_AREA.x, projects::ENGRAVE_AREA.y
                                ))
                                .color(MUTED()),
                            );
                        }
                        Some(kind) => {
                            let tpls = projects::templates(kind);
                            // what you need
                            egui::Frame::NONE.fill(SELECT_SOFT()).corner_radius(10).inner_margin(Margin::symmetric(12, 8)).show(ui, |ui| {
                                ui.set_width(ui.available_width());
                                let resp = ui.horizontal(|ui| {
                                    ui.label(RichText::new(format!("Tools & materials needed for {}", kind.label())).color(INK()));
                                    ui.with_layout(Layout::right_to_left(Align::Center), |ui| {
                                        if ui.small_button(if np.show_needs { "Hide" } else { "Show" }).clicked() {
                                            np.show_needs = !np.show_needs;
                                        }
                                    });
                                });
                                let _ = resp;
                                if np.show_needs {
                                    ui.label(RichText::new(kind.needs()).size(13.0).color(MUTED()));
                                }
                            });
                            ui.add_space(8.0);
                            let tiles_w = (ui.available_width() - 250.0).max(300.0);
                            ui.horizontal_top(|ui| {
                                ui.allocate_ui_with_layout(vec2(tiles_w, ui.available_height()), Layout::top_down(Align::Min), |ui| {
                                    egui::ScrollArea::vertical().id_salt("kind-templates").show(ui, |ui| {
                                        ui.horizontal_wrapped(|ui| {
                                            for (k, tpl) in tpls.iter().enumerate() {
                                                if template_tile(ui, tpl, k == np.template, 128.0).clicked() {
                                                    np.template = k;
                                                    np.size = 0;
                                                    np.w = tpl.sizes[0].1;
                                                    np.h = tpl.sizes[0].2;
                                                }
                                            }
                                        });
                                    });
                                });
                                ui.add_space(12.0);
                                ui.vertical(|ui| {
                                    ui.set_width(230.0);
                                    if let Some(tpl) = tpls.get(np.template) {
                                        let label = if kind == ProjectType::Stickers { "Printable material size" } else { "Size" };
                                        ui.label(RichText::new(label).strong().color(INK()));
                                        egui::ComboBox::from_id_salt("tpl-size").selected_text(tpl.sizes[np.size.min(tpl.sizes.len() - 1)].0).width(220.0).show_ui(ui, |ui| {
                                            for (k, s) in tpl.sizes.iter().enumerate() {
                                                if ui.selectable_label(k == np.size, s.0).clicked() {
                                                    np.size = k;
                                                    np.w = s.1;
                                                    np.h = s.2;
                                                }
                                            }
                                        });
                                        ui.add_space(8.0);
                                        ui.label(RichText::new("Design area size (in)").strong().color(INK()));
                                        ui.horizontal(|ui| {
                                            ui.add(egui::DragValue::new(&mut np.w).speed(0.05).range(0.25..=24.0).prefix("W ").fixed_decimals(2));
                                            ui.add(egui::DragValue::new(&mut np.h).speed(0.05).range(0.25..=48.0).prefix("H ").fixed_decimals(2));
                                        });
                                        if kind == ProjectType::Stickers {
                                            ui.add_space(8.0);
                                            ui.label(
                                                RichText::new("Stickers need printable sticker paper and an inkjet printer. Calibrate print then cut on your machine before printing.")
                                                    .size(12.0)
                                                    .color(MUTED()),
                                            );
                                        }
                                        if kind.mirrors() {
                                            ui.add_space(8.0);
                                            ui.label(RichText::new("Iron-on designs are cut mirrored; that's switched on for you.").size(12.0).color(MUTED()));
                                        }
                                    }
                                });
                            });
                        }
                    }
                }
                // footer buttons
                let fr = Rect::from_min_max(pos2(full.left(), full.bottom() - foot + 8.0), full.max);
                let mut fui = ui.new_child(egui::UiBuilder::new().max_rect(fr).layout(Layout::right_to_left(Align::Center)));
                {
                    let ui = &mut fui;
                    let ready = np.kind.is_some();
                    let go = ui.add_enabled(
                        ready,
                        egui::Button::new(RichText::new("Start Designing").color(Color32::WHITE).strong())
                            .fill(ACCENT)
                            .stroke(Stroke::NONE)
                            .corner_radius(CornerRadius::same(16))
                            .min_size(vec2(150.0, 34.0)),
                    );
                    if go.clicked() {
                        if let Some(kind) = np.kind {
                            match kind {
                                ProjectType::Blank => start = Some((kind, None, Vec2::ZERO)),
                                ProjectType::Engrave => start = Some((kind, Some("Engrave".into()), projects::ENGRAVE_AREA)),
                                _ => {
                                    let tpls = projects::templates(kind);
                                    if let Some(tpl) = tpls.get(np.template) {
                                        start = Some((kind, Some(tpl.name.to_string()), vec2(np.w, np.h)));
                                    }
                                }
                            }
                        }
                    }
                    if ui
                        .add(egui::Button::new(RichText::new("Cancel").color(ACCENT_DARK())).fill(Color32::TRANSPARENT).stroke(Stroke::new(1.0, ACCENT)).corner_radius(CornerRadius::same(16)).min_size(vec2(100.0, 34.0)))
                        .clicked()
                    {
                        cancel = true;
                    }
                }
                ui.allocate_rect(full, Sense::hover());
            });
        if let Some((kind, tpl, size)) = start {
            self.new_project = None;
            self.start_project(kind, tpl.as_deref(), size);
        } else if cancel || !open {
            self.new_project = None;
        }
    }
}

/// A file-name-safe version of a project name.
pub(super) fn safe_name(s: &str) -> String {
    let t: String = s.chars().map(|c| if c.is_alphanumeric() || c == ' ' || c == '-' || c == '_' { c } else { '_' }).collect();
    let t = t.trim().to_string();
    if t.is_empty() {
        "Untitled".into()
    } else {
        t
    }
}

/// A tile for a template in the project picker.
fn template_tile(ui: &mut Ui, tpl: &Template, selected: bool, w: f32) -> egui::Response {
    let (r, resp) = ui.allocate_exact_size(vec2(w, w + 26.0), Sense::click());
    let hovered = resp.hovered();
    if hovered || selected {
        ui.painter().add(shadow(10, 18).as_shape(r, CornerRadius::same(12)));
    }
    ui.painter().rect_filled(r, 12.0, SURFACE());
    ui.painter().rect_stroke(r, 12.0, Stroke::new(if selected { 2.0 } else { 1.0 }, if selected { ACCENT } else { BORDER() }), StrokeKind::Inside);
    let pic = Rect::from_min_size(r.min + vec2(8.0, 8.0), vec2(w - 16.0, w - 16.0));
    ui.painter().rect_filled(pic, 8.0, SUBTLE());
    draw_template_icon(ui.painter(), pic.shrink(w * 0.18), tpl.icon, MUTED());
    let g = ui.painter().layout(tpl.name.to_string(), FontId::proportional(12.0), INK(), w - 10.0);
    let gw = g.size().x;
    ui.painter().galley(pos2(r.center().x - gw / 2.0, pic.bottom() + 6.0), g, INK());
    resp.on_hover_cursor(egui::CursorIcon::PointingHand)
}

/// A tile for a saved project: a small drawing of its layers and its name.
fn project_tile(ui: &mut Ui, sp: &SavedProject) -> egui::Response {
    let (r, resp) = ui.allocate_exact_size(vec2(170.0, 196.0), Sense::click());
    if resp.hovered() {
        ui.painter().add(shadow(10, 18).as_shape(r, CornerRadius::same(12)));
    }
    ui.painter().rect_filled(r, 12.0, SURFACE());
    ui.painter().rect_stroke(r, 12.0, Stroke::new(1.0, BORDER()), StrokeKind::Inside);
    let pic = Rect::from_min_size(r.min + vec2(8.0, 8.0), vec2(154.0, 140.0));
    ui.painter().rect_filled(pic, 8.0, SUBTLE());
    let b = sp.items.iter().filter(|i| i.visible).fold(Rect::NOTHING, |acc, i| acc.union(i.world_bounds()));
    if b.is_finite() && b.width() > 0.0 {
        let k = ((pic.width() - 20.0) / b.width()).min((pic.height() - 20.0) / b.height().max(1e-3));
        let off = pic.center() - b.center().to_vec2() * k;
        let to_s = |q: Pos2| off + q.to_vec2() * k;
        let painter = ui.painter_at(pic);
        for it in sp.items.iter().filter(|i| i.visible) {
            draw_item(&painter, it, &to_s, Vec2::ZERO, None, None, None);
        }
    }
    let mut name = sp.name.clone();
    if name.chars().count() > 22 {
        name = name.chars().take(21).collect::<String>() + "…";
    }
    ui.painter().text(pos2(r.left() + 10.0, pic.bottom() + 14.0), Align2::LEFT_CENTER, name, FontId::proportional(13.0), INK());
    let when = sp
        .modified
        .and_then(|m| m.elapsed().ok())
        .map(|d| {
            let m = d.as_secs() / 60;
            if m < 1 {
                "just now".to_string()
            } else if m < 60 {
                format!("{m} min ago")
            } else if m < 60 * 24 {
                format!("{} h ago", m / 60)
            } else {
                format!("{} days ago", m / 60 / 24)
            }
        })
        .unwrap_or_default();
    ui.painter().text(pos2(r.left() + 10.0, pic.bottom() + 32.0), Align2::LEFT_CENTER, format!("{}  ·  {when}", sp.kind.label()), FontId::proportional(11.0), MUTED());
    resp.on_hover_cursor(egui::CursorIcon::PointingHand)
}
