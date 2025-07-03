use ropey::Rope;
use std::fs::{self, File};
use std::io::{BufReader, BufWriter, Write};
use std::path::{Path, PathBuf};
use anyhow::{Context, Result};
use crate::syntax::SupportedLanguage;
use crate::syntax::{SyntaxEngine, HighlightSpan};

#[derive(Debug, Clone)]
pub struct BufferMetadata {
    pub path: Option<PathBuf>,
    pub trailing_newline: bool,
}

pub struct TextBuffer {
    rope: Rope,
    metadata: BufferMetadata,
}

#[allow(dead_code)]
pub struct Cursor {
    line: usize,
    column: usize,
    selection: Option<(usize, usize)>,
}

impl TextBuffer {
    pub fn from_file<P: AsRef<Path>>(path: P) -> Result<Self> {
        let path_ref = path.as_ref();
        let file = File::open(path_ref)
            .with_context(|| format!("Failed to open file: {}", path_ref.display()))?;
        let reader = BufReader::new(file);
        let rope = Rope::from_reader(reader)
            .with_context(|| format!("Failed to parse Rope from file: {}", path_ref.display()))?;

        let text = fs::read_to_string(path_ref)?;
        let trailing_newline = text.ends_with('\n');

        Ok(Self {
            rope,
            metadata: BufferMetadata {
                path: Some(path_ref.to_path_buf()),
                trailing_newline,
            },
        })
    }

    pub fn empty() -> Self {
        Self {
            rope: Rope::new(),
            metadata: BufferMetadata {
                path: None,
                trailing_newline: false,
            },
        }
    }

    pub fn normalize_newlines(&mut self) {
        let text = self.rope.to_string().replace("\r\n", "\n");
        self.rope = Rope::from_str(&text);
    }

    pub fn line_count(&self) -> usize {
        self.rope.len_lines()
    }

    pub fn char_count(&self) -> usize {
        self.rope.len_chars()
    }

    pub fn get_line(&self, index: usize) -> Option<String> {
        if index >= self.line_count() {
            None
        } else {
            Some(self.rope.line(index).to_string())
        }
    }

    pub fn set_line(&mut self, index: usize, text: &str) -> Result<()> {
        if index >= self.line_count() {
            anyhow::bail!("Line index out of bounds");
        }
        let start = self.rope.line_to_char(index);
        let end = self.rope.line_to_char(index + 1);
        self.rope.remove(start..end);
        self.rope.insert(start, text);
        Ok(())
    }

    pub fn save(&self) -> Result<()> {
        let path = self
            .metadata
            .path
            .as_ref()
            .context("No associated file path")?;

        let file = File::create(path)?;
        let mut writer = BufWriter::new(file);

        let last_line = self.rope.len_lines().saturating_sub(1);
        for i in 0..=last_line {
            let line = self.rope.line(i);
            let is_last = i == last_line;

            if !is_last || self.metadata.trailing_newline {
                write!(writer, "{}", line)?;
            } else {
                let string = line.to_string();
                let trimmed = string.trim_end_matches('\n');
                write!(writer, "{}", trimmed)?;
            }
        }

        Ok(())
    }

    pub fn save_as<P: AsRef<Path>>(&mut self, new_path: P) -> Result<()> {
        self.metadata.path = Some(new_path.as_ref().to_path_buf());
        self.save()
    }

    pub fn has_trailing_newline(&self) -> bool {
        self.metadata.trailing_newline
    }

    pub fn set_trailing_newline(&mut self, value: bool) {
        self.metadata.trailing_newline = value;
    }

    pub fn insert_line(&mut self, index: usize, text: &str) -> Result<()> {
        if index > self.line_count() {
            anyhow::bail!("Line index out of bounds");
        }

    
        let char_idx = self.rope.line_to_char(index);
        let line = if text.ends_with('\n') {
            text.to_string()
        } else {
            format!("{text}\n")
        };

        self.rope.insert(char_idx, &line);
        Ok(())
    }

    pub fn append_line(&mut self, text: &str) -> Result<()> {
        let line = if text.ends_with('\n') {
            text.to_string()
        } else {
            format!("{text}\n")
        };

        self.rope.append(Rope::from_str(&line));
        Ok(())
    }

    pub fn remove_line(&mut self, index: usize) -> Result<()> {
        if index >= self.line_count() {
            anyhow::bail!("Line index out of bounds");
        }

        let start = self.rope.line_to_char(index);
        let end = self.rope.line_to_char(index + 1);
        self.rope.remove(start..end);
        Ok(())
    }

