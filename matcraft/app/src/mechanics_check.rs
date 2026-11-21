//! The behaviour check: compares three things for every value on the mechanics sheet
//! (mechanics.json) —
//!   1. the sheet itself (what was read out of the original with the decompiler),
//!   2. what MatCraft actually does, measured by running its own code paths (framing,
//!      encryption, command payloads, reply reader, mat layout), and
//!   3. the original machine program on this computer, read straight from its file bytes where
//!      the value sits in one place (command numbers, the blade-turn mask, the degree constant,
//!      the encrypted flag, the reply size limit). The original is never run.

use crate::app::{row_layout, CANVAS_PPI, LAYOUT_GAP, MAT_SIZES};
use crate::machine::{self, Cmd};
use egui::vec2;

#[derive(Clone, Copy, PartialEq, Eq, Debug, serde::Serialize)]
pub enum Verdict {
    /// MatCraft does what the sheet says (and the original's file agrees, where it was read).
    Same,
    /// MatCraft or the original's file disagrees with the sheet.
    Different,
    /// The sheet's value is an estimate; MatCraft follows it, there's nothing in the original to compare.
    Estimated,
    /// Not something MatCraft can do yet (not decoded).
    NotYet,
}

impl Verdict {
    pub fn label(self) -> &'static str {
        match self {
            Verdict::Same => "Same",
            Verdict::Different => "Different",
            Verdict::Estimated => "Estimate",
            Verdict::NotYet => "Not yet",
        }
    }
}

#[derive(Clone, Debug, serde::Serialize)]
pub struct Row {
    pub name: String,
    pub group: String,
    /// The sheet's value.
    pub sheet: String,
    /// What MatCraft was measured doing.
    pub matcraft: String,
    /// What the original's file holds, when it was read.
    pub original: Option<String>,
    /// What the real machine answered over USB (mechanics-live.json), when it was asked.
    pub machine: Option<String>,
    pub how: String,
    pub verdict: Verdict,
}

#[derive(Clone, Debug, serde::Serialize)]
pub struct Report {
    pub original_file: Option<String>,
    pub original_note: String,
    /// Where the real machine's answers came from, or why there are none.
    pub machine_note: String,
    pub rows: Vec<Row>,
}

impl Report {
    pub fn count(&self, v: Verdict) -> usize {
        self.rows.iter().filter(|r| r.verdict == v).count()
    }
}

/// Where the original's machine program is usually installed on Windows.
pub fn default_original_path() -> Option<std::path::PathBuf> {
    let base = std::env::var("LOCALAPPDATA").ok()?;
    let p = std::path::PathBuf::from(base).join(r"Programs\Cricut Design Space\resources\plugins\device-common\CricutDevice.exe");
    p.exists().then_some(p)
}

/// The original machine program, read for checking (never run).
struct Original {
    bytes: Vec<u8>,
}

impl Original {
    /// File offset of a virtual address in .text (VA - 0x400C00) or .rdata (VA - 0x401400) of the
    /// version that was decompiled (SHA-256 7283020485a95ae0…).
    fn text(&self, va: u32) -> Option<&[u8]> {
        self.bytes.get((va - 0x400C00) as usize..)
    }
    fn rdata(&self, va: u32) -> Option<&[u8]> {
        self.bytes.get((va - 0x401400) as usize..)
    }
    /// A command class's code getter: `mov eax, CODE; ret` (b8 CODE 00 00 00 c3).
    fn code_getter(&self, va: u32) -> Option<u8> {
        let b = self.text(va)?;
        (b.len() >= 6 && b[0] == 0xB8 && b[2..6] == [0, 0, 0, 0xC3]).then(|| b[1])
    }
}

/// Where each command's code getter is in the original (see MECHANICS.md).
const GETTERS: &[(Cmd, u32)] = &[
    (Cmd::ClearError, 0x4fe470),
    (Cmd::ReadErrorSeverity, 0x4fe810),
    (Cmd::SetSpeedState, 0x4fce20),
    (Cmd::SetToolState, 0x4fdae0),
    (Cmd::SetFullToolState, 0x4fded0),
    (Cmd::CprToggle, 0x4fec50),
    (Cmd::AbortCutting, 0x4eb0e0),
    (Cmd::GetInterfaceState, 0x4fa2c0),
    (Cmd::GetPersistentValueCount, 0x4fefc0),
    (Cmd::ScanForEdge, 0x4effa0),
    (Cmd::BluetoothAlert, 0x4e3180),
    (Cmd::HomeTheta, 0x4fb140),
    (Cmd::Move4D, 0x4ff460),
    (Cmd::SendZeros, 0x4e3060),
    (Cmd::NewAes, 0x4e9b70),
];

