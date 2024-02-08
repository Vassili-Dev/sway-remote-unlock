use serde::{Deserialize, Serialize};

#[repr(i8)]
#[derive(Serialize, Deserialize)]
pub enum MessageType {
    BeginEnroll = 10,
    Terminate = 99,
}

#[derive(Serialize, Deserialize)]
pub enum MessageData {
    BeginEnroll(()),
    Terminate(()),
}

#[derive(Serialize, Deserialize)]
pub struct Message {
    message_type: MessageType,
    data: MessageData,
}
