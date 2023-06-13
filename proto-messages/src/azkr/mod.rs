pub mod tlcs {
    pub mod v1beta1 {
        use ibc_proto::{google::protobuf::Any, protobuf::Protobuf};
        use prost::Message;
        use proto_types::AccAddress;
        use serde::{Deserialize, Serialize};

        use crate::Error;

        // Participant Contribution Section
        
        //#[derive(Serialize, Deserialize, Clone)]
        #[derive(Serialize, Deserialize, Clone, Message)]
        pub struct RawMsgParticipantContribution {
            #[prost(string, tag = "1")]
            pub address: String,
            #[prost(uint32, tag = "2")]
            pub round: u32,
            #[prost(uint32, tag = "3")]
            pub scheme: u32,
            #[prost(string, tag = "4")]
            pub data: String,
        }

        #[derive(Serialize, Deserialize, Clone, Debug, PartialEq)]
        pub struct MsgParticipantContribution {
            pub address: AccAddress,
            pub round: u32,
            pub scheme: u32,
            pub data: String,
        }

        impl TryFrom<RawMsgParticipantContribution> for MsgParticipantContribution {
            type Error = Error;

            fn try_from(raw: RawMsgParticipantContribution) -> Result<Self, Self::Error> {
                let address = AccAddress::from_bech32(&raw.address)
                    .map_err(|e| Error::DecodeAddress(e.to_string()))?;

                Ok(MsgParticipantContribution {
                    address,
                    round: raw.round,
                    scheme: raw.scheme,
                    data: raw.data,
                })
            }
        }

        impl From<MsgParticipantContribution> for RawMsgParticipantContribution {
            fn from(msg: MsgParticipantContribution) -> RawMsgParticipantContribution {
                RawMsgParticipantContribution {
                    address: msg.address.into(),
                    round: msg.round,
                    scheme: msg.scheme,
                    data: msg.data,
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

        #[derive(Serialize, Deserialize, Clone, Message)]
        pub struct QueryAllParticipantsContributionsResponse {
            #[prost(message, repeated, tag = "1")]
            pub contributions: Vec<RawMsgParticipantContribution>,
        }

        // KeyPair Section
        
        #[derive(Serialize, Deserialize, Clone, Message)]
        pub struct RawMsgKeyPair {
            #[prost(string, tag = "1")]
            pub public_key: String,
            #[prost(string, tag = "2")]
            pub private_key: String,
        }

        #[derive(Serialize, Deserialize, Clone, Debug, PartialEq)]
        pub struct MsgKeyPair {
            pub public_key: String,
            pub private_key: String,
        }

        impl TryFrom<RawMsgKeyPair> for MsgKeyPair {
            type Error = Error;

            fn try_from(raw: RawMsgKeyPair) -> Result<Self, Self::Error> {
                Ok(MsgKeyPair {
                    public_key: raw.public_key,
                    private_key: raw.private_key,
                })
            }
        }

        impl From<MsgKeyPair> for RawMsgKeyPair {
            fn from(msg: MsgKeyPair) -> RawMsgKeyPair {
                RawMsgKeyPair {
                    public_key: msg.public_key,
                    private_key: msg.private_key,
                }
            }
        }

        impl Protobuf<RawMsgKeyPair> for MsgKeyPair {}

        //TODO: should to Any be implemented at the individual message type?
        impl From<MsgKeyPair> for Any {
            fn from(msg: MsgKeyPair) -> Self {
                Any {
                    type_url: "/azkr.tlcs.v1beta1.MsgKeyPair".to_string(),
                    value: msg.encode_vec(),
                }
            }
        }

        #[derive(Serialize, Deserialize, Clone, Message)]
        pub struct QueryAllKeyPair {
            #[prost(message, repeated, tag = "1")]
            pub contributions: Vec<RawMsgKeyPair>,
        }
    }
}