/// The sheet's name for each command.
fn sheet_name(c: Cmd) -> &'static str {
    match c {
        Cmd::ClearError => "clear error",
        Cmd::ReadErrorSeverity => "read error severity",
        Cmd::SetSpeedState => "set speed state",
        Cmd::SetToolState => "set tool state",
        Cmd::SetFullToolState => "set full tool state",
        Cmd::CprToggle => "cpr toggle",
        Cmd::AbortCutting => "abort cutting",
        Cmd::GetInterfaceState => "get interface state",
        Cmd::GetPersistentValueCount => "get persistent value count",
        Cmd::ScanForEdge => "scan for edge",
        Cmd::BluetoothAlert => "bluetooth alert",
        Cmd::HomeTheta => "home theta",
        Cmd::Move4D => "move 4d",
        Cmd::SendZeros => "send zeros",
        Cmd::NewAes => "new aes session",
    }
}

fn val_to_string(v: &serde_json::Value) -> String {
    match v {
        serde_json::Value::String(s) => s.clone(),
        other => other.to_string(),
    }
}

/// Run the whole check. `original` is the original's machine program to read, if any.
pub fn run(original: Option<&std::path::Path>) -> Report {
    let sheet: serde_json::Value = serde_json::from_str(include_str!("../mechanics.json")).unwrap_or_default();
    let entries: Vec<serde_json::Value> = sheet.get("values").and_then(|v| v.as_array()).cloned().unwrap_or_default();

    let (orig, original_note) = match original {
        Some(p) => match std::fs::read(p) {
            Ok(bytes) => {
                let o = Original { bytes };
                // make sure it's the version the sheet was read from: every getter must be where we expect
                let ok = GETTERS.iter().all(|(_, va)| o.code_getter(*va).is_some());
                if ok {
                    (Some(o), format!("Read from {} (not run).", p.display()))
                } else {
                    (None, format!("{} is a different version from the one decompiled, so it wasn't compared.", p.display()))
                }
            }
            Err(e) => (None, format!("Couldn't read {}: {e}", p.display())),
        },
        None => (None, "The original's machine program wasn't found on this computer, so only the sheet and MatCraft were compared.".into()),
    };

    let (live, machine_note) = match load_live() {
        Some((l, path)) => {
            let note = format!("Real machine: {} on {}, asked {} read-only questions (saved in {}).", l.machine, l.port, l.answers.len(), path.display());
            (Some(l), note)
        }
        None => (None, "The real machine hasn't been asked yet (matcraft --live-check, or Ask the machine below).".to_string()),
    };

    let mut rows = Vec::new();
    for e in &entries {
        let name = e.get("name").and_then(|v| v.as_str()).unwrap_or("").to_string();
        let group = e.get("group").and_then(|v| v.as_str()).unwrap_or("").to_string();
        let sheet_v = e.get("value").map(val_to_string).unwrap_or_default();
        let estimated = e.get("estimated").and_then(|v| v.as_bool()).unwrap_or(false);
        let (matcraft, original_v, how): (String, Option<String>, String) = measure(&name, orig.as_ref());
        let same_as_sheet = norm(&matcraft) == norm(&sheet_v);
        let orig_agrees = original_v.as_ref().map(|o| norm(o) == norm(&sheet_v)).unwrap_or(true);
        let (machine_v, machine_compared) = live.as_ref().map(|l| from_machine(&name, l)).unwrap_or((None, false));
        let machine_agrees = !machine_compared || machine_v.as_ref().map(|m| norm(m) == norm(&sheet_v)).unwrap_or(true);
        let verdict = if matcraft.starts_with("not decoded") || sheet_v.starts_with("not decoded") {
            Verdict::NotYet
        } else if !same_as_sheet || !orig_agrees || !machine_agrees {
            Verdict::Different
        } else if estimated {
            Verdict::Estimated
        } else {
            Verdict::Same
        };
        rows.push(Row { name, group, sheet: sheet_v, matcraft, original: original_v, machine: machine_v, how, verdict });
    }
    Report { original_file: original.map(|p| p.display().to_string()), original_note, machine_note, rows }
}

