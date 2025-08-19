use loco_rs::prelude::*;

use crate::models::_entities::sshes;

/// Render a list view of `sshes`.
///
/// # Errors
///
/// When there is an issue with rendering the view.
pub fn list(v: &impl ViewRenderer, items: &Vec<sshes::Model>) -> Result<Response> {
    format::render().view(v, "ssh/list.html", data!({"items": items}))
}

/// Render a single `ssh` view.
///
/// # Errors
///
/// When there is an issue with rendering the view.
pub fn show(v: &impl ViewRenderer, item: &sshes::Model) -> Result<Response> {
    format::render().view(v, "ssh/show.html", data!({"item": item}))
}

/// Render a `ssh` create form.
///
/// # Errors
///
/// When there is an issue with rendering the view.
pub fn create(v: &impl ViewRenderer) -> Result<Response> {
    format::render().view(v, "ssh/create.html", data!({}))
}

/// Render a `ssh` edit form.
///
/// # Errors
///
/// When there is an issue with rendering the view.
pub fn edit(v: &impl ViewRenderer, item: &sshes::Model) -> Result<Response> {
    format::render().view(v, "ssh/edit.html", data!({"item": item}))
}
