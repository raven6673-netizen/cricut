//! Talking to the cutting machine over a Bluetooth serial (RFCOMM) link.
//!
//! The framing and command numbers come from the reverse-engineering sheet (MECHANICS.md):
//! a 2-byte big-endian header (top bit = encrypted, low 15 bits = length, at most 512),
//! optional AES-256 (ECB, 16-byte blocks, last byte = 8-bit sum of the bytes before it),
//! and one-byte command codes whose reply starts with the same code.

use std::io::{Read, Write};
use std::sync::mpsc::{channel, Receiver, RecvTimeoutError, Sender};
use std::time::{Duration, Instant};

pub const DEFAULT_MAC: &str = "00:06:66:00:12:34";
pub const MAX_PAYLOAD: usize = 0x200;

/// Command codes of the Maker family ("Athena" in the original's code).
#[allow(dead_code)] // the full known set, including ones whose payload isn't decoded yet
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum Cmd {
    ClearError,
    ReadErrorSeverity,
    SetSpeedState,
    SetToolState,
    SetFullToolState,
    CprToggle,
    AbortCutting,
    GetInterfaceState,
    GetPersistentValueCount,
    ScanForEdge,
    BluetoothAlert,
    HomeTheta,
    Move4D,
    SendZeros,
    NewAes,
}

impl Cmd {
    pub fn code(self) -> u8 {
        match self {
            Cmd::ClearError => 0x0A,
            Cmd::ReadErrorSeverity => 0x0B,
            Cmd::SetSpeedState => 0x31,
            Cmd::SetToolState => 0x37,
            Cmd::SetFullToolState => 0x3E,
            Cmd::CprToggle => 0x3F,
            Cmd::AbortCutting => 0x68,
            Cmd::GetInterfaceState => 0x73,
            Cmd::GetPersistentValueCount => 0xA4,
            Cmd::ScanForEdge => 0xE2,
            Cmd::BluetoothAlert => 0xE5,
            Cmd::HomeTheta => 0xF0,
            Cmd::Move4D => 0xF1,
            Cmd::SendZeros => 0xFF,
            Cmd::NewAes => 0xC7,
        }
    }
    pub fn label(self) -> &'static str {
        match self {
            Cmd::ClearError => "Clear error",
            Cmd::ReadErrorSeverity => "Read error level",
            Cmd::SetSpeedState => "Set speed",
            Cmd::SetToolState => "Set tool",
            Cmd::SetFullToolState => "Set full tool",
            Cmd::CprToggle => "Toggle CPR",
            Cmd::AbortCutting => "Stop cutting",
            Cmd::GetInterfaceState => "Read buttons & lights",
            Cmd::GetPersistentValueCount => "Read stored values",
            Cmd::ScanForEdge => "Find mat edge",
            Cmd::BluetoothAlert => "Bluetooth alert",
            Cmd::HomeTheta => "Home the blade turn",
            Cmd::Move4D => "Turn blade",
            Cmd::SendZeros => "Wake / flush (zeros)",
            Cmd::NewAes => "Ask for encrypted session",
        }
    }
}

/// Build the payload of a command (before framing).
pub fn payload(cmd: Cmd, theta_degrees: f32) -> Vec<u8> {
    match cmd {
        Cmd::Move4D => {
            // code, axis mask 0x17, then big-endian f32: x, y, z, theta (radians).
            let mut v = vec![cmd.code(), 0x17];
            for f in [0.0f32, 0.0, 0.0, (theta_degrees as f64 * 0.017453292519943295) as f32] {
                v.extend_from_slice(&f.to_be_bytes());
            }
            v
        }
        Cmd::NewAes => {
            // code, key slot (2..=13), then 47 random bytes: 49 bytes, so the frame header is 00 31.
            // the clock plus a running count, so requests made in the same instant still differ
            static COUNT: std::sync::atomic::AtomicU64 = std::sync::atomic::AtomicU64::new(0);
            let n = COUNT.fetch_add(1, std::sync::atomic::Ordering::Relaxed).wrapping_add(1);
            let mut seed = (std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .map(|d| d.as_nanos() as u64)
                .unwrap_or(0x9E37_79B9_7F4A_7C15)
                ^ n.wrapping_mul(0x9E37_79B9_7F4A_7C15))
                | 1;
            for _ in 0..4 {
                seed ^= seed << 13;
                seed ^= seed >> 7;
                seed ^= seed << 17;
            }
            let mut next = || {
                seed ^= seed << 13;
                seed ^= seed >> 7;
                seed ^= seed << 17;
                seed
            };
            let mut v = vec![cmd.code(), (next() % 12) as u8 + 2];
            for _ in 0..47 {
                v.push(next() as u8);
            }
            v
        }
        _ => vec![cmd.code()],
    }
}

// ---------------------------------------------------------------- framing

pub fn frame_plain(payload: &[u8]) -> Vec<u8> {
    let n = payload.len().min(0x7fff);
    let mut v = vec![((n >> 8) & 0x7f) as u8, (n & 0xff) as u8];
    v.extend_from_slice(&payload[..n]);
    v
}

/// Pad, add the 8-bit sum and encrypt with AES-256 (ECB), then add the header with the top bit set.
pub fn frame_encrypted(payload: &[u8], key: &[u8; 32]) -> Vec<u8> {
    use aes::cipher::{generic_array::GenericArray, BlockEncrypt, KeyInit};
    let size = payload.len() - payload.len() % 16 + 16;
    let mut buf = vec![0u8; size];
    buf[..payload.len()].copy_from_slice(payload);
    let sum = buf[..size - 1].iter().fold(0u8, |a, b| a.wrapping_add(*b));
    buf[size - 1] = sum;
    let cipher = aes::Aes256::new(GenericArray::from_slice(key));
    for block in buf.chunks_mut(16) {
        cipher.encrypt_block(GenericArray::from_mut_slice(block));
    }
    let mut v = vec![0x80 | ((size >> 8) as u8 & 0x7f), (size & 0xff) as u8];
    v.extend_from_slice(&buf);
    v
}

/// Undo [`frame_encrypted`] on a reply body: decrypt, check the sum, then apply the byte swap the
/// original applies to decrypted replies (first two bytes swapped when the first is non-zero).
pub fn open_encrypted(body: &[u8], key: &[u8; 32]) -> Result<Vec<u8>, String> {
    use aes::cipher::{generic_array::GenericArray, BlockDecrypt, KeyInit};
    if body.len() % 16 != 0 || body.is_empty() {
        return Err("encrypted reply is not a whole number of 16-byte blocks".into());
    }
    let cipher = aes::Aes256::new(GenericArray::from_slice(key));
    let mut buf = body.to_vec();
    for block in buf.chunks_mut(16) {
        cipher.decrypt_block(GenericArray::from_mut_slice(block));
    }
    let n = buf.len();
    let sum = buf[..n - 1].iter().fold(0u8, |a, b| a.wrapping_add(*b));
    if sum != buf[n - 1] {
        return Err("reply checksum mismatch (wrong key?)".into());
    }
    if n > 1 && buf[0] != 0 {
        buf.swap(0, 1);
    }
    Ok(buf)
}

/// The Bluetooth name the machine advertises: "Maker-XXXX", where XXXX is the last two bytes of
/// its address in hex. For example, address ...12:34 advertises "Maker-1234".
/// Useful for picking the right device out of a Bluetooth list when pairing.
pub fn bluetooth_name(mac: &str) -> Option<String> {
    let m = parse_mac(mac)?;
    Some(format!("Maker-{:02X}{:02X}", m[4], m[5]))
}

pub fn parse_key(hex: &str) -> Option<[u8; 32]> {
    let clean: String = hex.chars().filter(|c| c.is_ascii_hexdigit()).collect();
    if clean.len() != 64 {
        return None;
    }
    let mut k = [0u8; 32];
    for i in 0..32 {
        k[i] = u8::from_str_radix(&clean[i * 2..i * 2 + 2], 16).ok()?;
    }
    Some(k)
}

pub fn parse_mac(s: &str) -> Option<[u8; 6]> {
    let parts: Vec<&str> = s.trim().split(|c| c == ':' || c == '-').collect();
    if parts.len() != 6 {
        return None;
    }
    let mut m = [0u8; 6];
    for (i, p) in parts.iter().enumerate() {
        m[i] = u8::from_str_radix(p, 16).ok()?;
    }
    Some(m)
}

pub fn hex(bytes: &[u8]) -> String {
    bytes.iter().map(|b| format!("{b:02X}")).collect::<Vec<_>>().join(" ")
}

// ---------------------------------------------------------------- transports

