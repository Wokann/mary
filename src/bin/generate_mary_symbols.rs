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

fn condition_for(regions: &[(&str, &[Slot])], selected: &[usize]) -> Option<String> {
    (selected.len() != regions.len()).then(|| {
        selected
            .iter()
            .map(|&index| format!("defined(REGION_{})", regions[index].0))
            .collect::<Vec<_>>()
            .join(" || ")
    })
}

fn render_family(family: &str, regions: &[(&str, &[Slot])]) -> String {
    let upper = family.to_ascii_uppercase();
    let mut out = format!(
        "// {upper}-only decompiler symbols. Post-preprocess positions are IDs.\n\
mary_script_symbols\n{{\n"
    );
    let max = regions
        .iter()
        .map(|(_, slots)| slots.len())
        .max()
        .unwrap_or(0);
    for id in 0..max {
        let present = regions
            .iter()
            .enumerate()
            .filter_map(|(index, (_, slots))| {
                slots
                    .get(id)
                    .is_some_and(|slot| slot.present)
                    .then_some(index)
            })
            .collect::<Vec<_>>();
        out.push_str(&format!("    // Script ID: {id:04}\n"));
        if present.is_empty() {
            out.push_str("    NULL,\n\n");
            continue;
        }
        let condition = condition_for(regions, &present);
        if let Some(condition) = &condition {
            out.push_str(&format!("    #if {condition}\n"));
        }
        let indent = if condition.is_some() {
            "        "
        } else {
            "    "
        };
        out.push_str(&format!("{indent}EventScript_{id:04}\n{indent}{{\n"));
        let max_texts = present
            .iter()
            .map(|&index| regions[index].1[id].texts)
            .max()
            .unwrap_or(0);
        for text_id in 0..max_texts {
            let text_regions = present
                .iter()
                .copied()
                .filter(|&index| text_id < regions[index].1[id].texts)
                .collect::<Vec<_>>();
            let text_condition = condition_for(regions, &text_regions);
            if let Some(text_condition) = text_condition {
                out.push_str(&format!("{indent}    #if {text_condition}\n"));
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
        if condition.is_some() {
            out.push_str("    #else\n        NULL,\n    #endif\n");
        }
        out.push('\n');
    }
    out.push_str("};\n");
    out
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let args: Vec<_> = env::args_os().collect();
    if args.len() != 8 {
        return Err(
            "usage: generate_mary_symbols FOMT_JP FOMT_US FOMT_EU FOMT_DE MFOMT_JP MFOMT_US OUTPUT_DIRECTORY".into(),
        );
    }
    let fomt_jp = slots(Path::new(&args[1]))?;
    let fomt_us = slots(Path::new(&args[2]))?;
    let fomt_eu = slots(Path::new(&args[3]))?;
    let fomt_de = slots(Path::new(&args[4]))?;
    let mfomt_jp = slots(Path::new(&args[5]))?;
    let mfomt_us = slots(Path::new(&args[6]))?;
    let output = Path::new(&args[7]);
    fs::create_dir_all(output)?;
    fs::write(
        output.join("fomt_scripts_text.mary.sym"),
        render_family(
            "fomt",
            &[
                ("JP", &fomt_jp),
                ("US", &fomt_us),
                ("EU", &fomt_eu),
                ("DE", &fomt_de),
            ],
        ),
    )?;
    fs::write(
        output.join("mfomt_scripts_text.mary.sym"),
        render_family("mfomt", &[("JP", &mfomt_jp), ("US", &mfomt_us)]),
    )?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::{render_family, Slot};
    use mary::mary_c::{parse_text_name_table, Options};

    #[test]
    fn generated_symbols_separate_family_and_keep_region_differences() {
        let jp = [
            Slot {
                present: true,
                texts: 1,
            },
            Slot {
                present: false,
                texts: 0,
            },
        ];
        let western = [
            Slot {
                present: true,
                texts: 1,
            },
            Slot {
                present: true,
                texts: 2,
            },
        ];
        let de = [
            Slot {
                present: true,
                texts: 1,
            },
            Slot {
                present: true,
                texts: 3,
            },
        ];
        let source = render_family(
            "fomt",
            &[("JP", &jp), ("US", &western), ("EU", &western), ("DE", &de)],
        );
        assert!(!source.contains("MARY_MFOMT"));
        assert!(source.contains("#if defined(REGION_US)"));
        assert!(source.contains("gText_EventScript_0001_001"));
        assert!(
            source.contains("#if defined(REGION_US) || defined(REGION_EU) || defined(REGION_DE)")
        );
        assert!(source.contains("#if defined(REGION_DE)"));
        for (target, texts) in [
            ("MARY_FOMT_JP", 0),
            ("MARY_FOMT_US", 2),
            ("MARY_FOMT_EU", 2),
            ("MARY_FOMT_DE", 3),
        ] {
            let symbols =
                parse_text_name_table(&source, &Options::default().define(target).unwrap())
                    .unwrap();
            assert_eq!(symbols.script_table().unwrap().slots().len(), 2);
            assert_eq!(symbols.text_count(1), texts);
        }
    }
}
