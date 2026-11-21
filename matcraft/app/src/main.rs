//! MatCraft: design shapes and text on a cutting mat and drive a Bluetooth cutting machine.
//! Built for Linux (Bazzite); also runs on Windows.

mod app;
mod compare;
mod fonts;
mod imaging;
mod mechanics_check;
mod geometry;
mod machine;
mod materials;

use std::sync::{Arc, Mutex};

/// Wraps the app for `--screenshot <file.png>`: draw ~120 frames, save the window and quit.
struct Shot {
    inner: app::App,
    path: std::path::PathBuf,
    frames: u32,
    requested: bool,
    image: Arc<Mutex<Option<Arc<egui::ColorImage>>>>,
}

impl eframe::App for Shot {
    fn ui(&mut self, ui: &mut egui::Ui, frame: &mut eframe::Frame) {
        self.inner.ui(ui, frame);
        let ctx = ui.ctx().clone();
        self.frames += 1;
        if self.frames >= 120 && !self.requested {
            self.requested = true;
            let slot = self.image.clone();
            let c2 = ctx.clone();
            ctx.request_screenshot(move |img| {
                *slot.lock().unwrap() = Some(img);
                c2.request_repaint();
            });
        }
        let taken = self.image.lock().unwrap().take();
        if let Some(img) = taken {
            let [w, h] = img.size;
            let mut buf = image::RgbaImage::new(w as u32, h as u32);
            for (i, px) in img.pixels.iter().enumerate() {
                let [r, g, b, a] = px.to_srgba_unmultiplied();
                buf.put_pixel((i % w) as u32, (i / w) as u32, image::Rgba([r, g, b, a]));
            }
            if let Some(d) = self.path.parent() {
                let _ = std::fs::create_dir_all(d);
            }
            match buf.save(&self.path) {
                Ok(()) => eprintln!("saved {}", self.path.display()),
                Err(e) => eprintln!("couldn't save the picture: {e}"),
            }
            ctx.send_viewport_cmd(egui::ViewportCommand::Close);
        }
        // keep drawing even without focus
        ctx.request_repaint();
        if self.frames > 2000 {
            ctx.send_viewport_cmd(egui::ViewportCommand::Close);
        }
    }
}

