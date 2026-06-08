use serde::Deserialize;
use std::collections::HashSet;
use std::fs;
use std::path::Path;

#[derive(Clone, Debug, Deserialize)]
pub(crate) struct ExportConfig {
    pub(crate) how_to_cite: CitationConfig,
    pub(crate) chapters: Vec<ChapterNav>,
}

#[derive(Clone, Debug, Deserialize)]
pub(crate) struct CitationConfig {
    pub(crate) authors: String,
    pub(crate) key_prefix: String,
    pub(crate) title_template: String,
    pub(crate) note_template: String,
    pub(crate) year: u16,
}

#[derive(Clone, Debug, Deserialize)]
pub(crate) struct ChapterNav {
    pub(crate) number: u8,
    pub(crate) source: String,
    pub(crate) short_title: String,
}

impl ExportConfig {
    pub(crate) fn load(path: &Path) -> Result<Self, String> {
        let yaml = fs::read_to_string(path)
            .map_err(|err| format!("could not read export config {}: {err}", path.display()))?;
        let book: Self = serde_yaml::from_str(&yaml)
            .map_err(|err| format!("could not parse export config {}: {err}", path.display()))?;
        book.validate(path)?;
        Ok(book)
    }

    pub(crate) fn current_index_for_input(&self, input: &Path) -> Option<usize> {
        let input_file = input.file_name()?.to_string_lossy();
        self.chapters.iter().position(|chapter| {
            Path::new(&chapter.source)
                .file_name()
                .is_some_and(|source_file| source_file == input_file.as_ref())
        })
    }

    fn validate(&self, path: &Path) -> Result<(), String> {
        self.how_to_cite.validate(path)?;

        if self.chapters.is_empty() {
            return Err(format!("export config {} has no chapters", path.display()));
        }

        let mut numbers = HashSet::new();
        let mut sources = HashSet::new();
        let mut hrefs = HashSet::new();
        for chapter in &self.chapters {
            if chapter.number == 0 {
                return Err(format!(
                    "export config {} contains chapter number 0",
                    path.display()
                ));
            }
            if chapter.source.trim().is_empty() {
                return Err(format!(
                    "export config {} contains a chapter with an empty source",
                    path.display()
                ));
            }
            if chapter.short_title.trim().is_empty() {
                return Err(format!(
                    "export config {} contains a chapter with an empty short_title",
                    path.display()
                ));
            }
            if !numbers.insert(chapter.number) {
                return Err(format!(
                    "export config {} repeats chapter number {}",
                    path.display(),
                    chapter.number
                ));
            }
            if !sources.insert(chapter.source.as_str()) {
                return Err(format!(
                    "export config {} repeats source {}",
                    path.display(),
                    chapter.source
                ));
            }
            let href = chapter.href()?;
            if !hrefs.insert(href.clone()) {
                return Err(format!(
                    "export config {} repeats derived href {}",
                    path.display(),
                    href
                ));
            }
        }

        Ok(())
    }
}

impl CitationConfig {
    pub(crate) fn citation_title(&self, number: u8, title: &str) -> String {
        apply_chapter_template(&self.title_template, number, title)
    }

    pub(crate) fn citation_note(&self, number: u8, title: &str) -> String {
        apply_chapter_template(&self.note_template, number, title)
    }

    fn validate(&self, path: &Path) -> Result<(), String> {
        if self.authors.trim().is_empty() {
            return Err(format!(
                "export config {} has empty authors",
                path.display()
            ));
        }
        if self.key_prefix.trim().is_empty() {
            return Err(format!(
                "export config {} has empty key_prefix",
                path.display()
            ));
        }
        if self.title_template.trim().is_empty() {
            return Err(format!(
                "export config {} has empty title_template",
                path.display()
            ));
        }
        if self.note_template.trim().is_empty() {
            return Err(format!(
                "export config {} has empty note_template",
                path.display()
            ));
        }
        Ok(())
    }
}

impl ChapterNav {
    pub(crate) fn href(&self) -> Result<String, String> {
        let source = Path::new(&self.source);
        let stem = source
            .file_stem()
            .ok_or_else(|| format!("chapter source {} has no file stem", self.source))?
            .to_string_lossy();
        Ok(format!("{stem}.html"))
    }
}

fn apply_chapter_template(template: &str, number: u8, title: &str) -> String {
    template
        .replace("{number}", &number.to_string())
        .replace("{title}", title)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn finds_current_chapter_by_input_file_name() {
        let book = ExportConfig {
            how_to_cite: citation_config(),
            chapters: vec![ChapterNav {
                number: 4,
                source: "notes/P4-multicalibration.typ".to_owned(),
                short_title: "Multicalibration".to_owned(),
            }],
        };

        assert_eq!(
            book.current_index_for_input(Path::new("P4-multicalibration.typ")),
            Some(0)
        );
    }

    #[test]
    fn derives_href_from_source_file() {
        let chapter = ChapterNav {
            number: 4,
            source: "notes/P4-multicalibration.typ".to_owned(),
            short_title: "Multicalibration".to_owned(),
        };

        assert_eq!(chapter.href().as_deref(), Ok("P4-multicalibration.html"));
    }

    #[test]
    fn applies_citation_templates() {
        let config = citation_config();

        assert_eq!(
            config.citation_title(2, "Beyond Normal Form"),
            "Chapter 2: Beyond Normal Form"
        );
        assert_eq!(
            config.citation_note(2, "Beyond Normal Form"),
            "Chapter 2 of the ACM EC 2026 tutorial notes"
        );
    }

    fn citation_config() -> CitationConfig {
        CitationConfig {
            authors: "A. Author".to_owned(),
            key_prefix: "notes".to_owned(),
            title_template: "Chapter {number}: {title}".to_owned(),
            note_template: "Chapter {number} of the ACM EC 2026 tutorial notes".to_owned(),
            year: 2026,
        }
    }
}
