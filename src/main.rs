#[macro_use]
extern crate log;

use clap::{crate_version, App, Arg, ArgMatches};

fn get_arg_keycode(matches: &ArgMatches, name: &str) -> u32 {
  let keycode: u32 = matches
    .value_of(name)
    .and_then(|x| x.parse::<u32>().ok())
    .expect(&*format!("Invalid {name}", name=name));

  return keycode;
}

fn main() {
  env_logger::init();

  let matches = App::new("unmoved-mover")
    .version(crate_version!())
    .about("TODO")
    .arg(
      Arg::with_name("up-keycode")
        .long("up-keycode")
        .default_value("31")
        .takes_value(true)
        .required(true),
    )
    .arg(
      Arg::with_name("down-keycode")
        .long("down-keycode")
        .default_value("44")
        .takes_value(true)
        .required(true),
    )
    .arg(
      Arg::with_name("left-keycode")
        .long("left-keycode")
        .default_value("45")
        .takes_value(true)
        .required(true),
    )
    .arg(
      Arg::with_name("right-keycode")
        .long("right-keycode")
        .default_value("46")
        .takes_value(true)
        .required(true),
    )
    .arg(
      Arg::with_name("left-click-keycode")
        .long("left-click-keycode")
        .default_value("47")
        .takes_value(true)
        .required(true),
    )
    .arg(
      Arg::with_name("right-click-keycode")
        .long("right-click-keycode")
        .default_value("48")
        .takes_value(true)
        .required(true),
    )
    .get_matches();

  let left_keycode = get_arg_keycode(&matches, "left-keycode");
  let right_keycode = get_arg_keycode(&matches, "right-keycode");
  let up_keycode = get_arg_keycode(&matches, "up-keycode");
  let down_keycode = get_arg_keycode(&matches, "down-keycode");
  let left_click_keycode = get_arg_keycode(&matches, "left-click-keycode");
  let right_click_keycode = get_arg_keycode(&matches, "right-click-keycode");
}
