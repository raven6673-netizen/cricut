# Windows capture baseline for Linux comparisons

The Cricut Maker appeared as USB VID:PID `20d3:0015` during Windows testing. Use the same design, material, pressure, mirror setting, and mat when comparing a Linux run with this baseline.

| Windows run | Design | Observed larger outbound payloads near the cut |
| --- | --- | --- |
| 5 | One 1 × 1 in square | 64, 32, 32, **336**, 32 bytes |
| 6 | Two 1 × 1 in squares, confirmed by the operator | 64, 32, 32, **512 + 160**, 32 bytes |
| 7 | One 1 in circle | 64, 32, 32, **512 + 512 + 272**, 32 bytes |
| 8 | One 3 in square | 64, 32, 32, **336**, 32 bytes |
| 9 | One 5-point star | 64, 32, 32, **512 + 352**, 32 bytes |

The highlighted payloads total 336 bytes (a 1 in OR 3 in square — same 4 corners), 672 bytes (two squares, 8 corners) and 1296 bytes (one circle, a polygonised curve). Sizes exclude the two-byte framing header. The byte count tracks node/segment count, not physical size or shape count, at roughly 84 bytes per node (near-linear, ~0 fixed overhead): the 5-point star's 864 bytes is ~10 nodes. Frames are capped at 512 bytes. The payload contents remain unverified (encrypted). Each run included a connection setup exchange before the cut. Exact Go and unload times were unavailable.

For a Linux comparison, record which transport was used (USB or Bluetooth), the Design Space and Wine runner versions, and the local timestamps for Go, cut completion, and mat unload. Keep the capture running until after the mat is unloaded.

The original Windows packet captures and detailed notes are in `Downloads/cricut-capture` on the Windows machine.
The capture and decoding tools, the full protocol findings and the run log are in [`windows-capture/`](windows-capture/README.md).
