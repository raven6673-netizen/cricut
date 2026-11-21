Design Space’s recovered host path uses `02` move-to and `00` line-to for a straight-sided cut. It also emits a **terminal `02` at the last point**. MatCraft already emits the travel and cutting records for a square, but omits that terminal move. The static evidence does not support adding `03`, `04`, `08`, or `09` to fix the blade bobbing.

## One closed square

For the direct builder, this is the **confirmed** `0x43` record list for a fresh send of `M A L B L C L D Z`, with builder mode other than `4` or `5`. Coordinates below are the resulting wire coordinates in inches. `0x46d260` constructs the vectors and `0x4ed0b0` / `0x4ed2d0` convert and serialize them. [Vector builder](decomp-machine/functions/0x46d260.c)

| Kind byte | X | Y | Host operation |
|---|---:|---:|---|
| `02` | 1 | 1 | Travel to A |
| `00` | 2 | 1 | First cut edge, A → B |
| `00` | 2 | 2 | Cut B → C |
| `00` | 1 | 2 | Cut C → D |
| `00` | 1 | 1 | Close D → A |
| `02` | 1 | 1 | Terminal move at A, with no XY displacement |

The kind sequence is **`02 00 00 00 00 02`**. In direct builder modes `4` and `5`, `0x46d260` also emits a `00` **at A immediately after the initial `02`**, giving **`02 00 00 00 00 00 02`**. That extra record has zero XY displacement. The separate job-level EndDraw command is `0x22`. The precise geometry preparation mode and any extra points selected for the user’s Maker job remain unverified.

## Static trace

| Stage | Finding | Confidence |
|---|---|---|
| `0x45a1c0` → `0x46d0c0` | Preparation dispatches by geometry mode. Mode `3` calls `0x46e510`, modes `2` and `5` call `0x4dbde0`, and the remaining case calls `0x46e100`. | **Confirmed** |
| `0x46e100` → `0x46e1a0` and `0x46e510` → `0x46e5c0` | Both parsers use `02` for `M`, `00` for straight segments, and `00` back to the start for `Z`/`z` if a closing edge is needed. `0x46e1a0` keeps cubic data as `01` intermediate entries. `0x46e5c0` writes `00` for cubic subdivision points and the endpoint. Neither inserts specialized entry, exit, or end records. | **Confirmed** |
| `0x46d260` | The output kind assignments are `00`, `01`, and `02`. The `iVar10 == 9` comparison tests an **input** kind and its branch emits `00` or `01`. The unconditional final append writes `02` at the tracked point. For a square closed by the named parsers, the closing `00` has already set that point to A. | **Confirmed** |
| `0x4dbde0` → `0x80c790` → `0x80c390` | Disassembly shows `0x80c790` calling `0x80c390` at `0x80d05d` to populate the point list. `0x80c390` preserves moves, emits zero-kind line entries, and appends a move using the last XY at `0x80c714–0x80c72b`. The outer `0x4dbde0` conversion explicitly maps intermediate kind `2` to `02` and every other kind to `00`. This confirms the wire-kind vocabulary and terminal move for this branch. Geometry preprocessing can affect the number and coordinates of the records. | **Confirmed** |
| `0x4df770` → `0x4ddc20` → `0x4ed0b0` → `0x4ed2d0` | The fresh-send path batches and forwards the records in order. `0x4ed0b0` copies into the command and divides XY by 72. `0x4ed2d0` writes the low byte of the kind field followed by two LE `f32` coordinates. On resume, `0x4df770` can rewrite the first pending record to `02` when it skips an already-completed prefix. It does that before the batch loop. There is no per-kind Z command in this send path. | **Confirmed** |

## MatCraft change and the Z symptom

For the straight-contour path in `contours_to_records`, keep the leading `VEC_MOVE` and the consecutive `VEC_LINE` edges. Emit the closing `VEC_LINE` only if the contour needs it. Then append `VEC_MOVE` at the final cutting endpoint, which is the first point for a closed contour and the last point for an open one. For the example square, update the existing test to expect six records and assert the final `(VEC_MOVE, start_x, start_y)`. This reproduces the direct builder’s ordinary sequence. [MatCraft record builder](</home/igpx12/Documents/Cricut Anything/app/src/machine.rs:1026>) · [Square test](</home/igpx12/Documents/Cricut Anything/app/src/machine.rs:1225>)

**Likely:** the first `02` requests travel, the contiguous `00` records request cutting at the configured pressure, and the terminal `02` requests a return to the travel state at the same XY. **Unknown:** the exact Z timing at those transitions, whether a zero-distance `02` immediately lifts on the target firmware, and why this machine bobs on each segment. A missing terminal move affects the end of the stream. It does not explain a lift between adjacent `00` records. The host itself emits adjacent `00` records without a preceding `03`, so the theory that `03` is required to hold pressure has no support in the recovered host path. Establishing the actual per-segment Z behavior requires evidence from the firmware’s `0x43` consumer.