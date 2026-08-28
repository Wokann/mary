use std::collections::HashSet;

use thiserror::Error;

use super::{preprocess, Options, PreprocessError, ScriptSlot, ScriptTable, ScriptTableError};

#[derive(Clone, Debug)]
enum SymbolSlot {
    Empty,
    Script { name: String, texts: Vec<String> },
}

#[derive(Clone, Debug, Default)]
pub struct ScriptSymbols {
    slots: Vec<SymbolSlot>,
}

impl ScriptSymbols {
    pub fn script_name(&self, id: usize) -> Option<&str> {
        match self.slots.get(id) {
            Some(SymbolSlot::Script { name, .. }) => Some(name),
            _ => None,
        }
    }

    pub fn names(&self, id: usize, count: usize) -> Vec<Option<String>> {
        let names = match self.slots.get(id) {
            Some(SymbolSlot::Script { texts, .. }) => texts.as_slice(),
            _ => &[],
        };
        (0..count)
            .map(|text_id| names.get(text_id).cloned())
            .collect()
    }

    pub fn text_count(&self, id: usize) -> usize {
        match self.slots.get(id) {
            Some(SymbolSlot::Script { texts, .. }) => texts.len(),
            _ => 0,
        }
    }

    pub fn script_table(&self) -> Result<ScriptTable, ScriptTableError> {
        ScriptTable::from_slots(
            self.slots
                .iter()
                .map(|slot| match slot {
                    SymbolSlot::Empty => ScriptSlot::Empty,
                    SymbolSlot::Script { name, .. } => ScriptSlot::Script(name.clone()),
                })
                .collect(),
        )
    }
}

