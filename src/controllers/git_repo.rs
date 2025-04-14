#![allow(clippy::missing_errors_doc)]
#![allow(clippy::unnecessary_struct_initialization)]
#![allow(clippy::unused_async)]
use loco_rs::{controller::middleware, prelude::*};
use serde::{Deserialize, Serialize};
use axum::response::Redirect;
use axum_extra::extract::Form;
use sea_orm::{sea_query::Order, QueryOrder};
use axum::debug_handler;

use crate::{
    models::_entities::git_repos::{ActiveModel, Column, Entity, Model},
    views,
};

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Params {
    pub name: Option<String>,
    pub path: Option<String>,
    }

impl Params {
    fn update(&self, item: &mut ActiveModel) {
      item.name = Set(self.name.clone());
      item.path = Set(self.path.clone());
      }
}

async fn load_item(ctx: &AppContext, id: i32) -> Result<Model> {
    let item = Entity::find_by_id(id).one(&ctx.db).await?;
    item.ok_or_else(|| Error::NotFound)
}

#[debug_handler]
pub async fn list(
    auth: middleware::auth::JWT,
    ViewEngine(v): ViewEngine<TeraView>,
    State(ctx): State<AppContext>,
) -> Result<Response> {
    let item = Entity::find()
        .order_by(Column::Id, Order::Desc)
        .all(&ctx.db)
        .await?;
    views::git_repo::list(&v, &item)
}

#[debug_handler]
pub async fn new(
    auth: middleware::auth::JWT,
    ViewEngine(v): ViewEngine<TeraView>,
    State(_ctx): State<AppContext>,
) -> Result<Response> {
    views::git_repo::create(&v)
}

#[debug_handler]
pub async fn update(
    auth: middleware::auth::JWT,
    Path(id): Path<i32>,
    State(ctx): State<AppContext>,
    Form(params): Form<Params>,
) -> Result<Redirect> {
    let item = load_item(&ctx, id).await?;
    let mut item = item.into_active_model();
    params.update(&mut item);
    item.update(&ctx.db).await?;
    Ok(Redirect::to("../git_repos"))
}

#[debug_handler]
pub async fn edit(
    auth: middleware::auth::JWT,
    Path(id): Path<i32>,
    ViewEngine(v): ViewEngine<TeraView>,
    State(ctx): State<AppContext>,
) -> Result<Response> {
    let item = load_item(&ctx, id).await?;
    views::git_repo::edit(&v, &item)
}

#[debug_handler]
pub async fn show(
    auth: middleware::auth::JWT,
    Path(id): Path<i32>,
    ViewEngine(v): ViewEngine<TeraView>,
    State(ctx): State<AppContext>,
) -> Result<Response> {

    let item = load_item(&ctx, id).await?;
    views::git_repo::show(&v, &item)
}

#[debug_handler]
pub async fn add(
    auth: middleware::auth::JWT,
    State(ctx): State<AppContext>,
    Form(params): Form<Params>,
) -> Result<Redirect> {
    let mut item = ActiveModel {
        ..Default::default()
    };
    params.update(&mut item);
    item.insert(&ctx.db).await?;
    Ok(Redirect::to("git_repos"))
}

#[debug_handler]
pub async fn remove(Path(id): Path<i32>, State(ctx): State<AppContext>) -> Result<Response> {
    load_item(&ctx, id).await?.delete(&ctx.db).await?;
    format::empty()
}

pub fn routes() -> Routes {
    Routes::new()
        .prefix("git_repos/")
        .add("/", get(list))
        .add("/", post(add))
        .add("new", get(new))
        .add("{id}", get(show))
        .add("{id}/edit", get(edit))
        .add("{id}", delete(remove))
        .add("{id}", post(update))
}
