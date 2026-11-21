The Maker’s tool-selection path can run **`B9 → optional XY travel → F0 → optional EA`** before cut setup. The XY travel uses the draw-vector protocol. For an initial ordinary drag-knife selection, the home frame is **`00 02 F0 08`**. MatCraft’s `31` words exactly match the built-in Maker drag-knife profile for selector `0`.

`CricutDevice.exe` establishes when the host sends those commands and what bytes it sends. The physical cause of the per-segment bobbing remains unresolved because the executable does not contain the Maker firmware’s motion planner or Z-control loop.

## A. Tool preparation before cut setup

The bytes below are **plain inner frames**, including the two-byte big-endian body length. The initial-selection path assumes an original Maker/Athena, requested tool type `2` (`DRAG_KNIFE_TT`), and an unset preparation flag. A different QuickSwap tool enum can select a different branch.

| Order | Frame and condition | Static evidence |
|---|---|---|
| 1 | **`00 01 B9`**. The home slot queries position on model `0x0B`. The minimum position reply is **`00 09 B9 <X:f32 BE> <Y:f32 BE>`**. The parser reads X followed by Y and accepts a body of at least nine bytes. | **High.** `0x48dd50`, `0x4e01e0`, `0x4e0240`, `0x4e0360`, `0x4e0250`, `0x4e00d0`. |
| 2 | **Only if the position transaction succeeds and `Y > 8.0` inches**, travel to the returned X and `Y = 8.0`. The host opens a draw with **`00 04 21 00 00 00`**, sends the move below, closes with **`00 04 22 00 00 00`**, and polls machine status for completion. The status request is **`00 04 60 00 00 00`**. A failed reposition returns from the home slot before `F0`. A failed `B9` or `Y <= 8.0` skips the move and proceeds to `F0`. | **High.** Comparison and target at `0x48dda2–0x48ddd5`. Begin, vector, EndDraw, and completion handling at `0x471ce0`, with the vector callback explicitly set to `0x4df770` at `0x47203a`. The bulk sender can also poll status. |
| 3 | **Home** with **`00 02 F0 08`** on initial selection. A subsequent ordinary home after the preparation flag has been set uses **`00 02 F0 20`**. | **High.** `0x4a7280` passes internal selector `2` when host flag `+0xEA` is clear and `1` when it is set. `0x48de21` stores the passed selector over the constructor default. `0x4fb1c7–0x4fb1ea` maps internal `0 → 02`, `1 → 20`, `2 → 08`. |
| 4 | **`00 01 EA`** if the helper’s mode at `+0x0C` is anything other than `2`. The full parsed reply layout is **`00 06 EA <tool-code> <s0> <s1> <s2> <s3>`**. The first byte after `EA` identifies the tool or selects fingerprint matching. The next four bytes are fingerprint samples. | **High** for the request, parsed fields, and gate. **Unknown** for the physical source of the samples. Athena vtable `0x95d564`, slot `+0x80 → 0x48dc20`, constructor `0x4fb780`, writer `0x4fcb60`, parser `0x4fc610`. |

The **Y reposition payload** is one kind-`02` travel record. `X′` below is the returned X after the host’s conversion to 72 units per inch, truncation to three decimal places in those units, and conversion back to inches for the wire. `Y = 8.0` encodes as `00 00 00 41` in float32 little-endian.

```text
00 0A 43 02          <X′:f32 LE> 00 00 00 41    bulk-vector mode
00 0D 42 02 00 00 00 <X′:f32 LE> 00 00 00 41    single-vector mode
```

**High confidence.** `0x471d45–0x471da7` shows the coordinate conversion, `0x471ce0 → 0x46d030` builds the move, and `0x4df770` chooses `43` or `42` using device flag `+0x134`. `0x4ed0b0/0x4ed2d0` and `0x4eda60/0x4ed790` serialize the two forms. The separate Move 4D writer at `0x4ff470` emits **`00 12 F1 17`**, twelve literal zero bytes for XYZ, then theta in radians as BE float32. A real-Y `F1` payload has no basis in this host branch.

**When the sequence runs.** On the supported Maker branch, `0x4a7280` returns early if the cached tool already matches and `+0xEA` is set. Otherwise it clears the cached tool through slot `+0x78` (`0x477b00`, a host-memory store), calls the home slot, sets the prep flag, and conditionally calls Detect. Forced-selection modes `1` and `2` can then write the requested tool back to the host cache, and mode `2` skips `EA`. A tool mismatch can trigger a UI retry with another home and detect. That loop belongs to tool selection, before vector streaming. `0x4c5b40` case `0x0E` reaches the helper, and `0x4c81b0` shows a caller that explicitly sets mode `2`. **High confidence.**

