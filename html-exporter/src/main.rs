mod math;
mod options;

use math::MathMode;
use options::Config;
use regex::{Captures, Regex};
use scraper::{ElementRef, Html, Selector};
use std::collections::{HashMap, HashSet};
use std::env;
use std::fmt::Write as _;
use std::fs;
use std::path::{Path, PathBuf};
use std::sync::OnceLock;
use typst::diag::{FileError, FileResult, SourceDiagnostic};
use typst::foundations::{Bytes, Datetime, Dict, IntoValue};
use typst::syntax::{FileId, Source, VirtualPath};
use typst::text::{Font, FontBook};
use typst::utils::LazyHash;
use typst::{Feature, Library, LibraryExt, World};
use typst_html::HtmlDocument;
use typst_kit::download::{Downloader, ProgressSink};
use typst_kit::fonts::{FontSlot, Fonts};
use typst_kit::package::PackageStorage;

const BIBTEX_AUTHORS: &str = "Anagnostides, Ioannis and Farina, Gabriele and Zhang, Brian Hu";
const PAGE_CSS: &str = include_str!("gabri-notes.css");

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
    let config = options::parse()?;
    let raw_html = compile_typst_html(&config)?;
    let mut document = HtmlParts::parse(&raw_html);
    let title = config
        .title
        .clone()
        .or_else(|| document.meta.title.clone())
        .unwrap_or_else(|| title_from_path(&config.input));

    document.rewrite_heading_ids();
    let (body_html, rendered_endnotes) = postprocess_body(
        document.body_html,
        &document.bibliography,
        &document.endnotes,
        config.math_mode,
    );
    document.body_html = body_html;
    document.rendered_endnotes = rendered_endnotes;

    let html = render_document(&config, &title, &document);
    write_output(&config, html)?;
    Ok(())
}

fn compile_typst_html(config: &Config) -> Result<String, String> {
    let world = LocalWorld::new(&config.input, &config.root, config.math_mode)?;
    let warned = typst::compile::<HtmlDocument>(&world);
    for warning in &warned.warnings {
        eprintln!("typst warning: {}", format_diagnostic(warning));
    }
    let document = warned
        .output
        .map_err(|errors| format_diagnostics("Typst HTML compilation failed", &errors))?;
    typst_html::html(&document)
        .map_err(|errors| format_diagnostics("Typst HTML encoding failed", &errors))
}

struct LocalWorld {
    root: PathBuf,
    main: FileId,
    library: LazyHash<Library>,
    book: LazyHash<FontBook>,
    fonts: Vec<FontSlot>,
    package_storage: PackageStorage,
}

impl LocalWorld {
    fn new(input: &Path, root: &Path, math_mode: MathMode) -> Result<Self, String> {
        let root = root
            .canonicalize()
            .map_err(|err| format!("could not canonicalize root {}: {err}", root.display()))?;
        let input = if input.is_absolute() {
            input.to_path_buf()
        } else {
            env::current_dir()
                .map_err(|err| format!("could not read current directory: {err}"))?
                .join(input)
        };
        let input = input
            .canonicalize()
            .map_err(|err| format!("could not canonicalize input {}: {err}", input.display()))?;
        let main_path = VirtualPath::within_root(&input, &root).ok_or_else(|| {
            format!(
                "input {} is outside root {}",
                input.display(),
                root.display()
            )
        })?;
        let main = FileId::new(None, main_path);

        let mut inputs = Dict::new();
        inputs.insert("html".into(), "true".into_value());
        inputs.insert("combined".into(), "false".into_value());
        inputs.insert("html-math".into(), math_mode.as_typst_input().into_value());
        let features = [Feature::Html].into_iter().collect();
        let library = Library::builder()
            .with_inputs(inputs)
            .with_features(features)
            .build();

        let font_paths = [root.clone(), root.join("public")];
        let fonts = Fonts::searcher().search_with(font_paths);
        let package_storage =
            PackageStorage::new(None, None, Downloader::new("notes-html-exporter/0.1"));

        Ok(Self {
            root,
            main,
            library: LazyHash::new(library),
            book: LazyHash::new(fonts.book),
            fonts: fonts.fonts,
            package_storage,
        })
    }

    fn system_path(&self, id: FileId) -> FileResult<PathBuf> {
        let package_root;
        let root = if let Some(spec) = id.package() {
            let mut progress = ProgressSink;
            package_root = self.package_storage.prepare_package(spec, &mut progress)?;
            &package_root
        } else {
            &self.root
        };
        id.vpath().resolve(root).ok_or(FileError::AccessDenied)
    }

    fn read_bytes(&self, id: FileId) -> FileResult<Vec<u8>> {
        let path = self.system_path(id)?;
        let file_error = |err| FileError::from_io(err, &path);
        if fs::metadata(&path).map_err(file_error)?.is_dir() {
            Err(FileError::IsDirectory)
        } else {
            fs::read(&path).map_err(file_error)
        }
    }
}

impl World for LocalWorld {
    fn library(&self) -> &LazyHash<Library> {
        &self.library
    }

    fn book(&self) -> &LazyHash<FontBook> {
        &self.book
    }

