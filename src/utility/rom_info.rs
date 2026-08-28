use std::io;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ScriptTableEntry<'a> {
    Empty { id: usize },
    Script { id: usize, data: &'a [u8] },
}

impl ScriptTableEntry<'_> {
    pub fn id(&self) -> usize {
        match self {
            Self::Empty { id } | Self::Script { id, .. } => *id,
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum FomtVariant {
    FomtUs,
    MfomtUs,
    FomtJp,
    MfomtJp,
}

fn read_u32(from: &[u8]) -> usize {
    (from[0] as usize)
        | ((from[1] as usize) << 8)
        | ((from[2] as usize) << 16)
        | ((from[3] as usize) << 24)
}

pub fn identify_rom(rom: &[u8]) -> Option<FomtVariant> {
    match &rom[0xA0..0xAC] {
        b"HARVESTMOGBA" => Some(FomtVariant::FomtUs),
        b"HM MFOM USA\0" => Some(FomtVariant::MfomtUs),
        b"BOKUMONOGBA\0" => match &rom[0xAC..0xB0] {
            b"A4NJ" => Some(FomtVariant::FomtJp),
            b"BFGJ" => Some(FomtVariant::MfomtJp),
            _ => None,
        },
        _ => None,
    }
}

pub fn get_script_table(rom: &[u8]) -> io::Result<Vec<ScriptTableEntry<'_>>> {
    let variant = identify_rom(rom).unwrap();

    let (addr, table_slot_count) = match variant {
        FomtVariant::FomtUs => (0x080F89D4, 1329),
        FomtVariant::MfomtUs => (0x081014BC, 1416),
        FomtVariant::FomtJp => (0x080F8230, 1329),
        FomtVariant::MfomtJp => (0x0810145C, 1416),
    };

    let mut result = vec![];
    let table_bytes = &rom[addr & 0x01FFFFFF..];

    for i in 0..table_slot_count {
        let script_addr = read_u32(&table_bytes[i * 4..]);
        if script_addr == 0 {
            result.push(ScriptTableEntry::Empty { id: i });
            continue;
        }

        let script_offs = script_addr & 0x01FFFFFF;

        if script_offs + 8 >= rom.len() {
            break;
        }

        let riff_unbound = &rom[script_offs..];
        if &riff_unbound[..4] != b"RIFF" {
            break;
        }
        let riff_len = read_u32(&riff_unbound[4..8]);

        if script_offs + riff_len >= rom.len() {
            break;
        }

        result.push(ScriptTableEntry::Script {
            id: i,
            data: &riff_unbound[0..riff_len],
        });
    }

    Ok(result)
}

/// Compatibility view for consumers that operate only on RIFF bodies. Use
/// `get_script_table` whenever original table IDs or null slots matter.
pub fn get_all_scripts(rom: &[u8]) -> io::Result<Vec<&[u8]>> {
    Ok(get_script_table(rom)?
        .into_iter()
        .filter_map(|entry| match entry {
            ScriptTableEntry::Empty { .. } => None,
            ScriptTableEntry::Script { data, .. } => Some(data),
        })
        .collect())
}
