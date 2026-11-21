## What MatCraft is missing

**Confirmed host-side differences include a pre-setup `EndDraw`, the job’s work-area dimensions, and four gated `A1` writes. None is established as the command that engages the blade.** On the recovered passive drag-knife path, Design Space sends the calibrated setpoint in `30`, finishes tool setup, sends `21`, and streams cutting records. The host sends no separate Z-down or force-commit command between setup and the first ordinary `00` line record. The `30` echo establishes that the machine accepted the field. It does not expose the firmware’s active Z state or the force at the blade. `0x4a55e0`, `0x48e6e0`, `0x472740`, `0x4df770`, `0x4ddc20`.

For a 12 × 12 job using the supplied **1039-count** example, the passive drag-knife `30` frame is `00 0D 30 08 00 00 0C 0C 00 00 00 00 00 04 0F`. For a 12 × 24 work-area input, the dimension bytes are `0C 18`. The 2 × 2 design bounds are a parity mismatch when the original job supplies a 12 × 12 or 12 × 24 work area. `0x4c3c00`, `0x4a55e0`, `0x48e6e0`, `0x4f8d90`, `0x4f8de0`.

## The four persistent writes

**Confirmed.** Athena setup calls `0x48edc0` immediately after successful `3E`, before `3F`, when all three conditions hold:

```text
device [+0x98] == 0x0B
device [+0xE4] != 4
byte at device [+0x08] == 0
```

The Athena constructor sets `+0x98` to `0x0B`. The gate does not test the selected drag-knife type or the requested pressure. Factory selector `0x27` constructs `ZorroSetPersistentMemoryCommand`. Its body is `A1 <index:u16 BE> <value:u32 BE>`. Each write must succeed before the helper attempts the next. `0x48d5c0`, `0x48e6e0`, `0x48edc0`, `0x48c6d0`, `0x453100`, `0x4531a0`, `0x4531b0`, `0x4531d0`, `0x452a60`.

| Order | Index | Decimal value | Exact inner frame |
|---|---:|---:|---|
| 1 | `005D` | 10,000 | `00 07 A1 00 5D 00 00 27 10` |
| 2 | `005F` | 75,000 | `00 07 A1 00 5F 00 01 24 F8` |
| 3 | `0082` | 10,000 | `00 07 A1 00 82 00 00 27 10` |
| 4 | `0084` | 75,000 | `00 07 A1 00 84 00 01 24 F8` |

**Confirmed.** The helper sends four fixed values. It passes neither the pressure count nor the tool state into those values. The binary has a distinct `ZorroCommitPersistentValueCommand` with inner frame `00 04 A3 FF FF FF`, and this setup chain does not invoke it. The `A1` writes are a confirmed omission from MatCraft’s setup on runs where the gate is true. **Unknown** are the four firmware slot meanings, their effect on Z, and whether the target firmware requires them for force. `0x48edc0`, `0x4534f0`, `0x453580`, `0x453590`. [MatCraft setup list](</home/igpx12/Documents/Cricut Anything/app/src/machine.rs:1250>)

## Homing, tool detection, and Z

