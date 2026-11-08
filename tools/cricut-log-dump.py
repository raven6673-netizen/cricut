#!/usr/bin/env python3
"""Pretty-print a traffic log written by cricut-bridge.py --log.

Consecutive chunks in the same direction are merged into one message, then shown
as a hex + ASCII dump with timestamps. Usage: cricut-log-dump.py LOG [--raw]
"""
import argparse
import json
import sys


def hexdump(data, indent="    "):
    for off in range(0, len(data), 16):
        chunk = data[off:off + 16]
        hx = " ".join(f"{b:02x}" for b in chunk)
        asc = "".join(chr(b) if 32 <= b < 127 else "." for b in chunk)
        yield f"{indent}{off:06x}  {hx:<47}  {asc}"


def messages(lines, merge=True):
    cur = None
    for line in lines:
        line = line.strip()
        if not line:
            continue
        rec = json.loads(line)
        if "dir" not in rec:
            if cur:
                yield cur
                cur = None
            yield rec
            continue
        data = bytes.fromhex(rec["hex"])
        if merge and cur and cur["dir"] == rec["dir"]:
            cur["data"] += data
        else:
            if cur:
                yield cur
            cur = {"t": rec["t"], "dir": rec["dir"], "data": data}
    if cur:
        yield cur


def main():
    ap = argparse.ArgumentParser(description=__doc__.splitlines()[0])
    ap.add_argument("log")
    ap.add_argument("--raw", action="store_true", help="don't merge consecutive chunks")
    args = ap.parse_args()
    with open(args.log) as f:
        for m in messages(f, merge=not args.raw):
            if "dir" not in m:
                detail = m.get("target") or m.get("reason") or ""
                print(f"[{m['t']:10.4f}] --- {m.get('event')} {detail}")
                continue
            arrow = "PC  -> Maker" if m["dir"] == "host>dev" else "Maker -> PC "
            print(f"[{m['t']:10.4f}] {arrow}  {len(m['data'])} bytes")
            for row in hexdump(m["data"]):
                print(row)
    return 0


if __name__ == "__main__":
    sys.exit(main())