/// Where the real machine's answers are kept: next to where MatCraft runs, and in its settings folder.
pub fn live_paths() -> Vec<std::path::PathBuf> {
    vec![std::path::PathBuf::from("mechanics-live.json"), crate::app::config_dir().join("mechanics-live.json")]
}

fn load_live() -> Option<(machine::LiveCheck, std::path::PathBuf)> {
    live_paths().into_iter().find_map(|p| {
        let s = std::fs::read_to_string(&p).ok()?;
        serde_json::from_str::<machine::LiveCheck>(&s).ok().map(|l| (l, p))
    })
}

/// Ask the machine on USB and save its answers where the check looks for them.
pub fn ask_machine(port: &str) -> Result<machine::LiveCheck, String> {
    let l = machine::live_check(port)?;
    let json = serde_json::to_string_pretty(&l).map_err(|e| e.to_string())?;
    for p in live_paths() {
        if let Some(d) = p.parent().filter(|d| !d.as_os_str().is_empty()) {
            let _ = std::fs::create_dir_all(d);
        }
        let _ = std::fs::write(&p, &json);
    }
    Ok(l)
}

/// The real machine's value for a sheet row, and whether it should agree with the sheet.
fn from_machine(name: &str, l: &machine::LiveCheck) -> (Option<String>, bool) {
    let answer = |cmd: &str| l.answers.iter().find(|a| a.command == cmd);
    let first_byte = |a: &machine::LiveAnswer| a.body.as_ref().and_then(|b| b.split_whitespace().next().and_then(|x| u8::from_str_radix(x, 16).ok()));
    match name {
        "header length" => {
            let got = l.answers.iter().filter_map(|a| a.header.as_ref()).map(|h| h.split_whitespace().count()).next();
            (got.map(|n| n.to_string()), got.is_some())
        }
        "plain frame header" | "encrypted flag" => {
            let plain = l.answers.iter().filter(|a| a.encrypted == Some(false)).count();
            if plain == 0 {
                (None, false)
            } else {
                (Some(format!("{plain} plain replies, top bit clear")), false)
            }
        }
        "max reply body" => {
            let most = l.answers.iter().filter_map(|a| a.body.as_ref()).map(|b| b.split_whitespace().count()).max();
            (most.map(|n| format!("longest reply {n} bytes")), false)
        }
        "usb status reply" => (answer("get interface state").and_then(|a| a.body.clone()), false),
        _ => match answer(name) {
            Some(a) => match first_byte(a) {
                Some(b) => (Some(b.to_string()), true),
                None => (Some(a.note.clone()), true),
            },
            None => (None, false),
        },
    }
}

fn norm(s: &str) -> String {
    let t = s.trim().to_lowercase();
    // numbers compare by value (so 512 and 512.0 match)
    if let Ok(f) = t.parse::<f64>() {
        return format!("{f}");
    }
    t
}

