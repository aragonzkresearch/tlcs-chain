use database::Database;
use gears::{error::AppError, types::context::TxContext};
use ibc_proto::protobuf::Protobuf;
use prost::Message as ProstMessage;
use store::StoreKey;

use crate::{
    proto::tlcs::v1beta1::{QueryRoundRequest, QueryRoundSchemeRequest, QueryTimeRequest},
    Keeper, Message,
};

#[derive(Debug, Clone)]
pub struct Handler<SK: StoreKey> {
    keeper: Keeper<SK>,
}

impl<SK: StoreKey> Handler<SK> {
    pub fn new(keeper: Keeper<SK>) -> Self {
        Handler { keeper }
    }

    pub fn handle<DB: Database>(
        &self,
        ctx: &mut TxContext<DB, SK>,
        msg: &Message,
    ) -> Result<(), AppError> {
        match msg {
            Message::NewProcess(msg) => self.keeper.open_new_process(ctx, msg),
            Message::Participate(msg) => self.keeper.append_contribution(ctx, msg),
            Message::SubmitLoeData(msg) => self.keeper.append_loe_data(&mut ctx.as_any(), msg),
        }
    }

    pub fn handle_query<DB: Database>(
        &self,
        ctx: &gears::types::context::QueryContext<DB, SK>,
        query: tendermint_proto::abci::RequestQuery,
    ) -> std::result::Result<bytes::Bytes, AppError> {
        match query.path.as_str() {
            "/azkr.tlcs.v1beta1.Query/AllContributions" => Ok(self
                .keeper
                .query_all_contributions(&ctx)
                .encode_to_vec()
                .into()),
            "/azkr.tlcs.v1beta1.Query/AllContributionsByRound" => {
                let data = query.data.clone();
                let req = QueryRoundRequest::decode(data)?;

                Ok(self
                    .keeper
                    .query_contributions_by_round(&ctx, req.round)
                    .encode_to_vec()
                    .into())
            }
            "/azkr.tlcs.v1beta1.Query/AllContributionsByRoundAndScheme" => {
                let data = query.data.clone();
                let req = QueryRoundSchemeRequest::decode(data)?;

                Ok(self
                    .keeper
                    .query_contributions_by_round_and_scheme(&ctx, req.round, req.scheme)
                    .encode_to_vec()
                    .into())
            }
            "/azkr.tlcs.v1beta1.Query/AllKeyPairs" => {
                Ok(self.keeper.query_all_keypairs(&ctx).encode_to_vec().into())
            }
            "/azkr.tlcs.v1beta1.Query/AllKeyPairsByRound" => {
                let data = query.data.clone();
                let req = QueryRoundRequest::decode(data)?;

                Ok(self
                    .keeper
                    .query_keypairs_by_round(&ctx, req.round)
                    .encode_to_vec()
                    .into())
            }
            "/azkr.tlcs.v1beta1.Query/AllKeyPairsByRoundAndScheme" => {
                let data = query.data.clone();
                let req = QueryRoundSchemeRequest::decode(data)?;

                Ok(self
                    .keeper
                    .query_keypairs_by_round_and_scheme(&ctx, req.round, req.scheme)
                    .encode_to_vec()
                    .into())
            }
            "/azkr.tlcs.v1beta1.Query/AllKeyPairsByTime" => {
                let data = query.data.clone();
                let req = QueryTimeRequest::decode(data)?;

                Ok(self
                    .keeper
                    .query_keypairs_by_time(&ctx, req.time)
                    .encode_to_vec()
                    .into())
            }
            "/azkr.tlcs.v1beta1.Query/AllLoeData" => {
                Ok(self.keeper.query_all_loe_data(&ctx).encode_to_vec().into())
            }
            "/azkr.tlcs.v1beta1.Query/AllLoeDataByRound" => {
                let data = query.data.clone();
                let req = QueryRoundRequest::decode(data)?;

                Ok(self
                    .keeper
                    .query_loe_data_by_round(&ctx, req.round)
                    .encode_to_vec()
                    .into())
            }
            _ => Err(AppError::InvalidRequest("query path not found".into())),
        }
    }
}
