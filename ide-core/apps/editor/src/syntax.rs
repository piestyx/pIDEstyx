use tree_sitter::{Node, Parser, Tree};
use std::fmt;
use serde::Serialize;

// Language modules
use tree_sitter_cpp as tscpp;
use tree_sitter_c_sharp as tscs;
use tree_sitter_javascript as tsjs;
use tree_sitter_python as tspy;
use tree_sitter_rust as tsrs;
use tree_sitter_typescript::{language_tsx, language_typescript};

#[derive(Debug, Clone, Serialize)]
pub struct SerializableRange {
    pub start_row: usize,
    pub start_col: usize,
    pub end_row: usize,
    pub end_col: usize,
}

impl From<tree_sitter::Range> for SerializableRange {
    fn from(r: tree_sitter::Range) -> Self {
        Self {
            start_row: r.start_point.row,
            start_col: r.start_point.column,
            end_row: r.end_point.row,
            end_col: r.end_point.column,
        }
    }
}

#[derive(Debug, Clone, Serialize)]
pub struct HighlightSpan {
    pub range: SerializableRange,
    pub highlight_type: String,
}

#[derive(Debug, Clone)]
pub enum SupportedLanguage {
    CPP,
    CSharp,
    JavaScript,
    Python,
    Rust,
    TSX,
    TypeScript,
}

impl SupportedLanguage {
    pub fn tree_sitter_language(&self) -> tree_sitter::Language {
        match self {
            SupportedLanguage::CPP => tscpp::language(),
            SupportedLanguage::CSharp => tscs::language(),
            SupportedLanguage::JavaScript => tsjs::language(),
            SupportedLanguage::Python => tspy::language(),
            SupportedLanguage::Rust => tsrs::language(),
            SupportedLanguage::TypeScript => language_typescript(),
            SupportedLanguage::TSX => language_tsx(),
        }
    }

    pub fn from_extension(ext: &str) -> Option<Self> {
        match ext {
            "cpp" | "cxx" | "cc" => Some(Self::CPP),
            "cs" => Some(Self::CSharp),
            "js" => Some(Self::JavaScript),
            "py" => Some(Self::Python),
            "rs" => Some(Self::Rust),
            "ts" => Some(Self::TypeScript),
            "tsx" => Some(Self::TSX),
            _ => None,
        }
    }
}

impl fmt::Display for SupportedLanguage {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let name = match self {
            Self::CPP => "C++",
            Self::CSharp => "C#",
            Self::JavaScript => "JavaScript",
            Self::Python => "Python",
            Self::Rust => "Rust",
            Self::TypeScript => "TypeScript",
            Self::TSX => "TSX",
        };
        write!(f, "{name}")
    }
}

#[derive(Debug)]
pub enum SyntaxError {
    ParseFailed,
}

pub struct SyntaxEngine {
    parser: Parser,
    language: SupportedLanguage,
}

impl SyntaxEngine {
    pub fn new(language: SupportedLanguage) -> Self {
        let mut parser = Parser::new();
        parser
            .set_language(language.tree_sitter_language())
            .expect("Failed to set Tree-sitter language");
        Self { parser, language }
    }

    pub fn parse(&mut self, source: &str) -> Result<Tree, SyntaxError> {
        self.parser
            .parse(source, None)
            .ok_or(SyntaxError::ParseFailed)
    }

    pub fn current_language(&self) -> &SupportedLanguage {
        &self.language
    }

    pub fn extract_highlights(&mut self, source: &str) -> Vec<HighlightSpan> {
        let tree = match self.parser.parse(source, None) {
            Some(t) => t,
            None => return vec![],
        };
        Self::extract_highlights_from_tree(&tree)
    }

    pub fn extract_highlights_from_tree(tree: &Tree) -> Vec<HighlightSpan> {
        let mut highlights = Vec::new();
        let root_node = tree.root_node();

        fn recurse(node: Node, highlights: &mut Vec<HighlightSpan>) {
            if node.is_named() {
                highlights.push(HighlightSpan {
                    range: node.range().into(),
                    highlight_type: node.kind().into(),
                });
            }

            let mut child_cursor = node.walk();
            for child in node.children(&mut child_cursor) {
                recurse(child, highlights);
            }
        }

        recurse(root_node, &mut highlights);
        highlights
    }
}
