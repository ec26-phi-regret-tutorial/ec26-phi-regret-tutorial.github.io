mod chapters;
mod math;
mod options;

use chapters::{ChapterNav, ExportConfig};
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

const PAGE_CSS: &str = include_str!("gabri-notes.css");

fn main() {
    if let Err(err) = run() {
        eprintln!("error: {err}");
        std::process::exit(1);
    }
}

fn run() -> Result<(), String> {
    let config = options::parse()?;
    let export_config = load_export_config(&config)?;
    let raw_html = compile_typst_html(&config)?;
    let mut document = HtmlParts::parse(&raw_html);
    let title = config
        .title
        .clone()
        .or_else(|| document.meta.title.clone())
        .unwrap_or_else(|| title_from_path(&config.input));

    document.rewrite_heading_ids();
    document.rewrite_statement_ids();
    let (body_html, rendered_endnotes) = postprocess_body(
        document.body_html,
        &document.bibliography,
        &document.endnotes,
        config.math_mode,
    )?;
    document.body_html = body_html;
    document.rendered_endnotes = rendered_endnotes;

    let html = render_document(&config, &title, &document, export_config.as_ref());
    write_output(&config, html)?;
    Ok(())
}

fn load_export_config(config: &Config) -> Result<Option<ExportConfig>, String> {
    let Some(path) = &config.export_config else {
        return Ok(None);
    };
    let path = if path.is_absolute() {
        path.clone()
    } else {
        config.root.join(path)
    };
    ExportConfig::load(&path).map(Some)
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
    raw_id: Option<String>,
    #[allow(dead_code)]
    text: String,
    title_html: String,
    id: String,
    number: String,
}

#[derive(Clone, Debug)]
struct StatementAnchor {
    raw_id: Option<String>,
    id: String,
}

struct CrossrefBlock {
    href: String,
    body_html: String,
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
        let body_html = rewrite_crossref_links_and_remove(body_html);
        let cleaned_dom = Html::parse_document(&format!("<html><body>{body_html}</body></html>"));
        let meta = extract_document_meta(&cleaned_dom);
        let headings = extract_headings(&cleaned_dom);
        let bibliography =
            filter_bibliography_to_visible_refs(extract_bibliography(&cleaned_dom), &body_html);
        let endnotes = extract_endnotes(&cleaned_dom);
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
        let mut link_targets = HashMap::new();
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
                if let Some(raw_id) = &heading.raw_id {
                    if raw_id != &heading.id {
                        link_targets.insert(raw_id.clone(), format!("#{}", heading.id));
                    }
                }
                format!(
                    "<h{level}{}>{inner}</h{level}>",
                    set_id_attr(attrs, &heading.id)
                )
            })
            .to_string();
        self.body_html = rewrite_href_targets(self.body_html.clone(), &link_targets);
    }

    fn rewrite_statement_ids(&mut self) {
        let anchors = collect_statement_anchors(&self.body_html);
        let mut statement_idx = 0usize;
        let mut href_targets = HashMap::new();
        self.body_html = re_html_section()
            .replace_all(&self.body_html, |captures: &Captures| {
                let whole = captures.get(0).map_or("", |m| m.as_str());
                let attrs = captures.name("attrs").map_or("", |m| m.as_str());
                if !attrs_has_class(attrs, "env") || !attrs_has_class(attrs, "statement") {
                    return whole.to_owned();
                }
                let Some(anchor) = anchors.get(statement_idx) else {
                    return whole.to_owned();
                };
                statement_idx += 1;
                if let Some(raw_id) = &anchor.raw_id {
                    if raw_id != &anchor.id {
                        href_targets.insert(raw_id.clone(), format!("#{}", anchor.id));
                    }
                }
                format!("<section{}>", set_id_attr(attrs, &anchor.id))
            })
            .to_string();
        self.body_html = rewrite_href_targets(self.body_html.clone(), &href_targets);
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

struct CitationLabelRewrite {
    from_label: String,
    to_label: String,
    to_key_text: String,
}

impl CitationLabelRewrite {
    fn new(from_key_text: &str, to_key_text: &str) -> Self {
        Self {
            from_label: strip_bib_key_brackets(from_key_text).to_owned(),
            to_label: strip_bib_key_brackets(to_key_text).to_owned(),
            to_key_text: to_key_text.to_owned(),
        }
    }

    fn apply(&self, label_html: &str) -> String {
        label_html.replacen(&self.from_label, &self.to_label, 1)
    }
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
        let title_html = heading_title_html(&element);
        let text = heading_text_from_html(&title_html);
        let id = raw_id
            .clone()
            .unwrap_or_else(|| slugify(&format!("{number}-{text}")));
        headings.push(Heading {
            level,
            raw_id,
            id,
            number,
            text,
            title_html,
        });
    }

    headings
}

