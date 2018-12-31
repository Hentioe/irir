use lazy_static::lazy_static;
use log::info;

use actix_web::server;
use env_logger::{Builder, Target};
use irirserver::{cli, routes::*};
use libresizer::ImageOption;

lazy_static! {
    static ref MATCHES: clap::ArgMatches<'static> = cli::build_cli().get_matches();
}

fn main() {
    let port = MATCHES.value_of("port").unwrap();
    let host = MATCHES.value_of("host").unwrap();
    let originals = MATCHES.value_of("origin_path").unwrap();
    let outputs = MATCHES.value_of("output_path").unwrap();
    let filter_type =
        libresizer::gen_filter_type(MATCHES.value_of("filter_type").unwrap()).unwrap();
    let state = AppState::new(ImageOption::new(originals, outputs, filter_type));

    std::env::set_var("RUST_LOG", "actix_web=info,irirserver=info");

    let mut log_bld = Builder::from_default_env();
    log_bld.target(Target::Stdout);

    log_bld.init();
    let apps = move || {
        vec![
            DisplayApp::new().action(state.clone()).finish(),
            IndexApp::new().action(state.clone()).finish(),
        ]
    };

    info!("Starting app……");
    let bind_s = format!("{}:{}", host, port);
    info!("http://{}", bind_s);
    server::new(apps).bind(bind_s).unwrap().run();
}