    fn main(&self) -> FileId {
        self.main
    }

    fn source(&self, id: FileId) -> FileResult<Source> {
        let bytes = self.read_bytes(id)?;
        let bytes = bytes.strip_prefix(b"\xef\xbb\xbf").unwrap_or(&bytes);
        let text = std::str::from_utf8(bytes)?;
        let text = if id.package().is_none() {
            use_html_notes_style(text)
        } else {
            text.to_owned()
        };
        Ok(Source::new(id, text.into()))
    }

    fn file(&self, id: FileId) -> FileResult<Bytes> {
        Ok(Bytes::new(self.read_bytes(id)?))
    }

    fn font(&self, index: usize) -> Option<Font> {
        self.fonts.get(index)?.get()
    }

    fn today(&self, _offset: Option<i64>) -> Option<Datetime> {
        Datetime::from_ymd(2026, 6, 7)
    }
}

fn format_diagnostics(prefix: &str, diagnostics: &[SourceDiagnostic]) -> String {
    let mut out = String::from(prefix);
    for diagnostic in diagnostics {
        out.push('\n');
        out.push_str(&format_diagnostic(diagnostic));
    }
    out
}

fn format_diagnostic(diagnostic: &SourceDiagnostic) -> String {
    let mut out = diagnostic.message.to_string();
    for hint in &diagnostic.hints {
        write!(out, "\n  hint: {hint}").unwrap();
    }
    out
}

fn use_html_notes_style(source: &str) -> String {
    source
        .replace(
            r#"#import "meta/gabri_notes.typ": *"#,
            r#"#import "meta/gabri_notes_html.typ": *"#,
        )
        .replace(
            r#"#import "../meta/gabri_notes.typ": *"#,
            r#"#import "../meta/gabri_notes_html.typ": *"#,
        )
}

#[derive(Clone, Debug)]
struct Heading {
    level: u8,
    text: String,
    id: String,
    number: String,
}

struct HtmlParts {
    meta: DocumentMeta,
    body_html: String,
    headings: Vec<Heading>,
    bibliography: Vec<BibliographyItem>,
    endnotes: Vec<Endnote>,
    rendered_endnotes: Vec<RenderedEndnote>,
}

impl HtmlParts {
    fn parse(raw_html: &str) -> Self {
        let dom = Html::parse_document(raw_html);
        let body_html = select_first(&dom, "body")
            .map(|body| body.inner_html())
            .unwrap_or_else(|| raw_html.to_owned());
        let meta = extract_document_meta(&dom);
        let headings = extract_headings(&dom);
        let bibliography = extract_bibliography(&dom);
        let endnotes = extract_endnotes(&dom);
        Self {
            meta,
            body_html,
            headings,
            bibliography,
            endnotes,
            rendered_endnotes: Vec::new(),
        }
    }

    fn rewrite_heading_ids(&mut self) {
        let mut heading_idx = 0usize;
        self.body_html = re_html_heading()
            .replace_all(&self.body_html, |captures: &Captures| {
                let whole = captures.get(0).map_or("", |m| m.as_str());
                let level = captures.name("level").map_or("1", |m| m.as_str());
                let attrs = captures.name("attrs").map_or("", |m| m.as_str());
                let inner = captures.name("inner").map_or("", |m| m.as_str());
                if !attrs.contains("notes-heading")
                    || inner.contains("Bibliography for this chapter")
                {
                    return whole.to_owned();
                }
                let Some(heading) = self.headings.get(heading_idx) else {
                    return whole.to_owned();
                };
                heading_idx += 1;
                format!(
                    "<h{level}{}>{inner}</h{level}>",
                    set_id_attr(attrs, &heading.id)
                )
            })
            .to_string();
    }
}

#[derive(Clone, Default)]
struct DocumentMeta {
    lecture_number: Option<String>,
    title: Option<String>,
}

#[derive(Clone)]
struct BibliographyItem {
    id: String,
    key_text: String,
    entry_html: String,
}

#[derive(Clone)]
struct Endnote {
    id: String,
    label: String,
    body_html: String,
}

#[derive(Clone)]
struct RenderedEndnote {
    number: String,
    body_html: String,
}

fn extract_document_meta(dom: &Html) -> DocumentMeta {
    let selector = Selector::parse(".notes-meta").unwrap();
    let Some(element) = dom.select(&selector).next() else {
        return DocumentMeta::default();
    };
    DocumentMeta {
        lecture_number: non_empty_attr(&element, "data-lecture-number"),
        title: non_empty_attr(&element, "data-title"),
    }
}

