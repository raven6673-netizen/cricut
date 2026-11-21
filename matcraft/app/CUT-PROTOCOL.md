# Cricut Maker / Athena cut-streaming report

Athena sends cut geometry through shared draw commands. **`0x43` sends a batch of XY vectors and `0x42` sends one vector.** The vector type distinguishes travel from cutting. `0xF1` Move 4D has a separate builder and call path. The most consequential framing detail for MatCraft is that **the original encrypts the two-byte inner length along with the command body**.

This report uses static Ghidra exports and targeted PE inspection. Addresses are virtual addresses in `CricutDevice.exe`. **Confirmed** means the implementation or data is visible in the artifact. **Likely** means the code supports the interpretation and a caller, model condition, or firmware behavior remains unresolved. No hardware was accessed.

## 1. From `path_data` to draw vectors

| Confidence | Finding | Evidence |
|---|---|---|
| **Confirmed** | `PBMatPathData.path_data` is protobuf string field **2**, stored at object offset `+0x48`. The functions referencing the supplied string anchor handle protobuf decoding and encoding. The cut preparation chain processes the string later. | `0x51da10`, `0x51e700`, `0x4be370` → `0x45ac40` → `0x45a1c0` → `0x46d0c0` |
| **Confirmed** | The lexer skips whitespace and commas, recognizes SVG command letters, and parses numbers through `strtod`. The two directly recovered parser switches handle `M`, `L`, `H`, `V`, `C`, and `Z/z`. Lexer recognition alone does not establish support for lowercase relative commands, arcs, or quadratic curves in these two switches. | `0x4dba10`, `0x4dbad0`, `0x46e1a0`, `0x46e5c0` |
| **Confirmed** | The direct parsers use 24-byte intermediate records containing a 32-bit kind at offset 0 and doubles at offsets 8 and 16. `0x46d260` converts these to `PBDrawVectors` containing `PBDrawVector` entries with a `cut_type` and a `PBPoint`. Move-to is `2` and line-to is `0`. The non-flattening branch retains cubic information in multiple point records. The flattening branch emits line-to records for subdivision points and the cubic endpoint. Other preparation modes dispatch through `0x4dbde0` to `0x80c790`. | `0x46d0c0`, `0x46e100`, `0x46e510`, `0x46e1a0`, `0x46e5c0`, `0x46d260`, `0x4dbde0` |
| **Confirmed** | The recovered flattening branch recursively bisects the cubic at its midpoint. Its tests include a squared geometric deviation comparison, angular checks, and a configured recursion limit of 64. The dispatch passes a tolerance parameter whose baseline in this preparation path is `0.01`, overridden by a larger configured value. The geometric test uses this value against squared distances, so `0.01` is not a direct maximum error in inches or millimetres. | `0x45a1c0`, `0x46e5c0`, `0x5db740`, `0x5dc020`, `0x5db810` |
| **Confirmed for draw coordinates** | The vector senders convert host coordinates at **72 units per inch** to **inches**. The bulk command divides both point fields by 72 when appending the record, then serializes the results. The single-vector builder divides by 72 during serialization. Both preserve point-field order and sign at this stage. | `0x4ed0b0`, `0x4ed2d0`, `0x4df3b0`, `0x4eda60`, `0x4ed790` |

The PE’s `PBDrawVectorType` descriptor assigns `00` line-to, `01` curve-to, `02` move-to, `03` line-in, `04` line-out, `05` curve-in, `06` curve-out, `07` checksum, `08` none, and `09` end-path. **Confirmed** from the descriptor at file offset `0x577111` and the direct parser’s assignments at `0x46d260`. The existence of these values does not establish that every value appears in every Athena job.

## 2. Commands and job order

**Confirmed factory mapping.** The `AthenaDevice` vtable at `0x95d564` points to factory `0x48df60`. Its default path calls `0x48c6d0`, which constructs the shared BeginDraw, EndDraw, DrawVectorFloat, MultiDrawVector, GetCommandCount, and RepeatLastResponse classes. Athena specifically overrides interface state and abort. Factory selector values are internal and differ from wire opcodes.

