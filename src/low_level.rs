//! Lossless textual representation of decoded VM instructions.
//!
//! This is deliberately instruction-level rather than byte-level: users can
//! read and edit control flow, stack operations and strings while every
//! decoded item remains represented for exact re-encoding.

use crate::{
    ast::{IrArg, IrItem},
    compiler::error::CompileError,
    ir::{CallId, CaseEnum, Ins, JumpId, Script, SwitchId, VarId},
};

fn item(name: &str, args: Vec<IrArg>) -> IrItem {
    IrItem::new(name, args)
}

fn int(value: usize) -> IrArg {
    IrArg::Int(value as i64)
}

pub fn script_to_items(script: &Script) -> Vec<IrItem> {
    let mut items = Vec::with_capacity(script.strings.len() + script.instructions.len());
    for value in &script.strings {
        items.push(item("String", vec![IrArg::Str(value.clone())]));
    }
    for instruction in &script.instructions {
        items.push(match *instruction {
            Ins::Assign => item("Assign", vec![]),
            Ins::AssignAdd => item("AssignAdd", vec![]),
            Ins::AssignSub => item("AssignSub", vec![]),
            Ins::AssignMul => item("AssignMul", vec![]),
            Ins::AssignDiv => item("AssignDiv", vec![]),
            Ins::AssignMod => item("AssignMod", vec![]),
            Ins::Add => item("Add", vec![]),
            Ins::Sub => item("Sub", vec![]),
            Ins::Mul => item("Mul", vec![]),
            Ins::Div => item("Div", vec![]),
            Ins::Mod => item("Mod", vec![]),
            Ins::LogicalAnd => item("LogicalAnd", vec![]),
            Ins::LogicalOr => item("LogicalOr", vec![]),
            Ins::Inc => item("Inc", vec![]),
            Ins::Dec => item("Dec", vec![]),
            Ins::Neg => item("Neg", vec![]),
            Ins::LogicalNot => item("LogicalNot", vec![]),
            Ins::Cmp => item("Cmp", vec![]),
            Ins::PushVar(id) => item("PushVar", vec![int(id.0)]),
            Ins::PopVar(id) => item("PopVar", vec![int(id.0)]),
            Ins::Dupe => item("Dupe", vec![]),
            Ins::Discard => item("Discard", vec![]),
            Ins::PushInt(value) => item("PushInt", vec![IrArg::Int(value)]),
            Ins::Jmp(id) => item("Jmp", vec![int(id.0)]),
            Ins::Blt(id) => item("Blt", vec![int(id.0)]),
            Ins::Ble(id) => item("Ble", vec![int(id.0)]),
            Ins::Beq(id) => item("Beq", vec![int(id.0)]),
            Ins::Bne(id) => item("Bne", vec![int(id.0)]),
            Ins::Bge(id) => item("Bge", vec![int(id.0)]),
            Ins::Bgt(id) => item("Bgt", vec![int(id.0)]),
            Ins::Call(id) => item("Call", vec![int(id.0)]),
            Ins::Switch(id) => item("Switch", vec![int(id.0)]),
            Ins::Exit => item("Exit", vec![]),
            Ins::Case(id, CaseEnum::Val(value)) => item("Case", vec![int(id.0), IrArg::Int(value)]),
            Ins::Case(id, CaseEnum::Default) => item("Default", vec![int(id.0)]),
            Ins::Label(id) => item("Label", vec![int(id.0)]),
        });
    }
    items
}

fn values(item: &IrItem, count: usize) -> Result<Vec<i64>, CompileError> {
    if item.args.len() != count {
        return Err(CompileError::InvalidIr(format!(
            "{} expects {} integer argument(s), got {}",
            item.name,
            count,
            item.args.len()
        )));
    }
    item.args
        .iter()
        .map(|arg| match arg {
            IrArg::Int(value) => Ok(*value),
            IrArg::Str(_) => Err(CompileError::InvalidIr(format!(
                "{} expects integer arguments",
                item.name
            ))),
        })
        .collect()
}

fn id(value: i64, context: &str) -> Result<usize, CompileError> {
    usize::try_from(value).map_err(|_| {
        CompileError::InvalidIr(format!("{context} id must be non-negative, got {value}"))
    })
}