fn heading_title_html(element: &ElementRef) -> String {
    re_heading_secno_span()
        .replace(&element.inner_html(), "")
        .trim()
        .to_owned()
}

fn heading_text_from_html(html: &str) -> String {
    let fragment = Html::parse_fragment(&html);
    normalize_ws(&fragment.root_element().text().collect::<Vec<_>>().join(" "))
}

fn non_empty_attr(element: &ElementRef, name: &str) -> Option<String> {
    element
        .value()
        .attr(name)
        .map(str::trim)
        .filter(|value| !value.is_empty())
        .map(str::to_owned)
}

fn current_chapter(config: &Config, chapters: &ExportConfig) -> Option<usize> {
    chapters.current_index_for_input(&config.input)
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

fn filter_bibliography_to_visible_refs(
    bibliography: Vec<BibliographyItem>,
    body_html: &str,
) -> Vec<BibliographyItem> {
    let referenced = visible_bibliography_ref_ids(body_html);
    bibliography
        .into_iter()
        .filter(|item| referenced.contains(item.id.as_str()))
        .collect()
}

fn visible_bibliography_ref_ids(body_html: &str) -> HashSet<String> {
    let mut body = body_html.to_owned();
    body = re_notes_meta().replace_all(&body, "").to_string();
    body = re_hidden_bibliography().replace_all(&body, "").to_string();
    body = re_endnotes_section().replace_all(&body, "").to_string();
    let (body, _visible_bibliographies) = protect_visible_bibliographies(body);
    re_doc_biblioref_link()
        .captures_iter(&body)
        .filter_map(|captures| captures.name("attrs"))
        .filter_map(|attrs| extract_href(attrs.as_str()))
        .map(str::to_owned)
        .collect()
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

fn rewrite_crossref_links_and_remove(body: String) -> String {
    let (body, targets) = remove_crossrefs_and_collect_targets(&body);
    rewrite_href_targets(body, &targets)
}

fn remove_crossrefs_and_collect_targets(body: &str) -> (String, HashMap<String, String>) {
    let mut cleaned = String::new();
    let mut targets = HashMap::new();
    let mut stack: Vec<CrossrefBlock> = Vec::new();
    let mut last = 0usize;

    for captures in re_crossrefs_marker().captures_iter(body) {
        let Some(marker) = captures.get(0) else {
            continue;
        };
        let segment = &body[last..marker.start()];
        if let Some(block) = stack.last_mut() {
            block.body_html.push_str(segment);
        } else {
            cleaned.push_str(segment);
        }

        let kind = captures.name("kind").map_or("", |m| m.as_str());
        if kind == "start" {
            let attrs = captures.name("attrs").map_or("", |m| m.as_str());
            stack.push(CrossrefBlock {
                href: data_attr(attrs, "href").unwrap_or_default().to_owned(),
                body_html: String::new(),
            });
        } else if let Some(block) = stack.pop() {
            targets.extend(collect_crossref_targets(&block.body_html, &block.href));
        } else {
            cleaned.push_str(marker.as_str());
        }
        last = marker.end();
    }

    let tail = &body[last..];
    if let Some(block) = stack.last_mut() {
        block.body_html.push_str(tail);
    } else {
        cleaned.push_str(tail);
    }
    while let Some(block) = stack.pop() {
        targets.extend(collect_crossref_targets(&block.body_html, &block.href));
    }

    (cleaned, targets)
}

fn collect_crossref_targets(body: &str, href: &str) -> HashMap<String, String> {
    let id_selector = Selector::parse("[id]").unwrap();
    let mut targets = HashMap::new();
    let crossref_dom = Html::parse_fragment(body);
    for element in crossref_dom.select(&id_selector) {
        let Some(old_id) = element.value().attr("id") else {
            continue;
        };
        let Some(anchor) = stable_anchor_for(&element) else {
            continue;
        };
        targets.insert(old_id.to_owned(), format!("{href}#{anchor}"));
    }

    targets
}

fn collect_statement_anchors(body: &str) -> Vec<StatementAnchor> {
    let dom = Html::parse_fragment(body);
    let selector = Selector::parse("section.env.statement").unwrap();
    let mut anchors = Vec::new();

    for element in dom.select(&selector) {
        let Some(anchor) = stable_statement_id(&element) else {
            continue;
        };
        anchors.push(StatementAnchor {
            raw_id: element.value().attr("id").map(str::to_owned),
            id: anchor,
        });
    }

    anchors
}

fn stable_anchor_for(element: &ElementRef<'_>) -> Option<String> {
    if has_class(element, "notes-heading") {
        return stable_heading_id(element);
    }
    if has_class(element, "statement") {
        return stable_statement_id(element);
    }
    None
}

fn stable_heading_id(element: &ElementRef<'_>) -> Option<String> {
    let secno_selector = Selector::parse(".secno").unwrap();
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
        })?;
    let text = heading_text_from_html(&heading_title_html(element));
    Some(slugify(&format!("{number}-{text}")))
}

