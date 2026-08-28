#![cfg(feature = "test_with_roms")]

use std::fs;

use mary::{
    bytecode::{decode_script_with_backing, encode_script},
    charmap::Charmap,
    decompiler::decompile_script_named,
    mary_c::{
        format_named_script_with_charmap, parse_callable_table, parse_named_scripts_with_charmap,
        parse_script_table, parse_text_name_table, Options,
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

fn verify(case: &RomCase) -> Result<(), Box<dyn std::error::Error>> {
    let rom = fs::read(case.rom)?;
    let entries = get_script_table(&rom)?;
    assert_eq!(entries.len(), case.slots, "{} slot count", case.name);

    let options = Options::default().define(case.target)?;
    let callables = parse_callable_table(
        &fs::read_to_string("goodies/mary_callables.mary.h")?,
        &options,
    )?;
    let script_table = parse_script_table(
        &fs::read_to_string("goodies/mary_scripts.mary.h")?,
        &options,
    )?;
    let symbols = parse_text_name_table(
        &fs::read_to_string("goodies/mary_scripts_text.mary.sym")?,
        &options,
    )?;
    let symbol_scripts = symbols.script_table()?;
    assert_eq!(symbol_scripts.slots(), script_table.slots());
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
    for entry in entries {
        let ScriptTableEntry::Script { id, data, backing } = entry else {
            continue;
        };
        let decoded = decode_script_with_backing(data, backing)?;
        let name = format!("EventScript_{id:04}");
        assert_eq!(symbols.script_name(id), Some(name.as_str()));
        assert!(symbols
            .names(id, decoded.strings.len())
            .iter()
            .all(Option::is_some));
        let ast = decompile_script_named(&decoded, &callables.scope, &name)
            .map_err(|error| format!("{} script {id} decompile: {error}", case.name))?;
        let source = format_named_script_with_charmap(&name, &ast, &charmap)
            .map_err(|error| format!("{} script {id} format: {error}", case.name))?;
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
