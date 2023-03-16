use crate::path::Path;
use crate::state::{Mode, State};
use crossterm::event::{KeyCode, KeyEvent};
use prompter::PromptReader;

pub enum Action {
    ShellCommand(String),
    Command(String),
    ModeSwitch(Mode),
    Quit,
}

pub fn shell_mode_input(key: &KeyEvent, reader: &mut PromptReader) -> Vec<Action> {
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

pub fn command_mode_input(key: &KeyEvent, reader: &mut PromptReader) -> Vec<Action> {
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

pub fn normal_mode_input(key: &KeyEvent, state: &mut State) -> Vec<Action> {
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
