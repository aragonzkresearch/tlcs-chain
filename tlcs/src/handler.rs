use gears::x::params::Keeper as ParamsKeeper;
use proto_messages::cosmos::base::v1beta1::SendCoins;
use proto_types::AccAddress;
use tendermint_proto::abci::RequestQuery;

use database::Database;
use gears::{
    error::AppError,
    types::context::{Context, QueryContext},
};

use crate::{
    genesis::GenesisState,
    message::Message,
    store_keys::{TlcsParamsStoreKey, TlcsStoreKey},
};

#[derive(Debug, Clone)]
pub struct Handler {
    bank_handler: bank::Handler<TlcsStoreKey, TlcsParamsStoreKey>,
    auth_handler: auth::Handler<TlcsStoreKey, TlcsParamsStoreKey>,
}

impl Handler {
    pub fn new() -> Handler {
        let params_keeper = ParamsKeeper::new(TlcsStoreKey::Params);

        let auth_keeper = auth::Keeper::new(
            TlcsStoreKey::Auth,
            params_keeper.clone(),
            TlcsParamsStoreKey::Auth,
        );

        let bank_keeper = bank::Keeper::new(
            TlcsStoreKey::Bank,
            params_keeper,
            TlcsParamsStoreKey::Bank,
            auth_keeper.clone(),
        );

        Handler {
            bank_handler: bank::Handler::new(bank_keeper),
            auth_handler: auth::Handler::new(auth_keeper),
        }
    }
}

impl gears::baseapp::Handler<Message, TlcsStoreKey, GenesisState> for Handler {
    fn handle_tx<DB: Database>(
        &self,
        ctx: &mut Context<DB, TlcsStoreKey>,
        msg: &Message,
    ) -> Result<(), AppError> {
        match msg {
            Message::Bank(msg) => self.bank_handler.handle(ctx, msg),
        }
    }

    fn handle_init_genesis<DB: Database>(
        &self,
        ctx: &mut Context<DB, TlcsStoreKey>,
        genesis: GenesisState,
    ) {
        self.bank_handler.init_genesis(ctx, genesis.bank);
        self.auth_handler.init_genesis(ctx, genesis.auth);
    }

    fn handle_query<DB: Database>(
        &self,
        ctx: &QueryContext<DB, TlcsStoreKey>,
        query: RequestQuery,
    ) -> Result<bytes::Bytes, AppError> {
        if query.path.starts_with("/cosmos.auth") {
            self.auth_handler.handle_query(ctx, query)
        } else if query.path.starts_with("/cosmos.bank") {
            self.bank_handler.handle_query(ctx, query)
        } else {
            Err(AppError::InvalidRequest("query path not found".into()))
        }
    }

    fn handle_add_genesis_account(
        &self,
        genesis_state: &mut GenesisState,
        address: AccAddress,
        coins: SendCoins,
    ) -> Result<(), AppError> {
        self.auth_handler
            .handle_add_genesis_account(&mut genesis_state.auth, address.clone())?;
        self.bank_handler
            .handle_add_genesis_account(&mut genesis_state.bank, address, coins);

        Ok(())
    }
}
