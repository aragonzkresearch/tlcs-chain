use bytes::Bytes;
use ibc_proto::protobuf::Protobuf;
use prost::Message as ProstMessage;
use std::hash::Hash;
use tendermint_abci::Application;

use axum::{
    body::Body,
    extract::{Path, Query, State},
    routing::get,
    Json, Router,
};
use gears::{
    baseapp::{
        ante::{AuthKeeper, BankKeeper},
        BaseApp, Handler,
    },
    client::rest::{error::Error, Pagination},
    x::params::ParamsSubspaceKey,
};
use proto_messages::cosmos::tx::v1beta1::Message;
use serde::de::DeserializeOwned;
use store::StoreKey;
use strum::IntoEnumIterator;
use tendermint_proto::abci::RequestQuery;

use crate::proto::tlcs::v1beta1::{
    QueryAllContributionsResponse, QueryAllKeyPairsResponse, QueryAllLoeDataResponse,
    QueryRoundRequest, QueryRoundSchemeRequest, QueryTimeRequest,
};

/// Get all contributions
pub async fn get_all_contributions<
    SK: Hash + Eq + IntoEnumIterator + StoreKey + Clone + Send + Sync + 'static,
    PSK: ParamsSubspaceKey + Clone + Send + Sync + 'static,
    M: Message,
    BK: BankKeeper<SK> + Clone + Send + Sync + 'static,
    AK: AuthKeeper<SK> + Clone + Send + Sync + 'static,
    H: Handler<M, SK, G> + 'static,
    G: DeserializeOwned + Clone + Send + Sync + 'static,
>(
    _pagination: Query<Pagination>,
    State(app): State<BaseApp<SK, PSK, M, BK, AK, H, G>>,
) -> Result<Json<QueryAllContributionsResponse>, Error> {
    let request = RequestQuery {
        data: Default::default(),
        path: "/azkr.tlcs.v1beta1.Query/AllContributions".into(),
        height: 0,
        prove: false,
    };

    let response = app.query(request);

    Ok(Json(
        QueryAllContributionsResponse::decode(response.value)
            .expect("should be a valid QueryAllContributionsResponse"),
    ))
}

/// Get all contributions by round
pub async fn get_contributions_by_round<
    SK: Hash + Eq + IntoEnumIterator + StoreKey + Clone + Send + Sync + 'static,
    PSK: ParamsSubspaceKey + Clone + Send + Sync + 'static,
    M: Message,
    BK: BankKeeper<SK> + Clone + Send + Sync + 'static,
    AK: AuthKeeper<SK> + Clone + Send + Sync + 'static,
    H: Handler<M, SK, G> + 'static,
    G: DeserializeOwned + Clone + Send + Sync + 'static,
>(
    Path(round): Path<u64>,
    _pagination: Query<Pagination>,
    State(app): State<BaseApp<SK, PSK, M, BK, AK, H, G>>,
) -> Result<Json<QueryAllContributionsResponse>, Error> {
    let req = QueryRoundRequest { round };
    let request = RequestQuery {
        data: req.encode_vec().into(),
        path: "/azkr.tlcs.v1beta1.Query/AllContributionsByRound".into(),
        height: 0,
        prove: false,
    };

    let response = app.query(request);

    Ok(Json(
        QueryAllContributionsResponse::decode(response.value)
            .expect("should be a valid QueryAllContributionsResponse"),
    ))
}

/// Get all contributions for a round and scheme
pub async fn get_contributions_by_round_and_scheme<
    SK: Hash + Eq + IntoEnumIterator + StoreKey + Clone + Send + Sync + 'static,
    PSK: ParamsSubspaceKey + Clone + Send + Sync + 'static,
    M: Message,
    BK: BankKeeper<SK> + Clone + Send + Sync + 'static,
    AK: AuthKeeper<SK> + Clone + Send + Sync + 'static,
    H: Handler<M, SK, G> + 'static,
    G: DeserializeOwned + Clone + Send + Sync + 'static,
>(
    Path(round): Path<u64>,
    Path(scheme): Path<u32>,
    _pagination: Query<Pagination>,
    State(app): State<BaseApp<SK, PSK, M, BK, AK, H, G>>,
) -> Result<Json<QueryAllContributionsResponse>, Error> {
    let req = QueryRoundSchemeRequest { round, scheme };
    let request = RequestQuery {
        data: req.encode_vec().into(),
        path: "/azkr.tlcs.v1beta1.Query/AllContributionsByRoundAndScheme".into(),
        height: 0,
        prove: false,
    };

    let response = app.query(request);

    Ok(Json(
        QueryAllContributionsResponse::decode(response.value)
            .expect("should be a valid QueryAllContributionsResponse"),
    ))
}

