use super::*;
use actix_web::{fs, App};

pub struct IndexApp {}

impl IndexApp {
    pub fn new() -> Self {
        IndexApp {}
    }
}

impl ActixApp for IndexApp {
    fn action(&self, state: AppState) -> App<AppState> {
        App::with_state(state).handler(
            "/",
            fs::StaticFiles::new("www")
                .unwrap()
                .index_file("index.html"),
        )
    }
}
