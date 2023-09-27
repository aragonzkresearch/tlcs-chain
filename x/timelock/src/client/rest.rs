use bytes::Bytes;
use ibc_proto::protobuf::Protobuf;
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
        BaseApp, Genesis, Handler,
    },
    client::rest::{error::Error, Pagination, RestState},
    x::params::ParamsSubspaceKey,
};
use proto_messages::cosmos::tx::v1beta1::Message;
use store::StoreKey;
use tendermint_proto::abci::RequestQuery;

use crate::proto::tlcs::v1beta1::{
    QueryAllContributionsResponse, QueryAllKeyPairsResponse, QueryAllLoeDataResponse,
    QueryRoundRequest, QueryRoundSchemeRequest, QueryTimeRequest,
};

/// Get all contributions
pub async fn get_all_contributions<
    SK: StoreKey,
    PSK: ParamsSubspaceKey,
    M: Message,
    BK: BankKeeper<SK>,
    AK: AuthKeeper<SK>,
    H: Handler<M, SK, G>,
    G: Genesis,
>(
    _pagination: Query<Pagination>,
    State(app): State<BaseApp<SK, PSK, M, BK, AK, H, G>>,
) -> Result<Json<QueryAllContributionsResponse>, Error> {
    let request = RequestQuery {
        data: Default::default(),
        path: "/tlcs.timelock.v1beta1.Query/AllContributions".into(),
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
    SK: StoreKey,
    PSK: ParamsSubspaceKey,
    M: Message,
    BK: BankKeeper<SK>,
    AK: AuthKeeper<SK>,
    H: Handler<M, SK, G>,
    G: Genesis,
>(
    Path(round): Path<u64>,
    _pagination: Query<Pagination>,
    State(app): State<BaseApp<SK, PSK, M, BK, AK, H, G>>,
) -> Result<Json<QueryAllContributionsResponse>, Error> {
    let req = QueryRoundRequest { round };
    let request = RequestQuery {
        data: req.encode_vec().into(),
        path: "/tlcs.timelock.v1beta1.Query/AllContributionsByRound".into(),
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
    SK: StoreKey,
    PSK: ParamsSubspaceKey,
    M: Message,
    BK: BankKeeper<SK>,
    AK: AuthKeeper<SK>,
    H: Handler<M, SK, G>,
    G: Genesis,
>(
    Path((round, scheme)): Path<(u64, u32)>,
    _pagination: Query<Pagination>,
    State(app): State<BaseApp<SK, PSK, M, BK, AK, H, G>>,
) -> Result<Json<QueryAllContributionsResponse>, Error> {
    let req = QueryRoundSchemeRequest { round, scheme };
    let request = RequestQuery {
        data: req.encode_vec().into(),
        path: "/tlcs.timelock.v1beta1.Query/AllContributionsByRoundAndScheme".into(),
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
    SK: StoreKey,
    PSK: ParamsSubspaceKey,
    M: Message,
    BK: BankKeeper<SK>,
    AK: AuthKeeper<SK>,
    H: Handler<M, SK, G>,
    G: Genesis,
>(
    _pagination: Query<Pagination>,
    State(app): State<BaseApp<SK, PSK, M, BK, AK, H, G>>,
) -> Result<Json<QueryAllKeyPairsResponse>, Error> {
    let request = RequestQuery {
        data: Bytes::new(),
        path: "/tlcs.timelock.v1beta1.Query/AllKeyPairs".into(),
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
    SK: StoreKey,
    PSK: ParamsSubspaceKey,
    M: Message,
    BK: BankKeeper<SK>,
    AK: AuthKeeper<SK>,
    H: Handler<M, SK, G>,
    G: Genesis,
>(
    Path(round): Path<u64>,
    _pagination: Query<Pagination>,
    State(app): State<BaseApp<SK, PSK, M, BK, AK, H, G>>,
) -> Result<Json<QueryAllKeyPairsResponse>, Error> {
    let req = QueryRoundRequest { round };
    let request = RequestQuery {
        data: req.encode_vec().into(),
        path: "/tlcs.timelock.v1beta1.Query/AllKeyPairsByRound".into(),
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
    SK: StoreKey,
    PSK: ParamsSubspaceKey,
    M: Message,
    BK: BankKeeper<SK>,
    AK: AuthKeeper<SK>,
    H: Handler<M, SK, G>,
    G: Genesis,
>(
    Path(time): Path<i64>,
    _pagination: Query<Pagination>,
    State(app): State<BaseApp<SK, PSK, M, BK, AK, H, G>>,
) -> Result<Json<QueryAllKeyPairsResponse>, Error> {
    let req = QueryTimeRequest { time };
    let request = RequestQuery {
        data: req.encode_vec().into(),
        path: "/tlcs.timelock.v1beta1.Query/AllKeyPairsByTime".into(),
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
    SK: StoreKey,
    PSK: ParamsSubspaceKey,
    M: Message,
    BK: BankKeeper<SK>,
    AK: AuthKeeper<SK>,
    H: Handler<M, SK, G>,
    G: Genesis,
>(
    Path((round, scheme)): Path<(u64, u32)>,
    _pagination: Query<Pagination>,
    State(app): State<BaseApp<SK, PSK, M, BK, AK, H, G>>,
) -> Result<Json<QueryAllKeyPairsResponse>, Error> {
    let req = QueryRoundSchemeRequest { round, scheme };
    let request = RequestQuery {
        data: req.encode_vec().into(),
        path: "/tlcs.timelock.v1beta1.Query/AllKeyPairsByRoundAndScheme".into(),
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
    SK: StoreKey,
    PSK: ParamsSubspaceKey,
    M: Message,
    BK: BankKeeper<SK>,
    AK: AuthKeeper<SK>,
    H: Handler<M, SK, G>,
    G: Genesis,
>(
    _pagination: Query<Pagination>,
    State(app): State<BaseApp<SK, PSK, M, BK, AK, H, G>>,
) -> Result<Json<QueryAllLoeDataResponse>, Error> {
    let request = RequestQuery {
        data: Bytes::new(),
        path: "/tlcs.timelock.v1beta1.Query/AllLoeData".into(),
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
    SK: StoreKey,
    PSK: ParamsSubspaceKey,
    M: Message,
    BK: BankKeeper<SK>,
    AK: AuthKeeper<SK>,
    H: Handler<M, SK, G>,
    G: Genesis,
>(
    Path(round): Path<u64>,
    _pagination: Query<Pagination>,
    State(app): State<BaseApp<SK, PSK, M, BK, AK, H, G>>,
) -> Result<Json<QueryAllLoeDataResponse>, Error> {
    let req = QueryRoundRequest { round };
    let request = RequestQuery {
        data: req.encode_vec().into(),
        path: "/tlcs.timelock.v1beta1.Query/AllLoeDataByRound".into(),
        height: 0,
        prove: false,
    };

    let response = app.query(request);

    Ok(Json(
        QueryAllLoeDataResponse::decode(response.value)
            .expect("should be a valid QueryAllLoeDataResponse"),
    ))
}

/// Get all keys for a given round
pub async fn get_loe_data_needed<
    SK: StoreKey,
    PSK: ParamsSubspaceKey,
    M: Message,
    BK: BankKeeper<SK>,
    AK: AuthKeeper<SK>,
    H: Handler<M, SK, G>,
    G: Genesis,
>(
    _pagination: Query<Pagination>,
    State(app): State<BaseApp<SK, PSK, M, BK, AK, H, G>>,
) -> Result<Json<QueryAllKeyPairsResponse>, Error> {
    let request = RequestQuery {
        data: Bytes::new(),
        path: "/tlcs.timelock.v1beta1.Query/AllLoeDataNeeded".into(),
        height: 0,
        prove: false,
    };

    let response = app.query(request);

    Ok(Json(
        QueryAllKeyPairsResponse::decode(response.value)
            .expect("should be a valid QueryAllKeyPairsResponse"),
    ))
}

async fn endpoint_info() -> &'static str {
    "TLCS rest endpoints:\n\n\
     \t /tlcs/timelock/v1beta1/contributions\n\
     \t /tlcs/timelock/v1beta1/contributions_by_round/<round>\n\
     \t /tlcs/timelock/v1beta1/contributions_by_round_and_scheme/<round>/<scheme>\n\
     \t /tlcs/timelock/v1beta1/keypairs\n\
     \t /tlcs/timelock/v1beta1/keypairs/round/<round>\n\
     \t /tlcs/timelock/v1beta1/keypairs/time/<time>\n\
     \t /tlcs/timelock/v1beta1/keypairs/round_and_scheme/<round>/<scheme>\n\
     \t /tlcs/timelock/v1beta1/loe_data\n\
     \t /tlcs/timelock/v1beta1/loe_data/round/<round>\n\
     \t /tlcs/timelock/v1beta1/loe_data_needed\n\
    "
}

pub fn get_router<
    SK: StoreKey,
    PSK: ParamsSubspaceKey,
    M: Message,
    BK: BankKeeper<SK>,
    AK: AuthKeeper<SK>,
    H: Handler<M, SK, G>,
    G: Genesis,
>() -> Router<RestState<SK, PSK, M, BK, AK, H, G>, Body> {
    Router::new()
        .route("/", get(endpoint_info))
        .route("/contributions", get(get_all_contributions))
        .route(
            "/contributions_by_round/:round",
            get(get_contributions_by_round),
        )
        .route(
            "/contributions_by_round_and_scheme/:round/:scheme",
            get(get_contributions_by_round_and_scheme),
        )
        .route("/keypairs", get(get_all_keypairs))
        .route("/keypairs/round/:round", get(get_keypairs_by_round))
        .route("/keypairs/time/:time", get(get_keypairs_by_time))
        .route(
            "/keypairs/round_and_scheme/:round/:scheme",
            get(get_keypairs_by_round_and_scheme),
        )
        .route("/loe_data", get(get_all_loe_data))
        .route("/loe_data/round/:round", get(get_loe_data_by_round))
        .route("/loe_data_needed", get(get_loe_data_needed))
}
