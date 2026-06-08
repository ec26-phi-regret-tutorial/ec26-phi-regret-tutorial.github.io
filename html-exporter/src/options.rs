use crate::math::MathMode;
use std::path::{Path, PathBuf};

const DEFAULT_SITE_TITLE: &str = "Learning and Computation of Phi-Equilibria";
const DEFAULT_AUTHORS: &str = "Ioannis Anagnostides, Gabriele Farina, and Brian Hu Zhang";
const DEFAULT_INDEX_HREF: &str = "index.html";

#[derive(Debug)]
pub(crate) struct Config {
    pub(crate) input: PathBuf,
    pub(crate) output: PathBuf,
    pub(crate) root: PathBuf,
    pub(crate) title: Option<String>,
    pub(crate) site_title: String,
    pub(crate) authors: String,
    pub(crate) index_href: Option<String>,
    pub(crate) pdf_href: Option<String>,
    pub(crate) math_mode: MathMode,
}

/// Export the EC'26 Typst notes through Typst HTML plus postprocessing.
#[argopt::cmd]
#[opt(author, version, about, long_about = None)]
pub(crate) fn parse(
    /// Project root for includes, packages, fonts, and bibliography.
    #[opt(long)]
    root: Option<PathBuf>,
    /// Page title override.
    #[opt(long)]
    title: Option<String>,
    /// Header site title.
    #[opt(long = "site-title")]
    site_title: Option<String>,
    /// Header author line.
    #[opt(long)]
    authors: Option<String>,
    /// Header index link.
    #[opt(long)]
    index: Option<String>,
    /// Hide the index link.
    #[opt(long = "no-index")]
    no_index: bool,
    /// Header PDF link.
    #[opt(long)]
    pdf: Option<String>,
    /// Math rendering backend: svg or katex. Defaults to katex.
    #[opt(long)]
    math: Option<String>,
    /// Input Typst file.
    input: PathBuf,
    /// Output HTML file. Defaults to the input path with .html extension.
    output: Option<PathBuf>,
) -> Result<Config, String> {
    let math_mode = if let Some(math) = math {
        MathMode::parse(&math)?
    } else {
        MathMode::Katex
    };

    let output = output.unwrap_or_else(|| input.with_extension("html"));
    let root = root.unwrap_or_else(|| default_root_for_input(&input));
    let index_href = if no_index {
        None
    } else {
        Some(index.unwrap_or_else(|| DEFAULT_INDEX_HREF.to_owned()))
    };

    Ok(Config {
        input,
        output,
        root,
        title,
        site_title: site_title.unwrap_or_else(|| DEFAULT_SITE_TITLE.to_owned()),
        authors: authors.unwrap_or_else(|| DEFAULT_AUTHORS.to_owned()),
        index_href,
        pdf_href: pdf,
        math_mode,
    })
}

fn default_root_for_input(input: &Path) -> PathBuf {
    input
        .parent()
        .filter(|path| !path.as_os_str().is_empty())
        .unwrap_or_else(|| Path::new("."))
        .to_path_buf()
}