**Confirmed call paths, likely normal-job ordering.** The mat-cut entry reaches `0x4bd430` → `0x4be370` for preparation and `0x4bda90` → `0x4c0e40` for per-path execution. The integration path in `0x50c2c0` explicitly sends BeginDraw; Athena’s setup override `0x48e6e0` sends machine state, speed state, tool state, full tool state, and CPR toggle in that order on the branch that completes all five operations. Path execution reaches `0x4c15b0` → `0x4dd840` → `0x4df770` for vector streaming. The finish helper at `0x472b50` sends EndDraw and waits for acknowledgement. Tool changes, recovery, alignment, and model or firmware checks can add intervening commands.

The offsets below start at the opcode and exclude framing.

| Command | Body layout | Confidence and evidence |
|---|---|---|
| **BeginDraw `21`** | `21 00 00 00` | **Confirmed.** `0x4eab96`, `0x4eabe0`; send site `0x50c2c0`. |
| **Set Machine State `30`** | Byte 1 is `(state << 7) \| flags \| 08`. Bytes 2–3 are zero. Bytes 4–5 are the integer-converted dimension values, in reversed point-field order at the `0x48e6e0` call site. Bytes 6–9 are zero. Byte 10 comes from object `+0x11`. Bytes 11–12 are a big-endian 16-bit value from object `+0x30`. | **Confirmed layout.** Flag semantics, dimension rounding, and the physical meaning of the final word are only partially resolved. `0x48e6e0`, `0x4f8d90`, `0x4f8de0`, `0x452a60`. |
| **Set Speed State `31`** | `31 0F 0F 0F 0F 0F`, then 20 big-endian 16-bit words. The order is four groups of five, reading the same speed-record field across the five records before advancing to the next field. Values are scaled by 100 and rounded before serialization. | **Confirmed layout and scaling.** Physical units and channel meanings remain open. `0x4fce20`, `0x4fd240`, `0x452a60`. |
| **Set Tool State `37`** | `37`, one flag byte using bits 0–3, then three big-endian float32 values. The fields are lift threshold, parallel offset, and lagging offset. | **Confirmed layout.** Field names from the diagnostic formatter and the setup caller. `0x48ef60`, `0x4fdaf0`, `0x4fdc30`, `0x452a60`. |
| **Set Full Tool State `3E`** | `3E 00`, packed flags, three big-endian float32 values for threshold and offsets, then five one-byte node values. Total body length is 20 bytes. | **Confirmed layout.** The diagnostic formatter identifies the nodes and several flags. Their pressure calibration and physical units remain open. `0x4fdee0`, `0x4fe170`, `0x452a60`. |
| **CPR Toggle `3F`** | `3F <toggle:u8>`. The recovered setup caller sets the toggle to `1` for its tool mode `3`, and `0` otherwise. | **Confirmed.** `0x48e6e0`, `0x4fec50`, `0x4fed60`. |
| **DrawVectorFloat `42`** | `42 <kind:u32 LE> <X:f32 LE> <Y:f32 LE>`. Body length is 13 bytes. XY is in inches. | **Confirmed.** `0x4df3b0`, `0x4eda60`, `0x4ed790`. |
| **MultiDrawVector `43`** | `43` followed by `N` records. Each is `<kind:u8> <X:f32 LE> <Y:f32 LE>`. Body length is `1 + 9N`. The builder writes no separate count field. XY is in inches. | **Confirmed.** `0x4df770`, `0x4ddc20`, `0x4ed0b0`, `0x4ed2d0`. |
| **EndDraw `22`** | `22 00 00 00` | **Confirmed.** `0x4eae66`, `0x4eaea0`, `0x472b50`. |
| **Abort Cutting `68`** | `68 <selector:u8>`. The Athena constructor defaults the selector to `FF`, so the recovered default abort body is `68 FF`. The reply parser expects `68` in the echoed-opcode position of an ACK-style reply. | **Confirmed.** `0x472660`, `0x48df60`, `0x4fb310`, `0x4fb4a0`, `0x4fb370`. |

**Confirmed auxiliary motion.** Move 4D is `F1 17 <X:f32 BE> <Y:f32 BE> <Z:f32 BE> <theta:f32 BE>`. The builder multiplies theta in degrees by π/180. Home Theta is `F0 <selector:u8>`. Internal values `0` and `1` serialize to `02` and `20`, respectively, and the remaining branch serializes `08`. The constructor defaults to internal value `1`. The separate Move 4D sender at `0x48dcc0` and the vector senders at `0x4df770`/`0x4df3b0` establish distinct paths. Physical Move 4D XYZ units and selector meanings remain open. `0x4ff470`, `0x452a60`, `0x4fb0e0`, `0x4fb170`.

