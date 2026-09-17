//! Best-effort HTML → plain text for MDX entries.

pub fn html_to_plain(input: &str) -> String {
    let mut out = String::with_capacity(input.len());
    let mut rest = input;

    while !rest.is_empty() {
        if let Some(stripped) = strip_prefix_ci(rest, "<br") {
            out.push('\n');
            rest = skip_to_tag_end(stripped);
            continue;
        }
        if let Some(stripped) = strip_block_end(rest) {
            out.push('\n');
            rest = stripped;
            continue;
        }
        if rest.starts_with('<') {
            rest = skip_to_tag_end(&rest[1..]);
            continue;
        }
        let mut chars = rest.chars();
        if let Some(ch) = chars.next() {
            out.push(ch);
            rest = chars.as_str();
        } else {
            break;
        }
    }

    collapse_blank_lines(&decode_basic_entities(&out))
}

fn strip_prefix_ci<'a>(input: &'a str, prefix: &str) -> Option<&'a str> {
    let input_bytes = input.as_bytes();
    let prefix_bytes = prefix.as_bytes();
    if input_bytes.len() < prefix_bytes.len() {
        return None;
    }
    if input_bytes[..prefix_bytes.len()]
        .iter()
        .zip(prefix_bytes.iter())
        .all(|(a, b)| a.to_ascii_lowercase() == *b)
    {
        Some(&input[prefix.len()..])
    } else {
        None
    }
}

fn strip_block_end(input: &str) -> Option<&str> {
    for tag in ["</p", "</div", "</li", "</tr", "</h1", "</h2", "</h3", "</h4"] {
        if let Some(rest) = strip_prefix_ci(input, tag) {
            return Some(skip_to_tag_end(rest));
        }
    }
    None
}

fn skip_to_tag_end(input: &str) -> &str {
    match input.find('>') {
        Some(idx) => &input[idx + 1..],
        None => "",
    }
}

fn decode_basic_entities(input: &str) -> String {
    input
        .replace("&nbsp;", " ")
        .replace("&amp;", "&")
        .replace("&lt;", "<")
        .replace("&gt;", ">")
        .replace("&quot;", "\"")
        .replace("&#39;", "'")
        .replace("&apos;", "'")
}

fn collapse_blank_lines(input: &str) -> String {
    let mut lines: Vec<&str> = Vec::new();
    let mut prev_blank = true;
    for line in input.lines() {
        let trimmed = line.trim();
        if trimmed.is_empty() {
            if !prev_blank {
                lines.push("");
            }
            prev_blank = true;
        } else {
            lines.push(trimmed);
            prev_blank = false;
        }
    }
    while lines.last().is_some_and(|l| l.is_empty()) {
        lines.pop();
    }
    lines.join("\n")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn strips_tags_and_breaks() {
        let plain = html_to_plain("<p>hello<br/>world</p><div>next</div>");
        assert!(plain.contains("hello"));
        assert!(plain.contains("world"));
        assert!(plain.contains("next"));
        assert!(!plain.contains('<'));
    }

    #[test]
    fn keeps_unicode() {
        assert_eq!(html_to_plain("<b>你好</b>"), "你好");
    }

    #[test]
    fn decodes_entities() {
        assert_eq!(html_to_plain("A &amp; B"), "A & B");
    }
}
