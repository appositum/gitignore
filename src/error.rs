use ansi_term::Color::Red;
use std::fmt::{self, Display};
use std::error;
use tokio::task;

#[derive(Debug)]
pub enum GIError {
    Json(serde_json::Error),
    Request(reqwest::Error),
    TaskJoin(task::JoinError),
    TemplateNotFound(Vec<String>),
}

impl Display for GIError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            GIError::Json(e) => {
                write!(f, "{} {}", Red.paint("json parse error:"), e)
            }
            GIError::Request(e) => {
                write!(f, "{} {}", Red.paint("request error:"), e)
            }
            GIError::TaskJoin(e) => {
                write!(f, "{} {}", Red.paint("tokio join error:"), e)
            }
            GIError::TemplateNotFound(vec) => {
                let templates = vec
                    .into_iter()
                    .map(|t| format!("  {}", t))
                    .collect::<Vec<String>>()
                    .join("\n");

                write!(
                    f,
                    "{} template(s) not found:\n{}",
                    Red.paint("error:"),
                    templates
                )
            }
        }
    }
}

impl error::Error for GIError {
    fn source(&self) -> Option<&(dyn error::Error + 'static)> {
        match self {
            GIError::Json(e) => Some(e),
            GIError::Request(e) => Some(e),
            GIError::TaskJoin(e) => Some(e),
            GIError::TemplateNotFound(_vec) => None,
        }
    }
}

impl From<serde_json::Error> for GIError {
    fn from(err: serde_json::Error) -> GIError {
        GIError::Json(err)
    }
}

impl From<reqwest::Error> for GIError {
    fn from(err: reqwest::Error) -> GIError {
        GIError::Request(err)
    }
}

impl From<task::JoinError> for GIError {
    fn from(err: task::JoinError) -> GIError {
        GIError::TaskJoin(err)
    }
}
