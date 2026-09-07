//! Raw AES-GCM/CBC/CTR encrypt/decrypt over key bytes + params, with no JS/rquickjs types in
//! sight - callers (`crypto::key`, `crypto::module`) own translating [`CipherError`] into the
//! right named `DOMException`.

use aes::{
    Aes128, Aes192, Aes256,
    cipher::{
        Array, BlockCipherEncrypt, BlockModeDecrypt, BlockModeEncrypt, BlockSizeUser, KeyInit,
        KeyIvInit, block_padding::Pkcs7, consts::U12, consts::U16,
    },
};
use aes_gcm::{
    AesGcm,
    aead::{Aead, Payload},
};

type Aes128Gcm = aes_gcm::Aes128Gcm;
type Aes192Gcm = AesGcm<Aes192, U12>;
type Aes256Gcm = aes_gcm::Aes256Gcm;

/// Builds a 12-byte GCM nonce from `iv`. Callers must have already checked `iv.len() == 12`
/// (every call site here does, via the `WrongIvLength` guard at the top of `gcm_encrypt`/
/// `gcm_decrypt`) - the conversion is infallible at that point.
fn gcm_nonce(iv: &[u8]) -> Array<u8, U12> {
    Array::try_from(iv).expect("iv length already validated as 12 bytes")
}

type Aes128CbcEnc = cbc::Encryptor<Aes128>;
type Aes192CbcEnc = cbc::Encryptor<Aes192>;
type Aes256CbcEnc = cbc::Encryptor<Aes256>;
type Aes128CbcDec = cbc::Decryptor<Aes128>;
type Aes192CbcDec = cbc::Decryptor<Aes192>;
type Aes256CbcDec = cbc::Decryptor<Aes256>;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CipherError {
    /// Key material isn't 16/24/32 bytes (AES-128/192/256).
    WrongKeyLength,
    /// GCM's IV must be exactly 12 bytes for this implementation (see the module-level note
    /// below on why arbitrary-length IVs aren't supported yet).
    WrongIvLength,
    /// CTR's `counter` block must be exactly 16 bytes.
    WrongCounterLength,
    /// GCM's `tagLength` must be 128 (see the same note as `WrongIvLength`).
    UnsupportedTagLength,
    /// AEAD authentication failed (wrong key, tampered ciphertext/tag/additionalData).
    AuthenticationFailed,
    /// CBC padding was invalid on decrypt (wrong key, or genuinely corrupt ciphertext).
    InvalidPadding,
}

/// GCM encrypt. Only a 12-byte IV and a 128-bit tag are supported: the spec allows other
/// lengths (arbitrary IVs via GHASH-derived nonces, truncated tags down to 32 bits), but those
/// need lower-level `AeadInOut`/generic-tag-size plumbing this crate doesn't wire up yet - every
/// real-world caller uses the 96-bit-IV/128-bit-tag case this covers.
pub fn gcm_encrypt(
    key: &[u8],
    iv: &[u8],
    aad: Option<&[u8]>,
    tag_length_bits: u16,
    plaintext: &[u8],
) -> Result<Vec<u8>, CipherError> {
    if tag_length_bits != 128 {
        return Err(CipherError::UnsupportedTagLength);
    }
    if iv.len() != 12 {
        return Err(CipherError::WrongIvLength);
    }

    let payload = Payload {
        msg: plaintext,
        aad: aad.unwrap_or(&[]),
    };

    match key.len() {
        16 => {
            let cipher = Aes128Gcm::new_from_slice(key).map_err(|_| CipherError::WrongKeyLength)?;
            let nonce = gcm_nonce(iv);
            cipher
                .encrypt(&nonce, payload)
                .map_err(|_| CipherError::AuthenticationFailed)
        }
        24 => {
            let cipher = Aes192Gcm::new_from_slice(key).map_err(|_| CipherError::WrongKeyLength)?;
            let nonce = gcm_nonce(iv);
            cipher
                .encrypt(&nonce, payload)
                .map_err(|_| CipherError::AuthenticationFailed)
        }
        32 => {
            let cipher = Aes256Gcm::new_from_slice(key).map_err(|_| CipherError::WrongKeyLength)?;
            let nonce = gcm_nonce(iv);
            cipher
                .encrypt(&nonce, payload)
                .map_err(|_| CipherError::AuthenticationFailed)
        }
        _ => Err(CipherError::WrongKeyLength),
    }
}

