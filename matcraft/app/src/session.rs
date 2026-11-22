//! Encrypted wire session for the Maker: X25519 handshake + AEGIS AEAD.
//!
//! CLEAN-ROOM NOTE: this encodes device-protocol details derived from reading Cricut's binary
//! (see the `binary-check` analysis). It is included in MatCraft at the user's explicit direction;
//! it crosses the clean-room boundary that otherwise keeps MatCraft freely distributable.
//!
//! WHAT IS SOLID (from the capture + binary analysis):
//! - The handshake is X25519 ECDH — the plaintext `c7` 49→48-byte exchange carries the public keys.
//! - The per-frame wire cipher is an AEAD with a 16-byte tag, deterministic within a session
//!   (a fixed/counter nonce; no nonce is sent on the wire). AES-round-based (AEGIS family).
//!
//! WHAT IS A HYPOTHESIS TO CONFIRM against a captured (plaintext, ciphertext) frame pair — the
//! plaintext is available from the `NtWriteFile` capture, the ciphertext from the matching USBPcap:
//! - AEGIS variant: AEGIS-256 is chosen here (its 32-byte key matches the X25519 shared secret,
//!   16-byte tag matches the wire overhead). Could be AEGIS-128L with a derived 16-byte key.
//! - Key derivation: the X25519 shared secret is used directly as the key (could need a KDF step).
//! - Nonce: a per-direction counter starting at 0 (could be a fixed constant or a different width).
//! - Frame body layout: `tag(16) || ciphertext` (the README reads the poll as "16-B tag + 0-B body").
//!
//! [`validate_against_capture`] is the hook to confirm/tune these four parameters on hardware before
//! the cut path is switched over to the encrypted session. Until then this module is NOT wired into
//! the live cut (which still uses the plaintext command stream in `machine.rs`).

use aegis::aegis256::Aegis256;
use x25519_dalek::{EphemeralSecret, PublicKey};

/// New-AES / session-start command code.
pub const CMD_NEW_SESSION: u8 = 0xC7;
/// The plaintext handshake frame carried a 49-byte body (observed `00 31` length word).
pub const C7_BODY_LEN: usize = 49;

/// Our side of the X25519 handshake: an ephemeral keypair.
pub struct Handshake {
    secret: EphemeralSecret,
    public: PublicKey,
}

impl Handshake {
    pub fn new() -> Self {
        let secret = EphemeralSecret::random();
        let public = PublicKey::from(&secret);
        Self { secret, public }
    }

    /// Our 32-byte X25519 public key.
    pub fn public_key(&self) -> [u8; 32] {
        *self.public.as_bytes()
    }

    /// The plaintext `c7` body to send (unframed): `C7` + our 32-byte public key, padded to the
    /// observed 49-byte body. Frame it with `machine::frame_plain` before writing.
    pub fn request_body(&self) -> Vec<u8> {
        let mut v = Vec::with_capacity(C7_BODY_LEN);
        v.push(CMD_NEW_SESSION);
        v.extend_from_slice(self.public.as_bytes());
        v.resize(C7_BODY_LEN, 0);
        v
    }

    /// Complete the handshake from the machine's reply body (observed 48 bytes; opens with the
    /// machine's 32-byte X25519 public key). Returns the established session.
    pub fn complete(self, reply: &[u8]) -> Result<Session, String> {
        if reply.len() < 32 {
            return Err(format!("handshake reply too short: {} bytes (need ≥32)", reply.len()));
        }
        let mut their = [0u8; 32];
        their.copy_from_slice(&reply[..32]);
        let shared = self.secret.diffie_hellman(&PublicKey::from(their));
        Ok(Session::new(*shared.as_bytes()))
    }
}

impl Default for Handshake {
    fn default() -> Self {
        Self::new()
    }
}

/// An established AEGIS session. Key derivation and nonce scheme are the hypotheses noted in the
/// module docs; confirm with [`validate_against_capture`] before relying on it.
pub struct Session {
    key: [u8; 32],
    send_ctr: u64,
    recv_ctr: u64,
}

impl Session {
    pub fn new(key: [u8; 32]) -> Self {
        Self { key, send_ctr: 0, recv_ctr: 0 }
    }

