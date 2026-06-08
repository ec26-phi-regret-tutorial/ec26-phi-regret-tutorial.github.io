use regex::{Captures, Regex};
use std::sync::OnceLock;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum MathMode {
    Svg,
    Katex,
}

impl MathMode {
    pub fn parse(value: &str) -> Result<Self, String> {
        match value {
            "svg" => Ok(Self::Svg),
            "katex" => Ok(Self::Katex),
            _ => Err(format!(
                "unknown math mode `{value}`; expected `svg` or `katex`"
            )),
        }
    }

    pub fn as_typst_input(self) -> &'static str {
        match self {
            Self::Svg => "svg",
            Self::Katex => "katex",
        }
    }
}

pub fn katex_head_assets(mode: MathMode) -> &'static str {
    match mode {
        MathMode::Svg => "",
        MathMode::Katex => {
            r#"  <link rel="stylesheet" href="https://cdn.jsdelivr.net/npm/katex@0.16.22/dist/katex.min.css">
"#
        }
    }
}

pub fn katex_script_assets(mode: MathMode) -> &'static str {
    match mode {
        MathMode::Svg => "",
        MathMode::Katex => {
            r#"  <script defer src="https://cdn.jsdelivr.net/npm/katex@0.16.22/dist/katex.min.js"></script>
  <script defer src="https://cdn.jsdelivr.net/npm/katex@0.16.22/dist/contrib/auto-render.min.js" onload="renderMathInElement(document.body,{delimiters:[{left:'\\[',right:'\\]',display:true},{left:'\\(',right:'\\)',display:false}],throwOnError:false,strict:'warn'});requestAnimationFrame(window.markOverwideEquations || function(){});"></script>
"#
        }
    }
}

pub fn postprocess_html_math(body: String, mode: MathMode) -> String {
    match mode {
        MathMode::Svg => body,
        MathMode::Katex => render_katex_sources(body),
    }
}

pub fn normalize_bibliography_math(input: &str) -> String {
    let with_math_spans = re_tex_math_span()
        .replace_all(input, |captures: &Captures| {
            let raw = captures.get(1).map_or("", |m| m.as_str());
            format!(
                "<span class=\"bib-math\">{}</span>",
                escape_html(&normalize_tex_math_fragment(raw))
            )
        })
        .to_string();
    replace_tex_macros(&with_math_spans)
}

fn render_katex_sources(input: String) -> String {
    re_math_data_span()
        .replace_all(&input, |captures: &Captures| {
            let attrs = captures.name("attrs").map_or("", |m| m.as_str());
            let typst_repr = attr_value(attrs, "data-typst-math").unwrap_or_default();
            let explicit_katex = attr_value(attrs, "data-katex");
            let display = attr_value(attrs, "data-math-display")
                .as_deref()
                .map(|value| value == "block")
                .unwrap_or(false);
            let display_context = display || is_equation_math_attrs(attrs);
            let mut tex = explicit_katex.unwrap_or_else(|| typst_repr_to_katex(&typst_repr));
            if tex.trim().is_empty() && !is_intentionally_empty_math_repr(&typst_repr) {
                let label = first_call_name(&typst_repr).unwrap_or("unknown");
                eprintln!("warning: unsupported Typst math was not converted to KaTeX: {label}");
                tex = format!("\\text{{[unsupported math: {label}]}}");
            }
            if display_context {
                tex = prepare_display_tex(&tex);
            }
            let trimmed = tex.trim();
            let source = if trimmed.is_empty() {
                String::new()
            } else if display {
                format!("\\[{}\\]", trimmed)
            } else {
                format!("\\({}\\)", trimmed)
            };
            format!(
                "<span{}>{}</span>",
                add_class_to_attrs(attrs, "math-katex-source"),
                escape_html(&source)
            )
        })
        .to_string()
}

pub fn typst_repr_to_katex(input: &str) -> String {
    normalize_operator_phrases(&normalize_tex_spaces(&convert_expr(input.trim())))
}

fn convert_expr(input: &str) -> String {
    let input = input.trim();
    if input.is_empty() || input == ".." || input == "none" {
        return String::new();
    }
    if let Some(literal) = bracket_literal(input) {
        return literal_to_tex(literal);
    }
    if let Some(quoted) = quoted_literal(input) {
        return text_literal_to_tex(quoted);
    }
    if let Some((name, inner)) = call_parts(input) {
        return convert_call(name, inner);
    }
    literal_to_tex(input)
}

fn prepare_display_tex(input: &str) -> String {
    let sized = autosize_display_delimiters(input.trim());
    if sized.trim().is_empty() || sized.trim_start().starts_with("\\displaystyle") {
        sized
    } else {
        format!("\\displaystyle {sized}")
    }
}

