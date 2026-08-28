use std::{fmt, slice};

use crate::{
    ast::{Expr, Invoke, IrArg, Stmt, SwitchCase},
    charmap::Charmap,
};

#[derive(Debug, PartialEq, PartialOrd)]
enum Precedence {
    // Primary,
    // Call,
    Unary,
    Multiplicative,
    Additive,
    Relational,
    Equality,
    And,
    Or,
    Top,
}

#[derive(Debug)]
pub struct PrettyExpr<'a>(&'a Expr, Precedence, Option<&'a Charmap>);

impl<'a> PrettyExpr<'a> {
    pub fn new(expr: &'a Expr) -> Self {
        Self(expr, Precedence::Top, None)
    }

    pub fn with_charmap(expr: &'a Expr, charmap: &'a Charmap) -> Self {
        Self(expr, Precedence::Top, Some(charmap))
    }

    fn nested(&self, expr: &'a Expr, precedence: Precedence) -> Self {
        Self(expr, precedence, self.2)
    }
}

impl<'a> fmt::Display for PrettyExpr<'a> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self.0 {
            Expr::Name(name) => write!(f, "{0}", name),
            Expr::Int(int_value) => write!(f, "{0}", int_value),
            Expr::Str(items) => PrettyStringLit::with_optional_charmap(items, self.2).fmt(f),

            Expr::OpAdd(exprs) => {
                let pl = self.nested(&exprs.0, Precedence::Additive);
                let pr = self.nested(&exprs.1, Precedence::Multiplicative);
                self.fmt_binop(f, pl, "+", pr, Precedence::Additive)
            }

            Expr::OpSub(exprs) => {
                let pl = self.nested(&exprs.0, Precedence::Additive);
                let pr = self.nested(&exprs.1, Precedence::Multiplicative);
                self.fmt_binop(f, pl, "-", pr, Precedence::Additive)
            }

            Expr::OpMul(exprs) => {
                let pl = self.nested(&exprs.0, Precedence::Multiplicative);
                let pr = self.nested(&exprs.1, Precedence::Unary);
                self.fmt_binop(f, pl, "*", pr, Precedence::Multiplicative)
            }

            Expr::OpDiv(exprs) => {
                let pl = self.nested(&exprs.0, Precedence::Multiplicative);
                let pr = self.nested(&exprs.1, Precedence::Unary);
                self.fmt_binop(f, pl, "/", pr, Precedence::Multiplicative)
            }

            Expr::OpMod(exprs) => {
                let pl = self.nested(&exprs.0, Precedence::Multiplicative);
                let pr = self.nested(&exprs.1, Precedence::Unary);
                self.fmt_binop(f, pl, "%", pr, Precedence::Multiplicative)
            }

            Expr::OpOr(exprs) => {
                let pl = self.nested(&exprs.0, Precedence::Or);
                let pr = self.nested(&exprs.1, Precedence::And);
                self.fmt_binop(f, pl, "||", pr, Precedence::Or)
            }

            Expr::OpAnd(exprs) => {
                let pl = self.nested(&exprs.0, Precedence::And);
                let pr = self.nested(&exprs.1, Precedence::Equality);
                self.fmt_binop(f, pl, "&&", pr, Precedence::And)
            }

            Expr::OpNeg(expr) => {
                let pi = self.nested(expr, Precedence::Unary);
                write!(f, "-{0}", pi)
            }

            Expr::OpNot(expr) => {
                let pi = self.nested(expr, Precedence::Unary);
                write!(f, "!{0}", pi)
            }

            Expr::PostIncrement(name) => self.fmt_prepost(f, "", name, "++"),
            Expr::PreIncrement(name) => self.fmt_prepost(f, "++", name, ""),
            Expr::PostDecrement(name) => self.fmt_prepost(f, "", name, "--"),
            Expr::PreDecrement(name) => self.fmt_prepost(f, "--", name, ""),

            Expr::CmpEq(exprs) => {
                let pl = self.nested(&exprs.0, Precedence::Equality);
                let pr = self.nested(&exprs.1, Precedence::Relational);
                self.fmt_binop(f, pl, "==", pr, Precedence::Equality)
            }