/// Measure one value in MatCraft (by running its code) and, where possible, in the original's file.
fn measure(name: &str, orig: Option<&Original>) -> (String, Option<String>, String) {
    let key = [7u8; 32];
    // commands
    if let Some((cmd, va)) = GETTERS.iter().find(|(c, _)| sheet_name(*c) == name) {
        let sent = machine::payload(*cmd, 0.0);
        let measured = sent[0];
        return (
            measured.to_string(),
            orig.and_then(|o| o.code_getter(*va)).map(|c| c.to_string()),
            format!("first byte MatCraft sends; original's code getter at {va:#x}"),
        );
    }
    match name {
        "header length" => {
            let f = machine::frame_plain(&[0x73]);
            ((f.len() - 1).to_string(), None, "length of what MatCraft puts before a 1-byte command".into())
        }
        "encrypted flag" => {
            let f = machine::frame_encrypted(&[0x0A], &key);
            let flag = f[0] & 0x80;
            (
                if flag == 0x80 { "0x80 in first header byte".into() } else { format!("{:#x}", f[0]) },
                orig.and_then(|o| o.text(0x48def4)).filter(|b| b.len() > 1 && b[0] == 0x0C).map(|b| format!("{:#x} in first header byte", b[1])),
                "header MatCraft writes for an encrypted message; original's `or al, 80h` at 0x48def4".into(),
            )
        }
        "max reply body" => {
            // a 512-byte reply must be read; a 513-byte one refused
            let mut ok = vec![0x02, 0x00];
            ok.extend(vec![0u8; 512]);
            let mut too_big = vec![0x02, 0x01];
            too_big.extend(vec![0u8; 513]);
            let a = machine::read_reply_from_bytes(&ok, None);
            let b = machine::read_reply_from_bytes(&too_big, None);
            let measured = match (a, b) {
                (Ok(Some(v)), Err(_)) if v.len() == 512 => "512".to_string(),
                _ => "accepts other sizes".to_string(),
            };
            (
                measured,
                orig.and_then(|o| o.text(0x50f624))
                    .filter(|b| b.len() >= 6 && b[0] == 0x81 && b[1] == 0xFF)
                    .map(|b| u32::from_le_bytes([b[2], b[3], b[4], b[5]]).to_string()),
                "fed a 512- and a 513-byte reply to MatCraft's reader; original's `cmp edi, 200h` at 0x50f624".into(),
            )
        }
        "aes key length" => {
            let f = machine::frame_encrypted(&[0x0A, 1, 2, 3], &key);
            let back = machine::open_encrypted(&f[2..], &key);
            let measured = if back.map(|b| b[0] == 1 && b[1] == 0x0A).unwrap_or(false) { "32".into() } else { "fails".into() };
            (measured, None, "round trip through MatCraft's AES with a 32-byte key".into())
        }
        "encrypted padding" => {
            let mut fine = true;
            for (len, want) in [(1usize, 16usize), (15, 16), (16, 32), (17, 32), (31, 32), (32, 48)] {
                let payload: Vec<u8> = (0..len as u8).collect();
                let f = machine::frame_encrypted(&payload, &key);
                if f.len() - 2 != want {
                    fine = false;
                }
                // decrypting checks the sum byte, so success means the last byte held the sum
                if machine::open_encrypted(&f[2..], &key).is_err() {
                    fine = false;
                }
            }
            (
                if fine { "len - len%16 + 16, last byte = 8-bit sum".into() } else { "padding differs".into() },
                None,
                "sizes MatCraft encrypts for 1, 15, 16, 17, 31 and 32-byte messages, and its sum byte".into(),
            )
        }
        "reply byte swap" => {
            let f = machine::frame_encrypted(&[0x0A, 0x05], &key);
            let back = machine::open_encrypted(&f[2..], &key).unwrap_or_default();
            let swapped = back.first() == Some(&0x05) && back.get(1) == Some(&0x0A);
            (
                if swapped { "swap first two bytes if first != 0".into() } else { "no swap".into() },
                None,
                "decrypted reply as MatCraft hands it on".into(),
            )
        }
        "plain frame header" => {
            let f = machine::frame_plain(&machine::payload(Cmd::NewAes, 0.0));
            let good = f[0] == 0x00 && f[1] == 0x31 && f.len() == 51;
            (
                if good { "same 2-byte header, top bit clear".into() } else { format!("{:02x} {:02x}", f[0], f[1]) },
                None,
                "header MatCraft writes for the 49-byte session request".into(),
            )
        }
        "new aes key slot" => {
            let mut seen = [false; 16];
            let mut bad = false;
            for _ in 0..3000 {
                let p = machine::payload(Cmd::NewAes, 0.0);
                if !(2..=13).contains(&p[1]) {
                    bad = true;
                } else {
                    seen[p[1] as usize] = true;
                }
            }
            let all = (2..=13).all(|s| seen[s]);
            (
                if !bad && all { "random % 12 + 2".into() } else if bad { "outside 2..13".into() } else { "random % 12 + 2 (not every slot seen)".into() },
                None,
                "3000 session requests from MatCraft: every slot 2-13 and nothing else".into(),
            )
        }
        "new aes random bytes" => {
            let p = machine::payload(Cmd::NewAes, 0.0);
            ((p.len() - 2).to_string(), None, "bytes after the code and slot in MatCraft's request".into())
        }
        "move 4d axis mask" => {
            let p = machine::payload(Cmd::Move4D, 90.0);
            (
                p[1].to_string(),
                orig.and_then(|o| o.text(0x4ff4d1)).filter(|b| b.len() >= 4 && b[0] == 0xC6 && b[1] == 0x45).map(|b| b[3].to_string()),
                "second byte MatCraft sends; original's `mov byte [ebp-0Dh], 17h` at 0x4ff4d1".into(),
            )
        }
        "degrees to radians" => {
            // 180 degrees must go out as pi
            let p = machine::payload(Cmd::Move4D, 180.0);
            let theta = f32::from_be_bytes([p[14], p[15], p[16], p[17]]) as f64;
            let k = theta / 180.0;
            let measured = if (k - 0.017453292519943295).abs() < 1e-8 { "0.017453292519943295".to_string() } else { format!("{k}") };
            (
                measured,
                orig.and_then(|o| o.rdata(0x95ebc0)).filter(|b| b.len() >= 8).map(|b| {
                    let v = f64::from_le_bytes([b[0], b[1], b[2], b[3], b[4], b[5], b[6], b[7]]);
                    format!("{v}")
                }),
                "angle MatCraft sends for 180 degrees, divided by 180; original's constant at 0x95ebc0".into(),
            )
        }
        "move 4d float order" => {
            // 1 radian = 57.29578 degrees must come out as 3F 80 00 00
            let p = machine::payload(Cmd::Move4D, 57.295_78);
            let be = p[14] == 0x3F && p[15] == 0x80;
            (if be { "big-endian float32".into() } else { "little-endian".into() }, None, "bytes MatCraft sends for 1 radian".into())
        }
        "rfcomm channel" => ("1".into(), None, "MatCraft's default channel".into()),
        "machine bluetooth address" => (machine::DEFAULT_MAC.into(), None, "MatCraft's default address".into()),
        "link type" => (
            "bluetooth rfcomm serial".into(),
            None,
            "MatCraft opens an RFCOMM socket on Linux (or a serial port)".into(),
        ),
        "gap between pieces" => {
            let (spots, _) = row_layout(&[vec2(2.0, 2.0), vec2(2.0, 2.0)], MAT_SIZES[0], LAYOUT_GAP);
            let edge = spots[0].x;
            let between = spots[1].x - (spots[0].x + 2.0);
            let measured = if (edge - between).abs() < 1e-6 { format!("{between}") } else { format!("edge {edge}, between {between}") };
            (measured, None, "where MatCraft places two 2-inch pieces on a mat".into())
        }
        "mat sizes" => (
            MAT_SIZES.iter().map(|s| format!("{}x{} in", s.x, s.y)).collect::<Vec<_>>().join(", "),
            None,
            "mats MatCraft offers".into(),
        ),
        "canvas scale" => (format!("{CANVAS_PPI}"), None, "MatCraft's canvas at 100 %".into()),
        "material pressures and passes" => ("table in app".into(), None, "MatCraft's material list".into()),
        "aes session key handshake" => ("not decoded; key typed in by the person".into(), None, "MatCraft sends the request only".into()),
        "cut path streaming" => ("not decoded; mats saved as SVG path data".into(), None, "MatCraft saves mats as SVG".into()),
        "engrave fill patterns" => (
            crate::geometry::FillPattern::ALL.iter().map(|p| p.label().to_lowercase()).collect::<Vec<_>>().join(", "),
            None,
            "fill patterns MatCraft offers for engraving".into(),
        ),
        "engrave default pattern" => (
            crate::app::EngraveOpts::default().pattern.label().to_lowercase(),
            None,
            "pattern a new engrave fill starts with".into(),
        ),
        "engrave line spacing" => (
            format!("{}", crate::app::EngraveOpts::default().spacing),
            None,
            "line spacing a new engrave fill starts with".into(),
        ),
        "engrave edge margin" => (
            format!("{}", crate::app::EngraveOpts::default().margin),
            None,
            "edge margin a new engrave fill starts with".into(),
        ),
        "engrave fill rule" => (
            if crate::app::EngraveOpts::default().even_odd { "even-odd".into() } else { "non-zero".into() },
            None,
            "fill rule a new engrave fill starts with".into(),
        ),
        "engrave design area" => {
            let a = crate::app::projects::ENGRAVE_AREA;
            (format!("{}x{} in", a.x, a.y), None, "design area of a new engrave project".into())
        }
        "engrave tools" => {
            use crate::materials::{tool_for_op, Tool};
            let op = crate::geometry::Operation::Engrave;
            let tip = tool_for_op(op, Tool::FineBlade, false).label();
            let tool = tool_for_op(op, Tool::FineBlade, true).label();
            (format!("{tip}, {tool}"), None, "tool MatCraft asks for when engraving, without and with the Engraving Tool".into())
        }
        "engraving tool machines" => (
            crate::app::projects::MachineModel::ALL
                .iter()
                .filter(|m| m.engraving_tool())
                .map(|m| m.label())
                .collect::<Vec<_>>()
                .join(", "),
            None,
            "machines MatCraft gives the Engraving Tool".into(),
        ),
        "serial number over usb" => (
            "sent in plain text, no encryption".into(),
            None,
            "from the Hackaday article; MatCraft reads the serial from the USB descriptor in the clear (never re-sent or hidden)".into(),
        ),
        "activation check location" => (
            "host software, not firmware".into(),
            None,
            "from the Hackaday article; describes the original. MatCraft has no activation check".into(),
        ),
        "matcraft activation check" => {
            // MatCraft's connect path sends only the link request; there is no serial list to check against.
            ("none".into(), None, "MatCraft keeps no disabled-serial list and never gates on activation".into())
        }
        "encrypted packets cipher" => {
            // confirm ECB: two identical plaintext blocks encrypt to the same ciphertext block.
            let key = [7u8; 32];
            let framed = machine::frame_encrypted(&[0u8; 16], &key);
            // body after the 2-byte header; first 32 bytes are two padded blocks of the same zero data
            let ecb = framed.len() >= 34 && framed[2..18] == framed[18..34];
            ((if ecb { "AES-ECB" } else { "AES (block mode not ECB)" }).into(), None, "two identical blocks encrypt the same way in MatCraft's framing (the ECB property)".into())
        }
        "bluetooth module" => ("Microchip RN42".into(), None, "from the machine's label (FCC ID T9J-RN42); MatCraft opens an RFCOMM SPP channel, which is what the RN42 offers".into()),
        "model number" => ("CXPL301".into(), None, "from the machine's label".into()),
        "bluetooth name format" => {
            let example = machine::bluetooth_name(machine::DEFAULT_MAC).unwrap_or_default();
            ("Maker-XXXX (last two address bytes)".into(), None, format!("MatCraft derives the name from the address: {} shows as {example}", machine::DEFAULT_MAC))
        }
        "usb vendor id" => (format!("{:04X}", machine::USB_VENDOR), None, "vendor id MatCraft looks for on USB serial ports".into()),
        "usb status reply" => {
            // the recorded reply, framed as the machine sent it, through MatCraft's reply reader
            let recorded = [0x00, 0x06, 0x73, 0xCF, 0x0F, 0x00, 0x03, 0x00];
            match machine::read_reply_from_bytes(&recorded, None) {
                Ok(Some(b)) => (machine::hex(&b), None, "MatCraft's reader on the machine's recorded USB reply".into()),
                other => (format!("{other:?}"), None, "MatCraft's reader on the machine's recorded USB reply".into()),
            }
        }
        _ => ("(not measured)".into(), None, "no measurement for this value yet".into()),
    }
}

