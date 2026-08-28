use std::collections::{HashMap, HashSet};

use thiserror::Error;

use crate::{ast::ConstVal, const_scope::ConstScope, ir::IntValue};

use super::{preprocess, Options, PreprocessError};

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum ScriptSlot {
    Empty,
    Script(String),
}

#[derive(Clone, Debug)]
pub struct ScriptTable {
    slots: Vec<ScriptSlot>,
    ids: HashMap<String, IntValue>,
}

impl ScriptTable {
    pub fn from_slots(slots: Vec<ScriptSlot>) -> Result<Self, ScriptTableError> {
        let mut ids = HashMap::new();
        for (id, slot) in slots.iter().enumerate() {
            if let ScriptSlot::Script(name) = slot {
                if ids.insert(name.clone(), id as IntValue).is_some() {
                    return Err(ScriptTableError::Duplicate {
                        line: 1,
                        column: 1,
                        name: name.clone(),
                    });
                }
            }
        }
        Ok(Self { slots, ids })
    }

    pub fn slots(&self) -> &[ScriptSlot] {
        &self.slots
    }

    pub fn id(&self, name: &str) -> Option<IntValue> {
        self.ids.get(name).copied()
    }

    pub fn name(&self, id: usize) -> Option<&str> {
        match self.slots.get(id) {
            Some(ScriptSlot::Script(name)) => Some(name),
            _ => None,
        }
    }

    pub fn add_constants(&self, scope: &mut ConstScope) {
        for (name, id) in &self.ids {
            scope.add_const(name.clone(), ConstVal::Int(*id));
        }
    }
}

#[derive(Debug, Error)]
pub enum ScriptTableError {
    #[error(transparent)]
    Preprocess(#[from] PreprocessError),
    #[error("line {line}, column {column}: {message}")]
    Syntax {
        line: usize,
        column: usize,
        message: String,
    },
    #[error("line {line}, column {column}: script name '{name}' occurs more than once")]
    Duplicate {
        line: usize,
        column: usize,
        name: String,
    },
    #[error("script table has more slots than the VM ID space")]
    Overflow,
}

pub fn parse_script_table(
    source: &str,
    options: &Options,
) -> Result<ScriptTable, ScriptTableError> {
    let source = preprocess(source, options)?;
    let tokens = lex(&source)?;
    let mut cursor = 0;
    expect(
        &tokens,
        &mut cursor,
        TokenKind::Table,
        "expected 'mary_script_table'",
    )?;
    expect(&tokens, &mut cursor, TokenKind::LBrace, "expected '{'")?;
    let mut slots = Vec::new();
    let mut ids = HashMap::new();
    let mut names = HashSet::new();
    loop {
        let token = tokens
            .get(cursor)
            .ok_or_else(|| syntax_at_end(&tokens, "unterminated mary_script_table"))?;
        match &token.kind {
            TokenKind::RBrace => {
                cursor += 1;
                break;
            }
            TokenKind::Null => {
                slots.push(ScriptSlot::Empty);
                cursor += 1;
            }
            TokenKind::Ident(name) => {
                if !names.insert(name.clone()) {
                    return Err(ScriptTableError::Duplicate {
                        line: token.line,
                        column: token.column,
                        name: name.clone(),
                    });
                }
                let id = IntValue::try_from(slots.len()).map_err(|_| ScriptTableError::Overflow)?;
                ids.insert(name.clone(), id);
                slots.push(ScriptSlot::Script(name.clone()));
                cursor += 1;
            }
            _ => return syntax(token, "expected a script name, NULL, or '}'"),
        }
        let separator = tokens
            .get(cursor)
            .ok_or_else(|| syntax_at_end(&tokens, "unterminated mary_script_table"))?;
        match separator.kind {
            TokenKind::Comma => cursor += 1,
            TokenKind::RBrace => {}
            _ => return syntax(separator, "expected ',' or '}' after script slot"),
        }
    }
    if tokens
        .get(cursor)
        .is_some_and(|t| t.kind == TokenKind::Semi)
    {
        cursor += 1;
    }
    if let Some(token) = tokens.get(cursor) {
        return syntax(token, "unexpected text after mary_script_table");
    }
    Ok(ScriptTable { slots, ids })
}

#[derive(Clone, Debug, PartialEq, Eq)]
enum TokenKind {
    Table,
    Null,
    Ident(String),
    LBrace,
    RBrace,
    Comma,
    Semi,
}

#[derive(Clone, Debug)]
struct Token {
    kind: TokenKind,
    line: usize,
    column: usize,
}

fn lex(source: &str) -> Result<Vec<Token>, ScriptTableError> {
    let bytes = source.as_bytes();
    let (mut i, mut line, mut column) = (0, 1, 1);
    let mut tokens = Vec::new();
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
        if bytes[i] == b'/' && bytes.get(i + 1) == Some(&b'*') {
            let start = (line, column);
            i += 2;
            column += 2;
            let mut closed = false;
            while i < bytes.len() {
                if bytes[i] == b'*' && bytes.get(i + 1) == Some(&b'/') {
                    i += 2;
                    column += 2;
                    closed = true;
                    break;
                }
                if bytes[i] == b'\n' {
                    i += 1;
                    line += 1;
                    column = 1;
                } else {
                    i += 1;
                    column += 1;
                }
            }
            if !closed {
                return Err(ScriptTableError::Syntax {
                    line: start.0,
                    column: start.1,
                    message: "unterminated block comment".into(),
                });
            }
            continue;
        }
        let start_column = column;
        let kind = match bytes[i] {
            b'{' => {
                i += 1;
                column += 1;
                TokenKind::LBrace
            }
            b'}' => {
                i += 1;
                column += 1;
                TokenKind::RBrace
            }
            b',' => {
                i += 1;
                column += 1;
                TokenKind::Comma
            }
            b';' => {
                i += 1;
                column += 1;
                TokenKind::Semi
            }
            b if b == b'_' || b.is_ascii_alphabetic() => {
                let start = i;
                while i < bytes.len() && (bytes[i] == b'_' || bytes[i].is_ascii_alphanumeric()) {
                    i += 1;
                    column += 1;
                }
                match &source[start..i] {
                    "mary_script_table" => TokenKind::Table,
                    "NULL" => TokenKind::Null,
                    name => TokenKind::Ident(name.into()),
                }
            }
            other => {
                return Err(ScriptTableError::Syntax {
                    line,
                    column,
                    message: format!("unexpected character '{}'", other as char),
                })
            }
        };
        tokens.push(Token {
            kind,
            line,
            column: start_column,
        });
    }
    Ok(tokens)
}

