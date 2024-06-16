use std::{collections::HashMap, path::PathBuf, fs::File};

use crate::input::Action;

#[derive(serde::Deserialize, Debug, Clone)]
struct KeyConfig {
    simple: HashMap<char, String>
}

#[derive(serde::Deserialize, Debug, Clone)]
pub struct Config {
    keys: KeyConfig
}


impl Config {
    pub fn new(path: PathBuf) -> Self {
        let file = File::open(path).expect("Failed to open config.json");
        serde_json::from_reader(file).expect("Failed to parse config")
    }

    pub fn simple_keymap_actions(&self) -> HashMap<char, Vec<Action>> {
        self.keys.simple.iter().map(|(k, v)| {
            (*k, vec![Action::Command(v.to_string())])
        }).collect()
    }
}

