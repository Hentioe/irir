extern crate clap;

use clap::{App, Arg, SubCommand};

pub fn gen_clap_app<'a, 'b>() -> App<'a, 'b> {
    App::new("irir-server")
        .version("0.0.0")
        .about("Image Resizer In Rust")
        .author("Hentioe Cl (绅士喵)")
        .arg(
            Arg::with_name("dir-original")
                .long("dir-original")
                .help("Original image directory path")
                .required(true)
                .takes_value(true),
        )
        .arg(
            Arg::with_name("dir-output")
                .long("dir-output")
                .help("Output image directory path")
                .required(true)
                .takes_value(true),
        )
}
