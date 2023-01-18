use std::process::Command;

use crate::state::State;

fn rename(args: &[&str], state: &mut State) -> Result<(), String> {
    if args.len() != 1 {
        return Err(":rename takes one argument".into());
    }

    let old_path = state.path_of_selected();
    let new_name = old_path.parent().unwrap().join(args[0]);

    std::fs::rename(old_path, new_name).map_err(|e| e.to_string())?;
    Ok(())
}

fn delete(args: &[&str], state: &mut State) -> Result<(), String> {
    if args.len() != 1 {
        return Err(":delete takes one argument".into());
    }

    let path = state.path_of_selected().parent().unwrap().join(args[0]);

    if path.is_dir() {
        std::fs::remove_dir(path).map_err(|e| e.to_string())?;
    } else {
        std::fs::remove_file(path).map_err(|e| e.to_string())?;
    }

    Ok(())
}

pub fn execute_command(cmd: &str, state: &mut State) -> Result<(), String> {
    let split_cmd = cmd.split(' ').collect::<Vec<&str>>();

    if let Some((&cmd_name, args)) = split_cmd.split_first() {
        match cmd_name {
            ":rename" => rename(args, state)?,
            ":delete" => delete(args, state)?,
            _ => {
                return Err("Unrecognized command".into())
            }
        }
    } else {
        return Err("Could not split command string into command name and arguments".into())
    };

    Ok(())
}

pub fn execute_shell_command(cmd: &str) -> Result<(), String> {
    let split_cmd = cmd.split(' ').collect::<Vec<&str>>();
    if let Some((&cmd_name, args)) = split_cmd.split_first() {
        let mut chars = cmd_name.chars();
        chars.next();
        Command::new(chars.as_str())
            .args(args)
            .output()
            .map_err(|e| e.to_string())?;
    }
    Ok(())
}
