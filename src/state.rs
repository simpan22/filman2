use core::fmt;
use std::{
    collections::{HashMap, HashSet},
    fs,
    path::PathBuf,
};

use prompter::PromptReader;

use crate::error::FilmanError;
use crate::path::Path;

#[derive(Clone, Debug)]
pub enum Mode {
    NormalMode,
    CommandMode(PromptReader),
    ShellCommandMode(PromptReader),
}

#[derive(Clone)]
pub struct State {
    pub pwd: PathBuf,
    pub selected_in_pwd: HashMap<PathBuf, usize>,
    pub mode: Mode,

    pub file_contents: Option<String>,
    pub yanked: HashSet<PathBuf>,
    pub multi_select: HashSet<PathBuf>,
    pub error_message: Option<String>,
}

impl fmt::Debug for State {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("State")
            .field("pwd", &self.pwd)
            .field("selected_in_pwd", &self.selected_in_pwd)
            .field("mode", &self.mode)
            .field("yanked", &self.yanked)
            .field("multi_select", &self.multi_select)
            .field("error_message", &self.error_message)
            .finish()
    }
}

impl State {
    pub fn path_of_selected(&self) -> Result<Option<PathBuf>, FilmanError> {
        let selected_index = self.selected_index_in_pwd();
        let files = self.files_in_pwd()?;

        if selected_index >= files.len() {
            return Ok(None);
        }

        Ok(Some(files[selected_index].clone()))
    }

    pub fn selected_index_in_pwd(&self) -> usize {
        *self.selected_in_pwd.get(&self.pwd).unwrap_or(&0)
    }

    pub fn selected_index_in_parent(&self) -> Result<Option<usize>, FilmanError> {
        let parent_files = self.files_in_parent()?;
        Ok(parent_files.into_iter().position(|x| x == self.pwd))
    }

    pub fn files_in_parent(&self) -> Result<Vec<PathBuf>, FilmanError> {
        if let Some(parent) = self.pwd.parent() {
            let path_iter = fs::read_dir(parent)?;
            let path_bufs = path_iter
                .map(|x| x.map(|y| y.path()))
                .collect::<Result<Vec<PathBuf>, _>>()?;
            Ok(path_bufs)
        } else {
            Ok(vec![])
        }
    }

    pub fn files_in_pwd(&self) -> Result<Vec<PathBuf>, FilmanError> {
        let paths = fs::read_dir(&self.pwd)?;
        Ok(paths
            .into_iter()
            .map(|x| x.map(|y| y.path()))
            .collect::<std::io::Result<Vec<PathBuf>>>()?)
    }

    pub fn filename_of_selected(&self) -> Result<Option<String>, FilmanError> {
        if let Some(path_of_selected) = self.path_of_selected()? {
            Ok(Some(path_of_selected.filename()?.to_string()))
        } else {
            Ok(None)
        }
    }

    pub fn sync_preview_file(&mut self) {
        if let Ok(Some(path)) = self.path_of_selected() {
            self.file_contents = fs::read_to_string(path).ok();
        }
    }
}
