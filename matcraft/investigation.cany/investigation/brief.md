# Brief

**Question:** How does the desktop design app talk to a cutting machine (over Bluetooth serial at 00:06:66:8d:2b:fe), and what core features (canvas, shapes/text/images, mat preview, materials, cut) must a Linux-friendly rebuild provide?

**Purpose:** Interoperability: build the person's own design-and-cut app that runs on Linux (Bazzite) and drives their own Maker machine.

**Targets:**
- C:\Users\igpx1\AppData\Local\Programs\Cricut Design Space\Cricut Design Space.exe

**Constraints:**
- Rust + eframe/egui
- No brand names or logos
- Change files only in app/

**Done when:** app/ builds, supports designing on a mat and sending to the machine over a Bluetooth RFCOMM/serial link with the documented protocol facts, plus --screenshot.
