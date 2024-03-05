use clap::{Parser, Subcommand};

#[derive(Parser)]
#[command(name = "bitwarden-pin", subcommand_value_name = "KDF")]
pub struct Args {
    /// PIN-encrypted User Key
    #[clap(short, long)]
    pub encrypted: String,

    /// Email address (salt)
    #[clap(short = 'm', long)]
    pub email: String,

    /// Number of digits in the PIN
    #[clap(short, long, default_value = "4")]
    pub pin_length: usize,

    /// Key Deriviation Function (KDF) configuration
    #[command(subcommand)]
    pub kdf_config: Option<KDFConfig>,
}

#[derive(Subcommand, Clone, Copy, Debug)]
pub enum KDFConfig {
    /// kdfType=0: Password Based Key Derivation Function 2 (default)
    Pbkdf2 {
        /// Number of pbkdf2 iterations
        #[clap(short, long, default_value = "600000")]
        iterations: u32,
    },
    /// kdfType=1: Argon2
    Argon2 {
        /// Memory cost, in MiB. Note: this changes the hash output!
        #[clap(short, long, default_value = "64")]
        memory: u32,
        /// Time cost, number of argon2 iterations, time cost
        #[clap(short, long, default_value = "3")]
        iterations: u32,
        /// Parallelism, number of argon2 threads. Note: this changes the hash output!
        #[clap(short, long, default_value = "4")]
        parallelism: u32,
    },
}

impl Default for KDFConfig {
    fn default() -> Self {
        KDFConfig::Pbkdf2 { iterations: 600000 }
    }
}
