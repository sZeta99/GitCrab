#![allow(clippy::missing_errors_doc)]
#![allow(clippy::unnecessary_struct_initialization)]
#![allow(clippy::unused_async)]
use loco_rs::prelude::*;
use axum::debug_handler;

use crate::views;
#[debug_handler]
pub async fn login(
    ViewEngine(v): ViewEngine<TeraView>,
    State(_): State<AppContext>,
) -> Result<Response> {
    views::auth::login_view(&v)
}


pub fn routes() -> Routes {
    Routes::new()
        .prefix("login/")
            .add("/", get(login))
}
