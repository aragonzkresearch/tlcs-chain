pub mod tlcs {
    pub mod v1beta1 {
        use ibc_proto::{
            //azkr::tlcs::v1beta1::{
            //    QueryAllRoundSchemeRequest as RawQueryAllRoundSchemeRequest,
            //},
            google::protobuf::Any,
            protobuf::Protobuf,
        };
        use prost::Message;
        use proto_messages::Error;
        use proto_types::AccAddress;
        use serde::{Deserialize, Serialize};

        /////////////////////////////////////////////////////////////////////////////////////
        // New Process Section
        /////////////////////////////////////////////////////////////////////////////////////

        #[derive(Serialize, Deserialize, Clone, Message)]
        pub struct RawMsgNewProcess {
            #[prost(string, tag = "1")]
            pub address: String,
            #[prost(uint64, tag = "2")]
            pub round: u64,
            #[prost(uint32, tag = "3")]
            pub scheme: u32,
            #[prost(int64, tag = "4")]
            pub pubkey_time: i64,
        }

        #[derive(Serialize, Deserialize, Clone, Debug, PartialEq)]
        pub struct MsgNewProcess {
            pub address: AccAddress,
            pub round: u64,
            pub scheme: u32,
            pub pubkey_time: i64,
        }

        impl TryFrom<RawMsgNewProcess> for MsgNewProcess {
            type Error = Error;

            fn try_from(raw: RawMsgNewProcess) -> Result<Self, Self::Error> {
                let address = AccAddress::from_bech32(&raw.address)
                    .map_err(|e| Error::DecodeAddress(e.to_string()))?;

                Ok(MsgNewProcess {
                    address,
                    round: raw.round,
                    scheme: raw.scheme,
                    pubkey_time: raw.pubkey_time,
                })
            }
        }

        impl From<MsgNewProcess> for RawMsgNewProcess {
            fn from(msg: MsgNewProcess) -> RawMsgNewProcess {
                RawMsgNewProcess {
                    address: msg.address.into(),
                    round: msg.round,
                    scheme: msg.scheme,
                    pubkey_time: msg.pubkey_time,
                }
            }
        }

        impl Protobuf<RawMsgNewProcess> for MsgNewProcess {}

        //TODO: should to Any be implemented at the individual message type?
        impl From<MsgNewProcess> for Any {
            fn from(msg: MsgNewProcess) -> Self {
                Any {
                    type_url: "/azkr.tlcs.v1beta1.MsgNewProcess".to_string(),
                    value: msg.encode_vec(),
                }
            }
        }

        #[derive(Serialize, Deserialize, Clone, Message)]
        pub struct QueryAllNewProcesssResponse {
            #[prost(message, repeated, tag = "1")]
            pub contributions: Vec<RawMsgNewProcess>,
        }

        /////////////////////////////////////////////////////////////////////////////////////
        // Contribution Section
        /////////////////////////////////////////////////////////////////////////////////////

        #[derive(Serialize, Deserialize, Clone, Message)]
        pub struct RawMsgContribution {
            #[prost(string, tag = "1")]
            pub address: String,
            #[prost(uint64, tag = "2")]
            pub round: u64,
            #[prost(uint32, tag = "3")]
            pub scheme: u32,
            #[prost(uint32, tag = "4")]
            pub id: u32,
            #[prost(bytes, tag = "5")]
            pub data: Vec<u8>,
        }

        #[derive(Serialize, Deserialize, Clone, Debug, PartialEq)]
        pub struct MsgContribution {
            pub address: AccAddress,
            pub round: u64,
            pub scheme: u32,
            pub id: u32,
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
                    id: raw.id,
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
                    id: msg.id,
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

        impl Protobuf<QueryAllContributionsResponse> for QueryAllContributionsResponse {}

        /////////////////////////////////////////////////////////////////////////////////////
        // KeyPair Section
        /////////////////////////////////////////////////////////////////////////////////////

        #[derive(Serialize, Deserialize, Clone, Message)]
        pub struct RawMsgKeyPair {
            #[prost(uint64, tag = "1")]
            pub round: u64,
            #[prost(uint32, tag = "2")]
            pub scheme: u32,
            #[prost(uint32, tag = "3")]
            pub id: u32,
            #[prost(int64, tag = "4")]
            pub pubkey_time: i64,
            #[prost(string, tag = "5")]
            pub public_key: String,
            #[prost(string, tag = "6")]
            pub private_key: String,
        }

        #[derive(Serialize, Deserialize, Clone, Debug, PartialEq)]
        pub struct MsgKeyPair {
            pub round: u64,
            pub scheme: u32,
            pub id: u32,
            pub pubkey_time: i64,
            pub public_key: String,
            pub private_key: String,
        }

        impl TryFrom<RawMsgKeyPair> for MsgKeyPair {
            type Error = Error;

            fn try_from(raw: RawMsgKeyPair) -> Result<Self, Self::Error> {
                Ok(MsgKeyPair {
                    round: raw.round,
                    scheme: raw.scheme,
                    id: raw.id,
                    pubkey_time: raw.pubkey_time,
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
                    id: msg.id,
                    pubkey_time: msg.pubkey_time,
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

        impl Protobuf<QueryAllKeyPairsResponse> for QueryAllKeyPairsResponse {}

        /////////////////////////////////////////////////////////////////////////////////////
        // Round and Scheme Query message
        /////////////////////////////////////////////////////////////////////////////////////

        /// QueryRoundRequest is the request type for the Query/KeyPair RPC method.
        #[derive(Serialize, Deserialize, Clone, Message)]
        pub struct RawQueryRoundRequest {
            #[prost(uint64, tag = "1")]
            pub round: u64,
        }

        #[derive(Clone, PartialEq, Serialize, Deserialize)]
        pub struct QueryRoundRequest {
            pub round: u64,
        }

        impl TryFrom<RawQueryRoundRequest> for QueryRoundRequest {
            type Error = Error;

            fn try_from(raw: RawQueryRoundRequest) -> Result<Self, Self::Error> {
                Ok(QueryRoundRequest { round: raw.round })
            }
        }

        impl From<QueryRoundRequest> for RawQueryRoundRequest {
            fn from(query: QueryRoundRequest) -> RawQueryRoundRequest {
                RawQueryRoundRequest { round: query.round }
            }
        }

        impl Protobuf<RawQueryRoundRequest> for QueryRoundRequest {}

        /// QueryRoundSchemeRequest is the request type for the Query/KeyPair RPC method.
        #[derive(Serialize, Deserialize, Clone, Message)]
        pub struct RawQueryRoundSchemeRequest {
            #[prost(uint64, tag = "1")]
            pub round: u64,
            #[prost(uint32, tag = "2")]
            pub scheme: u32,
        }

        #[derive(Clone, PartialEq, Serialize, Deserialize)]
        pub struct QueryRoundSchemeRequest {
            pub round: u64,
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

        /// QueryTimeRequest is the request type for the Query/KeyPair RPC method.
        #[derive(Serialize, Deserialize, Clone, Message)]
        pub struct RawQueryTimeRequest {
            #[prost(int64, tag = "1")]
            pub time: i64,
        }

        #[derive(Clone, PartialEq, Serialize, Deserialize)]
        pub struct QueryTimeRequest {
            pub time: i64,
        }

        impl TryFrom<RawQueryTimeRequest> for QueryTimeRequest {
            type Error = Error;

            fn try_from(raw: RawQueryTimeRequest) -> Result<Self, Self::Error> {
                Ok(QueryTimeRequest { time: raw.time })
            }
        }

        impl From<QueryTimeRequest> for RawQueryTimeRequest {
            fn from(query: QueryTimeRequest) -> RawQueryTimeRequest {
                RawQueryTimeRequest { time: query.time }
            }
        }

        impl Protobuf<RawQueryTimeRequest> for QueryTimeRequest {}

        /////////////////////////////////////////////////////////////////////////////////////
        // LOE Data input structs
        /////////////////////////////////////////////////////////////////////////////////////

        #[derive(Serialize, Deserialize, Clone, Message)]
        pub struct RawMsgLoeData {
            #[prost(string, tag = "1")]
            pub address: String,
            #[prost(uint64, tag = "2")]
            pub round: u64,
            #[prost(string, tag = "3")]
            pub signature: String,
        }

        #[derive(Serialize, Deserialize, Clone, Debug, PartialEq)]
        pub struct MsgLoeData {
            pub address: AccAddress,
            pub round: u64,
            pub signature: String,
        }

        impl TryFrom<RawMsgLoeData> for MsgLoeData {
            type Error = Error;

            fn try_from(raw: RawMsgLoeData) -> Result<Self, Self::Error> {
                let address = AccAddress::from_bech32(&raw.address)
                    .map_err(|e| Error::DecodeAddress(e.to_string()))?;

                Ok(MsgLoeData {
                    address,
                    round: raw.round,
                    signature: raw.signature,
                })
            }
        }

        impl From<MsgLoeData> for RawMsgLoeData {
            fn from(msg: MsgLoeData) -> RawMsgLoeData {
                RawMsgLoeData {
                    address: msg.address.into(),
                    round: msg.round,
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

        impl Protobuf<QueryAllLoeDataResponse> for QueryAllLoeDataResponse {}
    }
}