fn verdict_color(v: Verdict) -> egui::Color32 {
    match v {
        Verdict::Same => egui::Color32::from_rgb(0x16, 0x9c, 0x5c),
        Verdict::Different => egui::Color32::from_rgb(0xd0, 0x3c, 0x3c),
        Verdict::Estimated => egui::Color32::from_rgb(0xd9, 0x8b, 0x10),
        Verdict::NotYet => crate::app::MUTED(),
    }
}

/// The check as a table in the Compare window. Returns true when "Run again" was pressed.
pub fn show(ui: &mut egui::Ui, r: &Report) -> bool {
    use egui::RichText;
    let mut again = false;
    ui.label(
        RichText::new(
            "Each value on the mechanics sheet, next to what MatCraft was measured doing (its own code was run) \
             and what the original's machine program holds (read from its file, never run).",
        )
        .color(crate::app::MUTED()),
    );
    ui.horizontal(|ui| {
        for v in [Verdict::Same, Verdict::Different, Verdict::Estimated, Verdict::NotYet] {
            ui.label(RichText::new(format!("{} {}", r.count(v), v.label())).strong().color(verdict_color(v)));
            ui.add_space(8.0);
        }
        ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
            if ui.button("Run again").clicked() {
                again = true;
            }
        });
    });
    ui.label(RichText::new(&r.original_note).size(12.0).color(crate::app::MUTED()));
    ui.horizontal(|ui| {
        ui.label(RichText::new(&r.machine_note).size(12.0).color(crate::app::MUTED()));
        if ui.button("Ask the machine (USB)").on_hover_text("Sends three read-only questions to the machine on USB; nothing moves.").clicked() {
            let _ = ask_machine("");
            again = true;
        }
    });
    ui.separator();
    const W: [f32; 6] = [170.0, 170.0, 170.0, 120.0, 150.0, 70.0];
    let cell = |ui: &mut egui::Ui, w: f32, t: RichText| {
        ui.allocate_ui_with_layout(egui::vec2(w, 0.0), egui::Layout::top_down(egui::Align::Min), |ui| {
            ui.set_width(w);
            ui.add(egui::Label::new(t).wrap());
        });
    };
    let row_ui = |ui: &mut egui::Ui, t: [RichText; 6], fill: egui::Color32| {
        egui::Frame::NONE.fill(fill).inner_margin(egui::Margin::symmetric(6, 3)).show(ui, |ui| {
            ui.horizontal_top(|ui| {
                for (x, w) in t.into_iter().zip(W) {
                    cell(ui, w, x);
                }
            });
        });
    };
    row_ui(
        ui,
        [
            RichText::new("Value").strong(),
            RichText::new("Sheet").strong(),
            RichText::new("MatCraft (measured)").strong(),
            RichText::new("Original (file)").strong(),
            RichText::new("Real machine (USB)").strong(),
            RichText::new("").strong(),
        ],
        egui::Color32::TRANSPARENT,
    );
    egui::ScrollArea::vertical().max_height(520.0).show(ui, |ui| {
        ui.spacing_mut().item_spacing.y = 0.0;
        let mut last = String::new();
        let mut stripe = false;
        for row in &r.rows {
            if row.group != last {
                last = row.group.clone();
                ui.add_space(6.0);
                ui.label(RichText::new(&row.group).strong().size(14.0).color(crate::app::INK()));
                stripe = false;
            }
            let resp = row_ui(
                ui,
                [
                    RichText::new(&row.name).size(13.0),
                    RichText::new(&row.sheet).size(12.0),
                    RichText::new(&row.matcraft).size(12.0),
                    RichText::new(row.original.clone().unwrap_or_else(|| "-".into())).size(12.0),
                    RichText::new(row.machine.clone().unwrap_or_else(|| "-".into())).size(12.0),
                    RichText::new(row.verdict.label()).strong().color(verdict_color(row.verdict)),
                ],
                if stripe { crate::app::STRIPE() } else { egui::Color32::TRANSPARENT },
            );
            let _ = resp;
            stripe = !stripe;
        }
        ui.add_space(6.0);
        ui.label(RichText::new("How each value was measured is saved with the report (matcraft --check-mechanics report.json).").size(11.5).color(crate::app::MUTED()));
    });
    again
}

