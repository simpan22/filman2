use thiserror::Error;

#[derive(Error, Debug)]
pub enum FilmanError {
    #[error("Error executing filman command: {0}")]
    CommandError(String),

    #[error("Error executing shell command: {0}")]
    ShellCommandError(String),

    #[error("Command parse error: {0}")]
    CommandParseError(String),

    #[error("Directory has no parent")]
    NoParentError,

    #[error("Failed to read directory")]
    ReadDirectoryError(#[from] std::io::Error),

    #[error("This would overwrite an existing file: {0}")]
    FileOverwriteError(String),

    #[error("Encountered non unicode characters in filename")]
    UnicodeError,

    #[error("No filename error")]
    PathHasNoFilename,

    #[error("Directory has no files")]
    EmptyDirectory,

    #[error("Not a directory")]
    NotADirectory,

    #[error("No selected file")]
    NoFileSelectedError,
}