            Expr::CmpNe(exprs) => {
                let pl = self.nested(&exprs.0, Precedence::Equality);
                let pr = self.nested(&exprs.1, Precedence::Relational);
                self.fmt_binop(f, pl, "!=", pr, Precedence::Equality)
            }

            Expr::CmpLt(exprs) => {
                let pl = self.nested(&exprs.0, Precedence::Relational);
                let pr = self.nested(&exprs.1, Precedence::Additive);
                self.fmt_binop(f, pl, "<", pr, Precedence::Relational)
            }

            Expr::CmpLe(exprs) => {
                let pl = self.nested(&exprs.0, Precedence::Relational);
                let pr = self.nested(&exprs.1, Precedence::Additive);
                self.fmt_binop(f, pl, "<=", pr, Precedence::Relational)
            }

            Expr::CmpGe(exprs) => {
                let pl = self.nested(&exprs.0, Precedence::Relational);
                let pr = self.nested(&exprs.1, Precedence::Additive);
                self.fmt_binop(f, pl, ">=", pr, Precedence::Relational)
            }

            Expr::CmpGt(exprs) => {
                let pl = self.nested(&exprs.0, Precedence::Relational);
                let pr = self.nested(&exprs.1, Precedence::Additive);
                self.fmt_binop(f, pl, ">", pr, Precedence::Relational)
            }

            Expr::Call(invoke) => fmt_invoke_with_charmap(f, invoke, self.2),
        }
    }
}

impl<'a> PrettyExpr<'a> {
    fn fmt_binop(
        &self,
        f: &mut fmt::Formatter<'_>,
        pl: PrettyExpr,
        op: &str,
        pr: PrettyExpr,
        precedence: Precedence,
    ) -> fmt::Result {
        if self.1 < precedence {
            write!(f, "({0} {2} {1})", pl, pr, op)
        } else {
            write!(f, "{0} {2} {1}", pl, pr, op)
        }
    }

    fn fmt_prepost(
        &self,
        f: &mut fmt::Formatter<'_>,
        pre: &str,
        name: &str,
        post: &str,
    ) -> fmt::Result {
        if self.1 < Precedence::Top {
            write!(f, "({1}{0}{2})", name, pre, post)
        } else {
            write!(f, "{1}{0}{2}", name, pre, post)
        }
    }
}

#[derive(Debug)]
pub struct PrettyStmts<'a>(&'a [Stmt], usize, Option<&'a Charmap>);

impl<'a> PrettyStmts<'a> {
    pub fn new(stmts: &'a [Stmt]) -> Self {
        Self(stmts, 0, None)
    }

    pub fn with_indent(stmts: &'a [Stmt], indent: usize) -> Self {
        Self(stmts, indent, None)
    }

    pub fn with_charmap(stmts: &'a [Stmt], indent: usize, charmap: &'a Charmap) -> Self {
        Self(stmts, indent, Some(charmap))
    }

