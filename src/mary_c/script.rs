use std::collections::{HashMap, HashSet};

use thiserror::Error;

use crate::{
    ast::{
        AssignOperation, ConstVal, Expr, Invoke, NameAccess, NameRef, Stmt, SwitchCase,
        SwitchLayout,
    },
    charmap::{Charmap, CharmapError},
    compiler::{
        compile_script,
        error::{CompileError, CompileErrors},
    },
    const_scope::ConstScope,
    ir::{IntValue, Script, SwitchId, ValueType},
};

use super::{preprocess, Options, PreprocessError, ScriptTable};

#[derive(Debug, Error)]
pub enum MaryScriptError {
    #[error(transparent)]
    Preprocess(#[from] PreprocessError),
    #[error("line {line}, column {column}: {message}")]
    Syntax {
        line: usize,
        column: usize,
        message: String,
    },
    #[error("script '{name}' failed semantic compilation: {errors}")]
    Compile { name: String, errors: CompileErrors },
    #[error("could not create the Mary-C parser worker: {0}")]
    Worker(String),
    #[error(transparent)]
    Charmap(#[from] CharmapError),
}

#[derive(Debug)]
pub struct ScriptContext {
    pub scripts: Vec<(IntValue, String, Script)>,
}

#[derive(Clone, Debug, PartialEq)]
enum K {
    Id(String),
    Int(i64),
    Str(Vec<u8>),
    Lp,
    Rp,
    Lb,
    Rb,
    Ls,
    Rs,
    Comma,
    Semi,
    Colon,
    Plus,
    Minus,
    Star,
    Slash,
    Percent,
    Bang,
    Eq,
    PlusEq,
    MinusEq,
    StarEq,
    SlashEq,
    PercentEq,
    EqEq,
    Ne,
    Lt,
    Le,
    Gt,
    Ge,
    AndAnd,
    OrOr,
    PlusPlus,
    MinusMinus,
}

#[derive(Clone, Debug)]
struct Tok {
    k: K,
    line: usize,
    col: usize,
}

pub fn parse_scripts(
    source: &str,
    options: &Options,
    scope: &ConstScope,
) -> Result<ScriptContext, MaryScriptError> {
    parse_scripts_with_optional_table(source, options, scope, None)
}

pub fn parse_named_scripts(
    source: &str,
    options: &Options,
    scope: &ConstScope,
    script_table: &ScriptTable,
) -> Result<ScriptContext, MaryScriptError> {
    let mut combined = scope.clone();
    script_table.add_constants(&mut combined);
    parse_scripts_with_optional_table(source, options, &combined, Some(script_table))
}

pub fn parse_named_scripts_with_charmap(
    source: &str,
    options: &Options,
    scope: &ConstScope,
    script_table: &ScriptTable,
    charmap: &Charmap,
) -> Result<ScriptContext, MaryScriptError> {
    let encoded = encode_source_strings(source, charmap)?;
    let mut combined = scope.clone();
    script_table.add_constants(&mut combined);
    parse_scripts_with_optional_table(&encoded, options, &combined, Some(script_table))
}

fn encode_source_strings(source: &str, charmap: &Charmap) -> Result<String, MaryScriptError> {
    let mut output = String::with_capacity(source.len());
    let mut chars = source.char_indices().peekable();
    while let Some((_, ch)) = chars.next() {
        output.push(ch);
        if ch != '"' {
            continue;
        }
        let mut text = String::new();
        loop {
            let Some((_, ch)) = chars.next() else {
                return Err(MaryScriptError::Syntax {
                    line: 1,
                    column: 1,
                    message: "unterminated string".into(),
                });
            };
            if ch == '"' {
                flush_encoded_text(&mut output, &mut text, charmap)?;
                output.push('"');
                break;
            }
            if ch != '\\' {
                text.push(ch);
                continue;
            }
            let Some((_, escaped)) = chars.next() else {
                return Err(MaryScriptError::Syntax {
                    line: 1,
                    column: 1,
                    message: "unterminated escape".into(),
                });
            };
            flush_encoded_text(&mut output, &mut text, charmap)?;
            if escaped == 'x' {
                output.push_str("\\x");
                for _ in 0..2 {
                    let Some((_, digit)) = chars.next() else {
                        return Err(MaryScriptError::Syntax {
                            line: 1,
                            column: 1,
                            message: "short hexadecimal escape".into(),
                        });
                    };
                    output.push(digit);
                }
            } else {
                let mapped = match escaped {
                    '"' => "\"".to_owned(),
                    '\\' => "\\".to_owned(),
                    other => format!("\\{other}"),
                };
                for byte in charmap.encode_text(&mapped)? {
                    output.push_str(&format!("\\x{byte:02X}"));
                }
            }
        }
    }
    Ok(output)
}

fn flush_encoded_text(
    output: &mut String,
    text: &mut String,
    charmap: &Charmap,
) -> Result<(), CharmapError> {
    if !text.is_empty() {
        for byte in charmap.encode_text(text)? {
            output.push_str(&format!("\\x{byte:02X}"));
        }
        text.clear();
    }
    Ok(())
}

fn parse_scripts_with_optional_table(
    source: &str,
    options: &Options,
    scope: &ConstScope,
    script_table: Option<&ScriptTable>,
) -> Result<ScriptContext, MaryScriptError> {
    // Vanilla contains exceptionally deep structured scripts. A deliberately
    // sized worker stack makes correctness independent of the platform's small
    // default test/main-thread stack; the grammar itself remains unbounded.
    std::thread::scope(|thread_scope| {
        let worker = std::thread::Builder::new()
            .name("mary-c-parser".into())
            .stack_size(16 * 1024 * 1024)
            .spawn_scoped(thread_scope, || {
                parse_scripts_inner(source, options, scope, script_table)
            })
            .map_err(|error| MaryScriptError::Worker(error.to_string()))?;
        match worker.join() {
            Ok(result) => result,
            Err(panic) => std::panic::resume_unwind(panic),
        }
    })
}

fn parse_scripts_inner(
    source: &str,
    options: &Options,
    scope: &ConstScope,
    script_table: Option<&ScriptTable>,
) -> Result<ScriptContext, MaryScriptError> {
    let source = preprocess(source, options)?;
    let tokens = lex(&source)?;
    let mut parser = P {
        tokens,
        at: 0,
        controls: Vec::new(),
    };
    let mut text_constants = parser.text_table_and_declarations()?;
    let mut scripts = Vec::new();
    let mut ids = HashSet::new();
    let mut names = HashSet::new();
    while !parser.end() {
        let explicit_id = if parser.eat_word("mary_script") {
            parser.take(K::Lp)?;
            let id = parser.int()?;
            if !(0..=u32::MAX as i64).contains(&id) {
                return parser.err("mary_script id must fit an unsigned 32-bit value");
            }
            parser.take(K::Rp)?;
            Some(id)
        } else {
            None
        };
        parser.word("void")?;
        let name = parser.id()?;
        parser.take(K::Lp)?;
        parser.word("void")?;
        parser.take(K::Rp)?;
        let id =
            match (explicit_id, script_table) {
                (Some(id), _) => id,
                (None, Some(table)) => table.id(&name).ok_or_else(|| MaryScriptError::Syntax {
                    line: parser.peek().map_or(1, |token| token.line),
                    column: parser.peek().map_or(1, |token| token.col),
                    message: format!("script '{name}' is not present in mary_script_table"),
                })?,
                (None, None) => return parser.err(
                    "plain script definitions require a mary_script_table; use parse_named_scripts",
                ),
            };
        if !ids.insert(id) {
            return parser.err("mary_script id is declared more than once");
        }
        if !names.insert(name.clone()) {
            return parser.err("mary_script name is declared more than once");
        }
        let mut body = parser.block()?;
        if !text_constants.is_empty() {
            if !scripts.is_empty() {
                return parser
                    .err("mary_text_table may only belong to one script per .mary.c file");
            }
            text_constants.append(&mut body);
            body = std::mem::take(&mut text_constants);
        }
        let semantic_errors = validate_calls(&body, scope);
        if !semantic_errors.is_empty() {
            return Err(MaryScriptError::Compile {
                name,
                errors: CompileErrors(semantic_errors),
            });
        }
        let compiled = compile_script(body, scope).map_err(|errors| MaryScriptError::Compile {
            name: name.clone(),
            errors,
        })?;
        scripts.push((id, name, compiled));
    }
    Ok(ScriptContext { scripts })
}

fn validate_calls(statements: &[Stmt], scope: &ConstScope) -> Vec<CompileError> {
    fn value_type(
        expr: &Expr,
        locals: &HashMap<String, ValueType>,
        scope: &ConstScope,
    ) -> ValueType {
        match expr {
            Expr::Str(_) => ValueType::String,
            Expr::Name(name) => locals
                .get(name)
                .copied()
                .or_else(|| match scope.lookup_name(name) {
                    Some(NameRef::Const(ConstVal::Str(_))) => Some(ValueType::String),
                    _ => None,
                })
                .unwrap_or(ValueType::Integer),
            _ => ValueType::Integer,
        }
    }
    fn type_name(value: ValueType) -> &'static str {
        if value == ValueType::String {
            "const char *"
        } else {
            "int"
        }
    }
    fn check_call(
        name: &str,
        args: &[Expr],
        locals: &HashMap<String, ValueType>,
        scope: &ConstScope,
        out: &mut Vec<CompileError>,
    ) {
        if let Some((_, shape)) = scope.callable_map().get(name) {
            if shape.num_parameters() != args.len() {
                out.push(CompileError::WrongArgumentCount {
                    name: name.into(),
                    expected: shape.num_parameters(),
                    actual: args.len(),
                })
            } else {
                for (index, (arg, expected)) in args.iter().zip(shape.parameter_types()).enumerate()
                {
                    if *expected == ValueType::Undefined {
                        continue;
                    }
                    let expected = if *expected == ValueType::String {
                        ValueType::String
                    } else {
                        ValueType::Integer
                    };
                    let actual = value_type(arg, locals, scope);
                    if actual != expected {
                        out.push(CompileError::ArgumentTypeMismatch {
                            name: name.into(),
                            index: index + 1,
                            expected: type_name(expected),
                            actual: type_name(actual),
                        })
                    }
                }
            }
        }
    }

    let mut locals = HashMap::new();
    let mut statement_stack: Vec<&Stmt> = statements.iter().collect();
    while let Some(stmt) = statement_stack.pop() {
        match stmt {
            Stmt::Vars(items) => {
                for (name, _) in items {
                    locals.insert(name.clone(), ValueType::Integer);
                }
            }
            Stmt::Consts(items) => {
                for (name, e) in items {
                    locals.insert(name.clone(), value_type(e, &locals, scope));
                }
            }
            Stmt::If(_, body) | Stmt::DoWhile(_, body) => statement_stack.extend(body),
            Stmt::IfElse(_, a, b) => {
                statement_stack.extend(a);
                statement_stack.extend(b)
            }
            Stmt::For(v) => {
                statement_stack.push(&v.1);
                statement_stack.push(&v.2);
                statement_stack.extend(&v.3)
            }
            Stmt::Switch(_, cases, _, _) => {
                for case in cases {
                    statement_stack.extend(case.stmts())
                }
            }
            _ => {}
        }
    }

    let mut out = Vec::new();
    let mut statements_to_visit: Vec<&Stmt> = statements.iter().collect();
    let mut expressions = Vec::new();
    while let Some(stmt) = statements_to_visit.pop() {
        match stmt {
            Stmt::Vars(items) => {
                for (_, e) in items {
                    if let Some(e) = e {
                        expressions.push(e)
                    }
                }
            }
            Stmt::Consts(items) => {
                for (_, e) in items {
                    expressions.push(e)
                }
            }
            Stmt::Assign(_, _, e) | Stmt::AssignNoDisc(_, _, e) | Stmt::Expr(e) => {
                expressions.push(e)
            }
            Stmt::Call(call) => {
                check_call(&call.func, &call.args, &locals, scope, &mut out);
                expressions.extend(&call.args)
            }
            Stmt::If(e, b) | Stmt::DoWhile(e, b) => {
                expressions.push(e);
                statements_to_visit.extend(b)
            }
            Stmt::IfElse(e, a, b) => {
                expressions.push(e);
                statements_to_visit.extend(a);
                statements_to_visit.extend(b)
            }
            Stmt::For(v) => {
                expressions.push(&v.0);
                statements_to_visit.push(&v.1);
                statements_to_visit.push(&v.2);
                statements_to_visit.extend(&v.3)
            }
            Stmt::Switch(e, cases, _, _) => {
                expressions.push(e);
                for case in cases {
                    match case {
                        SwitchCase::Case(values, _) | SwitchCase::Fallthrough(values, _) => {
                            expressions.extend(values)
                        }
                        _ => {}
                    }
                    statements_to_visit.extend(case.stmts())
                }
            }
            _ => {}
        }
    }
    while let Some(expr) = expressions.pop() {
        match expr {
            Expr::OpAdd(v)
            | Expr::OpSub(v)
            | Expr::OpMul(v)
            | Expr::OpDiv(v)
            | Expr::OpMod(v)
            | Expr::OpOr(v)
            | Expr::OpAnd(v)
            | Expr::CmpEq(v)
            | Expr::CmpNe(v)
            | Expr::CmpLt(v)
            | Expr::CmpLe(v)
            | Expr::CmpGe(v)
            | Expr::CmpGt(v) => {
                expressions.push(&v.0);
                expressions.push(&v.1)
            }
            Expr::OpNeg(v) | Expr::OpNot(v) => expressions.push(v),
            Expr::Call(call) => {
                check_call(&call.func, &call.args, &locals, scope, &mut out);
                expressions.extend(&call.args)
            }
            _ => {}
        }
    }
    out
}

struct P {
    tokens: Vec<Tok>,
    at: usize,
    controls: Vec<Control>,
}

#[derive(Clone, Copy, PartialEq, Eq)]
enum Control {
    Switch,
    Loop,
}

impl P {
    fn text_table_and_declarations(&mut self) -> Result<Vec<Stmt>, MaryScriptError> {
        if !self.eat_word("mary_text_table") {
            return Ok(Vec::new());
        }
        self.take(K::Lb)?;
        if self.is_word("const") {
            let mut result = Vec::new();
            let mut names = HashSet::new();
            while !self.eat(&K::Rb) {
                let (name, value) = self.text_declaration()?;
                if !names.insert(name.clone()) {
                    return self.err(&format!("text '{name}' is declared more than once"));
                }
                result.push(Stmt::Consts(vec![(name, value)]));
            }
            self.eat(&K::Semi);
            return Ok(result);
        }

        // Compatibility syntax: an ordered name list followed by declarations.
        let mut names = Vec::new();
        while !self.eat(&K::Rb) {
            names.push(self.id()?);
            if !self.eat(&K::Comma) && !self.is(&K::Rb) {
                return self.err("expected ',' or '}' after text slot");
            }
        }
        self.eat(&K::Semi);

        let mut declarations = HashMap::new();
        while self.eat_word("const") {
            let (name, value) = self.text_declaration_after_const()?;
            if declarations.insert(name.clone(), value).is_some() {
                return self.err(&format!("text '{name}' is declared more than once"));
            }
        }
        let mut result = Vec::new();
        for name in names {
            let value = declarations
                .remove(&name)
                .ok_or_else(|| MaryScriptError::Syntax {
                    line: self.peek().map_or(1, |token| token.line),
                    column: self.peek().map_or(1, |token| token.col),
                    message: format!("text '{name}' has a table slot but no declaration"),
                })?;
            result.push(Stmt::Consts(vec![(name, value)]));
        }
        if let Some(name) = declarations.keys().next() {
            return self.err(&format!("text declaration '{name}' has no table slot"));
        }
        Ok(result)
    }

