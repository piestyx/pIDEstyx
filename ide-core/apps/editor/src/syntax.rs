use tree_sitter::{Parser, Tree, Language};
use tree_sitter_python::language as tree_sitter_python;

pub struct SyntaxEngine {
    parser: Parser,
}

impl SyntaxEngine {
    pub fn new() -> Self {
        let mut parser = Parser::new();
        let language = tree_sitter_python();
        parser.set_language(language).expect("Failed to set Tree-sitter language");
        Self { parser }
    }

    pub fn parse(&mut self, source: &str) -> Option<Tree> {
        self.parser.parse(source, None)
    }
}