fn convert_call(name: &str, inner: &str) -> String {
    let args = parse_args(inner);
    match name {
        "sequence" => args
            .iter()
            .filter(|arg| arg.name.is_none())
            .map(|arg| convert_expr(&arg.value))
            .collect::<String>(),
        "equation" => named_arg(&args, "body")
            .map(convert_expr)
            .unwrap_or_default(),
        "styled" => named_arg(&args, "child")
            .map(convert_styled_child)
            .unwrap_or_default(),
        "lr" => named_arg(&args, "body")
            .map(convert_expr)
            .unwrap_or_default(),
        "attach" => convert_attach(&args),
        "op" => convert_operator(&args),
        "mat" => convert_matrix(&args),
        "cases" => convert_cases(&args),
        "primes" => convert_primes(&args),
        "frac" => {
            let num = named_arg(&args, "num")
                .or_else(|| positional_arg(&args, 0))
                .map(convert_expr)
                .unwrap_or_default();
            let denom = named_arg(&args, "denom")
                .or_else(|| named_arg(&args, "den"))
                .or_else(|| positional_arg(&args, 1))
                .map(convert_expr)
                .unwrap_or_default();
            format!("\\frac{{{}}}{{{}}}", num.trim(), denom.trim())
        }
        "sqrt" => {
            let body = named_arg(&args, "body")
                .or_else(|| positional_arg(&args, 0))
                .map(convert_expr)
                .unwrap_or_default();
            format!("\\sqrt{{{}}}", body.trim())
        }
        "root" => {
            let index = named_arg(&args, "index")
                .or_else(|| positional_arg(&args, 0))
                .map(convert_expr)
                .unwrap_or_default();
            let body = named_arg(&args, "radicand")
                .or_else(|| named_arg(&args, "body"))
                .or_else(|| positional_arg(&args, 1))
                .map(convert_expr)
                .unwrap_or_default();
            let index = index.trim();
            let body = body.trim();
            if index.is_empty() {
                format!("\\sqrt{{{body}}}")
            } else {
                format!("\\sqrt[{index}]{{{body}}}")
            }
        }
        "vec" => accent_command("vec", &args),
        "hat" => accent_command("hat", &args),
        "tilde" => accent_command("tilde", &args),
        "dot" => accent_command("dot", &args),
        "overline" => accent_command("overline", &args),
        "underline" => accent_command("underline", &args),
        "h" => convert_horizontal_space(&args),
        "linebreak" => "\\\\".to_owned(),
        "align-point" => "&".to_owned(),
        _ => args
            .iter()
            .find_map(|arg| {
                if matches!(arg.name.as_deref(), Some("body" | "child" | "text")) {
                    Some(convert_expr(&arg.value))
                } else {
                    None
                }
            })
            .unwrap_or_default(),
    }
}

fn convert_attach(args: &[Arg]) -> String {
    let base = named_arg(args, "base")
        .map(convert_expr)
        .unwrap_or_default();
    let sub = named_arg(args, "b").map(convert_expr);
    let sup = named_arg(args, "t").map(convert_expr);
    let top_right = named_arg(args, "tr").map(convert_expr);
    let has_attached_script = sub.as_deref().is_some_and(|value| !value.trim().is_empty())
        || sup.as_deref().is_some_and(|value| !value.trim().is_empty())
        || named_arg(args, "tr").is_some();
    let mut out = if has_attached_script && !base.trim().is_empty() {
        base.trim_end().to_owned()
    } else {
        base
    };
    if let Some(sub) = sub {
        let sub = simplify_script(sub.trim());
        if !sub.is_empty() {
            out.push_str(&format!("_{{{sub}}}"));
        }
    }
    if let Some(sup) = sup {
        let sup = simplify_script(sup.trim());
        if !sup.is_empty() {
            out.push_str(&format!("^{{{sup}}}"));
        }
    }
    if let Some(top_right) = top_right {
        if !top_right.trim().is_empty() {
            out.push_str(top_right.trim());
        }
    }
    out
}

fn simplify_script(input: &str) -> &str {
    input
        .strip_prefix("\\{")
        .and_then(|value| value.strip_suffix("\\}"))
        .filter(|inner| tex_text_wrapper_inner(inner.trim()).is_some())
        .map(str::trim)
        .unwrap_or(input)
}

fn convert_matrix(args: &[Arg]) -> String {
    let rows = named_arg(args, "rows")
        .map(parse_matrix_rows)
        .unwrap_or_default();
    if rows.is_empty() {
        return String::new();
    }

    let environment = match named_arg(args, "delim").map(str::trim) {
        Some("none") => "matrix",
        Some("[") | Some("bracket") => "bmatrix",
        Some("{") | Some("brace") => "Bmatrix",
        Some("|") | Some("bar") => "vmatrix",
        Some("‖") | Some("double-bar") => "Vmatrix",
        _ => "pmatrix",
    };
    let body = rows
        .into_iter()
        .map(|row| {
            row.into_iter()
                .map(|cell| convert_expr(cell.trim()))
                .collect::<Vec<_>>()
                .join(" & ")
        })
        .collect::<Vec<_>>()
        .join(r" \\ ");
    format!("\\begin{{{environment}}}{body}\\end{{{environment}}}")
}

fn convert_cases(args: &[Arg]) -> String {
    let rows = named_arg(args, "children")
        .map(tuple_items)
        .unwrap_or_default();
    if rows.is_empty() {
        return String::new();
    }

    let body = rows
        .into_iter()
        .map(convert_case_row)
        .filter(|row| !row.trim().is_empty())
        .collect::<Vec<_>>()
        .join(r" \\ ");
    if body.is_empty() {
        String::new()
    } else {
        format!("\\begin{{cases}}{body}\\end{{cases}}")
    }
}

fn convert_case_row(input: &str) -> String {
    let converted = convert_expr(input);
    if let Some(idx) = converted.find('&') {
        let left = converted[..idx].trim();
        let right = normalize_case_condition_spacing(converted[idx + '&'.len_utf8()..].trim());
        if right.is_empty() {
            left.to_owned()
        } else {
            format!("{left} & {right}")
        }
    } else {
        converted.trim().to_owned()
    }
}

fn normalize_case_condition_spacing(input: &str) -> String {
    input.replace("\\text{if} ", "\\text{if } ")
}