    /// Nonce for a message counter: the counter little-endian in the low 8 bytes of a 32-byte nonce.
    pub fn nonce(ctr: u64) -> [u8; 32] {
        let mut n = [0u8; 32];
        n[..8].copy_from_slice(&ctr.to_le_bytes());
        n
    }

    /// Encrypt one plaintext message into an (unframed) wire body: `tag(16) || ciphertext`.
    pub fn seal(&mut self, plaintext: &[u8]) -> Vec<u8> {
        let nonce = Self::nonce(self.send_ctr);
        self.send_ctr = self.send_ctr.wrapping_add(1);
        let cipher = Aegis256::<16>::new(&self.key, &nonce);
        let (ct, tag) = cipher.encrypt(plaintext, &[]);
        let mut body = Vec::with_capacity(16 + ct.len());
        body.extend_from_slice(&tag);
        body.extend_from_slice(&ct);
        body
    }

    /// Decrypt one (unframed) wire body `tag(16) || ciphertext`.
    pub fn open(&mut self, body: &[u8]) -> Result<Vec<u8>, String> {
        if body.len() < 16 {
            return Err("frame shorter than the 16-byte tag".into());
        }
        let (tag, ct) = body.split_at(16);
        let mut tag16 = [0u8; 16];
        tag16.copy_from_slice(tag);
        let nonce = Self::nonce(self.recv_ctr);
        let cipher = Aegis256::<16>::new(&self.key, &nonce);
        let pt = cipher
            .decrypt(ct, &tag16, &[])
            .map_err(|_| "AEGIS tag verification failed (wrong key/nonce/variant?)".to_string())?;
        self.recv_ctr = self.recv_ctr.wrapping_add(1);
        Ok(pt)
    }
}

/// Validate the session construction against a real captured frame. Given the session key, the
/// message counter, the encrypted wire body (`tag || ct`), and the known plaintext (from the
/// `NtWriteFile` capture), returns true if this construction reproduces the plaintext. Run this on
/// Windows with a captured (plaintext, ciphertext) pair to confirm the AEGIS variant / key / nonce
/// before switching the cut path to the encrypted session.
pub fn validate_against_capture(
    key: &[u8; 32],
    ctr: u64,
    wire_body: &[u8],
    expected_plaintext: &[u8],
) -> bool {
    if wire_body.len() < 16 {
        return false;
    }
    let (tag, ct) = wire_body.split_at(16);
    let mut tag16 = [0u8; 16];
    tag16.copy_from_slice(tag);
    let cipher = Aegis256::<16>::new(key, &Session::nonce(ctr));
    matches!(cipher.decrypt(ct, &tag16, &[]), Ok(pt) if pt == expected_plaintext)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn handshake_request_shape() {
        let hs = Handshake::new();
        let body = hs.request_body();
        assert_eq!(body.len(), C7_BODY_LEN);
        assert_eq!(body[0], CMD_NEW_SESSION);
        assert_eq!(&body[1..33], &hs.public_key()[..]);
    }

    #[test]
    fn two_parties_agree_on_a_key() {
        // Both sides run the handshake; the shared secret (session key) must match.
        let a = Handshake::new();
        let b = Handshake::new();
        let a_pub = a.public_key();
        let b_pub = b.public_key();
        let sa = a.complete(&b_pub).unwrap();
        let sb = b.complete(&a_pub).unwrap();
        assert_eq!(sa.key, sb.key);
    }

    #[test]
    fn aegis_frame_round_trips() {
        let key = [9u8; 32];
        let mut enc = Session::new(key);
        let mut dec = Session::new(key);
        let msg = b"PBMatCutExecutionPlan bytes...";
        let frame = enc.seal(msg);
        assert_eq!(frame.len(), 16 + msg.len()); // tag || ciphertext
        assert_eq!(dec.open(&frame).unwrap(), msg);
    }

    #[test]
    fn open_rejects_a_tampered_frame() {
        let key = [4u8; 32];
        let mut enc = Session::new(key);
        let mut frame = enc.seal(b"hello");
        frame[20] ^= 0xff;
        assert!(Session::new(key).open(&frame).is_err());
    }
}
