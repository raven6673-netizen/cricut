# Windows USB captures of Design Space ↔ Cricut Maker

Tools and findings from capturing the USB traffic between Cricut Design Space (Windows) and a
Cricut Maker (USB VID:PID `20d3:0015`). Use them as the baseline when checking what Design Space
under Wine (see the top-level README) sends on Linux.

Raw `.pcap` files are **not** kept in this repo (they contain the machine's serial number and
Bluetooth address). `.gitignore` excludes them.

## Files
| File | What it does |
|---|---|
| `capture.ps1` | Starts a USBPcap capture of one device; stops when a `STOP` file appears. Run elevated. |
| `decode_usbpcap.py` | Turns a capture into `summary.txt`, `bulk-dump.txt` and raw in/out streams. |
| `frames.py` | Lists the handshakes and job frames in a capture; compares two captures' ciphertext. |

## Workflow
### Windows (capture)
1. Install USBPcap (`winget install desowin.USBPcap`) and **reboot** (its driver loads at boot).
2. Find the hub and device address (see the comment at the top of `capture.ps1`).
3. Start the capture from an elevated PowerShell:
   ```powershell
   .\capture.ps1 -Hub \\.\USBPcap1 -Devices 5
   ```
   Optional: `-StopDir <folder>` adds folders that are watched for `STOP`, for example the
   working folder of a sandboxed agent that can't write to the capture folder.
4. Run the job in Design Space. Write down local times for Go, cut finished and mat unloaded.
5. Once the mat is unloaded, create an empty `STOP` file. `CAPTURE_DONE` appears when the capture
   file is closed.
6. Decode it:
   ```powershell
   python decode_usbpcap.py cricut-YYYYMMDD-HHMMSS.pcap analysis
   python frames.py cricut-YYYYMMDD-HHMMSS.pcap
   ```
7. Add a row to the run log below and push. Commit only the findings, never the `.pcap`.

### Linux (compare)
Capture the same job with `usbmon` (USB) or by logging the bridge's pty (Bluetooth), using the
same design, material, pressure, mirror and mat as a Windows run. Then compare frame sizes and order
with the run log. The 2-byte framing is the same; `decode_usbpcap.py` reads only USBPcap files,
so a usbmon capture needs its own header parser.

## Protocol findings (6 Windows runs, 2026-11-20)
**Transport**
- USB CDC-ACM serial device (bulk OUT `0x02`, bulk IN `0x82`; interrupt `0x81` unused).
  Design Space never sets a baud rate (no SET_LINE_CODING); it only sends DTR/RTS off when
  reopening the port.
- One frame per bulk transfer: **2-byte big-endian length word + payload**. The top bit of the word means
  **encrypted**; the low 15 bits are the payload length (`0x8010` = 16 encrypted bytes, `0x8150` = 336).
  Every request gets exactly one reply.

**Connection setup (plaintext)**, every time Design Space (re)connects:
| Host sends | Machine replies | Notes |
|---|---|---|
| `cf` + 3 random bytes | 16 random-looking bytes | nonce/challenge; different on every connection |
| `12 00 00 00`, `77`, `03`, `60 00 00 00`, `b9`, `0a`, `a4` | short status values | `77` and `b9` replies change between connections (counter/ID) |
| `a0 00 XX` (XX = `51 52 53 54 a3 9b 11 65 66`) | `a0 00 XX` + 4 bytes ×2 | register/info reads; `51..54` return short ASCII strings (version parts) |
| `3d 00` + text | `3d 00` + text | passthrough to the Bluetooth module: `$$$`→`CMD`, `GN`→name, `GB`→BT address, `---`→`END` (Roving Networks / Microchip RN-style command mode) |
| 49-byte frame starting `c7` | 48-byte frame | **key exchange**; everything after this is encrypted |

**Encrypted session**
- All payloads are multiples of 16 bytes (AES block size).
- **Key is per connection.** A fresh connection (app relaunch or reconnect) shares no ciphertext with
  earlier ones, even for the same job.
- **Deterministic within a connection.** The same plaintext always gives the same ciphertext: the idle poll
  (every 3 s) and the busy poll (18–36/s while working) are each one fixed 16-byte block, and the same
  job sent twice in one connection was byte-identical. Consistent with ECB or a fixed IV.
