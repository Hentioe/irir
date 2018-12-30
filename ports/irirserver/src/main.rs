#[macro_use]
extern crate lazy_static;
extern crate actix_web;
extern crate env_logger;
extern crate irirserver;
extern crate libresizer;

use actix_web::server;
use irirserver::{cli, routes::*};
use libresizer::ImageOption;

lazy_static! {
    static ref MATCHES: clap::ArgMatches<'static> = cli::build_cli().get_matches();
}

fn main() {
    let port = MATCHES.value_of("port").unwrap();
    let originals = MATCHES.value_of("origin_path").unwrap();
    let outputs = MATCHES.value_of("output_path").unwrap();
    let filter_type =
        libresizer::gen_filter_type(MATCHES.value_of("filter_type").unwrap()).unwrap();
    let state = AppState::new(ImageOption::new(originals, outputs, filter_type));
    
    std::env::set_var("RUST_LOG", "actix_web=info");
    env_logger::init();
    let apps = move || {
        vec![
            DisplayApp::new().action(state.clone()).finish(),
            IndexApp::new().action(state.clone()).finish(),
        ]
    };
    server::new(apps)
        .bind(format!("0.0.0.0:{}", port))
        .unwrap()
        .run();
}
