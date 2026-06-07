use std::collections::HashMap;
use std::env;
use std::ffi::OsString;
use std::fmt::Write as _;
use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;

const DEFAULT_SITE_TITLE: &str = "Learning and Computation of Phi-Equilibria";
const DEFAULT_AUTHORS: &str = "Ioannis Anagnostides, Gabriele Farina, and Brian Hu Zhang";
const BIBTEX_AUTHORS: &str =
    "Anagnostides, Ioannis and Farina, Gabriele and Zhang, Brian Hu";
const CITATION_MARGIN_NOTE_MIN_CHARS: usize = 900;

#[derive(Clone, Copy)]
struct ChapterNav {
    number: u8,
    source: &'static str,
    href: &'static str,
    short_title: &'static str,
}

const CHAPTERS: &[ChapterNav] = &[
    ChapterNav {
        number: 1,
        source: "P1-introduction.typ",
        href: "P1-introduction.html",
        short_title: "Introduction",
    },
    ChapterNav {
        number: 2,
        source: "P2-semi_separation.typ",
        href: "P2-semi_separation.html",
        short_title: "Semi-Separation",
    },
    ChapterNav {
        number: 3,
        source: "P3-phi-regret-learning.typ",
        href: "P3-phi-regret-learning.html",
        short_title: "Beyond Normal Form",
    },
    ChapterNav {
        number: 4,
        source: "P4-multicalibration.typ",
        href: "P4-multicalibration.html",
        short_title: "Multicalibration",
    },
    ChapterNav {
        number: 5,
        source: "P5-treeswap.typ",
        href: "P5-treeswap.html",
        short_title: "TreeSwap",
    },
    ChapterNav {
        number: 6,
        source: "P6-profile.typ",
        href: "P6-profile.html",
        short_title: "Profile Swap",
    },
];

fn main() {
    if let Err(err) = run() {
        eprintln!("error: {err}");
        std::process::exit(1);
    }
}

fn run() -> Result<(), String> {
    let config = Config::parse(env::args_os().skip(1))?;
    let source = read_typst_tree(&config.input)?;
    let meta = LectureMeta::from_source(&source);
    let bib = Bibliography::load(&config.root.join("meta/refs.bib"));

    let mut parser = Parser::new(&source);
    let mut blocks = parser.parse_blocks();
    let mut labels = LabelBook::new(meta.lecture_number);
    labels.number_blocks(&mut blocks);

    let title = config
        .title
        .clone()
        .or(meta.title)
        .unwrap_or_else(|| title_from_path(&config.input));
    let headings = collect_headings(&blocks);
    let html = render_document(&config, &title, &blocks, &headings, &labels, &bib);

    if let Some(parent) = config.output.parent() {
        if !parent.as_os_str().is_empty() {
            fs::create_dir_all(parent).map_err(|err| {
                format!(
                    "could not create output directory {}: {err}",
                    parent.display()
                )
            })?;
        }
    }
    fs::write(&config.output, html)
        .map_err(|err| format!("could not write {}: {err}", config.output.display()))?;
    Ok(())
}

#[derive(Debug)]
struct Config {
    input: PathBuf,
    output: PathBuf,
    root: PathBuf,
    title: Option<String>,
    site_title: String,
    authors: String,
    index_href: Option<String>,
    pdf_href: Option<String>,
}

impl Config {
    fn parse<I>(args: I) -> Result<Self, String>
    where
        I: IntoIterator<Item = OsString>,
    {
        let mut input = None;
        let mut output = None;
        let mut root = None;
        let mut title = None;
        let mut site_title = DEFAULT_SITE_TITLE.to_owned();
        let mut authors = DEFAULT_AUTHORS.to_owned();
        let mut index_href = Some("index.html".to_owned());
        let mut pdf_href = None;

        let mut args = args.into_iter();
        while let Some(arg) = args.next() {
            let text = arg.to_string_lossy();
            match text.as_ref() {
                "-h" | "--help" => {
                    print_usage();
                    std::process::exit(0);
                }
                "--root" => root = Some(next_path(&mut args, "--root")?),
                "--title" => title = Some(next_string(&mut args, "--title")?),
                "--site-title" => site_title = next_string(&mut args, "--site-title")?,
                "--authors" => authors = next_string(&mut args, "--authors")?,
                "--index" => index_href = Some(next_string(&mut args, "--index")?),
                "--no-index" => index_href = None,
                "--pdf" => pdf_href = Some(next_string(&mut args, "--pdf")?),
                "--typst" | "--keep-raw" => {
                    let _ = next_string(&mut args, text.as_ref())?;
                }
                _ if text.starts_with('-') => return Err(format!("unknown option `{text}`")),
                _ => {
                    if input.is_none() {
                        input = Some(PathBuf::from(arg));
                    } else if output.is_none() {
                        output = Some(PathBuf::from(arg));
                    } else {
                        return Err(format!("unexpected positional argument `{text}`"));
                    }
                }
            }
        }

        let input = input.ok_or_else(|| "missing input .typ file".to_owned())?;
        let output = output.unwrap_or_else(|| input.with_extension("html"));
        let root = root.unwrap_or_else(|| {
            input
                .parent()
                .filter(|path| !path.as_os_str().is_empty())
                .unwrap_or_else(|| Path::new("."))
                .to_path_buf()
        });

        Ok(Self {
            input,
            output,
            root,
            title,
            site_title,
            authors,
            index_href,
            pdf_href,
        })
    }
}

fn next_path<I>(args: &mut I, name: &str) -> Result<PathBuf, String>
where
    I: Iterator<Item = OsString>,
{
    Ok(PathBuf::from(next_string(args, name)?))
}

fn next_string<I>(args: &mut I, name: &str) -> Result<String, String>
where
    I: Iterator<Item = OsString>,
{
    args.next()
        .ok_or_else(|| format!("missing value for {name}"))?
        .into_string()
        .map_err(|_| format!("value for {name} is not valid UTF-8"))
}

fn print_usage() {
    println!(
        "Usage: notes-html-exporter [options] <input.typ> [output.html]\n\
         \n\
         Options:\n\
           --root <dir>          Project root for includes and bibliography\n\
           --title <title>       Page title\n\
           --site-title <title>  Header title\n\
           --authors <text>      Header author line\n\
           --index <href>        Header index link\n\
           --no-index            Hide the index link\n\
           --pdf <href>          Header PDF link\n"
    );
}

fn read_typst_tree(path: &Path) -> Result<String, String> {
    let source = fs::read_to_string(path)
        .map_err(|err| format!("could not read {}: {err}", path.display()))?;
    let base = path.parent().unwrap_or_else(|| Path::new("."));
    let mut out = String::new();
    for line in source.lines() {
        let trimmed = line.trim();
        if let Some(include) = parse_quoted_directive(trimmed, "#include") {
            let included = base.join(include);
            out.push_str(&read_typst_tree(&included)?);
            out.push('\n');
        } else {
            out.push_str(line);
            out.push('\n');
        }
    }
    Ok(out)
}

fn parse_quoted_directive<'a>(line: &'a str, directive: &str) -> Option<&'a str> {
    let rest = line.strip_prefix(directive)?.trim_start();
    let rest = rest.strip_prefix('"')?;
    let end = rest.find('"')?;
    Some(&rest[..end])
}

#[derive(Default)]
struct LectureMeta {
    lecture_number: Option<u32>,
    title: Option<String>,
}

impl LectureMeta {
    fn from_source(source: &str) -> Self {
        let mut meta = Self::default();
        if let Some(pos) = source.find("gabri_notes.with(") {
            let tail = &source[pos..source[pos..]
                .find(')')
                .map(|n| pos + n)
                .unwrap_or(source.len())];
            meta.lecture_number = find_named_u32(tail, "lec_num:");
            meta.title = find_named_string(tail, "title:");
        }
        meta
    }
}

fn find_named_u32(source: &str, name: &str) -> Option<u32> {
    let rest = source[source.find(name)? + name.len()..].trim_start();
    let digits: String = rest.chars().take_while(|ch| ch.is_ascii_digit()).collect();
    digits.parse().ok()
}

fn find_named_string(source: &str, name: &str) -> Option<String> {
    parse_typst_string(source[source.find(name)? + name.len()..].trim_start())
}

fn parse_typst_string(input: &str) -> Option<String> {
    let mut chars = input.chars();
    if chars.next()? != '"' {
        return None;
    }
    let mut out = String::new();
    let mut escaped = false;
    for ch in chars {
        if escaped {
            out.push(ch);
            escaped = false;
        } else if ch == '\\' {
            escaped = true;
        } else if ch == '"' {
            return Some(out);
        } else {
            out.push(ch);
        }
    }
    None
}

#[derive(Clone, Debug)]
enum Block {
    Heading {
        level: u8,
        text: String,
        label: Option<String>,
        id: String,
        number: String,
    },
    Paragraph(String),
    DisplayMath {
        source: String,
        label: Option<String>,
        number: Option<u32>,
    },
    Environment {
        kind: EnvKind,
        title: Option<String>,
        body: Vec<Block>,
        label: Option<String>,
        number: Option<String>,
    },
    Algorithm {
        title: Option<String>,
        label: Option<String>,
        items: Vec<PseudoItem>,
        number: Option<String>,
    },
    Figure {
        caption: Option<String>,
        label: Option<String>,
        number: Option<u32>,
        source: Option<String>,
    },
    Comment {
        text: String,
        figure_stem: Option<String>,
    },
    List {
        ordered: bool,
        items: Vec<String>,
    },
    RawBlock(String),
    RawHtml(String),
}

#[derive(Clone, Debug)]
enum PseudoItem {
    Step {
        indent: usize,
        text: String,
        label: Option<String>,
    },
    Math {
        indent: usize,
        source: String,
    },
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
enum EnvKind {
    Theorem,
    Proposition,
    Lemma,
    Corollary,
    Definition,
    Example,
    Exercise,
    Remark,
    Proof,
    ProofSketch,
    Solution,
    Algorithm,
}

impl EnvKind {
    fn from_name(name: &str) -> Option<Self> {
        match name {
            "theorem" => Some(Self::Theorem),
            "proposition" => Some(Self::Proposition),
            "lemma" => Some(Self::Lemma),
            "corollary" => Some(Self::Corollary),
            "definition" => Some(Self::Definition),
            "example" => Some(Self::Example),
            "exercise" => Some(Self::Exercise),
            "remark" => Some(Self::Remark),
            "proof" => Some(Self::Proof),
            "proofsketch" => Some(Self::ProofSketch),
            "solution" => Some(Self::Solution),
            _ => None,
        }
    }

    fn label(self) -> &'static str {
        match self {
            Self::Theorem => "Theorem",
            Self::Proposition => "Proposition",
            Self::Lemma => "Lemma",
            Self::Corollary => "Corollary",
            Self::Definition => "Definition",
            Self::Example => "Example",
            Self::Exercise => "Exercise",
            Self::Remark => "Remark",
            Self::Proof => "Proof",
            Self::ProofSketch => "Proof sketch",
            Self::Solution => "Solution",
            Self::Algorithm => "Algorithm",
        }
    }

    fn class(self) -> &'static str {
        match self {
            Self::Proof | Self::ProofSketch | Self::Solution => "proof",
            Self::Algorithm => "algorithm",
            _ => "statement",
        }
    }

    fn numbered(self) -> bool {
        !matches!(self, Self::Proof | Self::ProofSketch | Self::Solution)
    }
}

struct Parser<'a> {
    lines: Vec<&'a str>,
    pos: usize,
}

impl<'a> Parser<'a> {
    fn new(source: &'a str) -> Self {
        Self {
            lines: source.lines().collect(),
            pos: 0,
        }
    }

    fn parse_blocks(&mut self) -> Vec<Block> {
        let mut blocks = Vec::new();
        while self.pos < self.lines.len() {
            let line = strip_line_comment(self.lines[self.pos]);
            let trimmed = line.trim();
            if trimmed.is_empty() {
                self.pos += 1;
                continue;
            }
            if is_setup_directive(trimmed) {
                self.skip_setup_directive();
                continue;
            }

            if let Some(block) = self.parse_heading(trimmed) {
                blocks.push(block);
                self.pos += 1;
            } else if is_display_math_start(trimmed) {
                blocks.push(self.parse_display_math());
            } else if trimmed.starts_with("#figure(") {
                blocks.push(self.parse_figure());
            } else if trimmed.starts_with("#pseudocode-list") {
                blocks.push(self.parse_pseudocode());
            } else if trimmed.starts_with("```") {
                blocks.push(self.parse_raw_block());
            } else if trimmed.starts_with("#comment[") {
                blocks.push(self.parse_comment());
            } else if let Some(block) = self.parse_environment(trimmed) {
                blocks.push(block);
            } else if is_list_line(trimmed) {
                blocks.push(self.parse_list());
            } else if trimmed.starts_with("#align(") || trimmed.starts_with("#box(") {
                blocks.push(self.parse_generic_block());
            } else {
                blocks.push(self.parse_paragraph());
            }
        }
        blocks
    }

    fn skip_setup_directive(&mut self) {
        let mut depth = 0i32;
        let mut saw_open = false;
        while self.pos < self.lines.len() {
            let line = strip_line_comment(self.lines[self.pos]);
            for ch in line.chars() {
                if ch == '(' {
                    saw_open = true;
                    depth += 1;
                } else if ch == ')' && depth > 0 {
                    depth -= 1;
                }
            }
            self.pos += 1;
            if !saw_open || depth == 0 {
                break;
            }
        }
    }

    fn parse_heading(&self, trimmed: &str) -> Option<Block> {
        let marks = trimmed.chars().take_while(|ch| *ch == '=').count();
        if marks == 0 || !trimmed.chars().nth(marks).is_some_and(char::is_whitespace) {
            return None;
        }
        let mut text = trimmed[marks..].trim().to_owned();
        let label = pop_label_suffix(&mut text);
        let id = label.clone().unwrap_or_else(|| slugify(&text));
        Some(Block::Heading {
            level: marks as u8,
            text,
            label,
            id,
            number: String::new(),
        })
    }

    fn parse_display_math(&mut self) -> Block {
        let first = strip_line_comment(self.lines[self.pos]).trim().to_owned();
        let mut label = None;
        let mut source = String::new();

        if first.len() > 1 && first[1..].contains('$') {
            let mut line = first;
            label = pop_label_suffix(&mut line);
            let content = line
                .trim()
                .trim_start_matches('$')
                .trim_end_matches('$')
                .trim();
            source.push_str(content);
            self.pos += 1;
        } else {
            let first_content = first.trim_start_matches('$').trim();
            if !first_content.is_empty() {
                source.push_str(first_content);
                source.push('\n');
            }
            self.pos += 1;
            while self.pos < self.lines.len() {
                let mut line = strip_line_comment(self.lines[self.pos]).trim().to_owned();
                if line.starts_with('$') {
                    label = pop_label_suffix(&mut line);
                    let closing_content = line.trim_start_matches('$').trim();
                    if !closing_content.is_empty() {
                        source.push_str(closing_content);
                    }
                    self.pos += 1;
                    break;
                }
                source.push_str(self.lines[self.pos]);
                source.push('\n');
                self.pos += 1;
            }
        }

        Block::DisplayMath {
            source: source.trim().to_owned(),
            label,
            number: None,
        }
    }

    fn parse_figure(&mut self) -> Block {
        let raw = self.collect_balanced('(', ')');
        let mut suffix = raw.clone();
        let label = pop_label_suffix(&mut suffix);
        let caption = extract_bracket_after(&raw, "caption:");
        Block::Figure {
            caption,
            label,
            number: None,
            source: Some(raw),
        }
    }

    fn parse_pseudocode(&mut self) -> Block {
        let (raw, body) = self.collect_pseudocode_body();
        let mut title = extract_bracket_after(&raw, "title:");
        let label = title.as_mut().and_then(pop_label_suffix);
        Block::Algorithm {
            title,
            label,
            items: parse_pseudocode_items(&body),
            number: None,
        }
    }

    fn parse_raw_block(&mut self) -> Block {
        let opening = self.lines[self.pos].trim();
        let mut raw = String::new();
        let inline = opening.strip_prefix("```").unwrap_or_default();
        if !inline.trim().is_empty() {
            raw.push_str(inline.trim());
        }
        self.pos += 1;
        while self.pos < self.lines.len() {
            let line = self.lines[self.pos];
            if line.trim_start().starts_with("```") {
                self.pos += 1;
                break;
            }
            if !raw.is_empty() {
                raw.push('\n');
            }
            raw.push_str(line);
            self.pos += 1;
        }
        Block::RawBlock(raw)
    }

    fn parse_comment(&mut self) -> Block {
        let (_raw, body) = self.collect_bracket_body();
        let figure_stem = figure_stem_from_source(&body);
        Block::Comment {
            text: strip_comment_figure_lines(&body),
            figure_stem,
        }
    }

    fn parse_environment(&mut self, trimmed: &str) -> Option<Block> {
        let name = trimmed.strip_prefix('#')?.split(['(', '[']).next()?;
        let kind = EnvKind::from_name(name)?;
        let (mut raw, body) = self.collect_bracket_body();
        let label = pop_label_suffix(&mut raw);
        let prefix = raw
            .split_once('[')
            .map(|(prefix, _)| prefix)
            .unwrap_or(&raw);
        let title = extract_title_from_prefix(prefix);
        let mut nested = Parser::new(&body);
        let mut body = nested.parse_blocks();
        let label = label.or_else(|| pop_trailing_label_block(&mut body));
        Some(Block::Environment {
            kind,
            title,
            body,
            label,
            number: None,
        })
    }

    fn parse_list(&mut self) -> Block {
        let mut items = Vec::new();
        let ordered = strip_line_comment(self.lines[self.pos])
            .trim_start()
            .starts_with('+');
        while self.pos < self.lines.len() {
            let trimmed = strip_line_comment(self.lines[self.pos]).trim();
            if !is_list_line(trimmed) {
                break;
            }
            items.push(trimmed[2..].trim().to_owned());
            self.pos += 1;
        }
        Block::List { ordered, items }
    }

    fn parse_generic_block(&mut self) -> Block {
        let raw = if strip_line_comment(self.lines[self.pos]).contains('[') {
            let (_raw, body) = self.collect_bracket_body();
            body
        } else {
            self.collect_balanced('(', ')')
        };
        if raw.contains("#figure(") {
            let caption = extract_bracket_after(&raw, "caption:");
            let label = find_label_anywhere(&raw);
            return Block::Figure {
                caption,
                label,
                number: None,
                source: Some(raw),
            };
        }
        if raw.trim_start().starts_with("#align(") {
            if let Some(body) = extract_callout_body(&raw) {
                let mut state = RenderState::default();
                return Block::RawHtml(format!(
                    "<div class=\"callout\">{}</div>",
                    render_inline(
                        body.trim(),
                        &LabelBook::empty(),
                        &Bibliography::default(),
                        &mut state
                    )
                ));
            }
        }
        let mut state = RenderState::default();
        Block::RawHtml(format!(
            "<div class=\"typst-fallback\">{}</div>",
            render_inline(
                raw.trim(),
                &LabelBook::empty(),
                &Bibliography::default(),
                &mut state
            )
        ))
    }

    fn parse_paragraph(&mut self) -> Block {
        let mut text = String::new();
        while self.pos < self.lines.len() {
            let line = strip_line_comment(self.lines[self.pos]);
            let trimmed = line.trim();
            if trimmed.is_empty()
                || trimmed.starts_with('=')
                || is_display_math_start(trimmed)
                || trimmed.starts_with("#figure(")
                || trimmed.starts_with("#pseudocode-list")
                || trimmed.starts_with("#comment[")
                || trimmed.starts_with("#align(")
                || trimmed.starts_with("#box(")
                || trimmed.starts_with("#lec_bibliography")
                || trimmed.starts_with("#import")
                || trimmed.starts_with("#show")
                || trimmed.starts_with("```")
                || is_environment_start(trimmed)
                || is_list_line(trimmed)
            {
                break;
            }
            if !text.is_empty() {
                text.push(' ');
            }
            text.push_str(trimmed);
            self.pos += 1;
        }
        Block::Paragraph(text)
    }

    fn collect_balanced(&mut self, open: char, close: char) -> String {
        let mut raw = String::new();
        let mut depth = 0i32;
        let mut seen_open = false;
        while self.pos < self.lines.len() {
            let line = self.lines[self.pos];
            raw.push_str(line);
            raw.push('\n');
            for ch in line.chars() {
                if ch == open {
                    depth += 1;
                    seen_open = true;
                } else if ch == close && seen_open {
                    depth -= 1;
                }
            }
            self.pos += 1;
            if seen_open && depth <= 0 {
                break;
            }
        }
        raw
    }