/// GCM decrypt. `ciphertext` is expected in the `ciphertext || tag` layout `gcm_encrypt`
/// produces (the tag is the trailing 16 bytes) - matching what WebCrypto's `decrypt()` expects.
pub fn gcm_decrypt(
    key: &[u8],
    iv: &[u8],
    aad: Option<&[u8]>,
    tag_length_bits: u16,
    ciphertext: &[u8],
) -> Result<Vec<u8>, CipherError> {
    if tag_length_bits != 128 {
        return Err(CipherError::UnsupportedTagLength);
    }
    if iv.len() != 12 {
        return Err(CipherError::WrongIvLength);
    }

    let payload = Payload {
        msg: ciphertext,
        aad: aad.unwrap_or(&[]),
    };

    match key.len() {
        16 => {
            let cipher = Aes128Gcm::new_from_slice(key).map_err(|_| CipherError::WrongKeyLength)?;
            let nonce = gcm_nonce(iv);
            cipher
                .decrypt(&nonce, payload)
                .map_err(|_| CipherError::AuthenticationFailed)
        }
        24 => {
            let cipher = Aes192Gcm::new_from_slice(key).map_err(|_| CipherError::WrongKeyLength)?;
            let nonce = gcm_nonce(iv);
            cipher
                .decrypt(&nonce, payload)
                .map_err(|_| CipherError::AuthenticationFailed)
        }
        32 => {
            let cipher = Aes256Gcm::new_from_slice(key).map_err(|_| CipherError::WrongKeyLength)?;
            let nonce = gcm_nonce(iv);
            cipher
                .decrypt(&nonce, payload)
                .map_err(|_| CipherError::AuthenticationFailed)
        }
        _ => Err(CipherError::WrongKeyLength),
    }
}

pub fn cbc_encrypt(key: &[u8], iv: &[u8], plaintext: &[u8]) -> Result<Vec<u8>, CipherError> {
    if iv.len() != 16 {
        return Err(CipherError::WrongIvLength);
    }

    Ok(match key.len() {
        16 => Aes128CbcEnc::new_from_slices(key, iv)
            .map_err(|_| CipherError::WrongKeyLength)?
            .encrypt_padded_vec::<Pkcs7>(plaintext),
        24 => Aes192CbcEnc::new_from_slices(key, iv)
            .map_err(|_| CipherError::WrongKeyLength)?
            .encrypt_padded_vec::<Pkcs7>(plaintext),
        32 => Aes256CbcEnc::new_from_slices(key, iv)
            .map_err(|_| CipherError::WrongKeyLength)?
            .encrypt_padded_vec::<Pkcs7>(plaintext),
        _ => return Err(CipherError::WrongKeyLength),
    })
}

pub fn cbc_decrypt(key: &[u8], iv: &[u8], ciphertext: &[u8]) -> Result<Vec<u8>, CipherError> {
    if iv.len() != 16 {
        return Err(CipherError::WrongIvLength);
    }

    match key.len() {
        16 => Aes128CbcDec::new_from_slices(key, iv)
            .map_err(|_| CipherError::WrongKeyLength)?
            .decrypt_padded_vec::<Pkcs7>(ciphertext)
            .map_err(|_| CipherError::InvalidPadding),
        24 => Aes192CbcDec::new_from_slices(key, iv)
            .map_err(|_| CipherError::WrongKeyLength)?
            .decrypt_padded_vec::<Pkcs7>(ciphertext)
            .map_err(|_| CipherError::InvalidPadding),
        32 => Aes256CbcDec::new_from_slices(key, iv)
            .map_err(|_| CipherError::WrongKeyLength)?
            .decrypt_padded_vec::<Pkcs7>(ciphertext)
            .map_err(|_| CipherError::InvalidPadding),
        _ => Err(CipherError::WrongKeyLength),
    }
}

