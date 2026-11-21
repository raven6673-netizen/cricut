//! Project types and their templates ("What do you want to make?"), and the app's settings.
//! Template names and the sizes marked `seen` come from the original's project picker (the
//! person's screenshots); other sizes are estimates.

use egui::{vec2, Vec2};

#[derive(Clone, Copy, PartialEq, Eq, Debug, serde::Serialize, serde::Deserialize, Default)]
pub enum ProjectType {
    #[default]
    Blank,
    VinylDecal,
    IronOn,
    Card,
    CardstockCutout,
    Stickers,
    Engrave,
    IronOnAccessories,
    Dtf,
}

impl ProjectType {
    /// In the order the original lists them.
    pub const ALL: [ProjectType; 9] = [
        ProjectType::VinylDecal,
        ProjectType::IronOn,
        ProjectType::Card,
        ProjectType::CardstockCutout,
        ProjectType::Stickers,
        ProjectType::Engrave,
        ProjectType::IronOnAccessories,
        ProjectType::Dtf,
        ProjectType::Blank,
    ];
    pub fn label(self) -> &'static str {
        match self {
            ProjectType::Blank => "Blank Canvas",
            ProjectType::VinylDecal => "Vinyl Decal",
            ProjectType::IronOn => "Iron-On T-Shirt",
            ProjectType::Card => "Card",
            ProjectType::CardstockCutout => "Cardstock Cutout",
            ProjectType::Stickers => "Stickers & Labels",
            ProjectType::Engrave => "Engrave",
            ProjectType::IronOnAccessories => "Iron-on Accessories",
            ProjectType::Dtf => "DTF Transfer",
        }
    }
    /// What new shapes, text and pictures are set to in this kind of project.
    pub fn default_op(self) -> crate::geometry::Operation {
        use crate::geometry::Operation;
        match self {
            ProjectType::Engrave => Operation::Engrave,
            ProjectType::Stickers | ProjectType::Dtf => Operation::Print,
            _ => Operation::Cut,
        }
    }
    /// Whether layers stay where you placed them on the mat (the original's keep-layout list:
    /// iron-on accessories, cards, cardstock, stickers, constrained canvas and engrave).
    pub fn keeps_layout(self) -> bool {
        matches!(
            self,
            ProjectType::Card | ProjectType::CardstockCutout | ProjectType::Stickers | ProjectType::Engrave | ProjectType::IronOnAccessories
        )
    }
    /// Iron-on designs are cut mirrored.
    pub fn mirrors(self) -> bool {
        matches!(self, ProjectType::IronOn | ProjectType::IronOnAccessories)
    }
    /// The short "Tools & Materials needed" note.
    pub fn needs(self) -> &'static str {
        match self {
            ProjectType::VinylDecal => "Adhesive vinyl, transfer tape, a weeding tool and the fine-point blade.",
            ProjectType::IronOn => "Iron-on (heat transfer) material, a heat press or iron, and the fine-point blade. Designs are cut mirrored.",
            ProjectType::Card => "Cardstock, the fine-point blade, and a scoring tool or pen for details.",
            ProjectType::CardstockCutout => "Cardstock and the fine-point blade; heavier card may need two passes.",
            ProjectType::Stickers => "Printable sticker paper and an inkjet printer. Calibrate print then cut first for the best results.",
            ProjectType::Engrave => "Engravable material (aluminum, acrylic, leather...), the Engrave Tip or Engraving Tool, and tape.",
            ProjectType::IronOnAccessories => "Iron-on material, a heat press, and the item. Designs are cut mirrored.",
            ProjectType::Dtf => "DTF film and a DTF-capable printer; prints the whole design.",
            ProjectType::Blank => "Anything you like.",
        }
    }
}

/// One choice in the project picker: a thing to make, with the design area for it.
#[derive(Clone, Debug)]
pub struct Template {
    pub name: &'static str,
    /// Size choices: label and design area (width, height) in inches.
    pub sizes: &'static [(&'static str, f32, f32)],
    /// Simple drawing used for its tile.
    pub icon: Icon,
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub enum Icon {
    Bottle,
    Notebook,
    Mug,
    Car,
    Wineglass,
    Phone,
    Glass,
    Tumbler,
    Shirt,
    Bodysuit,
    Card,
    Tag,
    Bookmark,
    Holder,
    Sheet,
    Labels,
    Hat,
    Pillow,
    Bag,
    Plate,
    Plus,
}

