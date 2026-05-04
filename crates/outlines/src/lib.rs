use ast_grep_core::Node;
use ast_grep_language::{Language, LanguageExt, SupportLang};
use std::fs;
use std::path::Path;

pub fn render_file_outline(path: &Path, lang: Option<SupportLang>) -> Result<String, String> {
    let lang = match lang {
        Some(lang) => lang,
        None => SupportLang::from_path(path)
            .ok_or_else(|| format!("cannot infer language from {}", path.display()))?,
    };
    let source = fs::read_to_string(path)
        .map_err(|err| format!("failed to read {}: {err}", path.display()))?;
    let body = render_outline(&source, lang);
    Ok(format!("{} ({lang})\n{body}", path.display()))
}

pub fn render_outline(source: &str, lang: SupportLang) -> String {
    let root = lang.ast_grep(source);
    let items = collect_outline(root.root());
    if items.is_empty() {
        return "<no outline nodes matched the current heuristic>".to_string();
    }

    let mut output = String::new();
    for item in &items {
        write_item(item, 0, &mut output);
    }
    output
}

#[derive(Debug, PartialEq, Eq)]
struct OutlineItem {
    kind: String,
    name: Option<String>,
    start_line: usize,
    end_line: usize,
    children: Vec<OutlineItem>,
}

fn collect_outline(node: Node<'_, ast_grep_core::tree_sitter::StrDoc<SupportLang>>) -> Vec<OutlineItem> {
    let mut items = Vec::new();
    for child in node.children() {
        let descendants = collect_outline(child.clone());
        if is_outline_node(&child) {
            let start = child.start_pos();
            let end = child.end_pos();
            items.push(OutlineItem {
                kind: outline_kind(&child),
                name: node_name(&child),
                start_line: start.line() + 1,
                end_line: end.line() + 1,
                children: descendants,
            });
        } else {
            items.extend(descendants);
        }
    }
    items
}

fn is_outline_node(node: &Node<'_, ast_grep_core::tree_sitter::StrDoc<SupportLang>>) -> bool {
    if declares_closure(node) {
        return true;
    }

    let kind = node.kind();
    let kind = kind.as_ref();
    if matches!(
        kind,
        "use_declaration"
            | "import_declaration"
            | "let_declaration"
            | "attribute_item"
            | "parameter_declaration"
            | "short_var_declaration"
            | "field_declaration"
    ) {
        return false;
    }
    if matches!(kind, "lexical_declaration" | "variable_declaration" | "var_declaration") {
        return node
            .parent()
            .map(|parent| is_rootish(parent.kind().as_ref()))
            .unwrap_or(false);
    }
    kind.ends_with("_declaration")
        || kind.ends_with("_definition")
        || kind.ends_with("_item")
        || matches!(kind, "macro_rule" | "field_definition" | "public_field_definition")
}

fn outline_kind(node: &Node<'_, ast_grep_core::tree_sitter::StrDoc<SupportLang>>) -> String {
    if declares_closure(node) {
        return "closure_declaration".to_string();
    }

    if node.kind().as_ref() == "type_declaration" {
        if let Some(kind) = go_type_declaration_kind(node) {
            return kind.to_string();
        }
    }

    node.kind().into_owned()
}

fn go_type_declaration_kind(
    node: &Node<'_, ast_grep_core::tree_sitter::StrDoc<SupportLang>>,
) -> Option<&'static str> {
    let type_spec = node.children().find(|child| child.kind().as_ref() == "type_spec")?;
    let ty = type_spec.field("type")?;
    match ty.kind().as_ref() {
        "interface_type" => Some("interface_declaration"),
        "struct_type" => Some("struct_declaration"),
        "function_type" => Some("function_type_declaration"),
        _ => None,
    }
}