#[derive(Clone, Debug, PartialEq)]
pub enum Link {
    /// Bluetooth RFCOMM socket straight to the machine's address (Linux).
    Rfcomm { mac: String, channel: u8 },
    /// A serial device file: /dev/rfcomm0 on Linux after `rfcomm bind`, or \\.\COM5 on Windows.
    Device { path: String },
    /// The machine's USB cable: a USB serial port (COM6 on Windows, /dev/ttyACM0 on Linux).
    /// An empty port means "find it by its USB vendor id".
    Usb { port: String },
    /// A pretend machine inside the app, for trying things without hardware.
    Demo,
}

/// USB vendor id the machine reports (seen on this computer as VID_20D3&PID_0015).
pub const USB_VENDOR: u16 = 0x20D3;

/// One machine found on a USB serial port.
#[derive(Clone, Debug, PartialEq)]
pub struct UsbMachine {
    pub port: String,
    pub product_id: u16,
    pub serial: String,
}

/// Machines plugged in by USB, found by vendor id.
#[cfg(not(target_os = "linux"))]
pub fn find_usb_machines() -> Vec<UsbMachine> {
    let Ok(ports) = serialport::available_ports() else { return Vec::new() };
    ports
        .into_iter()
        .filter_map(|p| match p.port_type {
            serialport::SerialPortType::UsbPort(u) if u.vid == USB_VENDOR => {
                Some(UsbMachine { port: p.port_name, product_id: u.pid, serial: u.serial_number.unwrap_or_default() })
            }
            _ => None,
        })
        .collect()
}

/// Machines plugged in by USB, found by vendor id (read from sysfs, so no udev library is needed).
#[cfg(target_os = "linux")]
pub fn find_usb_machines() -> Vec<UsbMachine> {
    let mut out = Vec::new();
    let Ok(dir) = std::fs::read_dir("/sys/class/tty") else { return out };
    for e in dir.flatten() {
        let name = e.file_name().to_string_lossy().to_string();
        if !(name.starts_with("ttyACM") || name.starts_with("ttyUSB")) {
            continue;
        }
        // walk up from the tty's device to the USB device that carries idVendor
        let Ok(mut dev) = std::fs::canonicalize(e.path().join("device")) else { continue };
        for _ in 0..4 {
            let vid = std::fs::read_to_string(dev.join("idVendor")).ok();
            if let Some(v) = vid {
                if u16::from_str_radix(v.trim(), 16).ok() == Some(USB_VENDOR) {
                    let pid = std::fs::read_to_string(dev.join("idProduct")).ok().and_then(|p| u16::from_str_radix(p.trim(), 16).ok()).unwrap_or(0);
                    let serial = std::fs::read_to_string(dev.join("serial")).map(|s| s.trim().to_string()).unwrap_or_default();
                    out.push(UsbMachine { port: format!("/dev/{name}"), product_id: pid, serial });
                }
                break;
            }
            if !dev.pop() {
                break;
            }
        }
    }
    out.sort_by(|a, b| a.port.cmp(&b.port));
    out
}

/// Open a USB serial port raw (no echo or line-ending changes), with DTR raised so the
/// machine knows a program is listening.
fn open_usb(port: &str) -> Result<Box<dyn Port>, String> {
    let port = if port.trim().is_empty() {
        find_usb_machines()
            .into_iter()
            .next()
            .map(|m| m.port)
            .ok_or("No machine found on USB. Is the cable plugged in and the machine on?")?
    } else {
        port.trim().to_string()
    };
    let mut p = serialport::new(&port, 115_200)
        .timeout(Duration::from_millis(50))
        .open()
        .map_err(|e| serial_open_error(&port, &e.to_string()))?;
    let _ = p.write_data_terminal_ready(true);
    let _ = p.write_request_to_send(true);
    let _ = p.clear(serialport::ClearBuffer::All);
    let w = p.try_clone().map_err(|e| format!("Couldn't share {port}: {e}"))?;
    Ok(Box::new(StreamPort::new(p, Box::new(w))))
}

/// One question asked of the real machine and what came back.
#[derive(Clone, Debug, serde::Serialize, serde::Deserialize)]
pub struct LiveAnswer {
    /// The sheet's name for the command.
    pub command: String,
    pub code: u8,
    pub sent: String,
    /// The 2-byte header of the reply, as received.
    pub header: Option<String>,
    /// The reply body (after the header), plain.
    pub body: Option<String>,
    pub encrypted: Option<bool>,
    pub note: String,
}

/// The real machine's answers, saved as mechanics-live.json.
#[derive(Clone, Debug, serde::Serialize, serde::Deserialize)]
pub struct LiveCheck {
    pub port: String,
    pub machine: String,
    pub answers: Vec<LiveAnswer>,
}

/// Commands that only read from the machine, safe to ask any time.
pub const READ_ONLY: [(Cmd, &str); 3] = [
    (Cmd::GetInterfaceState, "get interface state"),
    (Cmd::ReadErrorSeverity, "read error severity"),
    (Cmd::GetPersistentValueCount, "get persistent value count"),
];

/// Ask the machine on USB each read-only question once, reading the raw header and body, so the
/// framing and command codes can be compared with the mechanics sheet. Nothing moves.
pub fn live_check(port: &str) -> Result<LiveCheck, String> {
    let found = find_usb_machines();
    let chosen = if port.trim().is_empty() { found.first().map(|m| m.port.clone()).unwrap_or_default() } else { port.trim().to_string() };
    let machine = found
        .iter()
        .find(|m| m.port == chosen)
        .map(|m| format!("USB {USB_VENDOR:04X}:{:04X} serial {}", m.product_id, m.serial))
        .unwrap_or_else(|| "USB machine".into());
    let mut p = open_usb(&chosen)?;
    let mut buf = [0u8; 600];
    // drop anything already waiting
    while let Ok(n) = p.read_some(&mut buf, Duration::from_millis(200)) {
        if n == 0 {
            break;
        }
    }
    let mut answers = Vec::new();
    for (cmd, name) in READ_ONLY {
        let frame = frame_plain(&payload(cmd, 0.0));
        let mut a = LiveAnswer { command: name.into(), code: cmd.code(), sent: hex(&frame), header: None, body: None, encrypted: None, note: String::new() };
        if let Err(e) = p.write_all(&frame) {
            a.note = format!("write failed: {e}");
            answers.push(a);
            break;
        }
        let deadline = Instant::now() + Duration::from_secs(5);
        match read_exact_timeout(p.as_mut(), 2, deadline) {
            Ok(Some(h)) => {
                let enc = h[0] & 0x80 != 0;
                let len = (((h[0] & 0x7f) as usize) << 8) | h[1] as usize;
                a.header = Some(hex(&h));
                a.encrypted = Some(enc);
                if len > MAX_PAYLOAD {
                    a.note = format!("header says {len} bytes, more than {MAX_PAYLOAD}");
                } else {
                    match read_exact_timeout(p.as_mut(), len, deadline) {
                        Ok(Some(b)) => {
                            a.note = if b.first() == Some(&cmd.code()) { "reply starts with the command's code".into() } else { "reply doesn't start with the command's code".into() };
                            a.body = Some(hex(&b));
                        }
                        Ok(None) => a.note = "body didn't arrive in 5 s".into(),
                        Err(e) => a.note = e,
                    }
                }
            }
            Ok(None) => a.note = "no reply in 5 s".into(),
            Err(e) => a.note = e,
        }
        answers.push(a);
        std::thread::sleep(Duration::from_millis(100));
    }
    Ok(LiveCheck { port: chosen, machine, answers })
}

/// Find the machine on USB, open it, send one read-only status question (get interface state)
/// and wait for its answer. Used by `--usb-probe` and the Machine panel's "Test USB" button.
pub fn probe_usb(port: &str, wait: Duration) -> Vec<String> {
    let mut log = Vec::new();
    let found = find_usb_machines();
    if found.is_empty() {
        log.push(format!("No USB serial port with vendor id {USB_VENDOR:04X} found."));
    }
    for m in &found {
        log.push(format!("Found machine on {} (product {:04X}, serial {})", m.port, m.product_id, if m.serial.is_empty() { "?" } else { &m.serial }));
    }
    let mut p = match open_usb(port) {
        Ok(p) => p,
        Err(e) => {
            log.push(e);
            return log;
        }
    };
    log.push("Opened the port.".into());
    // anything the machine says on its own when a program opens the port
    let mut buf = [0u8; 600];
    let mut hello = Vec::new();
    let until = Instant::now() + Duration::from_millis(600);
    while Instant::now() < until {
        match p.read_some(&mut buf, Duration::from_millis(100)) {
            Ok(n) if n > 0 => hello.extend_from_slice(&buf[..n]),
            _ => {}
        }
    }
    if !hello.is_empty() {
        log.push(format!("Machine said on open: {}", hex(&hello)));
    }
    let frame = frame_plain(&payload(Cmd::GetInterfaceState, 0.0));
    log.push(format!("→ get interface state : {}", hex(&frame)));
    if let Err(e) = p.write_all(&frame) {
        log.push(format!("Write failed: {e}"));
        return log;
    }
    match read_reply(p.as_mut(), None, wait) {
        Ok(Some(bytes)) => {
            log.push(format!("← {}", hex(&bytes)));
            log.push(if bytes.first() == Some(&Cmd::GetInterfaceState.code()) {
                "The machine answered in the expected format: USB link works.".into()
            } else {
                "The machine answered, but not with the expected command code.".into()
            });
        }
        Ok(None) => {
            // show raw bytes if a reply arrived that isn't framed as expected
            let mut raw = Vec::new();
            while let Ok(n) = p.read_some(&mut buf, Duration::from_millis(100)) {
                if n == 0 {
                    break;
                }
                raw.extend_from_slice(&buf[..n]);
            }
            log.push(if raw.is_empty() { format!("No reply within {} s.", wait.as_secs()) } else { format!("Unframed bytes: {}", hex(&raw)) });
        }
        Err(e) => log.push(e),
    }
    log
}

