#[derive(Debug, Clone, PartialEq)]
pub enum Block {
    Heading { level: u8, text: String },
    Paragraph(String),
    CodeBlock { lang: Option<String>, code: String },
    BulletList(Vec<String>),
    HorizontalRule,
}

#[derive(Debug, Default)]
pub struct MarkdownDoc {
    blocks: Vec<Block>,
}

impl MarkdownDoc {
    pub fn new() -> Self { Self::default() }
    pub fn push_block(&mut self, block: Block) { self.blocks.push(block); }
    pub fn block_count(&self) -> usize { self.blocks.len() }
    pub fn to_html(&self) -> String {
        let mut out = String::new();
        for block in &self.blocks {
            match block {
                Block::Heading { level, text } => out.push_str(&format!("<h{l}>{t}</h{l}>\n", l = level, t = text)),
                Block::Paragraph(text) => out.push_str(&format!("<p>{}</p>\n", text)),
                Block::CodeBlock { lang, code } => {
                    let cls = lang.as_deref().map(|l| format!(" class=\"language-{}\"", l)).unwrap_or_default();
                    out.push_str(&format!("<pre><code{}>{}</code></pre>\n", cls, code));
                }
                Block::BulletList(items) => {
                    out.push_str("<ul>\n");
                    for item in items { out.push_str(&format!("  <li>{}</li>\n", item)); }
                    out.push_str("</ul>\n");
                }
                Block::HorizontalRule => out.push_str("<hr/>\n"),
            }
        }
        out
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn heading_to_html() {
        let mut doc = MarkdownDoc::new();
        doc.push_block(Block::Heading { level: 1, text: "SigmaOS".into() });
        assert!(doc.to_html().contains("<h1>SigmaOS</h1>"));
    }
}
