//! Build the Cricut cut job as protobuf — a `PBMatCutExecutionPlan` — from the schema recovered in
//! the `binary-check` analysis (`proto/`, JOB-FORMAT.md).
//!
//! CLEAN-ROOM NOTE: this encodes device-protocol structure derived from reading Cricut's binary;
//! included in MatCraft at the user's explicit direction.
//!
//! This encodes the documented high-level job:
//!   PBMatCutExecutionPlan → grouped_executions → PBMatPathData → PBPathSplitData → PBDrawVectors →
//!   PBDrawVector { cut_type, point(x,y), command_number }.
//! The plaintext captured at the CricutDevice I/O boundary (JOB-DECODED.md) used a lower-level node
//! layout (field #46 with machine-step coordinates); which layer the encrypted wire actually carries
//! is still to be confirmed. The geometry here is in inches, one node per path point.

use crate::geometry::Operation;

// PBDrawVectorType (NativeModel.Machines)
pub const DVT_LINE_TO: i64 = 0;
pub const DVT_MOVE_TO: i64 = 2;

// ---------------------------------------------------------------- minimal protobuf writer

/// A tiny protobuf wire-format writer (varint / fixed64 / length-delimited). Enough to serialize the
/// cut-job message tree without a protobuf dependency or build-time .proto compilation.
#[derive(Default)]
pub struct Pb {
    buf: Vec<u8>,
}

impl Pb {
    pub fn new() -> Self {
        Self::default()
    }
    pub fn into_bytes(self) -> Vec<u8> {
        self.buf
    }
    pub fn as_bytes(&self) -> &[u8] {
        &self.buf
    }

    fn raw_varint(&mut self, mut v: u64) {
        loop {
            let b = (v & 0x7f) as u8;
            v >>= 7;
            if v != 0 {
                self.buf.push(b | 0x80);
            } else {
                self.buf.push(b);
                break;
            }
        }
    }
    fn key(&mut self, field: u32, wire: u8) {
        self.raw_varint(((field as u64) << 3) | wire as u64);
    }

    /// varint field (uint32/uint64/enum/bool).
    pub fn uint(&mut self, field: u32, v: u64) {
        self.key(field, 0);
        self.raw_varint(v);
    }
    /// int32/int64 field (two's-complement varint, as protobuf encodes signed ints without zigzag).
    pub fn int(&mut self, field: u32, v: i64) {
        self.key(field, 0);
        self.raw_varint(v as u64);
    }
    pub fn boolean(&mut self, field: u32, v: bool) {
        self.uint(field, v as u64);
    }
    /// double field (wire type 1, 8-byte little-endian IEEE-754).
    pub fn double(&mut self, field: u32, v: f64) {
        self.key(field, 1);
        self.buf.extend_from_slice(&v.to_bits().to_le_bytes());
    }
    pub fn string(&mut self, field: u32, s: &str) {
        self.bytes(field, s.as_bytes());
    }
    pub fn bytes(&mut self, field: u32, b: &[u8]) {
        self.key(field, 2);
        self.raw_varint(b.len() as u64);
        self.buf.extend_from_slice(b);
    }
    /// Embed a sub-message as a length-delimited field.
    pub fn message(&mut self, field: u32, m: &Pb) {
        self.bytes(field, &m.buf);
    }
}

// ---------------------------------------------------------------- job message tree

/// PBToolType + PBArtType for a MatCraft operation (values from the recovered enums).
pub fn tool_and_art(op: Operation) -> (i64, i64) {
    match op {
        Operation::Cut => (2, 1),       // DRAG_KNIFE_TT, CUT_ART_TYPE
        Operation::Draw => (4, 2),      // PEN_DT_TT, DRAW_ART_TYPE
        Operation::Score => (6, 4),     // SCORING_WHEEL_TT, SCORE_ART_TYPE
        Operation::Engrave => (12, 7),  // ENGRAVE_TOOL_TT, ENGRAVE_ART_TYPE
        Operation::Deboss => (10, 5),   // DEBOSSING_TOOL_TT, DEBOSS_ART_TYPE
        Operation::Wave => (11, 6),     // WAVE_TOOL_TT, WAVE_ART_TYPE
        Operation::Perforate => (9, 8), // PERFORATION_WHEEL_TT, PERFORATE_ART_TYPE
        Operation::Print => (2, 3),     // (print-then-cut: cut the outline) DRAG_KNIFE_TT, PRINT_ART_TYPE
        Operation::Guide => (0, 0),     // not sent
    }
}

