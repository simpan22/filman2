use std::{collections::HashMap, fs, path::PathBuf};

use prompter::PromptReader;

#[derive(Clone)]
pub enum Mode{
    NormalMode,
    CommandMode(PromptReader)
}

#[derive(Clone)]
pub struct State {
    pub pwd: PathBuf,
    pub selected_in_pwd: HashMap<PathBuf, usize>,
    pub mode: Mode,
}

impl State {
    pub fn try_next(&mut self) {
        let current = self.selected_in_pwd();
        if current != self.files_in_pwd().len() - 1 {
            self.selected_in_pwd.insert(self.pwd.clone(), current + 1);
        }
    }

    pub fn try_prev(&mut self) {
        let current = self.selected_in_pwd();
        if current != 0 {
            self.selected_in_pwd.insert(self.pwd.clone(), current - 1);
        }
    }

    pub fn try_down(&mut self) {
        if self.path_of_selected().is_dir() {
            self.pwd = self.path_of_selected()
        }
    }

    pub fn try_up(&mut self) {
        let parent_index = self.selected_in_parent().unwrap_or(0);
        self.pwd = self.pwd.parent().unwrap_or(&self.pwd).to_path_buf();
        self.selected_in_pwd.insert(self.pwd.clone(), parent_index);
    }

    pub fn parent(&self) -> Option<PathBuf> {
        self.pwd.parent().map(|x| x.to_path_buf())
    }

    pub fn path_of_selected(&self) -> PathBuf {
        self.files_in_pwd()[self.selected_in_pwd()].clone()
    }

    pub fn selected_in_pwd(&self) -> usize {
        *self.selected_in_pwd.get(&self.pwd).unwrap_or(&0)
    }

    pub fn selected_in_parent(&self) -> Option<usize> {
        let parent_files = self.files_in_parent();
        parent_files.into_iter().position(|x| x == self.pwd)
    }

    pub fn files_in_parent(&self) -> Vec<PathBuf> {
        if let Some(parent) = self.parent() {
            let paths = fs::read_dir(parent).unwrap();
            paths.into_iter().map(|x| x.unwrap().path()).collect()
        } else {
            vec![]
        }
    }

    pub fn files_in_pwd(&self) -> Vec<PathBuf> {
        let paths = fs::read_dir(self.pwd.clone()).unwrap();
        paths.into_iter().map(|x| x.unwrap().path()).collect()
    }
}