| Finding | Confidence and evidence |
|---|---|
| **Home Theta is reachable during tool preparation.** Athena vtable `0x95d564`, slot `+0x14`, resolves to `0x48dd50`. The tool-selection helper calls that slot on gated branches. The Maker helper can query position with `B9` and move Y to 8.0 if the returned Y exceeds 8.0 before homing. Internal selector `0` sends `00 02 F0 02`, `1` sends `00 02 F0 20`, and the other branch sends `00 02 F0 08`. | **Confirmed** conditional path and selector bytes. **Unknown** whether that branch runs on the specific simple-cut job and whether it affects usable Z travel. `0x4a7280`, `0x48dd50`, `0x4e01e0`, `0x471ce0`, `0x4fb0e0`, `0x4fb140`, `0x4fb170`. |
| **Tool Detect is `EA`.** The same tool-selection helper can dispatch through Athena slot `+0x80` after homing. It sends `00 01 EA` and updates the cached detected-tool field at device `+0x150`. One branch skips the detect call, and another can return early if the cached tool already matches. | **Confirmed** request and conditional ordering. **Unknown** whether the device performs physical Z seating or zeroing during detection. `0x4a7280`, `0x48dc20`, `0x4fb780`, `0x4fc26e`, `0x4fcb60`, `0x4fc610`. |
| **Move 4D cannot convey the cut setpoint through this writer.** The Athena inner frame is `00 12 F1 17`, then 12 literal zero bytes for X, Y, and Z, then one BE float32 theta in radians. | **Confirmed.** `0x48dcc0`, `0x4ff460`, `0x4ff470`. |
| **Scan-for-edge has a separate request format.** Athena serializes `00 0A E2 <f32 BE> <f32 BE> <u8>`. The recovered ordinary setup-to-draw chain does not call this builder. | **Confirmed** format and bounded call-path finding. Its use in other alignment paths does not establish a force prerequisite. `0x48df60`, `0x4fa980`, `0x4faf20`, `0x4a55e0`. |
| **The pressure word is the host’s cut setpoint.** The material/tool calibration result reaches the last two bytes of `30`. `43` carries only kind and XY. The direct straight-path builder emits leading `02`, consecutive `00` cutting records, and a terminal `02` at the final XY. | **Confirmed** host encoding and sequence. **Likely** the firmware uses the move/line transition to change tool state and applies the configured setpoint on line records. The terminal `02` accounts for an end-of-path transition. It cannot account for repeated bobbing between adjacent `00` records. `0x4ff680`, `0x4d6ba0`, `0x4f8d70`, `0x4f8de0`, `0x46d260`, `0x4ed2d0`. [Vector trace](</tmp/claude-1000/-home-igpx12--claude-spares-spare-mE4oLv/a2849e9e-3be2-48e5-b9c8-fc2042e257ba/scratchpad/codex-out/vector-kinds.md>) |

## Work area and flags

- **Confirmed.** `0x4c3c00` gets a job-level `PBSize`. `0x4a55e0` uses the supplied size, falls back to that job-level size if either dimension is zero, and uses **12 × 12** if the result is still incomplete. `0x48e6e0` passes the fields at `PBSize +0x10` and `+0x08`, in that order, to the `30` setter. The serializer emits them at body offsets 4 and 5, which the formatter labels width and height. This establishes the job’s work area as the source. A physical mat label alone does not determine the bytes if the job supplies a different usable area. `0x4c3c00`, `0x4a55e0`, `0x48e6e0`, `0x4f8d90`, `0x4f8de0`, `0x4f8990`.
- **Confirmed.** Athena emits byte 1 as `(state << 7) | flags | 08`. The ordinary passive drag-knife profile uses `08` and pressure-table byte `00`. Setup can set the state bit when device `+0x12C` is `1` or `4`, and `0x45a600` derives profile fields from the supplied tool configuration. There is no unconditional `08` → `88` switch when the host starts streaming. The `37 03`, `3E 00 03` with zeros, and `3F 00` examples apply to the recovered passive drag profile. The installed housing description alone does not establish that Design Space selected that profile. `0x45a600`, `0x48e6e0`, `0x4f8de0`, `0x4f4ac0`, `0x4fe170`.

## Ordered transcript recoverable from the static trace

These are outgoing **inner frames**. The two-byte BE length counts the command body. With an active AES session, the original wraps eligible inner frames in ciphertext. Consequently the fixed inner bytes below are also the plain-frame representation, while the original ciphertext and `C7` random bytes vary by run. `0x48f050`, `0x4528b0`, `0x47a0e0`, `0x48de60`.

