use loco_rs::prelude::*;


pub fn home(v: &impl ViewRenderer) -> Result<Response> {
    format::render().view(v, "home/hello.html", data!({}))
}


