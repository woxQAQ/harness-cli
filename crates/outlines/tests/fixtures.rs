use ast_grep_language::{Language, SupportLang};
use outlines::{render_file_outline, render_outline};
use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;

#[test]
fn fixture_outputs_match_expected() {
    let fixtures = collect_source_fixtures(&fixture_root());
    assert!(!fixtures.is_empty(), "no outline fixtures found");

    let mut failures = Vec::new();
    for fixture in fixtures {
        let lang = SupportLang::from_path(&fixture)
            .unwrap_or_else(|| panic!("failed to infer language for {}", fixture.display()));
        let source = fs::read_to_string(&fixture)
            .unwrap_or_else(|err| panic!("failed to read {}: {err}", fixture.display()));
        let actual = render_outline(&source, lang);
        let expected_path = fixture.with_extension("outline");
        let expected = fs::read_to_string(&expected_path)
            .unwrap_or_else(|err| panic!("failed to read {}: {err}", expected_path.display()));
        if actual != expected {
            failures.push(format!(
                "fixture mismatch: {}\n--- expected ---\n{}--- actual ---\n{}",
                fixture.display(), expected, actual
            ));
        }
    }

    assert!(failures.is_empty(), "{}", failures.join("\n\n"));
}

#[test]
fn file_outline_includes_header() {
    let fixture = fixture_root().join("rust/simple.rs");
    let output = render_file_outline(&fixture, None).expect("fixture should render");
    let expected_body = fs::read_to_string(fixture.with_extension("outline"))
        .expect("expected outline should exist");
    let expected = format!("{} (Rust)\n{}", fixture.display(), expected_body);
    assert_eq!(output, expected);
}

#[test]
fn cli_supports_multiple_files() {
    let rust_fixture = fixture_root().join("rust/simple.rs");
    let go_fixture = fixture_root().join("go/simple.go");
    let output = Command::new(env!("CARGO_BIN_EXE_outlines"))
        .arg(&rust_fixture)
        .arg(&go_fixture)
        .output()
        .expect("binary should run");
    assert!(output.status.success(), "stderr: {}", String::from_utf8_lossy(&output.stderr));

    let expected_rust = render_file_outline(&rust_fixture, None).expect("rust fixture should render");
    let expected_go = render_file_outline(&go_fixture, None).expect("go fixture should render");
    let expected = format!("{expected_rust}\n\n{expected_go}");
    assert_eq!(String::from_utf8_lossy(&output.stdout), expected);
}

#[test]
fn cli_expands_directories() {
    let go_dir = fixture_root().join("go");
    let output = Command::new(env!("CARGO_BIN_EXE_outlines"))
        .arg(&go_dir)
        .output()
        .expect("binary should run");
    assert!(output.status.success(), "stderr: {}", String::from_utf8_lossy(&output.stderr));

    let mut files = collect_source_fixtures(&go_dir);
    files.sort();
    let expected = files
        .iter()
        .map(|path| render_file_outline(path, None).expect("fixture should render"))
        .collect::<Vec<_>>()
        .join("\n\n");
    assert_eq!(String::from_utf8_lossy(&output.stdout), expected);
}

fn fixture_root() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("testdata")
}

fn collect_source_fixtures(root: &Path) -> Vec<PathBuf> {
    let mut fixtures = Vec::new();
    collect_source_fixtures_impl(root, &mut fixtures);
    fixtures.sort();
    fixtures
}

fn collect_source_fixtures_impl(root: &Path, fixtures: &mut Vec<PathBuf>) {
    for entry in fs::read_dir(root).unwrap_or_else(|err| panic!("failed to read {}: {err}", root.display())) {
        let entry = entry.expect("directory entry should be readable");
        let path = entry.path();
        if path.is_dir() {
            collect_source_fixtures_impl(&path, fixtures);
            continue;
        }
        if path.extension().and_then(|ext| ext.to_str()) == Some("outline") {
            continue;
        }
        fixtures.push(path);
    }
}