fn extract_headings(dom: &Html) -> Vec<Heading> {
    let selector = Selector::parse("body h1, body h2, body h3, body h4, body h5, body h6").unwrap();
    let secno_selector = Selector::parse(".secno").unwrap();
    let mut headings = Vec::new();

    for element in dom.select(&selector) {
        let class = element.value().attr("class").unwrap_or_default();
        if !class.split_whitespace().any(|name| name == "notes-heading")
            || class
                .split_whitespace()
                .any(|name| name == "notes-heading-unnumbered")
        {
            continue;
        }
        let raw_id = element.value().attr("id").map(str::to_owned);
        let level = element
            .value()
            .attr("data-level")
            .and_then(|level| level.parse::<u8>().ok())
            .or_else(|| {
                element
                    .value()
                    .name()
                    .strip_prefix('h')
                    .and_then(|level| level.parse::<u8>().ok())
            })
            .unwrap_or(1);
        let number = element
            .value()
            .attr("data-number")
            .map(normalize_ws)
            .filter(|number| !number.is_empty())
            .or_else(|| {
                element
                    .select(&secno_selector)
                    .next()
                    .map(|secno| normalize_ws(&secno.text().collect::<Vec<_>>().join(" ")))
            })
            .unwrap_or_default();
        let text = heading_text(&element, &number, &secno_selector);
        let id = raw_id.unwrap_or_else(|| slugify(&format!("{number}-{text}")));
        headings.push(Heading {
            level,
            id,
            number,
            text,
        });
    }

    headings
}

fn heading_text(element: &ElementRef, number: &str, secno_selector: &Selector) -> String {
    let full = normalize_ws(&element.text().collect::<Vec<_>>().join(" "));
    let secno = element
        .select(secno_selector)
        .next()
        .map(|secno| normalize_ws(&secno.text().collect::<Vec<_>>().join(" ")))
        .filter(|secno| !secno.is_empty())
        .unwrap_or_else(|| number.to_owned());
    let text = full
        .strip_prefix(&secno)
        .map(str::trim)
        .filter(|text| !text.is_empty())
        .unwrap_or(&full);
    text.to_owned()
}

fn non_empty_attr(element: &ElementRef, name: &str) -> Option<String> {
    element
        .value()
        .attr(name)
        .map(str::trim)
        .filter(|value| !value.is_empty())
        .map(str::to_owned)
}

fn chapter_index_from_input(input: &Path) -> Option<usize> {
    let file_name = input.file_name()?.to_string_lossy();
    CHAPTERS
        .iter()
        .position(|chapter| chapter.source == file_name.as_ref())
}

fn current_chapter(config: &Config) -> Option<usize> {
    chapter_index_from_input(&config.input)
}

fn extract_bibliography(dom: &Html) -> Vec<BibliographyItem> {
    let item_selector = Selector::parse("section[role=\"doc-bibliography\"] li").unwrap();
    let prefix_selector = Selector::parse("span.prefix").unwrap();
    let mut items = Vec::new();

    for item in dom.select(&item_selector) {
        let Some(id) = item.value().attr("id") else {
            continue;
        };
        let key_text = item
            .select(&prefix_selector)
            .next()
            .map(|prefix| normalize_ws(&prefix.text().collect::<Vec<_>>().join(" ")))
            .unwrap_or_default();
        let entry_html = re_bib_prefix()
            .replace(&item.inner_html(), "")
            .trim()
            .to_owned();
        items.push(BibliographyItem {
            id: id.to_owned(),
            key_text,
            entry_html: math::normalize_bibliography_math(&entry_html),
        });
    }

    items
}

fn extract_endnotes(dom: &Html) -> Vec<Endnote> {
    let item_selector = Selector::parse("section[role=\"doc-endnotes\"] li").unwrap();
    let sup_selector = Selector::parse("sup").unwrap();
    let mut notes = Vec::new();

    for item in dom.select(&item_selector) {
        let Some(id) = item.value().attr("id") else {
            continue;
        };
        let label = item
            .select(&sup_selector)
            .next()
            .map(|sup| normalize_ws(&sup.text().collect::<Vec<_>>().join(" ")))
            .unwrap_or_else(|| (notes.len() + 1).to_string());
        let body_html = re_endnote_backlink()
            .replace(&item.inner_html(), "")
            .trim()
            .to_owned();
        notes.push(Endnote {
            id: id.to_owned(),
            label,
            body_html,
        });
    }

    notes
}

fn select_first<'a>(dom: &'a Html, selector: &str) -> Option<ElementRef<'a>> {
    let selector = Selector::parse(selector).ok()?;
    dom.select(&selector).next()
}

fn postprocess_body(
    body_html: String,
    bibliography: &[BibliographyItem],
    endnotes: &[Endnote],
    math_mode: MathMode,
) -> (String, Vec<RenderedEndnote>) {
    let mut body = body_html;
    body = re_notes_meta().replace_all(&body, "").to_string();
    body = re_hidden_bibliography().replace_all(&body, "").to_string();
    body = re_endnotes_section().replace_all(&body, "").to_string();
    body = re_empty_hidden_div().replace_all(&body, "").to_string();
    body = normalize_typst_classes(body);
    body = math::postprocess_html_math(body, math_mode);
    let (body_without_bibliography, bibliography_blocks) = protect_visible_bibliographies(body);
    let body = rewrite_citations(body_without_bibliography, bibliography);
    let body = restore_visible_bibliographies(body, bibliography_blocks);
    rewrite_footnotes(body, endnotes)
}

fn normalize_typst_classes(mut body: String) -> String {
    body = body.replace(
        "<figure role=\"math\"",
        "<figure class=\"equation\" role=\"math\"",
    );
    body = re_typst_figure_class()
        .replace_all(&body, "${prefix}rendered-figure typst${suffix}")
        .to_string();
    body
}

