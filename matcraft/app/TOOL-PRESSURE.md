# Cricut Maker / Athena blade engagement

**The cut-pressure setpoint is the final big-endian 16-bit word of SetMachineState `0x30`.** Design Space computes that word in **Z encoder counts** from the requested material pressure and the Maker’s calibration. The ordinary mat-cut setup sends `30 → 31 → 37 → 3E → 3F` **before** BeginDraw `21`. The vector stream then supplies travel and cutting geometry. This is the strongest static explanation for why MatCraft’s `21 → 43 → 22` traced the path without cutting. The firmware’s exact response to an unset pressure word remains unverified. `0x48e6e0`, `0x4ff680`, `0x4f8990`, `0x4f8de0`, `0x4a55e0`.

## Setup bytes and blade defaults

The layouts below are command bodies. A plain MatCraft frame prepends the two-byte big-endian body length. The blade example uses `DRAG_KNIFE_TT = 2`, the standard passive drag-blade case, and a 12 × 12-inch work area. The executable’s `PBToolType` descriptor identifies tool `2` at PE file offset `0x572011`. [MatCraft framing](</home/igpx12/Documents/Cricut Anything/app/src/machine.rs:122>)

| Command | Exact body layout | Field meanings and evidence |
|---|---|---|
| **SetMachineState `30`** | `30 FF 00 00 WW HH 00 00 00 00 TT CC CC` | **Confirmed.** `FF` is a placeholder for `(state << 7) \| flags \| 08`. The ordinary drag-blade setup produces `08` for the example. `WW HH` are the integer-converted width and height bytes, with the input size fields reordered at the call site. `TT` is the pressure-table selector, `00` for the drag-blade branch. `CC CC` is the encoder count, BE. The command object also stores the requested grams and the derived position in millimetres, while the Athena serializer emits the count. `0x45a600`, `0x48e6e0`, `0x4f8900`, `0x4f8990`, `0x4f8d70–0x4f8de0`. |
| **SetSpeedState `31`** | `31 0F 0F 0F 0F 0F` followed by **20 BE `u16` words** | **Confirmed.** Four groups in order: cut velocity, move velocity, cut acceleration, move acceleration. Within each group the axis order is X, Y, ZA, ZB, theta. Each word is the rounded result of multiplying the profile value by 100. The five `0F` bytes are fixed in this writer. The physical time and angular units for every axis remain unverified. `0x4fd240`, `0x4f31f0`, `0x5aa4b0`; `PBSpeedValues` and `PBSpeedGroup` descriptors at PE file offsets `0x575BC3` and `0x575FE2`. |
| **SetToolState `37`** | `37 flags lift:f32BE parallel:f32BE lagging:f32BE` | **Confirmed.** Bits 0–3 are `tan_theta`, `centered_bl`, `use_cal_offsets`, and `t_bias_enabled`. Built-in drag blade gives flags **`03`** and three zero floats. `0x4f4ac0`, `0x4fdaf0`, `0x4fdc30`, `0x48ef60`. |
| **SetFullToolState `3E`** | `3E 00 flags lift:f32BE parallel:f32BE lagging:f32BE n1 n2 n3 n4 n5` | **Confirmed.** Bits 0–3 match `37`, bit 4 is `p_bias_enabled`, and bit 6 is `roc_threshold_enabled`. The Boolean packing leaves bit 5 clear. The built-in drag profile gives **`03`**, three zero floats, and **`00 00 00 00 00`** for the nodes. The five protobuf node fields are `int32` and the writer takes the low byte of each in order. Their physical meanings and units remain unresolved. The drag profile’s zero nodes coexist with a separately computed pressure in `30`. `0x4f4ac0`, `0x4fdee0`, `0x4fe170`; `PBToolSetting` descriptor at file offset `0x576188`. |
| **CPR Toggle `3F`** | `3F 00` for drag blade | **Confirmed.** The setup sends `3F 01` only for tool mode `3`, the tangential blade, and `3F 00` otherwise. This is a setup toggle, with no pressure operand. `0x48e6e0`, `0x4fec50`, `0x4fed60`; `PBToolType` descriptor at file offset `0x572024`. |

`lift_threshold` is a tool-behaviour threshold. **Likely** it is an angular threshold for lift or reorientation on tools that use it. The executable contains `rotaryBladeLiftThresholdAngle`, and the built-in rolling-blade profile uses `0.191986`, consistent with 11° expressed in radians. I did not establish the firmware comparison or its units independently. `parallel_offset` and `lagging_offset` are geometric blade offsets, **likely in inches**. For example, an offset getter’s fallback `0.0197` is approximately 0.50 mm in inches. The writer sends these values directly as floats. For drag knife `2`, both offset getters skip their persistent-setting lookup and return zero through this setup path. `0x4f4ac0`, `0x48e520`, `0x48e2b0`, `0x48e420`, `0x48ef60`.

## Where the pressure comes from

**Confirmed.** `0x4ff680` selects the material’s pressure profile, including available more/less variants. It uses an explicit pressure override when selected, or a positive pressure from the profile. The general fallback in that branch is **180 grams of requested force**. There is also a calibrated-position branch that converts a supplied position in millimetres directly to counts. `0x4ff680`, `0x4dbb30`.

