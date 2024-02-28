//! Cryptography interface.  
//! Uses the AES256-GCM algorithm: https://en.wikipedia.org/wiki/Galois/Counter_Mode  
//! Nonces and Passwords are stored as byte arrays, because they are not guaranteed to be valid UTF-8  

use aes_gcm::{
    aead::{consts::{B0, B1}, Aead, AeadCore, KeyInit, OsRng}, aes::cipher::typenum::{UInt, UTerm}, Aes256Gcm, Nonce
};

/// Struct for en/de-cryption
pub struct Cipher(Aes256Gcm);

// Generic array of numbers representing a Nonce
type NonceArray = Nonce<UInt<UInt<UInt<UInt<UTerm, B1>, B1>, B0>, B0>>;

impl Default for Cipher {
    fn default() -> Self {
        Cipher::new()
    }
}

impl Cipher {
    #[doc(hidden)]
    const KEY_BYTES: [u8; 32] = *include_bytes!("key.txt");

    pub fn new() -> Self {
        Self(Aes256Gcm::new(&Self::KEY_BYTES.into()))
    }

    /// Encrypt `text` in place and return a nonce
    pub fn encrypt<B: AsRef<[u8]>>(&self, text: B, output: &mut Vec<u8>) -> anyhow::Result<NonceArray> {
        let nonce = Aes256Gcm::generate_nonce(&mut OsRng);
        let bytes = self.0.encrypt(&nonce, text.as_ref())
            .map_err(|e| anyhow::anyhow!("failed to encrypt: {e}"))?;
        *output = bytes;

        Ok(nonce)
    }

    /// Return decrypted text from `text` and `nonce`
    pub fn decrypt<B: AsRef<[u8]>>(&self, data: B, nonce: &NonceArray) -> anyhow::Result<String> {
        let bytes = self.0.decrypt(nonce, data.as_ref()).map_err(|e| anyhow::anyhow!("failed to decrypt: {e}"))?;
        Ok(String::from_utf8(bytes)?)
    }
}

