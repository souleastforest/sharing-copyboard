use aes_gcm::{
    aead::{Aead, KeyInit},
    Aes256Gcm, Key, Nonce
};
use argon2::{self, password_hash::{PasswordHasher, SaltString, PasswordHash, PasswordVerifier}};
use argon2::Argon2;
use rand::{Rng, thread_rng};

// 生成随机密钥
pub fn generate_encryption_key() -> [u8; 32] {
    let mut key = [0u8; 32];
    thread_rng().fill(&mut key);
    key
}

// 生成随机IV (Initialization Vector)
pub fn generate_nonce() -> [u8; 12] {
    let mut nonce = [0u8; 12];
    thread_rng().fill(&mut nonce);
    nonce
}

// 加密数据
pub fn encrypt_data(data: &[u8], encryption_key: &[u8], nonce: &[u8; 12]) -> Result<Vec<u8>, String> {
    let key = Key::<Aes256Gcm>::from_slice(encryption_key);
    let cipher = Aes256Gcm::new(key);
    let nonce = Nonce::from_slice(nonce);
    
    cipher.encrypt(nonce, data)
        .map_err(|e| format!("Encryption failed: {}", e))
}

// 解密数据
pub fn decrypt_data(encrypted_data: &[u8], encryption_key: &[u8], nonce: &[u8; 12]) -> Result<String, String> {
    let key = Key::<Aes256Gcm>::from_slice(encryption_key);
    let cipher = Aes256Gcm::new(key);
    let nonce = Nonce::from_slice(nonce);
    
    let decrypted = cipher.decrypt(nonce, encrypted_data)
        .map_err(|e| format!("Decryption failed: {}", e))?;
    
    String::from_utf8(decrypted)
        .map_err(|e| format!("Invalid UTF-8 sequence: {}", e))
}

// 生成密码哈希
pub fn hash_password(password: &str) -> Result<String, String> {
    let salt = SaltString::generate(&mut thread_rng());
    let argon2 = Argon2::default();
    
    argon2.hash_password(password.as_bytes(), &salt)
        .map(|hash| hash.to_string())
        .map_err(|e| format!("Password hashing failed: {}", e))
}

// 验证密码
pub fn verify_password(hash: &str, password: &str) -> Result<bool, String> {
    let parsed_hash = PasswordHash::new(hash)
        .map_err(|e| format!("Invalid password hash: {}", e))?;
    
    Ok(Argon2::default().verify_password(password.as_bytes(), &parsed_hash).is_ok())
}