fn parse_matrix_rows(input: &str) -> Vec<Vec<&str>> {
    tuple_items(input)
        .into_iter()
        .map(tuple_items)
        .filter(|row| !row.is_empty())
        .collect()
}

fn convert_primes(args: &[Arg]) -> String {
    let count = named_arg(args, "count")
        .or_else(|| positional_arg(args, 0))
        .and_then(|value| value.trim().parse::<usize>().ok())
        .unwrap_or(1);
    "'".repeat(count)
}

fn convert_operator(args: &[Arg]) -> String {
    let text = named_arg(args, "text")
        .or_else(|| positional_arg(args, 0))
        .map(convert_expr)
        .unwrap_or_default();
    let limits = named_arg(args, "limits")
        .map(|value| value.trim() == "true")
        .unwrap_or(false);
    let stripped = strip_tex_text_wrappers(text.trim());
    if stripped == "E" {
        if limits {
            "\\mathop{\\mathbb{E}}\\limits".to_owned()
        } else {
            "\\mathbb{E}".to_owned()
        }
    } else if stripped
        .chars()
        .all(|ch| ch.is_ascii_alphabetic() || ch == '-')
    {
        let operator = stripped.replace('-', "\\text{-}");
        if limits {
            format!("\\operatorname*{{{operator}}}")
        } else {
            format!("\\operatorname{{{operator}}}")
        }
    } else if limits {
        format!("\\mathop{{{text}}}\\limits")
    } else {
        text
    }
}

fn convert_styled_child(input: &str) -> String {
    let converted = convert_expr(input);
    let stripped = strip_tex_text_wrappers(converted.trim());
    if let Some(styled) = styled_symbol(stripped) {
        return styled;
    }
    if let Some((base, suffix)) = styled_symbol_with_suffix(stripped) {
        if let Some(styled) = styled_symbol(base) {
            return format!("{styled}{suffix}");
        }
    }
    converted
}

fn styled_symbol(input: &str) -> Option<String> {
    match input {
        "A" | "C" | "H" | "K" | "S" | "U" | "X" | "Y" => Some(format!("\\mathcal{{{input}}}")),
        "R" | "N" | "Q" | "B" => Some(format!("\\mathbb{{{input}}}")),
        "a" | "b" | "c" | "p" | "q" | "s" | "u" | "x" | "y" | "z" => {
            Some(format!("\\boldsymbol{{{input}}}"))
        }
        "I" | "M" => Some(format!("\\mathbf{{{input}}}")),
        _ => None,
    }
}

fn styled_symbol_with_suffix(input: &str) -> Option<(&str, &str)> {
    let mut chars = input.char_indices();
    let (_, first) = chars.next()?;
    if !first.is_ascii_alphabetic() {
        return None;
    }
    let suffix_start = chars.next().map(|(idx, _)| idx)?;
    let suffix = &input[suffix_start..];
    if suffix.starts_with('_') || suffix.starts_with('^') || suffix.starts_with('\'') {
        Some((&input[..suffix_start], suffix))
    } else {
        None
    }
}

fn accent_command(name: &str, args: &[Arg]) -> String {
    let body = named_arg(args, "body")
        .or_else(|| named_arg(args, "base"))
        .or_else(|| positional_arg(args, 0))
        .map(convert_expr)
        .unwrap_or_default();
    format!("\\{name}{{{}}}", body.trim())
}