- The machine's 32-byte replies to the job frames keep an identical first block across runs and change only in
  the second block, so each 16-byte block looks independently encrypted.

**Job shape** (Vinyl, Default pressure, Mirror Off, 12 × 12 in mat):
| Design | Encrypted OUT frames around Go |
|---|---|
| one 1 × 1 in square | 64, 32, 32, 32 → **336** → … 32 |
| two 1 × 1 in squares | 64, 32, 32, 32 → **512 + 160** (= 672) → … 32 |

| one 1 in circle | 64, 32, 32, 32 → **512 + 512 + 272** (= 1296) → … 32 |
| one 3 in square | 64, 32, 32, 32 → **336** → … 32 |
| one 5-point star | 64, 32, 32, 32 → **512 + 352** (= 864) → … 32 |

So the byte count tracks **node/segment count, not physical size or shape count**: a 1 in and a 3 in square are
both 336 B (4 corners), two squares are 672 B (8 corners), and a 1 in circle is 1296 B (a polygonised curve, many
segments). ~336 B per 4-corner path. Frames are capped at **512 bytes** (the circle filled two 512-byte frames plus
a 272-byte remainder). Because each path's cost is predictable, the frame sizes reveal a job's rough complexity even
without the key.

Measured bytes-per-node (run 9): **≈ 84 B per node, near-linear, ~0 fixed overhead** — 1 in and 3 in squares
are 84×4 = 336 B, two squares 84×8 = 672 B, a 5-point star (10 nodes) 864 B (~86/node), and a 1 in circle
1296 B fits ~15–16 polygon segments. Job size ≈ 84 B × node count.

**A cut's end** (first captured in run 8): the busy status poll (18/s while cutting) drops back to the idle poll
(every 3 s) when the machine finishes.

**Not known yet:** the key exchange (how the `c7` frame relates to the nonces), the cipher mode, and the
content of the 336-byte records. USB traffic alone can't answer these; it would take analysing the
Design Space device service (`CricutDeviceIO` / `CricutDevice`), which `matcraft/capture/README.md`
rules out for the clean-room MatCraft work.

## Run log
| Run | Connection | Design | Job frames | Complete cut in capture? |
|---|---|---|---|---|
| 1 | new (app start) | 1 × 1 in square | 64/32/32/32 + 336 + 32 | no (stopped mid-cut) |
| 2 | same as 1 | same | identical ciphertext to run 1 | yes |
| 3 | same as 1 | same | identical ciphertext to run 1 | no |
| 4 | new (relaunch) + reconnect | same | same sizes, new ciphertext | yes |
| 5 | reconnect | 1 × 1 in square (confirmed) | 64/32/32/32 + 336 + 32 | no |
| 6 | new (relaunch) | two 1 × 1 in squares (operator-confirmed) | 64/32/32/32 + 512 + 160 + 32 | no |
| 7 | new (dev addr 12) | one 1 in circle | 64/32/32/32 + 512 + 512 + 272 + 32 | no (stopped mid-cut) |
| 8 | same session as 7 | one 3 in square | 64/32/32/32 + 336 + 32 (same ciphertext as 7's open/close) | circle's end captured; square mid-cut |
| 9 | new (restart, 2 handshakes) | one 5-point star | 64/32/32/32 + 512 + 352 + 32 | no (stopped mid-cut) |
| 10 | same session as 9 | one 1 in square (full-cut attempt) | 64/32/32/32 + 336 + 32 | no — STOP made ~33 s after Go, mid-cut |

Run 10 again stopped mid-cut: the manual STOP keeps arriving before the cut finishes. To finally capture a cut to
mat unload, run the capture for a fixed generous duration (e.g. 4 min) instead of a hand-timed STOP, or verify
Design Space shows "finished" and the mat is out before creating STOP.

Run 9 measured bytes-per-node: a 5-point star (10 nodes) is 864 B, so job size ≈ 84 B × node count (near-linear,
~0 overhead). A clean restart gave a fresh handshake and key. Still open: a capture kept running until after the mat
is unloaded (every run so far has stopped mid-cut), and whether the per-node cost changes with path type (line vs curve).
