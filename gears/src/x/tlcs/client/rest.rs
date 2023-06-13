use proto_messages::azkr::tlcs::v1beta1::QueryAllParticipantsContributionsResponse;

use rocket::State;
use rocket::{get, serde::json::Json};

use crate::app::BaseApp;
use crate::types::QueryContext;

use crate::client::rest::{Error, Pagination};
use crate::x::tlcs::{
    query_all_contributions,
    query_contributions_by_address,
    query_contributions_by_round_and_scheme,
    query_keys_by_round,
    query_all_keys,
};

/// Get all contributions
#[get("/azkr/tlcs/v1beta1/contributions?<pagination>")]
#[allow(unused_variables)]
pub async fn get_all_participants_contributions(
    app: &State<BaseApp>,
    pagination: Pagination,
) -> Result<Json<QueryAllParticipantsContributionsResponse>, Error> {
    let store = app.multi_store.read().expect("RwLock will not be poisoned");
    let ctx = QueryContext::new(&store, app.get_block_height());

    Ok(Json(query_all_contributions(&ctx)))
}

/// Get all contributions by participant address
#[get("/azkr/tlcs/v1beta1/contributions_by_address/<address>?<pagination>")]
#[allow(unused_variables)]
pub async fn get_contributions_by_address(
    app: &State<BaseApp>,
    address: &str,
    pagination: Pagination,
) -> Result<Json<QueryAllParticipantsContributionsResponse>, Error> {
    let store = app.multi_store.read().expect("RwLock will not be poisoned");
    let ctx = QueryContext::new(&store, app.get_block_height());

    Ok(Json(query_contributions_by_address(&ctx)))
}

/// Get all contributions for a round and scheme
#[get("/azkr/tlcs/v1beta1/contributions_by_round_and_scheme/<round>/<scheme>?<pagination>")]
#[allow(unused_variables)]
pub async fn get_contributions_by_round_and_scheme(
    app: &State<BaseApp>,
    round: u64,
    scheme: u64,
    pagination: Pagination,
) -> Result<Json<QueryAllParticipantsContributionsResponse>, Error> {
    let store = app.multi_store.read().expect("RwLock will not be poisoned");
    let ctx = QueryContext::new(&store, app.get_block_height());

    Ok(Json(query_contributions_by_round_and_scheme(&ctx)))
}

/// Get all keys for a given round
#[get("/azkr/tlcs/v1beta1/keys/<round>?<pagination>")]
#[allow(unused_variables)]
pub async fn get_keys_by_round(
    app: &State<BaseApp>,
    round: u64,
    pagination: Pagination,
) -> Result<Json<QueryAllParticipantsContributionsResponse>, Error> {
    let store = app.multi_store.read().expect("RwLock will not be poisoned");
    let ctx = QueryContext::new(&store, app.get_block_height());

    Ok(Json(query_keys_by_round(&ctx)))
}

/// Get all keys
#[get("/azkr/tlcs/v1beta1/keys?<pagination>")]
#[allow(unused_variables)]
pub async fn get_all_keys(
    app: &State<BaseApp>,
    pagination: Pagination,
) -> Result<Json<QueryAllParticipantsContributionsResponse>, Error> {
    let store = app.multi_store.read().expect("RwLock will not be poisoned");
    let ctx = QueryContext::new(&store, app.get_block_height());

    Ok(Json(query_all_keys(&ctx)))
}