    fn expr(&self, expr: &'a Expr) -> PrettyExpr<'a> {
        match self.2 {
            Some(charmap) => PrettyExpr::with_charmap(expr, charmap),
            None => PrettyExpr::new(expr),
        }
    }

    fn stmts(&self, stmts: &'a [Stmt], indent: usize) -> PrettyStmts<'a> {
        Self(stmts, indent, self.2)
    }
}

impl<'a> fmt::Display for PrettyStmts<'a> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let indent_string = "    ".repeat(self.1);

        let mut first = true;

        for stmt in self.0 {
            if !first {
                /* statement separator */
                writeln!(f)?;
            }

            match stmt {
                Stmt::Vars(items) => {
                    /* var {name}[ = {expr}][, ...] */
                    write!(f, "{indent_string}var ")?;

                    let (name, expr) = &items[0];

                    write!(f, "{name}")?;

                    if let Some(expr) = expr {
                        let pretty_expr = self.expr(expr);
                        write!(f, " = {pretty_expr}")?;
                    }

                    for (name, expr) in &items[1..] {
                        write!(f, ", {name}")?;

                        if let Some(expr) = expr {
                            let pretty_expr = self.expr(expr);
                            write!(f, " = {pretty_expr}")?;
                        }
                    }
                }

                Stmt::Consts(items) => {
                    /* const {name} = {expr}[, ...] */
                    write!(f, "{indent_string}const ")?;

                    let (name, expr) = &items[0];
                    if items.len() == 1 {
                        if let (Expr::Str(value), Some(charmap)) = (expr, self.2) {
                            let fragments =
                                PrettyStringLit::with_charmap(value, charmap).fragments();
                            writeln!(f, "{name} =")?;
                            let continuation_indent = "    ".repeat(self.1 + 1);
                            for (index, fragment) in fragments.iter().enumerate() {
                                write!(f, "{continuation_indent}\"{fragment}\"")?;
                                if index + 1 < fragments.len() {
                                    writeln!(f)?;
                                }
                            }
                            first = false;
                            continue;
                        }
                    }
                    let pretty_expr = self.expr(expr);
                    write!(f, "{name} = {pretty_expr}")?;

                    for (name, expr) in &items[1..] {
                        let pretty_expr = self.expr(expr);
                        write!(f, ", {name} = {pretty_expr}")?;
                    }
                }

                Stmt::Assign(assign_operation, var_name, expr) => {
                    write!(f, "{indent_string}{var_name} {assign_operation}")?;

                    let pretty_expr = self.expr(expr);
                    write!(f, " {pretty_expr}")?;
                }

                Stmt::AssignNoDisc(assign_operation, var_name, expr) => {
                    write!(f, "{indent_string}nodisc {var_name} {assign_operation}")?;

                    let pretty_expr = self.expr(expr);
                    write!(f, " {pretty_expr}")?;
                }

                Stmt::Expr(expr) => {
                    let pretty_expr = self.expr(expr);
                    match expr {
                        Expr::PostIncrement(_)
                        | Expr::PreIncrement(_)
                        | Expr::PostDecrement(_)
                        | Expr::PreDecrement(_) => {
                            write!(f, "{indent_string}{pretty_expr}")?;
                        }
                        _ => {
                            write!(f, "{indent_string}discard {pretty_expr}")?;
                        }
                    }
                }

                Stmt::Call(invoke) => {
                    write!(f, "{indent_string}")?;
                    fmt_invoke_with_charmap(f, invoke, self.2)?;
                }

                Stmt::If(expr, stmts) => {
                    let pretty_expr = self.expr(expr);

                    writeln!(f, "{indent_string}if {pretty_expr}")?;
                    writeln!(f, "{indent_string}{{")?;

                    let pretty_body = self.stmts(&stmts[..], self.1 + 1);
                    writeln!(f, "{pretty_body}")?;

                    write!(f, "{indent_string}}}")?;
                }

                Stmt::IfElse(expr, stmts0, stmts1) => {
                    let pretty_expr = self.expr(expr);

                    writeln!(f, "{indent_string}if {pretty_expr}")?;
                    writeln!(f, "{indent_string}{{")?;

                    let pretty_body = self.stmts(&stmts0[..], self.1 + 1);
                    writeln!(f, "{pretty_body}")?;

                    writeln!(f, "{indent_string}}}")?;

                    match &stmts1[..] {
                        [Stmt::IfElse(_, _, _)] | [Stmt::If(_, _)] => {
                            /* special case, we flatten this else if chain
                             * TODO: support in compiler */

                            let pretty_body = self.stmts(&stmts1[..], self.1);

                            /* HACK: we don't want the include the indent of the body, so we eat it manually */
                            let body_string = format!("{pretty_body}");
                            let body_string = &body_string[indent_string.len()..];

                            write!(f, "{indent_string}else {body_string}")?;
                        }

                        _ => {
                            writeln!(f, "{indent_string}else")?;
                            writeln!(f, "{indent_string}{{")?;

                            let pretty_body = self.stmts(&stmts1[..], self.1 + 1);
                            writeln!(f, "{pretty_body}")?;

                            write!(f, "{indent_string}}}")?;
                        }
                    }
                }

                Stmt::For(for_box) => {
                    let (expr, stmt_init, stmt_iter, body) = &**for_box;

                    let pretty_init = self.stmts(slice::from_ref(stmt_init), 0);
                    let pretty_expr = self.expr(expr);
                    let pretty_iter = self.stmts(slice::from_ref(stmt_iter), 0);

                    writeln!(
                        f,
                        "{indent_string}for {pretty_init}; {pretty_expr}; {pretty_iter}"
                    )?;

                    writeln!(f, "{indent_string}{{")?;

                    let pretty_body = self.stmts(&body[..], self.1 + 1);
                    writeln!(f, "{pretty_body}")?;

                    write!(f, "{indent_string}}}")?;
                }

                Stmt::DoWhile(expr, stmts) => {
                    writeln!(f, "{indent_string}do")?;
                    writeln!(f, "{indent_string}{{")?;

                    let pretty_body = self.stmts(&stmts[..], self.1 + 1);
                    writeln!(f, "{pretty_body}")?;

                    let pretty_expr = self.expr(expr);
                    write!(f, "{indent_string}}} while {pretty_expr}")?;
                }

                Stmt::Switch(expr, switch_cases, _, layout) => {
                    let pretty_expr = self.expr(expr);

                    let layout_marker = if *layout == crate::ast::SwitchLayout::Compact {
                        " compact"
                    } else {
                        ""
                    };
                    writeln!(f, "{indent_string}switch{layout_marker} {pretty_expr}")?;
                    writeln!(f, "{indent_string}{{")?;

                    for switch_case in switch_cases {
                        fmt_switch_case_with_charmap(f, switch_case, self.1 + 1, self.2)?;
                        writeln!(f)?;
                    }

                    write!(f, "{indent_string}}}")?;
                }

                Stmt::Ir(items) => {
                    writeln!(f, "{indent_string}ir")?;
                    writeln!(f, "{indent_string}{{")?;
                    for item in items {
                        write!(f, "{indent_string}    {}(", item.name)?;
                        for (index, arg) in item.args.iter().enumerate() {
                            if index != 0 {
                                write!(f, ", ")?;
                            }
                            match arg {
                                IrArg::Int(value) => write!(f, "{value}")?,
                                IrArg::Str(value) => {
                                    PrettyStringLit::with_optional_charmap(value, self.2).fmt(f)?
                                }
                            }
                        }
                        writeln!(f, ")")?;
                    }
                    write!(f, "{indent_string}}}")?;
                }

                Stmt::JumpNext => write!(f, "{indent_string}jump next")?,
                Stmt::Break => write!(f, "{indent_string}break")?,
                Stmt::Exit => write!(f, "{indent_string}exit")?,
            }

            first = false;
        }

        Ok(())
    }
}

