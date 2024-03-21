use crate::{config::Config, helper_types::ByteArray};
use spki::SubjectPublicKeyInfo;

use self::types::AnyOwned;

pub mod types;

pub type SubjectPublicKeyInfoOwned =
    SubjectPublicKeyInfo<AnyOwned, ByteArray<{ Config::BUFFER_SIZE }, 3>>;
