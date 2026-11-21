//! A side-by-side of the original design app and this one, from what was read in the original's
//! files (pages in its packed web app, its helper programs, its machine program).

use egui::{Color32, RichText, Ui};

#[derive(Clone, Copy, PartialEq)]
pub enum Status {
    Same,
    Partial,
    Missing,
    Extra,
}

impl Status {
    fn label(self) -> &'static str {
        match self {
            Status::Same => "Same",
            Status::Partial => "Partly",
            Status::Missing => "Not yet",
            Status::Extra => "Only here",
        }
    }
    fn color(self) -> Color32 {
        match self {
            Status::Same => Color32::from_rgb(0x16, 0x9c, 0x5c),
            Status::Partial => Color32::from_rgb(0xd9, 0x8b, 0x10),
            Status::Missing => Color32::from_rgb(0xc8, 0x3c, 0x3c),
            Status::Extra => Color32::from_rgb(0x2f, 0x80, 0xed),
        }
    }
}

pub struct Row {
    pub area: &'static str,
    pub feature: &'static str,
    pub original: &'static str,
    pub here: &'static str,
    pub status: Status,
}

pub const ROWS: &[Row] = &[
    // --- running it
    Row { area: "Running it", feature: "Computers", original: "Windows and macOS (Electron app)", here: "Linux (Bazzite) and Windows, one small program", status: Status::Extra },
    Row { area: "Running it", feature: "Account and sign-in", original: "Sign-in page; needs an online account", here: "No account, works fully offline", status: Status::Extra },
    Row { area: "Running it", feature: "Home page", original: "Home page with project-type shortcuts, search, recent projects and ready-made ideas", here: "Home page with the same project-type shortcuts, search and your recent projects; no online ideas", status: Status::Partial },
    Row { area: "Running it", feature: "Choose what to make (project picker)", original: "Pick a project type, then a template and size; shows what you'll need", here: "Same: project type, template and size (or custom), with a what-you'll-need list and a design area on the canvas", status: Status::Same },
    Row { area: "Running it", feature: "My Stuff (your saved projects)", original: "Cloud and computer projects, search", here: "Projects saved on your computer, with search, open and new", status: Status::Partial },
    Row { area: "Running it", feature: "Settings", original: "General, Content, Machines, Canvas, Load Type, Notifications, System", here: "General, Machines, Canvas, Load Type, System (no Content or Notifications: nothing online)", status: Status::Partial },
    Row { area: "Running it", feature: "Several projects in tabs", original: "Tabs page", here: "One project at a time", status: Status::Missing },
    // --- adding designs
    Row { area: "Adding designs", feature: "Basic shapes", original: "Square, circle, triangle, pentagon, hexagon, octagon, star, heart, score line", here: "The same nine, plus diamond, rectangle, rounded square, oval, plus, half circle and shield", status: Status::Extra },
    Row { area: "Adding designs", feature: "Image and project library", original: "Large online library (subscription)", here: "None (works offline)", status: Status::Missing },
    Row { area: "Adding designs", feature: "Monogram maker", original: "Built-in plugin", here: "Not yet", status: Status::Missing },
    Row { area: "Adding designs", feature: "Save and open projects", original: "Cloud saves, plus offline saves", here: "Project files on your computer (pictures included)", status: Status::Partial },
    // --- editing
    Row { area: "Editing", feature: "Canvas, grid, rulers, zoom", original: "Grid in inches or cm, scroll to move, zoom", here: "Full, outline or no grid; rulers in inches or cm; snap guides; scroll to move, Ctrl+scroll to zoom, Fit", status: Status::Same },
    Row { area: "Editing", feature: "Move, resize, rotate, lock proportions", original: "Corner handles plus exact numbers", here: "Same corner handles (delete, rotate, resize, lock) plus exact numbers", status: Status::Same },
    Row { area: "Editing", feature: "Select several", original: "Shift-click, drag a box, select all", here: "Shift-click, drag a box, select all, move/resize/rotate together", status: Status::Same },
    Row { area: "Editing", feature: "Cut, copy, paste, duplicate, delete", original: "Edit menu and keyboard", here: "Edit menu and keyboard (Ctrl+X/C/V/D, Delete)", status: Status::Same },
    Row { area: "Editing", feature: "Undo / redo", original: "Yes", here: "Yes (Ctrl+Z, Ctrl+Y)", status: Status::Same },
    Row { area: "Editing", feature: "Align and distribute", original: "Left, centre, right, top, middle, bottom, centre; distribute", here: "The same seven plus distribute across and down", status: Status::Same },
    Row { area: "Editing", feature: "Arrange", original: "Send to back, backward, forward, to front", here: "The same four", status: Status::Same },
    Row { area: "Editing", feature: "Flip", original: "Horizontal and vertical", here: "Horizontal and vertical", status: Status::Same },
    Row { area: "Editing", feature: "Group / ungroup", original: "Yes", here: "Yes (Ctrl+G, Ctrl+Shift+G), shown in the layers list", status: Status::Same },
    Row { area: "Editing", feature: "Weld", original: "Merges layers into one shape", here: "Merges layers into one shape", status: Status::Same },
    Row { area: "Editing", feature: "Combine: unite, subtract, intersect, exclude", original: "Yes", here: "Yes", status: Status::Same },
    Row { area: "Editing", feature: "Slice", original: "Two layers into their pieces", here: "Two layers into their three pieces", status: Status::Same },
    Row { area: "Editing", feature: "Attach / detach", original: "Keeps layers placed together on the mat", here: "Keeps layers placed together on the mat", status: Status::Same },
    Row { area: "Editing", feature: "Flatten / unflatten", original: "Makes one printed image for print then cut", here: "Turns layers into print then cut, attached", status: Status::Partial },
    Row { area: "Editing", feature: "Offset", original: "Outline around or inside, rounded or sharp corners", here: "Outline around or inside with a live preview; rounded corners only", status: Status::Partial },
    Row { area: "Editing", feature: "Contour (hide inner shapes)", original: "Click shapes to hide or show them", here: "Click shapes to hide or show them, or hide all inner ones", status: Status::Same },
    Row { area: "Editing", feature: "Operations (linetype)", original: "Cut, draw, score, engrave, deboss, wave, perforate, print then cut, guide", here: "The same nine", status: Status::Same },
    Row { area: "Editing", feature: "Colour sync / layer colours", original: "Colour per layer, colour sync panel", here: "Colour per layer (one mat per colour); no colour sync panel", status: Status::Partial },
    Row { area: "Editing", feature: "Pattern fills, warp", original: "Yes", here: "Not yet", status: Status::Missing },
    Row { area: "Engraving", feature: "Engrave outline", original: "Engraves along the shape's edge", here: "Engraves along the shape's edge", status: Status::Same },
    Row { area: "Engraving", feature: "Engrave fill", original: "Fills with lines: horizontal, vertical, grid, slashes, cross-hatch, concentric; spacing, edge margin, fill rule", here: "The same seven patterns and settings, with the same starting values, drawn live on the canvas", status: Status::Same },
    Row { area: "Engraving", feature: "Engrave a photo", original: "Turns a photo into engraved lines", here: "Photo to dotted engrave lines with brightness and invert", status: Status::Partial },
    Row { area: "Engraving", feature: "Engrave Tip and Engraving Tool, 11.5 x 11.5 in area", original: "Yes, with a things-to-know card before engraving", here: "Yes (tool picked from your machine model), with a before-you-engrave card", status: Status::Same },
    // --- text
    Row { area: "Text", feature: "Add and edit text", original: "Type on the canvas", here: "Type on the canvas (double-click text): cursor, arrows, Enter, Backspace, paste; letters update as you type", status: Status::Same },
    Row { area: "Text", feature: "Select part of the text with the mouse", original: "Yes", here: "Not yet (the cursor moves; no highlighted selection)", status: Status::Missing },
    Row { area: "Text", feature: "Fonts", original: "Its own font library plus fonts on the computer", here: "Fonts on the computer plus a built-in one, with search; arrow keys try each font live on the canvas", status: Status::Partial },
    Row { area: "Text", feature: "Bold and italic", original: "Yes", here: "Yes (made from the regular font when there's no bold/italic file)", status: Status::Same },
    Row { area: "Text", feature: "Letter and line spacing, alignment", original: "Yes", here: "Yes", status: Status::Same },
    Row { area: "Text", feature: "Curve text", original: "Yes", here: "Yes, up and down", status: Status::Same },
    Row { area: "Text", feature: "Ungroup to letters / lines", original: "Yes", here: "To letters", status: Status::Partial },
    Row { area: "Text", feature: "Writing (single-line) fonts for pens", original: "Yes", here: "Outline fonts only", status: Status::Missing },
    // --- images
    Row { area: "Images", feature: "Choose files or drag and drop", original: "File browser and drag and drop", here: "The computer's file picker (several at once) and drag and drop onto the window", status: Status::Same },
    Row { area: "Images", feature: "Convert pictures to SVG", original: "Not offered as a separate tool", here: "SVG converter: photos, drawings and DXF to cut-ready SVG files; Save as SVG from the upload window; export the whole design", status: Status::Extra },
    Row { area: "Images", feature: "Upload SVG and DXF", original: "Yes", here: "Yes, at their real size", status: Status::Same },
    Row { area: "Images", feature: "Upload photos (PNG, JPG, BMP, GIF, WebP)", original: "Yes", here: "Yes", status: Status::Same },
    Row { area: "Images", feature: "Remove background", original: "Click to erase areas, automatic remover", here: "Remove the background colour or keep dark parts, with a live preview", status: Status::Partial },
    Row { area: "Images", feature: "Cut image (trace outline)", original: "Yes", here: "Yes, with smoothing", status: Status::Same },
    Row { area: "Images", feature: "Print then cut image", original: "Yes", here: "Yes, with a cut line around the picture", status: Status::Same },
    // --- making
    Row { area: "Making", feature: "Mats by colour", original: "One mat per colour, pieces packed automatically", here: "One mat per colour, pieces placed in rows", status: Status::Same },
    Row { area: "Making", feature: "Mat sizes and mirror", original: "12x12, 12x24 and more; mirror for iron-on", here: "No mat, 12x12, 12x24 and card mat; page sizes for print then cut; mirror", status: Status::Same },
    Row { area: "Making", feature: "Moving pieces on a mat by hand", original: "Drag pieces, move between mats", here: "Automatic layout only", status: Status::Missing },
    Row { area: "Making", feature: "Material settings", original: "Hundreds of materials downloaded from its service, custom materials", here: "17 common materials (4 for engraving) plus your own custom materials; estimated pressures", status: Status::Partial },
    Row { area: "Making", feature: "Print Then Cut", original: "Sends the page to your printer with sensor marks, then cuts around", here: "Own print mat saved as SVG to print yourself; sensor marks shown but not printed; cutting not yet", status: Status::Partial },
    Row { area: "Making", feature: "Save mats as files", original: "Not offered", here: "Saves each mat as an SVG in millimetres", status: Status::Extra },
    // --- machine
    Row { area: "Machine", feature: "Bluetooth connection", original: "Bluetooth serial (RFCOMM) through its helper program", here: "Bluetooth serial straight to 00:06:66:8D:2B:FE on Linux, or a serial port, or a demo machine", status: Status::Same },
    Row { area: "Machine", feature: "USB connection", original: "Yes", here: "USB cable found automatically by its USB id (COM port on Windows, /dev/ttyACM on Linux); answers status questions", status: Status::Same },
    Row { area: "Machine", feature: "Message format", original: "2-byte length header, optional AES-256", here: "Same format, from the decoded program", status: Status::Same },
    Row { area: "Machine", feature: "Status and service commands", original: "Clear error, read buttons, home blade, stop, find edge...", here: "Same 15 command numbers, with a live message log", status: Status::Same },
    Row { area: "Machine", feature: "Encrypted session", original: "Automatic, using keys built into the program", here: "Sends the request; key must be pasted in (built-in keys not copied)", status: Status::Partial },
    Row { area: "Machine", feature: "Actually cutting your design", original: "Yes", here: "Not yet: Go only wakes, clears errors and homes the blade", status: Status::Missing },
    Row { area: "Machine", feature: "Activation / deactivation lock", original: "The software checks the machine's serial against a disabled list and refuses switched-off machines", here: "No activation check: works with any machine on the link, including one the maker marked off", status: Status::Extra },
    Row { area: "Machine", feature: "Firmware updates, calibration", original: "Yes", here: "No (so it can't send a bricking 'update' either)", status: Status::Missing },
];

