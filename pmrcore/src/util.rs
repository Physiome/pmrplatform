use std::cmp::max;

// This is modified from ammonia::clean_text with a couple rules relaxed.
pub fn clean_text_keep_mark(src: &str) -> String {
    let mut ret_val = String::with_capacity(max(4, src.len()));
    let mut chars = src.char_indices();
    while let Some((idx, c)) = chars.next() {
        let replacement = match c {
            // this character, when confronted, will start a tag
            '<' => {
                if src[idx..].starts_with("<mark>") {
                    chars.nth(4);
                    "<mark>"
                } else if src[idx..].starts_with("</mark>") {
                    chars.nth(5);
                    "</mark>"
                } else {
                    "&lt;"
                }
            }
            // in an unquoted attribute, will end the attribute value
            '>' => "&gt;",
            // in an attribute surrounded by double quotes, this character will end the attribute value
            '\"' => "&quot;",
            // in an attribute surrounded by single quotes, this character will end the attribute value
            '\'' => "&apos;",
            // in HTML5, returns a bogus parse error in an unquoted attribute, while in SGML/HTML, it will end an attribute value surrounded by backquotes
            '`' => "&grave;",
            // in an unquoted attribute, this character will end the attribute
            // '/' => "&#47;",
            // starts an entity reference
            '&' => "&amp;",
            // if at the beginning of an unquoted attribute, will get ignored
            '=' => "&#61;",
            // will end an unquoted attribute
            // ' ' => "&#32;",
            '\t' => "&#9;",
            '\n' => "&#10;",
            '\x0c' => "&#12;",
            '\r' => "&#13;",
            // a spec-compliant browser will perform this replacement anyway, but the middleware might not
            '\0' => "&#65533;",
            // ALL OTHER CHARACTERS ARE PASSED THROUGH VERBATIM
            _ => {
                ret_val.push(c);
                continue;
            }
        };
        ret_val.push_str(replacement);
    }
    ret_val
}

#[test]
fn smoke_clean_text_skip_mark() {
    assert_eq!(clean_text_keep_mark("This is <b>bold</b>ed."), "This is &lt;b&gt;bold&lt;/b&gt;ed.");
    assert_eq!(clean_text_keep_mark("This is <mark>marked</mark> done."), "This is <mark>marked</mark> done.");
    assert_eq!(
        clean_text_keep_mark("\u{2026}Conc.<10, <mark>values</mark>\u{2026}"),
        "\u{2026}Conc.&lt;10, <mark>values</mark>\u{2026}",
    );
}
