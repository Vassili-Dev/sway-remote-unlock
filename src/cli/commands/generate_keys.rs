use crate::args::{GenerateKeysCommand, KeyFormat};
use base64::{engine::general_purpose::STANDARD, Engine as _};
use dryoc::{
    sign::{PublicKey, SecretKey},
    types::ByteArray,
};
use remote_unlock_lib::config::Config;
use remote_unlock_lib::errors::RemoteUnlockError;
use std::path::Path;

#[cfg(debug_assertions)]
pub fn generate_keys(config: &Config, args: GenerateKeysCommand) -> Result<(), RemoteUnlockError> {
    let keypair: dryoc::sign::SigningKeyPair<PublicKey, SecretKey> =
        dryoc::sign::SigningKeyPair::gen();

    let (pubkey_str, privkey_str) = match args.format {
        KeyFormat::PEM => {
            let b64_pubkey = STANDARD.encode(keypair.public_key.as_array());
            let b64_privkey = STANDARD.encode(keypair.secret_key.as_array());
            let pubkey_str = format!(
                "-----BEGIN PUBLIC KEY-----\n{}\n-----END PUBLIC KEY-----",
                b64_pubkey
            );
            let privkey_str = format!(
                "-----BEGIN PRIVATE KEY-----\n{}\n-----END PRIVATE KEY-----",
                b64_privkey
            );
            (pubkey_str, privkey_str)
        }
        _ => {
            unimplemented!("Key format not implemented");
        }
    };

    match args.output {
        Some(ref output) => {
            // Ensure the directory for the keys exists
            std::fs::create_dir_all(&config.generated_keys_dir())?;

            // Write the public key to a file
            let public_key_path =
                Path::new(&config.generated_keys_dir()).join(format!("{}.pub", output));
            let private_key_path =
                Path::new(&config.generated_keys_dir()).join(format!("{}", output));

            if private_key_path.exists() && !args.force {
                return Err(RemoteUnlockError::KeyExists(output.clone()));
            }
            if public_key_path.exists() && !args.force {
                return Err(RemoteUnlockError::KeyExists(format!("{}.pub", output)));
            }

            std::fs::write(public_key_path, pubkey_str)?;
            std::fs::write(private_key_path, privkey_str)?;
        }
        None => {
            // Print to stdout
            println!("{}", pubkey_str);
            println!("{}", privkey_str);
        }
    }
    Ok(())
}
