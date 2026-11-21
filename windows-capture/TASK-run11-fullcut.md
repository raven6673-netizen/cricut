# Task for Codex — Run 11: 1 in square, timed capture to mat unload

**You (Codex) and Claude are on the same Windows machine.** Drive the native Windows **Cricut Design Space**
over USB (COM6). Claude runs the capture; you drive Design Space and log times.

**This run the capture auto-stops after 4 minutes**, so you do NOT need to time a `STOP`. Just run the
whole thing — cut and unload — inside that window.

## Steps
1. Cut one **1 in square** (Vinyl, Default pressure, Mirror Off, 12 × 12 in mat).
2. Let it cut to completion (Design Space says finished).
3. **Unload the mat** (load/unload button, take the mat out).
4. That's it — the capture closes itself. No `STOP` needed (though one still ends it early if you want).

Do the cut promptly after the capture starts so all of it — Go, cut, finish, unload — fits in the 4 minutes.

## Log
In your workspace `codex-notes.md`, add `## Run 11` with local times for: Go pressed, cut finished,
mat unload pressed, mat removed. These let Claude line the unload traffic up with what happened.

## Known blocks
- COM6 "Access denied" is the sandbox — request unsandboxed approval for only that command.
- If Design Space won't take your input, the operator runs the cut and unload; Claude still decodes it.
