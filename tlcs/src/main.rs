use serde::Deserialize; // TODO: remove this and get rid of the config reading in this file
use std::fs;
use std::str::FromStr; // TODO: remove this and get rid of the config reading in this file
                       //use toml; // TODO: remove this and get rid of the config reading in this file

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

#[derive(Deserialize)]
struct Fileconf {
    tendermint_url: String,
    from_user: String,
    chain_id: String,
}

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

    // This is only here until Kevin makes the config more universal
    let mut home_dir = get_default_home_dir(APP_NAME).unwrap();
    home_dir.push("config/resend.toml");

    //let contents = match fs::read_to_string("~/.tlcs/config/resend.toml") {
    let contents = match fs::read_to_string(home_dir) {
        Ok(s) => s,
        Err(_) => {
            "tendermint_url='http://localhost:26657'\nfrom_user='kevin'\nchain_id='test-chain'"
                .to_string()
        } //panic!("Could not read file resend.toml");
    };

    let file_conf: Fileconf = match toml::from_str(&contents) {
        Ok(d) => d,
        Err(_) => {
            panic!("File resend.toml is corrupt");
        }
    };
    // End of temp section

    let config = Config {
        //node: Url::from_str("http://localhost:26657").unwrap(),
        node: Url::from_str(&file_conf.tendermint_url).unwrap(),
        home: get_default_home_dir(APP_NAME).unwrap(),
        //from: "kevin".into(),
        //chain_id: tendermint_informal::chain::Id::try_from("test-chain").unwrap(),
        from: file_conf.from_user,
        chain_id: tendermint_informal::chain::Id::try_from(file_conf.chain_id).unwrap(),
        delay: 6,
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
