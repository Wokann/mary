#![cfg(feature = "test_with_roms")]

use std::fs;
use std::path::{Path, PathBuf};

use mary::{
    bytecode::{decode_script_with_backing, encode_script},
    charmap::Charmap,
    decompiler::decompile_script_with_metadata,
    ir::Ins,
    mary_c::{
        format_named_script_with_charmap, parse_callable_table_with_scope, parse_constant_header,
        parse_named_scripts_with_charmap, parse_text_name_table, Options,
    },
    utility::rom_info::{get_script_table, ScriptTableEntry},
};

struct RomCase {
    name: &'static str,
    rom: &'static str,
    target: &'static str,
    slots: usize,
}

const CASES: &[RomCase] = &[
    RomCase {
        name: "fomt-us",
        rom: "rom/fomt.gba",
        target: "MARY_FOMT_US",
        slots: 1329,
    },
    RomCase {
        name: "mfomt-us",
        rom: "rom/mfomt.gba",
        target: "MARY_MFOMT_US",
        slots: 1416,
    },
    RomCase {
        name: "fomt-jp",
        rom: "rom/fomtjp.gba",
        target: "MARY_FOMT_JP",
        slots: 1329,
    },
    RomCase {
        name: "mfomt-jp",
        rom: "rom/mfomtjp.gba",
        target: "MARY_MFOMT_JP",
        slots: 1416,
    },
];

fn local_rom_path(path: &str) -> PathBuf {
    let repository_path = Path::new(path);
    if repository_path.is_file() {
        repository_path.to_owned()
    } else {
        Path::new("..").join(path)
    }
}

fn direct_numeric_argument(call: &str, argument_index: usize) -> bool {
    let mut depth = 0usize;
    let mut current_index = 0usize;
    let mut argument_start = 0usize;
    for (offset, character) in call.char_indices() {
        match character {
            '(' => depth += 1,
            ')' if depth == 0 => {
                let argument = call[argument_start..offset].trim_start();
                return current_index == argument_index
                    && argument
                        .strip_prefix('-')
                        .unwrap_or(argument)
                        .starts_with(|character: char| character.is_ascii_digit());
            }
            ')' => depth -= 1,
            ',' if depth == 0 => {
                if current_index == argument_index {
                    let argument = call[argument_start..offset].trim_start();
                    return argument
                        .strip_prefix('-')
                        .unwrap_or(argument)
                        .starts_with(|character: char| character.is_ascii_digit());
                }
                current_index += 1;
                argument_start = offset + character.len_utf8();
            }
            _ => {}
        }
    }
    false
}

fn assert_complete_semantic_arguments_are_symbolic(case: &RomCase, script_id: usize, source: &str) {
    const COMPLETE_ARGUMENTS: &[(&str, &[usize])] = &[
        ("SetTalkPortrait", &[0]),
        ("SetTalkInteractionCharacter", &[0]),
        ("ChangeMap", &[0]),
        ("PlaySong", &[0, 1]),
        ("GetCharacterLove", &[0]),
        ("GetNpcFriendship", &[0]),
        ("DoesAnimalExist", &[0]),
        ("GetPresentedItemId", &[0]),
        ("VarGet", &[0]),
        ("VarSet", &[0]),
        ("SetEntityFacing", &[1]),
        ("StartEntityEffect", &[1, 2]),
        ("GetFoodIconId", &[0]),
        ("GetArticleIconId", &[0]),
        ("GetToolIconId", &[0]),
        ("AddFoodToRucksack", &[0]),
        ("AddArticleToRucksack", &[0]),
        ("AddToolToRucksack", &[0]),
        ("HasReceivedLetter", &[0]),
        ("CallScript", &[0]),
    ];

    for (callable, argument_indices) in COMPLETE_ARGUMENTS {
        let marker = format!("{callable}(");
        for call in source.split(&marker).skip(1) {
            for &argument_index in *argument_indices {
                assert!(
                    !direct_numeric_argument(call, argument_index),
                    "{} script {} renders complete semantic argument {} of {} as a direct number:\n{}",
                    case.name,
                    script_id,
                    argument_index,
                    callable,
                    source
                );
            }
        }
    }
}

