use mary::ast::{Stmt, SwitchCase};

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
