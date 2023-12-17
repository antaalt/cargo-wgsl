use std::path::PathBuf;
use std::str::FromStr;

use crate::common::Validator;
use crate::dxc::Dxc;
use crate::naga::Naga;
use crate::shader_error::ShaderErrorList;
use crate::common::ShadingLanguage;
use crate::shader_error::ShaderErrorSeverity;

use jsonrpc_stdio_server::jsonrpc_core::*;
use jsonrpc_stdio_server::ServerBuilder;

use serde::{Deserialize, Serialize};

#[allow(non_snake_case)]
#[derive(Debug, Serialize, Deserialize)]
struct ValidateFileParams {
    path: PathBuf,
    shadingLanguage: String,
}

#[derive(Debug, Serialize, Deserialize)]
enum ValidateFileError {
    ParserErr {
        severity: String,
        error: String,
        scopes: Vec<String>,
        line: usize,
        pos: usize,
    },
    ValidationErr {
        message: String,
        debug: String,
    },
    UnknownError(String),
}

#[allow(non_snake_case)]
#[derive(Debug, Serialize, Deserialize)]
struct ValidateFileResponse {
    IsOk: bool,
    Messages: Vec<ValidateFileError>,
}
impl ValidateFileResponse {
    fn ok() -> Self {
        Self {
            IsOk: true,
            Messages: Vec::new()
        }
    }
    fn error(error_list: &ShaderErrorList) -> Self {
        use crate::shader_error::ShaderError;
        let mut errors = Vec::new();
        for error in &error_list.errors {
            errors.push(match error {
                ShaderError::ParserErr { severity, error, line, pos } => {
                    ValidateFileError::ParserErr {
                        severity: severity.to_string(),
                        error: error.clone(),
                        scopes: vec![],
                        line: *line,
                        pos: *pos,
                    }
                }
                ShaderError::ValidationErr { src, error, .. } => {
                    if let Some((span, _)) = error.spans().next() {
                        let loc = span.location(&src);
                        ValidateFileError::ParserErr {
                            severity: ShaderErrorSeverity::Error.to_string(),
                            error: format!("{}.\n\n{:#?}", error, error),
                            scopes: vec![],
                            line: loc.line_number as usize,
                            pos: loc.line_position as usize,
                        }
                    } else {
                        ValidateFileError::ValidationErr {
                            message: format!("{}.\n\n{:#?}", error, error),
                            debug: format!("{:#?}", error),
                        }
                    }
                }
                err => ValidateFileError::UnknownError(format!("{:#?}", err)),
            });
        }
        Self {
            IsOk: false,
            Messages: errors
        }
    }
}

pub fn run() {
    let rt = tokio::runtime::Runtime::new().unwrap();

    rt.block_on(async {
        let mut io = IoHandler::default();

        // Protocol Version
        io.add_sync_method("version", move |_| Ok(Value::from("0.0.1")));
        // Binary Version
        io.add_sync_method("binary_version", move |_| Ok(Value::from("0.0.4")));

        io.add_sync_method("get_file_tree", move |params: Params| {
            let params: ValidateFileParams = params.parse()?;
            
            let shading_language_parsed = ShadingLanguage::from_str(params.shadingLanguage.as_str());
            let shading_language = match shading_language_parsed {
                Ok(res) => res,
                Err(_) => { return Err(Error::invalid_params(format!("Invalid shading language: {}", params.shadingLanguage))); }
            };

            let mut validator : Box<dyn Validator> = match shading_language {
                ShadingLanguage::Wgsl => Box::new(Naga::new()),
                ShadingLanguage::Hlsl => Box::new(Dxc::new().expect("Failed to create DXC"))
            };

            let tree = validator.get_shader_tree(&params.path).ok();

            Ok(to_value(tree).unwrap())
        });

        io.add_sync_method("validate_file", move |params: Params| {
            let params: ValidateFileParams = params.parse()?;
            
            let shading_language_parsed = ShadingLanguage::from_str(params.shadingLanguage.as_str());
            let shading_language = match shading_language_parsed {
                Ok(res) => res,
                Err(()) => { return Err(Error::invalid_params(format!("Invalid shading language: {}", params.shadingLanguage))); }
            };

            let mut validator : Box<dyn Validator> = match shading_language {
                ShadingLanguage::Wgsl => Box::new(Naga::new()),
                ShadingLanguage::Hlsl => Box::new(Dxc::new().expect("Failed to create DXC"))
            };

            let res = match validator.validate_shader(&params.path) {
                Ok(_) => ValidateFileResponse::ok(),
                Err(err) => ValidateFileResponse::error(&err)
            };

            Ok(to_value(res).unwrap())
        });

        let server = ServerBuilder::new(io).build();
        server.await;
    })
}