pub struct PrettyStringLit<'a>(&'a [u8], Option<&'a Charmap>);

impl<'a> PrettyStringLit<'a> {
    pub fn new(string_lit: &'a [u8]) -> Self {
        Self(string_lit, None)
    }

    pub fn with_charmap(string_lit: &'a [u8], charmap: &'a Charmap) -> Self {
        Self(string_lit, Some(charmap))
    }

    fn with_optional_charmap(string_lit: &'a [u8], charmap: Option<&'a Charmap>) -> Self {
        Self(string_lit, charmap)
    }

    fn escaped_text(text: &str) -> String {
        let mut output = String::new();
        for ch in text.chars() {
            match ch {
                '\x07' => output.push_str("\\a"),
                '\x08' => output.push_str("\\b"),
                '\x0C' => output.push_str("\\f"),
                '"' => output.push_str("\\\""),
                '\n' => output.push_str("\\n"),
                '\r' => output.push_str("\\r"),
                '\t' => output.push_str("\\t"),
                '\x0B' => output.push_str("\\v"),
                '\\' => output.push_str("\\\\"),
                _ => output.push(ch),
            }
        }
        output
    }

    fn fragments(&self) -> Vec<String> {
        let mut fragments = Vec::new();
        let mut fragment = String::new();
        let mut split_before_next_text = false;
        let mut at = 0;

        while at < self.0.len() {
            if let Some(charmap) = self.1 {
                if let Some((count, text)) = charmap.decode_one(&self.0[at..]) {
                    let named_escape = charmap.is_named_escape(text);
                    if split_before_next_text && !named_escape && !fragment.is_empty() {
                        fragments.push(std::mem::take(&mut fragment));
                    }
                    if named_escape {
                        fragment.push_str(text);
                        split_before_next_text = true;
                    } else {
                        fragment.push_str(&Self::escaped_text(text));
                        split_before_next_text = false;
                    }
                    at += count;
                    continue;
                }

                if split_before_next_text && !fragment.is_empty() {
                    fragments.push(std::mem::take(&mut fragment));
                    split_before_next_text = false;
                }
                let count = charmap.raw_sequence_len(&self.0[at..]);
                for byte in &self.0[at..at + count] {
                    fragment.push_str(&format!("\\x{byte:02X}"));
                }
                at += count;
                continue;
            }

            let byte = self.0[at];
            match byte {
                b'"' => fragment.push_str("\\\""),
                b'\n' => fragment.push_str("\\n"),
                b'\r' => fragment.push_str("\\r"),
                b'\t' => fragment.push_str("\\t"),
                b'\\' => fragment.push_str("\\\\"),
                0x20..0x7F => fragment.push(byte as char),
                _ => fragment.push_str(&format!("\\x{byte:02X}")),
            }
            at += 1;
        }

        if !fragment.is_empty() || fragments.is_empty() {
            fragments.push(fragment);
        }
        fragments
    }
}

