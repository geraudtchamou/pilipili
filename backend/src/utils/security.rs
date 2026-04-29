use argon2::{Argon2, PasswordHash, PasswordHasher, PasswordVerifier};
use argon2::password_hash::SaltString;
use rand::rngs::OsRng;
use uuid::Uuid;

pub fn hash_pin(pin: &str) -> Result<String, argon2::password_hash::Error> {
    let salt = SaltString::generate(&mut OsRng);
    let argon2 = Argon2::default();
    let password_hash = argon2.hash_password(pin.as_bytes(), &salt)?;
    Ok(password_hash.to_string())
}

pub fn verify_pin(pin: &str, hash: &str) -> Result<bool, argon2::password_hash::Error> {
    let parsed_hash = PasswordHash::new(hash)?;
    Ok(Argon2::default().verify_password(pin.as_bytes(), &parsed_hash).is_ok())
}

pub fn generate_otp() -> String {
    use rand::Rng;
    let mut rng = rand::thread_rng();
    format!("{:06}", rng.gen_range(100000..999999))
}

pub fn generate_idempotency_key() -> String {
    Uuid::new_v4().to_string()
}

pub fn encrypt_sensitive_data(data: &[u8], key: &[u8]) -> Result<Vec<u8>, Box<dyn std::error::Error>> {
    use aes_gcm::{Aes256Gcm, KeyInit, Nonce};
    use aes_gcm::aead::Aead;
    
    let cipher = Aes256Gcm::new_from_slice(key)?;
    let nonce = aes_gcm::Nonce::from_slice(b"unique_nonce");
    let ciphertext = cipher.encrypt(nonce, data)?;
    Ok(ciphertext)
}

pub fn decrypt_sensitive_data(ciphertext: &[u8], key: &[u8]) -> Result<Vec<u8>, Box<dyn std::error::Error>> {
    use aes_gcm::{Aes256Gcm, KeyInit, Nonce};
    use aes_gcm::aead::Aead;
    
    let cipher = Aes256Gcm::new_from_slice(key)?;
    let nonce = aes_gcm::Nonce::from_slice(b"unique_nonce");
    let plaintext = cipher.decrypt(nonce, ciphertext)?;
    Ok(plaintext)
}