    fn text_declaration(&mut self) -> Result<(String, Expr), MaryScriptError> {
        self.word("const")?;
        self.text_declaration_after_const()
    }

    fn text_declaration_after_const(&mut self) -> Result<(String, Expr), MaryScriptError> {
        self.word("char")?;
        let name = self.id()?;
        self.take(K::Ls)?;
        self.take(K::Rs)?;
        self.take(K::Eq)?;
        let mut bytes = match self.primary()? {
            Expr::Str(bytes) => bytes,
            _ => return self.err("text declarations require a string literal"),
        };
        while matches!(self.peek().map(|token| &token.k), Some(K::Str(_))) {
            let Expr::Str(next) = self.primary()? else {
                unreachable!()
            };
            bytes.extend(next);
        }
        self.take(K::Semi)?;
        Ok((name, Expr::Str(bytes)))
    }

    fn block(&mut self) -> Result<Vec<Stmt>, MaryScriptError> {
        self.take(K::Lb)?;
        let mut out = Vec::new();
        while !self.eat(&K::Rb) {
            if self.end() {
                return self.err("unterminated block");
            }
            out.push(self.stmt()?);
        }
        Ok(out)
    }

    fn stmt(&mut self) -> Result<Stmt, MaryScriptError> {
        if self.eat_word("int") {
            let values = self.declarations(false)?;
            self.take(K::Semi)?;
            return Ok(Stmt::Vars(values));
        }
        if self.eat_word("const") {
            if self.eat_word("int") { /* integer constant */
            } else if self.eat_word("char") {
                self.take(K::Star)?;
            } else {
                return self.err("const declarations must use int or char *");
            }
            let values = self.declarations(true)?;
            self.take(K::Semi)?;
            return Ok(Stmt::Consts(
                values
                    .into_iter()
                    .map(|(n, e)| (n, e.expect("required above")))
                    .collect(),
            ));
        }
        if self.eat_word("if") {
            let cond = self.paren_expr()?;
            let yes = self.block()?;
            if self.eat_word("else") {
                let no = if self.is_word("if") {
                    vec![self.stmt()?]
                } else {
                    self.block()?
                };
                return Ok(Stmt::IfElse(cond, yes, no));
            }
            return Ok(Stmt::If(cond, yes));
        }
        if self.eat_word("do") {
            self.controls.push(Control::Loop);
            let body = self.block()?;
            self.controls.pop();
            self.word("while")?;
            let cond = self.paren_expr()?;
            self.take(K::Semi)?;
            return Ok(Stmt::DoWhile(cond, body));
        }
        if self.eat_word("for") {
            self.take(K::Lp)?;
            let init = if self.eat_word("int") {
                Stmt::Vars(self.declarations(false)?)
            } else {
                self.simple_stmt(false)?
            };
            self.take(K::Semi)?;
            let cond = self.expr(0)?;
            self.take(K::Semi)?;
            let iter = self.simple_stmt(false)?;
            self.take(K::Rp)?;
            self.controls.push(Control::Loop);
            let body = self.block()?;
            self.controls.pop();
            return Ok(Stmt::For(Box::new((cond, init, iter, body))));
        }
        if self.eat_word("switch") {
            let expr = self.paren_expr()?;
            return self.switch(expr, SwitchLayout::Standard);
        }
        if self.eat_word("mary_switch_compact") {
            let expr = self.paren_expr()?;
            return self.switch(expr, SwitchLayout::Compact);
        }
        if self.eat_word("return") {
            self.take(K::Semi)?;
            return Ok(Stmt::Exit);
        }
        if self.eat_word("break") {
            if self.controls.last() != Some(&Control::Switch) {
                return self.err("standard break must target the nearest switch; loop break is not supported by the current structured backend");
            }
            self.take(K::Semi)?;
            return Ok(Stmt::Break);
        }
        if self.eat_word("mary_break_switch") {
            if !self.controls.contains(&Control::Switch) {
                return self.err("mary_break_switch requires an enclosing switch");
            }
            self.take(K::Semi)?;
            return Ok(Stmt::Break);
        }
        if self.eat_word("mary_nodisc") {
            self.take(K::Lp)?;
            let (op, n, e) = self.assignment()?;
            self.take(K::Rp)?;
            self.take(K::Semi)?;
            return Ok(Stmt::AssignNoDisc(op, n, e));
        }
        let result = self.simple_stmt(true)?;
        self.take(K::Semi)?;
        Ok(result)
    }