fn rewrite_citations(body: String, bibliography: &[BibliographyItem]) -> String {
    let by_id = bibliography
        .iter()
        .map(|item| (item.id.as_str(), item))
        .collect::<HashMap<_, _>>();
    let mut noted = HashSet::new();

    re_biblioref()
        .replace_all(&body, |captures: &Captures| {
            let attrs = captures.get(1).map_or("", |m| m.as_str());
            let label = captures.get(2).map_or("", |m| m.as_str());
            let href = extract_href(attrs);
            let attrs = add_class_to_attrs(attrs, "citation");
            let mut out = format!("<span class=\"citation-wrap\"><a {attrs}>{label}</a>");
            if let Some(item) = href.and_then(|href| by_id.get(href)) {
                if !noted.insert(item.id.as_str()) {
                    out.push_str("</span>");
                    return out;
                }
                write!(
                    out,
                    "<span class=\"citation-note\"><span class=\"citation-note-key\">{}</span> {}</span>",
                    escape_html(&item.key_text),
                    item.entry_html
                )
                .unwrap();
            }
            out.push_str("</span>");
            out
        })
        .to_string()
}

fn protect_visible_bibliographies(body: String) -> (String, Vec<String>) {
    let mut blocks = Vec::new();
    let body = re_visible_bibliography()
        .replace_all(&body, |captures: &Captures| {
            let index = blocks.len();
            let block = captures.get(0).map_or("", |m| m.as_str());
            blocks.push(clean_visible_bibliography(block));
            format!("<!--NOTES_HTML_BIBLIOGRAPHY_{index}-->")
        })
        .to_string();
    (body, blocks)
}

fn restore_visible_bibliographies(mut body: String, blocks: Vec<String>) -> String {
    for (index, block) in blocks.into_iter().enumerate() {
        body = body.replace(&format!("<!--NOTES_HTML_BIBLIOGRAPHY_{index}-->"), &block);
    }
    body
}

fn clean_visible_bibliography(block: &str) -> String {
    let block = re_bibliography_row()
        .replace_all(block, |captures: &Captures| {
            let whole = captures.get(0).map_or("", |m| m.as_str());
            let attrs = captures.name("attrs").map_or("", |m| m.as_str());
            let inner = captures.name("inner").map_or("", |m| m.as_str());
            let Some(id) = doc_biblioref_target(inner) else {
                return whole.to_owned();
            };
            if attrs.contains(" id=") || attrs.contains(" id='") {
                whole.to_owned()
            } else {
                format!("<tr id=\"{}\"{}>{}</tr>", escape_attr(id), attrs, inner)
            }
        })
        .to_string();

    let block = re_doc_biblioref_link()
        .replace_all(&block, |captures: &Captures| {
            let whole = captures.get(0).map_or("", |m| m.as_str());
            let attrs = captures.name("attrs").map_or("", |m| m.as_str());
            if attrs.contains(r#"role="doc-biblioref""#) {
                captures.name("inner").map_or("", |m| m.as_str()).to_owned()
            } else {
                whole.to_owned()
            }
        })
        .to_string();

    math::normalize_bibliography_math(&block)
}

fn doc_biblioref_target(html: &str) -> Option<&str> {
    re_doc_biblioref_href_before_role()
        .captures(html)
        .or_else(|| re_doc_biblioref_href_after_role().captures(html))
        .and_then(|captures| captures.name("id"))
        .map(|m| m.as_str())
}

fn rewrite_footnotes(body: String, endnotes: &[Endnote]) -> (String, Vec<RenderedEndnote>) {
    let by_id = endnotes
        .iter()
        .map(|note| (note.id.as_str(), note))
        .collect::<HashMap<_, _>>();
    let mut rendered = Vec::new();
    let mut next = 1usize;

    let body = re_noteref()
        .replace_all(&body, |captures: &Captures| {
            let attrs = captures.get(1).map_or("", |m| m.as_str());
            let fallback_label = captures.get(2).map_or("", |m| m.as_str());
            let Some(href) = extract_href(attrs) else {
                return captures.get(0).unwrap().as_str().to_owned();
            };
            let Some(note) = by_id.get(href) else {
                return captures.get(0).unwrap().as_str().to_owned();
            };
            let number = if note.label.is_empty() {
                next.to_string()
            } else {
                note.label.clone()
            };
            let seq = next;
            next += 1;
            rendered.push(RenderedEndnote {
                number: number.clone(),
                body_html: note.body_html.clone(),
            });
            format!(
                "<sup class=\"footnote-ref\" id=\"fnref-{seq}\"><a href=\"#fn-end-{seq}\">{}</a></sup><span class=\"footnote\" id=\"fn-side-{seq}\"><span class=\"footnote-num\">{}</span>{}</span>",
                escape_html(if number.is_empty() { fallback_label } else { &number }),
                escape_html(&number),
                note.body_html
            )
        })
        .to_string();

    (body, rendered)
}