impl<'a> fmt::Display for PrettyStringLit<'a> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "\"{}\"", self.fragments().concat())
    }
}

#[cfg(test)]
mod string_lit_tests {
    use super::{PrettyStmts, PrettyStringLit};
    use crate::ast::{Expr, Stmt};
    use crate::charmap::Charmap;

    #[test]
    fn multibyte_controls_follow_the_map_instead_of_a_hardcoded_prefix() {
        let map = Charmap::parse("20= \n21=!\nAA21={Player}\nAA22={Horse}\n").unwrap();

        assert_eq!(
            PrettyStringLit::with_charmap(&[0xAA, 0x21, 0xAA, 0x22], &map).to_string(),
            "\"{Player}{Horse}\""
        );
        assert_eq!(
            PrettyStringLit::with_charmap(&[0xAA, 0x20, 0x21], &map).to_string(),
            "\"\\xAA\\x20!\""
        );
        assert_eq!(
            PrettyStringLit::with_charmap(&[0xAA], &map).to_string(),
            "\"\\xAA\""
        );
    }

    #[test]
    fn named_escape_spelling_comes_from_the_map() {
        let map = Charmap::parse("0102=\\p\n030405=\\l\n").unwrap();

        assert_eq!(
            PrettyStringLit::with_charmap(&[1, 2, 3, 4, 5], &map).to_string(),
            "\"\\p\\l\""
        );
    }

    #[test]
    fn message_constants_split_after_escape_runs_and_align_fragments() {
        let map = Charmap::parse("41=A\n42=B\n0D=\\r\n0A=\\n\n05={END}\n").unwrap();
        let stmts = vec![
            Stmt::Consts(vec![(
                "MESSAGE_0".into(),
                Expr::Str(vec![0x41, 0x0D, 0x0A, 0x42, 0x05]),
            )]),
            Stmt::Consts(vec![("MESSAGE_1".into(), Expr::Str(vec![0x42, 0x05]))]),
        ];

        assert_eq!(
            PrettyStmts::with_charmap(&stmts, 1, &map).to_string(),
            "    const MESSAGE_0 =\n        \"A\\r\\n\"\n        \"B{END}\"\n    const MESSAGE_1 =\n        \"B{END}\""
        );
    }
}

