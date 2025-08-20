#![allow(clippy::missing_errors_doc)]
#![allow(clippy::unnecessary_struct_initialization)]
#![allow(clippy::unused_async)]
use loco_rs::prelude::*;
use serde::{Deserialize, Serialize};
use axum::response::Redirect;
use axum_extra::extract::Form;
use sea_orm::{sea_query::Order, QueryOrder};
use axum::debug_handler;
use tracing::{event, Level};

use crate::{
    models::_entities::sshes::{ActiveModel, Column, Entity, Model}, services::ssh_service::SshKeyService, views
};

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Params {
    pub public_key: Option<String>,
    pub title: Option<String>,
    }

impl Params {
    fn update(&self, item: &mut ActiveModel) {
      item.public_key = Set(self.public_key.clone());
      item.title = Set(self.title.clone());
      }
}

async fn load_item(ctx: &AppContext, id: i32) -> Result<Model> {
    let item = Entity::find_by_id(id).one(&ctx.db).await?;
    item.ok_or_else(|| Error::NotFound)
}

#[debug_handler]
pub async fn list(
    ViewEngine(v): ViewEngine<TeraView>,
    State(ctx): State<AppContext>,
) -> Result<Response> {
    let item = Entity::find()
        .order_by(Column::Id, Order::Desc)
        .all(&ctx.db)
        .await?;
    views::ssh::list(&v, &item)
}

#[debug_handler]
pub async fn new(
    ViewEngine(v): ViewEngine<TeraView>,
    State(_ctx): State<AppContext>,
) -> Result<Response> {
    views::ssh::create(&v)
}

#[debug_handler]
pub async fn update(
    Path(id): Path<i32>,
    State(ctx): State<AppContext>,
    Form(params): Form<Params>,
) -> Result<Redirect> {
    let item = load_item(&ctx, id).await?;
    let saved = load_item(&ctx, id).await?;
    let mut item = item.into_active_model();
    params.update(&mut item);
    let item = item.update(&ctx.db).await?;
    let service = SshKeyService::new(env!("GIT_HOME")); 
    service.update_key(&saved,&item)
        .map_err(|e| Error::Message(format!("Failed to update key to authorized_keys: {e}")))?;

    Ok(Redirect::to("../sshes"))
}

#[debug_handler]
pub async fn edit(
    Path(id): Path<i32>,
    ViewEngine(v): ViewEngine<TeraView>,
    State(ctx): State<AppContext>,
) -> Result<Response> {
    let item = load_item(&ctx, id).await?;
    views::ssh::edit(&v, &item)
}

#[debug_handler]
pub async fn show(
    Path(id): Path<i32>,
    ViewEngine(v): ViewEngine<TeraView>,
    State(ctx): State<AppContext>,
) -> Result<Response> {
    let item = load_item(&ctx, id).await?;
    views::ssh::show(&v, &item)
}

#[debug_handler]
pub async fn add(
    State(ctx): State<AppContext>,
    Form(params): Form<Params>,
) -> Result<Redirect> {
    let mut item = ActiveModel {
        ..Default::default()
    };
    params.update(&mut item);
    let saved = item.insert(&ctx.db).await?;
    let service = SshKeyService::new(env!("GIT_HOME")); 
    service.add_key(&saved)
        .map_err(|e| Error::Message(format!("Failed to add key to authorized_keys: {e}")))?;
    Ok(Redirect::to("sshes"))
}

#[debug_handler]
pub async fn remove(Path(id): Path<i32>, State(ctx): State<AppContext>) -> Result<Response> {
    let saved = load_item(&ctx, id).await?;
    let service = SshKeyService::new(env!("GIT_HOME")); 
    service.remove_key(&saved)
        .map_err(|e| Error::Message(format!("Failed to remove key to authorized_keys: {e}")))?;
    saved.delete(&ctx.db).await?;
    format::empty()
}

pub fn routes() -> Routes {
    Routes::new()
        .prefix("sshes/")
        .add("/", get(list))
        .add("/", post(add))
        .add("new", get(new))
        .add("{id}", get(show))
        .add("{id}/edit", get(edit))
        .add("{id}", delete(remove))
        .add("{id}", post(update))
}
