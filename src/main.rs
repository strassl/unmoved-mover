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
        .about("Daemon for emulating the mouse via keyboard inputs in sway through swayipc.")
        .arg(
            Arg::with_name("required-mode")
                .short("M")
                .long("required-state")
                .help("Mode in which to listen for keybindings")
                .default_value("Cursor")
                .takes_value(true)
                .required(true),
        )
        .arg(
            Arg::with_name("key-combo-enter-mode")
                .short("e")
                .long("key-combo-enter-mod")
                .help("Key combination to enter the required mode")
                .default_value("Shift+Mod1+u")
                .takes_value(true)
                .required(true),
        )
        .arg(
            Arg::with_name("key-combo-exit-mode")
                .short("E")
                .long("key-combo-exit-mode")
                .help("Key combination to leave the required mode again")
                .default_value("Escape")
                .takes_value(true)
                .required(true),
        )
        .arg(
            Arg::with_name("key-mod")
                .short("m")
                .long("key-mod")
                .help("Modifier that needs to be pressed for movement (set empty for no modifier)")
                .default_value("Mod1")
                .takes_value(true)
                .required(true),
        )
        .arg(
            Arg::with_name("key-up")
                .short("u")
                .long("key-up")
                .help("Key for moving the cursor up")
                .default_value("i")
                .takes_value(true)
                .required(true),
        )
        .arg(
            Arg::with_name("key-down")
                .short("d")
                .long("key-down")
                .help("Key for moving the cursor down")
                .default_value("k")
                .takes_value(true)
                .required(true),
        )
        .arg(
            Arg::with_name("key-left")
                .short("l")
                .long("key-left")
                .help("Key for moving the cursor left")
                .default_value("j")
                .takes_value(true)
                .required(true),
        )
        .arg(
            Arg::with_name("key-right")
                .short("r")
                .long("key-right")
                .help("Key for moving the cursor right")
                .default_value("l")
                .takes_value(true)
                .required(true),
        )
        .arg(
            Arg::with_name("key-left-click")
                .short("L")
                .long("key-left-click")
                .help("Key for performing a left-click")
                .default_value("u")
                .takes_value(true)
                .required(true),
        )
        .arg(
            Arg::with_name("key-right-click")
                .short("R")
                .long("key-right-click")
                .help("Key for performing a right-click")
                .default_value("o")
                .takes_value(true)
                .required(true),
        )
        .arg(
            Arg::with_name("tick-interval")
                .short("i")
                .long("tick-interval")
                .help("Interval between sending sway IPC messages")
                .default_value("10")
                .takes_value(true)
                .required(true),
        )
        .arg(
            Arg::with_name("cursor-velocity")
                .short("v")
                .long("cursor-velocity")
                .help("Cursor velocity (in pixel/s)")
                .default_value("500")
                .takes_value(true)
                .required(true),
        )
        .arg(
            Arg::with_name("no-configuration-registration")
                .short("C")
                .long("no-configuration-registration")
                .help("Do not register any keybindings via swayipc")
                .takes_value(false)
                .required(false),
        )
        .get_matches();

    // TODO add key bindings for scroll wheel up/down
    // TODO add fast/slow movement modifier keys
    let mod_key = get_arg_key(&matches, "key-mod");
    let left_key = get_arg_key(&matches, "key-left");
    let right_key = get_arg_key(&matches, "key-right");
    let up_key = get_arg_key(&matches, "key-up");
    let down_key = get_arg_key(&matches, "key-down");
    let left_click_key = get_arg_key(&matches, "key-left-click");
    let right_click_key = get_arg_key(&matches, "key-right-click");
    let required_mode = get_arg_key(&matches, "required-mode");
    let key_combo_enter_mode = get_arg_key(&matches, "key-combo-enter-mode");
    let key_combo_exit_mode = get_arg_key(&matches, "key-combo-exit-mode");

    let tick_interval = matches
        .value_of("tick-interval")
        .and_then(|x| x.parse::<u64>().ok())
        .map(|x| Duration::from_millis(x))
        .expect("Invalid tick-interval");

    let cursor_velocity = matches
        .value_of("cursor-velocity")
        .and_then(|x| x.parse::<u32>().ok())
        .expect("Invalid velocity");

    let skip_configuration = matches.is_present("no-configuration-registration");

    let config = daemon::Config {
        required_mode: if required_mode == "" { None } else { Some(required_mode)},
        key_combo_enter_mode,
        key_combo_exit_mode,
        mod_key,
        left_key,
        right_key,
        up_key,
        down_key,
        left_click_key,
        right_click_key,
        tick_interval,
        cursor_velocity,
        skip_configuration,
    };
    daemon::run(&config).expect("Unable to start daemon");
}
