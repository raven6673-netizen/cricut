# MatCraft — machine-side handoff for Codex

**Date:** 2026-11-20 · **Target machine:** Cricut Maker (M/N CXPL301, "Maker-2BFE", BT addr `00:06:66:8D:2B:FE`, RN42 module) on **Bazzite (Linux)** · **Link under test:** USB serial `/dev/ttyACM0` (vendor 20D3) · **App:** MatCraft, Rust + egui, in `app/` (`cargo build`/`cargo test` from `app/`).

## Goal
Drive the user's **own** Cricut Maker from Linux with MatCraft, replacing Cricut Design Space. This is interoperability / own-hardware work. Protocol was reverse-engineered **statically** from the installed `CricutDevice.exe` (the Design Space machine helper), decompiled with Ghidra via the REA tools. Source notes: `app/MECHANICS.md`, `app/CUT-PROTOCOL.md`, `app/TOOL-PRESSURE.md`, `app/VECTOR-KINDS.md`, `app/PRESSURE-CALIBRATION.md`, `app/CUT-INIT.md`, `app/TOOL-PREP.md`.
`CricutDevice.exe` path: `/home/igpx12/.var/app/com.usebottles.bottles/data/bottles/bottles/CricutDesignSpace/drive_c/users/igpx12/AppData/Local/Programs/Cricut Design Space/resources/plugins/device-common/CricutDevice.exe` (PE32 x86, image base `0x400000`).

## What WORKS (verified on hardware, USB, plain/unencrypted frames)
- **Connect + status**: machine answers plain status reads (e.g. `00 01 73` → interface state).
- **Full cut command stream, all ACKed**, with flow control. The machine moves the carriage along the design path.
- **Calibrated pressure**: MatCraft reads the machine's own grams→counts table and sends an in-range count.
- **Tool detect**: machine reports tool code `02` = drag blade.
- **No AES needed**: cutting works in plain frames; the NewAES key table is deliberately NOT copied.

### Framing (confirmed)
2-byte big-endian header: top bit = encrypted, low 15 bits = body length (≤512). Plain frame = `00 <len> <body>`. With a session key, the original encrypts the **whole inner frame** (`frame_plain(body)`), AES-256 ECB, pad to a whole extra 16-byte block, last byte = 8-bit sum. MatCraft defaults to plain.

### The exact sequence MatCraft currently sends for a cut (all confirmed against the host)
Tool prep → setup → draw. Each command sent singly; wait for ACK (`00 <opcode> … <buffer_space>`); NACK is `01 <opcode> …`; draw batches retried on a full-buffer NACK (buffer space = last ACK byte, −56 per `0x43` batch).
```
B9                      position query → reply 00 09 B9 <X f32 BE> <Y f32 BE>
(if Y>8.0: 21 / 43 move to (X,8.0) / 22)
F0 08                   home theta (initial drag-knife selection)
EA                      tool detect → reply 00 06 EA <tool-code> <4 samples>   (got EA 02 …)
22 00 00 00             pre-setup EndDraw
30 08 00 00 0C 0C 00 00 00 00 00 <pressure u16 BE>   SetMachineState (12x12 area, Z-count pressure)
31 0F 0F 0F 0F 0F + 20×u16 BE                         SetSpeedState (drag selector-0 profile, CONFIRMED correct)
37 03 + 3×f32 BE(0)                                   SetToolState (drag blade)
3E 00 03 + 3×f32 BE(0) + 5×u8(0)                      SetFullToolState
A1 00 5D 00 00 27 10                                  write persistent 0x5D = 10000
A1 00 5F 00 01 24 F8                                  write persistent 0x5F = 75000
A1 00 82 00 00 27 10                                  write persistent 0x82 = 10000
A1 00 84 00 01 24 F8                                  write persistent 0x84 = 75000
3F 00                                                 CprToggle (drag)
21 00 00 00             BeginDraw
43 <records…>           MultiDrawVector batches ≤56 records; record = kind:u8, x:f32 LE, y:f32 LE (inches)
                        kind 02 = travel/up, 00 = cut/down; per contour: 02 …00s… (close 00) terminal 02
22 00 00 00             EndDraw
```
Pressure in `0x30` = Z encoder counts, from the machine's calibration (A0 indices `0x21`–`0x30`, each reply bytes 7–8 = grams, 9–10 = counts; this Maker: 100–434 g → 845–1541 counts). `grams_to_counts` interpolates like the original (single-precision linear, truncate, saturate at ends).

