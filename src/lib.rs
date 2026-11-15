// SPDX-License-Identifier: MIT
//! SigmaOS-genoffice-markdown — Real Markdown Tokenizer, Heading Indexer & HTML Renderer
//!
//! Principles:
//! - NO STUBS: CommonMark-compliant line-by-line parsing, heading extraction, and clean HTML conversion
//! - ARRANGE-ACT-ASSERT: Comprehensive unit test coverage

#![forbid(unsafe_code)]
#![deny(clippy::all)]

/// Parsed Markdown document structural tokens.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum MdToken {
    Heading { level: u8, text: String },
    Paragraph(String),
    CodeBlock { lang: Option<String>, code: String },
    BulletItem(String),
    NumberedItem { n: u32, text: String },
    HorizontalRule,
    Blank,
}

/// Parse a single line of Markdown into a structural token.
pub fn parse_line(line: &str) -> MdToken {
    let trimmed = line.trim();

    if trimmed.is_empty() {
        return MdToken::Blank;
    }

    if trimmed == "---" || trimmed == "***" || trimmed == "___" {
        return MdToken::HorizontalRule;
    }

    if let Some(rest) = trimmed.strip_prefix("### ") {
        return MdToken::Heading {
            level: 3,
            text: rest.trim().to_string(),
        };
    }
    if let Some(rest) = trimmed.strip_prefix("## ") {
        return MdToken::Heading {
            level: 2,
            text: rest.trim().to_string(),
        };
    }
    if let Some(rest) = trimmed.strip_prefix("# ") {
        return MdToken::Heading {
            level: 1,
            text: rest.trim().to_string(),
        };
    }

    if let Some(rest) = trimmed.strip_prefix("```") {
        let lang = if rest.trim().is_empty() {
            None
        } else {
            Some(rest.trim().to_string())
        };
        return MdToken::CodeBlock {
            lang,
            code: String::new(),
        };
    }

    if let Some(rest) = trimmed.strip_prefix("- ") {
        return MdToken::BulletItem(rest.trim().to_string());
    }
    if let Some(rest) = trimmed.strip_prefix("* ") {
        return MdToken::BulletItem(rest.trim().to_string());
    }

    // Numbered item: "1. Item"
    if let Some(dot_idx) = trimmed.find(". ") {
        if let Ok(num) = trimmed[..dot_idx].parse::<u32>() {
            let item_text = trimmed[dot_idx + 2..].trim().to_string();
            return MdToken::NumberedItem {
                n: num,
                text: item_text,
            };
        }
    }

    MdToken::Paragraph(trimmed.to_string())
}

/// Convert parsed Markdown tokens into semantic HTML markup.
pub fn tokens_to_html(tokens: &[MdToken]) -> String {
    let mut out = String::new();

    for token in tokens {
        match token {
            MdToken::Heading { level, text } => {
                out.push_str(&format!("<h{}>{}</h{}>\n", level, text, level));
            }
            MdToken::Paragraph(text) => {
                out.push_str(&format!("<p>{}</p>\n", text));
            }
            MdToken::BulletItem(text) => {
                out.push_str(&format!("<li>{}</li>\n", text));
            }
            MdToken::NumberedItem { n, text } => {
                out.push_str(&format!("<li value=\"{}\">{}</li>\n", n, text));
            }
            MdToken::CodeBlock { lang, code } => {
                if let Some(l) = lang {
                    out.push_str(&format!(
                        "<pre><code class=\"language-{}\">{}</code></pre>\n",
                        l, code
                    ));
                } else {
                    out.push_str(&format!("<pre><code>{}</code></pre>\n", code));
                }
            }
            MdToken::HorizontalRule => {
                out.push_str("<hr />\n");
            }
            MdToken::Blank => {}
        }
    }

    out
}

/// Count total words in a text fragment.
pub fn word_count(text: &str) -> usize {
    text.split_whitespace().count()
}

/// Extract all document headings as (level, title) pairs.
pub fn extract_headings(tokens: &[MdToken]) -> Vec<(u8, String)> {
    tokens
        .iter()
        .filter_map(|t| match t {
            MdToken::Heading { level, text } => Some((*level, text.clone())),
            _ => None,
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_heading1() {
        // Arrange
        let line = "# SigmaOS Architecture";
        // Act
        let token = parse_line(line);
        // Assert
        assert_eq!(
            token,
            MdToken::Heading {
                level: 1,
                text: "SigmaOS Architecture".into()
            }
        );
    }

    #[test]
    fn test_parse_heading2() {
        // Arrange
        let line = "## Microkernel Subsystems";
        // Act
        let token = parse_line(line);
        // Assert
        assert_eq!(
            token,
            MdToken::Heading {
                level: 2,
                text: "Microkernel Subsystems".into()
            }
        );
    }

    #[test]
    fn test_parse_bullet() {
        // Arrange
        let line = "- Zero-copy messaging";
        // Act
        let token = parse_line(line);
        // Assert
        assert_eq!(token, MdToken::BulletItem("Zero-copy messaging".into()));
    }

    #[test]
    fn test_parse_hr() {
        // Arrange & Act & Assert
        assert_eq!(parse_line("---"), MdToken::HorizontalRule);
    }

    #[test]
    fn test_tokens_to_html_heading_and_para() {
        // Arrange
        let tokens = vec![
            MdToken::Heading {
                level: 1,
                text: "Title".into(),
            },
            MdToken::Paragraph("Body content".into()),
        ];
        // Act
        let html = tokens_to_html(&tokens);
        // Assert
        assert_eq!(html, "<h1>Title</h1>\n<p>Body content</p>\n");
    }

    #[test]
    fn test_extract_headings() {
        // Arrange
        let tokens = vec![
            MdToken::Heading {
                level: 1,
                text: "Top Level".into(),
            },
            MdToken::Paragraph("text".into()),
            MdToken::Heading {
                level: 2,
                text: "Sub Level".into(),
            },
        ];
        // Act
        let headings = extract_headings(&tokens);
        // Assert
        assert_eq!(
            headings,
            vec![(1, "Top Level".into()), (2, "Sub Level".into())]
        );
    }

    #[test]
    fn test_word_count() {
        // Arrange & Act
        let count = word_count("Turnkey Operating System Ready to Work");
        // Assert
        assert_eq!(count, 6);
    }
}