**What `EA` establishes.** `0x4fc610` maps codes `01` through `08` to host tool types `1, 2, 3, 6, 9, 10, 12, 11`, respectively. Code `FF` takes a fingerprint lookup path with a tolerance of ±3 per compared value. The parser zero-fills absent fields, so six bytes describe the full body layout, not a strict minimum enforced by that parser. `0x48dc20` caches the decoded type at device `+0x150`; on transaction failure it writes fallback type `2`. The host trace establishes classification and caching. Whether the firmware physically seats the tool or sets a Z reference during `EA` is **unknown**.

**Z zero, material height, and cut entry.** The executable does contain a shared `ZorroHomeZAxisCommand`, factory case `0x2C` at `0x48c6d0`, with a serializer that emits **`00 02 F0 04`** (`0x4f0bf0/0x4f0c60`). I found no call to it, a Z probe, or a working-Z datum setter in the traced ordinary Athena drag-cut path. `0x4bda90 → 0x4c0e40 → 0x4c1b70` runs per-path preparation, including conditional alignment or repositioning. On the setup branch, `0x4c0e40` then calls `0x4c3c00 → 0x4a55e0` for the pre-setup EndDraw and tool configuration, followed by `0x4c15b0` for execution. The pressure setpoint comes from calibration through `0x4ff680` and is serialized as the last word of `30` at `0x4f8de0`. **High confidence** for this host path and the shared home opcode. A firmware-internal Z reference remains **unknown**.

## B. Per-segment Z bobbing

**The speed words are correct for built-in Maker drag-knife selector `0`.** `0x48d5c0 → 0x4f49c0 → 0x4f31f0` installs the Maker groups. The first group is tagged tool `2`, selector `0`. `0x46a280` looks up tool plus selector, then falls back to a tool-only match and finally the first group. `0x4ff680` can apply material-dependent speed overrides. `0x4fd240` copies the selected group through `0x5aa4b0` and emits the twenty values scaled by 100, rounded, and serialized as BE `u16`s. **High confidence.**

| `31` group | X | Y | ZA | ZB | Theta |
|---|---:|---:|---:|---:|---:|
| Cut velocity | `00D2` | `00D2` | `0190` | `0190` | `03E8` |
| Move velocity | `0190` | `0190` | `0190` | `0190` | `03E8` |
| Cut acceleration | `0514` | `0514` | `07D0` | `07D0` | `09C4` |
| Move acceleration | `0F14` | `0F14` | `0F14` | `0F14` | `61A8` |

- **ZA and ZB are the two Z-axis channels.** The `PBSpeedGroup` descriptor identifies `za_axis` and `zb_axis`, and the executable has separate ZA/ZB homing velocity and acceleration settings. The `31` writer supplies rates, with no lift position or per-vector trigger. ZA and ZB both have a cut and move velocity of `4.00` in profile units, with distinct cut and move accelerations. **High confidence** for the fields and values. The firmware’s use of those rates in individual Z motions is **unknown**. `0x4f31f0`, `0x4fd240`, PE descriptor at file offset `0x575FE2`.
- **The host sends consecutive cut segments with no intervening lift or detect command.** `0x46d260` generates `02 00 … 00 02` for an ordinary straight contour. `0x4df770 → 0x4ddc20 → 0x4ed2d0` sends those records as kind plus XY. `0x4a7280` contains the host detection retry loop, outside the vector loop. A terminal `02` can account for a transition at the end of a contour. MatCraft already supplies that record. **High confidence** for the host sequence. A firmware retry caused by skipped preparation has **no supporting evidence in this trace**.
- **`37` and `3E` do carry a `lift_threshold` field.** The built-in passive drag profile sends flags `03` and a threshold of `0.0`. The built-in rolling-blade setting is approximately `0.191986`, consistent with 11° in radians and the executable’s `rotaryBladeLiftThresholdAngle` name. The host does not establish how the firmware interprets a zero threshold, whether it lifts a passive knife at corners, or whether vector length or speed can cause a lift at every point. **High confidence** for the host fields and defaults. **Inferred** angular meaning for the rolling-blade value. Firmware behavior remains **unknown**. `0x4f4ac0`, `0x48ef60`, `0x4fdc30`, `0x4fe170`.

For MatCraft, the source-backed addition is the **conditional** prep sequence before the existing pre-setup `22`, with the Y move emitted as a separate travel operation when needed. The evidence gives no basis for changing the selector-`0` speed words or inserting `F0 04` into the cut path. Acknowledgment of the stream and echo of the pressure count establish the protocol response. Actual Z travel, applied force, and the cause of the bobbing remain unverified. No hardware was accessed.