fn expect(
    tokens: &[Token],
    cursor: &mut usize,
    expected: TokenKind,
    message: &str,
) -> Result<(), ScriptTableError> {
    match tokens.get(*cursor) {
        Some(token) if token.kind == expected => {
            *cursor += 1;
            Ok(())
        }
        Some(token) => syntax(token, message),
        None => Err(syntax_at_end(tokens, message)),
    }
}

fn syntax<T>(token: &Token, message: &str) -> Result<T, ScriptTableError> {
    Err(ScriptTableError::Syntax {
        line: token.line,
        column: token.column,
        message: message.into(),
    })
}

fn syntax_at_end(tokens: &[Token], message: &str) -> ScriptTableError {
    let (line, column) = tokens.last().map_or((1, 1), |t| (t.line, t.column + 1));
    ScriptTableError::Syntax {
        line,
        column,
        message: message.into(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn conditional_slots_are_counted_after_preprocessing() {
        let source = r#"
mary_script_table {
    Opening,
#if defined(MARY_FOMT)
    FomtOnly,
#else
    NULL,
#endif
    Shared,
};
"#;
        let fomt =
            parse_script_table(source, &Options::default().define("MARY_FOMT").unwrap()).unwrap();
        let mfomt = parse_script_table(source, &Options::default()).unwrap();
        assert_eq!(fomt.id("FomtOnly"), Some(1));
        assert_eq!(fomt.id("Shared"), Some(2));
        assert_eq!(mfomt.id("FomtOnly"), None);
        assert_eq!(mfomt.id("Shared"), Some(2));
        assert_eq!(mfomt.slots()[1], ScriptSlot::Empty);
    }

    #[test]
    fn duplicate_names_are_rejected() {
        let error = parse_script_table("mary_script_table { Same, Same }", &Options::default())
            .unwrap_err();
        assert!(error.to_string().contains("more than once"));
    }
}
