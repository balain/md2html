pub fn markdown_to_html(input: &str) -> String {
    let blocks = parse_blocks(input);
    render_blocks(&blocks)
}

#[derive(Debug, Clone, PartialEq, Eq)]
enum Block {
    Heading { level: usize, text: String },
    Paragraph(String),
    CodeFence { language: Option<String>, content: String },
    UnorderedList(Vec<String>),
    OrderedList(Vec<String>),
    Blockquote(Vec<String>),
    Table { headers: Vec<String>, rows: Vec<Vec<String>> },
}

fn parse_blocks(input: &str) -> Vec<Block> {
    let lines: Vec<&str> = input.lines().collect();
    let mut blocks = Vec::new();
    let mut i = 0;

    while i < lines.len() {
        let line = lines[i].trim_end();

        if line.trim().is_empty() {
            i += 1;
            continue;
        }

        if let Some((level, text)) = parse_heading(line) {
            blocks.push(Block::Heading { level, text });
            i += 1;
            continue;
        }

        if let Some((language, content, next)) = parse_code_fence(&lines, i) {
            blocks.push(Block::CodeFence { language, content });
            i = next;
            continue;
        }

        if let Some((items, next)) = parse_list(&lines, i, ListKind::Unordered) {
            blocks.push(Block::UnorderedList(items));
            i = next;
            continue;
        }

        if let Some((items, next)) = parse_list(&lines, i, ListKind::Ordered) {
            blocks.push(Block::OrderedList(items));
            i = next;
            continue;
        }

        if let Some((items, next)) = parse_blockquote(&lines, i) {
            blocks.push(Block::Blockquote(items));
            i = next;
            continue;
        }

        if let Some((block, next)) = parse_table(&lines, i) {
            blocks.push(block);
            i = next;
            continue;
        }

        let (paragraph, next) = parse_paragraph(&lines, i);
        blocks.push(Block::Paragraph(paragraph));
        i = next;
    }

    blocks
}

fn render_blocks(blocks: &[Block]) -> String {
    let mut out = String::new();

    for block in blocks {
        match block {
            Block::Heading { level, text } => {
                out.push_str(&format!(
                    "<h{level}>{}</h{level}>\n",
                    render_inline(text)
                ));
            }
            Block::Paragraph(text) => {
                out.push_str(&format!("<p>{}</p>\n", render_inline(text)));
            }
            Block::CodeFence { language, content } => {
                let class = language
                    .as_ref()
                    .map(|lang| format!(" class=\"language-{}\"", escape_html(lang)))
                    .unwrap_or_default();
                out.push_str(&format!(
                    "<pre><code{class}>{}</code></pre>\n",
                    escape_html(content)
                ));
            }
            Block::UnorderedList(items) => {
                out.push_str("<ul>\n");
                for item in items {
                    out.push_str(&format!("<li>{}</li>\n", render_inline(item)));
                }
                out.push_str("</ul>\n");
            }
            Block::OrderedList(items) => {
                out.push_str("<ol>\n");
                for item in items {
                    out.push_str(&format!("<li>{}</li>\n", render_inline(item)));
                }
                out.push_str("</ol>\n");
            }
            Block::Blockquote(lines) => {
                out.push_str("<blockquote>\n");
                for line in lines {
                    out.push_str(&format!("<p>{}</p>\n", render_inline(line)));
                }
                out.push_str("</blockquote>\n");
            }
            Block::Table { headers, rows } => {
                out.push_str("<table>\n<thead>\n<tr>\n");
                for header in headers {
                    out.push_str(&format!("<th>{}</th>\n", render_inline(header)));
                }
                out.push_str("</tr>\n</thead>\n<tbody>\n");
                for row in rows {
                    out.push_str("<tr>\n");
                    for cell in row {
                        out.push_str(&format!("<td>{}</td>\n", render_inline(cell)));
                    }
                    out.push_str("</tr>\n");
                }
                out.push_str("</tbody>\n</table>\n");
            }
        }
    }

    out
}

fn parse_heading(line: &str) -> Option<(usize, String)> {
    let trimmed = line.trim_start();
    let mut level = 0;
    for ch in trimmed.chars() {
        if ch == '#' {
            level += 1;
        } else {
            break;
        }
    }

    if level == 0 || level > 6 {
        return None;
    }

    let rest = trimmed[level..].trim_start();
    if rest.is_empty() {
        return None;
    }

    Some((level, rest.to_string()))
}

