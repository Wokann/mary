#[cfg(feature = "test_with_roms")]
mod tests {
    use std::{
        fs, io,
        path::{Path, PathBuf},
    };

    use mary::utility::rom_info::FomtVariant;
    use thiserror::Error;

    #[derive(Clone, Copy)]
    struct RomCase {
        name: &'static str,
        rom_path: &'static str,
        variant: FomtVariant,
        slots: usize,
    }

    const CASES: [RomCase; 6] = [
        RomCase {
            name: "fomt-jp",
            rom_path: "rom/fomt_jp.gba",
            variant: FomtVariant::FomtJp,
            slots: 1329,
        },
        RomCase {
            name: "fomt-us",
            rom_path: "rom/fomt_us.gba",
            variant: FomtVariant::FomtUs,
            slots: 1329,
        },
        RomCase {
            name: "fomt-eu",
            rom_path: "rom/fomt_eu.gba",
            variant: FomtVariant::FomtEu,
            slots: 1329,
        },
        RomCase {
            name: "fomt-de",
            rom_path: "rom/fomt_de.gba",
            variant: FomtVariant::FomtDe,
            slots: 1329,
        },
        RomCase {
            name: "mfomt-jp",
            rom_path: "rom/mfomt_jp.gba",
            variant: FomtVariant::MfomtJp,
            slots: 1416,
        },
        RomCase {
            name: "mfomt-us",
            rom_path: "rom/mfomt_us.gba",
            variant: FomtVariant::MfomtUs,
            slots: 1416,
        },
    ];

    #[derive(Debug, Error)]
    enum TestFailure {
        #[error("I/O error")]
        Io(#[from] io::Error),
        #[error("script {script_id} could not be decoded: {source}")]
        Decode {
            script_id: usize,
            source: mary::bytecode::DecodeError,
        },
        #[error("byte round trip failed on script {script_id}; dumps: {dump_dir}")]
        ScriptFailed { script_id: usize, dump_dir: PathBuf },
    }

    fn local_rom_path(path: &str) -> PathBuf {
        let repository_path = Path::new(path);
        if repository_path.is_file() {
            repository_path.to_owned()
        } else {
            Path::new("..").join(path)
        }
    }

    fn reencode_scripts(case: RomCase) -> Result<(), TestFailure> {
        use mary::{
            bytecode,
            utility::rom_info::{get_script_table, identify_rom, ScriptTableEntry},
        };

        let rom = fs::read(local_rom_path(case.rom_path))?;
        let table = get_script_table(&rom)?;
        assert_eq!(
            identify_rom(&rom),
            Some(case.variant),
            "{} variant",
            case.name
        );
        assert_eq!(table.len(), case.slots, "{} slot count", case.name);
        assert!(matches!(
            table.first(),
            Some(ScriptTableEntry::Empty { id: 0 })
        ));
        assert!(table
            .windows(2)
            .all(|pair| pair[1].id() == pair[0].id() + 1));

        for entry in table {
            let ScriptTableEntry::Script { id, data, .. } = entry else {
                continue;
            };
            let decoded =
                bytecode::decode_script(&mut &data[..]).map_err(|source| TestFailure::Decode {
                    script_id: id,
                    source,
                })?;
            let rebuilt = bytecode::encode_script(&decoded);
            if data != rebuilt {
                let dump_dir = PathBuf::from("test_failures").join(case.name);
                fs::create_dir_all(&dump_dir)?;
                fs::write(dump_dir.join(format!("script_{id}_original.dmp")), data)?;
                fs::write(dump_dir.join(format!("script_{id}_rebuilt.dmp")), rebuilt)?;
                return Err(TestFailure::ScriptFailed {
                    script_id: id,
                    dump_dir,
                });
            }
        }
        Ok(())
    }

    #[test]
    fn all_vanilla_riffs_reencode_byte_exactly() -> Result<(), TestFailure> {
        for case in CASES {
            reencode_scripts(case)?;
        }
        Ok(())
    }
}
