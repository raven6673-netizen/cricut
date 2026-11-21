//! The fonts installed on this computer (Linux: /usr/share/fonts and ~/.local/share/fonts;
//! Windows: C:\Windows\Fonts), found in the background so start-up stays quick.

use std::collections::HashMap;
use std::sync::{Arc, Mutex, OnceLock};

struct Db {
    db: fontdb::Database,
    families: Vec<String>,
}

static DB: OnceLock<Db> = OnceLock::new();
static CACHE: OnceLock<Mutex<HashMap<(String, bool, bool), Arc<(Vec<u8>, u32)>>>> = OnceLock::new();

/// Start reading the computer's fonts in the background.
pub fn start_loading() {
    std::thread::spawn(|| {
        DB.get_or_init(|| {
            let mut db = fontdb::Database::new();
            db.load_system_fonts();
            let mut families: Vec<String> =
                db.faces().filter_map(|f| f.families.first().map(|(n, _)| n.clone())).collect();
            families.sort_by_key(|a| a.to_lowercase());
            families.dedup();
            Db { db, families }
        });
    });
}

/// Font family names, or None while they're still being read.
pub fn families() -> Option<&'static [String]> {
    DB.get().map(|d| d.families.as_slice())
}

/// The bytes and face index of a family in the asked style; the built-in typeface when the family
/// is empty or not found.
pub fn font_data(family: &str, bold: bool, italic: bool) -> Arc<(Vec<u8>, u32)> {
    let builtin = || Arc::new((epaint_default_fonts::UBUNTU_LIGHT.to_vec(), 0u32));
    if family.is_empty() {
        return builtin();
    }
    let cache = CACHE.get_or_init(|| Mutex::new(HashMap::new()));
    let key = (family.to_string(), bold, italic);
    if let Some(v) = cache.lock().unwrap().get(&key) {
        return v.clone();
    }
    let Some(d) = DB.get() else { return builtin() };
    let q = fontdb::Query {
        families: &[fontdb::Family::Name(family)],
        weight: if bold { fontdb::Weight::BOLD } else { fontdb::Weight::NORMAL },
        stretch: fontdb::Stretch::Normal,
        style: if italic { fontdb::Style::Italic } else { fontdb::Style::Normal },
    };
    let found = d.db.query(&q).and_then(|id| d.db.with_face_data(id, |data, idx| (data.to_vec(), idx)));
    let v = match found {
        Some(x) => Arc::new(x),
        None => builtin(),
    };
    cache.lock().unwrap().insert(key, v.clone());
    v
}

/// Whether the asked style exists as its own font file (otherwise it's made from the regular one).
pub fn has_style(family: &str, bold: bool, italic: bool) -> bool {
    let Some(d) = DB.get() else { return false };
    d.db.faces().any(|f| {
        f.families.iter().any(|(n, _)| n == family)
            && (f.weight.0 >= 600) == bold
            && (f.style != fontdb::Style::Normal) == italic
    })
}
