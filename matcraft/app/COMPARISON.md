# The original design app compared with MatCraft

Based on the original's files: the pages in its packed web app (sign-in, home, design canvas,
tabs, about, performance monitor, monogram maker), its helper programs (machine, device I/O,
canvas, image tracing, path operations, computer fonts, printer, offline data) and its decoded
machine program (see MECHANICS.md). The original wasn't run, so its look isn't compared
picture-for-picture. The same table is in the app under **Compare** in the left tool bar.

Totals: 34 the same, 14 partly, 9 not yet, 6 only in MatCraft (63 features).

## Adding and editing designs

| Feature | Original app | MatCraft | |
|---|---|---|---|
| Basic shapes | Square, circle, triangle, pentagon, hexagon, octagon, star, heart, score line | The same nine, plus diamond, rectangle, rounded square, oval, plus, half circle, shield | Only here |
| Image and project library | Large online library | None (offline) | Not yet |
| Monogram maker | Built-in plugin | Not yet | Not yet |
| Save / open projects | Cloud and offline | Files on your computer, pictures included | Partly |
| Canvas, grid, rulers, zoom | Inches or cm | Inches; scroll moves, Ctrl+scroll zooms, Fit | Partly |
| Move, resize, rotate, lock | Corner handles + numbers | Same corner handles + numbers | Same |
| Select several | Shift-click, box, select all | Shift-click, box, select all; transform together | Same |
| Cut, copy, paste, duplicate, delete | Edit menu + keys | Edit menu + keys | Same |
| Undo / redo | Yes | Yes | Same |
| Align and distribute | 7 aligns + distribute | Same | Same |
| Arrange | Back, backward, forward, front | Same | Same |
| Flip | Horizontal, vertical | Same | Same |
| Group / ungroup | Yes | Yes | Same |
| Weld | Yes | Yes | Same |
| Combine (unite, subtract, intersect, exclude) | Yes | Yes | Same |
| Slice | Yes | Yes | Same |
| Attach / detach | Yes | Yes | Same |
| Flatten / unflatten | One printed image | Layers become print then cut, attached | Partly |
| Offset | Rounded or sharp corners | Live preview, rounded corners only | Partly |
| Contour | Hide/show inner shapes | Same, plus "hide all inner" | Same |
| Operations | Cut, draw, score, engrave, deboss, wave, perforate, print then cut, guide | Same nine | Same |
| Colour sync | Panel | Colour per layer only | Partly |
| Pattern fills, warp | Yes | Not yet | Not yet |

## Home, projects and settings

| Feature | Original app | MatCraft | |
|---|---|---|---|
| Home page | Project-type shortcuts, search, recent projects, online ideas | Same shortcuts, search and recent projects; no online ideas | Partly |
| Choose what to make | Project type, template, size, what you'll need | Same, plus custom size; design area shown on the canvas | Same |
| My Stuff | Cloud and computer projects | Computer projects, search | Partly |
| Settings | General, Content, Machines, Canvas, Load Type, Notifications, System | All but Content and Notifications (nothing online) | Partly |
| Canvas grid, units, snap | Full/outline/no grid, inches or cm, snap | Same, with pink snap guides (hold Alt to turn off) | Same |

## Engraving

Read from the original's own engraving fill plugin and engrave project settings (the help
article itself was behind a bot check and couldn't be read).

| Feature | Original app | MatCraft | |
|---|---|---|---|
| Engrave outline | Along the edge | Along the edge | Same |
| Engrave fill | 7 line patterns; spacing, edge margin, fill rule | Same 7, same starting values (0.01 in, non-zero), live on the canvas | Same |
| Engrave a photo | Yes | Dotted lines with brightness and invert | Partly |
| Engrave Tip / Engraving Tool, 11.5 x 11.5 in area | Yes | Yes, tool from your machine model; before-you-engrave card | Same |

## Text

| Feature | Original app | MatCraft | |
|---|---|---|---|
| Add and edit text | Type on the canvas | Type on the canvas (double-click text); letters update live | Same |
| Select part of the text with the mouse | Yes | Not yet (cursor only) | Not yet |
| Fonts | Own library + computer fonts | Computer fonts + built-in, searchable | Partly |
| Bold, italic | Yes | Yes (made thicker/slanted when the font lacks them) | Same |
| Letter / line spacing, alignment | Yes | Yes | Same |
| Curve | Yes | Yes | Same |
| Ungroup | To letters or lines | To letters | Partly |
| Writing fonts for pens | Yes | Outline fonts only | Not yet |

## Images

| Feature | Original app | MatCraft | |
|---|---|---|---|
| Choose files / drag and drop | Yes | System file picker (several at once) and drag and drop | Same |
| Convert pictures to SVG | No separate tool | SVG converter, Save as SVG, export design as SVG | Only here |
| SVG, DXF | Yes | Yes, at their real size | Same |
| PNG, JPG, BMP, GIF, WebP | Yes | Yes | Same |
| Remove background | Eraser + automatic remover | Background colour or darkness, live preview | Partly |
| Cut image (trace) | Yes | Yes, with smoothing | Same |
| Print then cut image | Yes | Yes, cut line around the picture | Same |

## Making and the machine

| Feature | Original app | MatCraft | |
|---|---|---|---|
| Mats by colour | Yes | Yes; attached layers stay together | Same |
| Mat sizes, mirror | Yes | No mat, 12x12, 12x24, card mat; page sizes; mirror | Same |
| Move pieces on a mat by hand | Yes | Automatic layout only | Not yet |
| Materials | Hundreds, downloaded, plus custom | 17 (4 for engraving) plus your own, estimated | Partly |
| Print then cut | Prints with sensor marks, cuts | Print mat saved as SVG; marks shown, not printed; cutting not yet | Partly |
| Save mats as files | No | SVG per mat | Only here |
| Bluetooth to 00:06:66:8D:2B:FE | Through its helper | Straight from the app (Linux) | Same |
| Message format, service commands | Yes | Same, decoded | Same |
| Encrypted session | Automatic | Request only; key pasted in | Partly |
| USB | Yes | Found automatically by USB id; tested on the real machine | Same |
| Actually cutting a design | Yes | Not yet | Not yet |
| Firmware, calibration | Yes | No (so no bricking "updates" either) | Not yet |
| Activation / deactivation lock | Serial checked against a disabled list in the software | None: works with any machine on the link | Only here |
| Project tabs | Yes | One project at a time | Not yet |
| Computers, account | Windows/macOS, account | Linux + Windows, no account | Only here |
