use async_trait::async_trait;
use axum::Router;

use loco_rs::app::{AppContext, Initializer};
use loco_rs::Result;
#[allow(clippy::module_name_repetitions)]
pub struct AxumSessionInitializer;
#[async_trait]
impl Initializer for AxumSessionInitializer {
    fn name(&self) -> String {
        "axum-session".to_string()
    }

    async fn after_routes(&self, router: Router, _ctx: &AppContext) -> Result<Router> {
        let session_config =
            axum_session::SessionConfig::default().with_table_name("sessions_table");
        let session_store =
            axum_session::SessionStore::<axum_session::SessionNullPool>::new(None, session_config)
                .await
                .unwrap();
        Ok(router.layer(axum_session::SessionLayer::new(session_store)))
    }
}

