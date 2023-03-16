use crossterm::{
    event::{read, DisableMouseCapture, Event},
    execute,
    terminal::{disable_raw_mode, LeaveAlternateScreen},
};
use filman2::commands::{execute_command, execute_shell_command};
use filman2::draw::{create_terminal, draw, RenderState};
use filman2::input::{command_mode_input, normal_mode_input, shell_mode_input, Action};

use std::{
    collections::{HashMap, HashSet},
    io,
};

use filman2::state::{Mode, State};

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

        // eprintln!("State: {:?}", &state);
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

        // Execute all queued actions
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
