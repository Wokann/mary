use std::collections::{HashMap, HashSet};

use thiserror::Error;

use crate::{
    const_scope::ConstScope,
    ir::{CallId, ValueType},
};

use super::{preprocess, Options, PreprocessError};

#[derive(Debug, Error)]
pub enum MaryCError {
    #[error(transparent)]
    Preprocess(#[from] PreprocessError),
    #[error("line {line}, column {column}: {message}")]
    Syntax {
        line: usize,
        column: usize,
        message: String,
    },
    #[error("callable id {id:#X} does not fit the VM id space")]
    CallableIdOverflow { id: usize },
    #[error("line {line}, column {column}: callable name '{name}' is declared more than once")]
    DuplicateCallable {
        line: usize,
        column: usize,
        name: String,
    },
    #[error("invalid callable type metadata: {message}")]
    InvalidTypeMetadata { message: String },
}

pub struct CallableTable {
    pub name: String,
    pub base: usize,
    pub next_id: usize,
    pub scope: ConstScope,
}

#[derive(Clone, Debug, PartialEq, Eq)]
enum TokenKind {
    Ident(String),
    Integer(i64),
    LParen,
    RParen,
    LBrace,
    RBrace,
    Comma,
    Semicolon,
    Star,
}

#[derive(Clone, Debug)]
struct Token {
    kind: TokenKind,
    line: usize,
    column: usize,
}

pub fn parse_callable_table(source: &str, options: &Options) -> Result<CallableTable, MaryCError> {
    parse_callable_table_with_scope(source, options, &ConstScope::new())
}

pub fn parse_callable_table_with_scope(
    source: &str,
    options: &Options,
    base_scope: &ConstScope,
) -> Result<CallableTable, MaryCError> {
    let source = preprocess(source, options)?;
    let tokens = lex(&source)?;
    Parser { tokens, cursor: 0 }.table(base_scope.clone())
}

struct Parser {
    tokens: Vec<Token>,
    cursor: usize,
}

impl Parser {
    fn table(mut self, mut scope: ConstScope) -> Result<CallableTable, MaryCError> {
        self.keyword("mary_callable_table")?;
        if self.peek(&TokenKind::LBrace) {
            return self.ordered_name_table(scope);
        }
        self.punct(TokenKind::LParen)?;
        let name = self.ident()?;
        self.punct(TokenKind::Comma)?;
        let base = self.integer()?;
        if base < 0 {
            return self.error("callable table base cannot be negative");
        }
        let base = base as usize;
        self.punct(TokenKind::RParen)?;
        self.punct(TokenKind::LBrace)?;
        let mut names = HashSet::new();
        let mut id = base;
        while !self.eat(&TokenKind::RBrace) {
            if self.at_end() {
                return self.error("unterminated mary_callable_table block");
            }
            if self.peek_ident("mary_unknown") {
                self.cursor += 1;
                self.punct(TokenKind::LParen)?;
                let _diagnostic_name = self.ident()?;
                self.punct(TokenKind::RParen)?;
                self.punct(TokenKind::Semicolon)?;
            } else if self.peek_ident("mary_reserved") {
                self.cursor += 1;
                self.punct(TokenKind::LParen)?;
                self.punct(TokenKind::RParen)?;
                self.punct(TokenKind::Semicolon)?;
            } else {
                let return_type = if self.peek_ident("int") {
                    self.cursor += 1;
                    Some(ValueType::Integer)
                } else if self.peek_ident("void") {
                    self.cursor += 1;
                    None
                } else {
                    let type_name = self.ident()?;
                    Some(ValueType::UserType(scope.add_or_get_user_type(type_name)))
                };
                let callable_name = self.ident()?;
                if !names.insert(callable_name.clone()) {
                    let token = &self.tokens[self.cursor.saturating_sub(1)];
                    return Err(MaryCError::DuplicateCallable {
                        line: token.line,
                        column: token.column,
                        name: callable_name,
                    });
                }
                self.punct(TokenKind::LParen)?;
                let params = self.params(&mut scope)?;
                self.punct(TokenKind::RParen)?;
                self.punct(TokenKind::Semicolon)?;
                if id > u32::MAX as usize {
                    return Err(MaryCError::CallableIdOverflow { id });
                }
                let call_id = CallId(id);
                if let Some(return_type) = return_type {
                    scope.add_typed_func(callable_name, call_id, return_type, params);
                } else {
                    scope.add_proc(callable_name, call_id, params);
                }
            }
            id = id
                .checked_add(1)
                .ok_or(MaryCError::CallableIdOverflow { id })?;
        }
        self.eat(&TokenKind::Semicolon);
        if !self.at_end() {
            return self.error("unexpected text after callable table");
        }
        scope
            .validate_callable_type_metadata()
            .map_err(|message| MaryCError::InvalidTypeMetadata { message })?;
        Ok(CallableTable {
            name,
            base,
            next_id: id,
            scope,
        })
    }