pub fn counts() -> [(Status, usize); 4] {
    [Status::Same, Status::Partial, Status::Missing, Status::Extra].map(|s| (s, ROWS.iter().filter(|r| r.status == s).count()))
}

/// Draw the comparison table. `filter` limits it to one status when set.
pub fn show(ui: &mut Ui, filter: &mut Option<Status>) {
    ui.label(
        RichText::new(
            "What the original design app does next to what this app does. Read from the original's files; \
             nothing was copied.",
        )
        .color(crate::app::MUTED()),
    );
    ui.horizontal(|ui| {
        if ui.selectable_label(filter.is_none(), format!("All ({})", ROWS.len())).clicked() {
            *filter = None;
        }
        for (s, n) in counts() {
            let txt = RichText::new(format!("{} ({n})", s.label())).color(s.color());
            if ui.selectable_label(*filter == Some(s), txt).clicked() {
                *filter = if *filter == Some(s) { None } else { Some(s) };
            }
        }
    });
    ui.separator();
    const WIDTHS: [f32; 4] = [190.0, 270.0, 270.0, 80.0];
    let cell = |ui: &mut Ui, w: f32, text: RichText| {
        ui.allocate_ui_with_layout(egui::vec2(w, 0.0), egui::Layout::top_down(egui::Align::Min), |ui| {
            ui.set_width(w);
            ui.add(egui::Label::new(text).wrap());
        });
    };
    let row_ui = |ui: &mut Ui, texts: [RichText; 4], fill: Option<Color32>| {
        let frame = egui::Frame::NONE.inner_margin(egui::Margin::symmetric(6, 4)).fill(fill.unwrap_or(Color32::TRANSPARENT));
        frame.show(ui, |ui| {
            ui.horizontal_top(|ui| {
                for (t, w) in texts.into_iter().zip(WIDTHS) {
                    cell(ui, w, t);
                }
            });
        });
    };
    row_ui(
        ui,
        [
            RichText::new("Feature").strong(),
            RichText::new("Original app").strong(),
            RichText::new("This app").strong(),
            RichText::new("").strong(),
        ],
        None,
    );
    egui::ScrollArea::vertical().max_height(560.0).show(ui, |ui| {
        ui.spacing_mut().item_spacing.y = 0.0;
        let mut last_area = "";
        let mut stripe = false;
        for r in ROWS.iter().filter(|r| filter.map(|f| f == r.status).unwrap_or(true)) {
            if r.area != last_area {
                last_area = r.area;
                ui.add_space(6.0);
                ui.label(RichText::new(r.area).strong().size(15.0).color(crate::app::INK()));
                ui.add_space(2.0);
                stripe = false;
            }
            row_ui(
                ui,
                [
                    RichText::new(r.feature).size(13.0),
                    RichText::new(r.original).size(12.5),
                    RichText::new(r.here).size(12.5),
                    RichText::new(r.status.label()).color(r.status.color()).strong(),
                ],
                if stripe { Some(crate::app::STRIPE()) } else { None },
            );
            stripe = !stripe;
        }
    });
}
