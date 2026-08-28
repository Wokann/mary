use std::fmt::{self, Write};

use thiserror::Error;

use crate::{
    ast::{Expr, Stmt, SwitchCase, SwitchLayout},
    charmap::Charmap,
    ir::IntValue,
    pretty_print::{PrettyExpr, PrettyStringLit},
};

#[derive(Debug, Error, PartialEq, Eq)]
pub enum PrettyCError {
    #[error("low-level ir cannot be represented as lossless high-level Mary-C")]
    LowLevelIr,
    #[error("jump-next cannot be represented as lossless high-level Mary-C")]
    JumpNext,
    #[error("formatting failed")]
    Format,
}

pub struct PrettyCStmts<'a> {
    statements: &'a [Stmt],
    indent: usize,
}
impl<'a> PrettyCStmts<'a> {
    pub fn new(statements: &'a [Stmt]) -> Self {
        Self {
            statements,
            indent: 0,
        }
    }
    pub fn with_indent(statements: &'a [Stmt], indent: usize) -> Self {
        Self { statements, indent }
    }
}

impl fmt::Display for PrettyCStmts<'_> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let mut s = String::new();
        format_stmts(&mut s, self.statements, self.indent, &mut Vec::new())
            .map_err(|_| fmt::Error)?;
        f.write_str(&s)
    }
}

pub fn format_script(
    id: IntValue,
    name: &str,
    statements: &[Stmt],
) -> Result<String, PrettyCError> {
    let mut out = format!("mary_script({id}) void {name}(void)\n{{\n");
    format_stmts(&mut out, statements, 1, &mut Vec::new())?;
    out.push_str("}\n");
    Ok(out)
}

/// Format the canonical Mary-C form. The script ID lives in the separate
/// `mary_script_table`, so the definition itself only carries its symbol.
pub fn format_named_script(name: &str, statements: &[Stmt]) -> Result<String, PrettyCError> {
    format_named_script_inner(name, statements, None)
}

pub fn format_named_script_with_charmap(
    name: &str,
    statements: &[Stmt],
    charmap: &Charmap,
) -> Result<String, PrettyCError> {
    format_named_script_inner(name, statements, Some(charmap))
}

fn format_named_script_inner(
    name: &str,
    statements: &[Stmt],
    charmap: Option<&Charmap>,
) -> Result<String, PrettyCError> {
    let text_count = statements
        .iter()
        .take_while(|stmt| matches!(stmt, Stmt::Consts(items) if items.len() == 1 && matches!(items[0].1, Expr::Str(_))))
        .count();
    let mut out = String::new();
    if text_count != 0 {
        out.push_str("mary_text_table\n{\n");
        for stmt in &statements[..text_count] {
            let Stmt::Consts(items) = stmt else {
                unreachable!()
            };
            writeln!(out, "    {},", items[0].0).map_err(|_| PrettyCError::Format)?;
        }
        out.push_str("};\n\n");
        for stmt in &statements[..text_count] {
            let Stmt::Consts(items) = stmt else {
                unreachable!()
            };
            if let Expr::Str(bytes) = &items[0].1 {
                let literal = match charmap {
                    Some(charmap) => PrettyStringLit::with_charmap(bytes, charmap),
                    None => PrettyStringLit::new(bytes),
                };
                writeln!(out, "const char {}[] =", items[0].0).map_err(|_| PrettyCError::Format)?;
                let fragments = literal.source_fragments();
                for (index, fragment) in fragments.iter().enumerate() {
                    writeln!(
                        out,
                        "    \"{fragment}\"{}",
                        if index + 1 == fragments.len() {
                            ";"
                        } else {
                            ""
                        }
                    )
                    .map_err(|_| PrettyCError::Format)?;
                }
            } else {
                writeln!(out, "const char {}[] = {};", items[0].0, expr(&items[0].1))
                    .map_err(|_| PrettyCError::Format)?;
            }
        }
        out.push('\n');
    }
    writeln!(out, "void {name}(void)\n{{").map_err(|_| PrettyCError::Format)?;
    format_stmts(&mut out, &statements[text_count..], 1, &mut Vec::new())?;
    out.push_str("}\n");
    Ok(out)
}