    fn ordered_name_table(mut self, mut scope: ConstScope) -> Result<CallableTable, MaryCError> {
        self.punct(TokenKind::LBrace)?;
        let mut slots: Vec<Option<String>> = Vec::new();
        let mut names = HashSet::new();
        while !self.eat(&TokenKind::RBrace) {
            if self.at_end() {
                return self.error("unterminated mary_callable_table block");
            }
            let name = if self.peek_ident("NULL") {
                self.cursor += 1;
                None
            } else {
                let name = self.ident()?;
                if !names.insert(name.clone()) {
                    let token = &self.tokens[self.cursor.saturating_sub(1)];
                    return Err(MaryCError::DuplicateCallable {
                        line: token.line,
                        column: token.column,
                        name,
                    });
                }
                Some(name)
            };
            slots.push(name);
            if !self.eat(&TokenKind::Comma) && !self.peek(&TokenKind::RBrace) {
                return self.error("expected ',' or '}' after callable slot");
            }
        }
        self.eat(&TokenKind::Semicolon);

        let mut declarations = HashMap::new();
        while !self.at_end() {
            let return_type = if self.peek_ident("int") {
                self.cursor += 1;
                Some(ValueType::Integer)
            } else if self.peek_ident("void") {
                self.cursor += 1;
                None
            } else {
                let type_name = self.ident()?;
                Some(ValueType::UserType(scope.add_or_get_user_type(type_name)))
            };
            let name = self.ident()?;
            self.punct(TokenKind::LParen)?;
            let params = self.params(&mut scope)?;
            self.punct(TokenKind::RParen)?;
            self.punct(TokenKind::Semicolon)?;
            if declarations
                .insert(name.clone(), (return_type, params))
                .is_some()
            {
                let token = &self.tokens[self.cursor.saturating_sub(1)];
                return Err(MaryCError::DuplicateCallable {
                    line: token.line,
                    column: token.column,
                    name,
                });
            }
        }

        for (id, slot) in slots.iter().enumerate() {
            let Some(name) = slot else { continue };
            let Some((return_type, params)) = declarations.remove(name) else {
                return self.error(&format!(
                    "callable '{name}' has a table slot but no prototype"
                ));
            };
            let call_id = CallId(id);
            if let Some(return_type) = return_type {
                scope.add_typed_func(name.clone(), call_id, return_type, params);
            } else {
                scope.add_proc(name.clone(), call_id, params);
            }
        }
        // Prototypes for another target may remain in the shared header. They
        // do not enter the selected target's scope unless its ordered table
        // contains the same symbol.
        scope
            .validate_callable_type_metadata()
            .map_err(|message| MaryCError::InvalidTypeMetadata { message })?;
        Ok(CallableTable {
            name: "MARY_CALLABLES".into(),
            base: 0,
            next_id: slots.len(),
            scope,
        })
    }

    fn params(&mut self, scope: &mut ConstScope) -> Result<Vec<ValueType>, MaryCError> {
        if self.peek(&TokenKind::RParen) {
            return Ok(Vec::new());
        }
        if self.peek_ident("void") {
            self.cursor += 1;
            if !self.peek(&TokenKind::RParen) {
                return self.error("void must be the only parameter");
            }
            return Ok(Vec::new());
        }
        let mut result = Vec::new();
        loop {
            let ty = if self.peek_ident("int") {
                self.cursor += 1;
                ValueType::Integer
            } else if self.peek_ident("const") {
                self.cursor += 1;
                self.keyword("char")?;
                self.punct(TokenKind::Star)?;
                ValueType::String
            } else if matches!(
                self.tokens.get(self.cursor).map(|token| &token.kind),
                Some(TokenKind::Ident(_))
            ) {
                let name = self.ident()?;
                ValueType::UserType(scope.add_or_get_user_type(name))
            } else {
                return self.error("parameters must use int, const char *, or a named ID type");
            };
            self.ident()?;
            result.push(ty);
            if !self.eat(&TokenKind::Comma) {
                break;
            }
        }
        Ok(result)
    }