fn convert_horizontal_space(args: &[Arg]) -> String {
    let amount = named_arg(args, "amount")
        .or_else(|| positional_arg(args, 0))
        .unwrap_or("")
        .trim();
    match amount {
        "1em" => "\\quad ".to_owned(),
        "2em" => "\\qquad ".to_owned(),
        "" => String::new(),
        value => format!("\\hspace{{{value}}}"),
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
enum DelimKind {
    Paren,
    Bracket,
    Brace,
    Angle,
    Ceil,
    Floor,
    Vert,
}

#[derive(Clone, Copy, Debug)]
enum DelimRole {
    Open(DelimKind),
    Close(DelimKind),
    Symmetric(DelimKind),
}

#[derive(Debug)]
struct TexToken {
    text: String,
    role: Option<DelimRole>,
    protected: bool,
}

fn autosize_display_delimiters(input: &str) -> String {
    let mut tokens = tokenize_tex(input);
    let mut stack: Vec<(DelimKind, usize)> = Vec::new();
    let mut prefixes: Vec<Option<&'static str>> = vec![None; tokens.len()];

    for (index, token) in tokens.iter().enumerate() {
        if token.protected {
            continue;
        }
        match token.role {
            Some(DelimRole::Open(kind)) => stack.push((kind, index)),
            Some(DelimRole::Close(kind)) => {
                if let Some(pos) = stack.iter().rposition(|(open, _)| *open == kind) {
                    let (_, open_index) = stack.remove(pos);
                    prefixes[open_index] = Some("\\left");
                    prefixes[index] = Some("\\right");
                }
            }
            Some(DelimRole::Symmetric(kind)) => {
                if let Some(pos) = stack.iter().rposition(|(open, _)| *open == kind) {
                    let (_, open_index) = stack.remove(pos);
                    prefixes[open_index] = Some("\\left");
                    prefixes[index] = Some("\\right");
                } else {
                    stack.push((kind, index));
                }
            }
            None => {}
        }
    }

    let mut out = String::new();
    for (index, token) in tokens.drain(..).enumerate() {
        if let Some(prefix) = prefixes[index] {
            out.push_str(prefix);
        }
        out.push_str(&token.text);
    }
    out
}

fn tokenize_tex(input: &str) -> Vec<TexToken> {
    let mut tokens = Vec::new();
    let mut index = 0usize;
    let mut protect_next_delimiter = false;
    let mut protect_optional_square_depth = 0usize;
    let mut protect_next_optional_square = false;

    while index < input.len() {
        let rest = &input[index..];
        let ch = rest.chars().next().unwrap();
        if ch == '\\' {
            let (command, next) = read_tex_command(input, index);
            index = next;
            let role = command_delim_role(command);
            let mut protected = protect_next_delimiter || protect_optional_square_depth > 0;
            if matches!(command, "\\left" | "\\right") {
                protect_next_delimiter = true;
            } else {
                protect_next_delimiter = false;
            }
            if command == "\\sqrt" {
                protect_next_optional_square = true;
            }
            if role.is_some() && protect_next_delimiter {
                protected = true;
            }
            tokens.push(TexToken {
                text: command.to_owned(),
                role,
                protected,
            });
            continue;
        }

        let mut protected = protect_next_delimiter || protect_optional_square_depth > 0;
        protect_next_delimiter = false;
        let role = char_delim_role(ch);
        if protect_next_optional_square && !ch.is_whitespace() {
            if ch == '[' {
                protected = true;
                protect_optional_square_depth = 1;
            }
            protect_next_optional_square = false;
        } else if protect_optional_square_depth > 0 {
            if ch == '[' {
                protect_optional_square_depth += 1;
            } else if ch == ']' {
                protect_optional_square_depth = protect_optional_square_depth.saturating_sub(1);
            }
        }

        tokens.push(TexToken {
            text: ch.to_string(),
            role,
            protected,
        });
        index += ch.len_utf8();
    }

    tokens
}

fn read_tex_command(input: &str, start: usize) -> (&str, usize) {
    let after_slash = start + 1;
    let mut end = after_slash;
    for (offset, ch) in input[after_slash..].char_indices() {
        if ch.is_ascii_alphabetic() {
            end = after_slash + offset + ch.len_utf8();
        } else {
            break;
        }
    }
    if end == after_slash {
        let ch = input[after_slash..].chars().next();
        end = ch.map_or(after_slash, |ch| after_slash + ch.len_utf8());
    }
    (&input[start..end], end)
}

fn command_delim_role(command: &str) -> Option<DelimRole> {
    match command {
        "\\{" => Some(DelimRole::Open(DelimKind::Brace)),
        "\\}" => Some(DelimRole::Close(DelimKind::Brace)),
        "\\langle" => Some(DelimRole::Open(DelimKind::Angle)),
        "\\rangle" => Some(DelimRole::Close(DelimKind::Angle)),
        "\\lceil" => Some(DelimRole::Open(DelimKind::Ceil)),
        "\\rceil" => Some(DelimRole::Close(DelimKind::Ceil)),
        "\\lfloor" => Some(DelimRole::Open(DelimKind::Floor)),
        "\\rfloor" => Some(DelimRole::Close(DelimKind::Floor)),
        "\\lVert" => Some(DelimRole::Open(DelimKind::Vert)),
        "\\rVert" => Some(DelimRole::Close(DelimKind::Vert)),
        "\\Vert" | "\\|" => Some(DelimRole::Symmetric(DelimKind::Vert)),
        _ => None,
    }
}

fn char_delim_role(ch: char) -> Option<DelimRole> {
    match ch {
        '(' => Some(DelimRole::Open(DelimKind::Paren)),
        ')' => Some(DelimRole::Close(DelimKind::Paren)),
        '[' => Some(DelimRole::Open(DelimKind::Bracket)),
        ']' => Some(DelimRole::Close(DelimKind::Bracket)),
        '|' => Some(DelimRole::Symmetric(DelimKind::Vert)),
        _ => None,
    }
}

#[derive(Debug)]
struct Arg {
    name: Option<String>,
    value: String,
}

fn parse_args(input: &str) -> Vec<Arg> {
    split_top_level(input, ',')
        .into_iter()
        .filter_map(|raw| {
            let value = raw.trim();
            if value.is_empty() || value == ".." {
                return None;
            }
            if let Some(colon) = find_top_level(value, ':') {
                Some(Arg {
                    name: Some(value[..colon].trim().to_owned()),
                    value: value[colon + 1..].trim().to_owned(),
                })
            } else {
                Some(Arg {
                    name: None,
                    value: value.to_owned(),
                })
            }
        })
        .collect()
}

fn named_arg<'a>(args: &'a [Arg], name: &str) -> Option<&'a str> {
    args.iter()
        .find(|arg| arg.name.as_deref() == Some(name))
        .map(|arg| arg.value.as_str())
}

fn positional_arg(args: &[Arg], index: usize) -> Option<&str> {
    args.iter()
        .filter(|arg| arg.name.is_none())
        .nth(index)
        .map(|arg| arg.value.as_str())
}

fn tuple_items(input: &str) -> Vec<&str> {
    let value = input.trim();
    let inner = value
        .strip_prefix('(')
        .and_then(|value| value.strip_suffix(')'))
        .unwrap_or(value);
    split_top_level(inner, ',')
        .into_iter()
        .map(str::trim)
        .filter(|item| !item.is_empty() && *item != "..")
        .collect()
}

