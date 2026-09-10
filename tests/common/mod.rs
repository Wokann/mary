#![allow(dead_code)]

use mary::ast::{Stmt, SwitchCase};
use std::fs;

pub fn contains_stmt(stmts: &[Stmt], predicate: &impl Fn(&Stmt) -> bool) -> bool {
    stmts.iter().any(|stmt| {
        predicate(stmt)
            || match stmt {
                Stmt::If(_, body) | Stmt::DoWhile(_, body) => contains_stmt(body, predicate),
                Stmt::IfElse(_, then_body, else_body) => {
                    contains_stmt(then_body, predicate) || contains_stmt(else_body, predicate)
                }
                Stmt::For(elements) => contains_stmt(&elements.3, predicate),
                Stmt::Switch(_, cases, _, _) => cases.iter().any(|case| {
                    let body = match case {
                        SwitchCase::Case(_, body)
                        | SwitchCase::Fallthrough(_, body)
                        | SwitchCase::Default(body)
                        | SwitchCase::DefaultFallthrough(body)
                        | SwitchCase::ImplicitDefault(body)
                        | SwitchCase::DeadJump(body) => body,
                    };
                    contains_stmt(body, predicate)
                }),
                _ => false,
            }
    })
}

fn joined(fomt: &str, mfomt: &str) -> String {
    format!(
        "#if defined(MARY_FOMT_US) || defined(MARY_FOMT_JP)\n{fomt}\n#elif defined(MARY_MFOMT_US) || defined(MARY_MFOMT_JP)\n{mfomt}\n#endif\n"
    )
}

pub fn constants_source() -> String {
    joined(
        &fs::read_to_string("goodies/fomt_constants.mary.h").unwrap(),
        &fs::read_to_string("goodies/mfomt_constants.mary.h").unwrap(),
    )
}

pub fn callables_source() -> String {
    joined(
        &fs::read_to_string("goodies/fomt_callables.mary.h").unwrap(),
        &fs::read_to_string("goodies/mfomt_callables.mary.h").unwrap(),
    )
}

pub fn symbols_source() -> String {
    joined(
        &fs::read_to_string("goodies/fomt_scripts_text.mary.sym").unwrap(),
        &fs::read_to_string("goodies/mfomt_scripts_text.mary.sym").unwrap(),
    )
}

pub fn constants_for_target(target: &str) -> String {
    let family = if target.contains("MFOMT") {
        "mfomt"
    } else {
        "fomt"
    };
    fs::read_to_string(format!("goodies/{family}_constants.mary.h")).unwrap()
}

pub fn callables_for_target(target: &str) -> String {
    let family = if target.contains("MFOMT") {
        "mfomt"
    } else {
        "fomt"
    };
    fs::read_to_string(format!("goodies/{family}_callables.mary.h")).unwrap()
}

pub fn symbols_for_target(target: &str) -> String {
    let family = if target.contains("MFOMT") {
        "mfomt"
    } else {
        "fomt"
    };
    fs::read_to_string(format!("goodies/{family}_scripts_text.mary.sym")).unwrap()
}