    fn declarations(
        &mut self,
        require_init: bool,
    ) -> Result<Vec<(String, Option<Expr>)>, MaryScriptError> {
        let mut out = Vec::new();
        loop {
            let name = self.id()?;
            let value = if self.eat(&K::Eq) {
                Some(self.expr(0)?)
            } else if require_init {
                return self.err("const int requires an initializer");
            } else {
                None
            };
            out.push((name, value));
            if !self.eat(&K::Comma) {
                break;
            }
        }
        Ok(out)
    }

    fn simple_stmt(&mut self, calls_discard: bool) -> Result<Stmt, MaryScriptError> {
        if let Some(K::Id(name)) = self.peek().map(|t| &t.k) {
            let name = name.clone();
            if matches!(
                self.tokens.get(self.at + 1).map(|t| &t.k),
                Some(K::Eq | K::PlusEq | K::MinusEq | K::StarEq | K::SlashEq | K::PercentEq)
            ) {
                let (op, n, e) = self.assignment()?;
                return Ok(Stmt::Assign(op, n, e));
            }
            if matches!(
                self.tokens.get(self.at + 1).map(|t| &t.k),
                Some(K::PlusPlus | K::MinusMinus)
            ) {
                self.at += 2;
                return Ok(Stmt::Expr(if self.tokens[self.at - 1].k == K::PlusPlus {
                    Expr::PostIncrement(name)
                } else {
                    Expr::PostDecrement(name)
                }));
            }
        }
        let expr = self.expr(0)?;
        match expr {
            Expr::Call(call) if !calls_discard => Ok(Stmt::Call(call)),
            Expr::Call(call) if calls_discard => Ok(Stmt::Call(call)),
            other => Ok(Stmt::Expr(other)),
        }
    }

