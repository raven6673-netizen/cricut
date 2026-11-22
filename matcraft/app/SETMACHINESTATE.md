**MatCraft’s `30 08 00 00 0C 0C 00 00 00 00 00 CC CC` matches the Athena body for the ordinary passive drag-blade profile, a 12 × 12-inch work area, and the same encoder count.** Design Space computes a host pressure count and sends it with byte 1 set to `08` on that path. The host binary establishes the bytes it sends. Which conditions the Maker firmware uses to accept, replace, or limit that count remains unknown. There is no source-backed pressure flag correction for this body. **High confidence** in host-side parity. `0x45a600`, `0x48e6e0`, `0x4ff680`, `0x4f8de0`.

## Exact Athena `0x30` body

Offsets are zero-based and start at the opcode. The body is 13 bytes.

| Offset | Value | Source in `CricutDevice.exe` |
|---|---|---|
| `0` | `30` | Literal opcode at `0x4f8e39`. |
| `1` | `(state << 7) \| profile_flags \| 0x08`, narrowed to one byte | Command object `+0x0C` and `+0x10`. `0x4f8e0d–0x4f8e51`. |
| `2`, `3` | `00` each | Literal zeros at `0x4f8e59`, `0x4f8e6a`. |
| `4` | Low byte of `ceil(work_area_width_inches)` | `PBSize +0x10` → command object `+0x18`. `0x48e6e0`, `0x4f8d90`, `0x4f8e78`, `0x8c5460`. |
| `5` | Low byte of `ceil(work_area_height_inches)` | `PBSize +0x08` → command object `+0x20`. `0x48e6e0`, `0x4f8d90`, `0x4f8e9e`, `0x8c5460`. |
| `6`, `7`, `8`, `9` | `00` each | Literal zeros at `0x4f8ec7`, `0x4f8ed8`, `0x4f8ee9`, `0x4f8efa`. |
| `10` | Pressure table byte | `0x45a600` output argument 4 → `0x4f8dd0` → command object `+0x11`. The formatter labels it “pressure table.” `0x4f8990`, `0x4f8f08`. |
| `11`, `12` | High byte, then low byte of the encoder count | `0x4ff680` count output → `0x4f8d70` → command object `+0x30`. The serializer reads the low 16 bits and appends them big-endian. `0x48e8e7`, `0x4f8f1b`, `0x452a60`. |

**High confidence** in the byte layout. The dimension helper rounds upward for positive fractional dimensions. For example, a 12.25-inch input produces `0D`. The caller supplies two `PBSize` work-area doubles, which correspond to width and length/height in the raw body. The trace does not source them from `PBSetMachineState` fields 4 and 5, and there is no encoder-unit scaling in this conversion. `0x4a55e0`, `0x48e8a5–0x48e8c3`, `0x4f8d90`, `0x4f8990`, `0x8c5460`.

## Byte 1 and the protobuf fields

| Bits | Athena wire source | What the trace establishes |
|---|---|---|
| `7` (`80`) | Low bit of command object `+0x0C`, shifted left seven | The constructor clears it. The main setup caller sets it when device state `+0x12C` equals `1` or `4`. A separate helper also sets it and sends a calibrated count with flags and table both zero. This is a real `88` variant. Its equivalence to protobuf `do_override` and its firmware effect are unproven. `0x4f8900`, `0x48e8fa–0x48e90e`, `0x48f0a0`. |
| `6`, `5`, `4` | Same-numbered bits of profile flags | Clear for every output of the traced `0x45a600` builder. |
| `3` (`08`) | Forced on | Literal OR at `0x4f8e46`. The instruction supplies no protobuf field association. |
| `2`, `1` | Same-numbered bits of profile flags | Clear for every output of the traced `0x45a600` builder. In particular, bit 1 is not evidence of wire value `PS_DO_NOT_USE_MACHINE=2`. |
| `0` (`01`) | Bit 0 of profile flags | `0x45a600` output argument 3 supplies `0` or `1`, depending on profile. The ordinary passive drag profile supplies `0`. `0x48e740–0x48e751`, `0x48e897–0x48e8a0`, `0x4f8d80`. |

**High confidence** in the bit sources. The raw serializer reads an `AthenaSetMachineStateCommand` object. In particular, `0x4f8990` is a diagnostic formatter whose printed flags expression differs from the expression in the actual serializer, `0x4f8de0`.

- **`pressure_select`** has a plausible native counterpart at command object `+0x12`. A generic setup path passes a `1` or `2` selector through vtable slot `+0x30`, which resolves to setter `0x4f8dc0` for Athena. The Athena constructor initializes it to zero, the ordinary setup leaves it there, and **the Athena serializer never reads it**. The Zorro serializer does read that slot and shift it into its own flags byte. **High confidence** in the data flow and wire omission. **Medium confidence** in the protobuf name association. `0x476f80`, Athena vtable `0x962b74`, `0x4f8dc0`, `0x4f8900`, `0x4f8de0`, `0x4ede00`.
- **`do_override`, `head`, `cut_bezier`, `tangential_theta`, `ack`** have no verified protobuf-to-wire assignment on this Athena path. The observed inputs to byte 1 are exhausted by the table above. In particular, this trace cannot identify bit 7 as protobuf `do_override`. **High confidence** in the wire inputs. **Unresolved** protobuf associations. `0x45a600`, `0x48e6e0`, `0x4f8de0`.
- **`pressure_table` and `encoder_counts`** have directly identifiable wire counterparts at byte `10` and bytes `11–12`, respectively. The ordinary passive drag profile supplies table `00`, numerically the supplied schema’s `PT_RESERVED1=0`. **High confidence** in the raw fields and passive-profile values. `0x45a600`, `0x4f8990`, `0x4f8dd0`, `0x4f8d70`, `0x4f8de0`.
- **`max_speed`, `pen_motor_speed`, and `blade_motor_speed`** have no payload in Athena `0x30`. The diagnostic formatter prints a native max-speed member at `+0x28`, which the raw serializer skips. Athena sends a separate `0x31` speed profile after `0x30`. Its body starts `31 0F 0F 0F 0F 0F` and contains 20 big-endian 16-bit values in four groups of five. The correspondence between those speed-profile values and the three named protobuf fields remains unverified. **High confidence** in the omission and separate speed-profile layout. `0x4f8990`, `0x4f8de0`, `0x48e914–0x48e92b`, `0x4fce20`, `0x4fd240`.

## What this means for the light cut

For the ordinary passive drag setup, the verified values are **state bit `0`, profile flags `00`, pressure table `00`, and the computed count in the final word**. The native selector slot stays at its constructor default `0` and has no wire representation in this body. The trace establishes no value for a protobuf `do_override` on that path. Design Space’s ordinary path still computes and transmits the count with this exact shape:

```text
30 08 00 00 0C 0C 00 00 00 00 00 CC_HI CC_LO
```

So **there is no demonstrated missing `pressure_select` or `do_override` setting in MatCraft’s `0x30`**. Static host analysis cannot determine whether the firmware consumes the count in the reported job, substitutes a default, or applies a limit. It also cannot establish that the count MatCraft supplies matches the count Design Space would compute for the same tool, material, and machine calibration. The cause of the light cut and Z bobbing remains unresolved. `0x4ff680`, `0x4d6ba0`, `0x48e6e0`, `0x48f0a0`, `0x4f8de0`.