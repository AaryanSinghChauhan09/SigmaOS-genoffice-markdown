// SPDX-License-Identifier: MIT
//! SigmaOS-genoffice-markdown — Markdown document parser and HTML renderer.
//!
//! Provides CommonMark-style block parsing (headings, paragraphs, fenced code,
//! lists, rules), inline formatting (emphasis, strong, code spans, links),
//! HTML escaping, word counts, and heading extraction. CLI-friendly and
//! deterministic; no external dependencies.

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

/// Parse a single non-fence line into a structural token.
pub fn parse_line(line: &str) -> MdToken {
    let trimmed = line.trim();
    if trimmed.is_empty() {
        return MdToken::Blank;
    }
    if matches!(trimmed, "---" | "***" | "___") {
        return MdToken::HorizontalRule;
    }
    for (prefix, level) in [("#### ", 4u8), ("### ", 3), ("## ", 2), ("# ", 1)] {
        if let Some(rest) = trimmed.strip_prefix(prefix) {
            return MdToken::Heading { level, text: rest.trim().to_string() };
        }
    }
    if let Some(rest) = trimmed.strip_prefix("- ").or_else(|| trimmed.strip_prefix("* ")) {
        return MdToken::BulletItem(rest.trim().to_string());
    }
    if let Some(dot) = trimmed.find(". ") {
        if let Ok(n) = trimmed[..dot].parse::<u32>() {
            if !trimmed[..dot].is_empty() && trimmed[..dot].chars().all(|c| c.is_ascii_digit()) {
                return MdToken::NumberedItem { n, text: trimmed[dot + 2..].trim().to_string() };
            }
        }
    }
    MdToken::Paragraph(trimmed.to_string())
}

/// Parse a full Markdown document into block tokens, accumulating fenced code.
pub fn parse_markdown(input: &str) -> Vec<MdToken> {
    let mut tokens = Vec::new();
    let mut in_code = false;
    let mut lang: Option<String> = None;
    let mut code = String::new();

    for line in input.lines() {
        let trimmed = line.trim();
        if trimmed.starts_with("```") {
            if in_code {
                tokens.push(MdToken::CodeBlock { lang: lang.take(), code: std::mem::take(&mut code) });
                in_code = false;
            } else {
                in_code = true;
                let rest = trimmed.trim_start_matches('`').trim();
                lang = if rest.is_empty() { None } else { Some(rest.to_string()) };
            }
            continue;
        }
        if in_code {
            code.push_str(line);
            code.push('\n');
            continue;
        }
        tokens.push(parse_line(line));
    }
    if in_code {
        tokens.push(MdToken::CodeBlock { lang, code });
    }
    tokens
}

fn escape_html(s: &str) -> String {
    s.replace('&', "&amp;")
        .replace('<', "&lt;")
        .replace('>', "&gt;")
        .replace('"', "&quot;")
}

fn find_byte(haystack: &[u8], from: usize, needle: u8) -> Option<usize> {
    haystack.get(from..)?.iter().position(|&b| b == needle).map(|p| from + p)
}

fn find_subslice(haystack: &[u8], from: usize, needle: &[u8]) -> Option<usize> {
    let bytes = haystack.get(from..)?;
    bytes.windows(needle.len()).position(|w| w == needle).map(|p| from + p)
}

/// Render inline Markdown (emphasis, strong, code, links) to HTML.
pub fn render_inline(src: &str) -> String {
    let b = src.as_bytes();
    let mut out = String::new();
    let mut start = 0usize;
    let mut i = 0usize;

    while i < b.len() {
        if b[i] == b'`' {
            if let Some(j) = find_byte(b, i + 1, b'`') {
                out.push_str(&escape_html(&src[start..i]));
                out.push_str("<code>");
                out.push_str(&escape_html(&src[i + 1..j]));
                out.push_str("</code>");
                i = j + 1;
                start = i;
                continue;
            }
        }
        if i + 1 < b.len() && b[i] == b'*' && b[i + 1] == b'*' {
            if let Some(j) = find_subslice(b, i + 2, b"**") {
                out.push_str(&escape_html(&src[start..i]));
                out.push_str("<strong>");
                out.push_str(&render_inline(&src[i + 2..j]));
                out.push_str("</strong>");
                i = j + 2;
                start = i;
                continue;
            }
        }
        if b[i] == b'[' {
            if let Some(rb) = src[i + 1..].find(']').map(|x| i + 1 + x) {
                if rb + 1 < b.len() && b[rb + 1] == b'(' {
                    if let Some(pb) = src[rb + 2..].find(')').map(|x| rb + 2 + x) {
                        out.push_str(&escape_html(&src[start..i]));
                        out.push_str("<a href=\"");
                        out.push_str(&escape_html(&src[rb + 2..pb]));
                        out.push_str("\">");
                        out.push_str(&render_inline(&src[i + 1..rb]));
                        out.push_str("</a>");
                        i = pb + 1;
                        start = i;
                        continue;
                    }
                }
            }
        }
        if b[i] == b'*' || b[i] == b'_' {
            let marker = b[i];
            if let Some(j) = find_byte(b, i + 1, marker) {
                out.push_str(&escape_html(&src[start..i]));
                out.push_str("<em>");
                out.push_str(&escape_html(&src[i + 1..j]));
                out.push_str("</em>");
                i = j + 1;
                start = i;
                continue;
            }
        }
        i += 1;
    }

    out.push_str(&escape_html(&src[start..]));
    out
}

