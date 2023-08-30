use bytes::Bytes;
use ibc_proto::{google::protobuf::Any, protobuf::Protobuf};
use proto_types::AccAddress;
use serde::Serialize;

use crate::proto::tlcs::v1beta1::{MsgContribution, MsgLoeData, MsgNewProcess};

#[derive(Debug, Clone, Serialize)]
#[serde(tag = "@type")]
pub enum Message {
    #[serde(rename = "/azkr.tlcs.v1beta1.MsgNewProcess")]
    NewProcess(MsgNewProcess),
    #[serde(rename = "/azkr.tlcs.v1beta1.MsgContribution")]
    Participate(MsgContribution),
    #[serde(rename = "/azkr.tlcs.v1beta1.MsgLoeData")]
    SubmitLoeData(MsgLoeData),
}

impl proto_messages::cosmos::tx::v1beta1::Message for Message {
    fn get_signers(&self) -> Vec<&AccAddress> {
        match &self {
            Message::NewProcess(msg) => return vec![&msg.address],
            Message::Participate(msg) => return vec![&msg.address],
            Message::SubmitLoeData(msg) => return vec![&msg.address],
        }
    }

    fn validate_basic(&self) -> Result<(), String> {
        match &self {
            Message::NewProcess(_) => Ok(()),
            Message::Participate(_) => Ok(()),
            Message::SubmitLoeData(_) => Ok(()),
        }
    }
}

impl From<Message> for Any {
    fn from(msg: Message) -> Self {
        match msg {
            Message::NewProcess(msg) => Any {
                type_url: "/azkr.tlcs.v1beta1.MsgNewProcess".to_string(),
                value: msg.encode_vec(),
            },
            Message::Participate(msg) => Any {
                type_url: "/azkr.tlcs.v1beta1.MsgContribution".to_string(),
                value: msg.encode_vec(),
            },
            Message::SubmitLoeData(msg) => Any {
                type_url: "/azkr.tlcs.v1beta1.MsgLoeData".to_string(),
                value: msg.encode_vec(),
            },
        }
    }
}

impl TryFrom<Any> for Message {
    type Error = proto_messages::Error;

    fn try_from(value: Any) -> Result<Self, Self::Error> {
        match value.type_url.as_str() {
            "/cosmos.bank.v1beta1.MsgNewProcess" => {
                let msg = MsgNewProcess::decode::<Bytes>(value.value.clone().into())?;
                Ok(Message::NewProcess(msg))
            }
            "/cosmos.bank.v1beta1.MsgContribution" => {
                let msg = MsgContribution::decode::<Bytes>(value.value.clone().into())?;
                Ok(Message::Participate(msg))
            }
            "/cosmos.bank.v1beta1.MsgLoeData" => {
                let msg = MsgLoeData::decode::<Bytes>(value.value.clone().into())?;
                Ok(Message::SubmitLoeData(msg))
            }
            _ => Err(proto_messages::Error::DecodeGeneral(
                "message type not recognized".into(),
            )),
        }
    }
}
