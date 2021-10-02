pub struct Config {
  pub left_keycode: u32,
  pub right_keycode: u32,
  pub up_keycode: u32,
  pub down_keycode: u32,

  pub left_click_keycode: u32,
  pub right_click_keycode: u32,
}

pub fn run(config: &Config) -> Result<(), Box<dyn std::error::Error>> {
  Ok(())
}