trait Port: Send {
    fn write_all(&mut self, b: &[u8]) -> std::io::Result<()>;
    /// Read one chunk; Ok(0) means nothing yet.
    fn read_some(&mut self, buf: &mut [u8], wait: Duration) -> std::io::Result<usize>;
}

/// Any Read+Write handle, with a background reader so reads can time out.
struct StreamPort {
    writer: Box<dyn Write + Send>,
    rx: Receiver<Vec<u8>>,
    pending: Vec<u8>,
    closed: std::sync::Arc<std::sync::atomic::AtomicBool>,
    #[cfg(target_os = "linux")]
    rfcomm_fd: Option<std::os::fd::RawFd>,
}

impl StreamPort {
    fn new<R: Read + Send + 'static>(mut reader: R, writer: Box<dyn Write + Send>) -> Self {
        let (tx, rx) = channel();
        let closed = std::sync::Arc::new(std::sync::atomic::AtomicBool::new(false));
        let reader_closed = closed.clone();
        std::thread::spawn(move || {
            let mut buf = [0u8; 1024];
            while !reader_closed.load(std::sync::atomic::Ordering::Relaxed) {
                match reader.read(&mut buf) {
                    Ok(0) => break,
                    Ok(n) => {
                        if tx.send(buf[..n].to_vec()).is_err() {
                            break;
                        }
                    }
                    Err(e) if e.kind() == std::io::ErrorKind::WouldBlock || e.kind() == std::io::ErrorKind::TimedOut => {
                        std::thread::sleep(Duration::from_millis(5));
                    }
                    Err(_) => break,
                }
            }
        });
        Self {
            writer,
            rx,
            pending: Vec::new(),
            closed,
            #[cfg(target_os = "linux")]
            rfcomm_fd: None,
        }
    }
}

impl Drop for StreamPort {
    fn drop(&mut self) {
        self.closed.store(true, std::sync::atomic::Ordering::Relaxed);
        #[cfg(target_os = "linux")]
        if let Some(fd) = self.rfcomm_fd {
            // The writer still owns fd. Shutdown wakes the reader on the cloned descriptor.
            unsafe { libc::shutdown(fd, libc::SHUT_RDWR) };
        }
    }
}

impl Port for StreamPort {
    fn write_all(&mut self, b: &[u8]) -> std::io::Result<()> {
        self.writer.write_all(b)?;
        self.writer.flush()
    }
    fn read_some(&mut self, buf: &mut [u8], wait: Duration) -> std::io::Result<usize> {
        if self.pending.is_empty() {
            match self.rx.recv_timeout(wait) {
                Ok(v) => self.pending = v,
                Err(RecvTimeoutError::Timeout) => return Ok(0),
                Err(RecvTimeoutError::Disconnected) => {
                    return Err(std::io::Error::new(std::io::ErrorKind::BrokenPipe, "the link closed"))
                }
            }
        }
        let n = self.pending.len().min(buf.len());
        buf[..n].copy_from_slice(&self.pending[..n]);
        self.pending.drain(..n);
        Ok(n)
    }
}

/// The pretend machine: answers every command with its own code and status 0.
struct DemoPort {
    out: Vec<u8>,
}

impl Port for DemoPort {
    fn write_all(&mut self, b: &[u8]) -> std::io::Result<()> {
        if b.len() >= 3 && b[0] & 0x80 == 0 {
            let code = b[2];
            let body: Vec<u8> = match code {
                0x73 => vec![code, 0x01, 0x00, 0x00, 0x00],
                _ => vec![code, 0x00],
            };
            self.out.extend(frame_plain(&body));
        }
        Ok(())
    }
    fn read_some(&mut self, buf: &mut [u8], _wait: Duration) -> std::io::Result<usize> {
        let n = self.out.len().min(buf.len());
        buf[..n].copy_from_slice(&self.out[..n]);
        self.out.drain(..n);
        Ok(n)
    }
}

#[cfg(target_os = "linux")]
fn open_rfcomm(mac: &str, channel: u8) -> Result<Box<dyn Port>, String> {
    use std::os::fd::FromRawFd;
    const AF_BLUETOOTH: libc::c_int = 31;
    const BTPROTO_RFCOMM: libc::c_int = 3;
    #[repr(C)]
    struct SockaddrRc {
        family: libc::sa_family_t,
        bdaddr: [u8; 6],
        channel: u8,
    }
    let m = parse_mac(mac).ok_or("That doesn't look like a Bluetooth address (like 00:06:66:00:12:34).")?;
    let mut bd = m;
    bd.reverse(); // BlueZ stores the address little-endian.
    unsafe {
        let fd = libc::socket(AF_BLUETOOTH, libc::SOCK_STREAM | libc::SOCK_CLOEXEC, BTPROTO_RFCOMM);
        if fd < 0 {
            let e = std::io::Error::last_os_error();
            if e.raw_os_error() == Some(libc::EAFNOSUPPORT) && in_flatpak() {
                return Err("Bluetooth is blocked because MatCraft was started inside a Flatpak sandbox \
                     (for example from Craft Anything). Start MatCraft from a normal terminal or the app menu instead, \
                     or use the USB cable."
                    .into());
            }
            return Err(format!("Couldn't open a Bluetooth socket: {e}"));
        }
        let addr = SockaddrRc { family: AF_BLUETOOTH as libc::sa_family_t, bdaddr: bd, channel };
        let r = libc::connect(
            fd,
            &addr as *const SockaddrRc as *const libc::sockaddr,
            std::mem::size_of::<SockaddrRc>() as libc::socklen_t,
        );
        if r < 0 {
            let e = std::io::Error::last_os_error();
            libc::close(fd);
            return Err(format!(
                "Couldn't reach {mac} on channel {channel}: {e}. Is the machine on, paired and trusted?"
            ));
        }
        let read_fd = libc::fcntl(fd, libc::F_DUPFD_CLOEXEC, 0);
        if read_fd < 0 {
            let error = std::io::Error::last_os_error();
            libc::close(fd);
            return Err(format!("Couldn't open the Bluetooth reader: {error}"));
        }
        let reader = std::fs::File::from_raw_fd(read_fd);
        let writer = std::fs::File::from_raw_fd(fd);
        let mut port = StreamPort::new(reader, Box::new(writer));
        port.rfcomm_fd = Some(fd);
        Ok(Box::new(port))
    }
}

#[cfg(not(target_os = "linux"))]
fn open_rfcomm(_mac: &str, _channel: u8) -> Result<Box<dyn Port>, String> {
    Err("A direct Bluetooth socket is only built for Linux. On this computer pair the machine first, \
         then choose 'Serial device' with its COM port (for example \\\\.\\COM5)."
        .into())
}

/// True when running inside a Flatpak sandbox, which hides Bluetooth sockets and serial ports.
fn in_flatpak() -> bool {
    cfg!(target_os = "linux") && std::path::Path::new("/.flatpak-info").exists()
}

/// Explain the usual reasons a serial port won't open, in plain words.
fn serial_open_error(port: &str, err: &str) -> String {
    let denied = err.to_lowercase().contains("permission denied");
    if denied && cfg!(target_os = "linux") {
        if in_flatpak() {
            return format!("Couldn't open {port}: MatCraft is running inside a Flatpak sandbox, which can't use \
                 serial ports. Start MatCraft from a normal terminal or the app menu instead.");
        }
        return format!("Couldn't open {port}: your account isn't allowed to use serial ports. Add it to the \
             'dialout' group once, then log out and back in. On Bazzite: \
             grep -E '^dialout:' /usr/lib/group | sudo tee -a /etc/group && sudo usermod -aG dialout $USER");
    }
    format!("Couldn't open {port}: {err}. Is another program (like the original app) using it?")
}