pub fn templates(t: ProjectType) -> Vec<Template> {
    use Icon::*;
    let tpl = |name, sizes, icon| Template { name, sizes, icon };
    match t {
        ProjectType::VinylDecal => vec![
            tpl("Water Bottle", &[("12 oz / 355 ml", 2.0, 3.0), ("24 oz / 710 ml", 2.5, 4.0)], Bottle), // 12 oz seen
            tpl("Notebook", &[("Standard", 4.5, 6.0)], Notebook),
            tpl("Mug", &[("11 oz", 3.5, 3.0), ("15 oz", 3.75, 3.5)], Mug),
            tpl("Car Window", &[("Small", 6.0, 3.0), ("Large", 10.0, 4.0)], Car),
            tpl("Stemware", &[("Standard", 2.0, 2.0)], Wineglass),
            tpl("Phone", &[("Standard", 2.5, 4.5)], Phone),
            tpl("Glass", &[("16 oz", 3.0, 3.5)], Glass),
            tpl("Tumbler", &[("20 oz", 3.0, 5.0), ("40 oz", 3.5, 6.0)], Tumbler),
            tpl("Custom Vinyl Decal", &[("Custom", 6.0, 6.0)], Plus),
        ],
        ProjectType::IronOn => vec![
            tpl("Men's T-Shirt", &[("Small", 9.5, 9.5), ("Medium", 10.0, 10.0), ("Large", 11.0, 11.0), ("X-Large", 12.0, 12.0)], Shirt), // Small seen
            tpl("Women's T-Shirt", &[("Small", 8.5, 8.5), ("Medium", 9.0, 9.0), ("Large", 10.0, 10.0)], Shirt),
            tpl("Baby Bodysuit", &[("0-6 months", 4.0, 4.0), ("6-12 months", 5.0, 5.0)], Bodysuit),
            tpl("Child/Youth T-Shirt", &[("Small", 7.0, 7.0), ("Medium", 8.0, 8.0), ("Large", 9.0, 9.0)], Shirt),
            tpl("Custom Iron-On T-Shirt", &[("Custom", 10.0, 10.0)], Plus),
        ],
        ProjectType::Card => vec![
            tpl("Folded Card", &[("A2 (4.25 x 5.5 in)", 4.25, 5.5)], Card),
            tpl("Insert Card", &[("Standard", 4.5, 6.25)], Card),
        ],
        ProjectType::CardstockCutout => vec![
            tpl("Gift Tag", &[("Small", 1.25, 2.5), ("Large", 2.5, 4.0)], Tag), // Small seen
            tpl("Bookmark", &[("Standard", 2.0, 6.0)], Bookmark),
            tpl("Holder", &[("Standard", 3.0, 3.0)], Holder),
            tpl("Custom Cardstock Cut-out", &[("Custom", 6.0, 6.0)], Plus),
        ],
        ProjectType::Stickers => vec![
            tpl("Decorative Stickers", &[("Letter - 8.5 in x 11 in", 6.75, 9.25), ("A4 - 8.3 in x 11.7 in", 6.75, 9.25)], Sheet), // page sizes seen
            tpl("Stationery Stickers", &[("Letter - 8.5 in x 11 in", 6.75, 9.25), ("A4 - 8.3 in x 11.7 in", 6.75, 9.25)], Sheet),
            tpl("Organization Labels", &[("Letter - 8.5 in x 11 in", 6.75, 9.25), ("A4 - 8.3 in x 11.7 in", 6.75, 9.25)], Labels),
            tpl("Gift Labels", &[("Letter - 8.5 in x 11 in", 6.75, 9.25), ("A4 - 8.3 in x 11.7 in", 6.75, 9.25)], Labels),
            tpl("Custom Stickers", &[("Letter - 8.5 in x 11 in", 6.75, 9.25), ("A4 - 8.3 in x 11.7 in", 6.75, 9.25)], Plus),
        ],
        ProjectType::Engrave => vec![tpl("Engrave", &[("Design area", ENGRAVE_AREA.x, ENGRAVE_AREA.y)], Plate)],
        ProjectType::IronOnAccessories => vec![
            tpl("Baseball Hat", &[("Standard", 4.75, 2.75)], Hat), // seen
            tpl("Trucker Hat", &[("Standard", 4.5, 2.75)], Hat),
            tpl("Pillowcase", &[("Standard", 10.0, 10.0)], Pillow),
            tpl("Wine Bag", &[("Standard", 4.0, 6.0)], Bag),
            tpl("Cosmetic Bag", &[("Standard", 6.0, 4.0)], Bag),
            tpl("Tote Bag", &[("Standard", 10.0, 10.0)], Bag),
        ],
        ProjectType::Dtf => vec![
            tpl("DTF Transfer", &[("Standard", 10.0, 12.0)], Shirt),
            tpl("DTF Gang Sheet", &[("Sheet", 11.0, 17.0)], Sheet),
        ],
        ProjectType::Blank => vec![],
    }
}

/// The Engrave project's design area (the original: "a Design Area artboard (11.5 x 11.5)").
pub const ENGRAVE_AREA: Vec2 = vec2(11.5, 11.5);

/// The design area shown on the canvas for a project made from a template.
#[derive(Clone, Debug, PartialEq, serde::Serialize, serde::Deserialize)]
pub struct DesignArea {
    pub name: String,
    pub size: [f32; 2],
}

// ---------------------------------------------------------------- settings

#[derive(Clone, Copy, PartialEq, Eq, Debug, serde::Serialize, serde::Deserialize, Default)]
pub enum GridMode {
    #[default]
    Full,
    Partial,
    None,
}

