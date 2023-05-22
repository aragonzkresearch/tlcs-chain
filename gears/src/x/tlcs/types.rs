use ibc_proto::{google::protobuf::Any, protobuf::Protobuf};
use prost::Message;
use proto_messages::Error;
use proto_types::AccAddress;
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Clone, Message)]
pub struct RawMsgParticipantContribution {
    #[prost(string, tag = "1")]
    pub address: String,

    #[prost(string, tag = "2")]
    pub key: String,

    #[prost(string, tag = "3")]
    pub value: String,
}

#[derive(Serialize, Deserialize, Clone)]
pub struct MsgParticipantContribution {
    pub address: AccAddress,
    pub key: String,
    pub value: String,
}

impl TryFrom<RawMsgParticipantContribution> for MsgParticipantContribution {
    type Error = Error;

    fn try_from(raw: RawMsgParticipantContribution) -> Result<Self, Self::Error> {
        let address = AccAddress::from_bech32(&raw.address)
            .map_err(|e| Error::DecodeAddress(e.to_string()))?;

        Ok(MsgParticipantContribution {
            address,
            key: raw.key,
            value: raw.value,
        })
    }
}

impl From<MsgParticipantContribution> for RawMsgParticipantContribution {
    fn from(msg: MsgParticipantContribution) -> RawMsgParticipantContribution {
        RawMsgParticipantContribution {
            address: msg.address.into(),
            key: msg.key,
            value: msg.value,
        }
    }
}

impl Protobuf<RawMsgParticipantContribution> for MsgParticipantContribution {}

//TODO: should to Any be implemented at the individual message type?
impl From<MsgParticipantContribution> for Any {
    fn from(msg: MsgParticipantContribution) -> Self {
        Any {
            type_url: "/azkr.tlcs.v1beta1.MsgParticipantContribution".to_string(),
            value: msg.encode_vec(),
        }
    }
}