/// Get all keypairs
pub async fn get_all_keypairs<
    SK: Hash + Eq + IntoEnumIterator + StoreKey + Clone + Send + Sync + 'static,
    PSK: ParamsSubspaceKey + Clone + Send + Sync + 'static,
    M: Message,
    BK: BankKeeper<SK> + Clone + Send + Sync + 'static,
    AK: AuthKeeper<SK> + Clone + Send + Sync + 'static,
    H: Handler<M, SK, G> + 'static,
    G: DeserializeOwned + Clone + Send + Sync + 'static,
>(
    _pagination: Query<Pagination>,
    State(app): State<BaseApp<SK, PSK, M, BK, AK, H, G>>,
) -> Result<Json<QueryAllKeyPairsResponse>, Error> {
    let request = RequestQuery {
        data: Bytes::new(),
        path: "/azkr.tlcs.v1beta1.Query/AllKeyPairs".into(),
        height: 0,
        prove: false,
    };

    let response = app.query(request);

    Ok(Json(
        QueryAllKeyPairsResponse::decode(response.value)
            .expect("should be a valid QueryAllKeyPairsResponse"),
    ))
}

/// Get all keys for a given round
pub async fn get_keypairs_by_round<
    SK: Hash + Eq + IntoEnumIterator + StoreKey + Clone + Send + Sync + 'static,
    PSK: ParamsSubspaceKey + Clone + Send + Sync + 'static,
    M: Message,
    BK: BankKeeper<SK> + Clone + Send + Sync + 'static,
    AK: AuthKeeper<SK> + Clone + Send + Sync + 'static,
    H: Handler<M, SK, G> + 'static,
    G: DeserializeOwned + Clone + Send + Sync + 'static,
>(
    Path(round): Path<u64>,
    _pagination: Query<Pagination>,
    State(app): State<BaseApp<SK, PSK, M, BK, AK, H, G>>,
) -> Result<Json<QueryAllKeyPairsResponse>, Error> {
    let req = QueryRoundRequest { round };
    let request = RequestQuery {
        data: req.encode_vec().into(),
        path: "/azkr.tlcs.v1beta1.Query/AllKeyPairsByRound".into(),
        height: 0,
        prove: false,
    };

    let response = app.query(request);

    Ok(Json(
        QueryAllKeyPairsResponse::decode(response.value)
            .expect("should be a valid QueryAllKeyPairsResponse"),
    ))
}

/// Get all keys by time
pub async fn get_keypairs_by_time<
    SK: Hash + Eq + IntoEnumIterator + StoreKey + Clone + Send + Sync + 'static,
    PSK: ParamsSubspaceKey + Clone + Send + Sync + 'static,
    M: Message,
    BK: BankKeeper<SK> + Clone + Send + Sync + 'static,
    AK: AuthKeeper<SK> + Clone + Send + Sync + 'static,
    H: Handler<M, SK, G> + 'static,
    G: DeserializeOwned + Clone + Send + Sync + 'static,
>(
    Path(time): Path<i64>,
    _pagination: Query<Pagination>,
    State(app): State<BaseApp<SK, PSK, M, BK, AK, H, G>>,
) -> Result<Json<QueryAllKeyPairsResponse>, Error> {
    let req = QueryTimeRequest { time };
    let request = RequestQuery {
        data: req.encode_vec().into(),
        path: "/azkr.tlcs.v1beta1.Query/AllKeyPairsByTime".into(),
        height: 0,
        prove: false,
    };

    let response = app.query(request);

    Ok(Json(
        QueryAllKeyPairsResponse::decode(response.value)
            .expect("should be a valid QueryAllKeyPairsResponse"),
    ))
}

/// Get all keys for a given round and scheme
pub async fn get_keypairs_by_round_and_scheme<
    SK: Hash + Eq + IntoEnumIterator + StoreKey + Clone + Send + Sync + 'static,
    PSK: ParamsSubspaceKey + Clone + Send + Sync + 'static,
    M: Message,
    BK: BankKeeper<SK> + Clone + Send + Sync + 'static,
    AK: AuthKeeper<SK> + Clone + Send + Sync + 'static,
    H: Handler<M, SK, G> + 'static,
    G: DeserializeOwned + Clone + Send + Sync + 'static,