fn parse_code_fence(lines: &[&str], start: usize) -> Option<(Option<String>, String, usize)> {
    let line = lines[start].trim_start();
    if !line.starts_with("```") {
        return None;
    }

    let language = line[3..].trim();
    let language = if language.is_empty() {
        None
    } else {
        Some(language.to_string())
    };

    let mut content = String::new();
    let mut i = start + 1;
    while i < lines.len() {
        let current = lines[i];
        if current.trim_start().starts_with("```") {
            return Some((language, content, i + 1));
        }
        content.push_str(current);
        if i + 1 < lines.len() {
            content.push('\n');
        }
        i += 1;
    }

    Some((language, content, lines.len()))
}

#[derive(Copy, Clone)]
enum ListKind {
    Unordered,
    Ordered,
}

fn parse_list(lines: &[&str], start: usize, kind: ListKind) -> Option<(Vec<String>, usize)> {
    let mut items = Vec::new();
    let mut i = start;

    while i < lines.len() {
        let line = lines[i].trim_end();
        if line.trim().is_empty() {
            break;
        }

        let item = match kind {
            ListKind::Unordered => line
                .trim_start()
                .strip_prefix("- ")
                .or_else(|| line.trim_start().strip_prefix("* "))
                .or_else(|| line.trim_start().strip_prefix("+ ")),
            ListKind::Ordered => ordered_list_item(line.trim_start()),
        };

        let Some(item) = item else {
            break;
        };

        items.push(item.to_string());
        i += 1;
    }

    if items.is_empty() {
        None
    } else {
        Some((items, i))
    }
}

fn ordered_list_item(line: &str) -> Option<&str> {
    let mut seen_digit = false;
    let mut idx = 0;
    for (pos, ch) in line.char_indices() {
        if ch.is_ascii_digit() {
            seen_digit = true;
            idx = pos + ch.len_utf8();
            continue;
        }
        if ch == '.' && seen_digit && line[idx..].starts_with(". ") {
            return Some(&line[idx + 2..]);
        }
        break;
    }
    None
}

fn parse_blockquote(lines: &[&str], start: usize) -> Option<(Vec<String>, usize)> {
    if !lines[start].trim_start().starts_with('>') {
        return None;
    }

    let mut items = Vec::new();
    let mut i = start;
    while i < lines.len() {
        let line = lines[i].trim_start();
        if !line.starts_with('>') {
            break;
        }
        let content = line[1..].trim_start();
        items.push(content.to_string());
        i += 1;
    }

    Some((items, i))
}

fn parse_paragraph(lines: &[&str], start: usize) -> (String, usize) {
    let mut parts = Vec::new();
    let mut i = start;

    while i < lines.len() {
        let line = lines[i].trim_end();
        if line.trim().is_empty()
            || parse_heading(line).is_some()
            || line.trim_start().starts_with("```")
            || line.trim_start().starts_with('>')
            || is_list_start(line)
            || parse_table(lines, i).is_some()
        {
            break;
        }

        parts.push(line.trim().to_string());
        i += 1;
    }

    (parts.join(" "), i)
}

fn is_table_row(line: &str) -> bool {
    line.contains('|')
}

fn is_separator_row(line: &str) -> bool {
    let trimmed = line.trim();
    trimmed.contains('|') && trimmed.chars().all(|c| matches!(c, '|' | '-' | ':' | ' '))
}

fn split_table_row(line: &str) -> Vec<String> {
    let trimmed = line.trim();
    let inner = trimmed.strip_prefix('|').unwrap_or(trimmed);
    let inner = inner.strip_suffix('|').unwrap_or(inner);
    inner.split('|').map(|cell| cell.trim().to_string()).collect()
}

fn parse_table(lines: &[&str], start: usize) -> Option<(Block, usize)> {
    if start + 1 >= lines.len() {
        return None;
    }
    let header_line = lines[start].trim_end();
    let sep_line = lines[start + 1].trim_end();
    if !is_table_row(header_line) || !is_separator_row(sep_line) {
        return None;
    }
    let headers = split_table_row(header_line);
    let mut rows = Vec::new();
    let mut i = start + 2;
    while i < lines.len() {
        let line = lines[i].trim_end();
        if line.trim().is_empty() || !is_table_row(line) {
            break;
        }
        rows.push(split_table_row(line));
        i += 1;
    }
    Some((Block::Table { headers, rows }, i))
}

fn is_list_start(line: &str) -> bool {
    let trimmed = line.trim_start();
    trimmed.starts_with("- ")
        || trimmed.starts_with("* ")
        || trimmed.starts_with("+ ")
        || ordered_list_item(trimmed).is_some()
}