fn call_parts(input: &str) -> Option<(&str, &str)> {
    let open = input.find('(')?;
    if !input.ends_with(')') {
        return None;
    }
    let name = input[..open].trim();
    if name.is_empty()
        || !name
            .chars()
            .all(|ch| ch.is_ascii_alphanumeric() || ch == '-' || ch == '_')
    {
        return None;
    }
    Some((name, &input[open + 1..input.len() - 1]))
}

fn bracket_literal(input: &str) -> Option<&str> {
    input.strip_prefix('[')?.strip_suffix(']')
}

fn quoted_literal(input: &str) -> Option<&str> {
    input.strip_prefix('"')?.strip_suffix('"')
}

fn split_top_level(input: &str, delimiter: char) -> Vec<&str> {
    let mut parts = Vec::new();
    let mut start = 0usize;
    let mut paren_depth = 0usize;
    let mut bracket_depth = 0usize;
    let mut in_string = false;
    let mut escaped = false;
    for (idx, ch) in input.char_indices() {
        if in_string {
            if escaped {
                escaped = false;
            } else if ch == '\\' {
                escaped = true;
            } else if ch == '"' {
                in_string = false;
            }
            continue;
        }
        if bracket_depth > 0 {
            if ch == ']' {
                bracket_depth = bracket_depth.saturating_sub(1);
            }
            continue;
        }
        match ch {
            '"' => in_string = true,
            '(' => paren_depth += 1,
            ')' => paren_depth = paren_depth.saturating_sub(1),
            '[' => bracket_depth += 1,
            ']' => bracket_depth = bracket_depth.saturating_sub(1),
            _ if ch == delimiter && paren_depth == 0 && bracket_depth == 0 => {
                parts.push(&input[start..idx]);
                start = idx + ch.len_utf8();
            }
            _ => {}
        }
    }
    parts.push(&input[start..]);
    parts
}

fn find_top_level(input: &str, needle: char) -> Option<usize> {
    let mut paren_depth = 0usize;
    let mut bracket_depth = 0usize;
    let mut in_string = false;
    let mut escaped = false;
    for (idx, ch) in input.char_indices() {
        if in_string {
            if escaped {
                escaped = false;
            } else if ch == '\\' {
                escaped = true;
            } else if ch == '"' {
                in_string = false;
            }
            continue;
        }
        if bracket_depth > 0 {
            if ch == ']' {
                bracket_depth = bracket_depth.saturating_sub(1);
            }
            continue;
        }
        match ch {
            '"' => in_string = true,
            '(' => paren_depth += 1,
            ')' => paren_depth = paren_depth.saturating_sub(1),
            '[' => bracket_depth += 1,
            ']' => bracket_depth = bracket_depth.saturating_sub(1),
            _ if ch == needle && paren_depth == 0 && bracket_depth == 0 => return Some(idx),
            _ => {}
        }
    }
    None
}

fn literal_to_tex(input: &str) -> String {
    if input.trim().is_empty() {
        return input.to_owned();
    }
    if input.chars().any(|ch| ch.is_ascii_alphabetic())
        && (input.chars().count() > 1 || input.contains('-'))
    {
        return text_literal_to_tex(input);
    }
    input.chars().map(char_to_tex).collect()
}

fn text_literal_to_tex(input: &str) -> String {
    if input.chars().count() == 1 {
        return literal_to_tex(input);
    }
    format!("\\text{{{}}}", escape_tex_text(input))
}

fn escape_tex_text(input: &str) -> String {
    input
        .chars()
        .map(|ch| match ch {
            '\\' => "\\textbackslash{}".to_owned(),
            '&' | '%' | '$' | '#' | '_' | '{' | '}' => format!("\\{ch}"),
            _ => ch.to_string(),
        })
        .collect()
}