## The OPEN PROBLEM (unsolved)
Even with the complete sequence above accepted by the machine (pressure echoed, blade detected), **physically the drag blade bobs up and down on every segment and is too light to touch the material.** Raising pressure within the calibrated range did not fix it. Codex's static traces concluded the cause is **in the Maker firmware's motion/Z-control loop, which is NOT present in `CricutDevice.exe`** — so further static reading of that binary cannot resolve it. Our speed profile is confirmed correct for the drag-knife; there is no Z-datum/probe command in the host cut path (a shared `ZorroHomeZAxisCommand` serializes `F0 04` but is never called in the drag path).

### Latest hardware log evidence (USB)
```
← pos B9 3F C1 AD 30 3F A3 D7 AA      (X≈1.51, Y≈1.28 in)
← home 00 F0 45 01 FF                  (ACK)
← detect tool EA 02 00 00 00 00        (tool code 02 = drag blade)
← 30 00 00 00 0C 0C 00 00 00 00 00 04 0F   (echoes 12x12 + pressure 0x040F=1039)
← A1 00 5D 00 00 27 10 … (all four A1 writes echoed/accepted)
← 00 21 … / 00 43 … / 00 22 …          (begin/vectors/end all ACK)
Cut job finished: all 14 commands acknowledged.
```

## TWO PATHS FORWARD (pick based on whether Design Space can cut on this machine)