    fn assignment(&mut self) -> Result<(AssignOperation, String, Expr), MaryScriptError> {
        let name = self.id()?;
        let op = if self.eat(&K::Eq) {
            AssignOperation::None
        } else if self.eat(&K::PlusEq) {
            AssignOperation::Add
        } else if self.eat(&K::MinusEq) {
            AssignOperation::Sub
        } else if self.eat(&K::StarEq) {
            AssignOperation::Mul
        } else if self.eat(&K::SlashEq) {
            AssignOperation::Div
        } else if self.eat(&K::PercentEq) {
            AssignOperation::Mod
        } else {
            return self.err("expected assignment operator");
        };
        Ok((op, name, self.expr(0)?))
    }

    fn switch(&mut self, expr: Expr, layout: SwitchLayout) -> Result<Stmt, MaryScriptError> {
        self.take(K::Lb)?;
        self.controls.push(Control::Switch);
        let result = self.switch_cases();
        self.controls.pop();
        Ok(Stmt::Switch(expr, result?, SwitchId(0), layout))
    }

    fn switch_cases(&mut self) -> Result<Vec<SwitchCase>, MaryScriptError> {
        let mut cases = Vec::new();
        while !self.eat(&K::Rb) {
            if self.end() {
                return self.err("unterminated switch");
            }
            let kind = if self.eat_word("case") {
                let mut vals = vec![self.expr(0)?];
                self.take(K::Colon)?;
                while self.eat_word("case") {
                    vals.push(self.expr(0)?);
                    self.take(K::Colon)?;
                }
                Some(vals)
            } else if self.eat_word("default") {
                self.take(K::Colon)?;
                None
            } else if self.eat_word("mary_implicit_default") {
                self.take(K::Colon)?;
                let body = self.case_body()?;
                cases.push(SwitchCase::ImplicitDefault(body));
                continue;
            } else if self.eat_word("mary_dead_jump") {
                self.take(K::Colon)?;
                let body = self.case_body()?;
                cases.push(SwitchCase::DeadJump(body));
                continue;
            } else {
                return self.err("expected case/default label");
            };
            let body = self.case_body()?;
            let terminating = matches!(body.last(), Some(Stmt::Break | Stmt::Exit));
            cases.push(match kind {
                Some(v) if terminating => SwitchCase::Case(v, body),
                Some(v) => SwitchCase::Fallthrough(v, body),
                None if terminating => SwitchCase::Default(body),
                None => SwitchCase::DefaultFallthrough(body),
            });
        }
        Ok(cases)
    }

