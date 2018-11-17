extern crate clap;

use clap::{App, Arg};

pub fn build_cli() -> App<'static, 'static> {
    App::new("irir-server")
        .version("0.0.0")
        .about("Advanced image hosting server")
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
                .long("interpolation")
                .short("i")
                .help("Interpolation algorithm")
                .required(false)
                .default_value("lanczos3")
                .takes_value(true),
        )
        .arg(
            Arg::with_name("origin_path")
                .long("dir-origin")
                .short("o")
                .help("Original directory path")
                .required(true)
                .takes_value(true),
        )
        .arg(
            Arg::with_name("output_path")
                .long("dir-output")
                .short("O")
                .help("Output directory path")
                .required(true)
                .takes_value(true),
        )
        .arg(
            Arg::with_name("enable-blur")
                .long("enable-blur")
                .help("Enable gaussian blur")
                .required(false)
                .takes_value(false),
        )
        .arg(
            Arg::with_name("max-width")
                .long("max-width")
                .help("Limit the maximum width of the image")
                .default_value("1280")
                .required(false)
                .takes_value(true),
        )
}