    fn keyword(&mut self, expected: &str) -> Result<(), MaryCError> {
        if self.peek_ident(expected) {
            self.cursor += 1;
            Ok(())
        } else {
            self.error(&format!("expected '{expected}'"))
        }
    }
    fn ident(&mut self) -> Result<String, MaryCError> {
        match self.tokens.get(self.cursor).map(|t| &t.kind) {
            Some(TokenKind::Ident(v)) => {
                let v = v.clone();
                self.cursor += 1;
                Ok(v)
            }
            _ => self.error("expected identifier"),
        }
    }
    fn integer(&mut self) -> Result<i64, MaryCError> {
        match self.tokens.get(self.cursor).map(|t| &t.kind) {
            Some(TokenKind::Integer(v)) => {
                let v = *v;
                self.cursor += 1;
                Ok(v)
            }
            _ => self.error("expected integer"),
        }
    }
    fn punct(&mut self, expected: TokenKind) -> Result<(), MaryCError> {
        if self.eat(&expected) {
            Ok(())
        } else {
            self.error(&format!("expected {expected:?}"))
        }
    }
    fn eat(&mut self, expected: &TokenKind) -> bool {
        if self.peek(expected) {
            self.cursor += 1;
            true
        } else {
            false
        }
    }
    fn peek(&self, expected: &TokenKind) -> bool {
        self.tokens
            .get(self.cursor)
            .is_some_and(|t| &t.kind == expected)
    }
    fn peek_ident(&self, expected: &str) -> bool {
        matches!(self.tokens.get(self.cursor).map(|t| &t.kind), Some(TokenKind::Ident(v)) if v == expected)
    }
    fn at_end(&self) -> bool {
        self.cursor == self.tokens.len()
    }
    fn error<T>(&self, message: &str) -> Result<T, MaryCError> {
        let (line, column) = self
            .tokens
            .get(self.cursor)
            .map(|t| (t.line, t.column))
            .unwrap_or_else(|| {
                self.tokens
                    .last()
                    .map_or((1, 1), |t| (t.line, t.column + 1))
            });
        Err(MaryCError::Syntax {
            line,
            column,
            message: message.to_owned(),
        })
    }
}

fn lex(source: &str) -> Result<Vec<Token>, MaryCError> {
    let bytes = source.as_bytes();
    let (mut i, mut line, mut column) = (0, 1, 1);
    let mut out = Vec::new();
    while i < bytes.len() {
        let b = bytes[i];
        if b == b'\n' {
            i += 1;
            line += 1;
            column = 1;
            continue;
        }
        if b.is_ascii_whitespace() {
            i += 1;
            column += 1;
            continue;
        }
        if b == b'/' && bytes.get(i + 1) == Some(&b'/') {
            while i < bytes.len() && bytes[i] != b'\n' {
                i += 1;
                column += 1;
            }
            continue;
        }
        if b == b'/' && bytes.get(i + 1) == Some(&b'*') {
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
                return Err(MaryCError::Syntax {
                    line: start.0,
                    column: start.1,
                    message: "unterminated block comment".into(),
                });
            }
            continue;
        }
        let start_column = column;
        let kind = if b == b'_' || b.is_ascii_alphabetic() {
            let start = i;
            while i < bytes.len() && (bytes[i] == b'_' || bytes[i].is_ascii_alphanumeric()) {
                i += 1;
                column += 1;
            }
            TokenKind::Ident(source[start..i].to_owned())
        } else if b.is_ascii_digit() {
            let start = i;
            i += 1;
            column += 1;
            while i < bytes.len()
                && (bytes[i].is_ascii_hexdigit() || bytes[i] == b'x' || bytes[i] == b'X')
            {
                i += 1;
                column += 1;
            }
            let text = &source[start..i];
            let value =
                if let Some(hex) = text.strip_prefix("0x").or_else(|| text.strip_prefix("0X")) {
                    i64::from_str_radix(hex, 16)
                } else {
                    text.parse()
                }
                .map_err(|_| MaryCError::Syntax {
                    line,
                    column: start_column,
                    message: format!("invalid integer '{text}'"),
                })?;
            TokenKind::Integer(value)
        } else {
            i += 1;
            column += 1;
            match b {
                b'(' => TokenKind::LParen,
                b')' => TokenKind::RParen,
                b'{' => TokenKind::LBrace,
                b'}' => TokenKind::RBrace,
                b',' => TokenKind::Comma,
                b';' => TokenKind::Semicolon,
                b'*' => TokenKind::Star,
                _ => {
                    return Err(MaryCError::Syntax {
                        line,
                        column: start_column,
                        message: format!("unexpected character '{}'", b as char),
                    })
                }
            }
        };
        out.push(Token {
            kind,
            line,
            column: start_column,
        });
    }
    Ok(out)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::ast::{NameAccess, NameRef};
    use crate::mary_c::constants::parse_constant_header;

