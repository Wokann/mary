use std::{cell::RefCell, rc::Rc, sync::Arc};

use crate::{
    charmap::{Charmap, CharmapError},
    ir::IntValue,
};

use super::parser::Token;

use lexgen::lexer;

fn parse_dec(digits: &str) -> IntValue {
    IntValue::from_str_radix(digits, 10).unwrap()
}

fn parse_hex(digits: &str) -> IntValue {
    IntValue::from_str_radix(digits, 16).unwrap()
}

#[derive(Debug, Default)]
pub struct LexerState {
    /// buffer for building string literals
    string_buf: Vec<u8>,
    /// Source text waiting for one longest-match charmap encoding pass.
    text_buf: String,
    charmap: Option<Arc<Charmap>>,
    charmap_error: Rc<RefCell<Option<CharmapError>>>,
}

impl LexerState {
    pub fn with_charmap(
        charmap: Arc<Charmap>,
        charmap_error: Rc<RefCell<Option<CharmapError>>>,
    ) -> Self {
        Self {
            charmap: Some(charmap),
            charmap_error,
            ..Self::default()
        }
    }

    fn push_text(&mut self, text: &str) {
        self.text_buf.push_str(text);
    }

    fn flush_text(&mut self) {
        if self.text_buf.is_empty() {
            return;
        }
        let text = std::mem::take(&mut self.text_buf);
        let encoded = match &self.charmap {
            Some(charmap) => charmap.encode_text(&text),
            None => Ok(text.as_bytes().to_vec()),
        };
        match encoded {
            Ok(bytes) => self.string_buf.extend(bytes),
            Err(err) if self.charmap_error.borrow().is_none() => {
                *self.charmap_error.borrow_mut() = Some(err)
            }
            Err(_) => {}
        }
    }
}

lexer! {
    pub Lexer(LexerState) -> Token;

    let dec_digit = ['0'-'9'];
    let hex_digit = $dec_digit | ['a'-'f' 'A'-'F'];

    rule Init {
        /* ignore whitespace */
        $$ascii_whitespace,

        /* keywords */
        "func"    = Token::KwFunc,
        "proc"    = Token::KwProc,
        "const"   = Token::KwConst,
        "script"  = Token::KwScript,
        "var"     = Token::KwVar,
        "if"      = Token::KwIf,
        "else"    = Token::KwElse,
        "for"     = Token::KwFor,
        "do"      = Token::KwDo,
        "while"   = Token::KwWhile,
        "switch"  = Token::KwSwitch,
        "compact" = Token::KwCompact,
        "fallthrough" = Token::KwFallthrough,
        "implicit" = Token::KwImplicit,
        "case"    = Token::KwCase,
        "default" = Token::KwDefault,
        "dead"    = Token::KwDead,
        "discard" = Token::KwDiscard,
        "nodisc"  = Token::KwNoDisc,
        "exit"    = Token::KwExit,
        "break"   = Token::KwBreak,
        "jump"    = Token::KwJump,
        "next"    = Token::KwNext,
        "string"  = Token::KwString,
        "integer" = Token::KwInteger,
        "ir"      = Token::KwIr,

        /* punctuation */
        "(" = Token::LParen,
        ")" = Token::RParen,
        "{" = Token::LCurly,
        "}" = Token::RCurly,
        "," = Token::Comma,
        ":" = Token::Colon,
        ";" = Token::Semicolon,

        /* operators */
        "+"  = Token::Plus,
        "-"  = Token::Minus,
        "*"  = Token::Times,
        "/"  = Token::Divide,
        "%"  = Token::Modulus,
        "++" = Token::PlusPlus,
        "--" = Token::MinusMinus,
        "&&" = Token::LAnd,
        "||" = Token::LOr,
        "!"  = Token::Negate,
        "==" = Token::CompareEq,
        "!=" = Token::CompareNe,
        "<"  = Token::CompareLt,
        "<=" = Token::CompareLe,
        ">=" = Token::CompareGe,
        ">"  = Token::CompareGt,
        "="  = Token::Equal,
        "+=" = Token::AddEqual,
        "-=" = Token::SubEqual,
        "*=" = Token::MulEqual,
        "/=" = Token::DivEqual,
        "%=" = Token::ModEqual,

        /* names */

        let name_head = ['a'-'z' 'A'-'Z' '_'];
        let name_tail = $name_head | $dec_digit;

        $name_head $name_tail * => |lexer| lexer.return_(Token::Name(String::from(lexer.match_()))),

        /* integer literals */

        $dec_digit + => |lexer| lexer.return_(Token::Integer(parse_dec(lexer.match_()))),
        "0x" $hex_digit + => |lexer| lexer.return_(Token::Integer(parse_hex(&lexer.match_()[2..]))),

        /* string literals */

        '"' => |lexer| {
            lexer.switch(LexerRule::String)
        },

        /* comments */
        "//" => |lexer| lexer.switch(LexerRule::LineComment),
        "/*" => |lexer| lexer.switch(LexerRule::MultComment),

        /* CPP line markers (treat it as a comment for now) */
        "#" => |lexer| lexer.switch(LexerRule::LineComment),
    }

    rule LineComment {
        (_ # '\n') * ('\n' | $) => |lexer| {
            lexer.reset_match();
            lexer.switch(LexerRule::Init)
        },
    }

    rule MultComment {
        "*/" => |lexer| {
            lexer.reset_match();
            lexer.switch(LexerRule::Init)
        },

        _ => |lexer| lexer.continue_(),
    }

    rule String {
        '"' => |lexer| {
            use std::mem;
            lexer.state().flush_text();
            let str_bytes = mem::take(&mut lexer.state().string_buf);
            lexer.switch_and_return(LexerRule::Init, Token::StringLit(str_bytes))
        },

        // TODO: change those a bit to make writing scripts easier

        "\\\\" => |lexer| {
            lexer.state().push_text("\\");
            lexer.continue_()
        },

        "\\\"" => |lexer| {
            lexer.state().push_text("\"");
            lexer.continue_()
        },

        "\\\n" => |lexer| {
            lexer.state().push_text("\n");
            lexer.continue_()
        },

        "\\x" $hex_digit $hex_digit => |lexer| {
            let m = lexer.match_();
            let byte = u8::from_str_radix(&m[m.len() - 2..], 16).unwrap();
            lexer.state().flush_text();
            lexer.state().string_buf.push(byte);
            lexer.continue_()
        },

        "\\" _ => |lexer| {
            let matched = lexer.match_();
            let start = matched
                .char_indices()
                .rev()
                .nth(1)
                .map(|(offset, _)| offset)
                .expect("named escape must contain two characters");
            let escape = matched[start..].to_owned();
            lexer.state().push_text(&escape);
            lexer.continue_()
        },

        _ => |lexer| {
            let m = lexer.match_();
            let last = m
                .char_indices()
                .last()
                .map(|(offset, _)| &m[offset..])
                .expect("string lexer matched an empty character");
            lexer.state().push_text(last);
            lexer.continue_()
        },
    }

    // TODO: comments
    // TODO: locations
}