/// PBPoint { 1 double x, 2 double y }.
fn point(x: f64, y: f64) -> Pb {
    let mut p = Pb::new();
    p.double(1, x);
    p.double(2, y);
    p
}

/// PBDrawVector { 1 cut_type, 2 point, 5 command_number }.
fn draw_vector(cut_type: i64, x: f64, y: f64, command_number: i64) -> Pb {
    let mut v = Pb::new();
    v.int(1, cut_type);
    v.message(2, &point(x, y));
    v.int(5, command_number);
    v
}

/// Build a serialized `PBMatCutExecutionPlan` for one group of contours (points in inches). Each
/// contour becomes a MOVE_TO to its first point then LINE_TO to the rest; `custom_pressure` (machine
/// counts) is set on the path when given.
pub fn build_cut_plan(
    id: &str,
    op: Operation,
    contours: &[Vec<(f64, f64)>],
    custom_pressure: Option<i64>,
) -> Vec<u8> {
    let (tool_type, art_type) = tool_and_art(op);

    // PBDrawVectors { 1 repeated PBDrawVector }
    let mut vectors = Pb::new();
    let mut cmd = 0i64;
    for c in contours {
        if c.is_empty() {
            continue;
        }
        vectors.message(1, &draw_vector(DVT_MOVE_TO, c[0].0, c[0].1, cmd));
        cmd += 1;
        for p in &c[1..] {
            vectors.message(1, &draw_vector(DVT_LINE_TO, p.0, p.1, cmd));
            cmd += 1;
        }
    }

    // PBPathSplitData { 1 id, 5 vectors }
    let mut split = Pb::new();
    split.uint(1, 1);
    split.message(5, &vectors);

    // PBMatPathData { 5 path_id, 14 actual_path_type, 36 path_split_data, 45/46 custom pressure }
    let mut path = Pb::new();
    path.string(5, id);
    path.int(14, art_type);
    path.message(36, &split);
    if let Some(pr) = custom_pressure {
        path.boolean(45, true);
        path.int(46, pr);
    }

    // PBMatCutGroupExecution { 1 id, 2 tool_type, 3 line_type, 13 paths }
    let mut group = Pb::new();
    group.int(1, 1);
    group.int(2, tool_type);
    group.int(3, art_type);
    group.message(13, &path);

    // PBMatCutExecutionPlan { 1 id, 6 grouped_executions }
    let mut plan = Pb::new();
    plan.string(1, id);
    plan.message(6, &group);
    plan.into_bytes()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn varint_and_double_encode_to_protobuf_bytes() {
        let mut p = Pb::new();
        p.uint(1, 150); // field 1, varint: 0x08, 150 = 0x96 0x01
        assert_eq!(p.as_bytes(), &[0x08, 0x96, 0x01]);

        let mut d = Pb::new();
        d.double(1, 1.0); // field 1, fixed64: 0x09 then 1.0 LE
        assert_eq!(d.as_bytes(), &[0x09, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0xf0, 0x3f]);
    }

    #[test]
    fn point_has_two_doubles() {
        let p = point(1.0, 2.0).into_bytes();
        // field 1 double (0x09 + 8), field 2 double (0x11 + 8) = 18 bytes
        assert_eq!(p.len(), 18);
        assert_eq!(p[0], 0x09);
        assert_eq!(p[9], 0x11);
    }

    #[test]
    fn plan_builds_and_is_well_formed() {
        let square = vec![vec![(1.0, 1.0), (2.0, 1.0), (2.0, 2.0), (1.0, 2.0)]];
        let plan = build_cut_plan("job-1", Operation::Cut, &square, Some(1039));
        assert!(!plan.is_empty());
        // top-level starts with field 1 (id) as a length-delimited string: tag 0x0a
        assert_eq!(plan[0], 0x0a);
        // the id string "job-1" follows its length byte
        assert_eq!(&plan[2..7], b"job-1");
    }

    #[test]
    fn draw_vector_carries_cut_type_and_command() {
        let v = draw_vector(DVT_MOVE_TO, 1.0, 1.0, 7).into_bytes();
        // field 1 (cut_type) varint = 0x08 0x02 (MOVE_TO)
        assert_eq!(&v[..2], &[0x08, 0x02]);
    }
}