    #[test]
    fn conditional_slots_shift_only_selected_target() {
        let source = r#"
mary_callable_table(FOMT_FAMILY, 0x10) {
    mary_unknown(Unk010);
    int Before(int value);
#ifdef TARGET_GIRL
    void GirlOnly(void);
#endif
    const char * InvalidReturn(void);
}
"#;
        // Verify diagnostics too: unsupported return type must never be silently accepted.
        assert!(parse_callable_table(source, &Options::default()).is_err());

        let source = source.replace(
            "    const char * InvalidReturn(void);\n",
            "    int After(const char * message);\n",
        );
        let boy = parse_callable_table(&source, &Options::default()).unwrap();
        let girl =
            parse_callable_table(&source, &Options::default().define("TARGET_GIRL").unwrap())
                .unwrap();
        assert!(matches!(
            boy.scope.lookup_name("Before"),
            Some(NameRef::Func(CallId(0x11)))
        ));
        assert!(matches!(
            boy.scope.lookup_name("After"),
            Some(NameRef::Func(CallId(0x12)))
        ));
        assert!(matches!(
            girl.scope.lookup_name("GirlOnly"),
            Some(NameRef::Proc(CallId(0x12)))
        ));
        assert!(matches!(
            girl.scope.lookup_name("After"),
            Some(NameRef::Func(CallId(0x13)))
        ));
    }

    #[test]
    fn family_tables_do_not_repeat_target_selection_boilerplate() {
        for (source, own_target, other_region, other_family) in [
            (
                include_str!("../../goodies/fomt_callables.mary.h"),
                "MARY_FOMT_US",
                "MARY_FOMT_JP",
                "MARY_MFOMT_US",
            ),
            (
                include_str!("../../goodies/mfomt_callables.mary.h"),
                "MARY_MFOMT_US",
                "MARY_MFOMT_JP",
                "MARY_FOMT_US",
            ),
        ] {
            let default = parse_callable_table(source, &Options::default()).unwrap();
            for options in [
                Options::default().define(own_target).unwrap(),
                Options::default()
                    .define(own_target)
                    .unwrap()
                    .define(other_region)
                    .unwrap(),
                Options::default().define(other_family).unwrap(),
            ] {
                let selected = parse_callable_table(source, &options).unwrap();
                assert_eq!(selected.next_id, default.next_id);
            }
            assert!(!source.contains("#error Select exactly one"));
            assert!(!source.contains("#define MARY_"));
        }
    }

    #[test]
    fn canonical_table_separates_order_from_prototypes() {
        let source = r#"
mary_callable_table {
    Common,
#if defined(MARY_MFOMT_US) || defined(MARY_MFOMT_JP)
    GirlOnly,
#else
    NULL,
#endif
    After,
};

void Common(void);
#if defined(MARY_MFOMT_US) || defined(MARY_MFOMT_JP)
int GirlOnly(int value);
#endif
void After(const char *message);
"#;
        let boy = parse_callable_table(source, &Options::default()).unwrap();
        let girl =
            parse_callable_table(source, &Options::default().define("MARY_MFOMT_US").unwrap())
                .unwrap();
        assert!(matches!(
            boy.scope.lookup_name("After"),
            Some(NameRef::Proc(CallId(2)))
        ));
        assert!(matches!(
            girl.scope.lookup_name("GirlOnly"),
            Some(NameRef::Func(CallId(1)))
        ));
        assert!(matches!(
            girl.scope.lookup_name("After"),
            Some(NameRef::Proc(CallId(2)))
        ));
    }

    #[test]
    fn rejects_callable_type_metadata_with_an_invalid_parameter_index() {
        let constants = parse_constant_header(
            r#"
typedef enum MaryKind {
KIND_A = 0,
} MaryKind;
typedef enum MaryValue {
VALUE_A = 0,
} MaryValue;
mary_callable_parameter_type_when(Test, 0, KIND_A, 1, MaryValue);
"#,
            &Options::default(),
        )
        .unwrap();
        let error = parse_callable_table_with_scope(
            "mary_callable_table { Test, }; void Test(MaryKind kind);",
            &Options::default(),
            &constants,
        )
        .err()
        .expect("invalid metadata must be rejected");
        assert!(error.to_string().contains("target parameter 1"), "{error}");
    }
}