fn render_document(config: &Config, title: &str, document: &HtmlParts) -> String {
    let current = current_chapter(config);
    let mut html = String::new();
    html.push_str("<!doctype html>\n<html lang=\"en\">\n<head>\n");
    html.push_str("  <meta charset=\"utf-8\">\n");
    html.push_str("  <meta name=\"viewport\" content=\"width=device-width, initial-scale=1\">\n");
    html.push_str(math::katex_head_assets(config.math_mode));
    write!(html, "  <title>{}</title>\n", escape_html(title)).unwrap();
    html.push_str("  <style>\n");
    html.push_str(PAGE_CSS);
    html.push_str("\n  </style>\n");
    html.push_str(math::katex_script_assets(config.math_mode));
    html.push_str("</head>\n<body>\n");

    if let Some(current) = current {
        html.push_str(&render_chapter_rail(current, &document.headings, config));
    } else {
        html.push_str(&render_masthead(config));
    }

    html.push_str("<main class=\"page-shell\">\n<article class=\"lecture-content\"");
    if let Some(number) = &document.meta.lecture_number {
        write!(html, " data-lecture-number=\"{}\"", escape_attr(number)).unwrap();
    }
    html.push_str(">\n");
    if let Some(current) = current {
        write!(
            html,
            "<p class=\"chapter-kicker\">Chapter {}</p>\n",
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
    if let Some(current) = current {
        html.push_str(&render_chapter_citation_sidenote(
            current,
            title,
            &config.site_title,
            config.pdf_href.as_deref(),
        ));
    }
    if !document.headings.is_empty() {
        html.push_str(&render_toc(&document.headings));
    }
    html.push_str(&document.body_html);
    html.push_str(&render_endnotes(&document.rendered_endnotes));
    html.push_str("</article>\n</main>\n");
    if current.is_some() {
        html.push_str(chapter_nav_script());
    }
    html.push_str(equation_width_script());
    html.push_str("</body>\n</html>\n");
    html
}

fn render_masthead(config: &Config) -> String {
    let mut out = String::from("<header class=\"site-masthead\">\n");
    if let Some(index) = &config.index_href {
        write!(
            out,
            "<a class=\"course-title course-title-link\" href=\"{}\">{}</a>\n",
            escape_attr(index),
            escape_html(&config.site_title)
        )
        .unwrap();
    } else {
        write!(
            out,
            "<div class=\"course-title\">{}</div>\n",
            escape_html(&config.site_title)
        )
        .unwrap();
    }
    write!(
        out,
        "<div class=\"course-authors\">{}</div>\n",
        escape_html(&config.authors)
    )
    .unwrap();
    if config.index_href.is_some() || config.pdf_href.is_some() {
        out.push_str("<div class=\"top-links\">");
        if let Some(index) = &config.index_href {
            write!(out, "<a href=\"{}\">Index</a>", escape_attr(index)).unwrap();
        }
        if let Some(pdf) = &config.pdf_href {
            write!(out, "<a href=\"{}\">PDF</a>", escape_attr(pdf)).unwrap();
        }
        out.push_str("</div>");
    }
    out.push_str("</header>\n");
    out
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

fn render_chapter_rail(current: usize, headings: &[Heading], config: &Config) -> String {
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
        "<div class=\"course-authors\">{}</div></div>\n",
        escape_html(&config.authors)
    )
    .unwrap();
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
        for heading in headings {
            write!(
                out,
                "<a class=\"chapter-section-link chapter-section-l{}\" href=\"#{}\" data-section-link=\"{}\"><span class=\"chapter-section-no\">{}</span><span class=\"chapter-section-title\">{}</span></a>\n",
                heading.level,
                escape_attr(&heading.id),
                escape_attr(&heading.id),
                escape_html(&heading.number),
                escape_html(&heading.text)
            )
            .unwrap();
        }
    }
    out.push_str("</nav>\n");
    out
}

fn render_toc(headings: &[Heading]) -> String {
    let mut out = String::from("<nav class=\"toc\" aria-label=\"Contents\"><ol>\n");
    for heading in headings {
        write!(
            out,
            "<li class=\"toc-l{}\"><a href=\"#{}\"><span class=\"toc-no\">{}</span><span class=\"toc-title\">{}</span></a></li>\n",
            heading.level,
            escape_attr(&heading.id),
            escape_html(&heading.number),
            escape_html(&heading.text)
        )
        .unwrap();
    }
    out.push_str("</ol></nav>\n");
    out
}

fn render_endnotes(notes: &[RenderedEndnote]) -> String {
    if notes.is_empty() {
        return String::new();
    }
    let mut out = String::from("<section class=\"endnotes\" id=\"endnotes\">\n<h1>Notes</h1>\n");
    for (idx, note) in notes.iter().enumerate() {
        let seq = idx + 1;
        write!(
            out,
            "<p id=\"fn-end-{seq}\"><a class=\"footnote-backref\" href=\"#fnref-{seq}\">{}</a><span class=\"footnote-body\">{}</span></p>\n",
            escape_html(&note.number),
            note.body_html
        )
        .unwrap();
    }
    out.push_str("</section>\n");
    out
}

fn chapter_nav_script() -> &'static str {
    r##"<script>
(() => {
  const links = Array.from(document.querySelectorAll("[data-section-link]"));
  if (!links.length) return;
  const byId = new Map(links.map((link) => [link.getAttribute("data-section-link"), link]));
  const sections = Array.from(byId.keys()).map((id) => document.getElementById(id)).filter(Boolean);
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

fn equation_width_script() -> &'static str {
    r#"<script>
(() => {
  function boxWidth(el){
    if (!el) return 0;
    var rect = el.getBoundingClientRect();
    return Math.max(el.scrollWidth || 0, rect.width || 0);
  }
  function maxWidth(nodes){
    return nodes.reduce(function(max, node){ return Math.max(max, boxWidth(node)); }, 0);
  }
  function equationNeededWidth(eq){
    var style = window.getComputedStyle(eq);
    var columnGap = parseFloat(style.columnGap) || 0;
    var gap = 12;
    if (eq.classList.contains("equation-aligned")) {
      var left = maxWidth(Array.from(eq.querySelectorAll(".equation-align-left")));
      var right = maxWidth(Array.from(eq.querySelectorAll(".equation-align-right")));
      var full = maxWidth(Array.from(eq.querySelectorAll(".equation-align-full")));
      var eqno = maxWidth(Array.from(eq.querySelectorAll(".eqno")));
      var aligned = left + right + columnGap + (eqno ? eqno + columnGap : 0);
      return Math.max(full, aligned, eq.scrollWidth || 0);
    }
    var rows = Array.from(eq.querySelectorAll(".equation-line"));
    if (!rows.length) rows = Array.from(eq.querySelectorAll(".typst-frame"));
    var rowWidth = maxWidth(rows);
    var math = maxWidth(Array.from(eq.querySelectorAll(".equation-math, .equation-align-full, .typst-frame")));
    var eqno = maxWidth(Array.from(eq.querySelectorAll(".eqno")));
    return Math.max(rowWidth, math + (eqno ? eqno + gap : 0), eq.scrollWidth || 0);
  }
  function markOverwideEquations(){
    document.querySelectorAll(".equation").forEach(function(eq){
      eq.classList.remove("is-overwide");
      var available = eq.clientWidth;
      var needed = equationNeededWidth(eq);
      if (needed > available + 2) eq.classList.add("is-overwide");
    });
  }
  window.markOverwideEquations = markOverwideEquations;
  window.addEventListener("resize", markOverwideEquations);
  if (document.fonts && document.fonts.ready) document.fonts.ready.then(markOverwideEquations);
  window.setTimeout(markOverwideEquations, 0);
  window.setTimeout(markOverwideEquations, 80);
  window.setTimeout(markOverwideEquations, 300);
})();
</script>
"#
}

fn write_output(config: &Config, html: String) -> Result<(), String> {
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
        .map_err(|err| format!("could not write {}: {err}", config.output.display()))
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

fn normalize_ws(input: &str) -> String {
    input.split_whitespace().collect::<Vec<_>>().join(" ")
}

fn extract_href(attrs: &str) -> Option<&str> {
    re_href().captures(attrs)?.get(1).map(|m| m.as_str())
}

fn add_class_to_attrs(attrs: &str, class_name: &str) -> String {
    if re_class_attr().is_match(attrs) {
        re_class_attr()
            .replace(attrs, |captures: &Captures| {
                format!(
                    "class=\"{} {}\"",
                    captures.get(1).map_or("", |m| m.as_str()),
                    class_name
                )
            })
            .to_string()
    } else {
        format!("class=\"{}\" {}", class_name, attrs.trim())
    }
}

fn set_id_attr(attrs: &str, id: &str) -> String {
    if re_id_attr().is_match(attrs) {
        re_id_attr()
            .replace(attrs, format!("id=\"{}\"", escape_attr(id)))
            .to_string()
    } else {
        format!(" id=\"{}\"{}", escape_attr(id), attrs)
    }
}

fn bibtex_escape(input: &str) -> String {
    input.replace('&', "\\&")
}

fn escape_html(input: &str) -> String {
    input
        .replace('&', "&amp;")
        .replace('<', "&lt;")
        .replace('>', "&gt;")
        .replace('"', "&quot;")
}

fn escape_attr(input: &str) -> String {
    escape_html(input).replace('\'', "&#39;")
}

fn re_bib_prefix() -> &'static Regex {
    static RE: OnceLock<Regex> = OnceLock::new();
    RE.get_or_init(|| Regex::new(r#"(?s)^\s*<span class="prefix">.*?</span>\s*"#).unwrap())
}

fn re_endnote_backlink() -> &'static Regex {
    static RE: OnceLock<Regex> = OnceLock::new();
    RE.get_or_init(|| Regex::new(r#"(?s)\s*<a[^>]*role="doc-backlink"[^>]*>.*?</a>\s*"#).unwrap())
}

fn re_notes_meta() -> &'static Regex {
    static RE: OnceLock<Regex> = OnceLock::new();
    RE.get_or_init(|| {
        Regex::new(r#"(?s)\s*<div\b[^>]*\bclass="[^"]*\bnotes-meta\b[^"]*"[^>]*>.*?</div>\s*"#)
            .unwrap()
    })
}

fn re_hidden_bibliography() -> &'static Regex {
    static RE: OnceLock<Regex> = OnceLock::new();
    RE.get_or_init(|| {
        Regex::new(
            r#"(?s)\s*<div\s+hidden(?:="")?>\s*(?P<section><section role="doc-bibliography">.*?</section>)\s*</div>\s*"#,
        )
        .unwrap()
    })
}

fn re_empty_hidden_div() -> &'static Regex {
    static RE: OnceLock<Regex> = OnceLock::new();
    RE.get_or_init(|| Regex::new(r#"(?s)\s*<div\s+hidden(?:="")?>\s*</div>\s*"#).unwrap())
}

fn re_visible_bibliography() -> &'static Regex {
    static RE: OnceLock<Regex> = OnceLock::new();
    RE.get_or_init(|| {
        Regex::new(r#"(?s)<section\b[^>]*\bclass="[^"]*\bbibliography\b[^"]*"[^>]*>.*?</section>"#)
            .unwrap()
    })
}

fn re_bibliography_row() -> &'static Regex {
    static RE: OnceLock<Regex> = OnceLock::new();
    RE.get_or_init(|| Regex::new(r#"(?s)<tr(?P<attrs>[^>]*)>(?P<inner>.*?)</tr>"#).unwrap())
}

fn re_doc_biblioref_href_before_role() -> &'static Regex {
    static RE: OnceLock<Regex> = OnceLock::new();
    RE.get_or_init(|| {
        Regex::new(r##"<a\b[^>]*\bhref="#(?P<id>[^"]+)"[^>]*\brole="doc-biblioref""##).unwrap()
    })
}

fn re_doc_biblioref_href_after_role() -> &'static Regex {
    static RE: OnceLock<Regex> = OnceLock::new();
    RE.get_or_init(|| {
        Regex::new(r##"<a\b[^>]*\brole="doc-biblioref"[^>]*\bhref="#(?P<id>[^"]+)""##).unwrap()
    })
}

fn re_doc_biblioref_link() -> &'static Regex {
    static RE: OnceLock<Regex> = OnceLock::new();
    RE.get_or_init(|| Regex::new(r#"(?s)<a\b(?P<attrs>[^>]*)>(?P<inner>.*?)</a>"#).unwrap())
}

fn re_endnotes_section() -> &'static Regex {
    static RE: OnceLock<Regex> = OnceLock::new();
    RE.get_or_init(|| {
        Regex::new(r#"(?s)\s*<section role="doc-endnotes">.*?</section>\s*"#).unwrap()
    })
}

fn re_typst_figure_class() -> &'static Regex {
    static RE: OnceLock<Regex> = OnceLock::new();
    RE.get_or_init(|| {
        Regex::new(r#"(?P<prefix><figure\b[^>]*\bclass=")typst(?P<suffix>"[^>]*>)"#).unwrap()
    })
}

fn re_biblioref() -> &'static Regex {
    static RE: OnceLock<Regex> = OnceLock::new();
    RE.get_or_init(|| {
        Regex::new(r#"(?s)<a\s+([^>]*\brole="doc-biblioref"[^>]*)>(.*?)</a>"#).unwrap()
    })
}

fn re_noteref() -> &'static Regex {
    static RE: OnceLock<Regex> = OnceLock::new();
    RE.get_or_init(|| {
        Regex::new(r#"(?s)<a\s+([^>]*\brole="doc-noteref"[^>]*)><sup>(.*?)</sup></a>"#).unwrap()
    })
}

fn re_href() -> &'static Regex {
    static RE: OnceLock<Regex> = OnceLock::new();
    RE.get_or_init(|| Regex::new(r##"href="#([^"]+)""##).unwrap())
}

fn re_class_attr() -> &'static Regex {
    static RE: OnceLock<Regex> = OnceLock::new();
    RE.get_or_init(|| Regex::new(r#"class="([^"]*)""#).unwrap())
}

fn re_id_attr() -> &'static Regex {
    static RE: OnceLock<Regex> = OnceLock::new();
    RE.get_or_init(|| Regex::new(r#"id="([^"]*)""#).unwrap())
}

fn re_html_heading() -> &'static Regex {
    static RE: OnceLock<Regex> = OnceLock::new();
    RE.get_or_init(|| {
        Regex::new(r#"(?s)<h(?P<level>[1-6])(?P<attrs>[^>]*)>(?P<inner>.*?)</h[1-6]>"#).unwrap()
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn main_source_import_is_swapped_to_html_style() {
        let source = r#"#import "meta/gabri_notes.typ": *

#show: gabri_notes.with(lec_num: 1, title: "Intro")
"#;

        let rewritten = use_html_notes_style(source);

        assert!(rewritten.contains(r#"#import "meta/gabri_notes_html.typ": *"#));
        assert!(!rewritten.contains(r#"#import "meta/gabri_notes.typ": *"#));
    }

    #[test]
    fn relative_notes_import_is_swapped_to_html_style() {
        let source = r#"#import "../meta/gabri_notes.typ": *

#let body() = [Figure body]
"#;

        let rewritten = use_html_notes_style(source);

        assert!(rewritten.contains(r#"#import "../meta/gabri_notes_html.typ": *"#));
        assert!(!rewritten.contains(r#"#import "../meta/gabri_notes.typ": *"#));
    }

    #[test]
    fn extracts_metadata_and_headings_from_html_attributes() {
        let raw = r#"
<!doctype html>
<html><body>
<div class="notes-meta" hidden="" data-lecture-number="4" data-title="Phi-Regret"></div>
<h1 id="loc-4" class="notes-heading" data-level="1" data-number="4.1"><span class="secno">4.1</span> First section</h1>
<h2 class="notes-heading" data-level="2" data-number="4.1.1"><span class="secno">4.1.1</span> Real subsection</h2>
<h1 class="notes-heading notes-heading-unnumbered" data-level="1">Bibliography for this chapter</h1>
</body></html>
"#;

        let parts = HtmlParts::parse(raw);

        assert_eq!(parts.meta.lecture_number.as_deref(), Some("4"));
        assert_eq!(parts.meta.title.as_deref(), Some("Phi-Regret"));
        assert_eq!(parts.headings.len(), 2);
        assert_eq!(parts.headings[0].id, "loc-4");
        assert_eq!(parts.headings[0].number, "4.1");
        assert_eq!(parts.headings[0].text, "First section");
        assert_eq!(parts.headings[1].id, "4-1-1-real-subsection");
        assert_eq!(parts.headings[1].number, "4.1.1");
        assert_eq!(parts.headings[1].text, "Real subsection");
    }

    #[test]
    fn postprocess_keeps_visible_bibliography_and_removes_hidden_data() {
        let body = r##"
<p>Body</p>
<div class="notes-meta" hidden="" data-lecture-number="4" data-title="Phi-Regret"></div>
<section class="bibliography" id="bibliography"><table class="bibliography-table"><tr class="bibliography-row"><td class="bib-key">[<a class="citation" href="#loc-1" role="doc-biblioref">A</a>]</td><td class="bib-entry"><a class="citation" href="#loc-1" role="doc-biblioref">Entry</a></td></tr></table></section>
<div hidden=""><section role="doc-bibliography"><ol><li id="loc-1"><span class="prefix">[A]</span> Entry</li></ol></section></div>
<section role="doc-endnotes"><ol><li id="fn-1"><sup>1</sup> Note</li></ol></section>
"##
        .to_owned();

        let bibliography = vec![BibliographyItem {
            id: "loc-1".to_owned(),
            key_text: "[A]".to_owned(),
            entry_html: "Entry".to_owned(),
        }];

        let (body, _) = postprocess_body(body, &bibliography, &[], MathMode::Svg);

        assert!(!body.contains("hidden"));
        assert!(!body.contains("notes-meta"));
        assert!(body.contains("id=\"bibliography\""));
        assert!(body.contains("bibliography-table"));
        assert!(body.contains("<tr id=\"loc-1\" class=\"bibliography-row\">"));
        assert!(body.contains("<td class=\"bib-key\">[A]</td>"));
        assert!(body.contains("<td class=\"bib-entry\">Entry</td>"));
        assert!(!body.contains("citation-note"));
        assert!(!body.contains("role=\"doc-biblioref\""));
        assert!(!body.contains("role=\"doc-bibliography\""));
        assert!(!body.contains("doc-endnotes"));
    }

    #[test]
    fn repeated_citations_only_get_one_side_note() {
        let body = r##"
<p><a href="#loc-1" role="doc-biblioref">A</a></p>
<p><a href="#loc-1" role="doc-biblioref">A again</a></p>
<p><a href="#loc-2" role="doc-biblioref">B</a></p>
"##
        .to_owned();
        let bibliography = vec![
            BibliographyItem {
                id: "loc-1".to_owned(),
                key_text: "[A]".to_owned(),
                entry_html: "Entry A".to_owned(),
            },
            BibliographyItem {
                id: "loc-2".to_owned(),
                key_text: "[B]".to_owned(),
                entry_html: "Entry B".to_owned(),
            },
        ];

        let body = rewrite_citations(body, &bibliography);

        assert_eq!(body.matches("class=\"citation-note\"").count(), 2);
        assert!(body.contains(">A again</a></span>"));
        assert_eq!(body.matches("Entry A").count(), 1);
        assert_eq!(body.matches("Entry B").count(), 1);
    }

    #[test]
    fn heading_rewrite_adds_missing_ids() {
        let mut parts = HtmlParts {
            meta: DocumentMeta::default(),
            body_html:
                r#"<h2 class="notes-heading"><span class="secno">4.2.1</span> Real subsection</h2>"#
                    .to_owned(),
            headings: vec![Heading {
                level: 2,
                text: "Real subsection".to_owned(),
                id: "4-2-1-real-subsection".to_owned(),
                number: "4.2.1".to_owned(),
            }],
            bibliography: Vec::new(),
            endnotes: Vec::new(),
            rendered_endnotes: Vec::new(),
        };

        parts.rewrite_heading_ids();

        assert!(parts
            .body_html
            .contains(r#"<h2 id="4-2-1-real-subsection" class="notes-heading">"#));
    }
}
