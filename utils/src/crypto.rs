use aes_gcm::{
    aead::{Aead, KeyInit},
    Aes256Gcm, Nonce,
};
use base64::{engine::general_purpose::STANDARD, Engine as _};
use std::env;
use crate::{anyhow, Result};


/// 加密工具类
/// 实现原理: 使用 AES-256-GCM 算法进行对称加密。主密钥从环境变量 `MASTER_KEY` 读取。
pub struct Crypto;

impl Crypto {
    /// 获取主密钥 (32字节)
    fn get_master_key() -> Result<[u8; 32]> {
        let key_str = env::var("MASTER_KEY")
            .map_err(|_| anyhow!("环境变量 MASTER_KEY 未设置"))?;
        
        // 尝试从 hex 或 base64 解析，这里简化为直接取字节并补齐/截断
        let mut key = [0u8; 32];
        let bytes = key_str.as_bytes();
        let len = bytes.len().min(32);
        key[..len].copy_from_slice(&bytes[..len]);
        Ok(key)
    }

    /// 加密数据
    /// 返回 base64(nonce + ciphertext)
    pub fn encrypt(plain_text: &str) -> Result<String> {
        let key_bytes = Self::get_master_key()?;
        let cipher = Aes256Gcm::new_from_slice(&key_bytes)
            .map_err(|e| anyhow!("初始化加密器失败: {}", e))?;
            
        // 生成随机 Nonce (12字节)
        let mut nonce_bytes = [0u8; 12];
        getrandom::getrandom(&mut nonce_bytes)
            .map_err(|e| anyhow!("生成随机数失败: {}", e))?;
        let nonce = Nonce::from_slice(&nonce_bytes);

        let ciphertext = cipher
            .encrypt(nonce, plain_text.as_bytes())
            .map_err(|e| anyhow!("加密失败: {}", e))?;

        // 拼接 nonce + ciphertext 并转为 base64
        let mut combined = nonce_bytes.to_vec();
        combined.extend_from_slice(&ciphertext);
        
        Ok(STANDARD.encode(combined))
    }

    /// 解密数据
    pub fn decrypt(encrypted_base64: &str) -> Result<String> {
        let key_bytes = Self::get_master_key()?;
        let cipher = Aes256Gcm::new_from_slice(&key_bytes)
            .map_err(|e| anyhow!("初始化解密器失败: {}", e))?;

        let combined = STANDARD.decode(encrypted_base64)
            .map_err(|e| anyhow!("Base64 解码失败: {}", e))?;

        if combined.len() < 12 {
            return Err(anyhow!("加密数据长度非法"));
        }

        let (nonce_bytes, ciphertext) = combined.split_at(12);
        let nonce = Nonce::from_slice(nonce_bytes);

        let plaintext_bytes = cipher
            .decrypt(nonce, ciphertext)
            .map_err(|e| anyhow!("解密失败: {}", e))?;

        String::from_utf8(plaintext_bytes)
            .map_err(|e| anyhow!("UTF8 转换失败: {}", e))
    }
}
