mod draw;
pub mod state;

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

        match read()? {
            Event::Key(key) => {
                match &mut state.mode {
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
                            state.mode = Mode::NormalMode;
                            //TODO: Execute the command
                        }
                    }
                    Mode::NormalMode => match key.code {
                        KeyCode::Char('q') => break,
                        KeyCode::Char('j') => state.try_next(),

                        KeyCode::Char('k') => state.try_prev(),
                        KeyCode::Char('h') => state.try_up(),
                        KeyCode::Char('l') => state.try_down(),
                        KeyCode::Char(':') => {
                            state.mode =
                                Mode::CommandMode(PromptReader::new_with_placeholder(":", None))
                        }
                        KeyCode::Char('!') => {
                            state.mode =
                                Mode::CommandMode(PromptReader::new_with_placeholder("!", None))
                        }

                        _ => {}
                    },
                }
            }
            _ => {}
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