fn open_device(path: &str) -> Result<Box<dyn Port>, String> {
    let f = std::fs::OpenOptions::new()
        .read(true)
        .write(true)
        .open(path)
        .map_err(|e| serial_open_error(path, &e.to_string()))?;
    let w = f.try_clone().map_err(|e| e.to_string())?;
    Ok(Box::new(StreamPort::new(f, Box::new(w))))
}

/// Confirm the machine is answering before the app offers to send commands.
fn confirm_connection(port: &mut dyn Port, key: Option<&[u8; 32]>, wait: Duration) -> Result<(), String> {
    let cmd = Cmd::GetInterfaceState;
    port.write_all(&frame_plain(&payload(cmd, 0.0)))
        .map_err(|error| format!("Couldn't request machine status: {error}"))?;
    match read_reply(port, key, wait)? {
        Some(reply) if reply.first() == Some(&cmd.code()) => Ok(()),
        Some(_) => Err("The machine sent an unexpected reply to the status check.".into()),
        None => Err("The port opened, but the machine didn't answer the status check. Check that it's on and try again.".into()),
    }
}

// ---------------------------------------------------------------- worker

pub enum Request {
    #[cfg(target_os = "linux")]
    Pair { mac: String },
    Connect { link: Link, key: Option<[u8; 32]> },
    Send { cmd: Cmd, theta: f32 },
    /// Stream a cut job: tool setup (pressure/speed), BeginDraw, MultiDrawVector batches, EndDraw,
    /// waiting for each command's acknowledgement. Records are in inches (see [`contours_to_records`]).
    Cut { records: Vec<DrawRecord>, setup: CutSetup },
    /// Read the drag-blade pressure calibration (A0 indices 0x21..=0x30) from the machine.
    ReadCalibration,
    Disconnect,
}

#[derive(Clone, Debug)]
pub enum Event {
    #[cfg(target_os = "linux")]
    PairFinished(String),
    Log(String),
    Connected(String),
    Disconnected(String),
    Reply { cmd: Cmd, status: Option<u8>, bytes: Vec<u8> },
    /// The drag-blade pressure calibration read from the machine, as (grams, counts) rows sorted by
    /// force. Empty if the machine didn't answer the A0 reads.
    Calibration { rows: Vec<(u16, u16)> },
}

pub struct MachineHandle {
    pub tx: Sender<Request>,
    pub rx: Receiver<Event>,
}

pub fn spawn(ctx: egui::Context) -> MachineHandle {
    let (req_tx, req_rx) = channel::<Request>();
    let (ev_tx, ev_rx) = channel::<Event>();
    std::thread::spawn(move || {
        let send = |e: Event| {
            let _ = ev_tx.send(e);
            ctx.request_repaint();
        };
        let mut port: Option<Box<dyn Port>> = None;
        let mut key: Option<[u8; 32]> = None;
        while let Ok(req) = req_rx.recv() {
            match req {
                #[cfg(target_os = "linux")]
                Request::Pair { mac } => {
                    send(Event::PairFinished(match pair_bluetooth(&mac) {
                        Ok(()) => "Bluetooth pairing complete. Select Connect to open the machine link.".into(),
                        Err(error) => error,
                    }));
                }
                Request::Connect { link, key: k } => {
                    port = None;
                    key = k;
                    let opened = match &link {
                        Link::Rfcomm { mac, channel } => {
                            send(Event::Log(format!("Connecting to {mac} (RFCOMM channel {channel})…")));
                            open_rfcomm(mac, *channel)
                        }
                        Link::Device { path } => {
                            send(Event::Log(format!("Opening {path}…")));
                            open_device(path)
                        }
                        Link::Usb { port } => {
                            send(Event::Log(format!("Opening USB {}…", if port.is_empty() { "(finding the machine)" } else { port })));
                            open_usb(port)
                        }
                        Link::Demo => Ok(Box::new(DemoPort { out: Vec::new() }) as Box<dyn Port>),
                    };
                    let verified = opened.and_then(|mut p| {
                        send(Event::Log("Checking for a machine status reply…".into()));
                        confirm_connection(p.as_mut(), key.as_ref(), Duration::from_secs(5))?;
                        Ok(p)
                    });
                    match verified {
                        Ok(p) => {
                            port = Some(p);
                            let name = match &link {
                                Link::Rfcomm { mac, .. } => mac.clone(),
                                Link::Device { path } => path.clone(),
                                Link::Usb { port } => format!("USB {}", if port.is_empty() { "machine".to_string() } else { port.clone() }),
                                Link::Demo => "Demo machine".into(),
                            };
                            send(Event::Connected(name));
                        }
                        Err(e) => send(Event::Disconnected(e)),
                    }
                }
                Request::Cut { records, setup } => {
                    let Some(p) = port.as_mut() else {
                        send(Event::Log("Not connected: open the machine link before cutting.".into()));
                        continue;
                    };
                    // Tool prep: query position, reposition Y, home, and detect/seat the tool. Best
                    // effort — log what happens but don't abort the cut if a prep step is refused.
                    if setup.pressure != 0 {
                        send(Event::Log("Tool prep: position, home, detect…".into()));
                        let mut pos = None;
                        if p.write_all(&framed(&position_query_body(), key.as_ref())).is_ok() {
                            if let Ok(Some(reply)) = read_reply(p.as_mut(), key.as_ref(), Duration::from_secs(5)) {
                                send(Event::Log(format!("← pos {}", hex(&reply))));
                                pos = parse_position(&reply);
                            }
                        }
                        // If the carriage is past Y=8 in, bring it to 8 with a travel move.
                        if let Some((x, y)) = pos {
                            if y > 8.0 {
                                for body in [
                                    vec![DRAW_BEGIN, 0, 0, 0],
                                    multidraw_body(&[(VEC_MOVE, x, 8.0)]),
                                    vec![DRAW_END, 0, 0, 0],
                                ] {
                                    let _ = p.write_all(&framed(&body, key.as_ref()));
                                    let _ = read_reply(p.as_mut(), key.as_ref(), Duration::from_secs(10));
                                }
                            }
                        }
                        // Home the carriage, then detect/seat the installed tool.
                        for (label, body) in [
                            ("home", home_theta_body(HOME_INITIAL)),
                            ("detect tool", tool_detect_body()),
                        ] {
                            if p.write_all(&framed(&body, key.as_ref())).is_ok() {
                                match read_reply(p.as_mut(), key.as_ref(), Duration::from_secs(15)) {
                                    Ok(Some(reply)) => send(Event::Log(format!("← {label} {}", hex(&reply)))),
                                    Ok(None) => send(Event::Log(format!("   {label}: no reply"))),
                                    Err(e) => send(Event::Log(format!("   {label}: {e}"))),
                                }
                            }
                        }
                    }
                    let mut payloads = setup_payloads(&setup);
                    let setup_count = payloads.len();
                    payloads.extend(cut_job_payloads(&records, MULTIDRAW_MAX_RECORDS));
                    let total = payloads.len();
                    if setup.pressure == 0 {
                        send(Event::Log(
                            "Pressure is 0: sending motion only, the machine won't cut. Set a cut pressure in Machine settings."
                                .into(),
                        ));
                    } else {
                        send(Event::Log(format!(
                            "Tool setup: pressure {} counts, {}×{} in area ({} setup commands).",
                            setup.pressure, setup.width_in, setup.height_in, setup_count
                        )));
                    }
                    send(Event::Log(format!(
                        "Starting cut: {} records in {} commands ({}).",
                        records.len(),
                        total,
                        if key.is_some() { "encrypted" } else { "plain" }
                    )));
                    let mut stopped: Option<String> = None;
                    let mut disconnect: Option<String> = None;
                    'job: for (i, body) in payloads.iter().enumerate() {
                        let code = body[0];
                        let frame = match &key {
                            Some(k) => frame_encrypted(&frame_plain(body), k),
                            None => frame_plain(body),
                        };
                        send(Event::Log(format!(
                            "→ cut {}/{}: 0x{:02X} ({} bytes on the wire)",
                            i + 1,
                            total,
                            code,
                            frame.len()
                        )));
                        // The machine buffers draw records and reports its remaining space in the
                        // last byte of each ACK; when it's full it NACKs a batch. The original waits
                        // for the buffer to drain and retries the same command, up to ~60 s, rather
                        // than giving up. Do the same: resend on a NACK or a silence until it's taken.
                        let deadline = Instant::now() + Duration::from_secs(90);
                        let mut waited = false;
                        loop {
                            if let Err(e) = p.write_all(&frame) {
                                disconnect = Some(format!("Write failed during cut: {e}"));
                                break 'job;
                            }
                            match read_reply(p.as_mut(), key.as_ref(), Duration::from_secs(20)) {
                                Ok(Some(reply)) => {
                                    send(Event::Log(format!("← {}", hex(&reply))));
                                    match draw_ack(&reply, code) {
                                        Ok(()) => {
                                            if let Some(space) = reply.last() {
                                                send(Event::Log(format!("   accepted; machine buffer ~{space} left")));
                                            }
                                            break;
                                        }
                                        // NACK (reply starts 0x01): buffer full / not ready yet.
                                        Err(_) if reply.first() == Some(&0x01) && Instant::now() < deadline => {
                                            if !waited {
                                                send(Event::Log("   machine busy (buffer full) — waiting for it to catch up…".into()));
                                                waited = true;
                                            }
                                            std::thread::sleep(Duration::from_millis(500));
                                            continue;
                                        }
                                        Err(e) => {
                                            stopped = Some(e);
                                            break 'job;
                                        }
                                    }
                                }
                                Ok(None) if Instant::now() < deadline => {
                                    std::thread::sleep(Duration::from_millis(500));
                                    continue;
                                }
                                Ok(None) => {
                                    stopped = Some(format!("No reply to cut command {}/{} within 90 s.", i + 1, total));
                                    break 'job;
                                }
                                Err(e) => {
                                    disconnect = Some(e);
                                    break 'job;
                                }
                            }
                        }
                    }
                    if let Some(e) = disconnect {
                        port = None;
                        send(Event::Disconnected(e));
                    } else if let Some(e) = stopped {
                        send(Event::Log(format!("Cut stopped: {e}")));
                    } else {
                        send(Event::Log(format!("Cut job finished: all {total} commands acknowledged.")));
                    }
                }
                Request::ReadCalibration => {
                    let Some(p) = port.as_mut() else {
                        send(Event::Log("Not connected: can't read calibration.".into()));
                        continue;
                    };
                    send(Event::Log("Reading the machine's pressure calibration…".into()));
                    let mut rows = Vec::new();
                    let mut io_error = None;
                    for index in DRAG_CAL_FIRST..=DRAG_CAL_LAST {
                        let body = read_persistent_body(index);
                        let frame = match &key {
                            Some(k) => frame_encrypted(&frame_plain(&body), k),
                            None => frame_plain(&body),
                        };
                        if let Err(e) = p.write_all(&frame) {
                            io_error = Some(format!("Calibration read failed: {e}"));
                            break;
                        }
                        match read_reply(p.as_mut(), key.as_ref(), Duration::from_secs(5)) {
                            Ok(Some(reply)) => {
                                if let Some((idx, grams, counts)) = parse_persistent_pressure(&reply) {
                                    if idx == index {
                                        rows.push((grams, counts));
                                    }
                                }
                            }
                            Ok(None) => {}
                            Err(e) => {
                                io_error = Some(e);
                                break;
                            }
                        }
                    }
                    if let Some(e) = io_error {
                        port = None;
                        send(Event::Disconnected(e));
                    } else {
                        rows.sort_by_key(|r| r.0);
                        rows.dedup();
                        if rows.is_empty() {
                            send(Event::Log("The machine didn't return calibration rows; pressure stays a raw count.".into()));
                        } else {
                            send(Event::Log(format!(
                                "Read {} calibration rows: {}–{} g maps to {}–{} counts.",
                                rows.len(),
                                rows.first().map(|r| r.0).unwrap_or(0),
                                rows.last().map(|r| r.0).unwrap_or(0),
                                rows.iter().map(|r| r.1).min().unwrap_or(0),
                                rows.iter().map(|r| r.1).max().unwrap_or(0),
                            )));
                        }
                        send(Event::Calibration { rows });
                    }
                }
                Request::Disconnect => {
                    port = None;
                    send(Event::Disconnected("Disconnected.".into()));
                }
                Request::Send { cmd, theta } => {
                    let Some(p) = port.as_mut() else {
                        send(Event::Log("Not connected.".into()));
                        continue;
                    };
                    let body = payload(cmd, theta);
                    let frame = match &key {
                        // The original encrypts the *complete inner frame* (2-byte length + body),
                        // not the body alone (CricutDevice.exe 0x48f050 → 0x48de60). CUT-PROTOCOL.md.
                        Some(k) => frame_encrypted(&frame_plain(&body), k),
                        None => frame_plain(&body),
                    };
                    send(Event::Log(format!("→ {} : {}", cmd.label(), hex(&frame))));
                    if let Err(e) = p.write_all(&frame) {
                        port = None;
                        send(Event::Disconnected(format!("Write failed: {e}")));
                        continue;
                    }
                    match read_reply(p.as_mut(), key.as_ref(), Duration::from_secs(15)) {
                        Ok(Some(bytes)) => {
                            send(Event::Log(format!("← {}", hex(&bytes))));
                            let ok = bytes.first() == Some(&cmd.code());
                            if !ok {
                                send(Event::Log("Unexpected reply: it doesn't start with the command's code.".into()));
                            }
                            send(Event::Reply { cmd, status: bytes.get(1).copied(), bytes });
                        }
                        Ok(None) => send(Event::Log("No reply within 15 s.".into())),
                        Err(e) => {
                            port = None;
                            send(Event::Disconnected(e));
                        }
                    }
                }
            }
        }
    });
    MachineHandle { tx: req_tx, rx: ev_rx }
}