fn stable_statement_id(element: &ElementRef<'_>) -> Option<String> {
    let kind_selector = Selector::parse(".env-kind").unwrap();
    let number_selector = Selector::parse(".env-number").unwrap();
    let kind = element
        .select(&kind_selector)
        .next()
        .map(|kind| element_text(&kind))
        .filter(|kind| !kind.is_empty())?;
    let number = element
        .select(&number_selector)
        .next()
        .map(|number| element_text(&number))
        .filter(|number| !number.is_empty())?;
    Some(slugify(&format!("{kind} {number}")))
}

fn has_class(element: &ElementRef<'_>, class_name: &str) -> bool {
    element
        .value()
        .attr("class")
        .unwrap_or_default()
        .split_whitespace()
        .any(|class| class == class_name)
}

fn attrs_has_class(attrs: &str, class_name: &str) -> bool {
    re_class_attr()
        .captures(attrs)
        .and_then(|captures| captures.get(1))
        .is_some_and(|classes| {
            classes
                .as_str()
                .split_whitespace()
                .any(|class| class == class_name)
        })
}

fn element_text(element: &ElementRef<'_>) -> String {
    normalize_ws(&element.text().collect::<Vec<_>>().join(" "))
}

fn rewrite_href_targets(body: String, targets: &HashMap<String, String>) -> String {
    if targets.is_empty() {
        return body;
    }
    re_href()
        .replace_all(&body, |captures: &Captures| {
            let whole = captures.get(0).map_or("", |m| m.as_str());
            let id = captures.get(1).map_or("", |m| m.as_str());
            targets
                .get(id)
                .map(|target| format!("href=\"{}\"", escape_attr(target)))
                .unwrap_or_else(|| whole.to_owned())
        })
        .to_string()
}

