"""Decode a USBPcap capture (.pcap, linktype 249) of a Cricut machine.

usage: python decode_usbpcap.py CAPTURE.pcap OUTDIR

Writes to OUTDIR:
  summary.txt    endpoints, control requests, frame sizes, activity per second
  bulk-dump.txt  every bulk/interrupt transfer with payload, hex + ASCII
  stream-out.bin / stream-in.bin  concatenated host->device / device->host payloads

Only USBPcap (Windows) captures are handled. Linux usbmon captures (linktype 220) use a different
per-packet header; see frames.py for the framing, which is the same on both.
"""
import collections, datetime, os, struct, sys

XFER = {0: "ISO", 1: "INT", 2: "CTRL", 3: "BULK"}
REQ = {0x20: "SET_LINE_CODING", 0x21: "GET_LINE_CODING", 0x22: "SET_CONTROL_LINE_STATE",
       0x06: "GET_DESCRIPTOR", 0x09: "SET_CONFIGURATION"}


def read_usbpcap(path):
    data = open(path, "rb").read()
    magic = struct.unpack_from("<I", data, 0)[0]
    if magic not in (0xA1B2C3D4, 0xA1B23C4D):
        raise SystemExit(f"not a little-endian pcap file: magic {magic:#x}")
    nano = magic == 0xA1B23C4D
    linktype = struct.unpack_from("<I", data, 20)[0]
    if linktype != 249:
        raise SystemExit(f"linktype {linktype} is not USBPcap (249)")
    off, pkts = 24, []
    while off + 16 <= len(data):
        ts_s, ts_u, incl, _ = struct.unpack_from("<IIII", data, off)
        off += 16
        rec = data[off:off + incl]
        off += incl
        hlen, irp, status, func, info, bus, dev, ep, xfer, dlen = struct.unpack_from("<HQIHBHHBBI", rec, 0)
        pkts.append(dict(
            t=ts_s + ts_u / (1e9 if nano else 1e6), dev=dev, ep=ep, xfer=XFER.get(xfer, xfer),
            stage=rec[27] if xfer == 2 and hlen >= 28 else None, data=rec[hlen:hlen + dlen]))
    return pkts


def ts(t):
    return datetime.datetime.fromtimestamp(t).strftime("%H:%M:%S.%f")[:-3]


def asc(b):
    return "".join(chr(c) if 32 <= c < 127 else "." for c in b)


def direction(p):
    return "IN" if p["ep"] & 0x80 else "OUT"


def main(path, outdir):
    os.makedirs(outdir, exist_ok=True)
    pkts = read_usbpcap(path)
    if not pkts:
        raise SystemExit("no packets")
    t0 = pkts[0]["t"]
    s = [f"file: {os.path.basename(path)}  packets: {len(pkts)}",
         f"time: {ts(pkts[0]['t'])} -> {ts(pkts[-1]['t'])}  ({pkts[-1]['t'] - t0:.1f}s)",
         "", "per endpoint (dev, ep, type, dir): records / payload bytes"]
    count, size = collections.Counter(), collections.Counter()
    for p in pkts:
        k = (p["dev"], f"ep 0x{p['ep']:02x}", p["xfer"], direction(p))
        count[k] += 1
        size[k] += len(p["data"])
    s += [f"  {k}: {count[k]} / {size[k]}" for k in sorted(count)]

    s += ["", "control SETUP packets:"]
    seen = collections.Counter()
    for p in pkts:
        if p["xfer"] == "CTRL" and p["stage"] == 0 and len(p["data"]) >= 8:
            rt, rq, val, idx, ln = struct.unpack_from("<BBHHH", p["data"])
            seen[f"bmRequestType=0x{rt:02x} {REQ.get(rq, hex(rq))} wValue=0x{val:04x} wIndex={idx} wLength={ln}"] += 1
    s += [f"  {n}x {line}" for line, n in seen.items()]

    bulk = [p for p in pkts if p["xfer"] in ("BULK", "INT") and p["data"]]
    sizes = collections.Counter((direction(p), len(p["data"])) for p in bulk)
    s += ["", f"bulk/int transfers with data: {len(bulk)}",
          "most common (dir, size): " + ", ".join(f"{k}x{v}" for k, v in sizes.most_common(12))]
    per_sec = collections.Counter(int(p["t"] - t0) for p in bulk)
    s += ["", "bulk transfers per second (offset s: count):",
          "  " + " ".join(f"{k}:{per_sec[k]}" for k in sorted(per_sec))]

    with open(os.path.join(outdir, "summary.txt"), "w", encoding="utf-8") as f:
        f.write("\n".join(s) + "\n")
    with open(os.path.join(outdir, "bulk-dump.txt"), "w", encoding="utf-8") as f:
        for p in bulk:
            b = p["data"]
            f.write(f"{ts(p['t'])} +{p['t'] - t0:8.3f}s {direction(p):3} ep0x{p['ep']:02x} {len(b)}B\n")
            for i in range(0, len(b), 16):
                f.write(f"    {i:04x}  {b[i:i + 16].hex(' '):<47}  |{asc(b[i:i + 16])}|\n")
    for d in ("OUT", "IN"):
        with open(os.path.join(outdir, f"stream-{d.lower()}.bin"), "wb") as f:
            f.write(b"".join(p["data"] for p in bulk if direction(p) == d))
    print("\n".join(s))


if __name__ == "__main__":
    if len(sys.argv) != 3:
        raise SystemExit(__doc__)
    main(sys.argv[1], sys.argv[2])