fn render_inline(text: &str) -> String {
    let mut out = String::new();
    let mut chars = text.chars().peekable();

    while let Some(ch) = chars.next() {
        match ch {
            '&' => out.push_str("&amp;"),
            '<' => out.push_str("&lt;"),
            '>' => out.push_str("&gt;"),
            '`' => {
                let mut code = String::new();
                while let Some(&next) = chars.peek() {
                    chars.next();
                    if next == '`' {
                        break;
                    }
                    code.push(next);
                }
                out.push_str("<code>");
                out.push_str(&escape_html(&code));
                out.push_str("</code>");
            }
            '[' => {
                if let Some((label, url)) = parse_link(&mut chars) {
                    out.push_str("<a href=\"");
                    out.push_str(&escape_html(&url));
                    out.push_str("\">");
                    out.push_str(&render_inline(&label));
                    out.push_str("</a>");
                } else {
                    out.push('[');
                }
            }
            '*' => {
                if chars.peek() == Some(&'*') {
                    chars.next();
                    let mut bold = String::new();
                    while let Some(next) = chars.next() {
                        if next == '*' && chars.peek() == Some(&'*') {
                            chars.next();
                            break;
                        }
                        bold.push(next);
                    }
                    out.push_str("<strong>");
                    out.push_str(&render_inline(&bold));
                    out.push_str("</strong>");
                } else {
                    let mut italic = String::new();
                    while let Some(&next) = chars.peek() {
                        chars.next();
                        if next == '*' {
                            break;
                        }
                        italic.push(next);
                    }
                    out.push_str("<em>");
                    out.push_str(&render_inline(&italic));
                    out.push_str("</em>");
                }
            }
            _ => out.push(ch),
        }
    }

    out
}

fn parse_link(chars: &mut std::iter::Peekable<std::str::Chars<'_>>) -> Option<(String, String)> {
    let mut label = String::new();
    while let Some(next) = chars.next() {
        if next == ']' {
            if chars.next()? != '(' {
                return None;
            }
            let mut url = String::new();
            while let Some(next_url) = chars.next() {
                if next_url == ')' {
                    return Some((label, url));
                }
                url.push(next_url);
            }
            return None;
        }
        label.push(next);
    }
    None
}

fn escape_html(text: &str) -> String {
    text.chars()
        .flat_map(|ch| match ch {
            '&' => "&amp;".chars().collect::<Vec<_>>(),
            '<' => "&lt;".chars().collect::<Vec<_>>(),
            '>' => "&gt;".chars().collect::<Vec<_>>(),
            '"' => "&quot;".chars().collect::<Vec<_>>(),
            '\'' => "&#39;".chars().collect::<Vec<_>>(),
            _ => vec![ch],
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use super::markdown_to_html;

    #[test]
    fn renders_basic_blocks() {
        let input = "# Title\n\nHello *world*.\n\n- One\n- Two\n";
        let html = markdown_to_html(input);

        assert!(html.contains("<h1>Title</h1>"));
        assert!(html.contains("<p>Hello <em>world</em>.</p>"));
        assert!(html.contains("<ul>"));
        assert!(html.contains("<li>One</li>"));
        assert!(html.contains("<li>Two</li>"));
    }

    #[test]
    fn renders_table() {
        let input = "| Name | Age |\n|------|-----|\n| Alice | 30 |\n| Bob | 25 |\n";
        let html = markdown_to_html(input);

        assert!(html.contains("<table>"));
        assert!(html.contains("<thead>"));
        assert!(html.contains("<th>Name</th>"));
        assert!(html.contains("<th>Age</th>"));
        assert!(html.contains("<tbody>"));
        assert!(html.contains("<td>Alice</td>"));
        assert!(html.contains("<td>30</td>"));
        assert!(html.contains("<td>Bob</td>"));
        assert!(html.contains("<td>25</td>"));
        assert!(html.contains("</table>"));
    }

    #[test]
    fn renders_table_with_inline_formatting() {
        let input = "| Item | Description |\n|------|-------------|\n| **Bold** | `code` |\n";
        let html = markdown_to_html(input);

        assert!(html.contains("<th>Item</th>"));
        assert!(html.contains("<td><strong>Bold</strong></td>"));
        assert!(html.contains("<td><code>code</code></td>"));
    }

    #[test]
    fn renders_code_and_links() {
        let input = "```rust\nfn main() {}\n```\n\nSee [docs](https://example.com).";
        let html = markdown_to_html(input);

        assert!(html.contains("<code class=\"language-rust\">fn main() {}\n</code>"));
        assert!(html.contains("<a href=\"https://example.com\">docs</a>"));
    }
}

