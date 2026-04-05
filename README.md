# md2html

`md2html` is a small Rust Markdown-to-HTML converter with a CLI.

## Features

- Headings
- Paragraphs
- Fenced code blocks
- Unordered lists
- Ordered lists
- Blockquotes
- Inline emphasis, strong text, links, code spans, and HTML escaping

## Build

```bash
cargo build
```

## Test

```bash
cargo test
```

## Use

Read Markdown from standard input:

```bash
cat input.md | cargo run --quiet
```

Or pass a file path:

```bash
cargo run --quiet -- input.md
```

## Example

Input:

```markdown
# Title

Hello **world**.

- One
- Two
```

Output:

```html
<h1>Title</h1>
<p>Hello <strong>world</strong>.</p>
<ul>
<li>One</li>
<li>Two</li>
</ul>
```