#[cfg(target_os = "linux")]
fn pair_bluetooth(mac: &str) -> Result<(), String> {
    let address = parse_mac(mac.trim()).ok_or("Enter a valid Bluetooth address before pairing.")?;
    let mac = address.map(|byte| format!("{byte:02X}")).join(":");
    let run = |seconds: &str, args: &[&str]| {
        std::process::Command::new("bluetoothctl")
            .args(["--timeout", seconds])
            .args(args)
            .output()
            .map_err(|error| format!("Couldn't start Bluetooth setup: {error}"))
    };
    let has_property = |output: &std::process::Output, property: &str| {
        String::from_utf8_lossy(&output.stdout)
            .lines()
            .any(|line| line.trim() == property)
    };
    let failure = |action: &str, output: &std::process::Output| {
        let stdout = String::from_utf8_lossy(&output.stdout);
        let stderr = String::from_utf8_lossy(&output.stderr);
        format!("Bluetooth {action} failed for {mac}. {} {}", stderr.trim(), stdout.trim())
    };

    let mut info = run("5", &["info", &mac])?;
    if !String::from_utf8_lossy(&info.stdout).lines().any(|line| {
        line.starts_with("Device ") && line.split_whitespace().nth(1) == Some(mac.as_str())
    }) {
        // A timed scan may exit with a timeout even when it found the machine.
        let scan = run("12", &["scan", "bredr"])?;
        info = run("5", &["info", &mac])?;
        if !info.status.success() {
            return Err(format!(
                "Couldn't find {mac}. Turn on Bluetooth and the machine, then try again. {}",
                failure("discovery", &scan)
            ));
        }
    }

    if !has_property(&info, "Paired: yes") {
        let pair = run("20", &["pair", &mac])?;
        info = run("5", &["info", &mac])?;
        if !has_property(&info, "Paired: yes") {
            return Err(failure("pairing", &pair));
        }
    }
    if !has_property(&info, "Trusted: yes") {
        let trust = run("5", &["trust", &mac])?;
        info = run("5", &["info", &mac])?;
        if !has_property(&info, "Trusted: yes") {
            return Err(failure("trust", &trust));
        }
    }
    Ok(())
}
fn read_exact_timeout(p: &mut dyn Port, n: usize, deadline: Instant) -> Result<Option<Vec<u8>>, String> {
    let mut out = Vec::with_capacity(n);
    let mut buf = [0u8; 600];
    while out.len() < n {
        let now = Instant::now();
        if now >= deadline {
            return Ok(None);
        }
        let want = (n - out.len()).min(buf.len());
        match p.read_some(&mut buf[..want], deadline - now) {
            Ok(0) => continue,
            Ok(k) => out.extend_from_slice(&buf[..k]),
            Err(e) => return Err(format!("Read failed: {e}")),
        }
    }
    Ok(Some(out))
}

/// Feed recorded reply bytes through the same reader the live link uses (for the behaviour check).
pub fn read_reply_from_bytes(bytes: &[u8], key: Option<&[u8; 32]>) -> Result<Option<Vec<u8>>, String> {
    struct Recorded(Vec<u8>);
    impl Port for Recorded {
        fn write_all(&mut self, _b: &[u8]) -> std::io::Result<()> {
            Ok(())
        }
        fn read_some(&mut self, buf: &mut [u8], _wait: Duration) -> std::io::Result<usize> {
            if self.0.is_empty() {
                return Err(std::io::Error::new(std::io::ErrorKind::UnexpectedEof, "no more recorded bytes"));
            }
            let n = self.0.len().min(buf.len());
            buf[..n].copy_from_slice(&self.0[..n]);
            self.0.drain(..n);
            Ok(n)
        }
    }
    let mut port = Recorded(bytes.to_vec());
    read_reply(&mut port, key, Duration::from_millis(50))
}

