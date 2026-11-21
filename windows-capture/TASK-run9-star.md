# Task for Codex — Run 9: one 5-point star (fresh connection)

**You (Codex) and Claude are on the same Windows machine.** Drive the native Windows **Cricut Design Space**
app directly (not Wine) and cut on the Maker over USB (COM6). Claude runs the capture and decodes it; you
drive Design Space and log times.

## 1. Restart Design Space first (fresh connection)
Before cutting, **fully close Cricut Design Space and reopen it**, so this run starts a new USB
connection and a new key:
1. Quit Design Space. Make sure the helper processes `CricutDevice` and `CricutDeviceIO` have exited
   (end them in Task Manager if they linger) — they hold the USB session.
2. Reopen Design Space and let it connect to the Maker.

This gives a fresh plaintext handshake (`cf …` nonce, `c7 …` key exchange) and a new session key, which
Claude needs to see the setup as well as the cut.

## 2. Cut
One **5-point star** on the Maker, same settings as the earlier runs:
- Material: **Vinyl**, Pressure: **Default**, Mirror: **Off**, Mat: **12 × 12 in**

**Keep the capture running until after the mat is unloaded.** Only create `STOP` once the mat is out —
we still want a square/star cut recorded all the way to the end.

## 3. Log
In your workspace `codex-notes.md`, add `## Run 9` with local times for: Design Space closed, reopened,
connected, Go pressed, cut finished, mat unloaded. Then create `STOP` in your workspace
(`C:\Users\igpx1\Documents\Codex\2026-11-20\new-chat-5`) or the capture folder
(`C:\Users\igpx1\Downloads\cricut-capture`).

## Why a star
A square is 336 B for 4 corners. A 5-point star has 10 points (nodes). This measures bytes-per-node
directly and checks whether the size grows linearly with node count. No keys are extracted — this maps
the job format only.

## Known blocks (from earlier runs)
- COM6 / serial "Access denied" is the sandbox, not the port — request unsandboxed approval for only that
  one command; send nothing to the machine until the operator approves.
- If Design Space won't take your input, say so; the operator restarts it, sets up the star and presses Go.
