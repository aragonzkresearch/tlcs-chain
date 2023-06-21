use proto_messages::azkr::tlcs::v1beta1::QueryAllParticipantsContributionsResponse;

use rocket::State;
use rocket::{get, serde::json::Json};

use crate::app::BaseApp;
use crate::types::QueryContext;

use crate::client::rest::{Error, Pagination};
use crate::x::tlcs::query_all_participant_contributions;

/// Get all balances for a given address
#[get("/azkr/tlcs/v1beta1/contributions?<pagination>")]
#[allow(unused_variables)]
pub async fn get_all_participants_contributions(
    app: &State<BaseApp>,
    pagination: Pagination,
) -> Result<Json<QueryAllParticipantsContributionsResponse>, Error> {
    let store = app.multi_store.read().expect("RwLock will not be poisoned");
    let ctx = QueryContext::new(&store, app.get_block_height());

    Ok(Json(query_all_participant_contributions(&ctx)))
}
