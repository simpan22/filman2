use std::process::Command;

use crate::{error::FilmanError, state::State};

fn rename(args: &[&str], state: &mut State) -> Result<(), FilmanError> {
    if args.len() != 1 {
        return Err(FilmanError::CommandError(
            ":rename takes one argument".into(),
        ));
    }

    let old_path = state.path_of_selected();
    let new_name = state.path_of_parent()?.join(args[0]);

    std::fs::rename(old_path, new_name).map_err(|e| FilmanError::CommandError(e.to_string()))?;
    Ok(())
}

fn delete(args: &[&str], state: &mut State) -> Result<(), FilmanError> {
    if args.len() != 1 {
        return Err(FilmanError::CommandError(
            ":delete takes one argument".into(),
        ));
    }

    let path = state.path_of_parent()?.join(args[0]);

    if path.is_dir() {
        std::fs::remove_dir(path).map_err(|e| FilmanError::CommandError(e.to_string()))?;
    } else {
        std::fs::remove_file(path).map_err(|e| FilmanError::CommandError(e.to_string()))?;
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
        let path = state.path_of_parent()?.join(arg);
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

    let parent = state.path_of_parent()?;
    for path in &state.yanked {
        let filename = path.file_name().unwrap();

        let files_in_pwd = state.files_in_pwd();
        let filenames_in_pwd: Vec<_> = files_in_pwd.iter().map(|x| x.file_name().unwrap()).collect();


        if filenames_in_pwd.contains(&filename) {
            return Err(FilmanError::FileOverwriteError(filename.to_str().unwrap().to_string()));
        }

        std::fs::copy(path, parent.join(filename))
            .map_err(|e| FilmanError::CommandError(e.to_string()))?;
    }

    Ok(())
}

fn toggle_select(args: &[&str], state: &mut State) -> Result<(), FilmanError> {
    if args.is_empty() && args.len() > 1 {
        return Err(FilmanError::CommandError(
            ":select takes one argument".into(),
        ));
    }

    let path = state.path_of_parent()?.join(args[0]);
    if state.multi_select.contains(&path) {
        state.multi_select.remove(&path);
    } else {
        state.multi_select.insert(path);
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
            _ => return Err(FilmanError::CommandError(format!("Unrecognized command {cmd}"))),
        }
    } else {
        return Err(FilmanError::CommandParseError(
            format!("Could not split command string into command name and arguments {cmd}"),
        ));
    };

    Ok(())
}

pub fn execute_shell_command(cmd: &str) -> Result<(), FilmanError> {
    let split_cmd = cmd.split(' ').collect::<Vec<&str>>();
    if let Some((&cmd_name, args)) = split_cmd.split_first() {
        let mut chars = cmd_name.chars();
        chars.next();
        Command::new(chars.as_str())
            .args(args)
            .output()
            .map_err(|e| FilmanError::ShellCommandError(e.to_string()))?;
    }
    Ok(())
}