**Likely tool engagement.** A `02` travel record followed by `00` cutting records is the vector-level move/cut distinction. The recovered vector loops carry no separate pen-up, pen-down, or Z field. Exact lift timing and behavior of the specialized vector kinds remain firmware questions. `0x4df3b0`, `0x4ddc20`, `0x4eda60`, `0x4ed2d0`.

## 3. Streaming, replies, and encryption

- **Confirmed — batching.** `0x4df770` chooses one `42` per vector or batches of `43` based on device flag `+0x134`. The bulk loop caps the record count at device field `+0x13c`. Initialization includes a common value of **56**, and model, firmware, or selected-tool configuration can change it. For 56 records the body is 505 bytes, the inner frame is 507 bytes, and the encrypted wire frame is **514 bytes**, including its two-byte outer header. This is an application frame size. `0x4df770`, `0x476760`, `0x4a8ba0`, `0x4d5590`, `0x48de60`.
- **Confirmed — reply before advancing.** The send path writes one command and enters `0x4e5900` to read and parse the reply. The bulk loop examines its ACK state before marking records complete and advancing to the next batch. A successful ACK body accepted by the shared parser is `00 <echoed-opcode> <b2> <b3> <b4>`. Negative responses have `01 <echoed-opcode> …` and body length seven or eight. The `43` caller reads an ACK-derived byte, updates a device limit, checks machine status, and retries eligible cases with a roughly 60-second bound. The `42` path also waits and retries in its own loop. `0x47a0e0`, `0x4e5900`, `0x50ee00`, `0x4ddc20`, `0x4df3b0`.
- **Confirmed — command count is used during initialization.** Get Command Count has body `77`. On the version-gated initialization path, the helper queries it and seeds its local command tracker with the returned count plus one. The normal send path increments that tracker and excludes the `IGetLastResponse` class. The recovered bulk loop uses ACK and machine-status information directly. Repeat Last Response has body `02`; its construction, parser, and tracker exception are confirmed, while its use in recovery on a particular Maker firmware remains unresolved. `0x476760`, `0x4ec380`, `0x4ec150`, `0x47a0e0`, `0x4f09f0`, `0x4f0780`.
- **Confirmed — outgoing encryption covers a framed command.** The Athena frame builder `0x48f050`, reached through `0x4528b0`, prefixes the command body with its big-endian two-byte length. `0x47a0e0` passes that serialized buffer to the AES wrapper when a session key is installed, except for explicit bypasses including New AES, Reset AES, Send Zeros, and opcodes `45`, `46`, and `47`. Thus `21`, `42`, `43`, and `22` take the encrypted path with an active session. `0x48de60` pads the **inner frame** to `16 × (floor(inner_len / 16) + 1)`, puts the modulo-256 sum in the final byte, encrypts using AES-256 ECB, and adds an outer big-endian length with bit 15 set. The plain New AES request starts `00 31 C7`, followed by its slot byte and 47 random bytes. `0x48f050`, `0x4528b0`, `0x47a0e0`, `0x48de60`, `0x48d090`, `0x5cfbb0`, `0x4f9230`.
- **Confirmed — incoming framing.** The framed reader takes the lower 15 bits of the outer length and rejects bodies over 512 bytes. The eligible encrypted-reply branch decrypts, conditionally verifies the sum, then swaps the first two decrypted bytes if the first is nonzero. Command-specific parsing follows. ACK-style draw replies put the status byte before the echoed opcode. `0x50f540`, `0x4526d0`, `0x50ee00`.

In [MatCraft’s `machine.rs`](/home/igpx12/Documents/Cricut%20Anything/app/src/machine.rs:130), `frame_encrypted` copies its argument directly into the AES buffer. For parity with the Athena send path, that argument must be the **complete inner frame**, for example `frame_plain(&body)`. Passing the opcode and fields alone produces different ciphertext and, for some lengths, a different outer length.

## 4. Mat load, Go, completion, and abort

