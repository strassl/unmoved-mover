extern crate log;

use clap::{crate_version, App, Arg, ArgMatches};
use std::time::Duration;
use unmoved_mover::daemon;

fn get_arg_key(matches: &ArgMatches, name: &str) -> String {
    let key: String = matches
        .value_of(name)
        .expect(&*format!("Invalid {name}", name = name))
        .to_string();

    return key;
}

fn main() {
    env_logger::init();

    let matches = App::new("unmoved-mover")
        .version(crate_version!())
        .about("TODO")
        .arg(
            Arg::with_name("mod-key")
                .long("mod-key")
                .default_value("Mod1") // code: 64
                .takes_value(true)
                .required(true),
        )
        .arg(
            Arg::with_name("up-key")
                .long("up-key")
                .default_value("i") // code: 31
                .takes_value(true)
                .required(true),
        )
        .arg(
            Arg::with_name("down-key")
                .long("down-key")
                .default_value("k") // code: 44
                .takes_value(true)
                .required(true),
        )
        .arg(
            Arg::with_name("left-key")
                .long("left-key")
                .default_value("j") // code: 45
                .takes_value(true)
                .required(true),
        )
        .arg(
            Arg::with_name("right-key")
                .long("right-key")
                .default_value("l") // code: 46
                .takes_value(true)
                .required(true),
        )
        .arg(
            Arg::with_name("left-click-key")
                .long("left-click-key")
                .default_value("semicolon") // code 47
                .takes_value(true)
                .required(true),
        )
        .arg(
            Arg::with_name("right-click-key")
                .long("right-click-key")
                .default_value("apostrophe") // code: 48
                .takes_value(true)
                .required(true),
        )
        .arg(
            Arg::with_name("tick-interval-ms")
                .long("tick-interval-ms")
                .default_value("10")
                .takes_value(true)
                .required(true),
        )
        .arg(
            Arg::with_name("velocity-px-per-s")
                .long("velocity-px-per-s")
                .default_value("500")
                .takes_value(true)
                .required(true),
        )
        .get_matches();

    let mod_key = get_arg_key(&matches, "mod-key");
    let left_key = get_arg_key(&matches, "left-key");
    let right_key = get_arg_key(&matches, "right-key");
    let up_key = get_arg_key(&matches, "up-key");
    let down_key = get_arg_key(&matches, "down-key");
    let left_click_key = get_arg_key(&matches, "left-click-key");
    let right_click_key = get_arg_key(&matches, "right-click-key");

    let tick_interval = matches
      .value_of("tick-interval-ms")
      .and_then(|x| x.parse::<u64>().ok())
      .map(|x| Duration::from_millis(x))
      .expect("Invalid tick-interval-ms");

    let velocity_px_per_s = matches
      .value_of("velocity-px-per-s")
      .and_then(|x| x.parse::<u32>().ok())
      .expect("Invalid velocity-px-per-s");

    let config = daemon::Config {
        mod_key,
        left_key,
        right_key,
        up_key,
        down_key,

        left_click_key,
        right_click_key,

        tick_interval: tick_interval,
        velocity_px_per_s: velocity_px_per_s,
    };
    daemon::run(&config).expect("Unable to start daemon");
}
