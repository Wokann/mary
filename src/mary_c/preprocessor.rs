use std::collections::HashMap;

use thiserror::Error;

use super::{validate_macro_name, Options};

#[derive(Debug, Error, PartialEq, Eq)]
pub enum PreprocessError {
    #[error("line {line}: invalid macro name '{name}'")]
    InvalidMacroName { line: usize, name: String },
    #[error("line {line}: malformed preprocessor directive: {message}")]
    Malformed { line: usize, message: String },
    #[error("line {line}: unexpected #{directive}")]
    Unexpected { line: usize, directive: String },
    #[error("line {line}: duplicate #else in the same conditional block")]
    DuplicateElse { line: usize },
    #[error("line {line}: #elif after #else")]
    ElifAfterElse { line: usize },
    #[error("unterminated conditional block opened on line {line}")]
    Unterminated { line: usize },
}

#[derive(Clone, Copy)]
struct Conditional {
    line: usize,
    parent_active: bool,
    branch_taken: bool,
    active: bool,
    saw_else: bool,
}

/// Apply the intentionally small, deterministic C-preprocessor subset used by
/// callable tables. Inactive lines are retained as blank lines so later error
/// locations continue to refer to the original source.
pub fn preprocess(source: &str, options: &Options) -> Result<String, PreprocessError> {
    let mut macros = options.defines.clone();
    let mut stack: Vec<Conditional> = Vec::new();
    let mut output = String::with_capacity(source.len());

    for (index, raw_line) in source.split_inclusive('\n').enumerate() {
        let line = index + 1;
        let logical = raw_line.strip_suffix('\n').unwrap_or(raw_line);
        let trimmed = logical.trim_start();
        let active = stack.last().is_none_or(|c| c.active);

        if let Some(rest) = trimmed.strip_prefix('#') {
            handle_directive(rest.trim(), line, &mut macros, &mut stack)?;
            if raw_line.ends_with('\n') {
                output.push('\n');
            }
        } else if active {
            output.push_str(raw_line);
        } else if raw_line.ends_with('\n') {
            output.push('\n');
        }
    }

    if let Some(frame) = stack.last() {
        return Err(PreprocessError::Unterminated { line: frame.line });
    }
    Ok(output)
}

fn handle_directive(
    text: &str,
    line: usize,
    macros: &mut HashMap<String, String>,
    stack: &mut Vec<Conditional>,
) -> Result<(), PreprocessError> {
    let (directive, args) = text
        .split_once(char::is_whitespace)
        .map(|(a, b)| (a, b.trim()))
        .unwrap_or((text, ""));
    let parent_active = stack.last().is_none_or(|c| c.active);

    match directive {
        "define" => {
            if !parent_active {
                return Ok(());
            }
            let (name, value) = args
                .split_once(char::is_whitespace)
                .map(|(a, b)| (a, b.trim()))
                .unwrap_or((args, "1"));
            validate_macro_name(name, line)?;
            macros.insert(name.to_owned(), value.to_owned());
        }
        "undef" => {
            if parent_active {
                validate_macro_name(args, line)?;
                macros.remove(args);
            }
        }
        "ifdef" | "ifndef" => {
            validate_macro_name(args, line)?;
            let condition = macros.contains_key(args) == (directive == "ifdef");
            stack.push(Conditional {
                line,
                parent_active,
                branch_taken: condition,
                active: parent_active && condition,
                saw_else: false,
            });
        }
        "if" => {
            let condition = eval_condition(args, macros, line)?;
            stack.push(Conditional {
                line,
                parent_active,
                branch_taken: condition,
                active: parent_active && condition,
                saw_else: false,
            });
        }
        "elif" => {
            let frame = stack
                .last_mut()
                .ok_or_else(|| PreprocessError::Unexpected {
                    line,
                    directive: directive.to_owned(),
                })?;
            if frame.saw_else {
                return Err(PreprocessError::ElifAfterElse { line });
            }
            let condition = eval_condition(args, macros, line)?;
            frame.active = frame.parent_active && !frame.branch_taken && condition;
            frame.branch_taken |= condition;
        }
        "else" => {
            if !args.is_empty() {
                return malformed(line, "#else does not take arguments");
            }
            let frame = stack
                .last_mut()
                .ok_or_else(|| PreprocessError::Unexpected {
                    line,
                    directive: directive.to_owned(),
                })?;
            if frame.saw_else {
                return Err(PreprocessError::DuplicateElse { line });
            }
            frame.saw_else = true;
            frame.active = frame.parent_active && !frame.branch_taken;
            frame.branch_taken = true;
        }
        "endif" => {
            if !args.is_empty() {
                return malformed(line, "#endif does not take arguments");
            }
            stack.pop().ok_or_else(|| PreprocessError::Unexpected {
                line,
                directive: directive.to_owned(),
            })?;
        }
        "" => return malformed(line, "empty directive"),
        _ if parent_active => return malformed(line, &format!("unsupported #{directive}")),
        _ => {}
    }
    Ok(())
}

