//! Comment preservation for the Atlas formatter

use atlas_runtime::span::Span;
use atlas_runtime::token::{Token, TokenKind};

/// A collected comment from source code
#[derive(Debug, Clone, PartialEq)]
pub struct Comment {
    /// What kind of comment
    pub kind: CommentKind,
    /// The comment text (including delimiters)
    pub text: String,
    /// Source location
    pub span: Span,
    /// Where this comment is positioned relative to code
    pub position: CommentPosition,
}

/// Types of comments
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CommentKind {
    /// Single-line comment: // ...
    Line,
    /// Block comment: /* ... */
    Block,
    /// Doc comment: /// ...
    Doc,
}

/// Position of a comment relative to code
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CommentPosition {
    /// Before a statement or expression (on its own line)
    Leading,
    /// After code on the same line
    Trailing,
    /// Between other comments or at file boundaries
    Standalone,
}

/// Collects comments from a token stream and associates them with positions
pub struct CommentCollector {
    comments: Vec<Comment>,
}

impl Default for CommentCollector {
    fn default() -> Self {
        Self::new()
    }
}

impl CommentCollector {
    pub fn new() -> Self {
        Self {
            comments: Vec::new(),
        }
    }

    /// Extract comments from a token stream, classifying their positions
    pub fn collect_from_tokens(&mut self, tokens: &[Token], source: &str) {
        let mut i = 0;
        while i < tokens.len() {
            let token = &tokens[i];
            match token.kind {
                TokenKind::LineComment | TokenKind::BlockComment | TokenKind::DocComment => {
                    let kind = match token.kind {
                        TokenKind::LineComment => CommentKind::Line,
                        TokenKind::BlockComment => CommentKind::Block,
                        TokenKind::DocComment => CommentKind::Doc,
                        _ => unreachable!(),
                    };

                    let position = self.classify_position(token, tokens, i, source);

                    self.comments.push(Comment {
                        kind,
                        text: token.lexeme.clone(),
                        span: token.span,
                        position,
                    });
                }
                _ => {}
            }
            i += 1;
        }
    }

    /// Classify whether a comment is leading, trailing, or standalone
    fn classify_position(
        &self,
        comment_token: &Token,
        tokens: &[Token],
        index: usize,
        source: &str,
    ) -> CommentPosition {
        // Check if there's code before this comment on the same line
        let comment_line = line_of(source, comment_token.span.start);

        // Look backwards for a non-comment, non-EOF token on the same line
        let has_code_before_on_same_line = (0..index).rev().any(|j| {
            let t = &tokens[j];
            !matches!(
                t.kind,
                TokenKind::LineComment | TokenKind::BlockComment | TokenKind::DocComment
            ) && t.kind != TokenKind::Eof
                && line_of(source, t.span.end.saturating_sub(1)) == comment_line
        });

        if has_code_before_on_same_line {
            return CommentPosition::Trailing;
        }

        // Look forwards for a non-comment token
        let has_code_after = ((index + 1)..tokens.len()).any(|j| {
            let t = &tokens[j];
            !matches!(
                t.kind,
                TokenKind::LineComment
                    | TokenKind::BlockComment
                    | TokenKind::DocComment
                    | TokenKind::Eof
            )
        });

        if has_code_after {
            CommentPosition::Leading
        } else {
            CommentPosition::Standalone
        }
    }

    /// Consume and return collected comments
    pub fn into_comments(self) -> Vec<Comment> {
        self.comments
    }

    /// Get comments that are leading (before) a given byte offset
    pub fn leading_comments_before(&self, offset: usize) -> Vec<&Comment> {
        self.comments
            .iter()
            .filter(|c| c.span.end <= offset && c.position == CommentPosition::Leading)
            .collect()
    }

    /// Get trailing comments after a given byte offset
    pub fn trailing_comment_after(&self, offset: usize) -> Option<&Comment> {
        self.comments
            .iter()
            .find(|c| c.span.start >= offset && c.position == CommentPosition::Trailing)
    }

    /// Get all comments
    pub fn comments(&self) -> &[Comment] {
        &self.comments
    }
}

/// Get the line number (0-indexed) for a byte offset
fn line_of(source: &str, offset: usize) -> usize {
    source[..offset.min(source.len())]
        .chars()
        .filter(|&c| c == '\n')
        .count()
}

#[cfg(test)]
mod tests {
    use super::*;
    use atlas_runtime::lexer::Lexer;

    #[test]
    fn test_collect_line_comment() {
        let source = "// hello\nlet x = 5;";
        let mut lexer = Lexer::new(source);
        let (tokens, _) = lexer.tokenize_with_comments();
        let mut collector = CommentCollector::new();
        collector.collect_from_tokens(&tokens, source);
        let comments = collector.into_comments();
        assert_eq!(comments.len(), 1);
        assert_eq!(comments[0].kind, CommentKind::Line);
        assert_eq!(comments[0].text, "// hello");
        assert_eq!(comments[0].position, CommentPosition::Leading);
    }

    #[test]
    fn test_collect_trailing_comment() {
        let source = "let x = 5; // inline";
        let mut lexer = Lexer::new(source);
        let (tokens, _) = lexer.tokenize_with_comments();
        let mut collector = CommentCollector::new();
        collector.collect_from_tokens(&tokens, source);
        let comments = collector.into_comments();
        assert_eq!(comments.len(), 1);
        assert_eq!(comments[0].position, CommentPosition::Trailing);
    }

    #[test]
    fn test_collect_block_comment() {
        let source = "/* block */\nlet x = 5;";
        let mut lexer = Lexer::new(source);
        let (tokens, _) = lexer.tokenize_with_comments();
        let mut collector = CommentCollector::new();
        collector.collect_from_tokens(&tokens, source);
        let comments = collector.into_comments();
        assert_eq!(comments.len(), 1);
        assert_eq!(comments[0].kind, CommentKind::Block);
    }

    #[test]
    fn test_collect_doc_comment() {
        let source = "/// doc comment\nlet x = 5;";
        let mut lexer = Lexer::new(source);
        let (tokens, _) = lexer.tokenize_with_comments();
        let mut collector = CommentCollector::new();
        collector.collect_from_tokens(&tokens, source);
        let comments = collector.into_comments();
        assert_eq!(comments.len(), 1);
        assert_eq!(comments[0].kind, CommentKind::Doc);
    }

    #[test]
    fn test_standalone_comment() {
        let source = "// just a comment";
        let mut lexer = Lexer::new(source);
        let (tokens, _) = lexer.tokenize_with_comments();
        let mut collector = CommentCollector::new();
        collector.collect_from_tokens(&tokens, source);
        let comments = collector.into_comments();
        assert_eq!(comments.len(), 1);
        assert_eq!(comments[0].position, CommentPosition::Standalone);
    }
}
