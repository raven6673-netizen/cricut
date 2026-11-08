# cricut-linux

Run **Cricut Design Space** on **Bazzite** (Fedora Atomic) and cut with a **Cricut Maker** over **USB or Bluetooth**.

Cricut does not ship Design Space for Linux, and the Maker only accepts jobs from Design Space. This project runs the Windows
app in [Bottles](https://usebottles.com) (Flatpak) and exposes the Maker to Wine as a COM port:

- **USB:** a udev rule creates a stable `/dev/cricut` and grants your desktop user access (via `uaccess`, so you don't have to fiddle with the `dialout` group on an immutable OS). The device is then linked into the bottle as `COM3`.
- **Bluetooth:** a small user-space bridge (`tools/cricut-bridge.py`) connects to the Maker's RFCOMM serial channel and presents it as a pty, mapped as `COM4`. No root, and no deprecated `rfcomm` tool or `rpm-ostree` layering.

Nothing is layered onto the OS image. The only system change is one file in `/etc/udev/rules.d`.

## Setup

```bash
git clone https://github.com/raven6673-netizen/cricut ~/cricut
ln -s ~/cricut/bin/cricut-linux ~/.local/bin/cricut-linux
cricut-linux doctor
```

1. **Bottles + device access**
   ```bash
   cricut-linux install-bottles
   ```
2. **Install Design Space.** This downloads the latest Windows installer straight from Cricut, checks its SHA-512 against Cricut's published hash, and runs it in a new bottle:
   ```bash
   cricut-linux install
   ```
   (`cricut-linux download` fetches the installer on its own; `cricut-linux install /path/to/installer.exe` uses one you already have.)
3. **USB.** Plug in and power on the Maker:
   ```bash
   cricut-linux detect              # finds the Maker by its USB manufacturer string
   cricut-linux udev                # rule for the detected Maker (or: udev VID:PID); asks for sudo
   cricut-linux map-port            # /dev/cricut -> COM3
   ```
4. **Bluetooth (optional).**
   ```bash
   cricut-linux bt-find             # finds "Maker-XXXX" (the name printed on the label underneath)
   cricut-linux bt-pair AA:BB:CC:DD:EE:FF
   cricut-linux bt-bridge AA:BB:CC:DD:EE:FF   # keep running while cutting
   ```
5. **Cut.**
   ```bash
   cricut-linux run
   ```

Run `cricut-linux doctor` at any point to see what is missing.

## Towards a native Linux cutter

To replace Design Space completely, someone first has to work out how the Maker's protocol works. `tap` puts a logging
bridge between Design Space and the USB port. It records every byte in both directions while you cut normally:

```bash
cricut-linux tap                 # COM3 -> logging bridge -> /dev/cricut; leave running
cricut-linux run                 # in another terminal: cut something simple (e.g. a square)
cricut-linux dump ~/.local/share/cricut-linux/captures/<file>.jsonl
cricut-linux map-port            # afterwards: point COM3 straight at the Maker again
```

Captures of simple, varied jobs (square vs. circle, different materials and pressures) help with the analysis.
Captures may include account or machine identifiers, so review them before sharing.

## Machine notes

The original Cricut Maker (model **CXPL301**) has a Roving Networks RN42 Bluetooth module (FCC ID `T9J-RN42`).
It provides a plain serial-over-Bluetooth (SPP) port on RFCOMM channel 1, which is what `bt-bridge` uses.
Its Bluetooth name (`Maker-XXXX`, printed on the label) normally ends in the last four hex digits of its MAC address.

## Status / caveats

- Tested so far on Bazzite (NVIDIA): the Maker is detected over USB (`20d3:0015`, `/dev/ttyACM0`) and Design Space installs and its process starts. Sign-in and cutting are not confirmed yet. Please report results (`cricut-linux doctor` output helps).
- If Design Space shows no window, run `cricut-linux stop` and then `cricut-linux run`. It renders in software by default because Chromium's GPU process fails under Wine; `run --gpu` tries hardware rendering instead.
- How well Design Space runs under Wine depends on the Design Space and Wine/runner versions. If sign-in or the canvas misbehaves, try another runner in Bottles (e.g. a newer `soda`/`caffe`/`kron4ek` build).
- A fully native Linux cutter for the Maker is not possible yet. The Maker's protocol is encrypted and undocumented, so native support would first need protocol research.

## Layout

```
bin/cricut-linux             main CLI (doctor, detect, udev, install, map-port, run, bt-*, tap, dump)
tools/cricut-bridge.py       USB-serial / Bluetooth RFCOMM -> pty bridge, with optional traffic log
tools/cricut-log-dump.py     hex/ASCII viewer for traffic logs
```
