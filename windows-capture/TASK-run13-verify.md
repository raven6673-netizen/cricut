# Task for Codex / operator — Run 13: verify load + state-poll on a FRESH connection

**Goal:** confirm run 12's findings hold on a new session key — the mat-load command burst, and the status poll
changing ciphertext between "cutting" and "done". Capture auto-stops after **5 minutes**; no `STOP` to time.

## Steps
1. **Fully restart Design Space first** (so this is a new connection with a new key):
   - Quit Design Space; make sure `CricutDevice` and `CricutDeviceIO` have exited (end them in Task Manager
     if they linger).
   - Reopen Design Space and let it connect to the Maker.
2. Walk the **full flow** for one **1 in square** (Vinyl, Default pressure, Mirror Off, 12 × 12 in mat):
   load/unload to **load** → **Go** → let it cut → load/unload to **unload** → click **Done**.
3. Do it all inside the 5-minute window.

## Log — exact second of each, in your `codex-notes.md` under `## Run 13`
- Design Space closed, reopened, connected
- load/unload pressed to load
- Go pressed
- cut finished (Design Space "Unload mat")
- load/unload pressed to unload
- mat out
- Done clicked

Then push `codex-notes.md` (you did this well for run 12 — same again). Claude will pull it and align the traffic.

## Known blocks
- COM6 "Access denied" is the sandbox — request unsandboxed approval for only that command. If Design Space won't
  take input, the operator runs the flow; Claude decodes either way.