For the grams path, `0x4d6ba0` loads the Maker’s calibration if needed, finds the entries for the selected tool group, and interpolates grams to counts. The more/less adjustment can add or subtract the count equivalent of **0.5 mm**, depending on the selection and profile branch. The calibration comes from machine persistent memory via `A0 <index:u16 BE>`, whose reply supplies the stored data. This establishes a **static 180 g fallback and a machine-specific wire value**. The host trace starts with material/profile data already supplied to the native layer, so it does not establish whether a particular material record was downloaded, cached, or bundled. It also does not establish a universal usable count range. `0x4ff680`, `0x4d6ba0`, `0x4d6f40`, `0x4d71f0`, `0x4db9b0`, `0x452e30`, `0x452d80`.

## Send order and concrete plain frames

**Confirmed host order on the ordinary mat setup branch.** `0x4a55e0` can close the previous draw state with EndDraw, then calls Athena’s setup through vtable slot `+0x74` and calls the BeginDraw helper `0x472740` after setup succeeds. The per-path chain reaches `0x4c0e40 → 0x4c15b0 → 0x4dd840 → 0x4df770` for streaming. The finish helper is `0x472b50`. On a gated Maker branch the original also does persistent-memory writes **between `3E` and `3F`** through `0x48edc0`. Their necessity for a plain drag-blade session is unverified. `0x4a55e0`, `0x48e6e0`, Athena vtable `0x95d564`, `0x48edc0`.

The example below assumes a loaded, ready machine, the built-in drag-knife speed profile for selector `0`, and no per-material speed overrides. Its `30` uses **100 counts only to illustrate the byte encoding**. Substitute the calibrated count for the actual tool and material before using the sequence. Each command occupies its own frame. `0x4f31f0`, `0x46a280`, `0x4ff680`, `0x4fd240`.

```text
# 1. SetMachineState. Replace final 00 64 with calibrated u16 BE.
00 0D  30 08 00 00 0C 0C 00 00 00 00 00 00 64

# 2. SetSpeedState. One frame, shown across five lines.
00 2E  31 0F 0F 0F 0F 0F
       00 D2 00 D2 01 90 01 90 03 E8
       01 90 01 90 01 90 01 90 03 E8
       05 14 05 14 07 D0 07 D0 09 C4
       0F 14 0F 14 0F 14 0F 14 61 A8

# 3. SetToolState.
00 0E  37 03  00 00 00 00  00 00 00 00  00 00 00 00

# 4. SetFullToolState.
00 14  3E 00 03  00 00 00 00  00 00 00 00  00 00 00 00  00 00 00 00 00

# 5. CPR Toggle.
00 02  3F 00

# 6. BeginDraw.
00 04  21 00 00 00

# 7. Example MultiDrawVector. Travel to (1,1), then cut to (2,1), in inches.
00 13  43  02 00 00 80 3F 00 00 80 3F  00 00 00 00 40 00 00 80 3F

# 8. EndDraw.
00 04  22 00 00 00
```

Advance after each command’s successful reply. During streaming, use the buffer-space byte from the `43` ACK as established by the USB test. The bytes above reproduce the host’s setup order with plain framing. The minimum setup subset and plain acceptance of each setup command still require confirmation on the Maker. `0x48e6e0`, `0x4528a0`, `0x4ddc20`, `0x50ee00`.

## What raises and lowers the blade

- **Confirmed host format, likely firmware behaviour.** The vector kind supplies the travel/cut distinction. The parser produces `02` for move-to and `00` for line-to. The bulk writer sends exactly `kind:u8, X:f32LE, Y:f32LE`. That supports **(b)**: firmware likely lifts for travel and engages for cutting using the configured setpoint. The host’s setup commands supply **(d)** as configuration, with `30` carrying the setpoint and `37`/`3E` supplying tool geometry and behaviour. The recovered streaming loop has no per-vector pressure or Z field and issues no separate up/down command at the transition. `0x46d260`, `0x4ddc20`, `0x4ed0b0`, `0x4ed2d0`.
- **Confirmed Move 4D behaviour.** The recovered Athena writer emits `F1 17`, **three literal zero floats**, and a BE float32 theta converted from degrees to radians. The `17` is the fixed second body byte. In this implementation the Z slot is always `00 00 00 00`, so it cannot carry the cut setpoint. `0x48dcc0` sends it through a standalone motion path and then waits for the machine. There is no `F1` send inside the recovered `43` loop or a verified mandatory `F1` position in the ordinary drag-blade sequence. `0x4ff400`, `0x4ff460`, `0x4ff470`, `0x48dcc0`, `0x4ddc20`.
- **Confirmed Home Theta behaviour.** The body is `F0 <selector>`. Internal values 0 and 1 give `02` and `20`, respectively, while the remaining branch gives `08`. The constructor defaults to 1. The separate homing helper can reposition the Maker’s Y axis before sending `F0`. This is an auxiliary homing path, with no pressure operand and no per-stroke call in the bulk loop. Its exact placement in every tool-change or calibration workflow remains open. `0x4fb0e0`, `0x4fb140`, `0x4fb170`, `0x48dd50`.

## Still unknown

- The actual count for the user’s Maker and material, and whether that firmware accepts every setup command in plain framing.
- The minimum command subset that yields engagement, whether the gated persistent writes affect this cut, and the complete initialization requirements from power-on.
- The firmware’s lift timing at travel, line, and end-of-path transitions, plus the physical definitions of the five `3E` nodes, the remaining flag semantics, and the exact units of the nonzero tool thresholds and offsets.

This was a static trace of the supplied Ghidra exports and targeted PE disassembly. The 100-count value is illustrative, and the speed bytes represent one built-in profile. No hardware was accessed.