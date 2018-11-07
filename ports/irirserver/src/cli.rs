extern crate clap;

use clap::{App, Arg};

pub fn build_cli() -> App<'static, 'static> {
    App::new("irir-server")
        .version("0.0.0")
        .about("Image Resizer In Rust")
        .author("Hentioe Cl (绅士喵)")
        .arg(
            Arg::with_name("port")
                .long("port")
                .short("p")
                .help("Listening port")
                .required(false)
                .default_value("8080")
                .takes_value(true),
        )
        .arg(
            Arg::with_name("filter_type")
                .long("filter-type")
                .help("Sampling filter")
                .required(false)
                .default_value("lanczos3")
                .takes_value(true),
        )
        .arg(
            Arg::with_name("origin_path")
                .long("dir-origin")
                .help("Original image directory path")
                .required(true)
                .takes_value(true),
        )
        .arg(
            Arg::with_name("output_path")
                .long("dir-output")
                .help("Output image directory path")
                .required(true)
                .takes_value(true),
        )
}
