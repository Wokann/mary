use crate::compiler::{ParseContext, ScriptError};
use std::{cell::RefCell, rc::Rc, sync::Arc};

use crate::charmap::Charmap;

pub fn parse_string(code_string: &str) -> Result<ParseContext, ScriptError> {
    parse_string_inner(code_string, None)
}

pub fn parse_string_with_charmap(
    code_string: &str,
    charmap: Arc<Charmap>,
) -> Result<ParseContext, ScriptError> {
    parse_string_inner(code_string, Some(charmap))
}

fn parse_string_inner(
    code_string: &str,
    charmap: Option<Arc<Charmap>>,
) -> Result<ParseContext, ScriptError> {
    use lexgen_util::LexerErrorKind;

    use crate::compiler::Lexer;
    use crate::compiler::Parser;

    let charmap_error = Rc::new(RefCell::new(None));
    let l = match charmap {
        Some(charmap) => Lexer::new_with_state(
            code_string,
            crate::compiler::lexer::LexerState::with_charmap(charmap, Rc::clone(&charmap_error)),
        ),
        None => Lexer::new_with_state(
            code_string,
            crate::compiler::lexer::LexerState::with_error_sink(Rc::clone(&charmap_error)),
        ),
    };
    let mut p = Parser::new(ParseContext::new());

    for tok in l {
        match tok {
            Ok((start, tok, _)) => match p.parse(tok) {
                Ok(()) => {}
                Err(ScriptError::SyntaxError) => return Err(ScriptError::SyntaxErrorAt(start)),
                Err(err) => return Err(err),
            },

            Err(err) => match err.kind {
                LexerErrorKind::InvalidToken | LexerErrorKind::Custom(_) => {
                    return Err(ScriptError::LexError(err.location));
                }
            },
        }
    }

    if let Some(err) = charmap_error.borrow_mut().take() {
        return Err(err.into());
    }

    match p.end_of_input() {
        Ok((_, parse_ctx)) => Ok(parse_ctx),
        Err(err) => Err(err),
    }
}