fn read_reply(p: &mut dyn Port, key: Option<&[u8; 32]>, wait: Duration) -> Result<Option<Vec<u8>>, String> {
    let deadline = Instant::now() + wait;
    let Some(h) = read_exact_timeout(p, 2, deadline)? else { return Ok(None) };
    let encrypted = h[0] & 0x80 != 0;
    let len = (((h[0] & 0x7f) as usize) << 8) | h[1] as usize;
    if len > MAX_PAYLOAD {
        return Err(format!("Reply says it is {len} bytes long; the most allowed is {MAX_PAYLOAD}."));
    }
    let Some(body) = read_exact_timeout(p, len, deadline)? else { return Ok(None) };
    if encrypted {
        match key {
            Some(k) => open_encrypted(&body, k).map(Some),
            None => Err("The machine replied encrypted; add the session key in Machine settings.".into()),
        }
    } else {
        Ok(Some(body))
    }
}

// ---------------------------------------------------------------- cut jobs
//
// How the original streams a cut (from CricutDevice.exe, written up in app/CUT-PROTOCOL.md):
// the job is BeginDraw (0x21), optional machine/speed/tool state, then a run of MultiDrawVector
// (0x43) batches, then EndDraw (0x22). Each MultiDrawVector carries records of
// `kind:u8, x:f32 LE, y:f32 LE`, with x and y in inches on the mat; kind 0x02 is a travel move
// (pen up) and kind 0x00 is a cut/draw line. Each command is sent on its own and the host waits
// for an acknowledgement (`00 <opcode> …`; `01 <opcode> …` is a rejection) before the next one.
// The draw opcodes are only encrypted when a session key is installed; otherwise they go in plain
// frames, exactly like the status commands the machine already answers.

pub const DRAW_BEGIN: u8 = 0x21;
pub const DRAW_END: u8 = 0x22;
pub const DRAW_MULTI: u8 = 0x43;
/// Vector kind: travel to a point with the tool up.
pub const VEC_MOVE: u8 = 0x02;
/// Vector kind: cut/draw a straight line to a point.
pub const VEC_LINE: u8 = 0x00;
/// A common per-batch record cap seen in the original (device field +0x13c); model/firmware can
/// lower it. We use it as the default batch size.
pub const MULTIDRAW_MAX_RECORDS: usize = 56;

/// One draw record: (kind, x, y) with x and y in inches on the mat.
pub type DrawRecord = (u8, f32, f32);

/// Turn mat contours (polylines, points in inches) into ordered draw records: a travel move to the
/// first point of each contour, then a cut line to each following point (closed contours get a final
/// line back to their first point), and finally a terminal travel move at the contour's last cutting
/// point. The original emits that terminal move-to for every contour (e.g. a square is
/// `02 00 00 00 00 02`); it returns the tool to the travel state at the end of the path.
pub fn contours_to_records(contours: &[Vec<(f32, f32)>], closed: bool) -> Vec<DrawRecord> {
    let mut recs = Vec::new();
    for c in contours {
        if c.is_empty() {
            continue;
        }
        recs.push((VEC_MOVE, c[0].0, c[0].1));
        if c.len() > 1 {
            for p in &c[1..] {
                recs.push((VEC_LINE, p.0, p.1));
            }
            // End point of the cut: back at the start for a closed contour, else the last point.
            let end = if closed { c[0] } else { *c.last().unwrap() };
            if closed {
                recs.push((VEC_LINE, end.0, end.1));
            }
            recs.push((VEC_MOVE, end.0, end.1));
        }
    }
    recs
}

/// Body of a MultiDrawVector (0x43) command: the code, then each record as `kind, x:f32 LE, y:f32 LE`.
pub fn multidraw_body(records: &[DrawRecord]) -> Vec<u8> {
    let mut v = Vec::with_capacity(1 + 9 * records.len());
    v.push(DRAW_MULTI);
    for &(k, x, y) in records {
        v.push(k);
        v.extend_from_slice(&x.to_le_bytes());
        v.extend_from_slice(&y.to_le_bytes());
    }
    v
}

// Setup commands sent before BeginDraw. These configure the tool so the firmware actually lowers
// the blade and applies force during cut (kind 0x00) vectors: without them the carriage traces the
// path with the tool up. Command codes and layouts from CricutDevice.exe (app/CUT-PROTOCOL.md).
pub const SET_MACHINE_STATE: u8 = 0x30;
pub const SET_SPEED_STATE: u8 = 0x31;
pub const SET_TOOL_STATE: u8 = 0x37;
pub const SET_FULL_TOOL_STATE: u8 = 0x3E;
pub const CPR_TOGGLE: u8 = 0x3F;

/// What the per-job setup needs: the work area in whole inches and the cut pressure (Z encoder
/// counts) that goes in SetMachineState. Pressure 0 means "don't set it" — the machine won't cut.
#[derive(Clone, Copy, Debug)]
pub struct CutSetup {
    pub width_in: u8,
    pub height_in: u8,
    pub pressure: u16,
}

/// SetMachineState (0x30) for a drag blade: `30 08 00 00 W H 00 00 00 00 00 <pressure:u16 BE>`.
/// Byte 1 `08` is the ordinary drag-blade flag/state; bytes 4–5 are the work-area width and height
/// in inches; byte 10 is the pressure-table selector (0 for drag blade); the last two bytes are the
/// cut pressure in Z encoder counts, big-endian.
pub fn set_machine_state_body(width_in: u8, height_in: u8, pressure: u16) -> Vec<u8> {
    vec![
        SET_MACHINE_STATE, 0x08, 0x00, 0x00, width_in, height_in, 0x00, 0x00, 0x00, 0x00, 0x00,
        (pressure >> 8) as u8, (pressure & 0xff) as u8,
    ]
}

/// SetSpeedState (0x31): the built-in drag-knife speed profile read from the original —
/// `31` then five `0F` bytes, then 20 big-endian u16 words (cut/move velocity, cut/move
/// acceleration; axis order X, Y, ZA, ZB, theta; each the profile value ×100).
pub fn set_speed_state_body() -> Vec<u8> {
    let words: [u16; 20] = [
        0x00D2, 0x00D2, 0x0190, 0x0190, 0x03E8, // cut velocity
        0x0190, 0x0190, 0x0190, 0x0190, 0x03E8, // move velocity
        0x0514, 0x0514, 0x07D0, 0x07D0, 0x09C4, // cut acceleration
        0x0F14, 0x0F14, 0x0F14, 0x0F14, 0x61A8, // move acceleration
    ];
    let mut v = vec![SET_SPEED_STATE, 0x0F, 0x0F, 0x0F, 0x0F, 0x0F];
    for w in words {
        v.extend_from_slice(&w.to_be_bytes());
    }
    v
}

/// SetToolState (0x37) for the built-in drag blade: flags `03`, then three zero big-endian f32
/// (lift threshold, parallel offset, lagging offset — all zero for the passive drag knife).
pub fn set_tool_state_body() -> Vec<u8> {
    let mut v = vec![SET_TOOL_STATE, 0x03];
    v.extend_from_slice(&[0u8; 12]);
    v
}

/// SetFullToolState (0x3E) for the drag blade: `3E 00 03`, three zero f32, then five zero node bytes.
pub fn set_full_tool_state_body() -> Vec<u8> {
    let mut v = vec![SET_FULL_TOOL_STATE, 0x00, 0x03];
    v.extend_from_slice(&[0u8; 12]); // three zero f32
    v.extend_from_slice(&[0u8; 5]); // five node bytes
    v
}

/// CprToggle (0x3F): `3F 00` for a drag blade (01 is only for the tangential blade, tool mode 3).
pub fn cpr_toggle_body() -> Vec<u8> {
    vec![CPR_TOGGLE, 0x00]
}

/// Write Persistent Memory (0xA1): `A1 <index:u16 BE> <value:u32 BE>`.
pub const WRITE_PERSISTENT: u8 = 0xA1;
pub fn write_persistent_body(index: u16, value: u32) -> Vec<u8> {
    let mut v = vec![WRITE_PERSISTENT, (index >> 8) as u8, (index & 0xff) as u8];
    v.extend_from_slice(&value.to_be_bytes());
    v
}

