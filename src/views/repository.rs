use loco_rs::prelude::*;

use crate::models::_entities::repositories;

/// Render a list view of `repositories`.
///
/// # Errors
///
/// When there is an issue with rendering the view.
pub fn list(v: &impl ViewRenderer, items: &Vec<repositories::Model>) -> Result<Response> {
    format::render().view(v, "repository/list.html", data!({"items": items}))
}

/// Render a single `repository` view.
///
/// # Errors
///
/// When there is an issue with rendering the view.
pub fn show(v: &impl ViewRenderer, item: &repositories::Model) -> Result<Response> {
    format::render().view(v, "repository/show.html", data!({"item": item}))
}

/// Render a `repository` create form.
///
/// # Errors
///
/// When there is an issue with rendering the view.
pub fn create(v: &impl ViewRenderer) -> Result<Response> {
    format::render().view(v, "repository/create.html", data!({}))
}

/// Render a `repository` edit form.
///
/// # Errors
///
/// When there is an issue with rendering the view.
pub fn edit(v: &impl ViewRenderer, item: &repositories::Model) -> Result<Response> {
    format::render().view(v, "repository/edit.html", data!({"item": item}))
}
