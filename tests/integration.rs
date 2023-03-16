use filman2::{
    commands::{execute_command, execute_shell_command},
    state::State,
};
use serial_test::serial;
use std::{
    collections::{HashMap, HashSet},
    fs::{create_dir, remove_dir_all},
};

fn create_test_state() -> State {
    let pwd = std::env::current_dir().unwrap();
    let test_dir = pwd.join("test_env");
    if let Err(e) = remove_dir_all(test_dir.clone()) {
        println!("Not removing old test env: {:?}", e);
    }
    create_dir(test_dir.clone()).unwrap();
    let mut selected = HashMap::new();
    selected.insert(test_dir.clone(), 0);

    let state = State {
        pwd: test_dir.clone(),
        selected_in_pwd: selected,
        mode: filman2::state::Mode::NormalMode,
        yanked: HashSet::new(),
        multi_select: HashSet::new(),
        error_message: None,
        file_contents: Some("Example file contents".into()),
    };

    assert_eq!(state.selected_index_in_pwd(), 0);
    assert!(state.files_in_pwd().unwrap().is_empty());

    state
}

#[test]
#[serial]
fn test_create_file() {
    let state = create_test_state();
    execute_shell_command("!touch test.txt", &state.pwd).unwrap();
    assert!(!state.files_in_pwd().unwrap().is_empty());
}

#[test]
#[serial]
fn test_crate_delete_file() {
    let mut state = create_test_state();
    execute_shell_command("!touch test.txt", &state.pwd).unwrap();
    assert!(!state.files_in_pwd().unwrap().is_empty());

    execute_command(":delete test.txt", &mut state).unwrap();
    assert!(state.files_in_pwd().unwrap().is_empty());
}

#[test]
#[serial]
fn test_rename_file() {
    let mut state = create_test_state();
    execute_shell_command("!touch test.txt", &state.pwd).unwrap();
    assert!(!state.files_in_pwd().unwrap().is_empty());

    execute_command(":rename test2.txt", &mut state).unwrap();
    let files = state.files_in_pwd().unwrap();
    let file_names: Vec<&str> = files
        .iter()
        .map(|x| x.file_name().unwrap().to_str().unwrap())
        .collect();

    assert_eq!(file_names, vec!["test2.txt"]);
}

#[test]
#[serial]
fn yank_paste_test() {
    let mut state = create_test_state();
    // Create two directories: "from" and "to"
    execute_shell_command("!mkdir from", &state.pwd).unwrap();
    execute_shell_command("!mkdir to", &state.pwd).unwrap();

    // Descend into "from"
    execute_command(":cursor_descend", &mut state).unwrap();

    // Create test file and yank it
    execute_shell_command("!touch test.txt", &state.pwd).unwrap();
    execute_command(":yank test.txt", &mut state).unwrap();

    // Go to "to"-directory
    execute_command(":cursor_ascend", &mut state).unwrap();
    execute_command(":cursor_down", &mut state).unwrap();
    execute_command(":cursor_descend", &mut state).unwrap();

    // Paste
    execute_command(":paste", &mut state).unwrap();

    let files = state.files_in_pwd().unwrap();
    let file_names: Vec<&str> = files
        .iter()
        .map(|x| x.file_name().unwrap().to_str().unwrap())
        .collect();

    assert_eq!(file_names, vec!["test.txt"]);

    // Go back to "from" to make sure the file is still there
    execute_command(":cursor_ascend", &mut state).unwrap();
    execute_command(":cursor_down", &mut state).unwrap();
    execute_command(":cursor_descend", &mut state).unwrap();

    let files = state.files_in_pwd().unwrap();
    let file_names: Vec<&str> = files
        .iter()
        .map(|x| x.file_name().unwrap().to_str().unwrap())
        .collect();

    assert_eq!(file_names, vec!["test.txt"]);
}

#[test]
#[serial]
fn multi_select() {
    let mut state = create_test_state();
    // Create three test files
    execute_shell_command("!touch a", &state.pwd).unwrap();
    execute_shell_command("!touch b", &state.pwd).unwrap();
    execute_shell_command("!touch c", &state.pwd).unwrap();

    // Select two of them
    execute_command(":toggle_select a", &mut state).unwrap();
    execute_command(":toggle_select b", &mut state).unwrap();

    let selected: HashSet<&str> = state
        .multi_select
        .iter()
        .map(|x| x.file_name().unwrap().to_str().unwrap())
        .collect();

    assert!(selected.contains("a"));
    assert!(selected.contains("b"));
}
