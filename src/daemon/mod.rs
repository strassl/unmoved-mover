use log;
use std::boxed::Box;
use std::collections::{HashMap, HashSet};
use std::error::Error;
use std::sync::{Arc, Condvar, Mutex};
use std::thread;
use std::time::Duration;
use swayipc::{BindingEvent, Connection, Event, EventType};

const SWAY_COMMAND_PRESS: &str = "nop press";
const SWAY_COMMAND_RELEASE: &str = "nop release";

type Keyname = String;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Config {
    pub required_mode: Option<String>,
    pub key_combo_enter_mode: Keyname,
    pub key_combo_exit_mode: Keyname,
    pub mod_key: Keyname,
    pub left_key: Keyname,
    pub right_key: Keyname,
    pub up_key: Keyname,
    pub down_key: Keyname,
    pub left_click_key: Keyname,
    pub right_click_key: Keyname,
    pub tick_interval: Duration,
    pub cursor_velocity: u32,
    pub skip_configuration: bool,
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
enum Key {
    Mod,
    Up,
    Down,
    Left,
    Right,
    LeftClick,
    RightClick,
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
enum KeyState {
    Up,
    Down,
}

struct SharedState {
    state: Mutex<State>,
    state_change_cvar: Condvar,
}

#[derive(Default, Debug, Clone, PartialEq, Eq)]
struct State {
    mode: String,
    down_keys: HashSet<Key>,
}

impl State {
    fn get_key_state(&self, key: &Key) -> KeyState {
        return if self.down_keys.contains(&key) {
            KeyState::Down
        } else {
            KeyState::Up
        };
    }
}

macro_rules! map_of {
  ($($k:expr => $v:expr),* $(,)?) => {{
      use std::iter::{Iterator, IntoIterator};
      Iterator::collect(IntoIterator::into_iter([$(($k, $v),)*]))
  }};
}

macro_rules! collection_of {
  ($($v:expr),* $(,)?) => {{
      use std::iter::{Iterator, IntoIterator};
      Iterator::collect(IntoIterator::into_iter([$($v,)*]))
  }};
}

fn run_sway_command<T: AsRef<str> + std::fmt::Display>(
    conn: &mut Connection,
    command: T,
) -> Result<(), Box<dyn Error>> {
    log::trace!("Running command: {}", command);
    conn.run_command(&command)?;
    Ok(())
}

fn setup_sway_config(config: &Config) -> Result<(), Box<dyn Error>> {
    if config.skip_configuration {
        // if the user doesn't want to have unmoved-mover register its own keybidings
        // then so be it
        return Ok(());
    }

    let mut conn = Connection::new()?;

    let codes: HashSet<&str> = collection_of! {
      &*config.up_key,
      &*config.down_key,
      &*config.left_key,
      &*config.right_key,
      &*config.left_click_key,
      &*config.right_click_key,
    };

    let mode_prefix = match &config.required_mode {
        None => "".to_string(),
        Some(required_mode) => {
            log::debug!(
                "Setting up key combos to enter/exit mode: \"{}\" and \"{}\"",
                config.key_combo_enter_mode,
                config.key_combo_exit_mode
            );
            let unbind_enter_mode_command = format!("unbindsym {}", config.key_combo_enter_mode);
            let enter_mode_command = format!(
                "bindsym {} mode \"{}\"",
                config.key_combo_enter_mode, required_mode
            );
            let unbind_exit_mode_command = format!(
                "mode \"{}\" unbindsym {}",
                required_mode, config.key_combo_exit_mode
            );
            let exit_mode_command = format!(
                "mode \"{}\" bindsym {} mode default",
                required_mode, config.key_combo_exit_mode
            );
            run_sway_command(&mut conn, unbind_enter_mode_command)?;
            run_sway_command(&mut conn, enter_mode_command)?;
            run_sway_command(&mut conn, unbind_exit_mode_command)?;
            run_sway_command(&mut conn, exit_mode_command)?;

            format!("mode \"{}\" ", required_mode)
        }
    };

    for &key in &codes {
        let key_combo = if config.mod_key.is_empty() {
            format!("{}", key)
        } else {
            format!("{}+{}", config.mod_key, key)
        };

        // TODO remember the old bindings and restore them after the program exits
        log::debug!("Setting up: {}", key_combo);
        let unbind_press = format!("{}unbindsym {}", mode_prefix, key_combo);
        let unbind_release = format!("{}unbindsym --release {}", mode_prefix, key_combo);
        let bind_press = format!(
            "{}bindsym --no-repeat {} {}",
            mode_prefix, key_combo, SWAY_COMMAND_PRESS
        );
        let bind_release = format!(
            "{}bindsym --release {} {}",
            mode_prefix, key_combo, SWAY_COMMAND_RELEASE
        );
        run_sway_command(&mut conn, unbind_press)?;
        run_sway_command(&mut conn, unbind_release)?;
        run_sway_command(&mut conn, bind_press)?;
        run_sway_command(&mut conn, bind_release)?;
    }

    Ok(())
}

fn parse_binding_key(config: &Config, symbol: &str) -> Option<Key> {
    let symbol_to_key: HashMap<&str, Key> = map_of! {
      &*config.up_key => Key::Up,
      &*config.down_key => Key::Down,
      &*config.left_key => Key::Left,
      &*config.right_key => Key::Right,
      &*config.left_click_key => Key::LeftClick,
      &*config.right_click_key => Key::RightClick,
    };

    return symbol_to_key.get(symbol).map(|x| x.clone());
}

fn get_opposing_key(key: &Key) -> Option<Key> {
    return match key {
        Key::Up => Some(Key::Down),
        Key::Down => Some(Key::Up),
        Key::Right => Some(Key::Left),
        Key::Left => Some(Key::Right),
        _ => None,
    };
}

fn handle_mouse_key(
    conn: &mut Connection,
    key: &Key,
    key_down: bool,
) -> Result<(), Box<dyn Error>> {
    let button = match key {
        Key::LeftClick => Some("button1"),
        Key::RightClick => Some("button3"),
        _ => None,
    };

    let action = if key_down { "press" } else { "release" };

    match button {
        Some(button) => {
            let cmd = format!("seat - cursor {} {}", action, button);
            run_sway_command(conn, &cmd)?;
        }
        None => {}
    }

    Ok(())
}

fn handle_bound_key(
    conn: &mut Connection,
    state: &mut State,
    key: &Key,
    key_down: bool,
) -> Result<(), Box<dyn Error>> {
    if key_down {
        match get_opposing_key(&key) {
            Some(key) => {
                log::trace!("Removing opposing down key");
                state.down_keys.remove(&key);
            }
            _ => {}
        }
        log::trace!("Adding down key");
        state.down_keys.insert(key.clone());
    } else {
        // Sway does not send release events when switching between bindings (only down)
        // We clear the entire state here to prevent stuck movement events
        // state.down_keys.remove(&key);
        log::trace!("Clearing down keys");
        state.down_keys.clear();
    }

    handle_mouse_key(conn, &key, key_down)?;

    Ok(())
}

fn handle_binding_event(
    conn: &mut Connection,
    state: &mut State,
    config: &Config,
    event: &BindingEvent,
) -> Result<(), Box<dyn Error>> {
    let binding = &event.binding;

    // if a modifier is configured, check its state
    if !config.mod_key.is_empty() {
        let modifiers: HashSet<&String> = binding.event_state_mask.iter().collect();
        let mod_down = modifiers == collection_of! { &config.mod_key };
        handle_bound_key(conn, state, &Key::Mod, mod_down)?;
    }

    let bound_key = binding
        .symbol
        .as_ref()
        .and_then(|x| parse_binding_key(&config, &*x));

    match bound_key {
        Some(key) => {
            let our_commands: HashSet<&str> =
                collection_of! { SWAY_COMMAND_PRESS, SWAY_COMMAND_RELEASE };
            let our_action = our_commands.contains(&*binding.command);

            if our_action {
                let key_down = binding.command.ends_with("press");
                handle_bound_key(conn, state, &key, key_down)?;
            } else {
                log::warn!(
                    "Event was not bound correctly - bound to \"{}\"",
                    binding.command
                );
            }
        }
        None => {
            log::trace!("Ignoring unbound key event");
        }
    }

    Ok(())
}

fn run_event_receiver(
    daemon_state: &Arc<SharedState>,
    daemon_config: &Config,
) -> Result<(), Box<dyn Error>> {
    let config = daemon_config.clone();
    let mut conn = Connection::new()?;
    let event_types = [EventType::Binding];
    let event_iter = Connection::new()?.subscribe(&event_types)?;

    let thread_state = Arc::clone(daemon_state);
    thread::spawn(move || {
        for evt_result in event_iter {
            let event = evt_result.expect("Failed to get event");
            log::trace!("Received event: {:?}", event);

            match event {
                Event::Binding(event) => {
                    let mut state = thread_state.state.lock().expect("Failed to get state");
                    handle_binding_event(&mut conn, &mut state, &config, &event)
                        .expect("Failed to handle event");
                    drop(state);
                    thread_state.state_change_cvar.notify_all();
                }
                Event::Mode(event) => {
                    let mut state = thread_state.state.lock().expect("Failed to get state");
                    state.mode = event.change;
                    drop(state);
                    thread_state.state_change_cvar.notify_all();
                }
                _ => {}
            }
        }
    });

    Ok(())
}

enum TickAction {
    MoveCursor { dx_px: i32, dy_px: i32 },
}

fn get_action(
    config: &Config,
    current_state: &State,
    previous_state: &Option<State>,
    elapsed_time: &Duration,
) -> Option<TickAction> {
    log::trace!(
        "Tick state: {:?}, elapsed_time: {:?}",
        current_state,
        elapsed_time
    );

    let cursor_velocity = config.cursor_velocity;
    let elapsed_s = match previous_state {
        Some(prev) => {
            if prev == current_state {
                elapsed_time.as_secs_f32()
            } else {
                0.0
            }
        }
        None => 0.0,
    };

    let mod_state = current_state.get_key_state(&Key::Mod);
    let up_state = current_state.get_key_state(&Key::Up);
    let down_state = current_state.get_key_state(&Key::Down);
    let left_state = current_state.get_key_state(&Key::Left);
    let right_state = current_state.get_key_state(&Key::Right);

    let mod_not_pressed = !config.mod_key.is_empty() && mod_state != KeyState::Down;
    if mod_not_pressed {
        // Nothing to do, configured mod is not pressed
        log::trace!("Skipping tick: Configured modifier is not pressed");
        return None;
    }

    let not_in_mode = match &config.required_mode {
        Some(required_mode) => *required_mode == current_state.mode,
        None => true,
    };

    if not_in_mode {
        // Nothing to do, wrong mode is active
        log::trace!("Skipping tick: Not in configured mode");
        return None;
    }

    let delta_px = elapsed_s * cursor_velocity as f32;
    let mut move_vec_x: f32 = 0.0;
    let mut move_vec_y: f32 = 0.0;
    if up_state == KeyState::Down {
        move_vec_y -= 1.0;
    }
    if down_state == KeyState::Down {
        move_vec_y += 1.0;
    }
    if right_state == KeyState::Down {
        move_vec_x += 1.0;
    }
    if left_state == KeyState::Down {
        move_vec_x -= 1.0;
    }

    let move_vec_magnitude = (move_vec_x.powf(2.0) + move_vec_y.powf(2.0)).sqrt();
    let move_dx = move_vec_x / move_vec_magnitude;
    let move_dy = move_vec_y / move_vec_magnitude;
    let dx_px = (move_dx * delta_px).round() as i32;
    let dy_px = (move_dy * delta_px).round() as i32;

    return Some(TickAction::MoveCursor { dx_px, dy_px });
}

fn run_loop(config: &Config, daemon_state: &Arc<SharedState>) -> Result<(), Box<dyn Error>> {
    let tick_interval = config.tick_interval;

    let mut conn = Connection::new()?;
    let mut previous_state: Option<State> = None;
    let mut last_iteration_time: Option<std::time::Instant> = None;

    loop {
        let current_state = daemon_state
            .state_change_cvar
            .wait_while(
                daemon_state
                    .state
                    .lock()
                    .expect("Failed to unlock state mutex"),
                |_state| false,
            )
            .expect("Failed to unlock state mutex")
            .clone();

        let loop_start_time = std::time::Instant::now();

        let elapsed_time = last_iteration_time.map_or(Duration::ZERO, |t| t.elapsed());
        let tick_action = get_action(&config, &current_state, &previous_state, &elapsed_time);
        match tick_action {
            Some(TickAction::MoveCursor { dx_px, dy_px }) => {
                log::trace!(
                    "Moving mouse by x: {dx}px y: {dy}px",
                    dx = dx_px,
                    dy = dy_px
                );

                let move_cmd = format!("seat - cursor move {dx} {dy}", dx = dx_px, dy = dy_px);
                run_sway_command(&mut conn, move_cmd)?;
            }
            None => {}
        };
        previous_state = Some(current_state.clone());
        last_iteration_time = Some(loop_start_time);

        // Sleep
        let loop_end_time = std::time::Instant::now();
        let loop_elapsed = loop_end_time - loop_start_time;
        let sleep_for = tick_interval.saturating_sub(loop_elapsed);

        thread::sleep(sleep_for);
    }
}

pub fn run(config: &Config) -> Result<(), Box<dyn Error>> {
    // Setup sway config
    log::info!("Setting up sway config");
    setup_sway_config(&config)?;

    let state = Arc::new(SharedState {
        state: Mutex::new(State::default()),
        state_change_cvar: Condvar::new(),
    });

    // Spawn event receiver thread
    log::info!("Spawning event receiver");
    run_event_receiver(&state, &config)?;

    // Run main loop
    log::info!("Starting main loop");
    run_loop(&config, &state)?;

    Ok(())
}
