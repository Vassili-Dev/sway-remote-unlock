use crate::args::{GenerateKeysCommand, KeyFormat};
use dryoc::sign::{PublicKey, SecretKey};
use remote_unlock_lib::config::Config;
use remote_unlock_lib::errors::RemoteUnlockError;
use std::path::Path;

#[cfg(debug_assertions)]
pub fn generate_keys(config: &Config, args: GenerateKeysCommand) -> Result<(), RemoteUnlockError> {
    use dryoc::types::Bytes;
    use remote_unlock_lib::{
        crypto::{der::DerKeyBorrowed, key},
        helper_types::ByteArray,
    };

    let keypair: dryoc::sign::SigningKeyPair<PublicKey, SecretKey> =
        dryoc::sign::SigningKeyPair::gen();

    let (pubkey_str, privkey_str) = match args.format {
        KeyFormat::PEM => {
            let mut pubkey_str: [u8; 1024] = [0; 1024];
            let mut privkey_str: [u8; 1024] = [0; 1024];

            let der_pubkey = DerKeyBorrowed::from_key(keypair.public_key.as_slice())?;
            let der_privkey = DerKeyBorrowed::from_key(keypair.secret_key.as_slice())?;

            let len = key::KeyBorrowed::PublicKey { key: der_pubkey }.to_pem(&mut pubkey_str)?;
            let pubkey_str: ByteArray<1024> = ByteArray::new_from_slice(&pubkey_str[0..len]);

            let len = key::KeyBorrowed::SecretKey { key: der_privkey }.to_pem(&mut privkey_str)?;
            let privkey_str: ByteArray<1024> = ByteArray::new_from_slice(&privkey_str[0..len]);

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
                return Err(RemoteUnlockError::KeyExists(output.to_owned()));
            }
            if public_key_path.exists() && !args.force {
                return Err(RemoteUnlockError::KeyExists(format!("{}.pub", output)));
            }

            std::fs::write(public_key_path, pubkey_str.as_str())?;
            std::fs::write(private_key_path, privkey_str.as_str())?;
        }
        None => {
            // Print to stdout
            println!("{}", pubkey_str.as_str());
            println!("{}", privkey_str.as_str());
        }
    }
    Ok(())
}
