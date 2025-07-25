use loco_rs::prelude::*;

use crate::models::_entities::ssh_keys;

/// Render a list view of `ssh_keys`.
///
/// # Errors
///
/// When there is an issue with rendering the view.
pub fn list(v: &impl ViewRenderer, items: &Vec<ssh_keys::Model>) -> Result<Response> {
    format::render().view(v, "ssh_keys/list.html", data!({"items": items}))
}

/// Render a single `ssh_keys` view.
///
/// # Errors
///
/// When there is an issue with rendering the view.
pub fn show(v: &impl ViewRenderer, item: &ssh_keys::Model) -> Result<Response> {
    format::render().view(v, "ssh_keys/show.html", data!({"item": item}))
}

/// Render a `ssh_keys` create form.
///
/// # Errors
///
/// When there is an issue with rendering the view.
pub fn create(v: &impl ViewRenderer) -> Result<Response> {
    format::render().view(v, "ssh_keys/create.html", data!({}))
}

/// Render a `ssh_keys` edit form.
///
/// # Errors
///
/// When there is an issue with rendering the view.
pub fn edit(v: &impl ViewRenderer, item: &ssh_keys::Model) -> Result<Response> {
    format::render().view(v, "ssh_keys/edit.html", data!({"item": item}))
}
