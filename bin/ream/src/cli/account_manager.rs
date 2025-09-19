use anyhow::ensure;
use bip39::Mnemonic;
use clap::Parser;
use tracing::info;

const MIN_CHUNK_SIZE: u32 = 4;
const MIN_LIFETIME: u32 = 18;
const DEFAULT_ACTIVATION_EPOCH: u32 = 0;
const DEFAULT_NUM_ACTIVE_EPOCHS: u32 = 1 << 18;

#[derive(Debug, Parser)]
pub struct AccountManagerConfig {
    /// Verbosity level
    #[arg(short, long, default_value_t = 3)]
    pub verbosity: u8,

    /// Account lifetime in 2 ** lifetime slots
    #[arg(short, long, default_value_t = 18)]
    pub lifetime: u32,

    /// Chunk size for messages
    #[arg(short, long, default_value_t = 5)]
    pub chunk_size: u32,

    /// Seed phrase for key generation
    #[arg(short, long)]
    pub seed_phrase: Option<String>,

    /// Optional BIP39 passphrase used with the seed phrase
    #[arg(long)]
    pub passphrase: Option<String>,

    /// Activation epoch for the validator
    #[arg(long, default_value_t = DEFAULT_ACTIVATION_EPOCH)]
    pub activation_epoch: u32,

    /// Number of active epochs
    #[arg(long, default_value_t = DEFAULT_NUM_ACTIVE_EPOCHS)]
    pub num_active_epochs: u32,

    /// Path for keystore directory (relative to data-dir if not absolute)
    #[arg(long)]
    pub keystore_path: Option<String>,
}

impl Default for AccountManagerConfig {
    fn default() -> Self {
        Self {
            verbosity: 3,
            lifetime: 18,
            chunk_size: 5,
            seed_phrase: None,
            passphrase: None,
            activation_epoch: DEFAULT_ACTIVATION_EPOCH,
            num_active_epochs: DEFAULT_NUM_ACTIVE_EPOCHS,
            keystore_path: None,
        }
    }
}

impl AccountManagerConfig {
    pub fn new() -> Self {
        Self::parse()
    }

    pub fn validate(&mut self) -> anyhow::Result<()> {
        ensure!(
            self.chunk_size >= MIN_CHUNK_SIZE,
            "Chunk size must be at least {MIN_CHUNK_SIZE}"
        );
        ensure!(
            self.lifetime >= MIN_LIFETIME,
            "Lifetime must be at least {MIN_LIFETIME}"
        );

        Ok(())
    }

    pub fn get_seed_phrase(&self) -> String {
        if let Some(phrase) = &self.seed_phrase {
            phrase.clone()
        } else {
            // Generate a new BIP39 mnemonic with 24 words (256 bits of entropy)
            let entropy: [u8; 32] = rand::random();
            let mnemonic = Mnemonic::from_entropy(&entropy).expect("Failed to generate mnemonic");
            let phrase = mnemonic.words().collect::<Vec<_>>().join(" ");
            info!("{}", "=".repeat(89));
            info!("Generated new seed phrase (KEEP SAFE): {phrase}");
            info!("{}", "=".repeat(89));
            phrase
        }
    }
}
