use ibc_proto::cosmos::tx::v1beta1::{AuthInfo, TxBody};

/// MsgSend represents a message to send coins from one account to another.
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct MsgSend {
    #[prost(address, tag = "1")]
    pub from_address: proto_types::AccAddress,
    #[prost(address, tag = "2")]
    pub to_address: proto_types::AccAddress,
    #[prost(message, repeated, tag = "3")]
    pub amount: Vec<ibc_proto::cosmos::base::v1beta1::Coin>,
}

/// QueryBalanceRequest is the request type for the Query/Balance RPC method.
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct QueryBalanceRequest {
    /// address is the address to query balances for.
    #[prost(address, tag = "1")]
    pub address: proto_types::AccAddress,
    /// denom is the coin denom to query balances for.
    #[prost(string, tag = "2")]
    pub denom: ::prost::alloc::string::String,
}

/// QueryBalanceRequest is the request type for the Query/AllBalances RPC method.
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct QueryAllBalancesRequest {
    /// address is the address to query balances for.
    #[prost(address, tag = "1")]
    pub address: proto_types::AccAddress,
    /// pagination defines an optional pagination for the request.
    #[prost(message, optional, tag = "2")]
    pub pagination: Option<ibc_proto::cosmos::base::query::v1beta1::PageRequest>,
}

/// BaseAccount defines a base account type. It contains all the necessary fields
/// for basic account functionality. Any custom account type should extend this
/// type for additional functionality (e.g. vesting).
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct BaseAccount {
    #[prost(address, tag = "1")]
    pub address: proto_types::AccAddress,
    #[prost(message, optional, tag = "2")]
    pub pub_key: Option<ibc_proto::google::protobuf::Any>,
    #[prost(uint64, tag = "3")]
    pub account_number: u64,
    #[prost(uint64, tag = "4")]
    pub sequence: u64,
}

/// QueryAccountRequest is the request type for the Query/Account RPC method.
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct QueryAccountRequest {
    /// address defines the address to query for.
    #[prost(address, tag = "1")]
    pub address: proto_types::AccAddress,
}

/// Tx is the standard type used for broadcasting transactions.
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct Tx {
    /// body is the processable content of the transaction
    #[prost(message, required, tag = "1")]
    pub body: TxBody,
    /// auth_info is the authorization related content of the transaction,
    /// specifically signers, signer modes and fee
    #[prost(message, required, tag = "2")]
    pub auth_info: AuthInfo,
    /// signatures is a list of signatures that matches the length and order of
    /// AuthInfo's signer_infos to allow connecting signature meta information like
    /// public key and signing mode by position.
    #[prost(bytes = "vec", repeated, tag = "3")]
    pub signatures: Vec<Vec<u8>>,
}