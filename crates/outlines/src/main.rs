use ast_grep_language::SupportLang;
use outlines::render_file_outline;
use std::env;
use std::path::PathBuf;
use std::process::ExitCode;

fn main() -> ExitCode {
    match run() {
        Ok(()) => ExitCode::SUCCESS,
        Err(err) => {
            eprintln!("{err}");
            ExitCode::from(1)
        }
    }
}

fn run() -> Result<(), String> {
    let mut args = env::args().skip(1);
    let Some(path) = args.next() else {
        return Err("usage: outlines <path> [lang]".to_string());
    };
    let path = PathBuf::from(path);
    let lang = match args.next() {
        Some(lang) => Some(lang.parse::<SupportLang>().map_err(|err| err.to_string())?),
        None => None,
    };
    let output = render_file_outline(&path, lang)?;
    print!("{output}");
    Ok(())
}
