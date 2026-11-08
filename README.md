# cricut-linux

Run **Cricut Design Space** on **Bazzite** (Fedora Atomic) and cut with a **Cricut Maker** over **USB or Bluetooth**.

Cricut does not ship Design Space for Linux, and the Maker only accepts jobs from Design Space. This project runs the Windows
app in [Bottles](https://usebottles.com) (Flatpak) and exposes the Maker to Wine as a COM port:

- **USB:** a udev rule creates a stable `/dev/cricut` and grants your desktop user access (via `uaccess`, so you don't have to fiddle with the `dialout` group on an immutable OS). The device is then linked into the bottle as `COM3`.
- **Bluetooth:** a small user-space bridge (`tools/cricut-bt-bridge.py`) connects to the Maker's RFCOMM serial channel and presents it as a pty, mapped as `COM4`. No root, and no deprecated `rfcomm` tool or `rpm-ostree` layering.

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
2. **Install Design Space.** Download the Windows installer from design.cricut.com, then:
   ```bash
   cricut-linux install ~/Downloads/CricutDesignSpace-Install-*.exe
   ```
3. **USB.** Plug in and power on the Maker:
   ```bash
   cricut-linux detect              # note the VID:PID of the Maker's line
   cricut-linux udev 1234:abcd      # use your VID:PID; asks for sudo
   cricut-linux map-port            # /dev/cricut -> COM3
   ```
4. **Bluetooth (optional).**
   ```bash
   bluetoothctl scan on             # find the Maker's MAC
   cricut-linux bt-pair AA:BB:CC:DD:EE:FF
   cricut-linux bt-bridge AA:BB:CC:DD:EE:FF   # keep running while cutting
   ```
5. **Cut.**
   ```bash
   cricut-linux run
   ```

Run `cricut-linux doctor` at any point to see what is missing.

## Status / caveats

- These scripts have not been tested against real hardware yet. Please report results (`cricut-linux doctor` output helps).
- How well Design Space runs under Wine depends on the Design Space and Wine/runner versions. If sign-in or the canvas misbehaves, try another runner in Bottles (e.g. a newer `soda`/`caffe`/`kron4ek` build).
- A fully native Linux cutter for the Maker is not possible yet. The Maker's protocol is encrypted and undocumented, so native support would first need protocol research.

## Layout

```
bin/cricut-linux             main CLI (doctor, detect, udev, install, map-port, run, bt-*)
tools/cricut-bt-bridge.py    Bluetooth RFCOMM -> pty bridge
```
