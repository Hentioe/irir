use actix_web::App;
use libresizer::ImageOption;

mod display;
mod index;

pub use self::display::DisplayApp;
pub use self::index::IndexApp;

#[derive(Clone)]
pub struct AppState {
    pub options: ImageOption,
}

impl AppState {
    pub fn new(options: ImageOption) -> Self {
        AppState { options }
    }
}

pub trait ActixApp {
    fn action(&self, state: AppState) -> App<AppState>;
}