    fn collect_bracket_body(&mut self) -> (String, String) {
        let mut raw = String::new();
        let mut body = String::new();
        let mut depth = 0i32;
        let mut in_body = false;
        let mut done = false;

        while self.pos < self.lines.len() && !done {
            let line = self.lines[self.pos];
            let start = if !in_body {
                line.rfind('[').map(|idx| idx + 1)
            } else {
                Some(0)
            };
            raw.push_str(line);
            raw.push('\n');

            if let Some(mut idx) = start {
                if !in_body {
                    in_body = true;
                    depth = 1;
                }
                let chars: Vec<(usize, char)> = line.char_indices().collect();
                while idx <= line.len() {
                    let Some((byte_idx, ch)) = chars.iter().copied().find(|(i, _)| *i >= idx)
                    else {
                        break;
                    };
                    if ch == '[' {
                        depth += 1;
                        if depth > 1 {
                            body.push(ch);
                        }
                    } else if ch == ']' {
                        depth -= 1;
                        if depth == 0 {
                            done = true;
                            break;
                        }
                        body.push(ch);
                    } else {
                        body.push(ch);
                    }
                    idx = byte_idx + ch.len_utf8();
                }
                if !done {
                    body.push('\n');
                }
            }
            self.pos += 1;
        }
        (raw, body)
    }

    fn collect_pseudocode_body(&mut self) -> (String, String) {
        let mut raw = String::new();
        let mut body = String::new();
        let mut depth = 0i32;
        let mut in_body = false;

        while self.pos < self.lines.len() {
            let line = self.lines[self.pos];
            raw.push_str(line);
            raw.push('\n');

            let mut start = None;
            if !in_body {
                if let Some(marker) = line.find(")[") {
                    in_body = true;
                    depth = 1;
                    start = Some(marker + 2);
                }
            } else {
                start = Some(0);
            }

            if let Some(mut idx) = start {
                let chars: Vec<(usize, char)> = line.char_indices().collect();
                while idx <= line.len() {
                    let Some((byte_idx, ch)) = chars.iter().copied().find(|(i, _)| *i >= idx)
                    else {
                        break;
                    };
                    if ch == '[' {
                        depth += 1;
                        if depth > 1 {
                            body.push(ch);
                        }
                    } else if ch == ']' {
                        depth -= 1;
                        if depth == 0 {
                            self.pos += 1;
                            return (raw, body);
                        }
                        body.push(ch);
                    } else {
                        body.push(ch);
                    }
                    idx = byte_idx + ch.len_utf8();
                }
                body.push('\n');
            }
            self.pos += 1;
        }
        (raw, body)
    }
}

fn strip_line_comment(line: &str) -> &str {
    let trimmed = line.trim_start();
    if trimmed.starts_with("//") {
        ""
    } else {
        line
    }
}

fn is_setup_directive(trimmed: &str) -> bool {
    trimmed.starts_with("#import")
        || trimmed.starts_with("#show")
        || trimmed.starts_with("#set ")
        || trimmed.starts_with("#let ")
        || trimmed.starts_with("#lec_bibliography")
}

fn is_environment_start(trimmed: &str) -> bool {
    let Some(name) = trimmed
        .strip_prefix('#')
        .and_then(|s| s.split(['(', '[']).next())
    else {
        return false;
    };
    EnvKind::from_name(name).is_some()
}

fn parse_pseudocode_items(body: &str) -> Vec<PseudoItem> {
    let lines: Vec<&str> = body.lines().collect();
    let mut items = Vec::new();
    let mut pos = 0;
    while pos < lines.len() {
        let line = lines[pos];
        let trimmed = strip_line_comment(line).trim();
        if trimmed.is_empty() {
            pos += 1;
            continue;
        }

        if trimmed.starts_with('$') {
            let indent = leading_spaces(line) / 2;
            let mut source = String::new();
            let first = trimmed.trim_start_matches('$').trim();
            if !first.is_empty() {
                source.push_str(first);
                source.push('\n');
            }
            pos += 1;
            while pos < lines.len() {
                let next = strip_line_comment(lines[pos]).trim();
                if next.starts_with('$') {
                    let last = next.trim_start_matches('$').trim();
                    if !last.is_empty() {
                        source.push_str(last);
                    }
                    pos += 1;
                    break;
                }
                source.push_str(lines[pos]);
                source.push('\n');
                pos += 1;
            }
            items.push(PseudoItem::Math {
                indent,
                source: source.trim().to_owned(),
            });
            continue;
        }

        if let Some((indent, mut text)) = parse_pseudocode_step(line) {
            strip_layout_commands(&mut text);
            let label = pop_label_suffix(&mut text);
            items.push(PseudoItem::Step {
                indent,
                text: text.trim().to_owned(),
                label,
            });
        }
        pos += 1;
    }
    items
}

fn parse_pseudocode_step(line: &str) -> Option<(usize, String)> {
    let trimmed = strip_line_comment(line).trim_start();
    let text = trimmed.strip_prefix("- ")?;
    Some((leading_spaces(line) / 2, text.trim().to_owned()))
}

fn leading_spaces(line: &str) -> usize {
    line.chars().take_while(|ch| *ch == ' ').count()
}

fn strip_layout_commands(text: &mut String) {
    for command in ["#h", "#v"] {
        while let Some(pos) = text.find(&format!("{command}(")) {
            let tail = &text[pos..];
            let Some((whole, _)) = parse_layout_command(tail) else {
                break;
            };
            text.replace_range(pos..pos + whole, "");
        }
    }
}

fn is_display_math_start(trimmed: &str) -> bool {
    if !trimmed.starts_with('$') {
        return false;
    }
    if trimmed == "$" {
        return true;
    }
    if !trimmed[1..].chars().next().is_some_and(char::is_whitespace) {
        return false;
    }
    let Some(close) = trimmed[1..].find('$').map(|idx| idx + 1) else {
        return true;
    };
    let suffix = trimmed[close + 1..].trim();
    suffix.is_empty() || is_standalone_label(suffix)
}

fn is_list_line(trimmed: &str) -> bool {
    trimmed.starts_with("- ") || trimmed.starts_with("+ ")
}

fn is_standalone_label(trimmed: &str) -> bool {
    trimmed.starts_with('<')
        && trimmed.ends_with('>')
        && !trimmed[1..trimmed.len() - 1].contains(char::is_whitespace)
}

fn pop_trailing_label_block(blocks: &mut Vec<Block>) -> Option<String> {
    let Some(Block::Paragraph(text)) = blocks.last_mut() else {
        return None;
    };
    let trimmed = text.trim();
    if is_standalone_label(trimmed) {
        let label = trimmed[1..trimmed.len() - 1].to_owned();
        blocks.pop();
        return Some(label);
    }
    pop_label_suffix(text)
}

fn pop_label_suffix(text: &mut String) -> Option<String> {
    let trimmed = text.trim_end();
    let end = trimmed.rfind('>')?;
    if end + 1 != trimmed.len() {
        return None;
    }
    let start = trimmed[..end].rfind('<')?;
    let label = trimmed[start + 1..end].to_owned();
    if label.is_empty() || label.contains(char::is_whitespace) {
        return None;
    }
    text.truncate(start);
    text.truncate(text.trim_end().len());
    Some(label)
}

fn find_label_anywhere(text: &str) -> Option<String> {
    let end = text.rfind('>')?;
    let start = text[..end].rfind('<')?;
    let label = &text[start + 1..end];
    if label.is_empty() || label.contains(char::is_whitespace) {
        None
    } else {
        Some(label.to_owned())
    }
}

fn extract_bracket_after(source: &str, needle: &str) -> Option<String> {
    let start = source.find(needle)? + needle.len();
    let rest = &source[start..];
    let open = rest.find('[')? + start;
    let mut depth = 0i32;
    let mut out = String::new();
    for ch in source[open..].chars() {
        if ch == '[' {
            depth += 1;
            if depth > 1 {
                out.push(ch);
            }
        } else if ch == ']' {
            depth -= 1;
            if depth == 0 {
                return Some(out.trim().to_owned());
            }
            out.push(ch);
        } else if depth > 0 {
            out.push(ch);
        }
    }
    None
}

fn extract_callout_body(source: &str) -> Option<String> {
    let marker = source.find(")[")?;
    let open = marker + 1;
    extract_balanced_square_at(source, open)
}

fn extract_balanced_square_at(source: &str, open: usize) -> Option<String> {
    if source.as_bytes().get(open) != Some(&b'[') {
        return None;
    }
    let mut depth = 0i32;
    let mut out = String::new();
    for (idx, ch) in source.char_indices().skip_while(|(idx, _)| *idx < open) {
        if ch == '[' {
            depth += 1;
            if depth > 1 {
                out.push(ch);
            }
        } else if ch == ']' {
            depth -= 1;
            if depth == 0 {
                return Some(out);
            }
            out.push(ch);
        } else if idx > open {
            out.push(ch);
        }
    }
    None
}

fn extract_title_from_prefix(prefix: &str) -> Option<String> {
    parse_typst_string(prefix.split_once('(')?.1.trim()).or_else(|| {
        let start = prefix.find('[')?;
        let end = prefix.rfind(']')?;
        Some(prefix[start + 1..end].trim().to_owned())
    })
}

#[derive(Default)]
struct Bibliography {
    entries: HashMap<String, BibEntry>,
}

#[derive(Default)]
struct BibEntry {
    author: String,
    title: String,
    journal: String,
    booktitle: String,
    doi: String,
    url: String,
    eprint: String,
    year: String,
}

impl Bibliography {
    fn load(path: &Path) -> Self {
        let Ok(source) = fs::read_to_string(path) else {
            return Self::default();
        };
        let mut entries = HashMap::new();
        for chunk in source.split("\n@").filter(|chunk| chunk.contains('{')) {
            let Some(brace) = chunk.find('{') else {
                continue;
            };
            let Some(comma) = chunk[brace + 1..].find(',').map(|idx| brace + 1 + idx) else {
                continue;
            };
            let key = chunk[brace + 1..comma].trim().to_owned();
            if key.is_empty() || key.eq_ignore_ascii_case("string") {
                continue;
            }
            entries.insert(
                key,
                BibEntry {
                    author: field_value(chunk, "author").unwrap_or_default(),
                    title: field_value(chunk, "title").unwrap_or_default(),
                    journal: field_value(chunk, "journal").unwrap_or_default(),
                    booktitle: field_value(chunk, "booktitle").unwrap_or_default(),
                    doi: field_value(chunk, "doi").unwrap_or_default(),
                    url: field_value(chunk, "url").unwrap_or_default(),
                    eprint: field_value(chunk, "eprint").unwrap_or_default(),
                    year: field_value(chunk, "year").unwrap_or_default(),
                },
            );
        }
        Self { entries }
    }

    fn cite(&self, key: &str, textual: bool) -> String {
        let Some(entry) = self.entries.get(key) else {
            return if textual {
                key.to_owned()
            } else {
                format!("[{key}]")
            };
        };
        let label = self.alphanum_label(key);
        if textual {
            format!("{} [{label}]", prose_author_list(&entry.author))
        } else {
            format!("[{label}]")
        }
    }

    fn citation_key(&self, key: &str) -> String {
        format!("[{}]", self.alphanum_label(key))
    }

    fn alphanum_label(&self, key: &str) -> String {
        let Some(entry) = self.entries.get(key) else {
            return key.to_owned();
        };
        let authors = author_last_names(&entry.author);
        let year = year_suffix(&entry.year);
        let prefix = match authors.len() {
            0 => alpha_prefix_from_key(key),
            1 => alpha_prefix_from_name(&authors[0]),
            2 | 3 => authors
                .iter()
                .filter_map(|name| surname_initial(name))
                .collect(),
            _ => format!("{}+", alpha_prefix_from_name(&authors[0])),
        };
        format!("{prefix}{year}")
    }

    fn full_entry(&self, key: &str) -> Option<String> {
        let entry = self.entries.get(key)?;
        let mut parts = Vec::new();
        if !entry.author.is_empty() {
            parts.push(format!("{}.", bibliography_author_list(&entry.author)));
        }
        if !entry.year.is_empty() {
            parts.push(format!("({}).", entry.year));
        }
        if !entry.title.is_empty() {
            parts.push(format!("{}.", entry.title));
        }
        let venue = if !entry.journal.is_empty() {
            &entry.journal
        } else {
            &entry.booktitle
        };
        if !venue.is_empty() {
            parts.push(format!("{}.", venue));
        }
        if !entry.doi.is_empty() {
            parts.push(format!("doi:{}", entry.doi));
        }
        if !parts.iter().any(|part| arxiv_id_in_text(part).is_some()) {
            if let Some(id) = arxiv_id_from_entry(entry) {
                parts.push(format!("arXiv:{id}"));
            }
        }
        Some(parts.join(" "))
    }
}

fn field_value(chunk: &str, field: &str) -> Option<String> {
    let pos = chunk.find(field)?;
    let rest = &chunk[pos + field.len()..];
    let eq = rest.find('=')?;
    let rest = rest[eq + 1..].trim_start();
    let Some(rest) = rest.strip_prefix('{') else {
        let value = rest
            .split([',', '\n'])
            .next()
            .unwrap_or(rest)
            .trim()
            .trim_matches('"');
        return Some(resolve_bib_value(value));
    };
    let mut depth = 1i32;
    let mut out = String::new();
    for ch in rest.chars() {
        if ch == '{' {
            depth += 1;
            out.push(ch);
        } else if ch == '}' {
            depth -= 1;
            if depth == 0 {
                return Some(out.split_whitespace().collect::<Vec<_>>().join(" "));
            }
            out.push(ch);
        } else {
            out.push(ch);
        }
    }
    None
}

fn resolve_bib_value(value: &str) -> String {
    match value {
        "STOC" => "Symposium on Theory of Computing (STOC)".to_owned(),
        "FOCS" => "Annual Symposium on Foundations of Computer Science (FOCS)".to_owned(),
        _ => value.to_owned(),
    }
}

fn author_last_names(author: &str) -> Vec<String> {
    author
        .split(" and ")
        .map(author_last_name)
        .filter(|name| !name.is_empty())
        .collect()
}

fn author_last_name(author: &str) -> String {
    let clean = clean_bib_name(author);
    if let Some((last, _)) = clean.split_once(',') {
        last.trim().to_owned()
    } else {
        clean
            .split_whitespace()
            .last()
            .unwrap_or(clean.trim())
            .trim()
            .to_owned()
    }
}

fn prose_author_list(author: &str) -> String {
    join_author_names(&author_last_names(author))
}

fn bibliography_author_list(author: &str) -> String {
    let names: Vec<String> = author
        .split(" and ")
        .map(format_bibliography_author)
        .filter(|name| !name.is_empty())
        .collect();
    join_author_names(&names)
}

fn join_author_names(names: &[String]) -> String {
    match names {
        [] => String::new(),
        [one] => one.clone(),
        [first, second] => format!("{first} and {second}"),
        _ => {
            let (last, rest) = names.split_last().unwrap();
            format!("{} and {}", rest.join(", "), last)
        }
    }
}

fn format_bibliography_author(author: &str) -> String {
    let clean = clean_bib_name(author);
    let (given, last) = if let Some((last, given)) = clean.split_once(',') {
        (given.trim().to_owned(), last.trim().to_owned())
    } else {
        let mut parts: Vec<&str> = clean.split_whitespace().collect();
        let Some(last) = parts.pop() else {
            return String::new();
        };
        (parts.join(" "), last.to_owned())
    };
    let initials = initials(&given);
    if initials.is_empty() {
        last
    } else {
        format!("{initials} {last}")
    }
}

fn initials(given: &str) -> String {
    given
        .split_whitespace()
        .filter_map(|part| part.chars().find(|ch| ch.is_alphabetic()))
        .map(|ch| format!("{}.", ch.to_uppercase().collect::<String>()))
        .collect::<Vec<_>>()
        .join(" ")
}

fn alpha_prefix_from_name(name: &str) -> String {
    let letters: String = name
        .chars()
        .filter(|ch| ch.is_alphabetic())
        .take(3)
        .collect();
    titlecase_label_prefix(&letters)
}

fn alpha_prefix_from_key(key: &str) -> String {
    let letters: String = key
        .chars()
        .filter(|ch| ch.is_alphabetic())
        .take(3)
        .collect();
    titlecase_label_prefix(&letters)
}

fn titlecase_label_prefix(prefix: &str) -> String {
    let mut chars = prefix.chars();
    let Some(first) = chars.next() else {
        return String::new();
    };
    let mut out = first.to_uppercase().collect::<String>();
    out.push_str(&chars.flat_map(|ch| ch.to_lowercase()).collect::<String>());
    out
}

fn surname_initial(name: &str) -> Option<String> {
    name.chars()
        .find(|ch| ch.is_alphabetic())
        .map(|ch| ch.to_uppercase().collect())
}

fn year_suffix(year: &str) -> String {
    let digits: String = year.chars().filter(|ch| ch.is_ascii_digit()).collect();
    if digits.len() >= 2 {
        digits[digits.len() - 2..].to_owned()
    } else {
        digits
    }
}

fn clean_bib_name(name: &str) -> String {
    name.replace(['{', '}'], "")
        .replace("\\'", "")
        .replace("\\`", "")
        .replace("\\^", "")
        .replace("\\\"", "")
        .replace("\\~", "")
        .split_whitespace()
        .collect::<Vec<_>>()
        .join(" ")
}

struct LabelBook {
    lecture_number: Option<u32>,
    labels: HashMap<String, LabelInfo>,
    section_counts: Vec<u32>,
    equation: u32,
    figure: u32,
    statement: u32,
    algorithm: u32,
}

#[derive(Clone)]
struct LabelInfo {
    text: String,
    id: String,
}

impl LabelBook {
    fn new(lecture_number: Option<u32>) -> Self {
        Self {
            lecture_number,
            labels: HashMap::new(),
            section_counts: Vec::new(),
            equation: 0,
            figure: 0,
            statement: 0,
            algorithm: 0,
        }
    }

    fn empty() -> Self {
        Self::new(None)
    }

    fn number_blocks(&mut self, blocks: &mut [Block]) {
        for block in blocks {
            match block {
                Block::Heading {
                    level,
                    text,
                    label,
                    id,
                    number,
                } => {
                    self.bump_section(*level as usize);
                    *number = self.section_number();
                    *id = label
                        .clone()
                        .unwrap_or_else(|| slugify(&format!("{number}-{text}")));
                    if let Some(label) = label {
                        self.labels.insert(
                            label.clone(),
                            LabelInfo {
                                text: format!("Section {number}"),
                                id: id.clone(),
                            },
                        );
                    }
                }
                Block::DisplayMath {
                    label,
                    number,
                    source: _,
                } => {
                    self.equation += 1;
                    *number = Some(self.equation);
                    if let Some(label) = label {
                        self.labels.insert(
                            label.clone(),
                            LabelInfo {
                                text: format!("({})", self.equation),
                                id: format!("eq-{}", slugify(label)),
                            },
                        );
                    }
                }
                Block::Figure {
                    label,
                    number,
                    caption: _,
                    source: _,
                } => {
                    self.figure += 1;
                    *number = Some(self.figure);
                    if let Some(label) = label {
                        self.labels.insert(
                            label.clone(),
                            LabelInfo {
                                text: format!("Figure {}", self.figure),
                                id: format!("fig-{}", slugify(label)),
                            },
                        );
                    }
                }
                Block::Algorithm {
                    label,
                    number,
                    items,
                    title: _,
                } => {
                    self.algorithm += 1;
                    *number = Some(self.algorithm.to_string());
                    if let Some(label) = label {
                        self.labels.insert(
                            label.clone(),
                            LabelInfo {
                                text: format!("Algorithm {}", self.algorithm),
                                id: format!("algorithm-{}", slugify(label)),
                            },
                        );
                    }
                    for item in items {
                        if let PseudoItem::Step {
                            label: Some(label), ..
                        } = item
                        {
                            self.labels.insert(
                                label.clone(),
                                LabelInfo {
                                    text: format!("Algorithm {}", self.algorithm),
                                    id: format!("line-{}", slugify(label)),
                                },
                            );
                        }
                    }
                }
                Block::Environment {
                    kind,
                    label,
                    number,
                    body,
                    ..
                } => {
                    if *kind == EnvKind::Algorithm {
                        self.algorithm += 1;
                        *number = Some(self.algorithm.to_string());
                    } else if kind.numbered() {
                        self.statement += 1;
                        let num = match self.lecture_number {
                            Some(lec) => format!("{lec}.{}", self.statement),
                            None => self.statement.to_string(),
                        };
                        *number = Some(num.clone());
                        if let Some(label) = label {
                            self.labels.insert(
                                label.clone(),
                                LabelInfo {
                                    text: format!("{} {num}", kind.label()),
                                    id: format!("{}-{}", kind.class(), slugify(label)),
                                },
                            );
                        }
                    }
                    self.number_blocks(body);
                }
                Block::Paragraph(_)
                | Block::Comment { .. }
                | Block::List { .. }
                | Block::RawBlock(_)
                | Block::RawHtml(_) => {}
            }
        }
    }

