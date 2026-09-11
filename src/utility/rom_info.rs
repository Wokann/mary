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

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct NativeScriptLayout {
    pub pointer_table_offset: usize,
    pub slot_count: usize,
    pub pointer_reference_offsets: [usize; 3],
}

impl FomtVariant {
    pub fn native_script_layout(self) -> NativeScriptLayout {
        match self {
            Self::FomtJp => NativeScriptLayout {
                pointer_table_offset: 0x0F8230,
                slot_count: 1329,
                pointer_reference_offsets: [0x03F510, 0x0DF4D4, 0x0DFCE0],
            },
            Self::FomtUs => NativeScriptLayout {
                pointer_table_offset: 0x0F89D4,
                slot_count: 1329,
                pointer_reference_offsets: [0x03F89C, 0x0DFD20, 0x0E0534],
            },
            Self::FomtEu => NativeScriptLayout {
                pointer_table_offset: 0x0F8A20,
                slot_count: 1329,
                pointer_reference_offsets: [0x03F8B0, 0x0DFD6C, 0x0E0580],
            },
            Self::FomtDe => NativeScriptLayout {
                pointer_table_offset: 0x0F8EBC,
                slot_count: 1329,
                pointer_reference_offsets: [0x03F794, 0x0DFC84, 0x0E0498],
            },
            Self::MfomtJp => NativeScriptLayout {
                pointer_table_offset: 0x10145C,
                slot_count: 1416,
                pointer_reference_offsets: [0x03F7E4, 0x0E7D64, 0x0E8578],
            },
            Self::MfomtUs => NativeScriptLayout {
                pointer_table_offset: 0x1014BC,
                slot_count: 1416,
                pointer_reference_offsets: [0x03FA88, 0x0E8254, 0x0E8A68],
            },
        }
    }
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
    get_script_table_with_override(rom, None)
}

pub fn get_script_table_at(
    rom: &[u8],
    table_offset: usize,
    slot_count: usize,
) -> io::Result<Vec<ScriptTableEntry<'_>>> {
    get_script_table_with_override(rom, Some((table_offset, slot_count)))
}

fn get_script_table_with_override(
    rom: &[u8],
    manual_table: Option<(usize, usize)>,
) -> io::Result<Vec<ScriptTableEntry<'_>>> {
    let layout = crate::rom_import::resolve_script_layout(rom, manual_table)
        .map_err(|error| io::Error::new(io::ErrorKind::InvalidData, error))?;
    Ok(layout
        .scripts
        .into_iter()
        .enumerate()
        .map(|(id, location)| match location {
            Some(location) => ScriptTableEntry::Script {
                id,
                data: &rom[location.offset..location.offset + location.riff_len],
                backing: &rom[location.offset..],
            },
            None => ScriptTableEntry::Empty { id },
        })
        .collect())
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
