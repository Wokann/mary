use std::{fmt, ops::Range};

use thiserror::Error;

use crate::utility::rom_info::{identify_rom, FomtVariant, NativeScriptLayout};

pub const GBA_ROM_BASE: usize = 0x0800_0000;
pub const GBA_ROM_MAX_SIZE: usize = 0x0200_0000;
const METADATA_MAGIC: &[u8; 8] = b"MARYTAB1";
const METADATA_VERSION: u32 = 1;
const METADATA_SIZE: usize = 32;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct ScriptLocation {
    pub id: usize,
    pub offset: usize,
    pub riff_len: usize,
    pub allocation_end: usize,
}

impl ScriptLocation {
    pub fn allocation(&self) -> Range<usize> {
        self.offset..self.allocation_end
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ResolvedScriptLayout {
    pub variant: FomtVariant,
    pub native: NativeScriptLayout,
    pub pointer_table_offset: usize,
    pub slot_count: usize,
    pub script_area: Option<Range<usize>>,
    pub scripts: Vec<Option<ScriptLocation>>,
    pub metadata_present: bool,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct PackedScripts {
    pub bytes: Vec<u8>,
    pub relative_offsets: Vec<Option<usize>>,
}

impl PackedScripts {
    pub fn from_slots(slots: &[Option<Vec<u8>>]) -> Result<Self, ImportError> {
        let mut bytes = Vec::new();
        let mut relative_offsets = Vec::with_capacity(slots.len());
        for (id, slot) in slots.iter().enumerate() {
            let Some(riff) = slot else {
                relative_offsets.push(None);
                continue;
            };
            validate_riff(riff)
                .map_err(|reason| ImportError::InvalidCompiledRiff { id, reason })?;
            relative_offsets.push(Some(bytes.len()));
            bytes.extend_from_slice(riff);
            bytes.resize(align4(bytes.len()), 0);
        }
        Ok(Self {
            bytes,
            relative_offsets,
        })
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Occupancy {
    Free,
    ContainsData { offset: usize, byte: u8 },
}

#[derive(Debug, Error, PartialEq, Eq)]
pub enum ImportError {
    #[error("ROM header does not match a supported FoMT/MFoMT variant")]
    UnsupportedRom,
    #[error("address {address:#010X} is outside the GBA ROM address space")]
    AddressOutsideGba { address: usize },
    #[error("address {address:#010X} must be aligned to 4 bytes")]
    AddressNotAligned { address: usize },
    #[error("range {start:#010X}..{end:#010X} is outside this {rom_len:#X}-byte ROM")]
    RangeOutsideRom {
        start: usize,
        end: usize,
        rom_len: usize,
    },
    #[error("script pointer table is truncated")]
    TruncatedPointerTable,
    #[error("script {id} points outside the ROM at {offset:#010X}")]
    ScriptPointerOutsideRom { id: usize, offset: usize },
    #[error("script {id} at {offset:#010X} does not begin with RIFF")]
    InvalidScriptPointer { id: usize, offset: usize },
    #[error("script {id} has an invalid RIFF length")]
    InvalidRiffLength { id: usize },
    #[error("compiled script {id} is not a valid RIFF: {reason}")]
    InvalidCompiledRiff { id: usize, reason: &'static str },
    #[error("Mary pointer-table metadata in the original table is invalid")]
    InvalidMetadata,
    #[error("script table contains no scripts")]
    EmptyScriptTable,
    #[error("current scripts no longer form one contiguous four-byte-aligned block; specify a new script address")]
    NonContiguousScriptArea,
    #[error("script ID {id} is outside the current {slots}-slot pointer table")]
    ScriptIdOutsideTable { id: usize, slots: usize },
    #[error("script ID {id} is currently a null pointer slot")]
    EmptyScriptSlot { id: usize },
    #[error("script {id} needs {needed} bytes including alignment, but its original allocation is only {available} bytes ({start:#010X}..{end:#010X})")]
    SingleScriptTooLarge {
        id: usize,
        needed: usize,
        available: usize,
        start: usize,
        end: usize,
    },
    #[error("packed scripts need {needed} bytes, but the original script area has only {available} bytes ({start:#010X}..{end:#010X})")]
    ScriptAreaTooLarge {
        needed: usize,
        available: usize,
        start: usize,
        end: usize,
    },
    #[error("ranges overlap: {first_start:#010X}..{first_end:#010X} and {second_start:#010X}..{second_end:#010X}")]
    OverlappingRanges {
        first_start: usize,
        first_end: usize,
        second_start: usize,
        second_end: usize,
    },
    #[error("native pointer-table reference at {reference:#010X} contains {actual:#010X}, expected {expected:#010X}")]
    UnexpectedPointerReference {
        reference: usize,
        actual: u32,
        expected: u32,
    },
}

pub fn normalize_rom_offset(address: usize) -> Result<usize, ImportError> {
    if address < GBA_ROM_BASE {
        return (address < GBA_ROM_MAX_SIZE)
            .then_some(address)
            .ok_or(ImportError::AddressOutsideGba { address });
    }
    let offset = address
        .checked_sub(GBA_ROM_BASE)
        .ok_or(ImportError::AddressOutsideGba { address })?;
    (offset < GBA_ROM_MAX_SIZE)
        .then_some(offset)
        .ok_or(ImportError::AddressOutsideGba { address })
}

pub fn gba_pointer(offset: usize) -> Result<u32, ImportError> {
    if offset >= GBA_ROM_MAX_SIZE {
        return Err(ImportError::AddressOutsideGba { address: offset });
    }
    Ok((GBA_ROM_BASE + offset) as u32)
}

pub fn range_occupancy(rom: &[u8], range: Range<usize>) -> Result<Occupancy, ImportError> {
    checked_range(rom, &range)?;
    for (relative, byte) in rom[range.clone()].iter().copied().enumerate() {
        if byte != 0 && byte != 0xFF {
            return Ok(Occupancy::ContainsData {
                offset: range.start + relative,
                byte,
            });
        }
    }
    Ok(Occupancy::Free)
}

pub fn resolve_script_layout(
    rom: &[u8],
    manual_table: Option<(usize, usize)>,
) -> Result<ResolvedScriptLayout, ImportError> {
    let variant = identify_rom(rom).ok_or(ImportError::UnsupportedRom)?;
    let native = variant.native_script_layout();
    let (pointer_table_offset, slot_count, metadata_present) = match manual_table {
        Some((offset, count)) => {
            require_aligned(offset)?;
            let metadata_magic_present = rom
                .get(native.pointer_table_offset..native.pointer_table_offset + 8)
                .is_some_and(|bytes| bytes == METADATA_MAGIC);
            (offset, count, metadata_magic_present)
        }
        None => read_metadata(rom, native)?
            .map(|metadata| (metadata.table_offset, metadata.slot_count, true))
            .unwrap_or((native.pointer_table_offset, native.slot_count, false)),
    };
    let scripts = read_script_locations(rom, pointer_table_offset, slot_count)?;
    let script_area = contiguous_script_area(&scripts);
    Ok(ResolvedScriptLayout {
        variant,
        native,
        pointer_table_offset,
        slot_count,
        script_area,
        scripts,
        metadata_present,
    })
}

pub fn write_single_script_in_place(
    rom: &mut [u8],
    layout: &ResolvedScriptLayout,
    id: usize,
    riff: &[u8],
) -> Result<Range<usize>, ImportError> {
    validate_riff(riff).map_err(|reason| ImportError::InvalidCompiledRiff { id, reason })?;
    let location = layout
        .scripts
        .get(id)
        .ok_or(ImportError::ScriptIdOutsideTable {
            id,
            slots: layout.slot_count,
        })?
        .ok_or(ImportError::EmptyScriptSlot { id })?;
    let needed = align4(riff.len());
    let allocation = location.allocation();
    let available = allocation.len();
    if needed > available {
        return Err(ImportError::SingleScriptTooLarge {
            id,
            needed,
            available,
            start: allocation.start,
            end: allocation.end,
        });
    }
    rom[location.offset..location.offset + riff.len()].copy_from_slice(riff);
    rom[location.offset + riff.len()..location.allocation_end].fill(0);
    Ok(location.offset..location.allocation_end)
}

pub fn write_packed_scripts(
    rom: &mut [u8],
    layout: &ResolvedScriptLayout,
    packed: &PackedScripts,
    destination: usize,
    pointer_table_destination: Option<usize>,
) -> Result<(Range<usize>, Range<usize>), ImportError> {
    require_aligned(destination)?;
    let data_end =
        destination
            .checked_add(packed.bytes.len())
            .ok_or(ImportError::AddressOutsideGba {
                address: destination,
            })?;
    let data_range = destination..data_end;
    checked_range(rom, &data_range)?;

    let new_slot_count = packed.relative_offsets.len();
    let table_offset = match pointer_table_destination {
        Some(offset) => {
            require_aligned(offset)?;
            offset
        }
        None => layout.pointer_table_offset,
    };
    let table_end = new_slot_count
        .checked_mul(4)
        .and_then(|size| table_offset.checked_add(size))
        .ok_or(ImportError::TruncatedPointerTable)?;
    let table_range = table_offset..table_end;
    checked_range(rom, &table_range)?;
    ensure_disjoint(&data_range, &table_range)?;
    ensure_disjoint_from_native_references(layout, &data_range)?;
    ensure_disjoint_from_native_references(layout, &table_range)?;
    if layout.metadata_present || table_offset != layout.native.pointer_table_offset {
        ensure_disjoint(
            &data_range,
            &(layout.native.pointer_table_offset
                ..layout.native.pointer_table_offset + METADATA_SIZE),
        )?;
        ensure_disjoint(
            &table_range,
            &(layout.native.pointer_table_offset
                ..layout.native.pointer_table_offset + METADATA_SIZE),
        )?;
    }

    rom[data_range.clone()].copy_from_slice(&packed.bytes);
    write_pointer_table(rom, table_offset, destination, &packed.relative_offsets)?;
    if table_offset == layout.pointer_table_offset && new_slot_count < layout.slot_count {
        let old_end = table_offset + layout.slot_count * 4;
        rom[table_range.end..old_end].fill(0);
    }

    // An in-place whole-table rebuild owns the complete original allocation.
    // Clear its unused tail so bytes from longer old scripts cannot survive.
    if pointer_table_destination.is_none() {
        if let Some(area) = &layout.script_area {
            if destination == area.start && data_range.end <= area.end {
                rom[data_range.end..area.end].fill(0);
            }
        }
    }

    if table_offset != layout.pointer_table_offset {
        patch_pointer_table_references(rom, layout, table_offset)?;
        write_metadata(
            rom,
            layout.native,
            Metadata {
                table_offset,
                slot_count: new_slot_count,
                script_start: data_range.start,
                script_end: data_range.end,
            },
        )?;
    } else if layout.metadata_present {
        write_metadata(
            rom,
            layout.native,
            Metadata {
                table_offset,
                slot_count: new_slot_count,
                script_start: data_range.start,
                script_end: data_range.end,
            },
        )?;
    }

    Ok((data_range, table_range))
}

pub fn update_single_pointer(
    rom: &mut [u8],
    layout: &ResolvedScriptLayout,
    id: usize,
    destination: usize,
) -> Result<(), ImportError> {
    if id >= layout.slot_count {
        return Err(ImportError::ScriptIdOutsideTable {
            id,
            slots: layout.slot_count,
        });
    }
    let entry = layout.pointer_table_offset + id * 4;
    checked_range(rom, &(entry..entry + 4))?;
    rom[entry..entry + 4].copy_from_slice(&gba_pointer(destination)?.to_le_bytes());
    Ok(())
}

pub fn write_single_script_at(
    rom: &mut [u8],
    layout: &ResolvedScriptLayout,
    id: usize,
    riff: &[u8],
    destination: usize,
) -> Result<Range<usize>, ImportError> {
    require_aligned(destination)?;
    validate_riff(riff).map_err(|reason| ImportError::InvalidCompiledRiff { id, reason })?;
    let end =
        destination
            .checked_add(align4(riff.len()))
            .ok_or(ImportError::AddressOutsideGba {
                address: destination,
            })?;
    let range = destination..end;
    checked_range(rom, &range)?;
    ensure_disjoint(
        &range,
        &(layout.pointer_table_offset..layout.pointer_table_offset + layout.slot_count * 4),
    )?;
    ensure_disjoint_from_native_references(layout, &range)?;
    if layout.metadata_present {
        ensure_disjoint(
            &range,
            &(layout.native.pointer_table_offset
                ..layout.native.pointer_table_offset + METADATA_SIZE),
        )?;
    }
    rom[destination..destination + riff.len()].copy_from_slice(riff);
    rom[destination + riff.len()..end].fill(0);
    update_single_pointer(rom, layout, id, destination)?;
    Ok(range)
}

fn read_script_locations(
    rom: &[u8],
    table_offset: usize,
    slot_count: usize,
) -> Result<Vec<Option<ScriptLocation>>, ImportError> {
    let table_end = slot_count
        .checked_mul(4)
        .and_then(|size| table_offset.checked_add(size))
        .ok_or(ImportError::TruncatedPointerTable)?;
    if table_end > rom.len() {
        return Err(ImportError::TruncatedPointerTable);
    }
    let mut result = Vec::with_capacity(slot_count);
    for id in 0..slot_count {
        let entry = table_offset + id * 4;
        let pointer = read_u32(&rom[entry..entry + 4]) as usize;
        if pointer == 0 {
            result.push(None);
            continue;
        }
        let offset = normalize_rom_offset(pointer)?;
        let header = rom
            .get(offset..offset + 8)
            .ok_or(ImportError::ScriptPointerOutsideRom { id, offset })?;
        if &header[..4] != b"RIFF" {
            return Err(ImportError::InvalidScriptPointer { id, offset });
        }
        let riff_len = read_u32(&header[4..8]) as usize;
        if riff_len < 12
            || offset
                .checked_add(riff_len)
                .is_none_or(|end| end > rom.len())
        {
            return Err(ImportError::InvalidRiffLength { id });
        }
        let allocation_end = align4(offset + riff_len);
        if allocation_end > rom.len() {
            return Err(ImportError::InvalidRiffLength { id });
        }
        result.push(Some(ScriptLocation {
            id,
            offset,
            riff_len,
            allocation_end,
        }));
    }
    Ok(result)
}

fn contiguous_script_area(scripts: &[Option<ScriptLocation>]) -> Option<Range<usize>> {
    let mut locations = scripts.iter().flatten();
    let first = *locations.next()?;
    let mut previous = first;
    for current in locations {
        if current.offset != previous.allocation_end {
            return None;
        }
        previous = *current;
    }
    Some(first.offset..previous.allocation_end)
}

fn write_pointer_table(
    rom: &mut [u8],
    table_offset: usize,
    script_start: usize,
    relative_offsets: &[Option<usize>],
) -> Result<(), ImportError> {
    for (id, relative) in relative_offsets.iter().enumerate() {
        let value = relative
            .map(|relative| gba_pointer(script_start + relative))
            .transpose()?
            .unwrap_or(0);
        let entry = table_offset + id * 4;
        rom[entry..entry + 4].copy_from_slice(&value.to_le_bytes());
    }
    Ok(())
}

fn patch_pointer_table_references(
    rom: &mut [u8],
    layout: &ResolvedScriptLayout,
    new_table_offset: usize,
) -> Result<(), ImportError> {
    let expected = gba_pointer(layout.pointer_table_offset)?;
    let replacement = gba_pointer(new_table_offset)?;
    let rom_len = rom.len();
    for reference in layout.native.pointer_reference_offsets {
        let bytes = rom
            .get_mut(reference..reference + 4)
            .ok_or(ImportError::RangeOutsideRom {
                start: reference,
                end: reference + 4,
                rom_len,
            })?;
        let actual = read_u32(bytes);
        if actual != expected {
            return Err(ImportError::UnexpectedPointerReference {
                reference,
                actual,
                expected,
            });
        }
        bytes.copy_from_slice(&replacement.to_le_bytes());
    }
    Ok(())
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
struct Metadata {
    table_offset: usize,
    slot_count: usize,
    script_start: usize,
    script_end: usize,
}

fn read_metadata(rom: &[u8], native: NativeScriptLayout) -> Result<Option<Metadata>, ImportError> {
    let start = native.pointer_table_offset;
    let Some(bytes) = rom.get(start..start + METADATA_SIZE) else {
        return Err(ImportError::TruncatedPointerTable);
    };
    if &bytes[..8] != METADATA_MAGIC {
        return Ok(None);
    }
    let words = [
        read_u32(&bytes[8..12]),
        read_u32(&bytes[12..16]),
        read_u32(&bytes[16..20]),
        read_u32(&bytes[20..24]),
        read_u32(&bytes[24..28]),
        read_u32(&bytes[28..32]),
    ];
    let checksum = words[..5]
        .iter()
        .fold(0x4D41_5259u32, |sum, word| sum ^ word);
    if words[0] != METADATA_VERSION || words[5] != checksum {
        return Err(ImportError::InvalidMetadata);
    }
    let metadata = Metadata {
        table_offset: words[1] as usize,
        slot_count: words[2] as usize,
        script_start: words[3] as usize,
        script_end: words[4] as usize,
    };
    if metadata.slot_count == 0
        || metadata.table_offset >= GBA_ROM_MAX_SIZE
        || metadata
            .slot_count
            .checked_mul(4)
            .and_then(|size| metadata.table_offset.checked_add(size))
            .is_none_or(|end| end > rom.len())
        || metadata.script_start > metadata.script_end
        || metadata.script_end > rom.len()
    {
        return Err(ImportError::InvalidMetadata);
    }
    Ok(Some(metadata))
}

fn write_metadata(
    rom: &mut [u8],
    native: NativeScriptLayout,
    metadata: Metadata,
) -> Result<(), ImportError> {
    let start = native.pointer_table_offset;
    let end = start + METADATA_SIZE;
    checked_range(rom, &(start..end))?;
    let words = [
        METADATA_VERSION,
        metadata.table_offset as u32,
        metadata.slot_count as u32,
        metadata.script_start as u32,
        metadata.script_end as u32,
    ];
    let checksum = words.iter().fold(0x4D41_5259u32, |sum, word| sum ^ word);
    rom[start..start + 8].copy_from_slice(METADATA_MAGIC);
    for (index, word) in words.into_iter().chain([checksum]).enumerate() {
        let offset = start + 8 + index * 4;
        rom[offset..offset + 4].copy_from_slice(&word.to_le_bytes());
    }
    Ok(())
}

fn validate_riff(riff: &[u8]) -> Result<(), &'static str> {
    if riff.len() < 12 || &riff[..4] != b"RIFF" || &riff[8..12] != b"SCR " {
        return Err("missing RIFF/SCR header");
    }
    let declared = read_u32(&riff[4..8]) as usize;
    if declared != riff.len() {
        return Err("declared RIFF length does not match the compiled byte length");
    }
    Ok(())
}

fn checked_range(rom: &[u8], range: &Range<usize>) -> Result<(), ImportError> {
    if range.start > range.end || range.end > rom.len() {
        return Err(ImportError::RangeOutsideRom {
            start: range.start,
            end: range.end,
            rom_len: rom.len(),
        });
    }
    Ok(())
}

fn ensure_disjoint(first: &Range<usize>, second: &Range<usize>) -> Result<(), ImportError> {
    if first.start < second.end && second.start < first.end {
        return Err(ImportError::OverlappingRanges {
            first_start: first.start,
            first_end: first.end,
            second_start: second.start,
            second_end: second.end,
        });
    }
    Ok(())
}

fn ensure_disjoint_from_native_references(
    layout: &ResolvedScriptLayout,
    range: &Range<usize>,
) -> Result<(), ImportError> {
    for reference in layout.native.pointer_reference_offsets {
        ensure_disjoint(range, &(reference..reference + 4))?;
    }
    Ok(())
}

fn align4(value: usize) -> usize {
    (value + 3) & !3
}

fn require_aligned(address: usize) -> Result<(), ImportError> {
    if !address.is_multiple_of(4) {
        return Err(ImportError::AddressNotAligned { address });
    }
    Ok(())
}

fn read_u32(bytes: &[u8]) -> u32 {
    u32::from_le_bytes(bytes[..4].try_into().expect("four-byte input"))
}

impl fmt::Display for Occupancy {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Free => f.write_str("free (only 00/FF)"),
            Self::ContainsData { offset, byte } => {
                write!(f, "contains {byte:02X} at {offset:#010X}")
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn riff(len: usize, fill: u8) -> Vec<u8> {
        assert!(len >= 12);
        let mut result = vec![fill; len];
        result[..4].copy_from_slice(b"RIFF");
        result[4..8].copy_from_slice(&(len as u32).to_le_bytes());
        result[8..12].copy_from_slice(b"SCR ");
        result
    }

    fn synthetic_fomt_jp() -> Vec<u8> {
        let mut rom = vec![0xFF; 0x200000];
        rom[0xA0..0xAC].copy_from_slice(b"BOKUMONOGBA\0");
        rom[0xAC..0xB0].copy_from_slice(b"A4NJ");
        let native = FomtVariant::FomtJp.native_script_layout();
        let table_pointer = gba_pointer(native.pointer_table_offset).unwrap();
        for reference in native.pointer_reference_offsets {
            rom[reference..reference + 4].copy_from_slice(&table_pointer.to_le_bytes());
        }
        let table_end = native.pointer_table_offset + native.slot_count * 4;
        rom[native.pointer_table_offset..table_end].fill(0);
        let first = 0x180000;
        let first_riff = riff(13, 1);
        rom[first..first + first_riff.len()].copy_from_slice(&first_riff);
        rom[first + first_riff.len()..align4(first + first_riff.len())].fill(0);
        rom[native.pointer_table_offset + 4..native.pointer_table_offset + 8]
            .copy_from_slice(&gba_pointer(first).unwrap().to_le_bytes());
        rom
    }

    #[test]
    fn packing_uses_zero_filled_four_byte_alignment() {
        let packed = PackedScripts::from_slots(&[
            None,
            Some(riff(13, 1)),
            Some(riff(16, 2)),
            Some(riff(14, 3)),
        ])
        .unwrap();
        assert_eq!(
            packed.relative_offsets,
            vec![None, Some(0), Some(16), Some(32)]
        );
        assert_eq!(packed.bytes.len(), 48);
        assert_eq!(&packed.bytes[13..16], &[0, 0, 0]);
        assert_eq!(&packed.bytes[46..48], &[0, 0]);
    }

    #[test]
    fn occupancy_accepts_only_zero_and_ff() {
        let rom = [0, 0xFF, 0, 0x7E, 0xFF];
        assert_eq!(range_occupancy(&rom, 0..3).unwrap(), Occupancy::Free);
        assert_eq!(
            range_occupancy(&rom, 1..5).unwrap(),
            Occupancy::ContainsData {
                offset: 3,
                byte: 0x7E
            }
        );
    }

    #[test]
    fn address_parser_accepts_file_offsets_and_gba_pointers() {
        assert_eq!(normalize_rom_offset(0x123456).unwrap(), 0x123456);
        assert_eq!(normalize_rom_offset(0x08123456).unwrap(), 0x123456);
        assert_eq!(gba_pointer(0x123456).unwrap(), 0x08123456);
        assert!(normalize_rom_offset(0x0A000000).is_err());
    }

    #[test]
    fn explicit_destinations_must_be_four_byte_aligned() {
        assert_eq!(
            require_aligned(0x123455),
            Err(ImportError::AddressNotAligned { address: 0x123455 })
        );
        assert!(require_aligned(0x123454).is_ok());
    }

    #[test]
    fn single_in_place_can_use_alignment_padding_but_not_cross_it() {
        let mut rom = synthetic_fomt_jp();
        let layout = resolve_script_layout(&rom, None).unwrap();
        assert_eq!(layout.scripts[1].unwrap().allocation(), 0x180000..0x180010);

        write_single_script_in_place(&mut rom, &layout, 1, &riff(13, 2)).unwrap();
        assert_eq!(&rom[0x18000D..0x180010], &[0, 0, 0]);
        assert!(matches!(
            write_single_script_in_place(&mut rom, &layout, 1, &riff(17, 3)),
            Err(ImportError::SingleScriptTooLarge {
                needed: 20,
                available: 16,
                ..
            })
        ));
    }

    #[test]
    fn relocated_expanded_table_is_found_again_from_metadata() {
        let mut rom = synthetic_fomt_jp();
        let original = resolve_script_layout(&rom, None).unwrap();
        let mut slots = vec![None; original.slot_count + 2];
        slots[1] = Some(riff(13, 1));
        slots[original.slot_count + 1] = Some(riff(14, 2));
        let packed = PackedScripts::from_slots(&slots).unwrap();
        let data_start = 0x190000;
        let table_start = 0x1A0000;

        write_packed_scripts(&mut rom, &original, &packed, data_start, Some(table_start)).unwrap();

        let resolved = resolve_script_layout(&rom, None).unwrap();
        assert!(resolved.metadata_present);
        assert_eq!(resolved.pointer_table_offset, table_start);
        assert_eq!(resolved.slot_count, original.slot_count + 2);
        assert!(resolved.scripts[0].is_none());
        assert!(resolved.scripts[original.slot_count].is_none());
        assert_eq!(
            resolved.scripts[original.slot_count + 1].unwrap().offset,
            data_start + 16
        );
        for reference in original.native.pointer_reference_offsets {
            assert_eq!(
                read_u32(&rom[reference..reference + 4]),
                gba_pointer(table_start).unwrap()
            );
        }
    }

    #[test]
    fn pointer_table_can_grow_in_place_and_shrink_clears_old_entries() {
        let mut rom = synthetic_fomt_jp();
        let original = resolve_script_layout(&rom, None).unwrap();
        let mut expanded_slots = vec![None; original.slot_count + 2];
        expanded_slots[1] = Some(riff(13, 1));
        expanded_slots[original.slot_count + 1] = Some(riff(14, 2));
        let expanded = PackedScripts::from_slots(&expanded_slots).unwrap();
        let data_start = 0x190000;

        let (_, table_range) =
            write_packed_scripts(&mut rom, &original, &expanded, data_start, None).unwrap();
        assert_eq!(
            table_range,
            original.pointer_table_offset
                ..original.pointer_table_offset + (original.slot_count + 2) * 4
        );
        assert_ne!(
            &rom[original.pointer_table_offset + (original.slot_count + 1) * 4
                ..original.pointer_table_offset + (original.slot_count + 2) * 4],
            &[0, 0, 0, 0]
        );

        let mut expanded_layout = original.clone();
        expanded_layout.slot_count += 2;
        let shrunk_slots = vec![None; original.slot_count];
        let shrunk = PackedScripts::from_slots(&shrunk_slots).unwrap();
        write_packed_scripts(&mut rom, &expanded_layout, &shrunk, data_start, None).unwrap();
        assert_eq!(
            &rom[original.pointer_table_offset + original.slot_count * 4
                ..original.pointer_table_offset + (original.slot_count + 2) * 4],
            &[0; 8]
        );
    }

    #[test]
    fn relocated_ranges_must_not_overlap_each_other_or_native_state() {
        let mut rom = synthetic_fomt_jp();
        let layout = resolve_script_layout(&rom, None).unwrap();
        let packed = PackedScripts::from_slots(&[Some(riff(16, 1))]).unwrap();

        assert!(matches!(
            write_packed_scripts(&mut rom, &layout, &packed, 0x190000, Some(0x190000)),
            Err(ImportError::OverlappingRanges { .. })
        ));
        assert!(matches!(
            write_single_script_at(
                &mut rom,
                &layout,
                1,
                &riff(16, 1),
                layout.native.pointer_reference_offsets[0]
            ),
            Err(ImportError::OverlappingRanges { .. })
        ));
    }

    #[test]
    fn corrupted_relocation_metadata_is_rejected() {
        let mut rom = synthetic_fomt_jp();
        let layout = resolve_script_layout(&rom, None).unwrap();
        let slots = vec![None; layout.slot_count + 1];
        let packed = PackedScripts::from_slots(&slots).unwrap();
        write_packed_scripts(&mut rom, &layout, &packed, 0x190000, Some(0x1A0000)).unwrap();

        rom[layout.native.pointer_table_offset + 31] ^= 1;
        assert_eq!(
            resolve_script_layout(&rom, None),
            Err(ImportError::InvalidMetadata)
        );
        let recovered = resolve_script_layout(&rom, Some((0x1A0000, layout.slot_count + 1)))
            .expect("manual table selection must bypass damaged discovery metadata");
        assert_eq!(recovered.pointer_table_offset, 0x1A0000);
        assert_eq!(recovered.slot_count, layout.slot_count + 1);
        assert!(recovered.metadata_present);
    }

    #[test]
    fn later_relocations_cannot_overwrite_discovery_metadata() {
        let mut rom = synthetic_fomt_jp();
        let original = resolve_script_layout(&rom, None).unwrap();
        let mut slots = vec![None; original.slot_count];
        slots[1] = Some(riff(16, 1));
        let packed = PackedScripts::from_slots(&slots).unwrap();
        write_packed_scripts(&mut rom, &original, &packed, 0x190000, Some(0x1A0000)).unwrap();
        let relocated = resolve_script_layout(&rom, None).unwrap();

        assert!(matches!(
            write_single_script_at(
                &mut rom,
                &relocated,
                1,
                &riff(16, 2),
                relocated.native.pointer_table_offset
            ),
            Err(ImportError::OverlappingRanges { .. })
        ));
        assert!(matches!(
            write_packed_scripts(
                &mut rom,
                &relocated,
                &packed,
                relocated.native.pointer_table_offset,
                None
            ),
            Err(ImportError::OverlappingRanges { .. })
        ));
    }

    #[test]
    fn whole_in_place_rebuild_clears_the_unused_original_tail() {
        let mut rom = synthetic_fomt_jp();
        let layout = resolve_script_layout(&rom, None).unwrap();
        let area = layout.script_area.clone().unwrap();
        let packed = PackedScripts::from_slots(&vec![None; layout.slot_count]).unwrap();

        write_packed_scripts(&mut rom, &layout, &packed, area.start, None).unwrap();
        assert!(rom[area].iter().all(|byte| *byte == 0));
    }
}