fn postprocess_body(
    body_html: String,
    bibliography: &[BibliographyItem],
    endnotes: &[Endnote],
    math_mode: MathMode,
) -> Result<(String, Vec<RenderedEndnote>), String> {
    let mut body = body_html;
    body = re_notes_meta().replace_all(&body, "").to_string();
    body = re_hidden_bibliography().replace_all(&body, "").to_string();
    body = re_endnotes_section().replace_all(&body, "").to_string();
    body = re_empty_hidden_div().replace_all(&body, "").to_string();
    body = normalize_typst_classes(body);
    body = math::postprocess_html_math(body, math_mode);
    let (body_without_bibliography, bibliography_blocks) = protect_visible_bibliographies(body);
    let citation_bibliography =
        citation_bibliography_from_visible_blocks(&bibliography_blocks, bibliography)?;
    let (bibliography_blocks, citation_bibliography, citation_label_rewrites) =
        disambiguate_bibliography_labels(bibliography_blocks, citation_bibliography);
    let body = rewrite_citations(
        body_without_bibliography,
        &citation_bibliography,
        &citation_label_rewrites,
    );
    let body = restore_visible_bibliographies(body, bibliography_blocks);
    Ok(rewrite_footnotes(body, endnotes))
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

fn rewrite_citations(
    body: String,
    bibliography: &[BibliographyItem],
    label_rewrites: &HashMap<String, CitationLabelRewrite>,
) -> String {
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
            let label = href
                .and_then(|href| label_rewrites.get(href))
                .map(|rewrite| rewrite.apply(label))
                .unwrap_or_else(|| label.to_owned());
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

fn disambiguate_bibliography_labels(
    blocks: Vec<String>,
    mut bibliography: Vec<BibliographyItem>,
) -> (
    Vec<String>,
    Vec<BibliographyItem>,
    HashMap<String, CitationLabelRewrite>,
) {
    let mut by_key = HashMap::<String, Vec<usize>>::new();
    for (index, item) in bibliography.iter().enumerate() {
        if !item.key_text.is_empty() {
            by_key.entry(item.key_text.clone()).or_default().push(index);
        }
    }

    let mut rewrites = HashMap::new();
    for indexes in by_key.values().filter(|indexes| indexes.len() > 1) {
        for (position, index) in indexes.iter().copied().enumerate() {
            let original = bibliography[index].key_text.clone();
            let suffixed = append_suffix_to_bib_key(&original, &alphabetic_suffix(position));
            rewrites.insert(
                bibliography[index].id.clone(),
                CitationLabelRewrite::new(&original, &suffixed),
            );
            bibliography[index].key_text = suffixed;
        }
    }

    if rewrites.is_empty() {
        return (blocks, bibliography, rewrites);
    }

    let blocks = blocks
        .into_iter()
        .map(|block| rewrite_bibliography_key_cells(block, &rewrites))
        .collect();
    (blocks, bibliography, rewrites)
}

fn rewrite_bibliography_key_cells(
    block: String,
    rewrites: &HashMap<String, CitationLabelRewrite>,
) -> String {
    re_bibliography_key_cell()
        .replace_all(&block, |captures: &Captures| {
            let Some(id) = captures.name("id").map(|m| m.as_str()) else {
                return captures.get(0).unwrap().as_str().to_owned();
            };
            let Some(rewrite) = rewrites.get(id) else {
                return captures.get(0).unwrap().as_str().to_owned();
            };
            format!(
                "{}{}{}",
                captures.name("prefix").map_or("", |m| m.as_str()),
                escape_html(&rewrite.to_key_text),
                captures.name("suffix").map_or("", |m| m.as_str())
            )
        })
        .to_string()
}

fn append_suffix_to_bib_key(key_text: &str, suffix: &str) -> String {
    if let Some(inner) = key_text
        .strip_prefix('[')
        .and_then(|text| text.strip_suffix(']'))
    {
        format!("[{inner}{suffix}]")
    } else {
        format!("{key_text}{suffix}")
    }
}

fn strip_bib_key_brackets(key_text: &str) -> &str {
    key_text
        .strip_prefix('[')
        .and_then(|text| text.strip_suffix(']'))
        .unwrap_or(key_text)
}

fn alphabetic_suffix(mut index: usize) -> String {
    let mut chars = Vec::new();
    loop {
        chars.push((b'a' + (index % 26) as u8) as char);
        index /= 26;
        if index == 0 {
            break;
        }
        index -= 1;
    }
    chars.into_iter().rev().collect()
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

fn citation_bibliography_from_visible_blocks(
    blocks: &[String],
    fallback: &[BibliographyItem],
) -> Result<Vec<BibliographyItem>, String> {
    let items = extract_visible_bibliography_items(blocks);
    let seen = items
        .iter()
        .map(|item| item.id.as_str())
        .collect::<HashSet<_>>();
    let missing = fallback
        .iter()
        .filter(|item| !seen.contains(item.id.as_str()))
        .map(|item| format!("{} {}", item.key_text, item.id).trim().to_owned())
        .collect::<Vec<_>>();
    if !missing.is_empty() {
        return Err(format!(
            "visible lec_bibliography is missing row(s) for cited item(s): {}",
            missing.join(", ")
        ));
    }
    Ok(items)
}

fn extract_visible_bibliography_items(blocks: &[String]) -> Vec<BibliographyItem> {
    let row_selector = Selector::parse("tr.bibliography-row").unwrap();
    let key_selector = Selector::parse(".bib-key").unwrap();
    let entry_selector = Selector::parse(".bib-entry").unwrap();
    let mut items = Vec::new();

    for block in blocks {
        let dom = Html::parse_fragment(block);
        for row in dom.select(&row_selector) {
            let Some(id) = row
                .value()
                .attr("id")
                .map(str::trim)
                .filter(|id| !id.is_empty())
            else {
                continue;
            };
            let key_text = row
                .select(&key_selector)
                .next()
                .map(|key| normalize_ws(&key.text().collect::<Vec<_>>().join(" ")))
                .unwrap_or_default();
            let Some(entry) = row.select(&entry_selector).next() else {
                continue;
            };
            items.push(BibliographyItem {
                id: id.to_owned(),
                key_text,
                entry_html: entry.inner_html().trim().to_owned(),
            });
        }
    }

    items
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

fn render_document(
    config: &Config,
    title: &str,
    document: &HtmlParts,
    export_config: Option<&ExportConfig>,
) -> String {
    let current = export_config.and_then(|export_config| current_chapter(config, export_config));
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

    if let (Some(export_config), Some(current)) = (export_config, current) {
        html.push_str(&render_chapter_rail(
            export_config,
            current,
            &document.headings,
            config,
        ));
    } else {
        html.push_str(&render_masthead(config));
    }

    html.push_str("<main class=\"page-shell\">\n<article class=\"lecture-content\"");
    if let Some(number) = &document.meta.lecture_number {
        write!(html, " data-lecture-number=\"{}\"", escape_attr(number)).unwrap();
    }
    html.push_str(">\n");
    if let (Some(export_config), Some(current)) = (export_config, current) {
        html.push_str(&render_chapter_citation_sidenote(
            export_config,
            &export_config.chapters[current],
            title,
            &config.site_title,
            config.pdf_href.as_deref(),
        ));
    }
    if let (Some(export_config), Some(current)) = (export_config, current) {
        write!(
            html,
            "<p class=\"chapter-kicker\">Chapter {}</p>\n",
            export_config.chapters[current].number
        )
        .unwrap();
    }
    write!(
        html,
        "<h1 class=\"lecture-title\">{}</h1>\n",
        escape_html(title)
    )
    .unwrap();
    if !document.headings.is_empty() {
        html.push_str(&render_toc(&document.headings, config.math_mode));
    }
    html.push_str(&document.body_html);
    html.push_str(&render_endnotes(&document.rendered_endnotes));
    html.push_str("</article>\n</main>\n");
    if current.is_some() {
        html.push_str(chapter_nav_script());
        html.push_str(chapter_citation_script());
    }
    html.push_str(equation_width_script());
    html.push_str(settled_hash_scroll_script());
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
    export_config: &ExportConfig,
    chapter: &ChapterNav,
    title: &str,
    site_title: &str,
    pdf_href: Option<&str>,
) -> String {
    let citation = &export_config.how_to_cite;
    let key = format!("{}-chapter-{}", citation.key_prefix, chapter.number);
    let href = chapter.href().expect("chapter href was validated");
    let citation_title = citation.citation_title(chapter.number, title);
    let citation_note = citation.citation_note(chapter.number, title);
    let bibtex = format!(
        "@misc{{{key},\n  author = {{{}}},\n  title = {{{}}},\n  booktitle = {{{}}},\n  note = {{{}}},\n  year = {{{}}},\n  url = {{{}}}\n}}",
        citation.authors,
        bibtex_escape(&citation_title),
        bibtex_escape(site_title),
        bibtex_escape(&citation_note),
        citation.year,
        href
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

fn render_chapter_rail(
    export_config: &ExportConfig,
    current: usize,
    headings: &[Heading],
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
        "<div class=\"course-authors\">{}</div></div>\n",
        escape_html(&config.authors)
    )
    .unwrap();
    out.push_str("<div class=\"chapter-rail-heading\">Chapters</div>\n");
    for (idx, chapter) in export_config.chapters.iter().enumerate() {
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
            escape_attr(&chapter.href().expect("chapter href was validated")),
            aria,
            chapter.number,
            escape_html(&chapter.short_title)
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
                render_heading_title(heading, config.math_mode)
            )
            .unwrap();
        }
    }
    out.push_str("</nav>\n");
    out
}

fn render_toc(headings: &[Heading], math_mode: MathMode) -> String {
    let mut out = String::from("<nav class=\"toc\" aria-label=\"Contents\"><ol>\n");
    for heading in headings {
        write!(
            out,
            "<li class=\"toc-l{}\"><a href=\"#{}\"><span class=\"toc-no\">{}</span><span class=\"toc-title\">{}</span></a></li>\n",
            heading.level,
            escape_attr(&heading.id),
            escape_html(&heading.number),
            render_heading_title(heading, math_mode)
        )
        .unwrap();
    }
    out.push_str("</ol></nav>\n");
    out
}

fn render_heading_title(heading: &Heading, math_mode: MathMode) -> String {
    math::postprocess_html_math(heading.title_html.clone(), math_mode)
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

fn chapter_citation_script() -> &'static str {
    r#"<script>
(() => {
  const reduceMotion = window.matchMedia("(prefers-reduced-motion: reduce)").matches;
  for (const details of document.querySelectorAll(".chapter-citation-details")) {
    const summary = details.querySelector("summary");
    const panel = details.querySelector("pre");
    if (!summary || !panel) continue;

    const finish = () => {
      details.classList.remove("is-animating");
      panel.style.height = "";
      panel.style.opacity = "";
      panel.style.marginTop = "";
      panel.style.paddingTop = "";
      panel.style.paddingBottom = "";
      panel.style.overflow = "";
    };

    const afterHeight = (callback) => {
      const done = (event) => {
        if (event.propertyName !== "height") return;
        panel.removeEventListener("transitionend", done);
        window.clearTimeout(fallback);
        callback();
      };
      const fallback = window.setTimeout(() => {
        panel.removeEventListener("transitionend", done);
        callback();
      }, 360);
      panel.addEventListener("transitionend", done);
    };

    summary.addEventListener("click", (event) => {
      event.preventDefault();
      if (reduceMotion) {
        details.open = !details.open;
        return;
      }

      panel.getAnimations().forEach((animation) => animation.cancel());
      details.classList.add("is-animating");

      if (!details.open) {
        details.open = true;
        const endHeight = panel.scrollHeight;
        panel.style.height = "0px";
        panel.style.opacity = "0";
        panel.style.marginTop = "0px";
        panel.style.paddingTop = "0px";
        panel.style.paddingBottom = "0px";
        panel.style.overflow = "hidden";
        panel.offsetHeight;
        panel.style.height = `${endHeight}px`;
        panel.style.opacity = "1";
        panel.style.marginTop = "";
        panel.style.paddingTop = "";
        panel.style.paddingBottom = "";
        afterHeight(finish);
      } else {
        panel.style.height = `${panel.scrollHeight}px`;
        panel.style.opacity = "1";
        panel.style.overflow = "hidden";
        panel.offsetHeight;
        panel.style.height = "0px";
        panel.style.opacity = "0";
        panel.style.marginTop = "0px";
        panel.style.paddingTop = "0px";
        panel.style.paddingBottom = "0px";
        afterHeight(() => {
          details.open = false;
          finish();
        });
      }
    });
  }
})();
</script>
"#
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

fn settled_hash_scroll_script() -> &'static str {
    r#"<script>
(() => {
  let version = 0;

  function hashTarget(){
    const raw = window.location.hash ? window.location.hash.slice(1) : "";
    if (!raw) return null;
    try {
      return document.getElementById(decodeURIComponent(raw));
    } catch (_) {
      return document.getElementById(raw);
    }
  }

  function anchorOffset(){
    const value = window.getComputedStyle(document.documentElement).getPropertyValue("--anchor-offset");
    return parseFloat(value) || 0;
  }

  function targetDistance(target){
    return target.getBoundingClientRect().top - anchorOffset();
  }

  function scrollToTarget(target){
    const top = target.getBoundingClientRect().top + window.pageYOffset - anchorOffset();
    const y = Math.max(0, top);
    try {
      window.scrollTo({ top: y, left: 0, behavior: "smooth" });
    } catch (_) {
      window.scrollTo(0, y);
    }
  }

  function afterLoad(){
    if (document.readyState === "complete") return Promise.resolve();
    return new Promise(function(resolve){
      window.addEventListener("load", resolve, { once: true });
    });
  }

  function afterFonts(){
    if (document.fonts && document.fonts.ready) {
      return document.fonts.ready.catch(function(){});
    }
    return Promise.resolve();
  }

  function afterStableLayout(callback){
    let lastHeight = -1;
    let stableFrames = 0;
    let frames = 0;
    function tick(){
      const height = document.documentElement.scrollHeight;
      if (height === lastHeight) stableFrames += 1;
      else {
        stableFrames = 0;
        lastHeight = height;
      }
      frames += 1;
      if (stableFrames >= 2 || frames >= 24) callback();
      else window.requestAnimationFrame(tick);
    }
    window.requestAnimationFrame(tick);
  }

  function scheduleHashScroll(){
    const target = hashTarget();
    if (!target) return;
    const current = ++version;
    Promise.all([afterLoad(), afterFonts()]).then(function(){
      afterStableLayout(function(){
        if (current !== version) return;
        const target = hashTarget();
        if (!target) return;
        scrollToTarget(target);
        window.setTimeout(function(){
          if (current === version) {
            const target = hashTarget();
            if (target && Math.abs(targetDistance(target)) > 1) scrollToTarget(target);
          }
        }, 160);
      });
    });
  }

  scheduleHashScroll();
  window.addEventListener("hashchange", scheduleHashScroll);
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

fn data_attr<'a>(attrs: &'a str, name: &str) -> Option<&'a str> {
    let needle = format!("data-{name}=\"");
    let start = attrs.find(&needle)? + needle.len();
    let rest = &attrs[start..];
    let end = rest.find('"')?;
    Some(&rest[..end])
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

fn re_crossrefs_marker() -> &'static Regex {
    static RE: OnceLock<Regex> = OnceLock::new();
    RE.get_or_init(|| {
        Regex::new(
            r#"(?s)<span\b(?P<attrs>[^>]*\bclass="[^"]*\bcrossrefs-(?P<kind>start|end)\b[^"]*"[^>]*)>\s*</span>"#,
        )
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

fn re_bibliography_key_cell() -> &'static Regex {
    static RE: OnceLock<Regex> = OnceLock::new();
    RE.get_or_init(|| {
        Regex::new(
            r#"(?s)(?P<prefix><tr\b[^>]*\bid="(?P<id>[^"]+)"[^>]*>.*?<td\b[^>]*\bclass="[^"]*\bbib-key\b[^"]*"[^>]*>).*?(?P<suffix></td>)"#,
        )
        .unwrap()
    })
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

fn re_heading_secno_span() -> &'static Regex {
    static RE: OnceLock<Regex> = OnceLock::new();
    RE.get_or_init(|| {
        Regex::new(r#"(?s)^\s*<span\b[^>]*\bclass="[^"]*\bsecno\b[^"]*"[^>]*>.*?</span>\s*"#)
            .unwrap()
    })
}

fn re_html_section() -> &'static Regex {
    static RE: OnceLock<Regex> = OnceLock::new();
    RE.get_or_init(|| Regex::new(r#"<section(?P<attrs>[^>]*)>"#).unwrap())
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
    fn heading_math_is_preserved_in_rendered_toc() {
        let raw = r#"
<!doctype html>
<html><body>
<h1 id="sec-gordon" class="notes-heading" data-level="1" data-number="1.3"><span class="secno">1.3</span> A framework for minimizing <span class="math" data-math-display="inline" data-typst-math="[Φ]" role="math"><svg></svg></span>-regret</h1>
</body></html>
"#;

        let parts = HtmlParts::parse(raw);
        let toc = render_toc(&parts.headings, MathMode::Katex);

        assert!(toc.contains("A framework for minimizing"));
        assert!(toc.contains("math-katex-source"));
        assert!(toc.contains(r"\(\Phi\)"));
        assert!(toc.contains("-regret"));
        assert!(!toc.contains("<svg>"));
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

        let (body, _) = postprocess_body(body, &bibliography, &[], MathMode::Svg).unwrap();

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

        let body = rewrite_citations(body, &bibliography, &HashMap::new());

        assert_eq!(body.matches("class=\"citation-note\"").count(), 2);
        assert!(body.contains("<span class=\"citation-note-key\">[A]</span> Entry A"));
        assert!(body.contains(">A again</a></span>"));
        assert_eq!(body.matches("Entry A").count(), 1);
        assert_eq!(body.matches("Entry B").count(), 1);
    }

    #[test]
    fn citation_notes_prefer_visible_bibliography_entries() {
        let body = r##"
<p><a href="#loc-1" role="doc-biblioref">A</a></p>
<section class="bibliography"><table class="bibliography-table"><tr class="bibliography-row"><td class="bib-key">[<a href="#loc-1" role="doc-biblioref">A</a>]</td><td class="bib-entry">Visible Entry</td></tr></table></section>
"##
        .to_owned();
        let bibliography = vec![BibliographyItem {
            id: "loc-1".to_owned(),
            key_text: "[A]".to_owned(),
            entry_html: "Hidden Entry".to_owned(),
        }];

        let (body, _) = postprocess_body(body, &bibliography, &[], MathMode::Svg).unwrap();

        assert!(body.contains("<span class=\"citation-note-key\">[A]</span> Visible Entry"));
        assert!(!body.contains("Hidden Entry"));
    }

    #[test]
    fn duplicate_bibliography_labels_get_letter_suffixes() {
        let body = r##"
<p><a href="#loc-1" role="doc-biblioref">Zha+25</a></p>
<p><a href="#loc-2" role="doc-biblioref">Zhang et al. [Zha+25]</a></p>
<section class="bibliography" id="bibliography"><table class="bibliography-table">
<tr class="bibliography-row"><td class="bib-key">[<a href="#loc-1" role="doc-biblioref">Zha+25</a>]</td><td class="bib-entry">First Entry</td></tr>
<tr class="bibliography-row"><td class="bib-key">[<a href="#loc-2" role="doc-biblioref">Zha+25</a>]</td><td class="bib-entry">Second Entry</td></tr>
</table></section>
"##
        .to_owned();
        let bibliography = vec![
            BibliographyItem {
                id: "loc-1".to_owned(),
                key_text: "[Zha+25]".to_owned(),
                entry_html: "Hidden First".to_owned(),
            },
            BibliographyItem {
                id: "loc-2".to_owned(),
                key_text: "[Zha+25]".to_owned(),
                entry_html: "Hidden Second".to_owned(),
            },
        ];

        let (body, _) = postprocess_body(body, &bibliography, &[], MathMode::Svg).unwrap();

        assert!(body.contains(">Zha+25a</a>"));
        assert!(body.contains(">Zhang et al. [Zha+25b]</a>"));
        assert!(body.contains("<td class=\"bib-key\">[Zha+25a]</td>"));
        assert!(body.contains("<td class=\"bib-key\">[Zha+25b]</td>"));
        assert!(body.contains("<span class=\"citation-note-key\">[Zha+25a]</span> First Entry"));
        assert!(body.contains("<span class=\"citation-note-key\">[Zha+25b]</span> Second Entry"));
    }

    #[test]
    fn missing_visible_bibliography_row_is_an_error() {
        let body = r##"
<p><a href="#loc-1" role="doc-biblioref">A</a></p>
<section class="bibliography"><table class="bibliography-table"></table></section>
"##
        .to_owned();
        let bibliography = vec![BibliographyItem {
            id: "loc-1".to_owned(),
            key_text: "[A]".to_owned(),
            entry_html: "Hidden Entry".to_owned(),
        }];

        let err = match postprocess_body(body, &bibliography, &[], MathMode::Svg) {
            Ok(_) => panic!("postprocess_body unexpectedly succeeded"),
            Err(err) => err,
        };

        assert!(
            err.contains("visible lec_bibliography is missing row"),
            "{err}"
        );
        assert!(err.contains("[A] loc-1"), "{err}");
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
                raw_id: None,
                text: "Real subsection".to_owned(),
                title_html: "Real subsection".to_owned(),
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
