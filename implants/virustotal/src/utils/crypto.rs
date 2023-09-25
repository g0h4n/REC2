use log::error;
use aes::cipher::{block_padding::Pkcs7, BlockDecryptMut, BlockEncryptMut, KeyIvInit};
use pbkdf2::{
    password_hash::{PasswordHash, PasswordHasher, SaltString},
    Params, Pbkdf2,
};
use rand::{distributions::Alphanumeric, Rng};
type Aes256CbcDec = cbc::Decryptor<aes::Aes256>;
type Aes256CbcEnc = cbc::Encryptor<aes::Aes256>;

/// Function to provide simple mechanism to encrypt some bytes with a key using AES-256-CBC
/// Thanks to: <https://github.com/jj-style/stegosaurust/blob/master/src/crypto.rs>
pub fn aes_encrypt(
    plaintext: &[u8],
    key: &[u8]
) -> Vec<u8> {
    let s: String = rand::thread_rng()
        .sample_iter(&Alphanumeric)
        .take(8)
        .map(char::from)
        .collect();
    let salt = SaltString::new(&s).unwrap();
    let password_hash = hash_password(key, &salt).unwrap();
    let password_hash = password_hash.hash.unwrap();
    let (key, iv) = password_hash.as_bytes().split_at(32);
    let cipher = Aes256CbcEnc::new_from_slices(key, iv).unwrap();
    let ciphertext = cipher.encrypt_padded_vec_mut::<Pkcs7>(plaintext);
    let message = ["Salted__".as_bytes(), salt.as_bytes(), &ciphertext].concat();
    message
}

/// Function to provide simple mechanism to decrypt some bytes with a key using AES-256-CBC
/// Thanks to: <https://github.com/jj-style/stegosaurust/blob/master/src/crypto.rs>
pub fn aes_decrypt(
    ciphertext: &[u8],
    key: &[u8]
) -> Vec<u8> {
    if !ciphertext.starts_with(b"Salted__") {
        error!("Message was not encrypted when encoded");
    }
    if ciphertext.len() < 16 {
        error!("Ciphertext is too short");
    }
    let (_, rest) = ciphertext.split_at(8); //ignore prefix 'Salted__'
    let (s, rest) = rest.split_at(8);
    let s = String::from_utf8(s.to_vec()).unwrap();
    let salt = SaltString::new(&s).unwrap();
    let password_hash = hash_password(key, &salt).unwrap();
    let password_hash = password_hash.hash.unwrap();
    let (key, iv) = password_hash.as_bytes().split_at(32);
    let cipher = Aes256CbcDec::new_from_slices(key, iv).unwrap();
    let r = cipher.decrypt_padded_vec_mut::<Pkcs7>(rest);
    match r {
        Ok(plaintext) => { return plaintext }
        Err(err) => { 
            error!("Inccorect key, can't decode AES! Reason: {err}");
            return Vec::new()
        }
    }
}

/// Function to hash password and salt,
/// to generate key for use with AES-256 encryption.
///
/// Uses PBKDF2 with 10,000 rounds of SHA256 hashing to generate a 48-byte response.
/// 48-byte response contains the 16-byte IV and 32-byte key.
/// Thanks to: <https://github.com/jj-style/stegosaurust/blob/master/src/crypto.rs>
pub fn hash_password<'a>(
    key: &'a [u8],
    salt: &'a SaltString,
) -> Result<PasswordHash<'a>, pbkdf2::password_hash::Error> {
    Pbkdf2.hash_password_customized(
        key,
        None,
        None,
        Params {
            rounds: 10_000,
            output_length: 48,
        },
        salt,
    )
}