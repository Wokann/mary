use std::{
    cmp::Ordering,
    collections::{BTreeMap, HashMap},
    io,
    str::Utf8Error,
};

use thiserror::Error;

use crate::ir::{CallId, CaseEnum, Ins, IntValue, JumpId, Script, StrValue, SwitchId, VarId};

use super::opcodes::*;

#[derive(Error, Debug)]
pub enum DecodeError {
    #[error("IO Error: {0}")]
    IoError(#[from] io::Error),

    #[error("Script Error: {0}")]
    ScriptError(&'static str),

    #[error("Corrupt chunk name")]
    BadChunkName(#[from] Utf8Error),

    #[error("Missing CODE chunk")]
    MissingCodeChunk,

    #[error("RIFF size or chunk boundaries are inconsistent")]
    BadRiffSize,

    #[error("duplicate RIFF chunk '{0}'")]
    DuplicateChunk(String),

    #[error("Misaligned jump table offset: {0:02X}")]
    MisalignedJumpTable(usize),

    #[error("CODE chunk size mismatch")]
    BadCodeChunk,

    #[error("CODE instruction at offset 0x{offset:X} has unknown opcode 0x{opcode:02X}")]
    BadOpcode { offset: usize, opcode: u8 },

    #[error("CODE instruction at offset 0x{offset:X} has a truncated operand")]
    TruncatedOperand { offset: usize },

    #[error("CODE chunk has no terminating END opcode")]
    MissingCodeEnd,

    #[error("CODE bytes after the terminating END are not NOP padding")]
    BadCodePadding,

    #[error("jump target 0x{target:X} is not a CODE instruction boundary")]
    BadJumpTarget { target: usize },

    #[error("switch case target 0x{target:X} is not a CODE instruction boundary")]
    BadCaseTarget { target: usize },

    #[error("CODE uses switch table {id}, but JUMP contains {table_count} tables")]
    BadSwitchId { id: usize, table_count: usize },

    #[error("CODE requires {expected} switch tables, but JUMP contains {actual}")]
    BadSwitchTableCount { expected: usize, actual: usize },

    #[error("CODE repeats switch table ID {0}")]
    DuplicateSwitchId(usize),

    #[error("JUMP table {0} has more than one default case")]
    DuplicateDefault(usize),

    #[error("JUMP table {switch_id} repeats case value {value}")]
    DuplicateCaseValue { switch_id: usize, value: IntValue },

    #[error("Corrupt STR chunk")]
    BadStrChunk,

    #[error("STR string {index} at pool offset 0x{offset:X} has no 0x00 terminator")]
    UnterminatedString { index: usize, offset: usize },
}

// This is pub(crate) as it is used in encoder tests also
// This could be moved elsewhere for that reason
#[allow(dead_code)]
pub(crate) trait DecodeHelper {
    fn read_u32(&mut self) -> io::Result<u32>;
    fn read_u16(&mut self) -> io::Result<u16>;
    fn read_u8(&mut self) -> io::Result<u8>;
    fn read_4byte(&mut self) -> io::Result<[u8; 4]>;
}

impl<R: io::Read> DecodeHelper for R {
    fn read_u32(&mut self) -> io::Result<u32> {
        let mut buf = [0u8; 4];
        self.read_exact(&mut buf)?;

        Ok((buf[0] as u32)
            | ((buf[1] as u32) << 8)
            | ((buf[2] as u32) << 16)
            | ((buf[3] as u32) << 24))
    }

    fn read_u16(&mut self) -> io::Result<u16> {
        let mut buf = [0u8; 2];
        self.read_exact(&mut buf)?;

        Ok((buf[0] as u16) | ((buf[1] as u16) << 8))
    }

    fn read_u8(&mut self) -> io::Result<u8> {
        let mut buf = [0u8; 1];
        self.read_exact(&mut buf)?;
        Ok(buf[0])
    }

    fn read_4byte(&mut self) -> io::Result<[u8; 4]> {
        let mut buf = [0u8; 4];
        self.read_exact(&mut buf)?;
        Ok(buf)
    }
}

#[derive(Debug, Clone, Copy)]
struct JumpInfo {
    source_offset: usize,
    target_offset: usize,
    jump_id: JumpId,
}

impl JumpInfo {
    fn new(source_offset: usize, target_offset: usize, jump_id: JumpId) -> Self {
        Self {
            source_offset,
            target_offset,
            jump_id,
        }
    }
}

#[derive(Debug)]
struct JumpIdMap(Vec<JumpInfo>);

fn operand_size(opcode: u8) -> usize {
    match opcode {
        OPCODE_PUSHV => 4,  // push [A]
        OPCODE_POPV => 4,   // pop [A]
        OPCODE_PUSH32 => 4, // push imm32
        OPCODE_PUSH16 => 2, // push imm16
        OPCODE_PUSH8 => 1,  // push imm8
        OPCODE_JMP => 4,    // b label
        OPCODE_BLT => 4,    // blt label
        OPCODE_BLE => 4,    // ble label
        OPCODE_BEQ => 4,    // beq label
        OPCODE_BNE => 4,    // bne label
        OPCODE_BGE => 4,    // bge label
        OPCODE_BGT => 4,    // bgt label
        OPCODE_CALL => 4,   // call id
        OPCODE_SWITCH => 4, // switch id
        _ => 0,
    }
}

fn is_opcode(opcode: u8) -> bool {
    matches!(
        opcode,
        OPCODE_NOP
            | OPCODE_EQU
            | OPCODE_ADDEQU
            | OPCODE_SUBEQU
            | OPCODE_MULEQU
            | OPCODE_DIVEQU
            | OPCODE_MODEQU
            | OPCODE_ADD
            | OPCODE_SUB
            | OPCODE_MUL
            | OPCODE_DIV
            | OPCODE_MOD
            | OPCODE_AND
            | OPCODE_OR
            | OPCODE_INC
            | OPCODE_DEC
            | OPCODE_NEG
            | OPCODE_NOT
            | OPCODE_CMP
            | OPCODE_PUSHV
            | OPCODE_POPV
            | OPCODE_DUP
            | OPCODE_DISC
            | OPCODE_PUSH32
            | OPCODE_JMP
            | OPCODE_BLT
            | OPCODE_BLE
            | OPCODE_BEQ
            | OPCODE_BNE
            | OPCODE_BGE
            | OPCODE_BGT
            | OPCODE_END
            | OPCODE_CALL
            | OPCODE_PUSH16
            | OPCODE_PUSH8
            | OPCODE_SWITCH
    )
}

fn get_code_jumps(code_data: &[u8]) -> Result<JumpIdMap, DecodeError> {
    let mut offset = 0;

    let mut jump_id_counter = 0;

    /* second item is difference from jump location to target location (used for sorting) */
    let mut extended_jump_map: Vec<(JumpInfo, isize)> = Vec::new();

    while offset < code_data.len() {
        let instruction_offset = offset;
        let opcode = code_data[offset];
        if !is_opcode(opcode) {
            return Err(DecodeError::BadOpcode {
                offset: instruction_offset,
                opcode,
            });
        }
        offset += 1;

        let operand_end =
            offset
                .checked_add(operand_size(opcode))
                .ok_or(DecodeError::TruncatedOperand {
                    offset: instruction_offset,
                })?;
        if operand_end > code_data.len() {
            return Err(DecodeError::TruncatedOperand {
                offset: instruction_offset,
            });
        }

        if let OPCODE_JMP | OPCODE_BEQ | OPCODE_BNE | OPCODE_BLE | OPCODE_BLT | OPCODE_BGE
        | OPCODE_BGT = opcode
        {
            let jump_id = JumpId(jump_id_counter);
            let target_offset = (&code_data[offset..offset + 4]).read_u32()? as usize;
            let disp = (target_offset as isize) - (offset as isize);
            let jump_info = JumpInfo::new(offset - 1, target_offset, jump_id);

            extended_jump_map.push((jump_info, disp));
            jump_id_counter += 1;
        }

        offset = operand_end;
    }

    extended_jump_map.sort_by(|a, b| {
        let (a_info, a_disp) = *a;
        let (b_info, b_disp) = *b;

        if a_info.target_offset == b_info.target_offset {
            if (a_disp < 0 && b_disp < 0) || (a_disp > 0 && b_disp > 0) {
                /* Both are same direction jumps, ascending order */
                a_disp.cmp(&b_disp)
            } else {
                /* Forwards before backwards, descending order */
                b_disp.cmp(&a_disp)
            }
        } else {
            /* different jump offset, by order of jumps */
            a_info.target_offset.cmp(&b_info.target_offset)
        }
    });

    let jump_map = extended_jump_map.iter().map(|(info, _)| *info).collect();

    Ok(JumpIdMap(jump_map))
}

#[derive(Default)]
struct CaseTable(Vec<(CaseEnum, usize)>);

struct CaseMap(Vec<(usize, SwitchId, CaseEnum)>);

#[derive(Default)]
struct DecodedJumpChunk {
    case_tables: Vec<CaseTable>,
}

fn decode_jump_chunk(data: &[u8]) -> Result<DecodedJumpChunk, DecodeError> {
    let read = &mut &data[..];
    let ent_count = read.read_u32()? as usize;

    let mut offs = vec![];

    for _ in 0..ent_count {
        let off = read.read_u32()? as usize;
        offs.push(off);

        if !off.is_multiple_of(4) {
            return Err(DecodeError::MisalignedJumpTable(off));
        }
    }

    // Offsets are relative to the byte immediately following ent_count, not
    // merely documentation of the physical order of the tables.
    let jump_pool = data.get(4..).ok_or_else(|| {
        DecodeError::IoError(io::Error::new(
            io::ErrorKind::UnexpectedEof,
            "truncated JUMP chunk",
        ))
    })?;
    let mut case_tables = vec![];

    for off in offs {
        let table_data = jump_pool.get(off..).ok_or_else(|| {
            DecodeError::IoError(io::Error::new(
                io::ErrorKind::UnexpectedEof,
                "JUMP table offset is outside the chunk",
            ))
        })?;
        let read = &mut &table_data[..];
        let entries = read.read_u32()? as usize;
        let default = read.read_u32()? as usize;

        let mut case_table = CaseTable::default();

        if (default & 0x80000000) == 0 {
            case_table.0.push((CaseEnum::Default, default));
        }

        for _ in 0..entries {
            /* it's important here that we convert to i32 for JUMP sorting reasons
             * (game does signed compares, so any -1 need to be before any 0s)
             * raw cast to IntValue (i64) would result in -1 => 0xFFFFFFFF */

            let compare = read.read_u32()? as i32 as IntValue;
            let target = read.read_u32()? as usize;

            case_table.0.push((CaseEnum::Val(compare), target));
        }

        case_tables.push(case_table);
    }

    Ok(DecodedJumpChunk { case_tables })
}

fn build_case_map(jump_chunk: &DecodedJumpChunk) -> CaseMap {
    let mut case_map = Vec::new();

    for i in 0..jump_chunk.case_tables.len() {
        let switch_id = SwitchId(i);

        for &(case_enum, code_offset) in &jump_chunk.case_tables[i].0 {
            case_map.push((code_offset, switch_id, case_enum));
        }
    }

    /* this is a stable sort (this is important) */
    case_map.sort_by_key(|(code_offset, _, _)| *code_offset);

    CaseMap(case_map)
}

fn decode_code_chunk(code_data: &[u8]) -> Result<&[u8], DecodeError> {
    let head_size = (&code_data[0..]).read_u32()? as usize;

    if head_size != code_data.len() - 4 {
        return Err(DecodeError::BadCodeChunk);
    }

    let code_with_nops = &code_data[4..];
    let mut offset = 0usize;
    let mut final_end = None;
    while offset < code_with_nops.len() {
        let opcode = code_with_nops[offset];
        if !is_opcode(opcode) {
            return Err(DecodeError::BadOpcode { offset, opcode });
        }
        if opcode == OPCODE_END {
            final_end = Some(offset);
            offset += 1;
            continue;
        }

        let next = offset
            .checked_add(1 + operand_size(opcode))
            .ok_or(DecodeError::TruncatedOperand { offset })?;
        if next > code_with_nops.len() {
            return Err(DecodeError::TruncatedOperand { offset });
        }
        offset = next;
    }

    let final_end = final_end.ok_or(DecodeError::MissingCodeEnd)?;
    if code_with_nops[final_end + 1..]
        .iter()
        .any(|padding| *padding != OPCODE_NOP)
    {
        return Err(DecodeError::BadCodePadding);
    }

    /* Discard only the final format terminator; earlier END opcodes are real exits. */
    Ok(&code_with_nops[..final_end])
}

fn disassemble(
    code_data: &[u8],
    jump_targets: &JumpIdMap,
    case_map: &CaseMap,
) -> Result<Vec<Ins>, DecodeError> {
    let mut result = vec![];

    let mut offset = 0;

    let mut current_case_target_idx = 0;
    let mut current_jump_target_idx = 0;

    let mut jump_source_map = BTreeMap::<usize, JumpId>::new();

    let mut instruction_boundaries = std::collections::BTreeSet::new();
    let mut scan = 0usize;
    while scan < code_data.len() {
        instruction_boundaries.insert(scan);
        scan += 1 + operand_size(code_data[scan]);
    }
    instruction_boundaries.insert(code_data.len());

    for jump_info in &jump_targets.0 {
        if !instruction_boundaries.contains(&jump_info.target_offset) {
            return Err(DecodeError::BadJumpTarget {
                target: jump_info.target_offset,
            });
        }
        jump_source_map.insert(jump_info.source_offset, jump_info.jump_id);
    }

    if let Some((target, _, _)) = case_map
        .0
        .iter()
        .find(|(target, _, _)| !instruction_boundaries.contains(target))
    {
        return Err(DecodeError::BadCaseTarget { target: *target });
    }

    while offset < code_data.len() {
        /* before doing any decoding, let's insert any labels or cases here */

        while current_jump_target_idx < jump_targets.0.len() {
            let head = jump_targets.0[current_jump_target_idx];

            if head.target_offset > offset {
                break;
            }

            result.push(Ins::Label(head.jump_id));
            current_jump_target_idx += 1;
        }

        while current_case_target_idx < case_map.0.len() {
            let (target_offset, switch_id, case_id) = case_map.0[current_case_target_idx];

            if target_offset > offset {
                break;
            }

            result.push(Ins::Case(switch_id, case_id));
            current_case_target_idx += 1;
        }

        let pc = offset;

        let opcode = code_data[offset];
        offset += 1;

        let operand = match operand_size(opcode) {
            0 => 0,

            1 => {
                offset += 1;
                code_data[offset - 1] as usize
            }

            2 => {
                offset += 2;
                (&code_data[offset - 2..offset]).read_u16()? as usize
            }

            4 => {
                offset += 4;
                (&code_data[offset - 4..offset]).read_u32()? as usize
            }

            _ => unreachable!(),
        };

        match opcode {
            OPCODE_NOP => {}

            OPCODE_EQU => result.push(Ins::Assign),
            OPCODE_ADDEQU => result.push(Ins::AssignAdd),
            OPCODE_SUBEQU => result.push(Ins::AssignSub),
            OPCODE_MULEQU => result.push(Ins::AssignMul),
            OPCODE_DIVEQU => result.push(Ins::AssignDiv),
            OPCODE_MODEQU => result.push(Ins::AssignMod),

            OPCODE_ADD => result.push(Ins::Add),
            OPCODE_SUB => result.push(Ins::Sub),
            OPCODE_MUL => result.push(Ins::Mul),
            OPCODE_DIV => result.push(Ins::Div),
            OPCODE_MOD => result.push(Ins::Mod),

            OPCODE_AND => result.push(Ins::LogicalAnd),
            OPCODE_OR => result.push(Ins::LogicalOr),

            OPCODE_INC => result.push(Ins::Inc),
            OPCODE_DEC => result.push(Ins::Dec),

            OPCODE_NEG => result.push(Ins::Neg),
            OPCODE_NOT => result.push(Ins::LogicalNot),

            OPCODE_CMP => result.push(Ins::Cmp),

            OPCODE_PUSHV => result.push(Ins::PushVar(VarId(operand))),
            OPCODE_POPV => result.push(Ins::PopVar(VarId(operand))),

            OPCODE_DUP => result.push(Ins::Dupe),
            OPCODE_DISC => result.push(Ins::Discard),

            OPCODE_PUSH32 | OPCODE_PUSH16 | OPCODE_PUSH8 => {
                result.push(Ins::PushInt(operand as i32 as IntValue));
            }

            OPCODE_JMP => result.push(Ins::Jmp(jump_source_map[&pc])),
            OPCODE_BLT => result.push(Ins::Blt(jump_source_map[&pc])),
            OPCODE_BLE => result.push(Ins::Ble(jump_source_map[&pc])),
            OPCODE_BEQ => result.push(Ins::Beq(jump_source_map[&pc])),
            OPCODE_BNE => result.push(Ins::Bne(jump_source_map[&pc])),
            OPCODE_BGE => result.push(Ins::Bge(jump_source_map[&pc])),
            OPCODE_BGT => result.push(Ins::Bgt(jump_source_map[&pc])),

            OPCODE_END => result.push(Ins::Exit),

            OPCODE_CALL => result.push(Ins::Call(CallId(operand))),
            OPCODE_SWITCH => result.push(Ins::Switch(SwitchId(operand))),

            _ => unreachable!("opcodes are validated during the initial CODE scan"),
        }
    }

    while current_jump_target_idx < jump_targets.0.len() {
        let head = jump_targets.0[current_jump_target_idx];
        result.push(Ins::Label(head.jump_id));
        current_jump_target_idx += 1;
    }

    Ok(result)
}

fn read_str(data: &[u8]) -> Option<&[u8]> {
    data.iter()
        .position(|byte| *byte == 0)
        .map(|end| &data[..end])
}

fn decode_string_chunk(
    strings_data: &[u8],
    strings_backing: &[u8],
) -> Result<Vec<StrValue>, DecodeError> {
    if strings_data.len() < 4 {
        return Err(DecodeError::BadStrChunk);
    }
    let string_count = (&strings_data[0..4]).read_u32()? as usize;

    let pool_offset = 4usize
        .checked_add(
            4usize
                .checked_mul(string_count)
                .ok_or(DecodeError::BadStrChunk)?,
        )
        .ok_or(DecodeError::BadStrChunk)?;

    if pool_offset > strings_data.len() {
        return Err(DecodeError::BadStrChunk);
    }

    let string_offsets = &strings_data[4..pool_offset];
    let string_pool = strings_backing
        .get(pool_offset..)
        .ok_or(DecodeError::BadStrChunk)?;

    let mut string_table = vec![];

    for i in 0..string_count {
        let offset = (&(string_offsets[i * 4..])).read_u32()? as usize;

        if offset >= string_pool.len() {
            return Err(DecodeError::BadStrChunk);
        }

        let string = read_str(&string_pool[offset..])
            .ok_or(DecodeError::UnterminatedString { index: i, offset })?;
        string_table.push(string.to_vec());
    }

    Ok(string_table)
}

#[cfg(test)]
mod chunk_tests {
    use super::*;

    #[test]
    fn str_entries_are_resolved_through_the_offset_table() {
        let data = [
            2, 0, 0, 0, // count
            2, 0, 0, 0, // id 0 -> pool + 2 ("A")
            0, 0, 0, 0, // id 1 -> pool + 0 ("B")
            b'B', 0, b'A', 0,
        ];

        assert_eq!(
            decode_string_chunk(&data, &data).unwrap(),
            vec![b"A".to_vec(), b"B".to_vec()]
        );
    }

    #[test]
    fn str_offsets_may_target_relocated_text_beyond_the_declared_chunk() {
        let declared = [
            1, 0, 0, 0, // count
            8, 0, 0, 0, // id 0 -> pool + 8
        ];
        let mut backing = declared.to_vec();
        backing.extend(b"old\0pad\0new\0");

        assert_eq!(
            decode_string_chunk(&declared, &backing).unwrap(),
            vec![b"new".to_vec()]
        );
    }

    #[test]
    fn str_entry_must_have_a_null_terminator_in_the_available_backing() {
        let data = [
            1, 0, 0, 0, // count
            0, 0, 0, 0, // id 0 -> start of pool
            b'A', b'B',
        ];

        assert!(matches!(
            decode_string_chunk(&data, &data),
            Err(DecodeError::UnterminatedString {
                index: 0,
                offset: 0
            })
        ));
    }

    #[test]
    fn relocated_str_entry_must_have_a_null_terminator_in_the_external_backing() {
        let declared = [
            1, 0, 0, 0, // count
            4, 0, 0, 0, // id 0 -> pool + 4
        ];
        let mut backing = declared.to_vec();
        backing.extend(b"pad\0text without terminator");

        assert!(matches!(
            decode_string_chunk(&declared, &backing),
            Err(DecodeError::UnterminatedString {
                index: 0,
                offset: 4
            })
        ));
    }

    #[test]
    fn str_offsets_can_share_and_reorder_null_terminated_values() {
        let data = [
            4, 0, 0, 0, // count
            2, 0, 0, 0, // id 0 -> "B"
            0, 0, 0, 0, // id 1 -> "A"
            2, 0, 0, 0, // id 2 -> shared "B"
            4, 0, 0, 0, // id 3 -> empty string
            b'A', 0, b'B', 0, 0,
        ];

        assert_eq!(
            decode_string_chunk(&data, &data).unwrap(),
            vec![b"B".to_vec(), b"A".to_vec(), b"B".to_vec(), vec![]]
        );
    }

    #[test]
    fn truncated_chunk_body_is_rejected_instead_of_partially_decoded() {
        let encoded = [
            b'C', b'O', b'D', b'E', // name
            4, 0, 0, 0, // declared body length
            0x20, 0x00, // only half the body is available
        ];

        assert!(matches!(
            read_riff_chunks(&mut &encoded[..], 22),
            Err(DecodeError::BadRiffSize)
        ));
    }

    #[test]
    fn jump_entries_are_resolved_through_the_offset_table() {
        let mut data = vec![
            2, 0, 0, 0, // count
            24, 0, 0, 0, // switch 0 -> second physical table
            8, 0, 0, 0, // switch 1 -> first physical table
        ];
        // jump_pool + 8: one explicit case (10 -> code offset 100)
        data.extend([1, 0, 0, 0, 0xFF, 0xFF, 0xFF, 0xFF]);
        data.extend([10, 0, 0, 0, 100, 0, 0, 0]);
        // jump_pool + 24: default -> code offset 200
        data.extend([0, 0, 0, 0, 200, 0, 0, 0]);

        let decoded = decode_jump_chunk(&data).unwrap();
        assert!(matches!(
            decoded.case_tables[0].0[0],
            (CaseEnum::Default, 200)
        ));
        assert!(matches!(
            decoded.case_tables[1].0[0],
            (CaseEnum::Val(10), 100)
        ));
    }

    #[test]
    fn malformed_code_returns_diagnostics_instead_of_panicking() {
        assert!(matches!(
            get_code_jumps(&[0xFF]),
            Err(DecodeError::BadOpcode {
                offset: 0,
                opcode: 0xFF
            })
        ));
        assert!(matches!(
            get_code_jumps(&[OPCODE_CALL, 1, 2]),
            Err(DecodeError::TruncatedOperand { offset: 0 })
        ));

        let no_end = [1, 0, 0, 0, OPCODE_NOP];
        assert!(matches!(
            decode_code_chunk(&no_end),
            Err(DecodeError::MissingCodeEnd)
        ));

        let operand_is_not_end = [2, 0, 0, 0, OPCODE_PUSH8, OPCODE_END];
        assert!(matches!(
            decode_code_chunk(&operand_is_not_end),
            Err(DecodeError::MissingCodeEnd)
        ));

        let bad_padding = [3, 0, 0, 0, OPCODE_END, OPCODE_PUSH8, OPCODE_NOP];
        assert!(matches!(
            decode_code_chunk(&bad_padding),
            Err(DecodeError::BadCodePadding)
        ));
    }

    #[test]
    fn control_flow_targets_must_land_on_instruction_boundaries() {
        let code = [OPCODE_JMP, 2, 0, 0, 0];
        let jumps = get_code_jumps(&code).unwrap();
        assert!(matches!(
            disassemble(&code, &jumps, &CaseMap(vec![])),
            Err(DecodeError::BadJumpTarget { target: 2 })
        ));

        let code = [OPCODE_NOP];
        let cases = CaseMap(vec![(2, SwitchId(0), CaseEnum::Default)]);
        assert!(matches!(
            disassemble(&code, &JumpIdMap(vec![]), &cases),
            Err(DecodeError::BadCaseTarget { target: 2 })
        ));
    }

    #[test]
    fn riff_chunk_boundaries_and_names_are_not_silently_normalized() {
        assert!(matches!(
            read_riff_chunks(&mut &[][..], 16),
            Err(DecodeError::BadRiffSize)
        ));

        let duplicate = [
            b'C', b'O', b'D', b'E', 0, 0, 0, 0, b'C', b'O', b'D', b'E', 0, 0, 0, 0,
        ];
        assert!(matches!(
            read_riff_chunks(&mut &duplicate[..], 28),
            Err(DecodeError::DuplicateChunk(ref name)) if name == "CODE"
        ));
    }

    #[test]
    fn code_switch_requires_its_physical_jump_table() {
        let mut riff = b"RIFF".to_vec();
        riff.extend(32u32.to_le_bytes());
        riff.extend(b"SCR ");
        riff.extend(b"CODE");
        riff.extend(12u32.to_le_bytes());
        riff.extend(8u32.to_le_bytes());
        riff.extend([
            OPCODE_SWITCH,
            0,
            0,
            0,
            0,
            OPCODE_END,
            OPCODE_NOP,
            OPCODE_NOP,
        ]);

        assert!(matches!(
            decode_script(&mut &riff[..]),
            Err(DecodeError::BadSwitchId {
                id: 0,
                table_count: 0
            })
        ));
    }
}

#[derive(Debug)]
struct RiffChunk {
    data: Vec<u8>,
    body_offset: usize,
}

fn read_riff_chunks<R: io::Read>(
    read: &mut R,
    riff_size: usize,
) -> Result<HashMap<String, RiffChunk>, DecodeError> {
    if riff_size < 12 {
        return Err(DecodeError::BadRiffSize);
    }
    let mut chunks = HashMap::new();
    let mut offset = 12usize;

    while offset < riff_size {
        if riff_size - offset < 8 {
            return Err(DecodeError::BadRiffSize);
        }
        let chunk_name = read.read_4byte()?;
        let chunk_size = read.read_u32()? as usize;

        let chunk_end = offset
            .checked_add(8)
            .and_then(|body| body.checked_add(chunk_size))
            .ok_or(DecodeError::BadRiffSize)?;
        if chunk_end > riff_size {
            return Err(DecodeError::BadRiffSize);
        }

        let mut chunk_data = vec![0u8; chunk_size];
        read.read_exact(&mut chunk_data)?;

        let chunk_name = std::str::from_utf8(&chunk_name)?.to_string();
        if chunks.contains_key(&chunk_name) {
            return Err(DecodeError::DuplicateChunk(chunk_name));
        }
        chunks.insert(
            chunk_name,
            RiffChunk {
                data: chunk_data,
                body_offset: offset + 8,
            },
        );

        offset = chunk_end;
    }

    Ok(chunks)
}

fn decode_script_impl<R: io::Read>(
    read: &mut R,
    external_backing: Option<&[u8]>,
) -> Result<Script, DecodeError> {
    /*
     * Step 1: Decode RIFF header.
     */

    let riff_head = read.read_4byte()?;
    let riff_size = read.read_u32()? as usize;
    let riff_name = read.read_4byte()?;

    if riff_head.cmp(b"RIFF") != Ordering::Equal || riff_name.cmp(b"SCR ") != Ordering::Equal {
        return Err(DecodeError::ScriptError(
            "Not a valid FoMT script binary (invalid magics)",
        ));
    }

    /*
     * Step 2: Decode RIFF structure (chunks).
     */

    let chunks = read_riff_chunks(read, riff_size)?;

    /*
     * Step 2.1: Decode CODE
     */

    let code_data = match chunks.get("CODE") {
        Some(code_chunk) => decode_code_chunk(&code_chunk.data)?,
        None => return Err(DecodeError::MissingCodeChunk),
    };

    /*
     * Step 3: Find jump target and sources.
     */

    /* Step 3.1: Scan code for jumps. */

    let jump_map = get_code_jumps(code_data)?;

    /* Step 3.2: Decode JUMP chunk, if any. */

    let jumps = match chunks.get("JUMP") {
        Some(chunk) => decode_jump_chunk(&chunk.data)?,
        None => DecodedJumpChunk::default(),
    };

    let case_map = build_case_map(&jumps);

    /* Step 4: Disassemble, while adding case and label meta-instructions */

    let instructions = disassemble(code_data, &jump_map, &case_map)?;

    let mut switch_ids = instructions
        .iter()
        .filter_map(|ins| match ins {
            Ins::Switch(id) => Some(id.0),
            _ => None,
        })
        .collect::<Vec<_>>();
    switch_ids.sort_unstable();
    if let Some(id) = switch_ids
        .windows(2)
        .find_map(|pair| (pair[0] == pair[1]).then_some(pair[0]))
    {
        return Err(DecodeError::DuplicateSwitchId(id));
    }
    switch_ids.dedup();
    let table_count = jumps.case_tables.len();
    if let Some(id) = switch_ids.iter().copied().find(|id| *id >= table_count) {
        return Err(DecodeError::BadSwitchId { id, table_count });
    }
    let expected_tables = switch_ids.last().map_or(0, |id| id + 1);
    if expected_tables != table_count {
        return Err(DecodeError::BadSwitchTableCount {
            expected: expected_tables,
            actual: table_count,
        });
    }

    for (switch_id, table) in jumps.case_tables.iter().enumerate() {
        let mut values = std::collections::BTreeSet::new();
        for (case, _) in &table.0 {
            if !values.insert(*case) {
                return match case {
                    CaseEnum::Default => Err(DecodeError::DuplicateDefault(switch_id)),
                    CaseEnum::Val(value) => Err(DecodeError::DuplicateCaseValue {
                        switch_id,
                        value: *value,
                    }),
                };
            }
        }
    }

    /* Step 5: Get string table */

    let strings = match chunks.get("STR ") {
        Some(chunk) => {
            let backing = external_backing
                .and_then(|all| all.get(chunk.body_offset..))
                .unwrap_or(&chunk.data);
            decode_string_chunk(&chunk.data, backing)?
        }
        None => vec![],
    };

    /* DONE */

    Ok(Script::new(instructions, strings))
}

pub fn decode_script<R: io::Read>(read: &mut R) -> Result<Script, DecodeError> {
    decode_script_impl(read, None)
}

/// Decode a RIFF body while allowing STR offsets to address bytes beyond the
/// declared RIFF/STR lengths. Some modified ROMs relocate strings this way,
/// matching the game's unchecked `string_pool + offset` lookup.
pub fn decode_script_with_backing(
    riff: &[u8],
    backing_from_riff_start: &[u8],
) -> Result<Script, DecodeError> {
    decode_script_impl(&mut &riff[..], Some(backing_from_riff_start))
}
