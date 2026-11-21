# Task for Codex — Run 7: one circle

**Status:** Claude has a USBPcap capture running on the Windows machine (device address 12,
file `cricut-20261121-111818.pcap`). It stops when a `STOP` file appears in the capture folder
or in Codex's workspace.

## What to do in Cricut Design Space
Start Cricut Design Space and cut **one 1 in circle** on the Cricut Maker, using the same settings
as the earlier square runs:
- Material: **Vinyl**
- Pressure: **Default**
- Mirror: **Off**
- Mat: **12 × 12 in**

Keep the capture running until **after the mat is unloaded**.

## Logging
In your workspace `codex-notes.md`, add a `## Run 7` section with local timestamps for:
- Design Space started / connected to the Maker
- Go pressed
- cut finished
- mat unloaded

Then create an empty `STOP` file in your workspace to end the capture.

## If something blocks you (known from earlier runs)
- **COM6 / serial "Access denied"** — this is the sandbox, not the port. COM6 opens fine outside it.
  Request **unsandboxed/escalated approval for only the one command** that opens the port; don't send
  anything to the machine until the operator approves. See `claude-notes.md` in the Windows capture
  folder, section "COM6 Access denied".
- **Keyboard input into Design Space rejected** (seen in run 6) — if you cannot drive the app, say so,
  and the operator will set up the circle and press Go by hand. The capture records the traffic either way.

## Goal of this run
One square sent 336 bytes of design data; two squares sent 672. This circle tests whether a **curve**
changes the per-shape size — i.e. whether geometry is encoded per shape or per path segment. No keys are
being extracted; this maps the job format only.