fn declares_closure(node: &Node<'_, ast_grep_core::tree_sitter::StrDoc<SupportLang>>) -> bool {
    match node.kind().as_ref() {
        "short_var_declaration" => node
            .field("right")
            .map(|right| expression_has_func_literal(&right))
            .unwrap_or(false),
        "var_declaration" => node.children().any(|child| {
            child.kind().as_ref() == "var_spec"
                && child
                    .field("value")
                    .map(|value| expression_has_func_literal(&value))
                    .unwrap_or(false)
        }),
        _ => false,
    }
}

fn expression_has_func_literal(node: &Node<'_, ast_grep_core::tree_sitter::StrDoc<SupportLang>>) -> bool {
    node.children().any(|child| child.kind().as_ref() == "func_literal")
}

fn is_rootish(kind: &str) -> bool {
    matches!(kind, "program" | "module" | "source_file" | "translation_unit")
}

fn node_name(node: &Node<'_, ast_grep_core::tree_sitter::StrDoc<SupportLang>>) -> Option<String> {
    if node.kind().as_ref() == "method_declaration" {
        if let Some(name) = method_declaration_name(node) {
            return Some(name);
        }
    }

    for field in ["name", "declarator", "label", "type", "trait", "key"] {
        if let Some(name) = node.field(field).and_then(extract_name) {
            return Some(name);
        }
    }

    for child in node.children().filter(|child| child.is_named()).take(6) {
        if let Some(name) = extract_name(child) {
            return Some(name);
        }
    }
    None
}

fn method_declaration_name(
    node: &Node<'_, ast_grep_core::tree_sitter::StrDoc<SupportLang>>,
) -> Option<String> {
    let method = node.field("name").and_then(extract_name)?;
    let receiver = node.field("receiver").and_then(receiver_type_name);
    match receiver {
        Some(receiver) => Some(format!("{receiver}.{method}")),
        None => Some(method),
    }
}

fn receiver_type_name(
    node: Node<'_, ast_grep_core::tree_sitter::StrDoc<SupportLang>>,
) -> Option<String> {
    node.children()
        .find(|child| child.kind().as_ref() == "parameter_declaration")
        .and_then(|param| param.field("type").and_then(extract_name).or_else(|| extract_name(param)))
}

fn extract_name(node: Node<'_, ast_grep_core::tree_sitter::StrDoc<SupportLang>>) -> Option<String> {
    if is_identifier_kind(node.kind().as_ref()) {
        return Some(compact(node.text().as_ref()));
    }

    for field in ["name", "declarator", "label", "type", "trait", "key"] {
        if let Some(name) = node.field(field).and_then(extract_name) {
            return Some(name);
        }
    }

    for child in node.children().filter(|child| child.is_named()).take(6) {
        if is_identifier_kind(child.kind().as_ref()) {
            return Some(compact(child.text().as_ref()));
        }
        if child.kind().contains("declarator") {
            if let Some(name) = extract_name(child) {
                return Some(name);
            }
        }
    }
    None
}

fn is_identifier_kind(kind: &str) -> bool {
    matches!(
        kind,
        "identifier"
            | "type_identifier"
            | "property_identifier"
            | "field_identifier"
            | "shorthand_property_identifier"
            | "shorthand_property_identifier_pattern"
            | "simple_identifier"
            | "constant"
    ) || kind.contains("identifier")
}

fn compact(text: &str) -> String {
    let collapsed = text.split_whitespace().collect::<Vec<_>>().join(" ");
    if collapsed.len() > 60 {
        format!("{}...", &collapsed[..57])
    } else {
        collapsed
    }
}

fn write_item(item: &OutlineItem, depth: usize, output: &mut String) {
    let indent = "  ".repeat(depth);
    let range = if item.start_line == item.end_line {
        item.start_line.to_string()
    } else {
        format!("{}-{}", item.start_line, item.end_line)
    };
    let line = match &item.name {
        Some(name) => format!("{indent}- {} {} @{range}\n", item.kind, name),
        None => format!("{indent}- {} @{range}\n", item.kind),
    };
    output.push_str(&line);
    for child in &item.children {
        write_item(child, depth + 1, output);
    }
}