/// AES-CTR encrypt/decrypt (the same XOR operation both ways). `counter` is the 16-byte initial
/// counter block; `length_bits` (1-128) is how many of its low-order bits actually form the
/// wrapping counter - the rest of the block is a fixed prefix that must never change, per
/// WebCrypto's `AesCtrParams`. The generic `ctr` crate only implements the `length_bits == 128`
/// case (it wraps the entire 128-bit block linearly), so this drives the AES block cipher
/// directly instead.
pub fn ctr_encrypt_decrypt(
    key: &[u8],
    counter: &[u8; 16],
    length_bits: u8,
    data: &[u8],
) -> Result<Vec<u8>, CipherError> {
    match key.len() {
        16 => Ok(ctr_xor::<Aes128>(
            Aes128::new_from_slice(key).map_err(|_| CipherError::WrongKeyLength)?,
            counter,
            length_bits,
            data,
        )),
        24 => Ok(ctr_xor::<Aes192>(
            Aes192::new_from_slice(key).map_err(|_| CipherError::WrongKeyLength)?,
            counter,
            length_bits,
            data,
        )),
        32 => Ok(ctr_xor::<Aes256>(
            Aes256::new_from_slice(key).map_err(|_| CipherError::WrongKeyLength)?,
            counter,
            length_bits,
            data,
        )),
        _ => Err(CipherError::WrongKeyLength),
    }
}

