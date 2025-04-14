#![allow(clippy::missing_errors_doc)]
#![allow(clippy::unnecessary_struct_initialization)]
#![allow(clippy::unused_async)]
use loco_rs::prelude::*;
use axum::extract::State;
use axum_session::{Session, SessionNullPool};
use serde::Serialize;

#[derive(Serialize)]
pub struct SessionInfo {
    count: i32,
}

pub async fn get_session(
    State(_ctx): State<AppContext>,
    session: Session<SessionNullPool>,
) -> Result<Response> {
    // Get or initialize counter
    let count: i32 = session.get("counter").unwrap_or(0);
    let new_count = count + 1;
    
    // Store new count
    session.set("counter", new_count);

    // Return JSON response
    format::json(())
}

pub fn routes() -> Routes {
    Routes::new()
        .prefix("session")
        .add("/", get(get_session))
}