#[derive(Clone, Copy)]
enum Control {
    Switch,
    Loop,
}

fn format_stmts(
    out: &mut String,
    statements: &[Stmt],
    indent: usize,
    controls: &mut Vec<Control>,
) -> Result<(), PrettyCError> {
    for stmt in statements {
        format_stmt(out, stmt, indent, controls)?;
    }
    Ok(())
}
fn pad(out: &mut String, n: usize) {
    for _ in 0..n {
        out.push_str("    ")
    }
}
fn expr(e: &Expr) -> String {
    format!("{}", PrettyExpr::new(e))
}

fn inline_stmt(stmt: &Stmt) -> Result<String, PrettyCError> {
    Ok(match stmt {
        Stmt::Vars(v) => {
            let mut s = "int ".to_owned();
            for (i, (n, e)) in v.iter().enumerate() {
                if i > 0 {
                    s.push_str(", ")
                }
                s.push_str(n);
                if let Some(e) = e {
                    write!(s, " = {}", expr(e)).unwrap()
                }
            }
            s
        }
        Stmt::Assign(op, n, e) => format!("{n} {op} {}", expr(e)),
        Stmt::Expr(e) => expr(e),
        _ => return Err(PrettyCError::Format),
    })
}

fn format_stmt(
    out: &mut String,
    stmt: &Stmt,
    n: usize,
    controls: &mut Vec<Control>,
) -> Result<(), PrettyCError> {
    pad(out, n);
    match stmt {
        Stmt::Vars(_) => {
            out.push_str(&inline_stmt(stmt)?);
            out.push_str(";\n")
        }
        Stmt::Consts(v) => {
            if v.is_empty() {
                out.push_str("/* empty const declaration */\n")
            } else {
                for (i, (name, e)) in v.iter().enumerate() {
                    if i > 0 {
                        pad(out, n)
                    }
                    let ty = if matches!(e, Expr::Str(_)) {
                        "const char *"
                    } else {
                        "const int "
                    };
                    writeln!(out, "{ty}{name} = {};", expr(e)).map_err(|_| PrettyCError::Format)?;
                }
            }
        }
        Stmt::Assign(op, name, e) => {
            writeln!(out, "{name} {op} {};", expr(e)).map_err(|_| PrettyCError::Format)?
        }
        Stmt::AssignNoDisc(op, name, e) => writeln!(out, "mary_nodisc({name} {op} {});", expr(e))
            .map_err(|_| PrettyCError::Format)?,
        Stmt::Expr(e) => writeln!(out, "{};", expr(e)).map_err(|_| PrettyCError::Format)?,
        Stmt::Call(c) => {
            write!(out, "{}(", c.func).map_err(|_| PrettyCError::Format)?;
            for (i, a) in c.args.iter().enumerate() {
                if i > 0 {
                    out.push_str(", ")
                }
                out.push_str(&expr(a));
            }
            out.push_str(");\n")
        }
        Stmt::If(e, yes) => {
            writeln!(out, "if ({})", expr(e)).map_err(|_| PrettyCError::Format)?;
            block(out, yes, n, controls)?
        }
        Stmt::IfElse(e, yes, no) => {
            writeln!(out, "if ({})", expr(e)).map_err(|_| PrettyCError::Format)?;
            block(out, yes, n, controls)?;
            pad(out, n);
            out.push_str("else\n");
            block(out, no, n, controls)?
        }
        Stmt::For(v) => {
            let (e, init, iter, body) = &**v;
            writeln!(
                out,
                "for ({}; {}; {})",
                inline_stmt(init)?,
                expr(e),
                inline_stmt(iter)?
            )
            .map_err(|_| PrettyCError::Format)?;
            controls.push(Control::Loop);
            block(out, body, n, controls)?;
            controls.pop();
        }
        Stmt::DoWhile(e, body) => {
            out.push_str("do\n");
            controls.push(Control::Loop);
            block(out, body, n, controls)?;
            controls.pop();
            pad(out, n);
            writeln!(out, "while ({});", expr(e)).map_err(|_| PrettyCError::Format)?
        }
        Stmt::Switch(e, cases, _, layout) => {
            let head = if *layout == SwitchLayout::Compact {
                "mary_switch_compact"
            } else {
                "switch"
            };
            writeln!(out, "{head} ({})", expr(e)).map_err(|_| PrettyCError::Format)?;
            pad(out, n);
            out.push_str("{\n");
            controls.push(Control::Switch);
            for case in cases {
                format_case(out, case, n + 1, controls)?;
            }
            controls.pop();
            pad(out, n);
            out.push_str("}\n")
        }
        Stmt::Break => match controls.last() {
            Some(Control::Switch) => out.push_str("break;\n"),
            Some(Control::Loop) => out.push_str("mary_break_switch;\n"),
            None => return Err(PrettyCError::Format),
        },
        Stmt::Exit => out.push_str("return;\n"),
        Stmt::Ir(_) => return Err(PrettyCError::LowLevelIr),
        Stmt::JumpNext => return Err(PrettyCError::JumpNext),
    }
    Ok(())
}