| Phase | Recovered transactions in local order |
|---|---|
| **Common initialization** | The ordinary branch attempts Reset AES, `CF <3 generated bytes>`, then firmware query opcode `12`, version-gated command count `00 01 77`, the `03` query in `0x472380`, and machine status `00 04 60 00 00 00`. It then reads serial slots `00 03 A0 00 51` through `00 03 A0 00 54` in ascending order, runs a gated position query `B9`, reads the scale slots through `0x471710` (`00 03 A0 00 A3` and, on the two-scale path, `00 03 A0 00 9B`), then reads `00 03 A0 00 11`. **Confirmed** helper order for the recovered branch. Extra capability checks and session-establishment placement remain incompletely resolved. `0x476760`, `0x4e2460`, `0x4e2800`, `0x471500`, `0x472380`, `0x497360`, `0x472460`, `0x471710`. |
| **Athena initialization** | Where common init returns success and byte `+0xAF` is zero, `00 01 0A` Clear Error → `00 01 A4` Get Persistent Value Count → status refresh through `0x473860`. `A4` supplies the cached count. **Confirmed.** `0x48e5b0`, `0x4fe410`, `0x4fe540`, `0x4feee0`, `0x4fefc0`, `0x473860`. |
| **Mat load and Go** | Repeated status `00 04 60 00 00 00` and interface-state `00 01 73` queries, with state-dependent indicator/button-latch transactions. The number of polls depends on replies and user input. **Confirmed** query bytes and polling, incomplete indicator transcript. `0x4be8f0`, `0x4a5a50`, `0x4a64c0`, `0x4a1240`, `0x4a0ed0`. |
| **Conditional prep** | Tool-selection branches can issue position query and Y reposition, `F0`, then `EA` where detection is selected. The per-path workflow can also perform positioning or alignment before setup. **Confirmed** branches, **unknown** exact transactions selected in the target run. `0x4a7280`, `0x48dd50`, `0x4c0e40`, `0x4c1b70`. |
| **Draw reset** | On the recovered `0x4a55e0` setup branch, readiness handling through `0x4742e0` precedes `00 04 22 00 00 00`. The EndDraw helper can issue status polls before setup continues. **Confirmed.** `0x4a55e0`, `0x4742e0`, `0x472b50`, `0x472e60`. |
| **Pressure and tool setup** | If the calibration cache needs loading, the Maker reads `00 03 A0 <index BE>` for `0001`–`0010`, `0021`–`0030`, and `0041`–`0050`, each range ascending. Then the main setup sends `30 → 31 → 37 → 3E → [four A1 frames if gated] → 3F`. **Confirmed** on the successful branch. `0x4d6ba0`, `0x4d6f40`, `0x4d71f0`, `0x48e6e0`, `0x48edc0`. |
| **Draw** | `00 04 21 00 00 00`, followed by `43` batches or `42` single records according to the initialized device mode. The first ordinary cutting edge is the first `00` line after travel. **Confirmed.** `0x472740`, `0x4c0e40`, `0x4c15b0`, `0x4dd840`, `0x4df770`, `0x4ddc20`. |

The fixed draw example below assumes a 12 × 12 work area, the passive drag profile, the supplied 1039 counts, the built-in speed profile without job overrides, and a square that reaches the serializer at `(1,1)` through `(2,2)` inches. Each named command starts a new frame. The first cut edge is the second record in `43`. `0x4f8de0`, `0x4fd240`, `0x4fdc30`, `0x4fe170`, `0x4fed60`, `0x4eabe0`, `0x4ed2d0`.

```text
Pre-setup EndDraw
00 04 22 00 00 00

SetMachineState
00 0D 30 08 00 00 0C 0C 00 00 00 00 00 04 0F

SetSpeedState
00 2E 31 0F 0F 0F 0F 0F
      00 D2 00 D2 01 90 01 90 03 E8
      01 90 01 90 01 90 01 90 03 E8
      05 14 05 14 07 D0 07 D0 09 C4
      0F 14 0F 14 0F 14 0F 14 61 A8

SetToolState
00 0E 37 03  00 00 00 00  00 00 00 00  00 00 00 00

SetFullToolState
00 14 3E 00 03  00 00 00 00  00 00 00 00  00 00 00 00  00 00 00 00 00

Gated SetPersistentMemory, four separate transactions
00 07 A1 00 5D 00 00 27 10
00 07 A1 00 5F 00 01 24 F8
00 07 A1 00 82 00 00 27 10
00 07 A1 00 84 00 01 24 F8

CPR Toggle
00 02 3F 00

BeginDraw
00 04 21 00 00 00

MultiDrawVector
00 37 43
      02 00 00 80 3F 00 00 80 3F
      00 00 00 00 40 00 00 80 3F
      00 00 00 00 40 00 00 00 40
      00 00 00 80 3F 00 00 00 40
      00 00 00 80 3F 00 00 80 3F
      02 00 00 80 3F 00 00 80 3F

Job EndDraw
00 04 22 00 00 00
```

**What remains unknown.** This is a partial connect-to-cut transcript. The static exports leave portions of startup framing, interface-state writes, and the selected tool-prep branch unresolved. A concrete run would also supply the UI timing, cached state, firmware replies, and job profile that determine which branches execute. The cause of continuous bobbing and insufficient contact remains unresolved at the firmware/mechanical boundary. The strongest exact parity findings are the pre-setup `22`, the job-level size in `30`, and the four conditional `A1` frames. The host trace does not establish a missing force-commit opcode or show that any one of those differences will restore downforce.