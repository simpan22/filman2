mod commands;
mod draw;
mod error;
mod path;
pub mod state;

use commands::{execute_command, execute_shell_command};
use crossterm::{
    event::{read, DisableMouseCapture, Event, KeyCode, KeyEvent},
    execute,
    terminal::{disable_raw_mode, LeaveAlternateScreen},
};
use draw::{create_terminal, draw, RenderState};
use path::Path;
use prompter::PromptReader;

use std::{
    collections::{HashMap, HashSet},
    io,
};

use state::{Mode, State};

enum Action {
    ShellCommand(String),
    Command(String),
    ModeSwitch(Mode),
    Quit,
}

fn shell_mode_input(key: &KeyEvent, reader: &mut PromptReader) -> Vec<Action> {
    match key.code {
        KeyCode::Char(c) => reader.next_key(c.into()),
        KeyCode::Enter => reader.next_key(prompter::keycodes::KeyCode::Enter),
        KeyCode::Left => reader.next_key(prompter::keycodes::KeyCode::Left),
        KeyCode::Right => reader.next_key(prompter::keycodes::KeyCode::Right),
        KeyCode::Backspace => reader.next_key(prompter::keycodes::KeyCode::Backspace),
        KeyCode::Delete => reader.next_key(prompter::keycodes::KeyCode::Delete),
        _ => {}
    }

    if reader.done() {
        vec![
            Action::ShellCommand(reader.result().to_string()),
            Action::ModeSwitch(Mode::NormalMode),
        ]
    } else {
        vec![]
    }
}

fn command_mode_input(key: &KeyEvent, reader: &mut PromptReader) -> Vec<Action> {
    match key.code {
        KeyCode::Char(c) => reader.next_key(c.into()),
        KeyCode::Enter => reader.next_key(prompter::keycodes::KeyCode::Enter),
        KeyCode::Left => reader.next_key(prompter::keycodes::KeyCode::Left),
        KeyCode::Right => reader.next_key(prompter::keycodes::KeyCode::Right),
        KeyCode::Backspace => reader.next_key(prompter::keycodes::KeyCode::Backspace),
        KeyCode::Delete => reader.next_key(prompter::keycodes::KeyCode::Delete),
        _ => {}
    }

    if reader.done() {
        vec![
            Action::Command(reader.result().to_string()),
            Action::ModeSwitch(Mode::NormalMode),
        ]
    } else {
        vec![]
    }
}
fn normal_mode_input(key: &KeyEvent, state: &mut State) -> Vec<Action> {
    match key.code {
        KeyCode::Char('q') => return vec![Action::Quit],
        KeyCode::Char('j') => state.try_next().unwrap_or_else(|_| {
            state.error_message = Some("Failed to navigate".into());
        }),
        KeyCode::Char('k') => state.try_prev().unwrap_or_else(|_| {
            state.error_message = Some("Faled to navigate".into());
        }),
        KeyCode::Char('h') => state.try_up().unwrap_or_else(|_| {
            state.error_message = Some("Failed to go to parent dir".into());
        }),
        KeyCode::Char('l') => state.try_down().unwrap_or_else(|_| {
            state.error_message = Some("Failed to go into directory".into());
        }),
        KeyCode::Char('D') => {
            let args = if state.multi_select.len() != 0 {
                state
                    .multi_select
                    .iter()
                    .map(|x| x.full_path_str())
                    .collect::<Result<Vec<&str>, _>>()
                    .unwrap()
                    .join(" ")
            } else {
                if let Ok(Some(filename)) = state.filename_of_selected() {
                    filename
                } else {
                    state.error_message = Some("Faled to read filename".into());
                    return vec![];
                }
            };
            state.mode = Mode::CommandMode(PromptReader::new_with_placeholder(
                &format!(":delete {}", args),
                None,
            ))
        }
        KeyCode::Char(' ') => {
            if let Ok(Some(filename)) = state.filename_of_selected() {
                state.try_next().unwrap_or_else(|_| {
                    state.error_message = Some("Failed to advance cursor".into())
                });
                return vec![Action::Command(format!(":toggle_select {}", filename))];
            } else {
                state.error_message = Some("Faled to read filename".into());
            }
        }
        KeyCode::Char('y') => {
            if let Ok(Some(filename)) = state.filename_of_selected() {
                // TODO: If multiselect yank with all selected as arguments
                return vec![Action::Command(format!(":yank {}", filename))];
            } else {
                state.error_message = Some("Faled to read filename".into());
            }
        }
        KeyCode::Char('p') => {
            return vec![Action::Command(":paste".into())];
        }
        KeyCode::Char('A') => {
            if let Ok(Some(filename)) = state.filename_of_selected() {
                state.mode = Mode::CommandMode(PromptReader::new_with_placeholder(
                    &format!(":rename {}", filename),
                    None,
                ))
            } else {
                state.error_message = Some("Faled to read filename".into());
            }
        }
        KeyCode::Char(':') => {
            state.mode = Mode::CommandMode(PromptReader::new_with_placeholder(":", None))
        }
        KeyCode::Char('!') => {
            state.mode = Mode::ShellCommandMode(PromptReader::new_with_placeholder("!", None))
        }

        _ => {}
    };

    vec![]
}

fn main() -> Result<(), io::Error> {
    let mut terminal = create_terminal()?;

    let pwd = std::env::current_dir()?;
    let mut selected = HashMap::new();
    selected.insert(pwd.clone(), 0);

    let mut state = State {
        pwd,
        selected_in_pwd: selected,
        mode: state::Mode::NormalMode,
        yanked: vec![],
        multi_select: HashSet::new(),
        error_message: None,
        file_contents: Some("Example file contents".into()),
    };

    // Initialize preview window
    state.sync_file_contents();

    'main: loop {
        state.sync_file_contents();
        let render_state: RenderState = (&state)
            .try_into()
            .expect("Failed to generate render state");

        draw(&render_state, &mut terminal)?;

        let mut actions = vec![];

        if let Event::Key(key) = read()? {
            state.error_message = None;
            match &mut state.mode {
                Mode::ShellCommandMode(reader) => {
                    actions.append(&mut shell_mode_input(&key, reader))
                }
                Mode::CommandMode(reader) => {
                    actions.append(&mut command_mode_input(&key, reader));
                }
                Mode::NormalMode => {
                    actions.append(&mut normal_mode_input(&key, &mut state));
                }
            }
        }

        for action in actions {
            match action {
                Action::Quit => break 'main,
                Action::Command(cmd) => match execute_command(&cmd, &mut state) {
                    Err(e) => state.error_message = Some(e.to_string()),
                    _ => {}
                },
                Action::ShellCommand(cmd) => match execute_shell_command(&cmd) {
                    Err(e) => state.error_message = Some(e.to_string()),
                    _ => {}
                },
                Action::ModeSwitch(mode) => {
                    state.mode = mode;
                }
            }
        }
    }

    disable_raw_mode()?;
    execute!(
        terminal.backend_mut(),
        LeaveAlternateScreen,
        DisableMouseCapture
    )?;
    Ok(())
}
