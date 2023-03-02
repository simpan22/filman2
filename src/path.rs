use crate::error::FilmanError;
use std::{fs, path::PathBuf};

pub trait Path {
    fn filename(&self) -> Result<&str, FilmanError>;
    fn full_path_str(&self) -> Result<&str, FilmanError>;
    fn size(&self) -> Result<u64, FilmanError>;
}

impl Path for PathBuf {
    fn filename(&self) -> Result<&str, FilmanError> {
        Ok(self
            .file_name()
            .ok_or(FilmanError::PathHasNoFilename)?
            .to_str()
            .ok_or(FilmanError::UnicodeError)?)
    }

    fn full_path_str(&self) -> Result<&str, FilmanError> {
        Ok(self.to_str().ok_or(FilmanError::UnicodeError)?)
    }

    fn size(&self) -> Result<u64, FilmanError> {
        Ok(fs::metadata(self)?.len())
    }

}
