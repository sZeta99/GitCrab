use loco_rs::prelude::*;



pub fn home(v: &impl ViewRenderer) -> Result<Response> {
    format::render().view(v, "home/home.html", data!({}))
}
pub fn dashboard(v: &impl ViewRenderer) -> Result<Response> {
    format::render().view(v, "home/dashboard.html", data!({}))
}


