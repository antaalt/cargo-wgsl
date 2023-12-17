mod cli;
mod common;
mod naga;
mod dxc;
mod server;
mod shader_error;

use common::ShadingLanguage;

struct Config {
    server: bool,
    shading_language: ShadingLanguage
}

fn parse_args() -> Config {
    let mut server = false;
    let mut shading_language = ShadingLanguage::Wgsl;
    for argument in std::env::args() {
        match argument.as_str() {
            "--server" => { server = true; }
            "--hlsl" => { shading_language = ShadingLanguage::Hlsl; }
            "--wgsl" => { shading_language = ShadingLanguage::Wgsl; }
            _ => {}
        }
    }
    Config {
        server,
        shading_language,
    }
}

fn main() {
    let cfg = parse_args();

    let exit_code = if cfg.server {
        server::run();
        0
    } else {
        cli::run(cfg.shading_language)
    };

    std::process::exit(exit_code);
}
