use loco_rs::prelude::*;

use crate::models::_entities::git_repos;

/// Render a list view of `git_repos`.
///
/// # Errors
///
/// When there is an issue with rendering the view.
pub fn list(v: &impl ViewRenderer, items: &Vec<git_repos::Model>) -> Result<Response> {
    format::render().view(v, "git_repo/list.html", data!({"items": items}))
}

/// Render a single `git_repo` view.
///
/// # Errors
///
/// When there is an issue with rendering the view.
pub fn show(v: &impl ViewRenderer, item: &git_repos::Model) -> Result<Response> {
    format::render().view(v, "git_repo/show.html", data!({"item": item}))
}

/// Render a `git_repo` create form.
///
/// # Errors
///
/// When there is an issue with rendering the view.
pub fn create(v: &impl ViewRenderer) -> Result<Response> {
    format::render().view(v, "git_repo/create.html", data!({}))
}

/// Render a `git_repo` edit form.
///
/// # Errors
///
/// When there is an issue with rendering the view.
pub fn edit(v: &impl ViewRenderer, item: &git_repos::Model) -> Result<Response> {
    format::render().view(v, "git_repo/edit.html", data!({"item": item}))
}
