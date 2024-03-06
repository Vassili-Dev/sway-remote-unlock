#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Key {
    raw: ByteArray<2048>,
}