    fn bump_section(&mut self, level: usize) {
        while self.section_counts.len() < level {
            self.section_counts.push(0);
        }
        self.section_counts.truncate(level);
        if let Some(last) = self.section_counts.last_mut() {
            *last += 1;
        }
    }

    fn section_number(&self) -> String {
        let mut parts = Vec::new();
        if let Some(lec) = self.lecture_number {
            parts.push(lec.to_string());
        }
        parts.extend(self.section_counts.iter().map(u32::to_string));
        parts.join(".")
    }

    fn resolve(&self, label: &str) -> Option<&LabelInfo> {
        self.labels.get(label)
    }
}

#[derive(Clone)]
struct Heading {
    level: u8,
    text: String,
    id: String,
    number: String,
}

fn collect_headings(blocks: &[Block]) -> Vec<Heading> {
    let mut headings = Vec::new();
    for block in blocks {
        match block {
            Block::Heading {
                level,
                text,
                id,
                number,
                ..
            } if *level <= 3 => headings.push(Heading {
                level: *level,
                text: text.clone(),
                id: id.clone(),
                number: number.clone(),
            }),
            Block::Environment { body, .. } => headings.extend(collect_headings(body)),
            _ => {}
        }
    }
    headings
}

fn render_document(
    config: &Config,
    title: &str,
    blocks: &[Block],
    headings: &[Heading],
    labels: &LabelBook,
    bib: &Bibliography,
) -> String {
    let mut html = String::new();
    write!(
        html,
        "<!doctype html>\n<html lang=\"en\">\n<head>\n\
         <meta charset=\"utf-8\">\n\
         <meta name=\"viewport\" content=\"width=device-width, initial-scale=1\">\n\
         <title>{} - {}</title>\n\
         <link rel=\"stylesheet\" href=\"https://cdn.jsdelivr.net/npm/katex@0.16.22/dist/katex.min.css\">\n\
         <style>{}</style>\n\
         <script>{}</script>\n\
         <script defer src=\"https://cdn.jsdelivr.net/npm/katex@0.16.22/dist/katex.min.js\"></script>\n\
         <script defer src=\"https://cdn.jsdelivr.net/npm/katex@0.16.22/dist/contrib/auto-render.min.js\" onload=\"renderMathInElement(document.body, {});requestAnimationFrame(window.markOverwideEquations);\"></script>\n\
         </head>\n<body>\n",
        escape_html(title),
        escape_html(&config.site_title),
        page_css(),
        equation_width_script(),
        katex_config()
    )
    .unwrap();

    let current_chapter = current_chapter(config);
    if current_chapter.is_none() {
        html.push_str("<header class=\"site-masthead\">\n");
        write!(
            html,
            "<div class=\"course-title\">{}</div>\n<div class=\"course-authors\">{}</div>\n<nav class=\"top-links\" aria-label=\"Page links\">\n",
            escape_html(&config.site_title),
            escape_html(&config.authors)
        )
        .unwrap();
        if let Some(index) = &config.index_href {
            write!(html, "<a href=\"{}\">Home</a>\n", escape_attr(index)).unwrap();
        }
        if let Some(pdf) = &config.pdf_href {
            write!(html, "<a href=\"{}\">PDF</a>\n", escape_attr(pdf)).unwrap();
        }
        html.push_str("</nav>\n</header>\n");
    }

    if let Some(current) = current_chapter {
        html.push_str(&render_chapter_rail(current, headings, labels, bib, config));
    }

    html.push_str("<main class=\"page-shell\">\n");
    if let Some(current) = current_chapter {
        write!(
            html,
            "<div class=\"chapter-kicker\">Chapter {}</div>\n",
            CHAPTERS[current].number
        )
        .unwrap();
    }
    write!(
        html,
        "<h1 class=\"lecture-title\">{}</h1>\n",
        escape_html(title)
    )
    .unwrap();
    if let Some(current) = current_chapter {
        html.push_str(&render_chapter_citation_sidenote(
            current,
            title,
            &config.site_title,
            config.pdf_href.as_deref(),
        ));
    }
    if !headings.is_empty() {
        html.push_str(&render_toc(headings, labels, bib));
    }
    html.push_str("<article class=\"lecture-content\">\n");
    let mut render_state = RenderState::default();
    html.push_str(&render_blocks(
        blocks,
        labels,
        bib,
        &mut render_state,
        config,
    ));
    html.push_str(&render_bibliography(bib, &render_state));
    html.push_str(&render_endnotes(&render_state));
    html.push_str("</article>\n</main>\n");
    if current_chapter.is_some() {
        html.push_str(chapter_nav_script());
    }
    html.push_str("</body>\n</html>\n");
    html
}

fn current_chapter(config: &Config) -> Option<usize> {
    let file_name = config.input.file_name()?.to_string_lossy();
    CHAPTERS
        .iter()
        .position(|chapter| chapter.source == file_name.as_ref())
}

fn render_chapter_citation_sidenote(
    current: usize,
    title: &str,
    site_title: &str,
    pdf_href: Option<&str>,
) -> String {
    let chapter = CHAPTERS[current];
    let key = format!(
        "anagnostides-farina-zhang-2026-phi-equilibria-chapter-{}",
        chapter.number
    );
    let bibtex = format!(
        "@misc{{{key},\n  author = {{{}}},\n  title = {{{}}},\n  booktitle = {{{}}},\n  note = {{Chapter {} of the ACM EC 2026 tutorial notes}},\n  year = {{2026}},\n  url = {{{}}}\n}}",
        BIBTEX_AUTHORS,
        bibtex_escape(&format!("Chapter {}: {title}", chapter.number)),
        bibtex_escape(site_title),
        chapter.number,
        chapter.href
    );
    let mut out = String::from(
        "<aside class=\"chapter-citation-sidenote\" aria-label=\"Chapter links and citation\">",
    );
    if let Some(pdf) = pdf_href {
        write!(
            out,
            "<a class=\"chapter-citation-pdf\" href=\"{}\">Download as PDF</a>",
            escape_attr(pdf)
        )
        .unwrap();
    }
    write!(
        out,
        "<details class=\"chapter-citation-details\">\
         <summary class=\"chapter-citation-title\">How to cite</summary>\
         <pre><code>{}</code></pre>\
         </details>\
         </aside>\n",
        escape_html(&bibtex)
    )
    .unwrap();
    out
}

fn bibtex_escape(input: &str) -> String {
    input.replace('&', "\\&")
}

fn render_chapter_rail(
    current: usize,
    headings: &[Heading],
    labels: &LabelBook,
    bib: &Bibliography,
    config: &Config,
) -> String {
    let mut out = String::from("<nav class=\"chapter-rail\" aria-label=\"Chapters\">\n");
    out.push_str("<div class=\"chapter-rail-course\"><div class=\"course-event\">ACM EC&rsquo;26 Tutorial</div>");
    if let Some(index) = &config.index_href {
        write!(
            out,
            "<a class=\"course-title course-title-link\" href=\"{}\">{}</a>",
            escape_attr(index),
            escape_html(&config.site_title)
        )
        .unwrap();
    } else {
        write!(
            out,
            "<div class=\"course-title\">{}</div>",
            escape_html(&config.site_title)
        )
        .unwrap();
    }
    write!(
        out,
        "<div class=\"course-authors\">{}</div>",
        escape_html(&config.authors)
    )
    .unwrap();
    out.push_str("</div>\n");
    out.push_str("<div class=\"chapter-rail-heading\">Chapters</div>\n");
    for (idx, chapter) in CHAPTERS.iter().enumerate() {
        let class = if idx == current {
            "chapter-rail-link is-current"
        } else {
            "chapter-rail-link"
        };
        let aria = if idx == current {
            " aria-current=\"page\""
        } else {
            ""
        };
        write!(
            out,
            "<a class=\"{}\" href=\"{}\"{}><span>{}</span>{}</a>\n",
            class,
            escape_attr(chapter.href),
            aria,
            chapter.number,
            escape_html(chapter.short_title)
        )
        .unwrap();
    }
    if !headings.is_empty() {
        out.push_str(
            "<div class=\"chapter-rail-heading chapter-rail-section-heading\">This Chapter</div>\n",
        );
        let mut state = RenderState::default();
        for heading in headings {
            write!(
                out,
                "<a class=\"chapter-section-link chapter-section-l{}\" href=\"#{}\" data-section-link=\"{}\"><span class=\"chapter-section-no\">{}</span><span class=\"chapter-section-title\">{}</span></a>\n",
                heading.level,
                escape_attr(&heading.id),
                escape_attr(&heading.id),
                escape_html(&heading.number),
                render_inline(&heading.text, labels, bib, &mut state)
            )
            .unwrap();
        }
    }
    out.push_str("</nav>\n");
    out
}

fn chapter_nav_script() -> &'static str {
    r##"<script>
(() => {
  const links = Array.from(document.querySelectorAll("[data-section-link]"));
  if (!links.length) return;
  const byId = new Map(links.map((link) => [link.getAttribute("data-section-link"), link]));
  const sections = Array.from(byId.keys())
    .map((id) => document.getElementById(id))
    .filter(Boolean);
  const setActive = (id) => {
    for (const link of links) {
      const active = link.getAttribute("data-section-link") === id;
      link.classList.toggle("is-active", active);
      if (active) link.setAttribute("aria-current", "location");
      else link.removeAttribute("aria-current");
    }
  };
  const update = () => {
    const y = window.scrollY + 130;
    let current = sections[0]?.id;
    for (const section of sections) {
      if (section.offsetTop <= y) current = section.id;
      else break;
    }
    if (current) setActive(current);
  };
  update();
  window.setTimeout(update, 0);
  window.setTimeout(update, 150);
  document.addEventListener("scroll", update, { passive: true });
  window.addEventListener("resize", update);
  window.addEventListener("hashchange", update);
})();
</script>
"##
}

fn render_toc(headings: &[Heading], labels: &LabelBook, bib: &Bibliography) -> String {
    let mut out = String::from("<nav class=\"toc\" aria-label=\"Contents\"><ol>\n");
    let mut state = RenderState::default();
    for heading in headings {
        write!(
            out,
            "<li class=\"toc-l{}\"><a href=\"#{}\"><span class=\"toc-no\">{}</span><span class=\"toc-title\">{}</span></a></li>\n",
            heading.level,
            escape_attr(&heading.id),
            escape_html(&heading.number),
            render_inline(&heading.text, labels, bib, &mut state)
        )
        .unwrap();
    }
    out.push_str("</ol></nav>\n");
    out
}

#[derive(Default)]
struct RenderState {
    footnotes: u32,
    endnotes: Vec<(u32, String)>,
    citations: Vec<String>,
    content_offset: usize,
    citation_note_offsets: HashMap<String, usize>,
}

impl RenderState {
    fn advance_text(&mut self, text: &str) {
        self.content_offset += text.chars().count();
    }

    fn should_show_citation_note(&mut self, key: &str) -> bool {
        let show = self.citation_note_offsets.get(key).is_none_or(|last| {
            self.content_offset.saturating_sub(*last) >= CITATION_MARGIN_NOTE_MIN_CHARS
        });
        if show {
            self.citation_note_offsets
                .insert(key.to_owned(), self.content_offset);
        }
        show
    }
}

struct FigureAsset {
    stem: String,
    src: String,
    css_width: Option<String>,
}

fn render_figure_svg(config: &Config, label: &str, source: Option<&str>) -> Option<FigureAsset> {
    let stem =
        figure_stem_from_label(label).or_else(|| source.and_then(figure_stem_from_source))?;
    let typst_source = config.root.join("figures").join(format!("{stem}.typ"));
    if !typst_source.exists() {
        return None;
    }

    let output_dir = config.output.parent().unwrap_or_else(|| Path::new("."));
    let asset_dir = output_dir.join("figures");
    fs::create_dir_all(&asset_dir).ok()?;
    let svg_output = asset_dir.join(format!("{stem}.svg"));

    let status = Command::new("typst")
        .arg("compile")
        .arg("--root")
        .arg(&config.root)
        .arg(&typst_source)
        .arg(&svg_output)
        .status()
        .ok()?;
    status.success().then(|| FigureAsset {
        src: format!("figures/{stem}.svg"),
        css_width: svg_css_width(&svg_output),
        stem,
    })
}

fn figure_stem_from_label(label: &str) -> Option<String> {
    let stem = label.strip_prefix("fig:").unwrap_or(label);
    sanitize_figure_stem(stem)
}

fn figure_stem_from_source(source: &str) -> Option<String> {
    source
        .split(|ch: char| !(ch.is_ascii_alphanumeric() || ch == '_' || ch == '-'))
        .find_map(|token| token.strip_prefix("fig_"))
        .and_then(|stem| sanitize_figure_stem(&stem.replace('_', "-")))
}

fn sanitize_figure_stem(stem: &str) -> Option<String> {
    let stem = stem.trim_matches('-');
    if stem.is_empty() {
        return None;
    }
    let clean: String = stem
        .chars()
        .map(|ch| {
            if ch.is_ascii_alphanumeric() || ch == '-' || ch == '_' {
                ch
            } else {
                '-'
            }
        })
        .collect();
    (!clean.is_empty()).then_some(clean)
}

fn strip_comment_figure_lines(body: &str) -> String {
    body.lines()
        .filter(|line| !(line.contains("fig_") && line.contains(".body")))
        .map(str::trim)
        .filter(|line| !line.is_empty())
        .collect::<Vec<_>>()
        .join("\n")
}

fn svg_css_width(path: &Path) -> Option<String> {
    let svg = fs::read_to_string(path).ok()?;
    let value = svg_root_attr(&svg, "width")?;
    svg_length_to_css_px(value)
}

fn svg_root_attr<'a>(svg: &'a str, attr: &str) -> Option<&'a str> {
    let root_end = svg.find('>')?;
    let root = &svg[..root_end];
    let needle = format!("{attr}=\"");
    let start = root.find(&needle)? + needle.len();
    let rest = &root[start..];
    let end = rest.find('"')?;
    Some(&rest[..end])
}

fn svg_length_to_css_px(value: &str) -> Option<String> {
    let value = value.trim();
    let (number, multiplier) = if let Some(number) = value.strip_suffix("pt") {
        (number, 4.0 / 3.0)
    } else if let Some(number) = value.strip_suffix("px") {
        (number, 1.0)
    } else {
        (value, 1.0)
    };
    let px = number.trim().parse::<f64>().ok()? * multiplier;
    Some(format_css_px(px))
}

fn format_css_px(px: f64) -> String {
    let mut text = format!("{px:.3}");
    while text.contains('.') && text.ends_with('0') {
        text.pop();
    }
    if text.ends_with('.') {
        text.pop();
    }
    format!("{text}px")
}

fn figure_img_style(asset: &FigureAsset) -> String {
    if asset.stem == "mcroute" {
        return " style=\"width:80%\"".to_owned();
    }
    asset
        .css_width
        .as_deref()
        .map(|width| format!(" style=\"width:min(100%, {})\"", escape_attr(width)))
        .unwrap_or_default()
}

fn render_blocks(
    blocks: &[Block],
    labels: &LabelBook,
    bib: &Bibliography,
    state: &mut RenderState,
    config: &Config,
) -> String {
    let mut out = String::new();
    for block in blocks {
        match block {
            Block::Heading {
                level,
                text,
                id,
                number,
                ..
            } => {
                write!(
                    out,
                    "<h{level} id=\"{}\"><span class=\"secno\">{}</span> {}</h{level}>\n",
                    escape_attr(id),
                    escape_html(number),
                    render_inline(text, labels, bib, state)
                )
                .unwrap();
            }
            Block::Paragraph(text) => {
                write!(out, "<p>{}</p>\n", render_inline(text, labels, bib, state)).unwrap();
            }
            Block::DisplayMath {
                source,
                label,
                number,
            } => {
                let id = label
                    .as_ref()
                    .map(|label| format!(" id=\"eq-{}\"", escape_attr(&slugify(label))))
                    .unwrap_or_default();
                write!(
                    out,
                    "<div class=\"equation\"{}>\\[{}\\]",
                    id,
                    display_math_to_tex(source)
                )
                .unwrap();
                if let Some(number) = number {
                    write!(out, "<span class=\"eqno\">({number})</span>").unwrap();
                }
                out.push_str("</div>\n");
            }
            Block::Environment {
                kind,
                title,
                body,
                label,
                number,
            } => {
                let id = label
                    .as_ref()
                    .map(|label| {
                        format!(" id=\"{}-{}\"", kind.class(), escape_attr(&slugify(label)))
                    })
                    .unwrap_or_default();
                write!(out, "<section class=\"env {}\"{}>", kind.class(), id).unwrap();
                out.push_str(&render_environment_body(
                    *kind, title, number, body, labels, bib, state, config,
                ));
                out.push_str("</section>\n");
            }
            Block::Figure {
                caption,
                label,
                number,
                source,
            } => {
                let id = label
                    .as_ref()
                    .map(|label| format!(" id=\"fig-{}\"", escape_attr(&slugify(label))))
                    .unwrap_or_default();
                let src = label
                    .as_ref()
                    .and_then(|label| render_figure_svg(config, label, source.as_deref()));
                if let Some(asset) = src {
                    write!(out, "<figure class=\"rendered-figure\"{}>", id).unwrap();
                    let alt = caption
                        .as_ref()
                        .map(|caption| plain_text(caption))
                        .unwrap_or_else(|| {
                            number
                                .map(|number| format!("Figure {number}"))
                                .unwrap_or_else(|| "Figure".to_owned())
                        });
                    write!(
                        out,
                        "<img src=\"{}\" alt=\"{}\"{}>",
                        escape_attr(&asset.src),
                        escape_attr(&alt),
                        figure_img_style(&asset)
                    )
                    .unwrap();
                } else {
                    write!(out, "<figure class=\"figure-placeholder\"{}>", id).unwrap();
                    out.push_str("<div>Figure generated by Typst in the PDF version.</div>");
                }
                if let Some(caption) = caption {
                    write!(
                        out,
                        "<figcaption><strong>Figure {}.</strong> {}</figcaption>",
                        number.unwrap_or(0),
                        render_inline(caption, labels, bib, state)
                    )
                    .unwrap();
                }
                out.push_str("</figure>\n");
            }
            Block::Comment { text, figure_stem } => {
                out.push_str("<aside class=\"special-comment\">");
                if !text.trim().is_empty() {
                    write!(
                        out,
                        "<div class=\"special-comment-text\">{}</div>",
                        render_inline(text, labels, bib, state)
                    )
                    .unwrap();
                }
                if let Some(stem) = figure_stem {
                    if let Some(asset) =
                        render_figure_svg(config, &format!("fig:{stem}"), Some(stem.as_str()))
                    {
                        write!(
                            out,
                            "<img class=\"special-comment-figure\" src=\"{}\" alt=\"{}\"{}>",
                            escape_attr(&asset.src),
                            escape_attr(&plain_text(text)),
                            figure_img_style(&asset)
                        )
                        .unwrap();
                    }
                }
                out.push_str("</aside>\n");
            }
            Block::Algorithm {
                title,
                label,
                items,
                number,
            } => {
                let id = label
                    .as_ref()
                    .map(|label| format!(" id=\"algorithm-{}\"", escape_attr(&slugify(label))))
                    .unwrap_or_default();
                write!(out, "<section class=\"env algorithm\"{}>", id).unwrap();
                let rendered_title = title
                    .as_ref()
                    .map(|title| clean_algorithm_title(title))
                    .filter(|title| !title.trim().is_empty())
                    .map(|title| render_inline(&title, labels, bib, state));
                write!(
                    out,
                    "<div class=\"env-title\"><span class=\"algorithm-label\">Algorithm {}</span>{}</div>\n",
                    escape_html(number.as_deref().unwrap_or("")),
                    rendered_title
                        .as_ref()
                        .map(|title| format!(" <span class=\"algorithm-caption\">({title})</span>"))
                        .unwrap_or_default()
                )
                .unwrap();
                out.push_str("<div class=\"pseudocode\">\n");
                let base_indent = pseudo_base_indent(items);
                for (idx, item) in items.iter().enumerate() {
                    let indent = pseudo_item_indent(item).saturating_sub(base_indent);
                    let next_indent = items
                        .get(idx + 1)
                        .map(|item| pseudo_item_indent(item).saturating_sub(base_indent))
                        .unwrap_or(0);
                    let guides = render_pseudo_guides(indent, next_indent);
                    match item {
                        PseudoItem::Step { text, label, .. } => {
                            let id = label
                                .as_ref()
                                .map(|label| {
                                    format!(" id=\"line-{}\"", escape_attr(&slugify(label)))
                                })
                                .unwrap_or_default();
                            write!(
                                out,
                                "<div class=\"pseudo-line\"{} style=\"--indent:{}\">{}<span class=\"pseudo-text\">{}</span></div>\n",
                                id,
                                indent,
                                guides,
                                render_inline(text, labels, bib, state)
                            )
                            .unwrap();
                        }
                        PseudoItem::Math { source, .. } => {
                            write!(
                                out,
                                "<div class=\"pseudo-math\" style=\"--indent:{}\">{}\\[{}\\]</div>\n",
                                indent,
                                guides,
                                display_math_to_tex(source)
                            )
                            .unwrap();
                        }
                    }
                }
                out.push_str("</div>\n</section>\n");
            }
            Block::List { ordered, items } => {
                out.push_str(if *ordered { "<ol>\n" } else { "<ul>\n" });
                for item in items {
                    write!(
                        out,
                        "<li>{}</li>\n",
                        render_inline(item, labels, bib, state)
                    )
                    .unwrap();
                }
                out.push_str(if *ordered { "</ol>\n" } else { "</ul>\n" });
            }
            Block::RawBlock(raw) => {
                write!(
                    out,
                    "<pre class=\"raw-block\"><code>{}</code></pre>\n",
                    escape_html(raw)
                )
                .unwrap();
            }
            Block::RawHtml(raw) => out.push_str(raw),
        }
    }
    out
}