fn verify(case: &RomCase) -> Result<(), Box<dyn std::error::Error>> {
    let rom = fs::read(local_rom_path(case.rom))?;
    let entries = get_script_table(&rom)?;
    assert_eq!(entries.len(), case.slots, "{} slot count", case.name);

    let options = Options::default().define(case.target)?;
    let constants = parse_constant_header(
        &fs::read_to_string("goodies/mary_constants.mary.h")?,
        &options,
    )?;
    let callables = parse_callable_table_with_scope(
        &fs::read_to_string("goodies/mary_callables.mary.h")?,
        &options,
        &constants,
    )?;
    let symbols = parse_text_name_table(
        &fs::read_to_string("goodies/mary_scripts_text.mary.sym")?,
        &options,
    )?;
    let script_table = symbols.script_table()?;
    let mut decompile_scope = callables.scope.clone();
    script_table.add_constants(&mut decompile_scope);
    let charmap = Charmap::parse(&fs::read_to_string("charmap_jp.txt")?)?;
    assert_eq!(script_table.slots().len(), entries.len());
    for entry in &entries {
        assert_eq!(
            script_table.name(entry.id()).is_some(),
            matches!(entry, ScriptTableEntry::Script { .. }),
            "{} slot {} occupancy",
            case.name,
            entry.id(),
        );
    }

    let mut verified = 0;
    let mut symbolic_script_calls = 0;
    for entry in entries {
        let ScriptTableEntry::Script { id, data, backing } = entry else {
            continue;
        };
        let decoded = decode_script_with_backing(data, backing)?;
        let name = symbols
            .script_name(id)
            .ok_or_else(|| format!("{} script {id} has no semantic name", case.name))?
            .to_owned();
        let numbered_name = format!("EventScript_{id:04}");
        if name == numbered_name {
            assert!(
                decoded.strings.is_empty()
                    && decoded.instructions.iter().all(|instruction| matches!(instruction, Ins::Exit)),
                "{} script {id} keeps a numbered placeholder name despite containing semantic work: {:?}",
                case.name,
                decoded.instructions
            );
        }
        assert!(symbols
            .names(id, decoded.strings.len())
            .iter()
            .all(Option::is_some));
        let text_names = symbols.names(id, decoded.strings.len());
        let local_types = symbols.local_types(id, &decompile_scope)?;
        let ast = decompile_script_with_metadata(
            &decoded,
            &decompile_scope,
            &name,
            &text_names,
            &local_types,
        )
        .map_err(|error| format!("{} script {id} decompile: {error}", case.name))?;
        let source = format_named_script_with_charmap(&name, &ast, &charmap)
            .map_err(|error| format!("{} script {id} format: {error}", case.name))?;
        assert_complete_semantic_arguments_are_symbolic(case, id, &source);
        symbolic_script_calls += source.matches("CallScript(EventScript_").count();
        let rebuilt = parse_named_scripts_with_charmap(
            &source,
            &options,
            &callables.scope,
            &script_table,
            &charmap,
        )
        .map_err(|error| format!("{} script {id} parse: {error}\n{source}", case.name))?;
        let encoded = encode_script(&rebuilt.scripts[0].2);
        if data != &encoded[..] {
            let first = data
                .iter()
                .zip(&encoded)
                .position(|(a, b)| a != b)
                .unwrap_or(data.len().min(encoded.len()));
            return Err(format!(
                "{} script {id} byte mismatch at {first:#X}: original {}, rebuilt {}\n{source}",
                case.name,
                data.len(),
                encoded.len()
            )
            .into());
        }
        verified += 1;
    }
    assert!(
        symbolic_script_calls > 0,
        "{} did not render any CallScript target as a script symbol",
        case.name
    );
    println!("{} Mary-C strict round-trip: {verified} scripts", case.name);
    Ok(())
}

#[test]
fn all_four_roms_mary_c_round_trip() -> Result<(), Box<dyn std::error::Error>> {
    for case in CASES {
        verify(case)?;
    }
    Ok(())
}
