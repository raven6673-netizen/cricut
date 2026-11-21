# Task for Codex — Run 8: one 3 in square

**Status:** Claude has a USBPcap capture running on the Windows machine (device address 12).
It stops when a `STOP` file appears in the capture folder or in Codex's workspace.

## What to cut
One **3 in square** on the Cricut Maker, same settings as the earlier runs:
- Material: **Vinyl**, Pressure: **Default**, Mirror: **Off**, Mat: **12 × 12 in**

**Keep the capture running until after the mat is unloaded** — earlier runs stopped mid-cut and we
still have no capture of a cut's end. Only create `STOP` once the mat is out.

## Why a 3 in square
A 1 in square = 336 B of design data; a 1 in circle = 1296 B. The circle showed size is driven by
path segments. A 3 in square has the **same 4 corners** as the 1 in square. If it is still ~336 B, the
byte count depends on node count only, not physical size. If it is larger, size matters too.

## Logging
In your workspace `codex-notes.md`, add `## Run 8` with local times for: Design Space connected,
Go pressed, cut finished, mat unloaded. Then create `STOP` in your workspace.

## Known blocks (from earlier runs)
- COM6 / serial "Access denied" is the sandbox, not the port — request unsandboxed approval for only
  that one command; send nothing to the machine until the operator approves.
- If Design Space won't take your input, say so; the operator sets up the square and presses Go.
