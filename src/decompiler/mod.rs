mod decorator;
mod error;
mod ins_decompiler;
mod state;

use std::collections::HashMap;

pub use error::{DecompileError, DecompileErrorExtra};
use ins_decompiler::decompile_instructions;
use state::DecompileState;
use state::DecompileToken;

use crate::{
    ast::{Stmt, SwitchCase},
    const_scope::ConstScope,
    ir::Script,
};

fn contains_low_level_node(stmts: &[Stmt]) -> bool {
    stmts.iter().any(|stmt| {
        matches!(stmt, Stmt::Ir(_) | Stmt::JumpNext)
            || match stmt {
                Stmt::If(_, body) | Stmt::DoWhile(_, body) => contains_low_level_node(body),
                Stmt::IfElse(_, then_body, else_body) => {
                    contains_low_level_node(then_body) || contains_low_level_node(else_body)
                }
                Stmt::For(parts) => contains_low_level_node(&parts.3),
                Stmt::Switch(_, cases, _, _) => cases.iter().any(|case| match case {
                    SwitchCase::Case(_, body)
                    | SwitchCase::Fallthrough(_, body)
                    | SwitchCase::Default(body)
                    | SwitchCase::DefaultFallthrough(body)
                    | SwitchCase::ImplicitDefault(body)
                    | SwitchCase::DeadJump(body) => contains_low_level_node(body),
                }),
                _ => false,
            }
    })
}

pub fn decompile_script<'a>(
    script: &'a Script,
    const_scope: &ConstScope,
) -> Result<Vec<Stmt>, DecompileErrorExtra<'a>> {
    decompile_script_structured(script, const_scope)
}

pub fn decompile_script_named<'a>(
    script: &'a Script,
    const_scope: &ConstScope,
    script_name: &str,
) -> Result<Vec<Stmt>, DecompileErrorExtra<'a>> {
    decompile_script_structured_inner(
        script,
        const_scope,
        Some(script_name),
        None,
        &HashMap::new(),
    )
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

/// Strictly decompile to high-level statements. Residual low-level nodes are
/// errors, which makes this suitable for measuring real structuring coverage.
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

    if contains_low_level_node(&stmts) {
        return Err(DecompileErrorExtra(
            DecompileError::ResidualLowLevelControlFlow,
            DecompileState::new(&[]),
        ));
    }

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
    fn short_malformed_ir_never_panics_the_structured_decompiler() {
        use crate::ir::{CaseEnum, SwitchId};
        use std::panic::{catch_unwind, AssertUnwindSafe};

        let alphabet = [
            Ins::PushInt(0),
            Ins::Assign,
            Ins::Add,
            Ins::Neg,
            Ins::Cmp,
            Ins::Dupe,
            Ins::Discard,
            Ins::Jmp(JumpId(0)),
            Ins::Beq(JumpId(0)),
            Ins::Label(JumpId(0)),
            Ins::Switch(SwitchId(0)),
            Ins::Case(SwitchId(0), CaseEnum::Default),
            Ins::Exit,
        ];
        let scope = ConstScope::new();

        for len in 0..=5usize {
            let combinations = alphabet.len().pow(len as u32);
            for mut encoded in 0..combinations {
                let mut instructions = Vec::with_capacity(len);
                for _ in 0..len {
                    instructions.push(alphabet[encoded % alphabet.len()]);
                    encoded /= alphabet.len();
                }
                let script = Script::new(instructions.clone(), vec![]);
                let result = catch_unwind(AssertUnwindSafe(|| {
                    let _ = decompile_script_structured(&script, &scope);
                }));
                assert!(
                    result.is_ok(),
                    "structured decompiler panicked for {instructions:?}"
                );
            }
        }
    }

    #[test]
    fn comparison_with_only_one_operand_returns_an_error_instead_of_panicking() {
        use std::panic::{catch_unwind, AssertUnwindSafe};

        let script = Script::new(
            vec![
                Ins::PushInt(1),
                Ins::Cmp,
                Ins::Beq(JumpId(0)),
                Ins::PushInt(0),
                Ins::Jmp(JumpId(1)),
                Ins::Label(JumpId(0)),
                Ins::PushInt(1),
                Ins::Label(JumpId(1)),
            ],
            vec![],
        );
        let scope = ConstScope::new();
        let result = catch_unwind(AssertUnwindSafe(|| {
            decompile_script_structured(&script, &scope)
        }));

        assert!(result.is_ok(), "malformed comparison must not panic");
        assert!(result.unwrap().is_err(), "malformed comparison must fail");
    }

    #[test]
    fn sampled_long_malformed_ir_never_panics_the_structured_decompiler() {
        use crate::ir::{CaseEnum, SwitchId};
        use std::panic::{catch_unwind, AssertUnwindSafe};

        let alphabet = [
            Ins::PushInt(0),
            Ins::PushInt(1),
            Ins::PushVar(VarId(0)),
            Ins::PopVar(VarId(0)),
            Ins::Assign,
            Ins::AssignAdd,
            Ins::Add,
            Ins::Sub,
            Ins::Mul,
            Ins::Div,
            Ins::Mod,
            Ins::Neg,
            Ins::LogicalAnd,
            Ins::LogicalOr,
            Ins::LogicalNot,
            Ins::Cmp,
            Ins::Dupe,
            Ins::Inc,
            Ins::Dec,
            Ins::Discard,
            Ins::Jmp(JumpId(0)),
            Ins::Beq(JumpId(0)),
            Ins::Bne(JumpId(0)),
            Ins::Blt(JumpId(0)),
            Ins::Label(JumpId(0)),
            Ins::Switch(SwitchId(0)),
            Ins::Case(SwitchId(0), CaseEnum::Val(0)),
            Ins::Case(SwitchId(0), CaseEnum::Default),
            Ins::Call(CallId(2)),
            Ins::Exit,
        ];
        let mut seed = 0xD1B5_4A32_D192_ED03_u64;
        let mut scope = ConstScope::new();
        scope.add_func(
            "SampledFunction".into(),
            CallId(2),
            vec![ValueType::Integer, ValueType::Integer, ValueType::Integer],
        );

        for sample in 0..100_000 {
            seed = seed.wrapping_mul(6_364_136_223_846_793_005).wrapping_add(1);
            let len = (seed as usize % 20) + 1;
            let mut instructions = Vec::with_capacity(len);
            for _ in 0..len {
                seed = seed.wrapping_mul(6_364_136_223_846_793_005).wrapping_add(1);
                instructions.push(alphabet[seed as usize % alphabet.len()]);
            }
            let script = Script::new(instructions.clone(), vec![]);
            let result = catch_unwind(AssertUnwindSafe(|| {
                let _ = decompile_script_structured(&script, &scope);
            }));
            assert!(
                result.is_ok(),
                "structured decompiler panicked for sample {sample}: {instructions:?}"
            );
        }
    }

    #[test]
    fn public_decompiler_entries_reject_unstructured_ir_instead_of_hiding_it() {
        let script = Script::new(vec![Ins::Add], vec![]);
        let scope = ConstScope::new();

        assert!(decompile_script_named(&script, &scope, "BrokenScript").is_err());
        assert!(decompile_script(&script, &scope).is_err());
    }

    #[test]
    fn structured_entry_rejects_residual_jump_next() {
        let script = Script::new(vec![Ins::Jmp(JumpId(0)), Ins::Label(JumpId(0))], vec![]);
        let scope = ConstScope::new();

        let error = decompile_script_structured(&script, &scope).unwrap_err();
        assert!(matches!(
            DecompileError::from(error),
            DecompileError::ResidualLowLevelControlFlow
        ));
    }

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