fn block(
    out: &mut String,
    body: &[Stmt],
    n: usize,
    controls: &mut Vec<Control>,
) -> Result<(), PrettyCError> {
    pad(out, n);
    out.push_str("{\n");
    format_stmts(out, body, n + 1, controls)?;
    pad(out, n);
    out.push_str("}\n");
    Ok(())
}
fn format_case(
    out: &mut String,
    case: &SwitchCase,
    n: usize,
    controls: &mut Vec<Control>,
) -> Result<(), PrettyCError> {
    let (body, automatic_break) = match case {
        SwitchCase::Case(values, b) => {
            for v in values {
                pad(out, n);
                writeln!(out, "case {}:", expr(v)).map_err(|_| PrettyCError::Format)?
            }
            (b, true)
        }
        SwitchCase::Fallthrough(values, b) => {
            for v in values {
                pad(out, n);
                writeln!(out, "case {}:", expr(v)).map_err(|_| PrettyCError::Format)?
            }
            (b, false)
        }
        SwitchCase::Default(b) => {
            pad(out, n);
            out.push_str("default:\n");
            (b, true)
        }
        SwitchCase::DefaultFallthrough(b) => {
            pad(out, n);
            out.push_str("default:\n");
            (b, false)
        }
        SwitchCase::ImplicitDefault(b) => {
            pad(out, n);
            out.push_str("mary_implicit_default:\n");
            (b, false)
        }
        SwitchCase::DeadJump(b) => {
            pad(out, n);
            out.push_str("mary_dead_jump:\n");
            (b, false)
        }
    };
    format_stmts(out, body, n + 1, controls)?;
    if automatic_break && !matches!(body.last(), Some(Stmt::Break | Stmt::Exit)) {
        pad(out, n + 1);
        out.push_str("break;\n")
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::mary_c::{parse_scripts, Options};
    use crate::{
        ast::{Invoke, SwitchLayout},
        bytecode::encode_script,
        compiler::compile_script,
        const_scope::ConstScope,
        ir::{CallId, SwitchId, ValueType},
    };
    #[test]
    fn printed_c_is_byte_exact_when_reparsed() {
        let mut scope = ConstScope::new();
        scope.add_proc("P".into(), CallId(3), vec![ValueType::Integer]);
        let ast = vec![
            Stmt::Vars(vec![("x".into(), Some(Expr::Int(2)))]),
            Stmt::Switch(
                Expr::Name("x".into()),
                vec![
                    SwitchCase::Case(
                        vec![Expr::Int(2)],
                        vec![Stmt::Call(Invoke::new(
                            "P".into(),
                            vec![Expr::Name("x".into())],
                        ))],
                    ),
                    SwitchCase::DefaultFallthrough(vec![Stmt::Exit]),
                ],
                SwitchId(0),
                SwitchLayout::Standard,
            ),
        ];
        let text = format_script(9, "RoundTrip", &ast).unwrap();
        let original = compile_script(ast, &scope).unwrap();
        let parsed = parse_scripts(&text, &Options::default(), &scope).unwrap();
        assert_eq!(
            encode_script(&original),
            encode_script(&parsed.scripts[0].2)
        );
    }
}
