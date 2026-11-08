#!/usr/bin/env python3
"""Expose a Cricut Maker link (USB serial or Bluetooth RFCOMM) as a pseudo-terminal.

Wine opens the pty like a real serial port, so mapping it to a COM port in the
Design Space bottle lets the Windows app talk to the machine. With --log, every
byte in both directions is recorded (JSON Lines) for protocol research, which is
the groundwork for a native Linux cutter.

Usage:
  cricut-bridge.py --bt AA:BB:CC:DD:EE:FF [--channel 1] [--link PATH] [--log FILE]
  cricut-bridge.py --serial /dev/cricut   [--link PATH] [--log FILE]
"""
import argparse
import json
import os
import pty
import select
import socket
import sys
import termios
import time
import tty

HOST_TO_DEV = "host>dev"
DEV_TO_HOST = "dev>host"


class Logger:
    def __init__(self, path):
        self.f = open(path, "a", buffering=1) if path else None
        self.t0 = time.monotonic()

    def event(self, **kw):
        if self.f:
            kw["t"] = round(time.monotonic() - self.t0, 6)
            self.f.write(json.dumps(kw) + "\n")

    def data(self, direction, data):
        if self.f:
            self.event(dir=direction, hex=data.hex())


class SerialLink:
    def __init__(self, path):
        self.path = path
        self.fd = os.open(path, os.O_RDWR | os.O_NOCTTY)
        tty.setraw(self.fd)  # 8N1 raw; CDC-ACM devices ignore the baud rate
        attrs = termios.tcgetattr(self.fd)
        attrs[2] |= termios.CLOCAL | termios.CREAD
        termios.tcsetattr(self.fd, termios.TCSANOW, attrs)

    def fileno(self):
        return self.fd

    def recv(self):
        return os.read(self.fd, 4096)  # b"" or OSError(EIO) when unplugged

    def send(self, data):
        write_all(self.fd, data)

    def close(self):
        os.close(self.fd)

    def describe(self):
        return self.path


class BluetoothLink:
    def __init__(self, mac, channel):
        self.mac, self.channel = mac, channel
        self.sock = socket.socket(socket.AF_BLUETOOTH, socket.SOCK_STREAM, socket.BTPROTO_RFCOMM)
        try:
            self.sock.connect((mac, channel))
        except OSError:
            self.sock.close()
            raise

    def fileno(self):
        return self.sock.fileno()

    def recv(self):
        return self.sock.recv(4096)

    def send(self, data):
        self.sock.sendall(data)

    def close(self):
        self.sock.close()

    def describe(self):
        return f"{self.mac} channel {self.channel}"


def write_all(fd, data):
    while data:
        n = os.write(fd, data)
        data = data[n:]


def pump(link, master, log):
    """Copy bytes both ways until the machine link drops."""
    while True:
        readable, _, _ = select.select([link, master], [], [])
        if link in readable:
            data = link.recv()
            if not data:
                raise ConnectionError("machine closed the connection")
            log.data(DEV_TO_HOST, data)
            write_all(master, data)
        if master in readable:
            try:
                data = os.read(master, 4096)
            except OSError:
                time.sleep(0.1)  # EIO: nothing has the pty open
                continue
            if data:
                log.data(HOST_TO_DEV, data)
                link.send(data)


def main():
    ap = argparse.ArgumentParser(description=__doc__.splitlines()[0])
    src = ap.add_mutually_exclusive_group(required=True)
    src.add_argument("--bt", metavar="MAC", help="Bluetooth address of the Maker")
    src.add_argument("--serial", metavar="DEV", help="USB serial device, e.g. /dev/cricut")
    ap.add_argument("--channel", type=int, default=1, help="RFCOMM channel (SPP is normally 1)")
    ap.add_argument("--link", default=os.path.expanduser("~/.local/share/cricut-linux/cricut-pty"),
                    help="stable symlink pointing at the pty")
    ap.add_argument("--log", metavar="FILE", help="append all traffic to FILE as JSON Lines")
    args = ap.parse_args()

    if args.bt and not hasattr(socket, "AF_BLUETOOTH"):
        print("This Python build has no Bluetooth socket support.", file=sys.stderr)
        return 1

    def open_link():
        return BluetoothLink(args.bt, args.channel) if args.bt else SerialLink(args.serial)

    master, slave = pty.openpty()
    tty.setraw(slave)  # slave stays open so the master never sees EIO between sessions
    slave_path = os.ttyname(slave)
    os.makedirs(os.path.dirname(os.path.abspath(args.link)), exist_ok=True)
    if os.path.lexists(args.link):
        os.unlink(args.link)
    os.symlink(slave_path, args.link)
    print(f"pty {slave_path} linked at {args.link}", flush=True)
    log = Logger(args.log)
    if args.log:
        print(f"logging traffic to {args.log}", flush=True)

    try:
        while True:
            target = args.bt and f"{args.bt} channel {args.channel}" or args.serial
            try:
                print(f"connecting to {target}...", flush=True)
                link = open_link()
                print("connected", flush=True)
                log.event(event="connect", target=link.describe())
                try:
                    pump(link, master, log)
                finally:
                    link.close()
            except (OSError, ConnectionError) as e:
                log.event(event="disconnect", reason=str(e))
                print(f"link down: {e}; retrying in 3s (is the Maker on and connected?)", flush=True)
                time.sleep(3)
    except KeyboardInterrupt:
        pass
    finally:
        if os.path.islink(args.link):
            os.unlink(args.link)
    return 0


if __name__ == "__main__":
    sys.exit(main())
