use lazy_static::lazy_static;
use aes_gcm::{
    aead::{consts::{B0, B1}, Aead, AeadCore, KeyInit, OsRng}, aes::cipher::typenum::{UInt, UTerm}, AeadInPlace, Aes256Gcm, Key, Nonce
};

#[doc(hidden)]
const KEY_BYTES: [u8; 32] = *include_bytes!("key.txt");

lazy_static! {
    static ref KEY: Key<Aes256Gcm> = KEY_BYTES.into();
    static ref CIPHER: Aes256Gcm = Aes256Gcm::new(&KEY);
}

type NonceArray = Nonce<UInt<UInt<UInt<UInt<UTerm, B1>, B1>, B0>, B0>>;

/// Encrypt `text` in place and return a nonce
pub fn encrypt(text: &str, output: &mut Vec<u8>) -> anyhow::Result<NonceArray> {
    let nonce = Aes256Gcm::generate_nonce(&mut OsRng);
    let bytes = CIPHER.encrypt(&nonce, text.as_bytes()).map_err(|e| anyhow::anyhow!("failed to encrypt: {e}"))?;
    *output = bytes;

    Ok(nonce)
}

/// Return decrypted text from `text` and `nonce`
pub fn decrypt(data: &[u8], nonce: &NonceArray) -> anyhow::Result<String> {
    let bytes = CIPHER.decrypt(nonce, data).map_err(|e| anyhow::anyhow!("failed to decrypt: {e}"))?;
    Ok(String::from_utf8(bytes)?)
}