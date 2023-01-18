mod commands;
mod draw;
pub mod state;

use commands::{execute_command, execute_shell_command};
use crossterm::{
    event::{read, DisableMouseCapture, Event, KeyCode},
    execute,
    terminal::{disable_raw_mode, LeaveAlternateScreen},
};
use draw::{create_terminal, draw};
use prompter::PromptReader;

use std::{collections::HashMap, io};

use state::{Mode, State};

fn main() -> Result<(), io::Error> {
    let mut terminal = create_terminal()?;

    let pwd = std::env::current_dir()?;
    let mut selected = HashMap::new();
    selected.insert(pwd.clone(), 0);

    let mut state = State {
        pwd,
        selected_in_pwd: selected,
        mode: state::Mode::NormalMode,
    };

    loop {
        draw(&state.clone().into(), &mut terminal)?;
        let mut command_to_run: Option<String> = None;
        let mut shell_command_to_run: Option<String> = None;
        match read()? {
            Event::Key(key) => match &mut state.mode {
                Mode::ShellCommandMode(reader) => {
                    match key.code {
                        KeyCode::Char(c) => reader.next_key(c.into()),
                        KeyCode::Enter => reader.next_key(prompter::keycodes::KeyCode::Enter),
                        KeyCode::Left => reader.next_key(prompter::keycodes::KeyCode::Left),
                        KeyCode::Right => reader.next_key(prompter::keycodes::KeyCode::Right),

                        KeyCode::Backspace => {
                            reader.next_key(prompter::keycodes::KeyCode::Backspace)
                        }

                        KeyCode::Delete => reader.next_key(prompter::keycodes::KeyCode::Delete),
                        _ => {}
                    }

                    if reader.done() {
                        shell_command_to_run = Some(reader.result().to_string());
                        state.mode = Mode::NormalMode;
                    }
                }
                Mode::CommandMode(reader) => {
                    match key.code {
                        KeyCode::Char(c) => reader.next_key(c.into()),
                        KeyCode::Enter => reader.next_key(prompter::keycodes::KeyCode::Enter),
                        KeyCode::Left => reader.next_key(prompter::keycodes::KeyCode::Left),
                        KeyCode::Right => reader.next_key(prompter::keycodes::KeyCode::Right),

                        KeyCode::Backspace => {
                            reader.next_key(prompter::keycodes::KeyCode::Backspace)
                        }

                        KeyCode::Delete => reader.next_key(prompter::keycodes::KeyCode::Delete),
                        _ => {}
                    }

                    if reader.done() {
                        command_to_run = Some(reader.result().to_string());
                        state.mode = Mode::NormalMode;
                    }
                }
                Mode::NormalMode => match key.code {
                    KeyCode::Char('q') => break,
                    KeyCode::Char('j') => state.try_next(),
                    KeyCode::Char('k') => state.try_prev(),
                    KeyCode::Char('h') => state.try_up(),
                    KeyCode::Char('l') => state.try_down(),
                    KeyCode::Char('D') => {
                        state.mode = Mode::CommandMode(PromptReader::new_with_placeholder(
                            &format!(":delete {}", state.filename_of_selected()),
                            None,
                        ))
                    }
                    KeyCode::Char('A') => {
                        state.mode = Mode::CommandMode(PromptReader::new_with_placeholder(
                            &format!(":rename {}", state.filename_of_selected()),
                            None,
                        ))
                    }
                    KeyCode::Char(':') => {
                        state.mode =
                            Mode::CommandMode(PromptReader::new_with_placeholder(":", None))
                    }
                    KeyCode::Char('!') => {
                        state.mode =
                            Mode::ShellCommandMode(PromptReader::new_with_placeholder("!", None))
                    }

                    _ => {}
                },
            },
            _ => {}
        }

        if let Some(cmd) = command_to_run {
            execute_command(&cmd, &mut state).unwrap();
        }
        if let Some(cmd) = shell_command_to_run {
            execute_shell_command(&cmd).unwrap();
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
