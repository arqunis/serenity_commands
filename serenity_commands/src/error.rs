use std::error::Error as StdError;
use std::fmt;

use serenity::model::interactions::application_command::ApplicationCommandOptionType;

#[derive(Debug, Clone)]
pub enum ParseError {
    InvalidType(ApplicationCommandOptionType),
    UnknownCommand(String),
    UnknownSubCommand(String),
    UnknownSubCommandGroup(String),
    UnknownOption(String),
    MissingOption(&'static str),
}

impl fmt::Display for ParseError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::InvalidType(kind) => write!(f, "invalid option type, expected {:?}", kind),
            Self::UnknownCommand(cmd) => write!(f, "unknown command \"{}\"", cmd),
            Self::UnknownSubCommand(cmd) => write!(f, "unknown subcommand \"{}\"", cmd),
            Self::UnknownSubCommandGroup(cmd) => write!(f, "unknown subcommand group \"{}\"", cmd),
            Self::UnknownOption(opt) => write!(f, "unknown option \"{}\"", opt),
            Self::MissingOption(opt) => write!(f, "missing option \"{}\"", opt),
        }
    }
}

impl StdError for ParseError {}
