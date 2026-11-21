# Cricut Maker CXPL301 code map

Target observed on Linux at `/dev/ttyACM1`, USB VID:PID `20d3:0015`, Q-prefix unit. This maps MatCraft and the recovered Windows host code to the machine’s wire interface. Firmware handler addresses require a matching stock image.

## Control path

```text
MatCraft app.rs                                MatCraft machine.rs
machine_window → cut_on_machine → Request::Cut → prep → setup → draw
                    ↓                                           ↓
          machine_cut_records                            USB serial port
                    ↓                                           ↓
          contours_to_records                      Maker command dispatcher
                                                      ↓             ↓
                                               setup state    motion queue
                                                      └──────┬──────┘
                                                        tool/Z control
```

The last three machine-side stages are investigation targets inferred from the command stream and observed motion. Their implementation and boundaries remain unknown.

| Layer | MatCraft entry point | Behavior |
| --- | --- | --- |
| USB discovery and transport | `machine.rs:248` `find_usb_machines`, `:279` `open_usb` | Discovers the Cricut USB vendor and opens the serial port at 115200 baud with DTR/RTS. |
| Framing | `machine.rs:122` `frame_plain`, `:1076` `read_reply` | Two-byte big-endian header. Bit 15 signals encryption, low 15 bits give body length. The observed session uses plain frames. |
| Connect | `machine.rs:647` `confirm_connection` | Requires an interface-state reply before reporting a connection. |
| Force conversion | `machine.rs:886` `Request::ReadCalibration`, `:1299` `parse_persistent_pressure`, `:1313` `grams_to_counts` | Reads machine calibration, then interpolates and clamps grams to Z encoder counts. |
| Job construction | `app.rs:2345` `machine_cut_records`, `:2358` `cut_on_machine`; `machine.rs:1125` `contours_to_records` | Creates travel and line records from mat geometry and queues a cut. |
| Streaming | `machine.rs:747` `Request::Cut`, `:1342` `setup_payloads`, `:1363` `cut_job_payloads`, `:1381` `draw_ack` | Sends prep, setup, and draw commands in order. Waits for replies and retries on leading `01` or timeout within a deadline. The last reply byte is logged as an estimate of buffer space even for setup replies that echo parameters. |

## Command dispatch landmarks

Wire values below are hexadecimal. Command bodies exclude the two-byte frame header. Addresses are from the recovered x86 `CricutDevice.exe` and refer to host functions.

| Command | Role and fields | Host trace | Evidence from this Maker |
| --- | --- | --- | --- |
| `73` | Interface state | `0x4a64c0`, `0x4a1240` | Live frame `00 06 73 D3 0F 04 03 00`. |
| `12` | Firmware check | `0x471500`; response parser `0x4e1e00` | Standalone query returned `00 08 01 12 45 09 FF 49 01 00`. The prefix matches the observed NACK convention. No version is assigned from this reply. |
| `A0 <index:u16 BE>` | Persistent read. Drag calibration slots `0021–0030`. Reply offsets 7–8 are grams and 9–10 are counts, both BE. | `0x4d6ba0`, `0x4d6f40`, `0x4d71f0` | Recorded drag table spans 100–434 g and 845–1541 counts. |
| `B9` → `F0 08` → `EA` | Position query, theta homing, and tool detection. MatCraft can reposition Y before homing. | `0x48dd50`, `0x48dc20`, `0x4a7280` | Position reply recorded, homing acknowledged, tool code `02` with four zero sample bytes. |
| `22 00 00 00` | EndDraw, used before setup and after the final batch. | `0x472b50`, `0x472e60` | Both instances acknowledged. |
| `30` | SetMachineState. Drag profile begins `30 08 00 00`, followed by width and height, five zero bytes, and a BE pressure count. | `0x48e6e0`, `0x4f8de0` | Echoed 12 × 12 and `040F` (1,039 counts). |
| `31` → `37` → `3E` | Speed profile, tool state, full tool state. | Setup chain `0x48e6e0` | Speed acknowledged, tool settings echoed. |
| `A1 <index:u16 BE> <value:u32 BE>` | Four host-gated persistent writes at indices `005D`, `005F`, `0082`, `0084`. | `0x48edc0` | All four echoed. Firmware meanings of the slots remain unknown. |
| `3F 00` | CPR toggle for drag profile. | `0x48e6e0`, `0x4fe170` | Echoed. |
| `21 00 00 00` | BeginDraw. | `0x472740` | Acknowledged. |
| `43 <records>` | MultiDrawVector. Each record is kind, X and Y as little-endian float32 inches. `02` travels and `00` cuts. MatCraft uses batches of up to 56; a closed square emits `02 00 00 00 00 02`. | `0x4df770`, `0x4ddc20`, `0x4ed2d0` | Both batches in the 99-record hardware job acknowledged. The blade still bobbed and failed to reach the material. |

## Firmware work

1. Identify the firmware build and obtain a matching stock image. The Q prefix makes PIC32MX470F512L a candidate, with board revision still unknown. A rejected standalone version query leaves the installed build unresolved.
2. Find the USB dispatch table and match handlers using field layouts, starting with the `30` work-area and pressure fields and the `43` nine-byte record stride.
3. Follow the pressure value from the `30` handler to the active tool setpoint. Follow the `43` kind byte from the queue into the motion loop. Check whether consecutive `00` records preserve Z engagement and what event releases it.
4. Compare startup and readiness gates if those paths lead to a disabled or reset tool state. The host notes describe a separate connect-time sequence that MatCraft currently omits. An echoed setpoint and acknowledged vectors establish receipt, while physical downforce remains unverified.

Sources include MatCraft `app/src/machine.rs` and `app/src/app.rs`, the project’s `app/CUT-INIT.md` and `app/CUT-PROTOCOL.md`, the saved machine log, the live USB replies, and the [CutcutGo board inventory](https://virtualabs.github.io/cutcutgo/setup.html).
