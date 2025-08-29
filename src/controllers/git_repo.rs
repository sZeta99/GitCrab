#![allow(clippy::missing_errors_doc)]
#![allow(clippy::unnecessary_struct_initialization)]
#![allow(clippy::unused_async)]
use std::{fs, path::PathBuf};
use chrono::Local;
use loco_rs::{controller::middleware, prelude::*};
use serde::{Deserialize, Serialize};
use axum::response::Redirect;
use axum_extra::extract::Form;
use sea_orm::{sea_query::Order, QueryOrder};
use axum::debug_handler;
use tracing::{error, info, warn};

use crate::{
    models::_entities::git_repos::{ActiveModel, Column, Entity, Model}, services::{git_service::GitService, repo_retrive_service::{clone_bare_repo, count_files_in_structure, get_total_size_from_structure, read_repository_structure, RepoResponse}}, views
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
     info!("Fetching repository structure for repo: {}", item.name.clone().unwrap());
    
    // In a real application, you might fetch repo info from database
    // For now, we'll assume the repo_id corresponds to a directory path
    let bare_repo_path = PathBuf::new().join(env!("REPO_BASE_PATH")).join(format!("{}.git",&item.name.clone().unwrap()));
    
    info!("Repo Path : {}", bare_repo_path.to_str().unwrap());
    if !bare_repo_path.exists() {
        return Err(Error::NotFound);
    }
    let worktree_path = PathBuf::from(format!("./worktrees/{}", item.name.clone().unwrap()));

    // Clone the bare repository into a working directory
    if !worktree_path.exists() {

        fs::create_dir_all(&worktree_path.parent().unwrap())?;

        clone_bare_repo(&bare_repo_path, &worktree_path)?;

    }

    match read_repository_structure(&worktree_path, &worktree_path) {
        Ok(structure) => {
            let total_files = count_files_in_structure(&structure);
            let total_size = get_total_size_from_structure(&structure);
            
            let response = RepoResponse {
                id: item.id.to_string(),
                name: item.name.clone().unwrap(),
                structure,
                total_files,
                total_size,
            };
            
            info!("Successfully fetched repository structure");
            views::git_repo::show(&v, &item, response)
        }
        Err(e) => {
            error!("Failed to read repository structure: {}", e);
            Err(Error::InternalServerError)
        }
    }

    
}

#[debug_handler]

pub async fn add(
    auth: middleware::auth::JWT,
    State(ctx): State<AppContext>,
    Form(params): Form<Params>,
) -> Result<Redirect> {

    let service = GitService::new(PathBuf::new().join(env!("REPO_BASE_PATH")), USER);
    let repo_name = params.name.clone().unwrap_or_default();

    // Handle the Result from create_bare_repository
    let path = match service.create_bare_repository(&repo_name).await {
        Ok(path) => path,
        Err(err) => {
            error!("Failed to create repository '{}': {}", repo_name, err);
            // Redirect with the error message in the URL
            return Ok(Redirect::to(&format!("git_repos?error={}", 
                urlencoding::encode(&format!("Failed to create repository: {}", err)))));
        }
    };

    let local_now = Local::now();
    let item = ActiveModel { 
        created_at: ActiveValue::set(local_now.with_timezone(local_now.offset())), 
        updated_at: ActiveValue::set(local_now.with_timezone(local_now.offset())), 
        id: ActiveValue::NotSet,
        name: ActiveValue::set(params.name.clone()), 
        path: ActiveValue::set(Some(path.to_string_lossy().to_string()))
    };

    // Handle database insertion error as well
    if let Err(err) = item.insert(&ctx.db).await {
        error!("Failed to insert repository '{}' into database: {}", repo_name, err);

        // You might want to clean up the created repository here if needed
        let _ = service.delete_repository(&repo_name).await;
        // service.delete_repository(&repo_name).await;
        return Ok(Redirect::to(&format!("git_repos?error={}", 
            urlencoding::encode(&format!("Failed to save repository to database: {}", err)))));
    }
    info!("Successfully created repository '{}'", repo_name);
    Ok(Redirect::to("git_repos"))
}

#[debug_handler]

pub async fn remove(Path(id): Path<i32>, State(ctx): State<AppContext>) -> Result<Response> {

    let service = GitService::new(PathBuf::new().join(env!("REPO_BASE_PATH")), USER);
    let item = load_item(&ctx, id).await?;
    let repo_name = item.name.clone().unwrap_or_default();

    // Handle the Result from delete_repository
    if let Err(err) = service.delete_repository(&repo_name).await {
        error!("Failed to delete repository '{}': {}", repo_name, err);
        // Return redirect with error message
        return Ok(Redirect::to(&format!("git_repos?error={}", 
            urlencoding::encode(&format!("Failed to delete repository: {}", err))))
            .into_response());
    }

    // Handle database deletion error
    if let Err(err) = item.delete(&ctx.db).await {
        error!("Failed to delete repository '{}' from database: {}", repo_name, err);
        warn!("Repository '{}' was deleted from filesystem but not from database", repo_name);
        return Ok(Redirect::to(&format!("git_repos?error={}", 
            urlencoding::encode(&format!("Failed to remove repository from database: {}", err))))
            .into_response());

    }
    info!("Successfully deleted repository '{}'", repo_name);
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