/// The four fixed persistent-memory writes the original makes between SetFullToolState (0x3E) and
/// CprToggle (0x3F) on the Maker (CricutDevice.exe 0x48edc0). They don't carry the pressure count;
/// they appear to configure the Z force/motor, and are likely needed for the blade to press down.
pub const ATHENA_SETUP_WRITES: [(u16, u32); 4] =
    [(0x005D, 10_000), (0x005F, 75_000), (0x0082, 10_000), (0x0084, 75_000)];

// Tool preparation, run before the cut setup (CricutDevice.exe 0x4a7280): query position (B9),
// optionally bring Y to 8.0, home the carriage (F0), and detect/seat the installed tool (EA). The
// EA tool-detect is how the machine identifies and seats the tool; skipping it may be why the blade
// never commits force. (app/CUT-INIT.md, prep trace.)
pub const POSITION_QUERY: u8 = 0xB9;
pub const HOME_THETA: u8 = 0xF0;
pub const TOOL_DETECT: u8 = 0xEA;
/// Home selector for an initial drag-knife selection: `F0 08`.
pub const HOME_INITIAL: u8 = 0x08;

pub fn position_query_body() -> Vec<u8> {
    vec![POSITION_QUERY]
}
pub fn home_theta_body(selector: u8) -> Vec<u8> {
    vec![HOME_THETA, selector]
}
pub fn tool_detect_body() -> Vec<u8> {
    vec![TOOL_DETECT]
}

/// Parse a B9 position reply `B9 <X:f32 BE> <Y:f32 BE>` into (x, y) inches. Needs ≥9 bytes.
pub fn parse_position(reply: &[u8]) -> Option<(f32, f32)> {
    if reply.len() < 9 || reply[0] != POSITION_QUERY {
        return None;
    }
    Some((
        f32::from_be_bytes([reply[1], reply[2], reply[3], reply[4]]),
        f32::from_be_bytes([reply[5], reply[6], reply[7], reply[8]]),
    ))
}

/// Frame a command body for the wire: plain, or (with a session key) the encrypted whole inner frame.
pub fn framed(body: &[u8], key: Option<&[u8; 32]>) -> Vec<u8> {
    match key {
        Some(k) => frame_encrypted(&frame_plain(body), k),
        None => frame_plain(body),
    }
}

// ---- pressure calibration (grams -> Z encoder counts), read from the machine ----
//
// The original converts a requested force in grams to the Z encoder-count value it puts in 0x30,
// using a calibration table it reads from the machine's persistent memory with the Read Persistent
// Memory command 0xA0. For the Maker/Athena drag blade the table is the 16 entries at indices
// 0x0021..=0x0030; in each reply, bytes 7–8 are grams and bytes 9–10 are counts (both u16 BE).
// Interpolating within the machine's own table keeps the count in the firmware's stable range —
// a raw guessed count makes the Z servo hunt. (app/TOOL-PRESSURE.md, calibration decode.)

pub const READ_PERSISTENT: u8 = 0xA0;
/// Persistent-memory indices holding the drag-blade (tool group 2) pressure calibration.
pub const DRAG_CAL_FIRST: u16 = 0x0021;
pub const DRAG_CAL_LAST: u16 = 0x0030;

/// Request body for Read Persistent Memory: `A0 <index:u16 BE>`.
pub fn read_persistent_body(index: u16) -> Vec<u8> {
    vec![READ_PERSISTENT, (index >> 8) as u8, (index & 0xff) as u8]
}

/// Parse an A0 reply body into (index, grams, counts). Needs ≥11 bytes and opcode 0xA0.
pub fn parse_persistent_pressure(reply: &[u8]) -> Option<(u16, u16, u16)> {
    if reply.len() < 11 || reply[0] != READ_PERSISTENT {
        return None;
    }
    Some((
        u16::from_be_bytes([reply[1], reply[2]]),
        u16::from_be_bytes([reply[7], reply[8]]),
        u16::from_be_bytes([reply[9], reply[10]]),
    ))
}

/// Convert a requested force in grams to Z encoder counts, interpolating a force-sorted calibration
/// table of (grams, counts). Saturates at the first and last rows; between two rows it does the
/// single-precision linear interpolation the original does, truncated toward zero. None if empty.
pub fn grams_to_counts(table: &[(u16, u16)], grams: u16) -> Option<u16> {
    if table.is_empty() {
        return None;
    }
    if grams <= table[0].0 {
        return Some(table[0].1);
    }
    let last = table[table.len() - 1];
    if grams >= last.0 {
        return Some(last.1);
    }
    for w in table.windows(2) {
        let (g0, c0) = w[0];
        let (g1, c1) = w[1];
        if grams >= g0 && grams <= g1 {
            if g1 == g0 {
                return Some(c1);
            }
            let t = (grams as f32 - g0 as f32) / (g1 as f32 - g0 as f32);
            let c = (t * (c1 as f32 - c0 as f32) + c0 as f32).trunc();
            return Some(c.clamp(0.0, u16::MAX as f32) as u16);
        }
    }
    Some(last.1)
}

/// The setup command payloads sent before BeginDraw, in the original's order: a pre-setup EndDraw to
/// reset the draw state, machine state (with pressure), speed, tool, full tool, the four Maker
/// persistent-memory writes, then CPR. Empty when `pressure` is 0 (nothing to configure).
pub fn setup_payloads(setup: &CutSetup) -> Vec<Vec<u8>> {
    if setup.pressure == 0 {
        return Vec::new();
    }
    let mut v = vec![
        vec![DRAW_END, 0, 0, 0], // pre-setup EndDraw: bring the draw state to a known point
        set_machine_state_body(setup.width_in.max(1), setup.height_in.max(1), setup.pressure),
        set_speed_state_body(),
        set_tool_state_body(),
        set_full_tool_state_body(),
    ];
    for (index, value) in ATHENA_SETUP_WRITES {
        v.push(write_persistent_body(index, value));
    }
    v.push(cpr_toggle_body());
    v
}

/// The ordered command *payloads* (unframed) of a cut job: BeginDraw, one or more MultiDrawVector
/// batches of at most `batch` (capped at [`MULTIDRAW_MAX_RECORDS`]) records, then EndDraw. Frame
/// each with [`frame_plain`], or [`frame_encrypted`] when a session key is installed.
pub fn cut_job_payloads(records: &[DrawRecord], batch: usize) -> Vec<Vec<u8>> {
    let batch = batch.clamp(1, MULTIDRAW_MAX_RECORDS);
    let mut out = vec![vec![DRAW_BEGIN, 0, 0, 0]];
    if records.is_empty() {
        out.push(multidraw_body(&[]));
    } else {
        for chunk in records.chunks(batch) {
            out.push(multidraw_body(chunk));
        }
    }
    out.push(vec![DRAW_END, 0, 0, 0]);
    out
}