| Confidence | Stage | Evidence |
|---|---|---|
| **Confirmed** | The host has explicit mat-load and Go wait states. In the mat workflow it refreshes both machine status and interface state, sets interface indicators, checks the relevant conditions, and sleeps between polls. Machine status has request body `60 00 00 00`. | `0x4c5b40`, `0x4be8f0`, `0x4a55e0`, `0x4a64c0`, `0x4a0ed0`, `0x4f9d30` |
| **Confirmed** | Get Interface State’s body is `73`, so its plain frame is `00 01 73`. In the normalized reply body, at zero-based offset 3, bit 2 is Go and bit 1 is Mat. The Go wait reads the decoded Go boolean and can clear latched button state around polling. | `0x471860`, `0x48df60`, `0x4fa720`, `0x4fa530`, `0x4fa2e0`, `0x4fa2d0`, `0x474610`, `0x4a64c0`, `0x4728f0` |
| **Confirmed** | Athena initialization uses common setup, then Clear Error `0A`, Get Persistent Value Count `A4`, and a further initialization helper. Some mat setup paths call the EndDraw helper to bring the draw state to a known point and then invoke motion preparation. | `0x48e5b0`, `0x476760`, `0x4fe540`, `0x4feee0`, `0x4a55e0`, `0x472b50`, `0x472740` |
| **Confirmed** | Finish uses EndDraw and its ACK. The completion helper can subsequently poll machine status. The outer mat workflow can wait for a load or unload interaction depending on the selected mode. Abort is a distinct `68 FF` transaction on the recovered Athena path. | `0x472b50`, `0x472e60`, `0x4bf700`, `0x472660`, `0x4fb310`, `0x4fb4a0` |

## 5. Byte example — one-inch horizontal line

**Confirmed encoding for the stated coordinates.** Assume the mat is loaded, setup is complete, and the geometry reaching the draw sender is a move to **(72, 72)** host units followed by a line to **(144, 72)**. This is a one-inch line in positive machine X. The example shows the draw transactions and their inner frames. It omits material-dependent setup, any path prelude or trailer, and physical mat-origin assumptions. Sources are `0x4eabe0`, `0x4ed0b0`, `0x4ed2d0`, `0x4eaea0`, and `0x48de60`.

```text
BeginDraw inner frame
00 04 21 00 00 00

MultiDrawVector inner frame
00 13 43
      02  00 00 80 3F  00 00 80 3F     move to (1.0, 1.0) in
      00  00 00 00 40  00 00 80 3F     line to (2.0, 1.0) in

EndDraw inner frame
00 04 22 00 00 00
```

With an active session, the wire headers would be `80 10`, `80 20`, and `80 10`. The corresponding pre-encryption sum bytes are `25`, `D5`, and `26`, in each padded buffer’s final byte. The other padding bytes are zero. For the single-vector branch the two draw inner frames would instead be:

```text
00 0D 42 02 00 00 00 00 00 80 3F 00 00 80 3F
00 0D 42 00 00 00 00 00 00 00 40 00 00 80 3F
```

## Still unknown

- **Mat datum and physical axes.** The senders preserve signed XY and divide by 72. The full mapping from original design coordinates to a loaded Maker’s physical origin, positive carriage and feed directions, and offsets for each mat or tool has not been recovered. `0x4be370`, `0x45ac40`, `0x4ed0b0`, `0x4eda60`.
- **Geometry variants.** The full grammar and flattening rules inside `0x80c790`, which parser mode is selected for each Maker tool and firmware, and the required placement of specialized vector kinds such as `09` remain open. `0x45a1c0`, `0x46d0c0`, `0x4dbde0`.
- **Model-specific startup and tool values.** The host polls Mat and Go and applies tool-specific state. A complete model- and firmware-specific load/home/alignment transcript, speed channel definitions, pressure-node calibration, and precise flag semantics remain open. `0x4a55e0`, `0x4a64c0`, `0x48e6e0`, `0x4fd240`, `0x4fe170`.
- **Recovery and firmware enforcement.** The meanings of all ACK/NACK data bytes, conditions for choosing Repeat Last Response, supported maximum bulk size for each firmware, and whether the firmware accepts any draw command without an active AES session remain unverified. The host’s encrypted send path is established. Session-key derivation from the `C7` response was outside this trace. `0x4ddc20`, `0x50ee00`, `0x4f0780`, `0x47a0e0`, `0x4f9180`.