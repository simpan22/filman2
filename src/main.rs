use crossterm::{
    event::{read, DisableMouseCapture, Event, KeyCode, KeyEvent},
    execute,
    terminal::{disable_raw_mode, LeaveAlternateScreen},
};
use filman2::commands::{execute_command, execute_shell_command};
use filman2::draw::{create_terminal, draw, RenderState};
use filman2::path::Path;
use prompter::PromptReader;

use std::{
    collections::{HashMap, HashSet},
    io,
};

use filman2::state::{Mode, State};

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
        KeyCode::Esc => return vec![Action::ModeSwitch(Mode::NormalMode)],
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
        KeyCode::Esc => return vec![Action::ModeSwitch(Mode::NormalMode)],
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
        KeyCode::Char('j') => return vec![Action::Command(":cursor_down".into())],
        KeyCode::Char('k') => return vec![Action::Command(":cursor_up".into())],
        KeyCode::Char('h') => return vec![Action::Command(":cursor_ascend".into())],
        KeyCode::Char('l') => return vec![Action::Command(":cursor_descend".into())],
        KeyCode::Char('D') => {
            let args = if !state.multi_select.is_empty() {
                state
                    .multi_select
                    .iter()
                    .filter(|p| p.parent() == Some(&state.pwd))
                    .map(|x| x.full_path_str())
                    .collect::<Result<Vec<&str>, _>>()
                    .unwrap()
                    .join(" ")
            } else if let Ok(Some(filename)) = state.filename_of_selected() {
                filename
            } else {
                state.error_message = Some("Faled to read filename".into());
                return vec![];
            };
            state.mode = Mode::CommandMode(PromptReader::new_with_placeholder(
                &format!(":delete {}", args),
                None,
            ))
        }
        KeyCode::Char(' ') => {
            if let Ok(Some(filename)) = state.filename_of_selected() {
                return vec![
                    Action::Command(format!(":toggle_select {}", filename)),
                    Action::Command(":cursor_down".into()),
                ];
            } else {
                state.error_message = Some("Faled to read filename".into());
            }
        }
        KeyCode::Char('y') => {
            let args = if !state.multi_select.is_empty() {
                state
                    .multi_select
                    .iter()
                    .filter(|p| p.parent() == Some(&state.pwd))
                    .map(|x| x.full_path_str())
                    .collect::<Result<Vec<&str>, _>>()
                    .unwrap()
                    .join(" ")
            } else if let Ok(Some(filename)) = state.filename_of_selected() {
                filename
            } else {
                state.error_message = Some("No files seem to be selected".into());
                return vec![];
            };

            return vec![
                Action::Command(format!(":yank {}", args)),
                Action::Command(":clear_selection".into()),
            ];
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
        mode: filman2::state::Mode::NormalMode,
        yanked: HashSet::new(),
        multi_select: HashSet::new(),
        error_message: None,
        file_contents: Some("Example file contents".into()),
    };

    // Initialize preview window
    state.sync_preview_file();

    'main: loop {
        state.sync_preview_file();
        let render_state: RenderState = (&state)
            .try_into()
            .expect("Failed to generate render state");

        eprintln!("State: {:?}", &state);
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
                Action::Command(cmd) => {
                    if let Err(e) = execute_command(&cmd, &mut state) {
                        state.error_message = Some(e.to_string());
                    }
                }
                Action::ShellCommand(cmd) => {
                    if let Err(e) = execute_shell_command(&cmd, &state.pwd) {
                        state.error_message = Some(e.to_string());
                    }
                }
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
