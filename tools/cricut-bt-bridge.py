#!/usr/bin/env python3
"""Bridge a Cricut Maker's Bluetooth serial (RFCOMM/SPP) link to a pseudo-terminal.

Wine can open the pty like a real serial port, so mapping it to a COM port in the
Design Space bottle lets the Windows app talk to the machine over Bluetooth without
the deprecated `rfcomm` tool or root access.

Usage: cricut-bt-bridge.py AA:BB:CC:DD:EE:FF [--channel 1] [--link PATH]
"""
import argparse
import os
import pty
import select
import socket
import sys
import time
import tty


def connect(mac: str, channel: int) -> socket.socket:
    s = socket.socket(socket.AF_BLUETOOTH, socket.SOCK_STREAM, socket.BTPROTO_RFCOMM)
    s.connect((mac, channel))
    return s


def pump(sock: socket.socket, master: int) -> None:
    """Copy bytes both ways until the Bluetooth link drops."""
    while True:
        readable, _, _ = select.select([sock, master], [], [])
        if sock in readable:
            data = sock.recv(4096)
            if not data:
                raise ConnectionError("machine closed the connection")
            os.write(master, data)
        if master in readable:
            try:
                data = os.read(master, 4096)
            except OSError:
                # EIO: no process has the pty open yet (Design Space closed the port).
                time.sleep(0.1)
                continue
            if data:
                sock.sendall(data)


def main() -> int:
    ap = argparse.ArgumentParser(description=__doc__.splitlines()[0])
    ap.add_argument("mac")
    ap.add_argument("--channel", type=int, default=1, help="RFCOMM channel (SPP is normally 1)")
    ap.add_argument("--link", default=os.path.expanduser("~/.local/share/cricut-linux/cricut-bt"),
                    help="stable symlink pointing at the pty")
    args = ap.parse_args()

    if not hasattr(socket, "AF_BLUETOOTH"):
        print("This Python build has no Bluetooth socket support.", file=sys.stderr)
        return 1

    master, slave = pty.openpty()
    tty.setraw(slave)
    slave_path = os.ttyname(slave)
    os.makedirs(os.path.dirname(args.link), exist_ok=True)
    if os.path.islink(args.link) or os.path.exists(args.link):
        os.unlink(args.link)
    os.symlink(slave_path, args.link)
    print(f"pty {slave_path} linked at {args.link}", flush=True)

    try:
        while True:
            try:
                print(f"connecting to {args.mac} channel {args.channel}...", flush=True)
                sock = connect(args.mac, args.channel)
                print("connected", flush=True)
                with sock:
                    pump(sock, master)
            except (OSError, ConnectionError) as e:
                print(f"link down: {e}; retrying in 3s (is the Maker on and paired?)", flush=True)
                time.sleep(3)
    except KeyboardInterrupt:
        pass
    finally:
        if os.path.islink(args.link):
            os.unlink(args.link)
    return 0


if __name__ == "__main__":
    sys.exit(main())
