use std::{env, fs};

use mary::mary_c::{parse_text_name_table, Options, ScriptSlot};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let args: Vec<_> = env::args_os().collect();
    if args.len() != 4 {
        return Err("usage: generate_mary_script_table SYMBOLS MARY_TARGET OUTPUT".into());
    }
    let target = args[2].to_string_lossy();
    let options = Options::default().define(&target)?;
    let symbols = parse_text_name_table(&fs::read_to_string(&args[1])?, &options)?;
    let scripts = symbols.script_table()?;
    let mut output = String::from("mary_script_table\n{\n");
    for (id, slot) in scripts.slots().iter().enumerate() {
        match slot {
            ScriptSlot::Empty => output.push_str(&format!("    /* 0x{id:04X} */ NULL,\n")),
            ScriptSlot::Script(name) => {
                output.push_str(&format!("    /* 0x{id:04X} */ {name},\n"));
            }
        }
    }
    output.push_str("};\n");
    fs::write(&args[3], output)?;
    Ok(())
}