pub fn items_to_script(items: Vec<IrItem>) -> Result<Script, CompileError> {
    let mut strings = vec![];
    let mut instructions = vec![];
    let mut saw_instruction = false;

    for item in items {
        if item.name == "String" {
            if saw_instruction || item.args.len() != 1 {
                return Err(CompileError::InvalidIr(
                    "String entries must precede instructions and have one argument".into(),
                ));
            }
            match item.args.into_iter().next().unwrap() {
                IrArg::Str(value) => strings.push(value),
                IrArg::Int(_) => {
                    return Err(CompileError::InvalidIr(
                        "String expects a string literal".into(),
                    ))
                }
            }
            continue;
        }

        saw_instruction = true;
        let ins = match item.name.as_str() {
            "Assign" => {
                values(&item, 0)?;
                Ins::Assign
            }
            "AssignAdd" => {
                values(&item, 0)?;
                Ins::AssignAdd
            }
            "AssignSub" => {
                values(&item, 0)?;
                Ins::AssignSub
            }
            "AssignMul" => {
                values(&item, 0)?;
                Ins::AssignMul
            }
            "AssignDiv" => {
                values(&item, 0)?;
                Ins::AssignDiv
            }
            "AssignMod" => {
                values(&item, 0)?;
                Ins::AssignMod
            }
            "Add" => {
                values(&item, 0)?;
                Ins::Add
            }
            "Sub" => {
                values(&item, 0)?;
                Ins::Sub
            }
            "Mul" => {
                values(&item, 0)?;
                Ins::Mul
            }
            "Div" => {
                values(&item, 0)?;
                Ins::Div
            }
            "Mod" => {
                values(&item, 0)?;
                Ins::Mod
            }
            "LogicalAnd" => {
                values(&item, 0)?;
                Ins::LogicalAnd
            }
            "LogicalOr" => {
                values(&item, 0)?;
                Ins::LogicalOr
            }
            "Inc" => {
                values(&item, 0)?;
                Ins::Inc
            }
            "Dec" => {
                values(&item, 0)?;
                Ins::Dec
            }
            "Neg" => {
                values(&item, 0)?;
                Ins::Neg
            }
            "LogicalNot" => {
                values(&item, 0)?;
                Ins::LogicalNot
            }
            "Cmp" => {
                values(&item, 0)?;
                Ins::Cmp
            }
            "Dupe" => {
                values(&item, 0)?;
                Ins::Dupe
            }
            "Discard" => {
                values(&item, 0)?;
                Ins::Discard
            }
            "Exit" => {
                values(&item, 0)?;
                Ins::Exit
            }
            "PushVar" => Ins::PushVar(VarId(id(values(&item, 1)?[0], "variable")?)),
            "PopVar" => Ins::PopVar(VarId(id(values(&item, 1)?[0], "variable")?)),
            "PushInt" => Ins::PushInt(values(&item, 1)?[0]),
            "Jmp" => Ins::Jmp(JumpId(id(values(&item, 1)?[0], "jump")?)),
            "Blt" => Ins::Blt(JumpId(id(values(&item, 1)?[0], "jump")?)),
            "Ble" => Ins::Ble(JumpId(id(values(&item, 1)?[0], "jump")?)),
            "Beq" => Ins::Beq(JumpId(id(values(&item, 1)?[0], "jump")?)),
            "Bne" => Ins::Bne(JumpId(id(values(&item, 1)?[0], "jump")?)),
            "Bge" => Ins::Bge(JumpId(id(values(&item, 1)?[0], "jump")?)),
            "Bgt" => Ins::Bgt(JumpId(id(values(&item, 1)?[0], "jump")?)),
            "Call" => Ins::Call(CallId(id(values(&item, 1)?[0], "call")?)),
            "Switch" => Ins::Switch(SwitchId(id(values(&item, 1)?[0], "switch")?)),
            "Label" => Ins::Label(JumpId(id(values(&item, 1)?[0], "label")?)),
            "Default" => Ins::Case(
                SwitchId(id(values(&item, 1)?[0], "switch")?),
                CaseEnum::Default,
            ),
            "Case" => {
                let args = values(&item, 2)?;
                Ins::Case(SwitchId(id(args[0], "switch")?), CaseEnum::Val(args[1]))
            }
            name => {
                return Err(CompileError::InvalidIr(format!(
                    "unknown instruction {name}"
                )))
            }
        };
        instructions.push(ins);
    }

    Ok(Script::new(instructions, strings))
}