fn malformed<T>(line: usize, message: &str) -> Result<T, PreprocessError> {
    Err(PreprocessError::Malformed {
        line,
        message: message.to_owned(),
    })
}

fn eval_condition(
    expression: &str,
    macros: &HashMap<String, String>,
    line: usize,
) -> Result<bool, PreprocessError> {
    let mut parser = ConditionParser::new(expression, macros, line);
    let value = parser.parse_or()?;
    parser.skip_space();
    if parser.rest().is_empty() {
        Ok(value)
    } else {
        malformed(
            line,
            &format!("unexpected condition text '{}'", parser.rest()),
        )
    }
}

struct ConditionParser<'a> {
    input: &'a str,
    offset: usize,
    macros: &'a HashMap<String, String>,
    line: usize,
}

impl<'a> ConditionParser<'a> {
    fn new(input: &'a str, macros: &'a HashMap<String, String>, line: usize) -> Self {
        Self {
            input,
            offset: 0,
            macros,
            line,
        }
    }
    fn rest(&self) -> &'a str {
        &self.input[self.offset..]
    }
    fn skip_space(&mut self) {
        while self.rest().starts_with(char::is_whitespace) {
            self.offset += self.rest().chars().next().unwrap().len_utf8();
        }
    }
    fn eat(&mut self, token: &str) -> bool {
        self.skip_space();
        if self.rest().starts_with(token) {
            self.offset += token.len();
            true
        } else {
            false
        }
    }
    fn parse_or(&mut self) -> Result<bool, PreprocessError> {
        let mut v = self.parse_and()?;
        while self.eat("||") {
            v = self.parse_and()? || v;
        }
        Ok(v)
    }
    fn parse_and(&mut self) -> Result<bool, PreprocessError> {
        let mut v = self.parse_unary()?;
        while self.eat("&&") {
            v = self.parse_unary()? && v;
        }
        Ok(v)
    }
    fn parse_unary(&mut self) -> Result<bool, PreprocessError> {
        if self.eat("!") {
            return Ok(!self.parse_unary()?);
        }
        if self.eat("(") {
            let v = self.parse_or()?;
            if !self.eat(")") {
                return malformed(self.line, "missing ')' in condition");
            }
            return Ok(v);
        }
        self.skip_space();
        if self.rest().starts_with("defined") {
            self.offset += "defined".len();
            self.skip_space();
            let paren = self.eat("(");
            let name = self.ident()?;
            if paren && !self.eat(")") {
                return malformed(self.line, "missing ')' after defined");
            }
            return Ok(self.macros.contains_key(name));
        }
        let atom = self.ident()?;
        Ok(self.macros.get(atom).is_some_and(|v| v != "0"))
    }
    fn ident(&mut self) -> Result<&'a str, PreprocessError> {
        self.skip_space();
        let start = self.offset;
        while self
            .rest()
            .chars()
            .next()
            .is_some_and(|c| c == '_' || c.is_ascii_alphanumeric())
        {
            self.offset += 1;
        }
        if start == self.offset {
            return malformed(self.line, "expected macro name");
        }
        Ok(&self.input[start..self.offset])
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn selects_independent_target_layouts() {
        let source = "head\n#if defined(FOMT_US) || defined(FOMT_JP)\nboy\n#elif defined(MFOMT_US)\ngirl\n#else\nother\n#endif\ntail\n";
        let fomt = preprocess(source, &Options::default().define("FOMT_US").unwrap()).unwrap();
        let mfomt = preprocess(source, &Options::default().define("MFOMT_US").unwrap()).unwrap();
        assert!(fomt.contains("boy") && !fomt.contains("girl"));
        assert!(mfomt.contains("girl") && !mfomt.contains("boy"));
        assert_eq!(fomt.lines().count(), source.lines().count());
    }

    #[test]
    fn rejects_unbalanced_conditionals() {
        assert!(matches!(
            preprocess("#if defined(X)\n", &Options::default()),
            Err(PreprocessError::Unterminated { .. })
        ));
        assert!(matches!(
            preprocess("#endif\n", &Options::default()),
            Err(PreprocessError::Unexpected { .. })
        ));
    }
}