    fn case_body(&mut self) -> Result<Vec<Stmt>, MaryScriptError> {
        let mut body = Vec::new();
        while !self.end()
            && !self.is(&K::Rb)
            && !self.is_word("case")
            && !self.is_word("default")
            && !self.is_word("mary_implicit_default")
            && !self.is_word("mary_dead_jump")
        {
            body.push(self.stmt()?);
        }
        Ok(body)
    }

    fn paren_expr(&mut self) -> Result<Expr, MaryScriptError> {
        self.take(K::Lp)?;
        let e = self.expr(0)?;
        self.take(K::Rp)?;
        Ok(e)
    }
    fn expr(&mut self, min: u8) -> Result<Expr, MaryScriptError> {
        let mut lhs = if self.eat(&K::Minus) {
            Expr::OpNeg(Box::new(self.expr(11)?))
        } else if self.eat(&K::Bang) {
            Expr::OpNot(Box::new(self.expr(11)?))
        } else if self.eat(&K::PlusPlus) {
            Expr::PreIncrement(self.id()?)
        } else if self.eat(&K::MinusMinus) {
            Expr::PreDecrement(self.id()?)
        } else {
            self.primary()?
        };
        while let Some((prec, op)) = self.binop() {
            if prec < min {
                break;
            }
            self.at += 1;
            let rhs = self.expr(prec + 1)?;
            lhs = match op {
                K::Plus => Expr::OpAdd(Box::new((lhs, rhs))),
                K::Minus => Expr::OpSub(Box::new((lhs, rhs))),
                K::Star => Expr::OpMul(Box::new((lhs, rhs))),
                K::Slash => Expr::OpDiv(Box::new((lhs, rhs))),
                K::Percent => Expr::OpMod(Box::new((lhs, rhs))),
                K::OrOr => Expr::OpOr(Box::new((lhs, rhs))),
                K::AndAnd => Expr::OpAnd(Box::new((lhs, rhs))),
                K::EqEq => Expr::CmpEq(Box::new((lhs, rhs))),
                K::Ne => Expr::CmpNe(Box::new((lhs, rhs))),
                K::Lt => Expr::CmpLt(Box::new((lhs, rhs))),
                K::Le => Expr::CmpLe(Box::new((lhs, rhs))),
                K::Gt => Expr::CmpGt(Box::new((lhs, rhs))),
                K::Ge => Expr::CmpGe(Box::new((lhs, rhs))),
                _ => unreachable!(),
            };
        }
        Ok(lhs)
    }
    fn primary(&mut self) -> Result<Expr, MaryScriptError> {
        if self.eat(&K::Lp) {
            let e = self.expr(0)?;
            self.take(K::Rp)?;
            return Ok(e);
        }
        let tok = self
            .peek()
            .cloned()
            .ok_or_else(|| self.syntax("expected expression"))?;
        self.at += 1;
        match tok.k {
            K::Int(v) => Ok(Expr::Int(v)),
            K::Str(v) => Ok(Expr::Str(v)),
            K::Id(name) => {
                if self.eat(&K::Lp) {
                    let mut args = Vec::new();
                    if !self.eat(&K::Rp) {
                        loop {
                            args.push(self.expr(0)?);
                            if self.eat(&K::Rp) {
                                break;
                            }
                            self.take(K::Comma)?;
                        }
                    }
                    Ok(Expr::Call(Invoke::new(name, args)))
                } else if self.eat(&K::PlusPlus) {
                    Ok(Expr::PostIncrement(name))
                } else if self.eat(&K::MinusMinus) {
                    Ok(Expr::PostDecrement(name))
                } else {
                    Ok(Expr::Name(name))
                }
            }
            _ => self.err("expected expression"),
        }
    }
    fn binop(&self) -> Option<(u8, K)> {
        let k = self.peek()?.k.clone();
        let p = match k {
            K::OrOr => 1,
            K::AndAnd => 2,
            K::EqEq | K::Ne => 3,
            K::Lt | K::Le | K::Gt | K::Ge => 4,
            K::Plus | K::Minus => 5,
            K::Star | K::Slash | K::Percent => 6,
            _ => return None,
        };
        Some((p, k))
    }
    fn int(&mut self) -> Result<i64, MaryScriptError> {
        match self.peek().map(|t| &t.k) {
            Some(K::Int(v)) => {
                let v = *v;
                self.at += 1;
                Ok(v)
            }
            _ => self.err("expected integer"),
        }
    }
    fn id(&mut self) -> Result<String, MaryScriptError> {
        match self.peek().map(|t| &t.k) {
            Some(K::Id(v)) => {
                let v = v.clone();
                self.at += 1;
                Ok(v)
            }
            _ => self.err("expected identifier"),
        }
    }
    fn word(&mut self, w: &str) -> Result<(), MaryScriptError> {
        if self.eat_word(w) {
            Ok(())
        } else {
            self.err(&format!("expected '{w}'"))
        }
    }
    fn eat_word(&mut self, w: &str) -> bool {
        if self.is_word(w) {
            self.at += 1;
            true
        } else {
            false
        }
    }
    fn is_word(&self, w: &str) -> bool {
        matches!(self.peek().map(|t|&t.k),Some(K::Id(v))if v==w)
    }
    fn take(&mut self, k: K) -> Result<(), MaryScriptError> {
        if self.eat(&k) {
            Ok(())
        } else {
            self.err(&format!("expected {k:?}"))
        }
    }
    fn eat(&mut self, k: &K) -> bool {
        if self.is(k) {
            self.at += 1;
            true
        } else {
            false
        }
    }
    fn is(&self, k: &K) -> bool {
        self.peek().is_some_and(|t| &t.k == k)
    }
    fn peek(&self) -> Option<&Tok> {
        self.tokens.get(self.at)
    }
    fn end(&self) -> bool {
        self.at == self.tokens.len()
    }
    fn syntax(&self, m: &str) -> MaryScriptError {
        let (line, column) = self.peek().map_or_else(
            || self.tokens.last().map_or((1, 1), |t| (t.line, t.col + 1)),
            |t| (t.line, t.col),
        );
        MaryScriptError::Syntax {
            line,
            column,
            message: m.into(),
        }
    }
    fn err<T>(&self, m: &str) -> Result<T, MaryScriptError> {
        Err(self.syntax(m))
    }
}

fn lex(s: &str) -> Result<Vec<Tok>, MaryScriptError> {
    let b = s.as_bytes();
    let (mut i, mut line, mut col) = (0, 1, 1);
    let mut out = Vec::new();
    while i < b.len() {
        if b[i] == b'\n' {
            i += 1;
            line += 1;
            col = 1;
            continue;
        }
        if b[i].is_ascii_whitespace() {
            i += 1;
            col += 1;
            continue;
        }
        if b[i] == b'/' && b.get(i + 1) == Some(&b'/') {
            while i < b.len() && b[i] != b'\n' {
                i += 1;
                col += 1
            }
            continue;
        }
        if b[i] == b'/' && b.get(i + 1) == Some(&b'*') {
            let (l, c) = (line, col);
            i += 2;
            col += 2;
            while i + 1 < b.len() && !(b[i] == b'*' && b[i + 1] == b'/') {
                if b[i] == b'\n' {
                    i += 1;
                    line += 1;
                    col = 1
                } else {
                    i += 1;
                    col += 1
                }
            }
            if i + 1 >= b.len() {
                return Err(MaryScriptError::Syntax {
                    line: l,
                    column: c,
                    message: "unterminated comment".into(),
                });
            }
            i += 2;
            col += 2;
            continue;
        }
        let c0 = col;
        let k = if b[i] == b'_' || b[i].is_ascii_alphabetic() {
            let st = i;
            while i < b.len() && (b[i] == b'_' || b[i].is_ascii_alphanumeric()) {
                i += 1;
                col += 1
            }
            K::Id(s[st..i].into())
        } else if b[i].is_ascii_digit() {
            let st = i;
            i += 1;
            col += 1;
            while i < b.len() && (b[i].is_ascii_hexdigit() || b[i] == b'x' || b[i] == b'X') {
                i += 1;
                col += 1
            }
            let t = &s[st..i];
            let v = if let Some(h) = t.strip_prefix("0x").or_else(|| t.strip_prefix("0X")) {
                i64::from_str_radix(h, 16)
            } else {
                t.parse()
            }
            .map_err(|_| MaryScriptError::Syntax {
                line,
                column: c0,
                message: format!("invalid integer {t}"),
            })?;
            K::Int(v)
        } else if b[i] == b'"' {
            i += 1;
            col += 1;
            let mut v = Vec::new();
            while i < b.len() && b[i] != b'"' {
                if b[i] == b'\\' {
                    i += 1;
                    col += 1;
                    if i >= b.len() {
                        break;
                    }
                    match b[i] {
                        b'n' => v.push(b'\n'),
                        b'r' => v.push(b'\r'),
                        b't' => v.push(b'\t'),
                        b'"' => v.push(b'"'),
                        b'\\' => v.push(b'\\'),
                        b'x' => {
                            if i + 2 >= b.len() {
                                return Err(MaryScriptError::Syntax {
                                    line,
                                    column: col,
                                    message: "short \\x escape".into(),
                                });
                            }
                            let h = std::str::from_utf8(&b[i + 1..i + 3]).unwrap();
                            v.push(u8::from_str_radix(h, 16).map_err(|_| {
                                MaryScriptError::Syntax {
                                    line,
                                    column: col,
                                    message: "invalid \\x escape".into(),
                                }
                            })?);
                            i += 2;
                            col += 2
                        }
                        x => v.push(x),
                    }
                } else {
                    v.push(b[i])
                }
                i += 1;
                col += 1
            }
            if i >= b.len() {
                return Err(MaryScriptError::Syntax {
                    line,
                    column: c0,
                    message: "unterminated string".into(),
                });
            }
            i += 1;
            col += 1;
            K::Str(v)
        } else {
            let two = b.get(i + 1).map(|n| (b[i], *n));
            if let Some(k) = match two {
                Some((b'+', b'=')) => Some(K::PlusEq),
                Some((b'-', b'=')) => Some(K::MinusEq),
                Some((b'*', b'=')) => Some(K::StarEq),
                Some((b'/', b'=')) => Some(K::SlashEq),
                Some((b'%', b'=')) => Some(K::PercentEq),
                Some((b'=', b'=')) => Some(K::EqEq),
                Some((b'!', b'=')) => Some(K::Ne),
                Some((b'<', b'=')) => Some(K::Le),
                Some((b'>', b'=')) => Some(K::Ge),
                Some((b'&', b'&')) => Some(K::AndAnd),
                Some((b'|', b'|')) => Some(K::OrOr),
                Some((b'+', b'+')) => Some(K::PlusPlus),
                Some((b'-', b'-')) => Some(K::MinusMinus),
                _ => None,
            } {
                i += 2;
                col += 2;
                k
            } else {
                let x = match b[i] {
                    b'(' => K::Lp,
                    b')' => K::Rp,
                    b'{' => K::Lb,
                    b'}' => K::Rb,
                    b'[' => K::Ls,
                    b']' => K::Rs,
                    b',' => K::Comma,
                    b';' => K::Semi,
                    b':' => K::Colon,
                    b'+' => K::Plus,
                    b'-' => K::Minus,
                    b'*' => K::Star,
                    b'/' => K::Slash,
                    b'%' => K::Percent,
                    b'!' => K::Bang,
                    b'=' => K::Eq,
                    b'<' => K::Lt,
                    b'>' => K::Gt,
                    _ => {
                        return Err(MaryScriptError::Syntax {
                            line,
                            column: c0,
                            message: format!("unexpected character '{}'", b[i] as char),
                        })
                    }
                };
                i += 1;
                col += 1;
                x
            }
        };
        out.push(Tok { k, line, col: c0 });
    }
    Ok(out)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{
        bytecode::encode_script,
        ir::{CallId, ValueType},
    };
    #[test]
    fn parses_nested_c_and_compiles() {
        let mut scope = ConstScope::new();
        scope.add_func("Get".into(), CallId(1), vec![ValueType::Integer]);
        scope.add_proc("Show".into(), CallId(2), vec![ValueType::Integer]);
        let src = r#"mary_script(7) void Wedding(void){int x=Get(1);if(x>0){switch(x){case 1:Show(x);break;default:do{x--;}while(x>0);break;}}return;}"#;
        let parsed = parse_scripts(src, &Options::default(), &scope).unwrap();
        assert_eq!(parsed.scripts[0].0, 7);
        assert!(!encode_script(&parsed.scripts[0].2).is_empty());
    }
    #[test]
    fn rejects_loop_break_instead_of_hiding_semantics() {
        let scope = ConstScope::new();
        let e = parse_scripts(
            "mary_script(1) void X(void){do{break;}while(1);}",
            &Options::default(),
            &scope,
        )
        .unwrap_err();
        assert!(e.to_string().contains("loop break is not supported"));
    }

    #[test]
    fn plain_function_name_uses_script_table_slot() {
        let scope = ConstScope::new();
        let table = crate::mary_c::parse_script_table(
            "mary_script_table { Opening, NULL, Wedding }",
            &Options::default(),
        )
        .unwrap();
        let parsed = parse_named_scripts(
            "void Wedding(void){return;}",
            &Options::default(),
            &scope,
            &table,
        )
        .unwrap();
        assert_eq!(parsed.scripts[0].0, 2);
        assert_eq!(parsed.scripts[0].1, "Wedding");
    }

    #[test]
    fn script_symbol_and_numeric_slot_compile_identically() {
        let mut scope = ConstScope::new();
        scope.add_proc("VerifiedNative".into(), CallId(9), vec![ValueType::Integer]);
        let table = crate::mary_c::parse_script_table(
            "mary_script_table { Opening, NULL, Wedding }",
            &Options::default(),
        )
        .unwrap();
        let symbolic = parse_named_scripts(
            "void Wedding(void){VerifiedNative(Wedding);}",
            &Options::default(),
            &scope,
            &table,
        )
        .unwrap();
        let numeric = parse_named_scripts(
            "void Wedding(void){VerifiedNative(2);}",
            &Options::default(),
            &scope,
            &table,
        )
        .unwrap();
        assert_eq!(
            encode_script(&symbolic.scripts[0].2),
            encode_script(&numeric.scripts[0].2),
        );
    }

    #[test]
    fn mary_c_string_escapes_are_encoded_only_by_the_charmap() {
        let map = Charmap::parse("41=A\nAA=\\n\nBB=\"\nCC=\\\n05={Press}\n").unwrap();
        let mut scope = ConstScope::new();
        scope.add_proc("Text".into(), CallId(1), vec![ValueType::String]);
        let table =
            crate::mary_c::parse_script_table("mary_script_table { Script }", &Options::default())
                .unwrap();
        let parsed = parse_named_scripts_with_charmap(
            r#"mary_text_table { const char T[] = "A\n\"\\{Press}"; }; void Script(void){Text(T);}"#,
            &Options::default(),
            &scope,
            &table,
            &map,
        )
        .unwrap();
        assert_eq!(
            parsed.scripts[0].2.strings[0],
            vec![0x41, 0xAA, 0xBB, 0xCC, 0x05]
        );
    }
}
