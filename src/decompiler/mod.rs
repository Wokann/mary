mod decorator;
mod error;
mod ins_decompiler;
mod state;

use std::collections::HashMap;

pub use error::{DecompileError, DecompileErrorExtra};
use ins_decompiler::decompile_instructions;
use state::DecompileState;
use state::DecompileToken;

use crate::{ast::Stmt, const_scope::ConstScope, ir::Script};

pub fn decompile_script<'a>(
    script: &'a Script,
    const_scope: &ConstScope,
) -> Result<Vec<Stmt>, DecompileErrorExtra<'a>> {
    match decompile_script_structured(script, const_scope) {
        Ok(stmts) => Ok(stmts),
        Err(_) => Ok(vec![Stmt::Ir(crate::low_level::script_to_items(script))]),
    }
}

pub fn decompile_script_named<'a>(
    script: &'a Script,
    const_scope: &ConstScope,
    script_name: &str,
) -> Result<Vec<Stmt>, DecompileErrorExtra<'a>> {
    match decompile_script_structured_inner(
        script,
        const_scope,
        Some(script_name),
        None,
        &HashMap::new(),
    ) {
        Ok(stmts) => Ok(stmts),
        Err(_) => Ok(vec![Stmt::Ir(crate::low_level::script_to_items(script))]),
    }
}

pub fn decompile_script_with_text_names<'a>(
    script: &'a Script,
    const_scope: &ConstScope,
    script_name: &str,
    text_names: &[Option<String>],
) -> Result<Vec<Stmt>, DecompileErrorExtra<'a>> {
    decompile_script_with_metadata(
        script,
        const_scope,
        script_name,
        text_names,
        &HashMap::new(),
    )
}

pub fn decompile_script_with_metadata<'a>(
    script: &'a Script,
    const_scope: &ConstScope,
    script_name: &str,
    text_names: &[Option<String>],
    local_types: &HashMap<String, crate::ir::ValueType>,
) -> Result<Vec<Stmt>, DecompileErrorExtra<'a>> {
    decompile_script_structured_inner(
        script,
        const_scope,
        Some(script_name),
        Some(text_names),
        local_types,
    )
}

/// Decompile without the lossless instruction-level fallback.  This is useful
/// for measuring and improving actual high-level structuring coverage.
pub fn decompile_script_structured<'a>(
    script: &'a Script,
    const_scope: &ConstScope,
) -> Result<Vec<Stmt>, DecompileErrorExtra<'a>> {
    decompile_script_structured_inner(script, const_scope, None, None, &HashMap::new())
}

fn decompile_script_structured_inner<'a>(
    script: &'a Script,
    const_scope: &ConstScope,
    script_name: Option<&str>,
    text_names: Option<&[Option<String>]>,
    local_types: &HashMap<String, crate::ir::ValueType>,
) -> Result<Vec<Stmt>, DecompileErrorExtra<'a>> {
    let mut known_callables = HashMap::new();

    let callable_map = const_scope.callable_map();

    for (name, (call_id, shape)) in callable_map {
        known_callables.insert(*call_id, (name.clone(), shape.clone()));
    }

    let mut stmts = decompile_instructions(&script.instructions, &known_callables)?;

    if let Err(err) = decorator::decorate_stmts_with_strings(
        &mut stmts,
        &script.strings,
        const_scope,
        script_name,
        text_names,
        local_types,
    ) {
        return Err(DecompileErrorExtra(err, DecompileState::new(&[])));
    }

    // TODO: decorate
    Ok(stmts)
}

#[cfg(test)]
mod tests {
    use crate::ast::{AssignOperation, Expr, Invoke, Stmt};
    use crate::bytecode;
    use crate::ir::{CallId, Ins, JumpId, ValueType, VarId};

    use super::*;

    #[test]
    fn test_decompile() {
        let mut const_sope = ConstScope::new();

        const_sope.add_proc("FakeProcedure".to_string(), CallId(2), vec![]);

        let var_0 = "var_0".to_string();

        let scripts = [(
            Script {
                instructions: vec![
                    Ins::PushInt(0),
                    Ins::PushInt(10),
                    Ins::Assign,
                    Ins::Discard,
                    Ins::Label(JumpId(2)),
                    Ins::Call(CallId(2)),
                    Ins::PushVar(VarId(0)),
                    Ins::Dec,
                    Ins::Dupe,
                    Ins::PopVar(VarId(0)),
                    Ins::Discard,
                    Ins::PushVar(VarId(0)),
                    Ins::PushInt(0),
                    Ins::Cmp,
                    Ins::Bne(JumpId(0)),
                    Ins::PushInt(0),
                    Ins::Jmp(JumpId(1)),
                    Ins::Label(JumpId(0)),
                    Ins::PushInt(1),
                    Ins::Label(JumpId(1)),
                    Ins::Bne(JumpId(2)),
                ],
                strings: vec![],
            },
            vec![
                // TODO: declare variables
                Stmt::Vars(vec![(var_0.clone(), None)]),
                Stmt::Assign(AssignOperation::None, var_0.clone(), Expr::Int(10)),
                Stmt::DoWhile(
                    Expr::CmpNe(Box::new((Expr::Name(var_0.clone()), Expr::Int(0)))),
                    vec![
                        Stmt::Call(Invoke {
                            func: "FakeProcedure".to_string(),
                            args: vec![],
                        }),
                        Stmt::Expr(Expr::PreDecrement(var_0.clone())),
                    ],
                ),
            ],
        )];

        for (script, expected_decompiled) in scripts {
            let encoded = bytecode::encode_script(&script);
            let decoded = bytecode::decode_script(&mut &encoded[..]).unwrap();

            let decompiled = decompile_script(&decoded, &const_sope).unwrap();

            assert_eq!(&decompiled[..], &expected_decompiled[..]);
        }
    }

    #[test]
    fn explicit_local_type_decorates_literal_definitions_and_rejects_typos() {
        let script = Script {
            instructions: vec![Ins::PushInt(0), Ins::PushInt(1), Ins::Assign, Ins::Discard],
            strings: vec![],
        };
        let mut scope = ConstScope::new();
        scope.add_typed_int_const("MaryBool", "FALSE".into(), 0);
        scope.add_typed_int_const("MaryBool", "TRUE".into(), 1);
        let bool_type = ValueType::UserType(scope.user_type("MaryBool").unwrap());
        let local_types = HashMap::from([("var_0".to_owned(), bool_type)]);

        let stmts =
            decompile_script_with_metadata(&script, &scope, "Test", &[], &local_types).unwrap();
        assert!(stmts.iter().any(|stmt| matches!(
            stmt,
            Stmt::Assign(AssignOperation::None, name, Expr::Name(value))
                if name == "var_0" && value == "TRUE"
        )));

        let typo = HashMap::from([("var_9".to_owned(), bool_type)]);
        assert!(
            decompile_script_with_metadata(&script, &scope, "Test", &[], &typo)
                .unwrap_err()
                .to_string()
                .contains("unknown local 'var_9'")
        );
    }
}
