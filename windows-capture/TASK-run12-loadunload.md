# Task for Codex / operator — Run 12: identify load, unload and Done signals

**Goal:** map each step of the Design Space cut flow to its USB traffic. The capture auto-stops after
**5 minutes**, so there is no `STOP` to time — just walk the whole flow inside the window.

The known flow (from the operator) is:
1. Design Space prompts to press the Maker's **load/unload** button → mat loads in.
2. Design Space prompts to press **Go** → the cut runs.
3. After the cut, Design Space prompts to press **load/unload** again → mat unloads.
4. Design Space shows a **Done** button → user clicks it to finish.

## What to cut
One **1 in square** (Vinyl, Default pressure, Mirror Off, 12 × 12 in mat). The shape doesn't matter; the
button sequence does.

## The important part — record the exact second of each press
This is the whole point of the run. In your workspace `codex-notes.md`, add `## Run 12` and note the local
clock time (to the second) of each of these, as they happen:
- load/unload pressed to **load** the mat
- **Go** pressed
- cut **finished** (Design Space says so)
- load/unload pressed to **unload** the mat
- mat fully **out**
- **Done** clicked

Leave a few seconds between steps where the flow allows, so each event is easy to separate in the traffic.

## Notes
- Do the flow promptly after the capture starts so all of it fits in the 5-minute window.
- COM6 "Access denied" is the sandbox — request unsandboxed approval for only that command. If Design Space
  won't take input, the operator does the flow; Claude maps the traffic to the timestamps either way.
