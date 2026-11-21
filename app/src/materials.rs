//! Materials and tools. Tool names follow the tool list in the original's machine code and app
//! text (MECHANICS.md); the pressure and pass numbers are estimates, since the original downloads
//! its material settings from its online service.

#[derive(Clone, Copy, PartialEq, Eq, Debug, serde::Serialize, serde::Deserialize)]
pub enum Tool {
    FineBlade,
    DeepBlade,
    RotaryBlade,
    KnifeBlade,
    ScoringWheel,
    DoubleScoringWheel,
    ScoringStylus,
    Pen,
    Perforation,
    Debossing,
    /// The QuickSwap "Engrave Tip" (MAT_CUT_HEAD_NAME_ENGRAVE_TIP).
    Engraving,
    /// The separate "Engraving Tool" (MAT_CUT_HEAD_NAME_ENGRAVE_STATIC_TOOL).
    EngravingTool,
    Wavy,
}

impl Tool {
    pub const ALL: [Tool; 13] = [
        Tool::FineBlade,
        Tool::DeepBlade,
        Tool::RotaryBlade,
        Tool::KnifeBlade,
        Tool::ScoringWheel,
        Tool::DoubleScoringWheel,
        Tool::ScoringStylus,
        Tool::Pen,
        Tool::Perforation,
        Tool::Debossing,
        Tool::Engraving,
        Tool::EngravingTool,
        Tool::Wavy,
    ];
    pub fn label(self) -> &'static str {
        match self {
            Tool::FineBlade => "Fine-point blade",
            Tool::DeepBlade => "Deep-point blade",
            Tool::RotaryBlade => "Rotary blade",
            Tool::KnifeBlade => "Knife blade",
            Tool::ScoringWheel => "Scoring wheel",
            Tool::DoubleScoringWheel => "Double scoring wheel",
            Tool::ScoringStylus => "Scoring stylus",
            Tool::Pen => "Pen / marker",
            Tool::Perforation => "Perforation blade",
            Tool::Debossing => "Debossing tip",
            Tool::Engraving => "Engrave Tip",
            Tool::EngravingTool => "Engraving Tool",
            Tool::Wavy => "Wavy blade",
        }
    }
}

pub struct Material {
    pub name: &'static str,
    pub tool: Tool,
    /// Relative pressure, 1 (lightest) to 7 (heaviest).
    pub pressure: u8,
    pub passes: u8,
}

pub const MATERIALS: &[Material] = &[
    Material { name: "Printer paper", tool: Tool::FineBlade, pressure: 2, passes: 1 },
    Material { name: "Cardstock (medium)", tool: Tool::FineBlade, pressure: 4, passes: 1 },
    Material { name: "Cardstock (heavy)", tool: Tool::FineBlade, pressure: 5, passes: 2 },
    Material { name: "Adhesive vinyl", tool: Tool::FineBlade, pressure: 3, passes: 1 },
    Material { name: "Iron-on (heat transfer)", tool: Tool::FineBlade, pressure: 3, passes: 1 },
    Material { name: "Sticker paper", tool: Tool::FineBlade, pressure: 3, passes: 1 },
    Material { name: "Felt", tool: Tool::RotaryBlade, pressure: 5, passes: 1 },
    Material { name: "Cotton fabric", tool: Tool::RotaryBlade, pressure: 4, passes: 1 },
    Material { name: "Faux leather", tool: Tool::RotaryBlade, pressure: 5, passes: 1 },
    Material { name: "Chipboard (2 mm)", tool: Tool::KnifeBlade, pressure: 7, passes: 8 },
    Material { name: "Balsa wood (1.5 mm)", tool: Tool::KnifeBlade, pressure: 7, passes: 4 },
    Material { name: "Craft foam", tool: Tool::DeepBlade, pressure: 5, passes: 2 },
    Material { name: "Magnet sheet", tool: Tool::DeepBlade, pressure: 6, passes: 2 },
    // engraving: the original's engrave text says "in metal, acrylic & more"
    Material { name: "Aluminum sheet (engrave)", tool: Tool::Engraving, pressure: 7, passes: 1 },
    Material { name: "Anodized aluminum (engrave)", tool: Tool::Engraving, pressure: 6, passes: 1 },
    Material { name: "Acrylic sheet (engrave)", tool: Tool::Engraving, pressure: 6, passes: 2 },
    Material { name: "Leather (engrave)", tool: Tool::Engraving, pressure: 5, passes: 1 },
];

/// A material the person added (Settings > Machines > Custom materials).
#[derive(Clone, PartialEq, Debug, serde::Serialize, serde::Deserialize)]
pub struct CustomMaterial {
    pub name: String,
    pub tool: Tool,
    pub pressure: u8,
    pub passes: u8,
}

/// One row of the material list: built-in or custom.
#[derive(Clone, PartialEq, Debug)]
pub struct MaterialRow {
    pub name: String,
    pub tool: Tool,
    pub pressure: u8,
    pub passes: u8,
    pub custom: bool,
}

/// Built-in materials followed by the person's own.
pub fn all_materials(custom: &[CustomMaterial]) -> Vec<MaterialRow> {
    MATERIALS
        .iter()
        .map(|m| MaterialRow { name: m.name.to_string(), tool: m.tool, pressure: m.pressure, passes: m.passes, custom: false })
        .chain(custom.iter().map(|c| MaterialRow { name: c.name.clone(), tool: c.tool, pressure: c.pressure, passes: c.passes, custom: true }))
        .collect()
}

/// The tool an operation needs. Engraving uses the Engraving Tool on machines that take it
/// (`engraving_tool`), otherwise the QuickSwap Engrave Tip.
pub fn tool_for_op(op: crate::geometry::Operation, cut_tool: Tool, engraving_tool: bool) -> Tool {
    use crate::geometry::Operation::*;
    match op {
        Cut | Print => cut_tool,
        Draw | Guide => Tool::Pen,
        Score => Tool::ScoringWheel,
        Engrave => {
            if engraving_tool {
                Tool::EngravingTool
            } else {
                Tool::Engraving
            }
        }
        Deboss => Tool::Debossing,
        Wave => Tool::Wavy,
        Perforate => Tool::Perforation,
    }
}