fn pseudo_item_indent(item: &PseudoItem) -> usize {
    match item {
        PseudoItem::Step { indent, .. } | PseudoItem::Math { indent, .. } => *indent,
    }
}

fn pseudo_base_indent(items: &[PseudoItem]) -> usize {
    items.iter().map(pseudo_item_indent).min().unwrap_or(0)
}

fn render_pseudo_guides(indent: usize, next_indent: usize) -> String {
    let mut out = String::new();
    for guide in 1..=indent {
        let end_class = if next_indent < guide {
            " pseudo-guide-end"
        } else {
            ""
        };
        write!(
            out,
            "<span class=\"pseudo-guide{}\" style=\"--guide:{}\"></span>",
            end_class, guide
        )
        .unwrap();
    }
    out
}

fn render_environment_body(
    kind: EnvKind,
    title: &Option<String>,
    number: &Option<String>,
    body: &[Block],
    labels: &LabelBook,
    bib: &Bibliography,
    state: &mut RenderState,
    config: &Config,
) -> String {
    let mut out = String::new();
    out.push_str("<p>");
    out.push_str("<span class=\"env-title\">");
    out.push_str(kind.label());
    if let Some(number) = number {
        write!(out, " {}", escape_html(number)).unwrap();
    }
    if let Some(title) = title {
        write!(
            out,
            " <span>({})</span>",
            render_inline(title, labels, bib, state)
        )
        .unwrap();
    }
    out.push_str(".</span>&nbsp;&nbsp;");

    if let Some((Block::Paragraph(first), rest)) = body.split_first() {
        out.push_str(&render_inline(first, labels, bib, state));
        out.push_str("</p>\n");
        out.push_str(&render_blocks(rest, labels, bib, state, config));
    } else {
        out.push_str("</p>\n");
        out.push_str(&render_blocks(body, labels, bib, state, config));
    }
    out
}

fn render_inline(
    text: &str,
    labels: &LabelBook,
    bib: &Bibliography,
    state: &mut RenderState,
) -> String {
    let mut out = String::new();
    let mut emph_open = false;
    let mut strong_open = false;
    let mut i = 0;

    while i < text.len() {
        let tail = &text[i..];
        if tail.starts_with("#footnote[") {
            if let Some((whole, body)) = parse_square_balanced(tail, "#footnote") {
                out.push_str(&render_sidenote(&body, labels, bib, state));
                i += whole;
                continue;
            }
        }

        if tail.starts_with('$') {
            if let Some(end) = tail[1..].find('$') {
                let math = &tail[1..1 + end];
                write!(out, "\\({}\\)", typst_inline_math_to_tex(math)).unwrap();
                state.advance_text(math);
                i += end + 2;
                continue;
            }
        }

        let next_math = tail.find('$').unwrap_or(tail.len());
        let next_footnote = tail.find("#footnote[").unwrap_or(tail.len());
        let take = next_math.min(next_footnote);
        if take > 0 {
            out.push_str(&render_markup(
                &tail[..take],
                labels,
                bib,
                state,
                &mut emph_open,
                &mut strong_open,
            ));
            i += take;
        } else {
            let ch = tail.chars().next().unwrap();
            out.push_str(&escape_html(&ch.to_string()));
            state.advance_text(&ch.to_string());
            i += ch.len_utf8();
        }
    }

    if strong_open {
        out.push_str("</strong>");
    }
    if emph_open {
        out.push_str("</em>");
    }
    out
}

fn render_markup(
    text: &str,
    labels: &LabelBook,
    bib: &Bibliography,
    state: &mut RenderState,
    emph_open: &mut bool,
    strong_open: &mut bool,
) -> String {
    let mut out = String::new();
    let mut i = 0;
    while i < text.len() {
        let tail = &text[i..];
        if let Some((whole, key, textual)) = parse_citation(tail) {
            out.push_str(&render_citation(key, textual, bib, state));
            i += whole;
        } else if tail.starts_with("TreeSwap") {
            out.push_str("<code>TreeSwap</code>");
            state.advance_text("TreeSwap");
            i += "TreeSwap".len();
        } else if let Some((whole, label)) = parse_ref(tail) {
            if let Some(info) = labels.resolve(label) {
                let display_text = reference_text_for_context(
                    info,
                    text[..i].chars().next_back(),
                    tail[whole..].chars().next(),
                );
                write!(
                    out,
                    "<a class=\"ref-link\" href=\"#{}\">{}</a>",
                    escape_attr(&info.id),
                    escape_html(&display_text)
                )
                .unwrap();
                state.advance_text(&display_text);
            } else {
                write!(
                    out,
                    "<span class=\"missing-ref\">{}</span>",
                    escape_html(label)
                )
                .unwrap();
                state.advance_text(label);
            }
            i += whole;
        } else if tail.starts_with("#todo[") {
            if let Some((whole, body)) = parse_square_command(tail, "#todo") {
                write!(out, "<mark class=\"todo\">{}</mark>", escape_html(body)).unwrap();
                state.advance_text(body);
                i += whole;
            } else {
                out.push('#');
                state.advance_text("#");
                i += 1;
            }
        } else if tail.starts_with("#todo(") {
            if let Some((whole, body)) = parse_paren_command(tail, "#todo") {
                write!(out, "<mark class=\"todo\">{}</mark>", escape_html(&body)).unwrap();
                state.advance_text(&body);
                i += whole;
            } else {
                out.push('#');
                state.advance_text("#");
                i += 1;
            }
        } else if tail.starts_with("#set ") {
            if let Some(whole) = parse_set_command(tail) {
                i += whole;
            } else {
                out.push('#');
                state.advance_text("#");
                i += 1;
            }
        } else if tail.starts_with("#h(") || tail.starts_with("#v(") {
            if let Some((whole, _body)) = parse_layout_command(tail) {
                i += whole;
            } else {
                out.push('#');
                state.advance_text("#");
                i += 1;
            }
        } else if tail.starts_with("#footnote[") {
            if let Some((whole, body)) = parse_square_balanced(tail, "#footnote") {
                out.push_str(&render_sidenote(&body, labels, bib, state));
                i += whole;
            } else {
                out.push('#');
                i += 1;
            }
        } else if tail.starts_with('`') {
            if let Some(end) = tail[1..].find('`') {
                write!(out, "<code>{}</code>", escape_html(&tail[1..1 + end])).unwrap();
                i += end + 2;
            } else {
                out.push('`');
                state.advance_text("`");
                i += 1;
            }
        } else {
            let ch = tail.chars().next().unwrap();
            if ch == '_' {
                if *emph_open {
                    out.push_str("</em>");
                } else {
                    out.push_str("<em>");
                }
                *emph_open = !*emph_open;
            } else if ch == '*' {
                if *strong_open {
                    out.push_str("</strong>");
                } else {
                    out.push_str("<strong>");
                }
                *strong_open = !*strong_open;
            } else if let Some(entity) = smart_quote_entity(
                ch,
                text[..i].chars().next_back(),
                tail[ch.len_utf8()..].chars().next(),
            ) {
                out.push_str(entity);
                state.advance_text(&ch.to_string());
            } else if ch == '~' {
                out.push_str("&nbsp;");
                state.advance_text(" ");
            } else {
                out.push_str(&escape_html(&ch.to_string()));
                state.advance_text(&ch.to_string());
            }
            i += ch.len_utf8();
        }
    }
    out.replace("---", "&mdash;")
}

fn reference_text_for_context(
    info: &LabelInfo,
    before: Option<char>,
    after: Option<char>,
) -> String {
    if info.id.starts_with("eq-") && before == Some('(') && after == Some(')') {
        info.text
            .strip_prefix('(')
            .and_then(|text| text.strip_suffix(')'))
            .unwrap_or(&info.text)
            .to_owned()
    } else {
        info.text.clone()
    }
}

fn render_sidenote(
    body: &str,
    labels: &LabelBook,
    bib: &Bibliography,
    state: &mut RenderState,
) -> String {
    state.footnotes += 1;
    let number = state.footnotes;
    let body = render_inline(body, labels, bib, state);
    state.endnotes.push((number, body.clone()));
    format!(
        "<sup class=\"footnote-ref\" id=\"fnref-{number}\"><a href=\"#fn-end-{number}\">{number}</a></sup>\
         <span class=\"footnote\" id=\"fn-side-{number}\"><span class=\"footnote-num\">{number}</span>{body}</span>"
    )
}

fn clean_algorithm_title(title: &str) -> String {
    let mut title = title.replace("#h(1fr)", "");
    let trimmed = title.trim();
    for prefix in ["*Algorithm*:", "*Algorithm*"] {
        if let Some(rest) = trimmed.strip_prefix(prefix) {
            title = rest.trim_start_matches(':').trim().to_owned();
            break;
        }
    }
    title
}

fn parse_citation(input: &str) -> Option<(usize, &str, bool)> {
    let (prefix, textual) = if input.starts_with("#citet(<") {
        ("#citet(<", true)
    } else if input.starts_with("#citep(<") {
        ("#citep(<", false)
    } else if input.starts_with("#cite(<") {
        ("#cite(<", false)
    } else {
        return None;
    };
    let rest = &input[prefix.len()..];
    let end = rest.find(">)")?;
    Some((prefix.len() + end + 2, &rest[..end], textual))
}

fn render_citation(
    key: &str,
    textual: bool,
    bib: &Bibliography,
    state: &mut RenderState,
) -> String {
    if !state.citations.iter().any(|seen| seen == key) {
        state.citations.push(key.to_owned());
    }
    let id = slugify(key);
    let href = format!("bib-{id}");
    let label = bib.citation_key(key);
    let cite_text = bib.cite(key, textual);
    let note = if state.should_show_citation_note(key) {
        let entry = bib.full_entry(key).unwrap_or_else(|| key.to_owned());
        format!(
            "<span class=\"citation-note\"><a class=\"citation-note-key\" href=\"#{}\">{}</a> {}</span>",
            escape_attr(&href),
            escape_html(&label),
            render_bib_text(&entry)
        )
    } else {
        String::new()
    };
    state.advance_text(&cite_text);
    format!(
        "<span class=\"citation-wrap\"><a class=\"citation\" href=\"#{}\">{}</a>{}</span>",
        escape_attr(&href),
        escape_html(&cite_text),
        note
    )
}

fn render_bibliography(bib: &Bibliography, state: &RenderState) -> String {
    if state.citations.is_empty() {
        return String::new();
    }
    let mut out = String::from(
        "<section class=\"bibliography\" id=\"bibliography\"><h1>Bibliography for this chapter</h1>\n",
    );
    for key in &state.citations {
        let entry = bib.full_entry(key).unwrap_or_else(|| key.clone());
        write!(
            out,
            "<p id=\"bib-{}\"><span class=\"bib-key\">{}</span><span class=\"bib-entry\">{}</span></p>\n",
            escape_attr(&slugify(key)),
            escape_html(&bib.citation_key(key)),
            render_bib_text(&entry)
        )
        .unwrap();
    }
    out.push_str("</section>\n");
    out
}

fn render_endnotes(state: &RenderState) -> String {
    if state.endnotes.is_empty() {
        return String::new();
    }
    let mut out = String::from("<section class=\"endnotes\" id=\"footnotes\"><h1>Footnotes</h1>\n");
    for (number, body) in &state.endnotes {
        write!(
            out,
            "<p id=\"fn-end-{number}\"><a class=\"footnote-backref\" href=\"#fnref-{number}\">{number}</a><span class=\"footnote-body\">{body}</span></p>\n"
        )
        .unwrap();
    }
    out.push_str("</section>\n");
    out
}

fn render_bib_text(text: &str) -> String {
    let mut out = escape_html(&normalize_bib_latex_text(text));
    out = out.replace("doi:", "DOI: ");
    out = link_arxiv_refs(&out);
    out
}

fn arxiv_id_from_entry(entry: &BibEntry) -> Option<String> {
    [&entry.journal, &entry.doi, &entry.url, &entry.eprint]
        .iter()
        .find_map(|text| arxiv_id_in_text(text))
}

fn arxiv_id_in_text(text: &str) -> Option<String> {
    for marker in ["arXiv:", "arXiv.", "arxiv.org/abs/", "arxiv.org/pdf/"] {
        if let Some(pos) = text.find(marker) {
            let start = pos + marker.len();
            if let Some(id) = consume_arxiv_id(&text[start..]) {
                return Some(id);
            }
        }
    }
    consume_arxiv_id(text.trim())
}

fn link_arxiv_refs(input: &str) -> String {
    let mut out = String::new();
    let mut i = 0;
    while i < input.len() {
        let tail = &input[i..];
        let match_info = if tail.starts_with("arXiv:") {
            consume_arxiv_id(&tail["arXiv:".len()..]).map(|id| ("arXiv:".len() + id.len(), id))
        } else if tail.starts_with("arXiv.") {
            consume_arxiv_id(&tail["arXiv.".len()..]).map(|id| ("arXiv.".len() + id.len(), id))
        } else if tail.starts_with("https://arxiv.org/abs/")
            || tail.starts_with("http://arxiv.org/abs/")
        {
            let prefix = if tail.starts_with("https://") {
                "https://arxiv.org/abs/"
            } else {
                "http://arxiv.org/abs/"
            };
            consume_arxiv_id(&tail[prefix.len()..]).map(|id| (prefix.len() + id.len(), id))
        } else {
            None
        };

        if let Some((whole, id)) = match_info {
            let label = &tail[..whole];
            write!(
                out,
                "<a href=\"https://arxiv.org/abs/{}\">{}</a>",
                escape_attr(&id),
                label
            )
            .unwrap();
            i += whole;
        } else {
            let ch = tail.chars().next().unwrap();
            out.push(ch);
            i += ch.len_utf8();
        }
    }
    out
}

fn consume_arxiv_id(input: &str) -> Option<String> {
    let mut chars = input.char_indices().peekable();
    let mut end = 0;
    let mut saw_dot = false;
    let mut saw_digit = false;
    while let Some((idx, ch)) = chars.peek().copied() {
        if ch.is_ascii_digit() {
            saw_digit = true;
            end = idx + ch.len_utf8();
            chars.next();
        } else if ch == '.' {
            saw_dot = true;
            end = idx + ch.len_utf8();
            chars.next();
        } else if ch == 'v'
            && chars
                .clone()
                .nth(1)
                .is_some_and(|(_, next)| next.is_ascii_digit())
        {
            end = idx + ch.len_utf8();
            chars.next();
        } else {
            break;
        }
    }
    (saw_digit && saw_dot).then(|| input[..end].trim_end_matches('.').to_owned())
}

fn normalize_bib_latex_text(text: &str) -> String {
    let mut out = String::new();
    let mut chars = text.chars().peekable();
    while let Some(ch) = chars.next() {
        match ch {
            '{' | '}' => {}
            '\\' => match chars.next() {
                Some('&') => out.push('&'),
                Some('%') => out.push('%'),
                Some('$') => out.push('$'),
                Some('#') => out.push('#'),
                Some('_') => out.push('_'),
                Some('{') => out.push('{'),
                Some('}') => out.push('}'),
                Some('~') => out.push(' '),
                Some(other) => {
                    out.push('\\');
                    out.push(other);
                }
                None => out.push('\\'),
            },
            _ => out.push(ch),
        }
    }
    out.split_whitespace().collect::<Vec<_>>().join(" ")
}

fn parse_ref(input: &str) -> Option<(usize, &str)> {
    if input.starts_with('@') {
        let rest = &input[1..];
        let end = rest
            .find(|ch: char| !(ch.is_ascii_alphanumeric() || ch == ':' || ch == '-' || ch == '_'))
            .unwrap_or(rest.len());
        if end > 0 {
            return Some((1 + end, &rest[..end]));
        }
    }
    None
}

fn parse_square_command<'a>(input: &'a str, command: &str) -> Option<(usize, &'a str)> {
    let rest = input.strip_prefix(command)?.strip_prefix('[')?;
    let end = rest.find(']')?;
    Some((command.len() + 1 + end + 1, &rest[..end]))
}

fn parse_paren_command(input: &str, command: &str) -> Option<(usize, String)> {
    let rest = input.strip_prefix(command)?.strip_prefix('(')?;
    let mut depth = 1i32;
    let mut body = String::new();
    let mut in_string = false;
    let mut escaped = false;
    for (idx, ch) in rest.char_indices() {
        if escaped {
            body.push(ch);
            escaped = false;
            continue;
        }
        if ch == '\\' {
            body.push(ch);
            escaped = true;
            continue;
        }
        if ch == '"' {
            in_string = !in_string;
            body.push(ch);
            continue;
        }
        if !in_string && ch == '(' {
            depth += 1;
            body.push(ch);
        } else if !in_string && ch == ')' {
            depth -= 1;
            if depth == 0 {
                let body = body.trim();
                let body = parse_typst_string(body).unwrap_or_else(|| body.to_owned());
                return Some((command.len() + 1 + idx + 1, body));
            }
            body.push(ch);
        } else {
            body.push(ch);
        }
    }
    None
}

fn parse_set_command(input: &str) -> Option<usize> {
    let rest = input.strip_prefix("#set ")?;
    let open = input.len() - rest.len() + rest.find('(')?;
    let close = find_matching_paren(input, open)?;
    Some(close + 1)
}

fn parse_layout_command(input: &str) -> Option<(usize, &str)> {
    let command = if input.starts_with("#h(") {
        "#h"
    } else if input.starts_with("#v(") {
        "#v"
    } else {
        return None;
    };
    let rest = input.strip_prefix(command)?.strip_prefix('(')?;
    let end = rest.find(')')?;
    Some((command.len() + 1 + end + 1, &rest[..end]))
}

fn parse_square_balanced(input: &str, command: &str) -> Option<(usize, String)> {
    let rest = input.strip_prefix(command)?;
    let mut chars = rest.char_indices();
    let (_, first) = chars.next()?;
    if first != '[' {
        return None;
    }

    let mut depth = 1i32;
    let mut body = String::new();
    for (idx, ch) in chars {
        if ch == '[' {
            depth += 1;
            body.push(ch);
        } else if ch == ']' {
            depth -= 1;
            if depth == 0 {
                return Some((command.len() + idx + 1, body));
            }
            body.push(ch);
        } else {
            body.push(ch);
        }
    }
    None
}

fn typst_math_to_tex(source: &str) -> String {
    typst_math_to_tex_with_options(source, true)
}

fn typst_inline_math_to_tex(source: &str) -> String {
    typst_math_to_tex_with_options(source, false)
}

