use proto_messages::azkr::tlcs::v1beta1::{
    QueryAllContributionsResponse,
    QueryAllKeyPairsResponse,
    QueryAllLoeDataResponse,
};

use rocket::State;
use rocket::{get, serde::json::Json};

use crate::app::BaseApp;
use crate::types::QueryContext;

use crate::client::rest::{Error, Pagination};

use crate::x::tlcs::{
    query_all_contributions,
    query_contributions_by_round,
    query_contributions_by_round_and_scheme,
    query_all_keypairs,
    query_keypairs_by_round,
    query_keypairs_by_round_and_scheme,
    query_keypairs_by_time,
    query_all_loe_data,
    query_loe_data_by_round,
};

/// Get all contributions
#[get("/azkr/tlcs/v1beta1/contributions?<pagination>")]
#[allow(unused_variables)]
pub async fn get_all_contributions(
    app: &State<BaseApp>,
    pagination: Pagination,
) -> Result<Json<QueryAllContributionsResponse>, Error> {
    let store = app.multi_store.read().expect("RwLock will not be poisoned");
    let ctx = QueryContext::new(&store, app.get_block_height());

    Ok(Json(query_all_contributions(&ctx)))
}

/// Get all contributions by round
#[get("/azkr/tlcs/v1beta1/contributions_by_round/<round>?<pagination>")]
#[allow(unused_variables)]
pub async fn get_contributions_by_round(
    app: &State<BaseApp>,
    round: u32,
    pagination: Pagination,
) -> Result<Json<QueryAllContributionsResponse>, Error> {
    let store = app.multi_store.read().expect("RwLock will not be poisoned");
    let ctx = QueryContext::new(&store, app.get_block_height());

    Ok(Json(query_contributions_by_round(&ctx, round)))
}

/// Get all contributions for a round and scheme
#[get("/azkr/tlcs/v1beta1/contributions_by_round_and_scheme/<round>/<scheme>?<pagination>")]
#[allow(unused_variables)]
pub async fn get_contributions_by_round_and_scheme(
    app: &State<BaseApp>,
    round: u32,
    scheme: u32,
    pagination: Pagination,
) -> Result<Json<QueryAllContributionsResponse>, Error> {
    let store = app.multi_store.read().expect("RwLock will not be poisoned");
    let ctx = QueryContext::new(&store, app.get_block_height());

    Ok(Json(query_contributions_by_round_and_scheme(&ctx, round, scheme)))
}

/// Get all keypairs
#[get("/azkr/tlcs/v1beta1/keypairs?<pagination>")]
#[allow(unused_variables)]
pub async fn get_all_keypairs(
    app: &State<BaseApp>,
    pagination: Pagination,
) -> Result<Json<QueryAllKeyPairsResponse>, Error> {
    let store = app.multi_store.read().expect("RwLock will not be poisoned");
    let ctx = QueryContext::new(&store, app.get_block_height());

    Ok(Json(query_all_keypairs(&ctx)))
}

/// Get all keys for a given round
#[get("/azkr/tlcs/v1beta1/keypairs/round/<round>?<pagination>")]
#[allow(unused_variables)]
pub async fn get_keypairs_by_round(
    app: &State<BaseApp>,
    round: u32,
    pagination: Pagination,
) -> Result<Json<QueryAllKeyPairsResponse>, Error> {
    let store = app.multi_store.read().expect("RwLock will not be poisoned");
    let ctx = QueryContext::new(&store, app.get_block_height());

    Ok(Json(query_keypairs_by_round(&ctx, round)))
}

/// Get all keys for a given round
#[get("/azkr/tlcs/v1beta1/keypairs/time/<time>?<pagination>")]
#[allow(unused_variables)]
pub async fn get_keypairs_by_time(
    app: &State<BaseApp>,
    time: i64,
    pagination: Pagination,
) -> Result<Json<QueryAllKeyPairsResponse>, Error> {
    let store = app.multi_store.read().expect("RwLock will not be poisoned");
    let ctx = QueryContext::new(&store, app.get_block_height());

    Ok(Json(query_keypairs_by_time(&ctx, time)))
}

/// Get all keys for a given round and scheme
#[get("/azkr/tlcs/v1beta1/keypairs/round_and_scheme/<round>/<scheme>?<pagination>")]
#[allow(unused_variables)]
pub async fn get_keypairs_by_round_and_scheme(
    app: &State<BaseApp>,
    round: u32,
    scheme: u32,
    pagination: Pagination,
) -> Result<Json<QueryAllKeyPairsResponse>, Error> {
    let store = app.multi_store.read().expect("RwLock will not be poisoned");
    let ctx = QueryContext::new(&store, app.get_block_height());

    Ok(Json(query_keypairs_by_round_and_scheme(&ctx, round, scheme)))
}

/// Get all loe_data
#[get("/azkr/tlcs/v1beta1/loe_data?<pagination>")]
#[allow(unused_variables)]
pub async fn get_all_loe_data(
    app: &State<BaseApp>,
    pagination: Pagination,
) -> Result<Json<QueryAllLoeDataResponse>, Error> {
    let store = app.multi_store.read().expect("RwLock will not be poisoned");
    let ctx = QueryContext::new(&store, app.get_block_height());

    Ok(Json(query_all_loe_data(&ctx)))
}

/// Get all loe_data by round
#[get("/azkr/tlcs/v1beta1/loe_data_by_round/<round>?<pagination>")]
#[allow(unused_variables)]
pub async fn get_loe_data_by_round(
    app: &State<BaseApp>,
    round: u32,
    pagination: Pagination,
) -> Result<Json<QueryAllLoeDataResponse>, Error> {
    let store = app.multi_store.read().expect("RwLock will not be poisoned");
    let ctx = QueryContext::new(&store, app.get_block_height());

    Ok(Json(query_loe_data_by_round(&ctx, round)))

}
