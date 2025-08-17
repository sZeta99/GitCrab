#![allow(clippy::missing_errors_doc)]
#![allow(clippy::unnecessary_struct_initialization)]
#![allow(clippy::unused_async)]
use loco_rs::{controller::middleware, prelude::*};

use axum::debug_handler;

use crate::views;
#[debug_handler]
pub async fn dashboard(
    auth: middleware::auth::JWT,
    ViewEngine(v): ViewEngine<TeraView>,
    State(_): State<AppContext>,
) -> Result<Response> {
    views::home::dashboard(&v)
}
pub async fn home(
    ViewEngine(v): ViewEngine<TeraView>,
    State(_): State<AppContext>,
) -> Result<Response> {
    views::home::home(&v)
}

pub fn routes() -> Routes {
    Routes::new()
        .prefix("/")
        .add("/dashboard", get(dashboard))
        .add("/", get(home))

}