fn typst_math_to_tex_with_options(source: &str, scale_parentheses: bool) -> String {
    let mut tex = source.trim().to_owned();
    tex = tex.replace(r"\/", "@@SLASH@@");
    tex = protect_typst_set_braces(&tex);
    tex = tex.replace(r"\(", "(").replace(r"\)", ")");
    tex = convert_quoted_text(&tex);
    let replacements = [
        ("dots.c", "\\cdots"),
        ("dot.c", "\\cdot"),
        ("times.o.big", "\\bigotimes"),
        ("times.o", "\\otimes"),
        ("subset.eq", "@@SUBSETEQ@@"),
        ("subset", "\\subset"),
        ("in.not", "\\notin"),
        ("<<", "\\ll"),
        ("<=", "\\le"),
        (">=", "\\ge"),
        ("!=", "\\ne"),
        ("RR", "\\mathbb{R}"),
        ("NN", "\\mathbb{N}"),
        ("EE", "\\mathbb{E}"),
        ("qquad", "\\qquad"),
        ("|->", "\\mapsto"),
        ("<-", "\\leftarrow"),
        ("->", "\\to"),
        ("=>", "\\Rightarrow"),
        (":=", "\\coloneqq"),
        ("forall", "\\forall"),
        ("exists", "\\exists"),
        (" tilde ", " \\sim "),
        (" in ", " \\in "),
    ];
    for (from, to) in replacements {
        tex = tex.replace(from, to);
    }
    tex = tex.replace("@@SUBSETEQ@@", "\\subseteq");
    tex = replace_wordish(&tex, "times", "\\times");
    tex = replace_wordish(&tex, "dots", "\\dots");
    tex = replace_wordish(&tex, "quad", "\\quad");
    for (from, to) in [
        ("sum", "\\sum"),
        ("product", "\\prod"),
        ("max", "\\max"),
        ("min", "\\min"),
        ("argmax", "\\operatorname*{arg\\,max}"),
        ("argmin", "\\operatorname*{arg\\,min}"),
        ("arg", "\\operatorname{arg}"),
        ("conv", "\\operatorname{conv}"),
        ("log", "\\log"),
        ("mod", "\\bmod"),
        ("top", "\\top"),
        ("star", "\\star"),
        ("eps", "\\epsilon"),
        ("tau", "\\tau"),
        ("oo", "\\infty"),
    ] {
        tex = replace_wordish(&tex, from, to);
    }
    tex = tex.replace(" ~ ", " \\sim ");
    for (from, to) in [
        ("cA", "\\mathcal{A}"),
        ("cC", "\\mathcal{C}"),
        ("cH", "\\mathcal{H}"),
        ("cU", "\\mathcal{U}"),
        ("cX", "\\mathcal{X}"),
        ("cY", "\\mathcal{Y}"),
        ("vb", "\\mathbf{b}"),
        ("vx", "\\mathbf{x}"),
        ("vy", "\\mathbf{y}"),
        ("vu", "\\mathbf{u}"),
        ("vp", "\\mathbf{p}"),
        ("vc", "\\mathbf{c}"),
        ("vs", "\\mathbf{s}"),
        ("vz", "\\mathbf{z}"),
        ("matI", "\\mathbf{I}"),
        ("matM", "\\mathbf{M}"),
        ("matK", "\\mathbf{K}"),
        ("matA", "\\mathbf{A}"),
        ("matU", "\\mathbf{U}"),
    ] {
        tex = replace_wordish(&tex, from, to);
    }
    for (from, to) in [
        ("Gamma", "\\Gamma"),
        ("Delta", "\\Delta"),
        ("Omega", "\\Omega"),
        ("Phi", "\\Phi"),
        ("alpha", "\\alpha"),
        ("beta", "\\beta"),
        ("gamma", "\\gamma"),
        ("delta", "\\delta"),
        ("epsilon", "\\epsilon"),
        ("varepsilon", "\\varepsilon"),
        ("eta", "\\eta"),
        ("kappa", "\\kappa"),
        ("ell", "\\ell"),
        ("lambda", "\\lambda"),
        ("mu", "\\mu"),
        ("phi", "\\phi"),
        ("psi", "\\psi"),
        ("rho", "\\rho"),
        ("sigma", "\\sigma"),
        ("theta", "\\theta"),
    ] {
        tex = replace_wordish(&tex, from, to);
    }
    tex = convert_script_groups_fixpoint(&tex);
    tex = convert_cal_calls(&tex);
    tex = convert_typst_sqrt_calls(&tex);
    tex = convert_typst_fractions_fixpoint(&tex);
    tex = convert_typst_functions(&tex);
    tex = if scale_parentheses {
        scale_source_delimiters(&tex)
    } else {
        scale_source_delimiters_with_options(&tex, false)
    };
    tex = tex
        .replace("@@LBRACE@@", "\\left\\{")
        .replace("@@RBRACE@@", "\\right\\}");
    tex = tex.replace("@@SLASH@@", "/");
    tex = collapse_doubled_command_backslashes(&tex);
    tex
}

fn display_math_to_tex(source: &str) -> String {
    let tex = typst_math_to_tex(source);
    let tex = normalize_typst_linebreaks(&tex);
    if tex.contains('&') {
        format!("\\begin{{aligned}}{}\\end{{aligned}}", tex)
    } else {
        tex
    }
}

fn normalize_typst_linebreaks(input: &str) -> String {
    let mut out = String::new();
    for (idx, line) in input.lines().enumerate() {
        if idx > 0 {
            out.push('\n');
        }
        let trimmed = line.trim_end();
        if let Some(prefix) = trimmed.strip_suffix('\\') {
            out.push_str(prefix.trim_end());
            out.push_str(r" \\");
        } else {
            out.push_str(line);
        }
    }
    out
}

fn convert_typst_functions(input: &str) -> String {
    let mut out = input.to_owned();
    out = convert_named_call_fixpoint(&out, "underbrace", render_underbrace_call);
    out = convert_named_call_fixpoint(&out, "sqrt", render_sqrt_call);
    out = convert_named_call_fixpoint(&out, "binom", render_binom_call);
    out = convert_named_call_fixpoint(&out, "cal", render_cal_call);
    out = convert_named_call_fixpoint(&out, "ip", render_ip_call);
    out = convert_named_call_fixpoint(&out, "mat", render_mat_call);
    out = convert_named_call_fixpoint(&out, "cases", render_cases_call);
    out = convert_named_call_fixpoint(&out, "norm", render_norm_call);
    out = convert_named_call_fixpoint(&out, "abs", render_abs_call);
    out = convert_named_call_fixpoint(&out, "tilde", render_tilde_call);
    out = convert_named_call_fixpoint(&out, "overline", render_overline_call);
    out = convert_named_call_fixpoint(&out, "underline", render_underline_call);
    out
}

fn convert_typst_sqrt_calls(input: &str) -> String {
    convert_named_call_fixpoint(input, "sqrt", render_sqrt_call)
}

fn convert_named_call_fixpoint(
    input: &str,
    name: &str,
    render: fn(&str) -> Option<String>,
) -> String {
    let mut current = input.to_owned();
    for _ in 0..16 {
        let next = convert_named_call_once(&current, name, render);
        if next == current {
            return next;
        }
        current = next;
    }
    current
}

fn convert_named_call_once(input: &str, name: &str, render: fn(&str) -> Option<String>) -> String {
    let needle = format!("{name}(");
    let mut out = String::new();
    let mut rest = input;
    while let Some(pos) = rest.find(&needle) {
        out.push_str(&rest[..pos]);
        let before_ok = pos == 0
            || !rest[..pos]
                .chars()
                .next_back()
                .is_some_and(|ch| ch.is_ascii_alphabetic() || ch == '\\');
        if !before_ok {
            out.push_str(&needle);
            rest = &rest[pos + needle.len()..];
            continue;
        }
        let args_start = pos + needle.len();
        if let Some(end) = find_matching_paren(rest, args_start - 1) {
            let args = &rest[args_start..end];
            if let Some(rendered) = render(args) {
                out.push_str(&rendered);
            } else {
                out.push_str(&rest[pos..=end]);
            }
            rest = &rest[end + 1..];
        } else {
            out.push_str(&rest[pos..]);
            rest = "";
            break;
        }
    }
    out.push_str(rest);
    out
}

fn find_matching_paren(input: &str, open_idx: usize) -> Option<usize> {
    let mut depth = 0i32;
    for (idx, ch) in input.char_indices().skip_while(|(idx, _)| *idx < open_idx) {
        if ch == '(' {
            depth += 1;
        } else if ch == ')' {
            depth -= 1;
            if depth == 0 {
                return Some(idx);
            }
        }
    }
    None
}

fn render_ip_call(args: &str) -> Option<String> {
    let parts = split_top_level(args, ',');
    if parts.len() != 2 {
        return None;
    }
    Some(format!(
        "\\left\\langle {}, {}\\right\\rangle",
        parts[0].trim(),
        parts[1].trim()
    ))
}

fn render_mat_call(args: &str) -> Option<String> {
    let rows: Vec<String> = split_top_level(args, ';')
        .into_iter()
        .map(|row| {
            split_top_level(row.trim(), ',')
                .into_iter()
                .map(|cell| cell.trim().to_owned())
                .collect::<Vec<_>>()
                .join(" & ")
        })
        .collect();
    if rows.is_empty() {
        None
    } else {
        Some(format!(
            "\\begin{{pmatrix}}{}\\end{{pmatrix}}",
            rows.join(r" \\ ")
        ))
    }
}

fn render_cases_call(args: &str) -> Option<String> {
    let rows: Vec<String> = split_top_level(args, ',')
        .into_iter()
        .map(render_case_row)
        .collect();
    if rows.is_empty() {
        None
    } else {
        Some(format!(
            "\\begin{{cases}}{}\\end{{cases}}",
            rows.join(r" \\ ")
        ))
    }
}

fn render_case_row(row: &str) -> String {
    let cells: Vec<String> = split_top_level(row.trim(), '&')
        .into_iter()
        .enumerate()
        .map(|(idx, cell)| {
            let cell = cell.trim();
            if idx == 0 {
                cell.to_owned()
            } else {
                normalize_case_condition_spacing(cell)
            }
        })
        .collect();
    cells.join(" & ")
}

fn normalize_case_condition_spacing(cell: &str) -> String {
    if !cell.starts_with("\\mathrm{") {
        return cell.to_owned();
    }

    let open = "\\mathrm".len();
    let Some(close) = find_matching_group_right(cell, open, '{', '}') else {
        return cell.to_owned();
    };
    let after = &cell[close + 1..];
    if after.chars().next().is_some_and(char::is_whitespace) {
        format!("{}\\ {}", &cell[..=close], after.trim_start())
    } else {
        cell.to_owned()
    }
}

fn render_norm_call(args: &str) -> Option<String> {
    Some(format!("\\left\\lVert {}\\right\\rVert", args.trim()))
}

fn render_abs_call(args: &str) -> Option<String> {
    let arg = args.trim();
    if arg.is_empty() {
        None
    } else {
        Some(format!("\\left\\lvert {}\\right\\rvert", arg))
    }
}

fn render_sqrt_call(args: &str) -> Option<String> {
    let arg = args.trim();
    if arg.is_empty() {
        None
    } else {
        Some(format!("\\sqrt{{{arg}}}"))
    }
}

fn render_binom_call(args: &str) -> Option<String> {
    let parts = split_top_level(args, ',');
    if parts.len() != 2 {
        return None;
    }
    Some(format!(
        "\\binom{{{}}}{{{}}}",
        parts[0].trim(),
        parts[1].trim()
    ))
}

fn render_underbrace_call(args: &str) -> Option<String> {
    let parts = split_top_level(args, ',');
    if parts.len() != 2 {
        return None;
    }
    Some(format!(
        "\\underbrace{{{}}}_{{{}}}",
        parts[0].trim(),
        parts[1].trim()
    ))
}

fn render_tilde_call(args: &str) -> Option<String> {
    let arg = args.trim();
    if arg.is_empty() {
        None
    } else {
        Some(format!("\\tilde{{{arg}}}"))
    }
}

fn render_overline_call(args: &str) -> Option<String> {
    let arg = args.trim();
    if arg.is_empty() {
        None
    } else {
        Some(format!("\\overline{{{arg}}}"))
    }
}

fn render_underline_call(args: &str) -> Option<String> {
    let arg = args.trim();
    if arg.is_empty() {
        None
    } else {
        Some(format!("\\underline{{{arg}}}"))
    }
}

fn render_cal_call(args: &str) -> Option<String> {
    let arg = args.trim();
    if arg.is_empty() {
        return None;
    }
    if arg.starts_with("\\mathcal{") {
        return Some(arg.to_owned());
    }
    let mut chars = arg.char_indices();
    let (_, first) = chars.next()?;
    if !first.is_ascii_alphabetic() {
        return None;
    }
    let suffix_start = first.len_utf8();
    Some(format!("\\mathcal{{{first}}}{}", &arg[suffix_start..]))
}

fn split_top_level(input: &str, separator: char) -> Vec<&str> {
    let mut parts = Vec::new();
    let mut start = 0;
    let mut paren = 0i32;
    let mut bracket = 0i32;
    let mut brace = 0i32;
    let mut in_string = false;
    let mut escaped = false;
    for (idx, ch) in input.char_indices() {
        if escaped {
            escaped = false;
            continue;
        }
        if ch == '\\' {
            escaped = true;
            continue;
        }
        if ch == '"' {
            in_string = !in_string;
            continue;
        }
        if in_string {
            continue;
        }
        match ch {
            '(' => paren += 1,
            ')' => paren -= 1,
            '[' => bracket += 1,
            ']' => bracket -= 1,
            '{' => brace += 1,
            '}' => brace -= 1,
            _ if ch == separator && paren == 0 && bracket == 0 && brace == 0 => {
                parts.push(&input[start..idx]);
                start = idx + ch.len_utf8();
            }
            _ => {}
        }
    }
    parts.push(&input[start..]);
    parts
}

fn convert_quoted_text(input: &str) -> String {
    let mut out = String::new();
    let mut chars = input.chars().peekable();
    while let Some(ch) = chars.next() {
        if ch == '"' {
            let mut text = String::new();
            let mut closed = false;
            while let Some(inner) = chars.next() {
                if inner == '"' {
                    closed = true;
                    break;
                }
                text.push(inner);
            }
            if closed {
                out.push_str("\\mathrm{");
                out.push_str(&format_quoted_math_text(&text));
                out.push('}');
            } else {
                out.push('"');
                out.push_str(&text);
            }
        } else {
            out.push(ch);
        }
    }
    out
}

fn format_quoted_math_text(text: &str) -> String {
    let mut out = String::new();
    for ch in text.chars() {
        match ch {
            ' ' => out.push_str("\\ "),
            '-' => out.push_str("\\text{-}"),
            _ => out.push(ch),
        }
    }
    out
}

fn protect_typst_set_braces(input: &str) -> String {
    let mut out = String::new();
    let mut chars = input.char_indices().peekable();
    let mut previous_significant = None;
    while let Some((_, ch)) = chars.next() {
        if ch == '{' && matches!(previous_significant, Some('_' | '^')) {
            out.push(ch);
            previous_significant = Some(ch);
            let mut depth = 1i32;
            for (_, inner) in chars.by_ref() {
                out.push(inner);
                if inner == '{' {
                    depth += 1;
                } else if inner == '}' {
                    depth -= 1;
                    if depth == 0 {
                        previous_significant = Some(inner);
                        break;
                    }
                } else if !inner.is_whitespace() {
                    previous_significant = Some(inner);
                }
            }
        } else if ch == '{' {
            out.push_str("@@LBRACE@@");
            previous_significant = Some(ch);
        } else if ch == '}' {
            out.push_str("@@RBRACE@@");
            previous_significant = Some(ch);
        } else {
            out.push(ch);
            if !ch.is_whitespace() {
                previous_significant = Some(ch);
            }
        }
    }
    out
}

fn convert_script_groups_fixpoint(input: &str) -> String {
    let mut current = input.to_owned();
    for _ in 0..8 {
        let next = convert_script_groups(&current);
        if next == current {
            return next;
        }
        current = next;
    }
    current
}

fn replace_wordish(input: &str, from: &str, to: &str) -> String {
    let mut out = String::new();
    let mut rest = input;
    while let Some(pos) = rest.find(from) {
        out.push_str(&rest[..pos]);
        let before_ok = pos == 0
            || !rest[..pos]
                .chars()
                .next_back()
                .is_some_and(|ch| ch.is_ascii_alphabetic());
        let after = pos + from.len();
        let after_ok = after == rest.len()
            || !rest[after..]
                .chars()
                .next()
                .is_some_and(|ch| ch.is_ascii_alphabetic());
        if before_ok && after_ok {
            out.push_str(to);
        } else {
            out.push_str(from);
        }
        rest = &rest[after..];
    }
    out.push_str(rest);
    out
}

fn convert_script_groups(input: &str) -> String {
    let mut out = String::new();
    let chars: Vec<char> = input.chars().collect();
    let mut i = 0;
    while i < chars.len() {
        if (chars[i] == '_' || chars[i] == '^') && chars.get(i + 1) == Some(&'(') {
            out.push(chars[i]);
            out.push('{');
            i += 2;
            let mut depth = 1;
            while i < chars.len() {
                if chars[i] == '(' {
                    depth += 1;
                    out.push(chars[i]);
                } else if chars[i] == ')' {
                    depth -= 1;
                    if depth == 0 {
                        break;
                    }
                    out.push(chars[i]);
                } else {
                    out.push(chars[i]);
                }
                i += 1;
            }
            out.push('}');
            i += 1;
        } else {
            out.push(chars[i]);
            i += 1;
        }
    }
    out
}

fn convert_cal_calls(input: &str) -> String {
    let mut out = input.to_owned();
    for cap in 'A'..='Z' {
        out = out.replace(&format!("cal({cap})"), &format!("\\mathcal{{{cap}}}"));
    }
    out
}

fn collapse_doubled_command_backslashes(input: &str) -> String {
    let mut out = String::new();
    let chars: Vec<char> = input.chars().collect();
    let mut i = 0;
    while i < chars.len() {
        if chars[i] == '\\'
            && chars.get(i + 1) == Some(&'\\')
            && chars.get(i + 2).is_some_and(|ch| ch.is_ascii_alphabetic())
        {
            out.push('\\');
            i += 2;
        } else {
            out.push(chars[i]);
            i += 1;
        }
    }
    out
}

fn scale_source_delimiters(input: &str) -> String {
    scale_source_delimiters_with_options(input, true)
}

fn scale_source_delimiters_with_options(input: &str, scale_parentheses: bool) -> String {
    let mut out = String::new();
    let chars: Vec<char> = input.chars().collect();
    let mut i = 0;
    while i < chars.len() {
        let ch = chars[i];
        if ch == '\\' {
            out.push(ch);
            i += 1;
            let command_start = i;
            while i < chars.len() && chars[i].is_ascii_alphabetic() {
                out.push(chars[i]);
                i += 1;
            }
            if i < chars.len() && chars[i] == '*' {
                out.push(chars[i]);
                i += 1;
            }
            let command: String = chars[command_start..i].iter().collect();
            if matches!(command.as_str(), "left" | "right")
                && i < chars.len()
                && is_sized_delimiter_after_command(&chars, i)
            {
                if chars[i] == '\\' && chars.get(i + 1) == Some(&'{') {
                    out.push('\\');
                    out.push('{');
                    i += 2;
                } else if chars[i] == '\\' && chars.get(i + 1) == Some(&'}') {
                    out.push('\\');
                    out.push('}');
                    i += 2;
                } else {
                    out.push(chars[i]);
                    i += 1;
                }
            }
            continue;
        }
        match ch {
            '(' if scale_parentheses => out.push_str("\\left("),
            ')' if scale_parentheses => out.push_str("\\right)"),
            '[' => out.push_str("\\left["),
            ']' => out.push_str("\\right]"),
            _ => out.push(ch),
        }
        i += 1;
    }
    out
}

fn is_sized_delimiter_after_command(chars: &[char], index: usize) -> bool {
    matches!(
        chars.get(index),
        Some('(' | ')' | '[' | ']' | '.' | '|' | '<' | '>')
    ) || (chars.get(index) == Some(&'\\') && matches!(chars.get(index + 1), Some('{' | '}')))
}

fn convert_typst_fractions_fixpoint(input: &str) -> String {
    let mut current = input.to_owned();
    for _ in 0..16 {
        let next = convert_typst_fraction_once(&current);
        if next == current {
            return next;
        }
        current = next;
    }
    current
}

fn convert_typst_fraction_once(input: &str) -> String {
    for (idx, ch) in input.char_indices() {
        if ch != '/' {
            continue;
        }
        if input[..idx].ends_with('\\') {
            continue;
        }
        let Some(left) = fraction_left_operand(input, idx) else {
            continue;
        };
        let Some(right) = fraction_right_operand(input, idx + 1) else {
            continue;
        };
        let mut out = String::new();
        out.push_str(&input[..left.range_start]);
        write!(
            out,
            "\\frac{{{}}}{{{}}}",
            &input[left.content_start..left.content_end],
            &input[right.content_start..right.content_end]
        )
        .unwrap();
        out.push_str(&input[right.range_end..]);
        return out;
    }
    input.to_owned()
}