>(
    Path(round): Path<u64>,
    Path(scheme): Path<u32>,
    _pagination: Query<Pagination>,
    State(app): State<BaseApp<SK, PSK, M, BK, AK, H, G>>,
) -> Result<Json<QueryAllKeyPairsResponse>, Error> {
    let req = QueryRoundSchemeRequest { round, scheme };
    let request = RequestQuery {
        data: req.encode_vec().into(),
        path: "/azkr.tlcs.v1beta1.Query/AllKeyPairsByRoundAndScheme".into(),
        height: 0,
        prove: false,
    };

    let response = app.query(request);

    Ok(Json(
        QueryAllKeyPairsResponse::decode(response.value)
            .expect("should be a valid QueryAllKeyPairsResponse"),
    ))
}

/// Get all loe_data
pub async fn get_all_loe_data<
    SK: Hash + Eq + IntoEnumIterator + StoreKey + Clone + Send + Sync + 'static,
    PSK: ParamsSubspaceKey + Clone + Send + Sync + 'static,
    M: Message,
    BK: BankKeeper<SK> + Clone + Send + Sync + 'static,
    AK: AuthKeeper<SK> + Clone + Send + Sync + 'static,
    H: Handler<M, SK, G> + 'static,
    G: DeserializeOwned + Clone + Send + Sync + 'static,
>(
    _pagination: Query<Pagination>,
    State(app): State<BaseApp<SK, PSK, M, BK, AK, H, G>>,
) -> Result<Json<QueryAllLoeDataResponse>, Error> {
    let request = RequestQuery {
        data: Bytes::new(),
        path: "/azkr.tlcs.v1beta1.Query/AllLoeData".into(),
        height: 0,
        prove: false,
    };

    let response = app.query(request);

    Ok(Json(
        QueryAllLoeDataResponse::decode(response.value)
            .expect("should be a valid QueryAllLoeDataResponse"),
    ))
}

/// Get all loe_data by round
pub async fn get_loe_data_by_round<
    SK: Hash + Eq + IntoEnumIterator + StoreKey + Clone + Send + Sync + 'static,
    PSK: ParamsSubspaceKey + Clone + Send + Sync + 'static,
    M: Message,
    BK: BankKeeper<SK> + Clone + Send + Sync + 'static,
    AK: AuthKeeper<SK> + Clone + Send + Sync + 'static,
    H: Handler<M, SK, G> + 'static,
    G: DeserializeOwned + Clone + Send + Sync + 'static,
>(
    Path(round): Path<u64>,
    _pagination: Query<Pagination>,
    State(app): State<BaseApp<SK, PSK, M, BK, AK, H, G>>,
) -> Result<Json<QueryAllLoeDataResponse>, Error> {
    let req = QueryRoundRequest { round };
    let request = RequestQuery {
        data: req.encode_vec().into(),
        path: "/azkr.tlcs.v1beta1.Query/AllLoeDataByRound".into(),
        height: 0,
        prove: false,
    };

    let response = app.query(request);

    Ok(Json(
        QueryAllLoeDataResponse::decode(response.value)
            .expect("should be a valid QueryAllLoeDataResponse"),
    ))
}

pub fn get_router<
    SK: Hash + Eq + IntoEnumIterator + StoreKey + Clone + Send + Sync + 'static,
    PSK: ParamsSubspaceKey + Clone + Send + Sync + 'static,
    M: Message,
    BK: BankKeeper<SK> + Clone + Send + Sync + 'static,
    AK: AuthKeeper<SK> + Clone + Send + Sync + 'static,
    H: Handler<M, SK, G> + 'static,
    G: DeserializeOwned + Clone + Send + Sync + 'static,
>() -> Router<BaseApp<SK, PSK, M, BK, AK, H, G>, Body> {
    Router::new()
        .route(
            "/azkr/tlcs/v1beta1/contributions",
            get(get_all_contributions),
        )
        .route(
            "/azkr/tlcs/v1beta1/contributions_by_round/:round",
            get(get_contributions_by_round),
        )
        .route(
            "/azkr/tlcs/v1beta1/contributions_by_round_and_scheme/:round/:scheme",
            get(get_contributions_by_round_and_scheme),
        )
        .route("/azkr/tlcs/v1beta1/keypairs", get(get_all_keypairs))
        .route(
            "/azkr/tlcs/v1beta1/keypairs/round/:round",
            get(get_keypairs_by_round),
        )
        .route(
            "/azkr/tlcs/v1beta1/keypairs/time/:time",
            get(get_keypairs_by_time),
        )
        .route(
            "/azkr/tlcs/v1beta1/keypairs/round_and_scheme/:round/:scheme",
            get(get_keypairs_by_round_and_scheme),
        )
        .route("/azkr/tlcs/v1beta1/loe_data", get(get_all_loe_data))
        .route(
            "/azkr/tlcs/v1beta1/loe_data/round/:round",
            get(get_loe_data_by_round),
        )
}