/// Classify an (already unframed) reply to a draw command. The original's ACK is `00 <opcode> …`
/// and its rejection is `01 <opcode> …`; a plain reply may instead echo the opcode first, so both
/// orders are accepted as success. The raw reply is always logged by the caller so a live test can
/// confirm the exact shape.
pub fn draw_ack(reply: &[u8], code: u8) -> Result<(), String> {
    match (reply.first().copied(), reply.get(1).copied()) {
        (Some(0x00), Some(c)) if c == code => Ok(()),
        (Some(c), Some(0x00)) if c == code => Ok(()),
        (Some(c), _) if c == code => Ok(()), // opcode echoed, no status byte (as status replies do)
        (Some(0x01), _) => Err(format!("The machine rejected command 0x{code:02X} (NACK).")),
        (Some(other), _) => Err(format!("Unexpected reply 0x{other:02X} to command 0x{code:02X}.")),
        (None, _) => Err("Empty reply to a draw command.".into()),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn connection_check_requires_a_status_reply() {
        let wait = Duration::from_millis(10);
        let mut demo = DemoPort { out: Vec::new() };
        assert!(confirm_connection(&mut demo, None, wait).is_ok());

        let mut wrong_reply = DemoPort { out: frame_plain(&[Cmd::ReadErrorSeverity.code(), 0]) };
        assert!(confirm_connection(&mut wrong_reply, None, wait).is_err());
    }

    #[test]
    fn plain_frame() {
        assert_eq!(frame_plain(&[0x73]), vec![0x00, 0x01, 0x73]);
    }
    #[test]
    fn encrypted_roundtrip() {
        let key = [7u8; 32];
        let f = frame_encrypted(&[0x0A], &key);
        assert_eq!(f[0], 0x80);
        assert_eq!(f[1], 16);
        let back = open_encrypted(&f[2..], &key).unwrap();
        // first byte non-zero → first two bytes swapped, as the original does on replies
        assert_eq!(back[1], 0x0A);
    }
    #[test]
    fn block_padding_always_adds_a_block() {
        let key = [1u8; 32];
        assert_eq!(frame_encrypted(&[0u8; 16], &key).len(), 2 + 32);
    }
    #[test]
    fn move4d_layout() {
        let p = payload(Cmd::Move4D, 180.0);
        assert_eq!(p.len(), 18);
        assert_eq!(&p[..2], &[0xF1, 0x17]);
    }
    #[test]
    fn new_aes_request_shape() {
        let f = frame_plain(&payload(Cmd::NewAes, 0.0));
        assert_eq!(f.len(), 0x33);
        assert_eq!(&f[..3], &[0x00, 0x31, 0xC7]);
        assert!((2..=13).contains(&f[3]));
    }
    #[test]
    fn mac() {
        assert_eq!(parse_mac(DEFAULT_MAC), Some([0x00, 0x06, 0x66, 0x00, 0x12, 0x34]));
    }
    #[test]
    fn bluetooth_name_matches_the_label() {
        // the example address yields "Maker-1234" for the address ending 12:34
        assert_eq!(bluetooth_name(DEFAULT_MAC).as_deref(), Some("Maker-1234"));
    }

    #[test]
    fn records_move_lines_then_terminal_move() {
        let square = vec![vec![(0.0, 0.0), (1.0, 0.0), (1.0, 1.0), (0.0, 1.0)]];
        let recs = contours_to_records(&square, true);
        // 02 00 00 00 00 02 : move to start, 4 cut lines (incl. close), terminal move at start
        assert_eq!(recs.len(), 6);
        let kinds: Vec<u8> = recs.iter().map(|r| r.0).collect();
        assert_eq!(kinds, vec![VEC_MOVE, VEC_LINE, VEC_LINE, VEC_LINE, VEC_LINE, VEC_MOVE]);
        assert_eq!((recs[5].1, recs[5].2), (0.0, 0.0));
    }

    #[test]
    fn multidraw_record_layout() {
        // one line record to (1.0, 0.0): code, kind, x f32 LE, y f32 LE = 1 + 9 bytes
        let body = multidraw_body(&[(VEC_LINE, 1.0, 0.0)]);
        assert_eq!(body.len(), 10);
        assert_eq!(body[0], DRAW_MULTI);
        assert_eq!(body[1], VEC_LINE);
        assert_eq!(&body[2..6], &1.0f32.to_le_bytes());
        assert_eq!(&body[6..10], &0.0f32.to_le_bytes());
    }

    #[test]
    fn cut_job_brackets_batches_with_begin_and_end() {
        // 120 records with the 56-cap → 3 batches, plus BeginDraw and EndDraw = 5 commands
        let recs: Vec<DrawRecord> = (0..120).map(|i| (VEC_LINE, i as f32, 0.0)).collect();
        let cmds = cut_job_payloads(&recs, MULTIDRAW_MAX_RECORDS);
        assert_eq!(cmds.len(), 5);
        assert_eq!(cmds[0], vec![DRAW_BEGIN, 0, 0, 0]);
        assert_eq!(*cmds.last().unwrap(), vec![DRAW_END, 0, 0, 0]);
        assert_eq!(cmds[1][0], DRAW_MULTI);
        // first two batches are full (56 records → 1 + 9*56 bytes), the third holds the remaining 8
        assert_eq!(cmds[1].len(), 1 + 9 * 56);
        assert_eq!(cmds[2].len(), 1 + 9 * 56);
        assert_eq!(cmds[3].len(), 1 + 9 * 8);
    }

    #[test]
    fn draw_ack_accepts_both_orders_and_flags_nack() {
        assert!(draw_ack(&[0x00, DRAW_MULTI, 0, 0, 0], DRAW_MULTI).is_ok());
        assert!(draw_ack(&[DRAW_MULTI, 0x00], DRAW_MULTI).is_ok());
        assert!(draw_ack(&[0x01, DRAW_MULTI], DRAW_MULTI).is_err());
        assert!(draw_ack(&[0x99], DRAW_MULTI).is_err());
    }

    #[test]
    fn setup_commands_have_the_traced_layout() {
        let m = set_machine_state_body(12, 12, 150);
        // 30 08 00 00 0C 0C 00 00 00 00 00 <pressure BE> ; 13 bytes, pressure in the last two
        assert_eq!(m.len(), 13);
        assert_eq!(&m[..6], &[0x30, 0x08, 0x00, 0x00, 12, 12]);
        assert_eq!(&m[11..], &150u16.to_be_bytes());
        assert_eq!(set_speed_state_body().len(), 6 + 40); // 31, five 0F, 20 u16
        assert_eq!(set_speed_state_body()[0], 0x31);
        assert_eq!(set_tool_state_body(), {
            let mut v = vec![0x37, 0x03];
            v.extend_from_slice(&[0u8; 12]);
            v
        });
        assert_eq!(set_full_tool_state_body().len(), 20); // 3E 00 03 + 3 f32 + 5 nodes
        assert_eq!(&set_full_tool_state_body()[..3], &[0x3E, 0x00, 0x03]);
        assert_eq!(cpr_toggle_body(), vec![0x3F, 0x00]);
    }

    #[test]
    fn setup_sequence_matches_the_original_and_is_skipped_at_zero_pressure() {
        let s = setup_payloads(&CutSetup { width_in: 12, height_in: 12, pressure: 1039 });
        let codes: Vec<u8> = s.iter().map(|c| c[0]).collect();
        // pre-EndDraw, machine/speed/tool/full-tool, four A1 writes, CPR
        assert_eq!(codes, vec![0x22, 0x30, 0x31, 0x37, 0x3E, 0xA1, 0xA1, 0xA1, 0xA1, 0x3F]);
        // the four A1 writes carry the original's fixed index/value pairs
        assert_eq!(write_persistent_body(0x005D, 10_000), vec![0xA1, 0x00, 0x5D, 0x00, 0x00, 0x27, 0x10]);
        assert_eq!(write_persistent_body(0x005F, 75_000), vec![0xA1, 0x00, 0x5F, 0x00, 0x01, 0x24, 0xF8]);
        assert!(setup_payloads(&CutSetup { width_in: 6, height_in: 6, pressure: 0 }).is_empty());
    }

    #[test]
    fn prep_builders_and_position_parse() {
        assert_eq!(position_query_body(), vec![0xB9]);
        assert_eq!(home_theta_body(HOME_INITIAL), vec![0xF0, 0x08]);
        assert_eq!(tool_detect_body(), vec![0xEA]);
        // B9 reply: opcode, X f32 BE, Y f32 BE
        let mut reply = vec![0xB9];
        reply.extend_from_slice(&1.5f32.to_be_bytes());
        reply.extend_from_slice(&9.0f32.to_be_bytes());
        assert_eq!(parse_position(&reply), Some((1.5, 9.0)));
        assert_eq!(parse_position(&[0xB9, 0, 0]), None);
    }

    #[test]
    fn persistent_request_and_reply_parse() {
        assert_eq!(read_persistent_body(0x0021), vec![0xA0, 0x00, 0x21]);
        // A0 reply: opcode, index BE, word1(4), grams BE, counts BE
        let reply = [0xA0, 0x00, 0x27, 0x11, 0x22, 0x33, 0x44, 0x00, 0xA0, 0x03, 0xE8];
        assert_eq!(parse_persistent_pressure(&reply), Some((0x0027, 160, 1000)));
        assert_eq!(parse_persistent_pressure(&[0xA0, 0x00]), None); // too short
        assert_eq!(parse_persistent_pressure(&[0x0B, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0]), None); // wrong opcode
    }

    #[test]
    fn grams_interpolate_and_saturate() {
        let table = [(100u16, 800u16), (160, 1000), (200, 1080), (300, 1400)];
        assert_eq!(grams_to_counts(&table, 50), Some(800)); // below first → first count
        assert_eq!(grams_to_counts(&table, 100), Some(800)); // exact first
        assert_eq!(grams_to_counts(&table, 180), Some(1040)); // between 160→1000 and 200→1080, t=0.5
        assert_eq!(grams_to_counts(&table, 300), Some(1400)); // exact last
        assert_eq!(grams_to_counts(&table, 999), Some(1400)); // above last → last count
        assert_eq!(grams_to_counts(&[], 180), None);
    }

    #[test]
    fn encrypted_cut_frame_wraps_the_inner_frame() {
        // parity with the original: the encrypted payload is frame_plain(body), not body alone
        let key = [3u8; 32];
        let body = vec![DRAW_BEGIN, 0, 0, 0];
        let inner = frame_plain(&body); // 00 04 21 00 00 00
        let framed = frame_encrypted(&inner, &key);
        let back = open_encrypted(&framed[2..], &key).unwrap();
        // inner frame decrypts back; first byte is the length high byte (0x00), so no swap
        assert_eq!(&back[..inner.len()], &inner[..]);
    }
}
