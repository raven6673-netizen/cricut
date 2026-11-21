# Task for Codex — Run 10: one 1 in square, full cut to mat unload

**You (Codex) and Claude are on the same Windows machine.** Drive the native Windows **Cricut Design Space**
app over USB (COM6). Claude runs the capture and decodes it; you drive Design Space and log times.

**Goal of this run:** capture one cut **end to end** — every run so far stopped while the Maker was still
cutting, so we have no capture of a job from start to mat unload. This run is about completeness, not a new
shape.

## What to cut
One **1 in square** (our baseline), same settings:
- Material: **Vinyl**, Pressure: **Default**, Mirror: **Off**, Mat: **12 × 12 in**

## The important part — DO NOT stop early
1. Press **Go** and let the cut run.
2. Wait for Design Space to say the cut is **finished**.
3. **Unload the mat** (press the load/unload button on the Maker and take the mat out).
4. **Only then** create the `STOP` file. Give it a few extra seconds after unload.

If you stop at "cut finished" without unloading, we miss the unload traffic again.

## Log (this run's timestamps matter most)
In your workspace `codex-notes.md`, add `## Run 10` with local times for:
- Go pressed
- cut finished (Design Space says done)
- mat unload button pressed
- mat removed
- STOP created

Create `STOP` in your workspace (`C:\Users\igpx1\Documents\Codex\2026-11-20\new-chat-5`) or the capture
folder (`C:\Users\igpx1\Downloads\cricut-capture`).

## Known blocks
- COM6 "Access denied" is the sandbox — request unsandboxed approval for only that command.
- If Design Space won't take your input, the operator runs the cut and unload; Claude still decodes it.