fn fmt_args(f: &mut fmt::Formatter<'_>, args: &[Expr], charmap: Option<&Charmap>) -> fmt::Result {
    let mut first = true;

    for arg in args {
        let pretty_expr = match charmap {
            Some(charmap) => PrettyExpr::with_charmap(arg, charmap),
            None => PrettyExpr::new(arg),
        };

        if first {
            write!(f, "{pretty_expr}")
        } else {
            write!(f, ", {pretty_expr}")
        }?;

        first = false;
    }

    Ok(())
}

// TODO: PrettyInvoke
pub(crate) fn fmt_invoke(f: &mut fmt::Formatter<'_>, invoke: &Invoke) -> fmt::Result {
    fmt_invoke_with_charmap(f, invoke, None)
}

fn fmt_invoke_with_charmap(
    f: &mut fmt::Formatter<'_>,
    invoke: &Invoke,
    charmap: Option<&Charmap>,
) -> fmt::Result {
    write!(f, "{0}(", invoke.func)?;
    fmt_args(f, &invoke.args, charmap)?;
    write!(f, ")")
}

// TODO: PrettySwitchCase
pub(crate) fn fmt_switch_case(
    f: &mut fmt::Formatter<'_>,
    switch_case: &SwitchCase,
    indent: usize,
) -> Result<(), fmt::Error> {
    fmt_switch_case_with_charmap(f, switch_case, indent, None)
}

fn fmt_switch_case_with_charmap(
    f: &mut fmt::Formatter<'_>,
    switch_case: &SwitchCase,
    indent: usize,
    charmap: Option<&Charmap>,
) -> Result<(), fmt::Error> {
    let indent_string = "    ".repeat(indent);

    Ok(match switch_case {
        SwitchCase::Case(exprs, stmts) => {
            write!(f, "{indent_string}case ")?;
            fmt_args(f, exprs, charmap)?;
            writeln!(f)?;

            writeln!(f, "{indent_string}{{")?;

            if stmts.len() > 0 {
                let pretty_body = PrettyStmts(&stmts[..], indent + 1, charmap);
                writeln!(f, "{pretty_body}")?;
            }

            write!(f, "{indent_string}}}")?;
        }

        SwitchCase::Default(stmts) => {
            writeln!(f, "{indent_string}default")?;
            writeln!(f, "{indent_string}{{")?;

            let pretty_body = PrettyStmts(&stmts[..], indent + 1, charmap);
            writeln!(f, "{pretty_body}")?;

            write!(f, "{indent_string}}}")?;
        }

        SwitchCase::DefaultFallthrough(stmts) => {
            writeln!(f, "{indent_string}default fallthrough")?;
            writeln!(f, "{indent_string}{{")?;
            if !stmts.is_empty() {
                let pretty_body = PrettyStmts(&stmts[..], indent + 1, charmap);
                writeln!(f, "{pretty_body}")?;
            }
            write!(f, "{indent_string}}}")?;
        }

        SwitchCase::ImplicitDefault(stmts) => {
            writeln!(f, "{indent_string}default implicit")?;
            writeln!(f, "{indent_string}{{")?;
            if !stmts.is_empty() {
                let pretty_body = PrettyStmts(&stmts[..], indent + 1, charmap);
                writeln!(f, "{pretty_body}")?;
            }
            write!(f, "{indent_string}}}")?;
        }

        SwitchCase::Fallthrough(exprs, stmts) => {
            write!(f, "{indent_string}case ")?;
            fmt_args(f, exprs, charmap)?;
            writeln!(f, " fallthrough")?;
            writeln!(f, "{indent_string}{{")?;
            if !stmts.is_empty() {
                let pretty_body = PrettyStmts(&stmts[..], indent + 1, charmap);
                writeln!(f, "{pretty_body}")?;
            }
            write!(f, "{indent_string}}}")?;
        }

        SwitchCase::DeadJump(stmts) => {
            writeln!(f, "{indent_string}dead")?;
            writeln!(f, "{indent_string}{{")?;

            if !stmts.is_empty() {
                let pretty_body = PrettyStmts(&stmts[..], indent + 1, charmap);
                writeln!(f, "{pretty_body}")?;
            }

            write!(f, "{indent_string}}}")?;
        }
    })
}