#[derive(Clone, Copy)]
struct FractionOperand {
    range_start: usize,
    range_end: usize,
    content_start: usize,
    content_end: usize,
}

fn fraction_left_operand(input: &str, slash_idx: usize) -> Option<FractionOperand> {
    let range_end = trim_end_boundary(input, slash_idx);
    if range_end == 0 {
        return None;
    }
    let (range_start, range_end, content_start, content_end) =
        if let Some((open, close, content_start, content_end)) =
            grouped_operand_left(input, range_end)
        {
            (open, close, content_start, content_end)
        } else {
            let start = scan_atom_left(input, range_end)?;
            (start, range_end, start, range_end)
        };
    Some(FractionOperand {
        range_start,
        range_end,
        content_start,
        content_end,
    })
}

fn fraction_right_operand(input: &str, after_slash: usize) -> Option<FractionOperand> {
    let range_start = trim_start_boundary(input, after_slash);
    if range_start >= input.len() {
        return None;
    }
    let (range_end, content_start, content_end) =
        if let Some((_open, close, content_start, content_end)) =
            grouped_operand_right(input, range_start)
        {
            (close, content_start, content_end)
        } else {
            let end = scan_atom_right(input, range_start)?;
            (end, range_start, end)
        };
    Some(FractionOperand {
        range_start,
        range_end,
        content_start,
        content_end,
    })
}

fn trim_end_boundary(input: &str, end: usize) -> usize {
    let mut boundary = end;
    while boundary > 0 {
        let ch = input[..boundary].chars().next_back().unwrap();
        if !ch.is_whitespace() {
            break;
        }
        boundary -= ch.len_utf8();
    }
    boundary
}

fn trim_start_boundary(input: &str, start: usize) -> usize {
    let mut boundary = start;
    while boundary < input.len() {
        let ch = input[boundary..].chars().next().unwrap();
        if !ch.is_whitespace() {
            break;
        }
        boundary += ch.len_utf8();
    }
    boundary
}

fn grouped_operand_left(input: &str, end: usize) -> Option<(usize, usize, usize, usize)> {
    let close = input[..end].chars().next_back()?;
    let (open_ch, close_ch) = match close {
        ')' => ('(', ')'),
        ']' => ('[', ']'),
        '}' => ('{', '}'),
        _ => return None,
    };
    let close_start = end - close.len_utf8();
    let open = find_matching_group_left(input, close_start, open_ch, close_ch)?;
    if open_ch == '{' {
        if let Some(command_start) = command_start_before_group(input, open) {
            return Some((command_start, end, command_start, end));
        }
    }
    let range_start = if open_ch == '{' { open } else { open };
    Some((range_start, end, open + open_ch.len_utf8(), close_start))
}

fn grouped_operand_right(input: &str, start: usize) -> Option<(usize, usize, usize, usize)> {
    let open = input[start..].chars().next()?;
    let close_ch = match open {
        '(' => ')',
        '[' => ']',
        '{' => '}',
        _ => return None,
    };
    let close = find_matching_group_right(input, start, open, close_ch)?;
    Some((
        start,
        close + close_ch.len_utf8(),
        start + open.len_utf8(),
        close,
    ))
}

fn find_matching_group_left(
    input: &str,
    close_idx: usize,
    open_ch: char,
    close_ch: char,
) -> Option<usize> {
    let mut depth = 0i32;
    for (idx, ch) in input[..=close_idx].char_indices().rev() {
        if ch == close_ch {
            depth += 1;
        } else if ch == open_ch {
            depth -= 1;
            if depth == 0 {
                return Some(idx);
            }
        }
    }
    None
}

fn find_matching_group_right(
    input: &str,
    open_idx: usize,
    open_ch: char,
    close_ch: char,
) -> Option<usize> {
    let mut depth = 0i32;
    for (idx, ch) in input.char_indices().skip_while(|(idx, _)| *idx < open_idx) {
        if ch == open_ch {
            depth += 1;
        } else if ch == close_ch {
            depth -= 1;
            if depth == 0 {
                return Some(idx);
            }
        }
    }
    None
}

fn command_start_before_group(input: &str, group_start: usize) -> Option<usize> {
    let mut start = group_start;
    while start > 0 {
        let ch = input[..start].chars().next_back().unwrap();
        if ch.is_ascii_alphabetic() {
            start -= ch.len_utf8();
        } else {
            break;
        }
    }
    if start > 0 && input[..start].ends_with('\\') {
        Some(start - 1)
    } else {
        None
    }
}

fn scan_atom_left(input: &str, end: usize) -> Option<usize> {
    let mut start = end;
    while start > 0 {
        let ch = input[..start].chars().next_back().unwrap();
        if is_fraction_atom_char(ch) {
            start -= ch.len_utf8();
        } else {
            break;
        }
    }
    (start < end).then_some(start)
}

fn scan_atom_right(input: &str, start: usize) -> Option<usize> {
    let mut end = start;
    while end < input.len() {
        let ch = input[end..].chars().next().unwrap();
        if is_fraction_atom_char(ch) {
            end += ch.len_utf8();
        } else {
            break;
        }
    }
    (start < end).then_some(end)
}

fn is_fraction_atom_char(ch: char) -> bool {
    ch.is_ascii_alphanumeric()
        || matches!(ch, '\\' | '_' | '^' | '{' | '}' | '|' | '.' | '\'' | '@')
}

fn title_from_path(path: &Path) -> String {
    path.file_stem()
        .and_then(|name| name.to_str())
        .unwrap_or("Notes")
        .replace(['-', '_'], " ")
}

fn slugify(text: &str) -> String {
    let mut out = String::new();
    let mut dash = false;
    for ch in text.chars().flat_map(|ch| ch.to_lowercase()) {
        if ch.is_ascii_alphanumeric() {
            out.push(ch);
            dash = false;
        } else if !dash {
            out.push('-');
            dash = true;
        }
    }
    let out = out.trim_matches('-').to_owned();
    if out.is_empty() {
        "section".to_owned()
    } else {
        out
    }
}

fn escape_html(input: &str) -> String {
    input
        .replace('&', "&amp;")
        .replace('<', "&lt;")
        .replace('>', "&gt;")
        .replace('"', "&quot;")
}

fn smart_quote_entity(ch: char, before: Option<char>, after: Option<char>) -> Option<&'static str> {
    match ch {
        '"' => {
            if quote_opens_after(before) {
                Some("&ldquo;")
            } else {
                Some("&rdquo;")
            }
        }
        '\'' => {
            if before.is_some_and(is_word_char)
                || after.is_some_and(|next| next.is_ascii_digit())
                || !quote_opens_after(before)
            {
                Some("&rsquo;")
            } else {
                Some("&lsquo;")
            }
        }
        _ => None,
    }
}

fn quote_opens_after(before: Option<char>) -> bool {
    before.is_none_or(|ch| ch.is_whitespace() || matches!(ch, '(' | '[' | '{' | '<' | '-' | '/'))
}

fn is_word_char(ch: char) -> bool {
    ch.is_alphanumeric()
}

fn escape_attr(input: &str) -> String {
    escape_html(input).replace('\'', "&#39;")
}

fn plain_text(input: &str) -> String {
    input
        .chars()
        .filter(|ch| !matches!(ch, '[' | ']' | '$' | '#'))
        .collect::<String>()
        .split_whitespace()
        .collect::<Vec<_>>()
        .join(" ")
}

fn page_css() -> &'static str {
    r#"
