use filman2::{
    commands::{execute_command, execute_shell_command},
    state::State,
};
use serial_test::serial;
use std::{
    collections::{HashMap, HashSet},
    fs::{create_dir, remove_dir_all},
    path::PathBuf,
};

struct TestContext {
    state: State,
    directory: PathBuf,
}

impl TestContext {
    fn new() -> Self {
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

        TestContext {
            state: state,
            directory: test_dir,
        }
    }
}

impl Drop for TestContext {
    fn drop(&mut self) {
        if let Err(e) = remove_dir_all(self.directory.clone()) {
            println!("Not removing old test env: {:?}", e);
        }
    }
}

#[test]
#[serial]
fn test_create_file() {
    let ctx = TestContext::new();
    execute_shell_command("!touch test.txt", &ctx.state.pwd).unwrap();
    assert!(!ctx.state.files_in_pwd().unwrap().is_empty());
}

#[test]
#[serial]
fn test_crate_delete_file() {
    let mut ctx = TestContext::new();
    execute_shell_command("!touch test.txt", &ctx.state.pwd).unwrap();
    assert!(!ctx.state.files_in_pwd().unwrap().is_empty());

    execute_command(":delete test.txt", &mut ctx.state).unwrap();
    assert!(ctx.state.files_in_pwd().unwrap().is_empty());
}

#[test]
#[serial]
fn test_rename_file() {
    let mut ctx = TestContext::new();
    execute_shell_command("!touch test.txt", &ctx.state.pwd).unwrap();
    assert!(!ctx.state.files_in_pwd().unwrap().is_empty());

    execute_command(":rename test2.txt", &mut ctx.state).unwrap();
    let files = ctx.state.files_in_pwd().unwrap();
    let file_names: Vec<&str> = files
        .iter()
        .map(|x| x.file_name().unwrap().to_str().unwrap())
        .collect();

    assert_eq!(file_names, vec!["test2.txt"]);
}

#[test]
#[serial]
fn yank_paste_test() {
    let mut ctx = TestContext::new();
    // Create two directories: "from" and "to"
    execute_shell_command("!mkdir from", &ctx.state.pwd).unwrap();
    execute_shell_command("!mkdir to", &ctx.state.pwd).unwrap();

    // Descend into "from"
    execute_command(":cursor_descend", &mut ctx.state).unwrap();

    // Create test file and yank it
    execute_shell_command("!touch test.txt", &ctx.state.pwd).unwrap();
    execute_command(":yank test.txt", &mut ctx.state).unwrap();

    // Go to "to"-directory
    execute_command(":cursor_ascend", &mut ctx.state).unwrap();
    execute_command(":cursor_down", &mut ctx.state).unwrap();
    execute_command(":cursor_descend", &mut ctx.state).unwrap();

    // Paste
    execute_command(":paste", &mut ctx.state).unwrap();

    let files = ctx.state.files_in_pwd().unwrap();
    let file_names: Vec<&str> = files
        .iter()
        .map(|x| x.file_name().unwrap().to_str().unwrap())
        .collect();

    assert_eq!(file_names, vec!["test.txt"]);

    // Go back to "from" to make sure the file is still there
    execute_command(":cursor_ascend", &mut ctx.state).unwrap();
    execute_command(":cursor_down", &mut ctx.state).unwrap();
    execute_command(":cursor_descend", &mut ctx.state).unwrap();

    let files = ctx.state.files_in_pwd().unwrap();
    let file_names: Vec<&str> = files
        .iter()
        .map(|x| x.file_name().unwrap().to_str().unwrap())
        .collect();

    assert_eq!(file_names, vec!["test.txt"]);
}

#[test]
#[serial]
fn multi_select() {
    let mut ctx = TestContext::new();
    // Create three test files
    execute_shell_command("!touch a", &ctx.state.pwd).unwrap();
    execute_shell_command("!touch b", &ctx.state.pwd).unwrap();
    execute_shell_command("!touch c", &ctx.state.pwd).unwrap();

    // Select two of them
    execute_command(":toggle_select a", &mut ctx.state).unwrap();
    execute_command(":toggle_select b", &mut ctx.state).unwrap();

    let selected: HashSet<&str> = ctx
        .state
        .multi_select
        .iter()
        .map(|x| x.file_name().unwrap().to_str().unwrap())
        .collect();

    assert!(selected.contains("a"));
    assert!(selected.contains("b"));
}
