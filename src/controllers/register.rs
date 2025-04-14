#![allow(clippy::missing_errors_doc)]
#![allow(clippy::unnecessary_struct_initialization)]
#![allow(clippy::unused_async)]
use axum::debug_handler;
use loco_rs::prelude::*;
use crate::views;

#[debug_handler]
pub async fn register(
    ViewEngine(v): ViewEngine<TeraView>,
    State(_): State<AppContext>,
) -> Result<Response> {
    views::auth::register_view(&v)
}


pub fn routes() -> Routes {
    Routes::new()
        .prefix("register/")
            .add("/", get(register))

}
