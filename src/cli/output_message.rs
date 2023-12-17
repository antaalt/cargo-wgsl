use colored::*;

use crate::shader_error::{ShaderError, ShaderErrorList, ShaderErrorSeverity};
use std::path::Path;

pub struct OutputMessage {
    pub is_err: bool,
    pub text: String,
}

impl OutputMessage {
    pub fn success(path: &Path) -> Self {
        let success = "Success".bright_green().bold();
        OutputMessage {
            is_err: false,
            text: format!("✅ {} {}", success, path.display()),
        }
    }

    pub fn error(path: &Path, error_list: ShaderErrorList) -> Self {
        let mut err_text = String::new();
        for error in error_list.errors {
            let str = match error {
                ShaderError::ParserErr { severity, error, line, pos } => {
                    let arrow = "-->".blue();
                    let location = format!("{}:{}:{}", path.display(), line, pos);
                    let severity_string = severity.to_string();
                    let error = format!("{}: {}", match severity {
                        ShaderErrorSeverity::Error => severity_string.red().bold(),
                        ShaderErrorSeverity::Warning => severity_string.yellow().bold(),
                        ShaderErrorSeverity::Information => severity_string.blue().bold(),
                        ShaderErrorSeverity::Hint => severity_string.bright_black().bold(),
                    }, error);

                    format!("{} {}\n{}\n", arrow, location, error)
                }
                ShaderError::ValidationErr { error, emitted, .. } => {
                    format!("❌ {} \n{:?} {}\n", path.display(), error, emitted)
                }
                ShaderError::IoErr(err) => {
                    format!("❌ {} \n{:?}\n", path.display(), err)
                }
                ShaderError::InternalErr(message) => {
                    format!("❌ {} \n{}\n", path.display(), message)
                }
            };
            err_text.push_str(&str);
        };

        Self {
            is_err: true,
            text: err_text,
        }
    }
}
