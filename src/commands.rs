use std::{process::Command, path::Path};

use crate::{error::FilmanError, state::State};

fn rename(args: &[&str], state: &mut State) -> Result<(), FilmanError> {
    if args.len() != 1 {
        return Err(FilmanError::CommandError(
            ":rename takes one argument".into(),
        ));
    }

    if let Some(old_path) = state.path_of_selected()? {
        let new_name = state.pwd.join(args[0]);
        std::fs::rename(old_path, new_name)
            .map_err(|e| FilmanError::CommandError(e.to_string()))?;
        Ok(())
    } else {
        Err(FilmanError::EmptyDirectory)
    }
}

fn delete(args: &[&str], state: &mut State) -> Result<(), FilmanError> {
    if args.is_empty() {
        return Err(FilmanError::CommandError(
            ":delete takes at least one argument".into(),
        ));
    }
    for arg in args {
        let path = state.pwd.join(arg);

        // Remove from selection before deleting
        if state.multi_select.contains(&path) {
            state.multi_select.remove(&path);
        }

        if path.is_dir() {
            std::fs::remove_dir(path).map_err(|e| FilmanError::CommandError(e.to_string()))?;
        } else {
            std::fs::remove_file(path).map_err(|e| FilmanError::CommandError(e.to_string()))?;
        }
    }
    Ok(())
}

fn yank(args: &[&str], state: &mut State) -> Result<(), FilmanError> {
    if args.is_empty() {
        return Err(FilmanError::CommandError(
            ":yank takes at least one argument".into(),
        ));
    }

    state.yanked = vec![];

    for &arg in args.iter() {
        let path = state.pwd.join(arg);
        state.yanked.push(path);
    }

    Ok(())
}

fn paste(args: &[&str], state: &mut State) -> Result<(), FilmanError> {
    if !args.is_empty() {
        return Err(FilmanError::CommandError(
            ":paste takes not arguments".into(),
        ));
    }

    use crate::path::Path;

    let parent = &state.pwd;
    for path in &state.yanked {
        let filename = path.filename()?;

        let files_in_pwd = state.files_in_pwd()?;
        let filenames_in_pwd = files_in_pwd
            .iter()
            .map(|x| x.filename())
            .collect::<Result<Vec<_>, FilmanError>>()?;

        if filenames_in_pwd.contains(&filename) {
            return Err(FilmanError::FileOverwriteError(filename.into()));
        }

        std::fs::copy(path, parent.join(filename))
            .map_err(|e| FilmanError::CommandError(e.to_string()))?;
    }

    Ok(())
}

fn toggle_select(args: &[&str], state: &mut State) -> Result<(), FilmanError> {
    if args.is_empty() {
        return Err(FilmanError::CommandError(
            ":select takes at least one argument".into(),
        ));
    }
    for arg in args {
        let path = state.pwd.join(arg);
        if state.multi_select.contains(&path) {
            state.multi_select.remove(&path);
        } else {
            state.multi_select.insert(path);
        }
    }
    Ok(())
}

fn cursor_down(state: &mut State) -> Result<(), FilmanError> {
    let cursor_idx = state.selected_index_in_pwd();
    let files_in_pwd = state.files_in_pwd()?;

    // Wrap around
    let next_cursor_idx = if cursor_idx + 1 >= files_in_pwd.len() {
        0
    } else {
        cursor_idx + 1
    };
    state
        .selected_in_pwd
        .insert(state.pwd.clone(), next_cursor_idx);
    Ok(())
}

fn cursor_up(state: &mut State) -> Result<(), FilmanError> {
    let cursor_idx = state.selected_index_in_pwd();
    let files_in_pwd = state.files_in_pwd()?;

    // If empty directory don't do anything
    if files_in_pwd.is_empty() {
        return Ok(());
    }

    // Wrap around
    let next_cursor_idx = if cursor_idx == 0 {
        files_in_pwd.len() - 1
    } else {
        cursor_idx - 1
    };
    state
        .selected_in_pwd
        .insert(state.pwd.clone(), next_cursor_idx);
    Ok(())
}

fn cursor_descend(state: &mut State) -> Result<(), FilmanError> {
    let new_pwd = state
        .path_of_selected()?
        .ok_or(FilmanError::EmptyDirectory)?;
    if new_pwd.is_dir() {
        state.pwd = new_pwd;
        Ok(())
    } else {
        Err(FilmanError::NotADirectory)
    }
}

fn cursor_ascend(state: &mut State) -> Result<(), FilmanError> {
    let new_selected_index = state.selected_index_in_parent()?.ok_or(FilmanError::NoParentError)?;
    state.pwd = state.pwd.parent().ok_or(FilmanError::NoParentError)?.to_path_buf();
    let files_in_new_pwd = state.files_in_pwd()?;

    // Update cursor in new directory handling the case when parent has changed
    if new_selected_index >= files_in_new_pwd.len() {
        state.selected_in_pwd.insert(state.pwd.clone(), 0);
    } else {
        state.selected_in_pwd.insert(state.pwd.clone(), new_selected_index);
    }
    Ok(())
}

pub fn execute_command(cmd: &str, state: &mut State) -> Result<(), FilmanError> {
    let split_cmd = cmd.split(' ').collect::<Vec<&str>>();

    if let Some((&cmd_name, args)) = split_cmd.split_first() {
        match cmd_name {
            ":rename" => rename(args, state)?,
            ":delete" => delete(args, state)?,
            ":yank" => yank(args, state)?,
            ":paste" => paste(args, state)?,
            ":toggle_select" => toggle_select(args, state)?,
            ":cursor_down" => cursor_down(state)?,
            ":cursor_up" => cursor_up(state)?,
            ":cursor_ascend" => cursor_ascend(state)?,
            ":cursor_descend" => cursor_descend(state)?,
            _ => {
                return Err(FilmanError::CommandError(format!(
                    "Unrecognized command {cmd}"
                )))
            }
        }
    } else {
        return Err(FilmanError::CommandParseError(format!(
            "Could not split command string into command name and arguments {cmd}"
        )));
    };

    Ok(())
}

pub fn execute_shell_command(cmd: &str, pwd: &Path) -> Result<(), FilmanError> {
    let split_cmd = cmd.split(' ').collect::<Vec<&str>>();
    if let Some((&cmd_name, args)) = split_cmd.split_first() {
        let mut chars = cmd_name.chars();
        chars.next();
        Command::new(chars.as_str())
            .current_dir(pwd)
            .args(args)
            .output()
            .map_err(|e| FilmanError::ShellCommandError(e.to_string()))?;
    }
    Ok(())
}