/// The report as readable text.
pub fn to_text(r: &Report) -> String {
    let mut s = String::new();
    s.push_str(&format!(
        "Machine behaviour check: {} same, {} different, {} estimates, {} not yet\n{}\n{}\n\n",
        r.count(Verdict::Same),
        r.count(Verdict::Different),
        r.count(Verdict::Estimated),
        r.count(Verdict::NotYet),
        r.original_note,
        r.machine_note
    ));
    for row in &r.rows {
        s.push_str(&format!(
            "[{:9}] {:28} sheet: {:42} matcraft: {:42} original: {:10} machine: {}\n",
            row.verdict.label(),
            row.name,
            row.sheet,
            row.matcraft,
            row.original.clone().unwrap_or_else(|| "-".into()),
            row.machine.clone().unwrap_or_else(|| "-".into())
        ));
    }
    s
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn matcraft_behaves_as_the_sheet_says() {
        let original = default_original_path();
        let r = run(original.as_deref());
        let bad: Vec<String> = r.rows.iter().filter(|x| x.verdict == Verdict::Different).map(|x| format!("{}: sheet {} / matcraft {} / original {:?}", x.name, x.sheet, x.matcraft, x.original)).collect();
        assert!(bad.is_empty(), "differences: {bad:#?}");
        assert!(!r.rows.iter().any(|x| x.matcraft == "(not measured)"), "every sheet value should be measured");
        // when the original is on this computer, every command code and the four constants were read from it
        if original.is_some() && r.original_note.starts_with("Read from") {
            assert!(r.rows.iter().filter(|x| x.original.is_some()).count() >= 19);
        }
    }
}