    pub fn parse_syntax(&self, language: SupportedLanguage) -> Option<tree_sitter::Tree> {
        let mut engine = SyntaxEngine::new(language);
        let text = self.rope.to_string();
        engine.parse(&text)
    }

    pub fn extract_highlights(&self, language: SupportedLanguage) -> Vec<HighlightSpan> {
        let mut engine = SyntaxEngine::new(language);
        let text = self.rope.to_string();
        engine.extract_highlights(&text)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use tempfile::NamedTempFile;

    #[test]
    fn test_empty_buffer() {
        let buf = TextBuffer::empty();
        assert_eq!(buf.line_count(), 1);
        assert_eq!(buf.char_count(), 0);
    }

    #[test]
    fn test_line_get_set() {
        let mut buf = TextBuffer::empty();
        buf.set_line(0, "Hello world\n").unwrap();
        assert_eq!(buf.get_line(0).unwrap(), "Hello world\n");
    }

    #[test]
    fn test_file_io() -> Result<()> {
        let mut tmp = NamedTempFile::new()?;
        write!(tmp, "Line one\nLine two")?;

        let mut buf = TextBuffer::from_file(tmp.path())?;
        assert_eq!(buf.line_count(), 2);
        assert!(buf.get_line(1).unwrap().starts_with("Line two"));

        buf.set_line(1, "Changed line\n")?;
        buf.save()?;

        let saved = fs::read_to_string(tmp.path())?;
        assert!(saved.contains("Changed line"));

        Ok(())
    }

    #[test]
    fn test_normalize_newlines() {
        let mut buf = TextBuffer::empty();
        buf.set_line(0, "Hello\r\nWorld\r\n").unwrap();
        buf.normalize_newlines();

        assert_eq!(buf.line_count(), 3);
        assert_eq!(buf.get_line(0).unwrap(), "Hello\n");
        assert_eq!(buf.get_line(1).unwrap(), "World\n");
        assert_eq!(buf.get_line(2).unwrap(), "");
    }

    #[test]
    fn test_insert_append_remove() {
        let mut buf = TextBuffer::empty();
        buf.set_line(0, "Alpha\n").unwrap();
        buf.append_line("Beta").unwrap();
        buf.insert_line(1, "Insert").unwrap();

        assert_eq!(buf.line_count(), 4);
        assert_eq!(buf.get_line(0).unwrap(), "Alpha\n");
        assert_eq!(buf.get_line(1).unwrap(), "Insert\n");
        assert_eq!(buf.get_line(2).unwrap(), "Beta\n");
        assert_eq!(buf.get_line(3).unwrap(), "");

        buf.remove_line(1).unwrap();
        assert_eq!(buf.line_count(), 3);
        assert_eq!(buf.get_line(1).unwrap(), "Beta\n");
        assert_eq!(buf.get_line(2).unwrap(), "");
    }

    #[test]
    fn test_syntax_parse_python() {
        let mut buf = TextBuffer::empty();
        buf.set_line(0, "def foo():\n").unwrap();
        buf.append_line("    return 42").unwrap();

        let tree = buf.parse_syntax(SupportedLanguage::Python).expect("Failed to parse");
        let root = tree.root_node();

        assert_eq!(root.kind(), "module");
        assert!(root.named_child_count() > 0);
    }

    use crate::syntax::{SyntaxEngine, SupportedLanguage};
    #[test]
    fn test_syntax_parse_rust() {
        let mut engine = SyntaxEngine::new(SupportedLanguage::Rust);
        let source = r#"fn main() { println!("Hello"); }"#;
        let tree = engine.parse(source);
        assert!(tree.is_some());
    }

    #[test]
    fn test_syntax_parse_typescript() {
        let mut engine = SyntaxEngine::new(SupportedLanguage::TypeScript);
        let source = r#"function greet(name: string): void { console.log(name); }"#;
        let tree = engine.parse(source);
        assert!(tree.is_some());
    }

    #[test]
    fn test_extract_highlights_python() {
        let mut buf = TextBuffer::empty();
        buf.set_line(0, "def foo():\n").unwrap();
        buf.append_line("    return 42").unwrap();

        let highlights = buf.extract_highlights(SupportedLanguage::Python);
        assert!(highlights.iter().any(|h| h.highlight_type == "function_definition"));
    }
}

// End of file: ide-core/apps/editor/src/text_buffer.rs
// This file is part of pIDEstyx, a Rust-based IDE.