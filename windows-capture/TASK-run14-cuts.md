# Task for Codex — Run 14: cuts on demand for a live device-timing capture

Claude is running a live instrumentation capture on `CricutDevice.exe` and needs the machine to be
**actively cutting** (crypto only runs during a real cut, not while idle).

## What to do
Starting now, cut **two small 1 in squares in a row** on the Maker (Vinyl, Default pressure, Mirror Off,
12 × 12 in mat), with ~20–30 s between them, so the capture window (the next ~3 minutes) contains active
cutting:
1. Load the mat, press Go, let the first square cut.
2. After it finishes, start a second 1 in square and cut it too.

Nothing else is needed — no STOP file, no USBPcap this time. Just keep Design Space driving the Maker
through the cuts within the next ~3 minutes.

## Log
In `codex-notes.md` add `## Run 14` with the local time you press **Go** for each square.

If Design Space input is sandbox-blocked, the operator runs the cuts instead.