@font-face{font-family:"Frutiger Notes";src:url("Frutiger_bold.ttf") format("truetype");font-weight:700;font-style:normal;font-display:swap}
@font-face{font-family:"New Computer Modern";src:url("NewCM10-Regular.otf") format("opentype");font-weight:400;font-style:normal;font-display:swap}
@font-face{font-family:"New Computer Modern";src:url("NewCM10-Italic.otf") format("opentype");font-weight:400;font-style:italic;font-display:swap}
@font-face{font-family:"New Computer Modern";src:url("NewCM10-Bold.otf") format("opentype");font-weight:700;font-style:normal;font-display:swap}
@font-face{font-family:"New Computer Modern";src:url("NewCM10-BoldItalic.otf") format("opentype");font-weight:700;font-style:italic;font-display:swap}
@font-face{font-family:"NewComputerModern 10";src:url("NewCM10-Regular.otf") format("opentype");font-weight:400;font-style:normal;font-display:swap}
@font-face{font-family:"NewComputerModern 10";src:url("NewCM10-Italic.otf") format("opentype");font-weight:400;font-style:italic;font-display:swap}
@font-face{font-family:"NewComputerModern 10";src:url("NewCM10-Bold.otf") format("opentype");font-weight:700;font-style:normal;font-display:swap}
@font-face{font-family:"NewComputerModern 10";src:url("NewCM10-BoldItalic.otf") format("opentype");font-weight:700;font-style:italic;font-display:swap}
@font-face{font-family:"NewComputerModern10";src:url("NewCM10-Regular.otf") format("opentype");font-weight:400;font-style:normal;font-display:swap}
@font-face{font-family:"NewComputerModern10";src:url("NewCM10-Italic.otf") format("opentype");font-weight:400;font-style:italic;font-display:swap}
@font-face{font-family:"NewComputerModern10";src:url("NewCM10-Bold.otf") format("opentype");font-weight:700;font-style:normal;font-display:swap}
@font-face{font-family:"NewComputerModern10";src:url("NewCM10-BoldItalic.otf") format("opentype");font-weight:700;font-style:italic;font-display:swap}
:root{--paper:#fbfaf7;--ink:#171411;--muted:#6b635b;--rule:#d7d0c6;--soft:#f2efe8;--block:#eeeeee;--accent:#325d8a;--accent-dark:#173f64;--brown:#8a4c20;--serif:"New Computer Modern","NewComputerModern 10","NewComputerModern10","New Computer Modern 10","Computer Modern Serif","Latin Modern Roman",Georgia,"Times New Roman",Times,serif;--page-total:828px;--rail-width:180px;--rail-gutter:2cm;--rail-left:max(24px,calc((100vw - var(--page-total))/2 - var(--rail-width) - var(--rail-gutter)))}
html{background:var(--paper);scroll-behavior:smooth}
body{margin:0;color:var(--ink);background:var(--paper);font-family:var(--serif);font-size:18px;line-height:1.58}
h1,h2,h3,h4,strong,b,.env-title,.algorithm-label,.chapter-kicker,.course-title,.course-event,.chapter-rail-heading,.chapter-rail-tool,.chapter-rail-link.is-current,.citation-note-key{font-family:"Frutiger Notes",var(--serif)}
a{color:var(--accent);text-decoration:none}a:hover{color:var(--accent-dark);text-decoration:underline}.ref-link{display:inline-block;padding:0 .22em;border-radius:2px;background:#e8f1fb;color:#24527e;line-height:1.28}.ref-link:hover{background:#dcebf8;color:#173f64;text-decoration:none}.citation{color:#8a4c20}.citation:hover{color:#5f3213}.citation-wrap{position:relative}.citation-note{float:right;clear:right;width:210px;margin:.2rem -234px .7rem 24px;color:#8a4c20;font-size:.78rem;font-weight:400;line-height:1.35;text-align:left}.citation-note a{color:#8a4c20}.citation-note-key{font-weight:700}
.site-masthead{max-width:780px;margin:0 auto;padding:28px 24px 16px;border-bottom:1px solid var(--rule)}
.course-title{font-size:1.06rem;font-weight:700}.course-title-link{display:block!important;padding:0!important;color:var(--ink)!important}.course-title-link:hover{color:var(--accent)!important;text-decoration:none!important;background:transparent!important}.course-authors{margin-top:2px;color:var(--muted);font-size:.92rem}.top-links{display:flex;gap:14px;margin-top:10px;font-size:.9rem}
.course-sidenote{float:right;clear:right;width:210px;margin:.1rem -234px 1rem 24px;color:var(--muted);font-size:.78rem;line-height:1.35;text-align:left}.course-sidenote .course-title{color:var(--ink);font-size:.85rem;line-height:1.25}.course-sidenote .course-authors{font-size:.78rem}.chapter-citation-sidenote{float:right;clear:right;width:210px;margin:.1rem -234px 1.2rem 24px;color:var(--muted);font-size:.72rem;line-height:1.28;text-align:left}.chapter-citation-pdf{display:block;margin:0 0 .55rem!important;padding:0!important;color:var(--accent)!important;font-family:"Frutiger Notes",var(--serif);font-size:.72rem;font-weight:700}.chapter-citation-pdf:hover{color:var(--accent-dark)!important;background:transparent!important}.chapter-citation-details{margin:0}.chapter-citation-title{display:inline-flex;align-items:center;gap:.28rem;cursor:pointer;color:var(--brown);font-family:"Frutiger Notes",var(--serif);font-size:.68rem;font-weight:700;letter-spacing:.04em;text-transform:uppercase}.chapter-citation-title::before{content:"";width:0;height:0;border-top:.28em solid transparent;border-bottom:.28em solid transparent;border-left:.42em solid currentColor;transition:transform .16s ease}.chapter-citation-details[open] .chapter-citation-title::before{transform:rotate(90deg)}.chapter-citation-title::-webkit-details-marker{display:none}.chapter-citation-title::marker{content:""}.chapter-citation-title:hover{color:#5f3213}.chapter-citation-sidenote pre{margin:.35rem 0 0;padding:.45rem .5rem;overflow-x:auto;background:#fffdf9;border-left:2px solid var(--rule);white-space:pre-wrap}.chapter-citation-sidenote code{font-size:.72rem;line-height:1.25}
.chapter-rail{position:fixed;top:32px;left:var(--rail-left);bottom:24px;width:180px;padding:.2rem 0 1rem;border-left:1px solid var(--rule);font-size:.84rem;line-height:1.25;overflow-y:auto;scrollbar-width:thin}.chapter-rail-course{margin:0 0 .7rem;padding:0 .3rem .75rem .75rem;border-bottom:1px solid var(--rule)}.course-event{margin:0 0 .28rem;color:var(--brown);font-size:.68rem;font-weight:700;letter-spacing:.04em;text-transform:uppercase}.chapter-rail-course .course-title{color:var(--ink);font-size:.84rem;line-height:1.25}.chapter-rail-course .course-authors{margin-top:.28rem;color:var(--muted);font-size:.76rem;line-height:1.28}.chapter-rail-tools{display:flex;gap:.55rem;margin:.55rem 0 0;padding:0}.chapter-rail-tool{display:inline!important;padding:0!important;color:var(--accent)!important;font-weight:700}.chapter-rail-heading{margin:.1rem 0 .35rem;padding:0 .3rem 0 .75rem;color:var(--ink);font-size:.72rem;font-weight:700;letter-spacing:.04em;text-transform:uppercase}.chapter-rail-section-heading{margin-top:.85rem;padding-top:.7rem;border-top:1px solid var(--rule);color:var(--muted)}.chapter-rail a{display:block;padding:.32rem .3rem .32rem .75rem;color:var(--muted)}.chapter-rail a:hover{color:var(--accent-dark);text-decoration:none;background:#fffdf9}.chapter-rail-link span{display:inline-block;width:1.45rem;color:var(--muted);font-variant-numeric:tabular-nums}.chapter-rail-link.is-current{color:var(--ink);font-weight:700;background:#fffdf9;border-left:3px solid var(--accent);margin-left:-2px}.chapter-section-link{display:grid!important;grid-template-columns:2.55rem minmax(0,1fr);align-items:baseline;font-size:.78rem}.chapter-section-no{color:var(--muted);font-variant-numeric:tabular-nums}.chapter-section-title{min-width:0}.chapter-section-l2{padding-left:1.25rem!important}.chapter-section-l3{padding-left:1.75rem!important;font-size:.74rem}.chapter-section-link.is-active{color:var(--ink);background:#fffdf9;border-left:3px solid var(--brown);margin-left:-2px}
.page-shell{max-width:780px;margin:0 auto;padding:30px 24px 64px;position:relative}.chapter-kicker{margin:0 0 .25rem;color:var(--brown);font-size:.86rem;font-weight:700;letter-spacing:.04em;text-transform:uppercase}.lecture-title{display:inline-block;max-width:100%;margin:0 0 calc(1.85rem + 2cm);padding-bottom:.32rem;border-bottom:4px solid #000;font-size:2.65rem;line-height:1.06;letter-spacing:0}
.toc{margin:0 0 2rem;padding:.85rem 1rem;background:#fffdf9;border:1px solid var(--rule);font-size:.95rem}.toc ol{margin:0;padding-left:0;list-style:none}.toc li{margin:.12rem 0}.toc a{display:grid;grid-template-columns:3rem minmax(0,1fr);align-items:baseline}.toc .toc-l1{margin-left:0}.toc .toc-l2{margin-left:1.1rem}.toc .toc-l3{margin-left:2.2rem;font-size:.92em}.toc-no{color:var(--muted);font-variant-numeric:tabular-nums;text-align:left}.toc-title{min-width:0}
.lecture-content h1,.lecture-content h2,.lecture-content h3,.lecture-content h4{line-height:1.18;letter-spacing:0}.lecture-content h1,.lecture-content h2,.lecture-content h3{position:relative}.lecture-content h1::before,.lecture-content h2::before,.lecture-content h3::before{content:"";position:absolute;left:-1.05rem;top:.18em;bottom:.16em;width:4px;background:#bdb7ad}.lecture-content h2::before{width:3px;background:#c8c2b9}.lecture-content h3::before{width:2px;background:#d1cbc3}.lecture-content h1{margin:2.2rem 0 .8rem;padding-top:.15rem;font-size:1.75rem}.lecture-content h2{margin:2rem 0 .65rem;font-size:1.38rem}.lecture-content h3{margin:1.55rem 0 .45rem;font-size:1.12rem}.secno{color:var(--muted);font-weight:400}
.lecture-content p{margin:.72rem 0}.lecture-content ul,.lecture-content ol{padding-left:1.35rem}.lecture-content li{margin:.25rem 0}
code{font-family:ui-monospace,SFMono-Regular,Menlo,Consolas,"Liberation Mono",monospace;font-size:.92em}.raw-block{margin:1rem 0;padding:.75rem 1rem;overflow-x:auto;background:#fffdf9;border:1px solid var(--rule);font-size:.9rem;line-height:1.45}.raw-block code{font-size:1em}
.env{margin:1.1rem 0;padding:.42rem .85rem;border:1px solid #d8d8d8;border-radius:3px}.env.statement{background:var(--block)}.env.algorithm{padding:.15rem 0 .45rem;background:transparent;border:0;border-radius:0}.env.proof{background:transparent;border:0;border-left:1.5px solid #a8a098;border-radius:0;padding:.16rem 0 .16rem .85rem}.env-title{font-weight:700}.env.algorithm .env-title{margin:0 0 .45rem;padding:.38rem 0 .42rem;border-top:1.5px solid #1f1f1f;border-bottom:1.5px solid #1f1f1f;font-size:.98em;font-weight:400;line-height:1.28}.algorithm-label{font-weight:700}.algorithm-caption{font-weight:400}.env.proof .env-title{font-family:var(--serif);font-style:italic;font-weight:400}.algorithm-subtitle{margin:.15rem 0 .55rem;color:var(--muted);font-style:italic}.pseudocode{--pseudo-indent:1.42rem;--pseudo-stroke:#252525;margin:.1rem 0 0;padding:.12rem 0 .08rem;border-bottom:1.5px solid #1f1f1f;font-size:.94em;line-height:1.36}.pseudo-line,.pseudo-math{position:relative;margin:0;padding-left:calc(var(--indent) * var(--pseudo-indent) + .18rem)}.pseudo-line{min-height:1.32em;padding-top:.11rem;padding-bottom:.11rem}.pseudo-text{display:block;max-width:100%;text-indent:-.04rem}.pseudo-text strong{font-weight:700}.pseudo-math{overflow-x:auto;padding-top:.08rem;padding-bottom:.12rem}.pseudo-math .katex-display{margin:.16rem 0}.pseudo-guide{position:absolute;top:-.12rem;bottom:-.12rem;left:calc((var(--guide) - .52) * var(--pseudo-indent));border-left:1.25px solid var(--pseudo-stroke)}.pseudo-guide-end{bottom:.58em}.pseudo-guide-end::after{content:"";position:absolute;left:-1.25px;bottom:0;width:.48rem;border-bottom:1.25px solid var(--pseudo-stroke)}
.equation{box-sizing:border-box;position:relative;margin:1rem 0;overflow-x:auto;padding-right:3.5rem}.equation.is-overwide{clear:right}.equation .katex-display{margin:.35rem 0}.eqno{position:absolute;right:.2rem;top:50%;transform:translateY(-50%);color:var(--muted)}
.rendered-figure{position:relative;margin:1.25rem 0;text-align:center}.rendered-figure img{display:block;max-width:100%;height:auto;margin:0 auto}.figure-placeholder{margin:1.15rem 0;padding:.8rem 1rem;background:var(--soft);border:1px dashed var(--rule);color:var(--muted)}figcaption{margin-top:.5rem;color:var(--ink);text-align:left}.special-comment{margin:1rem 0;padding:.55rem .8rem .65rem 1rem;border-left:3px solid var(--rule);background:#fffdf9;color:var(--muted);font-size:.94rem}.special-comment::before{content:"\25B8";margin-right:.35rem;color:var(--muted)}.special-comment-text{display:inline}.special-comment-figure{display:block;max-width:100%;height:auto;margin:.55rem auto 0}
.typst-fallback,.callout{margin:1rem 0;padding:.75rem 1rem;background:var(--soft);border-left:3px solid var(--rule)}.callout{font-size:.98rem}.footnote-ref{font-size:.68em;line-height:0;vertical-align:super}.footnote-ref a{color:var(--accent);text-decoration:none}.footnote{float:right;clear:right;width:210px;margin:.2rem -234px .7rem 24px;color:var(--muted);font-size:.78rem;line-height:1.35;text-align:left}.footnote-num{font-size:.78em;line-height:0;vertical-align:super;margin-right:.3em;color:var(--accent)}.bibliography,.endnotes{margin-top:2.5rem;padding-top:.75rem;border-top:1px solid var(--rule)}.bibliography{font-size:16px}.endnotes{font-size:.92rem}.bibliography h1,.endnotes h1{border-top:0;margin-top:0}.bibliography p{display:grid;grid-template-columns:5.4rem minmax(0,1fr);column-gap:.7rem;align-items:baseline;margin:.58rem 0;padding-left:0;text-indent:0}.endnotes{display:none}.endnotes p{display:grid;grid-template-columns:1.75rem minmax(0,1fr);column-gap:.4rem;align-items:baseline;margin:.58rem 0;padding-left:0;text-indent:0}.footnote-backref{display:inline-block;font-variant-numeric:tabular-nums;text-align:right}.footnote-backref::after{content:".";color:var(--muted)}.footnote-body{min-width:0}.bib-key{color:var(--ink);font-size:1em;font-variant-numeric:tabular-nums;text-align:left}.bib-entry{min-width:0}mark{background:#fff1a8;padding:0 .15em}.todo{background:#ffd39a;color:#4a2500;border-radius:2px}.missing-ref{color:#9b2f2f}
@media(max-width:1220px){.citation-note,.footnote,.course-sidenote{display:none}.chapter-citation-sidenote{float:none;clear:both;width:auto;margin:.75rem 0 2rem;color:var(--muted);font-size:.78rem}.chapter-citation-sidenote pre{max-width:100%}.chapter-citation-sidenote code{font-size:.76rem}.endnotes{display:block}.footnote-ref{margin-right:.1em}}
@media(min-width:1260px){.equation.is-overwide{width:calc(100% + 234px);overflow-x:visible}.rendered-figure figcaption{position:absolute;top:0;left:calc(100% + 24px);width:210px;margin-top:0;color:var(--muted);font-size:.78rem;line-height:1.35;text-align:left}}
@media(max-width:1259px){.equation.is-overwide{display:flex;align-items:center;gap:.5rem;overflow-x:auto;padding-right:0}.equation.is-overwide .katex-display{flex:0 0 auto}.equation.is-overwide .eqno{position:static;flex:0 0 auto;padding-left:.35rem;transform:none}}
@media(min-width:1261px){.toc{display:none}}
@media(max-width:1260px){.chapter-rail{display:none}}
@media(max-width:900px){.lecture-content h1::before,.lecture-content h2::before,.lecture-content h3::before{display:none}}
@media(max-width:640px){body{font-size:16.5px}.site-masthead,.page-shell{padding-left:17px;padding-right:17px}.lecture-title{font-size:2.12rem;line-height:1.08}}
"#
}

fn equation_width_script() -> &'static str {
    r#"
function markOverwideEquations(){
  document.querySelectorAll(".equation").forEach(function(eq){
    eq.classList.remove("is-overwide");
    var math = eq.querySelector(".katex-display");
    if (!math) return;
    var eqno = eq.querySelector(".eqno");
    var numberWidth = eqno ? eqno.getBoundingClientRect().width + 12 : 0;
    var available = eq.clientWidth - numberWidth;
    var needed = Math.max(math.scrollWidth, math.getBoundingClientRect().width);
    if (needed > available + 2) eq.classList.add("is-overwide");
  });
}
window.markOverwideEquations = markOverwideEquations;
window.addEventListener("resize", markOverwideEquations);
if (document.fonts && document.fonts.ready) {
  document.fonts.ready.then(markOverwideEquations);
}
"#
}

fn katex_config() -> &'static str {
    r#"
{
  delimiters:[
    {left:'\\[',right:'\\]',display:true},
    {left:'\\(',right:'\\)',display:false}
  ],
  throwOnError:false,
  strict:'warn'
}
"#
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn converts_typst_script_groups() {
        let tex = typst_math_to_tex("vp^((t)) tilde D^((t))");
        assert!(tex.contains("\\mathbf{p}^{\\left(t\\right)}"), "{tex}");
        assert!(tex.contains("D^{\\left(t\\right)}"), "{tex}");
    }

    #[test]
    fn converts_script_groups_inside_expectations() {
        let tex =
            typst_math_to_tex("EE_(vp^((t)) tilde D^((t))) [ip(h(vc^((t)), vp^((t))), vu^((t)))]");
        assert!(!tex.contains("^("), "{tex}");
        assert!(tex.contains("\\mathbf{p}^{\\left(t\\right)}"), "{tex}");
        assert!(tex.contains("D^{\\left(t\\right)}"), "{tex}");
        assert!(tex.contains("\\left["), "{tex}");
        assert!(tex.contains("\\right]"), "{tex}");
    }

    #[test]
    fn scales_source_delimiters_without_doubling_existing_left_right() {
        let tex = typst_math_to_tex("EE_(t in [T]) [f(vx) + { vx : vx in cX }]");
        assert!(tex.contains("\\left[T\\right]"), "{tex}");
        assert!(tex.contains("\\left[f\\left(\\mathbf{x}\\right)"), "{tex}");
        assert!(
            tex.contains("\\left\\{ \\mathbf{x} : \\mathbf{x} \\in \\mathcal{X} \\right\\}"),
            "{tex}"
        );

        let already_scaled = scale_source_delimiters("\\left( a \\right) + \\left[ b \\right]");
        assert_eq!(already_scaled, "\\left( a \\right) + \\left[ b \\right]");
    }

    #[test]
    fn converts_common_arrows_and_spacing() {
        let tex = typst_math_to_tex("vp |-> sigma(vp), qquad h : cX -> cX, vx <- vy");
        assert!(tex.contains("\\mapsto"), "{tex}");
        assert!(tex.contains("\\qquad"), "{tex}");
        assert!(tex.contains("\\to"), "{tex}");
        assert!(tex.contains("\\leftarrow"), "{tex}");
    }

    #[test]
    fn converts_much_less_relation() {
        let tex = typst_math_to_tex("epsilon << 1");
        assert_eq!(tex, "\\epsilon \\ll 1");
    }

    #[test]
    fn skips_multiline_setup_directives() {
        let source = r#"#show: gabri_notes.with(
  lec_num: 6,
  date: none,
  title: [Profile Swap Regret],
)

First paragraph.
"#;
        let mut parser = Parser::new(source);
        let blocks = parser.parse_blocks();
        assert_eq!(blocks.len(), 1, "{blocks:?}");
        assert!(matches!(&blocks[0], Block::Paragraph(text) if text == "First paragraph."));
    }

    #[test]
    fn converts_bare_subset_relation() {
        let tex = typst_math_to_tex("cX subset RR^d and cal(U) subset.eq RR^d");
        assert!(tex.contains("\\mathcal{X} \\subset \\mathbb{R}^d"), "{tex}");
        assert!(tex.contains("\\mathcal{U} \\subseteq \\mathbb{R}^d"), "{tex}");
    }

    #[test]
    fn converts_math_tilde_relation() {
        let tex = typst_math_to_tex("EE_(vx ~ mu) [phi(vx) - vx]");
        assert!(tex.contains("\\mathbf{x} \\sim \\mu"), "{tex}");
        assert!(!tex.contains(" ~ "), "{tex}");
    }

    #[test]
    fn does_not_mangle_otimes() {
        let tex = typst_math_to_tex("vx_1 times.o dots.c times.o vx_n");
        assert!(tex.contains("\\otimes"), "{tex}");
        assert!(!tex.contains("\\o\\times"), "{tex}");
    }

    #[test]
    fn converts_typst_inner_products_and_operators() {
        let tex = typst_math_to_tex("sum_(t=1)^T ip(vx^((t)), vu^((t)))");
        assert!(tex.contains("\\sum_{t=1}^T"), "{tex}");
        assert!(tex.contains("\\left\\langle"), "{tex}");
        assert!(tex.contains("\\mathbf{x}^{\\left(t\\right)}"), "{tex}");
    }

    #[test]
    fn converts_typst_matrices_cases_and_text() {
        let tex = typst_math_to_tex(
            r#"mat(1/2, 0; 0, 1/2), cases("2" & "if" a = "1,", "1" & "if" a = "2,")"#,
        );
        assert!(tex.contains("\\begin{pmatrix}"), "{tex}");
        assert!(tex.contains("\\frac{1}{2} & 0"), "{tex}");
        assert!(tex.contains("0 & \\frac{1}{2}"), "{tex}");
        assert!(!tex.contains("\\frac{\\begin{pmatrix}1}{2}"), "{tex}");
        assert!(tex.contains("\\begin{cases}"), "{tex}");
        assert!(tex.contains("\\mathrm{if}\\ a"), "{tex}");
        assert!(tex.contains("\\mathrm{1,}"), "{tex}");
        assert!(tex.contains("\\mathrm{2,}"), "{tex}");
        assert!(!tex.contains("\\mathrm{1 \\\\ }"), "{tex}");
    }

    #[test]
    fn converts_typst_norm_function() {
        let tex = typst_math_to_tex("norm(vx - vy)_2 <= epsilon");
        assert!(
            tex.contains("\\left\\lVert \\mathbf{x} - \\mathbf{y}\\right\\rVert_2"),
            "{tex}"
        );
        assert!(tex.contains("\\le"), "{tex}");
    }

    #[test]
    fn converts_abs_calls_to_absolute_values() {
        let tex = typst_math_to_tex("abs(ip(vx, vu)) <= 1");
        assert!(
            tex.contains("\\left\\lvert \\left\\langle \\mathbf{x}, \\mathbf{u}\\right\\rangle\\right\\rvert"),
            "{tex}"
        );
        assert!(!tex.contains("abs\\left"), "{tex}");
    }

    #[test]
    fn converts_sqrt_calls_to_latex_radicals() {
        let tex = typst_math_to_tex("O(sqrt(T |cA| log |cA|)) + sqrt(d T)");
        assert!(
            tex.contains("\\sqrt{T |\\mathcal{A}| \\log |\\mathcal{A}|}"),
            "{tex}"
        );
        assert!(tex.contains("\\sqrt{d T}"), "{tex}");
        assert!(!tex.contains("\\sqrt\\left"), "{tex}");
    }

    #[test]
    fn converts_sqrt_denominators_before_fractions() {
        let tex = typst_math_to_tex("(2 B) / sqrt(T) + 2 / sqrt(T)");
        assert!(tex.contains("\\frac{2 B}{\\sqrt{T}}"), "{tex}");
        assert!(tex.contains("\\frac{2}{\\sqrt{T}}"), "{tex}");
        assert!(!tex.contains("{sqrt}"), "{tex}");
        assert!(!tex.contains("sqrt\\left"), "{tex}");
    }

    #[test]
    fn converts_binomial_ell_and_vector_b() {
        let tex = typst_math_to_tex("binom(d, <= ell) + cX = { vx : matA vx <= vb }");
        assert!(tex.contains("\\binom{d}{\\le \\ell}"), "{tex}");
        assert!(tex.contains("\\mathbf{b}"), "{tex}");
        assert!(!tex.contains("binom\\left"), "{tex}");
        assert!(!tex.contains(" ell"), "{tex}");
    }

    #[test]
    fn converts_algorithm_math_shorthands() {
        let tex = typst_math_to_tex("eps + sum_(tau=t - M^k)^(t-1) vu^((tau))");
        assert!(tex.contains("\\epsilon"), "{tex}");
        assert!(tex.contains("\\tau"), "{tex}");
        assert!(!tex.contains("eps +"), "{tex}");
    }

    #[test]
    fn converts_underbrace_and_tilde_o() {
        let tex = typst_math_to_tex(
            r#"underbrace(EE_(t in [T]) ip(vu^((t)), vx^((t))), <= eps) + d^(tilde(O)(1\/epsilon))"#,
        );
        assert!(tex.contains("\\underbrace{"), "{tex}");
        assert!(tex.contains("}_{\\le \\epsilon}"), "{tex}");
        assert!(
            tex.contains("\\tilde{O}\\left(1/\\epsilon\\right)"),
            "{tex}"
        );
    }

    #[test]
    fn converts_profile_chapter_math_shorthands() {
        let tex = typst_math_to_tex(
            "matU^((t)) + overline(kappa)^((T)) - underline(kappa) + kappa^((t)) + vs + vz + conv { vx : vx in cX }",
        );
        assert!(tex.contains("\\mathbf{U}^{\\left(t\\right)}"), "{tex}");
        assert!(tex.contains("\\mathbf{s}"), "{tex}");
        assert!(tex.contains("\\mathbf{z}"), "{tex}");
        assert!(
            tex.contains("\\overline{\\kappa}^{\\left(T\\right)}"),
            "{tex}"
        );
        assert!(tex.contains("\\underline{\\kappa}"), "{tex}");
        assert!(tex.contains("\\kappa^{\\left(t\\right)}"), "{tex}");
        assert!(tex.contains("\\operatorname{conv}"), "{tex}");
        assert!(!tex.contains("matU"), "{tex}");
        assert!(!tex.contains(" conv "), "{tex}");
        assert!(!tex.contains(" vs"), "{tex}");
        assert!(!tex.contains(" vz"), "{tex}");
        assert!(!tex.contains("overline(kappa"), "{tex}");
        assert!(!tex.contains("underline(kappa"), "{tex}");
    }

    #[test]
    fn converts_cal_calls() {
        let tex = typst_math_to_tex("cal(cX)_n + cal(R)_k");
        assert!(tex.contains("\\mathcal{X}_n"), "{tex}");
        assert!(tex.contains("\\mathcal{R}_k"), "{tex}");
        assert!(!tex.contains("cal\\left"), "{tex}");
    }

    #[test]
    fn converts_typst_slash_to_fractions() {
        let tex = typst_math_to_tex("1/T + log(1/epsilon) + (a+b)/(c+d)");
        assert!(tex.contains("\\frac{1}{T}"), "{tex}");
        assert!(
            tex.contains("\\log\\left(\\frac{1}{\\epsilon}\\right)"),
            "{tex}"
        );
        assert!(tex.contains("\\frac{a+b}{c+d}"), "{tex}");
    }

    #[test]
    fn preserves_escaped_typst_slashes() {
        let tex = typst_math_to_tex("M^(1\\/eps) + 1/K");
        assert!(tex.contains("1/"), "{tex}");
        assert!(!tex.contains("M^{\\frac{1}{\\epsilon}}"), "{tex}");
        assert!(!tex.contains("\\\\epsilon"), "{tex}");
        assert!(tex.contains("\\frac{1}{K}"), "{tex}");
    }

    #[test]
    fn renders_typst_sets_with_visible_braces() {
        let tex = typst_math_to_tex(r#"cX = { vx : matA vx <= vb }, {"1", "2"}"#);
        assert!(
            tex.contains(
                "\\left\\{ \\mathbf{x} : \\mathbf{A} \\mathbf{x} \\le \\mathbf{b} \\right\\}"
            ),
            "{tex}"
        );
        assert!(
            tex.contains("\\left\\{\\mathrm{1}, \\mathrm{2}\\right\\}"),
            "{tex}"
        );
    }

    #[test]
    fn keeps_script_group_braces_invisible() {
        let tex = typst_math_to_tex(r#"Phi_{"const"}"#);
        assert!(tex.contains("\\Phi_{\\mathrm{const}}"), "{tex}");
        assert!(!tex.contains("\\Phi_\\{"), "{tex}");
    }

    #[test]
    fn renders_inline_code_spans() {
        let labels = LabelBook::empty();
        let bib = Bibliography::default();
        let mut state = RenderState::default();
        let html = render_inline("call `NextStrategy`() now", &labels, &bib, &mut state);
        assert!(html.contains("<code>NextStrategy</code>"), "{html}");
        assert!(!html.contains("`NextStrategy`"), "{html}");
    }

    #[test]
    fn inline_math_does_not_auto_scale_parentheses() {
        let labels = LabelBook::empty();
        let bib = Bibliography::default();
        let mut state = RenderState::default();
        let html = render_inline("$phi(vx) + [T]$", &labels, &bib, &mut state);
        assert!(html.contains("\\phi(\\mathbf{x})"), "{html}");
        assert!(!html.contains("\\phi\\left(\\mathbf{x}\\right)"), "{html}");
        assert!(html.contains("\\left[T\\right]"), "{html}");
    }

    #[test]
    fn curls_quotes_in_prose() {
        let labels = LabelBook::empty();
        let bib = Bibliography::default();
        let mut state = RenderState::default();
        let html = render_inline(
            r#""Go," she said, "and don't stop learners' choices.""#,
            &labels,
            &bib,
            &mut state,
        );
        assert!(html.contains("&ldquo;Go,&rdquo;"), "{html}");
        assert!(html.contains("&ldquo;and don&rsquo;t"), "{html}");
        assert!(html.contains("learners&rsquo; choices.&rdquo;"), "{html}");
    }

    #[test]
    fn keeps_quotes_straight_in_code_and_math() {
        let labels = LabelBook::empty();
        let bib = Bibliography::default();
        let mut state = RenderState::default();
        let html = render_inline(r#"`"raw"` and $"MC-Err"$"#, &labels, &bib, &mut state);
        assert!(html.contains("<code>&quot;raw&quot;</code>"), "{html}");
        assert!(html.contains("\\(\\mathrm{MC\\text{-}Err}\\)"), "{html}");
        assert!(!html.contains("&ldquo;raw&rdquo;"), "{html}");
    }

    #[test]
    fn renders_hyphens_in_quoted_math_text_as_text_hyphens() {
        let tex = typst_math_to_tex(r#""MC-Err"^((T)) + "such that""#);
        assert!(
            tex.contains("\\mathrm{MC\\text{-}Err}^{\\left(T\\right)}"),
            "{tex}"
        );
        assert!(tex.contains("\\mathrm{such\\ that}"), "{tex}");
    }

    #[test]
    fn renders_typst_nonbreaking_spaces_without_literal_tildes() {
        let labels = LabelBook::empty();
        let bib = Bibliography::default();
        let mut state = RenderState::default();
        let html = render_inline("due to~someone", &labels, &bib, &mut state);
        assert!(html.contains("due to&nbsp;someone"), "{html}");
        assert!(!html.contains("to~someone"), "{html}");
    }

    #[test]
    fn strips_inline_set_commands() {
        let labels = LabelBook::empty();
        let bib = Bibliography::default();
        let mut state = RenderState::default();
        let html = render_inline(
            r#"Matrices #set math.equation(numbering: "(1)") continue"#,
            &labels,
            &bib,
            &mut state,
        );
        assert!(html.contains("Matrices  continue"), "{html}");
        assert!(!html.contains("#set"), "{html}");
    }

    #[test]
    fn avoids_double_parentheses_for_equation_references() {
        let mut labels = LabelBook::empty();
        labels.labels.insert(
            "eq:foo".to_owned(),
            LabelInfo {
                text: "(10)".to_owned(),
                id: "eq-eq-foo".to_owned(),
            },
        );
        let bib = Bibliography::default();

        let mut state = RenderState::default();
        let wrapped = render_inline("Continuing from (@eq:foo),", &labels, &bib, &mut state);
        assert!(
            wrapped.contains("(<a class=\"ref-link\" href=\"#eq-eq-foo\">10</a>)"),
            "{wrapped}"
        );
        assert!(!wrapped.contains("((10))"), "{wrapped}");

        let mut state = RenderState::default();
        let bare = render_inline("See @eq:foo.", &labels, &bib, &mut state);
        assert!(
            bare.contains("<a class=\"ref-link\" href=\"#eq-eq-foo\">(10)</a>"),
            "{bare}"
        );
    }

    #[test]
    fn renders_pdf_style_citation_labels() {
        let mut entries = HashMap::new();
        entries.insert(
            "Dagan24:From".to_owned(),
            BibEntry {
                author: "Yuval Dagan and Constantinos Daskalakis and Maxwell Fishelson and Noah Golowich".to_owned(),
                title: "From External to Swap Regret 2.0".to_owned(),
                booktitle: "Symposium on Theory of Computing (STOC)".to_owned(),
                year: "2024".to_owned(),
                ..BibEntry::default()
            },
        );
        entries.insert(
            "Peng24:Fast".to_owned(),
            BibEntry {
                author: "Binghui Peng and Aviad Rubinstein".to_owned(),
                title: "Fast Swap Regret Minimization".to_owned(),
                year: "2024".to_owned(),
                ..BibEntry::default()
            },
        );
        let bib = Bibliography { entries };

        assert_eq!(bib.cite("Peng24:Fast", true), "Peng and Rubinstein [PR24]");
        assert_eq!(
            bib.cite("Dagan24:From", true),
            "Dagan, Daskalakis, Fishelson and Golowich [Dag+24]"
        );
        assert_eq!(bib.cite("Dagan24:From", false), "[Dag+24]");
        assert_eq!(bib.citation_key("Dagan24:From"), "[Dag+24]");
        assert!(bib
            .full_entry("Dagan24:From")
            .unwrap()
            .contains("Y. Dagan, C. Daskalakis, M. Fishelson and N. Golowich."),);
    }

    #[test]
    fn renders_profile_chapter_citation_labels_from_bib() {
        let mut entries = HashMap::new();
        entries.insert(
            "Arunachaleswaran25:Profile".to_owned(),
            BibEntry {
                author: "Eshwar Ram Arunachaleswaran and Natalie Collina and Yishay Mansour and Mehryar Mohri and Jon Schneider and Balasubramanian Sivan".to_owned(),
                title: "Swap Regret and Correlated Equilibria Beyond Normal-Form Games".to_owned(),
                journal: "arXiv preprint arXiv:2502.20229".to_owned(),
                year: "2025".to_owned(),
                ..BibEntry::default()
            },
        );
        let bib = Bibliography { entries };
        assert_eq!(
            bib.cite("Arunachaleswaran25:Profile", true),
            "Arunachaleswaran, Collina, Mansour, Mohri, Schneider and Sivan [Aru+25]"
        );
        assert_eq!(
            bib.cite("Arunachaleswaran25:Profile", false),
            "[Aru+25]"
        );
        assert!(bib
            .full_entry("Arunachaleswaran25:Profile")
            .unwrap()
            .contains("arXiv:2502.20229"));
    }

    #[test]
    fn renders_citations_with_margin_reference_notes() {
        let mut entries = HashMap::new();
        entries.insert(
            "Peng24:Fast".to_owned(),
            BibEntry {
                author: "Binghui Peng and Aviad Rubinstein".to_owned(),
                title: "Fast Swap Regret Minimization".to_owned(),
                booktitle: "Symposium on Theory of Computing (STOC)".to_owned(),
                year: "2024".to_owned(),
                ..BibEntry::default()
            },
        );
        let bib = Bibliography { entries };
        let mut state = RenderState::default();

        let html = render_citation("Peng24:Fast", true, &bib, &mut state);

        assert!(html.contains("class=\"citation\""), "{html}");
        assert!(html.contains("Peng and Rubinstein [PR24]"), "{html}");
        assert!(html.contains("class=\"citation-note\""), "{html}");
        assert!(html.contains("class=\"citation-note-key\""), "{html}");
        assert!(html.contains("B. Peng and A. Rubinstein."), "{html}");
    }

    #[test]
    fn renders_bibliography_entries_as_keyed_rows() {
        let mut entries = HashMap::new();
        entries.insert(
            "Peng24:Fast".to_owned(),
            BibEntry {
                author: "Binghui Peng and Aviad Rubinstein".to_owned(),
                title: "Fast Swap Regret Minimization".to_owned(),
                year: "2024".to_owned(),
                ..BibEntry::default()
            },
        );
        let bib = Bibliography { entries };
        let state = RenderState {
            citations: vec!["Peng24:Fast".to_owned()],
            ..RenderState::default()
        };

        let html = render_bibliography(&bib, &state);

        assert!(
            html.contains("<span class=\"bib-key\">[PR24]</span>"),
            "{html}"
        );
        assert!(html.contains("<span class=\"bib-entry\">"), "{html}");
        assert!(html.contains("B. Peng and A. Rubinstein."), "{html}");
    }

    #[test]
    fn strips_bibtex_grouping_braces_from_rendered_bib_text() {
        let html = render_bib_text(
            r#"The Complexity of Computing a {N}ash Equilibrium. {SIAM} Journal. Taylor {\&} Francis. {{Lectures on Convex Optimization}}."#,
        );

        assert!(html.contains("a Nash Equilibrium"), "{html}");
        assert!(html.contains("SIAM Journal"), "{html}");
        assert!(html.contains("Taylor &amp; Francis"), "{html}");
        assert!(html.contains("Lectures on Convex Optimization"), "{html}");
        assert!(!html.contains("{N}"), "{html}");
        assert!(!html.contains("{SIAM}"), "{html}");
        assert!(!html.contains("{\\&}"), "{html}");
    }

    #[test]
    fn links_arxiv_identifiers_in_bibliography_text() {
        let html = render_bib_text(
            "arXiv preprint arXiv:2604.19592. DOI: 10.48550/arXiv.1506.08187. https://arxiv.org/abs/1412.6980.",
        );

        assert!(
            html.contains("<a href=\"https://arxiv.org/abs/2604.19592\">arXiv:2604.19592</a>"),
            "{html}"
        );
        assert!(
            html.contains("<a href=\"https://arxiv.org/abs/1506.08187\">arXiv.1506.08187</a>"),
            "{html}"
        );
        assert!(
            html.contains(
                "<a href=\"https://arxiv.org/abs/1412.6980\">https://arxiv.org/abs/1412.6980</a>"
            ),
            "{html}"
        );
    }

    #[test]
    fn appends_arxiv_eprint_when_not_visible_elsewhere() {
        let entry = BibEntry {
            author: "Sébastien Bubeck and Yin Tat Lee".to_owned(),
            title: "A geometric alternative".to_owned(),
            journal: "arXiv".to_owned(),
            eprint: "1506.08187".to_owned(),
            year: "2015".to_owned(),
            ..BibEntry::default()
        };
        let mut entries = HashMap::new();
        entries.insert("Bubeck2015:Geometric".to_owned(), entry);
        let bib = Bibliography { entries };
        let full = bib.full_entry("Bubeck2015:Geometric").unwrap();
        assert!(full.contains("arXiv:1506.08187"), "{full}");
    }

    #[test]
    fn suppresses_nearby_repeated_margin_reference_notes() {
        let mut entries = HashMap::new();
        entries.insert(
            "Peng24:Fast".to_owned(),
            BibEntry {
                author: "Binghui Peng and Aviad Rubinstein".to_owned(),
                title: "Fast Swap Regret Minimization".to_owned(),
                year: "2024".to_owned(),
                ..BibEntry::default()
            },
        );
        let bib = Bibliography { entries };
        let labels = LabelBook::empty();
        let mut state = RenderState::default();

        let nearby = render_inline(
            "#citet(<Peng24:Fast>) and #citep(<Peng24:Fast>)",
            &labels,
            &bib,
            &mut state,
        );
        assert_eq!(
            nearby.matches("class=\"citation-note\"").count(),
            1,
            "{nearby}"
        );

        let far = render_inline(
            &format!(
                "{} #citep(<Peng24:Fast>)",
                "x".repeat(CITATION_MARGIN_NOTE_MIN_CHARS)
            ),
            &labels,
            &bib,
            &mut state,
        );
        assert_eq!(far.matches("class=\"citation-note\"").count(), 1, "{far}");
    }

    #[test]
    fn collects_footnotes_for_narrow_endnotes() {
        let labels = LabelBook::empty();
        let bib = Bibliography::default();
        let mut state = RenderState::default();

        let inline = render_inline("Text #footnote[side text].", &labels, &bib, &mut state);
        let endnotes = render_endnotes(&state);

        assert!(inline.contains("class=\"footnote\""), "{inline}");
        assert!(inline.contains("href=\"#fn-end-1\""), "{inline}");
        assert!(endnotes.contains("class=\"endnotes\""), "{endnotes}");
        assert!(endnotes.contains("id=\"fn-end-1\""), "{endnotes}");
        assert!(
            endnotes.contains("<a class=\"footnote-backref\" href=\"#fnref-1\">1</a><span class=\"footnote-body\">side text</span>"),
            "{endnotes}"
        );
        assert!(endnotes.contains("side text"), "{endnotes}");
    }

    #[test]
    fn renders_treeswap_as_code() {
        let labels = LabelBook::empty();
        let bib = Bibliography::default();
        let mut state = RenderState::default();
        let html = render_inline("TreeSwap algorithm", &labels, &bib, &mut state);
        assert!(html.contains("<code>TreeSwap</code>"), "{html}");
    }

    #[test]
    fn renders_todo_commands() {
        let labels = LabelBook::empty();
        let bib = Bibliography::default();
        let mut state = RenderState::default();
        let html = render_inline(
            r#"#todo("finish this") and #todo[that]"#,
            &labels,
            &bib,
            &mut state,
        );
        assert!(
            html.contains("<mark class=\"todo\">finish this</mark>"),
            "{html}"
        );
        assert!(html.contains("<mark class=\"todo\">that</mark>"), "{html}");
    }

    #[test]
    fn parses_pseudocode_line_labels_and_layout_commands() {
        let items = parse_pseudocode_items(
            r#"
  - Set $vx := vy$ #h(1fr) <line:step>
"#,
        );
        match &items[0] {
            PseudoItem::Step {
                text,
                label,
                indent,
            } => {
                assert_eq!(*indent, 1);
                assert_eq!(label.as_deref(), Some("line:step"));
                assert!(!text.contains("#h"), "{text}");
                assert!(!text.contains("<line:"), "{text}");
            }
            item => panic!("expected step, got {item:?}"),
        }
    }

    #[test]
    fn strips_trailing_space_before_suffix_labels() {
        let mut text = "A framework for minimizing $Phi$-regret <sec:Gordon>".to_owned();
        let label = pop_label_suffix(&mut text);

        assert_eq!(label.as_deref(), Some("sec:Gordon"));
        assert_eq!(text, "A framework for minimizing $Phi$-regret");
    }

    #[test]
    fn renders_pseudocode_block_guides_with_hooks() {
        let continuing = render_pseudo_guides(2, 2);
        assert_eq!(continuing.matches("class=\"pseudo-guide\"").count(), 2);
        assert!(!continuing.contains("pseudo-guide-end"), "{continuing}");

        let ending_inner = render_pseudo_guides(2, 1);
        assert!(
            ending_inner.contains("style=\"--guide:1\""),
            "{ending_inner}"
        );
        assert!(
            ending_inner.contains("class=\"pseudo-guide pseudo-guide-end\" style=\"--guide:2\""),
            "{ending_inner}"
        );
    }

    #[test]
    fn styles_algorithm_label_but_not_caption_as_bold() {
        let css = page_css();
        assert!(css.contains(".algorithm-label{font-weight:700}"), "{css}");
        assert!(css.contains(".algorithm-caption{font-weight:400}"), "{css}");
        assert!(
            css.contains(".env.algorithm .env-title{")
                && css.contains("font-weight:400;line-height"),
            "{css}"
        );
    }

    #[test]
    fn chapter_rail_title_links_home_and_shows_ec_branding() {
        let config = Config {
            input: PathBuf::from("P1-introduction.typ"),
            output: PathBuf::from("public/P1-introduction.html"),
            root: PathBuf::from("."),
            title: None,
            site_title: DEFAULT_SITE_TITLE.to_owned(),
            authors: DEFAULT_AUTHORS.to_owned(),
            index_href: Some("index.html".to_owned()),
            pdf_href: None,
        };
        let rail = render_chapter_rail(
            0,
            &[],
            &LabelBook::empty(),
            &Bibliography::default(),
            &config,
        );
        assert!(rail.contains("ACM EC&rsquo;26 Tutorial"), "{rail}");
        assert!(
            rail.contains("<a class=\"course-title course-title-link\" href=\"index.html\">Learning and Computation of Phi-Equilibria</a>"),
            "{rail}"
        );
        assert!(!rail.contains(">Home</a>"), "{rail}");

        assert!(!rail.contains("Download as PDF"), "{rail}");

        let css = page_css();
        assert!(
            css.contains(".course-title-link:hover{color:var(--accent)!important"),
            "{css}"
        );
        assert!(
            css.contains("--rail-left:max(24px,calc((100vw - var(--page-total))/2 - var(--rail-width) - var(--rail-gutter)))"),
            "{css}"
        );
        assert!(css.contains("--rail-gutter:2cm"), "{css}");
        assert!(
            css.contains("@media(max-width:1260px){.chapter-rail{display:none}}"),
            "{css}"
        );
        assert!(
            css.contains("@media(min-width:1261px){.toc{display:none}}"),
            "{css}"
        );
    }

    #[test]
    fn renders_chapter_citation_sidenote() {
        let note = render_chapter_citation_sidenote(
            3,
            "Phi-Regret and Multicalibration",
            DEFAULT_SITE_TITLE,
            Some("../P4-multicalibration.pdf"),
        );
        assert!(
            note.contains("class=\"chapter-citation-sidenote\""),
            "{note}"
        );
        assert!(
            note.contains("<details class=\"chapter-citation-details\">"),
            "{note}"
        );
        assert!(
            note.contains("<a class=\"chapter-citation-pdf\" href=\"../P4-multicalibration.pdf\">Download as PDF</a>"),
            "{note}"
        );
        assert!(
            note.find("Download as PDF").unwrap() < note.find("How to cite").unwrap(),
            "{note}"
        );
        assert!(!note.contains("<details open"), "{note}");
        assert!(
            note.contains("<summary class=\"chapter-citation-title\">How to cite</summary>"),
            "{note}"
        );
        assert!(note.contains("How to cite"), "{note}");
        assert!(
            note.contains("@misc{anagnostides-farina-zhang-2026-phi-equilibria-chapter-4"),
            "{note}"
        );
        assert!(
            note.contains("title = {Chapter 4: Phi-Regret and Multicalibration}"),
            "{note}"
        );
        assert!(note.contains("url = {P4-multicalibration.html}"), "{note}");

        let css = page_css();
        assert!(
            css.contains(".chapter-citation-title::before{content:\"\""),
            "{css}"
        );
        assert!(
            css.contains(".chapter-citation-details[open] .chapter-citation-title::before{transform:rotate(90deg)}"),
            "{css}"
        );
    }

    #[test]
    fn chapter_body_prefers_new_computer_modern() {
        let css = page_css();
        assert!(
            css.contains("src:url(\"NewCM10-Regular.otf\") format(\"opentype\")"),
            "{css}"
        );
        assert!(css.contains("NewCM10-Italic.otf"), "{css}");
        assert!(css.contains("NewCM10-Bold.otf"), "{css}");
        assert!(css.contains("NewCM10-BoldItalic.otf"), "{css}");
        assert!(css.contains("font-family:\"NewComputerModern10\""), "{css}");
        assert!(css.contains("font-family:\"New Computer Modern\""), "{css}");
        assert!(css.contains("font-family:\"NewComputerModern 10\""), "{css}");
        assert!(css.contains("--serif:\"New Computer Modern\""), "{css}");
        assert!(css.contains("\"New Computer Modern\""), "{css}");
        assert!(
            css.contains(
                "body{margin:0;color:var(--ink);background:var(--paper);font-family:var(--serif)"
            ),
            "{css}"
        );
        assert!(
            css.contains("font-family:\"Frutiger Notes\",var(--serif)"),
            "{css}"
        );
        assert!(
            css.contains(".env.proof .env-title{font-family:var(--serif);"),
            "{css}"
        );
    }

    #[test]
    fn toc_and_rail_section_titles_use_fixed_number_columns() {
        let css = page_css();
        assert!(
            css.contains(".toc a{display:grid;grid-template-columns:3rem minmax(0,1fr);align-items:baseline}"),
            "{css}"
        );
        assert!(!css.contains(".toc a{display:flex"), "{css}");
        assert!(
            !css.contains(".toc a{display:flex;align-items:baseline;gap"),
            "{css}"
        );
        assert!(
            css.contains(".chapter-section-link{display:grid!important;grid-template-columns:2.55rem minmax(0,1fr);align-items:baseline"),
            "{css}"
        );
        assert!(css.contains(".toc-title{min-width:0}"), "{css}");
        assert!(css.contains(".chapter-section-title{min-width:0}"), "{css}");

        let labels = LabelBook::empty();
        let bib = Bibliography::default();
        let headings = vec![Heading {
            level: 1,
            number: "1.3".to_owned(),
            id: "sec".to_owned(),
            text: "A framework for minimizing $Phi$-regret".to_owned(),
        }];
        let toc = render_toc(&headings, &labels, &bib);
        assert!(toc.contains("<span class=\"toc-title\">"), "{toc}");
    }

    #[test]
    fn lets_wide_display_equations_enter_side_margin() {
        let css = page_css();
        assert!(css.contains(".equation{box-sizing:border-box;"), "{css}");
        assert!(css.contains(".equation.is-overwide{clear:right}"), "{css}");
        assert!(
            css.contains(
                "@media(min-width:1260px){.equation.is-overwide{width:calc(100% + 234px);overflow-x:visible}"
            ),
            "{css}"
        );
        assert!(
            css.contains(
                "@media(max-width:1259px){.equation.is-overwide{display:flex;align-items:center;gap:.5rem;overflow-x:auto;padding-right:0}"
            ),
            "{css}"
        );
    }

    #[test]
    fn marks_only_oversized_equations_after_katex_render() {
        let script = equation_width_script();
        assert!(script.contains("markOverwideEquations"), "{script}");
        assert!(
            script.contains("window.markOverwideEquations = markOverwideEquations"),
            "{script}"
        );
        assert!(
            script.contains("document.fonts.ready.then(markOverwideEquations)"),
            "{script}"
        );
        assert!(
            script.contains("eq.classList.remove(\"is-overwide\")"),
            "{script}"
        );
        assert!(script.contains("needed > available + 2"), "{script}");
        assert!(
            script.contains("eq.classList.add(\"is-overwide\")"),
            "{script}"
        );
    }

    #[test]
    fn wraps_aligned_display_math() {
        let tex = display_math_to_tex("a &:= b \\\nc &<= d");
        assert!(tex.starts_with("\\begin{aligned}"), "{tex}");
        assert!(tex.contains(r" \\"));
        assert!(tex.contains("\\coloneqq"), "{tex}");
    }

    #[test]
    fn extracts_typst_callout_body() {
        let body =
            extract_callout_body("#align(center, box(\n  inset: 3mm,\n)[Hello $Phi$ world])")
                .expect("callout body");
        assert_eq!(body.trim(), "Hello $Phi$ world");
    }

    #[test]
    fn converts_svg_pt_width_to_css_pixels() {
        assert_eq!(svg_length_to_css_px("373.5pt").as_deref(), Some("498px"));
        let asset = FigureAsset {
            stem: "chicken".to_owned(),
            src: "figures/chicken.svg".to_owned(),
            css_width: Some("498px".to_owned()),
        };
        assert_eq!(
            figure_img_style(&asset),
            " style=\"width:min(100%, 498px)\""
        );
        let mcroute = FigureAsset {
            stem: "mcroute".to_owned(),
            src: "figures/mcroute.svg".to_owned(),
            css_width: Some("498px".to_owned()),
        };
        assert_eq!(figure_img_style(&mcroute), " style=\"width:80%\"");
    }

    #[test]
    fn distinguishes_inline_math_at_line_start_from_display_math() {
        assert!(!is_display_math_start(
            "$mu in Delta(cA)$ is an $epsilon$-equilibrium"
        ));
        assert!(!is_display_math_start("$i in [n]$"));
        assert!(is_display_math_start("$ phi: a |-> b $"));
        assert!(is_display_math_start("$ <eq:label>"));
    }
}