#[derive(Debug, Error)]
pub enum TextNameTableError {
    #[error(transparent)]
    Preprocess(#[from] PreprocessError),
    #[error("line {line}, column {column}: {message}")]
    Syntax {
        line: usize,
        column: usize,
        message: String,
    },
}

pub fn parse_text_name_table(
    source: &str,
    options: &Options,
) -> Result<ScriptSymbols, TextNameTableError> {
    let source = preprocess(source, options)?;
    let tokens = lex(&source)?;
    let mut at = 0;
    word(&tokens, &mut at, "mary_script_symbols")?;
    punct(&tokens, &mut at, Kind::Lb, "expected '{'")?;
    let mut slots = Vec::new();
    let mut script_names = HashSet::new();
    while !eat(&tokens, &mut at, &Kind::Rb) {
        if peek_word(&tokens, at, "NULL") {
            at += 1;
            slots.push(SymbolSlot::Empty);
        } else {
            let name = ident(&tokens, &mut at)?;
            if !script_names.insert(name.clone()) {
                return error(
                    &tokens,
                    at.saturating_sub(1),
                    &format!("script '{name}' occurs more than once"),
                );
            }
            punct(&tokens, &mut at, Kind::Lb, "expected '{' after script name")?;
            let mut texts = Vec::new();
            let mut text_names = HashSet::new();
            while !eat(&tokens, &mut at, &Kind::Rb) {
                let text = ident(&tokens, &mut at)?;
                if !text_names.insert(text.clone()) {
                    return error(
                        &tokens,
                        at.saturating_sub(1),
                        &format!("text '{text}' occurs more than once in this script"),
                    );
                }
                texts.push(text);
                if !eat(&tokens, &mut at, &Kind::Comma) && !peek(&tokens, at, &Kind::Rb) {
                    return error(&tokens, at, "expected ',' or '}' after text symbol");
                }
            }
            slots.push(SymbolSlot::Script { name, texts });
        }
        if !eat(&tokens, &mut at, &Kind::Comma) && !peek(&tokens, at, &Kind::Rb) {
            return error(&tokens, at, "expected ',' or '}' after script symbol");
        }
    }
    eat(&tokens, &mut at, &Kind::Semi);
    if at != tokens.len() {
        return error(&tokens, at, "unexpected text after symbol table");
    }
    Ok(ScriptSymbols { slots })
}

#[derive(Clone, Debug, PartialEq, Eq)]
enum Kind {
    Id(String),
    Lb,
    Rb,
    Comma,
    Semi,
}
#[derive(Clone, Debug)]
struct Token {
    kind: Kind,
    line: usize,
    column: usize,
}

fn lex(source: &str) -> Result<Vec<Token>, TextNameTableError> {
    let bytes = source.as_bytes();
    let (mut i, mut line, mut column) = (0, 1, 1);
    let mut out = Vec::new();
    while i < bytes.len() {
        if bytes[i] == b'\n' {
            i += 1;
            line += 1;
            column = 1;
            continue;
        }
        if bytes[i].is_ascii_whitespace() {
            i += 1;
            column += 1;
            continue;
        }
        if bytes[i] == b'/' && bytes.get(i + 1) == Some(&b'/') {
            while i < bytes.len() && bytes[i] != b'\n' {
                i += 1;
                column += 1;
            }
            continue;
        }
        let start = column;
        let kind = if bytes[i] == b'_' || bytes[i].is_ascii_alphabetic() {
            let begin = i;
            while i < bytes.len() && (bytes[i] == b'_' || bytes[i].is_ascii_alphanumeric()) {
                i += 1;
                column += 1;
            }
            Kind::Id(source[begin..i].into())
        } else {
            let byte = bytes[i];
            i += 1;
            column += 1;
            match byte {
                b'{' => Kind::Lb,
                b'}' => Kind::Rb,
                b',' => Kind::Comma,
                b';' => Kind::Semi,
                _ => {
                    return Err(TextNameTableError::Syntax {
                        line,
                        column: start,
                        message: format!("unexpected character '{}'", byte as char),
                    })
                }
            }
        };
        out.push(Token {
            kind,
            line,
            column: start,
        });
    }
    Ok(out)
}

fn word(tokens: &[Token], at: &mut usize, expected: &str) -> Result<(), TextNameTableError> {
    if peek_word(tokens, *at, expected) {
        *at += 1;
        Ok(())
    } else {
        error(tokens, *at, &format!("expected '{expected}'"))
    }
}
fn peek_word(tokens: &[Token], at: usize, expected: &str) -> bool {
    matches!(tokens.get(at).map(|t| &t.kind), Some(Kind::Id(value)) if value == expected)
}
fn ident(tokens: &[Token], at: &mut usize) -> Result<String, TextNameTableError> {
    match tokens.get(*at).map(|t| &t.kind) {
        Some(Kind::Id(value)) => {
            let value = value.clone();
            *at += 1;
            Ok(value)
        }
        _ => error(tokens, *at, "expected identifier"),
    }
}
fn punct(
    tokens: &[Token],
    at: &mut usize,
    expected: Kind,
    message: &str,
) -> Result<(), TextNameTableError> {
    if eat(tokens, at, &expected) {
        Ok(())
    } else {
        error(tokens, *at, message)
    }
}
fn eat(tokens: &[Token], at: &mut usize, expected: &Kind) -> bool {
    if peek(tokens, *at, expected) {
        *at += 1;
        true
    } else {
        false
    }
}
fn peek(tokens: &[Token], at: usize, expected: &Kind) -> bool {
    tokens.get(at).is_some_and(|token| &token.kind == expected)
}
fn error<T>(tokens: &[Token], at: usize, message: &str) -> Result<T, TextNameTableError> {
    let (line, column) = tokens.get(at).map_or_else(
        || tokens.last().map_or((1, 1), |t| (t.line, t.column + 1)),
        |t| (t.line, t.column),
    );
    Err(TextNameTableError::Syntax {
        line,
        column,
        message: message.into(),
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn preprocessing_controls_script_and_text_ids_by_order() {
        let source = r#"
mary_script_symbols {
    NULL,
    Shared { CommonText,
#if defined(MARY_MFOMT)
        GirlText,
#endif
    },
#if defined(MARY_MFOMT)
    GirlOnly { GirlOnlyText, },
#endif
    After { },
};
"#;
        let boy = parse_text_name_table(source, &Options::default()).unwrap();
        let girl =
            parse_text_name_table(source, &Options::default().define("MARY_MFOMT_US").unwrap())
                .unwrap();
        assert_eq!(boy.script_name(2), Some("After"));
        assert_eq!(girl.script_name(2), Some("GirlOnly"));
        assert_eq!(girl.script_name(3), Some("After"));
        assert_eq!(boy.names(1, 2), vec![Some("CommonText".into()), None]);
        assert_eq!(
            girl.names(1, 2),
            vec![Some("CommonText".into()), Some("GirlText".into())]
        );
    }

    #[test]
    fn duplicate_symbols_and_broken_layout_are_rejected() {
        let duplicate_script = "mary_script_symbols { Same {}, Same {}, };";
        assert!(parse_text_name_table(duplicate_script, &Options::default())
            .unwrap_err()
            .to_string()
            .contains("more than once"));

        let duplicate_text = "mary_script_symbols { Script { Text, Text, }, };";
        assert!(parse_text_name_table(duplicate_text, &Options::default())
            .unwrap_err()
            .to_string()
            .contains("more than once"));

        assert!(parse_text_name_table(
            "mary_script_symbols { Script { Text, }",
            &Options::default(),
        )
        .is_err());
    }
}
