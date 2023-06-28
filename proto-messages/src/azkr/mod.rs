pub mod tlcs {
    pub mod v1beta1 {
        use ibc_proto::{
            //azkr::tlcs::v1beta1::{
            //    QueryAllRoundSchemeRequest as RawQueryAllRoundSchemeRequest,
            //},
            google::protobuf::Any,
            protobuf::Protobuf
        };
        use prost::Message;
        use proto_types::AccAddress;
        use serde::{Deserialize, Serialize};

        use crate::Error;

        /////////////////////////////////////////////////////////////////////////////////////
        // Contribution Section
        /////////////////////////////////////////////////////////////////////////////////////
        
        #[derive(Serialize, Deserialize, Clone, Message)]
        pub struct RawMsgContribution {
            #[prost(string, tag = "1")]
            pub address: String,
            #[prost(uint32, tag = "2")]
            pub round: u32,
            #[prost(uint32, tag = "3")]
            pub scheme: u32,
            #[prost(bytes, tag = "4")]
            pub data: Vec<u8>,
        }

        #[derive(Serialize, Deserialize, Clone, Debug, PartialEq)]
        pub struct MsgContribution {
            pub address: AccAddress,
            pub round: u32,
            pub scheme: u32,
            pub data: Vec<u8>,
        }

        impl TryFrom<RawMsgContribution> for MsgContribution {
            type Error = Error;

            fn try_from(raw: RawMsgContribution) -> Result<Self, Self::Error> {
                let address = AccAddress::from_bech32(&raw.address)
                    .map_err(|e| Error::DecodeAddress(e.to_string()))?;

                Ok(MsgContribution {
                    address,
                    round: raw.round,
                    scheme: raw.scheme,
                    data: raw.data,
                })
            }
        }

        impl From<MsgContribution> for RawMsgContribution {
            fn from(msg: MsgContribution) -> RawMsgContribution {
                RawMsgContribution {
                    address: msg.address.into(),
                    round: msg.round,
                    scheme: msg.scheme,
                    data: msg.data,
                }
            }
        }

        impl Protobuf<RawMsgContribution> for MsgContribution {}

        //TODO: should to Any be implemented at the individual message type?
        impl From<MsgContribution> for Any {
            fn from(msg: MsgContribution) -> Self {
                Any {
                    type_url: "/azkr.tlcs.v1beta1.MsgContribution".to_string(),
                    value: msg.encode_vec(),
                }
            }
        }

        #[derive(Serialize, Deserialize, Clone, Message)]
        pub struct QueryAllContributionsResponse {
            #[prost(message, repeated, tag = "1")]
            pub contributions: Vec<RawMsgContribution>,
        }

        /////////////////////////////////////////////////////////////////////////////////////
        // KeyPair Section
        /////////////////////////////////////////////////////////////////////////////////////
        
        #[derive(Serialize, Deserialize, Clone, Message)]
        pub struct RawMsgKeyPair {
            #[prost(uint32, tag = "1")]
            pub round: u32,
            #[prost(uint32, tag = "2")]
            pub scheme: u32,
            #[prost(bytes, tag = "3")]
            pub public_key: Vec<u8>,
            #[prost(bytes, tag = "4")]
            pub private_key: Vec<u8>,
        }

        #[derive(Serialize, Deserialize, Clone, Debug, PartialEq)]
        pub struct MsgKeyPair {
            pub round: u32,
            pub scheme: u32,
            pub public_key: Vec<u8>,
            pub private_key: Vec<u8>,
        }

        impl TryFrom<RawMsgKeyPair> for MsgKeyPair {
            type Error = Error;

            fn try_from(raw: RawMsgKeyPair) -> Result<Self, Self::Error> {
                Ok(MsgKeyPair {
                    round: raw.round,
                    scheme: raw.scheme,
                    public_key: raw.public_key,
                    private_key: raw.private_key,
                })
            }
        }

        impl From<MsgKeyPair> for RawMsgKeyPair {
            fn from(msg: MsgKeyPair) -> RawMsgKeyPair {
                RawMsgKeyPair {
                    round: msg.round,
                    scheme: msg.scheme,
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
        pub struct QueryAllKeyPairsResponse {
            #[prost(message, repeated, tag = "1")]
            pub keypairs: Vec<RawMsgKeyPair>,
        }

        /////////////////////////////////////////////////////////////////////////////////////
        // Round and Scheme Query message
        /////////////////////////////////////////////////////////////////////////////////////
        
        /// QueryRoundRequest is the request type for the Query/KeyPair RPC method.
        #[derive(Serialize, Deserialize, Clone, Message)]
        pub struct RawQueryRoundRequest {
            #[prost(uint32, tag = "1")]
            pub round: u32,
        }

        #[derive(Clone, PartialEq, Serialize, Deserialize)]
        pub struct QueryRoundRequest {
            pub round: u32,
        }
    
        impl TryFrom<RawQueryRoundRequest> for QueryRoundRequest {
            type Error = Error;
        
            fn try_from(raw: RawQueryRoundRequest) -> Result<Self, Self::Error> {
                Ok(QueryRoundRequest {
                    round: raw.round,
                })
            }
        }
    
        impl From<QueryRoundRequest> for RawQueryRoundRequest {
            fn from(query: QueryRoundRequest) -> RawQueryRoundRequest {
                RawQueryRoundRequest {
                    round: query.round,
                }
            }
        }
    
        impl Protobuf<RawQueryRoundRequest> for QueryRoundRequest {}

        /// QueryRoundSchemeRequest is the request type for the Query/KeyPair RPC method.
        #[derive(Serialize, Deserialize, Clone, Message)]
        pub struct RawQueryRoundSchemeRequest {
            #[prost(uint32, tag = "1")]
            pub round: u32,
            #[prost(uint32, tag = "2")]
            pub scheme: u32,
        }

        #[derive(Clone, PartialEq, Serialize, Deserialize)]
        pub struct QueryRoundSchemeRequest {
            pub round: u32,
            pub scheme: u32,
        }
    
        impl TryFrom<RawQueryRoundSchemeRequest> for QueryRoundSchemeRequest {
            type Error = Error;
        
            fn try_from(raw: RawQueryRoundSchemeRequest) -> Result<Self, Self::Error> {
                Ok(QueryRoundSchemeRequest {
                    round: raw.round,
                    scheme: raw.scheme,
                    //pagination: raw.pagination,
                })
            }
        }
    
        impl From<QueryRoundSchemeRequest> for RawQueryRoundSchemeRequest {
            fn from(query: QueryRoundSchemeRequest) -> RawQueryRoundSchemeRequest {
                RawQueryRoundSchemeRequest {
                    round: query.round,
                    scheme: query.scheme,
                    //pagination: query.pagination,
                }
            }
        }
    
        impl Protobuf<RawQueryRoundSchemeRequest> for QueryRoundSchemeRequest {}

        /////////////////////////////////////////////////////////////////////////////////////
        // LOE Data input structs
        /////////////////////////////////////////////////////////////////////////////////////
        
        #[derive(Serialize, Deserialize, Clone, Message)]
        pub struct RawMsgLoeData {
            //#[prost(string, tag = "1")]
            //pub address: String,
            #[prost(uint32, tag = "1")]
            pub round: u32,
            #[prost(bytes, tag = "2")]
            pub randomness: Vec<u8>,
            #[prost(bytes, tag = "3")]
            pub signature: Vec<u8>,
        }

        #[derive(Serialize, Deserialize, Clone, Debug, PartialEq)]
        pub struct MsgLoeData {
            //pub address: AccAddress,
            pub round: u32,
            pub randomness: Vec<u8>,
            pub signature: Vec<u8>,
        }

        impl TryFrom<RawMsgLoeData> for MsgLoeData {
            type Error = Error;

            fn try_from(raw: RawMsgLoeData) -> Result<Self, Self::Error> {
                Ok(MsgLoeData {
                    round: raw.round,
                    randomness: raw.randomness,
                    signature: raw.signature,
                })
            }
        }

        impl From<MsgLoeData> for RawMsgLoeData {
            fn from(msg: MsgLoeData) -> RawMsgLoeData {
                RawMsgLoeData {
                    round: msg.round,
                    randomness: msg.randomness,
                    signature: msg.signature,
                }
            }
        }

        impl Protobuf<RawMsgLoeData> for MsgLoeData {}

        //TODO: should to Any be implemented at the individual message type?
        impl From<MsgLoeData> for Any {
            fn from(msg: MsgLoeData) -> Self {
                Any {
                    type_url: "/azkr.tlcs.v1beta1.MsgLoeData".to_string(),
                    value: msg.encode_vec(),
                }
            }
        }

        #[derive(Serialize, Deserialize, Clone, Message)]
        pub struct QueryAllLoeDataResponse {
            #[prost(message, repeated, tag = "1")]
            pub randomnesses: Vec<RawMsgLoeData>,
        }
    }
}
