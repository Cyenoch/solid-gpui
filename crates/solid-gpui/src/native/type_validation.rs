/// Inspect generated type syntax without treating documentation or literal text as types.
pub(super) fn has_unbounded_type(declaration: &str) -> bool {
    Scanner { rest: declaration }.code(false, false)
}

#[derive(Clone)]
struct Scanner<'a> {
    rest: &'a str,
}

impl Scanner<'_> {
    fn take(&mut self) -> Option<char> {
        let character = self.rest.chars().next()?;
        self.rest = &self.rest[character.len_utf8()..];
        Some(character)
    }

    fn trivia(&mut self) {
        loop {
            self.rest = self.rest.trim_start();
            if let Some(comment) = self.rest.strip_prefix("/*") {
                self.rest = comment.find("*/").map_or("", |end| &comment[end + 2..]);
            } else if let Some(comment) = self.rest.strip_prefix("//") {
                self.rest = comment
                    .find(['\n', '\r', '\u{2028}', '\u{2029}'])
                    .map_or("", |end| &comment[end..]);
            } else {
                break;
            }
        }
    }

    fn quoted(&mut self, quote: char) {
        while let Some(character) = self.take() {
            if character == '\\' {
                self.take();
            } else if character == quote {
                break;
            }
        }
    }

    fn property_follows(&self) -> bool {
        let mut following = self.clone();
        following.trivia();
        if following.rest.starts_with('?') {
            following.take();
            following.trivia();
        }
        following.rest.starts_with([':', '('])
    }

    fn template(&mut self) -> bool {
        while let Some(character) = self.take() {
            match character {
                '\\' => {
                    self.take();
                }
                '`' => break,
                '$' if self.rest.starts_with('{') => {
                    self.take();
                    // An interpolation starts a type, not an object member name.
                    if self.code(false, true) {
                        return true;
                    }
                }
                _ => {}
            }
        }
        false
    }

    fn code(&mut self, mut field_start: bool, end_brace: bool) -> bool {
        loop {
            self.trivia();
            let start = self.rest;
            let Some(character) = self.take() else {
                return false;
            };
            match character {
                '\'' | '"' => {
                    self.quoted(character);
                    field_start = false;
                }
                '`' => {
                    if self.template() {
                        return true;
                    }
                    field_start = false;
                }
                '{' => {
                    if self.code(true, true) {
                        return true;
                    }
                    field_start = false;
                }
                '}' if end_brace => return false,
                value if identifier(value) => {
                    while self.rest.chars().next().is_some_and(identifier) {
                        self.take();
                    }
                    let word = &start[..start.len() - self.rest.len()];
                    if matches!(word, "any" | "bigint") && !(field_start && self.property_follows())
                    {
                        return true;
                    }
                    field_start = field_start && word == "readonly";
                }
                value => field_start = matches!(value, ';' | ',' | '(' | '['),
            }
        }
    }
}

fn identifier(character: char) -> bool {
    character.is_alphanumeric() || matches!(character, '_' | '$')
}

#[cfg(test)]
mod tests {
    use super::has_unbounded_type;

    #[test]
    fn property_names_and_template_text_are_not_value_types() {
        for declaration in [
            "type State = { any /* documentation */ ?: string; readonly bigint: number; }",
            "type State = { any: { bigint: 'any' }; label: `any bigint`; }",
            r"type State = `escaped \` any \${bigint}`;",
            "type State = `any ${'bigint' | `any ${string}`} bigint`;",
            "type State = { value: `text ${ { any: string } }`; bigint: number; }",
        ] {
            assert!(!has_unbounded_type(declaration), "{declaration}");
        }
    }

    #[test]
    fn template_interpolations_and_nested_types_remain_validated() {
        for declaration in [
            "type State = `prefix ${any}`;",
            "type State = `prefix ${Array<bigint>}`;",
            "type State = `outer ${`inner ${any}`} tail`;",
            "type State = `prefix ${ { any: bigint } }`;",
            "type State = `prefix ${string}` | any;",
            "type State = `prefix ${'/*'} ${bigint}`;",
            "type State = { values: [string, any]; }",
            "type State = { values: Array<string | { nested: bigint[] }>; }",
            "type State<T> = T extends string ? any /* note */ : number;",
            "type State = { any: any; }",
        ] {
            assert!(has_unbounded_type(declaration), "{declaration}");
        }
    }

    #[test]
    fn documentation_and_serializable_literal_values_are_not_unbounded_types() {
        assert!(!has_unbounded_type(
            "/** The pending request, if any. */ type State = { value: string; }"
        ));
        assert!(!has_unbounded_type(
            "type State = { any?: string; bigint: boolean; label: 'any' | \"bigint\"; }"
        ));
        assert!(!has_unbounded_type(
            "// any is forbidden as a type\ntype State = string;"
        ));
    }

    #[test]
    fn comment_or_literal_delimiters_cannot_hide_an_unbounded_field() {
        assert!(has_unbounded_type(
            "type State = { label: '/*'; value: bigint; }"
        ));
        assert!(has_unbounded_type(
            "type State = { label: '//'; value: any; }"
        ));
        assert!(has_unbounded_type(
            "type State = { label: '\\'/*'; value: any; }"
        ));
        assert!(has_unbounded_type(
            "/* any */ type State = { values: Array<bigint>; }"
        ));
        assert!(has_unbounded_type(
            "type State = { value: any\u{2003}| null; }"
        ));
        assert!(has_unbounded_type(
            "type State<T> = T extends string ? any : never;"
        ));
        assert!(has_unbounded_type(
            "type State<T> = T extends string ? bigint : number;"
        ));
    }

    #[test]
    fn names_and_bounded_types_keep_their_meaning() {
        assert!(!has_unbounded_type(
            "type Company = { many: number; anything: unknown; }"
        ));
        assert!(has_unbounded_type("bigint"));
        assert!(has_unbounded_type("any | null"));
    }
}
