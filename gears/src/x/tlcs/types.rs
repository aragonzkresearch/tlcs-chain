use prost::Message;
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Clone, Message)]
pub struct MsgParticipantContribution {
    #[prost(message, repeated, tag = "1")]
    pub supply: Vec<String>,

    #[prost(message, optional, tag = "2")]
    pub pagination: Option<String>,
}