#[derive(Clone, Copy, PartialEq, Eq, Debug, serde::Serialize, serde::Deserialize, Default)]
pub enum MachineModel {
    #[default]
    Maker,
    Maker3,
    Maker5,
}

impl MachineModel {
    pub const ALL: [MachineModel; 3] = [MachineModel::Maker, MachineModel::Maker3, MachineModel::Maker5];
    pub fn label(self) -> &'static str {
        match self {
            MachineModel::Maker => "Maker (original)",
            MachineModel::Maker3 => "Maker 3",
            MachineModel::Maker5 => "Maker 5",
        }
    }
    /// Whether engraving uses the separate Engraving Tool (estimated: Maker 5) rather than the
    /// QuickSwap Engrave Tip.
    pub fn engraving_tool(self) -> bool {
        self == MachineModel::Maker5
    }
    /// Whether it cuts Smart Materials without a mat.
    pub fn matless(self) -> bool {
        self != MachineModel::Maker
    }
}

/// How material goes into the machine (Settings > Load Type).
#[derive(Clone, Copy, PartialEq, Eq, Debug, serde::Serialize, serde::Deserialize, Default)]
pub enum LoadType {
    None,
    #[default]
    Mat12,
    Mat24,
    CardMat,
}

impl LoadType {
    pub const ALL: [LoadType; 4] = [LoadType::None, LoadType::Mat12, LoadType::Mat24, LoadType::CardMat];
    pub fn label(self) -> &'static str {
        match self {
            LoadType::None => "None (no mat, Smart Materials)",
            LoadType::Mat12 => "On Mat - 12 in x 12 in",
            LoadType::Mat24 => "On Mat - 12 in x 24 in",
            LoadType::CardMat => "On Card Mat",
        }
    }
    pub fn short(self) -> &'static str {
        match self {
            LoadType::None => "No mat",
            LoadType::Mat12 => "12 x 12 in",
            LoadType::Mat24 => "12 x 24 in",
            LoadType::CardMat => "Card mat",
        }
    }
    /// The area pieces can go on, in inches. Matless Smart Materials are 13 in wide (length
    /// estimated at 24 in); the card mat holds a 4.5 x 6.25 in card (estimated).
    pub fn size(self) -> Vec2 {
        match self {
            LoadType::None => vec2(13.0, 24.0),
            LoadType::Mat12 => vec2(12.0, 12.0),
            LoadType::Mat24 => vec2(12.0, 24.0),
            LoadType::CardMat => vec2(4.5, 6.25),
        }
    }
}

/// Paper for print then cut (Settings > Load Type > Print Then Cut Page Size).
#[derive(Clone, Copy, PartialEq, Eq, Debug, serde::Serialize, serde::Deserialize, Default)]
pub enum PageSize {
    A4,
    #[default]
    Letter,
    Legal,
    Tabloid,
    A3,
}

impl PageSize {
    pub const ALL: [PageSize; 5] = [PageSize::A4, PageSize::Letter, PageSize::Legal, PageSize::Tabloid, PageSize::A3];
    pub fn label(self) -> &'static str {
        match self {
            PageSize::A4 => "8.3 in x 11.7 in (A4)",
            PageSize::Letter => "8.5 in x 11 in (Letter)",
            PageSize::Legal => "8.5 in x 14 in (Legal)",
            PageSize::Tabloid => "11 in x 17 in (Tabloid)",
            PageSize::A3 => "11.7 in x 16.5 in (A3)",
        }
    }
    /// The printable area inside the sensor marks. Letter/A4 6.75 x 9.25 in is the original's
    /// figure; the larger pages are estimated the same way (page minus the same margins).
    pub fn print_area(self) -> Vec2 {
        match self {
            PageSize::A4 | PageSize::Letter => vec2(6.75, 9.25),
            PageSize::Legal => vec2(6.75, 12.25),
            PageSize::Tabloid => vec2(9.25, 15.25),
            PageSize::A3 => vec2(9.25, 14.75),
        }
    }
}

#[derive(Clone, Debug, serde::Serialize, serde::Deserialize)]
#[serde(default)]
pub struct Prefs {
    pub autosave: bool,
    pub grid: GridMode,
    pub metric: bool,
    pub show_operation: bool,
    pub snap: bool,
    pub model: MachineModel,
    pub autoconnect: bool,
    pub load: LoadType,
    pub page: PageSize,
    pub custom_materials: Vec<crate::materials::CustomMaterial>,
}

impl Default for Prefs {
    fn default() -> Self {
        Self {
            autosave: true,
            grid: GridMode::Full,
            metric: false,
            show_operation: true,
            snap: true,
            model: MachineModel::Maker,
            autoconnect: false,
            load: LoadType::Mat12,
            page: PageSize::Letter,
            custom_materials: Vec::new(),
        }
    }
}

/// A saved project file: the design and what kind of project it is.
#[derive(Clone, serde::Serialize, serde::Deserialize)]
pub struct ProjectFile {
    pub version: u32,
    pub name: String,
    #[serde(default)]
    pub project_type: ProjectType,
    #[serde(default)]
    pub design_area: Option<DesignArea>,
    pub items: Vec<super::Item>,
}
