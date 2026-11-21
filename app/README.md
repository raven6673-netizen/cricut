# MatCraft

MatCraft is a Rust desktop app for the original Cricut Maker (CXPL301). This directory carries the Linux-developed source and protocol notes to Windows.

## Build and run on Windows

Install stable Rust with the default MSVC toolchain and Visual Studio Build Tools with the Desktop development with C++ workload and Windows SDK. From PowerShell:

```powershell
git clone -b matcraft-windows-handoff https://github.com/raven6673-netizen/cricut.git
cd cricut\app
cargo test --locked
cargo run --locked -- --cut-dry-run
cargo run --locked
```

The dry run prints the square's frames without opening a device. For USB, close Design Space, connect and power on the Maker, and find its COM port in Device Manager under **Ports (COM & LPT)**. In MatCraft's Machine panel choose USB, select the port and click **Connect**. To run the status probe from PowerShell, substitute the actual port for `COM6`:

```powershell
cargo run --locked -- --usb-probe COM6
```

Linux uses the same Cargo commands from this directory, with the Linux serial device in place of the COM port.

## Handoff status

On Linux the Maker replied to status queries, reported a drag blade, echoed calibrated pressure, and accepted the cut command stream. The carriage followed the path, but the blade bobbed and did not reach the material. The standalone firmware version query returned a rejection. The installed firmware version and stock image are unresolved.

See the [Maker code map](MAKER-CODE-MAP.md), [cut protocol](CUT-PROTOCOL.md), and [initialization trace](CUT-INIT.md) to resume the investigation.
