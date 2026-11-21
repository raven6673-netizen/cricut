For the original Cricut Maker (`Athena`), **drag-blade calibration is the 16 A0 entries at indices `0x0021`–`0x0030` inclusive**. In each reply body, bytes **7–8** give force in grams and bytes **9–10** give Z encoder counts, both unsigned big-endian. Design Space interpolates between those points, truncates to an integer, and writes the result to the final two bytes of `0x30`. The actual count for 180 g depends on the machine’s table.

## 1. Persistent-memory commands

Offsets below start at the opcode, after the transport length prefix.

| Command | Request body | Successful reply body | Confidence and evidence |
|---|---|---|---|
| Read Persistent Memory `A0` | `A0 <index:u16 BE>` | `A0 <echoed_index:u16 BE> <word1:32 bits BE> <word2:32 bits BE>` | **Confirmed.** Constructor `0x452c80`, index setter `0x452e20`, builder `0x452e30`, parser `0x452d80`. The builder uses reverse-byte append `0x452a60`, and the parser uses reverse copy `0x879fc0`. |
| Get Persistent Value Count `A4` | `A4` | `A4 <count:i32 LE>` | **Confirmed.** Constructor `0x4feee0`, opcode getter `0x4fefc0`, builder `0x4ff200`, parser `0x4fefd0`. The parser removes the opcode and loads the next four bytes directly as a native x86 integer. |

**A0 reply layout, confirmed for the pressure-table indices**

| Body offset | Size | Type | Meaning |
|---|---:|---|---|
| `0x00` | 1 | `u8` | Reply opcode `A0`. |
| `0x01` | 2 | `u16 BE` | Echoed persistent-memory index. `0x452d80` only copies the two words when the index matches the request. |
| `0x03` | 4 | 32-bit BE word | First per-index value. The generic class exposes both raw and IEEE-754 `f32` getters (`0x452d60`, `0x452d40`). Its meaning for the pressure rows is **unknown**. The calibration loader leaves it unused. |
| `0x07` | 2 | `u16 BE` | Force in grams, the high half of `word2`. |
| `0x09` | 2 | `u16 BE` | Z encoder counts, the low half of `word2`. |

The A0 parser requires at least 11 body bytes and reads the first 11. At vtable `0x95c294`, slot `+0x2c` resolves to `0x452d30`, which returns raw `word2`. That is the getter `0x4d71f0` calls before splitting the word. The class also exposes `word2` as IEEE-754 `f32` through `0x452d10` for persistent indices that store floats.

**Confirmed framing.** The Athena inner length is a two-byte BE body length (`0x48f050`). The first drag-blade request is `00 03 A0 00 21`. A minimal reply starts `00 0B A0 00 21`. The A4 request is `00 01 A4`. The supplied prior USB report records `00 05 A4 DE 00 00 00`, which decodes to **222 persistent values** on that Maker. `0x48e5b0` caches the count and other callers use it as an exclusive index bound (`0x4a1620`). The calibration loader uses fixed blocks independent of A4. Plain A0 acceptance on the user's firmware remains unverified. [Prior Maker USB report](</home/igpx12/Documents/Cricut Anything/app/MECHANICS.md:19>)

## 2. Table organization

**Confirmed for the original Maker.** Athena initializes the relevant model fields to `0x0b` (`0x48d5c0`). For that model, `0x4d6f40` calls `0x4d71f0` for these blocks in this order. Each successful block reads 16 consecutive indices.

| Load order | Host tool group | Inclusive A0 indices |
|---:|---:|---|
| 1 | `0` | `0x0001`–`0x0010` |
| 2 | `2` — drag blade | `0x0021`–`0x0030` |
| 3 | `1` | `0x0041`–`0x0050` |

That is **48 pressure rows across three blocks**. The group comes from the block the loader is reading. The cached row layout in `0x4d71f0` is:

| Row offset | Host type | Value |
|---|---|---|
| `+0x08` | 32-bit enum | Tool group supplied by the loader |
| `+0x0c` | 32-bit integer | A0 index |
| `+0x10` | `u32` | Zero-extended grams from reply offset `0x07` |
| `+0x14` | `u32` | Zero-extended count from reply offset `0x09` |
| `+0x18` | `f64` | Count converted to mm |
| `+0x20`, `+0x24` | 32-bit integer | Count plus and minus the count equivalent of 0.5 mm |
| `+0x28`, `+0x30` | `f64` | Those two adjusted positions in mm |

`0x4d6ba0` normalizes the requested tool and the cached row’s group with `0x46a620`. On Athena, tool type `2` stays `2` and selects the middle block. The loader appends rows in index order, and the lookup scans in that order. **Confirmed** for the scan order. **Likely** for the on-machine data contract that force increases with index within a valid block, because the lookup stops at the first force at or above the request. The host performs no sort in this path.

## 3. Grams to counts and material pressure

