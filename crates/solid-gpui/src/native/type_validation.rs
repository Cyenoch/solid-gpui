/// Detect unsupported value types, excluding documentation, string literals and field names.
pub(super) fn has_unbounded_type(declaration: &str) -> bool {
    let mut characters = declaration.char_indices().peekable();
    let mut field_start = false;
    while let Some((start, character)) = characters.next() {
        match character {
            '\'' | '"' | '`' => {
                while let Some((_, next)) = characters.next() {
                    if next == '\\' {
                        characters.next();
                    } else if next == character {
                        break;
                    }
                }
                field_start = false;
            }
            '/' if characters.peek().is_some_and(|(_, next)| *next == '*') => {
                characters.next();
                let mut star = false;
                for (_, next) in characters.by_ref() {
                    if star && next == '/' {
                        break;
                    }
                    star = next == '*';
                }
            }
            '/' if characters.peek().is_some_and(|(_, next)| *next == '/') => {
                for (_, next) in characters.by_ref() {
                    if next == '\n' || next == '\r' {
                        break;
                    }
                }
            }
            value if identifier(value) => {
                while characters.peek().is_some_and(|(_, next)| identifier(*next)) {
                    characters.next();
                }
                let end = characters
                    .peek()
                    .map_or(declaration.len(), |(index, _)| *index);
                if matches!(&declaration[start..end], "any" | "bigint") {
                    let following = declaration[end..].trim_start();
                    let following = following
                        .strip_prefix('?')
                        .unwrap_or(following)
                        .trim_start();
                    let property = field_start && following.starts_with(':');
                    if !property {
                        return true;
                    }
                }
                field_start = &declaration[start..end] == "readonly";
            }
            value if !value.is_whitespace() => field_start = matches!(value, '{' | ';' | ','),
            _ => {}
        }
    }
    false
}

fn identifier(character: char) -> bool {
    character.is_alphanumeric() || matches!(character, '_' | '$')
}

#[cfg(test)]
mod tests {
    use super::has_unbounded_type;

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
