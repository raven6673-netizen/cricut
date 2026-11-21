# How the original talks to the machine

Read from the original's machine helper program (`resources/plugins/device-common/CricutDevice.exe`,
32-bit, SHA-256 7283020485a95ae0…) with the Deep engine (Ghidra). Addresses are in that file.
This app's code was written from these notes, not copied.

## Connection

| What | Value | Where |
|---|---|---|
| Link | Bluetooth serial (RFCOMM); a USB serial path also exists | strings `bluetooth_serial_transport`, `usb_serial_transport`, `Windows.Devices.Bluetooth.Rfcomm.RfcommServiceId` (0x9950f0) |
| Machine family of the Maker | "Athena" (tool list: blade, drag, deep, rolling, scoring wheel, double scoring wheel, stylus, pen, perforation, debossing, etch, wave) | strings 0x95be58–0x95bf78, used by 0x44dac0 |
| Your machine's address | 00:06:66:8D:2B:FE (from the request; 00:06:66 is a Roving Networks / Microchip serial Bluetooth module) | request |
| Bluetooth module | Microchip RN42 | FCC ID **T9J-RN42** on the machine's own label |
| Model number | CXPL301 | M/N on the machine's label |
| Bluetooth name | Maker-XXXX, where XXXX is the last two address bytes; the label's machine is **Maker-2BFE** for address ...2B:FE. MatCraft derives and shows this when pairing | the machine's label |
| RFCOMM channel | 1 | **estimated** (the RN42's default SPP channel; adjustable in the app) |
| USB | USB serial device, vendor 20D3, product 0015; a COM port on Windows (COM6 here), /dev/ttyACM on Linux | seen on this computer |
| Live check | `matcraft --live-check` asks the machine on USB three read-only questions; on this computer: get interface state `00 01 73` → `00 06 73 CF 0F 00 03 00`, read error severity `00 01 0B` → `00 02 0B 00` (no error), get persistent value count `00 01 A4` → `00 05 A4 DE 00 00 00`. All plain, all starting with the command's code. Saved in mechanics-live.json and shown as the "Real machine" column of the behaviour check | run on this computer |
| USB test | MatCraft sent `00 01 73` (get interface state, unencrypted) and the machine replied `00 06 73 CF 0F 00 03 00`: same framing as Bluetooth, no encryption needed for status questions | `matcraft --usb-probe`, run on this computer |

## Message framing

From `read_response` (0x50f540) and `encrypt_command_bytes` (0x48de60), AES block step 0x5cfbb0:

* Every message starts with a **2-byte big-endian header**: top bit = encrypted, low 15 bits = body length.
* A reply body longer than **512 bytes (0x200)** is refused.
* **Encrypted** bodies: AES with a **32-byte key (AES-256)**, each 16-byte block on its own (ECB).
  Before encrypting, the body is zero-padded to `len - len % 16 + 16` bytes (so an extra block is
  always added) and the **last byte is set to the 8-bit sum of all bytes before it**.
* Replies: decrypt, check that same sum, then **swap the first two bytes when the first is not zero**.
* The first byte of a reply is the command's own code; anything else is "Unexpected response"
  (e.g. 0x4fe820).
* Plain (unencrypted) frames use the same 2-byte header with the top bit clear: confirmed by the
  NewAES request builder (0x4f9230), which writes `00 31` (length 49) in front of its 49-byte body.

## Command codes (each class's code getter = vtable slot 5)

| Command | Code | Getter |
|---|---|---|
| Clear error | 0x0A | 0x4fe470 |
| Read error severity (reply: severity 0–3) | 0x0B | 0x4fe810, reply parser 0x4fe820 |
| Set speed state | 0x31 | 0x4fce20 |
| Set tool state | 0x37 | 0x4fdae0 |
| Set full tool state | 0x3E | 0x4fded0 |
| CPR toggle | 0x3F | 0x4fec50 |
| Abort cutting | 0x68 | 0x4eb0e0 |
| Get interface state (buttons, lights) | 0x73 | 0x4fa2c0 |
| Get persistent value count | 0xA4 | 0x4fefc0 |
| Scan for edge | 0xE2 | 0x4effa0 |
| Bluetooth alert | 0xE5 | 0x4e3180 |
| Home theta (blade turn) | 0xF0 | 0x4fb140 |
| Move 4D | 0xF1 | 0x4ff460, payload builder 0x4ff470 |
| Send zeros | 0xFF | 0x4e3060 |
| New AES session | 0xC7 | 0x4e9b70, request builder 0x4f9230 (vtable 0x962c64) |

### New AES session request (0x4f9230)
51 bytes in all: header `00 31`, code `C7`, a **key slot = random % 12 + 2** (so 2–13), then
**47 random bytes** (MT19937 generator). The slot picks one of the keys built into the original
program; this app does **not** copy that built-in key table, so it can send the request and show
the reply, but can't derive the session key on its own. A key can be pasted in by the person.

Class names come from the RTTI descriptors (0xa33354…0xa335cc) matched to their vtables
(0x962d80–0x9634b0) through the complete-object locators (0x9c2938…).

### Move 4D payload (0x4ff470)
`code, 0x17 (axis mask), x, y, z, theta` — four **big-endian float32** values (bytes are written
last-to-first by 0x452a60). Theta is given in degrees and multiplied by **0.017453292519943295**
(π/180, constant at 0x95ebc0) to send radians.

## Not decoded yet

* **AES session key set-up**: the request is known (above); how the reply plus the built-in key
  slot become the 32-byte session key wasn't followed, and the built-in keys aren't copied.
  The app accepts a key typed in by the person and otherwise talks unencrypted.
* Material pressures and passes: the original downloads these from its online service, so the
  table in the app is **estimated**.

## Streaming cut paths (now decoded — see CUT-PROTOCOL.md)

Traced from CricutDevice.exe on 2026-11-20 (full report in **app/CUT-PROTOCOL.md**). A cut job is:

1. **BeginDraw** `21` (body `21 00 00 00`), then the setup commands the Athena path sends — machine
   state `30`, speed state `31`, tool state `37`, full tool state `3E`, CPR toggle `3F`.
2. A run of **MultiDrawVector** `43` batches. Each body is `43` then N records of
   `kind:u8, x:f32 LE, y:f32 LE`, x and y in **inches** on the mat (the original divides its 72
   units/inch coordinates by 72). Kind `02` = travel move (tool up), `00` = cut line. Batches are
   capped at ~**56** records (device field +0x13c; model/firmware can lower it). There is a
   single-vector form **DrawVectorFloat** `42` (`42 kind:u32 LE x:f32 LE y:f32 LE`) chosen per the
   device flag.
3. **EndDraw** `22` (body `22 00 00 00`).

Each command is sent on its own and the host waits for an acknowledgement before the next:
`00 <opcode> …` is an ACK, `01 <opcode> …` a rejection; draw ACKs are retried for ~60 s. Mat-load
and **Go** are waited on with Get Interface State `73` (reply offset 3: bit 2 = Go, bit 1 = Mat).

Framing: the draw opcodes go in **plain** frames unless a session key is installed, in which case
the original encrypts the **complete inner frame** (2-byte length + body), with `New AES`,
`Reset AES`, `Send Zeros`, `45`, `46`, `47` bypassing encryption. Status commands already work in
plain on this machine, so MatCraft streams a cut in plain by default.

MatCraft implements this in `machine.rs` (`contours_to_records`, `multidraw_body`,
`cut_job_payloads`, `draw_ack`, and the `Request::Cut` worker path; "Cut design on machine" in the
Machine panel). `matcraft --cut-dry-run` prints the framed bytes of a 1-inch test square.

**Verified on hardware 2026-11-20 (USB, Maker-2BFE):** the machine accepts **plain (unencrypted)
cutting** — `BeginDraw` ACKed `00 21 …` and nine `0x43` batches ACKed `00 43 …`, so no AES session
is needed. The ACK is `00 <opcode> <..> <..> <space>` and a rejection is `01 <opcode> …`. The ACK's
**last byte is the machine's remaining buffer space**, falling ~`0x38` (56, one batch) per command;
when it runs low the machine NACKs the next batch. MatCraft's `Request::Cut` therefore **waits and
resends the same batch (up to 90 s) on a NACK or silence** until the buffer drains, matching the
original's retry. Still to tune: mat origin and axis directions and the tool/speed/pressure values
for a clean cut, and telling a buffer-full NACK from a genuine error.

* **AES session key set-up (left unimplemented on purpose)**: the `C7` request is known (above); the
  reply plus the built-in key slot derive the 32-byte session key, but **the built-in key table is
  Cricut's and is not copied into MatCraft**. The app accepts a key typed in by the person and
  otherwise talks unencrypted. This is only needed if the machine refuses plain motion.

## Serial number and activation

From a Hackaday write-up (21 Aug 2026, "Spoofed Serial Number Unlocks Cricut Machine", by
Zoe Skyforest, about xssfox's repair of an e-waste machine):

* The machine sends its **serial number to the PC software in plain text over USB**, with no
  checksum or encryption. MatCraft reads the same serial from the USB device descriptor
  (`0580717011` on this computer) and shows it in the Machine panel.
* **Deactivation is enforced in the host software, not the machine.** The maker's program reads
  the serial, checks it against a disabled list (a machine switched off after, for example, a
  warranty replacement) and refuses to work with it. The firmware itself still talks normally.
* Because the check is host-side, **MatCraft has no activation gate at all**: it keeps no serial
  list and talks to whatever machine is on the link, so a machine the maker marked off still
  works here.
* The encrypted cut packets use **AES-ECB** (identical plaintext blocks give identical
  ciphertext), which matches the ECB framing read from the original.
* The article unlocks a machine by **spoofing another machine's serial** with a man-in-the-middle
  board. MatCraft does **not** do this: impersonating a serial would stop the real owner's
  machine working, so it is deliberately left out.

## Engraving (from the original's app bundle)

- Styles: engrave outline, engrave fill and engrave photo.
- Fill patterns: horizontal lines (default), vertical lines, grid, forward slash, backward slash,
  cross-hatch, concentric. Line spacing starts at 0.01 in, edge margin (kerf) at 0.01 in, fill
  rule non-zero.
- Tools: the QuickSwap "Engrave Tip" and a separate "Engraving Tool". Which machines take the
  Engraving Tool is an estimate (Maker 5 here).
- Engrave projects use an 11.5 x 11.5 in design area and keep their layout on the mat.
- Photo engraving here uses ordered dithering into short lines; the original's photo method
  wasn't decoded, so this is MatCraft's own.

## Behaviour check (sheet vs MatCraft vs the original)

`matcraft --check-mechanics [report.json] [--original <CricutDevice.exe>]` (also the **Machine
behaviour** tab of the Compare window, and a test in `cargo test`) compares, for every value in
mechanics.json:

* **the sheet** (this file),
* **MatCraft, measured**: its real code is run — framing a 1-byte command, encrypting 1/15/16/17/
  31/32-byte messages and decrypting them back (checks padding sizes and the sum byte), feeding a
  512- and a 513-byte reply through the live reply reader, 3000 session requests (every key slot
  2–13 and nothing else), the bytes sent for 180° and 1 radian, and where the mat layout puts two
  2-inch pieces,
* **the original's file, read** (never run): each command's code getter (`b8 CODE 00 00 00 c3`),
  the blade-turn mask (`c6 45 f3 17` at 0x4ff4d1), the encrypted flag (`0c 80` at 0x48def4), the
  reply limit (`81 ff 00 02 00 00` at 0x50f624) and the degree constant (f64 at 0x95ebc0). File
  offset = VA − 0x400C00 for .text and VA − 0x401400 for .rdata; a different version is detected
  and not compared.

Result on 2026-11-19 with the installed original (SHA-256 7283020485a95ae0…): **29 same, 0
different, 5 estimates, 2 not decoded**; 19 of the matches were also read from the original's file.
Saved in mechanics-measured.json. What still can't be compared: anything only seen when the original
runs against a machine (timing, the session-key reply, cut-path streaming).

## Design rules in this app

The original's NewAES reply parser at `0x4f9180` accepts a big-endian body length
of 48 bytes and stores the reply (evidence `ev_5cb1d3360d1f22d6531e0f92c86dcf2b5ac6d274000bf68f35456f4e5bf8f017`).
The session key derivation and cut-path streaming are still open. This observation
is currently documentation only because MatCraft does not consume that handshake reply.

* Units on the canvas: inches; 48 screen pixels per inch at 100 % zoom (own choice).
* Mats: 12 × 12 in or 12 × 24 in; pieces kept **0.25 in** from each other and from the mat edge
  (**estimated**), one mat per cut colour; draw/score lines ride on the first mat.
* Mat files are written in millimetres (1 in = 25.4 mm).
