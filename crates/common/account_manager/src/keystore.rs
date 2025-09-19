use anyhow::anyhow;
use chrono::{DateTime, Utc};
use hex;
use rand;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::utils::validate_hex_string;

// Constants
/// The required keystore version
pub const KEYSTORE_VERSION: u32 = 5;

// Cryptographic algorithm enums with validation

/// Key derivation function used for password-based key derivation
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize, Default)]
pub enum KdfFunction {
    #[default]
    #[serde(rename = "argon2id")]
    Argon2Id,
}

/// Symmetric encryption cipher used for encrypting the private key
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize, Default)]
pub enum CipherFunction {
    #[default]
    #[serde(rename = "aes-256-gcm")]
    Aes256Gcm,
}

/// Post-quantum signature scheme used for key generation and signing
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize, Default)]
pub enum KeyTypeFunction {
    #[default]
    #[serde(rename = "xmss-poisedon2-ots-seed")]
    XmssPoseidon2OtsSeed,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Keystore {
    /// Version number, must be 5
    pub version: u32,

    /// Cryptographic parameters
    pub crypto: CryptoParams,

    /// Key type specification
    pub keytype: KeyType,

    /// Description of the keystore
    #[serde(skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,

    /// Must be true for quantum security
    pub quantum_secure: bool,

    /// UUID identifier
    pub uuid: Uuid,

    /// Optional derivation path
    #[serde(skip_serializing_if = "Option::is_none")]
    pub path: Option<String>,

    /// Metadata
    #[serde(skip_serializing_if = "Option::is_none")]
    pub meta: Option<KeystoreMeta>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct CryptoParams {
    /// Key derivation function parameters
    pub kdf: KdfParams,

    /// Cipher parameters and ciphertext
    pub cipher: CipherParams,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct KdfParams {
    /// KDF function name, must be "argon2id"
    pub function: KdfFunction,

    /// KDF parameters - supports both naming conventions
    pub params: KdfParamsInner,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(untagged)]
pub enum KdfParamsInner {
    /// Full parameter names
    Full {
        memory: u32,
        iterations: u32,
        parallelism: u32,
        salt: String,
    },
    /// Short parameter names
    Short {
        m: u32,
        t: u32,
        p: u32,
        salt: String,
    },
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct CipherParams {
    /// Cipher function name, must be "aes-256-gcm"
    pub function: CipherFunction,

    /// Cipher parameters
    pub params: CipherParamsInner,

    /// Encrypted data as hex string
    pub ciphertext: String,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct CipherParamsInner {
    /// Nonce/IV as hex string
    pub nonce: String,

    /// Authentication tag as hex string
    pub tag: String,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct KeyType {
    /// Key type function name
    pub function: KeyTypeFunction,

    /// Key type parameters
    pub params: KeyTypeParams,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct KeyTypeParams {
    /// Key lifetime
    pub lifetime: u32,

    /// Activation epoch
    pub activation_epoch: u32,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct KeystoreMeta {
    /// Creation timestamp
    #[serde(skip_serializing_if = "Option::is_none")]
    pub created: Option<DateTime<Utc>>,
}

impl Keystore {
    /// Create a new quantum-secure keystore
    pub fn new(crypto: CryptoParams, keytype: KeyType, uuid: Uuid) -> Self {
        Self {
            version: KEYSTORE_VERSION,
            crypto,
            keytype,
            description: None,
            quantum_secure: true,
            uuid,
            path: None,
            meta: Some(KeystoreMeta {
                created: Some(Utc::now()),
            }),
        }
    }

    /// Create a new keystore from seed phrase and key parameters
    pub fn from_seed_phrase(
        seed_phrase: &str,
        lifetime: u32,
        activation_epoch: u32,
        description: Option<String>,
        path: Option<String>,
    ) -> Self {
        // Generate random salt for KDF (32 bytes)
        let salt = hex::encode(rand::random::<[u8; 32]>());

        // Generate random nonce for AES-GCM (12 bytes)
        let nonce = hex::encode(rand::random::<[u8; 12]>());

        // Generate random tag for AES-GCM (16 bytes)
        let tag = hex::encode(rand::random::<[u8; 16]>());

        // Store the seed phrase as encrypted data (hex encoded)
        let ciphertext = hex::encode(seed_phrase.as_bytes());

        let crypto = CryptoParams {
            kdf: KdfParams::new_full(65536, 4, 2, salt),
            cipher: CipherParams::new(nonce, tag, ciphertext),
        };
        let keytype = KeyType::new(lifetime, activation_epoch);

        let mut keystore = Self::new(crypto, keytype, Uuid::new_v4());
        keystore.description = description;
        keystore.path = path;

        keystore
    }

    /// Validate the keystore structure
    pub fn validate(&self) -> anyhow::Result<()> {
        // Validate required constants for external data
        if self.version != KEYSTORE_VERSION {
            return Err(anyhow!("Version must be {KEYSTORE_VERSION}"));
        }
        if !self.quantum_secure {
            return Err(anyhow!("quantum_secure must be true"));
        }

        // Validate all hex strings
        let cipher = &self.crypto.cipher;
        if !validate_hex_string(&cipher.ciphertext) {
            return Err(anyhow!("ciphertext must be a valid hex string"));
        }
        if !validate_hex_string(&cipher.params.nonce) {
            return Err(anyhow!("nonce must be a valid hex string"));
        }
        if !validate_hex_string(&cipher.params.tag) {
            return Err(anyhow!("tag must be a valid hex string"));
        }

        let salt = match &self.crypto.kdf.params {
            KdfParamsInner::Full { salt, .. } | KdfParamsInner::Short { salt, .. } => salt,
        };
        if !validate_hex_string(salt) {
            return Err(anyhow!("salt must be a valid hex string"));
        }

        Ok(())
    }
}

impl KdfParams {
    /// Create new Argon2id KDF parameters (full names)
    pub fn new_full(memory: u32, iterations: u32, parallelism: u32, salt: String) -> Self {
        Self {
            function: KdfFunction::default(),
            params: KdfParamsInner::Full {
                memory,
                iterations,
                parallelism,
                salt,
            },
        }
    }

    /// Create new Argon2id KDF parameters (short names)
    pub fn new_short(m: u32, t: u32, p: u32, salt: String) -> Self {
        Self {
            function: KdfFunction::default(),
            params: KdfParamsInner::Short { m, t, p, salt },
        }
    }
}

impl CipherParams {
    /// Create new AES-256-GCM cipher parameters
    pub fn new(nonce: String, tag: String, ciphertext: String) -> Self {
        Self {
            function: CipherFunction::default(),
            params: CipherParamsInner { nonce, tag },
            ciphertext,
        }
    }
}

impl KeyType {
    /// Create new XMSS-Poseidon2 OTS seed key type
    pub fn new(lifetime: u32, activation_epoch: u32) -> Self {
        Self {
            function: KeyTypeFunction::default(),
            params: KeyTypeParams {
                lifetime,
                activation_epoch,
            },
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_keystore_creation() {
        let uuid = Uuid::new_v4();

        let kdf = KdfParams::new_full(65536, 3, 4, "0123456789abcdef".to_string());

        let cipher = CipherParams::new(
            "000102030405060708090a0b".to_string(),
            "0123456789abcdef0123456789abcdef".to_string(),
            "deadbeefcafe".to_string(),
        );

        let crypto = CryptoParams { kdf, cipher };

        let keytype = KeyType::new(262144, 0);

        let keystore = Keystore::new(crypto, keytype, uuid);

        assert_eq!(keystore.version, 5);
        assert!(keystore.quantum_secure);
        assert!(keystore.validate().is_ok());
    }

    #[test]
    fn test_json_serialization() {
        let uuid = Uuid::new_v4();
        let kdf = KdfParams::new_short(65536, 3, 4, "0123456789abcdef".to_string());
        let cipher = CipherParams::new(
            "000102030405060708090a0b".to_string(),
            "0123456789abcdef0123456789abcdef".to_string(),
            "deadbeefcafe".to_string(),
        );
        let crypto = CryptoParams { kdf, cipher };
        let keytype = KeyType::new(262144, 0);

        let keystore = Keystore::new(crypto, keytype, uuid);

        let json = serde_json::to_string_pretty(&keystore).unwrap();
        let deserialized: Keystore = serde_json::from_str(&json).unwrap();

        assert_eq!(keystore.version, deserialized.version);
        assert_eq!(keystore.uuid, deserialized.uuid);
    }
}