fn char_to_tex(ch: char) -> String {
    match ch {
        'Φ' => tex_command("Phi"),
        'φ' => tex_command("phi"),
        'ε' => tex_command("epsilon"),
        'ϵ' => tex_command("epsilon"),
        'α' => tex_command("alpha"),
        'β' => tex_command("beta"),
        'γ' => tex_command("gamma"),
        'Γ' => tex_command("Gamma"),
        'δ' => tex_command("delta"),
        'Δ' => tex_command("Delta"),
        'λ' => tex_command("lambda"),
        'μ' => tex_command("mu"),
        'σ' => tex_command("sigma"),
        'Ω' => tex_command("Omega"),
        'Θ' => tex_command("Theta"),
        '𝔹' => "\\mathbb{B}".to_owned(),
        'ℝ' => "\\mathbb{R}".to_owned(),
        'ℕ' => "\\mathbb{N}".to_owned(),
        'ℚ' => "\\mathbb{Q}".to_owned(),
        'ℂ' => "\\mathbb{C}".to_owned(),
        '𝔼' => "\\mathbb{E}".to_owned(),
        '𝓐' => "\\mathcal{A}".to_owned(),
        '𝓒' => "\\mathcal{C}".to_owned(),
        '𝓗' => "\\mathcal{H}".to_owned(),
        '𝓚' => "\\mathcal{K}".to_owned(),
        '𝓢' => "\\mathcal{S}".to_owned(),
        '𝓤' => "\\mathcal{U}".to_owned(),
        '𝓧' => "\\mathcal{X}".to_owned(),
        '𝓨' => "\\mathcal{Y}".to_owned(),
        '𝐀' => "\\mathbf{A}".to_owned(),
        '𝐈' => "\\mathbf{I}".to_owned(),
        '𝐊' => "\\mathbf{K}".to_owned(),
        '𝐌' => "\\mathbf{M}".to_owned(),
        '𝐔' => "\\mathbf{U}".to_owned(),
        '𝐚' => "\\boldsymbol{a}".to_owned(),
        '𝐛' => "\\boldsymbol{b}".to_owned(),
        '𝐜' => "\\boldsymbol{c}".to_owned(),
        '𝐩' => "\\boldsymbol{p}".to_owned(),
        '𝐪' => "\\boldsymbol{q}".to_owned(),
        '𝐬' => "\\boldsymbol{s}".to_owned(),
        '𝐮' => "\\boldsymbol{u}".to_owned(),
        '𝐱' => "\\boldsymbol{x}".to_owned(),
        '𝐲' => "\\boldsymbol{y}".to_owned(),
        '𝐳' => "\\boldsymbol{z}".to_owned(),
        '≤' => tex_command("le"),
        '≥' => tex_command("ge"),
        '≠' => tex_command("ne"),
        '∈' => tex_command("in"),
        '∉' => tex_command("notin"),
        '⊂' => tex_command("subset"),
        '⊆' => tex_command("subseteq"),
        '×' => tex_command("times"),
        '∀' => tex_command("forall"),
        '∃' => tex_command("exists"),
        '∑' => tex_command("sum"),
        '∏' => tex_command("prod"),
        '∼' => tex_command("sim"),
        '→' => tex_command("to"),
        '↦' => tex_command("mapsto"),
        '⟨' => "\\left\\langle ".to_owned(),
        '⟩' => "\\right\\rangle ".to_owned(),
        '⌈' => "\\lceil ".to_owned(),
        '⌉' => "\\rceil ".to_owned(),
        '⌊' => "\\lfloor ".to_owned(),
        '⌋' => "\\rfloor ".to_owned(),
        '‖' | '∥' => tex_command("Vert"),
        '·' => tex_command("cdot"),
        '…' | '⋯' => tex_command("dots"),
        '≔' => tex_command("coloneqq"),
        '−' => "-".to_owned(),
        ' ' => " ".to_owned(),
        '&' | '%' | '$' | '#' | '_' | '{' | '}' => format!("\\{ch}"),
        _ => ch.to_string(),
    }
}

fn tex_command(name: &str) -> String {
    format!("\\{name} ")
}

fn strip_tex_text_wrappers(input: &str) -> &str {
    tex_text_wrapper_inner(input).unwrap_or(input)
}

fn tex_text_wrapper_inner(input: &str) -> Option<&str> {
    input
        .strip_prefix("\\mathrm{")
        .or_else(|| input.strip_prefix("\\text{"))
        .and_then(|value| value.strip_suffix('}'))
}

fn normalize_tex_spaces(input: &str) -> String {
    let mut out = String::new();
    let mut last_space = false;
    for ch in input.chars() {
        if ch.is_whitespace() {
            if !last_space {
                out.push(' ');
                last_space = true;
            }
        } else {
            out.push(ch);
            last_space = false;
        }
    }
    out.trim().to_owned()
}

fn normalize_operator_phrases(input: &str) -> String {
    input
        .replace(
            "\\operatorname{arg} \\operatorname*{max}",
            "\\operatorname*{arg\\,max}",
        )
        .replace(
            "\\operatorname{arg} \\operatorname*{min}",
            "\\operatorname*{arg\\,min}",
        )
}

fn normalize_tex_math_fragment(input: &str) -> String {
    let trimmed = input.trim();
    let normalized = if trimmed == "Phi" {
        "Φ".to_owned()
    } else {
        replace_tex_macro_text(trimmed)
    };
    normalized.replace("\\/", "/")
}

fn replace_tex_macros(input: &str) -> String {
    let mut out = input.to_owned();
    for (from, to) in TEX_MACROS {
        out = out.replace(
            from,
            &format!("<span class=\"bib-math\">{}</span>", escape_html(to)),
        );
    }
    out
}

fn replace_tex_macro_text(input: &str) -> String {
    let mut out = input.to_owned();
    for (from, to) in TEX_MACROS {
        out = out.replace(from, to);
    }
    out
}

