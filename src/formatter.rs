use ansi_term::Style;

use crate::outputstyle::OutputStyle;

pub fn format_cg(text: &str, ostyle: &OutputStyle) -> String {
    let tag_pairs = vec![
        (ostyle.monospace, "`", "`"),
        (ostyle.variable, "[[", "]]"),
        (ostyle.constant, "{{", "}}"),
        (ostyle.bold, "<<", ">>"),
    ];
    let default_style = Style::default();
    let mut sections: Vec<(Style, usize, usize)> = vec![];
    let mut stack: Vec<(Style, &str)> = vec![];
    let mut section_start = 0;

    for (i, ch) in text.char_indices() {
        if i < section_start {
            continue
        }
        if ch == '\n' {
            let style = match stack.last() {
                Some((style, _)) => style,
                None => &default_style,
            };
            sections.push((style.to_owned(), section_start, i));
            sections.push((default_style, i, i+1));
            section_start = i + 1;
            continue
        }
        let slice = &text[i..];
        let found_closing = match stack.last() {
            Some((style, closing)) if slice.starts_with(closing) => {
                sections.push((style.to_owned(), section_start, i));
                section_start = i + closing.len();
                true
            }
            _ => {
                for (style, tag_open, tag_close) in &tag_pairs {
                    if slice.starts_with(tag_open) {
                        match stack.last() {
                            Some((prev_style, _)) => {
                                sections.push((prev_style.to_owned(), section_start, i));
                                stack.push((inner_style(prev_style, style), tag_close));
                            }
                            None => {
                                sections.push((default_style.to_owned(), section_start, i));
                                stack.push((style.to_owned(), tag_close));
                            }
                        }
                        section_start = i + tag_open.len();
                    } else if slice.starts_with(tag_close) {
                        println!("WARNING: invalid closing token \"{}\" at {}", tag_close, i);
                    }
                }
                false
            }
        };
        if found_closing {
            stack.pop();
        }
    }
    if !stack.is_empty() {
        println!("WARNING: sections without closing tags");
    }
    if section_start < text.len() {
        sections.push((default_style, section_start, text.len()))
    }

    let mut result = String::new();

    for (style, a, b) in sections {
        if a == b { continue }
        let styled_text = style.paint(&text[a..b]).to_string();
        // println!("{:2}..{:2} | \"{}\" ({:?})", a, b, &styled_text.replace('\n', "\\n"), style);
        result += &styled_text;
    }

    result
}

fn inner_style(outer: &Style, inner: &Style) -> Style {
    let mut style = inner.to_owned();
    if style.background.is_none() {
        style.background = outer.background;
    }
    style
}

#[test]
fn asd() {
    //let original = "aa`bb[[cc<<dd>>ee]]f`FF`f{{gg}}hh`ii";
    let original = "aa\n\n`bb\ncccc\nddd\n`\n\n\neee";
    println!("{}", original);
    let styled = format_cg(original, &OutputStyle::default());
    println!("{}", styled);
}

/// Replaces spaces with "•" and newlines with "⏎" and paints them with
/// `ws_style`. Other characters are painted with `style`.
pub fn show_whitespace(text: &str, style: &Style, ws_style: &Style) -> String {
    let newl = format!("{}\n", ws_style.paint("⏎"));
    let space = format!("{}", ws_style.paint("•"));
    let mut result = String::new();
    let mut buf = String::new();
    for ch in text.chars() {
        match ch {
            '\n' => {
                result += &style.paint(&buf);
                result += &newl;
                buf.clear();
            }
            ' ' => {
                result += &style.paint(&buf);
                result += &space;
                buf.clear();
            }
            c => {
                buf.push(c)
            }
        }
    }
    result += &style.paint(&buf);
    result
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn trim_spaces_with_format() {
        let text = "hello  world";

        assert_eq!(format_cg(text, &OutputStyle::default()), "hello world");
    }

    #[test]
    fn does_not_trim_spaces_in_monospace() {
        let text = "`{\n    let x = 5;\n}`";

        assert!(format_cg(text, &OutputStyle::default()).contains("{\n    let x = 5;\n}"));
    }

    #[test]
    fn format_monospace() {
        let text = "To create a new variable use `let x = 5`";
        let formatted_text = format_cg(text, &OutputStyle::default());

        assert!(!formatted_text.contains("`"));
    }

    #[test]
    fn format_monospace_adds_newline_if_there_is_none() {
        let text = "I have `no whitespace`";
        let formatted_text = format_cg(text, &OutputStyle::default());

        assert!(formatted_text.contains("\n"));
    }

    #[test]
    fn format_monospace_trims_trailing_spaces() {
        let text = "I have `no whitespace`        and more text";
        let formatted_text = format_cg(text, &OutputStyle::default());

        assert!(!formatted_text.contains("\n "));
    }

    #[test]
    fn format_monospace_does_not_add_additional_newlines() {
        let text = "I have \n\n`lots of whitespace`";
        let formatted_text = format_cg(text, &OutputStyle::default());

        assert!(!formatted_text.contains("\n\n\n"));
    }

    #[test]
    fn format_nested() {
        let text = "<<Next [[N]] {{3}} lines:>>";
        let ostyle = &OutputStyle::default();
        let formatted_text = format_cg(text, ostyle);
        let expected = vec![
            format_cg("<<Next >>", ostyle),
            format_cg("[[N]]", ostyle),
            format_cg("<< >>", ostyle),
            format_cg("{{3}}", ostyle),
            format_cg("<< lines:>>", ostyle),
        ]
        .join("");

        assert_eq!(formatted_text, expected);
    }

}
