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

#[cfg(test)]
mod tests {
    use super::*;
    use crate::args::GenerateKeysCommand;
    use der::asn1::{ObjectIdentifier, OctetStringRef};
    use der::{
        Decode, DecodeValue, Encode, EncodeValue, FixedTag, Header, Length, Reader, Tag, Writer,
    };
    use remote_unlock_lib::config::Config;
    use remote_unlock_lib::helper_types::der::NestedOctetString;
    use std::borrow::BorrowMut;
    use std::path::PathBuf;

    // 302e020100300506032b657004220420e6d402bca22a67721c8ce8b1ff7ac6b4a556462f558fac148da972992b6f32df
    const KEY: [u8; 48] = [
        0x30, 0x2e, 0x02, 0x01, 0x00, 0x30, 0x05, 0x06, 0x03, 0x2b, 0x65, 0x70, 0x04, 0x22, 0x04,
        0x20, 0xe6, 0xd4, 0x02, 0xbc, 0xa2, 0x2a, 0x67, 0x72, 0x1c, 0x8c, 0xe8, 0xb1, 0xff, 0x7a,
        0xc6, 0xb4, 0xa5, 0x56, 0x46, 0x2f, 0x55, 0x8f, 0xac, 0x14, 0x8d, 0xa9, 0x72, 0x99, 0x2b,
        0x6f, 0x32, 0xdf,
    ];

    #[derive(Debug, PartialEq, Eq, der::Sequence)]
    struct AlgorithmIdentifier {
        oid: ObjectIdentifier,
    }

    #[derive(Debug, PartialEq, Eq, der::Sequence)]
    struct DerKeyBorrowed<'a> {
        version: u8,              // 0
        oid: AlgorithmIdentifier, // 1.3.101.112
        key: NestedOctetString<'a, OctetStringRef<'a>>,
    }

    #[test]
    fn test_parse_der() {
        let key = DerKeyBorrowed::from_der(&KEY).unwrap();
        assert_eq!(key.version, 0);
        assert_eq!(
            key.oid,
            AlgorithmIdentifier {
                oid: ObjectIdentifier::new("1.3.101.112").expect("Invalid OID")
            }
        );
        assert_eq!(key.key.as_bytes(), &KEY[16..]);
    }
}
