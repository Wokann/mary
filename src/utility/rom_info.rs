use std::io;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ScriptTableEntry<'a> {
    Empty {
        id: usize,
    },
    Script {
        id: usize,
        data: &'a [u8],
        backing: &'a [u8],
    },
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
    FomtJp,
    FomtUs,
    FomtEu,
    FomtDe,
    MfomtJp,
    MfomtUs,
}

fn read_u32(from: &[u8]) -> usize {
    (from[0] as usize)
        | ((from[1] as usize) << 8)
        | ((from[2] as usize) << 16)
        | ((from[3] as usize) << 24)
}

pub fn identify_rom(rom: &[u8]) -> Option<FomtVariant> {
    match (rom.get(0xA0..0xAC)?, rom.get(0xAC..0xB0)?) {
        (b"BOKUMONOGBA\0", b"A4NJ") => Some(FomtVariant::FomtJp),
        (b"HARVESTMOGBA", b"A4NE") => Some(FomtVariant::FomtUs),
        (b"HARVESTMOGBA", b"A4NP") => Some(FomtVariant::FomtEu),
        (b"HARVESTMOGER", b"A4ND") => Some(FomtVariant::FomtDe),
        (b"BOKUMONOGBA\0", b"BFGJ") => Some(FomtVariant::MfomtJp),
        (b"HM MFOM USA\0", b"BFGE") => Some(FomtVariant::MfomtUs),
        _ => None,
    }
}

pub fn get_script_table(rom: &[u8]) -> io::Result<Vec<ScriptTableEntry<'_>>> {
    let variant = identify_rom(rom).ok_or_else(|| {
        io::Error::new(
            io::ErrorKind::InvalidData,
            "ROM header does not match a supported FoMT/MFoMT variant",
        )
    })?;

    let (addr, table_slot_count) = match variant {
        FomtVariant::FomtJp => (0x080F8230, 1329),
        FomtVariant::FomtUs => (0x080F89D4, 1329),
        FomtVariant::FomtEu => (0x080F8A20, 1329),
        FomtVariant::FomtDe => (0x080F8EBC, 1329),
        FomtVariant::MfomtJp => (0x0810145C, 1416),
        FomtVariant::MfomtUs => (0x081014BC, 1416),
    };

    let mut result = vec![];
    let table_offset = addr & 0x01FFFFFF;
    let table_size = table_slot_count * 4;
    let table_bytes = rom
        .get(table_offset..table_offset + table_size)
        .ok_or_else(|| io::Error::new(io::ErrorKind::UnexpectedEof, "truncated script table"))?;

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
            backing: riff_unbound,
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn short_or_unknown_roms_are_rejected_without_panicking() {
        assert_eq!(identify_rom(&[]), None);
        assert_eq!(identify_rom(&[0; 0xB0]), None);
        assert_eq!(
            get_script_table(&[]).unwrap_err().kind(),
            io::ErrorKind::InvalidData
        );
    }

    #[test]
    fn supported_headers_identify_all_six_targets_in_canonical_order() {
        for (title, code, expected) in [
            (
                b"BOKUMONOGBA\0".as_slice(),
                b"A4NJ".as_slice(),
                FomtVariant::FomtJp,
            ),
            (
                b"HARVESTMOGBA".as_slice(),
                b"A4NE".as_slice(),
                FomtVariant::FomtUs,
            ),
            (
                b"HARVESTMOGBA".as_slice(),
                b"A4NP".as_slice(),
                FomtVariant::FomtEu,
            ),
            (
                b"HARVESTMOGER".as_slice(),
                b"A4ND".as_slice(),
                FomtVariant::FomtDe,
            ),
            (
                b"BOKUMONOGBA\0".as_slice(),
                b"BFGJ".as_slice(),
                FomtVariant::MfomtJp,
            ),
            (
                b"HM MFOM USA\0".as_slice(),
                b"BFGE".as_slice(),
                FomtVariant::MfomtUs,
            ),
        ] {
            let mut header = vec![0; 0xB0];
            header[0xA0..0xAC].copy_from_slice(title);
            header[0xAC..0xB0].copy_from_slice(code);
            assert_eq!(identify_rom(&header), Some(expected));
        }
    }
}
