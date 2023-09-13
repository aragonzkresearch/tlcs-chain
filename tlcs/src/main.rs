use std::str::FromStr;

use anyhow::Result;
use auth::Keeper as AuthKeeper;
use bank::Keeper as BankKeeper;
use client::query_command_handler;
use client::tx_command_handler;
use gears::utils::get_default_home_dir;
use gears::x::params::Keeper as ParamsKeeper;
use rest::get_router;
use tendermint_rpc::Url;
use timelock::Config;

use crate::genesis::GenesisState;
use crate::handler::Handler;
use crate::store_keys::{TlcsParamsStoreKey, TlcsStoreKey};

mod client;
mod genesis;
mod handler;
mod message;
mod rest;
mod store_keys;

pub const APP_NAME: &str = env!("CARGO_PKG_NAME");
pub const VERSION: &str = env!("GIT_HASH");

fn main() -> Result<()> {
    let params_keeper = ParamsKeeper::new(TlcsStoreKey::Params);

    let auth_keeper = AuthKeeper::new(
        TlcsStoreKey::Auth,
        params_keeper.clone(),
        TlcsParamsStoreKey::Auth,
    );

    let bank_keeper = BankKeeper::new(
        TlcsStoreKey::Bank,
        params_keeper.clone(),
        TlcsParamsStoreKey::Bank,
        auth_keeper.clone(),
    );

    let config = Config {
        node: Url::from_str("http://localhost:26657").unwrap(),
        //home: "/Users/craig/.tlcs".into(),
        home: get_default_home_dir(APP_NAME).unwrap(),
        from: "kevin".into(),
        chain_id: tendermint_informal::chain::Id::try_from("chain-id").unwrap(),
    };

    gears::app::run(
        APP_NAME,
        VERSION,
        GenesisState::default(),
        bank_keeper,
        auth_keeper,
        params_keeper,
        TlcsParamsStoreKey::BaseApp,
        Handler::new(config),
        query_command_handler,
        tx_command_handler,
        get_router(),
    )
}
