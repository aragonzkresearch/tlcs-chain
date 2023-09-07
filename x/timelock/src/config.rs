use std::path::PathBuf;

use tendermint_informal::chain::Id;
use tendermint_rpc::Url;

#[derive(Debug, Clone)]
pub struct Config {
    pub node: Url,
    pub home: PathBuf,
    pub from: String,
    pub chain_id: Id,
}