**Confirmed — `0x4d6ba0`.** This routine lazily loads the calibration when the cache is empty. For integer requested grams `g`, it scans matching rows and remembers the last one below `g`.

- An exact match returns that row’s count.
- Below the first row, it uses the first row’s count.
- Between adjacent rows `(g0, c0)` and `(g1, c1)`, it evaluates the following in single precision and truncates toward zero. The instruction at `0x4d6d77` is `cvttss2si`.
- Above the last row, it uses the last row’s count.

```text
t = (f32(g) - f32(g0)) / (f32(g1) - f32(g0))
count = trunc_toward_zero(t * (f32(c1) - f32(c0)) + f32(c0))
```

That is endpoint saturation in **force order**. A missing group returns zero. There is also a zero-result edge case: if an exact match or interpolation yields zero after a lower row was saved, the function returns the lower row’s count instead.

**Confirmed — `0x4ff680`.** On the grams branch, the caller selects the material profile and any applicable pressure variant. An explicit grams override takes precedence on that branch. Otherwise it tries the positive operation-specific profile field for operation `0x0f` or `0x10`, then the positive general profile field, with **180 g** as the fallback. A separate branch for a profile with a positive calibrated position converts mm directly to counts through `0x4dbb30`.

After the grams lookup, selector `1` adds `Δ` counts, selector `2` subtracts `Δ`, and other selector values leave the result alone. A material variant can change the effective selector before this step. For drag type `2`, `0x4db9b0` chooses the first scale argument and evaluates:

```text
Δ = trunc_toward_zero((0.5_f32 / 25.4_f32) * drag_counts_per_inch + 0.5_f32)
```

On original Athena, `0x471710` → `0x471790` reads the drag scale from **A0 index `0x00A3`, reply bytes `7..10` interpreted together as one BE IEEE-754 `f32`**. The other scale comes from `0x009B` on the normal two-scale path. `0x4db940` performs the inverse conversion `mm = counts / counts_per_inch * 25.4`. The scale is unnecessary for the neutral grams-to-counts arithmetic, though the original loader reads scales to populate its derived row fields. `0x4ff680` applies **no second clamp** after the ±0.5 mm adjustment.

## 4. Worked 180 g example and MatCraft recipe

1. At connect, cache the drag block by reading `A0 00 21`, `A0 00 22`, …, `A0 00 30`, one transaction per index. Check the opcode, length and echoed index before accepting each row. Read `A0 00 A3` as well if implementing the ±0.5 mm adjustment. To mirror the full original loader, read all three blocks in the order above.
2. For 180 g, scan the drag rows in index order. The two actual bracketing indices are discovered from **bytes `7..8`** of their replies. At those same indices, **bytes `9..10`** supply the two counts. An exact 180 g row supplies the count directly.
3. For an arithmetic example only, suppose `0x0027` returned `160 g → 1000 counts` and `0x0028` returned `200 g → 1080 counts`. Then `t = (180 − 160)/(200 − 160) = 0.5`, so the neutral result is `trunc(1000 + 0.5 × 80) = 1040`. The final `0x30` bytes would be `04 10`. These example rows are illustrative. The real indices and values depend on the A0 replies.
4. Pass the checked count to MatCraft’s existing `set_machine_state_body`. In Design Space, `0x48e6e0` takes the count output from `0x4ff680` and passes it to `0x4f8d70`, which stores it at command-object `+0x30`. `0x4f8990` labels that field “encoder counts.” The serializer at `0x4f8de0` reads its low 16 bits at instruction `0x4f8f1b` and emits them BE. **Confirmed**, including the object offset in disassembly. [MatCraft `machine.rs`](</home/igpx12/Documents/Cricut Anything/app/src/machine.rs:1080>)

## 5. UI bounds and what remains unknown

**Confirmed host behavior.** The traced conversion accepts integer grams and saturates at the first and last calibration rows. It has no fixed numerical drag-blade force ceiling or count ceiling apart from the wire field width. The material caller can subsequently go outside the table’s count span with the ±0.5 mm adjustment.

For MatCraft, use the **machine’s calibrated interpolation interval** as the neutral UI bound. With 16 valid, ascending-force rows, the gram endpoints come from **`0x0021` and `0x0030`, bytes `7..8`**. With increasing counts, the corresponding count endpoints come from **those same replies, bytes `9..10`**. Compute the count span across all accepted rows if the counts vary non-monotonically. Validate the table before enabling pressure, and if the UI is intended to stay within that span, reject adjusted counts outside it before serialization. That last guard is a MatCraft choice.

The supplied evidence establishes the **host’s calibration envelope**. Firmware stability limits, the role of `word1` in pressure rows, and this Maker’s actual drag calibration values remain unknown. A4’s reported 222 is the persistent-value inventory size and supplies none of those calibration values. This analysis used the saved decompilation, static PE disassembly, and the existing A4 report. No hardware was accessed.