fn ctr_xor<C: BlockCipherEncrypt + BlockSizeUser<BlockSize = U16>>(
    cipher: C,
    counter: &[u8; 16],
    length_bits: u8,
    data: &[u8],
) -> Vec<u8> {
    // The high `128 - length_bits` bits of `counter` are a fixed prefix; the low `length_bits`
    // bits are the actual counter, wrapping at 2^length_bits rather than 2^128.
    let counter_int = u128::from_be_bytes(*counter);
    let prefix_mask: u128 = if length_bits >= 128 {
        0
    } else {
        !0u128 << length_bits
    };
    let prefix = counter_int & prefix_mask;
    let initial_low = counter_int & !prefix_mask;
    let modulus: u128 = if length_bits >= 128 {
        0
    } else {
        1u128 << length_bits
    };

    let mut out = Vec::with_capacity(data.len());
    for (block_index, chunk) in data.chunks(16).enumerate() {
        let low = if modulus == 0 {
            initial_low.wrapping_add(block_index as u128)
        } else {
            initial_low.wrapping_add(block_index as u128) % modulus
        };
        let block_int = prefix | low;
        let mut block: Array<u8, U16> = Array::from(block_int.to_be_bytes());
        cipher.encrypt_block(&mut block);

        for (d, k) in chunk.iter().zip(block.iter()) {
            out.push(d ^ k);
        }
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn ctr_low_bits_wrap_at_2_pow_length_without_carrying_into_prefix() {
        let key = [0x2bu8; 16];
        let mut counter = [0x11u8; 16]; // arbitrary non-zero fixed prefix throughout.
        counter[15] = 0xff; // low byte (length_bits == 8) is about to wrap on the next block.
        let data = vec![0x41u8; 32]; // 2 blocks.

        let out = ctr_encrypt_decrypt(&key, &counter, 8, &data).unwrap();
        let cipher = Aes128::new_from_slice(&key).unwrap();

        let keystream_block = |block: [u8; 16]| -> Array<u8, U16> {
            let mut arr: Array<u8, U16> = Array::from(block);
            cipher.encrypt_block(&mut arr);
            arr
        };

        // Block 0: counter unchanged (low byte still 0xff).
        let ks0 = keystream_block(counter);
        let expected0: Vec<u8> = ks0.iter().zip(&data[0..16]).map(|(k, d)| k ^ d).collect();
        assert_eq!(&out[0..16], &expected0[..]);

        // Block 1: low byte wraps 0xff -> 0x00; the rest of the block (the fixed prefix) must
        // stay exactly as in the original `counter`, not carry into byte 14.
        let mut wrapped = counter;
        wrapped[15] = 0x00;
        let ks1 = keystream_block(wrapped);
        let expected1: Vec<u8> = ks1.iter().zip(&data[16..32]).map(|(k, d)| k ^ d).collect();
        assert_eq!(&out[16..32], &expected1[..]);
    }

    #[test]
    fn ctr_round_trips() {
        let key = [0x11u8; 32];
        let counter = [0x00u8; 16];
        let data = b"the quick brown fox jumps over the lazy dog!!!!".to_vec();

        let ciphertext = ctr_encrypt_decrypt(&key, &counter, 64, &data).unwrap();
        assert_ne!(ciphertext, data);
        let plaintext = ctr_encrypt_decrypt(&key, &counter, 64, &ciphertext).unwrap();
        assert_eq!(plaintext, data);
    }

    #[test]
    fn ctr_preserves_fixed_prefix_bits() {
        // A non-trivial prefix (top 8 bytes all 0xAA) must never change across blocks even
        // though the low 64 bits wrap.
        let key = [0x99u8; 16];
        let mut counter = [0u8; 16];
        counter[..8].copy_from_slice(&[0xAA; 8]);
        let data = vec![0u8; 64]; // 4 blocks - encrypting zeros exposes the raw keystream.

        let keystream = ctr_encrypt_decrypt(&key, &counter, 64, &data).unwrap();

        // Recompute each block's keystream directly and confirm the prefix (top 8 bytes of the
        // counter block actually fed to the block cipher) never changes.
        let cipher = Aes128::new_from_slice(&key).unwrap();
        for block_index in 0..4u128 {
            let mut block = counter;
            let low = block_index.to_be_bytes();
            block[8..].copy_from_slice(&low[8..]);
            let mut arr: Array<u8, U16> = Array::from(block);
            cipher.encrypt_block(&mut arr);
            assert_eq!(&keystream[block_index as usize * 16..][..16], &arr[..]);
        }
    }

    #[test]
    fn gcm_round_trips_and_detects_tampering() {
        let key = [0x42u8; 16];
        let iv = [0x24u8; 12];
        let aad = b"header";
        let plaintext = b"hello world! this is my plaintext.";

        let ciphertext = gcm_encrypt(&key, &iv, Some(aad), 128, plaintext).unwrap();
        let recovered = gcm_decrypt(&key, &iv, Some(aad), 128, &ciphertext).unwrap();
        assert_eq!(recovered, plaintext);

        // Wrong AAD.
        assert_eq!(
            gcm_decrypt(&key, &iv, Some(b"tampered"), 128, &ciphertext),
            Err(CipherError::AuthenticationFailed)
        );

        // Tampered ciphertext.
        let mut tampered = ciphertext.clone();
        tampered[0] ^= 1;
        assert_eq!(
            gcm_decrypt(&key, &iv, Some(aad), 128, &tampered),
            Err(CipherError::AuthenticationFailed)
        );

        // Wrong key.
        let wrong_key = [0x43u8; 16];
        assert_eq!(
            gcm_decrypt(&wrong_key, &iv, Some(aad), 128, &ciphertext),
            Err(CipherError::AuthenticationFailed)
        );
    }

    #[test]
    fn cbc_round_trips_non_block_aligned_plaintext() {
        let key = [0x42u8; 32];
        let iv = [0x24u8; 16];
        let plaintext = b"hello world! this is my plaintext.";

        let ciphertext = cbc_encrypt(&key, &iv, plaintext).unwrap();
        assert_eq!(ciphertext.len() % 16, 0);
        let recovered = cbc_decrypt(&key, &iv, &ciphertext).unwrap();
        assert_eq!(recovered, plaintext);
    }

    #[test]
    fn cbc_rejects_invalid_padding() {
        let key = [0x42u8; 16];
        let iv = [0x24u8; 16];
        let ciphertext = cbc_encrypt(&key, &iv, b"0123456789abcdef").unwrap();

        let mut tampered = ciphertext.clone();
        let last = tampered.len() - 1;
        tampered[last] ^= 1;

        assert_eq!(
            cbc_decrypt(&key, &iv, &tampered),
            Err(CipherError::InvalidPadding)
        );
    }
}
