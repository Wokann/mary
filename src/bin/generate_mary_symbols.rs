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

fn condition(selected: [bool; 4], within: [bool; 4]) -> Option<String> {
    if selected == within {
        return None;
    }
    let derived = [
        ("MARY_FOMT", [true, false, true, false]),
        ("MARY_MFOMT", [false, true, false, true]),
        ("MARY_US", [true, true, false, false]),
        ("MARY_JP", [false, false, true, true]),
    ];
    for (name, mask) in derived {
        if std::array::from_fn(|index| within[index] && mask[index]) == selected {
            return Some(format!("defined({name})"));
        }
    }
    let names = [
        "MARY_FOMT_US",
        "MARY_MFOMT_US",
        "MARY_FOMT_JP",
        "MARY_MFOMT_JP",
    ];
    Some(
        selected
            .into_iter()
            .zip(names)
            .filter(|(yes, _)| *yes)
            .map(|(_, name)| format!("defined({name})"))
            .collect::<Vec<_>>()
            .join(" || "),
    )
}

fn begin(out: &mut String, selected: [bool; 4], within: [bool; 4], indent: &str) {
    if let Some(condition) = condition(selected, within) {
        out.push_str(&format!("{indent}#if {condition}\n"));
    }
}
fn end(out: &mut String, selected: [bool; 4], within: [bool; 4], indent: &str) {
    if condition(selected, within).is_some() {
        out.push_str(indent);
        out.push_str("#endif\n");
    }
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let args: Vec<_> = env::args_os().collect();
    if args.len() != 6 {
        return Err("usage: generate_mary_symbols FOMT_US MFOMT_US FOMT_JP MFOMT_JP OUTPUT".into());
    }
    let targets = [
        slots(Path::new(&args[1]))?,
        slots(Path::new(&args[2]))?,
        slots(Path::new(&args[3]))?,
        slots(Path::new(&args[4]))?,
    ];
    let max = targets.iter().map(Vec::len).max().unwrap_or(0);
    let mut out = String::from(
        "// Decompiler-only ordered symbols. Post-preprocess positions are IDs.\n\
         #if defined(MARY_FOMT_US)\n\
         #define MARY_FOMT\n\
         #define MARY_US\n\
         #elif defined(MARY_MFOMT_US)\n\
         #define MARY_MFOMT\n\
         #define MARY_US\n\
         #elif defined(MARY_FOMT_JP)\n\
         #define MARY_FOMT\n\
         #define MARY_JP\n\
         #elif defined(MARY_MFOMT_JP)\n\
         #define MARY_MFOMT\n\
         #define MARY_JP\n\
         #endif\n\
         mary_script_symbols\n{\n",
    );
    let all_targets = [true; 4];
    let mut active_scripts: Option<[bool; 4]> = None;
    for id in 0..max {
        let present =
            std::array::from_fn(|target| targets[target].get(id).is_some_and(|slot| slot.present));
        let exists = std::array::from_fn(|target| targets[target].get(id).is_some());
        if !exists.into_iter().any(|yes| yes) {
            continue;
        }
        let selected = if present.into_iter().any(|yes| yes) {
            present
        } else {
            exists
        };
        if active_scripts != Some(selected) {
            if let Some(active) = active_scripts {
                end(&mut out, active, all_targets, "");
            }
            begin(&mut out, selected, all_targets, "");
            active_scripts = Some(selected);
        }
        out.push_str(&format!("    // Script ID: {id:04}\n"));
        if !present.into_iter().any(|yes| yes) {
            out.push_str("    NULL,\n\n");
            continue;
        }
        out.push_str(&format!("    EventScript_{id:04}\n    {{\n"));
        let max_texts = (0..4)
            .filter(|&target| present[target])
            .map(|target| targets[target][id].texts)
            .max()
            .unwrap_or(0);
        let mut text = 0;
        while text < max_texts {
            let text_present =
                std::array::from_fn(|target| present[target] && targets[target][id].texts > text);
            let mut end_text = text + 1;
            while end_text < max_texts {
                let next = std::array::from_fn(|target| {
                    present[target] && targets[target][id].texts > end_text
                });
                if next != text_present {
                    break;
                }
                end_text += 1;
            }
            let needs_text_condition = text_present != selected;
            if needs_text_condition {
                begin(&mut out, text_present, selected, "    ");
            }
            for text_id in text..end_text {
                out.push_str(&format!(
                    "        gText_EventScript_{id:04}_{text_id:03},\n"
                ));
            }
            if needs_text_condition {
                end(&mut out, text_present, selected, "    ");
            }
            text = end_text;
        }
        out.push_str("    },\n\n");
    }
    if let Some(active) = active_scripts {
        end(&mut out, active, all_targets, "");
    }
    out.push_str("};\n");
    fs::write(&args[5], out)?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::condition;

    #[test]
    fn derived_conditions_are_relative_to_the_parent_script_range() {
        let all = [true; 4];
        let mfomt = [false, true, false, true];
        assert_eq!(
            condition(mfomt, all).as_deref(),
            Some("defined(MARY_MFOMT)")
        );
        assert_eq!(
            condition([false, false, false, true], mfomt).as_deref(),
            Some("defined(MARY_JP)")
        );
        assert_eq!(condition(mfomt, mfomt), None);
    }
}