fn main() -> eframe::Result<()> {
    let args: Vec<String> = std::env::args().collect();

    // `--check-mechanics [report.json] [--original <CricutDevice.exe>]`: compare MatCraft's machine
    // behaviour with the mechanics sheet and the original's file, print it, and exit (no window).
    if let Some(i) = args.iter().position(|a| a == "--live-check") {
        let port = args.get(i + 1).filter(|a| !a.starts_with("--")).cloned().unwrap_or_default();
        match mechanics_check::ask_machine(&port) {
            Ok(l) => {
                println!("Asked {} on {}", l.machine, l.port);
                for a in &l.answers {
                    println!(
                        "{:28} sent {:10} header {:6} body {:24} {}",
                        a.command,
                        a.sent,
                        a.header.clone().unwrap_or_else(|| "-".into()),
                        a.body.clone().unwrap_or_else(|| "-".into()),
                        a.note
                    );
                }
                println!("Saved mechanics-live.json");
            }
            Err(e) => {
                println!("{e}");
                std::process::exit(1);
            }
        }
        return Ok(());
    }
    if let Some(i) = args.iter().position(|a| a == "--usb-probe") {
        let port = args.get(i + 1).filter(|a| !a.starts_with("--")).cloned().unwrap_or_default();
        for line in machine::probe_usb(&port, std::time::Duration::from_secs(5)) {
            println!("{line}");
        }
        return Ok(());
    }
    // `--cut-dry-run`: encode a 1-inch test square as a cut job and print each command's framed
    // bytes, without opening any device. Verifies the cut encoder against CUT-PROTOCOL.md.
    if args.iter().any(|a| a == "--cut-dry-run") {
        let square = vec![vec![(1.0f32, 1.0), (2.0, 1.0), (2.0, 2.0), (1.0, 2.0)]];
        let records = machine::contours_to_records(&square, true);
        let setup = machine::CutSetup { width_in: 12, height_in: 12, pressure: 150 };
        let mut payloads = machine::setup_payloads(&setup);
        payloads.extend(machine::cut_job_payloads(&records, machine::MULTIDRAW_MAX_RECORDS));
        println!(
            "Cut dry run: a 1-inch square at (1,1) in, pressure {} counts — {} records, {} commands (plain frames):",
            setup.pressure,
            records.len(),
            payloads.len()
        );
        for (i, body) in payloads.iter().enumerate() {
            let frame = machine::frame_plain(body);
            println!("  {}/{}: 0x{:02X}  {}", i + 1, payloads.len(), body[0], machine::hex(&frame));
        }
        return Ok(());
    }
    if let Some(i) = args.iter().position(|a| a == "--check-mechanics") {
        let original = args
            .iter()
            .position(|a| a == "--original")
            .and_then(|j| args.get(j + 1))
            .map(std::path::PathBuf::from)
            .or_else(mechanics_check::default_original_path);
        let report = mechanics_check::run(original.as_deref());
        print!("{}", mechanics_check::to_text(&report));
        if let Some(out) = args.get(i + 1).filter(|a| !a.starts_with("--")) {
            if let Ok(json) = serde_json::to_string_pretty(&report) {
                let _ = std::fs::write(out, json);
                println!("\nSaved {out}");
            }
        }
        std::process::exit(if report.count(mechanics_check::Verdict::Different) == 0 { 0 } else { 1 });
    }
    let shot = args.iter().position(|a| a == "--screenshot").and_then(|i| args.get(i + 1)).map(std::path::PathBuf::from);

    let mut viewport = egui::ViewportBuilder::default()
        .with_title("MatCraft")
        .with_inner_size([1440.0, 900.0])
        .with_min_inner_size([980.0, 620.0])
        .with_drag_and_drop(true);
    if shot.is_some() {
        viewport = viewport.with_active(false);
        #[cfg(windows)]
        {
            viewport = viewport.with_window_level(egui::WindowLevel::AlwaysOnBottom);
        }
    }
    let options = eframe::NativeOptions { viewport, ..Default::default() };

    eframe::run_native(
        "MatCraft",
        options,
        Box::new(move |cc| {
            let mut inner = app::App::new(cc);
            if std::env::args().any(|a| a == "--make") {
                inner.open_make();
            }
            let argv: Vec<String> = std::env::args().collect();
            if let Some(n) = argv.iter().position(|a| a == "--select").and_then(|i| argv.get(i + 1)).and_then(|s| s.parse().ok()) {
                inner.select_index(n);
            }
            if let Some(n) = argv.iter().position(|a| a == "--edit").and_then(|i| argv.get(i + 1)).and_then(|s| s.parse().ok()) {
                inner.edit_index(n);
            }
            if let Some(d) = argv.iter().position(|a| a == "--drawer").and_then(|i| argv.get(i + 1)) {
                inner.open_drawer(d);
            }
            if let Some(f) = argv.iter().position(|a| a == "--upload").and_then(|i| argv.get(i + 1)) {
                inner.upload_path(f);
            }
            if std::env::args().any(|a| a == "--compare") {
                inner.open_compare();
            }
            if std::env::args().any(|a| a == "--engrave-sample") {
                inner.engrave_sample();
            }
            if let Some(n) = argv.iter().position(|a| a == "--settings").and_then(|i| argv.get(i + 1)).and_then(|s| s.parse().ok()) {
                inner.open_settings(n);
            }
            if std::env::args().any(|a| a == "--new-project") {
                inner.open_picker();
            }
            if std::env::args().any(|a| a == "--machine") {
                inner.open_machine_panel();
            }
            Ok(match shot {
                Some(path) => Box::new(Shot {
                    inner,
                    path,
                    frames: 0,
                    requested: false,
                    image: Arc::new(Mutex::new(None)),
                }) as Box<dyn eframe::App>,
                None => Box::new(inner),
            })
        }),
    )
}