fn attr_value(attrs: &str, name: &str) -> Option<String> {
    let pattern = format!(r#"{name}="([^"]*)""#);
    let re = Regex::new(&pattern).ok()?;
    re.captures(attrs)
        .and_then(|captures| captures.get(1))
        .map(|value| decode_attr(value.as_str()))
}

fn add_class_to_attrs(attrs: &str, class_name: &str) -> String {
    if re_class_attr().is_match(attrs) {
        re_class_attr()
            .replace(attrs, |captures: &Captures| {
                format!(
                    " class=\"{} {}\"",
                    captures.get(1).map_or("", |m| m.as_str()),
                    class_name
                )
            })
            .to_string()
    } else {
        format!(" class=\"{}\"{}", class_name, attrs)
    }
}

fn is_equation_math_attrs(attrs: &str) -> bool {
    let Some(classes) = attr_value(attrs, "class") else {
        return false;
    };
    classes.split_whitespace().any(|class| {
        matches!(
            class,
            "equation-math"
                | "equation-align-left"
                | "equation-align-right"
                | "equation-align-full"
        )
    })
}

fn is_intentionally_empty_math_repr(input: &str) -> bool {
    matches!(input.trim(), "" | "none" | ".." | "[]")
}

fn first_call_name(input: &str) -> Option<&str> {
    let trimmed = input.trim();
    let open = trimmed.find('(')?;
    let name = trimmed[..open].trim();
    if name.is_empty()
        || !name
            .chars()
            .all(|ch| ch.is_ascii_alphanumeric() || ch == '-' || ch == '_')
    {
        None
    } else {
        Some(name)
    }
}

fn decode_attr(input: &str) -> String {
    input
        .replace("&quot;", "\"")
        .replace("&#39;", "'")
        .replace("&lt;", "<")
        .replace("&gt;", ">")
        .replace("&amp;", "&")
}

fn escape_html(input: &str) -> String {
    input
        .replace('&', "&amp;")
        .replace('<', "&lt;")
        .replace('>', "&gt;")
        .replace('"', "&quot;")
}

fn re_math_data_span() -> &'static Regex {
    static RE: OnceLock<Regex> = OnceLock::new();
    RE.get_or_init(|| {
        Regex::new(r#"(?s)<span(?P<attrs>[^>]*\bdata-typst-math="[^"]*"[^>]*)>.*?</span>"#).unwrap()
    })
}

fn re_tex_math_span() -> &'static Regex {
    static RE: OnceLock<Regex> = OnceLock::new();
    RE.get_or_init(|| Regex::new(r#"\$([^$<>]+)\$"#).unwrap())
}

fn re_class_attr() -> &'static Regex {
    static RE: OnceLock<Regex> = OnceLock::new();
    RE.get_or_init(|| Regex::new(r#"\sclass="([^"]*)""#).unwrap())
}

const TEX_MACROS: &[(&str, &str)] = &[
    (r"\Phi", "Φ"),
    (r"\phi", "φ"),
    (r"\epsilon", "ε"),
    (r"\varepsilon", "ε"),
    (r"\Delta", "Δ"),
    (r"\Gamma", "Γ"),
    (r"\Omega", "Ω"),
    (r"\Theta", "Θ"),
    (r"\alpha", "α"),
    (r"\beta", "β"),
    (r"\lambda", "λ"),
    (r"\mu", "μ"),
];

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn converts_simple_sequence_repr_to_katex() {
        let input = "sequence([x], [ ], [+], [ ], [y], [ ], [≤], [ ], [z])";
        assert_eq!(typst_repr_to_katex(input), "x + y \\le z");
    }

    #[test]
    fn converts_attach_repr_to_katex() {
        let input = "attach(base: [h], b: [φ], t: lr(body: sequence([(], [T], [)])))";
        assert_eq!(typst_repr_to_katex(input), "h_{\\phi}^{(T)}");
    }

    #[test]
    fn keeps_parentheses_inside_bracket_literals() {
        let input = "sequence([(], [t], [)])";
        assert_eq!(typst_repr_to_katex(input), "(t)");
    }

    #[test]
    fn keeps_square_brackets_inside_bracket_literals() {
        let input = "sequence([[], [x], []])";
        assert_eq!(typst_repr_to_katex(input), "[x]");
    }

    #[test]
    fn treats_typst_none_as_empty_math() {
        assert_eq!(typst_repr_to_katex("none"), "");
    }

    #[test]
    fn converts_typst_horizontal_space_to_katex_space() {
        let input = "sequence([a], h(amount: 1em), h(amount: 1em), h(amount: 1em), [b], h(amount: .5em), [c])";
        assert_eq!(
            typst_repr_to_katex(input),
            "a\\quad \\quad \\quad b\\hspace{.5em}c"
        );
    }

    #[test]
    fn custom_operator_limits_use_katex_limits_form() {
        let input = "attach(base: op(text: styled(child: equation(block: false, body: [E]), ..), limits: true), b: sequence([x], [ ], [∼], [ ], [D]))";
        assert_eq!(
            typst_repr_to_katex(input),
            "\\mathop{\\mathbb{E}}\\limits_{x \\sim D}"
        );
    }

    #[test]
    fn split_argmax_operator_uses_single_limits_operator() {
        let input = "sequence(op(text: [arg], limits: false), [ ], attach(base: op(text: [max], limits: true), b: sequence([x], [ ], [∈], [ ], equation(block: false, body: styled(child: [X], ..)))))";
        assert_eq!(
            typst_repr_to_katex(input),
            "\\operatorname*{arg\\,max}_{x \\in \\mathcal{X}}"
        );
    }

    #[test]
    fn converts_typst_matrix_rows_to_katex_matrix() {
        let input =
            "mat(rows: ((frac(num: [1], denom: [2]), [0]), ([0], frac(num: [1], denom: [2]))))";
        assert_eq!(
            typst_repr_to_katex(input),
            r"\begin{pmatrix}\frac{1}{2} & 0 \\ 0 & \frac{1}{2}\end{pmatrix}"
        );
    }

    #[test]
    fn converts_typst_cases_to_katex_cases() {
        let input = "sequence([φ], [:], [ ], [a], [ ], [↦], [ ], cases(children: (sequence([2], [ ], align-point(), [ ], [if], [ ], [a], [ ], [=], [ ], [1,]), sequence([1], [ ], align-point(), [ ], [if], [ ], [a], [ ], [=], [ ], [2,]))))";
        assert_eq!(
            typst_repr_to_katex(input),
            r"\phi : a \mapsto \begin{cases}2 & \text{if } a = 1, \\ 1 & \text{if } a = 2,\end{cases}"
        );
    }

    #[test]
    fn text_literals_use_text_mode_to_preserve_phrase_spacing() {
        let input = "sequence([∃], [ ], [ such that ], [ ], [x])";
        assert_eq!(typst_repr_to_katex(input), r"\exists \text{ such that } x");
    }

    #[test]
    fn text_subscripts_drop_literal_group_braces() {
        let input = "attach(base: [Φ], b: lr(body: sequence([{], [const], [}])))";
        assert_eq!(typst_repr_to_katex(input), r"\Phi_{\text{const}}");
    }

    #[test]
    fn preserves_trailing_prime_attachments() {
        let input = "attach(base: [μ], tr: primes(count: 1))";
        assert_eq!(typst_repr_to_katex(input), "\\mu'");
    }

    #[test]
    fn display_math_uses_displaystyle_and_sized_delimiters() {
        let input = r#"<p><span class="equation-math" data-typst-math="sequence([E], [ ], [[], [x], []], [ ], [(], [y], [)])" data-math-display="block"><svg></svg></span></p>"#;
        let out = postprocess_html_math(input.to_owned(), MathMode::Katex);
        assert!(out.contains(r#"\[\displaystyle E \left[x\right] \left(y\right)\]"#));
        assert!(!out.contains("<svg>"));
    }

    #[test]
    fn aligned_equation_pieces_use_displaystyle_without_display_delimiters() {
        let input = r#"<p><span class="equation-align-right" data-typst-math="sequence([=], [ ], [[], [x], []])" data-math-display="inline"><svg></svg></span></p>"#;
        let out = postprocess_html_math(input.to_owned(), MathMode::Katex);
        assert!(out.contains(r#"\(\displaystyle = \left[x\right]\)"#));
    }

    #[test]
    fn display_delimiter_sizing_preserves_sqrt_optional_argument() {
        assert_eq!(
            prepare_display_tex(r#"\sqrt[n]{(x)}"#),
            r#"\displaystyle \sqrt[n]{\left(x\right)}"#
        );
    }

    #[test]
    fn display_delimiter_sizing_pairs_symmetric_delimiters() {
        assert_eq!(
            prepare_display_tex(r#"\Vert A\Vert _F"#),
            r#"\displaystyle \left\Vert A\right\Vert _F"#
        );
    }

    #[test]
    fn converts_typst_split_superscript_repr() {
        let input = "sequence(equation(block: false, body: styled(child: [c], ..)), attach(base: [ ], t: lr(body: sequence([(], [t], [)]))), [ ], [∈], [ ], equation(block: false, body: styled(child: [C], ..)))";
        assert_eq!(
            typst_repr_to_katex(input),
            "\\boldsymbol{c} ^{(t)} \\in \\mathcal{C}"
        );
    }

    #[test]
    fn direct_calligraphic_s_fallback_is_preserved() {
        assert_eq!(
            typst_repr_to_katex("styled(child: [S], ..)"),
            "\\mathcal{S}"
        );
    }

    #[test]
    fn styled_symbols_with_subscripts_keep_their_style() {
        assert_eq!(
            typst_repr_to_katex("styled(child: attach(base: [X], b: [i]), ..)"),
            "\\mathcal{X}_{i}"
        );
        assert_eq!(
            typst_repr_to_katex("styled(child: attach(base: [x], b: [i]), ..)"),
            "\\boldsymbol{x}_{i}"
        );
    }

    #[test]
    fn unicode_math_alphabet_notation_is_unambiguous() {
        let input = "sequence([𝓢], [ ], [⊂], [ ], [𝓧], [ ], [×], [ ], [𝐱], [ ], [∈], [ ], [ℝ])";
        assert_eq!(
            typst_repr_to_katex(input),
            "\\mathcal{S} \\subset \\mathcal{X} \\times \\boldsymbol{x} \\in \\mathbb{R}"
        );
    }

    #[test]
    fn upright_bold_matrix_notation_stays_mathbf() {
        let input = "sequence([𝐀], [ ], [𝐱], [ ], [≤], [ ], [𝐛])";
        assert_eq!(
            typst_repr_to_katex(input),
            "\\mathbf{A} \\boldsymbol{x} \\le \\boldsymbol{b}"
        );
    }

    #[test]
    fn replaces_svg_span_with_katex_source() {
        let input = r#"<p><span role="math" data-typst-math="sequence([x], [≤], [y])" data-math-display="inline"><svg></svg></span></p>"#;
        let out = postprocess_html_math(input.to_owned(), MathMode::Katex);
        assert!(out.contains(r#"\(x\le y\)"#));
        assert!(out.contains("math-katex-source"));
        assert!(!out.contains("<svg>"));
    }

    #[test]
    fn root_omits_empty_optional_argument() {
        let input = r#"<p>on the order of <span role="math" data-typst-math="root(radicand: [d])" data-math-display="inline"><svg></svg></span>. Then</p>"#;
        let out = postprocess_html_math(input.to_owned(), MathMode::Katex);
        assert!(out.contains(r#"\(\sqrt{d}\)"#), "{out}");
        assert!(!out.contains(r#"\sqrt[]{d}"#), "{out}");
    }

    #[test]
    fn root_preserves_nonempty_optional_argument() {
        assert_eq!(
            typst_repr_to_katex("root(index: [n], radicand: [d])"),
            r"\sqrt[n]{d}"
        );
    }

    #[test]
    fn unsupported_nonempty_math_gets_visible_placeholder() {
        let input = r#"<p><span role="math" data-typst-math="mystery(value: [x])" data-math-display="inline"><svg></svg></span></p>"#;
        let out = postprocess_html_math(input.to_owned(), MathMode::Katex);
        assert!(
            out.contains(r#"\(\text{[unsupported math: mystery]}\)"#),
            "{out}"
        );
    }
}
