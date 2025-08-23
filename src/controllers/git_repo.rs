#![allow(clippy::missing_errors_doc)]
#![allow(clippy::unnecessary_struct_initialization)]
#![allow(clippy::unused_async)]
use std::path::PathBuf;
use chrono::Local;
use loco_rs::{controller::middleware, prelude::*};
use serde::{Deserialize, Serialize};
use axum::response::Redirect;
use axum_extra::extract::Form;
use sea_orm::{sea_query::Order, QueryOrder};
use axum::debug_handler;
use tracing::{error, info};

use crate::{
    models::_entities::git_repos::{ActiveModel, Column, Entity, Model}, services::git::GitService, views
};

const USER : &str = "git";

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
    let old_name = item.name.clone().unwrap().unwrap_or_default();
    let new_name = params.name.clone().unwrap_or_default();

    if old_name == new_name {
        info!("The repository name is unchanged; no action required.");
        return Ok(Redirect::to("../git_repos"));
    }
    let git_service = GitService::new(PathBuf::new().join(env!("REPO_BASE_PATH")),USER);
    if let Err(err) = git_service.rename_repository(&old_name, &new_name).await {
        error!("Failed to rename repository '{}' to '{}': {}", old_name, new_name, err);
        // Redirect with the error message in the URL
        return Ok(Redirect::to(&format!("../git_repos?error={}", urlencoding::encode(&format!("Failed to rename repository: {}", err)))));
    }
    params.update(&mut item);
    if let Err(err) = item.update(&ctx.db).await {
        error!("Failed to update repository in the database: {}", err);
        if let Err(rollback_err) = git_service.rename_repository(&new_name, &old_name).await {
            error!("Failed to rollback filesystem rename: {}", rollback_err);
        }
        return Ok(Redirect::to(&format!("../git_repos?error={}", urlencoding::encode(&format!("Failed to update repository in the database: {}", err)))));
    }
    info!("Successfully updated repository '{}' to '{}'", old_name, new_name);
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

    let service = GitService::new(PathBuf::new().join(env!("REPO_BASE_PATH")),USER);
    let path = service.create_bare_repository(&params.name.clone().unwrap()).await;
    let local_now = Local::now();
    let item = ActiveModel { 
        created_at: ActiveValue::set(local_now.with_timezone(local_now.offset())), 
        updated_at: ActiveValue::set(local_now.with_timezone(local_now.offset())), 
        id: ActiveValue::NotSet,
        name:  ActiveValue::set(params.name.clone()), 
        path:  ActiveValue::set(Some(path.unwrap().to_string_lossy().to_string()))
    };
    item.insert(&ctx.db).await?;
    Ok(Redirect::to("git_repos"))
}

#[debug_handler]
pub async fn remove(Path(id): Path<i32>, State(ctx): State<AppContext>) -> Result<Response> {
    let service = GitService::new(PathBuf::new().join(env!("REPO_BASE_PATH")),USER);
    let item = load_item(&ctx, id).await?;
    let _ = service.delete_repository(&item.name.unwrap()).await;
    let item = load_item(&ctx, id).await?;
    item.delete(&ctx.db).await?;
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
