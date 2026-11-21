"""List Cricut protocol frames from a USBPcap capture, and compare two captures.

usage:
  python frames.py A.pcap              list handshakes and job frames in A
  python frames.py A.pcap B.pcap       also compare encrypted OUT frames of A and B

Frame format (same over USB and, as far as known, Bluetooth RFCOMM):
  2-byte big-endian length word, then the payload.
  Top bit of the length word set  -> payload is encrypted (16-byte blocks).
  Low 15 bits                     -> payload length.
Each bulk transfer carries exactly one frame. The job's design data is sent as
encrypted frames larger than 32 bytes (336 B for one 1x1 in square).
"""
import collections, sys

from decode_usbpcap import direction, read_usbpcap


def frames(path):
    pkts = read_usbpcap(path)
    t0 = pkts[0]["t"] if pkts else 0
    out = []
    for p in pkts:
        if p["xfer"] != "BULK" or len(p["data"]) < 2:
            continue
        word = int.from_bytes(p["data"][:2], "big")
        out.append(dict(t=p["t"] - t0, dir=direction(p), enc=bool(word & 0x8000),
                        length=word & 0x7FFF, payload=bytes(p["data"][2:])))
    return out


def describe(path):
    fr = frames(path)
    print(f"== {path}: {len(fr)} frames")
    for f in fr:
        start = not f["enc"] and f["dir"] == "OUT" and f["payload"][:1] == b"\xcf"
        keyx = not f["enc"] and f["length"] in (0x30, 0x31)
        job = f["enc"] and f["length"] > 16
        if start or keyx or job:
            tag = "handshake start" if start else "key exchange" if keyx else "job/encrypted"
            print(f"  +{f['t']:8.3f}s {f['dir']:3} {f['length']:4}B  {tag:16} {f['payload'][:16].hex(' ')}")
    return fr


def compare(a, b):
    oa = collections.Counter(f["payload"] for f in a if f["enc"] and f["dir"] == "OUT")
    ob = collections.Counter(f["payload"] for f in b if f["enc"] and f["dir"] == "OUT")
    print(f"== encrypted OUT frames: A {len(oa)} distinct, B {len(ob)} distinct, shared {len(set(oa) & set(ob))}")
    print("   (shared > 0 means the same session key: ciphertext is deterministic within a connection)")


if __name__ == "__main__":
    if len(sys.argv) not in (2, 3):
        raise SystemExit(__doc__)
    a = describe(sys.argv[1])
    if len(sys.argv) == 3:
        compare(a, describe(sys.argv[2]))
