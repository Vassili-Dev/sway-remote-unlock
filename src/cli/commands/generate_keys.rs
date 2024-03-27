use crate::args::{GenerateKeysCommand, KeyFormat};
use rand::rngs::OsRng;
use remote_unlock_lib::prelude::*;
use std::path::Path;

#[cfg(debug_assertions)]
pub fn generate_keys(config: &Config, args: GenerateKeysCommand) -> Result<(), Error> {
    use pkcs8::{EncodePrivateKey, EncodePublicKey};
    use remote_unlock_lib::crypto::key::{PrivateKey, PublicKey};

    let privkey = p256::SecretKey::random(&mut OsRng);
    let pubkey = privkey.public_key();

    let pubkey = PublicKey::from_der(pubkey.to_public_key_der()?.as_bytes())?;
    let privkey = PrivateKey::from_der(privkey.to_pkcs8_der()?.as_bytes())?;

    match args.output {
        Some(ref output) => {
            // Ensure the directory for the keys exists
            std::fs::create_dir_all(config.generated_keys_dir())?;

            // Write the public key to a file
            let public_key_path =
                Path::new(&config.generated_keys_dir()).join(format!("{}.pub", output));
            let private_key_path = Path::new(&config.generated_keys_dir()).join(output);

            if private_key_path.exists() && !args.force {
                return Err(Error::new(ErrorKind::KeyExists, Some(output)));
            }
            if public_key_path.exists() && !args.force {
                return Err(Error::new(ErrorKind::KeyExists, Some(output)));
            }

            match args.format {
                KeyFormat::Der => {
                    pubkey.save_to_der_file(public_key_path.as_path())?;
                    privkey.save_to_der_file(private_key_path.as_path())?;
                }
                KeyFormat::Pem => {
                    pubkey.save_to_pem_file(public_key_path.as_path())?;
                    privkey.save_to_pem_file(private_key_path.as_path())?;
                }
            }
        }
        None => {
            // Print to stdout
            match args.format {
                KeyFormat::Der => {
                    pubkey.der()?.to_stdout_raw()?;
                    privkey.der()?.to_stdout_raw()?;
                }
                KeyFormat::Pem => {
                    println!("{}", pubkey.pem()?.as_str()?);
                    println!("{}", privkey.pem()?.as_str()?);
                }
            }
        }
    }
    Ok(())
}
