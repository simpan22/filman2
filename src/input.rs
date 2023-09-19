use std::collections::HashMap;

use crate::config::Config;
use crate::path::Path;
use crate::state::{Mode, State};
use crossterm::event::{KeyCode, KeyEvent};
use lazy_static::lazy_static;
use prompter::PromptReader;

lazy_static! {
    static ref KEYBINDINGS: HashMap<char, Vec<Action>> = {
        let mut keymap = HashMap::new();
        keymap.insert('q', vec![Action::Quit]);
        keymap.insert(
            ':',
            vec![Action::ModeSwitch(Mode::CommandMode(
                PromptReader::new_with_placeholder(":", None),
            ))],
        );
        keymap.insert(
            '!',
            vec![Action::ModeSwitch(Mode::ShellCommandMode(
                PromptReader::new_with_placeholder("!", None),
            ))],
        );
        let config = Config::new("config.json".into());
        let keys = config.simple_keymap_actions();
        keymap.extend(keys.into_iter());
        keymap
    };
}
#[derive(Clone)]
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
        KeyCode::Char(' ') => {
            if let Some(filename) = state.filename_of_selected() {
                vec![
                    Action::Command(format!(":toggle_select {}", filename)),
                    Action::Command(":cursor_down".into()),
                ]
            } else {
                state.error_message = Some("Faled to read filename".into());
                vec![]
            }
        }
        KeyCode::Char('D') => {
            let args = state
                .multi_select_or_selected()
                .iter()
                .map(|x| x.full_path_str().unwrap())
                .collect::<Vec<&str>>()
                .join(" ");
            vec![Action::ModeSwitch(Mode::CommandMode(
                PromptReader::new_with_placeholder(&format!(":delete {}", args), None),
            ))]
        }
        KeyCode::Char('y') => {
            let args = state
                .multi_select_or_selected()
                .iter()
                .map(|x| x.full_path_str().unwrap())
                .collect::<Vec<&str>>()
                .join(" ");
            vec![
                Action::Command(format!(":yank {}", args)),
                Action::Command(":clear_selection".into()),
            ]
        }
        KeyCode::Char('A') => {
            if let Some(filename) = state.filename_of_selected() {
                vec![Action::ModeSwitch(Mode::CommandMode(
                    PromptReader::new_with_placeholder(&format!(":rename {}", filename), None),
                ))]
            } else {
                state.error_message = Some("Faled to read filename".into());
                vec![]
            }
        }

        KeyCode::Char(c) => custom_simple_binding(c),
        _ => vec![],
    }
}

fn custom_simple_binding(c: char) -> Vec<Action> {
    KEYBINDINGS.get(&c).cloned().unwrap_or(vec![])
}
