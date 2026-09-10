use std::{env, fs, path::Path};

use mary::{
    bytecode::decode_script_with_backing,
    utility::rom_info::{get_script_table, ScriptTableEntry},
};

#[derive(Clone)]
struct Slot {
    present: bool,
    texts: usize,
}

fn slots(path: &Path) -> Result<Vec<Slot>, Box<dyn std::error::Error>> {
    get_script_table(&fs::read(path)?)?
        .into_iter()
        .map(|entry| {
            Ok(match entry {
                ScriptTableEntry::Empty { .. } => Slot {
                    present: false,
                    texts: 0,
                },
                ScriptTableEntry::Script { data, backing, .. } => Slot {
                    present: true,
                    texts: decode_script_with_backing(data, backing)?.strings.len(),
                },
            })
        })
        .collect()
}

fn render_family(family: &str, us: &[Slot], jp: &[Slot]) -> String {
    let upper = family.to_ascii_uppercase();
    let mut out = format!(
        "// {upper}-only decompiler symbols. Post-preprocess positions are IDs.\n\
mary_script_symbols\n{{\n"
    );
    let max = us.len().max(jp.len());
    for id in 0..max {
        let us_slot = us.get(id);
        let jp_slot = jp.get(id);
        let us_present = us_slot.is_some_and(|slot| slot.present);
        let jp_present = jp_slot.is_some_and(|slot| slot.present);
        if us_slot.is_none() && jp_slot.is_none() {
            continue;
        }
        out.push_str(&format!("    // Script ID: {id:04}\n"));
        if !us_present && !jp_present {
            out.push_str("    NULL,\n\n");
            continue;
        }
        let region = match (us_present, jp_present) {
            (true, false) => Some("REGION_US"),
            (false, true) => Some("REGION_JP"),
            _ => None,
        };
        if let Some(region) = region {
            out.push_str(&format!("    #if defined({region})\n"));
        }
        let indent = if region.is_some() { "        " } else { "    " };
        out.push_str(&format!("{indent}EventScript_{id:04}\n{indent}{{\n"));
        let us_texts = us_slot.map_or(0, |slot| slot.texts);
        let jp_texts = jp_slot.map_or(0, |slot| slot.texts);
        for text_id in 0..us_texts.max(jp_texts) {
            let in_us = us_present && text_id < us_texts;
            let in_jp = jp_present && text_id < jp_texts;
            let text_region = match (in_us, in_jp) {
                (true, false) if jp_present => Some("REGION_US"),
                (false, true) if us_present => Some("REGION_JP"),
                (false, false) => continue,
                _ => None,
            };
            if let Some(text_region) = text_region {
                out.push_str(&format!("{indent}    #if defined({text_region})\n"));
                out.push_str(&format!(
                    "{indent}        gText_EventScript_{id:04}_{text_id:03},\n"
                ));
                out.push_str(&format!("{indent}    #endif\n"));
            } else {
                out.push_str(&format!(
                    "{indent}    gText_EventScript_{id:04}_{text_id:03},\n"
                ));
            }
        }
        out.push_str(&format!("{indent}}},\n"));
        if region.is_some() {
            out.push_str("    #else\n        NULL,\n    #endif\n");
        }
        out.push('\n');
    }
    out.push_str("};\n");
    out
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let args: Vec<_> = env::args_os().collect();
    if args.len() != 6 {
        return Err(
            "usage: generate_mary_symbols FOMT_US MFOMT_US FOMT_JP MFOMT_JP OUTPUT_DIRECTORY"
                .into(),
        );
    }
    let fomt_us = slots(Path::new(&args[1]))?;
    let mfomt_us = slots(Path::new(&args[2]))?;
    let fomt_jp = slots(Path::new(&args[3]))?;
    let mfomt_jp = slots(Path::new(&args[4]))?;
    let output = Path::new(&args[5]);
    fs::create_dir_all(output)?;
    fs::write(
        output.join("fomt_scripts_text.mary.sym"),
        render_family("fomt", &fomt_us, &fomt_jp),
    )?;
    fs::write(
        output.join("mfomt_scripts_text.mary.sym"),
        render_family("mfomt", &mfomt_us, &mfomt_jp),
    )?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::{render_family, Slot};
    use mary::mary_c::{parse_text_name_table, Options};

    #[test]
    fn generated_symbols_separate_family_and_keep_region_differences() {
        let source = render_family(
            "fomt",
            &[
                Slot {
                    present: true,
                    texts: 1,
                },
                Slot {
                    present: true,
                    texts: 2,
                },
            ],
            &[
                Slot {
                    present: true,
                    texts: 1,
                },
                Slot {
                    present: false,
                    texts: 0,
                },
            ],
        );
        assert!(!source.contains("MARY_MFOMT"));
        assert!(source.contains("#if defined(REGION_US)"));
        assert!(source.contains("gText_EventScript_0001_001"));
        for target in ["MARY_FOMT_US", "MARY_FOMT_JP"] {
            let symbols =
                parse_text_name_table(&source, &Options::default().define(target).unwrap())
                    .unwrap();
            assert_eq!(symbols.script_table().unwrap().slots().len(), 2);
        }
    }
}