### Path 1 — Capture a real Design Space cut over USB and diff (PREFERRED, if feasible)
Only viable **if the official Cricut Design Space (running in the user's Bottles/Wine prefix) can actually drive and cut on this Maker.** If it can:
- Capture the live USB serial traffic to `/dev/ttyACM0` while Design Space performs a real cut (Linux `usbmon` / Wireshark on the usbmon interface, or a serial-port proxy/sniffer between Wine and the device).
- Decode the captured frames (same 2-byte header framing) into the command stream **with timing**.
- Diff against MatCraft's sequence above. Look specifically for: (a) **connect-time initialization we skip entirely** (see Path 2), (b) any command between setup and streaming that engages/holds Z, (c) inter-command timing/pacing the firmware may require, (d) different tool/speed/pressure values for the actual blade+material.
- Implement the diff in `app/src/machine.rs`.
This is the definitive route — it replaces guessing with ground truth.

### Path 2 — Implement the full connect-time initialization we skip (static, speculative)
MatCraft jumps straight to prep+setup+cut and **skips Design Space's entire power-on/connect handshake**. From `app/CUT-INIT.md` "Common initialization" (helper order `0x476760`, partly resolved): Reset AES (`CF <3 bytes>`), firmware query opcode `12`, version-gated command count (`00 01 77`), a `03` query (`0x472380`), machine status (`00 04 60 00 00 00`), serial slot reads `A0 0051`–`A0 0054`, gated position `B9`, scale reads `A0 00A3` (and `A0 009B`), `A0 0011`; then Athena init: Clear Error `0A`, Get Persistent Value Count `A4`, status refresh. It is plausible the Z/force subsystem stays uninitialized without this. Trace these to exact bytes (functions: `0x476760`, `0x4e2460`, `0x4e2800`, `0x471500`, `0x472380`, `0x497360`, `0x471710`, `0x48e5b0`) and send them at connect before the first cut. Lower confidence than Path 1.

### Other leads worth checking
- The `EA` detect returned fingerprint samples `00 00 00 00`; a real cut may read/use non-zero samples — confirm from a captured session.
- The Maker's Adaptive Tool System may expect a tool/profile other than "passive drag-knife" (flags `03`, zero lift threshold). Verify the actual installed blade's profile against a captured cut.
- Confirm whether the firmware needs the mat-load/**Go**-button wait (interface state `0x73`, reply offset 3: bit2=Go, bit1=Mat) before motion; MatCraft currently does not poll for Go.

## Wire session (NEW — `app/src/session.rs`)
Per the `binary-check` README, the official wire is **X25519 handshake + AEGIS AEAD** over an
anonymous session. `session.rs` implements it (clean-room boundary crossed at the user's direction):
- `Handshake` — ephemeral X25519 keypair, `request_body()` builds the plaintext `c7` frame (code +
  our 32-byte pubkey, padded to the observed 49 bytes), `complete(reply)` derives the shared secret.
- `Session` — AEGIS-256 `seal`/`open` of wire bodies as `tag(16) || ciphertext`.
- **Four parameters are hypotheses, not confirmed:** AEGIS variant (256 chosen), key derivation
  (shared secret used raw), nonce (per-direction counter), frame layout (tag-prepended). Confirm with
  `session::validate_against_capture(key, ctr, wire_body, plaintext)` against a captured
  (NtWriteFile plaintext, USBPcap ciphertext) pair before trusting it.
- Live probe: `Request::TestHandshake` ("Test encrypted handshake" button) sends `c7` and reports
  whether the machine returns a public key — i.e. whether it accepts an **anonymous** key (open Q #2).
  It does NOT switch the link to encrypted frames; the plaintext cut path is unchanged.
- Tests pass (handshake agreement, AEGIS round-trip, tamper reject). Still blocked for a real
  encrypted cut on: validating the four params, then building the `PBMatCutExecutionPlan` protobuf.

## Protobuf job builder (NEW — `app/src/job.rs`)
Builds the official cut job as protobuf, from the schema recovered in `binary-check` (`proto/`,
JOB-FORMAT.md). Clean-room boundary crossed at the user's direction.
- `Pb` — a minimal protobuf writer (varint / fixed64 / length-delimited); no protobuf dep.
- `build_cut_plan(id, op, contours_in_inches, custom_pressure)` → serialized `PBMatCutExecutionPlan`:
  `grouped_executions → PBMatCutGroupExecution(tool_type, line_type) → PBMatPathData(custom_pressure)
  → PBPathSplitData → PBDrawVectors → PBDrawVector{cut_type, point, command_number}`.
- `tool_and_art(op)` maps MatCraft ops to `PBToolType`/`PBArtType` (Cut→DRAG_KNIFE/CUT, etc.).
- `matcraft --job-dump` prints the protobuf bytes for a test square (verified: decodes correctly).
- Tests pass. **Not yet sent to the machine** — needs the encrypted session (session.rs) validated,
  and confirmation of which layer the wire carries (this high-level `PBMatCutExecutionPlan` vs the
  lower-level machine-step `#46` node layout seen in the NtWriteFile capture, JOB-DECODED.md).

## Code map (`app/src/machine.rs`)
- `Request::Cut { records, setup }` worker (line ~747): runs prep (B9/F0/EA), then `setup_payloads` + `cut_job_payloads`, streaming with ACK/retry + buffer-space flow control.
- `setup_payloads` (~1342): pre-EndDraw, `set_machine_state_body`, speed/tool/full-tool, `ATHENA_SETUP_WRITES` (four A1), CPR.
- `cut_job_payloads` (~1363), `contours_to_records` (~1125, emits `02 …00s… 02`), `multidraw_body`.
- Calibration: `Request::ReadCalibration` (reads A0 `DRAG_CAL_FIRST..=DRAG_CAL_LAST`), `parse_persistent_pressure`, `grams_to_counts` (~1313).
- Prep: `position_query_body`/`home_theta_body`/`tool_detect_body`/`parse_position`, `framed` helper.
- UI in `app/src/app.rs`: `cut_on_machine`, `machine_cut_records`, `log_machine` (writes `~/.config/matcraft/machine-log.txt` — the log file to read back), pressure-in-grams control in `machine_window`.
- CLI: `matcraft --cut-dry-run` prints the framed byte sequence of a test square (no hardware).

## Environment / how to run here
- Build/test: `cd app && cargo build && cargo test` (cc/cargo available in this shell).
- MatCraft must run on the **host** (not in the container/Flatpak) for USB access; the user launches it from a desktop entry. The in-app machine log is also saved to `~/.config/matcraft/machine-log.txt` — read that to see machine replies (the UI log panel isn't copyable).
- Decompiled Ghidra pseudocode (one file per function, by address) is cached at:
  `/tmp/claude-1000/-home-igpx12--claude-spares-spare-mE4oLv/a2849e9e-3be2-48e5-b9c8-fc2042e257ba/scratchpad/{decomp-machine,decomp-crypto,cricutdevice-decomp}/functions/` (each with `decompile-index.json`). These are session-temp; re-decompile with REA if gone.
- Do NOT copy cryptographic keys from the binary. Static analysis only unless the user asks for a live capture; never drive the machine from a script — the user initiates physical cuts from MatCraft.

## Bottom line for Codex
The host command protocol is fully decoded and replicated; the machine accepts everything and identifies the blade, but the blade won't engage/hold force — the gap is firmware-side and invisible to `CricutDevice.exe`. **Start with Path 1 (capture a real Design Space cut and diff) if Design Space can cut on this machine; otherwise Path 2.**
