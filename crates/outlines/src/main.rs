use ast_grep_language::{Language, SupportLang};
use outlines::render_file_outline;
use std::env;
use std::fs;
use std::path::{Path, PathBuf};
use std::process::ExitCode;

const USAGE: &str = "usage: outlines <path>... [--lang <lang>]";

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
    let (paths, lang) = parse_args(env::args().skip(1))?;
    let expanded = expand_paths(&paths, lang)?;
    if expanded.is_empty() {
        return Err("no supported source files found".to_string());
    }

    let mut outputs = Vec::with_capacity(expanded.len());
    for path in &expanded {
        outputs.push(render_file_outline(path, lang)?);
    }
    print!("{}", outputs.join("\n\n"));
    Ok(())
}

fn expand_paths(paths: &[PathBuf], lang: Option<SupportLang>) -> Result<Vec<PathBuf>, String> {
    let mut expanded = Vec::new();
    for path in paths {
        let metadata = fs::metadata(path)
            .map_err(|err| format!("failed to access {}: {err}", path.display()))?;
        if metadata.is_dir() {
            collect_dir_files(path, lang, &mut expanded)?;
        } else {
            expanded.push(path.clone());
        }
    }
    Ok(expanded)
}

fn collect_dir_files(
    dir: &Path,
    lang: Option<SupportLang>,
    output: &mut Vec<PathBuf>,
) -> Result<(), String> {
    let mut entries = Vec::new();
    for entry in fs::read_dir(dir)
        .map_err(|err| format!("failed to read directory {}: {err}", dir.display()))?
    {
        let entry = entry.map_err(|err| format!("failed to read entry in {}: {err}", dir.display()))?;
        entries.push(entry.path());
    }
    entries.sort();

    for path in entries {
        let metadata = fs::metadata(&path)
            .map_err(|err| format!("failed to access {}: {err}", path.display()))?;
        if metadata.is_dir() {
            collect_dir_files(&path, lang, output)?;
        } else if should_include_file(&path, lang) {
            output.push(path);
        }
    }
    Ok(())
}

fn should_include_file(path: &Path, lang: Option<SupportLang>) -> bool {
    match lang {
        Some(_) => SupportLang::from_path(path).is_some(),
        None => SupportLang::from_path(path).is_some(),
    }
}

fn parse_args(args: impl Iterator<Item = String>) -> Result<(Vec<PathBuf>, Option<SupportLang>), String> {
    let mut paths = Vec::new();
    let mut lang = None;
    let mut positional = Vec::new();
    let args: Vec<_> = args.collect();
    let mut iter = args.iter();

    while let Some(arg) = iter.next() {
        match arg.as_str() {
            "--lang" | "-l" => {
                let Some(value) = iter.next() else {
                    return Err(format!("missing language after {arg}\n{USAGE}"));
                };
                lang = Some(value.parse::<SupportLang>().map_err(|err| err.to_string())?);
            }
            _ => positional.push(arg.clone()),
        }
    }

    if positional.is_empty() {
        return Err(USAGE.to_string());
    }

    // Backward compatibility for the original `outlines <path> <lang>` form.
    if lang.is_none() && positional.len() == 2 {
        if let Ok(parsed) = positional[1].parse::<SupportLang>() {
            paths.push(PathBuf::from(&positional[0]));
            return Ok((paths, Some(parsed)));
        }
    }

    for path in positional {
        paths.push(PathBuf::from(path));
    }
    Ok((paths, lang))
}