/// Convert parsed Markdown tokens into semantic HTML markup.
pub fn tokens_to_html(tokens: &[MdToken]) -> String {
    let mut out = String::new();
    let mut i = 0usize;

    while i < tokens.len() {
        match &tokens[i] {
            MdToken::Heading { level, text } => {
                out.push_str(&format!("<h{level}>{}</h{level}>\n", render_inline(text)));
                i += 1;
            }
            MdToken::Paragraph(text) => {
                out.push_str(&format!("<p>{}</p>\n", render_inline(text)));
                i += 1;
            }
            MdToken::CodeBlock { lang, code } => {
                let class = lang.as_deref().map(|l| format!(" class=\"language-{l}\"")).unwrap_or_default();
                out.push_str(&format!("<pre><code{class}>{}</code></pre>\n", escape_html(code)));
                i += 1;
            }
            MdToken::BulletItem(_) => {
                out.push_str("<ul>\n");
                while i < tokens.len() {
                    if let MdToken::BulletItem(text) = &tokens[i] {
                        out.push_str(&format!("<li>{}</li>\n", render_inline(text)));
                        i += 1;
                    } else {
                        break;
                    }
                }
                out.push_str("</ul>\n");
            }
            MdToken::NumberedItem { .. } => {
                out.push_str("<ol>\n");
                while i < tokens.len() {
                    if let MdToken::NumberedItem { n, text } = &tokens[i] {
                        let _ = n;
                        out.push_str(&format!("<li>{}</li>\n", render_inline(text)));
                        i += 1;
                    } else {
                        break;
                    }
                }
                out.push_str("</ol>\n");
            }
            MdToken::HorizontalRule => {
                out.push_str("<hr />\n");
                i += 1;
            }
            MdToken::Blank => {
                i += 1;
            }
        }
    }

    out
}

/// Parse Markdown source and render it to HTML in one step.
pub fn markdown_to_html(input: &str) -> String {
    tokens_to_html(&parse_markdown(input))
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
        assert_eq!(parse_line("# SigmaOS Architecture"), MdToken::Heading { level: 1, text: "SigmaOS Architecture".into() });
    }

    #[test]
    fn test_parse_bullet() {
        assert_eq!(parse_line("- Zero-copy messaging"), MdToken::BulletItem("Zero-copy messaging".into()));
    }

    #[test]
    fn test_parse_markdown_code_block() {
        let md = "# T\n\n```rust\nfn main() {}\n```\n";
        let tokens = parse_markdown(md);
        assert!(tokens.iter().any(|t| matches!(t, MdToken::CodeBlock { lang: Some(l), .. } if l == "rust")));
        let html = markdown_to_html(md);
        assert!(html.contains("<code class=\"language-rust\">fn main() {}\n</code>"));
    }

    #[test]
    fn test_inline_formatting() {
        assert_eq!(render_inline("**bold** and *em* and `x`"), "<strong>bold</strong> and <em>em</em> and <code>x</code>");
        assert_eq!(render_inline("[Sigma](https://example.invalid)"), "<a href=\"https://example.invalid\">Sigma</a>");
    }

    #[test]
    fn test_list_grouping() {
        let html = markdown_to_html("- a\n- b\n1. c\n");
        assert!(html.contains("<ul>\n<li>a</li>\n<li>b</li>\n</ul>"));
        assert!(html.contains("<ol>\n<li>c</li>\n</ol>"));
    }

    #[test]
    fn test_html_escape() {
        assert_eq!(render_inline("<script>"), "&lt;script&gt;");
    }

    #[test]
    fn test_extract_headings_and_word_count() {
        let tokens = parse_markdown("# A\n\ntext here\n");
        assert_eq!(extract_headings(&tokens), vec![(1, "A".to_string())]);
        assert_eq!(word_count("one two three"), 3);
    }
}
