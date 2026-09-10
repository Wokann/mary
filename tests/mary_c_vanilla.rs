#![cfg(feature = "test_with_roms")]

use std::collections::{HashMap, HashSet};
use std::fs;
use std::path::{Path, PathBuf};

use mary::{
    bytecode::{decode_script_with_backing, encode_script},
    charmap::Charmap,
    const_scope::ConstScope,
    decompiler::decompile_script_with_metadata,
    ir::Ins,
    mary_c::{
        format_named_script_with_charmap, parse_callable_table_with_scope, parse_constant_header,
        parse_named_scripts_with_charmap, parse_text_name_table, Options,
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

fn local_rom_path(path: &str) -> PathBuf {
    let repository_path = Path::new(path);
    if repository_path.is_file() {
        repository_path.to_owned()
    } else {
        Path::new("..").join(path)
    }
}

fn thumb_call_target(rom: &[u8], pc: usize) -> usize {
    let hi = u16::from_le_bytes(rom[pc..pc + 2].try_into().unwrap());
    let lo = u16::from_le_bytes(rom[pc + 2..pc + 4].try_into().unwrap());
    assert_eq!(hi & 0xF800, 0xF000);
    assert_eq!(lo & 0xF800, 0xF800);
    let raw = (((hi & 0x7FF) as i32) << 12) | (((lo & 0x7FF) as i32) << 1);
    (pc as i32 + 4 + ((raw << 9) >> 9)) as usize
}

#[test]
fn native_player_fishing_states_and_charge_table_match_all_four_roms() {
    const CHARGE_ANIMATIONS: [u16; 42] = [
        358, 362, 366, 370, 374, 378, 382, 26, 30, 34, 38, 42, 46, 50, 462, 466, 470, 474, 478,
        482, 486, 194, 198, 202, 206, 210, 214, 218, 158, 162, 166, 170, 174, 178, 182, 82, 86, 90,
        94, 98, 102, 106,
    ];

    for (case, action_state_handler) in CASES.iter().zip([0x2634Cusize, 0x264C4, 0x260E0, 0x26338])
    {
        let rom = fs::read(local_rom_path(case.rom)).unwrap();
        let jump_table = action_state_handler + 0x90;

        for (state, animation, expected_offset) in [
            (30usize, 118u8, 0x308usize),
            (31, 110, 0x322),
            (32, 114, 0x33E),
        ] {
            let branch_address = u32::from_le_bytes(
                rom[jump_table + state * 4..jump_table + state * 4 + 4]
                    .try_into()
                    .unwrap(),
            );
            let branch_offset = (branch_address - 0x0800_0000) as usize;
            assert_eq!(
                branch_offset,
                action_state_handler + expected_offset,
                "{} player action state {state} branch",
                case.name
            );
            assert_eq!(
                &rom[branch_offset..branch_offset + 2],
                &[animation, 0x25],
                "{} player action state {state} animation",
                case.name
            );
            assert_eq!(
                &rom[branch_offset + 2..branch_offset + 8],
                &[0xB4, 0x6B, 0x20, 0x1C, 0x5C, 0x30],
                "{} player action state {state} current-tool path",
                case.name
            );
        }

        let charge_table_address = u32::from_le_bytes(
            rom[action_state_handler + 0x238..action_state_handler + 0x23C]
                .try_into()
                .unwrap(),
        );
        let charge_table_offset = (charge_table_address - 0x0800_0000) as usize;
        let actual = (0..CHARGE_ANIMATIONS.len())
            .map(|index| {
                u16::from_le_bytes(
                    rom[charge_table_offset + index * 2..charge_table_offset + index * 2 + 2]
                        .try_into()
                        .unwrap(),
                )
            })
            .collect::<Vec<_>>();
        assert_eq!(
            actual, CHARGE_ANIMATIONS,
            "{} complete six-tool by seven-charge animation table",
            case.name
        );
        assert_eq!(
            &actual[35..42],
            &[82, 86, 90, 94, 98, 102, 106],
            "{} fishing-rod charge stages",
            case.name
        );
    }
}

#[test]
fn native_player_raise_arms_states_share_animation_66_on_all_targets() {
    for (case, action_state_handler) in CASES.iter().zip([0x2634Cusize, 0x264C4, 0x260E0, 0x26338])
    {
        let rom = fs::read(local_rom_path(case.rom)).unwrap();
        let jump_table = action_state_handler + 0x90;
        let expected_branch = action_state_handler + 0x46E;

        for state in [10usize, 21, 47, 52] {
            let branch_address = u32::from_le_bytes(
                rom[jump_table + state * 4..jump_table + state * 4 + 4]
                    .try_into()
                    .unwrap(),
            );
            assert_eq!(
                branch_address,
                0x0800_0000 + expected_branch as u32,
                "{} player action state {state}",
                case.name
            );
        }
        assert_eq!(
            &rom[expected_branch..expected_branch + 4],
            &[0x42, 0x25, 0x12, 0xE0],
            "{} shared raise-arms animation branch",
            case.name
        );
    }
}

#[test]
fn native_camera_pan_and_wait_callables_keep_physical_slots_on_all_targets() {
    fn call_target(rom: &[u8], pc: usize) -> usize {
        let hi = u16::from_le_bytes(rom[pc..pc + 2].try_into().unwrap());
        let lo = u16::from_le_bytes(rom[pc + 2..pc + 4].try_into().unwrap());
        assert_eq!(hi & 0xF800, 0xF000);
        assert_eq!(lo & 0xF800, 0xF800);
        let raw = (((hi & 0x7FF) as i32) << 12) | (((lo & 0x7FF) as i32) << 1);
        (pc as i32 + 4 + ((raw << 9) >> 9)) as usize
    }

    for (case, (table, pan_handler, pan_wrapper, wait_handler, wait_wrapper)) in CASES.iter().zip([
        (0x3F904, 0x403AC, 0x1238C, 0x40404, 0x123A4),
        (0x3FAF0, 0x405C0, 0x1247C, 0x40618, 0x12494),
        (0x3F578, 0x40020, 0x1225C, 0x40078, 0x12274),
        (0x3F84C, 0x4031C, 0x12320, 0x40374, 0x12338),
    ]) {
        let rom = fs::read(local_rom_path(case.rom)).unwrap();
        let pan_slot = table + 0x17 * 4;
        assert_eq!(
            u32::from_le_bytes(rom[pan_slot..pan_slot + 4].try_into().unwrap()) & !1,
            0x08000000 + pan_handler as u32,
            "{} PanCameraTo physical callable slot",
            case.name
        );
        assert_eq!(
            call_target(&rom, pan_handler + 0x50),
            pan_wrapper,
            "{} PanCameraTo scene wrapper",
            case.name
        );
        let wait_slot = table + 0x18 * 4;
        assert_eq!(
            u32::from_le_bytes(rom[wait_slot..wait_slot + 4].try_into().unwrap()) & !1,
            0x08000000 + wait_handler as u32,
            "{} WaitForCameraMovement physical callable slot",
            case.name
        );
        assert_eq!(
            call_target(&rom, wait_handler + 0x10),
            wait_wrapper,
            "{} WaitForCameraMovement scene wrapper",
            case.name
        );
    }
}

#[test]
fn native_new_record_audio_slots_exist_only_in_mfomt() {
    const TABLES: [(&str, usize, bool); 4] = [
        ("rom/fomt.gba", 0x13ABF0, false),
        ("rom/fomtjp.gba", 0x13BD34, false),
        ("rom/mfomt.gba", 0x144FF4, true),
        ("rom/mfomtjp.gba", 0x146A64, true),
    ];

    for (path, table_offset, is_mfomt) in TABLES {
        let rom = fs::read(local_rom_path(path)).unwrap();
        let null_sequence =
            u32::from_le_bytes(rom[table_offset..table_offset + 4].try_into().unwrap());

        for slot in 38..=42 {
            let entry_offset = table_offset + slot * 8;
            let sequence =
                u32::from_le_bytes(rom[entry_offset..entry_offset + 4].try_into().unwrap());
            assert_eq!(
                sequence != null_sequence,
                is_mfomt,
                "{path}: audio slot {slot} physical sequence pointer"
            );
        }
    }
}

#[test]
fn native_sickle_audio_family_uses_one_program_on_all_targets() {
    const TABLES: [(&str, usize); 4] = [
        ("rom/fomt.gba", 0x13ABF0),
        ("rom/fomtjp.gba", 0x13BD34),
        ("rom/mfomt.gba", 0x144FF4),
        ("rom/mfomtjp.gba", 0x146A64),
    ];

    for (path, table_offset) in TABLES {
        let rom = fs::read(local_rom_path(path)).unwrap();
        for slot in 111..=117 {
            let sequence_address = u32::from_le_bytes(
                rom[table_offset + slot * 8..table_offset + slot * 8 + 4]
                    .try_into()
                    .unwrap(),
            );
            let sequence_offset = (sequence_address - 0x0800_0000) as usize;
            assert_eq!(rom[sequence_offset], 1, "{path} slot {slot} track count");
            let track_address = u32::from_le_bytes(
                rom[sequence_offset + 8..sequence_offset + 12]
                    .try_into()
                    .unwrap(),
            );
            let track_offset = (track_address - 0x0800_0000) as usize;
            let program = rom[track_offset..track_offset + 96]
                .windows(2)
                .find_map(|bytes| (bytes[0] == 0xBD).then_some(bytes[1]))
                .unwrap();
            assert_eq!(program, 0x50, "{path} slot {slot} program");
        }

        let next_sequence_address = u32::from_le_bytes(
            rom[table_offset + 118 * 8..table_offset + 118 * 8 + 4]
                .try_into()
                .unwrap(),
        );
        let next_sequence_offset = (next_sequence_address - 0x0800_0000) as usize;
        let next_track_address = u32::from_le_bytes(
            rom[next_sequence_offset + 8..next_sequence_offset + 12]
                .try_into()
                .unwrap(),
        );
        let next_track_offset = (next_track_address - 0x0800_0000) as usize;
        let next_program = rom[next_track_offset..next_track_offset + 96]
            .windows(2)
            .find_map(|bytes| (bytes[0] == 0xBD).then_some(bytes[1]))
            .unwrap();
        assert_eq!(next_program, 0x4B, "{path} slot 118 family boundary");
    }
}

#[test]
fn native_record_player_maps_all_fifteen_albums_to_audio_slots_18_through_32() {
    const TABLES: [(&str, usize); 4] = [
        ("rom/fomt.gba", 0x0E9605),
        ("rom/fomtjp.gba", 0x0E8AA5),
        ("rom/mfomt.gba", 0x0F1B49),
        ("rom/mfomtjp.gba", 0x0F16C5),
    ];
    const EXPECTED: [u8; 15] = [18, 19, 20, 21, 22, 23, 24, 25, 26, 27, 28, 29, 30, 31, 32];

    for (path, table_offset) in TABLES {
        let rom = fs::read(local_rom_path(path)).unwrap();
        assert_eq!(
            &rom[table_offset..table_offset + EXPECTED.len()],
            EXPECTED.as_slice(),
            "{path}: RecordPlayer album-to-audio table"
        );
        assert_eq!(
            rom.windows(EXPECTED.len())
                .filter(|window| *window == EXPECTED)
                .count(),
            1,
            "{path}: RecordPlayer album-to-audio table must be unique"
        );
    }
}

#[test]
fn native_audio_slots_28_through_31_are_identical_short_sequences() {
    const TABLES: [(&str, usize); 4] = [
        ("rom/fomt.gba", 0x13ABF0),
        ("rom/fomtjp.gba", 0x13BD34),
        ("rom/mfomt.gba", 0x144FF4),
        ("rom/mfomtjp.gba", 0x146A64),
    ];
    const TRACK: [u8; 16] = [
        0xBC, 0x00, 0xBB, 0x4B, 0xBD, 0x7F, 0xBE, 0x7F, 0xC0, 0x40, 0xEF, 0x45, 0x50, 0xB0, 0xB1,
        0x00,
    ];

    for (path, table_offset) in TABLES {
        let rom = fs::read(local_rom_path(path)).unwrap();
        for slot in 28..=31 {
            let entry_offset = table_offset + slot * 8;
            let sequence_address =
                u32::from_le_bytes(rom[entry_offset..entry_offset + 4].try_into().unwrap());
            let next_address =
                u32::from_le_bytes(rom[entry_offset + 8..entry_offset + 12].try_into().unwrap());
            assert_eq!(next_address - sequence_address, 0x1C, "{path} slot {slot}");

            let sequence_offset = (sequence_address - 0x0800_0000) as usize;
            assert_eq!(
                &rom[sequence_offset..sequence_offset + 4],
                &[0x01, 0x00, 0x0A, 0x80],
                "{path} slot {slot} header"
            );
            let track_address = u32::from_le_bytes(
                rom[sequence_offset + 8..sequence_offset + 12]
                    .try_into()
                    .unwrap(),
            );
            assert_eq!(track_address, sequence_address - 16, "{path} slot {slot}");
            assert_eq!(
                &rom[sequence_offset - 16..sequence_offset],
                TRACK.as_slice(),
                "{path} slot {slot} track"
            );
        }
    }
}

#[test]
fn native_numbered_script_audio_tracks_match_across_all_four_roms() {
    const TABLES: [(&str, usize); 4] = [
        ("rom/fomt.gba", 0x13ABF0),
        ("rom/fomtjp.gba", 0x13BD34),
        ("rom/mfomt.gba", 0x144FF4),
        ("rom/mfomtjp.gba", 0x146A64),
    ];
    const TRACKS: [(usize, usize, &[u8]); 8] = [
        (
            131,
            1,
            &[
                0xBC, 0x00, 0xBB, 0x4B, 0xBD, 0x4A, 0xBE, 0x7F, 0xC0, 0x40, 0xDE, 0x31, 0x7F, 0x8F,
                0xB1, 0x00,
            ],
        ),
        (
            138,
            1,
            &[
                0xBC, 0x00, 0xBB, 0x4B, 0xBD, 0x4C, 0xBE, 0x7F, 0xC0, 0x40, 0xE9, 0x2D, 0x7F, 0x98,
                0x86, 0xB1,
            ],
        ),
        (
            144,
            1,
            &[
                0xBC, 0x00, 0xBB, 0x4B, 0xBD, 0x4E, 0xBE, 0x7F, 0xC0, 0x40, 0xFE, 0x35, 0x7F, 0xB0,
                0xB1, 0x00,
            ],
        ),
        (
            145,
            1,
            &[
                0xBC, 0x00, 0xBB, 0x4B, 0xBD, 0x4E, 0xBE, 0x7F, 0xC0, 0x40, 0xFE, 0x34, 0x7F, 0x86,
                0x86, 0x86, 0x86, 0x86, 0x86, 0x86, 0x86, 0x86, 0x86, 0x86, 0x86, 0x86, 0x86, 0x86,
                0x83, 0xB1, 0x00, 0x00,
            ],
        ),
        (
            170,
            1,
            &[
                0xBC, 0x00, 0xBB, 0x4B, 0xBD, 0x62, 0xBE, 0x5A, 0xC0, 0x40, 0xF3, 0x3C, 0x7F, 0xA4,
                0x82, 0xB1,
            ],
        ),
        (
            173,
            2,
            &[
                0xBC, 0x00, 0xBB, 0x4B, 0xBD, 0x00, 0xBE, 0x7F, 0xF6, 0x3C, 0x7F, 0xAA, 0xBD, 0x01,
                0xBE, 0x50, 0x86, 0xF1, 0x53, 0x48, 0x8C, 0x9F, 0x81, 0xB1, 0xBC, 0x00, 0xBD, 0x02,
                0x98, 0xBE, 0x7F, 0xA0, 0xC0, 0x4F, 0x8C, 0xF9, 0x6C, 0x50, 0x8C, 0xA7, 0x83, 0xB1,
                0x00, 0x00,
            ],
        ),
        (
            174,
            1,
            &[
                0xBC, 0x00, 0xBB, 0x4B, 0xBD, 0x71, 0xBE, 0x7F, 0xC0, 0x40, 0xE1, 0x3C, 0x7F, 0x86,
                0x86, 0x86, 0xB1, 0x00, 0x00, 0x00,
            ],
        ),
        (
            198,
            1,
            &[
                0xBC, 0x00, 0xBB, 0x4B, 0xBD, 0x62, 0xBE, 0x5A, 0xC0, 0x40, 0xF3, 0x3C, 0x7F, 0xA4,
                0x82, 0xB1,
            ],
        ),
    ];

    let mut expected_voice_metadata = None;
    let mut expected_wave_data: Option<Vec<Vec<u8>>> = None;
    for (path, table_offset) in TABLES {
        let rom = fs::read(local_rom_path(path)).unwrap();
        let sequence_addresses = (0..211)
            .map(|slot| {
                u32::from_le_bytes(
                    rom[table_offset + slot * 8..table_offset + slot * 8 + 4]
                        .try_into()
                        .unwrap(),
                )
            })
            .collect::<Vec<_>>();
        let mut shared_voicegroup = None;
        for (slot, track_count, expected_track_bytes) in TRACKS {
            let entry_offset = table_offset + slot * 8;
            let sequence_address =
                u32::from_le_bytes(rom[entry_offset..entry_offset + 4].try_into().unwrap());
            assert_eq!(
                sequence_addresses
                    .iter()
                    .filter(|&&address| address == sequence_address)
                    .count(),
                1,
                "{path} slot {slot} unexpectedly aliases another physical audio slot"
            );
            let sequence_offset = (sequence_address - 0x0800_0000) as usize;
            assert_eq!(
                rom[sequence_offset] as usize, track_count,
                "{path} slot {slot}"
            );

            let voicegroup = u32::from_le_bytes(
                rom[sequence_offset + 4..sequence_offset + 8]
                    .try_into()
                    .unwrap(),
            );
            assert_eq!(
                *shared_voicegroup.get_or_insert(voicegroup),
                voicegroup,
                "{path} slot {slot} voicegroup"
            );

            let first_track_address = u32::from_le_bytes(
                rom[sequence_offset + 8..sequence_offset + 12]
                    .try_into()
                    .unwrap(),
            );
            let first_track_offset = (first_track_address - 0x0800_0000) as usize;
            assert_eq!(
                &rom[first_track_offset..sequence_offset],
                expected_track_bytes,
                "{path} slot {slot} complete track region"
            );
        }

        let voicegroup_offset = (shared_voicegroup.unwrap() - 0x0800_0000) as usize;
        let voice_metadata = (0..128)
            .flat_map(|program| {
                let entry =
                    &rom[voicegroup_offset + program * 12..voicegroup_offset + (program + 1) * 12];
                entry[..4].iter().chain(&entry[8..]).copied()
            })
            .collect::<Vec<_>>();
        assert_eq!(
            expected_voice_metadata.get_or_insert_with(|| voice_metadata.clone()),
            &voice_metadata,
            "{path} voicegroup metadata differs after pointer normalization"
        );

        let wave_data = [
            (0usize, 8u8),
            (1, 0),
            (74, 0),
            (76, 0),
            (78, 0),
            (98, 0),
            (113, 0),
        ]
        .into_iter()
        .map(|(program, expected_type)| {
            let entry_offset = voicegroup_offset + program * 12;
            assert_eq!(
                rom[entry_offset], expected_type,
                "{path} program {program} instrument type"
            );
            let wave_address =
                u32::from_le_bytes(rom[entry_offset + 4..entry_offset + 8].try_into().unwrap());
            let wave_offset = (wave_address - 0x0800_0000) as usize;
            let sample_length =
                u32::from_le_bytes(rom[wave_offset + 12..wave_offset + 16].try_into().unwrap())
                    as usize;
            rom[wave_offset..wave_offset + 16 + sample_length].to_vec()
        })
        .collect::<Vec<_>>();
        assert_eq!(
            expected_wave_data.get_or_insert_with(|| wave_data.clone()),
            &wave_data,
            "{path} direct PCM wave data differs"
        );
    }
}

#[test]
fn native_audio_198_is_the_b_button_menu_cancel_cue_on_all_targets() {
    for (case, (start, call_sites)) in CASES.iter().zip([
        (0x8B6Cusize, [0x4770usize, 0x47A4]),
        (0x8BD4, [0x4784, 0x47B8]),
        (0x8B74, [0x4788, 0x47BC]),
        (0x8B88, [0x479C, 0x47D0]),
    ]) {
        let rom = fs::read(local_rom_path(case.rom)).unwrap();
        let direct_starts = (0..0xD0000usize - 6)
            .step_by(2)
            .filter(|&offset| {
                rom[offset] == 0xC6
                    && (0x20..=0x27).contains(&rom[offset + 1])
                    && (u16::from_le_bytes(rom[offset + 2..offset + 4].try_into().unwrap())
                        & 0xF800)
                        == 0xF000
                    && (u16::from_le_bytes(rom[offset + 4..offset + 6].try_into().unwrap())
                        & 0xF800)
                        == 0xF800
                    && thumb_call_target(&rom, offset + 2) == start
            })
            .collect::<Vec<_>>();
        assert_eq!(
            direct_starts.len(),
            11,
            "{} direct native menu-cancel starts",
            case.name
        );
        for call_site in call_sites {
            assert_eq!(
                &rom[call_site - 4..call_site],
                &[0x30, 0x1C, 0x18, 0x30],
                "{} menu audio-player address",
                case.name
            );
            assert_eq!(
                &rom[call_site..call_site + 2],
                &[0xC6, 0x21],
                "{} menu cancel sequence ID",
                case.name
            );
            assert_eq!(
                thumb_call_target(&rom, call_site + 2),
                start,
                "{} menu cancel audio start",
                case.name
            );
        }
    }
}

#[test]
fn native_audio_168_is_the_horse_race_neigh_on_all_targets() {
    for (case, (start, call_sites)) in CASES.iter().zip([
        (0x8B6Cusize, [0x5CD3Ausize, 0x5CE0C]),
        (0x8BD4, [0x5FABA, 0x5FB8C]),
        (0x8B74, [0x5CA82, 0x5CB54]),
        (0x8B88, [0x5F736, 0x5F808]),
    ]) {
        let rom = fs::read(local_rom_path(case.rom)).unwrap();
        for call_site in call_sites {
            assert_eq!(
                &rom[call_site..call_site + 2],
                &[0xA8, 0x21],
                "{} horse-race neigh sequence ID",
                case.name
            );
            assert_eq!(
                thumb_call_target(&rom, call_site + 2),
                start,
                "{} horse-race neigh audio start",
                case.name
            );
        }
    }
}

#[test]
fn native_audio_187_is_the_shared_minigame_time_up_cue_on_all_targets() {
    for (case, (start, call_sites)) in CASES.iter().zip([
        (
            0x8B6Cusize,
            [
                0x52E1Eusize,
                0xB090C,
                0xB1D56,
                0xBE934,
                0xBF310,
                0xC3AF0,
                0xC56BA,
                0xC7A8A,
            ],
        ),
        (
            0x8BD4,
            [
                0x553D6, 0xB5B58, 0xB6F86, 0xC486C, 0xC522C, 0xC9A04, 0xCB5B2, 0xCD982,
            ],
        ),
        (
            0x8B74,
            [
                0x52BAE, 0xB0350, 0xB178A, 0xBE370, 0xBED3C, 0xC3524, 0xC50DE, 0xC74AE,
            ],
        ),
        (
            0x8B88,
            [
                0x5509A, 0xB55A4, 0xB69DE, 0xC42CC, 0xC4C98, 0xC9478, 0xCB032, 0xCD402,
            ],
        ),
    ]) {
        let rom = fs::read(local_rom_path(case.rom)).unwrap();
        for call_site in call_sites {
            assert_eq!(
                &rom[call_site..call_site + 2],
                &[0xBB, 0x21],
                "{} minigame time-up sequence ID",
                case.name
            );
            assert_eq!(
                thumb_call_target(&rom, call_site + 2),
                start,
                "{} minigame time-up audio start",
                case.name
            );
        }
    }
}

#[test]
fn native_audio_slots_176_and_183_preserve_real_target_variants() {
    const TABLES: [(&str, usize); 4] = [
        ("rom/fomt.gba", 0x13ABF0),
        ("rom/fomtjp.gba", 0x13BD34),
        ("rom/mfomt.gba", 0x144FF4),
        ("rom/mfomtjp.gba", 0x146A64),
    ];

    fn tracks(rom: &[u8], table_offset: usize, slot: usize) -> Vec<Vec<u8>> {
        let entry_offset = table_offset + slot * 8;
        let song_address =
            u32::from_le_bytes(rom[entry_offset..entry_offset + 4].try_into().unwrap());
        let song_offset = (song_address - 0x0800_0000) as usize;
        let track_count = rom[song_offset] as usize;
        let starts = (0..track_count)
            .map(|track| {
                let pointer_offset = song_offset + 8 + track * 4;
                (u32::from_le_bytes(rom[pointer_offset..pointer_offset + 4].try_into().unwrap())
                    - 0x0800_0000) as usize
            })
            .collect::<Vec<_>>();

        starts
            .iter()
            .enumerate()
            .map(|(track, &start)| {
                let end = starts.get(track + 1).copied().unwrap_or(song_offset);
                rom[start..end].to_vec()
            })
            .collect()
    }

    let roms = TABLES.map(|(path, table_offset)| {
        let rom = fs::read(local_rom_path(path)).unwrap();
        (
            path,
            tracks(&rom, table_offset, 176),
            tracks(&rom, table_offset, 183),
        )
    });

    assert_eq!(roms[0].1, roms[1].1, "FoMT slot 176 differs by region");
    assert_eq!(roms[2].1, roms[3].1, "MFoMT slot 176 differs by region");
    assert_eq!(roms[0].1.len(), 2);
    for track in 0..2 {
        let fomt = &roms[0].1[track];
        let mfomt = &roms[2].1[track];
        assert_eq!(&fomt[..2], &[0xBE, 0x7F]);
        assert_eq!(&mfomt[..2], &[0xBE, 0x64]);
        assert_eq!(
            &fomt[2..],
            &mfomt[2..],
            "slot 176 track {track} has an unaccounted game-family difference"
        );
    }

    assert_eq!(roms[0].2, roms[2].2, "slot 183 differs in MFoMT-US");
    assert_eq!(roms[0].2, roms[3].2, "slot 183 differs in MFoMT-JP");
    let common = &roms[0].2;
    let fomt_jp = &roms[1].2;
    assert_eq!(common.len(), 3);
    assert_eq!(fomt_jp.len(), 3);

    for (track, marker) in [(0usize, [0xF1, 0x2D, 0x7F]), (1, [0xF1, 0x2D, 0x82])] {
        let mut normalized = fomt_jp[track].clone();
        let note = normalized
            .windows(marker.len())
            .position(|window| window == marker)
            .unwrap();
        normalized[note] = 0xF3;
        assert_eq!(normalized.last(), Some(&0xB1));
        normalized.insert(normalized.len() - 1, 0x83);
        assert_eq!(
            &normalized, &common[track],
            "FoMT-JP slot 183 track {track} has an unaccounted difference"
        );
    }

    fn through_fine(track: &[u8]) -> &[u8] {
        let end = track.iter().rposition(|&byte| byte == 0xB1).unwrap();
        &track[..=end]
    }
    assert_eq!(through_fine(&fomt_jp[2]), through_fine(&common[2]));
    assert_eq!(&common[2][through_fine(&common[2]).len()..], &[0, 0, 0]);
    assert_eq!(&fomt_jp[2][through_fine(&fomt_jp[2]).len()..], &[0]);
}

#[test]
fn native_harvest_sprite_status_accessors_match_all_four_targets() {
    const CURRENT_TASK: &[u8] = &[0x80, 0x7E, 0x80, 0x07, 0x80, 0x0F, 0x70, 0x47];
    const WORK_DAYS: &[u8] = &[0x80, 0x7E, 0xC0, 0x06, 0x40, 0x0F, 0x70, 0x47];
    const TASK_EXPERIENCE: &[u8] = &[
        0x00, 0xB5, 0x02, 0x29, 0x01, 0xD9, 0x00, 0x20, 0x02, 0xE0, 0x14, 0x30, 0x40, 0x18, 0x00,
        0x78, 0x02, 0xBC, 0x08, 0x47,
    ];
    const PLAYED_MINIGAME: &[u8] = &[0x80, 0x7E, 0x80, 0x06, 0xC0, 0x0F, 0x70, 0x47];
    const MINIGAME_EXPERIENCE: &[u8] = &[
        0x00, 0xB5, 0x02, 0x29, 0x01, 0xD9, 0x00, 0x20, 0x02, 0xE0, 0x17, 0x30, 0x40, 0x18, 0x00,
        0x78, 0x02, 0xBC, 0x08, 0x47,
    ];

    for (case, dispatch, base_id, handlers, resolver, leaves) in [
        (
            &CASES[0],
            0x3F904,
            0x0CB,
            [0x42610, 0x42654, 0x42698, 0x426F0, 0x42738],
            0xA0930,
            [0x9E65C, 0x9E664, 0x9E66C, 0x9E680, 0x9E688],
        ),
        (
            &CASES[2],
            0x3F578,
            0x0CB,
            [0x42284, 0x422C8, 0x4230C, 0x42364, 0x423AC],
            0xA0368,
            [0x9E094, 0x9E09C, 0x9E0A4, 0x9E0B8, 0x9E0C0],
        ),
        (
            &CASES[1],
            0x3FAF0,
            0x0CE,
            [0x42856, 0x42898, 0x428DC, 0x42934, 0x4297C],
            0xA5B04,
            [0xA37F0, 0xA37F8, 0xA3800, 0xA3814, 0xA381C],
        ),
        (
            &CASES[3],
            0x3F84C,
            0x0CE,
            [0x425B2, 0x425F4, 0x42638, 0x42690, 0x426D8],
            0xA5544,
            [0xA3230, 0xA3238, 0xA3240, 0xA3254, 0xA325C],
        ),
    ] {
        let rom = fs::read(local_rom_path(case.rom)).unwrap();
        for (index, &handler) in handlers.iter().enumerate() {
            let entry = dispatch + (base_id + index) * 4;
            assert_eq!(
                u32::from_le_bytes(rom[entry..entry + 4].try_into().unwrap()),
                0x0800_0000 + handler as u32,
                "{} Harvest Sprite accessor slot {index}",
                case.name
            );
            let resolver_call = handler + if matches!(index, 2 | 4) { 0x3A } else { 0x28 };
            let leaf_call = handler
                + match index {
                    2 => 0x4A,
                    4 => 0x46,
                    _ => 0x36,
                };
            assert_eq!(
                thumb_call_target(&rom, resolver_call),
                resolver,
                "{}",
                case.name
            );
            assert_eq!(
                thumb_call_target(&rom, leaf_call),
                leaves[index],
                "{}",
                case.name
            );
        }

        for (leaf, expected) in leaves.into_iter().zip([
            CURRENT_TASK,
            WORK_DAYS,
            TASK_EXPERIENCE,
            PLAYED_MINIGAME,
            MINIGAME_EXPERIENCE,
        ]) {
            assert_eq!(
                &rom[leaf..leaf + expected.len()],
                expected,
                "{} Harvest Sprite leaf {leaf:#X}",
                case.name
            );
        }

        assert_eq!(&rom[handlers[0] + 0x2C..handlers[0] + 0x2E], &[0x03, 0x22]);
        assert_eq!(&rom[handlers[1] + 0x2C..handlers[1] + 0x2E], &[0x00, 0x22]);
        assert_eq!(&rom[handlers[2] + 0x3E..handlers[2] + 0x40], &[0x00, 0x22]);
        assert_eq!(&rom[handlers[3] + 0x2C..handlers[3] + 0x2E], &[0x00, 0x22]);
        assert_eq!(&rom[handlers[4] + 0x3E..handlers[4] + 0x40], &[0x00, 0x21]);
        assert_eq!(
            &rom[handlers[4] + 0x4A..handlers[4] + 0x50],
            &[0x41, 0x42, 0x01, 0x43, 0xC9, 0x0F]
        );
    }
}

#[test]
fn native_mfomt_outfit_color_accessors_preserve_the_six_value_contract() {
    for (case, dispatch, getter, setter, getter_leaf, setter_leaf) in [
        (&CASES[1], 0x3FAF0, 0x45478, 0x4549C, 0x0EA30, 0x0EFE4),
        (&CASES[3], 0x3F84C, 0x451D4, 0x451F8, 0x0E9E4, 0x0EF98),
    ] {
        let rom = fs::read(local_rom_path(case.rom)).unwrap();
        assert_eq!(
            u32::from_le_bytes(
                rom[dispatch + 0x78 * 4..dispatch + 0x78 * 4 + 4]
                    .try_into()
                    .unwrap()
            ),
            0x08000000 + getter as u32,
            "{} outfit getter slot",
            case.name
        );
        assert_eq!(
            u32::from_le_bytes(
                rom[dispatch + 0x79 * 4..dispatch + 0x79 * 4 + 4]
                    .try_into()
                    .unwrap()
            ),
            0x08000000 + setter as u32,
            "{} outfit setter slot",
            case.name
        );
        assert_eq!(
            u32::from_le_bytes(rom[getter + 0x20..getter + 0x24].try_into().unwrap()),
            0x1BE8,
            "{} getter state offset",
            case.name
        );
        assert_eq!(
            u32::from_le_bytes(rom[setter + 0x28..setter + 0x2C].try_into().unwrap()),
            0x1BE8,
            "{} setter state offset",
            case.name
        );
        assert_eq!(thumb_call_target(&rom, getter + 0x0C), getter_leaf);
        assert_eq!(thumb_call_target(&rom, setter + 0x22), setter_leaf);
        assert_eq!(
            &rom[getter_leaf..getter_leaf + 10],
            &[0x52, 0x30, 0x00, 0x78, 0x40, 0x07, 0x40, 0x0F, 0x70, 0x47]
        );
        assert_eq!(
            &rom[setter_leaf..setter_leaf + 42],
            &[
                0x00, 0xB5, 0x05, 0x29, 0x0F, 0xD8, 0x03, 0x1C, 0x52, 0x33, 0x1A, 0x78, 0x50, 0x07,
                0x40, 0x0F, 0x88, 0x42, 0x08, 0xD0, 0x07, 0x20, 0x01, 0x40, 0x08, 0x20, 0x40, 0x42,
                0x10, 0x40, 0x08, 0x43, 0x08, 0x21, 0x08, 0x43, 0x18, 0x70, 0x01, 0xBC, 0x00, 0x47,
            ],
            "{} outfit setter bounds/preserve/dirty-bit contract",
            case.name
        );
    }
}

#[test]
fn native_incubator_capacity_is_one_plus_the_coop_upgrade_bit_on_all_targets() {
    const CAPACITY_LEAF: &[u8] = &[
        0x00, 0xB5, 0x01, 0x78, 0x01, 0x20, 0x08, 0x40, 0x02, 0x21, 0x00, 0x28, 0x00, 0xD1, 0x01,
        0x21, 0x08, 0x1C, 0x02, 0xBC, 0x08, 0x47,
    ];
    for (case, dispatch, slot, handler, leaf, coop_offset_bytes) in [
        (
            &CASES[0],
            0x3F904,
            0xBB,
            0x44B4E,
            0x0C660,
            &[0x82, 0x26, 0xF6, 0x00][..],
        ),
        (
            &CASES[1],
            0x3FAF0,
            0xBE,
            0x44DAE,
            0x0C6D4,
            &[0x84, 0x26, 0xF6, 0x00][..],
        ),
        (
            &CASES[2],
            0x3F578,
            0xBB,
            0x447C2,
            0x0C640,
            &[0x82, 0x26, 0xF6, 0x00][..],
        ),
        (
            &CASES[3],
            0x3F84C,
            0xBE,
            0x44B0A,
            0x0C688,
            &[0x84, 0x26, 0xF6, 0x00][..],
        ),
    ] {
        let rom = fs::read(local_rom_path(case.rom)).unwrap();
        assert_eq!(
            u32::from_le_bytes(
                rom[dispatch + slot * 4..dispatch + slot * 4 + 4]
                    .try_into()
                    .unwrap()
            ),
            0x08000000 + handler as u32,
            "{} incubator-capacity slot",
            case.name
        );
        // FoMT adds 0x82 << 2 (0x208); MFoMT adds 0x84 << 3
        // (0x420). Both addresses are their family's Coop object.
        assert_eq!(&rom[handler + 8..handler + 12], coop_offset_bytes);
        assert_eq!(thumb_call_target(&rom, handler + 0x0E), leaf);
        assert_eq!(
            &rom[leaf..leaf + CAPACITY_LEAF.len()],
            CAPACITY_LEAF,
            "{} capacity reads bit 0 and returns 1 or 2",
            case.name
        );
    }
}

#[test]
fn native_pregnancy_stall_capacity_is_one_plus_the_barn_upgrade_bit_on_all_targets() {
    const CAPACITY_LEAF: &[u8] = &[0x00, 0x78, 0xC0, 0x07, 0xC0, 0x0F, 0x01, 0x30, 0x70, 0x47];
    for (case, dispatch, slot, handler, leaf, barn_offset_bytes) in [
        (
            &CASES[0],
            0x3F904,
            0xC1,
            0x44BFE,
            0x0CE9C,
            &[0xBE, 0x21, 0xC9, 0x00][..],
        ),
        (
            &CASES[1],
            0x3FAF0,
            0xC4,
            0x44E56,
            0x0CF10,
            &[0xC0, 0x21, 0xC9, 0x00][..],
        ),
        (
            &CASES[2],
            0x3F578,
            0xC1,
            0x44872,
            0x0CE7C,
            &[0xBE, 0x21, 0xC9, 0x00][..],
        ),
        (
            &CASES[3],
            0x3F84C,
            0xC4,
            0x44BB2,
            0x0CEC4,
            &[0xC0, 0x21, 0xC9, 0x00][..],
        ),
    ] {
        let rom = fs::read(local_rom_path(case.rom)).unwrap();
        assert_eq!(
            u32::from_le_bytes(
                rom[dispatch + slot * 4..dispatch + slot * 4 + 4]
                    .try_into()
                    .unwrap()
            ),
            0x08000000 + handler as u32,
            "{} pregnancy-stall-capacity slot",
            case.name
        );
        // FoMT adds 0xBE << 1 (0x17C); MFoMT adds 0xC0 << 1
        // (0x180). Both addresses are their family's Barn object.
        assert_eq!(&rom[handler + 8..handler + 12], barn_offset_bytes);
        assert_eq!(thumb_call_target(&rom, handler + 0x0E), leaf);
        assert_eq!(
            &rom[leaf..leaf + CAPACITY_LEAF.len()],
            CAPACITY_LEAF,
            "{} capacity extracts bit 0 and adds one",
            case.name
        );
    }
}

#[test]
fn native_shipment_box_deposit_animation_only_dispatches_the_scene_action() {
    let reference = fs::read(local_rom_path(CASES[0].rom)).unwrap();
    for (case, dispatch, slot, handler, leaf) in [
        (&CASES[0], 0x3F904, 0x0C0, 0x423D8, 0x140F4),
        (&CASES[1], 0x3FAF0, 0x0C3, 0x42624, 0x14280),
        (&CASES[2], 0x3F578, 0x0C0, 0x4204C, 0x13FC8),
        (&CASES[3], 0x3F84C, 0x0C3, 0x42380, 0x14128),
    ] {
        let rom = fs::read(local_rom_path(case.rom)).unwrap();
        assert_eq!(
            u32::from_le_bytes(
                rom[dispatch + slot * 4..dispatch + slot * 4 + 4]
                    .try_into()
                    .unwrap()
            ),
            0x0800_0000 + handler as u32,
            "{} shipment-animation physical slot",
            case.name
        );
        assert_eq!(thumb_call_target(&rom, handler + 0x10), leaf);

        // Complete leaf through return. It reads event context +4, obtains the
        // scene object at +A8 and invokes virtual +80. The sole difference is
        // the relocated indirect-call helper BL at +0x0E.
        for offset in 0..22 {
            if (0x0E..0x12).contains(&offset) {
                continue;
            }
            assert_eq!(
                rom[leaf + offset],
                reference[0x140F4 + offset],
                "{} shipment-animation leaf +{offset:#x}",
                case.name
            );
        }
    }
}

#[test]
fn native_audio_null_slot_topology_matches_neutral_symbols_on_all_targets() {
    let constants_source = fs::read_to_string("goodies/mary_constants.mary.h").unwrap();

    for case in CASES {
        let table_offset = match case.target {
            "MARY_FOMT_US" => 0x13ABF0,
            "MARY_FOMT_JP" => 0x13BD34,
            "MARY_MFOMT_US" => 0x144FF4,
            "MARY_MFOMT_JP" => 0x146A64,
            _ => unreachable!(),
        };
        let expected_nulls = if case.target.contains("MFOMT") {
            (std::iter::once(0).chain(43..=100).chain(194..=196)).collect::<Vec<_>>()
        } else {
            (std::iter::once(0).chain(38..=100).chain(194..=196)).collect::<Vec<_>>()
        };
        let rom = fs::read(local_rom_path(case.rom)).unwrap();
        let null_sequence =
            u32::from_le_bytes(rom[table_offset..table_offset + 4].try_into().unwrap());
        let null_sequence_offset = (null_sequence - 0x0800_0000) as usize;
        assert_eq!(
            rom[null_sequence_offset], 0,
            "{} null SongEnt track count",
            case.name
        );
        let actual_nulls = (0..211)
            .filter(|slot| {
                let offset = table_offset + slot * 8;
                u32::from_le_bytes(rom[offset..offset + 4].try_into().unwrap()) == null_sequence
            })
            .collect::<Vec<_>>();
        assert_eq!(
            actual_nulls, expected_nulls,
            "{} null audio slots",
            case.name
        );

        let options = Options::default().define(case.target).unwrap();
        let constants = parse_constant_header(&constants_source, &options).unwrap();
        let audio_type = constants.user_type("MaryAudioSequenceId").unwrap();
        for slot in actual_nulls {
            let name = constants
                .typed_int_const_name(audio_type, slot as i64)
                .unwrap();
            assert!(
                name == format!("AUDIO_UNUSED_SLOT_{slot:03}"),
                "{} null audio slot {slot} lacks its physical unused-slot name: {name}",
                case.name
            );
        }
    }
}

#[test]
fn native_animation_tables_are_complete_and_region_identical_within_each_game() {
    fn animation_script(rom: &[u8], table: usize, pool: usize, id: usize) -> Vec<(u16, u16)> {
        let record =
            u32::from_le_bytes(rom[table + id * 4..table + id * 4 + 4].try_into().unwrap());
        let count = (record & 0xFFFF) as usize;
        let first = (record >> 16) as usize;
        assert!(count > 0, "animation {id} has a zero-frame record");
        (0..count)
            .map(|index| {
                let offset = pool + (first + index) * 4;
                (
                    u16::from_le_bytes(rom[offset..offset + 2].try_into().unwrap()),
                    u16::from_le_bytes(rom[offset + 2..offset + 4].try_into().unwrap()),
                )
            })
            .collect()
    }

    for (game, us_path, us_table, us_pool, jp_path, jp_table, jp_pool, last_id) in [
        (
            "FoMT",
            "rom/fomt.gba",
            0x58BA2C,
            0x663208,
            "rom/fomtjp.gba",
            0x311B88,
            0x3E9364,
            2551,
        ),
        (
            "MFoMT",
            "rom/mfomt.gba",
            0x522C4C,
            0x6036B8,
            "rom/mfomtjp.gba",
            0x52455C,
            0x604FC8,
            2635,
        ),
    ] {
        let us = fs::read(local_rom_path(us_path)).unwrap();
        let jp = fs::read(local_rom_path(jp_path)).unwrap();
        for id in 0..=last_id {
            assert_eq!(
                animation_script(&us, us_table, us_pool, id),
                animation_script(&jp, jp_table, jp_pool, id),
                "{game} animation {id} differs between US and JP"
            );
        }
    }
}

fn direct_numeric_argument(call: &str, argument_index: usize) -> Option<i64> {
    let mut depth = 0usize;
    let mut current_index = 0usize;
    let mut argument_start = 0usize;
    for (offset, character) in call.char_indices() {
        match character {
            '(' => depth += 1,
            ')' if depth == 0 => {
                let argument = call[argument_start..offset].trim();
                return (current_index == argument_index)
                    .then(|| argument.parse().ok())
                    .flatten();
            }
            ')' => depth -= 1,
            ',' if depth == 0 => {
                if current_index == argument_index {
                    return call[argument_start..offset].trim().parse().ok();
                }
                current_index += 1;
                argument_start = offset + character.len_utf8();
            }
            _ => {}
        }
    }
    None
}

fn assert_statically_typed_arguments_are_symbolic(
    case: &RomCase,
    script_id: usize,
    source: &str,
    callables: &mary::mary_c::CallableTable,
) {
    // These Mary types intentionally describe open quantities or context-local
    // indices.  A bare integer is therefore meaningful even when no global
    // symbolic constant exists for that exact value.  Every other user type is
    // a closed semantic domain: if a retail script still renders a bare integer
    // for it, either the constant table is incomplete or propagation regressed.
    const NUMERIC_USER_TYPES: &[&str] = &[
        "MaryAnimalAffectionDelta",
        "MaryCharacterLoveDelta",
        "MaryClockHour",
        "MaryClockMinute",
        "MaryFatigueDelta",
        "MaryFrameCount",
        "MaryMineFloorIndex",
        "MaryMoneyAmount",
        "MaryNpcFriendshipDelta",
        "MaryNpcFriendshipValue",
        "MaryRequestedInventoryCount",
        "MaryRequestedToolStackCount",
        "MaryRgb5Channel",
        "MaryStaminaDelta",
        "MaryTextNumberFieldWidth",
    ];

    for (callable, (_, shape)) in callables.scope.callable_map() {
        let argument_indices = shape
            .parameter_types()
            .iter()
            .enumerate()
            .filter_map(|(index, value_type)| {
                matches!(value_type, mary::ir::ValueType::UserType(_)).then_some(index)
            })
            .collect::<Vec<_>>();
        if argument_indices.is_empty() {
            continue;
        }
        let marker = format!("{callable}(");
        for call in source.split(&marker).skip(1) {
            for &argument_index in &argument_indices {
                let Some(value) = direct_numeric_argument(call, argument_index) else {
                    continue;
                };
                let mary::ir::ValueType::UserType(type_id) =
                    shape.parameter_types()[argument_index]
                else {
                    unreachable!();
                };
                let has_semantic_constant = callables
                    .scope
                    .typed_int_const_name(type_id, value)
                    .is_some();
                assert!(
                    !has_semantic_constant,
                    "{} script {} renders statically typed argument {} of {} as {value}, although that value has a semantic constant:\n{}",
                    case.name,
                    script_id,
                    argument_index,
                    callable,
                    source
                );
                assert!(
                    NUMERIC_USER_TYPES.iter().any(|type_name| {
                        callables.scope.user_type(type_name) == Some(type_id)
                    }),
                    "{} script {} renders bare integer {value} for closed-domain argument {} of {}; add the missing semantic constant or repair type propagation:\n{}",
                    case.name,
                    script_id,
                    argument_index,
                    callable,
                    source
                );
            }
        }
    }
}

#[test]
fn every_target_exposes_statically_typed_callable_arguments() {
    for case in CASES {
        let options = Options::default().define(case.target).unwrap();
        let constants = parse_constant_header(
            &fs::read_to_string("goodies/mary_constants.mary.h").unwrap(),
            &options,
        )
        .unwrap();
        let callables = parse_callable_table_with_scope(
            &fs::read_to_string("goodies/mary_callables.mary.h").unwrap(),
            &options,
            &constants,
        )
        .unwrap();
        assert!(
            callables
                .scope
                .callable_map()
                .values()
                .any(|(_, shape)| shape
                    .parameter_types()
                    .iter()
                    .any(|value_type| matches!(value_type, mary::ir::ValueType::UserType(_)))),
            "{} exposes no statically typed callable arguments",
            case.name
        );
    }
}

fn verify(case: &RomCase) -> Result<(), Box<dyn std::error::Error>> {
    let rom = fs::read(local_rom_path(case.rom))?;
    let entries = get_script_table(&rom)?;
    assert_eq!(entries.len(), case.slots, "{} slot count", case.name);

    let options = Options::default().define(case.target)?;
    let constants = parse_constant_header(
        &fs::read_to_string("goodies/mary_constants.mary.h")?,
        &options,
    )?;
    let callables = parse_callable_table_with_scope(
        &fs::read_to_string("goodies/mary_callables.mary.h")?,
        &options,
        &constants,
    )?;
    let symbols = parse_text_name_table(
        &fs::read_to_string("goodies/mary_scripts_text.mary.sym")?,
        &options,
    )?;
    let script_table = symbols.script_table()?;
    let mut decompile_scope = callables.scope.clone();
    script_table.add_constants(&mut decompile_scope);
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
    let mut symbolic_script_calls = 0;
    let mut numbered_audio_sequences = HashSet::new();
    let mut numbered_audio_usage: HashMap<String, HashMap<String, usize>> = HashMap::new();
    let preamble = format!("#define {}\n#include \"mary_constants.mary.h\"\n#include \"mary_callables.mary.h\"\n#include \"mary_scripts.mary.h\"\n\n", case.target);
    let mut outputs = Vec::new();
    let mut table = String::from("mary_script_table\n{\n");
    for entry in &entries {
        match script_table.name(entry.id()) {
            Some(name) => table.push_str(&format!("    {name},\n")),
            None => table.push_str("    NULL,\n"),
        }
    }
    table.push_str("};\n");
    outputs.push((String::from("mary_scripts.mary.h"), table));
    for header in ["mary_constants.mary.h", "mary_callables.mary.h"] {
        outputs.push((
            header.to_owned(),
            fs::read_to_string(Path::new("goodies").join(header))?,
        ));
    }
    for entry in entries {
        let slot_id = entry.id();
        let ScriptTableEntry::Script { id, data, backing } = entry else {
            outputs.push((format!("EventScript_{slot_id:04}.mary.c"), format!("{preamble}// NULL script pointer-table slot {slot_id}; no RIFF body exists.\n// This placeholder intentionally contains no script definition.\n")));
            continue;
        };
        let decoded = decode_script_with_backing(data, backing)?;
        let name = symbols
            .script_name(id)
            .ok_or_else(|| format!("{} script {id} has no semantic name", case.name))?
            .to_owned();
        assert_numbered_script_name_matches_slot(&name, id);
        let numbered_name = format!("EventScript_{id:04}");
        if name == numbered_name {
            assert!(
                decoded.instructions.is_empty() || decoded.instructions == [Ins::Exit],
                "{} numbered placeholder slot {id} contains behavior beyond a terminal exit and needs semantic review",
                case.name
            );
            assert!(
                decoded.strings.is_empty(),
                "{} numbered placeholder slot {id} contains text and needs semantic review",
                case.name
            );
        }
        assert!(symbols
            .names(id, decoded.strings.len())
            .iter()
            .all(Option::is_some));
        let text_names = symbols.names(id, decoded.strings.len());
        let local_types = symbols.local_types(id, &decompile_scope)?;
        let ast = decompile_script_with_metadata(
            &decoded,
            &decompile_scope,
            &name,
            &text_names,
            &local_types,
        )
        .map_err(|error| format!("{} script {id} decompile: {error}", case.name))?;
        let source = format_named_script_with_charmap(&name, &ast, &charmap)
            .map_err(|error| format!("{} script {id} format: {error}", case.name))?;
        assert!(
            !source.contains("ENTITY_EMOTE_08"),
            "{} script {id} uses unverified entity-emote ID 8",
            case.name
        );
        if name == "EventScript_SystemEvent_OneHundredQuestionQuiz" {
            assert!(
                source.contains("switch (RandomIntInclusive(1, 255))"),
                "{} quiz must select from the physical 255-question pool",
                case.name
            );
            assert!(
                source.contains("while (var_1 < 100);"),
                "{} quiz must require 100 successful rounds",
                case.name
            );
        }
        if name == "EventScript_NPCEvent_DougAndDuke_ArgumentChoice" {
            if case.target.contains("FOMT") && !case.target.contains("MFOMT") {
                // This is the vanilla script's physical behavior: the event
                // awards Ann love but asks the heart UI to read Popuri's love.
                // Keep the mismatch visible instead of silently rewriting it.
                assert!(source.contains("ShowTalkHeartIndicator(CHARACTER_POPURI)"));
                assert!(source.contains("AddCharacterLove(CHARACTER_ANN, 3000)"));
            } else {
                assert!(!source.contains("ShowTalkHeartIndicator("));
                assert!(source.contains("AddNpcFriendship(CHARACTER_ANN, 20)"));
            }
        }
        if name == "EventScript_NPCEvent_DougAndDuke_Argument_FollowupAnnDialogue" {
            if case.target.contains("FOMT") && !case.target.contains("MFOMT") {
                assert!(source.contains("ShowTalkHeartIndicator(CHARACTER_POPURI)"));
            } else {
                assert!(!source.contains("ShowTalkHeartIndicator("));
            }
        }
        if name == "EventScript_NPCEvent_Cliff_DialogueAndGiftResponses"
            && case.target.contains("MFOMT")
        {
            // The vanilla MFoMT bytecode tests >= green first and only enters
            // the following >= yellow/orange/red chain when that test is false.
            // Those later branches are therefore unreachable.  Preserve this
            // original behavior byte-for-byte instead of silently repairing it.
            let green_needle = "if (GetCharacterLove(CHARACTER_CLIFF) >= LOVE_HEART_GREEN_MIN)";
            let chains = source.match_indices(green_needle).collect::<Vec<_>>();
            assert_eq!(
                chains.len(),
                2,
                "{}: expected morning and afternoon copies of Cliff's original threshold bug",
                case.name
            );
            for (offset, _) in chains {
                let tail = &source[offset..source.len().min(offset + 12_000)];
                let yellow = tail
                    .find("if (GetCharacterLove(CHARACTER_CLIFF) >= LOVE_HEART_YELLOW_MIN)")
                    .expect("MFoMT Cliff unreachable yellow threshold");
                let orange = tail
                    .find("if (GetCharacterLove(CHARACTER_CLIFF) >= LOVE_HEART_ORANGE_MIN)")
                    .expect("MFoMT Cliff unreachable orange threshold");
                let red = tail
                    .find("if (GetCharacterLove(CHARACTER_CLIFF) >= LOVE_HEART_RED_MIN)")
                    .expect("MFoMT Cliff unreachable red threshold");
                assert!(
                    yellow < orange && orange < red,
                    "{}: Cliff's original unreachable threshold chain changed",
                    case.name
                );
            }
        }
        if name == "EventScript_LocationTransition_EnterInn1FWithEventDispatch" {
            assert!(source.contains(
                "VarGet(VAR_ANN_CLIFF_WEDDING_EVENT_STATE) == EVENT_LIFECYCLE_COMPLETED"
            ));
            assert!(source
                .contains("VarGet(VAR_ANN_CLIFF_RIVAL_MARRIAGE_STATE) == MARRIAGE_STATE_MARRIED"));
            assert!(source.contains(
                "CallScript(EventScript_RivalMarriageEvent_AnnAndCliff_06_SiblingComparisonArgument)"
            ));
        }
        if name == "EventScript_NPCEvent_Gotz_LosesMotivation" {
            assert!(source.contains(
                "VarSet(VAR_GOTZ_LOSES_MOTIVATION_EVENT_STATE, EVENT_LIFECYCLE_COMPLETED)"
            ));
            assert!(source.contains("VarSet(VAR_GOTZ_WORK_SUSPENDED, TRUE)"));
        }
        if name == "EventScript_NPCEvent_Gotz_RegainsMotivation" {
            assert!(source.contains(
                "VarSet(VAR_GOTZ_REGAINS_MOTIVATION_EVENT_STATE, EVENT_LIFECYCLE_COMPLETED)"
            ));
            assert!(source.contains("VarSet(VAR_GOTZ_WORK_SUSPENDED, FALSE)"));
        }
        if name == "EventScript_LocationTransition_EnterGotzHouseWithEventDispatch" {
            assert!(source.contains("GetNpcFriendship(CHARACTER_GOTZ) < 100"));
            assert!(source.contains("GetNpcFriendship(CHARACTER_GOTZ) >= 150"));
            assert!(source.contains("var_1 == SEASON_SUMMER"));
            assert!(source.contains("var_1 != SEASON_SUMMER"));
            assert!(source.contains("CallScript(EventScript_NPCEvent_Gotz_LosesMotivation)"));
            assert!(source.contains("CallScript(EventScript_NPCEvent_Gotz_RegainsMotivation)"));
            assert!(
                source.contains("CallScript(EventScript_NPCEvent_GotzAndHarris_PatrolDiscussion)")
            );
        }
        if name == "EventScript_NPCEvent_GotzAndHarris_PatrolDiscussion" {
            assert!(source.contains(
                "VarSet(VAR_GOTZ_AND_HARRIS_PATROL_DISCUSSION_EVENT_STATE, EVENT_LIFECYCLE_IN_PROGRESS)"
            ));
            assert!(source.contains(
                "SetEntityEventScript(ENTITY_GOTZ, EventScript_NPCEvent_GotzAndHarris_PatrolDiscussion_FollowupGotzDialogue)"
            ));
            assert!(source.contains(
                "SetEntityEventScript(ENTITY_HARRIS, EventScript_NPCEvent_GotzAndHarris_PatrolDiscussion_FollowupHarrisDialogue)"
            ));
        }
        if name == "EventScript_LocationTransition_ExitGotzHouseWithEventStateCleanupAndDispatch" {
            assert!(source.contains(
                "VarSet(VAR_GOTZ_AND_HARRIS_PATROL_DISCUSSION_EVENT_STATE, EVENT_LIFECYCLE_COMPLETED)"
            ));
            assert!(source.contains("ClearEntityEventScript(ENTITY_GOTZ)"));
            assert!(source.contains("ClearEntityEventScript(ENTITY_HARRIS)"));
        }
        if name == "EventScript_NPCEvent_Zack_GivesFishingRod" {
            assert!(source.contains("SetPlayerHeldTool(TOOL_FISHING_ROD_IRON, 1)"));
            assert!(source.contains("AddToolToRucksack(TOOL_FISHING_ROD_IRON, 1)"));
            assert!(source.contains(
                "VarSet(VAR_ZACK_GIVES_FISHING_ROD_EVENT_STATE, EVENT_LIFECYCLE_IN_PROGRESS)"
            ));
            assert!(source.contains(
                "SetEntityEventScript(ENTITY_ZACK, EventScript_NPCEvent_Zack_GivesFishingRod_FollowupZackDialogue)"
            ));
        }
        if name == "EventScript_AchievementEvent_Fishing_CaughtEveryFishSpeciesZackCongratulations"
        {
            assert!(source.contains("AddNpcFriendship(CHARACTER_ZACK, 10)"));
            assert!(source.contains(
                "VarSet(VAR_ZACK_CONGRATULATES_CATCHING_EVERY_FISH_EVENT_STATE, EVENT_LIFECYCLE_COMPLETED)"
            ));
        }
        if name == "EventScript_LocationTransition_EnterZackHouseWithEventDispatch" {
            assert!(source.contains("GetFirstFreeRucksackToolSlot() >= RUCKSACK_SLOT_1"));
            assert!(source.contains("CallScript(EventScript_NPCEvent_Zack_GivesFishingRod)"));
        }
        if name == "EventScript_LocationTransition_EnterFarmWithDailyEventDispatch" {
            assert!(source.contains(
                "VarGet(VAR_ACHIEVEMENT_CAUGHT_EVERY_FISH_SPECIES_STATE) == ACHIEVEMENT_EVENT_COMPLETED"
            ));
            assert!(source.contains(
                "CallScript(EventScript_AchievementEvent_Fishing_CaughtEveryFishSpeciesZackCongratulations)"
            ));
        }
        if name == "EventScript_LocationTransition_ExitSeasideLodge" {
            assert!(source.contains(
                "VarSet(VAR_ZACK_GIVES_FISHING_ROD_EVENT_STATE, EVENT_LIFECYCLE_COMPLETED)"
            ));
            assert!(source.contains("ClearEntityEventScript(ENTITY_ZACK)"));
            assert!(source.contains("ClearEntityEventScript(ENTITY_WON)"));
        }
        if name == "EventScript_NPCEvent_Zack_VisitsSickLillia" {
            assert!(source.contains(
                "VarSet(VAR_ZACK_VISITS_SICK_LILLIA_EVENT_STATE, EVENT_LIFECYCLE_IN_PROGRESS)"
            ));
            assert!(source.contains(
                "SetEntityEventScript(ENTITY_LILLIA, EventScript_NPCEvent_Zack_VisitsSickLillia_FollowupLilliaDialogue)"
            ));
        }
        if name == "EventScript_LocationTransition_EnterPoultryFarmHouse1FWithEventDispatch" {
            assert!(source.contains("var_6 >= 4"));
            assert!(source.contains("var_1 == SEASON_SUMMER"));
            assert!(source.contains("var_3 == DAY_OF_WEEK_SUNDAY"));
            assert!(source.contains("CallScript(EventScript_NPCEvent_Zack_VisitsSickLillia)"));
        }
        if name == "EventScript_LocationTransition_ExitPoultryFarmHouse" {
            assert!(source.contains(
                "VarSet(VAR_ZACK_VISITS_SICK_LILLIA_EVENT_STATE, EVENT_LIFECYCLE_COMPLETED)"
            ));
            assert!(source.contains("ClearEntityEventScript(ENTITY_LILLIA)"));
        }
        if name == "EventScript_NPCEvent_Won_Introduction" {
            assert!(source
                .contains("VarSet(VAR_WON_INTRODUCTION_EVENT_STATE, EVENT_LIFECYCLE_COMPLETED)"));
        }
        if name == "EventScript_NPCEvent_Won_AppleChallenge" {
            for food in ["FOOD_SUGDW_APPLE", "FOOD_HMSGB_APPLE", "FOOD_AEPFE_APPLE"] {
                assert!(source.contains(&format!("SetPlayerHeldFood({food})")));
            }
            assert!(source.contains("SubtractMoney(500)"));
            assert!(source.contains(
                "VarSet(VAR_WON_APPLE_CHALLENGE_EVENT_STATE, EVENT_LIFECYCLE_COMPLETED)"
            ));
        }
        if name == "EventScript_NPCEvent_Won_VasePurchase" {
            assert!(source.contains("VarSet(VAR_HAS_VASE, TRUE)"));
            assert!(source.contains("SubtractMoney(5000)"));
            assert!(source
                .contains("VarSet(VAR_WON_VASE_PURCHASE_EVENT_STATE, EVENT_LIFECYCLE_COMPLETED)"));
        }
        if name == "EventScript_NPCEvent_Van_Introduction" {
            assert!(source.contains("SetTalkNameplateCharacter(CHARACTER_VAN)"));
            assert!(source
                .contains("VarSet(VAR_VAN_INTRODUCTION_EVENT_STATE, EVENT_LIFECYCLE_COMPLETED)"));
        }
        if name == "EventScript_NPCEvent_LouOrRuby_Introduction" {
            assert!(source.contains("SetTalkNameplateCharacter(CHARACTER_LOU_OR_RUBY)"));
            assert!(source.contains(
                "VarSet(VAR_LOU_OR_RUBY_INTRODUCTION_EVENT_STATE, EVENT_LIFECYCLE_COMPLETED)"
            ));
        }
        if name == "EventScript_FestivalEvent_HarvestSpriteTeaParty" {
            assert!(source.contains("SetPlayerHeldFood(FOOD_RELAX_TEA_LEAVES)"));
            for sprite in ["STAID", "NAPPY", "BOLD", "CHEF", "AQUA", "HOGGY", "TIMID"] {
                assert!(source.contains(&format!("AddNpcFriendship(CHARACTER_{sprite}, 10)")));
            }
            assert!(source.contains("SetGameTime(18, 1)"));
            assert!(source.contains(
                "VarSet(VAR_HARVEST_SPRITE_TEA_PARTY_EVENT_STATE, EVENT_LIFECYCLE_COMPLETED)"
            ));
        }
        if name == "EventScript_AchievementEvent_Collection_HarvestGoddessJewelsExchange" {
            assert!(source.contains("RemoveAllOwnedArticles(ARTICLE_HARVEST_GODDESS_JEWEL)"));
            assert!(source.contains("TOOL_GEM_GODDESS"));
            assert!(source.contains(
                "VarSet(VAR_HARVEST_GODDESS_JEWEL_EXCHANGE_STATE, EVENT_LIFECYCLE_COMPLETED)"
            ));
        }
        if name == "EventScript_AchievementEvent_Collection_KappaJewelsExchange" {
            assert!(source.contains("RemoveAllOwnedArticles(ARTICLE_KAPPA_JEWEL)"));
            assert!(source.contains("TOOL_GEM_KAPPA"));
            assert!(source
                .contains("VarSet(VAR_KAPPA_JEWEL_EXCHANGE_STATE, EVENT_LIFECYCLE_COMPLETED)"));
        }
        if name == "EventScript_AchievementEvent_Collection_JewelsOfTruthExchange" {
            assert!(source.contains("GetFirstFreeRucksackToolSlot()"));
            assert!(source.contains(
                "VarSet(VAR_JEWELS_OF_TRUTH_EXCHANGE_RETRY_STATE, EVENT_LIFECYCLE_COMPLETED)"
            ));
            assert!(source.contains("RemoveAllOwnedArticles(ARTICLE_JEWEL_OF_TRUTH)"));
            assert!(source.contains("TOOL_GEM_TRUTH"));
            assert!(source
                .contains("VarSet(VAR_JEWELS_OF_TRUTH_REWARD_STATE, EVENT_LIFECYCLE_COMPLETED)"));
        }
        if name == "EventScript_NPCEvent_Kappa_DailyOfferingAppearance" {
            assert!(source.contains(
                "VarSet(VAR_KAPPA_DAILY_APPEARANCE_EVENT_STATE, EVENT_LIFECYCLE_COMPLETED)"
            ));
        }
        if name == "EventScript_NPCEvent_Staid_DialogueAndWorkChoices" {
            let invitation = if case.target.contains("MFOMT") {
                "ARTICLE_HARVEST_SPRITE_INVITATION"
            } else {
                "ARTICLE_INVITATION"
            };
            assert!(source.contains(&format!("case {invitation}:")));
            assert!(source.contains("var_0 = HARVEST_SPRITE_INTERACTION_TEA_PARTY_INVITATION"));
            assert!(source.contains("VarGet(VAR_SEASON) != SEASON_SPRING"));
            assert!(source.contains("VarGet(VAR_HOUR) >= 10 && VarGet(VAR_HOUR) < 17"));
            for sprite in ["NAPPY", "BOLD", "CHEF", "AQUA", "HOGGY", "TIMID"] {
                assert!(source.contains(&format!(
                    "GetEntityLocation(ENTITY_{sprite}) == MAP_HARVEST_SPRITES_HUT"
                )));
            }
            assert!(source.contains("UsePlayerHeldItem()"));
            assert!(source.contains("CallScript(EventScript_FestivalEvent_HarvestSpriteTeaParty)"));
        }
        if name == "EventScript_LocationTransition_EnterFarmWithDailyEventDispatch" {
            assert!(source.contains("var_1 == SEASON_SPRING && var_2 == DAY_OF_MONTH_03"));
            assert!(source.contains("CallScript(EventScript_NPCEvent_Won_Introduction)"));
            assert!(source.contains("GetMoney() >= 500"));
            assert!(source.contains("CallScript(EventScript_NPCEvent_Won_AppleChallenge)"));
            assert!(source.contains("GetMoney() >= 5000"));
            assert!(source.contains("CallScript(EventScript_NPCEvent_Won_VasePurchase)"));
            assert!(source.contains("VarGet(VAR_GAMECUBE_LINK_VAN_INTRODUCTION_AVAILABLE) == TRUE"));
            assert!(source.contains("var_3 == DAY_OF_WEEK_WEDNESDAY"));
            assert!(source.contains("CallScript(EventScript_NPCEvent_Van_Introduction)"));
            assert!(source
                .contains("VarGet(VAR_GAMECUBE_LINK_LOU_OR_RUBY_INTRODUCTION_AVAILABLE) == TRUE"));
            assert!(source.contains("var_3 == DAY_OF_WEEK_SUNDAY"));
            assert!(source.contains("CallScript(EventScript_NPCEvent_LouOrRuby_Introduction)"));
        }
        for token in source
            .split(|character: char| !character.is_ascii_alphanumeric() && character != '_')
            .filter(|token| {
                token.strip_prefix("AUDIO_SEQUENCE_").is_some_and(|suffix| {
                    suffix.len() == 3 && suffix.bytes().all(|byte| byte.is_ascii_digit())
                })
            })
        {
            numbered_audio_sequences.insert(token.to_owned());
            *numbered_audio_usage
                .entry(token.to_owned())
                .or_default()
                .entry(name.to_owned())
                .or_default() += 1;
        }
        assert_statically_typed_arguments_are_symbolic(case, id, &source, &callables);
        symbolic_script_calls += source.matches("CallScript(EventScript_").count();
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
        outputs.push((format!("{name}.mary.c"), format!("{preamble}{source}")));
        verified += 1;
    }
    assert!(
        symbolic_script_calls > 0,
        "{} did not render any CallScript target as a script symbol",
        case.name
    );
    let expected_numbered_audio = HashSet::new();
    assert_eq!(
        numbered_audio_sequences, expected_numbered_audio,
        "{} numbered audio inventory changed and needs semantic review",
        case.name
    );
    let expected_numbered_audio_usage = HashMap::new();
    assert_eq!(
        numbered_audio_usage, expected_numbered_audio_usage,
        "{} numbered audio call sites changed and need renewed semantic review",
        case.name
    );
    assert_generated_output_invariants(case, &outputs, &callables.scope);
    publish_verified_outputs(case.name, &outputs)?;
    println!("{} Mary-C strict round-trip: {verified} scripts", case.name);
    Ok(())
}

fn assert_generated_output_invariants(
    case: &RomCase,
    outputs: &[(String, String)],
    scope: &ConstScope,
) {
    const NUMBERED_DOMAIN_PREFIXES: [&str; 9] = [
        "MAP_",
        "CHARACTER_",
        "TALK_PORTRAIT_",
        "ENTITY_",
        "ANIMATION_",
        "ANIMATION_ID_",
        "FOOD_",
        "ARTICLE_",
        "TOOL_",
    ];
    const SEMANTIC_VARIABLE_SUFFIXES: [&str; 9] = [
        "STATE",
        "RESULT",
        "STATUS",
        "PHASE",
        "TYPE",
        "KIND",
        "MODE",
        "CHOICE",
        "SELECTION",
    ];

    fn starts_with_integer(source: &str) -> bool {
        let source = source.trim_start();
        let source = source.strip_prefix('-').unwrap_or(source);
        source.as_bytes().first().is_some_and(u8::is_ascii_digit)
    }

    fn parse_integer_prefix(source: &str) -> Option<i64> {
        let source = source.trim_start();
        let (negative, source) = source
            .strip_prefix('-')
            .map_or((false, source), |source| (true, source));
        let (radix, digits) = source
            .strip_prefix("0x")
            .or_else(|| source.strip_prefix("0X"))
            .map_or((10, source), |source| (16, source));
        let length = digits
            .bytes()
            .take_while(|byte| match radix {
                16 => byte.is_ascii_hexdigit(),
                _ => byte.is_ascii_digit(),
            })
            .count();
        if length == 0 {
            return None;
        }
        let value = i64::from_str_radix(&digits[..length], radix).ok()?;
        Some(if negative { -value } else { value })
    }

    fn matching_parenthesis(source: &str, open: usize) -> Option<usize> {
        let mut depth = 0usize;
        let mut string = false;
        let mut escaped = false;
        for (offset, byte) in source.as_bytes()[open..].iter().copied().enumerate() {
            if string {
                if escaped {
                    escaped = false;
                } else if byte == b'\\' {
                    escaped = true;
                } else if byte == b'"' {
                    string = false;
                }
                continue;
            }
            match byte {
                b'"' => string = true,
                b'(' => depth += 1,
                b')' => {
                    depth = depth.checked_sub(1)?;
                    if depth == 0 {
                        return Some(open + offset);
                    }
                }
                _ => {}
            }
        }
        None
    }

    let scripts = outputs
        .iter()
        .filter(|(filename, _)| filename.ends_with(".mary.c"))
        .collect::<Vec<_>>();
    assert_eq!(
        scripts.len(),
        case.slots,
        "{} generated slot files",
        case.name
    );

    let generic_guard_ids: &[i64] = if case.target.contains("MFOMT") {
        &[
            232, 233, 244, 245, 250, 284, 324, 325, 327, 328, 343, 344, 352, 411, 412, 413, 415,
        ]
    } else {
        &[
            224, 225, 236, 237, 242, 276, 316, 317, 319, 320, 334, 335, 336, 344, 381, 382, 383,
            385, 449,
        ]
    };
    let expected_guard_files = if case.target.contains("MFOMT") {
        28
    } else {
        27
    };
    let expected_exceptions: HashMap<&str, Vec<i64>> = if case.target.contains("MFOMT") {
        HashMap::from([(
            "EventScript_SystemEvent_ResetDailyWorldState.mary.c",
            vec![467],
        )])
    } else {
        HashMap::from([(
            "EventScript_SystemEvent_ResetDailyWorldState.mary.c",
            vec![423, 449],
        )])
    };
    let expected_writers: Vec<i64> = if case.target.contains("MFOMT") {
        vec![467]
    } else {
        vec![423, 449]
    };

    let mut guard_files = 0usize;
    let mut exceptions = HashMap::<String, Vec<i64>>::new();
    let mut writers = Vec::<(String, String, i64)>::new();
    let mut event_context_callers = Vec::<String>::new();
    let mut staff_credit_callers = Vec::<String>::new();

    for (filename, source) in scripts {
        assert!(
            !source.contains("ANIMATION_ID_"),
            "{} {filename} emits a numbered animation placeholder",
            case.name
        );

        for token in
            source.split(|character: char| !character.is_ascii_alphanumeric() && character != '_')
        {
            let numbered_domain = NUMBERED_DOMAIN_PREFIXES.iter().any(|prefix| {
                token.strip_prefix(prefix).is_some_and(|suffix| {
                    !suffix.is_empty() && suffix.bytes().all(|byte| byte.is_ascii_digit())
                })
            });
            assert!(
                !numbered_domain,
                "{} {filename} regressed to numbered game-domain symbol {token}",
                case.name
            );
            if token.contains("UNKNOWN") || token.contains("Unknown") {
                assert!(
                    token.starts_with("VAR_UNKNOWN_SLOT_"),
                    "{} {filename} contains unaudited unknown symbol {token}",
                    case.name
                );
            }
        }

        for line in source.lines() {
            for call in ["VarGet(VAR_", "VarSet(VAR_"] {
                let mut remainder = line;
                while let Some(start) = remainder.find(call) {
                    remainder = &remainder[start + call.len()..];
                    let Some(name_end) = remainder.find(|character: char| {
                        !character.is_ascii_alphanumeric() && character != '_'
                    }) else {
                        break;
                    };
                    let name = &remainder[..name_end];
                    remainder = &remainder[name_end..];
                    if !SEMANTIC_VARIABLE_SUFFIXES
                        .iter()
                        .any(|suffix| name.ends_with(suffix))
                    {
                        continue;
                    }
                    let numeric_value = if call.starts_with("VarGet") {
                        remainder
                            .strip_prefix(')')
                            .map(str::trim_start)
                            .and_then(|tail| {
                                ["==", "!=", "<=", ">=", "<", ">"]
                                    .into_iter()
                                    .find_map(|operator| tail.strip_prefix(operator))
                            })
                            .is_some_and(starts_with_integer)
                    } else {
                        remainder.strip_prefix(',').is_some_and(starts_with_integer)
                    };
                    assert!(
                        !numeric_value,
                        "{} {filename} lost the symbolic value domain for VAR_{name}: {line}",
                        case.name
                    );
                }
            }
        }

        for (line_number, line) in source.lines().enumerate() {
            for (name, (_, shape)) in scope.callable_map() {
                let mary::ir::ValueType::UserType(type_id) = shape.return_type() else {
                    continue;
                };
                let needle = format!("{name}(");
                let mut search_start = 0usize;
                while let Some(relative_open) = line[search_start..].find(&needle) {
                    let open = search_start + relative_open + name.len();
                    let Some(close) = matching_parenthesis(line, open) else {
                        break;
                    };
                    let tail = line[close + 1..].trim_start();
                    let numeric = ["==", "!=", "<=", ">=", "<", ">"]
                        .into_iter()
                        .find_map(|operator| tail.strip_prefix(operator))
                        .and_then(parse_integer_prefix);
                    if let Some(value) = numeric {
                        assert!(
                            scope.typed_int_const_name(type_id, value).is_none(),
                            "{} {filename}:{} compares {name}'s typed return with known numeric member {value}: {line}",
                            case.name,
                            line_number + 1
                        );
                    }
                    search_start = close + 1;
                }
            }
        }

        let mut local_return_types = HashMap::<String, Option<usize>>::new();
        for line in source.lines() {
            let line = line.trim();
            let Some((local, value)) = line.split_once(" = ") else {
                continue;
            };
            if !local.starts_with("var_")
                || !local
                    .bytes()
                    .all(|byte| byte.is_ascii_alphanumeric() || byte == b'_')
            {
                continue;
            }
            let callable = value.split_once('(').and_then(|(name, arguments)| {
                let open = value.len() - arguments.len() - 1;
                let close = matching_parenthesis(value, open)?;
                (value[close + 1..].trim() == ";").then(|| name.trim())
            });
            let assigned_type = callable
                .and_then(|name| scope.callable_map().get(name))
                .and_then(|(_, shape)| match shape.return_type() {
                    mary::ir::ValueType::UserType(type_id) => Some(type_id),
                    _ => None,
                });
            local_return_types
                .entry(local.to_owned())
                .and_modify(|current| {
                    if *current != assigned_type {
                        *current = None;
                    }
                })
                .or_insert(assigned_type);
        }
        for (local, type_id) in local_return_types {
            let Some(type_id) = type_id else {
                continue;
            };
            let needle = format!("switch ({local})");
            let mut remainder = source.as_str();
            while let Some(switch_start) = remainder.find(&needle) {
                remainder = &remainder[switch_start + needle.len()..];
                let Some(open) = remainder.find('{') else {
                    break;
                };
                let mut depth = 1usize;
                for line in remainder[open + 1..].lines() {
                    if depth == 1 {
                        if let Some(case_label) = line.trim().strip_prefix("case ") {
                            if let Some(value) = parse_integer_prefix(case_label) {
                                assert!(
                                    scope.typed_int_const_name(type_id, value).is_none(),
                                    "{} {filename} switches on {local}'s typed callable return with known numeric member {value}: {line}",
                                    case.name
                                );
                            }
                        }
                    }
                    for byte in line.bytes() {
                        match byte {
                            b'{' => depth += 1,
                            b'}' => depth = depth.saturating_sub(1),
                            _ => {}
                        }
                    }
                    if depth == 0 {
                        break;
                    }
                }
            }
        }

        let mut unknown_ids = source
            .split(|character: char| !character.is_ascii_alphanumeric() && character != '_')
            .filter_map(|token| token.strip_prefix("VAR_UNKNOWN_SLOT_"))
            .map(|suffix| suffix.parse::<i64>().unwrap())
            .collect::<Vec<_>>();
        unknown_ids.sort_unstable();
        unknown_ids.dedup();
        if unknown_ids == generic_guard_ids {
            guard_files += 1;
            for id in &unknown_ids {
                assert!(
                    source.contains(&format!("VarGet(VAR_UNKNOWN_SLOT_{id:03}) == 1")),
                    "{} {filename}: unknown slot {id} escaped the audited guard form",
                    case.name
                );
            }
        } else if !unknown_ids.is_empty() {
            exceptions.insert(filename.clone(), unknown_ids);
        }

        for line in source.lines() {
            for operation in ["VarSet(", "VarAdd(", "VarSub("] {
                let Some(arguments) = line.split_once(operation).map(|(_, rest)| rest) else {
                    continue;
                };
                let Some(suffix) = arguments.strip_prefix("VAR_UNKNOWN_SLOT_") else {
                    continue;
                };
                let digits = suffix
                    .bytes()
                    .take_while(u8::is_ascii_digit)
                    .collect::<Vec<_>>();
                writers.push((
                    filename.clone(),
                    operation.trim_end_matches('(').to_owned(),
                    std::str::from_utf8(&digits).unwrap().parse().unwrap(),
                ));
            }
        }

        let context_calls = source.matches("GetEventContextValue()").count();
        if context_calls != 0 {
            assert_eq!(context_calls, 1, "{} {filename}", case.name);
            event_context_callers.push(filename.clone());
        }
        let credit_calls = source.matches("RunStaffCredits();").count();
        if credit_calls != 0 {
            assert_eq!(credit_calls, 1, "{} {filename}", case.name);
            assert!(
                source
                    .find("FadeOutScreen(")
                    .is_some_and(|fade| fade < source.find("RunStaffCredits();").unwrap()),
                "{} {filename}: staff credits lost the preceding fade",
                case.name
            );
            staff_credit_callers.push(filename.clone());
        }
    }

    assert_eq!(
        guard_files, expected_guard_files,
        "{} unknown guards",
        case.name
    );
    assert_eq!(
        exceptions,
        expected_exceptions
            .into_iter()
            .map(|(filename, ids)| (filename.to_owned(), ids))
            .collect(),
        "{} unknown exceptions",
        case.name
    );
    writers.sort_unstable();
    let expected_writers = expected_writers
        .into_iter()
        .map(|id| {
            (
                "EventScript_SystemEvent_ResetDailyWorldState.mary.c".to_owned(),
                "VarSet".to_owned(),
                id,
            )
        })
        .collect::<Vec<_>>();
    assert_eq!(writers, expected_writers, "{} unknown writers", case.name);

    event_context_callers.sort_unstable();
    let mut expected_context_callers = [
        "EventScript_AchievementEvent_Shipping_10000Items.mary.c",
        "EventScript_AchievementEvent_Shipping_100000Items.mary.c",
        "EventScript_AchievementEvent_Shipping_1000000Items.mary.c",
        "EventScript_AchievementEvent_Shipping_10000000Items.mary.c",
        "EventScript_AchievementEvent_Shipping_100000000Items.mary.c",
        "EventScript_AchievementEvent_Shipping_1000000000Items.mary.c",
        "EventScript_FarmEvent_Zack_DailyShippingPickup.mary.c",
    ]
    .map(str::to_owned)
    .to_vec();
    expected_context_callers.sort_unstable();
    assert_eq!(
        event_context_callers, expected_context_callers,
        "{} event-context consumers",
        case.name
    );

    staff_credit_callers.sort_unstable();
    let (expected_credit_calls, expected_weddings, expect_opening, expect_thomas) =
        if case.target.contains("MFOMT") {
            (10, 8, true, true)
        } else {
            (6, 6, false, false)
        };
    assert_eq!(
        staff_credit_callers.len(),
        expected_credit_calls,
        "{} staff-credit callers",
        case.name
    );
    assert_eq!(
        staff_credit_callers
            .iter()
            .filter(|name| name.contains("_Wedding"))
            .count(),
        expected_weddings,
        "{} wedding credits",
        case.name
    );
    assert_eq!(
        staff_credit_callers
            .iter()
            .any(|name| name.contains("MFoMTOpening")),
        expect_opening,
        "{} opening credits",
        case.name
    );
    assert_eq!(
        staff_credit_callers
            .iter()
            .any(|name| name.contains("Thomas_Dialogue")),
        expect_thomas,
        "{} Thomas credits",
        case.name
    );
}

fn assert_numbered_script_name_matches_slot(name: &str, id: usize) {
    if let Some(number) = name.strip_prefix("EventScript_") {
        if !number.is_empty() && number.bytes().all(|byte| byte.is_ascii_digit()) {
            assert_eq!(
                number.parse::<usize>().unwrap(),
                id,
                "numbered script name must identify its physical slot"
            );
        }
    }
    // An unproven event may contain real instructions or text. Retaining its
    // physical ID is honest, not a decompilation failure. The caller still
    // parses every instruction and compares the complete recompiled bytes.
}

#[test]
fn numbered_script_symbols_are_only_empty_or_terminal_placeholders_in_roms() {
    for case in CASES {
        let rom = fs::read(local_rom_path(case.rom)).unwrap();
        let entries = get_script_table(&rom).unwrap();
        let options = Options::default().define(case.target).unwrap();
        let symbols = parse_text_name_table(
            &fs::read_to_string("goodies/mary_scripts_text.mary.sym").unwrap(),
            &options,
        )
        .unwrap();
        let mut empty = 0;
        let mut terminal = 0;

        for entry in entries {
            let ScriptTableEntry::Script { id, data, backing } = entry else {
                continue;
            };
            let Some(name) = symbols.script_name(id) else {
                continue;
            };
            if name != format!("EventScript_{id:04}") {
                continue;
            }

            let decoded = decode_script_with_backing(data, backing).unwrap();
            assert!(
                decoded.strings.is_empty(),
                "{} numbered slot {id} contains text and needs an event-level name",
                case.name
            );
            match decoded.instructions.as_slice() {
                [] => empty += 1,
                [Ins::Exit] => terminal += 1,
                instructions => panic!(
                    "{} numbered slot {id} contains semantic instructions and needs review: {instructions:?}",
                    case.name
                ),
            }
        }

        let expected = if case.target.contains("MFOMT") {
            (35, 9)
        } else {
            (22, 7)
        };
        assert_eq!(
            (empty, terminal),
            expected,
            "{} numbered placeholder shape inventory changed",
            case.name
        );
    }
}

#[test]
fn unknown_event_names_do_not_require_fabricated_semantics() {
    assert_numbered_script_name_matches_slot("EventScript_0017", 17);
    assert_numbered_script_name_matches_slot("EventScript_FestivalEvent_Test", 17);
}

#[test]
#[should_panic(expected = "numbered script name must identify its physical slot")]
fn numbered_event_names_must_not_point_to_a_different_slot() {
    assert_numbered_script_name_matches_slot("EventScript_0017", 18);
}

/// Stage the verified strings before retiring old filenames; never mix generations.
fn publish_verified_outputs(name: &str, outputs: &[(String, String)]) -> std::io::Result<()> {
    let root = Path::new(env!("CARGO_MANIFEST_DIR")).canonicalize()?;
    let output_root = root.join("decompiled_text");
    let archive_root = root.join("test_failures");
    for directory in [&output_root, &archive_root] {
        fs::create_dir_all(directory)?;
        if directory.canonicalize()? != *directory {
            return Err(std::io::Error::other(
                "output directory redirects outside its expected path",
            ));
        }
    }
    let variant = name.replace('-', "_");
    let destination = output_root.join(&variant);
    if destination.exists() && destination.canonicalize()? != destination {
        return Err(std::io::Error::other(
            "refusing to replace a redirected output directory",
        ));
    }
    let stamp = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map_err(std::io::Error::other)?
        .as_nanos();
    let staging = archive_root.join(format!("{variant}_verified_{stamp}"));
    fs::create_dir(&staging)?;
    for (filename, contents) in outputs {
        if Path::new(filename)
            .file_name()
            .and_then(|name| name.to_str())
            != Some(filename.as_str())
        {
            return Err(std::io::Error::other("invalid output filename"));
        }
        fs::write(staging.join(filename), contents)?;
    }
    let previous = archive_root.join(format!("{variant}_previous_{stamp}"));
    let had_previous = destination.exists();
    if had_previous {
        fs::rename(&destination, &previous)?;
    }
    if let Err(error) = fs::rename(&staging, &destination) {
        if had_previous {
            fs::rename(&previous, &destination)?;
        }
        return Err(error);
    }
    if had_previous {
        if let Err(error) = fs::remove_dir_all(&previous) {
            eprintln!(
                "warning: verified output was published, but stale backup {} could not be removed: {error}",
                previous.display()
            );
        }
    }
    println!("Verified scripts saved to {}", destination.display());
    Ok(())
}

#[test]
fn native_remaining_retail_noops_have_no_hidden_business_handler() {
    fn thumb_bl_target(rom: &[u8], instruction: usize) -> i32 {
        let hi = u16::from_le_bytes(rom[instruction..instruction + 2].try_into().unwrap());
        let lo = u16::from_le_bytes(rom[instruction + 2..instruction + 4].try_into().unwrap());
        assert_eq!(hi & 0xF800, 0xF000);
        assert_eq!(lo & 0xF800, 0xF800);
        let displacement =
            (((i32::from(hi & 0x7FF) << 12) | (i32::from(lo & 0x7FF) << 1)) << 9) >> 9;
        instruction as i32 + 4 + displacement
    }

    for (case, (dispatch_table, animal_slot, handler, empty_leaf, epilogue)) in CASES.iter().zip([
        (0x3F904, 0x143, 0x4021C, 0x12218, 0x45572),
        (0x3FAF0, 0x147, 0x40430, 0x12304, 0x45C88),
        (0x3F578, 0x143, 0x3FE90, 0x120E8, 0x451E6),
        (0x3F84C, 0x147, 0x4018C, 0x121A8, 0x459E4),
    ]) {
        let rom = fs::read(local_rom_path(case.rom)).unwrap();
        let no_op_014_entry = dispatch_table + 0x00E * 4;
        assert_eq!(
            u32::from_le_bytes(
                rom[no_op_014_entry..no_op_014_entry + 4]
                    .try_into()
                    .unwrap()
            ),
            handler as u32 + 0x08000000,
            "{} NoOp014 physical slot",
            case.name
        );
        // Pop one VM operand, test the current-script-entity pointer, then
        // either leave immediately or call a two-byte BX LR leaf before leaving.
        assert_eq!(thumb_bl_target(&rom, handler + 0x22), epilogue);
        assert_eq!(thumb_bl_target(&rom, handler + 0x26), empty_leaf as i32);
        assert_eq!(thumb_bl_target(&rom, handler + 0x2A), epilogue);
        assert_eq!(&rom[empty_leaf..empty_leaf + 2], &[0x70, 0x47]);

        let animal_entry = dispatch_table + animal_slot * 4;
        assert_eq!(
            u32::from_le_bytes(rom[animal_entry..animal_entry + 4].try_into().unwrap()),
            epilogue as u32 + 0x08000000,
            "{} animal-event initialization placeholder",
            case.name
        );
    }
}

#[test]
fn native_retail_tutorial_placeholders_share_the_zero_return_epilogue() {
    // Four consecutive tutorial ABI slots survive in every retail target, but
    // every dispatch entry jumps directly to the interpreter's zero-return
    // epilogue. They do not enter a field, object, or egg implementation.
    for (case, (dispatch_table, first_slot, epilogue, stack_words)) in CASES.iter().zip([
        (0x3F904, 0x13E, 0x45572, 0x10),
        (0x3FAF0, 0x142, 0x45C88, 0x17),
        (0x3F578, 0x13E, 0x451E6, 0x10),
        (0x3F84C, 0x142, 0x459E4, 0x17),
    ]) {
        let rom = fs::read(local_rom_path(case.rom)).unwrap();
        for slot in first_slot..first_slot + 4 {
            let entry = dispatch_table + slot * 4;
            assert_eq!(
                u32::from_le_bytes(rom[entry..entry + 4].try_into().unwrap()),
                epilogue as u32 + 0x08000000,
                "{} tutorial placeholder slot {slot:#x}",
                case.name
            );
        }
        assert_eq!(
            &rom[epilogue..epilogue + 18],
            &[
                0x00,
                0x20,
                stack_words,
                0xB0,
                0x38,
                0xBC,
                0x98,
                0x46,
                0xA1,
                0x46,
                0xAA,
                0x46,
                0xF0,
                0xBC,
                0x02,
                0xBC,
                0x08,
                0x47,
            ],
            "{} zero-return interpreter epilogue",
            case.name
        );
    }
}

#[test]
fn native_callable_dispatch_tables_have_only_the_audited_shared_handlers() {
    for (case, dispatch_table, slot_count, expected_groups) in [
        (
            &CASES[0],
            0x3F904usize,
            0x147usize,
            vec![vec![0x13E, 0x13F, 0x140, 0x141, 0x143]],
        ),
        (
            &CASES[1],
            0x3FAF0,
            0x153,
            vec![vec![0x122, 0x123], vec![0x142, 0x143, 0x144, 0x145, 0x147]],
        ),
        (
            &CASES[2],
            0x3F578,
            0x147,
            vec![vec![0x13E, 0x13F, 0x140, 0x141, 0x143]],
        ),
        (
            &CASES[3],
            0x3F84C,
            0x153,
            vec![vec![0x122, 0x123], vec![0x142, 0x143, 0x144, 0x145, 0x147]],
        ),
    ] {
        let rom = fs::read(local_rom_path(case.rom)).unwrap();
        let mut slots_by_handler = HashMap::<u32, Vec<usize>>::new();
        for slot in 0..slot_count {
            let entry = dispatch_table + slot * 4;
            let handler = u32::from_le_bytes(rom[entry..entry + 4].try_into().unwrap());
            slots_by_handler.entry(handler).or_default().push(slot);
        }

        let mut actual_groups = slots_by_handler
            .into_values()
            .filter(|slots| slots.len() > 1)
            .collect::<Vec<_>>();
        actual_groups.sort();
        assert_eq!(
            actual_groups, expected_groups,
            "{} callable handler-sharing topology changed",
            case.name
        );
    }
}

#[test]
fn native_character_birthday_callable_uses_the_verified_physical_handlers() {
    for (case, dispatch_table, slot, expected_handler) in [
        (&CASES[0], 0x3F904usize, 0x07Busize, 0x41B5Cusize),
        (&CASES[1], 0x3FAF0, 0x07E, 0x41DA4),
        (&CASES[2], 0x3F578, 0x07B, 0x417D0),
        (&CASES[3], 0x3F84C, 0x07E, 0x41B00),
    ] {
        let rom = fs::read(local_rom_path(case.rom)).unwrap();
        let entry = dispatch_table + slot * 4;
        let handler = (u32::from_le_bytes(rom[entry..entry + 4].try_into().unwrap()) & !1)
            .wrapping_sub(0x0800_0000) as usize;
        assert_eq!(handler, expected_handler, "{} birthday handler", case.name);
        assert_eq!(
            &rom[handler..handler + 40],
            &[
                0x24, 0x24, 0x64, 0x19, 0xA0, 0x46, 0xDA, 0x20, 0x40, 0x00, 0x2E, 0x18, 0x32, 0x68,
                0x90, 0x00, 0x04, 0x38, 0x40, 0x44, 0x07, 0x68, 0x00, 0x2A, 0x01, 0xD0, 0x50, 0x1E,
                0x30, 0x60, 0xD4, 0x21, 0x89, 0x00, 0x68, 0x18, 0x00, 0x68, 0x45, 0x7C,
            ],
            "{} birthday argument/date setup",
            case.name
        );
        assert!(
            rom[handler..handler + 0x70].windows(16).any(|window| {
                window
                    == [
                        0x00, 0x23, 0x7F, 0x21, 0x0A, 0x1C, 0x2A, 0x40, 0x01, 0x40, 0x8A, 0x42,
                        0x00, 0xD1, 0x01, 0x23,
                    ]
            }),
            "{} birthday handler must compare the low seven encoded date bits",
            case.name
        );
    }
}

#[test]
fn native_callable_stack_pop_counts_match_every_public_declaration() {
    for (case, dispatch_table, no_op_slots) in [
        (
            &CASES[0],
            0x3F904usize,
            &[0x13E, 0x13F, 0x140, 0x141, 0x143][..],
        ),
        (
            &CASES[1],
            0x3FAF0usize,
            &[0x142, 0x143, 0x144, 0x145, 0x147][..],
        ),
        (
            &CASES[2],
            0x3F578usize,
            &[0x13E, 0x13F, 0x140, 0x141, 0x143][..],
        ),
        (
            &CASES[3],
            0x3F84Cusize,
            &[0x142, 0x143, 0x144, 0x145, 0x147][..],
        ),
    ] {
        let rom = fs::read(case.rom).unwrap();
        let options = Options::default().define(case.target).unwrap();
        let constants = parse_constant_header(
            &fs::read_to_string("goodies/mary_constants.mary.h").unwrap(),
            &options,
        )
        .unwrap();
        let callables = parse_callable_table_with_scope(
            &fs::read_to_string("goodies/mary_callables.mary.h").unwrap(),
            &options,
            &constants,
        )
        .unwrap();
        let by_id = callables
            .scope
            .callable_map()
            .iter()
            .map(|(name, (id, shape))| (id.0, (name.as_str(), shape.num_parameters())))
            .collect::<HashMap<_, _>>();
        let handlers = (0..callables.next_id)
            .map(|slot| {
                (u32::from_le_bytes(
                    rom[dispatch_table + slot * 4..dispatch_table + slot * 4 + 4]
                        .try_into()
                        .unwrap(),
                ) as usize
                    & !1)
                    - 0x08000000
            })
            .collect::<Vec<_>>();
        let mut unique_handlers = handlers.clone();
        unique_handlers.sort_unstable();
        unique_handlers.dedup();

        for (slot, handler) in handlers.into_iter().enumerate().skip(2) {
            let &(name, declared_count) = by_id
                .get(&slot)
                .unwrap_or_else(|| panic!("{} slot 0x{slot:03X} is undeclared", case.name));
            let end = unique_handlers
                .iter()
                .copied()
                .find(|candidate| *candidate > handler)
                .unwrap_or(handler + 0x400)
                .min(handler + 0x400);
            let halfwords = rom[handler..end]
                .chunks_exact(2)
                .map(|bytes| u16::from_le_bytes([bytes[0], bytes[1]]))
                .collect::<Vec<_>>();
            let mut writes_by_base = HashMap::<usize, usize>::new();
            for (index, instruction) in halfwords.iter().copied().enumerate() {
                // Thumb SUB (immediate 3): `subs Rd, Rn, #1`.
                if instruction & 0xFE00 != 0x1E00 || (instruction >> 6) & 7 != 1 {
                    continue;
                }
                let destination = instruction as usize & 7;
                for following in halfwords.iter().copied().skip(index + 1).take(3) {
                    // Thumb STR (immediate): `str Rd, [Rb, #0]`.
                    if following & 0xF800 == 0x6000
                        && (following >> 6) & 0x1F == 0
                        && following as usize & 7 == destination
                    {
                        *writes_by_base
                            .entry((following as usize >> 3) & 7)
                            .or_default() += 1;
                        break;
                    }
                }
            }
            let native_count = writes_by_base.values().copied().max().unwrap_or(0);
            if no_op_slots.contains(&slot) {
                assert_eq!(
                    native_count, 0,
                    "{} {name} stopped being a true native no-op",
                    case.name
                );
            } else {
                assert_eq!(
                    native_count, declared_count,
                    "{} slot 0x{slot:03X} {name} native stack-pop count changed",
                    case.name
                );
            }
        }
    }
}

#[test]
fn mfomt_var_access_vm_wrappers_match_between_regions() {
    let us = fs::read(local_rom_path(CASES[1].rom)).unwrap();
    let jp = fs::read(local_rom_path(CASES[3].rom)).unwrap();
    let us_dispatch = 0x3FAF0usize;
    let jp_dispatch = 0x3F84Cusize;

    fn handler_at(rom: &[u8], dispatch: usize, slot: usize) -> usize {
        (u32::from_le_bytes(
            rom[dispatch + slot * 4..dispatch + slot * 4 + 4]
                .try_into()
                .unwrap(),
        ) as usize
            & !1)
            - 0x0800_0000
    }

    let us_get = handler_at(&us, us_dispatch, 0x03E);
    let us_set = handler_at(&us, us_dispatch, 0x03F);
    let jp_get = handler_at(&jp, jp_dispatch, 0x03E);
    let jp_set = handler_at(&jp, jp_dispatch, 0x03F);
    assert_eq!((us_get, us_set), (0x41390, 0x413B6));
    assert_eq!((jp_get, jp_set), (0x410EC, 0x41112));

    // Both regions pop the same VM operands in byte-identical wrappers. The
    // BL displacements are also identical because the downstream native
    // routines move by the same regional relocation delta.
    assert_eq!(&us[us_get..us_get + 0x26], &jp[jp_get..jp_get + 0x26]);
    assert_eq!(&us[us_set..us_set + 0x38], &jp[jp_set..jp_set + 0x38]);

    assert_eq!(thumb_call_target(&us, us_get + 0x1E), 0x46024);
    assert_eq!(thumb_call_target(&us, us_get + 0x22), 0x45908);
    assert_eq!(thumb_call_target(&jp, jp_get + 0x1E), 0x45D80);
    assert_eq!(thumb_call_target(&jp, jp_get + 0x22), 0x45664);

    assert_eq!(thumb_call_target(&us, us_set + 0x30), 0x4A3F8);
    assert_eq!(thumb_call_target(&us, us_set + 0x34), 0x45C88);
    assert_eq!(thumb_call_target(&jp, jp_set + 0x30), 0x4A154);
    assert_eq!(thumb_call_target(&jp, jp_set + 0x34), 0x459E4);
}

#[test]
fn native_fade_in_variants_differ_by_active_scene_hook_on_all_targets() {
    for (case, dispatch_table, fade_in_slot, no_hook_slot, fade_in, no_hook) in [
        (
            &CASES[0],
            0x3F904usize,
            0x034usize,
            0x035usize,
            0x40EF6usize,
            0x40F44usize,
        ),
        (&CASES[1], 0x3FAF0, 0x035, 0x036, 0x41122, 0x41170),
        (&CASES[2], 0x3F578, 0x034, 0x035, 0x40B6A, 0x40BB8),
        (&CASES[3], 0x3F84C, 0x035, 0x036, 0x40E7E, 0x40ECC),
    ] {
        let rom = fs::read(local_rom_path(case.rom)).unwrap();
        let pointer = |slot: usize| {
            let entry = dispatch_table + slot * 4;
            u32::from_le_bytes(rom[entry..entry + 4].try_into().unwrap())
        };
        assert_eq!(pointer(fade_in_slot), 0x0800_0000 + fade_in as u32);
        assert_eq!(pointer(no_hook_slot), 0x0800_0000 + no_hook as u32);
        assert_ne!(fade_in, no_hook, "{} fade-in entries", case.name);

        // Both handlers pop style and speed, request the inward transition
        // (r3 = 0), and call the same core routine. The stack byte consumed by
        // that routine is the sole semantic difference: 1 executes the entity
        // manager's vtable+0xB8 hook, while 0 skips it.
        assert_eq!(
            &rom[fade_in + 0x3C..fade_in + 0x48],
            &[0x6B, 0x46, 0x01, 0x20, 0x18, 0x70, 0x08, 0x1C, 0x31, 0x1C, 0x00, 0x23]
        );
        assert_eq!(
            &rom[no_hook + 0x3C..no_hook + 0x48],
            &[0x6B, 0x46, 0x00, 0x20, 0x18, 0x70, 0x08, 0x1C, 0x31, 0x1C, 0x00, 0x23]
        );
        assert_eq!(
            thumb_call_target(&rom, fade_in + 0x48),
            thumb_call_target(&rom, no_hook + 0x48),
            "{} shared inward-transition routine",
            case.name
        );
    }
}

#[test]
fn native_screen_fade_styles_and_speeds_match_all_targets() {
    for (case, core) in CASES.iter().zip([0x1297C, 0x12B08, 0x1284C, 0x129AC]) {
        let rom = fs::read(local_rom_path(case.rom)).unwrap();

        // The five-entry style jump table reaches these register assignments:
        // r6 = 2/1 selects brightness decrease/increase, while r7 = 1 enables
        // all four MOSAIC size fields. Style 4 leaves r6 at zero (mosaic only).
        let table = core + 0x28;
        let targets = (0..5)
            .map(|index| {
                (u32::from_le_bytes(
                    rom[table + index * 4..table + index * 4 + 4]
                        .try_into()
                        .unwrap(),
                ) as usize
                    & !1)
                    - 0x0800_0000
            })
            .collect::<Vec<_>>();
        assert_eq!(
            targets,
            [
                core + 0x3C,
                core + 0x42,
                core + 0x48,
                core + 0x4C,
                core + 0x50
            ],
            "{} fade-style dispatch",
            case.name
        );
        assert_eq!(&rom[core + 0x3C..core + 0x40], &[0x02, 0x26, 0x00, 0x27]);
        assert_eq!(&rom[core + 0x42..core + 0x46], &[0x01, 0x26, 0x00, 0x27]);
        assert_eq!(&rom[core + 0x48..core + 0x4C], &[0x02, 0x26, 0x02, 0xE0]);
        assert_eq!(&rom[core + 0x4C..core + 0x50], &[0x01, 0x26, 0x00, 0xE0]);
        assert_eq!(&rom[core + 0x50..core + 0x54], &[0x00, 0x26, 0x01, 0x27]);

        // Speed 0, 1, and 2 load the exact Q0.16 progress increments used by
        // the public FAST, NORMAL, and SLOW names.
        assert_eq!(
            u32::from_le_bytes(rom[core + 0x6C..core + 0x70].try_into().unwrap()),
            0x1112,
            "{} fast fade increment",
            case.name
        );
        assert_eq!(
            u32::from_le_bytes(rom[core + 0x74..core + 0x78].try_into().unwrap()),
            0x0889,
            "{} normal fade increment",
            case.name
        );
        assert_eq!(
            u32::from_le_bytes(rom[core + 0xB0..core + 0xB4].try_into().unwrap()),
            0x0444,
            "{} slow fade increment",
            case.name
        );
    }
}

#[test]
fn native_event_icon_priority_and_capacity_evidence_matches_all_four_roms() {
    // Independent native-evidence gate, not a replacement for script round trips.
    // Addresses were located from the FoMT source and Ghidra Headless; the
    // instructions retain layer & 3, replicate its two bits, and visit six slots.
    let locations = [
        (0x1C5E0, 0x19708, 0x197AA, 0x75BA44),
        (0x1C764, 0x1988C, 0x1992E, 0x9D90C8),
        (0x1C374, 0x1949C, 0x1953E, 0x77F65C),
        (0x1C5D8, 0x19700, 0x197A2, 0x9E02B0),
    ];
    let reference = fs::read(local_rom_path(CASES[0].rom)).unwrap();
    for (case, (store, expand, loop_tail, renderer)) in CASES.iter().zip(locations) {
        let rom = fs::read(local_rom_path(case.rom)).unwrap();
        for (offset, expected, role) in [
            (
                store,
                &[
                    0x60, 0x46, 0x07, 0x40, 0xBA, 0x00, 0x21, 0x78, 0x0D, 0x20, 0x40, 0x42, 0x08,
                    0x40, 0x10, 0x43, 0x20, 0x70,
                ][..],
                "two-bit record write",
            ),
            (
                expand,
                &[
                    0xAB, 0x00, 0x2B, 0x43, 0x28, 0x01, 0x03, 0x43, 0xAD, 0x01, 0x2B, 0x43,
                ][..],
                "four identical priority pairs",
            ),
            (
                loop_tail,
                &[
                    0x0C, 0x37, 0xC9, 0x25, 0x2D, 0x01, 0x6D, 0x44, 0x2C, 0x68, 0x01, 0x34, 0x2C,
                    0x60, 0x06, 0x2C, 0x00, 0xD2, 0x5D, 0xE7,
                ][..],
                "six-record draw loop",
            ),
        ] {
            assert_eq!(
                &rom[offset..offset + expected.len()],
                expected,
                "{}: {role}",
                case.name
            );
        }
        assert_eq!(
            &rom[renderer..renderer + 0x238],
            &reference[0x75BA44..0x75BA44 + 0x238],
            "{}: shared ARM OBJ renderer",
            case.name,
        );
    }
}

#[test]
fn native_event_icon_callables_and_item_icon_fallbacks_match_all_targets() {
    for (case, (dispatch, first_slot, handlers, scene_create, scene_remove, item_leaves)) in
        CASES.iter().zip([
            (
                0x3F904,
                0x139,
                [0x45054, 0x450E2, 0x4513A, 0x4517A, 0x4510C],
                0x16EC4,
                0x16EF0,
                [0xDCE0, 0xDF84, 0xDB60],
            ),
            (
                0x3FAF0,
                0x13D,
                [0x452E2, 0x45372, 0x453CE, 0x453FC, 0x453A0],
                0x16F44,
                0x16F70,
                [0xDD54, 0xDFF8, 0xDBD4],
            ),
            (
                0x3F578,
                0x139,
                [0x44CC8, 0x44D56, 0x44DAE, 0x44DEE, 0x44D80],
                0x16C58,
                0x16C84,
                [0xDCC0, 0xDF64, 0xDB40],
            ),
            (
                0x3F84C,
                0x13D,
                [0x4503E, 0x450CE, 0x4512A, 0x45158, 0x450FC],
                0x16DB8,
                0x16DE4,
                [0xDD08, 0xDFAC, 0xDB88],
            ),
        ])
    {
        let rom = fs::read(local_rom_path(case.rom)).unwrap();
        for (index, handler) in handlers.into_iter().enumerate() {
            let slot = first_slot + index;
            assert_eq!(
                u32::from_le_bytes(
                    rom[dispatch + slot * 4..dispatch + slot * 4 + 4]
                        .try_into()
                        .unwrap(),
                ),
                0x0800_0000 + handler as u32,
                "{} event/item icon slot {slot:#x}",
                case.name
            );
        }

        assert_eq!(thumb_call_target(&rom, handlers[0] + 0x88), scene_create);
        let remove_call = if case.name.starts_with("fomt-") {
            0x24
        } else {
            0x26
        };
        assert_eq!(
            thumb_call_target(&rom, handlers[1] + remove_call),
            scene_remove
        );

        // Table order is Food, Article, Tool even though the three handlers
        // are laid out as Tool, Food, Article in native code.
        for ((handler, leaf), call_offset) in handlers[2..]
            .iter()
            .copied()
            .zip(item_leaves)
            .zip([0x24, 0x28, 0x28])
        {
            assert_eq!(thumb_call_target(&rom, handler + call_offset), leaf);
        }

        // Complete public validity domains and their native fallback icon IDs.
        // The family differences are intentional and must not be inferred from
        // the FoMT-US C++ source alone.
        assert_eq!(
            &rom[item_leaves[0] + 0x06..item_leaves[0] + 0x08],
            &[0xAA, 0x29]
        );
        assert_eq!(
            &rom[item_leaves[2] + 0x06..item_leaves[2] + 0x08],
            &[0x50, 0x2A]
        );
        let (article_max, food_fallback, article_tool_fallback) = if case.name.starts_with("fomt-")
        {
            (0x5Eu8, 428u16, 457u16)
        } else {
            (0x69, 439, 468)
        };
        assert_eq!(rom[item_leaves[1] + 0x06], article_max);

        let food_actual = if case.name.starts_with("fomt-") {
            u16::from(rom[item_leaves[0] + 0x10]) << 1
        } else {
            u16::from_le_bytes(
                rom[item_leaves[0] + 0x14..item_leaves[0] + 0x16]
                    .try_into()
                    .unwrap(),
            )
        };
        assert_eq!(food_actual, food_fallback, "{} food fallback", case.name);

        for leaf in [item_leaves[1], item_leaves[2]] {
            let actual = if case.name.starts_with("fomt-") {
                u16::from_le_bytes(rom[leaf + 0x14..leaf + 0x16].try_into().unwrap())
            } else {
                u16::from(rom[leaf + 0x10]) << 1
            };
            assert_eq!(actual, article_tool_fallback, "{} item fallback", case.name);
        }
    }
}

#[test]
fn native_actor_animation_offsets_and_absent_facing_match_all_four_roms() {
    // SetAnimFacing stores an 8-bit direction; SetAnim stores a 16-bit base.
    // Both reload the fields and add base + facing before refreshing sprites.
    // This tests native evidence independently of the Mary-C bytecode compiler.
    let reference = fs::read(local_rom_path(CASES[0].rom)).unwrap();
    for (case, (dispatch, handler, leaf, resolver, facing, animation, getter_tail)) in
        CASES.iter().zip([
            (
                0x3F904, 0x401D6, 0x121EC, 0xD3914, 0x32198, 0x321B0, 0x12124,
            ),
            (
                0x3FAF0, 0x403EA, 0x122D8, 0xDB53C, 0x32554, 0x3256C, 0x1220C,
            ),
            (
                0x3F578, 0x3FE4A, 0x120BC, 0xD30CC, 0x31F2C, 0x31F44, 0x11FF4,
            ),
            (
                0x3F84C, 0x40146, 0x1217C, 0xDB050, 0x323C8, 0x323E0, 0x120B0,
            ),
        ])
    {
        let rom = fs::read(local_rom_path(case.rom)).unwrap();
        assert_eq!(
            u32::from_le_bytes(
                rom[dispatch + 0x0D * 4..dispatch + 0x0D * 4 + 4]
                    .try_into()
                    .unwrap()
            ),
            0x0800_0000 + handler as u32,
            "{} SetEntityAnim physical slot",
            case.name
        );
        assert_eq!(thumb_call_target(&rom, handler + 0x3E), leaf);
        assert_eq!(thumb_call_target(&rom, leaf + 0x0E), resolver);
        assert_eq!(thumb_call_target(&rom, leaf + 0x22), animation);
        // The wrapper compares the stored 16-bit base at +0x22 and avoids
        // restarting the animation when the requested group is unchanged.
        assert_eq!(
            &rom[leaf + 0x12..leaf + 0x22],
            &[
                0x01, 0x1C, 0x00, 0x29, 0x06, 0xD0, 0x48, 0x8C, 0xA0, 0x42, 0x03, 0xD0, 0x08, 0x1C,
                0x21, 0x1C
            ]
        );
        for (actual, expected, length, role) in [
            (facing, 0x32198, 24, "direction byte and base addition"),
            (
                animation,
                0x321B0,
                24,
                "animation halfword and facing addition",
            ),
            (
                getter_tail,
                0x12124,
                16,
                "absent entity returns zero direction",
            ),
        ] {
            assert_eq!(
                &rom[actual..actual + length],
                &reference[expected..expected + length],
                "{}: {role}",
                case.name
            );
        }
    }
}

#[test]
fn native_mfomt_opening_dog_animation_group_maps_to_fomt_resource_group() {
    fn animation_script(rom: &[u8], table: usize, pool: usize, id: usize) -> Vec<(u16, u16)> {
        let record =
            u32::from_le_bytes(rom[table + id * 4..table + id * 4 + 4].try_into().unwrap());
        let count = (record & 0xFFFF) as usize;
        let script = (record >> 16) as usize;
        (0..count)
            .map(|index| {
                let offset = pool + (script + index) * 4;
                (
                    u16::from_le_bytes(rom[offset..offset + 2].try_into().unwrap()),
                    u16::from_le_bytes(rom[offset + 2..offset + 4].try_into().unwrap()),
                )
            })
            .collect()
    }

    let fomt_us = fs::read(local_rom_path(CASES[0].rom)).unwrap();
    let mfomt_us = fs::read(local_rom_path(CASES[1].rom)).unwrap();
    let fomt_jp = fs::read(local_rom_path(CASES[2].rom)).unwrap();
    let mfomt_jp = fs::read(local_rom_path(CASES[3].rom)).unwrap();
    let tables = [0x58BA2C, 0x522C4C, 0x311B88, 0x52455C];
    let pools = [0x663208, 0x6036B8, 0x3E9364, 0x604FC8];

    for offset in 0..8 {
        let fomt_id = 942 + offset;
        let mfomt_id = 978 + offset;
        let fomt_us_script = animation_script(&fomt_us, tables[0], pools[0], fomt_id);
        let fomt_jp_script = animation_script(&fomt_jp, tables[2], pools[2], fomt_id);
        let mfomt_us_script = animation_script(&mfomt_us, tables[1], pools[1], mfomt_id);
        let mfomt_jp_script = animation_script(&mfomt_jp, tables[3], pools[3], mfomt_id);

        assert_eq!(fomt_jp_script, fomt_us_script, "FoMT animation {fomt_id}");
        assert_eq!(
            mfomt_jp_script, mfomt_us_script,
            "MFoMT animation {mfomt_id}"
        );
        assert_eq!(
            mfomt_us_script.len(),
            fomt_us_script.len(),
            "cross-family animation {fomt_id}/{mfomt_id} frame count"
        );
        for ((fomt_frame, fomt_wait), (mfomt_frame, mfomt_wait)) in
            fomt_us_script.iter().zip(&mfomt_us_script)
        {
            assert_eq!(*mfomt_frame, *fomt_frame + 9, "animation frame alignment");
            assert_eq!(mfomt_wait, fomt_wait, "animation timing alignment");
        }
    }

    // The MFoMT opening supplies base 978 after setting facing left (2), so the
    // renderer selects physical entry 980. Direct reconstruction of the matching
    // FoMT 942..945 records proves the four stationary puppy directions.
    assert_eq!(978 + 2, 980);
}

#[test]
fn native_cliff_hospital_and_collapse_animations_align_across_all_targets() {
    fn animation_script(rom: &[u8], table: usize, pool: usize, id: usize) -> Vec<(u16, u16)> {
        let record =
            u32::from_le_bytes(rom[table + id * 4..table + id * 4 + 4].try_into().unwrap());
        let count = (record & 0xFFFF) as usize;
        let script = (record >> 16) as usize;
        (0..count)
            .map(|index| {
                let offset = pool + (script + index) * 4;
                (
                    u16::from_le_bytes(rom[offset..offset + 2].try_into().unwrap()),
                    u16::from_le_bytes(rom[offset + 2..offset + 4].try_into().unwrap()),
                )
            })
            .collect()
    }

    let roms: Vec<_> = CASES
        .iter()
        .map(|case| fs::read(local_rom_path(case.rom)).unwrap())
        .collect();
    let tables = [0x58BA2C, 0x522C4C, 0x311B88, 0x52455C];
    let pools = [0x663208, 0x6036B8, 0x3E9364, 0x604FC8];
    let fomt_ids = [659, 663, 667];
    let mfomt_ids = [671, 675, 679];

    for (fomt_id, mfomt_id) in fomt_ids.into_iter().zip(mfomt_ids) {
        let fomt_us = animation_script(&roms[0], tables[0], pools[0], fomt_id);
        let mfomt_us = animation_script(&roms[1], tables[1], pools[1], mfomt_id);
        let fomt_jp = animation_script(&roms[2], tables[2], pools[2], fomt_id);
        let mfomt_jp = animation_script(&roms[3], tables[3], pools[3], mfomt_id);
        assert_eq!(fomt_jp, fomt_us, "FoMT animation {fomt_id}");
        assert_eq!(mfomt_jp, mfomt_us, "MFoMT animation {mfomt_id}");
        assert_eq!(fomt_us.len(), mfomt_us.len(), "{fomt_id}/{mfomt_id}");
        for ((fomt_frame, fomt_wait), (mfomt_frame, mfomt_wait)) in fomt_us.iter().zip(&mfomt_us) {
            assert_eq!(*mfomt_frame, *fomt_frame + 3, "{fomt_id}/{mfomt_id}");
            assert_eq!(mfomt_wait, fomt_wait, "{fomt_id}/{mfomt_id}");
        }
    }

    assert_eq!(
        animation_script(&roms[0], tables[0], pools[0], 667),
        [(1013, 100), (1044, 30), (1045, 30), (1046, 0)]
    );
}

#[test]
fn native_child_age_selected_off_map_animations_align_across_all_targets() {
    fn animation_script(rom: &[u8], table: usize, pool: usize, id: usize) -> Vec<(u16, u16)> {
        let record =
            u32::from_le_bytes(rom[table + id * 4..table + id * 4 + 4].try_into().unwrap());
        let count = (record & 0xFFFF) as usize;
        let script = (record >> 16) as usize;
        (0..count)
            .map(|index| {
                let offset = pool + (script + index) * 4;
                (
                    u16::from_le_bytes(rom[offset..offset + 2].try_into().unwrap()),
                    u16::from_le_bytes(rom[offset + 2..offset + 4].try_into().unwrap()),
                )
            })
            .collect()
    }

    let roms: Vec<_> = CASES
        .iter()
        .map(|case| fs::read(local_rom_path(case.rom)).unwrap())
        .collect();
    let tables = [0x58BA2C, 0x522C4C, 0x311B88, 0x52455C];
    let pools = [0x663208, 0x6036B8, 0x3E9364, 0x604FC8];
    let slot_12_script = [(5, 50), (6, 50)];
    for index in 0..4 {
        assert_eq!(
            animation_script(&roms[index], tables[index], pools[index], 12),
            slot_12_script,
            "{} child age-selected off-map animation 12",
            CASES[index].name
        );
    }

    let fomt_us = animation_script(&roms[0], tables[0], pools[0], 631);
    let mfomt_us = animation_script(&roms[1], tables[1], pools[1], 643);
    assert_eq!(fomt_us, [(1012, 4)]);
    assert_eq!(mfomt_us, [(1015, 4)]);
    assert_eq!(
        animation_script(&roms[2], tables[2], pools[2], 631),
        fomt_us
    );
    assert_eq!(
        animation_script(&roms[3], tables[3], pools[3], 643),
        mfomt_us
    );
}

#[test]
fn native_player_back_pain_animation_ignores_facing_on_all_targets() {
    let tables = [0x58BA2C, 0x522C4C, 0x311B88, 0x52455C];
    let pools = [0x663208, 0x6036B8, 0x3E9364, 0x604FC8];
    for ((case, table), pool) in CASES.iter().zip(tables).zip(pools) {
        let rom = fs::read(local_rom_path(case.rom)).unwrap();
        let mut scripts = Vec::new();
        for id in 258..=261 {
            let record =
                u32::from_le_bytes(rom[table + id * 4..table + id * 4 + 4].try_into().unwrap());
            let count = (record & 0xFFFF) as usize;
            let first = (record >> 16) as usize;
            scripts.push(rom[pool + first * 4..pool + (first + count) * 4].to_vec());
        }
        assert!(
            scripts.windows(2).all(|pair| pair[0] == pair[1]),
            "{} animations 258..261 must share one three-frame sequence",
            case.name
        );
        assert_eq!(
            scripts[0],
            [0x5F, 0x01, 0x0C, 0x00, 0x60, 0x01, 0x0C, 0x00, 0x61, 0x01, 0x32, 0x00,],
            "{} player back-pain frames and waits",
            case.name
        );
    }
}

#[test]
fn native_player_lower_head_animation_has_four_directional_frames_on_all_targets() {
    let tables = [0x58BA2C, 0x522C4C, 0x311B88, 0x52455C];
    let pools = [0x663208, 0x6036B8, 0x3E9364, 0x604FC8];
    for ((case, table), pool) in CASES.iter().zip(tables).zip(pools) {
        let rom = fs::read(local_rom_path(case.rom)).unwrap();
        let mut actual = Vec::new();
        for id in 510..=513 {
            let record =
                u32::from_le_bytes(rom[table + id * 4..table + id * 4 + 4].try_into().unwrap());
            assert_eq!(
                record & 0xFFFF,
                1,
                "{} animation {id} frame count",
                case.name
            );
            let script = (record >> 16) as usize;
            actual.push((
                u16::from_le_bytes(
                    rom[pool + script * 4..pool + script * 4 + 2]
                        .try_into()
                        .unwrap(),
                ),
                u16::from_le_bytes(
                    rom[pool + script * 4 + 2..pool + script * 4 + 4]
                        .try_into()
                        .unwrap(),
                ),
            ));
        }
        assert_eq!(
            actual,
            [(876, 12), (877, 12), (878, 12), (879, 12)],
            "{} player lower-head directional group",
            case.name
        );
    }
}

#[test]
fn native_entity_priority_is_not_a_seating_state_on_any_target() {
    let reference = fs::read(local_rom_path(CASES[0].rom)).unwrap();
    for (case, (setter_tail, reader)) in CASES.iter().zip([
        (0x12166, 0x327D4),
        (0x12252, 0x32B90),
        (0x12036, 0x32568),
        (0x120F6, 0x32A04),
    ]) {
        let rom = fs::read(local_rom_path(case.rom)).unwrap();
        // Writer: store a byte at actor +0x21. Reader: mask with 3,
        // replicate the pair, or use visual-controller override 0x19/0x1A.
        assert_eq!(
            &rom[setter_tail..setter_tail + 14],
            &reference[0x12166..0x12174],
            "{} setter",
            case.name
        );
        assert_eq!(
            &rom[reader..reader + 30],
            &reference[0x327D4..0x327F2],
            "{} OBJ priority mapping",
            case.name
        );
    }
}

#[test]
fn native_audio_pool_insertion_continuations_match_all_targets() {
    let reference = fs::read(local_rom_path(CASES[0].rom)).unwrap();
    for (case, offsets) in CASES.iter().zip([
        [0xD843A, 0xD8532, 0xD8614],
        [0xE0966, 0xE0A5E, 0xE0B40],
        [0xD7BF2, 0xD7CEA, 0xD7DCC],
        [0xE047A, 0xE0572, 0xE0654],
    ]) {
        let rom = fs::read(local_rom_path(case.rom)).unwrap();
        // Normal-path preparation binds +C0, +C4, +C8, +CC from the same
        // state base before these continuations insert them. This does not
        // establish that no other runtime path can later mutate the pool.
        let preparation = offsets[0] - 0x23A;
        assert_eq!(
            &rom[preparation..preparation + 0xD8],
            &reference[0xD8200..0xD82D8],
            "{} audio pool wrapper preparation",
            case.name
        );
        for (offset, (original, length)) in
            offsets
                .into_iter()
                .zip([(0xD843A, 50), (0xD8532, 48), (0xD8614, 48)])
        {
            assert_eq!(
                &rom[offset..offset + length],
                &reference[original..original + length],
                "{} pool insertion {original:#x}",
                case.name
            );
        }
    }
}

#[test]
fn native_relocation_slots_forward_zero_facing_and_family_side_effects() {
    for (case, (table, slot, call, target, visit_call, visit_target, mine_call, mine_target)) in
        CASES.iter().zip([
            (0x3F904, 0x12E, 0x70, 0x16FA4, 0x180, 0xA0A90, 0x1BA, 0xEECC),
            (0x3FAF0, 0x132, 0x6E, 0x17024, 0x280, 0xA5BF0, 0x2C6, 0xEF6C),
            (0x3F578, 0x12E, 0x70, 0x16D38, 0x180, 0xA04C8, 0x1BA, 0xEEAC),
            (0x3F84C, 0x132, 0x6E, 0x16E98, 0x280, 0xA5630, 0x2C6, 0xEF20),
        ])
    {
        let rom = fs::read(local_rom_path(case.rom)).unwrap();
        let entry = table + slot * 4;
        let handler =
            u32::from_le_bytes(rom[entry..entry + 4].try_into().unwrap()) as usize - 0x08000000;
        assert_eq!(
            &rom[handler + 0x66..handler + 0x6A],
            &[0x00, 0x20, 0x01, 0x90],
            "{} zero facing",
            case.name
        );
        let pc = handler + call;
        let hi = u16::from_le_bytes(rom[pc..pc + 2].try_into().unwrap());
        let lo = u16::from_le_bytes(rom[pc + 2..pc + 4].try_into().unwrap());
        assert_eq!(hi & 0xF800, 0xF000);
        assert_eq!(lo & 0xF800, 0xF800);
        let displacement =
            (((i32::from(hi & 0x7FF) << 12) | (i32::from(lo & 0x7FF) << 1)) << 9) >> 9;
        assert_eq!(
            pc as i32 + 4 + displacement,
            target,
            "{} relocation target",
            case.name
        );
        assert_eq!(
            thumb_call_target(&rom, target as usize + visit_call as usize),
            visit_target as usize,
            "{} guarded visit-counter updater",
            case.name
        );
        assert_eq!(
            thumb_call_target(&rom, target as usize + mine_call as usize),
            mine_target as usize,
            "{} deepest-mine-floor updater",
            case.name
        );
    }
}

#[test]
fn native_chicken_feed_queries_preserve_capacity_and_boolean_contract() {
    const CAPACITY: &[u8] = &[
        0x00, 0xB5, 0x82, 0xB0, 0x08, 0x21, 0x00, 0x91, 0x00, 0x78, 0xC0, 0x07, 0xC0, 0x0F, 0x80,
        0x00, 0x04, 0x30, 0x01, 0x90, 0x01, 0xAA, 0x6B, 0x46, 0x81, 0x42, 0x00, 0xD9, 0x13, 0x1C,
        0x18, 0x68, 0x02, 0xB0, 0x02, 0xBC, 0x08, 0x47,
    ];
    const QUERY: &[u8] = &[
        0x30, 0xB5, 0x04, 0x1C, 0x0D, 0x1C, 0xFF, 0xF7, 0xE7, 0xFF, 0x85, 0x42, 0x01, 0xD3, 0x00,
        0x20, 0x08, 0xE0, 0x21, 0x68, 0x09, 0x03, 0x09, 0x0E, 0x01, 0x20, 0xA8, 0x40, 0x01, 0x40,
        0x48, 0x42, 0x08, 0x43, 0xC0, 0x0F, 0x30, 0xBC, 0x02, 0xBC, 0x08, 0x47,
    ];

    for (case, (dispatch, slot, handler, query, capacity)) in CASES.iter().zip([
        (0x3F904, 0xB7, 0x422A8, 0xC598, 0xC570),
        (0x3FAF0, 0xBA, 0x424F4, 0xC60C, 0xC5E4),
        (0x3F578, 0xB7, 0x41F1C, 0xC578, 0xC550),
        (0x3F84C, 0xBA, 0x42250, 0xC5C0, 0xC598),
    ]) {
        let rom = fs::read(local_rom_path(case.rom)).unwrap();
        assert_eq!(
            u32::from_le_bytes(
                rom[dispatch + slot * 4..dispatch + slot * 4 + 4]
                    .try_into()
                    .unwrap()
            ),
            0x0800_0000 + handler as u32,
            "{} chicken-feed query slot",
            case.name
        );
        assert_eq!(thumb_call_target(&rom, handler + 0x2A), query);
        assert_eq!(thumb_call_target(&rom, query + 6), capacity);
        assert_eq!(
            &rom[capacity..capacity + CAPACITY.len()],
            CAPACITY,
            "{}",
            case.name
        );
        assert_eq!(&rom[query..query + QUERY.len()], QUERY, "{}", case.name);
    }
}

#[test]
fn native_chicken_feed_refresh_uses_original_index_on_all_targets() {
    let reference = fs::read(local_rom_path(CASES[0].rom)).unwrap();
    for (case, (table, thunk, refresh)) in CASES.iter().zip([
        (0xE5EC4, 0x1DD6C, 0xA6024),
        (0xEE404, 0x1DEEC, 0xAB258),
        (0xE5304, 0x1DB00, 0xA5A5C),
        (0xEDF14, 0x1DD60, 0xAAC98),
    ]) {
        let rom = fs::read(local_rom_path(case.rom)).unwrap();
        assert_eq!(
            u32::from_le_bytes(rom[table + 0xA0..table + 0xA4].try_into().unwrap()),
            thunk as u32 + 0x08000001
        );
        for offset in 0..0x58 {
            if (0x28..0x30).contains(&offset) || (0x50..0x58).contains(&offset) {
                continue; // Four relocated descriptor/coordinate pointers.
            }
            assert_eq!(
                rom[refresh + offset],
                reference[0xA6024 + offset],
                "{} refresh+{offset:X}",
                case.name
            );
        }
        let coords = u32::from_le_bytes(rom[refresh + 0x2C..refresh + 0x30].try_into().unwrap());
        let coords = (coords - 0x08000000) as usize;
        assert_eq!(&rom[coords..coords + 8], &[16, 19, 22, 25, 28, 31, 34, 37]);
    }
}

#[test]
fn native_barn_feed_queries_return_guarded_booleans() {
    let reference = fs::read(local_rom_path(CASES[0].rom)).unwrap();
    for (case, (table, slot, handler, ordinary, pregnancy)) in CASES.iter().zip([
        (0x3F904, 0xBE, 0x42322, 0xCEA8, 0xCED4),
        (0x3FAF0, 0xC1, 0x4256E, 0xCF1C, 0xCF48),
        (0x3F578, 0xBE, 0x41F96, 0xCE88, 0xCEB4),
        (0x3F84C, 0xC1, 0x422CA, 0xCED0, 0xCEFC),
    ]) {
        let rom = fs::read(local_rom_path(case.rom)).unwrap();
        let entry = table + slot * 4;
        assert_eq!(
            u32::from_le_bytes(rom[entry..entry + 4].try_into().unwrap()),
            handler as u32 + 0x08000000
        );
        for (offset, target, base) in [(0x30, pregnancy, 0xCED4), (0x4A, ordinary, 0xCEA8)] {
            let pc = handler + offset;
            let hi = u16::from_le_bytes(rom[pc..pc + 2].try_into().unwrap());
            let lo = u16::from_le_bytes(rom[pc + 2..pc + 4].try_into().unwrap());
            assert_eq!(hi & 0xF800, 0xF000);
            assert_eq!(lo & 0xF800, 0xF800);
            let raw = (i32::from(hi & 0x7FF) << 12) | (i32::from(lo & 0x7FF) << 1);
            assert_eq!(pc as i32 + 4 + ((raw << 9) >> 9), target as i32);
            assert_eq!(&rom[target..target + 44], &reference[base..base + 44]);
        }
    }
}

#[test]
fn native_barn_birth_readiness_and_link_clear_match_all_targets() {
    let reference = fs::read(local_rom_path(CASES[0].rom)).unwrap();
    for (case, (table, slot, handler, birth_handler, ready, birth)) in CASES.iter().zip([
        (0x3F904, 0xC2, 0x44C20, 0x44C54, 0xD158, 0xD8A8),
        (0x3FAF0, 0xC5, 0x44E78, 0x44EAE, 0xD1CC, 0xD91C),
        (0x3F578, 0xC2, 0x44894, 0x448C8, 0xD138, 0xD888),
        (0x3F84C, 0xC5, 0x44BD4, 0x44C0A, 0xD180, 0xD8D0),
    ]) {
        let rom = fs::read(local_rom_path(case.rom)).unwrap();
        let target = |pc: usize| {
            let hi = u16::from_le_bytes(rom[pc..pc + 2].try_into().unwrap());
            let lo = u16::from_le_bytes(rom[pc + 2..pc + 4].try_into().unwrap());
            assert_eq!(hi & 0xF800, 0xF000);
            assert_eq!(lo & 0xF800, 0xF800);
            let delta = (((i32::from(hi & 0x7FF) << 12) | (i32::from(lo & 0x7FF) << 1)) << 9) >> 9;
            (pc as i32 + 4 + delta) as usize
        };
        for (id, wrapper, callee) in [(slot, handler, ready), (slot + 1, birth_handler, birth)] {
            let entry = table + id * 4;
            assert_eq!(
                u32::from_le_bytes(rom[entry..entry + 4].try_into().unwrap()),
                0x08000000 + wrapper as u32
            );
            assert_eq!(target(wrapper + 0x2A), callee);
        }
        // Complete readiness body: signed linked index and capacity guards,
        // healthy-days > 20 OR total-days > 29. Only BL relocations differ.
        for offset in 0..0x5A {
            if [6, 0x1C, 0x2E, 0x3A, 0x44]
                .iter()
                .any(|start| (*start..*start + 4).contains(&offset))
            {
                continue;
            }
            assert_eq!(
                rom[ready + offset],
                reference[0xD158 + offset],
                "{} readiness+{offset:X}",
                case.name
            );
        }
        // Follow both getter edges instead of trusting equal compare operands.
        // Each getter checks pregnancy before extracting its five-bit counter.
        for (offset, base, length) in [(0x3A, 0x9B8D4, 28), (0x44, 0x9B8BC, 24)] {
            let getter = target(ready + offset);
            assert_eq!(
                &rom[getter..getter + length],
                &reference[base..base + length],
                "{} pregnancy counter getter",
                case.name
            );
        }
        assert_eq!(target(birth + 0x10), ready);
        // After readiness succeeds, clear the linked index before constructing
        // the newborn. This test deliberately does not claim insertion is atomic.
        assert_eq!(&rom[birth + 0x1C..birth + 0x2C], &reference[0xD8C4..0xD8D4]);
        // FoMT-JP alone has the earlier unconditional RNG call layout;
        // MFoMT-JP matches the US-family layout, not FoMT-JP.
        let jp = case.name == "fomt-jp";
        for rng_offset in if jp { [0xD0, 0x116] } else { [0xDC, 0x124] } {
            let rng = target(birth + rng_offset);
            assert_eq!(&rom[rng..rng + 32], &reference[0xD11E4..0xD1204]);
        }
        if jp {
            // Actual FoMT-JP sheep path passes NULL to the affection getter.
            // Its byte result is halved, used as the PRNG remainder divisor,
            // then written as newborn affection. Preserve this shipped flow,
            // not a guessed replacement parent pointer or fixed outcome.
            assert_eq!(&rom[birth + 0x11C..birth + 0x11E], &[0x00, 0x20]);
            let getter = target(birth + 0x11E);
            assert_eq!(
                &rom[getter..getter + 8],
                &[0x80, 0x69, 0xC0, 0x02, 0x00, 0x0E, 0x70, 0x47]
            );
            assert_eq!(
                &rom[birth + 0x122..birth + 0x128],
                &[0x01, 0x1C, 0x49, 0x08, 0x20, 0x1C]
            );
            let remainder = target(birth + 0x128);
            assert_eq!(remainder, 0xD0706);
            // Positive operands take the BIOS Div path and return its remainder;
            // a zero divisor takes the local zero-return path.
            assert_eq!(
                &rom[remainder..remainder + 14],
                &[
                    0x03, 0x1C, 0x0B, 0x43, 0xBD, 0xD5, 0x00, 0x29, 0xB3, 0xD0, 0x01, 0x23, 0x88,
                    0x42
                ]
            );
            let affection_setter = target(birth + 0x130);
            assert_eq!(affection_setter, 0x9ACE0);
        }
        let regional_reference = if jp {
            fs::read(local_rom_path(CASES[2].rom)).unwrap()
        } else {
            reference.clone()
        };
        let regional_base = if jp { 0xD888 } else { 0xD8A8 };
        // Full birth body within each region, including return paths. Mask only
        // verified Thumb BL pairs; preserve all conditional and local branches.
        let mut offset = 0;
        while offset < if jp { 0x152 } else { 0x154 } {
            let pc = regional_base + offset;
            let hi = u16::from_le_bytes(regional_reference[pc..pc + 2].try_into().unwrap());
            let lo = u16::from_le_bytes(regional_reference[pc + 2..pc + 4].try_into().unwrap());
            if hi & 0xF800 == 0xF000 && lo & 0xF800 == 0xF800 {
                let _ = target(birth + offset);
                offset += 4;
            } else {
                assert_eq!(
                    &rom[birth + offset..birth + offset + 2],
                    &regional_reference[pc..pc + 2],
                    "{} birth+{offset:X}",
                    case.name
                );
                offset += 2;
            }
        }
        for reset_offset in [0xBE, if jp { 0x106 } else { 0x108 }] {
            let reset = target(birth + reset_offset);
            assert_eq!(&rom[reset..reset + 36], &reference[0x9B91C..0x9B940]);
        }
        for (call_offset, base) in [(0xF2, 0xD448), (if jp { 0x138 } else { 0x13A }, 0xD484)] {
            let insert = target(birth + call_offset);
            assert_eq!(&rom[insert..insert + 60], &reference[base..base + 60]);
            let search = target(insert + 0x28);
            assert_eq!(&rom[search..search + 60], &reference[0xCF88..0xCFC4]);
            let capacity = target(search + 6);
            assert_eq!(&rom[capacity..capacity + 40], &reference[0xCE74..0xCE9C]);
            let is_empty = target(search + 0x1C);
            assert_eq!(&rom[is_empty..is_empty + 12], &reference[0xDA08..0xDA14]);
        }
    }
}

#[test]
fn native_egg_hatch_clears_incubator_before_insertion_on_all_targets() {
    let reference = fs::read(local_rom_path(CASES[0].rom)).unwrap();
    for (case, (table, slot, wrapper, hatch_wrapper, ready, hatch)) in CASES.iter().zip([
        (0x3F904, 0xBC, 0x44B70, 0x44BA4, 0xC720, 0xCBCC),
        (0x3FAF0, 0xBF, 0x44DC6, 0x44DFC, 0xC794, 0xCC40),
        (0x3F578, 0xBC, 0x447E4, 0x44818, 0xC700, 0xCBAC),
        (0x3F84C, 0xBF, 0x44B22, 0x44B58, 0xC748, 0xCBF4),
    ]) {
        let rom = fs::read(local_rom_path(case.rom)).unwrap();
        let target = |pc: usize| {
            let hi = u16::from_le_bytes(rom[pc..pc + 2].try_into().unwrap());
            let lo = u16::from_le_bytes(rom[pc + 2..pc + 4].try_into().unwrap());
            assert_eq!(hi & 0xF800, 0xF000);
            assert_eq!(lo & 0xF800, 0xF800);
            let delta = (((i32::from(hi & 0x7FF) << 12) | (i32::from(lo & 0x7FF) << 1)) << 9) >> 9;
            (pc as i32 + 4 + delta) as usize
        };
        for (id, handler, callee) in [(slot, wrapper, ready), (slot + 1, hatch_wrapper, hatch)] {
            let entry = table + id * 4;
            assert_eq!(
                u32::from_le_bytes(rom[entry..entry + 4].try_into().unwrap()),
                0x08000000 + handler as u32
            );
            assert_eq!(target(handler + 0x2A), callee);
        }
        assert_eq!(target(hatch + 0xE), ready);
        let occupied_entry = table + (slot - 2) * 4;
        let occupied_wrapper =
            (u32::from_le_bytes(rom[occupied_entry..occupied_entry + 4].try_into().unwrap())
                - 0x08000000) as usize;
        let occupied_query = target(occupied_wrapper + 0x2A);
        assert_eq!(
            &rom[occupied_query..occupied_query + 38],
            &reference[0xC678..0xC69E]
        );
        let occupied_capacity = target(occupied_query + 6);
        assert_eq!(
            &rom[occupied_capacity..occupied_capacity + 22],
            &reference[0xC660..0xC676]
        );
        let occupied_bit = target(occupied_query + 0x1C);
        assert_eq!(
            &rom[occupied_bit..occupied_bit + 8],
            &reference[0xCD38..0xCD40]
        );
        let chicken = target(hatch + 0xAE);
        let livestock = target(chicken + 4);
        let animal = target(livestock + 6);
        for (address, base, length, call_offset) in [
            (chicken, 0x9BC8C, 30, 4),
            (livestock, 0x9B4A4, 80, 6),
            (animal, 0x9B1A4, 80, 6),
        ] {
            for offset in 0..length {
                if (call_offset..call_offset + 4).contains(&offset) {
                    continue;
                }
                assert_eq!(
                    rom[address + offset],
                    reference[base + offset],
                    "{} chick constructor+{offset:X}",
                    case.name
                );
            }
        }
        // Constructor clears exactly the affection field at bits13..20.
        assert_eq!(
            u32::from_le_bytes(rom[animal + 0x4C..animal + 0x50].try_into().unwrap()),
            !(0xFFu32 << 13)
        );
        // The newborn receives a numeric affection increment, not a status ID.
        // Verify the actual RNG, signed remainder and clamped addition callees.
        for (offset, base, length) in [(0xB2, 0xD11E4, 32), (0xB8, 0xD0ED0, 6), (0xC0, 0x9B2A8, 52)]
        {
            let callee = target(hatch + offset);
            assert_eq!(
                &rom[callee..callee + length],
                &reference[base..base + length],
                "{} hatch affection helper",
                case.name
            );
        }
        assert_eq!(&rom[hatch + 0xB6..hatch + 0xB8], &[0x0A, 0x21]);
        let start_wrapper = match case.name {
            "fomt-us" => 0x43798,
            "mfomt-us" => 0x439FC,
            "fomt-jp" => 0x4340C,
            "mfomt-jp" => 0x43758,
            _ => unreachable!(),
        };
        let entry = table + (slot - 3) * 4;
        assert_eq!(
            u32::from_le_bytes(rom[entry..entry + 4].try_into().unwrap()),
            0x08000000 + start_wrapper as u32
        );
        let start = target(start_wrapper + 0x26);
        let setter = target(start + 0x14);
        assert_eq!(&rom[setter..setter + 34], &reference[0xC994..0xC9B6]);
        let leaf = target(setter + 0x18);
        assert_eq!(&rom[leaf..leaf + 14], &reference[0xCD78..0xCD86]);
        let tick_call = match case.name {
            "fomt-us" => 0xCB60,
            "mfomt-us" => 0xCBD4,
            "fomt-jp" => 0xCB40,
            "mfomt-jp" => 0xCB88,
            _ => unreachable!(),
        };
        // Daily coop loop checks occupancy before ticking. The tick itself
        // decreases the two-bit countdown only when nonzero, without wrapping.
        assert_eq!(
            &rom[tick_call - 20..tick_call + 12],
            &reference[0xCB4C..0xCB6C]
        );
        let tick = target(tick_call);
        assert_eq!(&rom[tick..tick + 40], &reference[0xCD88..0xCDB0]);
        let occupied = target(tick_call - 12);
        assert_eq!(
            &rom[occupied..occupied + 8],
            &[0x00, 0x68, 0x40, 0x07, 0xC0, 0x0F, 0x70, 0x47]
        );
        // The context method forwards the original index unconditionally to
        // virtual +AC after the guarded storage call, including invalid indices.
        assert_eq!(
            &rom[start + 0x18..start + 0x24],
            &reference[0x146E4..0x146F0]
        );
        let runtime_table = match case.name {
            "fomt-us" => 0xE5EC4,
            "mfomt-us" => 0xEE404,
            "fomt-jp" => 0xE5304,
            "mfomt-jp" => 0xEDF14,
            _ => unreachable!(),
        };
        let thunk = (u32::from_le_bytes(
            rom[runtime_table + 0xAC..runtime_table + 0xB0]
                .try_into()
                .unwrap(),
        ) - 0x08000001) as usize;
        assert_eq!(&rom[thunk..thunk + 4], &[0x00, 0xB5, 0x40, 0x68]);
        let refresh = target(thunk + 4);
        for offset in 0..0x68 {
            if [0x28, 0x3C, 0x64]
                .iter()
                .any(|p| (*p..*p + 4).contains(&offset))
            {
                continue; // Three tile-patch descriptor pointer relocations.
            }
            assert_eq!(
                rom[refresh + offset],
                reference[0xA63B8 + offset],
                "{} incubator refresh+{offset:X}",
                case.name
            );
        }
        for (address, base, length) in [(ready, 0xC720, 0x26), (hatch, 0xCBCC, 0xDA)] {
            let mut offset = 0;
            while offset < length {
                let pc = base + offset;
                let hi = u16::from_le_bytes(reference[pc..pc + 2].try_into().unwrap());
                let lo = u16::from_le_bytes(reference[pc + 2..pc + 4].try_into().unwrap());
                if hi & 0xF800 == 0xF000 && lo & 0xF800 == 0xF800 {
                    let _ = target(address + offset);
                    offset += 4;
                } else {
                    assert_eq!(
                        &rom[address + offset..address + offset + 2],
                        &reference[pc..pc + 2],
                        "{} egg routine+{offset:X}",
                        case.name
                    );
                    offset += 2;
                }
            }
        }
        for (pc, base, length) in [
            (ready + 0x1C, 0xCD58, 32),
            (hatch + 0x28, 0xCDB0, 12),
            (hatch + 0xC8, 0xC8BC, 60),
        ] {
            let callee = target(pc);
            assert_eq!(
                &rom[callee..callee + length],
                &reference[base..base + length]
            );
        }
    }
}

#[test]
fn native_romance_event_count_updates_match_four_targets() {
    let reference = fs::read(local_rom_path(CASES[0].rom)).unwrap();
    let female_us = fs::read(local_rom_path(CASES[1].rom)).unwrap();
    let female_jp = fs::read(local_rom_path(CASES[3].rom)).unwrap();
    let mut relocated_calls = 0;
    let mut offset = 0;
    // Whole MF helper including epilogue and literal pools. This guards its
    // control flow and storage offsets, not semantic equivalence of every callee.
    while offset < 0x2D4 {
        let us = u16::from_le_bytes(
            female_us[0x45D50 + offset..0x45D52 + offset]
                .try_into()
                .unwrap(),
        );
        let jp = u16::from_le_bytes(
            female_jp[0x45AAC + offset..0x45AAE + offset]
                .try_into()
                .unwrap(),
        );
        if us & 0xF800 == 0xF000 && jp & 0xF800 == 0xF000 {
            let us_lo = u16::from_le_bytes(
                female_us[0x45D52 + offset..0x45D54 + offset]
                    .try_into()
                    .unwrap(),
            );
            let jp_lo = u16::from_le_bytes(
                female_jp[0x45AAE + offset..0x45AB0 + offset]
                    .try_into()
                    .unwrap(),
            );
            assert_eq!(us_lo & 0xF800, 0xF800);
            assert_eq!(jp_lo & 0xF800, 0xF800);
            relocated_calls += 1;
            offset += 4;
        } else {
            assert_eq!(us, jp, "MF regional difference at +{offset:X}");
            offset += 2;
        }
    }
    assert_eq!(relocated_calls, 17);
    for (case, setter, player, rival) in [
        (&CASES[0], 0x45638, 0x9E4F8, 0x9E520),
        (&CASES[1], 0x45D50, 0xA368C, 0xA36B4),
        (&CASES[2], 0x452AC, 0x9DF30, 0x9DF58),
        (&CASES[3], 0x45AAC, 0xA30CC, 0xA30F4),
    ] {
        let rom = fs::read(local_rom_path(case.rom)).unwrap();
        for (offset, expected) in [(0x9C, player), (0xA4, rival)] {
            let pc = setter + offset;
            let hi = u16::from_le_bytes(rom[pc..pc + 2].try_into().unwrap());
            let lo = u16::from_le_bytes(rom[pc + 2..pc + 4].try_into().unwrap());
            assert_eq!(hi & 0xF800, 0xF000);
            assert_eq!(lo & 0xF800, 0xF800);
            let delta = (((i32::from(hi & 0x7FF) << 12) | (i32::from(lo & 0x7FF) << 1)) << 9) >> 9;
            assert_eq!((pc as i32 + 4 + delta) as usize, expected);
        }
        // Complete leaf bodies, including the rival field's preservation mask.
        // Source: Bachelorette::PlayerEventUpdate / RivalEventUpdate.
        assert_eq!(&rom[player..player + 40], &reference[0x9E4F8..0x9E520]);
        assert_eq!(&rom[rival..rival + 48], &reference[0x9E520..0x9E550]);
        assert_eq!(&rom[player + 10..player + 14], &[5, 0x28, 10, 0xD8]);
        assert_eq!(&rom[rival + 10..rival + 14], &[4, 0x28, 12, 0xD8]);
        assert_eq!(&rom[rival + 44..rival + 48], &[0x3F, 0xFE, 0xFF, 0xFF]);
    }
}

#[test]
fn native_wedding_exclusion_tables_preserve_fomt_jp_differences() {
    let reference = fs::read(local_rom_path(CASES[0].rom)).unwrap();
    let common: &[u8] = &[
        0, 13, 16, 17, 20, 21, 29, 30, 35, 36, 48, 49, 53, 61, 62, 67, 68, 72, 76, 77, 79, 80, 89,
        103, 113, 114, 119,
    ];
    for (case, routine, table, count) in [
        (&CASES[0], 0xADBB4, 0x1074F6, 27),
        (&CASES[1], 0xB2E00, 0x1102CA, 27),
        (&CASES[2], 0xAD5EC, 0x10703E, 25),
        (&CASES[3], 0xB2840, 0x11061A, 27),
    ] {
        let rom = fs::read(local_rom_path(case.rom)).unwrap();
        let pc = routine + 40;
        let hi = u16::from_le_bytes(rom[pc..pc + 2].try_into().unwrap());
        let lo = u16::from_le_bytes(rom[pc + 2..pc + 4].try_into().unwrap());
        assert_eq!(hi & 0xF800, 0xF000);
        assert_eq!(lo & 0xF800, 0xF800);
        let delta = (((i32::from(hi & 0x7FF) << 12) | (i32::from(lo & 0x7FF) << 1)) << 9) >> 9;
        let search = (pc as i32 + 4 + delta) as usize;
        // Complete lower_bound helper: first table byte >= requested date.
        assert_eq!(&rom[search..search + 52], &reference[0xE0E80..0xE0EB4]);
        // Caller rejects end and tests requested date >= found byte, so the
        // combined predicate is equality, not a forbidden interval.
        assert_eq!(
            &rom[routine + 44..routine + 72],
            &reference[0xADBE0..0xADBFC]
        );
        let ldr = u16::from_le_bytes(rom[routine + 24..routine + 26].try_into().unwrap());
        assert_eq!(ldr & 0xFF00, 0x4800);
        let pool = ((routine + 28) & !3) + usize::from(ldr & 255) * 4;
        assert_eq!(
            u32::from_le_bytes(rom[pool..pool + 4].try_into().unwrap()) as usize,
            table + 0x08000000
        );
        assert_eq!(&rom[routine + 28..routine + 30], &[count as u8, 0x34]);
        let expected: Vec<u8> = common
            .iter()
            .copied()
            .filter(|v| count != 25 || ![89, 114].contains(v))
            .collect();
        assert_eq!(&rom[table..table + count], expected);
    }
}

#[test]
fn native_cooking_theme_getter_and_rating_share_the_same_field() {
    for (case, table, id, getter, dispatch, slot, handler, rating, field) in [
        (
            &CASES[0], 0x45934, 397, 0x48188, 0x3F904, 0x120, 0x4315A, 0x43198, 0x219B,
        ),
        (
            &CASES[1], 0x46050, 441, 0x48E2C, 0x3FAF0, 0x124, 0x433A8, 0x433E4, 0x21BC,
        ),
        (
            &CASES[2], 0x4575C, 397, 0x47FB0, 0x3F578, 0x120, 0x42DCE, 0x42E0C, 0x219B,
        ),
        (
            &CASES[3], 0x45DAC, 441, 0x48B88, 0x3F84C, 0x124, 0x43104, 0x43140, 0x21BC,
        ),
    ] {
        let rom = fs::read(local_rom_path(case.rom)).unwrap();
        let word = |p| u32::from_le_bytes(rom[p..p + 4].try_into().unwrap());
        assert_eq!(word(table + id * 4), getter as u32 + 0x08000000);
        assert_eq!(
            word(dispatch + slot * 4),
            handler as u32 + 0x08000000,
            "{} cooking-rating physical callable",
            case.name
        );
        // Empty/non-food hands take the ineligible path; valid food reaches
        // the rating block below. This binds the public physical slot to the
        // same implementation whose category/threshold behavior is tested.
        assert!(handler < rating);
        assert_eq!(
            &rom[getter..getter + 18],
            &[
                0xD4, 0x21, 0x89, 0x00, 0x50, 0x18, 0x00, 0x68, 0x03, 0x4A, 0x80, 0x18, 0x00, 0x78,
                0x00, 0x07, 0x40, 0x0F,
            ]
        );
        assert_eq!(word(getter + 24), field);
        // Rating uses the same field and bit extraction; category >4
        // bypasses the five-entry dispatch table and reaches rejection.
        assert_eq!(
            &rom[rating + 28..rating + 46],
            &[
                0x07, 0x4B, 0xF8, 0x18, 0x00, 0x78, 0x00, 0x07, 0x47, 0x0F, 0x00, 0x24, 0x04, 0x2F,
                0x00, 0xD9, 0x0E, 0xE1,
            ]
        );
        assert_eq!(word(rating + 60), field);
        assert_eq!(
            &rom[rating + 0x24C..rating + 0x250],
            &[0x00, 0x2C, 0x27, 0xD0]
        );
    }
}

#[test]
fn native_cooking_category_admission_matches_four_targets() {
    let mut reference = None;
    for (case, entry, get_id) in [
        (&CASES[0], 0x431F0, 0xDCB4),
        (&CASES[1], 0x4343C, 0xDD28),
        (&CASES[2], 0x42E64, 0xDC94),
        (&CASES[3], 0x43198, 0xDCDC),
    ] {
        let rom = fs::read(local_rom_path(case.rom)).unwrap();
        let mut body = rom[entry..entry + 0x1F8].to_vec();
        for offset in [2, 0x4C, 0x17E, 0x1AA, 0x1BE] {
            let hi = u16::from_le_bytes(body[offset..offset + 2].try_into().unwrap());
            let lo = u16::from_le_bytes(body[offset + 2..offset + 4].try_into().unwrap());
            assert_eq!(hi & 0xF800, 0xF000);
            assert_eq!(lo & 0xF800, 0xF800);
            let delta = (((i32::from(hi & 0x7FF) << 12) | (i32::from(lo & 0x7FF) << 1)) << 9) >> 9;
            assert_eq!((entry + offset) as i32 + 4 + delta, get_id);
            body[offset..offset + 4].fill(0);
        }
        // The sole internal table pointer and all 69 dispatch entries.
        for offset in (0x64..0x17C).step_by(4) {
            let pointer = u32::from_le_bytes(body[offset..offset + 4].try_into().unwrap());
            let relative = pointer - 0x08000000 - entry as u32;
            if offset == 0x64 {
                assert_eq!(relative, 0x68);
            } else {
                let id = 0x65 + (offset - 0x68) / 4;
                let accepted = [
                    0x65, 0x68, 0x69, 0x71, 0x73, 0x75, 0x8B, 0x8C, 0x8E, 0x8F, 0x90, 0x94, 0x96,
                    0x97, 0x98, 0x9D, 0xA7, 0xA9,
                ]
                .contains(&id);
                assert_eq!(relative, if accepted { 0x1F8 } else { 0x1F4 });
            }
            body[offset..offset + 4].copy_from_slice(&relative.to_le_bytes());
        }
        if let Some(expected) = &reference {
            assert_eq!(&body, expected, "{}", case.name);
        } else {
            reference = Some(body);
        }
    }
}

#[test]
fn native_food_bonus_addition_saturates_signed_bytes_in_four_targets() {
    // FoMT-US Food::AddBonuses: valid IDs 0..=0xAA only; each signed
    // bonus sum saturates independently to -128..=127 before STRB.
    // Full leaf includes both clamps, the invalid-ID branch and the return.
    let expected = [
        0x10, 0xB5, 0x03, 0x1C, 0x09, 0x06, 0x0C, 0x0E, 0x12, 0x06, 0x12, 0x0E, 0x18, 0x78, 0x00,
        0x21, 0xAA, 0x28, 0x00, 0xD8, 0x01, 0x21, 0x00, 0x29, 0x1D, 0xD0, 0x01, 0x21, 0x59, 0x56,
        0x20, 0x06, 0x00, 0x16, 0x08, 0x18, 0x80, 0x21, 0x49, 0x42, 0x88, 0x42, 0x01, 0xDA, 0x08,
        0x1C, 0x02, 0xE0, 0x7F, 0x28, 0x00, 0xDD, 0x7F, 0x20, 0x58, 0x70, 0x02, 0x21, 0x59, 0x56,
        0x10, 0x06, 0x00, 0x16, 0x08, 0x18, 0x80, 0x21, 0x49, 0x42, 0x88, 0x42, 0x01, 0xDA, 0x08,
        0x1C, 0x02, 0xE0, 0x7F, 0x28, 0x00, 0xDD, 0x7F, 0x20, 0x98, 0x70, 0x10, 0xBC, 0x01, 0xBC,
        0x00, 0x47,
    ];
    for (case, entry) in [
        (&CASES[0], 0xDE0C),
        (&CASES[1], 0xDE80),
        (&CASES[2], 0xDDEC),
        (&CASES[3], 0xDE34),
    ] {
        let rom = fs::read(local_rom_path(case.rom)).unwrap();
        assert_eq!(
            &rom[entry..entry + expected.len()],
            &expected,
            "{}",
            case.name
        );
    }
}

#[test]
fn native_cooking_rating_threshold_dispatch_matches_four_targets() {
    let mut reference = None;
    let vanilla_us = fs::read(local_rom_path(CASES[0].rom)).unwrap();
    let charmap = Charmap::parse(&fs::read_to_string("charmap_jp.txt").unwrap()).unwrap();
    let thresholds = [
        100, 80, 50, 30, 80, 60, 30, 10, 80, 50, 20, 10, 100, 70, 40, 20, 80, 60, 30, 20,
    ];
    for (case, entry, table) in [
        (&CASES[0], 0x433E8, 0xF9EC5),
        (&CASES[1], 0x43634, 0x102B09),
        (&CASES[2], 0x4305C, 0xF9721),
        (&CASES[3], 0x43390, 0x102AA9),
    ] {
        let rom = fs::read(local_rom_path(case.rom)).unwrap();
        let mut body = rom[entry..entry + 0x52].to_vec();
        for offset in [2, 10] {
            assert_eq!(
                u16::from_le_bytes(body[offset..offset + 2].try_into().unwrap()) & 0xF800,
                0xF000
            );
            assert_eq!(
                u16::from_le_bytes(body[offset + 2..offset + 4].try_into().unwrap()) & 0xF800,
                0xF800
            );
            let hi = u16::from_le_bytes(body[offset..offset + 2].try_into().unwrap());
            let lo = u16::from_le_bytes(body[offset + 2..offset + 4].try_into().unwrap());
            let delta = (((i32::from(hi & 0x7FF) << 12) | (i32::from(lo & 0x7FF) << 1)) << 9) >> 9;
            let leaf = ((entry + offset) as i32 + 4 + delta) as usize;
            let (reference_leaf, code_length) = if offset == 2 {
                (0xDD08, 48)
            } else {
                (0xDD3C, 44)
            };
            // Entire executable leaf, followed by its four-byte food-table pointer.
            assert_eq!(
                &rom[leaf..leaf + code_length],
                &vanilla_us[reference_leaf..reference_leaf + code_length]
            );
            let food_table = u32::from_le_bytes(
                rom[leaf + code_length..leaf + code_length + 4]
                    .try_into()
                    .unwrap(),
            ) as usize
                - 0x08000000;
            if offset == 2 {
                let name_pointer = |id: usize| {
                    u32::from_le_bytes(
                        rom[food_table + id * 16..food_table + id * 16 + 4]
                            .try_into()
                            .unwrap(),
                    )
                };
                let description_pointer = |id: usize| {
                    u32::from_le_bytes(
                        rom[food_table + id * 16 + 12..food_table + id * 16 + 16]
                            .try_into()
                            .unwrap(),
                    )
                };
                if case.target.ends_with("_US") {
                    assert_eq!(name_pointer(0x86), name_pointer(0x89), "{}", case.name);
                    assert_eq!(
                        description_pointer(0x86),
                        description_pointer(0x89),
                        "{}",
                        case.name
                    );
                } else {
                    assert_ne!(name_pointer(0x86), name_pointer(0x89), "{}", case.name);
                    assert_ne!(
                        description_pointer(0x86),
                        description_pointer(0x89),
                        "{}",
                        case.name
                    );
                    for (id, expected_text) in [(0x86, "焼きうどん"), (0x89, "焼きそば")] {
                        let expected = charmap.encode_text(expected_text).unwrap();
                        let name = (name_pointer(id) - 0x0800_0000) as usize;
                        assert_eq!(
                            &rom[name..name + expected.len()],
                            expected.as_slice(),
                            "{} food 0x{id:02X} name",
                            case.name
                        );
                        assert_eq!(rom[name + expected.len()], 0, "{}", case.name);
                    }
                }
            }
            for id in 0..171 {
                assert_eq!(
                    &rom[food_table + id * 16 + 5..food_table + id * 16 + 7],
                    &vanilla_us[0xEDCD8 + id * 16 + 5..0xEDCD8 + id * 16 + 7]
                );
            }
            body[offset..offset + 4].fill(0);
        }
        assert_eq!(&body[16..18], &[3, 0x4A]);
        assert_eq!(
            u32::from_le_bytes(body[0x20..0x24].try_into().unwrap()) as usize,
            table + 0x08000000
        );
        body[0x20..0x24].fill(0);
        if let Some(expected) = &reference {
            assert_eq!(&body, expected);
        } else {
            reference = Some(body);
        }
        assert_eq!(&rom[table..table + 20], &thresholds);
    }
    // The matched block uses signed BLE to advance to each lower rating:
    // threshold equality must not be treated as a strict-greater success.
    for row in thresholds.chunks_exact(4) {
        for (index, threshold) in row.iter().enumerate() {
            let classify = |score: i32| row.iter().position(|t| score > i32::from(*t)).unwrap_or(4);
            assert_eq!(classify(i32::from(*threshold)), index + 1);
            assert_eq!(classify(i32::from(*threshold) + 1), index);
        }
    }
}

#[test]
fn native_wool_p_and_x_article_records_preserve_target_specific_text_pointers() {
    let charmap = Charmap::parse(&fs::read_to_string("charmap_jp.txt").unwrap()).unwrap();

    for (case, (table, expected_icons)) in CASES.iter().zip([
        (0xEFED4usize, [487u16, 489u16]),
        (0xF8648, [498, 500]),
        (0xEF738, [487, 489]),
        (0xF8624, [498, 500]),
    ]) {
        let rom = fs::read(local_rom_path(case.rom)).unwrap();
        let record = |id: usize| table + id * 12;
        let pointer = |offset: usize| {
            let address = u32::from_le_bytes(rom[offset..offset + 4].try_into().unwrap());
            assert!(
                (0x0800_0000..0x0A00_0000).contains(&address),
                "{} article text pointer 0x{address:08X}",
                case.name
            );
            (address - 0x0800_0000) as usize
        };

        let p = record(0x09);
        let x = record(0x0A);
        assert_eq!(
            [
                u16::from_le_bytes(rom[p + 4..p + 6].try_into().unwrap()),
                u16::from_le_bytes(rom[x + 4..x + 6].try_into().unwrap()),
            ],
            expected_icons,
            "{} Wool P/X icon identity",
            case.name
        );

        let p_name = pointer(p);
        let x_name = pointer(x);
        let p_description = pointer(p + 8);
        let x_description = pointer(x + 8);
        if case.target == "MARY_FOMT_US" {
            assert_eq!(p_name, x_name, "{} localized name-pointer bug", case.name);
            assert_eq!(
                p_description, x_description,
                "{} localized description-pointer bug",
                case.name
            );
            assert_eq!(&rom[x_name..x_name + 9], b"Wool (P)\0");
        } else {
            assert_ne!(p_name, x_name, "{} distinct Wool X name", case.name);
            assert_ne!(
                p_description, x_description,
                "{} distinct Wool X description",
                case.name
            );
            if case.target == "MARY_MFOMT_US" {
                assert_eq!(&rom[p_name..p_name + 9], b"Wool (P)\0");
                assert_eq!(&rom[x_name..x_name + 9], b"Wool (X)\0");
            } else {
                for (name, expected_text) in [(p_name, "羊毛Ｐサイズ"), (x_name, "羊毛Ｘサイズ")]
                {
                    let expected = charmap.encode_text(expected_text).unwrap();
                    assert_eq!(
                        &rom[name..name + expected.len()],
                        expected.as_slice(),
                        "{} {expected_text}",
                        case.name
                    );
                    assert_eq!(rom[name + expected.len()], 0, "{}", case.name);
                }
            }
        }
    }
}

#[test]
fn native_final_player_event_timer_accessors_reproduce_rival_timer_bug() {
    const BUGGED_GETTER: [u8; 8] = [0xC0, 0x7D, 0x00, 0x07, 0x40, 0x0F, 0x70, 0x47];

    for (case, (table, player_handlers, rival_handlers, player_getter, rival_getter)) in
        CASES.iter().zip([
            (
                0x45934usize,
                0x46728usize,
                0x46754usize,
                0x9E4A4usize,
                0x9E4B4usize,
            ),
            (0x46050, 0x47068, 0x470A0, 0xA3638, 0xA3648),
            (0x4575C, 0x46550, 0x4657C, 0x9DEDC, 0x9DEEC),
            (0x45DAC, 0x46DC4, 0x46DFC, 0xA3078, 0xA3088),
        ])
    {
        let rom = fs::read(local_rom_path(case.rom)).unwrap();
        let player_count = if case.target.contains("MFOMT") { 8 } else { 6 };
        let rival_start = if case.target.contains("MFOMT") {
            62
        } else {
            60
        };

        for index in 0..player_count {
            let slot = 54 + index;
            assert_eq!(
                u32::from_le_bytes(
                    rom[table + slot * 4..table + slot * 4 + 4]
                        .try_into()
                        .unwrap()
                ) as usize,
                0x0800_0000 + player_handlers + index * 6,
                "{} player-event timer slot {slot}",
                case.name
            );
        }
        for index in 0..5 {
            let slot = rival_start + index;
            assert_eq!(
                u32::from_le_bytes(
                    rom[table + slot * 4..table + slot * 4 + 4]
                        .try_into()
                        .unwrap()
                ) as usize,
                0x0800_0000 + rival_handlers + index * 6,
                "{} rival-event timer slot {slot}",
                case.name
            );
        }

        let final_player_handler = player_handlers + (player_count - 1) * 6;
        let final_rival_handler = rival_handlers + 4 * 6;
        assert_eq!(
            &rom[final_player_handler + 4..final_player_handler + 6],
            &[0, 0x22]
        );
        assert_eq!(
            &rom[final_rival_handler + 4..final_rival_handler + 6],
            &[1, 0x22]
        );
        assert_eq!(
            &rom[player_getter..player_getter + BUGGED_GETTER.len()],
            &BUGGED_GETTER,
            "{} bugged player-event getter",
            case.name
        );
        assert_eq!(
            &rom[rival_getter..rival_getter + BUGGED_GETTER.len()],
            &BUGGED_GETTER,
            "{} rival-event getter",
            case.name
        );
    }
}

#[test]
fn native_script_string_lookup_retains_off_by_one_bug_on_all_targets() {
    const PREFIX: [u8; 12] = [
        0x00, 0xB5, 0x02, 0x1C, 0x10, 0x69, 0x81, 0x42, 0x04, 0xD9, 0x01, 0x48,
    ];
    const SUFFIX: [u8; 12] = [
        0x50, 0x69, 0x89, 0x00, 0x09, 0x18, 0x90, 0x69, 0x09, 0x68, 0x40, 0x18,
    ];

    for (case, handler) in CASES.iter().zip([0x3F7E0usize, 0x3F9CC, 0x3F454, 0x3F728]) {
        let rom = fs::read(local_rom_path(case.rom)).unwrap();
        assert_eq!(
            &rom[handler..handler + PREFIX.len()],
            &PREFIX,
            "{}",
            case.name
        );
        assert_eq!(
            &rom[handler + 20..handler + 20 + SUFFIX.len()],
            &SUFFIX,
            "{}",
            case.name
        );
        // CMP id,string_count followed by BLS enters the table lookup when
        // id <= count.  Equality is the original off-by-one bug: valid IDs
        // stop at count - 1. Mary-C intentionally requires a declared symbol
        // and does not expose this unsafe extra entry.
        assert_eq!(&rom[handler + 6..handler + 10], &[0x81, 0x42, 0x04, 0xD9]);
    }
}

#[test]
fn native_cooking_submission_variable_has_no_store_or_notification() {
    for (case, id, set_table, tail, post_table, lower, exit) in [
        (&CASES[0], 401, 0x49034, 0x4DA2E, 0x4DA5C, 70, 0x4E0E6),
        (&CASES[1], 445, 0x4A430, 0x4FEEE, 0x4FF14, 72, 0x506A2),
        (&CASES[2], 401, 0x48E5C, 0x4D856, 0x4D884, 70, 0x4DF0E),
        (&CASES[3], 445, 0x4A18C, 0x4FC4A, 0x4FC70, 72, 0x503FE),
    ] {
        let rom = fs::read(local_rom_path(case.rom)).unwrap();
        for (p, expected) in [
            (set_table + (id - 28) * 4, tail),
            (post_table + (id - lower) * 4, exit),
        ] {
            assert_eq!(
                u32::from_le_bytes(rom[p..p + 4].try_into().unwrap()) as usize,
                expected + 0x08000000
            );
        }
        assert_eq!(
            &rom[exit..exit + 14],
            &[2, 0xB0, 0x18, 0xBC, 0x98, 0x46, 0xA1, 0x46, 0xF0, 0xBC, 1, 0xBC, 0, 0x47]
        );
    }
}

#[test]
fn native_child_dates_pregnancy_counter_and_wedding_dates_are_read_only() {
    for (case, set_table, no_store_tail) in [
        (&CASES[0], 0x49034, 0x4DA2E),
        (&CASES[1], 0x4A430, 0x4FEEE),
        (&CASES[2], 0x48E5C, 0x4D856),
        (&CASES[3], 0x4A18C, 0x4FC4A),
    ] {
        let rom = fs::read(local_rom_path(case.rom)).unwrap();
        for id in [46, 48, 49, 50, 51, 52] {
            let p = set_table + (id - 28) * 4;
            assert_eq!(
                u32::from_le_bytes(rom[p..p + 4].try_into().unwrap()) as usize,
                0x08000000 + no_store_tail
            );
        }
        // Child age/birthday, pregnancy counter and wedding date IDs skip storage and enter
        // the common readback tail directly, before notification dispatch.
        assert_eq!(
            &rom[no_store_tail..no_store_tail + 4],
            &[0x38, 0x1C, 0x41, 0x46]
        );
    }
    for (case, table, season, day, roster, getter, season_tail) in [
        (
            &CASES[0], 0x45934, 0x467E4, 0x46800, 0x1CD4_u32, 0xA1480, 0x48F32,
        ),
        (
            &CASES[1], 0x46050, 0x47144, 0x47160, 0x1CE4, 0xA6668, 0x49512,
        ),
        (
            &CASES[2], 0x4575C, 0x4660C, 0x46628, 0x1CD4, 0xA0EB8, 0x48D5A,
        ),
        (
            &CASES[3], 0x45DAC, 0x46EA0, 0x46EBC, 0x1CE4, 0xA60A8, 0x4926E,
        ),
    ] {
        let rom = fs::read(local_rom_path(case.rom)).unwrap();
        for (id, entry) in [(51, season), (52, day)] {
            let p = table + id * 4;
            assert_eq!(
                u32::from_le_bytes(rom[p..p + 4].try_into().unwrap()) as usize,
                entry + 0x08000000
            );
            let pc = entry + 12;
            let hi = u16::from_le_bytes(rom[pc..pc + 2].try_into().unwrap());
            let lo = u16::from_le_bytes(rom[pc + 2..pc + 4].try_into().unwrap());
            assert_eq!(hi & 0xF800, 0xF000);
            assert_eq!(lo & 0xF800, 0xF800);
            let delta = (((i32::from(hi & 0x7FF) << 12) | (i32::from(lo & 0x7FF) << 1)) << 9) >> 9;
            assert_eq!((pc as i32 + 4 + delta) as usize, getter);
            let ldr = u16::from_le_bytes(rom[entry + 8..entry + 10].try_into().unwrap());
            assert_eq!(ldr & 0xFF00, 0x4A00);
            let pool = ((entry + 12) & !3) + usize::from(ldr & 255) * 4;
            assert_eq!(&rom[pool..pool + 4], &roster.to_le_bytes());
        }
        assert_eq!(&rom[getter..getter + 4], &[0x2C, 0x30, 0x70, 0x47]);
        assert_eq!(&rom[season + 16..season + 18], &[0x40, 0x78]);
        let pc = season + 18;
        let hi = u16::from_le_bytes(rom[pc..pc + 2].try_into().unwrap());
        let lo = u16::from_le_bytes(rom[pc + 2..pc + 4].try_into().unwrap());
        assert_eq!(hi & 0xF800, 0xF000);
        assert_eq!(lo & 0xF800, 0xF800);
        let delta = (((i32::from(hi & 0x7FF) << 12) | (i32::from(lo & 0x7FF) << 1)) << 9) >> 9;
        assert_eq!((pc as i32 + 4 + delta) as usize, season_tail);
        assert_eq!(&rom[season_tail..season_tail + 4], &[0x80, 7, 0x80, 15]);
        assert_eq!(
            &rom[day + 16..day + 24],
            &[0x40, 0x78, 0x40, 6, 0xC0, 14, 1, 0x30]
        );
    }
}

#[test]
fn native_event_registration_search_and_removal_match_four_targets() {
    fn target(rom: &[u8], pc: usize) -> usize {
        let hi = u16::from_le_bytes(rom[pc..pc + 2].try_into().unwrap());
        let lo = u16::from_le_bytes(rom[pc + 2..pc + 4].try_into().unwrap());
        assert_eq!(hi & 0xF800, 0xF000);
        assert_eq!(lo & 0xF800, 0xF800);
        let delta = (((i32::from(hi & 0x7FF) << 12) | (i32::from(lo & 0x7FF) << 1)) << 9) >> 9;
        (pc as i32 + 4 + delta) as usize
    }
    let mut baseline = None;
    let mut search_baseline = None;
    let mut context_baseline = None;
    for (case, context, virtual_thunk, release) in [
        (&CASES[0], 0x125EC, 0xD3914, 0x50D0C),
        (&CASES[1], 0x12778, 0xDB53C, 0x532C4),
        (&CASES[2], 0x124BC, 0xD30CC, 0x50A98),
        (&CASES[3], 0x1261C, 0xDB050, 0x52F84),
    ] {
        let rom = fs::read(local_rom_path(case.rom)).unwrap();
        let mut body = rom[context..context + 0x6C].to_vec();
        let table = match context {
            0x125EC => 0x3F904,
            0x12778 => 0x3FAF0,
            0x124BC => 0x3F578,
            0x1261C => 0x3F84C,
            _ => unreachable!(),
        };
        let p = table + 0x1E * 4;
        let reset_handler =
            u32::from_le_bytes(rom[p..p + 4].try_into().unwrap()) as usize - 0x08000000;
        // ResetTalkUi uses exactly the same context cleanup entry.
        assert_eq!(target(&rom, reset_handler + 0x10), context);
        for (offset, expected) in [
            (0x1C, virtual_thunk),
            (0x3A, virtual_thunk),
            (0x52, release),
        ] {
            assert_eq!(target(&rom, context + offset), expected);
            body[offset..offset + 4].fill(0);
        }
        // Includes the bit0100 clear mask in the literal pool. Object gameplay
        // identities and virtual destructor bodies are not inferred here.
        assert_eq!(&rom[context + 0x68..context + 0x6C], &[0xFF, 0xFE, 0, 0]);
        if let Some(expected) = &context_baseline {
            assert_eq!(&body, expected);
        } else {
            context_baseline = Some(body);
        }
    }
    for (case, add, remove, find_add, find_remove, move_bytes) in [
        (&CASES[0], 0x9C600, 0x9C644, 0xE3DB4, 0xE3E28, 0xD39F8),
        (&CASES[1], 0xA1570, 0xA15B4, 0xEC2F4, 0xEC368, 0xDB620),
        (&CASES[2], 0x9C038, 0x9C07C, 0xE354C, 0xE35C0, 0xD31B0),
        (&CASES[3], 0xA0FB0, 0xA0FF4, 0xEBE04, 0xEBE78, 0xDB134),
    ] {
        let rom = fs::read(local_rom_path(case.rom)).unwrap();
        assert_eq!(target(&rom, add + 0x18), find_add);
        assert_eq!(target(&rom, remove + 0x18), find_remove);
        assert_eq!(target(&rom, remove + 0x3C), move_bytes);
        // Both helpers are the same unsigned-halfword linear search:
        // first matching address, or the supplied end pointer when absent.
        let search = &rom[find_add..find_add + 0x74];
        assert_eq!(search, &rom[find_remove..find_remove + 0x74]);
        if let Some(expected) = &search_baseline {
            assert_eq!(search, expected);
        } else {
            search_baseline = Some(search.to_vec());
        }
        // Source: func_0809C644. A found entry is removed by shifting the
        // following halfwords left; count decrements, unused tail is not cleared.
        // This comparison does not claim to verify the memmove leaf itself.
        let mut body = rom[remove..remove + 0x50].to_vec();
        for offset in [0x18, 0x3C] {
            body[offset..offset + 4].fill(0);
        }
        if let Some(expected) = &baseline {
            assert_eq!(&body, expected);
        } else {
            baseline = Some(body);
        }
    }
}

#[test]
fn native_mfomt_unknown_53_normalizes_writes_but_reads_raw_byte() {
    // Logical slot 53 is also a roster-tail byte in FoMT, but it is a
    // different physical field. FoMT scripts prove the spouse-work meaning
    // for save+0x2148; that meaning must not be copied to MFoMT save+0x2160
    // merely because the public variable number and getter shape match.
    for (case, get_table, getter) in [(&CASES[0], 0x45934, 0x46820), (&CASES[2], 0x4575C, 0x46648)]
    {
        let rom = fs::read(local_rom_path(case.rom)).unwrap();
        assert_eq!(
            u32::from_le_bytes(
                rom[get_table + 53 * 4..get_table + 53 * 4 + 4]
                    .try_into()
                    .unwrap()
            ) as usize,
            0x08000000 + getter
        );
        assert_eq!(&rom[getter + 20..getter + 24], &[0x48, 0x21, 0, 0]);
    }

    for (case, get_table, set_table, getter, setter, read_leaf, store_leaf) in [
        (
            &CASES[1], 0x46050, 0x4A430, 0x47180, 0x4AFEE, 0x4A346, 0x4FEEC,
        ),
        (
            &CASES[3], 0x45DAC, 0x4A18C, 0x46EDC, 0x4AD4A, 0x4A0A2, 0x4FC48,
        ),
    ] {
        let rom = fs::read(local_rom_path(case.rom)).unwrap();
        let init_tail = if case.target == "MARY_MFOMT_US" {
            0xA4F6C
        } else {
            0xA49AC
        };
        // Roster tail: clear word+478, then the distinct byte+47C (Var53).
        assert_eq!(
            &rom[init_tail..init_tail + 16],
            &[
                0x8F, 0x21, 0xC9, 0x00, 0x78, 0x18, 0x00, 0x21, 0x01, 0x60, 0x08, 0x4A, 0xB8, 0x18,
                0x01, 0x70,
            ]
        );
        assert_eq!(&rom[init_tail + 44..init_tail + 48], &[0x7C, 0x04, 0, 0]);
        for (table, index, entry) in [(get_table, 53, getter), (set_table, 53 - 28, setter)] {
            let p = table + index * 4;
            assert_eq!(
                u32::from_le_bytes(rom[p..p + 4].try_into().unwrap()) as usize,
                0x08000000 + entry
            );
        }
        for (pc, expected) in [(getter + 10, read_leaf), (setter + 18, store_leaf)] {
            let hi = u16::from_le_bytes(rom[pc..pc + 2].try_into().unwrap());
            let lo = u16::from_le_bytes(rom[pc + 2..pc + 4].try_into().unwrap());
            assert_eq!(hi & 0xF800, 0xF000);
            assert_eq!(lo & 0xF800, 0xF800);
            let delta = (((i32::from(hi & 0x7FF) << 12) | (i32::from(lo & 0x7FF) << 1)) << 9) >> 9;
            assert_eq!((pc as i32 + 4 + delta) as usize, expected);
        }
        assert_eq!(&rom[getter + 16..getter + 20], &[0x60, 0x21, 0, 0]);
        assert_eq!(&rom[setter + 22..setter + 26], &[0x60, 0x21, 0, 0]);
        assert_eq!(&rom[read_leaf..read_leaf + 4], &[0x80, 0x18, 0x00, 0x78]);
        assert_eq!(
            &rom[setter + 8..setter + 14],
            &[0x70, 0x42, 0x30, 0x43, 0xC0, 0x0F]
        );
        assert_eq!(&rom[store_leaf..store_leaf + 2], &[0x08, 0x70]);
        // Post-write dispatch subtracts72 with an unsigned range check.
        // ID53 is outside this domain and returns without notification.
        assert_eq!(
            &rom[store_leaf + 10..store_leaf + 24],
            &[0x02, 0x1C, 0x41, 0x46, 0x48, 0x39, 0xE6, 0x20, 0x40, 0x00, 0x81, 0x42, 0x00, 0xD9]
        );
        let pc = store_leaf + 24;
        let jump = u16::from_le_bytes(rom[pc..pc + 2].try_into().unwrap());
        assert_eq!(jump & 0xF800, 0xE000);
        let delta = (i32::from(jump & 0x7FF) << 21) >> 20;
        let exit = (pc as i32 + 4 + delta) as usize;
        assert_eq!(
            &rom[exit..exit + 14],
            &[0x02, 0xB0, 0x18, 0xBC, 0x98, 0x46, 0xA1, 0x46, 0xF0, 0xBC, 0x01, 0xBC, 0x00, 0x47]
        );
    }
}

#[test]
fn native_mfomt_roster_copy_preserves_unknown_53_byte() {
    let reference = fs::read(local_rom_path(CASES[1].rom)).unwrap();
    for (case, entry, caller, outer_call, outer_entry, length_pool) in [
        (&CASES[1], 0xDE4A0, 0xDBEDC, 0x4124, 0xDBDA4, 0x119B8),
        (&CASES[3], 0xDDFB4, 0xDB9F0, 0x413C, 0xDB8B8, 0x1196C),
    ] {
        let rom = fs::read(local_rom_path(case.rom)).unwrap();
        // Entire roster copy, including the unnormalized byte at +47C.
        assert_eq!(&rom[entry..entry + 0x414], &reference[0xDE4A0..0xDE8B4]);
        assert_eq!(&rom[length_pool..length_pool + 4], &[0x30, 0x35, 0, 0]);
        assert_eq!(
            &rom[caller - 6..caller],
            &[0x64, 0x49, 0x68, 0x18, 0x71, 0x18]
        );
        let pool = ((caller - 6 + 4) & !3) + 0x64 * 4;
        assert_eq!(&rom[pool..pool + 4], &[0xE4, 0x1C, 0, 0]);
        for (pc, target) in [(caller, entry), (outer_call, outer_entry)] {
            let hi = u16::from_le_bytes(rom[pc..pc + 2].try_into().unwrap());
            let lo = u16::from_le_bytes(rom[pc + 2..pc + 4].try_into().unwrap());
            assert_eq!(hi & 0xF800, 0xF000);
            assert_eq!(lo & 0xF800, 0xF800);
            let delta = (((i32::from(hi & 0x7FF) << 12) | (i32::from(lo & 0x7FF) << 1)) << 9) >> 9;
            assert_eq!((pc as i32 + 4 + delta) as usize, target);
        }
    }
}

#[test]
fn native_mfomt_unknown_winter_thanksgiving_interleaved_slots_preserve_two_bit_domains() {
    for (case, get_table, set_table, getters, setters) in [
        (
            &CASES[1],
            0x46050,
            0x4A430,
            [0x49140, 0x49164, 0x4918C, 0x491B0, 0x491D8],
            [0x4E170, 0x4E1B4, 0x4E1F4, 0x4E234, 0x4E274],
        ),
        (
            &CASES[3],
            0x45DAC,
            0x4A18C,
            [0x48E9C, 0x48EC0, 0x48EE8, 0x48F0C, 0x48F34],
            [0x4DECC, 0x4DF10, 0x4DF50, 0x4DF90, 0x4DFD0],
        ),
    ] {
        let rom = fs::read(local_rom_path(case.rom)).unwrap();
        for ((id, getter), setter) in [482usize, 484, 486, 488, 490]
            .into_iter()
            .zip(getters)
            .zip(setters)
        {
            assert_eq!(
                u32::from_le_bytes(
                    rom[get_table + id * 4..get_table + id * 4 + 4]
                        .try_into()
                        .unwrap()
                ),
                0x08000000 + getter as u32,
                "{} getter slot {id}",
                case.name
            );
            assert_eq!(
                u32::from_le_bytes(
                    rom[set_table + (id - 28) * 4..set_table + (id - 28) * 4 + 4]
                        .try_into()
                        .unwrap()
                ),
                0x08000000 + setter as u32,
                "{} setter slot {id}",
                case.name
            );
            if id == 486 {
                // This packed position uses the top two bits: its shared getter
                // loads a byte then shifts right by six. The setter reaches the
                // matching preserve-low-six-bits path rather than using the
                // inline `value & 3` shape of the other four fields.
                let getter_leaf = if case.target == "MARY_MFOMT_US" {
                    0x497AA
                } else {
                    0x49506
                };
                assert_eq!(
                    &rom[getter_leaf..getter_leaf + 6],
                    &[0x80, 0x18, 0x00, 0x78, 0x80, 0x09],
                    "{} top-two-bit getter for slot {id}",
                    case.name
                );
                assert_eq!(
                    &rom[setter + 16..setter + 20],
                    &[0x3F, 0x20, 0x01, 0xF0],
                    "{} preserve-low-six-bits setter path for slot {id}",
                    case.name
                );
            } else {
                // MOV r0,#3; AND r0,r0,r? proves the input is limited to two bits.
                assert_eq!(
                    &rom[setter + 12..setter + 16],
                    &[0x03, 0x20, 0x06, 0x40],
                    "{} two-bit setter mask for slot {id}",
                    case.name
                );
            }
        }
    }
}

#[test]
fn native_child_age_initialization_and_saturation_match_four_targets() {
    for (case, constructor, update, age, walking) in [
        (&CASES[0], 0x9EA6C, 0x9EAF8, 0x9EAD8, 0x9EAE0),
        (&CASES[1], 0xA3C00, 0xA3C8C, 0xA3C6C, 0xA3C74),
        (&CASES[2], 0x9E4A4, 0x9E530, 0x9E510, 0x9E518),
        (&CASES[3], 0xA3640, 0xA36CC, 0xA36AC, 0xA36B4),
    ] {
        let rom = fs::read(local_rom_path(case.rom)).unwrap();
        // Birthday was just written at +24; increment the address and clear age.
        assert_eq!(
            &rom[constructor + 0x3E..constructor + 0x44],
            &[0x01, 0x31, 0x00, 0x20, 0x08, 0x70]
        );
        // Clear only the walking bit at +26, preserving the other milestone bits.
        assert_eq!(
            &rom[constructor + 0x44..constructor + 0x50],
            &[0x2A, 0x1C, 0x26, 0x32, 0x11, 0x78, 0x02, 0x38, 0x08, 0x40, 0x10, 0x70]
        );
        assert_eq!(&rom[age..age + 6], &[0x25, 0x30, 0x00, 0x78, 0x70, 0x47]);
        assert_eq!(
            &rom[walking..walking + 10],
            &[0x26, 0x30, 0x00, 0x78, 0xC0, 0x07, 0xC0, 0x0F, 0x70, 0x47]
        );
        // Unsigned byte >254 skips increment: 255 does not wrap to zero.
        assert_eq!(
            &rom[update + 0x16..update + 0x22],
            &[0x25, 0x34, 0x20, 0x78, 0xFE, 0x28, 0x01, 0xD8, 0x01, 0x30, 0x20, 0x70]
        );
    }
}

#[test]
fn native_pregnancy_day_counter_keeps_family_storage_and_exact_stop() {
    let fomt = fs::read(local_rom_path(CASES[0].rom)).unwrap();
    let mfomt = fs::read(local_rom_path(CASES[1].rom)).unwrap();
    for (case, table, getter, pool, stored_offset, update, reference, reference_getter, size) in [
        (
            &CASES[0], 0x45934, 0x4677A, 0x467E0, 0x2144u32, 0xA104A, &fomt, 0x4677A, 0x6A,
        ),
        (
            &CASES[1], 0x46050, 0x470C6, 0x47140, 0x215Cu32, 0xA625C, &mfomt, 0x470C6, 0x7E,
        ),
        (
            &CASES[2], 0x4575C, 0x465A2, 0x46608, 0x2144u32, 0xA0A82, &fomt, 0x4677A, 0x6A,
        ),
        (
            &CASES[3], 0x45DAC, 0x46E22, 0x46E9C, 0x215Cu32, 0xA5C9C, &mfomt, 0x470C6, 0x7E,
        ),
    ] {
        let rom = fs::read(local_rom_path(case.rom)).unwrap();
        let slot = table + 50 * 4;
        assert_eq!(
            u32::from_le_bytes(rom[slot..slot + 4].try_into().unwrap()),
            0x08000000 + getter as u32
        );
        // Regional comparison includes the pregnancy predicates and literal pool.
        assert_eq!(
            &rom[getter..getter + size],
            &reference[reference_getter..reference_getter + size]
        );
        assert_eq!(&rom[pool..pool + 4], &stored_offset.to_le_bytes());
        // CMP #60 / BEQ, not >=60: edited values are not saturated.
        assert_eq!(
            &rom[update + 8..update + 16],
            &[0x3C, 0x28, 0x01, 0xD0, 0x01, 0x30, 0x08, 0x60]
        );
        let prefix: &[u8] = if stored_offset == 0x2144 {
            &[0x8E, 0x20, 0xC0, 0x00, 0x39, 0x18, 0x08, 0x68]
        } else {
            &[0x8F, 0x22, 0xD2, 0x00, 0xB9, 0x18, 0x08, 0x68]
        };
        assert_eq!(&rom[update..update + 8], prefix);
    }
}

#[test]
fn native_barn_animal_daily_update_advances_both_pregnancy_counters() {
    fn call_target(rom: &[u8], pc: usize) -> usize {
        let hi = u16::from_le_bytes(rom[pc..pc + 2].try_into().unwrap());
        let lo = u16::from_le_bytes(rom[pc + 2..pc + 4].try_into().unwrap());
        assert_eq!(hi & 0xF800, 0xF000);
        assert_eq!(lo & 0xF800, 0xF800);
        let raw = (((hi & 0x7FF) as i32) << 12) | (((lo & 0x7FF) as i32) << 1);
        (pc as i32 + 4 + ((raw << 9) >> 9)) as usize
    }

    let reference = fs::read(local_rom_path(CASES[0].rom)).unwrap();
    for (case, (update, cow_update, sheep_update)) in CASES.iter().zip([
        (0x9B970, 0x9BE74, 0x9BFB4),
        (0xA08DC, 0xA0DE0, 0xA0F20),
        (0x9B3A8, 0x9B8AC, 0x9B9EC),
        (0xA031C, 0xA0820, 0xA0960),
    ]) {
        let rom = fs::read(local_rom_path(case.rom)).unwrap();

        // The complete 0x64-byte BarnAnimal daily updater, including both
        // saturating five-bit counters and the sickness gate, is identical.
        assert_eq!(
            &rom[update..update + 0x64],
            &reference[0x9B970..0x9B9D4],
            "{} BarnAnimal daily pregnancy-counter update",
            case.name
        );
        // Both species-specific daily methods actually invoke this updater.
        assert_eq!(
            call_target(&rom, cow_update + 6),
            update,
            "{} cow daily update",
            case.name
        );
        assert_eq!(
            call_target(&rom, sheep_update + 6),
            update,
            "{} sheep daily update",
            case.name
        );
    }
}

#[test]
fn native_child_entity_creation_uses_same_slot_and_virtual_protocol() {
    for (case, entry, roster) in [
        (&CASES[0], 0x167DC, 0x1CD4u32),
        (&CASES[1], 0x1685C, 0x1CE4u32),
        (&CASES[2], 0x16570, 0x1CD4u32),
        (&CASES[3], 0x166D0, 0x1CE4u32),
    ] {
        let rom = fs::read(local_rom_path(case.rom)).unwrap();
        // Both manager operations use entity 35, not the callable's logical ID.
        assert_eq!(
            &rom[entry + 0x14..entry + 0x1E],
            &[0xA8, 0x34, 0x20, 0x68, 0x01, 0x68, 0x8A, 0x6B, 0x23, 0x21]
        );
        assert_eq!(
            &rom[entry + 0x22..entry + 0x2A],
            &[0x20, 0x68, 0x01, 0x68, 0x0A, 0x6C, 0x23, 0x21]
        );
        // Null entity skips state query; +28 queries, +20 enables when false.
        assert_eq!(
            &rom[entry + 0x2E..entry + 0x3A],
            &[0x04, 0x1C, 0x00, 0x2C, 0x0C, 0xD0, 0x60, 0x69, 0x81, 0x6A, 0x20, 0x1C]
        );
        assert_eq!(
            &rom[entry + 0x3E..entry + 0x4A],
            &[0x00, 0x06, 0x00, 0x28, 0x04, 0xD1, 0x60, 0x69, 0x01, 0x6A, 0x20, 0x1C]
        );
        assert_eq!(&rom[entry + 0x54..entry + 0x58], &roster.to_le_bytes());
    }
}

#[test]
fn native_child_entity_dispatch_and_vtables_match_four_targets() {
    for (case, manager, table, branch, constructor, pool_offset, child, enable, query) in [
        (
            &CASES[0], 0xE5EC4, 0x1A924, 0x1B324, 0x36E2C, 0x40, 0xE6918, 0x1FF78, 0x1FF6C,
        ),
        (
            &CASES[1], 0xEE404, 0x1AAA8, 0x1B4A8, 0x370F8, 0x3C, 0xEEE58, 0x20100, 0x200F4,
        ),
        (
            &CASES[2], 0xE5304, 0x1A6B8, 0x1B0B8, 0x36BC0, 0x40, 0xE5D58, 0x1FD0C, 0x1FD00,
        ),
        (
            &CASES[3], 0xEDF14, 0x1A91C, 0x1B31C, 0x36F6C, 0x3C, 0xEE968, 0x1FF74, 0x1FF68,
        ),
    ] {
        let rom = fs::read(local_rom_path(case.rom)).unwrap();
        let word = |at: usize| u32::from_le_bytes(rom[at..at + 4].try_into().unwrap());
        assert_eq!(word(manager + 0x38), 0x08000001 + (table - 0x44) as u32);
        assert_eq!(word(table + 35 * 4), 0x08000000 + branch as u32);
        assert_eq!(word(constructor + pool_offset), 0x08000000 + child as u32);
        assert_eq!(word(child + 0x20), 0x08000001 + enable as u32);
        assert_eq!(word(child + 0x28), 0x08000001 + query as u32);
        assert_eq!(&rom[enable..enable + 6], &[1, 0x21, 0x81, 0x71, 0x70, 0x47]);
        assert_eq!(&rom[query..query + 4], &[0x80, 0x79, 0x70, 0x47]);
    }
}

#[test]
fn native_child_walking_setter_cannot_clear_existing_state() {
    for (case, table, entry, readback, set_bit) in [
        (&CASES[0], 0x49034, 0x499E4, 0x4DA2E, 0x9EAEC),
        (&CASES[1], 0x4A430, 0x4B008, 0x4FEEE, 0xA3C80),
        (&CASES[2], 0x48E5C, 0x4980C, 0x4D856, 0x9E524),
        (&CASES[3], 0x4A18C, 0x4AD64, 0x4FC4A, 0xA36C0),
    ] {
        let rom = fs::read(local_rom_path(case.rom)).unwrap();
        let slot = table + (47 - 28) * 4;
        assert_eq!(
            u32::from_le_bytes(rom[slot..slot + 4].try_into().unwrap()),
            0x08000000 + entry as u32
        );
        // Nonnull child, nonzero input, and not already walking are required.
        assert_eq!(
            &rom[entry + 0x10..entry + 0x16],
            &[4, 0x1C, 0, 0x2C, 1, 0xD1]
        );
        assert_eq!(&rom[entry + 0x1A..entry + 0x1E], &[0, 0x2E, 1, 0xD1]);
        assert_eq!(&rom[entry + 0x26..entry + 0x2C], &[0, 6, 0, 0x28, 1, 0xD0]);
        for (offset, target) in [
            (0x16, readback),
            (0x1E, readback),
            (0x2C, readback),
            (0x32, set_bit),
            (0x36, readback),
        ] {
            let pc = entry + offset;
            let hi = u16::from_le_bytes(rom[pc..pc + 2].try_into().unwrap());
            let lo = u16::from_le_bytes(rom[pc + 2..pc + 4].try_into().unwrap());
            assert_eq!(hi & 0xF800, 0xF000);
            assert_eq!(lo & 0xF800, 0xF800);
            let delta = (((i32::from(hi & 0x7FF) << 12) | (i32::from(lo & 0x7FF) << 1)) << 9) >> 9;
            assert_eq!((pc as i32 + 4 + delta) as usize, target);
        }
        // OR bit 0 preserves all other milestone bits; there is no clear path.
        assert_eq!(
            &rom[set_bit..set_bit + 12],
            &[0x26, 0x30, 1, 0x78, 1, 0x22, 0x11, 0x43, 1, 0x70, 0x70, 0x47]
        );
    }
}

#[test]
fn native_fomt_unknown_236_is_same_two_bit_field_in_both_regions() {
    for (case, get_table, getter, set_table, setter) in [
        (&CASES[0], 0x45934, 0x474C0, 0x49034, 0x4AFA8),
        (&CASES[2], 0x4575C, 0x472E8, 0x48E5C, 0x4ADD0),
    ] {
        let rom = fs::read(local_rom_path(case.rom)).unwrap();
        let word = |at| u32::from_le_bytes(rom[at..at + 4].try_into().unwrap());
        assert_eq!(word(get_table + 236 * 4), 0x08000000 + getter as u32);
        assert_eq!(word(set_table + (236 - 28) * 4), 0x08000000 + setter as u32);
        assert_eq!(
            &rom[getter..getter + 12],
            &[0xD4, 0x21, 0x89, 0, 0x50, 0x18, 0, 0x68, 1, 0x4A, 1, 0xF0]
        );
        assert_eq!(word(getter + 16), 0x2173);
        assert_eq!(
            &rom[setter..setter + 24],
            &[
                0xD4, 0x21, 0x89, 0, 0x78, 0x18, 1, 0x68, 4, 0x4A, 0x89, 0x18, 3, 0x20, 6, 0x40,
                0x33, 1, 0x0A, 0x78, 0x31, 0x20, 0x40, 0x42,
            ]
        );
        assert_eq!(word(setter + 28), 0x2173);

        for old in 0u8..=255 {
            for input in 0u8..=255 {
                let stored = (old & !0x30) | ((input & 3) << 4);
                assert_eq!((stored >> 4) & 3, input & 3);
                assert_eq!(stored & !0x30, old & !0x30);
            }
        }
    }
}

#[test]
fn native_fomt_unknown_237_and_242_are_two_bit_fields_in_both_regions() {
    for (case, get_table, set_table, entries) in [
        (
            &CASES[0],
            0x45934,
            0x49034,
            [
                (237, 0x474D4, 0x4AFC8, 0x2173, 6),
                (242, 0x47538, 0x4B064, 0x2175, 0),
            ],
        ),
        (
            &CASES[2],
            0x4575C,
            0x48E5C,
            [
                (237, 0x472FC, 0x4ADF0, 0x2173, 6),
                (242, 0x47360, 0x4AE8C, 0x2175, 0),
            ],
        ),
    ] {
        let rom = fs::read(local_rom_path(case.rom)).unwrap();
        let word = |at| u32::from_le_bytes(rom[at..at + 4].try_into().unwrap());
        for (id, getter, setter, field, shift) in entries {
            assert_eq!(word(get_table + id * 4), 0x08000000 + getter as u32);
            assert_eq!(word(set_table + (id - 28) * 4), 0x08000000 + setter as u32);
            assert_eq!(
                &rom[getter..getter + 12],
                &[0xD4, 0x21, 0x89, 0, 0x50, 0x18, 0, 0x68, 1, 0x4A, 1, 0xF0]
            );
            assert_eq!(word(getter + 16), field);
            let literal_offset = if id == 237 { 24 } else { 32 };
            assert_eq!(word(setter + literal_offset), field);
            let expected_setter: &[u8] = if id == 237 {
                &[
                    0xD4, 0x21, 0x89, 0, 0x78, 0x18, 1, 0x68, 3, 0x4A, 0x89, 0x18, 0xB3, 1, 0x0A,
                    0x78, 0x3F, 0x20,
                ]
            } else {
                &[
                    0xD4, 0x21, 0x89, 0, 0x78, 0x18, 1, 0x68, 5, 0x4A, 0x89, 0x18, 3, 0x20, 6,
                    0x40, 0x0A, 0x78, 4, 0x20, 0x40, 0x42, 0x10, 0x40, 0x30, 0x43,
                ]
            };
            assert_eq!(
                &rom[setter..setter + expected_setter.len()],
                expected_setter
            );
            for old in 0u8..=255 {
                for input in 0u8..=255 {
                    let mask = 3u8 << shift;
                    let stored = (old & !mask) | ((input & 3) << shift);
                    assert_eq!((stored >> shift) & 3, input & 3);
                    assert_eq!(stored & !mask, old & !mask);
                }
            }
        }
    }
}

#[test]
fn native_fomt_unknown_236_242_storage_references_only_expose_known_slot_244() {
    for (case, fields, slot_244_extract) in [
        (
            &CASES[0],
            [
                (
                    0x2173u32,
                    &[
                        0x474A0usize,
                        0x474B4,
                        0x474C8,
                        0x474DC,
                        0x4AF6C,
                        0x4AF90,
                        0x4AFB0,
                        0x4AFD0,
                    ][..],
                ),
                (
                    0x2175,
                    &[
                        0x3DE88usize,
                        0x47540,
                        0x47554,
                        0x47568,
                        0x4757C,
                        0x47590,
                        0x475A4,
                        0x4B06C,
                        0x4B090,
                        0x4B0B0,
                        0x4B0D0,
                        0x4B0F0,
                        0x4B104,
                    ][..],
                ),
            ],
            0x3DE88usize,
        ),
        (
            &CASES[2],
            [
                (
                    0x2173,
                    &[
                        0x472C8usize,
                        0x472DC,
                        0x472F0,
                        0x47304,
                        0x4AD94,
                        0x4ADB8,
                        0x4ADD8,
                        0x4ADF8,
                    ][..],
                ),
                (
                    0x2175,
                    &[
                        0x3DAFCusize,
                        0x47368,
                        0x4737C,
                        0x47390,
                        0x473A4,
                        0x473B8,
                        0x473CC,
                        0x4AE94,
                        0x4AEB8,
                        0x4AED8,
                        0x4AEF8,
                        0x4AF18,
                        0x4AF2C,
                    ][..],
                ),
            ],
            0x3DAFC,
        ),
    ] {
        let rom = fs::read(local_rom_path(case.rom)).unwrap();
        for (field, expected_references) in fields {
            let literal_pools: HashSet<_> = (0..0x100000usize)
                .step_by(4)
                .filter(|&offset| {
                    u32::from_le_bytes(rom[offset..offset + 4].try_into().unwrap()) == field
                })
                .collect();
            let references = (0..0xE0000usize)
                .step_by(2)
                .filter(|&instruction| {
                    let opcode =
                        u16::from_le_bytes(rom[instruction..instruction + 2].try_into().unwrap());
                    if opcode & 0xF800 != 0x4800 {
                        return false;
                    }
                    let pool = ((instruction + 4) & !3) + 4 * (opcode as usize & 0xFF);
                    literal_pools.contains(&pool)
                })
                .collect::<Vec<_>>();
            assert_eq!(
                references, expected_references,
                "{} field {field:#X}",
                case.name
            );
        }

        // The sole business read of +0x2175 tests bit 4, the independently
        // named slot 244, rather than unknown slot 242 at bits 0..1.
        assert_eq!(
            &rom[slot_244_extract..slot_244_extract + 12],
            &[0x03, 0x49, 0x68, 0x18, 0x00, 0x78, 0xC0, 0x06, 0x00, 0x28, 0x03, 0xDA]
        );
    }
}

#[test]
fn native_fomt_unknown_224_and_225_preserve_physical_domains_without_direct_users() {
    for (case, get_table, set_table, entries, expected_literals) in [
        (
            &CASES[0],
            0x45934usize,
            0x49034usize,
            [
                (224usize, 0x473D0usize, 0x4AE24usize, 0x48662usize, 5u8),
                (225, 0x473E4, 0x4AE44, 0x48626, 7),
            ],
            &[
                0x473B8usize,
                0x473CC,
                0x473E0,
                0x473F4,
                0x4AE00,
                0x4AE20,
                0x4AE40,
                0x4AE60,
            ][..],
        ),
        (
            &CASES[2],
            0x4575C,
            0x48E5C,
            [
                (224, 0x471F8, 0x4AC4C, 0x4848A, 5),
                (225, 0x4720C, 0x4AC6C, 0x4844E, 7),
            ],
            &[
                0x471E0usize,
                0x471F4,
                0x47208,
                0x4721C,
                0x4AC28,
                0x4AC48,
                0x4AC68,
                0x4AC88,
            ][..],
        ),
    ] {
        let rom = fs::read(local_rom_path(case.rom)).unwrap();
        let word = |at| u32::from_le_bytes(rom[at..at + 4].try_into().unwrap());
        for (id, getter, setter, getter_leaf, shift) in entries {
            assert_eq!(word(get_table + id * 4), 0x0800_0000 + getter as u32);
            assert_eq!(word(set_table + (id - 28) * 4), 0x0800_0000 + setter as u32);
            assert_eq!(word(getter + 16), 0x2170);
            assert_eq!(word(setter + 28), 0x2170);
            assert_eq!(thumb_call_target(&rom, getter + 10), getter_leaf);
            let expected_leaf = if id == 224 {
                [0x80, 0x18, 0x00, 0x78, 0x40, 0x06, 0x80, 0x0F]
            } else {
                [0x80, 0x18, 0x00, 0x88, 0xC0, 0x05, 0x80, 0x0F]
            };
            assert_eq!(&rom[getter_leaf..getter_leaf + 8], &expected_leaf);

            for old in 0u16..=u16::MAX {
                for input in 0u16..=3 {
                    let mask = 3u16 << shift;
                    let stored = (old & !mask) | ((input & 3) << shift);
                    assert_eq!((stored >> shift) & 3, input & 3);
                    assert_eq!(stored & !mask, old & !mask);
                }
            }
        }

        let field_bytes = 0x2170u32.to_le_bytes();
        let actual_literals = rom[..0x100000]
            .windows(4)
            .enumerate()
            .filter_map(|(offset, bytes)| (bytes == field_bytes).then_some(offset))
            .collect::<Vec<_>>();
        assert_eq!(actual_literals, expected_literals, "{}", case.name);
    }
}

#[test]
fn native_fomt_unknown_275_and_276_preserve_physical_domains_in_both_regions() {
    for (case, get_table, set_table, entries) in [
        (
            &CASES[0],
            0x45934,
            0x49034,
            [
                (275, 0x477E8, 0x4B49C, 0x217C, 7, 1),
                (276, 0x477FC, 0x4B4B8, 0x217D, 0, 3),
            ],
        ),
        (
            &CASES[2],
            0x4575C,
            0x48E5C,
            [
                (275, 0x47610, 0x4B2C4, 0x217C, 7, 1),
                (276, 0x47624, 0x4B2E0, 0x217D, 0, 3),
            ],
        ),
    ] {
        let rom = fs::read(local_rom_path(case.rom)).unwrap();
        let word = |at| u32::from_le_bytes(rom[at..at + 4].try_into().unwrap());
        for (id, getter, setter, field, shift, input_mask) in entries {
            assert_eq!(word(get_table + id * 4), 0x08000000 + getter as u32);
            assert_eq!(word(set_table + (id - 28) * 4), 0x08000000 + setter as u32);
            assert_eq!(
                &rom[getter..getter + 12],
                &[0xD4, 0x21, 0x89, 0, 0x50, 0x18, 0, 0x68, 1, 0x4A, 1, 0xF0]
            );
            assert_eq!(word(getter + 16), field);
            let literal_offset = if id == 275 { 24 } else { 32 };
            assert_eq!(word(setter + literal_offset), field);
            let expected_setter: &[u8] = if id == 275 {
                &[
                    0xD4, 0x21, 0x89, 0, 0x78, 0x18, 1, 0x68, 3, 0x4A, 0x89, 0x18, 0xF3, 1, 0x0A,
                    0x78, 0x7F, 0x20,
                ]
            } else {
                &[
                    0xD4, 0x21, 0x89, 0, 0x78, 0x18, 1, 0x68, 5, 0x4A, 0x89, 0x18, 3, 0x20, 6,
                    0x40, 0x0A, 0x78, 4, 0x20, 0x40, 0x42, 0x10, 0x40, 0x30, 0x43,
                ]
            };
            assert_eq!(
                &rom[setter..setter + expected_setter.len()],
                expected_setter
            );
            for old in 0u8..=255 {
                for input in 0u8..=255 {
                    let field_mask = input_mask << shift;
                    let stored = (old & !field_mask) | ((input & input_mask) << shift);
                    assert_eq!((stored >> shift) & input_mask, input & input_mask);
                    assert_eq!(stored & !field_mask, old & !field_mask);
                }
            }
        }

        let getter_extractors = if case.target == "MARY_FOMT_US" {
            [
                (0x47CBCusize, 0x48966usize, [0x40, 0x07, 0x80, 0x0F]),
                (0x47CD0, 0x486B2, [0xC0, 0x06, 0x80, 0x0F]),
                (0x47CE4, 0x48662, [0x40, 0x06, 0x80, 0x0F]),
                (0x47D78, 0x48FC6, [0x80, 0x06, 0xC0, 0x0F]),
                (0x47D8C, 0x48E36, [0x80, 0x09, 0xD7, 0xE0]),
                (0x48048, 0x48F2E, [0x80, 0x07, 0x80, 0x0F]),
                (0x4805C, 0x48E5E, [0x00, 0x07, 0x80, 0x0F]),
                (0x48070, 0x48E76, [0x80, 0x06, 0x80, 0x0F]),
                (0x48098, 0x48F2E, [0x80, 0x07, 0x80, 0x0F]),
            ]
        } else {
            [
                (0x47AE4usize, 0x4878Eusize, [0x40, 0x07, 0x80, 0x0F]),
                (0x47AF8, 0x484DA, [0xC0, 0x06, 0x80, 0x0F]),
                (0x47B0C, 0x4848A, [0x40, 0x06, 0x80, 0x0F]),
                (0x47BA0, 0x48DEE, [0x80, 0x06, 0xC0, 0x0F]),
                (0x47BB4, 0x48C5E, [0x80, 0x09, 0xD7, 0xE0]),
                (0x47E70, 0x48D56, [0x80, 0x07, 0x80, 0x0F]),
                (0x47E84, 0x48C86, [0x00, 0x07, 0x80, 0x0F]),
                (0x47E98, 0x48C9E, [0x80, 0x06, 0x80, 0x0F]),
                (0x47EC0, 0x48D56, [0x80, 0x07, 0x80, 0x0F]),
            ]
        };
        for (getter, leaf, extraction) in getter_extractors {
            assert_eq!(thumb_call_target(&rom, getter + 10), leaf);
            assert_eq!(&rom[leaf + 4..leaf + 8], &extraction);
        }
    }
}

#[test]
fn native_ellen_stocking_gate_unknowns_have_no_direct_business_literal_users() {
    for (case, field, expected_literals) in [
        (
            &CASES[0],
            0x217Cu32,
            &[
                0x477A8usize,
                0x477BC,
                0x477D0,
                0x477E4,
                0x477F8,
                0x4B438,
                0x4B458,
                0x4B478,
                0x4B498,
                0x4B4B4,
            ][..],
        ),
        (
            &CASES[2],
            0x217Cu32,
            &[
                0x475D0usize,
                0x475E4,
                0x475F8,
                0x4760C,
                0x47620,
                0x4B260,
                0x4B280,
                0x4B2A0,
                0x4B2C0,
                0x4B2DC,
            ][..],
        ),
        (
            &CASES[0],
            0x217Du32,
            &[
                0x4780Cusize,
                0x47820,
                0x47834,
                0x47848,
                0x4B4D8,
                0x4B4F8,
                0x4B518,
                0x4B534,
            ][..],
        ),
        (
            &CASES[2],
            0x217Du32,
            &[
                0x47634usize,
                0x47648,
                0x4765C,
                0x47670,
                0x4B300,
                0x4B320,
                0x4B340,
                0x4B35C,
            ][..],
        ),
        (
            &CASES[1],
            0x2195u32,
            &[
                0x48200usize,
                0x48214,
                0x48228,
                0x4823C,
                0x4CA8C,
                0x4CAAC,
                0x4CACC,
                0x4CAE8,
            ][..],
        ),
        (
            &CASES[3],
            0x2195u32,
            &[
                0x47F5Cusize,
                0x47F70,
                0x47F84,
                0x47F98,
                0x4C7E8,
                0x4C808,
                0x4C828,
                0x4C844,
            ][..],
        ),
    ] {
        let rom = fs::read(local_rom_path(case.rom)).unwrap();
        let field_bytes = field.to_le_bytes();
        let actual_literals = rom[..0x100000]
            .windows(4)
            .enumerate()
            .filter_map(|(offset, bytes)| (bytes == field_bytes).then_some(offset))
            .collect::<Vec<_>>();
        assert_eq!(
            actual_literals, expected_literals,
            "{} Ellen-adjacent storage-byte literals escaped the audited variable accessor cluster",
            case.name
        );
    }
}

#[test]
fn native_fomt_unknown_314_through_320_preserve_physical_domains() {
    for (case, get_table, set_table, entries) in [
        (
            &CASES[0],
            0x45934,
            0x49034,
            [
                (314, 0x47AF4, 0x4B98C, 0x2186),
                (316, 0x47B24, 0x4B9CC, 0x2186),
                (317, 0x47B38, 0x4B9F0, 0x2187),
                (319, 0x47B60, 0x4BA30, 0x2187),
                (320, 0x47B74, 0x4BA50, 0x2187),
            ],
        ),
        (
            &CASES[2],
            0x4575C,
            0x48E5C,
            [
                (314, 0x4791C, 0x4B7B4, 0x2186),
                (316, 0x4794C, 0x4B7F4, 0x2186),
                (317, 0x47960, 0x4B818, 0x2187),
                (319, 0x47988, 0x4B858, 0x2187),
                (320, 0x4799C, 0x4B878, 0x2187),
            ],
        ),
    ] {
        let rom = fs::read(local_rom_path(case.rom)).unwrap();
        let word = |at| u32::from_le_bytes(rom[at..at + 4].try_into().unwrap());
        for (id, getter, setter, field) in entries {
            assert_eq!(word(get_table + id * 4), 0x08000000 + getter as u32);
            assert_eq!(word(set_table + (id - 28) * 4), 0x08000000 + setter as u32);
            let getter_literal = match id {
                314 | 320 => 0x18 + if id == 320 { 4 } else { 0 },
                _ => 0x10,
            };
            assert_eq!(word(getter + getter_literal), field);
            let setter_literal = match id {
                314 | 316 | 317 | 319 => 0x1C,
                320 => 0x30,
                _ => unreachable!(),
            };
            assert_eq!(word(setter + setter_literal), field);
            if id == 320 {
                assert_eq!(word(getter + 0x20), 0x2188);
                assert_eq!(word(setter + 0x34), 0x2188);
            }
        }
    }

    for old in 0u16..=u16::MAX {
        for input in 0u16..=7 {
            let stored_314 = (old & !(7 << 2)) | ((input & 7) << 2);
            assert_eq!((stored_314 >> 2) & 7, input & 7);
            let stored_316 = (old & !(3 << 7)) | ((input & 3) << 7);
            assert_eq!((stored_316 >> 7) & 3, input & 3);
            let stored_320 = (old & !(3 << 7)) | ((input & 3) << 7);
            assert_eq!((stored_320 >> 7) & 3, input & 3);
        }
    }
    for (shift, mask) in [(1, 3u8 << 1), (5, 3u8 << 5)] {
        for old in 0u8..=255 {
            for input in 0u8..=3 {
                let stored = (old & !mask) | ((input & 3) << shift);
                assert_eq!((stored >> shift) & 3, input & 3);
            }
        }
    }

    // Slot 319 is the distinct bits 5..6 field in save+0x2187. Pin the
    // extraction itself so it cannot be confused again with slot 318's
    // bits 3..4 field merely because both share the same backing byte.
    for (case, getter, getter_leaf) in
        [(&CASES[0], 0x47B60, 0x48662), (&CASES[2], 0x47988, 0x4848A)]
    {
        let rom = fs::read(local_rom_path(case.rom)).unwrap();
        assert_eq!(thumb_call_target(&rom, getter + 10), getter_leaf);
        assert_eq!(
            &rom[getter_leaf..getter_leaf + 8],
            &[0x80, 0x18, 0x00, 0x78, 0x40, 0x06, 0x80, 0x0F]
        );
    }
}

#[test]
fn native_fomt_lou_or_ruby_introduction_state_drives_sunday_schedule() {
    let us = fs::read(local_rom_path(CASES[0].rom)).unwrap();
    let jp = fs::read(local_rom_path(CASES[2].rom)).unwrap();
    let us_gate = 0x3E40C;
    let jp_gate = 0x3E080;
    let us_schedule = 0xF4974;
    let jp_schedule = 0xF41D0;

    assert_eq!(&us[us_gate..us_gate + 60], &jp[jp_gate..jp_gate + 60]);
    assert_eq!(
        u32::from_le_bytes(us[us_schedule..us_schedule + 4].try_into().unwrap()),
        0x0803E40D
    );
    assert_eq!(
        u32::from_le_bytes(jp[jp_schedule..jp_schedule + 4].try_into().unwrap()),
        0x0803E081
    );

    for (rom, pointer) in [(&us, 0x0803E40Du32), (&jp, 0x0803E081u32)] {
        let needle = pointer.to_le_bytes();
        assert_eq!(
            rom.chunks_exact(4)
                .filter(|word| *word == needle.as_slice())
                .count(),
            1,
            "schedule gate pointer must have one aligned owner"
        );
    }

    let mut us_table = us[us_schedule..us_schedule + 0x4C].to_vec();
    let mut jp_table = jp[jp_schedule..jp_schedule + 0x4C].to_vec();
    for pointer_offset in [0, 8, 16, 24, 32, 36] {
        us_table[pointer_offset..pointer_offset + 4].fill(0);
        jp_table[pointer_offset..pointer_offset + 4].fill(0);
    }
    assert_eq!(us_table, jp_table, "relocation-normalized schedule data");

    assert_eq!(
        &us[us_gate + 26..us_gate + 34],
        &[0xC0, 0x06, 0x80, 0x0F, 2, 0x28, 6, 0xD1]
    );
    assert_eq!(
        &us[us_gate + 34..us_gate + 40],
        &[0, 0x29, 4, 0xD1, 1, 0x20]
    );
    assert_eq!(
        u32::from_le_bytes(us[us_gate + 44..us_gate + 48].try_into().unwrap()),
        0x2187
    );

    // The gate's <<27 >>30 extracts original bits 3..4: known variable 318,
    // not adjacent unknown variable 319 (whose getter uses <<25 >>30).
    for (rom, getter_table, getter_entry, getter_leaf) in [
        (&us, 0x45934, 0x47B4C, 0x486B2),
        (&jp, 0x4575C, 0x47974, 0x484DA),
    ] {
        assert_eq!(
            u32::from_le_bytes(
                rom[getter_table + 318 * 4..getter_table + 318 * 4 + 4]
                    .try_into()
                    .unwrap()
            ) & !1,
            0x08000000 + getter_entry as u32
        );
        assert_eq!(thumb_call_target(rom, getter_entry + 10), getter_leaf);
        assert_eq!(
            &rom[getter_leaf..getter_leaf + 8],
            &[0x80, 0x18, 0x00, 0x78, 0xC0, 0x06, 0x80, 0x0F]
        );
    }
}

#[test]
fn native_mfomt_lou_or_ruby_introduction_state_drives_sunday_schedule() {
    let us = fs::read(local_rom_path(CASES[1].rom)).unwrap();
    let jp = fs::read(local_rom_path(CASES[3].rom)).unwrap();
    let us_gate = 0x3E698;
    let jp_gate = 0x3E3F4;
    let us_schedule = 0xFD3FC;
    let jp_schedule = 0xFD39C;

    assert_eq!(&us[us_gate..us_gate + 60], &jp[jp_gate..jp_gate + 60]);
    assert_eq!(
        u32::from_le_bytes(us[us_schedule..us_schedule + 4].try_into().unwrap()),
        0x0803E699
    );
    assert_eq!(
        u32::from_le_bytes(jp[jp_schedule..jp_schedule + 4].try_into().unwrap()),
        0x0803E3F5
    );

    for (rom, pointer) in [(&us, 0x0803E699u32), (&jp, 0x0803E3F5u32)] {
        let needle = pointer.to_le_bytes();
        assert_eq!(
            rom.chunks_exact(4)
                .filter(|word| *word == needle.as_slice())
                .count(),
            1,
            "schedule gate pointer must have one aligned owner"
        );
    }

    let mut us_table = us[us_schedule..us_schedule + 0x40].to_vec();
    let mut jp_table = jp[jp_schedule..jp_schedule + 0x40].to_vec();
    for pointer_offset in [0, 8, 16, 24, 32, 36] {
        us_table[pointer_offset..pointer_offset + 4].fill(0);
        jp_table[pointer_offset..pointer_offset + 4].fill(0);
    }
    assert_eq!(us_table, jp_table, "relocation-normalized schedule data");

    assert_eq!(
        &us[us_gate + 26..us_gate + 34],
        &[0x40, 0x07, 0x80, 0x0F, 2, 0x28, 6, 0xD1]
    );
    assert_eq!(
        &us[us_gate + 34..us_gate + 40],
        &[0, 0x29, 4, 0xD1, 1, 0x20]
    );
    assert_eq!(
        u32::from_le_bytes(us[us_gate + 44..us_gate + 48].try_into().unwrap()),
        0x21A0
    );

    // Each entity factory embeds the same schedule pointer and the independently
    // confirmed MFoMT Lou/Ruby idle animation ID 2082 in adjacent literals.
    assert_eq!(
        u32::from_le_bytes(us[0x36AD0..0x36AD4].try_into().unwrap()),
        0x080FD3FC
    );
    assert_eq!(
        u32::from_le_bytes(jp[0x36944..0x36948].try_into().unwrap()),
        0x080FD39C
    );
    assert_eq!(
        u32::from_le_bytes(us[0x36AD4..0x36AD8].try_into().unwrap()),
        2082
    );
    assert_eq!(
        u32::from_le_bytes(jp[0x36948..0x3694C].try_into().unwrap()),
        2082
    );

    // The gate's <<29 >>30 extracts original bits 1..2: known variable 326,
    // not adjacent unknown variable 327 (whose getter uses <<27 >>30).
    for (rom, getter_table, getter_entry, getter_leaf) in [
        (&us, 0x46050, 0x48594, 0x4A2C6),
        (&jp, 0x45DAC, 0x482F0, 0x4A022),
    ] {
        assert_eq!(
            u32::from_le_bytes(
                rom[getter_table + 326 * 4..getter_table + 326 * 4 + 4]
                    .try_into()
                    .unwrap()
            ) & !1,
            0x08000000 + getter_entry as u32
        );
        assert_eq!(thumb_call_target(rom, getter_entry + 10), getter_leaf);
        assert_eq!(
            &rom[getter_leaf..getter_leaf + 8],
            &[0x80, 0x18, 0x00, 0x78, 0x40, 0x07, 0x80, 0x0F]
        );
    }
}

#[test]
fn native_fomt_remaining_unknown_slots_preserve_physical_domains() {
    for (case, get_table, set_table, entries) in [
        (
            &CASES[0],
            0x45934,
            0x49034,
            [
                (334, 0x47CBC, 0x4BC48, 0x218C, 1, 3),
                (335, 0x47CD0, 0x4BC68, 0x218C, 3, 3),
                (336, 0x47CE4, 0x4BC88, 0x218C, 5, 3),
                (343, 0x47D78, 0x4BD70, 0x218E, 5, 1),
                (344, 0x47D8C, 0x4BD90, 0x218E, 6, 3),
                (381, 0x48048, 0x4C1C8, 0x2198, 0, 3),
                (382, 0x4805C, 0x4C1EC, 0x2198, 2, 3),
                (383, 0x48070, 0x4C20C, 0x2198, 4, 3),
                (385, 0x48098, 0x4C248, 0x2199, 0, 3),
            ],
        ),
        (
            &CASES[2],
            0x4575C,
            0x48E5C,
            [
                (334, 0x47AE4, 0x4BA70, 0x218C, 1, 3),
                (335, 0x47AF8, 0x4BA90, 0x218C, 3, 3),
                (336, 0x47B0C, 0x4BAB0, 0x218C, 5, 3),
                (343, 0x47BA0, 0x4BB98, 0x218E, 5, 1),
                (344, 0x47BB4, 0x4BBB8, 0x218E, 6, 3),
                (381, 0x47E70, 0x4BFF0, 0x2198, 0, 3),
                (382, 0x47E84, 0x4C014, 0x2198, 2, 3),
                (383, 0x47E98, 0x4C034, 0x2198, 4, 3),
                (385, 0x47EC0, 0x4C070, 0x2199, 0, 3),
            ],
        ),
    ] {
        let rom = fs::read(local_rom_path(case.rom)).unwrap();
        let word = |at| u32::from_le_bytes(rom[at..at + 4].try_into().unwrap());
        for (id, getter, setter, field, shift, input_mask) in entries {
            assert_eq!(word(get_table + id * 4), 0x08000000 + getter as u32);
            assert_eq!(word(set_table + (id - 28) * 4), 0x08000000 + setter as u32);
            assert_eq!(word(getter + 16), field);
            assert!(rom[setter..setter + 64]
                .windows(4)
                .any(|bytes| bytes == field.to_le_bytes()));

            for old in 0u8..=255 {
                for input in 0u8..=255 {
                    let field_mask = input_mask << shift;
                    let stored = (old & !field_mask) | ((input & input_mask) << shift);
                    assert_eq!((stored >> shift) & input_mask, input & input_mask);
                    assert_eq!(stored & !field_mask, old & !field_mask);
                }
            }
        }
    }
}

#[test]
fn native_fomt_unknown_327_and_333_preserve_cross_byte_domains() {
    for (case, get_table, set_table, slot_327, slot_333) in [
        (
            &CASES[0],
            0x45934,
            0x49034,
            (0x47C10, 0x4BB4C),
            (0x47C90, 0x4BC10),
        ),
        (
            &CASES[2],
            0x4575C,
            0x48E5C,
            (0x47A38, 0x4B974),
            (0x47AB8, 0x4BA38),
        ),
    ] {
        let rom = fs::read(local_rom_path(case.rom)).unwrap();
        let word = |at| u32::from_le_bytes(rom[at..at + 4].try_into().unwrap());

        for (id, getter, setter) in [
            (327usize, slot_327.0, slot_327.1),
            (333usize, slot_333.0, slot_333.1),
        ] {
            assert_eq!(word(get_table + id * 4), 0x0800_0000 + getter as u32);
            assert_eq!(word(set_table + (id - 28) * 4), 0x0800_0000 + setter as u32);
        }

        // Slot 327 is a four-bit word field at save offset +0x2188 bits 14..17.
        // Both the getter and setter use a 32-bit load/store because the field
        // crosses the +0x2189/+0x218A byte boundary.
        assert_eq!(
            &rom[slot_327.0 + 12..slot_327.0 + 18],
            &[0x00, 0x68, 0x80, 0x03, 0x00, 0x0F]
        );
        assert_eq!(word(slot_327.0 + 24), 0x2188);
        assert_eq!(word(slot_327.1 + 28), 0x2188);

        // Slot 333 joins +0x218B bits 5..7 with +0x218C bit 0. The two literal
        // offsets and the split extraction are fixed explicitly so this field
        // cannot later be mistaken for an entire byte-sized gameplay flag.
        assert_eq!(
            &rom[slot_333.0 + 12..slot_333.0 + 26],
            &[0x0A, 0x78, 0x52, 0x09, 0x05, 0x49, 0x40, 0x18, 0x00, 0x78, 0x01, 0x21, 0x08, 0x40]
        );
        assert_eq!(word(slot_333.0 + 36), 0x218B);
        assert_eq!(word(slot_333.0 + 40), 0x218C);
        assert_eq!(word(slot_333.1 + 48), 0x218B);
        assert_eq!(word(slot_333.1 + 52), 0x218C);

        let representative_words = [
            0u32,
            u32::MAX,
            0xAAAA_AAAA,
            0x5555_5555,
            0x0000_C000,
            0x0003_0000,
        ];
        for old in representative_words {
            for input in 0u32..=255 {
                let stored = (old & !(0xFu32 << 14)) | ((input & 0xF) << 14);
                assert_eq!((stored >> 14) & 0xF, input & 0xF);
                assert_eq!(stored & !(0xF << 14), old & !(0xF << 14));
            }
        }

        for old in [0u16, u16::MAX, 0xAAAA, 0x5555, 0x00E0, 0x0100] {
            for input in 0u16..=255 {
                let stored = (old & !(0xFu16 << 5)) | ((input & 0xF) << 5);
                assert_eq!((stored >> 5) & 0xF, input & 0xF);
                assert_eq!(stored & !(0xF << 5), old & !(0xF << 5));
            }
        }
    }
}

#[test]
fn native_fomt_late_unknown_storage_references_do_not_absorb_adjacent_fields() {
    fn thumb_literal_references(rom: &[u8], value: u32) -> Vec<usize> {
        let literal_locations = (0..0x100000usize)
            .step_by(4)
            .filter(|&at| u32::from_le_bytes(rom[at..at + 4].try_into().unwrap()) == value)
            .collect::<HashSet<_>>();

        (0..0xE0000usize)
            .step_by(2)
            .filter(|&at| {
                let opcode = u16::from_le_bytes(rom[at..at + 2].try_into().unwrap());
                if opcode & 0xF800 != 0x4800 {
                    return false;
                }
                let literal = ((at + 4) & !3) + usize::from(opcode & 0xFF) * 4;
                literal_locations.contains(&literal)
            })
            .collect()
    }

    for (case, expected) in [
        (
            &CASES[0],
            [
                (
                    0x218Bu32,
                    &[
                        0x2B0CEusize,
                        0x3076C,
                        0x47C70,
                        0x47C84,
                        0x47C98,
                        0x4BBD4,
                        0x4BBF8,
                        0x4BC1C,
                    ][..],
                ),
                (
                    0x218C,
                    &[
                        0x47CA0usize,
                        0x47CC4,
                        0x47CD8,
                        0x47CEC,
                        0x47D00,
                        0x47D3C,
                        0x4BC30,
                        0x4BC50,
                        0x4BC70,
                        0x4BC90,
                        0x4BCB0,
                        0x4BD14,
                        0x5D5D8,
                        0x80084,
                    ][..],
                ),
                (
                    0x218E,
                    &[
                        0x1B8D8usize,
                        0x47D58,
                        0x47D6C,
                        0x47D80,
                        0x47D94,
                        0x4BD38,
                        0x4BD58,
                        0x4BD78,
                        0x4BD98,
                    ][..],
                ),
                (
                    0x2198,
                    &[
                        0x48050usize,
                        0x48064,
                        0x48078,
                        0x4808C,
                        0x48118,
                        0x4C1D0,
                        0x4C1F4,
                        0x4C214,
                        0x4C234,
                        0x4C308,
                    ][..],
                ),
                (
                    0x2199,
                    &[
                        0x480A0usize,
                        0x480B4,
                        0x480C8,
                        0x480DC,
                        0x480F0,
                        0x48104,
                        0x4C250,
                        0x4C274,
                        0x4C294,
                        0x4C2B4,
                        0x4C2D4,
                        0x4C2F4,
                    ][..],
                ),
            ],
        ),
        (
            &CASES[2],
            [
                (
                    0x218B,
                    &[
                        0x2AE62usize,
                        0x30500,
                        0x47A98,
                        0x47AAC,
                        0x47AC0,
                        0x4B9FC,
                        0x4BA20,
                        0x4BA44,
                    ][..],
                ),
                (
                    0x218C,
                    &[
                        0x47AC8usize,
                        0x47AEC,
                        0x47B00,
                        0x47B14,
                        0x47B28,
                        0x47B64,
                        0x4BA58,
                        0x4BA78,
                        0x4BA98,
                        0x4BAB8,
                        0x4BAD8,
                        0x4BB3C,
                        0x5D320,
                        0x7FBEC,
                    ][..],
                ),
                (
                    0x218E,
                    &[
                        0x1B66Cusize,
                        0x47B80,
                        0x47B94,
                        0x47BA8,
                        0x47BBC,
                        0x4BB60,
                        0x4BB80,
                        0x4BBA0,
                        0x4BBC0,
                    ][..],
                ),
                (
                    0x2198,
                    &[
                        0x47E78usize,
                        0x47E8C,
                        0x47EA0,
                        0x47EB4,
                        0x47F40,
                        0x4BFF8,
                        0x4C01C,
                        0x4C03C,
                        0x4C05C,
                        0x4C130,
                    ][..],
                ),
                (
                    0x2199,
                    &[
                        0x47EC8usize,
                        0x47EDC,
                        0x47EF0,
                        0x47F04,
                        0x47F18,
                        0x47F2C,
                        0x4C078,
                        0x4C09C,
                        0x4C0BC,
                        0x4C0DC,
                        0x4C0FC,
                        0x4C11C,
                    ][..],
                ),
            ],
        ),
    ] {
        let rom = fs::read(local_rom_path(case.rom)).unwrap();
        for (field, references) in expected {
            assert_eq!(
                thumb_literal_references(&rom, field),
                references,
                "{} +{field:#06X}",
                case.name
            );
        }

        let (byte_218b_bits_3_4, byte_218b_bits_0_1, word_218c_bits_13_16, byte_218e_bits_3_4) =
            if case.target == "MARY_FOMT_US" {
                (0x2B0D2, 0x30770, 0x5D5DC, 0x1B8DC)
            } else {
                (0x2AE66, 0x30504, 0x5D324, 0x1B670)
            };

        // The only non-accessor users of +0x218B read bits 3..4 and 0..1;
        // slot 333 occupies bits 5..7 and therefore does not own either use.
        assert_eq!(
            &rom[byte_218b_bits_3_4..byte_218b_bits_3_4 + 8],
            &[0x00, 0x78, 0x18, 0x24, 0x04, 0x40, 0x00, 0x2C]
        );
        assert_eq!(
            &rom[byte_218b_bits_0_1..byte_218b_bits_0_1 + 8],
            &[0x00, 0x78, 0x03, 0x24, 0x04, 0x40, 0x00, 0x2C]
        );

        // The two non-accessor +0x218C paths use the same word extraction
        // (original bits 13..16); it does not overlap +0x218C bit 0 of slot 333.
        assert_eq!(
            &rom[word_218c_bits_13_16..word_218c_bits_13_16 + 8],
            &[0x13, 0x68, 0xD9, 0x03, 0x09, 0x0F, 0x01, 0x31]
        );
        let second_word_user = if case.target == "MARY_FOMT_US" {
            0x80088
        } else {
            0x7FBF0
        };
        assert_eq!(
            &rom[second_word_user..second_word_user + 8],
            &[0x13, 0x68, 0xD9, 0x03, 0x09, 0x0F, 0x01, 0x31]
        );

        // The sole non-accessor +0x218E path extracts bits 3..4. Unknown slots
        // 343 and 344 occupy bits 5 and 6..7, respectively.
        assert_eq!(
            &rom[byte_218e_bits_3_4..byte_218e_bits_3_4 + 8],
            &[0x00, 0x78, 0xC0, 0x06, 0x80, 0x0F, 0x01, 0x28]
        );
    }
}

#[test]
fn native_fomt_unknown_314_320_storage_references_only_expose_known_introductions() {
    for (case, fields, van_extractors, lou_or_ruby_extract) in [
        (
            &CASES[0],
            [
                (
                    0x2186u32,
                    &[
                        0x3EA70usize,
                        0x47AE8,
                        0x47AFC,
                        0x47B18,
                        0x47B2C,
                        0x4B970,
                        0x4B994,
                        0x4B9B4,
                        0x4B9D4,
                        0xA69F8,
                    ][..],
                ),
                (
                    0x2187,
                    &[
                        0x3E420usize,
                        0x47B40,
                        0x47B54,
                        0x47B68,
                        0x47B7C,
                        0x4B9F8,
                        0x4BA18,
                        0x4BA38,
                        0x4BA5E,
                    ][..],
                ),
                (
                    0x2188,
                    &[
                        0x47B84usize,
                        0x47BA0,
                        0x47BB4,
                        0x47BC8,
                        0x47BDC,
                        0x47C18,
                        0x4BA70,
                        0x4BA90,
                        0x4BAB0,
                        0x4BAD0,
                        0x4BAF0,
                        0x4BB54,
                    ][..],
                ),
            ],
            &[0x3EA70usize, 0xA69F8][..],
            0x3E420usize,
        ),
        (
            &CASES[2],
            [
                (
                    0x2186,
                    &[
                        0x3E6E4usize,
                        0x47910,
                        0x47924,
                        0x47940,
                        0x47954,
                        0x4B798,
                        0x4B7BC,
                        0x4B7DC,
                        0x4B7FC,
                        0xA6430,
                    ][..],
                ),
                (
                    0x2187,
                    &[
                        0x3E094usize,
                        0x47968,
                        0x4797C,
                        0x47990,
                        0x479A4,
                        0x4B820,
                        0x4B840,
                        0x4B860,
                        0x4B886,
                    ][..],
                ),
                (
                    0x2188,
                    &[
                        0x479ACusize,
                        0x479C8,
                        0x479DC,
                        0x479F0,
                        0x47A04,
                        0x47A40,
                        0x4B898,
                        0x4B8B8,
                        0x4B8D8,
                        0x4B8F8,
                        0x4B918,
                        0x4B97C,
                    ][..],
                ),
            ],
            &[0x3E6E4usize, 0xA6430][..],
            0x3E094,
        ),
    ] {
        let rom = fs::read(local_rom_path(case.rom)).unwrap();
        for (field, expected_references) in fields {
            let literal_pools: HashSet<_> = (0..0x100000usize)
                .step_by(4)
                .filter(|&offset| {
                    u32::from_le_bytes(rom[offset..offset + 4].try_into().unwrap()) == field
                })
                .collect();
            let references = (0..0xE0000usize)
                .step_by(2)
                .filter(|&instruction| {
                    let opcode =
                        u16::from_le_bytes(rom[instruction..instruction + 2].try_into().unwrap());
                    if opcode & 0xF800 != 0x4800 {
                        return false;
                    }
                    let pool = ((instruction + 4) & !3) + 4 * (opcode as usize & 0xFF);
                    literal_pools.contains(&pool)
                })
                .collect::<Vec<_>>();
            assert_eq!(
                references, expected_references,
                "{} field {field:#X}",
                case.name
            );
        }

        for &extractor in van_extractors {
            assert_eq!(
                &rom[extractor + 6..extractor + 10],
                &[0x40, 0x06, 0x80, 0x0F]
            );
        }
        assert_eq!(
            &rom[lou_or_ruby_extract + 6..lou_or_ruby_extract + 10],
            &[0xC0, 0x06, 0x80, 0x0F]
        );
    }
}

#[test]
fn native_mfomt_early_unknown_slots_are_matching_boolean_fields() {
    for (case, get_table, set_table, entries) in [
        (
            &CASES[1],
            0x46050,
            0x4A430,
            [
                (86, 0x476C0, 0x4B980, 0x2172, 7),
                (94, 0x476E8, 0x4B9C0, 0x2173, 3),
                (96, 0x476FC, 0x4B9E0, 0x2173, 4),
                (115, 0x47760, 0x4BA84, 0x2174, 5),
            ],
        ),
        (
            &CASES[3],
            0x45DAC,
            0x4A18C,
            [
                (86, 0x4741C, 0x4B6DC, 0x2172, 7),
                (94, 0x47444, 0x4B71C, 0x2173, 3),
                (96, 0x47458, 0x4B73C, 0x2173, 4),
                (115, 0x474BC, 0x4B7E0, 0x2174, 5),
            ],
        ),
    ] {
        let rom = fs::read(local_rom_path(case.rom)).unwrap();
        let word = |at| u32::from_le_bytes(rom[at..at + 4].try_into().unwrap());
        for (id, getter, setter, field, shift) in entries {
            assert_eq!(word(get_table + id * 4), 0x08000000 + getter as u32);
            assert_eq!(word(set_table + (id - 28) * 4), 0x08000000 + setter as u32);
            assert_eq!(word(getter + 16), field);
            assert!(rom[setter..setter + 48]
                .windows(4)
                .any(|bytes| bytes == field.to_le_bytes()));

            for old in 0u8..=255 {
                for input in 0u8..=255 {
                    let mask = 1 << shift;
                    let stored = (old & !mask) | ((input & 1) << shift);
                    assert_eq!((stored >> shift) & 1, input & 1);
                    assert_eq!(stored & !mask, old & !mask);
                }
            }
        }
    }
}

#[test]
fn native_mfomt_unknown_94_96_115_have_no_extra_direct_offset_literal_users() {
    for (case, fields) in [
        (
            &CASES[1],
            [
                (
                    0x2173u32,
                    &[
                        0x47628usize,
                        0x4763C,
                        0x476DC,
                        0x476F0,
                        0x47704,
                        0x47718,
                        0x4B894,
                        0x4B8B4,
                        0x4B9A4,
                        0x4B9C8,
                        0x4B9E8,
                        0x4BA08,
                    ][..],
                ),
                (
                    0x2174,
                    &[
                        0x47650usize,
                        0x47664,
                        0x4772C,
                        0x47740,
                        0x47754,
                        0x47768,
                        0x4777C,
                        0x4B8D0,
                        0x4B8F0,
                        0x4BA28,
                        0x4BA4C,
                        0x4BA6C,
                        0x4BA8C,
                        0x4BAAC,
                    ][..],
                ),
            ],
        ),
        (
            &CASES[3],
            [
                (
                    0x2173,
                    &[
                        0x47384usize,
                        0x47398,
                        0x47438,
                        0x4744C,
                        0x47460,
                        0x47474,
                        0x4B5F0,
                        0x4B610,
                        0x4B700,
                        0x4B724,
                        0x4B744,
                        0x4B764,
                    ][..],
                ),
                (
                    0x2174,
                    &[
                        0x473ACusize,
                        0x473C0,
                        0x47488,
                        0x4749C,
                        0x474B0,
                        0x474C4,
                        0x474D8,
                        0x4B62C,
                        0x4B64C,
                        0x4B784,
                        0x4B7A8,
                        0x4B7C8,
                        0x4B7E8,
                        0x4B808,
                    ][..],
                ),
            ],
        ),
    ] {
        let rom = fs::read(local_rom_path(case.rom)).unwrap();
        for (field, expected_references) in fields {
            let literal_pools: HashSet<_> = (0..0x100000usize)
                .step_by(4)
                .filter(|&offset| {
                    u32::from_le_bytes(rom[offset..offset + 4].try_into().unwrap()) == field
                })
                .collect();
            let references: Vec<_> = (0..0xE0000usize)
                .step_by(2)
                .filter(|&instruction| {
                    let opcode =
                        u16::from_le_bytes(rom[instruction..instruction + 2].try_into().unwrap());
                    if opcode & 0xF800 != 0x4800 {
                        return false;
                    }
                    let pool = ((instruction + 4) & !3) + 4 * (opcode as usize & 0xFF);
                    literal_pools.contains(&pool)
                })
                .collect();

            assert_eq!(
                references, expected_references,
                "{} field {field:#X}",
                case.name
            );
        }
    }
}

#[test]
fn native_mfomt_unknown_86_disables_two_null_schedule_branches() {
    for (
        case,
        selector_a,
        schedule_a,
        selector_b,
        schedule_b,
        may_constructor,
        may_literal,
        stu_constructor,
        stu_literal,
        var_set_dispatcher,
        var_set_caller,
    ) in [
        (
            &CASES[1], 0x3E978, 0xFE8BC, 0x3EFB8, 0x100CA4, 0x3DB0C, 0x3DCB8, 0x3DBA0, 0x3DCE4,
            0x4A3F8, 0x413E6,
        ),
        (
            &CASES[3], 0x3E6D4, 0xFE85C, 0x3ED14, 0x100C44, 0x3D868, 0x3DA14, 0x3D8FC, 0x3DA40,
            0x4A154, 0x41142,
        ),
    ] {
        let rom = fs::read(local_rom_path(case.rom)).unwrap();
        let word = |at| u32::from_le_bytes(rom[at..at + 4].try_into().unwrap());

        // Both ScheduleInfo records point at selectors that directly read
        // save byte 0x2172 and shift bit 7 down to the boolean result.
        assert_eq!(word(schedule_a), 0x08000001 + selector_a as u32);
        assert_eq!(word(schedule_b), 0x08000001 + selector_b as u32);
        assert_eq!(word(selector_a + 0x68), 0x2172);
        assert_eq!(word(selector_b + 0x5C), 0x2172);
        assert_eq!(
            &rom[selector_a + 0x5C..selector_a + 0x60],
            &[0x00, 0x78, 0xC0, 0x09]
        );
        assert_eq!(
            &rom[selector_b + 0x50..selector_b + 0x54],
            &[0x00, 0x78, 0xC0, 0x09]
        );

        // Index zero is a real null pointer in both schedule arrays. Returning
        // zero therefore suppresses the entity schedule rather than selecting
        // an ordinary route. This test intentionally does not guess the event
        // that owns or produces slot 86.
        for schedule_info in [schedule_a, schedule_b] {
            let schedule_count = word(schedule_info + 4);
            let schedule_array = word(schedule_info + 8) as usize - 0x08000000;
            assert!(schedule_count >= 1);
            assert_eq!(word(schedule_array), 0);
        }

        // The schedule manager constructs the May-shaped route table in the
        // fifth roster-order object (+0xC4) and the Stu-shaped route table in
        // the sixteenth (+0x1A8). These positions agree with character IDs 5
        // and 16, independently of the exact route-location matches in the
        // retail dialogue scripts. They identify the disabled schedules, not
        // the event that produces slot 86.
        assert_eq!(word(may_literal), 0x08000000 + schedule_b as u32);
        assert_eq!(word(stu_literal), 0x08000000 + schedule_a as u32);
        assert_eq!(
            &rom[may_constructor..may_constructor + 4],
            &[0x20, 0x1C, 0xC4, 0x30]
        );
        assert_eq!(
            &rom[stu_constructor..stu_constructor + 6],
            &[0xD4, 0x21, 0x49, 0x00, 0x60, 0x18]
        );

        // In the first MiB of executable code, the generic VarSet dispatcher
        // has one direct Thumb BL caller: the script-VM opcode handler. Thus a
        // native producer would have to use a computed/indirect call or bypass
        // VarSet; the four retail script sets contain no slot-86 access.
        let mut direct_callers = Vec::new();
        for instruction in (0..0x100000usize.min(rom.len() - 3)).step_by(2) {
            let high = u16::from_le_bytes(rom[instruction..instruction + 2].try_into().unwrap());
            let low = u16::from_le_bytes(rom[instruction + 2..instruction + 4].try_into().unwrap());
            if high & 0xF800 != 0xF000 || low & 0xF800 != 0xF800 {
                continue;
            }
            let mut displacement =
                (((high & 0x07FF) as i32) << 12) | (((low & 0x07FF) as i32) << 1);
            if displacement & (1 << 22) != 0 {
                displacement -= 1 << 23;
            }
            let target = 0x08000000i32 + instruction as i32 + 4 + displacement;
            if target == 0x08000000i32 + var_set_dispatcher {
                direct_callers.push(instruction);
            }
        }
        assert_eq!(direct_callers, [var_set_caller]);
    }
}

#[test]
fn native_mfomt_unknown_232_and_233_are_preserved_by_event_structure_copy() {
    let us = fs::read(local_rom_path(CASES[1].rom)).unwrap();
    let jp = fs::read(local_rom_path(CASES[3].rom)).unwrap();

    for (case, rom, get_table, set_table, entries) in [
        (
            &CASES[1],
            &us,
            0x46050usize,
            0x4A430usize,
            [
                (232usize, 0x47DFCusize, 0x4C468usize, 0x4A2DEusize, 3u8),
                (233, 0x47E10, 0x4C488, 0x4A2F6, 5),
            ],
        ),
        (
            &CASES[3],
            &jp,
            0x45DAC,
            0x4A18C,
            [
                (232, 0x47B58, 0x4C1C4, 0x4A03A, 3),
                (233, 0x47B6C, 0x4C1E4, 0x4A052, 5),
            ],
        ),
    ] {
        let word = |at| u32::from_le_bytes(rom[at..at + 4].try_into().unwrap());
        for (id, getter, setter, getter_leaf, shift) in entries {
            assert_eq!(word(get_table + id * 4), 0x0800_0000 + getter as u32);
            assert_eq!(word(set_table + (id - 28) * 4), 0x0800_0000 + setter as u32);
            assert_eq!(word(getter + 16), 0x2189);
            assert_eq!(thumb_call_target(rom, getter + 10), getter_leaf);
            let expected_leaf = if shift == 3 {
                [0x80, 0x18, 0x00, 0x78, 0xC0, 0x06, 0x80, 0x0F]
            } else {
                [0x80, 0x18, 0x00, 0x78, 0x40, 0x06, 0x80, 0x0F]
            };
            assert_eq!(&rom[getter_leaf..getter_leaf + 8], &expected_leaf);
            assert_eq!(word(setter + 28), 0x2189, "{} slot {id}", case.name);

            for old in 0u8..=255 {
                for input in 0u8..=255 {
                    let mask = 3u8 << shift;
                    let stored = (old & !mask) | ((input & 3) << shift);
                    assert_eq!((stored >> shift) & 3, input & 3);
                    assert_eq!(stored & !mask, old & !mask);
                }
            }
        }
    }

    // The copy path transfers the +0x25 byte in three independent two-bit
    // groups. Slots 232/233 are bits 3..4 and 5..6 respectively; this proves
    // preservation across a structure copy, not a gameplay state producer.
    let us_copy = 0xDC6F2;
    let jp_copy = 0xDC206;
    assert_eq!(&us[us_copy..us_copy + 0x40], &jp[jp_copy..jp_copy + 0x40]);
    assert_eq!(
        &us[us_copy + 0x14..us_copy + 0x3E],
        &[
            0x25, 0x30, 0x03, 0x78, 0x3D, 0x1C, 0x25, 0x35, 0x06, 0x21, 0x19, 0x40, 0x2A, 0x78,
            0x20, 0x1C, 0x10, 0x40, 0x08, 0x43, 0x18, 0x21, 0x19, 0x40, 0x42, 0x46, 0x10, 0x40,
            0x08, 0x43, 0x60, 0x21, 0x19, 0x40, 0x4B, 0x46, 0x18, 0x40, 0x08, 0x43, 0x28, 0x70,
        ]
    );
}

#[test]
fn native_mfomt_unknown_244_250_and_284_are_two_bit_fields() {
    for (case, get_table, set_table, entries) in [
        (
            &CASES[1],
            0x46050,
            0x4A430,
            [
                (244, 0x47EEC, 0x4C5DC, 0x218C, 2),
                (245, 0x47F00, 0x4C5FC, 0x218C, 4),
                (250, 0x47F64, 0x4C69C, 0x218D, 6),
                (284, 0x4822C, 0x4CAD0, 0x2195, 6),
            ],
        ),
        (
            &CASES[3],
            0x45DAC,
            0x4A18C,
            [
                (244, 0x47C48, 0x4C338, 0x218C, 2),
                (245, 0x47C5C, 0x4C358, 0x218C, 4),
                (250, 0x47CC0, 0x4C3F8, 0x218D, 6),
                (284, 0x47F88, 0x4C82C, 0x2195, 6),
            ],
        ),
    ] {
        let rom = fs::read(local_rom_path(case.rom)).unwrap();
        let word = |at| u32::from_le_bytes(rom[at..at + 4].try_into().unwrap());
        for (id, getter, setter, field, shift) in entries {
            assert_eq!(word(get_table + id * 4), 0x08000000 + getter as u32);
            assert_eq!(word(set_table + (id - 28) * 4), 0x08000000 + setter as u32);
            assert_eq!(word(getter + 16), field);
            assert!(rom[setter..setter + 48]
                .windows(4)
                .any(|bytes| bytes == field.to_le_bytes()));

            for old in 0u8..=255 {
                for input in 0u8..=255 {
                    let mask = 3 << shift;
                    let stored = (old & !mask) | ((input & 3) << shift);
                    assert_eq!((stored >> shift) & 3, input & 3);
                    assert_eq!(stored & !mask, old & !mask);
                }
            }
        }
    }
}

#[test]
fn native_mfomt_unknown_244_245_and_250_storage_bytes_have_no_direct_business_literals() {
    for (case, fields) in [
        (
            &CASES[1],
            [
                (
                    0x218Cu32,
                    &[
                        0x47EE8usize,
                        0x47EFC,
                        0x47F10,
                        0x47F24,
                        0x4C5D8,
                        0x4C5F8,
                        0x4C618,
                        0x4C634,
                    ][..],
                ),
                (
                    0x218D,
                    &[
                        0x47F38usize,
                        0x47F4C,
                        0x47F60,
                        0x47F74,
                        0x4C658,
                        0x4C678,
                        0x4C698,
                        0x4C6B4,
                    ][..],
                ),
            ],
        ),
        (
            &CASES[3],
            [
                (
                    0x218C,
                    &[
                        0x47C44usize,
                        0x47C58,
                        0x47C6C,
                        0x47C80,
                        0x4C334,
                        0x4C354,
                        0x4C374,
                        0x4C390,
                    ][..],
                ),
                (
                    0x218D,
                    &[
                        0x47C94usize,
                        0x47CA8,
                        0x47CBC,
                        0x47CD0,
                        0x4C3B4,
                        0x4C3D4,
                        0x4C3F4,
                        0x4C410,
                    ][..],
                ),
            ],
        ),
    ] {
        let rom = fs::read(local_rom_path(case.rom)).unwrap();
        for (field, expected_literals) in fields {
            let field_bytes = field.to_le_bytes();
            let actual_literals = rom[..0x100000]
                .windows(4)
                .enumerate()
                .filter_map(|(offset, bytes)| (bytes == field_bytes).then_some(offset))
                .collect::<Vec<_>>();
            assert_eq!(
                actual_literals, expected_literals,
                "{} field {field:#X}",
                case.name
            );
        }
    }
}

#[test]
fn native_mfomt_unknown_324_328_preserve_adjacent_two_bit_fields() {
    for (case, get_table, set_table, entries) in [
        (
            &CASES[1],
            0x46050,
            0x4A430,
            [
                (324, 0x4855C, 0x4CFE8, 0x219F, 5),
                (325, 0x48570, 0x4D008, 0x219F, 7),
                (327, 0x485A8, 0x4D060, 0x21A0, 3),
                (328, 0x485BC, 0x4D080, 0x21A0, 5),
            ],
        ),
        (
            &CASES[3],
            0x45DAC,
            0x4A18C,
            [
                (324, 0x482B8, 0x4CD44, 0x219F, 5),
                (325, 0x482CC, 0x4CD64, 0x219F, 7),
                (327, 0x48304, 0x4CDBC, 0x21A0, 3),
                (328, 0x48318, 0x4CDDC, 0x21A0, 5),
            ],
        ),
    ] {
        let rom = fs::read(local_rom_path(case.rom)).unwrap();
        let word = |at| u32::from_le_bytes(rom[at..at + 4].try_into().unwrap());
        for (id, getter, setter, field, shift) in entries {
            assert_eq!(word(get_table + id * 4), 0x08000000 + getter as u32);
            assert_eq!(word(set_table + (id - 28) * 4), 0x08000000 + setter as u32);
            if id == 325 {
                assert_eq!(word(getter + 28), 0x219F);
                assert_eq!(word(getter + 32), 0x21A0);
            } else {
                assert_eq!(word(getter + 16), field);
            }
            assert!(rom[setter..setter + 64]
                .windows(4)
                .any(|bytes| bytes == field.to_le_bytes()));

            for old in 0u16..=u16::MAX {
                for input in 0u16..=3 {
                    let mask = 3 << shift;
                    let stored = (old & !mask) | ((input & 3) << shift);
                    assert_eq!((stored >> shift) & 3, input & 3);
                    assert_eq!(stored & !mask, old & !mask);
                }
            }
        }
    }

    // Slot 327 is save+0x21A0 bits 3..4. Pin the leaf extraction separately
    // from adjacent known slot 326 (bits 1..2) and unknown slot 328 (bits 5..6).
    for (case, getter, getter_leaf) in
        [(&CASES[1], 0x485A8, 0x4A2DE), (&CASES[3], 0x48304, 0x4A03A)]
    {
        let rom = fs::read(local_rom_path(case.rom)).unwrap();
        assert_eq!(thumb_call_target(&rom, getter + 10), getter_leaf);
        assert_eq!(
            &rom[getter_leaf..getter_leaf + 8],
            &[0x80, 0x18, 0x00, 0x78, 0xC0, 0x06, 0x80, 0x0F]
        );
    }
}

#[test]
fn native_mfomt_unknown_324_328_storage_literals_only_expose_adjacent_known_fields() {
    for (case, fields, lou_or_ruby_extract) in [
        (
            &CASES[1],
            [
                (
                    0x219Fu32,
                    &[
                        0x3EB1Cusize,
                        0x48544,
                        0x48558,
                        0x4856C,
                        0x4858C,
                        0x4CFC4,
                        0x4CFE4,
                        0x4D004,
                        0x4D038,
                        0xABCA4,
                    ][..],
                ),
                (
                    0x21A0,
                    &[
                        0x3E6C4usize,
                        0x48590,
                        0x485A4,
                        0x485B8,
                        0x485CC,
                        0x485E0,
                        0x4D03C,
                        0x4D05C,
                        0x4D07C,
                        0x4D09C,
                        0x4D0B0,
                    ][..],
                ),
            ],
            0x3E6ACusize,
        ),
        (
            &CASES[3],
            [
                (
                    0x219F,
                    &[
                        0x3E878usize,
                        0x482A0,
                        0x482B4,
                        0x482C8,
                        0x482E8,
                        0x4CD20,
                        0x4CD40,
                        0x4CD60,
                        0x4CD94,
                        0xAB6E4,
                    ][..],
                ),
                (
                    0x21A0,
                    &[
                        0x3E420usize,
                        0x482EC,
                        0x48300,
                        0x48314,
                        0x48328,
                        0x4833C,
                        0x4CD98,
                        0x4CDB8,
                        0x4CDD8,
                        0x4CDF8,
                        0x4CE0C,
                    ][..],
                ),
            ],
            0x3E408,
        ),
    ] {
        let rom = fs::read(local_rom_path(case.rom)).unwrap();
        for (field, expected_literals) in fields {
            let field_bytes = field.to_le_bytes();
            let actual_literals = rom[..0x100000]
                .windows(4)
                .enumerate()
                .filter_map(|(offset, bytes)| (bytes == field_bytes).then_some(offset))
                .collect::<Vec<_>>();
            assert_eq!(
                actual_literals, expected_literals,
                "{} field {field:#X}",
                case.name
            );
        }

        // The only business-code literal for +0x21A0 extracts bits 1..2,
        // which belong to known slot 326, not unknown slots 327/328.
        assert_eq!(
            &rom[lou_or_ruby_extract..lou_or_ruby_extract + 16],
            &[
                0x05, 0x48, 0x24, 0x18, 0x20, 0x78, 0x40, 0x07, 0x80, 0x0F, 0x02, 0x28, 0x06, 0xD1,
                0x00, 0x29,
            ]
        );
    }
}

#[test]
fn native_mfomt_unknown_335_352_keep_mixed_width_domains() {
    for (case, get_table, set_table, entries) in [
        (
            &CASES[1],
            0x46050,
            0x4A430,
            [
                (335, 0x4864C, 0x4D154, 0x21A2, 4, 15),
                (341, 0x486C4, 0x4D210, 0x21A4, 3, 15),
                (343, 0x486EC, 0x4D244, 0x21A5, 1, 3),
                (344, 0x48700, 0x4D264, 0x21A5, 3, 3),
                (351, 0x4878C, 0x4D32C, 0x21A7, 3, 1),
                (352, 0x487A0, 0x4D34C, 0x21A7, 4, 3),
            ],
        ),
        (
            &CASES[3],
            0x45DAC,
            0x4A18C,
            [
                (335, 0x483A8, 0x4CEB0, 0x21A2, 4, 15),
                (341, 0x48420, 0x4CF6C, 0x21A4, 3, 15),
                (343, 0x48448, 0x4CFA0, 0x21A5, 1, 3),
                (344, 0x4845C, 0x4CFC0, 0x21A5, 3, 3),
                (351, 0x484E8, 0x4D088, 0x21A7, 3, 1),
                (352, 0x484FC, 0x4D0A8, 0x21A7, 4, 3),
            ],
        ),
    ] {
        let rom = fs::read(local_rom_path(case.rom)).unwrap();
        let word = |at| u32::from_le_bytes(rom[at..at + 4].try_into().unwrap());
        for (id, getter, setter, field, shift, input_mask) in entries {
            assert_eq!(word(get_table + id * 4), 0x08000000 + getter as u32);
            assert_eq!(word(set_table + (id - 28) * 4), 0x08000000 + setter as u32);
            assert_eq!(word(getter + 16), field);
            assert!(rom[setter..setter + 48]
                .windows(4)
                .any(|bytes| bytes == field.to_le_bytes()));

            for old in 0u8..=255 {
                for input in 0u8..=255 {
                    let mask = input_mask << shift;
                    let stored = (old & !mask) | ((input & input_mask) << shift);
                    assert_eq!((stored >> shift) & input_mask, input & input_mask);
                    assert_eq!(stored & !mask, old & !mask);
                }
            }
        }
    }
}

#[test]
fn native_mfomt_unknown_335_344_storage_literals_only_expose_adjacent_jewel_fields() {
    for (case, fields, goddess_extract, kappa_extract) in [
        (
            &CASES[1],
            [
                (
                    0x21A2u32,
                    &[
                        0x2B328usize,
                        0x48634,
                        0x48648,
                        0x4865C,
                        0x4D130,
                        0x4D150,
                        0x4D16C,
                    ][..],
                ),
                (
                    0x21A4,
                    &[
                        0x2B330usize,
                        0x486C0,
                        0x486D4,
                        0x486E8,
                        0x48738,
                        0x4D20C,
                        0x4D22C,
                        0x4D240,
                        0x4D2B4,
                    ][..],
                ),
                (
                    0x21A5,
                    &[0x486FCusize, 0x48710, 0x48724, 0x4D260, 0x4D280, 0x4D2A0][..],
                ),
            ],
            0x2B260usize,
            0x2B2BEusize,
        ),
        (
            &CASES[3],
            [
                (
                    0x21A2,
                    &[
                        0x2B19Cusize,
                        0x48390,
                        0x483A4,
                        0x483B8,
                        0x4CE8C,
                        0x4CEAC,
                        0x4CEC8,
                    ][..],
                ),
                (
                    0x21A4,
                    &[
                        0x2B1A4usize,
                        0x4841C,
                        0x48430,
                        0x48444,
                        0x48494,
                        0x4CF68,
                        0x4CF88,
                        0x4CF9C,
                        0x4D010,
                    ][..],
                ),
                (
                    0x21A5,
                    &[0x48458usize, 0x4846C, 0x48480, 0x4CFBC, 0x4CFDC, 0x4CFFC][..],
                ),
            ],
            0x2B0D4,
            0x2B132,
        ),
    ] {
        let rom = fs::read(local_rom_path(case.rom)).unwrap();
        for (field, expected_literals) in fields {
            let field_bytes = field.to_le_bytes();
            let actual_literals = rom[..0x100000]
                .windows(4)
                .enumerate()
                .filter_map(|(offset, bytes)| (bytes == field_bytes).then_some(offset))
                .collect::<Vec<_>>();
            assert_eq!(
                actual_literals, expected_literals,
                "{} field {field:#X}",
                case.name
            );
        }

        let common_prefix = [0x68, 0x6B];
        assert_eq!(&rom[goddess_extract..goddess_extract + 2], &common_prefix);
        assert_eq!(
            &rom[goddess_extract + 8..goddess_extract + 12],
            &[0x0C, 0x24, 0x04, 0x40]
        );
        assert_eq!(&rom[kappa_extract..kappa_extract + 2], &common_prefix);
        assert_eq!(
            &rom[kappa_extract + 8..kappa_extract + 12],
            &[0x06, 0x24, 0x04, 0x40]
        );
    }
}

#[test]
fn native_mfomt_unknown_351_and_352_are_not_the_same_byte_shooting_star_selector() {
    for (
        case,
        get_table,
        known_getter,
        known_leaf,
        unknown_351_getter,
        unknown_352_getter,
        unknown_352_leaf,
        business_read,
        expected_literals,
    ) in [
        (
            &CASES[1],
            0x46050usize,
            0x48778usize,
            0x4A2C6usize,
            0x4878Cusize,
            0x487A0usize,
            0x494E2usize,
            0x1BA56usize,
            &[
                0x1BA78usize,
                0x48788,
                0x4879C,
                0x487B0,
                0x487C4,
                0x4D328,
                0x4D348,
                0x4D368,
                0x4D384,
            ][..],
        ),
        (
            &CASES[3],
            0x45DAC,
            0x484D4,
            0x4A022,
            0x484E8,
            0x484FC,
            0x4923E,
            0x1B8CA,
            &[
                0x1B8ECusize,
                0x484E4,
                0x484F8,
                0x4850C,
                0x48520,
                0x4D084,
                0x4D0A4,
                0x4D0C4,
                0x4D0E0,
            ][..],
        ),
    ] {
        let rom = fs::read(local_rom_path(case.rom)).unwrap();
        let word = |at| u32::from_le_bytes(rom[at..at + 4].try_into().unwrap());

        assert_eq!(word(get_table + 350 * 4), 0x0800_0000 + known_getter as u32);
        assert_eq!(
            word(get_table + 351 * 4),
            0x0800_0000 + unknown_351_getter as u32
        );
        assert_eq!(
            word(get_table + 352 * 4),
            0x0800_0000 + unknown_352_getter as u32
        );
        assert_eq!(word(known_getter + 16), 0x21A7);
        assert_eq!(word(unknown_351_getter + 16), 0x21A7);
        assert_eq!(word(unknown_352_getter + 16), 0x21A7);
        assert_eq!(thumb_call_target(&rom, known_getter + 10), known_leaf);
        assert_eq!(
            thumb_call_target(&rom, unknown_352_getter + 10),
            unknown_352_leaf
        );
        assert_eq!(
            &rom[known_leaf..known_leaf + 8],
            &[0x80, 0x18, 0x00, 0x78, 0x40, 0x07, 0x80, 0x0F]
        );
        assert_eq!(
            &rom[unknown_352_leaf..unknown_352_leaf + 8],
            &[0x80, 0x18, 0x00, 0x78, 0x80, 0x06, 0x80, 0x0F]
        );
        assert_eq!(
            &rom[business_read..business_read + 18],
            &[
                0x07, 0x49, 0x70, 0x18, 0x00, 0x68, 0x06, 0x49, 0x40, 0x18, 0x00, 0x78, 0x40, 0x07,
                0x80, 0x0F, 0x01, 0x28,
            ]
        );

        let field_bytes = 0x21A7u32.to_le_bytes();
        let actual_literals = rom[..0x100000]
            .windows(4)
            .enumerate()
            .filter_map(|(offset, bytes)| (bytes == field_bytes).then_some(offset))
            .collect::<Vec<_>>();
        assert_eq!(actual_literals, expected_literals, "{}", case.name);
    }
}

#[test]
fn native_mfomt_unknown_411_415_are_two_bit_fields() {
    for (case, get_table, set_table, entries) in [
        (
            &CASES[1],
            0x46050,
            0x4A430,
            [
                (411, 0x48BC4, 0x4D9EC, 0x21B6, 2),
                (412, 0x48BD8, 0x4DA0C, 0x21B6, 4),
                (413, 0x48BEC, 0x4DA2C, 0x21B6, 6),
                (415, 0x48C14, 0x4DA6C, 0x21B7, 2),
            ],
        ),
        (
            &CASES[3],
            0x45DAC,
            0x4A18C,
            [
                (411, 0x48920, 0x4D748, 0x21B6, 2),
                (412, 0x48934, 0x4D768, 0x21B6, 4),
                (413, 0x48948, 0x4D788, 0x21B6, 6),
                (415, 0x48970, 0x4D7C8, 0x21B7, 2),
            ],
        ),
    ] {
        let rom = fs::read(local_rom_path(case.rom)).unwrap();
        let word = |at| u32::from_le_bytes(rom[at..at + 4].try_into().unwrap());
        for (id, getter, setter, field, shift) in entries {
            assert_eq!(word(get_table + id * 4), 0x08000000 + getter as u32);
            assert_eq!(word(set_table + (id - 28) * 4), 0x08000000 + setter as u32);
            assert_eq!(word(getter + 16), field);
            assert!(rom[setter..setter + 48]
                .windows(4)
                .any(|bytes| bytes == field.to_le_bytes()));

            for old in 0u8..=255 {
                for input in 0u8..=255 {
                    let mask = 3 << shift;
                    let stored = (old & !mask) | ((input & 3) << shift);
                    assert_eq!((stored >> shift) & 3, input & 3);
                    assert_eq!(stored & !mask, old & !mask);
                }
            }
        }
    }
}

#[test]
fn native_mfomt_unknown_411_415_storage_bytes_have_no_direct_business_literals() {
    for (case, fields) in [
        (
            &CASES[1],
            [
                (
                    0x21B6u32,
                    &[
                        0x48BC0usize,
                        0x48BD4,
                        0x48BE8,
                        0x48BFC,
                        0x4D9E8,
                        0x4DA08,
                        0x4DA28,
                        0x4DA44,
                    ][..],
                ),
                (
                    0x21B7,
                    &[
                        0x48C10usize,
                        0x48C24,
                        0x48C38,
                        0x48C4C,
                        0x48C60,
                        0x48C74,
                        0x4DA68,
                        0x4DA88,
                        0x4DAA8,
                        0x4DAC8,
                        0x4DADC,
                        0x4DAF8,
                    ][..],
                ),
            ],
        ),
        (
            &CASES[3],
            [
                (
                    0x21B6,
                    &[
                        0x4891Cusize,
                        0x48930,
                        0x48944,
                        0x48958,
                        0x4D744,
                        0x4D764,
                        0x4D784,
                        0x4D7A0,
                    ][..],
                ),
                (
                    0x21B7,
                    &[
                        0x4896Cusize,
                        0x48980,
                        0x48994,
                        0x489A8,
                        0x489BC,
                        0x489D0,
                        0x4D7C4,
                        0x4D7E4,
                        0x4D804,
                        0x4D824,
                        0x4D838,
                        0x4D854,
                    ][..],
                ),
            ],
        ),
    ] {
        let rom = fs::read(local_rom_path(case.rom)).unwrap();
        for (field, expected_literals) in fields {
            let field_bytes = field.to_le_bytes();
            let actual_literals = rom[..0x100000]
                .windows(4)
                .enumerate()
                .filter_map(|(offset, bytes)| (bytes == field_bytes).then_some(offset))
                .collect::<Vec<_>>();
            assert_eq!(
                actual_literals, expected_literals,
                "{} field {field:#X}",
                case.name
            );
        }
    }
}

#[test]
fn native_mfomt_unknown_482_490_are_untyped_two_bit_fields() {
    for (case, get_table, set_table, entries) in [
        (
            &CASES[1],
            0x46050,
            0x4A430,
            [
                (482, 0x49140, 0x4E170, 0x21C5, 0),
                (484, 0x49164, 0x4E1B4, 0x21C5, 3),
                (486, 0x4918C, 0x4E1F4, 0x21C5, 6),
                (488, 0x491B0, 0x4E234, 0x21C6, 1),
                (490, 0x491D8, 0x4E274, 0x21C6, 4),
            ],
        ),
        (
            &CASES[3],
            0x45DAC,
            0x4A18C,
            [
                (482, 0x48E9C, 0x4DECC, 0x21C5, 0),
                (484, 0x48EC0, 0x4DF10, 0x21C5, 3),
                (486, 0x48EE8, 0x4DF50, 0x21C5, 6),
                (488, 0x48F0C, 0x4DF90, 0x21C6, 1),
                (490, 0x48F34, 0x4DFD0, 0x21C6, 4),
            ],
        ),
    ] {
        let rom = fs::read(local_rom_path(case.rom)).unwrap();
        let word = |at| u32::from_le_bytes(rom[at..at + 4].try_into().unwrap());
        for (id, getter, setter, field, shift) in entries {
            assert_eq!(word(get_table + id * 4), 0x08000000 + getter as u32);
            assert_eq!(word(set_table + (id - 28) * 4), 0x08000000 + setter as u32);
            let getter_literal = if matches!(id, 482 | 486 | 490) {
                12
            } else {
                16
            };
            assert_eq!(word(getter + getter_literal), field);
            assert!(rom[setter..setter + 48]
                .windows(4)
                .any(|bytes| bytes == field.to_le_bytes()));

            for old in 0u8..=255 {
                for input in 0u8..=255 {
                    let mask = 3 << shift;
                    let stored = (old & !mask) | ((input & 3) << shift);
                    assert_eq!((stored >> shift) & 3, input & 3);
                    assert_eq!(stored & !mask, old & !mask);
                }
            }
        }
    }
}

#[test]
fn native_mfomt_unknown_482_490_have_no_direct_business_literal_reference() {
    for (case, fields) in [
        (
            &CASES[1],
            [
                (
                    0x21C5u32,
                    &[
                        0x4914Cusize,
                        0x49160,
                        0x49174,
                        0x49188,
                        0x49198,
                        0x4E190,
                        0x4E1B0,
                        0x4E1D0,
                        0x4E1F0,
                        0x4E20C,
                    ][..],
                ),
                (
                    0x21C6u32,
                    &[
                        0x491ACusize,
                        0x491C0,
                        0x491D4,
                        0x491E4,
                        0x491F8,
                        0x4920C,
                        0x4E230,
                        0x4E250,
                        0x4E270,
                        0x4E290,
                        0x4E2A4,
                        0x4E2B8,
                    ][..],
                ),
            ],
        ),
        (
            &CASES[3],
            [
                (
                    0x21C5u32,
                    &[
                        0x48EA8usize,
                        0x48EBC,
                        0x48ED0,
                        0x48EE4,
                        0x48EF4,
                        0x4DEEC,
                        0x4DF0C,
                        0x4DF2C,
                        0x4DF4C,
                        0x4DF68,
                    ][..],
                ),
                (
                    0x21C6u32,
                    &[
                        0x48F08usize,
                        0x48F1C,
                        0x48F30,
                        0x48F40,
                        0x48F54,
                        0x48F68,
                        0x4DF8C,
                        0x4DFAC,
                        0x4DFCC,
                        0x4DFEC,
                        0x4E000,
                        0x4E014,
                    ][..],
                ),
            ],
        ),
    ] {
        let rom = fs::read(local_rom_path(case.rom)).unwrap();
        for (field, expected_literals) in fields {
            let actual_literals = rom[..0x100000]
                .windows(4)
                .enumerate()
                .filter_map(|(offset, bytes)| (bytes == field.to_le_bytes()).then_some(offset))
                .collect::<Vec<_>>();
            assert_eq!(
                actual_literals, expected_literals,
                "{} field {field:#X}",
                case.name
            );
        }
    }
}

#[test]
fn native_mfomt_reserved_link_584_591_and_child_profile_628_preserve_physical_domains() {
    for (case, get_table, set_table, entries) in [
        (
            &CASES[1],
            0x46050,
            0x4A430,
            [
                (584, 0x49970, 0x4EE10, 0x21E5, 7, 1),
                (585, 0x49984, 0x4EE24, 0x21E5, 0, 1),
                (586, 0x49998, 0x4EE40, 0x21E6, 1, 1),
                (587, 0x499AC, 0x4EE64, 0x21E6, 2, 1),
                (588, 0x499C0, 0x4EE84, 0x21E6, 3, 1),
                (589, 0x499D4, 0x4EEA4, 0x21E6, 4, 1),
                (590, 0x499E8, 0x4EEC4, 0x21E6, 5, 1),
                (591, 0x499FC, 0x4EEE4, 0x21E6, 6, 1),
                (628, 0x49D00, 0x4F364, 0x21D1, 3, 3),
            ],
        ),
        (
            &CASES[3],
            0x45DAC,
            0x4A18C,
            [
                (584, 0x496CC, 0x4EB6C, 0x21E5, 7, 1),
                (585, 0x496E0, 0x4EB80, 0x21E5, 0, 1),
                (586, 0x496F4, 0x4EB9C, 0x21E6, 1, 1),
                (587, 0x49708, 0x4EBC0, 0x21E6, 2, 1),
                (588, 0x4971C, 0x4EBE0, 0x21E6, 3, 1),
                (589, 0x49730, 0x4EC00, 0x21E6, 4, 1),
                (590, 0x49744, 0x4EC20, 0x21E6, 5, 1),
                (591, 0x49758, 0x4EC40, 0x21E6, 6, 1),
                (628, 0x49A5C, 0x4F0C0, 0x21D1, 3, 3),
            ],
        ),
    ] {
        let rom = fs::read(local_rom_path(case.rom)).unwrap();
        let word = |at| u32::from_le_bytes(rom[at..at + 4].try_into().unwrap());
        for (id, getter, setter, field, shift, input_mask) in entries {
            assert_eq!(word(get_table + id * 4), 0x08000000 + getter as u32);
            assert_eq!(word(set_table + (id - 28) * 4), 0x08000000 + setter as u32);
            let literal_offset = if id == 628 { 12 } else { 16 };
            assert_eq!(word(getter + literal_offset), field);
            assert!(rom[setter..setter + 48]
                .windows(4)
                .any(|bytes| bytes == field.to_le_bytes()));

            for old in 0u8..=255 {
                for input in 0u8..=255 {
                    let mask = input_mask << shift;
                    let stored = (old & !mask) | ((input & input_mask) << shift);
                    assert_eq!((stored >> shift) & input_mask, input & input_mask);
                    assert_eq!(stored & !mask, old & !mask);
                }
            }
        }
    }
}

#[test]
fn native_awl_child_profile_marker_uses_the_previously_unnamed_variable_on_all_targets() {
    // Each retail ROM contains exactly one literal <321> profile-text marker.
    // The reference-guide filter divides that marker by ten, dispatches quotient
    // 32 through a 37-entry jump table, and compares the final digit with the
    // stored two-bit level. The case-32 leaf independently reaches VarGet 536
    // in FoMT and VarGet 628 in MFoMT; this is not inferred from adjacent IDs.
    for (case, function, table_pointer, table, child_leaf, getter_table, child_id, getter) in [
        (
            &CASES[0], 0x7A2AC, 0x7A2CC, 0x7A2D0, 0x7A4E4, 0x45934, 536, 0x48C2C,
        ),
        (
            &CASES[1], 0x7F164, 0x7F184, 0x7F188, 0x7F3C8, 0x46050, 628, 0x49D00,
        ),
        (
            &CASES[2], 0x79E54, 0x79E74, 0x79E78, 0x7A08C, 0x4575C, 536, 0x48A54,
        ),
        (
            &CASES[3], 0x7ED44, 0x7ED64, 0x7ED68, 0x7EFA8, 0x45DAC, 628, 0x49A5C,
        ),
    ] {
        let rom = fs::read(local_rom_path(case.rom)).unwrap();
        let word = |at| u32::from_le_bytes(rom[at..at + 4].try_into().unwrap());

        assert_eq!(&rom[function..function + 2], &[0x70, 0xB5]);
        assert_eq!(word(table_pointer), 0x08000000 + table as u32);
        assert_eq!(word(table + 32 * 4), 0x08000000 + child_leaf as u32);
        assert_eq!(
            word(getter_table + child_id * 4),
            0x08000000 + getter as u32
        );

        let marker_offsets = rom
            .windows(5)
            .enumerate()
            .filter_map(|(offset, bytes)| (bytes == b"<321>").then_some(offset))
            .collect::<Vec<_>>();
        assert_eq!(marker_offsets.len(), 1, "{} Child marker", case.name);

        if child_id == 536 {
            // save+0x21AB, bits 6..7
            assert_eq!(
                &rom[child_leaf..child_leaf + 16],
                &[
                    0xA0, 0x68, 0x02, 0x49, 0x40, 0x18, 0x00, 0x78, 0x84, 0x09, 0x26, 0xE0, 0xAB,
                    0x21, 0x00, 0x00
                ]
            );
        } else {
            // save+0x21D1, bits 3..4
            assert_eq!(
                &rom[child_leaf..child_leaf + 14],
                &[
                    0xA0, 0x68, 0x03, 0x4A, 0x80, 0x18, 0x00, 0x78, 0xC0, 0x06, 0x84, 0x0F, 0x27,
                    0xE0
                ]
            );
            assert_eq!(word(child_leaf + 16), 0x21D1);
        }
    }
}

#[test]
fn native_late_profile_and_unreferenced_unknown_slots_keep_target_specific_widths() {
    for (case, get_table, set_table, id, getter, setter, field, shift, input_mask) in [
        (
            &CASES[0], 0x45934, 0x49034, 536, 0x48C2C, 0x4D410, 0x21AB, 6, 3,
        ),
        (
            &CASES[2], 0x4575C, 0x48E5C, 536, 0x48A54, 0x4D238, 0x21AB, 6, 3,
        ),
        (
            &CASES[1], 0x46050, 0x4A430, 322, 0x48534, 0x4CFA4, 0x219F, 0, 7,
        ),
        (
            &CASES[3], 0x45DAC, 0x4A18C, 322, 0x48290, 0x4CD00, 0x219F, 0, 7,
        ),
    ] {
        let rom = fs::read(local_rom_path(case.rom)).unwrap();
        let word = |at| u32::from_le_bytes(rom[at..at + 4].try_into().unwrap());
        assert_eq!(word(get_table + id * 4), 0x08000000 + getter as u32);
        assert_eq!(word(set_table + (id - 28) * 4), 0x08000000 + setter as u32);
        let getter_literal = if id == 536 { 12 } else { 16 };
        assert_eq!(word(getter + getter_literal), field);
        assert!(rom[setter..setter + 48]
            .windows(4)
            .any(|bytes| bytes == field.to_le_bytes()));

        for old in 0u8..=255 {
            for input in 0u8..=255 {
                let mask = input_mask << shift;
                let stored = (old & !mask) | ((input & input_mask) << shift);
                assert_eq!((stored >> shift) & input_mask, input & input_mask);
                assert_eq!(stored & !mask, old & !mask);
            }
        }
    }
}

#[test]
fn native_mfomt_unknown_584_591_storage_bytes_have_no_direct_business_literals() {
    for (case, fields) in [
        (
            &CASES[1],
            [
                (
                    0x21E5u32,
                    &[
                        0x49908usize,
                        0x4991C,
                        0x49930,
                        0x49944,
                        0x49958,
                        0x4996C,
                        0x49980,
                        0x49994,
                        0x4ED6C,
                        0x4ED8C,
                        0x4EDAC,
                        0x4EDCC,
                        0x4EDEC,
                        0x4EE0C,
                        0x4EE20,
                        0x4EE3C,
                    ][..],
                ),
                (
                    0x21E6,
                    &[
                        0x499A8usize,
                        0x499BC,
                        0x499D0,
                        0x499E4,
                        0x499F8,
                        0x49A0C,
                        0x49A20,
                        0x49A34,
                        0x4EE60,
                        0x4EE80,
                        0x4EEA0,
                        0x4EEC0,
                        0x4EEE0,
                        0x4EF00,
                        0x4EF14,
                        0x4EF30,
                    ][..],
                ),
            ],
        ),
        (
            &CASES[3],
            [
                (
                    0x21E5,
                    &[
                        0x49664usize,
                        0x49678,
                        0x4968C,
                        0x496A0,
                        0x496B4,
                        0x496C8,
                        0x496DC,
                        0x496F0,
                        0x4EAC8,
                        0x4EAE8,
                        0x4EB08,
                        0x4EB28,
                        0x4EB48,
                        0x4EB68,
                        0x4EB7C,
                        0x4EB98,
                    ][..],
                ),
                (
                    0x21E6,
                    &[
                        0x49704usize,
                        0x49718,
                        0x4972C,
                        0x49740,
                        0x49754,
                        0x49768,
                        0x4977C,
                        0x49790,
                        0x4EBBC,
                        0x4EBDC,
                        0x4EBFC,
                        0x4EC1C,
                        0x4EC3C,
                        0x4EC5C,
                        0x4EC70,
                        0x4EC8C,
                    ][..],
                ),
            ],
        ),
    ] {
        let rom = fs::read(local_rom_path(case.rom)).unwrap();
        for (field, expected_literals) in fields {
            let field_bytes = field.to_le_bytes();
            let actual_literals = rom[..0x100000]
                .windows(4)
                .enumerate()
                .filter_map(|(offset, bytes)| (bytes == field_bytes).then_some(offset))
                .collect::<Vec<_>>();
            assert_eq!(
                actual_literals, expected_literals,
                "{} field {field:#X}",
                case.name
            );
        }
    }
}

#[test]
fn native_mfomt_unknown_697_and_705_are_distinct_boolean_fields_without_direct_users() {
    for (case, get_table, set_table, entries) in [
        (
            &CASES[1],
            0x46050usize,
            0x4A430usize,
            [
                (
                    697usize,
                    0x4A294usize,
                    0x4FD30usize,
                    0x21F9u32,
                    6u8,
                    0x4A3E0usize,
                    &[
                        0x4A158usize,
                        0x4A168,
                        0x4A178,
                        0x4A188,
                        0x4A290,
                        0x4A2A0,
                        0x4FAFC,
                        0x4FB1C,
                        0x4FB3C,
                        0x4FB5C,
                        0x4FD2C,
                        0x4FD3C,
                    ][..],
                ),
                (
                    705,
                    0x4A1AC,
                    0x4FBA0,
                    0x21FB,
                    3,
                    0x4A39E,
                    &[
                        0x4A198usize,
                        0x4A1A8,
                        0x4A1B8,
                        0x4A1C8,
                        0x4A1D8,
                        0x4A1E8,
                        0x4A1F8,
                        0x4FB7C,
                        0x4FB9C,
                        0x4FBBC,
                        0x4FBDC,
                        0x4FBFC,
                        0x4FC0C,
                        0x4FC24,
                    ][..],
                ),
            ],
        ),
        (
            &CASES[3],
            0x45DAC,
            0x4A18C,
            [
                (
                    697,
                    0x49FF0,
                    0x4FA8C,
                    0x21F9,
                    6,
                    0x4A13C,
                    &[
                        0x49EB4usize,
                        0x49EC4,
                        0x49ED4,
                        0x49EE4,
                        0x49FEC,
                        0x49FFC,
                        0x4F858,
                        0x4F878,
                        0x4F898,
                        0x4F8B8,
                        0x4FA88,
                        0x4FA98,
                    ][..],
                ),
                (
                    705,
                    0x49F08,
                    0x4F8FC,
                    0x21FB,
                    3,
                    0x4A0FA,
                    &[
                        0x49EF4usize,
                        0x49F04,
                        0x49F14,
                        0x49F24,
                        0x49F34,
                        0x49F44,
                        0x49F54,
                        0x4F8D8,
                        0x4F8F8,
                        0x4F918,
                        0x4F938,
                        0x4F958,
                        0x4F968,
                        0x4F980,
                    ][..],
                ),
            ],
        ),
    ] {
        let rom = fs::read(local_rom_path(case.rom)).unwrap();
        let word = |at| u32::from_le_bytes(rom[at..at + 4].try_into().unwrap());
        for (id, getter, setter, field, shift, getter_leaf, expected_literals) in entries {
            assert_eq!(word(get_table + id * 4), 0x0800_0000 + getter as u32);
            assert_eq!(word(set_table + (id - 28) * 4), 0x0800_0000 + setter as u32);
            assert_eq!(word(getter + 12), field);
            let expected_getter = if shift == 6 {
                [0x80, 0x18, 0x00, 0x78, 0x40, 0x06, 0xC0, 0x0F]
            } else {
                [0x80, 0x18, 0x00, 0x78, 0x00, 0x07, 0xC0, 0x0F]
            };
            assert_eq!(&rom[getter_leaf..getter_leaf + 8], &expected_getter);

            if id == 697 {
                let setter_leaf = if case.name == "mfomt-us" {
                    0x4FEDA
                } else {
                    0x4FC36
                };
                assert_eq!(
                    &rom[setter_leaf..setter_leaf + 20],
                    &[
                        0x89, 0x18, 0x01, 0x20, 0x06, 0x40, 0xB3, 0x01, 0x0A, 0x78, 0x41, 0x20,
                        0x40, 0x42, 0x10, 0x40, 0x18, 0x43, 0x08, 0x70,
                    ]
                );
            } else {
                assert_eq!(
                    &rom[setter + 12..setter + 24],
                    &[0x01, 0x20, 0x06, 0x40, 0xF3, 0x00, 0x0A, 0x78, 0x09, 0x20, 0x40, 0x42]
                );
            }

            let field_bytes = field.to_le_bytes();
            let actual_literals = rom[..0x100000]
                .windows(4)
                .enumerate()
                .filter_map(|(offset, bytes)| (bytes == field_bytes).then_some(offset))
                .collect::<Vec<_>>();
            assert_eq!(
                actual_literals, expected_literals,
                "{} slot {id} storage literals escaped the audited variable accessor cluster",
                case.name
            );

            for old in 0u8..=255 {
                for input in 0u8..=255 {
                    let mask = 1u8 << shift;
                    let stored = (old & !mask) | ((input & 1) << shift);
                    assert_eq!((stored >> shift) & 1, input & 1);
                    assert_eq!(stored & !mask, old & !mask);
                }
            }
        }
    }
}

#[test]
fn native_mfomt_van_schedule_reads_slot_323_not_unknown_slot_322() {
    for (
        case,
        schedule_gate,
        schedule_literal,
        map_overlay,
        overlay_literal,
        npc_ctor_literals,
        schedule_table,
    ) in [
        (
            &CASES[1],
            0x3EAF0usize,
            0x3EB1Cusize,
            0xABC3Cusize,
            0xABCA4usize,
            0x370BCusize,
            0xFF1D0usize,
        ),
        (
            &CASES[3],
            0x3E84Cusize,
            0x3E878usize,
            0xAB67Cusize,
            0xAB6E4usize,
            0x36F30usize,
            0xFF170usize,
        ),
    ] {
        let rom = fs::read(local_rom_path(case.rom)).unwrap();
        let word = |at| u32::from_le_bytes(rom[at..at + 4].try_into().unwrap());

        assert_eq!(
            word(schedule_literal),
            0x219F,
            "{} schedule field",
            case.name
        );
        assert_eq!(word(overlay_literal), 0x219F, "{} overlay field", case.name);
        // <<27 >>30 extracts bits 3..4 (slot 323), not bits 0..2 (slot 322).
        assert_eq!(
            &rom[schedule_gate + 0x18..schedule_gate + 0x20],
            &[0x20, 0x78, 0xC0, 0x06, 0x80, 0x0F, 0x02, 0x28],
            "{} Van schedule extraction",
            case.name,
        );
        assert_eq!(
            &rom[map_overlay + 4..map_overlay + 12],
            &[0x00, 0x78, 0xC0, 0x06, 0x80, 0x0F, 0x02, 0x28],
            "{} Van map-overlay extraction",
            case.name,
        );
        assert_eq!(word(npc_ctor_literals), 0x0800_0000 + schedule_table as u32);
        assert_eq!(
            word(npc_ctor_literals + 4),
            2324,
            "{} Van idle animation",
            case.name
        );
        assert_eq!(word(schedule_table), 0x0800_0001 + schedule_gate as u32);
    }

    let us = fs::read(local_rom_path(CASES[1].rom)).unwrap();
    let jp = fs::read(local_rom_path(CASES[3].rom)).unwrap();
    let us_copy = 0xDCCB0usize;
    let jp_copy = 0xDC7C4usize;
    // The save-structure copy treats bits 0..2 and bits 3..4 as separate
    // fields before preserving the remaining groups. It copies values; it
    // does not establish a gameplay producer or semantic name for slot 322.
    assert_eq!(&us[us_copy..us_copy + 0x54], &jp[jp_copy..jp_copy + 0x54]);
    assert_eq!(
        &us[us_copy..us_copy + 0x30],
        &[
            0x50, 0x46, 0x3B, 0x30, 0x02, 0x78, 0x50, 0x07, 0x3B, 0x23, 0xDB, 0x19, 0x98, 0x46,
            0x40, 0x0F, 0x19, 0x78, 0x47, 0x3D, 0x0D, 0x40, 0x05, 0x43, 0x18, 0x20, 0x10, 0x40,
            0x19, 0x21, 0x49, 0x42, 0x0D, 0x40, 0x05, 0x43, 0x60, 0x20, 0x10, 0x40, 0x61, 0x23,
            0x5B, 0x42, 0x1D, 0x40, 0x05, 0x43,
        ]
    );
}

#[test]
fn native_unknown_cross_byte_setter_prefixes_match_regions() {
    for (us_case, jp_case, us_table, jp_table, entries) in [
        (
            &CASES[0],
            &CASES[2],
            0x49034,
            0x48E5C,
            vec![
                (314, 0x4B98C, 0x4B7B4, 32),
                (320, 0x4BA50, 0x4B878, 56),
                (327, 0x4BB4C, 0x4B974, 36),
                (333, 0x4BC10, 0x4BA38, 56),
            ],
        ),
        (
            &CASES[1],
            &CASES[3],
            0x4A430,
            0x4A18C,
            vec![(325, 0x4D008, 0x4CD64, 56)],
        ),
    ] {
        let us = fs::read(local_rom_path(us_case.rom)).unwrap();
        let jp = fs::read(local_rom_path(jp_case.rom)).unwrap();
        for (id, us_entry, jp_entry, length) in entries {
            for (rom, table, expected) in [(&us, us_table, us_entry), (&jp, jp_table, jp_entry)] {
                let p = table + (id - 28) * 4;
                assert_eq!(
                    u32::from_le_bytes(rom[p..p + 4].try_into().unwrap()) as usize,
                    0x08000000 + expected
                );
            }
            // Compare masks, shifts, partial stores and trailing literals.
            // The actual shared writeback instructions are checked below.
            assert_eq!(
                &us[us_entry..us_entry + length],
                &jp[jp_entry..jp_entry + length],
                "{} unknown variable {id}",
                us_case.name
            );
            let (call_offset, expected_store): (usize, &[u8]) = match id {
                314 => (0x18, &[0x10, 0x40, 0x18, 0x43, 0x08, 0x70]),
                320 | 325 => (0x2C, &[0x10, 0x40, 0x08, 0x43, 0x18, 0x70]),
                327 => (0x16, &[0x10, 0x40, 0x08, 0x43, 0x18, 0x60]),
                333 => (0x2A, &[0x10, 0x40, 0x08, 0x43, 0x18, 0x70]),
                _ => unreachable!(),
            };
            for (rom, entry) in [(&us, us_entry), (&jp, jp_entry)] {
                let pc = entry + call_offset;
                let hi = u16::from_le_bytes(rom[pc..pc + 2].try_into().unwrap());
                let lo = u16::from_le_bytes(rom[pc + 2..pc + 4].try_into().unwrap());
                assert_eq!(hi & 0xF800, 0xF000);
                assert_eq!(lo & 0xF800, 0xF800);
                let delta =
                    (((i32::from(hi & 0x7FF) << 12) | (i32::from(lo & 0x7FF) << 1)) << 9) >> 9;
                let tail = (pc as i32 + 4 + delta) as usize;
                assert_eq!(&rom[tail..tail + expected_store.len()], expected_store);
            }
        }
    }
}

#[test]
fn native_event_initialization_clears_family_specific_unknown_storage() {
    for (case, calls, constructor, clear, base, preserve) in [
        (&CASES[0], [0x1047A, 0x1171E], 0x9C6BC, 0x9CB2E, 0x214C, 12),
        (&CASES[2], [0x1045A, 0x116FE], 0x9C0F4, 0x9C566, 0x214C, 12),
        (&CASES[1], [0x10554, 0x117FC], 0xA162C, 0xA1AA6, 0x2164, 10),
        (&CASES[3], [0x10508, 0x117B0], 0xA106C, 0xA14E6, 0x2164, 10),
    ] {
        let rom = fs::read(local_rom_path(case.rom)).unwrap();
        for pc in calls {
            let ldr = u16::from_le_bytes(rom[pc - 4..pc - 2].try_into().unwrap());
            assert_eq!(ldr & 0xF800, 0x4800);
            let literal = (pc & !3) + usize::from(ldr & 255) * 4;
            assert_eq!(
                u32::from_le_bytes(rom[literal..literal + 4].try_into().unwrap()),
                base
            );
            let hi = u16::from_le_bytes(rom[pc..pc + 2].try_into().unwrap());
            let lo = u16::from_le_bytes(rom[pc + 2..pc + 4].try_into().unwrap());
            assert_eq!(hi & 0xF800, 0xF000);
            assert_eq!(lo & 0xF800, 0xF800);
            let delta = (((i32::from(hi & 0x7FF) << 12) | (i32::from(lo & 0x7FF) << 1)) << 9) >> 9;
            assert_eq!(pc as i32 + 4 + delta, constructor);
        }
        // FoMT clears +54..58 (423 at21A0); MFoMT clears +5A..5D
        // (467 at21C1). Equal reset behavior does not prove equal ownership.
        assert_eq!(
            &rom[constructor as usize + preserve..constructor as usize + preserve + 2],
            &[7, 0x1C]
        );
        if base == 0x2164 {
            // The MFoMT constructor explicitly zeroes event-structure bytes
            // +0x32..+0x3B. The final byte contains slot 322 in bits 0..2
            // and slot 323 in bits 3..4; initialization alone is not a
            // gameplay producer for either semantic state.
            let mut cursor = constructor as usize + 0x2DA;
            assert_eq!(
                &rom[cursor..cursor + 6],
                &[0x39, 0x1C, 0x32, 0x31, 0x00, 0x20]
            );
            cursor += 6;
            for index in 0..10 {
                if index != 0 {
                    assert_eq!(&rom[cursor..cursor + 2], &[0x01, 0x31]);
                    cursor += 2;
                }
                assert_eq!(&rom[cursor..cursor + 2], &[0x08, 0x70]);
                cursor += 2;
            }
            assert_eq!(
                &rom[clear..clear + 26],
                &[
                    0x39, 0x1C, 0x5A, 0x31, 0x04, 0x22, 0x52, 0x42, 0x94, 0x46, 0x00, 0x20, 0x08,
                    0x70, 0x01, 0x31, 0x08, 0x70, 0x01, 0x31, 0x08, 0x70, 0x01, 0x31, 0x08, 0x70,
                ]
            );
            continue;
        }
        assert_eq!(
            &rom[clear..clear + 28],
            &[
                0x39, 0x1C, 0x54, 0x31, 0x00, 0x20, 0x08, 0x70, 0x01, 0x31, 0x3F, 0x22, 0x90, 0x46,
                0x08, 0x70, 0x01, 0x31, 0x08, 0x70, 0x01, 0x31, 0x08, 0x70, 0x01, 0x31, 0x08, 0x70,
            ]
        );
    }
}

#[test]
fn native_fomt_unknown_449_preserves_two_bit_storage() {
    let reference = fs::read(local_rom_path(CASES[0].rom)).unwrap();
    for (case, get_table, set_table, get, set) in [
        (&CASES[0], 0x45934, 0x49034, 0x4858C, 0x4C930),
        (&CASES[2], 0x4575C, 0x48E5C, 0x483B4, 0x4C758),
    ] {
        let rom = fs::read(local_rom_path(case.rom)).unwrap();
        // VarSet subtracts 28 before dispatch.  Thus source-disassembly jump
        // table comments call public slot 449 "case 421"; the ROM lookup below
        // deliberately uses the normalized table index rather than that label
        // as a public variable ID.
        for (table, index, expected) in [(get_table, 449, get), (set_table, 449 - 28, set)] {
            let p = table + index * 4;
            assert_eq!(
                u32::from_le_bytes(rom[p..p + 4].try_into().unwrap()) as usize,
                0x08000000 + expected
            );
        }
        assert_eq!(&rom[get..get + 16], &reference[0x4858C..0x4859C]);
        // Getter B+0A selects save byte bits5..6, not its low two bits.
        let branch = u16::from_le_bytes(rom[get + 10..get + 12].try_into().unwrap());
        assert_eq!(branch & 0xF800, 0xE000);
        let delta = ((i32::from(branch & 0x7FF) << 21) >> 20) as isize;
        let leaf = (get as isize + 14 + delta) as usize;
        assert_eq!(&rom[leaf..leaf + 8], &reference[0x48662..0x4866A]);
        assert_eq!(&rom[set..set + 24], &reference[0x4C930..0x4C948]);
        assert_eq!(&rom[set + 28..set + 32], &[0xA5, 0x21, 0, 0]);
        let pc = set + 24;
        let hi = u16::from_le_bytes(rom[pc..pc + 2].try_into().unwrap());
        let lo = u16::from_le_bytes(rom[pc + 2..pc + 4].try_into().unwrap());
        assert_eq!(hi & 0xF800, 0xF000);
        assert_eq!(lo & 0xF800, 0xF800);
        let delta = (((i32::from(hi & 0x7FF) << 12) | (i32::from(lo & 0x7FF) << 1)) << 9) >> 9;
        let store = (pc as i32 + 4 + delta) as usize;
        assert_eq!(
            &rom[store..store + 6],
            &[0x10, 0x40, 0x18, 0x43, 0x08, 0x70]
        );
    }
    // Arithmetic model of the pinned mask/shift/store instructions, not ROM
    // emulation. Include overflowing and negative script values.
    for old in 0_u8..=255 {
        for value in [-1_i32, 0, 1, 2, 3, 4, 255, i32::MIN, i32::MAX] {
            let next = (old & !0x60) | (((value as u32 & 3) as u8) << 5);
            assert_eq!(next & !0x60, old & !0x60);
            assert_eq!((next >> 5) & 3, (value as u32 & 3) as u8);
        }
    }
}

#[test]
fn native_fomt_unknown_449_is_not_the_same_byte_low_two_bit_selector() {
    for (case, expected_literals, getter_leaf, selector_extract) in [
        (
            &CASES[0],
            &[
                0x3E5E4usize,
                0x48564,
                0x48578,
                0x48588,
                0x48598,
                0x4C8EC,
                0x4C90C,
                0x4C92C,
                0x4C94C,
            ][..],
            0x48662usize,
            0x3E5D2usize,
        ),
        (
            &CASES[2],
            &[
                0x3E258usize,
                0x4838C,
                0x483A0,
                0x483B0,
                0x483C0,
                0x4C714,
                0x4C734,
                0x4C754,
                0x4C774,
            ][..],
            0x4848Ausize,
            0x3E246usize,
        ),
    ] {
        let rom = fs::read(local_rom_path(case.rom)).unwrap();
        let actual_literals = rom[..0x100000]
            .windows(4)
            .enumerate()
            .filter_map(|(offset, bytes)| (bytes == 0x21A5u32.to_le_bytes()).then_some(offset))
            .collect::<Vec<_>>();
        assert_eq!(
            actual_literals, expected_literals,
            "{} direct literals for the slot-449 storage byte",
            case.name
        );
        // Slot 449 extracts bits 5..6: left 25, then right 30.
        assert_eq!(
            &rom[getter_leaf + 4..getter_leaf + 8],
            &[0x40, 0x06, 0x80, 0x0F],
            "{} slot 449 bit field",
            case.name
        );
        // The only business-code literal extracts bits 0..1 instead.
        assert_eq!(
            &rom[selector_extract..selector_extract + 4],
            &[0x80, 0x07, 0x80, 0x0F],
            "{} same-byte low selector",
            case.name
        );
    }
}

#[test]
fn native_harvest_boundary_unknown_boolean_is_not_the_adjacent_session_field() {
    for (case, field, expected_literals, getter_leaf, session_extractors) in [
        (
            &CASES[0],
            0x21A0u32,
            &[
                0x48354usize,
                0x48368,
                0x4837C,
                0x48390,
                0x483A0,
                0x4C5F8,
                0x4C618,
                0x4C638,
                0x4C658,
                0x4C678,
                0xA6518,
                0xA6634,
                0xA6824,
            ][..],
            0x48F76usize,
            &[0xA64D2usize, 0xA65A6, 0xA66EA, 0xA6704][..],
        ),
        (
            &CASES[2],
            0x21A0,
            &[
                0x4817Cusize,
                0x48190,
                0x481A4,
                0x481B8,
                0x481C8,
                0x4C420,
                0x4C440,
                0x4C460,
                0x4C480,
                0x4C4A0,
                0xA5F50,
                0xA606C,
                0xA625C,
            ][..],
            0x48D9E,
            &[0xA5F0Ausize, 0xA5FDE, 0xA6122, 0xA613C][..],
        ),
        (
            &CASES[1],
            0x21C1,
            &[
                0x48FF8usize,
                0x4900C,
                0x49020,
                0x49034,
                0x49048,
                0x4DFB4,
                0x4DFD4,
                0x4DFF4,
                0x4E014,
                0x4E034,
                0xAB74C,
                0xAB868,
                0xABA68,
            ][..],
            0x4A3B4,
            &[0xAB706usize, 0xAB7DA, 0xAB93C, 0xABA28][..],
        ),
        (
            &CASES[3],
            0x21C1,
            &[
                0x48D54usize,
                0x48D68,
                0x48D7C,
                0x48D90,
                0x48DA4,
                0x4DD10,
                0x4DD30,
                0x4DD50,
                0x4DD70,
                0x4DD90,
                0xAB18C,
                0xAB2A8,
                0xAB4A8,
            ][..],
            0x4A110,
            &[0xAB146usize, 0xAB21A, 0xAB37C, 0xAB468][..],
        ),
    ] {
        let rom = fs::read(local_rom_path(case.rom)).unwrap();
        let field_bytes = field.to_le_bytes();
        let actual_literals = rom[..0x100000]
            .windows(4)
            .enumerate()
            .filter_map(|(offset, bytes)| (bytes == field_bytes).then_some(offset))
            .collect::<Vec<_>>();
        assert_eq!(
            actual_literals, expected_literals,
            "{} direct literals for the shared festival byte",
            case.name
        );

        // The logical unknown field is bit 4: left 27, then right 31.
        assert_eq!(
            &rom[getter_leaf + 4..getter_leaf + 8],
            &[0xC0, 0x06, 0xC0, 0x0F],
            "{} unknown festival-boundary bit",
            case.name
        );
        // Every direct business-code reference outside the generic variable
        // getter/setter cluster extracts bits 0..1 instead. Those accesses
        // belong to the adjacent Harvest Festival session field, not this bit.
        for &extractor in session_extractors {
            let left = u16::from_le_bytes(rom[extractor..extractor + 2].try_into().unwrap());
            let right = u16::from_le_bytes(rom[extractor + 2..extractor + 4].try_into().unwrap());
            assert_eq!(left & 0xF800, 0, "{} low-bit left shift", case.name);
            assert_eq!((left >> 6) & 0x1F, 30, "{} low-bit width", case.name);
            assert_eq!(right & 0xF800, 0x0800, "{} low-bit right shift", case.name);
            assert_eq!((right >> 6) & 0x1F, 30, "{} low-bit width", case.name);
            assert_eq!(
                (right >> 3) & 7,
                left & 7,
                "{} low-bit shift register flow",
                case.name
            );
        }
    }
}

#[test]
fn native_animal_affection_accessors_preserve_raw_read_and_clamped_add() {
    let reference = fs::read(local_rom_path(CASES[0].rom)).unwrap();
    for (case, (table, slot, add, get, resolver)) in CASES.iter().zip([
        (0x3F904, 0x106, 0x44562, 0x446E0, 0x4E3D8),
        (0x3FAF0, 0x109, 0x447D8, 0x44954, 0x50994),
        (0x3F578, 0x106, 0x441D6, 0x44354, 0x4E200),
        (0x3F84C, 0x109, 0x44534, 0x446B0, 0x506F0),
    ]) {
        let rom = fs::read(local_rom_path(case.rom)).unwrap();
        let target = |pc: usize| {
            let hi = u16::from_le_bytes(rom[pc..pc + 2].try_into().unwrap());
            let lo = u16::from_le_bytes(rom[pc + 2..pc + 4].try_into().unwrap());
            assert_eq!(hi & 0xF800, 0xF000);
            assert_eq!(lo & 0xF800, 0xF800);
            let delta = (((i32::from(hi & 0x7FF) << 12) | (i32::from(lo & 0x7FF) << 1)) << 9) >> 9;
            (pc as i32 + 4 + delta) as usize
        };
        for (id, handler) in [(slot, add), (slot + 6, get)] {
            let entry = table + id * 4;
            assert_eq!(
                u32::from_le_bytes(rom[entry..entry + 4].try_into().unwrap()),
                0x08000000 + handler as u32
            );
        }
        assert_eq!(target(add + 0x42), resolver);
        assert_eq!(target(get + 0x30), resolver);
        // GetAnimalName pops index/kind/text slot and returns early on a null
        // animal or missing text context; neither path clears the text slot.
        let name_entry = table + (slot - 4) * 4;
        let name_handler = u32::from_le_bytes(rom[name_entry..name_entry + 4].try_into().unwrap())
            as usize
            - 0x08000000;
        assert_eq!(target(name_handler + 0x42), resolver);
        let mut name_offset = 0;
        // The final text-copy continuation differs by family; compare only
        // through name retrieval here, not an assumed common handler length.
        while name_offset < 0x68 {
            let pc = 0x44406 + name_offset;
            let hi = u16::from_le_bytes(reference[pc..pc + 2].try_into().unwrap());
            let lo = u16::from_le_bytes(reference[pc + 2..pc + 4].try_into().unwrap());
            if hi & 0xF800 == 0xF000 && lo & 0xF800 == 0xF800 {
                let _ = target(name_handler + name_offset);
                name_offset += 4;
            } else {
                assert_eq!(
                    &rom[name_handler + name_offset..name_handler + name_offset + 2],
                    &reference[pc..pc + 2],
                    "{} name handler+{name_offset:X}",
                    case.name
                );
                name_offset += 2;
            }
        }
        let is_mfomt = case.name.starts_with("mfomt");
        let continuation = target(name_handler + if is_mfomt { 0x6A } else { 0x68 });
        if is_mfomt {
            assert_eq!(
                &rom[name_handler + 0x68..name_handler + 0x6A],
                &[0x20, 0x1C]
            );
            assert_eq!(&rom[continuation..continuation + 2], &[0x31, 0x1C]);
        } else {
            assert_eq!(
                &rom[continuation..continuation + 4],
                &[0x20, 0x1C, 0x31, 0x1C]
            );
        }
        let text_thunk = target(continuation + if is_mfomt { 2 } else { 4 });
        assert_eq!(
            &rom[text_thunk..text_thunk + 8],
            &reference[0x12ACC..0x12AD4]
        );
        assert_eq!(
            &rom[text_thunk + 12..text_thunk + 16],
            &reference[0x12AD8..0x12ADC]
        );
        let text_copy = target(text_thunk + 8);
        let string_slot = if is_mfomt { 0x3B } else { 0x3A };
        let string_entry = table + string_slot * 4;
        let string_handler =
            u32::from_le_bytes(rom[string_entry..string_entry + 4].try_into().unwrap()) as usize
                - 0x08000000;
        let mut shared_copier_calls = 0;
        for offset in (0..0x60).step_by(2) {
            let pc = string_handler + offset;
            let hi = u16::from_le_bytes(rom[pc..pc + 2].try_into().unwrap());
            let lo = u16::from_le_bytes(rom[pc + 2..pc + 4].try_into().unwrap());
            if hi & 0xF800 == 0xF000 && lo & 0xF800 == 0xF800 && target(pc) == text_thunk {
                shared_copier_calls += 1;
            }
        }
        assert_eq!(shared_copier_calls, 1, "{} direct text setter", case.name);
        let wide_text_slot = case.name == "mfomt-us";
        let copy_reference = if wide_text_slot {
            fs::read(local_rom_path(CASES[1].rom)).unwrap()
        } else {
            reference.clone()
        };
        let copy_base = if wide_text_slot { 0x3B9B0 } else { 0x3B6B8 };
        let copy_length = if wide_text_slot { 0x3E } else { 0x42 };
        let copy_calls = [0xC, 0x1E, if wide_text_slot { 0x32 } else { 0x36 }];
        for offset in 0..copy_length {
            if offset == 0x12 || offset == 0x16 {
                let limit = match case.name {
                    "fomt-jp" => 20,
                    "mfomt-jp" => 20,
                    _ if is_mfomt => 28,
                    _ => 22,
                };
                assert_eq!(
                    rom[text_copy + offset],
                    limit,
                    "{} copy byte limit",
                    case.name
                );
                continue;
            }
            if copy_calls
                .iter()
                .any(|start| (*start..*start + 4).contains(&offset))
            {
                continue;
            }
            assert_eq!(
                rom[text_copy + offset],
                copy_reference[copy_base + offset],
                "{} text copy+{offset:X}",
                case.name
            );
        }
        for offset in copy_calls {
            let _ = target(text_copy + offset);
        }
        // Daily talk, unhappiness, sickness, pregnancy, healthy pregnancy
        // days and age use this same roster resolver. Read each physical
        // table entry: handler address order is not callable ID order.
        for relative in [-2_i32, -1, 1, 2, 3, 4, 5] {
            let entry = table + (slot as i32 + relative) as usize * 4;
            let handler =
                u32::from_le_bytes(rom[entry..entry + 4].try_into().unwrap()) as usize - 0x08000000;
            assert_eq!(
                target(handler + 0x30),
                resolver,
                "{} animal slot {relative}",
                case.name
            );
        }
        for (relative, leaf_base, length) in [
            (-2_i32, 0x9B238, 8), // talked bit
            (1, 0x9B504, 8),      // unhappy bit
            (2, 0x9B50C, 6),      // sick bit
            (3, 0x9B8B0, 10),     // pregnant bit
            (4, 0x9B8D4, 28),     // guarded healthy-day counter
            (5, 0x9B220, 8),      // ten-bit age
        ] {
            let entry = table + (slot as i32 + relative) as usize * 4;
            let handler =
                u32::from_le_bytes(rom[entry..entry + 4].try_into().unwrap()) as usize - 0x08000000;
            // No species guard is inserted between resolution and field read.
            assert_eq!(
                &rom[handler + 0x34..handler + 0x3A],
                &reference[0x445EE..0x445F4]
            );
            let leaf = target(handler + 0x3E);
            assert_eq!(
                &rom[leaf..leaf + length],
                &reference[leaf_base..leaf_base + length],
                "{} animal field {relative}",
                case.name
            );
        }
        let branches = u32::from_le_bytes(rom[resolver + 0x18..resolver + 0x1C].try_into().unwrap())
            as usize
            - 0x08000000;
        assert_eq!(branches, resolver + 0x1C);
        for (kind, relative) in [0x30, 0x42, 0x6A, 0x92, 0xBA].into_iter().enumerate() {
            let entry = branches + kind * 4;
            assert_eq!(
                u32::from_le_bytes(rom[entry..entry + 4].try_into().unwrap()) as usize - 0x08000000,
                resolver + relative
            );
        }
        // Preserve every branch and index/capacity comparison; exclude only
        // table pointers, validated BL relocations and the family dog offset.
        let resolver_reference = if case.name.starts_with("mfomt") {
            fs::read(local_rom_path(CASES[1].rom)).unwrap()
        } else {
            reference.clone()
        };
        let resolver_base = if case.name.starts_with("mfomt") {
            0x50994
        } else {
            0x4E3D8
        };
        let mut offset = 0;
        while offset < 0xD0 {
            if (0x18..0x30).contains(&offset) {
                offset = 0x30;
                continue;
            }
            let pc = resolver_base + offset;
            let hi = u16::from_le_bytes(resolver_reference[pc..pc + 2].try_into().unwrap());
            let lo = u16::from_le_bytes(resolver_reference[pc + 2..pc + 4].try_into().unwrap());
            if hi & 0xF800 == 0xF000 && lo & 0xF800 == 0xF800 {
                let _ = target(resolver + offset);
                offset += 4;
            } else {
                assert_eq!(
                    &rom[resolver + offset..resolver + offset + 2],
                    &resolver_reference[pc..pc + 2],
                    "{} animal resolver+{offset:X}",
                    case.name
                );
                offset += 2;
            }
        }
        // Cow and sheep resolve the same barn roster offset/stride, then
        // select the requested species. Chickens use a separate coop roster.
        // Include unsigned capacity guards and the complete species/occupied
        // leaf rather than accepting a matching outer resolver alone.
        for (call_offset, base, leaf_base, leaf_len) in [
            (0x62, 0xD3D0, 0xDA7C, 28),
            (0x8A, 0xD3F8, 0xDA98, 28),
            (0xB2, 0xC894, 0xCCD0, 22),
        ] {
            let lookup = target(resolver + call_offset);
            for offset in 0..40 {
                if (6..10).contains(&offset) || (0x1C..0x20).contains(&offset) {
                    continue;
                }
                assert_eq!(rom[lookup + offset], reference[base + offset]);
            }
            let _ = target(lookup + 6);
            let leaf = target(lookup + 0x1C);
            assert_eq!(
                &rom[leaf..leaf + leaf_len],
                &reference[leaf_base..leaf_base + leaf_len],
                "{} animal roster leaf",
                case.name
            );
        }
        let adder = target(add + 0x50);
        let talk_entry = table + (slot - 1) * 4;
        let talk_handler = u32::from_le_bytes(rom[talk_entry..talk_entry + 4].try_into().unwrap())
            as usize
            - 0x08000000;
        assert_eq!(
            &rom[talk_handler + 0x34..talk_handler + 0x38],
            &reference[0x4450A..0x4450E]
        );
        let talk_setter = target(talk_handler + 0x3C);
        // Full leaf through return: tests bit 4 of byte +0x19, sets only
        // that bit if clear. No affection update or interaction/UI call.
        assert_eq!(
            &rom[talk_setter..talk_setter + 22],
            &reference[0x9B290..0x9B2A6]
        );
        let getter = target(get + 0x3E);
        assert_eq!(&rom[adder..adder + 52], &reference[0x9B2A8..0x9B2DC]);
        assert_eq!(
            &rom[getter..getter + 8],
            &[0x80, 0x69, 0xC0, 0x02, 0x00, 0x0E, 0x70, 0x47]
        );
        // Null resolution skips mutation; the query prepares a zero result.
        assert_eq!(&rom[add + 0x46..add + 0x4A], &reference[0x445A8..0x445AC]);
        assert_eq!(&rom[get + 0x34..get + 0x3A], &reference[0x44714..0x4471A]);
    }
}

#[test]
fn native_add_affection_to_all_animals_visits_each_owned_roster_and_clamps() {
    fn call_target(rom: &[u8], pc: usize) -> Option<usize> {
        let hi = u16::from_le_bytes(rom[pc..pc + 2].try_into().unwrap());
        let lo = u16::from_le_bytes(rom[pc + 2..pc + 4].try_into().unwrap());
        if hi & 0xF800 != 0xF000 || lo & 0xF800 != 0xF800 {
            return None;
        }
        let raw = (((hi & 0x7FF) as i32) << 12) | (((lo & 0x7FF) as i32) << 1);
        Some((pc as i32 + 4 + ((raw << 9) >> 9)) as usize)
    }

    let reference = fs::read(local_rom_path(CASES[0].rom)).unwrap();
    for (case, (table, slot, handler, clamp)) in CASES.iter().zip([
        (0x3F904, 0x12A, 0x4532A, 0x9B2A8),
        (0x3FAF0, 0x12E, 0x455FE, 0xA0214),
        (0x3F578, 0x12A, 0x44F9E, 0x9ACE0),
        (0x3F84C, 0x12E, 0x4535A, 0x9FC54),
    ]) {
        let rom = fs::read(local_rom_path(case.rom)).unwrap();
        assert_eq!(
            u32::from_le_bytes(
                rom[table + slot * 4..table + slot * 4 + 4]
                    .try_into()
                    .unwrap()
            ),
            0x08000000u32 + handler as u32,
            "{} physical all-animal affection callable",
            case.name
        );

        // The handler calls the same Animal::AddAffection implementation four
        // times: fixed dog, optional horse, occupied chicken slots, and occupied
        // slots in the shared cow/sheep barn roster.
        let calls = (0..0xA0)
            .step_by(2)
            .filter(|offset| call_target(&rom, handler + offset) == Some(clamp))
            .count();
        assert_eq!(calls, 4, "{} animal roster coverage", case.name);

        // Complete arithmetic/clamp core through the upper-bound branch. It
        // extracts the eight-bit affection field, performs signed addition,
        // clamps below zero and above 250, and is byte-identical in all targets.
        assert_eq!(
            &rom[clamp..clamp + 0x1A],
            &reference[0x9B2A8..0x9B2C2],
            "{} shared animal-affection clamp",
            case.name
        );
    }
}

#[test]
fn native_cure_all_sick_livestock_scans_fixed_entity_range_and_resets_only_sick() {
    fn call_target(rom: &[u8], pc: usize) -> usize {
        let hi = u16::from_le_bytes(rom[pc..pc + 2].try_into().unwrap());
        let lo = u16::from_le_bytes(rom[pc + 2..pc + 4].try_into().unwrap());
        assert_eq!(hi & 0xF800, 0xF000);
        assert_eq!(lo & 0xF800, 0xF800);
        let raw = (((hi & 0x7FF) as i32) << 12) | (((lo & 0x7FF) as i32) << 1);
        (pc as i32 + 4 + ((raw << 9) >> 9)) as usize
    }

    let reference = fs::read(local_rom_path(CASES[0].rom)).unwrap();
    for (case, (table, slot, handler, routine)) in CASES.iter().zip([
        (0x3F904, 0x129, 0x453CC, 0x16F60),
        (0x3FAF0, 0x12D, 0x4569E, 0x16FE0),
        (0x3F578, 0x129, 0x45040, 0x16CF4),
        (0x3F84C, 0x12D, 0x453FA, 0x16E54),
    ]) {
        let rom = fs::read(local_rom_path(case.rom)).unwrap();
        assert_eq!(
            u32::from_le_bytes(
                rom[table + slot * 4..table + slot * 4 + 4]
                    .try_into()
                    .unwrap()
            ),
            0x08000000u32 + handler as u32,
            "{} physical cure-all callable",
            case.name
        );
        assert_eq!(
            call_target(&rom, handler + 0x0E),
            routine,
            "{} cure-all routine",
            case.name
        );

        // All targets use the same 0x2E..=0x45 entity loop, null guard,
        // virtual IsSick test at vtable+0x5C, conditional ResetSick call at
        // vtable+0x7C, and no other mutation in the loop body.
        let mut body = rom[routine..routine + 0x42].to_vec();
        for offset in [0x12, 0x22, 0x32] {
            body[offset..offset + 4].fill(0);
        }
        let mut expected = reference[0x16F60..0x16FA2].to_vec();
        for offset in [0x12, 0x22, 0x32] {
            expected[offset..offset + 4].fill(0);
        }
        assert_eq!(body, expected, "{} complete cure-all loop", case.name);
        assert_eq!(
            rom[routine + 4],
            0x2E,
            "{} first animal entity ID",
            case.name
        );
        assert_eq!(
            rom[routine + 0x38],
            0x45,
            "{} final animal entity ID",
            case.name
        );
    }
}

#[test]
fn native_shooting_star_shipping_bonus_doubles_one_settlement_then_clears() {
    fn call_target(rom: &[u8], pc: usize) -> usize {
        let hi = u16::from_le_bytes(rom[pc..pc + 2].try_into().unwrap());
        let lo = u16::from_le_bytes(rom[pc + 2..pc + 4].try_into().unwrap());
        assert_eq!(hi & 0xF800, 0xF000);
        assert_eq!(lo & 0xF800, 0xF800);
        let raw = (((hi & 0x7FF) as i32) << 12) | (((lo & 0x7FF) as i32) << 1);
        (pc as i32 + 4 + ((raw << 9) >> 9)) as usize
    }

    for (case, (table, slot, handler, field, field_literal, settlement)) in CASES.iter().zip([
        (0x3F904, 0x12B, 0x4414C, 0x34C5u32, 0x14, 0x1140C),
        (0x3FAF0, 0x12F, 0x443BE, 0x3501u32, 0x16, 0x114EC),
        (0x3F578, 0x12B, 0x43DC0, 0x34C5u32, 0x14, 0x113EC),
        (0x3F84C, 0x12F, 0x4411A, 0x3501u32, 0x16, 0x114A0),
    ]) {
        let rom = fs::read(local_rom_path(case.rom)).unwrap();
        assert_eq!(
            u32::from_le_bytes(
                rom[table + slot * 4..table + slot * 4 + 4]
                    .try_into()
                    .unwrap()
            ),
            0x08000000u32 + handler as u32,
            "{} physical shooting-star bonus callable",
            case.name
        );

        // The callable stores exactly byte 1 at the family-specific field.
        assert_eq!(
            u32::from_le_bytes(
                rom[handler + field_literal..handler + field_literal + 4]
                    .try_into()
                    .unwrap()
            ),
            field,
            "{} bonus field",
            case.name
        );
        assert_eq!(
            &rom[handler + 0x0C..handler + 0x10],
            &[0x01, 0x21, 0x01, 0x70]
        );

        // Settlement obtains one accumulated shipping value and sends the same
        // value to the same money-adder once unconditionally and once only when
        // the byte is set. It then clears the byte and resets the accumulator.
        let add_once = call_target(&rom, settlement + 0x18);
        assert_eq!(
            call_target(&rom, settlement + 0x2A),
            add_once,
            "{} repeated payout",
            case.name
        );
        assert_eq!(
            u32::from_le_bytes(
                rom[settlement + 0x48..settlement + 0x4C]
                    .try_into()
                    .unwrap()
            ),
            field,
            "{} settlement field",
            case.name
        );
        assert_eq!(
            &rom[settlement + 0x2E..settlement + 0x32],
            &[0x00, 0x20, 0x20, 0x70]
        );
        assert_eq!(
            &rom[settlement + 0x20..settlement + 0x26],
            &[0x20, 0x78, 0x00, 0x28, 0x05, 0xD0]
        );
    }
}

#[test]
fn native_cottage_unlocks_and_vacation_villa_query_share_one_persistent_byte() {
    fn call_target(rom: &[u8], pc: usize) -> usize {
        let hi = u16::from_le_bytes(rom[pc..pc + 2].try_into().unwrap());
        let lo = u16::from_le_bytes(rom[pc + 2..pc + 4].try_into().unwrap());
        assert_eq!(hi & 0xF800, 0xF000);
        assert_eq!(lo & 0xF800, 0xF800);
        let raw = (((hi & 0x7FF) as i32) << 12) | (((lo & 0x7FF) as i32) << 1);
        (pc as i32 + 4 + ((raw << 9) >> 9)) as usize
    }

    for (
        case,
        (
            table,
            mountain_slot,
            seaside_slot,
            mountain_handler,
            seaside_handler,
            vacation_slot,
            vacation_handler,
            save_offset,
        ),
    ) in CASES.iter().zip([
        (
            0x3F904, 0xC4, 0xC5, 0x44E3C, 0x44E4A, 0xE3, 0x42C98, 0x1AA8u32,
        ),
        (
            0x3FAF0, 0xC7, 0xC8, 0x45098, 0x450A8, 0xE6, 0x42EC4, 0x1AB8u32,
        ),
        (
            0x3F578, 0xC4, 0xC5, 0x44AB0, 0x44ABE, 0xE3, 0x4290C, 0x1AA8u32,
        ),
        (
            0x3F84C, 0xC7, 0xC8, 0x44DF4, 0x44E04, 0xE6, 0x42C20, 0x1AB8u32,
        ),
    ]) {
        let rom = fs::read(local_rom_path(case.rom)).unwrap();
        for (slot, handler, mask) in [
            (mountain_slot, mountain_handler, 1u8),
            (seaside_slot, seaside_handler, 4u8),
        ] {
            assert_eq!(
                u32::from_le_bytes(
                    rom[table + slot * 4..table + slot * 4 + 4]
                        .try_into()
                        .unwrap()
                ),
                0x08000000u32 + handler as u32,
                "{} cottage callable slot {slot:X}",
                case.name
            );
            let setter = call_target(&rom, handler + 8);
            assert_eq!(
                &rom[setter..setter + 10],
                &[0x01, 0x78, mask, 0x22, 0x11, 0x43, 0x01, 0x70, 0x70, 0x47],
                "{} cottage mask {mask}",
                case.name
            );
        }

        assert_eq!(
            u32::from_le_bytes(
                rom[table + vacation_slot * 4..table + vacation_slot * 4 + 4]
                    .try_into()
                    .unwrap()
            ),
            0x0800_0000 + vacation_handler as u32,
            "{} Vacation Villa query slot",
            case.name
        );
        assert_eq!(
            &rom[vacation_handler + 0x0C..vacation_handler + 0x12],
            &[0x00, 0x79, 0x80, 0x07, 0xC4, 0x0F],
            "{} Vacation Villa query extracts shared facility bit 1",
            case.name
        );
        assert_eq!(
            u32::from_le_bytes(
                rom[vacation_handler + 0x20..vacation_handler + 0x24]
                    .try_into()
                    .unwrap()
            ),
            save_offset,
            "{} cottage facility-byte owner offset",
            case.name
        );
    }
}

#[test]
fn native_television_callables_keep_target_slots_and_viewer_protocol() {
    for (case, (dispatch, first_slot, handlers, leaves)) in CASES.iter().zip([
        (
            0x3F904,
            0xFB,
            [0x44DF0, 0x423F0, 0x4241E],
            [0x126BC, 0x142B8, 0x142D4],
        ),
        (
            0x3FAF0,
            0xFE,
            [0x4504A, 0x4263C, 0x4266A],
            [0x12848, 0x14444, 0x14460],
        ),
        (
            0x3F578,
            0xFB,
            [0x44A64, 0x42064, 0x42092],
            [0x1258C, 0x1418C, 0x141A8],
        ),
        (
            0x3F84C,
            0xFE,
            [0x44DA6, 0x42398, 0x423C6],
            [0x126EC, 0x142EC, 0x14308],
        ),
    ]) {
        let rom = fs::read(local_rom_path(case.rom)).unwrap();
        for (index, handler) in handlers.into_iter().enumerate() {
            assert_eq!(
                u32::from_le_bytes(
                    rom[dispatch + (first_slot + index) * 4
                        ..dispatch + (first_slot + index) * 4 + 4]
                        .try_into()
                        .unwrap()
                ),
                0x0800_0000 + handler as u32,
                "{} television callable slot {}",
                case.name,
                first_slot + index
            );
        }
        for ((handler, offset), leaf) in handlers.into_iter().zip([0x40, 0x26, 0x10]).zip(leaves) {
            assert_eq!(
                thumb_call_target(&rom, handler + offset),
                leaf,
                "{} television viewer leaf",
                case.name
            );
        }

        // ShowTelevisionMessage submits text at native rate 0x40 and requests
        // viewer state 0x11. This is distinct from ordinary TalkMessage.
        assert_eq!(&rom[leaves[0] + 0x14..leaves[0] + 0x16], &[0x40, 0x22]);
        assert_eq!(
            &rom[leaves[0] + 0x1A..leaves[0] + 0x22],
            &[0x21, 0x1C, 0x9C, 0x31, 0x11, 0x20, 0x08, 0x60]
        );
        // Set/End dispatch through adjacent viewer virtual methods 0x118/0x11C.
        assert_eq!(
            &rom[leaves[1] + 0x0A..leaves[1] + 0x12],
            &[0x8C, 0x23, 0x5B, 0x00, 0xD2, 0x18, 0x12, 0x68]
        );
        assert_eq!(
            &rom[leaves[2] + 0x0A..leaves[2] + 0x12],
            &[0x8E, 0x22, 0x52, 0x00, 0x89, 0x18, 0x09, 0x68]
        );
    }
}

#[test]
fn native_livestock_counts_scan_only_current_capacity_and_present_species_records() {
    for (case, (dispatch, first_slot, handlers, leaves, capacity_getters, record_getters)) in
        CASES.iter().zip([
            (
                0x3F904,
                0x115,
                [0x441AC, 0x441C2, 0x441E4],
                [0x0CFC4, 0x0CFF4, 0x0C630],
                [0x0CE74, 0x0CE74, 0x0C570],
                [0x0DA2C, 0x0DA48, 0x0CCB8],
            ),
            (
                0x3FAF0,
                0x118,
                [0x4441E, 0x44440, 0x44462],
                [0x0D038, 0x0D068, 0x0C6A4],
                [0x0CEE8, 0x0CEE8, 0x0C5E4],
                [0x0DAA0, 0x0DABC, 0x0CD2C],
            ),
            (
                0x3F578,
                0x115,
                [0x43E20, 0x43E36, 0x43E58],
                [0x0CFA4, 0x0CFD4, 0x0C610],
                [0x0CE54, 0x0CE54, 0x0C550],
                [0x0DA0C, 0x0DA28, 0x0CC98],
            ),
            (
                0x3F84C,
                0x118,
                [0x4417A, 0x4419C, 0x441BE],
                [0x0CFEC, 0x0D01C, 0x0C658],
                [0x0CE9C, 0x0CE9C, 0x0C598],
                [0x0DA54, 0x0DA70, 0x0CCE0],
            ),
        ])
    {
        let rom = fs::read(local_rom_path(case.rom)).unwrap();
        for index in 0..3 {
            assert_eq!(
                u32::from_le_bytes(
                    rom[dispatch + (first_slot + index) * 4
                        ..dispatch + (first_slot + index) * 4 + 4]
                        .try_into()
                        .unwrap()
                ),
                0x0800_0000 + handlers[index] as u32,
                "{} livestock-count callable slot {}",
                case.name,
                first_slot + index
            );
            assert_eq!(
                thumb_call_target(&rom, handlers[index] + 0x0E),
                leaves[index],
                "{} livestock-count leaf {index}",
                case.name
            );
            assert_eq!(
                thumb_call_target(&rom, leaves[index] + 0x04),
                capacity_getters[index],
                "{} livestock-count capacity getter {index}",
                case.name
            );
            assert_eq!(
                thumb_call_target(&rom, leaves[index] + 0x16),
                record_getters[index],
                "{} livestock-count species resolver {index}",
                case.name
            );
        }

        for leaf in &leaves[..2] {
            assert_eq!(&rom[leaf + 0x12..leaf + 0x16], &[0x2C, 0x34, 0x20, 0x1C]);
            assert_eq!(&rom[leaf + 0x20..leaf + 0x24], &[0x3C, 0x34, 0x01, 0x36]);
        }
        let chicken = leaves[2];
        assert_eq!(
            &rom[chicken + 0x12..chicken + 0x16],
            &[0x18, 0x34, 0x20, 0x1C]
        );
        assert_eq!(
            &rom[chicken + 0x20..chicken + 0x24],
            &[0x30, 0x34, 0x01, 0x36]
        );
    }
}

#[test]
fn native_golden_lumber_query_scans_the_complete_placed_field_grid() {
    fn call_target(rom: &[u8], pc: usize) -> usize {
        let hi = u16::from_le_bytes(rom[pc..pc + 2].try_into().unwrap());
        let lo = u16::from_le_bytes(rom[pc + 2..pc + 4].try_into().unwrap());
        assert_eq!(hi & 0xF800, 0xF000);
        assert_eq!(lo & 0xF800, 0xF800);
        let raw = (((hi & 0x7FF) as i32) << 12) | (((lo & 0x7FF) as i32) << 1);
        (pc as i32 + 4 + ((raw << 9) >> 9)) as usize
    }

    for (case, (table, slot, handler, query)) in CASES.iter().zip([
        (0x3F904, 0xC6, 0x43578, 0x09B20),
        (0x3FAF0, 0xC9, 0x437C8, 0x09B8C),
        (0x3F578, 0xC6, 0x431EC, 0x09B28),
        (0x3F84C, 0xC9, 0x43524, 0x09B40),
    ]) {
        let rom = fs::read(local_rom_path(case.rom)).unwrap();
        assert_eq!(
            u32::from_le_bytes(
                rom[table + slot * 4..table + slot * 4 + 4]
                    .try_into()
                    .unwrap()
            ),
            0x08000000u32 + handler as u32,
            "{} physical Golden Lumber query",
            case.name
        );
        assert_eq!(
            call_target(&rom, handler + 0x0A),
            query,
            "{} Field query target",
            case.name
        );

        // Extract placed-state bits 12..15, then object-ID bits 2..7 and
        // compare the latter with the Golden Lumber field object ID 0x1A.
        assert_eq!(
            &rom[query + 0x10..query + 0x20],
            &[
                0x08, 0x05, 0x00, 0x0F, 0x00, 0x28, 0x07, 0xD0, 0x08, 0x06, 0x80, 0x0E, 0x1A, 0x28,
                0x03, 0xD1
            ],
            "{} placed Golden Lumber predicate",
            case.name
        );
        // Inclusive column 0..0x2A and row 0..0x18 bounds establish the
        // complete 43-by-25 grid; row stride is 0xAC = 43 * 4 bytes.
        assert_eq!(
            &rom[query + 0x28..query + 0x38],
            &[
                0x04, 0x32, 0x01, 0x33, 0x2A, 0x2B, 0xEE, 0xD9, 0xAC, 0x34, 0x01, 0x35, 0x18, 0x2D,
                0xE8, 0xD9
            ],
            "{} full farm grid bounds",
            case.name
        );
    }
}

#[test]
fn native_door_operations_dispatch_distinct_virtual_methods_without_waiting() {
    fn call_target(rom: &[u8], pc: usize) -> usize {
        let hi = u16::from_le_bytes(rom[pc..pc + 2].try_into().unwrap());
        let lo = u16::from_le_bytes(rom[pc + 2..pc + 4].try_into().unwrap());
        assert_eq!(hi & 0xF800, 0xF000);
        assert_eq!(lo & 0xF800, 0xF800);
        let raw = (((hi & 0x7FF) as i32) << 12) | (((lo & 0x7FF) as i32) << 1);
        (pc as i32 + 4 + ((raw << 9) >> 9)) as usize
    }

    for (
        case,
        (table, open_slot, close_slot, open_handler, close_handler, call_offset, open, close),
    ) in CASES.iter().zip([
        (
            0x3F904, 0xC7, 0xC8, 0x44E58, 0x44E82, 0x24, 0x16BA4, 0x16BC0,
        ),
        (
            0x3FAF0, 0xCA, 0xCB, 0x450B8, 0x450E6, 0x26, 0x16C24, 0x16C40,
        ),
        (
            0x3F578, 0xC7, 0xC8, 0x44ACC, 0x44AF6, 0x24, 0x16938, 0x16954,
        ),
        (
            0x3F84C, 0xCA, 0xCB, 0x44E14, 0x44E42, 0x26, 0x16A98, 0x16AB4,
        ),
    ]) {
        let rom = fs::read(local_rom_path(case.rom)).unwrap();
        for (slot, handler, shim, selector) in [
            (open_slot, open_handler, open, 0x86u8),
            (close_slot, close_handler, close, 0x88u8),
        ] {
            assert_eq!(
                u32::from_le_bytes(
                    rom[table + slot * 4..table + slot * 4 + 4]
                        .try_into()
                        .unwrap()
                ),
                0x08000000u32 + handler as u32,
                "{} door slot {slot:X}",
                case.name
            );
            assert_eq!(
                call_target(&rom, handler + call_offset),
                shim,
                "{} door dispatch shim",
                case.name
            );
            // The shim leaves r1 (door_index) untouched and performs a virtual
            // call through vtable offsets 0x10C/open or 0x110/close.
            assert_eq!(
                &rom[shim..shim + 0x12],
                &[
                    0x00, 0xB5, 0x40, 0x68, 0xA8, 0x30, 0x00, 0x68, 0x02, 0x68, selector, 0x23,
                    0x5B, 0x00, 0xD2, 0x18, 0x12, 0x68
                ],
                "{} door virtual selector",
                case.name
            );
        }
    }
}

#[test]
fn native_supermarket_purchase_hooks_redraw_shelf_tiles_without_changing_inventory() {
    fn call_target(rom: &[u8], pc: usize) -> usize {
        let hi = u16::from_le_bytes(rom[pc..pc + 2].try_into().unwrap());
        let lo = u16::from_le_bytes(rom[pc + 2..pc + 4].try_into().unwrap());
        assert_eq!(hi & 0xF800, 0xF000);
        assert_eq!(lo & 0xF800, 0xF800);
        let raw = (((hi & 0x7FF) as i32) << 12) | (((lo & 0x7FF) as i32) << 1);
        (pc as i32 + 4 + ((raw << 9) >> 9)) as usize
    }

    for (
        case,
        (
            dispatch,
            rucksack_slot,
            feather_slot,
            rucksack_handler,
            feather_handler,
            rucksack_shim,
            feather_shim,
            game_vtable,
            rucksack_thunk,
            feather_thunk,
            rucksack_leaf,
            feather_leaf,
        ),
    ) in CASES.iter().zip([
        (
            0x3F904, 0xC9, 0xCA, 0x4249E, 0x424B6, 0x16BDC, 0x16BF4, 0xE5EC4, 0x1DE30, 0x1DE3C,
            0xAA89C, 0xAA8BC,
        ),
        (
            0x3FAF0, 0xCC, 0xCD, 0x426EA, 0x42702, 0x16C5C, 0x16C74, 0xEE404, 0x1DFB0, 0x1DFBC,
            0xAFAE4, 0xAFB04,
        ),
        (
            0x3F578, 0xC9, 0xCA, 0x42112, 0x4212A, 0x16970, 0x16988, 0xE5304, 0x1DBC4, 0x1DBD0,
            0xAA2D4, 0xAA2F4,
        ),
        (
            0x3F84C, 0xCC, 0xCD, 0x42446, 0x4245E, 0x16AD0, 0x16AE8, 0xEDF14, 0x1DE24, 0x1DE30,
            0xAF524, 0xAF544,
        ),
    ]) {
        let rom = fs::read(local_rom_path(case.rom)).unwrap();
        for (slot, handler, shim, thunk, leaf, row, vtable_offset) in [
            (
                rucksack_slot,
                rucksack_handler,
                rucksack_shim,
                rucksack_thunk,
                rucksack_leaf,
                0x14u8,
                0xFCusize,
            ),
            (
                feather_slot,
                feather_handler,
                feather_shim,
                feather_thunk,
                feather_leaf,
                0x17u8,
                0x100usize,
            ),
        ] {
            assert_eq!(
                u32::from_le_bytes(
                    rom[dispatch + slot * 4..dispatch + slot * 4 + 4]
                        .try_into()
                        .unwrap()
                ),
                0x08000000u32 + handler as u32,
                "{} physical supermarket hook slot",
                case.name
            );
            assert_eq!(
                call_target(&rom, handler + 0x10),
                shim,
                "{} handler to game-object shim",
                case.name
            );
            assert_eq!(
                &rom[shim..shim + 10],
                &[0x00, 0xB5, 0x40, 0x68, 0xA8, 0x30, 0x00, 0x68, 0x01, 0x68],
                "{} game-object lookup",
                case.name
            );
            assert_eq!(
                u32::from_le_bytes(
                    rom[game_vtable + vtable_offset..game_vtable + vtable_offset + 4]
                        .try_into()
                        .unwrap()
                ) & !1,
                0x08000000u32 + thunk as u32,
                "{} game-object virtual slot",
                case.name
            );
            assert_eq!(
                call_target(&rom, thunk + 4),
                leaf,
                "{} shelf redraw leaf",
                case.name
            );

            // Fixed leaf: copy one 2x3 tile patch at X=0x21 and the selected
            // shelf row, then mark game-object byte +0x29 dirty. Exclude only
            // the relocated BL and data-pointer literal from byte comparison.
            assert_eq!(
                &rom[leaf..leaf + 10],
                &[0x10, 0xB5, 0x04, 0x1C, 0x05, 0x49, 0x21, 0x22, row, 0x23],
                "{} shelf redraw setup",
                case.name
            );
            assert_eq!(
                &rom[leaf + 14..leaf + 26],
                &[0x29, 0x34, 0x01, 0x20, 0x20, 0x70, 0x10, 0xBC, 0x01, 0xBC, 0x00, 0x47],
                "{} shelf redraw dirty flag",
                case.name
            );
            let patch_pointer = u32::from_le_bytes(rom[leaf + 28..leaf + 32].try_into().unwrap());
            let patch = (patch_pointer - 0x08000000) as usize;
            assert_eq!(
                &rom[patch..patch + 4],
                &[2, 3, 0, 0],
                "{} shelf background patch dimensions",
                case.name
            );
        }
    }
}

#[test]
fn native_harvest_sprite_task_state_callables_keep_distinct_persistent_and_runtime_semantics() {
    fn call_target(rom: &[u8], pc: usize) -> usize {
        let hi = u16::from_le_bytes(rom[pc..pc + 2].try_into().unwrap());
        let lo = u16::from_le_bytes(rom[pc + 2..pc + 4].try_into().unwrap());
        assert_eq!(hi & 0xF800, 0xF000);
        assert_eq!(lo & 0xF800, 0xF800);
        let raw = (((hi & 0x7FF) as i32) << 12) | (((lo & 0x7FF) as i32) << 1);
        (pc as i32 + 4 + ((raw << 9) >> 9)) as usize
    }

    for (
        case,
        (
            dispatch,
            minigame_slot,
            start_slot,
            end_slot,
            daily_slot,
            minigame_handler,
            start_handler,
            end_handler,
            daily_handler,
            minigame_getter,
            start_task,
            schedule_end,
            daily_complete,
        ),
    ) in CASES.iter().zip([
        (
            0x3F904, 0xCF, 0xD0, 0xD1, 0xD2, 0x42738, 0x427A0, 0x42808, 0x42844, 0x9E688, 0x9E6C4,
            0x9E6EC, 0x15920,
        ),
        (
            0x3FAF0, 0xD2, 0xD3, 0xD4, 0xD5, 0x4297C, 0x429E4, 0x42A4C, 0x42A88, 0xA381C, 0xA3858,
            0xA3880, 0x159A0,
        ),
        (
            0x3F578, 0xCF, 0xD0, 0xD1, 0xD2, 0x423AC, 0x42414, 0x4247C, 0x424B8, 0x9E0C0, 0x9E0FC,
            0x9E124, 0x156B4,
        ),
        (
            0x3F84C, 0xD2, 0xD3, 0xD4, 0xD5, 0x426D8, 0x42740, 0x427A8, 0x427E4, 0xA325C, 0xA3298,
            0xA32C0, 0x15814,
        ),
    ]) {
        let rom = fs::read(local_rom_path(case.rom)).unwrap();
        for (slot, handler) in [
            (minigame_slot, minigame_handler),
            (start_slot, start_handler),
            (end_slot, end_handler),
            (daily_slot, daily_handler),
        ] {
            assert_eq!(
                u32::from_le_bytes(
                    rom[dispatch + slot * 4..dispatch + slot * 4 + 4]
                        .try_into()
                        .unwrap()
                ),
                0x08000000u32 + handler as u32,
                "{} Harvest Sprite callable slot {slot:X}",
                case.name
            );
        }

        // This predicate deliberately calls GetMinigameExp, whose storage is
        // separate from the ordinary task-experience array.
        assert_eq!(
            call_target(&rom, minigame_handler + 0x46),
            minigame_getter,
            "{} minigame-experience getter",
            case.name
        );
        assert_eq!(
            call_target(&rom, start_handler + 0x5C),
            start_task,
            "{} task assignment setter",
            case.name
        );
        assert_eq!(
            call_target(&rom, end_handler + 0x2E),
            schedule_end,
            "{} assignment-end scheduler",
            case.name
        );
        assert_eq!(
            call_target(&rom, daily_handler + 0x2E),
            daily_complete,
            "{} runtime daily-work query",
            case.name
        );

        // All four native implementations clear the three-bit work-days-left
        // field in byte +0x1A and then store one. They do not clear the adjacent
        // two-bit current-task field, so cancellation takes effect at the next
        // daily task update rather than immediately.
        assert_eq!(
            &rom[schedule_end..schedule_end + 16],
            &[
                0x82, 0x7E, 0x1D, 0x21, 0x49, 0x42, 0x11, 0x40, 0x04, 0x22, 0x11, 0x43, 0x81, 0x76,
                0x70, 0x47
            ],
            "{} remaining-days field update",
            case.name
        );
    }
}

#[test]
fn native_harvest_sprite_minigame_slots_preserve_menu_order_despite_handler_address_order() {
    fn call_target(rom: &[u8], pc: usize) -> usize {
        let hi = u16::from_le_bytes(rom[pc..pc + 2].try_into().unwrap());
        let lo = u16::from_le_bytes(rom[pc + 2..pc + 4].try_into().unwrap());
        assert_eq!(hi & 0xF800, 0xF000);
        assert_eq!(lo & 0xF800, 0xF800);
        let raw = (((hi & 0x7FF) as i32) << 12) | (((lo & 0x7FF) as i32) << 1);
        (pc as i32 + 4 + ((raw << 9) >> 9)) as usize
    }

    for (case, (dispatch, first_slot, handlers, leaves)) in CASES.iter().zip([
        (
            0x3F904,
            0xD3,
            [0x4287E, 0x428DA, 0x428AC, 0x42908],
            [0x139F0, 0x13A94, 0x13B38, 0x1437C],
        ),
        (
            0x3FAF0,
            0xD6,
            [0x42AC2, 0x42B1E, 0x42AF0, 0x42B4C],
            [0x13B7C, 0x13C20, 0x13CC4, 0x14508],
        ),
        (
            0x3F578,
            0xD3,
            [0x424F2, 0x4254E, 0x42520, 0x4257C],
            [0x138C4, 0x13968, 0x13A0C, 0x14250],
        ),
        (
            0x3F84C,
            0xD6,
            [0x4281E, 0x4287A, 0x4284C, 0x428A8],
            [0x13A24, 0x13AC8, 0x13B6C, 0x143B0],
        ),
    ]) {
        let rom = fs::read(local_rom_path(case.rom)).unwrap();
        for index in 0..4 {
            let slot = first_slot + index;
            assert_eq!(
                u32::from_le_bytes(
                    rom[dispatch + slot * 4..dispatch + slot * 4 + 4]
                        .try_into()
                        .unwrap()
                ),
                0x08000000u32 + handlers[index] as u32,
                "{} minigame/contest slot {slot:X}",
                case.name
            );
            let call_offset = if index == 3 { 0x10 } else { 0x26 };
            assert_eq!(
                call_target(&rom, handlers[index] + call_offset),
                leaves[index],
                "{} minigame/contest leaf index {index}",
                case.name
            );
        }

        // Slots are animal care, harvesting, watering, then Chicken Festival.
        // Harvesting is intentionally stored after the watering handler in
        // address order; callable identity must follow the dispatch slot.
        assert!(
            handlers[1] > handlers[2],
            "{} non-monotonic handler order",
            case.name
        );
    }
}

#[test]
fn native_horse_race_interface_and_entry_preparation_follow_target_physical_slots() {
    fn call_target(rom: &[u8], pc: usize) -> usize {
        let hi = u16::from_le_bytes(rom[pc..pc + 2].try_into().unwrap());
        let lo = u16::from_le_bytes(rom[pc + 2..pc + 4].try_into().unwrap());
        assert_eq!(hi & 0xF800, 0xF000);
        assert_eq!(lo & 0xF800, 0xF800);
        let raw = (((hi & 0x7FF) as i32) << 12) | (((lo & 0x7FF) as i32) << 1);
        (pc as i32 + 4 + ((raw << 9) >> 9)) as usize
    }

    for (
        case,
        (
            dispatch,
            first_slot,
            race_handler,
            entries_handler,
            exchange_handler,
            race_leaf,
            exchange_leaf,
        ),
    ) in CASES.iter().zip([
        (0x3F904, 0xD7, 0x42920, 0x4294E, 0x429A8, 0x14410, 0x144BC),
        (0x3FAF0, 0xDA, 0x42B64, 0x42B92, 0x42BEC, 0x1459C, 0x14648),
        (0x3F578, 0xD7, 0x42594, 0x425C2, 0x4261C, 0x142E4, 0x14390),
        (0x3F84C, 0xDA, 0x428C0, 0x428EE, 0x42948, 0x14444, 0x144F0),
    ]) {
        let rom = fs::read(local_rom_path(case.rom)).unwrap();
        for (index, handler) in [race_handler, entries_handler, exchange_handler]
            .into_iter()
            .enumerate()
        {
            let slot = first_slot + index;
            assert_eq!(
                u32::from_le_bytes(
                    rom[dispatch + slot * 4..dispatch + slot * 4 + 4]
                        .try_into()
                        .unwrap()
                ),
                0x08000000u32 + handler as u32,
                "{} horse-race callable slot {slot:X}",
                case.name
            );
        }
        assert_eq!(
            call_target(&rom, race_handler + 0x26),
            race_leaf,
            "{} horse-race interface leaf",
            case.name
        );
        assert_eq!(
            call_target(&rom, exchange_handler + 0x10),
            exchange_leaf,
            "{} medal exchange leaf",
            case.name
        );

        // Entry preparation converts every nonzero script argument to one
        // before calling the native roster builder. This proves the public
        // parameter is a two-state include-player selector, not a race number.
        assert_eq!(
            &rom[entries_handler + 0x22..entries_handler + 0x2A],
            &[0x58, 0x42, 0x18, 0x43, 0xC4, 0x0F, 0xC8, 0x23],
            "{} entry-mode boolean normalization and 200 cap setup",
            case.name
        );
    }
}

#[test]
fn native_frisbee_and_animal_festival_setup_follow_target_physical_slots() {
    fn call_target(rom: &[u8], pc: usize) -> usize {
        let hi = u16::from_le_bytes(rom[pc..pc + 2].try_into().unwrap());
        let lo = u16::from_le_bytes(rom[pc + 2..pc + 4].try_into().unwrap());
        assert_eq!(hi & 0xF800, 0xF000);
        assert_eq!(lo & 0xF800, 0xF800);
        let raw = (((hi & 0x7FF) as i32) << 12) | (((lo & 0x7FF) as i32) << 1);
        (pc as i32 + 4 + ((raw << 9) >> 9)) as usize
    }

    for (case, (dispatch, first_slot, handlers, leaves)) in CASES.iter().zip([
        (
            0x3F904,
            0xDA,
            [0x429C0, 0x429F4, 0x42A0C],
            [0x14550, 0x145F8, 0x52230],
        ),
        (
            0x3FAF0,
            0xDD,
            [0x42C04, 0x42C38, 0x42C50],
            [0x146DC, 0x14784, 0x547E8],
        ),
        (
            0x3F578,
            0xDA,
            [0x42634, 0x42668, 0x42680],
            [0x14424, 0x144CC, 0x51FC0],
        ),
        (
            0x3F84C,
            0xDD,
            [0x42960, 0x42994, 0x429AC],
            [0x14584, 0x1462C, 0x544AC],
        ),
    ]) {
        let rom = fs::read(local_rom_path(case.rom)).unwrap();
        for (index, handler) in handlers.into_iter().enumerate() {
            let slot = first_slot + index;
            assert_eq!(
                u32::from_le_bytes(
                    rom[dispatch + slot * 4..dispatch + slot * 4 + 4]
                        .try_into()
                        .unwrap()
                ),
                0x08000000u32 + handler as u32,
                "{} frisbee/festival setup slot {slot:X}",
                case.name
            );
        }

        // RunFrisbeeGame pops exactly one VM value and normalizes every
        // nonzero value to one before starting the modal frisbee interface.
        assert_eq!(call_target(&rom, handlers[0] + 0x2C), leaves[0]);
        assert_eq!(
            &rom[handlers[0] + 0x26..handlers[0] + 0x2C],
            &[0x59, 0x42, 0x19, 0x43, 0xC9, 0x0F]
        );

        // The tournament entry has no VM arguments. The following setup entry
        // passes the persistent ten-record opponent array and the extracted
        // year value to its native generator.
        assert_eq!(call_target(&rom, handlers[1] + 0x10), leaves[1]);
        assert_eq!(call_target(&rom, handlers[2] + 0x16), leaves[2]);
        assert_eq!(
            u32::from_le_bytes(
                rom[handlers[2] + 0x20..handlers[2] + 0x24]
                    .try_into()
                    .unwrap()
            ),
            if case.name.starts_with("mfomt") {
                0x2C88
            } else {
                0x2C4C
            },
            "{} opponent record array offset",
            case.name
        );
        assert_eq!(
            u32::from_le_bytes(
                rom[handlers[2] + 0x24..handlers[2] + 0x28]
                    .try_into()
                    .unwrap()
            ),
            if case.name.starts_with("mfomt") {
                0x1CAC
            } else {
                0x1C9C
            },
            "{} year field offset",
            case.name
        );
    }
}

#[test]
fn native_completion_predicates_preserve_required_npc_exclusions_on_all_targets() {
    fn call_target(rom: &[u8], pc: usize) -> usize {
        let hi = u16::from_le_bytes(rom[pc..pc + 2].try_into().unwrap());
        let lo = u16::from_le_bytes(rom[pc + 2..pc + 4].try_into().unwrap());
        assert_eq!(hi & 0xF800, 0xF000);
        assert_eq!(lo & 0xF800, 0xF800);
        let raw = (((hi & 0x7FF) as i32) << 12) | (((lo & 0x7FF) as i32) << 1);
        (pc as i32 + 4 + ((raw << 9) >> 9)) as usize
    }

    for (
        case,
        (
            dispatch,
            first_slot,
            handlers,
            villagers_leaf,
            jump_table,
            check,
            skip,
            product_leaf,
            crop_leaf,
            mineral_leaf,
            fish_leaf,
        ),
    ) in CASES.iter().zip([
        (
            0x3F904,
            0xE3,
            [0x42C98, 0x42CBC, 0x42CE4, 0x42D1A, 0x42D38, 0x42D4E],
            0xA0518,
            0xA0538,
            0xA056C,
            0xA0582,
            0xB15C,
            0xB18C,
            0xB1CC,
            0x9CDCC,
        ),
        (
            0x3FAF0,
            0xE6,
            [0x42EC4, 0x42EE8, 0x42F00, 0x42F40, 0x42F5E, 0x42F7E],
            0xA56D8,
            0xA56F8,
            0xA572C,
            0xA5742,
            0xB1CC,
            0xB1FC,
            0xB23C,
            0xA1F50,
        ),
        (
            0x3F578,
            0xE3,
            [0x4290C, 0x42930, 0x42958, 0x4298E, 0x429AC, 0x429C2],
            0x9FF50,
            0x9FF70,
            0x9FFA4,
            0x9FFBA,
            0xB13C,
            0xB16C,
            0xB1AC,
            0x9C804,
        ),
        (
            0x3F84C,
            0xE6,
            [0x42C20, 0x42C44, 0x42C5C, 0x42C9C, 0x42CBA, 0x42CDA],
            0xA5118,
            0xA5138,
            0xA516C,
            0xA5182,
            0xB180,
            0xB1B0,
            0xB1F0,
            0xA1990,
        ),
    ]) {
        let rom = fs::read(local_rom_path(case.rom)).unwrap();
        for (index, handler) in handlers.into_iter().enumerate() {
            let slot = first_slot + index;
            assert_eq!(
                u32::from_le_bytes(
                    rom[dispatch + slot * 4..dispatch + slot * 4 + 4]
                        .try_into()
                        .unwrap()
                ),
                0x08000000u32 + handler as u32,
                "{} completion predicate slot {slot:X}",
                case.name
            );
        }
        assert_eq!(call_target(&rom, handlers[1] + 0x0C), villagers_leaf);

        // IDs 23..35 have an explicit jump table. Cliff (23), Kai (26),
        // Gourmet (30), Kappa (32), and the player's child (35) bypass the
        // presence/friendship test; all other IDs in this range are checked.
        let excluded_cases = [0usize, 3, 7, 9, 12];
        for index in 0..13 {
            let target = u32::from_le_bytes(
                rom[jump_table + index * 4..jump_table + index * 4 + 4]
                    .try_into()
                    .unwrap(),
            ) as usize
                - 0x08000000;
            assert_eq!(
                target,
                if excluded_cases.contains(&index) {
                    skip
                } else {
                    check
                },
                "{} character ID {} inclusion",
                case.name,
                index + 23
            );
        }
        assert_eq!(
            &rom[villagers_leaf + 0x64..villagers_leaf + 0x68],
            &[0xF9, 0x28, 0x00, 0xD8]
        );
        assert_eq!(
            &rom[villagers_leaf + 0x6C..villagers_leaf + 0x70],
            &[0x2A, 0x2C, 0xCB, 0xD9]
        );

        assert_eq!(
            u16::from_le_bytes(
                rom[product_leaf + 0x24..product_leaf + 0x26]
                    .try_into()
                    .unwrap()
            ),
            0x2D66
        );
        assert_eq!(
            u16::from_le_bytes(rom[crop_leaf + 0x32..crop_leaf + 0x34].try_into().unwrap()),
            0x2D0E
        );
        assert_eq!(
            u16::from_le_bytes(
                rom[mineral_leaf + 0x32..mineral_leaf + 0x34]
                    .try_into()
                    .unwrap()
            ),
            0x2D13
        );

        // The complete all-products predicate is byte-identical across all
        // four ROMs. Crop and mineral predicates have identical executable
        // bodies; only their final relocated table pointer differs.
        let reference = fs::read(local_rom_path(CASES[0].rom)).unwrap();
        assert_eq!(
            &rom[product_leaf..product_leaf + 0x30],
            &reference[0xB15C..0xB15C + 0x30],
            "{} all-product predicate body",
            case.name
        );
        assert_eq!(
            &rom[crop_leaf..crop_leaf + 0x2C],
            &reference[0xB18C..0xB18C + 0x2C],
            "{} crop predicate body",
            case.name
        );
        assert_eq!(
            &rom[mineral_leaf..mineral_leaf + 0x2C],
            &reference[0xB1CC..0xB1CC + 0x2C],
            "{} mineral predicate body",
            case.name
        );

        let crop_table =
            u32::from_le_bytes(rom[crop_leaf + 0x2C..crop_leaf + 0x30].try_into().unwrap())
                as usize
                - 0x08000000;
        let mineral_table = u32::from_le_bytes(
            rom[mineral_leaf + 0x2C..mineral_leaf + 0x30]
                .try_into()
                .unwrap(),
        ) as usize
            - 0x08000000;
        assert_eq!(
            &rom[crop_table..crop_table + 15],
            &[0, 1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12, 13, 14]
        );
        assert_eq!(
            &rom[mineral_table..mineral_table + 20],
            &[83, 84, 85, 86, 87, 88, 89, 90, 91, 92, 93, 94, 95, 96, 97, 98, 99, 100, 101, 102]
        );

        // The fish predicate walks eight-byte records 8..58 inclusive. Slots
        // 0..7 are non-fish catches; slots 53..58 (Fish Kings) remain included.
        assert_eq!(
            &rom[fish_leaf..fish_leaf + 42],
            &fs::read(local_rom_path(CASES[0].rom)).unwrap()[0x9CDCC..0x9CDF6]
        );
    }
}

#[test]
fn native_total_fish_count_and_mythic_milestone_bit_match_all_targets() {
    fn call_target(rom: &[u8], pc: usize) -> usize {
        let hi = u16::from_le_bytes(rom[pc..pc + 2].try_into().unwrap());
        let lo = u16::from_le_bytes(rom[pc + 2..pc + 4].try_into().unwrap());
        assert_eq!(hi & 0xF800, 0xF000);
        assert_eq!(lo & 0xF800, 0xF800);
        let raw = (((hi & 0x7FF) as i32) << 12) | (((lo & 0x7FF) as i32) << 1);
        (pc as i32 + 4 + ((raw << 9) >> 9)) as usize
    }

    let reference = fs::read(local_rom_path(CASES[0].rom)).unwrap();
    for (
        case,
        (
            table,
            first_slot,
            fish_handler,
            fish_call,
            mythic_handler,
            fish_leaf,
            mythic_leaf,
            collect_slot,
            collect_handler,
            order_table,
            set_block,
            setter,
        ),
    ) in CASES.iter().zip([
        (
            0x3F904, 0xE9, 0x42D72, 0x0E, 0x42D94, 0x9CDEC, 0x10E60, 0xF3, 0x43848, 0x43C90,
            0x43CF8, 0x10F48,
        ),
        (
            0x3FAF0, 0xEC, 0x42FA4, 0x0C, 0x42FC0, 0xA1F70, 0x10F44, 0xF6, 0x43AA4, 0x43EEC,
            0x43F54, 0x11028,
        ),
        (
            0x3F578, 0xE9, 0x429E6, 0x0E, 0x42A08, 0x9C824, 0x10E40, 0xF3, 0x434BC, 0x43904,
            0x4396C, 0x10F28,
        ),
        (
            0x3F84C, 0xEC, 0x42D00, 0x0C, 0x42D1C, 0xA19B0, 0x10EF8, 0xF6, 0x43800, 0x43C48,
            0x43CB0, 0x10FDC,
        ),
    ]) {
        let rom = fs::read(local_rom_path(case.rom)).unwrap();
        for (slot, handler) in [(first_slot, fish_handler), (first_slot + 1, mythic_handler)] {
            assert_eq!(
                u32::from_le_bytes(
                    rom[table + slot * 4..table + slot * 4 + 4]
                        .try_into()
                        .unwrap()
                ),
                0x08000000 + handler as u32,
                "{} callable slot {slot:X}",
                case.name
            );
        }
        assert_eq!(call_target(&rom, fish_handler + fish_call), fish_leaf);
        assert_eq!(call_target(&rom, mythic_handler + 0x08), mythic_leaf);

        // The complete counter body is identical in all four ROMs. It starts
        // at record 8, includes record 58, and carries both saturation values.
        assert_eq!(
            &rom[fish_leaf..fish_leaf + 0x30],
            &reference[0x9CDEC..0x9CE1C]
        );
        assert_eq!(
            u32::from_le_bytes(rom[fish_leaf + 0x28..fish_leaf + 0x2C].try_into().unwrap()),
            999_999_999
        );
        assert_eq!(
            u32::from_le_bytes(rom[fish_leaf + 0x2C..fish_leaf + 0x30].try_into().unwrap()),
            1_000_000_000
        );

        // The public Mythic milestone predicate is physically just byte 0,
        // bit 3. Do not infer its writer from the neighboring cottage bits.
        assert_eq!(
            &rom[mythic_leaf..mythic_leaf + 8],
            &[0x00, 0x78, 0x00, 0x07, 0xC0, 0x0F, 0x70, 0x47]
        );

        assert_eq!(
            u32::from_le_bytes(
                rom[table + collect_slot * 4..table + collect_slot * 4 + 4]
                    .try_into()
                    .unwrap()
            ),
            0x08000000 + collect_handler as u32
        );
        // The table is indexed by order_id - 5. Only IDs 5, 10, 15, 20,
        // 25 and 30, the six Mythic tool orders, enter the bit setter.
        for index in 0..26 {
            let target = u32::from_le_bytes(
                rom[order_table + index * 4..order_table + index * 4 + 4]
                    .try_into()
                    .unwrap(),
            ) as usize
                - 0x08000000;
            assert_eq!(
                target == set_block,
                index % 5 == 0,
                "{} order {}",
                case.name,
                index + 5
            );
        }
        assert_eq!(call_target(&rom, set_block + 8), setter);
        assert_eq!(
            &rom[setter..setter + 10],
            &[0x01, 0x78, 0x08, 0x22, 0x11, 0x43, 0x01, 0x70, 0x70, 0x47]
        );
    }
}

#[test]
fn native_blacksmith_order_id_and_ready_fields_match_all_targets() {
    fn call_target(rom: &[u8], pc: usize) -> usize {
        let hi = u16::from_le_bytes(rom[pc..pc + 2].try_into().unwrap());
        let lo = u16::from_le_bytes(rom[pc + 2..pc + 4].try_into().unwrap());
        let raw = (((hi & 0x7FF) as i32) << 12) | (((lo & 0x7FF) as i32) << 1);
        assert_eq!((hi & 0xF800, lo & 0xF800), (0xF000, 0xF800));
        (pc as i32 + 4 + ((raw << 9) >> 9)) as usize
    }

    let reference = fs::read(local_rom_path(CASES[0].rom)).unwrap();
    for (case, (table, first_slot, get_handler, ready_handler, get_leaf, ready_leaf)) in
        CASES.iter().zip([
            (0x3F904, 0xF1, 0x437FC, 0x43820, 0x9ECD8, 0x9ECE0),
            (0x3FAF0, 0xF4, 0x43A60, 0x43A7C, 0xA3E6C, 0xA3E74),
            (0x3F578, 0xF1, 0x43470, 0x43494, 0x9E710, 0x9E718),
            (0x3F84C, 0xF4, 0x437BC, 0x437D8, 0xA38AC, 0xA38B4),
        ])
    {
        let rom = fs::read(local_rom_path(case.rom)).unwrap();
        for (slot, handler) in [(first_slot, get_handler), (first_slot + 1, ready_handler)] {
            assert_eq!(
                u32::from_le_bytes(
                    rom[table + slot * 4..table + slot * 4 + 4]
                        .try_into()
                        .unwrap()
                ),
                0x08000000 + handler as u32,
                "{} blacksmith slot {slot:X}",
                case.name
            );
        }
        assert_eq!(call_target(&rom, get_handler + 0x0C), get_leaf);
        assert_eq!(call_target(&rom, ready_handler + 0x0C), ready_leaf);
        // Getter: byte offset 0x14, low six bits.
        assert_eq!(
            &rom[get_leaf..get_leaf + 8],
            &[0x00, 0x7D, 0x80, 0x06, 0x80, 0x0E, 0x70, 0x47]
        );
        // Ready: order ID must be nonzero and the packed bits 6-8 timer must
        // be zero. The complete implementation is identical in all targets.
        assert_eq!(
            &rom[ready_leaf..ready_leaf + 0x22],
            &reference[0x9ECE0..0x9ED02]
        );
    }
}

#[test]
fn native_barn_feed_setters_preserve_capacity_guards() {
    let reference = fs::read(local_rom_path(CASES[0].rom)).unwrap();
    // The chicken setter is likewise an idempotent feed-bit write, not an
    // inventory debit. Include its trailing mask literal, not the next routine.
    for (case, setter) in CASES.iter().zip([0xC814, 0xC888, 0xC7F4, 0xC83C]) {
        let rom = fs::read(local_rom_path(case.rom)).unwrap();
        assert_eq!(&rom[setter..setter + 60], &reference[0xC814..0xC850]);
    }
    for (case, (ordinary, pregnancy, capacity, pregnancy_capacity)) in CASES.iter().zip([
        (0xD2BC, 0xD334, 0xCE74, 0xCE9C),
        (0xD330, 0xD3A8, 0xCEE8, 0xCF10),
        (0xD29C, 0xD314, 0xCE54, 0xCE7C),
        (0xD2E4, 0xD35C, 0xCE9C, 0xCEC4),
    ]) {
        let rom = fs::read(local_rom_path(case.rom)).unwrap();
        for (address, base, length) in [
            (ordinary, 0xD2BC, 64),
            (pregnancy, 0xD334, 56),
            (capacity, 0xCE74, 40),
            (pregnancy_capacity, 0xCE9C, 10),
        ] {
            assert_eq!(
                &rom[address..address + length],
                &reference[base..base + length],
                "{} barn routine {address:X}",
                case.name
            );
        }
        for (setter, expected) in [(ordinary, capacity), (pregnancy, pregnancy_capacity)] {
            let pc = setter + 6;
            let hi = u16::from_le_bytes(rom[pc..pc + 2].try_into().unwrap());
            let lo = u16::from_le_bytes(rom[pc + 2..pc + 4].try_into().unwrap());
            assert_eq!(hi & 0xF800, 0xF000);
            assert_eq!(lo & 0xF800, 0xF800);
            let raw = (i32::from(hi & 0x7FF) << 12) | (i32::from(lo & 0x7FF) << 1);
            assert_eq!(pc as i32 + 4 + ((raw << 9) >> 9), expected as i32);
        }
    }
}

#[test]
fn native_barn_feed_refresh_keeps_unchecked_coordinate_lookup() {
    let reference = fs::read(local_rom_path(CASES[0].rom)).unwrap();
    for (case, (table, thunk, refresh)) in CASES.iter().zip([
        (0xE5EC4, 0x1DD60, 0xA607C),
        (0xEE404, 0x1DEE0, 0xAB2B0),
        (0xE5304, 0x1DAF4, 0xA5AB4),
        (0xEDF14, 0x1DD54, 0xAACF0),
    ]) {
        let rom = fs::read(local_rom_path(case.rom)).unwrap();
        assert_eq!(
            u32::from_le_bytes(rom[table + 0x9C..table + 0xA0].try_into().unwrap()),
            thunk as u32 + 0x08000001
        );
        assert_eq!(&rom[thunk..thunk + 4], &[0x00, 0xB5, 0x40, 0x68]);
        let descriptor =
            u32::from_le_bytes(rom[refresh + 0x38..refresh + 0x3C].try_into().unwrap());
        let descriptor = (descriptor - 0x08000000) as usize;
        let pregnancy_coordinates =
            u32::from_le_bytes(rom[refresh + 0x3C..refresh + 0x40].try_into().unwrap());
        let pregnancy_coordinates = (pregnancy_coordinates - 0x08000000) as usize;
        let ordinary_coordinates = [
            26, 8, 31, 8, 36, 8, 41, 8, 26, 20, 31, 20, 36, 20, 41, 20, 56, 8, 61, 8, 66, 8, 71, 8,
            56, 20, 61, 20, 66, 20, 71, 20,
        ];
        for (pointer_offset, length) in [(0x4C, 16), (0x90, 32)] {
            let pointer = refresh + pointer_offset;
            let coordinates = u32::from_le_bytes(rom[pointer..pointer + 4].try_into().unwrap());
            let coordinates = (coordinates - 0x08000000) as usize;
            assert_eq!(
                &rom[coordinates..coordinates + length],
                &ordinary_coordinates[..length]
            );
        }
        // IDs16/17: same X, north/south Y. These are tile coordinates,
        // not pixel-space entity coordinates or an animal roster index.
        assert_eq!(
            &rom[pregnancy_coordinates..pregnancy_coordinates + 4],
            &[3, 8, 3, 20]
        );
        // All four use a 2x2 patch on the second map layer. No optional
        // per-tile attribute patch is present in this descriptor.
        assert_eq!(&rom[descriptor..descriptor + 8], &[2, 2, 0, 0, 0, 0, 0, 0]);
        assert_eq!(&rom[descriptor + 12..descriptor + 24], &[0; 12]);
        let pixels = u32::from_le_bytes(rom[descriptor + 8..descriptor + 12].try_into().unwrap());
        let pixels = (pixels - 0x08000000) as usize;
        assert!(pixels.checked_add(8).is_some_and(|end| end <= rom.len()));
        // Complete refresh routine, excluding only its ten relocated data
        // pointer words. Instructions (including BLs) match in these ROMs.
        for offset in 0..0xBC {
            if [0x38, 0x3C, 0x48, 0x4C, 0x6C, 0x70, 0x8C, 0x90, 0xB4, 0xB8]
                .iter()
                .any(|start| (*start..*start + 4).contains(&offset))
            {
                continue;
            }
            assert_eq!(
                rom[refresh + offset],
                reference[0xA607C + offset],
                "{} refresh+{offset:X}",
                case.name
            );
        }
    }
}

#[test]
fn native_clock_target_advances_simulation_on_all_targets() {
    let reference = fs::read(local_rom_path(CASES[0].rom)).unwrap();
    for (case, (dispatch_table, slot, handler, wrapper)) in CASES.iter().zip([
        (0x3F904, 0xF4, 0x43DBC, 0x14318),
        (0x3FAF0, 0xF7, 0x44018, 0x144A4),
        (0x3F578, 0xF4, 0x43A30, 0x141EC),
        (0x3F84C, 0xF7, 0x43D74, 0x1434C),
    ]) {
        let rom = fs::read(local_rom_path(case.rom)).unwrap();
        let slot = dispatch_table + slot * 4;
        assert_eq!(
            u32::from_le_bytes(rom[slot..slot + 4].try_into().unwrap()),
            handler as u32 + 0x08000000
        );
        // Target time packing differs only in the caller's stack temporary.
        assert_eq!(
            &rom[handler + 0x3E..handler + 0x46],
            &reference[0x43DFA..0x43E02]
        );
        assert_eq!(
            &rom[handler + 0x48..handler + 0x66],
            &reference[0x43E04..0x43E22]
        );
        let pc = handler + 0x68;
        let hi = u16::from_le_bytes(rom[pc..pc + 2].try_into().unwrap());
        let lo = u16::from_le_bytes(rom[pc + 2..pc + 4].try_into().unwrap());
        assert_eq!(hi & 0xF800, 0xF000);
        assert_eq!(lo & 0xF800, 0xF800);
        let raw = (i32::from(hi & 0x7FF) << 12) | (i32::from(lo & 0x7FF) << 1);
        assert_eq!(pc as i32 + 4 + ((raw << 9) >> 9), wrapper as i32);
        // Resolve the active scene manager, then its vtable+BC time method.
        assert_eq!(&rom[wrapper..wrapper + 16], &reference[0x14318..0x14328]);
    }
    for (case, (constructor, table, clock)) in CASES.iter().zip([
        (0x175B4, 0xE5EC4, 0x1EAA0),
        (0x17740, 0xEE404, 0x1EC28),
        (0x17348, 0xE5304, 0x1E834),
        (0x175B4, 0xEDF14, 0x1EA9C),
    ]) {
        let rom = fs::read(local_rom_path(case.rom)).unwrap();
        let load = constructor + 0x1E;
        let opcode = u16::from_le_bytes(rom[load..load + 2].try_into().unwrap());
        assert_eq!(opcode & 0xFF00, 0x4900);
        let literal = ((load + 4) & !3) + usize::from(opcode & 255) * 4;
        assert_eq!(
            u32::from_le_bytes(rom[literal..literal + 4].try_into().unwrap()),
            table as u32 + 0x08000000
        );
        assert_eq!(&rom[load + 2..load + 6], &[0x38, 0x1C, 0x02, 0xC0]);
        assert_eq!(
            u32::from_le_bytes(rom[table + 0xBC..table + 0xC0].try_into().unwrap()),
            clock as u32 + 0x08000001
        );
        // Complete code body before the final literal pool: includes minute,
        // hour, date, season/year rollover and the post-advance equality test.
        // Two embedded entity-order table pointers and relocated BL operands
        // differ; do not ignore arbitrary instruction differences.
        let mut offset = 0;
        while offset < 0x340 {
            if (0x11C..0x124).contains(&offset) {
                offset += 2;
                continue;
            }
            let half = |bytes: &[u8], at| u16::from_le_bytes(bytes[at..at + 2].try_into().unwrap());
            let hi = half(&reference, 0x1EAA0 + offset);
            let lo = half(&reference, 0x1EAA0 + offset + 2);
            if hi & 0xF800 == 0xF000 && lo & 0xF800 == 0xF800 {
                assert_eq!(half(&rom, clock + offset) & 0xF800, 0xF000);
                assert_eq!(half(&rom, clock + offset + 2) & 0xF800, 0xF800);
                offset += 4;
            } else {
                assert_eq!(
                    half(&rom, clock + offset),
                    hi,
                    "{} clock+{offset:X}",
                    case.name
                );
                offset += 2;
            }
        }
    }
}

#[test]
fn native_new_day_callable_reaches_visit_reset_before_entity_rebuild() {
    let branch_target = |rom: &[u8], pc: usize| {
        let hi = u16::from_le_bytes(rom[pc..pc + 2].try_into().unwrap());
        let lo = u16::from_le_bytes(rom[pc + 2..pc + 4].try_into().unwrap());
        assert_eq!(hi & 0xF800, 0xF000);
        assert_eq!(lo & 0xF800, 0xF800);
        let raw = (i32::from(hi & 0x7FF) << 12) | (i32::from(lo & 0x7FF) << 1);
        (pc as i32 + 4 + ((raw << 9) >> 9)) as usize
    };
    for (case, (table, slot, handler, wrapper, update, reset_call, reset, manager, owner)) in
        CASES.iter().zip([
            (
                0x3F904, 0x128, 0x44038, 0x14AF8, 0x10F54, 0x111CC, 0xA0B18, 0x1FB7C, 0x1CD4,
            ),
            (
                0x3FAF0, 0x12C, 0x442AA, 0x14C84, 0x11034, 0x112AE, 0xA5CE8, 0x1FD04, 0x1CE4,
            ),
            (
                0x3F578, 0x128, 0x43CAC, 0x149CC, 0x10F34, 0x111AC, 0xA0550, 0x1F910, 0x1CD4,
            ),
            (
                0x3F84C, 0x12C, 0x44006, 0x14B2C, 0x10FE8, 0x11262, 0xA5728, 0x1FB78, 0x1CE4,
            ),
        ])
    {
        let rom = fs::read(local_rom_path(case.rom)).unwrap();
        let entry = table + slot * 4;
        assert_eq!(
            u32::from_le_bytes(rom[entry..entry + 4].try_into().unwrap()),
            handler as u32 + 0x08000000
        );
        assert_eq!(branch_target(&rom, handler + 0x10), wrapper);
        assert_eq!(branch_target(&rom, wrapper + 0x2A), update);
        assert_eq!(branch_target(&rom, wrapper + 0x3E), manager);
        assert_eq!(branch_target(&rom, reset_call), reset);
        // ldr r0,literal; add r4,r6,r0 ... mov r0,r4: same game owner
        // consumed by the position-triggered counter increment.
        let load = reset_call - 18;
        let instruction = u16::from_le_bytes(rom[load..load + 2].try_into().unwrap());
        assert_eq!(instruction & 0xFF00, 0x4800);
        let literal = ((load + 4) & !3) + usize::from(instruction & 0xFF) * 4;
        assert_eq!(
            u32::from_le_bytes(rom[literal..literal + 4].try_into().unwrap()),
            owner
        );
        assert_eq!(&rom[load + 2..load + 4], &[0x34, 0x18]);
        assert_eq!(&rom[reset_call - 6..reset_call - 4], &[0x20, 0x1C]);
    }
}

#[test]
fn native_mfomt_church_counter_and_reward_data_match_regions() {
    let us = fs::read(local_rom_path(CASES[1].rom)).unwrap();
    let jp = fs::read(local_rom_path(CASES[3].rom)).unwrap();
    // Complete updater: low six bits saturate at 50; bit6 prevents another
    // increment until cleared. This is not evidence of a confession counter.
    assert_eq!(&us[0xA5BF0..0xA5C30], &jp[0xA5630..0xA5670]);
    assert_eq!(&us[0xA1604..0xA161A], &jp[0xA1044..0xA105A]);
    // Full candidate-selection resolver, reward stage getter/incrementer,
    // and love updater are identical across these two regions.
    for (a, b, length) in [
        (0xA562C, 0xA506C, 0x96),
        (0xA3650, 0xA3090, 8),
        (0xA3658, 0xA3098, 36),
        (0xA36E4, 0xA3124, 36),
    ] {
        assert_eq!(&us[a..a + length], &jp[b..b + length]);
    }
    for (rom, lookup) in [(&us, 0xA5870), (&jp, 0xA52B0)] {
        let table = u32::from_le_bytes(rom[lookup + 20..lookup + 24].try_into().unwrap());
        let slot = (table - 0x08000000) as usize + (23 - 1) * 4;
        let branch = u32::from_le_bytes(rom[slot..slot + 4].try_into().unwrap());
        let branch = (branch - 0x08000000) as usize;
        // Character23: mov r1,0x8e; lsl r1,2; branch to add r0,r2,r1.
        assert_eq!(
            &rom[branch..branch + 6],
            &[0x8E, 0x21, 0x89, 0x00, 0x33, 0xE0]
        );
        assert_eq!(&rom[branch + 0x6E..branch + 0x70], &[0x50, 0x18]);
    }
    for (rom, reset, gate) in [(&us, 0xA5CE8, 0xA633E), (&jp, 0xA5728, 0xA5D7E)] {
        // Matching reset prologue; update cadence remains separately audited.
        assert_eq!(&rom[reset..reset + 0x20], &us[0xA5CE8..0xA5D08]);
        assert_eq!(&rom[gate..gate + 0x2A], &us[0xA633E..0xA6368]);
        let table = u32::from_le_bytes(rom[gate + 0x2E..gate + 0x32].try_into().unwrap());
        let table = (table - 0x08000000) as usize;
        assert_eq!(&rom[table..table + 5], &[10, 20, 30, 40, 50]);
        assert_eq!(&rom[gate + 0x82..gate + 0x86], &2500u32.to_le_bytes());
    }
}

#[test]
fn native_mfomt_church_visit_gate_excludes_romance_events_and_music_festival() {
    let us = fs::read(local_rom_path(CASES[1].rom)).unwrap();
    let jp = fs::read(local_rom_path(CASES[3].rom)).unwrap();
    let gates = [(&us, 0xE053E), (&jp, 0xE0052)];

    // Event-state +0x0C is the active romance-event variable ID. The native
    // gate excludes the five ordinary-spouse marriage variables, Cliff's
    // complete heart/proposal chain, and the three special-spouse wedding
    // variables before incrementing the church-visit counter.
    let selectors = [
        (0x18, 0x0050u16),
        (0x28, 0x005A),
        (0x38, 0x0064),
        (0x48, 0x006D),
        (0x58, 0x0077),
        (0x68, 0x017B),
        (0x78, 0x0186),
        (0x8A, 0x0191),
        (0xAA, 0x005C),
        (0xBA, 0x005D),
        (0xCA, 0x005F),
        (0xDA, 0x0061),
        (0xEA, 0x0063),
    ];

    for (rom, gate) in gates {
        for (offset, expected) in selectors {
            let pc = gate + offset;
            let value = match expected {
                0x017B | 0x0191 => {
                    let ldr = u16::from_le_bytes(rom[pc..pc + 2].try_into().unwrap());
                    assert_eq!(ldr & 0xF800, 0x4800);
                    let literal = ((pc + 4) & !3) + usize::from(ldr & 0xFF) * 4;
                    u32::from_le_bytes(rom[literal..literal + 4].try_into().unwrap()) as u16
                }
                0x0186 => {
                    assert_eq!(&rom[pc..pc + 4], &[0xC3, 0x21, 0x49, 0x00]);
                    0x0186
                }
                _ => {
                    let mov = u16::from_le_bytes(rom[pc..pc + 2].try_into().unwrap());
                    assert_eq!(mov & 0xFF00, 0x2100);
                    mov & 0xFF
                }
            };
            assert_eq!(value, expected);
        }

        // save+0x21C0 bits4..5 are VAR_MUSIC_FESTIVAL_ACTIVE. Value 1 is
        // FESTIVAL_PHASE_INITIAL and suppresses this visit increment.
        assert_eq!(
            &rom[gate + 0x96..gate + 0xA8],
            &[
                0x22, 0x68, 0x87, 0x21, 0x89, 0x01, 0x50, 0x18, 0x01, 0x78, 0x30, 0x20, 0x08, 0x40,
                0x10, 0x28, 0x2B, 0xD0
            ]
        );
    }
}

#[test]
fn native_fomt_library_reward_gate_matches_us_and_jp() {
    let us = fs::read(local_rom_path(CASES[0].rom)).unwrap();
    let jp = fs::read(local_rom_path(CASES[2].rom)).unwrap();
    // Complete first-candidate resolver: event count > 5, in character order
    // 3, 12, 19, 21, 25, 31; zero when none qualifies. Not an all-spouses set.
    assert_eq!(&us[0xA0490..0xA0504], &jp[0x9FEC8..0x9FF3C]);
    for (rom, update, resolver, gate, table) in [
        (&us, 0xA0B18, 0xA0490, 0xA112A, 0x10410E),
        (&jp, 0xA0550, 0x9FEC8, 0xA0B62, 0x103C02),
    ] {
        // Link the guard reset/update to the actual resolver, not just a
        // matching standalone function. Decode the signed Thumb BL offset.
        let pc = update + 0x20;
        let hi = u16::from_le_bytes(rom[pc..pc + 2].try_into().unwrap());
        let lo = u16::from_le_bytes(rom[pc + 2..pc + 4].try_into().unwrap());
        assert_eq!(hi & 0xF800, 0xF000);
        assert_eq!(lo & 0xF800, 0xF800);
        let delta = (((i32::from(hi & 0x7FF) << 12) | (i32::from(lo & 0x7FF) << 1)) << 9) >> 9;
        assert_eq!(pc as i32 + 4 + delta, resolver);
        assert_eq!(&rom[update..update + 0x20], &us[0xA0B18..0xA0B38]);
        // Recipient +0x154, stage <=4, then no candidate or character12.
        assert_eq!(&rom[gate..gate + 26], &us[0xA112A..0xA1144]);
        assert_eq!(&rom[table..table + 5], &[30, 60, 90, 120, 150]);
        assert_eq!(
            u32::from_le_bytes(rom[gate + 0x2E..gate + 0x32].try_into().unwrap()),
            (table as u32) + 0x08000000
        );
        assert_eq!(&rom[gate + 0x2A..gate + 0x2E], &2500u32.to_le_bytes());
    }
}

#[test]
fn native_deepest_mine_floor_updater_matches_all_targets() {
    let reference = fs::read(local_rom_path(CASES[0].rom)).unwrap();
    for (case, offset) in CASES.iter().zip([0xEECC, 0xEF6C, 0xEEAC, 0xEF20]) {
        let rom = fs::read(local_rom_path(case.rom)).unwrap();
        // Complete unsigned maximum update of Farmer+4C bits7..14, with
        // low-eight-bit storage and a Boolean changed return value.
        assert_eq!(
            &rom[offset..offset + 44],
            &reference[0xEECC..0xEEF8],
            "{} maximum mine depth",
            case.name
        );
    }
}

#[test]
fn native_hide_entity_selects_family_specific_off_map_location() {
    for (case, (wrapper, map)) in CASES.iter().zip([
        (0x1221C, 0x234),
        (0x12308, 0x23A),
        (0x120EC, 0x234),
        (0x121AC, 0x23A),
    ]) {
        let rom = fs::read(local_rom_path(case.rom)).unwrap();
        assert_eq!(
            &rom[wrapper..wrapper + 12],
            &[0x00, 0xB5, 0x40, 0x68, 0xA8, 0x30, 0x00, 0x68, 0x02, 0x68, 0x12, 0x6C]
        );
        assert_eq!(&rom[wrapper + 16..wrapper + 18], &[0x00, 0x28]);
        if map == 0x234 {
            // Immediate 0x8D shifted by two, after a null-entity branch.
            assert_eq!(
                &rom[wrapper + 18..wrapper + 24],
                &[0x03, 0xD0, 0x8D, 0x21, 0x89, 0x00]
            );
            assert_eq!(0x8D << 2, map);
        } else {
            // MFoMT uses a PC-relative literal, not the FoMT immediate.
            assert_eq!(&rom[wrapper + 18..wrapper + 22], &[0x02, 0xD0, 0x02, 0x49]);
            assert_eq!(
                u32::from_le_bytes(rom[wrapper + 32..wrapper + 36].try_into().unwrap()),
                map,
                "{} MAP_NONE",
                case.name
            );
        }
    }
}

#[test]
fn native_entity_script_clear_passes_zero_to_virtual_setter() {
    let reference = fs::read(local_rom_path(CASES[0].rom)).unwrap();
    for (case, base) in CASES.iter().zip([0x14264, 0x143F0, 0x14138, 0x14298]) {
        let rom = fs::read(local_rom_path(case.rom)).unwrap();
        // Compare every non-relocated instruction in both complete wrappers.
        // Lookup uses virtual +40; a non-null entity uses virtual +38.
        // Set preserves r2 in r4, while Clear explicitly passes r1 = 0.
        for (offset, length, calls) in [(0, 44, [14, 32]), (44, 40, [12, 30])] {
            for index in 0..length {
                if calls.iter().any(|call| (*call..*call + 4).contains(&index)) {
                    continue;
                }
                assert_eq!(
                    rom[base + offset + index],
                    reference[0x14264 + offset + index],
                    "{} wrapper {offset:#x} byte {index:#x}",
                    case.name
                );
            }
        }
    }
}

#[test]
fn native_entity_script_physical_slots_reach_expected_wrappers() {
    let reference = fs::read(local_rom_path(CASES[0].rom)).unwrap();
    for (case, (table, slot, wrapper)) in CASES.iter().zip([
        (0x3F904, 0x89, 0x14264),
        (0x3FAF0, 0x8C, 0x143F0),
        (0x3F578, 0x89, 0x14138),
        (0x3F84C, 0x8C, 0x14298),
    ]) {
        let rom = fs::read(local_rom_path(case.rom)).unwrap();
        for (index, (original, call, expected)) in
            [(0x41F80, 0x3E, wrapper), (0x41FC6, 0x26, wrapper + 44)]
                .into_iter()
                .enumerate()
        {
            let entry = table + (slot + index) * 4;
            let handler =
                u32::from_le_bytes(rom[entry..entry + 4].try_into().unwrap()) as usize - 0x08000000;
            // Complete stack-pop and event-object null guard, before relocated BLs.
            assert_eq!(
                &rom[handler..handler + call - 4],
                &reference[original..original + call - 4],
                "{} argument order",
                case.name
            );
            let pc = handler + call;
            let hi = u16::from_le_bytes(rom[pc..pc + 2].try_into().unwrap());
            let lo = u16::from_le_bytes(rom[pc + 2..pc + 4].try_into().unwrap());
            assert_eq!(hi & 0xF800, 0xF000);
            assert_eq!(lo & 0xF800, 0xF800);
            let displacement =
                (((i32::from(hi & 0x7FF) << 12) | (i32::from(lo & 0x7FF) << 1)) << 9) >> 9;
            assert_eq!(
                pc as i32 + 4 + displacement,
                expected,
                "{} binding slot {index}",
                case.name
            );
        }
    }
}

#[test]
fn native_record_entity_factory_has_exactly_four_slots() {
    for (case, (table, block, getter)) in CASES.iter().zip([
        (0x1A924, 0x1B390, 0x329CC),
        (0x1AAA8, 0x1B514, 0x32D88),
        (0x1A6B8, 0x1B124, 0x32760),
        (0x1A91C, 0x1B388, 0x32BFC),
    ]) {
        let rom = fs::read(local_rom_path(case.rom)).unwrap();
        let slots: Vec<_> = (0..94)
            .filter(|slot| {
                let entry = table + slot * 4;
                u32::from_le_bytes(rom[entry..entry + 4].try_into().unwrap())
                    == 0x08000000 + block as u32
            })
            .collect();
        assert_eq!(slots, [70, 71, 72, 73], "{} record slots", case.name);
        // Selection and record index (ID - 70), not a guessed gameplay role.
        assert_eq!(&rom[block..block + 4], &[0x55, 0x46, 0x46, 0x3D]);
        // Follow the factory BL, then the constructor's actual vtable literal.
        let pc = block + 30;
        let hi = u16::from_le_bytes(rom[pc..pc + 2].try_into().unwrap());
        let lo = u16::from_le_bytes(rom[pc + 2..pc + 4].try_into().unwrap());
        assert_eq!(hi & 0xF800, 0xF000);
        assert_eq!(lo & 0xF800, 0xF800);
        let displacement =
            (((i32::from(hi & 0x7FF) << 12) | (i32::from(lo & 0x7FF) << 1)) << 9) >> 9;
        let constructor = (pc as i32 + 4 + displacement) as usize;
        assert_eq!(constructor + 0x98, getter, "{} constructor", case.name);
        assert_eq!(
            &rom[constructor + 20..constructor + 26],
            &[0x03, 0x48, 0x68, 0x61, 0x2C, 0x63]
        );
        let table = u32::from_le_bytes(rom[constructor + 36..constructor + 40].try_into().unwrap())
            as usize
            - 0x08000000;
        for (slot, routine) in [(0x34, getter), (0x38, getter + 8)] {
            assert_eq!(
                u32::from_le_bytes(rom[table + slot..table + slot + 4].try_into().unwrap()),
                0x08000001 + routine as u32,
                "{} record vtable slot {slot:#x}",
                case.name
            );
        }
        // Complete accessor pair: record+0A halfword, no NPC zero fallback.
        assert_eq!(
            &rom[getter..getter + 14],
            &[0x00, 0x6B, 0x40, 0x89, 0x70, 0x47, 0x00, 0x00, 0x00, 0x6B, 0x41, 0x81, 0x70, 0x47],
            "{} record script accessors",
            case.name
        );
    }
}

#[test]
fn native_npc_script_override_zero_selects_default() {
    for (case, (offset, tables)) in CASES.iter().zip([
        (0x34EE0, 0xE6918),
        (0x352B4, 0xEEE58),
        (0x34C74, 0xE5D58),
        (0x35128, 0xEE968),
    ]) {
        let rom = fs::read(local_rom_path(case.rom)).unwrap();
        // The 36-table NPC block binds these routines at virtual +34/+38.
        // This does not classify every other actor subclass as an NPC.
        for index in 0..36 {
            let table = tables + index * 0x40;
            for (slot, routine) in [(0x34, offset), (0x38, offset + 24)] {
                assert_eq!(
                    u32::from_le_bytes(rom[table + slot..table + slot + 4].try_into().unwrap()),
                    0x08000001 + routine as u32,
                    "{} NPC table {index} slot {slot:#x}",
                    case.name
                );
            }
        }
        // Complete getter and setter: NPC+10 halfword override, zero falls
        // back to entity+42 halfword. Does not prove every entity uses it.
        assert_eq!(
            &rom[offset..offset + 30],
            &[
                0x00, 0xB5, 0x02, 0x1C, 0x11, 0x6B, 0x08, 0x8A, 0x00, 0x28, 0x02, 0xD1, 0x10, 0x1C,
                0x42, 0x30, 0x00, 0x88, 0x02, 0xBC, 0x08, 0x47, 0x00, 0x00, 0x00, 0x6B, 0x01, 0x82,
                0x70, 0x47,
            ],
            "{} NPC script override",
            case.name
        );
    }
}

#[test]
fn native_love_add_and_set_use_halfword_not_friendship_byte() {
    let reference = fs::read(local_rom_path(CASES[0].rom)).unwrap();
    for (case, dispatchers) in CASES.iter().zip([
        [0x41ED8, 0x41F2C],
        [0x42124, 0x42178],
        [0x41B4C, 0x41BA0],
        [0x41E80, 0x41ED4],
    ]) {
        let rom = fs::read(local_rom_path(case.rom)).unwrap();
        for (dispatcher, (base, length)) in
            dispatchers.into_iter().zip([(0x9E4C4, 36), (0x9E4F4, 4)])
        {
            let pc = dispatcher + 0x48;
            let hi = u16::from_le_bytes(rom[pc..pc + 2].try_into().unwrap());
            let lo = u16::from_le_bytes(rom[pc + 2..pc + 4].try_into().unwrap());
            assert_eq!(hi & 0xF800, 0xF000);
            assert_eq!(lo & 0xF800, 0xF800);
            let raw = (((hi & 0x7FF) as i32) << 12) | (((lo & 0x7FF) as i32) << 1);
            let target = (pc as i32 + 4 + ((raw << 9) >> 9)) as usize;
            // Complete helper (and 0xFFFF literal for add). Love is at +14
            // as a halfword; setting truncates, adding bounds its 32-bit sum.
            assert_eq!(
                &rom[target..target + length],
                &reference[base..base + length],
                "{} love helper",
                case.name
            );
        }
    }
}

#[test]
fn native_friendship_add_and_set_have_distinct_boundaries() {
    for (case, dispatchers) in CASES.iter().zip([
        [0x41C10, 0x41C64],
        [0x41E5C, 0x41EB0],
        [0x41884, 0x418D8],
        [0x41BB8, 0x41C0C],
    ]) {
        let rom = fs::read(local_rom_path(case.rom)).unwrap();
        for (dispatcher, expected) in dispatchers.into_iter().zip([
            &[
                0x00, 0xB5, 0x02, 0x1C, 0x10, 0x7A, 0x40, 0x18, 0x00, 0x28, 0x01, 0xDA, 0x00, 0x20,
                0x02, 0xE0, 0xFF, 0x28, 0x00, 0xD9, 0xFF, 0x20, 0x10, 0x72, 0x01, 0xBC, 0x00, 0x47,
            ][..],
            &[0x01, 0x72, 0x70, 0x47][..],
        ]) {
            let pc = dispatcher + 0x48;
            let hi = u16::from_le_bytes(rom[pc..pc + 2].try_into().unwrap());
            let lo = u16::from_le_bytes(rom[pc + 2..pc + 4].try_into().unwrap());
            assert_eq!(hi & 0xF800, 0xF000);
            assert_eq!(lo & 0xF800, 0xF800);
            let raw = (((hi & 0x7FF) as i32) << 12) | (((lo & 0x7FF) as i32) << 1);
            let target = (pc as i32 + 4 + ((raw << 9) >> 9)) as usize;
            // Full native helper: add clamps after 32-bit arithmetic, whereas
            // set stores only a byte. Neither changes the original VM operand.
            assert_eq!(
                &rom[target..target + expected.len()],
                expected,
                "{} friendship helper",
                case.name
            );
        }
    }
}

#[test]
fn native_npc_relationship_and_love_leaf_implementations_match_all_targets() {
    // These are the complete native leaf bodies used by the public relationship
    // callables, not only their dispatch-table positions or VM wrapper targets.
    // Exact equality proves that the bit fields, first-meeting branch, reset
    // behavior, byte/halfword widths, and clamping rules do not diverge among
    // the four vanilla targets. FoMT-US is only the byte template here; the
    // independently addressed leaf in every target must match it in full.
    let reference = fs::read(local_rom_path(CASES[0].rom)).unwrap();
    let reference_leaves = [
        (0x9E33Cusize, 4usize), // GetFriendship
        (0x9E340, 8),           // GetDaysSinceLastSpoken
        (0x9E348, 8),           // HasBeenGiftedToday
        (0x9E350, 8),           // HasBeenSpokenToToday
        (0x9E358, 8),           // HasBeenSpokenToJustNow
        (0x9E360, 8),           // HasBeenMet
        (0x9E370, 40),          // AddFriendship
        (0x9E398, 4),           // SetFriendship
        (0x9E39C, 48),          // SetSpokenTo
        (0x9E3CC, 20),          // SetGifted
        (0x9E498, 4),           // GetLove
        (0x9E4C4, 44),          // AddLove
        (0x9E4F4, 4),           // SetLove
    ];
    let target_leaves = [
        [
            0x9E33C, 0x9E340, 0x9E348, 0x9E350, 0x9E358, 0x9E360, 0x9E370, 0x9E398, 0x9E39C,
            0x9E3CC, 0x9E498, 0x9E4C4, 0x9E4F4,
        ],
        [
            0xA34D0, 0xA34D4, 0xA34DC, 0xA34E4, 0xA34EC, 0xA34F4, 0xA3504, 0xA352C, 0xA3530,
            0xA3560, 0xA362C, 0xA3658, 0xA3688,
        ],
        [
            0x9DD74, 0x9DD78, 0x9DD80, 0x9DD88, 0x9DD90, 0x9DD98, 0x9DDA8, 0x9DDD0, 0x9DDD4,
            0x9DE04, 0x9DED0, 0x9DEFC, 0x9DF2C,
        ],
        [
            0xA2F10, 0xA2F14, 0xA2F1C, 0xA2F24, 0xA2F2C, 0xA2F34, 0xA2F44, 0xA2F6C, 0xA2F70,
            0xA2FA0, 0xA306C, 0xA3098, 0xA30C8,
        ],
    ];

    for (case_index, case) in CASES.iter().enumerate() {
        let rom = fs::read(local_rom_path(case.rom)).unwrap();
        for (leaf_index, ((reference_offset, length), target_offset)) in reference_leaves
            .iter()
            .copied()
            .zip(target_leaves[case_index])
            .enumerate()
        {
            assert_eq!(
                &rom[target_offset..target_offset + length],
                &reference[reference_offset..reference_offset + length],
                "{} relationship/love native leaf {leaf_index}",
                case.name
            );
        }
    }
}

#[test]
fn native_link_reward_counter_reads_four_bits() {
    for (case, table, slot, handler, expected) in [
        (
            &CASES[0],
            0x45934,
            509,
            0x48A4C,
            [
                0xD4, 0x21, 0x89, 0, 0x50, 0x18, 0, 0x68, 2, 0x4A, 0x80, 0x18, 0, 0x78, 0x40, 6, 0,
                0x0F, 0xC6, 0xE2, 0xC3, 0x21, 0, 0,
            ],
        ),
        (
            &CASES[2],
            0x4575C,
            509,
            0x48874,
            [
                0xD4, 0x21, 0x89, 0, 0x50, 0x18, 0, 0x68, 2, 0x4A, 0x80, 0x18, 0, 0x78, 0x40, 6, 0,
                0x0F, 0xC6, 0xE2, 0xC3, 0x21, 0, 0,
            ],
        ),
        (
            &CASES[1],
            0x46050,
            595,
            0x49A48,
            [
                0xD4, 0x21, 0x89, 0, 0x50, 0x18, 0, 0x68, 2, 0x4A, 0x80, 0x18, 0, 0x78, 0, 9, 0,
                0xF0, 0xC8, 0xFC, 0xE7, 0x21, 0, 0,
            ],
        ),
        (
            &CASES[3],
            0x45DAC,
            595,
            0x497A4,
            [
                0xD4, 0x21, 0x89, 0, 0x50, 0x18, 0, 0x68, 2, 0x4A, 0x80, 0x18, 0, 0x78, 0, 9, 0,
                0xF0, 0xC8, 0xFC, 0xE7, 0x21, 0, 0,
            ],
        ),
    ] {
        let rom = fs::read(local_rom_path(case.rom)).unwrap();
        let entry = table + slot * 4;
        assert_eq!(
            u32::from_le_bytes(rom[entry..entry + 4].try_into().unwrap()),
            0x08000000 + handler as u32,
            "{} slot {slot}",
            case.name
        );
        // FoMT extracts byte bits 3..6; MFoMT reads the high nibble of a
        // different save byte. Neither read is boolean. Both regions were
        // traced through their own variable dispatch tables.
        assert_eq!(
            &rom[handler..handler + expected.len()],
            &expected,
            "{}",
            case.name
        );
    }
}

#[test]
fn native_link_reward_counter_producers_use_the_same_ten_recipe_transitions() {
    for (us_case, us, jp_case, jp, relocation_bytes) in [
        (
            &CASES[0],
            0x97800usize,
            &CASES[2],
            0x972BCusize,
            &[0x2Cusize, 0x2D, 0x90, 0xB4][..],
        ),
        (
            &CASES[1],
            0x9C728,
            &CASES[3],
            0x9C1F0,
            &[0x2Cusize, 0x2D, 0x88, 0xAC][..],
        ),
    ] {
        let us_rom = fs::read(local_rom_path(us_case.rom)).unwrap();
        let jp_rom = fs::read(local_rom_path(jp_case.rom)).unwrap();
        for offset in 0..0xDC {
            if !relocation_bytes.contains(&offset) {
                assert_eq!(
                    us_rom[us + offset],
                    jp_rom[jp + offset],
                    "{} / {} recipe producer byte +{offset:#x}",
                    us_case.name,
                    jp_case.name
                );
            }
        }
    }

    for (case, producer, failure_recorder, record_copy, recipe_writer, copy_bl, writer_bl) in [
        (&CASES[0], 0x97800, 0x9AA58, 0x9A970, 0x9A9C0, 0x8E, 0xB2),
        (&CASES[1], 0x9C728, 0x9F994, 0x9F890, 0x9F8E0, 0x86, 0xAA),
        (&CASES[2], 0x972BC, 0x9A490, 0x9A3A8, 0x9A3F8, 0x8E, 0xB2),
        (&CASES[3], 0x9C1F0, 0x9F3D4, 0x9F2D0, 0x9F320, 0x86, 0xAA),
    ] {
        let rom = fs::read(local_rom_path(case.rom)).unwrap();
        // Result selectors 0x9B through 0xA0 form the six first-time failed
        // dish transitions.  The unsigned subtract/range check is identical
        // in all four versions.
        assert_eq!(
            &rom[producer + 0x10..producer + 0x1A],
            &[0x08, 0x88, 0x9B, 0x38, 0x00, 0x04, 0x00, 0x0C, 0x05, 0x28],
            "{} failed-dish selector range",
            case.name
        );
        assert_eq!(thumb_call_target(&rom, producer + 0x2A), failure_recorder);
        assert_eq!(thumb_call_target(&rom, producer + copy_bl), record_copy);
        assert_eq!(thumb_call_target(&rom, producer + writer_bl), recipe_writer);

        // The successful first-record transitions are one-based recipe result
        // selectors 0x19, 0x50, 0x7E and 0x8D.  Their branch displacements may
        // differ between the FoMT and MFoMT storage layouts, but the compared
        // selector opcodes themselves are invariant.
        let first_compare = if copy_bl == 0x86 { 0xB2 } else { 0xBA };
        for (offset, expected) in [
            (first_compare, 0x2850u16),
            (first_compare + 4, 0x2850),
            (first_compare + 8, 0x2819),
            (first_compare + 0x16, 0x287E),
            (first_compare + 0x1A, 0x288D),
        ] {
            assert_eq!(
                u16::from_le_bytes(
                    rom[producer + offset..producer + offset + 2]
                        .try_into()
                        .unwrap()
                ),
                expected,
                "{} successful-recipe selector +{offset:#x}",
                case.name
            );
        }
    }
}

#[test]
fn native_mystic_berry_getter_matches_acquisition_flag() {
    for (case, table, handler, getter, vtable, action, setter) in [
        (
            &CASES[0], 0x45934, 0x463A8, 0xE53C, 0xE6658, 0x2634C, 0xEAF0,
        ),
        (
            &CASES[1], 0x46050, 0x46CF0, 0xE5C4, 0xEEB98, 0x264C4, 0xEB90,
        ),
        (
            &CASES[2], 0x4575C, 0x461D0, 0xE51C, 0xE5A98, 0x260E0, 0xEAD0,
        ),
        (
            &CASES[3], 0x45DAC, 0x46A4C, 0xE578, 0xEE6A8, 0x26338, 0xEB44,
        ),
    ] {
        let rom = fs::read(local_rom_path(case.rom)).unwrap();
        let word = |p| u32::from_le_bytes(rom[p..p + 4].try_into().unwrap());
        assert_eq!(word(table + 13 * 4), 0x08000000 + handler as u32);
        assert_eq!(word(vtable + 0x7C), 0x08000001 + action as u32);
        for (pc, target) in [(handler + 12, getter), (action + 8, setter)] {
            let hi = u16::from_le_bytes(rom[pc..pc + 2].try_into().unwrap());
            let lo = u16::from_le_bytes(rom[pc + 2..pc + 4].try_into().unwrap());
            assert_eq!(hi & 0xF800, 0xF000);
            assert_eq!(lo & 0xF800, 0xF800);
            let raw = ((hi as i32 & 0x7FF) << 12) | ((lo as i32 & 0x7FF) << 1);
            assert_eq!((pc as i32 + 4 + ((raw << 9) >> 9)) as usize, target);
        }
        assert_eq!(
            &rom[getter..getter + 10],
            &[0x44, 0x30, 0, 0x78, 0xC0, 6, 0xC0, 0x0F, 0x70, 0x47]
        );
        assert_eq!(
            &rom[setter..setter + 12],
            &[0x44, 0x30, 1, 0x78, 0x10, 0x22, 0x11, 0x43, 1, 0x70, 0x70, 0x47]
        );
        // The setter preserves every other bit; repeating it is idempotent.
        for byte in 0u32..=255 {
            let acquired = byte | 0x10;
            assert_eq!((acquired << 27) >> 31, 1);
            assert_eq!(acquired & !0x10, byte & !0x10);
            assert_eq!(acquired | 0x10, acquired);
        }
    }
}

#[test]
fn native_fatigue_varset_skips_storage_and_notification() {
    for (case, setter, tail, exit, mfomt) in [
        (&CASES[0], 0x48FFC, 0x4DA2E, 0x4E0E6, false),
        (&CASES[1], 0x4A3F8, 0x4FEEE, 0x506A2, true),
        (&CASES[2], 0x48E24, 0x4D856, 0x4DF0E, false),
        (&CASES[3], 0x4A154, 0x4FC4A, 0x503FE, true),
    ] {
        let rom = fs::read(local_rom_path(case.rom)).unwrap();
        assert_eq!(
            &rom[setter + 24..setter + 32],
            &[0x1C, 0x39, 5, 0x48, 0x81, 0x42, 1, 0xD9]
        );
        let limit = u32::from_le_bytes(rom[setter + 48..setter + 52].try_into().unwrap());
        assert_eq!(limit, if mfomt { 699 } else { 561 });
        assert!(12u32.wrapping_sub(28) > limit);
        let pc = setter + 32;
        let hi = u16::from_le_bytes(rom[pc..pc + 2].try_into().unwrap());
        let lo = u16::from_le_bytes(rom[pc + 2..pc + 4].try_into().unwrap());
        assert_eq!(hi & 0xF800, 0xF000);
        assert_eq!(lo & 0xF800, 0xF800);
        let raw = ((hi as i32 & 0x7FF) << 12) | ((lo as i32 & 0x7FF) << 1);
        assert_eq!((pc as i32 + 4 + ((raw << 9) >> 9)) as usize, tail);
        let branch_offset = if mfomt {
            assert_eq!(
                &rom[tail + 10..tail + 22],
                &[0x41, 0x46, 0x48, 0x39, 0xE6, 0x20, 0x40, 0, 0x81, 0x42, 0, 0xD9]
            );
            assert!(12u32.wrapping_sub(72) > 460);
            22
        } else {
            assert_eq!(
                &rom[tail + 10..tail + 20],
                &[0x41, 0x46, 0x46, 0x39, 5, 0x48, 0x81, 0x42, 0, 0xD9]
            );
            assert_eq!(
                u32::from_le_bytes(rom[tail + 38..tail + 42].try_into().unwrap()),
                395
            );
            assert!(12u32.wrapping_sub(70) > 395);
            20
        };
        let pc = tail + branch_offset;
        let branch = u16::from_le_bytes(rom[pc..pc + 2].try_into().unwrap());
        assert_eq!(branch & 0xF800, 0xE000);
        let displacement = (((branch as i32 & 0x7FF) << 21) >> 20) as isize;
        assert_eq!((pc as isize + 4 + displacement) as usize, exit);
        assert_eq!(
            &rom[exit..exit + 14],
            &[2, 0xB0, 0x18, 0xBC, 0x98, 0x46, 0xA1, 0x46, 0xF0, 0xBC, 1, 0xBC, 0, 0x47]
        );
    }
}

#[test]
fn native_fatigue_variable_returns_half_the_stored_value() {
    for (case, table, handler, getter) in [
        (&CASES[0], 0x45934, 0x46390, 0xE4FC),
        (&CASES[1], 0x46050, 0x46CD8, 0xE584),
        (&CASES[2], 0x4575C, 0x461B8, 0xE4DC),
        (&CASES[3], 0x45DAC, 0x46A34, 0xE538),
    ] {
        let rom = fs::read(local_rom_path(case.rom)).unwrap();
        let entry = table + 12 * 4;
        assert_eq!(
            u32::from_le_bytes(rom[entry..entry + 4].try_into().unwrap()),
            0x08000000 + handler as u32,
            "{} fatigue slot",
            case.name
        );
        let pc = handler + 12;
        let hi = u16::from_le_bytes(rom[pc..pc + 2].try_into().unwrap());
        let lo = u16::from_le_bytes(rom[pc + 2..pc + 4].try_into().unwrap());
        assert_eq!(hi & 0xF800, 0xF000);
        assert_eq!(lo & 0xF800, 0xF800);
        let raw = ((hi as i32 & 0x7FF) << 12) | ((lo as i32 & 0x7FF) << 1);
        assert_eq!((pc as i32 + 4 + ((raw << 9) >> 9)) as usize, getter);
        // LDR Farmer+0x44; LSL9; LSR25 drops the stored field's low bit.
        assert_eq!(
            &rom[getter..getter + 8],
            &[0x40, 0x6C, 0x40, 2, 0x40, 0x0E, 0x70, 0x47]
        );
        // Exhaust the eight-bit physical field, including corrupted >200
        // values. The getter truncates division and does not clamp to100.
        for stored in 0u32..=255 {
            let packed = (stored << 15) | 0xFF80007F;
            assert_eq!(packed.wrapping_shl(9) >> 25, stored / 2);
        }
    }
}

#[test]
fn native_scripted_control_toggles_only_the_shared_mode_byte() {
    for (case, table, slot, handlers, natives, mfomt) in [
        (
            &CASES[0],
            0x3F904,
            0xEF,
            [0x43618, 0x43630],
            [0x142F0, 0x14304],
            false,
        ),
        (
            &CASES[1],
            0x3FAF0,
            0xF2,
            [0x43874, 0x4388C],
            [0x1447C, 0x14490],
            true,
        ),
        (
            &CASES[2],
            0x3F578,
            0xEF,
            [0x4328C, 0x432A4],
            [0x141C4, 0x141D8],
            false,
        ),
        (
            &CASES[3],
            0x3F84C,
            0xF2,
            [0x435D0, 0x435E8],
            [0x14324, 0x14338],
            true,
        ),
    ] {
        let rom = fs::read(local_rom_path(case.rom)).unwrap();
        for (index, (handler, native)) in handlers.into_iter().zip(natives).enumerate() {
            let entry = table + (slot + index) * 4;
            assert_eq!(
                u32::from_le_bytes(rom[entry..entry + 4].try_into().unwrap()),
                0x08000000 + handler as u32
            );
            let pc = handler + 16;
            let hi = u16::from_le_bytes(rom[pc..pc + 2].try_into().unwrap());
            let lo = u16::from_le_bytes(rom[pc + 2..pc + 4].try_into().unwrap());
            assert_eq!(hi & 0xF800, 0xF000);
            assert_eq!(lo & 0xF800, 0xF800);
            let raw = ((hi as i32 & 0x7FF) << 12) | ((lo as i32 & 0x7FF) << 1);
            assert_eq!((pc as i32 + 4 + ((raw << 9) >> 9)) as usize, native);
            let value = (1 - index) as u8;
            let expected = if mfomt {
                vec![
                    0x40, 0x68, 0x8C, 0x30, 0, 0x68, 0xD4, 0x21, 0x89, 1, 0x40, 0x18, value, 0x21,
                    1, 0x70, 0x70, 0x47, 0, 0,
                ]
            } else {
                vec![
                    0x40, 0x68, 0x8C, 0x30, 0, 0x68, 2, 0x49, 0x40, 0x18, value, 0x21, 1, 0x70,
                    0x70, 0x47, 0xC4, 0x34, 0, 0,
                ]
            };
            assert_eq!(
                &rom[native..native + expected.len()],
                expected,
                "{} toggle {index}",
                case.name
            );
        }
    }
}

#[test]
fn native_power_berry_counter_chain_matches_all_targets() {
    let reference = fs::read(local_rom_path(CASES[0].rom)).unwrap();
    for (case, vtable, action, counter, stamina, maximum) in [
        (&CASES[0], 0xE6658, 0x26830, 0xEAFC, 0xE9E4, 0xE51C),
        (&CASES[1], 0xEEB98, 0x269B0, 0xEB9C, 0xEA84, 0xE5A4),
        (&CASES[2], 0xE5A98, 0x265C4, 0xEADC, 0xE9C4, 0xE4FC),
        (&CASES[3], 0xEE6A8, 0x26824, 0xEB50, 0xEA38, 0xE558),
    ] {
        let rom = fs::read(local_rom_path(case.rom)).unwrap();
        assert_eq!(
            u32::from_le_bytes(rom[vtable + 0x80..vtable + 0x84].try_into().unwrap()),
            0x08000001 + action as u32,
            "{} acquisition virtual method",
            case.name
        );
        assert_eq!(
            &rom[action..action + 8],
            &[0xF0, 0xB5, 0x82, 0xB0, 6, 0x1C, 0xB0, 0x6B]
        );
        for (pc, target) in [
            (action + 8, counter),
            (counter + 38, stamina),
            (stamina + 26, maximum),
        ] {
            let hi = u16::from_le_bytes(rom[pc..pc + 2].try_into().unwrap());
            let lo = u16::from_le_bytes(rom[pc + 2..pc + 4].try_into().unwrap());
            assert_eq!(hi & 0xF800, 0xF000);
            assert_eq!(lo & 0xF800, 0xF800);
            let raw = ((hi as i32 & 0x7FF) << 12) | ((lo as i32 & 0x7FF) << 1);
            assert_eq!((pc as i32 + 4 + ((raw << 9) >> 9)) as usize, target);
        }
        // Counts >=10 exit before both the increment and stamina call.
        assert_eq!(
            &rom[counter..counter + 48],
            &[
                0x10, 0xB5, 4, 0x1C, 0x23, 0x1C, 0x44, 0x33, 0x1A, 0x78, 0x10, 7, 0, 0x0F, 9, 0x28,
                0x0B, 0xD8, 0x41, 0x1C, 0x0F, 0x20, 1, 0x40, 0x10, 0x20, 0x40, 0x42, 0x10, 0x40, 8,
                0x43, 0x18, 0x70, 0x20, 0x1C, 0x0A, 0x21, 0xFF, 0xF7, 0x5F, 0xFF, 0x10, 0xBC, 1,
                0xBC, 0, 0x47
            ]
        );
        // Maximum stamina = 150 + 10 * the low-nibble berry count.
        assert_eq!(
            &rom[maximum..maximum + 18],
            &[
                0x44, 0x30, 1, 0x78, 9, 7, 9, 0x0F, 0x88, 0, 0x40, 0x18, 0x40, 0, 0x96, 0x30, 0x70,
                0x47
            ]
        );
        // Complete stamina adjustment helper. The only target-dependent bytes
        // are the already decoded BL to GetMaximumStamina at +0x1A.
        for offset in 0..84 {
            if (0x1A..0x1E).contains(&offset) {
                continue;
            }
            assert_eq!(
                rom[stamina + offset],
                reference[0xE9E4 + offset],
                "{} stamina adjustment+{offset:X}",
                case.name
            );
        }
    }
}

#[test]
fn native_tv_shopping_order_and_countdown_match_all_targets() {
    for (case, table, slot, handler, confirm, update, caller) in [
        (&CASES[0], 0x3F904, 0xE1, 0x42AD4, 0x11544, 0x11568, 0x111B6),
        (&CASES[1], 0x3FAF0, 0xE4, 0x42D00, 0x11624, 0x11648, 0x11298),
        (&CASES[2], 0x3F578, 0xE1, 0x42748, 0x11524, 0x11548, 0x11196),
        (&CASES[3], 0x3F84C, 0xE4, 0x42A5C, 0x115D8, 0x115FC, 0x1124C),
    ] {
        let rom = fs::read(local_rom_path(case.rom)).unwrap();
        let entry = table + slot * 4;
        assert_eq!(
            u32::from_le_bytes(rom[entry..entry + 4].try_into().unwrap()),
            0x08000000 + handler as u32,
            "{} confirmation slot",
            case.name
        );
        // Completion immediately follows confirmation in each physical table.
        let completion = handler + 0x18;
        let next_entry = entry + 4;
        assert_eq!(
            u32::from_le_bytes(rom[next_entry..next_entry + 4].try_into().unwrap()),
            0x08000000 + completion as u32
        );
        for (pc, target) in [
            (handler + 12, confirm),
            (caller, update),
            (completion + 0x19E, confirm + 12),
        ] {
            let hi = u16::from_le_bytes(rom[pc..pc + 2].try_into().unwrap());
            let lo = u16::from_le_bytes(rom[pc + 2..pc + 4].try_into().unwrap());
            assert_eq!(hi & 0xF800, 0xF000);
            assert_eq!(lo & 0xF800, 0xF800);
            let raw = ((hi as i32 & 0x7FF) << 12) | ((lo as i32 & 0x7FF) << 1);
            assert_eq!(
                (pc as i32 + 4 + ((raw << 9) >> 9)) as usize,
                target,
                "{} BL",
                case.name
            );
        }
        // Unconditional overwrite; no payment, validity or existing-order check.
        assert_eq!(
            &rom[confirm..confirm + 10],
            &[1, 0x68, 0x41, 0x60, 2, 0x21, 1, 0x72, 0x70, 0x47]
        );
        // Cleanup preserves a different current selection, always clears the
        // pending item, and does not write the countdown byte.
        assert_eq!(
            &rom[confirm + 12..confirm + 36],
            &[
                0, 0xB5, 2, 0x1C, 0x51, 0x68, 0x10, 0x68, 0x81, 0x42, 1, 0xD1, 0x10, 0x20, 0x10,
                0x60, 0x10, 0x20, 0x50, 0x60, 1, 0xBC, 0, 0x47
            ]
        );
        // Both selection and pending order must differ from NONE (16).
        // A nonzero byte counter decreases by one without underflow.
        assert_eq!(
            &rom[update..update + 30],
            &[
                0, 0xB5, 1, 0x1C, 8, 0x68, 0x10, 0x28, 7, 0xD0, 0x48, 0x68, 0x10, 0x28, 4, 0xD0, 8,
                0x7A, 0, 0x28, 1, 0xD0, 1, 0x38, 8, 0x72, 1, 0xBC, 0, 0x47
            ],
            "{} counter",
            case.name
        );
    }
}

#[test]
fn native_tv_shopping_selection_state_matches_all_targets() {
    let reference = fs::read(local_rom_path(CASES[0].rom)).unwrap();
    for (case, (table, first_slot, handlers, state_base)) in CASES.iter().zip([
        (0x3F904, 0xDD, [0x42A34, 0x42A58, 0x42A7C, 0x42AA4], 0x1151C),
        (0x3FAF0, 0xE0, [0x42C78, 0x42C94, 0x42CB8, 0x42CD0], 0x115FC),
        (0x3F578, 0xDD, [0x426A8, 0x426CC, 0x426F0, 0x42718], 0x114FC),
        (0x3F84C, 0xE0, [0x429D4, 0x429F0, 0x42A14, 0x42A2C], 0x115B0),
    ]) {
        let rom = fs::read(local_rom_path(case.rom)).unwrap();
        for (index, handler) in handlers.into_iter().enumerate() {
            let slot = first_slot + index;
            assert_eq!(
                u32::from_le_bytes(
                    rom[table + slot * 4..table + slot * 4 + 4]
                        .try_into()
                        .unwrap(),
                ),
                0x0800_0000 + handler as u32,
                "{} TV-shopping state slot {slot:#x}",
                case.name
            );
        }
        for (index, (handler, target)) in handlers
            .into_iter()
            .zip([
                state_base,
                state_base + 4,
                state_base + 8,
                state_base + 0x24,
            ])
            .enumerate()
        {
            let offset = if index == 3 { 0x22 } else { 0x0C };
            assert_eq!(
                thumb_call_target(&rom, handler + offset),
                target,
                "{} TV-shopping state leaf {index}",
                case.name
            );
        }

        // Getter, pending getter, ready predicate, raw selection setter,
        // confirmation, and cleanup form an identical state-machine block in
        // every target. This is stronger than matching public slot order.
        assert_eq!(
            &rom[state_base..state_base + 0x4C],
            &reference[0x1151C..0x1151C + 0x4C],
            "{} TV-shopping state implementation",
            case.name
        );
    }
}

#[test]
fn native_mfomt_unknown_stocking_gate_is_one_bit_in_both_regions() {
    for (case, get_table, getter, helper, set_table, setter) in [
        (&CASES[1], 0x46050, 0x48218, 0x4A3CA, 0x4A430, 0x4CAB0),
        (&CASES[3], 0x45DAC, 0x47F74, 0x4A126, 0x4A18C, 0x4C80C),
    ] {
        let rom = fs::read(local_rom_path(case.rom)).unwrap();
        for (table, index, handler) in [(get_table, 283, getter), (set_table, 283 - 28, setter)] {
            let entry = table + index * 4;
            assert_eq!(
                u32::from_le_bytes(rom[entry..entry + 4].try_into().unwrap()),
                0x08000000 + handler as u32,
                "{} slot 283",
                case.name
            );
        }
        // The same relative BL reaches the one-bit reader in both regions.
        assert_eq!(
            &rom[getter..getter + 20],
            &[
                0xD4, 0x21, 0x89, 0, 0x50, 0x18, 0, 0x68, 1, 0x4A, 2, 0xF0, 0xD2, 0xF8, 0, 0, 0x95,
                0x21, 0, 0
            ]
        );
        assert_eq!(getter + 10 + 4 + 0x2000 + 0x1A4, helper);
        // LSL 26 / LSR 31 extracts bit 5, not a two-bit lifecycle value.
        assert_eq!(
            &rom[helper..helper + 8],
            &[0x80, 0x18, 0, 0x78, 0x80, 6, 0xC0, 0x0F]
        );
        assert_eq!(
            &rom[setter + 12..setter + 24],
            &[1, 0x20, 6, 0x40, 0x73, 1, 0x0A, 0x78, 0x21, 0x20, 0x40, 0x42]
        );
        assert_eq!(&rom[setter + 28..setter + 32], &[0x95, 0x21, 0, 0]);
    }
}

#[test]
fn native_tv_shopping_furniture_branches_preserve_installation_guards() {
    for (case, table, slot, handler, targets) in [
        (
            &CASES[0],
            0x3F904,
            0xE2,
            0x42AEC,
            [0xC118, 0xC134, 0xC27C, 0xC178, 0xC15C],
        ),
        (
            &CASES[1],
            0x3FAF0,
            0xE5,
            0x42D18,
            [0xC18C, 0xC1A8, 0xC2F0, 0xC1EC, 0xC1D0],
        ),
        (
            &CASES[2],
            0x3F578,
            0xE2,
            0x42760,
            [0xC0F8, 0xC114, 0xC25C, 0xC158, 0xC13C],
        ),
        (
            &CASES[3],
            0x3F84C,
            0xE5,
            0x42A74,
            [0xC140, 0xC15C, 0xC2A4, 0xC1A0, 0xC184],
        ),
    ] {
        let rom = fs::read(local_rom_path(case.rom)).unwrap();
        let read_word = |offset| u32::from_le_bytes(rom[offset..offset + 4].try_into().unwrap());
        assert_eq!(read_word(table + slot * 4), 0x08000000 + handler as u32);
        let bl_target = |pc: usize| {
            let hi = u16::from_le_bytes(rom[pc..pc + 2].try_into().unwrap());
            let lo = u16::from_le_bytes(rom[pc + 2..pc + 4].try_into().unwrap());
            assert_eq!(hi & 0xF800, 0xF000);
            assert_eq!(lo & 0xF800, 0xF800);
            let raw = ((hi as i32 & 0x7FF) << 12) | ((lo as i32 & 0x7FF) << 1);
            (pc as i32 + 4 + ((raw << 9) >> 9)) as usize
        };
        for (id, target) in (2..=6).zip(targets) {
            let branch = (read_word(handler + 0x28 + id * 4) - 0x08000000) as usize;
            assert_eq!(bl_target(branch + 14), target, "{} item {id}", case.name);
            // Every furniture path jumps to cleanup regardless of installer success.
            let jump = u16::from_le_bytes(rom[branch + 18..branch + 20].try_into().unwrap());
            assert_eq!(jump & 0xF800, 0xE000);
            assert_eq!(branch + 22 + ((jump & 0x7FF) as usize * 2), handler + 0x192);
            if id == 4 {
                // Nonzero upgrade level and a nonnull refrigerator are both required.
                assert_eq!(
                    &rom[target..target + 38],
                    &[
                        0x10, 0xB5, 4, 0x1C, 0x21, 0x78, 3, 0x20, 8, 0x40, 0, 0x28, 8, 0xD0, 0x20,
                        0x1C, 0xFF, 0xF7, 0xDC, 0xFE, 0, 0x28, 3, 0xD0, 0x20, 0x79, 0x80, 0x21, 8,
                        0x43, 0x20, 0x71, 0x10, 0xBC, 1, 0xBC, 0, 0x47
                    ]
                );
            } else {
                let mask = match id {
                    2 => 2,
                    3 => 4,
                    5 => 0x20,
                    6 => 0x10,
                    _ => unreachable!(),
                };
                let mut expected = vec![
                    0, 0xB5, 2, 0x1C, 0x11, 0x78, 3, 0x20, 8, 0x40, 0, 0x28, 3, 0xD0, 0x50, 0x78,
                    mask, 0x21, 8, 0x43, 0x50, 0x70, 1, 0xBC, 0, 0x47,
                ];
                if id == 6 {
                    // Large bed: upgrade level > 1, not merely nonzero.
                    expected[4..14]
                        .copy_from_slice(&[0x10, 0x78, 0x80, 7, 0x80, 0x0F, 1, 0x28, 3, 0xD9]);
                }
                assert_eq!(
                    &rom[target..target + expected.len()],
                    expected,
                    "{} item {id}",
                    case.name
                );
            }
        }
    }
}

#[test]
fn native_horse_creation_stores_ten_bit_age_on_all_targets() {
    for (index, (case, create, horse, pet, animal)) in [
        (&CASES[0], 0x14C34, 0x9BBF0, 0x9B350, 0x9B1A4),
        (&CASES[1], 0x14DC0, 0xA0B5C, 0xA02BC, 0xA0110),
        (&CASES[2], 0x14B08, 0x9B628, 0x9AD88, 0x9ABDC),
        (&CASES[3], 0x14C68, 0xA059C, 0x9FCFC, 0x9FB50),
    ]
    .into_iter()
    .enumerate()
    {
        let rom = fs::read(local_rom_path(case.rom)).unwrap();
        let bl_target = |pc: usize| {
            let hi = u16::from_le_bytes(rom[pc..pc + 2].try_into().unwrap());
            let lo = u16::from_le_bytes(rom[pc + 2..pc + 4].try_into().unwrap());
            assert_eq!(hi & 0xF800, 0xF000, "{} BL prefix", case.name);
            assert_eq!(lo & 0xF800, 0xF800, "{} BL suffix", case.name);
            let displacement = (((hi as i32 & 0x7FF) << 12) | ((lo as i32 & 0x7FF) << 1)) << 9 >> 9;
            (pc as i32 + 4 + displacement) as usize
        };
        let (table, slot, create_handler, call_offset, remove_handler, remove, clear) = [
            (0x3F904, 0x100, 0x44336, 0x82, 0x443C0, 0x14D30, 0x9C54),
            (0x3FAF0, 0x103, 0x445A6, 0x80, 0x4462E, 0x14EBC, 0x9CC0),
            (0x3F578, 0x100, 0x43FAA, 0x82, 0x44034, 0x14C04, 0x9C5C),
            (0x3F84C, 0x103, 0x44302, 0x80, 0x4438A, 0x14D64, 0x9C74),
        ][index];
        for (id, handler) in [(slot, create_handler), (slot + 1, remove_handler)] {
            let entry = table + id * 4;
            assert_eq!(
                u32::from_le_bytes(rom[entry..entry + 4].try_into().unwrap()),
                0x08000000 + handler as u32,
                "{} callable {id:#x}",
                case.name
            );
        }
        assert_eq!(bl_target(create_handler + call_offset), create);
        assert_eq!(bl_target(remove_handler + 0x3E), remove);
        // Creation saves r1 as guard in r7; BNE bypasses creation when nonzero.
        assert_eq!(
            &rom[create + 12..create + 24],
            &[0x0F, 0x1C, 0x0F, 0x92, 0x40, 0x68, 0x10, 0x90, 0, 0x2F, 0x65, 0xD1]
        );
        // Removal also exits on a nonzero guard. The incoming unused r2
        // is overwritten with virtual method +0x3C before it can be used.
        assert_eq!(
            &rom[remove..remove + 20],
            &[
                0x10, 0xB5, 0x44, 0x68, 0, 0x29, 0x0D, 0xD1, 0x20, 0x1C, 0xA8, 0x30, 0, 0x68, 1,
                0x68, 0xCA, 0x6B, 0x2C, 0x21
            ]
        );
        assert_eq!(bl_target(remove + 0x20), clear);
        // Farm::RemoveHorse clears only the registration bit, not the record.
        assert_eq!(
            &rom[clear..clear + 12],
            &[0x42, 0x7C, 5, 0x21, 0x49, 0x42, 0x11, 0x40, 0x41, 0x74, 0x70, 0x47]
        );
        assert_eq!(bl_target(create + 0x9C), horse, "{} Horse", case.name);
        assert_eq!(bl_target(horse + 4), pet, "{} Pet", case.name);
        assert_eq!(bl_target(pet + 4), animal, "{} Animal", case.name);
        // r2 = ((stage << 4) - stage) << 3, followed by constructor arguments.
        assert_eq!(
            &rom[create + 0x90..create + 0x9C],
            &[0x0F, 0x9B, 0x1A, 0x01, 0xD2, 0x1A, 0xD2, 0, 0x68, 0x46, 0x21, 0x1C],
            "{} stage multiplication",
            case.name
        );
        // Preserve incoming r2 in r5; mask age to 0x3FF, preserve unrelated
        // bits using 0xFFFFFC00, and write the halfword at Animal + 0x18.
        assert_eq!(&rom[animal..animal + 6], &[0x30, 0xB5, 4, 0x1C, 0x15, 0x1C]);
        assert_eq!(
            &rom[animal + 0x0E..animal + 0x1E],
            &[
                0x0D, 0x49, 8, 0x1C, 5, 0x40, 0x21, 0x8B, 0x0C, 0x48, 8, 0x40, 0x28, 0x43, 0x20,
                0x83
            ],
            "{} age field store",
            case.name
        );
        assert_eq!(
            &rom[animal + 0x44..animal + 0x4C],
            &[0xFF, 3, 0, 0, 0, 0xFC, 0xFF, 0xFF],
            "{} age masks",
            case.name
        );
    }
}

#[test]
fn native_link_reward_counter_writes_truncated_nibbles() {
    for (case, table, slot, handler, tail, branch_offset) in [
        (&CASES[0], 0x49034, 509, 0x4D0BC, 0x4DA28, 24),
        (&CASES[1], 0x4A430, 595, 0x4EF58, 0x4FEE8, 18),
        (&CASES[2], 0x48E5C, 509, 0x4CEE4, 0x4D850, 24),
        (&CASES[3], 0x4A18C, 595, 0x4ECB4, 0x4FC44, 18),
    ] {
        let rom = fs::read(local_rom_path(case.rom)).unwrap();
        // VarSet's native jump table starts at variable 28, unlike VarGet.
        let entry = table + (slot - 28) * 4;
        assert_eq!(
            u32::from_le_bytes(rom[entry..entry + 4].try_into().unwrap()),
            0x08000000 + handler as u32,
            "{}",
            case.name
        );
        if slot == 509 {
            // input & 15, then shift 3; old byte is masked with 0x87.
            assert_eq!(
                &rom[handler + 12..handler + 24],
                &[0x0F, 0x20, 6, 0x40, 0xF3, 0, 0x0A, 0x78, 0x79, 0x20, 0x40, 0x42]
            );
        } else {
            // Shift input 4; preserve old low nibble. STRB discards upper bits.
            assert_eq!(
                &rom[handler + 12..handler + 18],
                &[0x33, 1, 0x0A, 0x78, 0x0F, 0x20]
            );
        }
        let pc = handler + branch_offset;
        let hi = u16::from_le_bytes(rom[pc..pc + 2].try_into().unwrap());
        let lo = u16::from_le_bytes(rom[pc + 2..pc + 4].try_into().unwrap());
        assert_eq!(hi & 0xF800, 0xF000);
        assert_eq!(lo & 0xF800, 0xF800);
        let raw = (((hi & 0x7FF) as i32) << 12) | (((lo & 0x7FF) as i32) << 1);
        assert_eq!((pc as i32 + 4 + ((raw << 9) >> 9)) as usize, tail);
        // AND old byte with mask; OR shifted input; STRB to selected field.
        assert_eq!(&rom[tail..tail + 6], &[0x10, 0x40, 0x18, 0x43, 8, 0x70]);
    }
}

#[test]
fn native_mfomt_saved_billion_variable_reads_account_flag() {
    for (case, table, handler) in [(&CASES[1], 0x46050, 0x47210), (&CASES[3], 0x45DAC, 0x46F6C)] {
        let rom = fs::read(local_rom_path(case.rom)).unwrap();
        let entry = table + 691 * 4;
        assert_eq!(
            u32::from_le_bytes(rom[entry..entry + 4].try_into().unwrap()),
            0x08000000 + handler as u32,
            "{} variable 691",
            case.name
        );
        assert_eq!(
            &rom[handler..handler + 14],
            &[0xD4, 0x21, 0x89, 0x00, 0x50, 0x18, 0x00, 0x68, 0x02, 0x4A, 0x80, 0x18, 0x00, 0x79]
        );
        // Account offset literal; ldrb above selects its flags at +4.
        assert_eq!(&rom[handler + 20..handler + 24], &[0xB8, 0x1A, 0, 0]);
        let pc = handler + 14;
        let hi = u16::from_le_bytes(rom[pc..pc + 2].try_into().unwrap());
        let lo = u16::from_le_bytes(rom[pc + 2..pc + 4].try_into().unwrap());
        assert_eq!(hi & 0xF800, 0xF000);
        assert_eq!(lo & 0xF800, 0xF800);
        let raw = (((hi & 0x7FF) as i32) << 12) | (((lo & 0x7FF) as i32) << 1);
        let target = (pc as i32 + 4 + ((raw << 9) >> 9)) as usize;
        // lsl 29; lsr 31 extracts bit 2 as a boolean, not the money balance.
        assert_eq!(&rom[target..target + 4], &[0x40, 0x07, 0xC0, 0x0F]);
    }
}

#[test]
fn native_mfomt_recipe_completion_latch_matches_variable_690() {
    for (case, table, handler, writer, counter, setter, getter) in [
        (
            &CASES[1], 0x46050, 0x471F4, 0x9F8E0, 0x9F964, 0x9FAC8, 0x9FAD8,
        ),
        (
            &CASES[3], 0x45DAC, 0x46F50, 0x9F320, 0x9F3A4, 0x9F508, 0x9F518,
        ),
    ] {
        let rom = fs::read(local_rom_path(case.rom)).unwrap();
        let entry = table + 690 * 4;
        assert_eq!(
            u32::from_le_bytes(rom[entry..entry + 4].try_into().unwrap()),
            0x08000000 + handler as u32
        );
        // Variable 690 first selects the save subobject at +0x2250; the
        // completion getter then reads +0xA06 within that object. It must not
        // be conflated with direct save bytes +0x21E5/+0x21E6 used by the
        // unrelated unknown variable slots 584 through 591.
        assert_eq!(
            u32::from_le_bytes(rom[handler + 24..handler + 28].try_into().unwrap()),
            0x2250,
            "{} completion subobject base",
            case.name
        );
        assert_eq!(0x2250 + 0xA06, 0x2C56);
        assert_ne!(0x2250 + 0xA06, 0x21E5);
        assert_ne!(0x2250 + 0xA06, 0x21E6);
        for (pc, target) in [
            (handler + 12, getter),
            (writer + 0x4C, counter),
            (writer + 0x5C, setter),
        ] {
            let hi = u16::from_le_bytes(rom[pc..pc + 2].try_into().unwrap());
            let lo = u16::from_le_bytes(rom[pc + 2..pc + 4].try_into().unwrap());
            assert_eq!(hi & 0xF800, 0xF000);
            assert_eq!(lo & 0xF800, 0xF800);
            let raw = (((hi & 0x7FF) as i32) << 12) | (((lo & 0x7FF) as i32) << 1);
            assert_eq!(
                (pc as i32 + 4 + ((raw << 9) >> 9)) as usize,
                target,
                "{}",
                case.name
            );
        }
        // Count at +A07; completion latch at +A06. These are distinct fields.
        assert_eq!(&rom[writer + 0x68..writer + 0x6C], &[7, 10, 0, 0]);
        assert_eq!(&rom[counter + 0x2C..counter + 0x30], &[7, 10, 0, 0]);
        assert_eq!(
            &rom[getter..getter + 12],
            &[1, 0x49, 0x40, 0x18, 0, 0x78, 0x70, 0x47, 6, 10, 0, 0]
        );
        assert_eq!(
            &rom[setter..setter + 16],
            &[2, 0x49, 0x40, 0x18, 1, 0x21, 1, 0x70, 0x70, 0x47, 0, 0, 6, 10, 0, 0]
        );
    }
}

#[test]
fn native_known_recipe_count_reads_each_learned_record_on_all_targets() {
    const COUNT_BODY: &[u8] = &[
        0x00, 0xB5, 0x01, 0x1C, 0x00, 0x23, 0x09, 0x4A, 0x88, 0x18, 0x00, 0x78, 0x83, 0x42, 0x0A,
        0xDA, 0x0A, 0x1C, 0x01, 0x1C, 0xD0, 0x7C, 0xC0, 0x06, 0x00, 0x28, 0x00, 0xDA, 0x01, 0x33,
        0x14, 0x32, 0x01, 0x39, 0x00, 0x29, 0xF6, 0xD1, 0x18, 0x1C, 0x02, 0xBC, 0x08, 0x47,
    ];
    for (case, dispatch, slot, handler, leaf, count_offset) in [
        (&CASES[0], 0x3F904, 0x133, 0x45496, 0x9AA28, 0x0A06u32),
        (&CASES[1], 0x3FAF0, 0x137, 0x45768, 0x9F964, 0x0A07),
        (&CASES[2], 0x3F578, 0x133, 0x4510A, 0x9A460, 0x0A06),
        (&CASES[3], 0x3F84C, 0x137, 0x454C4, 0x9F3A4, 0x0A07),
    ] {
        let rom = fs::read(local_rom_path(case.rom)).unwrap();
        assert_eq!(
            u32::from_le_bytes(
                rom[dispatch + slot * 4..dispatch + slot * 4 + 4]
                    .try_into()
                    .unwrap()
            ),
            0x0800_0000 + handler as u32,
            "{} known-recipe-count physical slot",
            case.name
        );
        assert_eq!(thumb_call_target(&rom, handler + 0x0C), leaf);
        assert_eq!(
            &rom[leaf..leaf + COUNT_BODY.len()],
            COUNT_BODY,
            "{}",
            case.name
        );
        assert_eq!(
            u32::from_le_bytes(
                rom[leaf + COUNT_BODY.len()..leaf + COUNT_BODY.len() + 4]
                    .try_into()
                    .unwrap()
            ),
            count_offset,
            "{} recipe-record count offset",
            case.name
        );
    }
}

#[test]
fn native_money_getter_reads_the_family_specific_saved_u32_directly() {
    for (case, (dispatch, slot, handler, save_offset, literal_offset)) in CASES.iter().zip([
        (0x3F904, 0xEC, 0x42DB2, 0x1AA8u32, 0x1E),
        (0x3FAF0, 0xEF, 0x42FDE, 0x1AB8u32, 0x12),
        (0x3F578, 0xEC, 0x42A26, 0x1AA8u32, 0x1E),
        (0x3F84C, 0xEF, 0x42D3A, 0x1AB8u32, 0x12),
    ]) {
        let rom = fs::read(local_rom_path(case.rom)).unwrap();
        assert_eq!(
            u32::from_le_bytes(
                rom[dispatch + slot * 4..dispatch + slot * 4 + 4]
                    .try_into()
                    .unwrap()
            ),
            0x0800_0000 + handler as u32,
            "{} GetMoney physical slot",
            case.name
        );
        assert_eq!(
            &rom[handler..handler + 8],
            &[0xD4, 0x24, 0xA4, 0x00, 0x28, 0x19, 0x00, 0x68],
            "{} GetMoney loads the save-state owner",
            case.name
        );
        assert_eq!(
            u32::from_le_bytes(
                rom[handler + literal_offset..handler + literal_offset + 4]
                    .try_into()
                    .unwrap()
            ),
            save_offset,
            "{} GetMoney saved balance offset",
            case.name
        );
        assert_eq!(
            &rom[handler + 0x0C..handler + 0x0E],
            &[0x04, 0x68],
            "{} GetMoney performs a direct 32-bit load",
            case.name
        );
    }
}

#[test]
fn native_money_addition_preserves_family_specific_tail() {
    let fomt = fs::read(local_rom_path(CASES[0].rom)).unwrap();
    let mfomt = fs::read(local_rom_path(CASES[1].rom)).unwrap();
    for (case, (dispatcher, function, reference, base, length)) in CASES.iter().zip([
        (0x42DD4, 0x9ABD8, &fomt, 0x9ABD8, 0xE8),
        (0x42FF4, 0x9FB34, &mfomt, 0x9FB34, 0xF8),
        (0x42A48, 0x9A610, &fomt, 0x9ABD8, 0xE8),
        (0x42D50, 0x9F574, &mfomt, 0x9FB34, 0xF8),
    ]) {
        let rom = fs::read(local_rom_path(case.rom)).unwrap();
        let pc = dispatcher + 0x22;
        let hi = u16::from_le_bytes(rom[pc..pc + 2].try_into().unwrap());
        let lo = u16::from_le_bytes(rom[pc + 2..pc + 4].try_into().unwrap());
        assert_eq!(hi & 0xF800, 0xF000);
        assert_eq!(lo & 0xF800, 0xF800);
        let raw = (((hi & 0x7FF) as i32) << 12) | (((lo & 0x7FF) as i32) << 1);
        assert_eq!((pc as i32 + 4 + ((raw << 9) >> 9)) as usize, function);
        // Includes account initialization, all balance/record updates and
        // literals. Compare within family, not by assuming MFoMT == FoMT.
        assert_eq!(
            &rom[function..function + length],
            &reference[base..base + length],
            "{} money addition",
            case.name
        );
    }
    // MFoMT-only tail: balance == cap sets account byte +4 bit 2. The paired
    // VarGet consumer is independently pinned by the following regression.
    assert_eq!(
        &mfomt[0x9FC08..0x9FC16],
        &[0x10, 0x68, 0xB0, 0x42, 0x03, 0xD1, 0x10, 0x79, 0x04, 0x21, 0x08, 0x43, 0x10, 0x71]
    );
}

#[test]
fn native_mfomt_billion_g_achievement_reads_the_add_money_flag() {
    for (case, getter_table, getter, extractor) in [
        (&CASES[1], 0x46050, 0x47210, 0x4A38C),
        (&CASES[3], 0x45DAC, 0x46F6C, 0x4A0E8),
    ] {
        let rom = fs::read(local_rom_path(case.rom)).unwrap();
        assert_eq!(
            u32::from_le_bytes(
                rom[getter_table + 691 * 4..getter_table + 691 * 4 + 4]
                    .try_into()
                    .unwrap()
            ),
            0x0800_0000 + getter as u32,
            "{} variable 691 getter",
            case.name
        );
        assert_eq!(
            u32::from_le_bytes(rom[getter + 0x14..getter + 0x18].try_into().unwrap()),
            0x1AB8,
            "{} account object offset",
            case.name
        );
        assert_eq!(
            &rom[getter + 0x0C..getter + 0x0E],
            &[0x00, 0x79],
            "{} reads account byte +4",
            case.name
        );
        assert_eq!(thumb_call_target(&rom, getter + 0x0E), extractor);
        assert_eq!(
            &rom[extractor..extractor + 4],
            &[0x40, 0x07, 0xC0, 0x0F],
            "{} extracts bit 2",
            case.name
        );
    }
}

#[test]
fn native_money_subtraction_implementation_matches_all_targets() {
    let reference = fs::read(local_rom_path(CASES[0].rom)).unwrap();
    for (case, (offset, dispatcher)) in CASES.iter().zip([
        (0x9ACC0, 0x42E04),
        (0x9FC2C, 0x43024),
        (0x9A6F8, 0x42A78),
        (0x9F66C, 0x42D80),
    ]) {
        let rom = fs::read(local_rom_path(case.rom)).unwrap();
        let target = |pc: usize| {
            let hi = u16::from_le_bytes(rom[pc..pc + 2].try_into().unwrap());
            let lo = u16::from_le_bytes(rom[pc + 2..pc + 4].try_into().unwrap());
            assert_eq!(hi & 0xF800, 0xF000);
            assert_eq!(lo & 0xF800, 0xF800);
            let raw = (((hi & 0x7FF) as i32) << 12) | (((lo & 0x7FF) as i32) << 1);
            (pc as i32 + 4 + ((raw << 9) >> 9)) as usize
        };
        assert_eq!(target(dispatcher + 0x22), offset);
        let exit = target(dispatcher + 0x26);
        // Immediately discards native boolean and exits: no VM result push.
        // The MFoMT dispatcher has a larger local stack frame (0x5C vs 0x40).
        let frame = if case.name.starts_with("mfomt") {
            0x17
        } else {
            0x10
        };
        assert_eq!(
            &rom[exit..exit + 16],
            &[
                0x00, 0x20, frame, 0xB0, 0x38, 0xBC, 0x98, 0x46, 0xA1, 0x46, 0xAA, 0x46, 0xF0,
                0xBC, 0x02, 0xBC
            ]
        );
        // Complete account subtraction implementation and literals. Includes
        // record initialization BEFORE the insufficient-funds check, and
        // false return without balance modification on insufficient funds.
        // Dispatcher linkage and discarded return are checked above.
        assert_eq!(
            &rom[offset..offset + 0xE8],
            &reference[0x9ACC0..0x9ADA8],
            "{} account subtraction implementation",
            case.name
        );
    }
}

#[test]
fn native_wait_counter_decrements_before_zero_check_on_all_targets() {
    let reference = fs::read(local_rom_path(CASES[0].rom)).unwrap();
    for (case, (setup, update)) in CASES.iter().zip([
        (0x12BAC, 0xDA758),
        (0x12D38, 0xE2C7C),
        (0x12A7C, 0xD9F0C),
        (0x12BDC, 0xE278C),
    ]) {
        let rom = fs::read(local_rom_path(case.rom)).unwrap();
        // Complete setup: store halfword at state+E0 and enter state 21.
        assert_eq!(
            &rom[setup..setup + 16],
            &reference[0x12BAC..0x12BBC],
            "{} wait setup",
            case.name
        );
        // Complete waiting-state branch: decrement, test zero, then restore
        // state 2 only on zero. Zero wraps through FFFF. Scheduler cadence
        // is outside this check.
        assert_eq!(
            &rom[update..update + 32],
            &reference[0xDA758..0xDA778],
            "{} wait decrement",
            case.name
        );
    }
}

#[test]
fn native_text_number_width_contract_differs_by_region_without_runtime_clamp() {
    fn call_target(rom: &[u8], pc: usize) -> usize {
        let hi = u16::from_le_bytes(rom[pc..pc + 2].try_into().unwrap());
        let lo = u16::from_le_bytes(rom[pc + 2..pc + 4].try_into().unwrap());
        assert_eq!(hi & 0xF800, 0xF000);
        assert_eq!(lo & 0xF800, 0xF800);
        let raw = (((hi & 0x7FF) as i32) << 12) | (((lo & 0x7FF) as i32) << 1);
        (pc as i32 + 4 + ((raw << 9) >> 9)) as usize
    }

    let cases = [
        (0x3F904, 0x39, 0x41032, 0x12ADC, 0x4EC84, false),
        (0x3FAF0, 0x3A, 0x4125E, 0x12C68, 0x5123C, false),
        (0x3F578, 0x39, 0x40CA6, 0x129AC, 0x4E98C, true),
        (0x3F84C, 0x3A, 0x40FBA, 0x12B0C, 0x50E78, true),
    ];
    let mut regional_reference: [Option<Vec<u8>>; 2] = [None, None];

    for (case, (table, slot, handler, wrapper, formatter, japanese)) in CASES.iter().zip(cases) {
        let rom = fs::read(local_rom_path(case.rom)).unwrap();
        assert_eq!(
            u32::from_le_bytes(
                rom[table + slot * 4..table + slot * 4 + 4]
                    .try_into()
                    .unwrap()
            ),
            0x08000000u32 + handler as u32,
            "{} physical callable entry",
            case.name
        );
        assert_eq!(
            call_target(&rom, handler + 0x50),
            wrapper,
            "{} wrapper",
            case.name
        );
        assert_eq!(
            call_target(&rom, wrapper + 0x0E),
            formatter,
            "{} formatter",
            case.name
        );

        let mut body = rom[formatter..formatter + 0x90].to_vec();
        let call_offsets: &[usize] = if japanese {
            &[0x3C, 0x48]
        } else {
            &[0x24, 0x32]
        };
        for &offset in call_offsets {
            body[offset..offset + 4].fill(0);
        }
        let family = usize::from(japanese);
        if let Some(reference) = &regional_reference[family] {
            assert_eq!(&body, reference, "{} regional formatter body", case.name);
        } else {
            regional_reference[family] = Some(body);
        }

        if japanese {
            // Width zero selects a separate unrestricted conversion path;
            // nonzero width is used directly as the loop/output bound.
            assert_eq!(
                &rom[formatter + 0x10..formatter + 0x16],
                &[0x17, 0x1C, 0x00, 0x2F, 0x44, 0xD0]
            );
            assert_eq!(
                &rom[formatter + 0x2C..formatter + 0x30],
                &[0xBD, 0x42, 0x1E, 0xD2]
            );
            assert_eq!(
                &rom[formatter + 0x60..formatter + 0x66],
                &[0x01, 0x35, 0xBD, 0x42, 0xFA, 0xD3]
            );
        } else {
            // The digit loop has its own ten-index ceiling, but the later
            // padding loop compares the caller width directly and has no clamp.
            assert_eq!(
                &rom[formatter + 0x40..formatter + 0x44],
                &[0x0A, 0x2E, 0xEC, 0xD9]
            );
            assert_eq!(
                &rom[formatter + 0x56..formatter + 0x60],
                &[0x20, 0x21, 0x6A, 0x46, 0x10, 0x19, 0x01, 0x70, 0x01, 0x34]
            );
        }
    }
}

#[test]
fn native_text_variable_setters_keep_slots_argument_order_and_byte_capacities() {
    for (case, (dispatch, first_slot, handlers, wrappers, setter, max_bytes, mfomt_us_stride)) in
        CASES.iter().zip([
            (
                0x3F904,
                0x38,
                [0x40FEC, 0x41032, 0x4108A],
                [0x12AA4, 0x12ADC, 0x12ACC],
                0x3B6B8,
                22u8,
                false,
            ),
            (
                0x3FAF0,
                0x39,
                [0x41218, 0x4125E, 0x412B6],
                [0x12C30, 0x12C68, 0x12C58],
                0x3B9B0,
                28,
                true,
            ),
            (
                0x3F578,
                0x38,
                [0x40C60, 0x40CA6, 0x40CFE],
                [0x12974, 0x129AC, 0x1299C],
                0x3B44C,
                20,
                false,
            ),
            (
                0x3F84C,
                0x39,
                [0x40F74, 0x40FBA, 0x41012],
                [0x12AD4, 0x12B0C, 0x12AFC],
                0x3B828,
                20,
                false,
            ),
        ])
    {
        let rom = fs::read(local_rom_path(case.rom)).unwrap();
        let handler_calls = [0x3E, 0x50, 0x5A];
        let setter_calls = [0x1A, 0x1A, 0x08];
        for index in 0..3 {
            let slot = first_slot + index;
            assert_eq!(
                u32::from_le_bytes(
                    rom[dispatch + slot * 4..dispatch + slot * 4 + 4]
                        .try_into()
                        .unwrap()
                ),
                0x0800_0000 + handlers[index] as u32,
                "{} text-variable callable slot {slot:#x}",
                case.name
            );
            assert_eq!(
                thumb_call_target(&rom, handlers[index] + handler_calls[index]),
                wrappers[index],
                "{} text-variable wrapper {index}",
                case.name
            );
            assert_eq!(
                thumb_call_target(&rom, wrappers[index] + setter_calls[index]),
                setter,
                "{} shared text-variable setter {index}",
                case.name
            );
        }

        // The setter clamps encoded byte length before writing a terminator.
        // It does not compare the variable index with four before calculating
        // the destination, so TEXT_VARIABLE_1..4 is a caller-side contract.
        assert_eq!(
            &rom[setter + 0x12..setter + 0x18],
            &[max_bytes, 0x2C, 0x00, 0xD9, max_bytes, 0x24]
        );
        if mfomt_us_stride {
            assert_eq!(
                &rom[setter + 0x2A..setter + 0x32],
                &[0x70, 0x01, 0xC0, 0x19, 0x08, 0x30, 0x69, 0x46]
            );
        } else {
            assert_eq!(
                &rom[setter + 0x2A..setter + 0x36],
                &[0x70, 0x00, 0x80, 0x19, 0xC0, 0x00, 0xC0, 0x19, 0x08, 0x30, 0x69, 0x46]
            );
        }
    }
}

#[test]
fn native_held_item_query_and_action_slots_match_all_targets() {
    for (
        case,
        (
            dispatch,
            first_slot,
            handlers,
            empty_query,
            kind_query,
            wrapped_query,
            use_leaf,
            clear_leaf,
        ),
    ) in CASES.iter().zip([
        (
            0x3F904,
            0x3F,
            [
                0x4129A, 0x41332, 0x41384, 0x413D6, 0x4143C, 0x414A4, 0x41500, 0x41514,
            ],
            0x0F190,
            0x0F204,
            0x0F388,
            0x141EC,
            0x14214,
        ),
        (
            0x3FAF0,
            0x40,
            [
                0x414D6, 0x41576, 0x415C8, 0x4161A, 0x41680, 0x416E8, 0x41744, 0x41758,
            ],
            0x0F26C,
            0x0F2E0,
            0x0F464,
            0x14378,
            0x143A0,
        ),
        (
            0x3F578,
            0x3F,
            [
                0x40F0E, 0x40FA6, 0x40FF8, 0x4104A, 0x410B0, 0x41118, 0x41174, 0x41188,
            ],
            0x0F170,
            0x0F1E4,
            0x0F368,
            0x140C0,
            0x140E8,
        ),
        (
            0x3F84C,
            0x40,
            [
                0x41232, 0x412D2, 0x41324, 0x41376, 0x413DC, 0x41444, 0x414A0, 0x414B4,
            ],
            0x0F220,
            0x0F294,
            0x0F418,
            0x14220,
            0x14248,
        ),
    ]) {
        let rom = fs::read(local_rom_path(case.rom)).unwrap();
        for (index, handler) in handlers.into_iter().enumerate() {
            let slot = first_slot + index;
            assert_eq!(
                u32::from_le_bytes(
                    rom[dispatch + slot * 4..dispatch + slot * 4 + 4]
                        .try_into()
                        .unwrap()
                ),
                0x0800_0000 + handler as u32,
                "{} held-item callable slot {slot:#x}",
                case.name
            );
        }

        assert_eq!(
            thumb_call_target(&rom, handlers[0] + 0x0C),
            empty_query,
            "{} empty query",
            case.name
        );
        assert_eq!(
            thumb_call_target(&rom, handlers[1] + 0x0E),
            empty_query,
            "{} kind empty guard",
            case.name
        );
        assert_eq!(
            thumb_call_target(&rom, handlers[1] + 0x1A),
            kind_query,
            "{} kind query",
            case.name
        );
        assert_eq!(
            thumb_call_target(&rom, handlers[2] + 0x0E),
            empty_query,
            "{} wrapped empty guard",
            case.name
        );
        assert_eq!(
            thumb_call_target(&rom, handlers[2] + 0x1A),
            wrapped_query,
            "{} wrapped query",
            case.name
        );
        assert_eq!(
            thumb_call_target(&rom, handlers[6] + 0x0E),
            use_leaf,
            "{} use held item",
            case.name
        );
        assert_eq!(
            thumb_call_target(&rom, handlers[7] + 0x10),
            clear_leaf,
            "{} clear held item",
            case.name
        );

        // Food, article and chicken getters guard kinds 0, 1 and 3 before
        // extracting the category-specific ID. Every invalid getter path,
        // including GetPlayerHeldItemKind on an empty hand, pushes -1.
        for (handler, kind) in [(handlers[3], 0u8), (handlers[4], 1), (handlers[5], 3)] {
            assert_eq!(
                &rom[handler + 0x1E..handler + 0x20],
                &[kind, 0x28],
                "{} held kind guard {kind}",
                case.name
            );
        }
        assert_eq!(
            &rom[handlers[1] + 0x4A..handlers[1] + 0x4E],
            &[0x01, 0x21, 0x49, 0x42],
            "{} empty kind sentinel",
            case.name
        );
        assert_eq!(
            &rom[handlers[3] + 0x5E..handlers[3] + 0x62],
            &[0x01, 0x21, 0x49, 0x42],
            "{} invalid food sentinel",
            case.name
        );
        assert_eq!(
            &rom[handlers[4] + 0x60..handlers[4] + 0x64],
            &[0x01, 0x21, 0x49, 0x42],
            "{} invalid article sentinel",
            case.name
        );
        assert_eq!(
            &rom[handlers[5] + 0x54..handlers[5] + 0x58],
            &[0x01, 0x21, 0x49, 0x42],
            "{} invalid chicken sentinel",
            case.name
        );
    }
}

#[test]
fn native_held_food_and_article_setters_share_constructors_and_preserve_wrapped_flag() {
    for (
        case,
        (
            dispatch,
            first_slot,
            handlers,
            food_value,
            article_value,
            food_constructor,
            article_constructor,
            apply_held,
        ),
    ) in CASES.iter().zip([
        (
            0x3F904,
            0x47,
            [0x4152C, 0x41580, 0x415C0, 0x41612],
            0x0DCA8,
            0x0DF50,
            0x0F004,
            0x0F040,
            0x14164,
        ),
        (
            0x3FAF0,
            0x48,
            [0x41770, 0x417C4, 0x41804, 0x41856],
            0x0DD1C,
            0x0DFC4,
            0x0F0E0,
            0x0F11C,
            0x142F0,
        ),
        (
            0x3F578,
            0x47,
            [0x411A0, 0x411F4, 0x41234, 0x41286],
            0x0DC88,
            0x0DF30,
            0x0EFE4,
            0x0F020,
            0x14038,
        ),
        (
            0x3F84C,
            0x48,
            [0x414CC, 0x41520, 0x41560, 0x415B2],
            0x0DCD0,
            0x0DF78,
            0x0F094,
            0x0F0D0,
            0x14198,
        ),
    ]) {
        let rom = fs::read(local_rom_path(case.rom)).unwrap();
        for (index, handler) in handlers.into_iter().enumerate() {
            let slot = first_slot + index;
            assert_eq!(
                u32::from_le_bytes(
                    rom[dispatch + slot * 4..dispatch + slot * 4 + 4]
                        .try_into()
                        .unwrap()
                ),
                0x0800_0000 + handler as u32,
                "{} held setter slot {slot:#x}",
                case.name
            );
        }

        for (
            index,
            (value_call, constructor_call, apply_call, value_leaf, constructor_leaf, wrapped),
        ) in [
            (0x28, 0x44, 0x4E, food_value, food_constructor, 0u8),
            (0x28, 0x30, 0x3A, article_value, article_constructor, 0),
            (0x26, 0x42, 0x4C, food_value, food_constructor, 1),
            (0x28, 0x30, 0x3A, article_value, article_constructor, 1),
        ]
        .into_iter()
        .enumerate()
        {
            let handler = handlers[index];
            assert_eq!(
                thumb_call_target(&rom, handler + value_call),
                value_leaf,
                "{} held setter value constructor {index}",
                case.name
            );
            assert_eq!(
                thumb_call_target(&rom, handler + constructor_call),
                constructor_leaf,
                "{} held setter object constructor {index}",
                case.name
            );
            assert_eq!(
                thumb_call_target(&rom, handler + apply_call),
                apply_held,
                "{} held setter apply leaf {index}",
                case.name
            );
            assert_eq!(
                &rom[handler + apply_call - 2..handler + apply_call],
                &[wrapped, 0x22],
                "{} held setter wrapped flag {index}",
                case.name
            );
        }
    }
}

#[test]
fn native_held_item_actions_and_tool_stack_slots_match_all_targets() {
    for (
        case,
        (
            dispatch,
            first_slot,
            handlers,
            item_empty,
            item_kind,
            item_article,
            article_discardable,
            try_ship,
            throw_item,
            tool_empty,
            tool_get,
            tool_id,
            tool_amount,
            tool_value,
            tool_stack,
            empty_stack,
            apply_tool,
        ),
    ) in CASES.iter().zip([
        (
            0x3F904,
            0x4B,
            [
                0x412B8, 0x41652, 0x41690, 0x416A2, 0x416F4, 0x4173C, 0x41860,
            ],
            0x0F190,
            0x0F204,
            0x0F258,
            0x0DFB0,
            0x1412C,
            0x141C4,
            0x0DC34,
            0x0DC10,
            0x0DB30,
            0x0DC48,
            0x0DB2C,
            0x0DBE0,
            0x0DBC8,
            0xD3994,
        ),
        (
            0x3FAF0,
            0x4C,
            [
                0x414FC, 0x41896, 0x418D4, 0x418E6, 0x41938, 0x41980, 0x41AA4,
            ],
            0x0F26C,
            0x0F2E0,
            0x0F334,
            0x0E024,
            0x142B8,
            0x14350,
            0x0DCA8,
            0x0DC84,
            0x0DBA4,
            0x0DCBC,
            0x0DBA0,
            0x0DC54,
            0x0DC3C,
            0xDB5BC,
        ),
        (
            0x3F578,
            0x4B,
            [
                0x40F2C, 0x412C6, 0x41304, 0x41316, 0x41368, 0x413B0, 0x414D4,
            ],
            0x0F170,
            0x0F1E4,
            0x0F238,
            0x0DF90,
            0x14000,
            0x14098,
            0x0DC14,
            0x0DBF0,
            0x0DB10,
            0x0DC28,
            0x0DB0C,
            0x0DBC0,
            0x0DBA8,
            0xD314C,
        ),
        (
            0x3F84C,
            0x4C,
            [
                0x41258, 0x415F2, 0x41630, 0x41642, 0x41694, 0x416DC, 0x41800,
            ],
            0x0F220,
            0x0F294,
            0x0F2E8,
            0x0DFD8,
            0x14160,
            0x141F8,
            0x0DC5C,
            0x0DC38,
            0x0DB58,
            0x0DC70,
            0x0DB54,
            0x0DC08,
            0x0DBF0,
            0xDB0D0,
        ),
    ]) {
        let rom = fs::read(local_rom_path(case.rom)).unwrap();
        for (index, handler) in handlers.into_iter().enumerate() {
            let slot = first_slot + index;
            assert_eq!(
                u32::from_le_bytes(
                    rom[dispatch + slot * 4..dispatch + slot * 4 + 4]
                        .try_into()
                        .unwrap()
                ),
                0x0800_0000 + handler as u32,
                "{} held action/tool slot {slot:#x}",
                case.name
            );
        }

        let discard = handlers[0];
        assert_eq!(
            thumb_call_target(&rom, discard + 0x0E),
            item_empty,
            "{} discard empty guard",
            case.name
        );
        assert_eq!(
            thumb_call_target(&rom, discard + 0x1A),
            item_kind,
            "{} discard kind query",
            case.name
        );
        assert_eq!(
            thumb_call_target(&rom, discard + 0x26),
            item_article,
            "{} discard article query",
            case.name
        );
        assert_eq!(
            thumb_call_target(&rom, discard + 0x30),
            article_discardable,
            "{} article discardable leaf",
            case.name
        );
        assert_eq!(
            thumb_call_target(&rom, handlers[1] + 0x0E),
            try_ship,
            "{} shipment attempt leaf",
            case.name
        );
        assert_eq!(
            thumb_call_target(&rom, handlers[2] + 0x0C),
            throw_item,
            "{} throw held item leaf",
            case.name
        );

        assert_eq!(
            thumb_call_target(&rom, handlers[3] + 0x12),
            tool_empty,
            "{} held tool empty guard",
            case.name
        );
        assert_eq!(
            thumb_call_target(&rom, handlers[3] + 0x1E),
            tool_get,
            "{} held tool getter",
            case.name
        );
        assert_eq!(
            thumb_call_target(&rom, handlers[3] + 0x28),
            tool_id,
            "{} held tool ID",
            case.name
        );
        assert_eq!(
            thumb_call_target(&rom, handlers[4] + 0x12),
            tool_empty,
            "{} held tool count empty guard",
            case.name
        );
        assert_eq!(
            thumb_call_target(&rom, handlers[4] + 0x1E),
            tool_amount,
            "{} held tool amount",
            case.name
        );
        assert_eq!(
            thumb_call_target(&rom, handlers[5] + 0x40),
            tool_value,
            "{} tool value constructor",
            case.name
        );
        assert_eq!(
            thumb_call_target(&rom, handlers[5] + 0x4A),
            tool_stack,
            "{} tool stack constructor",
            case.name
        );
        assert_eq!(
            thumb_call_target(&rom, handlers[6] + 0x0E),
            empty_stack,
            "{} empty tool stack constructor",
            case.name
        );
        assert_eq!(
            thumb_call_target(&rom, handlers[6] + 0x18),
            apply_tool,
            "{} shared held tool apply path",
            case.name
        );

        // The ToolStack constructor uses an unsigned minimum with 99. Zero is
        // replaced by one; negative VM integers therefore also clamp to 99.
        assert_eq!(
            &rom[tool_stack + 0x0C..tool_stack + 0x12],
            &[0x00, 0x2A, 0x08, 0xD0, 0x63, 0x20],
            "{} tool stack zero/max branches",
            case.name
        );
    }
}

#[test]
fn native_rucksack_search_slot_and_add_callables_match_all_targets() {
    for (
        case,
        (
            dispatch,
            first_slot,
            handlers,
            find_food,
            find_article,
            item_at,
            empty_item,
            free_tool,
            free_item,
            add_article,
            add_food,
            add_tool,
        ),
    ) in CASES.iter().zip([
        (
            0x3F904,
            0x52,
            [
                0x41884, 0x418BC, 0x418F4, 0x41930, 0x41948, 0x4196C, 0x419B4, 0x419FC,
            ],
            0x0F6B4,
            0x0F714,
            0x0FCDC,
            0x0EFEC,
            0x0F8A8,
            0x0F634,
            0x0FDC4,
            0x0FD50,
            0x0FEC8,
        ),
        (
            0x3FAF0,
            0x53,
            [
                0x41AC8, 0x41AFC, 0x41B30, 0x41B6C, 0x41B90, 0x41BB4, 0x41BFC, 0x41C44,
            ],
            0x0F790,
            0x0F7F0,
            0x0FDB8,
            0x0F0C8,
            0x0F984,
            0x0F710,
            0x0FEA0,
            0x0FE2C,
            0x0FFA4,
        ),
        (
            0x3F578,
            0x52,
            [
                0x414F8, 0x41530, 0x41568, 0x415A4, 0x415BC, 0x415E0, 0x41628, 0x41670,
            ],
            0x0F694,
            0x0F6F4,
            0x0FCBC,
            0x0EFCC,
            0x0F888,
            0x0F614,
            0x0FDA4,
            0x0FD30,
            0x0FEA8,
        ),
        (
            0x3F84C,
            0x53,
            [
                0x41824, 0x41858, 0x4188C, 0x418C8, 0x418EC, 0x41910, 0x41958, 0x419A0,
            ],
            0x0F744,
            0x0F7A4,
            0x0FD6C,
            0x0F07C,
            0x0F938,
            0x0F6C4,
            0x0FE54,
            0x0FDE0,
            0x0FF58,
        ),
    ]) {
        let rom = fs::read(local_rom_path(case.rom)).unwrap();
        for (index, handler) in handlers.into_iter().enumerate() {
            let slot = first_slot + index;
            assert_eq!(
                u32::from_le_bytes(
                    rom[dispatch + slot * 4..dispatch + slot * 4 + 4]
                        .try_into()
                        .unwrap()
                ),
                0x0800_0000 + handler as u32,
                "{} rucksack callable slot {slot:#x}",
                case.name
            );
        }

        assert_eq!(
            thumb_call_target(&rom, handlers[0] + 0x28),
            find_food,
            "{} find food",
            case.name
        );
        assert_eq!(
            thumb_call_target(&rom, handlers[1] + 0x28),
            find_article,
            "{} find article",
            case.name
        );
        assert_eq!(
            thumb_call_target(&rom, handlers[2] + 0x22),
            item_at,
            "{} rucksack item slot access",
            case.name
        );
        assert_eq!(
            thumb_call_target(&rom, handlers[2] + 0x2A),
            empty_item,
            "{} empty rucksack item constructor",
            case.name
        );
        assert_eq!(
            thumb_call_target(&rom, handlers[3] + 0x0C),
            free_tool,
            "{} first free tool slot",
            case.name
        );
        assert_eq!(
            thumb_call_target(&rom, handlers[4] + 0x0C),
            free_item,
            "{} first free item slot",
            case.name
        );
        assert_eq!(
            thumb_call_target(&rom, handlers[5] + 0x3A),
            add_article,
            "{} add article",
            case.name
        );
        assert_eq!(
            thumb_call_target(&rom, handlers[6] + 0x3A),
            add_food,
            "{} add food",
            case.name
        );
        assert_eq!(
            thumb_call_target(&rom, handlers[7] + 0x3A),
            add_tool,
            "{} add tool",
            case.name
        );
    }
}

#[test]
fn native_player_inventory_queries_distinguish_held_rucksack_and_storage_sources() {
    for (
        case,
        (dispatch, first_slot, handlers, holding_tool, owns_tool, owns_food, owns_article),
    ) in CASES.iter().zip([
        (
            0x3F904,
            0x5C,
            [0x43FDE, 0x440CE, 0x440F8, 0x44122],
            0x14D7C,
            0x4E354,
            0x4E2C0,
            0x4E22C,
        ),
        (
            0x3FAF0,
            0x5D,
            [0x44246, 0x44340, 0x4436A, 0x44394],
            0x14F08,
            0x50910,
            0x5087C,
            0x507E8,
        ),
        (
            0x3F578,
            0x5C,
            [0x43C52, 0x43D42, 0x43D6C, 0x43D96],
            0x14C50,
            0x4E17C,
            0x4E0E8,
            0x4E054,
        ),
        (
            0x3F84C,
            0x5D,
            [0x43FA2, 0x4409C, 0x440C6, 0x440F0],
            0x14DB0,
            0x5066C,
            0x505D8,
            0x50544,
        ),
    ]) {
        let rom = fs::read(local_rom_path(case.rom)).unwrap();
        for (index, handler) in handlers.into_iter().enumerate() {
            let slot = first_slot + index;
            assert_eq!(
                u32::from_le_bytes(
                    rom[dispatch + slot * 4..dispatch + slot * 4 + 4]
                        .try_into()
                        .unwrap()
                ),
                0x0800_0000 + handler as u32,
                "{} player inventory query slot {slot:#x}",
                case.name
            );
        }
        assert_eq!(
            thumb_call_target(&rom, handlers[0] + 0x08),
            holding_tool,
            "{} holding-tool query",
            case.name
        );
        assert_eq!(
            thumb_call_target(&rom, handlers[1] + 0x1E),
            owns_tool,
            "{} owns-tool aggregate",
            case.name
        );
        assert_eq!(
            thumb_call_target(&rom, handlers[2] + 0x1E),
            owns_food,
            "{} owns-food aggregate",
            case.name
        );
        assert_eq!(
            thumb_call_target(&rom, handlers[3] + 0x1E),
            owns_article,
            "{} owns-article aggregate",
            case.name
        );

        // Each aggregate has three ordered ownership sources: held value,
        // rucksack, then the category-specific long-term storage container.
        for (aggregate, source_offsets) in [
            (owns_tool, [0x16usize, 0x4A, 0x62]),
            (owns_food, [0x16, 0x52, 0x72]),
            (owns_article, [0x16, 0x54, 0x74]),
        ] {
            for offset in source_offsets {
                let _ = thumb_call_target(&rom, aggregate + offset);
            }
        }
    }
}

#[test]
fn native_show_player_holding_tool_enters_the_acquisition_action_on_all_targets() {
    for (case, (dispatch, slot, handler, native)) in CASES.iter().zip([
        (0x3F904, 0x5A, 0x45456, 0x16F34),
        (0x3FAF0, 0x5B, 0x45726, 0x16FB4),
        (0x3F578, 0x5A, 0x450CA, 0x16CC8),
        (0x3F84C, 0x5B, 0x45482, 0x16E28),
    ]) {
        let rom = fs::read(local_rom_path(case.rom)).unwrap();
        assert_eq!(
            u32::from_le_bytes(
                rom[dispatch + slot * 4..dispatch + slot * 4 + 4]
                    .try_into()
                    .unwrap()
            ),
            0x0800_0000 + handler as u32,
            "{} show-held-tool slot",
            case.name
        );
        assert_eq!(
            thumb_call_target(&rom, handler + 0x26),
            native,
            "{} show-held-tool native",
            case.name
        );
        // The native function stores event/player action state 0x19.
        assert_eq!(
            &rom[native + 0x20..native + 0x26],
            &[0x9C, 0x34, 0x19, 0x20, 0x20, 0x60],
            "{} acquisition action state",
            case.name
        );
    }
}

#[test]
fn native_remove_owned_articles_and_berry_acquisition_slots_match_all_targets() {
    for (
        case,
        (
            dispatch,
            remove_slot,
            remove_handler,
            item_at_offset,
            item_at,
            item_empty,
            item_kind,
            clear_held,
            power_handler,
            power_native,
            mystic_handler,
            mystic_native,
        ),
    ) in CASES.iter().zip([
        (
            0x3F904, 0x60, 0x451EE, 0x7A, 0x0FCDC, 0x0F06C, 0x0F09C, 0x0F390, 0x44A94, 0x158CC,
            0x44AAC, 0x158F8,
        ),
        (
            0x3FAF0, 0x61, 0x454C8, 0x78, 0x0FDB8, 0x0F148, 0x0F178, 0x0F46C, 0x44D04, 0x1594C,
            0x44D1C, 0x15978,
        ),
        (
            0x3F578, 0x60, 0x44E62, 0x7A, 0x0FCBC, 0x0F04C, 0x0F07C, 0x0F370, 0x44708, 0x15660,
            0x44720, 0x1568C,
        ),
        (
            0x3F84C, 0x61, 0x45224, 0x78, 0x0FD6C, 0x0F0FC, 0x0F12C, 0x0F420, 0x44A60, 0x157C0,
            0x44A78, 0x157EC,
        ),
    ]) {
        let rom = fs::read(local_rom_path(case.rom)).unwrap();
        for (slot, handler) in [
            (remove_slot, remove_handler),
            (remove_slot + 1, power_handler),
            (remove_slot + 2, mystic_handler),
        ] {
            assert_eq!(
                u32::from_le_bytes(
                    rom[dispatch + slot * 4..dispatch + slot * 4 + 4]
                        .try_into()
                        .unwrap()
                ),
                0x0800_0000 + handler as u32,
                "{} article-removal/berry slot {slot:#x}",
                case.name
            );
        }
        assert_eq!(
            thumb_call_target(&rom, remove_handler + 0x58),
            clear_held,
            "{} clear matching held article",
            case.name
        );
        assert_eq!(
            thumb_call_target(&rom, remove_handler + item_at_offset),
            item_at,
            "{} inspect rucksack article",
            case.name
        );
        assert_eq!(
            thumb_call_target(&rom, remove_handler + item_at_offset + 0x06),
            item_empty,
            "{} rucksack item empty query",
            case.name
        );
        assert_eq!(
            thumb_call_target(&rom, remove_handler + item_at_offset + 0x12),
            item_kind,
            "{} rucksack item kind query",
            case.name
        );
        assert_eq!(
            thumb_call_target(&rom, power_handler + 0x10),
            power_native,
            "{} power berry acquisition",
            case.name
        );
        assert_eq!(
            thumb_call_target(&rom, mystic_handler + 0x10),
            mystic_native,
            "{} mystic berry acquisition",
            case.name
        );
    }
}

#[test]
fn native_riding_bathing_and_preserved_location_slots_match_all_targets() {
    for (case, (dispatch, first_slot, handlers)) in CASES.iter().zip([
        (
            0x3F904,
            0x63,
            [0x44AC4, 0x44FA0, 0x44FB4, 0x44FC8, 0x44FDC, 0x44FF0],
        ),
        (
            0x3FAF0,
            0x64,
            [0x44D34, 0x4521C, 0x45232, 0x4524A, 0x45262, 0x4527A],
        ),
        (
            0x3F578,
            0x63,
            [0x44738, 0x44C14, 0x44C28, 0x44C3C, 0x44C50, 0x44C64],
        ),
        (
            0x3F84C,
            0x64,
            [0x44A90, 0x44F78, 0x44F8E, 0x44FA6, 0x44FBE, 0x44FD6],
        ),
    ]) {
        let rom = fs::read(local_rom_path(case.rom)).unwrap();
        for (index, handler) in handlers.into_iter().enumerate() {
            let slot = first_slot + index;
            assert_eq!(
                u32::from_le_bytes(
                    rom[dispatch + slot * 4..dispatch + slot * 4 + 4]
                        .try_into()
                        .unwrap()
                ),
                0x0800_0000 + handler as u32,
                "{} riding/bathing/preserved-location slot {slot:#x}",
                case.name
            );
        }
    }

    // FoMT-US exposes the preserved-location record directly. Clearing it
    // resets the enabled byte and stores the FoMT MAP_NONE value 0x0234.
    let rom = fs::read(local_rom_path(CASES[0].rom)).unwrap();
    assert_eq!(thumb_call_target(&rom, 0x44FC8 + 0x0E), 0x16D48);
    assert_eq!(thumb_call_target(&rom, 0x44FDC + 0x0E), 0x16D80);
    assert_eq!(thumb_call_target(&rom, 0x44FF0 + 0x0E), 0x16D9C);
    assert_eq!(
        &rom[0x16D8A..0x16D94],
        &[0x00, 0x21, 0x01, 0x70, 0x8D, 0x21, 0x89, 0x00, 0x41, 0x60]
    );
}

#[test]
fn native_random_meal_scripted_animation_and_presented_item_slots_match_all_targets() {
    for (case, (dispatch, first_slot, handlers, presented_leaves, presented_call_offsets)) in
        CASES.iter().zip([
            (
                0x3F904,
                0x69,
                [0x45010, 0x45028, 0x4503C, 0x44AE8, 0x44B0A, 0x44B2C],
                [0x15950, 0x15970, 0x15990],
                [0x0E, 0x0E, 0x0E],
            ),
            (
                0x3FAF0,
                0x6A,
                [0x4529A, 0x452B2, 0x452CA, 0x44D4E, 0x44D70, 0x44D8C],
                [0x159D0, 0x159F0, 0x15A10],
                [0x0E, 0x12, 0x0E],
            ),
            (
                0x3F578,
                0x69,
                [0x44C84, 0x44C9C, 0x44CB0, 0x4475C, 0x4477E, 0x447A0],
                [0x156E4, 0x15704, 0x15724],
                [0x0E, 0x0E, 0x0E],
            ),
            (
                0x3F84C,
                0x6A,
                [0x44FF6, 0x4500E, 0x45026, 0x44AAA, 0x44ACC, 0x44AE8],
                [0x15844, 0x15864, 0x15884],
                [0x0E, 0x12, 0x0E],
            ),
        ])
    {
        let rom = fs::read(local_rom_path(case.rom)).unwrap();
        for (index, handler) in handlers.into_iter().enumerate() {
            let slot = first_slot + index;
            assert_eq!(
                u32::from_le_bytes(
                    rom[dispatch + slot * 4..dispatch + slot * 4 + 4]
                        .try_into()
                        .unwrap()
                ),
                0x0800_0000 + handler as u32,
                "{} meal/animation/presented-item slot {slot:#x}",
                case.name
            );
        }

        // All four native implementations resolve the same three player-actor
        // virtuals, in order: captured kind (+0x84), captured ID (+0x88), and
        // wrapped flag (+0x8C). This guards against a per-version slot shift.
        for ((handler, leaf), call_offset) in handlers[3..]
            .iter()
            .copied()
            .zip(presented_leaves)
            .zip(presented_call_offsets)
        {
            assert_eq!(
                thumb_call_target(&rom, handler + call_offset),
                leaf,
                "{} presented-item native leaf",
                case.name
            );
        }
        for (leaf, virtual_offset) in presented_leaves.into_iter().zip([0x84, 0x88, 0x8C]) {
            assert_eq!(
                &rom[leaf + 0x12..leaf + 0x18],
                &[0x41, 0x69, virtual_offset, 0x31, 0x09, 0x68],
                "{} presented-item virtual offset",
                case.name
            );
        }
    }

    let rom = fs::read(local_rom_path(CASES[0].rom)).unwrap();
    assert_eq!(
        thumb_call_target(&rom, 0x45010 + 0x10),
        0x16DB0,
        "FoMT-US random meal native"
    );
    assert_eq!(
        thumb_call_target(&rom, 0x45028 + 0x0E),
        0x16E7C,
        "FoMT-US prepare scripted animation"
    );
    assert_eq!(
        thumb_call_target(&rom, 0x4503C + 0x10),
        0x16E9C,
        "FoMT-US restore scripted animation"
    );

    // Resolve the player actor vtable itself, rather than assuming that the
    // three virtual offsets happen to mean the same thing in every ROM. The
    // pointed-to leaf getters directly read +0xA8 (kind), +0xAC (ID), and
    // +0xB0 (wrapped) respectively.
    for (case, (vtable, getters)) in CASES.iter().zip([
        (0xE6658, [0x32044, 0x3203C, 0x32034]),
        (0xEEB98, [0x32400, 0x323F8, 0x323F0]),
        (0xE5A98, [0x31DD8, 0x31DD0, 0x31DC8]),
        (0xEE6A8, [0x32274, 0x3226C, 0x32264]),
    ]) {
        let rom = fs::read(local_rom_path(case.rom)).unwrap();
        for ((virtual_offset, getter), expected_body) in
            [0x84, 0x88, 0x8C].into_iter().zip(getters).zip([
                [0xA8, 0x30, 0x00, 0x68, 0x70, 0x47],
                [0xAC, 0x30, 0x00, 0x68, 0x70, 0x47],
                [0xB0, 0x30, 0x00, 0x78, 0x70, 0x47],
            ])
        {
            assert_eq!(
                u32::from_le_bytes(
                    rom[vtable + virtual_offset..vtable + virtual_offset + 4]
                        .try_into()
                        .unwrap()
                ),
                0x0800_0001 + getter as u32,
                "{} presented-item player vtable entry",
                case.name
            );
            assert_eq!(
                &rom[getter..getter + expected_body.len()],
                &expected_body,
                "{} presented-item cached-field getter",
                case.name
            );
        }
    }

    // Random-meal fallback branches construct food IDs 0x3F and 0x40,
    // FOOD_RICE_BALL and FOOD_BREAD in the complete food domain.
    assert_eq!(
        &rom[0x16E34..0x16E44],
        &[
            0x68, 0x46, 0x3F, 0x21, 0xF6, 0xF7, 0x36, 0xFF, 0x08, 0xE0, 0x68, 0x46, 0x40, 0x21,
            0xF6, 0xF7
        ]
    );
}

#[test]
fn native_receive_capacity_basket_and_rucksack_upgrade_slots_match_all_targets() {
    for (case, (dispatch, first_slot, handlers, can_tool, can_food, can_article)) in
        CASES.iter().zip([
            (
                0x3F904,
                0x6F,
                [
                    0x44050, 0x4407A, 0x440A4, 0x44206, 0x44234, 0x4425C, 0x44274, 0x451BE,
                ],
                0x4E0F8,
                0x4E17C,
                0x4E1D4,
            ),
            (
                0x3FAF0,
                0x70,
                [
                    0x442C2, 0x442EC, 0x44316, 0x4447A, 0x444A8, 0x444C0, 0x444D8, 0x45444,
                ],
                0x506B4,
                0x50738,
                0x50790,
            ),
            (
                0x3F578,
                0x6F,
                [
                    0x43CC4, 0x43CEE, 0x43D18, 0x43E7A, 0x43EA8, 0x43ED0, 0x43EE8, 0x44E32,
                ],
                0x4DF20,
                0x4DFA4,
                0x4DFFC,
            ),
            (
                0x3F84C,
                0x70,
                [
                    0x4401E, 0x44048, 0x44072, 0x441D6, 0x44204, 0x4421C, 0x44234, 0x451A0,
                ],
                0x50410,
                0x50494,
                0x504EC,
            ),
        ])
    {
        let rom = fs::read(local_rom_path(case.rom)).unwrap();
        for (index, handler) in handlers.into_iter().enumerate() {
            let slot = first_slot + index;
            assert_eq!(
                u32::from_le_bytes(
                    rom[dispatch + slot * 4..dispatch + slot * 4 + 4]
                        .try_into()
                        .unwrap()
                ),
                0x0800_0000 + handler as u32,
                "{} receive/basket/upgrade slot {slot:#x}",
                case.name
            );
        }
        assert_eq!(
            thumb_call_target(&rom, handlers[0] + 0x1E),
            can_tool,
            "{} can receive tool",
            case.name
        );
        assert_eq!(
            thumb_call_target(&rom, handlers[1] + 0x1E),
            can_food,
            "{} can receive food",
            case.name
        );
        assert_eq!(
            thumb_call_target(&rom, handlers[2] + 0x1E),
            can_article,
            "{} can receive article",
            case.name
        );
    }

    // FoMT-US tool capacity checks the held ToolStack, then rucksack, then
    // tool chest. A matching held stack is available only below 99.
    let rom = fs::read(local_rom_path(CASES[0].rom)).unwrap();
    assert_eq!(
        thumb_call_target(&rom, 0x4E0F8 + 0x14),
        0x0DC34,
        "held tool empty query"
    );
    assert_eq!(
        thumb_call_target(&rom, 0x4E0F8 + 0x50),
        0x0F990,
        "rucksack tool capacity"
    );
    assert_eq!(
        thumb_call_target(&rom, 0x4E0F8 + 0x62),
        0x0B410,
        "tool chest capacity"
    );
    assert_eq!(
        &rom[0x4E132..0x4E136],
        &[0x63, 0x21, 0x48, 0x40],
        "held tool stack compares against 99"
    );
}

#[test]
fn native_npc_relationship_and_love_slots_match_all_targets() {
    for (case, (dispatch, first_slot, handlers, leaves)) in CASES.iter().zip([
        (
            0x3F904,
            0x7B,
            [
                0x41B5C, 0x41BCC, 0x41C10, 0x41C64, 0x41CB8, 0x41CFC, 0x41D38, 0x41D80, 0x41DC8,
                0x41E10, 0x41E4C, 0x41E94, 0x41ED8, 0x41F2C,
            ],
            [
                0x9FE74, 0x9E33C, 0x9E370, 0x9E398, 0x9E340, 0x9E39C, 0x9E350, 0x9E358, 0x9E360,
                0x9E3CC, 0x9E348, 0x9E498, 0x9E4C4, 0x9E4F4,
            ],
        ),
        (
            0x3FAF0,
            0x7E,
            [
                0x41DA4, 0x41E18, 0x41E5C, 0x41EB0, 0x41F04, 0x41F48, 0x41F84, 0x41FCC, 0x42014,
                0x4205C, 0x42098, 0x420E0, 0x42124, 0x42178,
            ],
            [
                0xA4FD4, 0xA34D0, 0xA3504, 0xA352C, 0xA34D4, 0xA3530, 0xA34E4, 0xA34EC, 0xA34F4,
                0xA3560, 0xA34DC, 0xA362C, 0xA3658, 0xA3688,
            ],
        ),
        (
            0x3F578,
            0x7B,
            [
                0x417D0, 0x41840, 0x41884, 0x418D8, 0x4192C, 0x41970, 0x419AC, 0x419F4, 0x41A3C,
                0x41A84, 0x41AC0, 0x41B08, 0x41B4C, 0x41BA0,
            ],
            [
                0x9F8AC, 0x9DD74, 0x9DDA8, 0x9DDD0, 0x9DD78, 0x9DDD4, 0x9DD88, 0x9DD90, 0x9DD98,
                0x9DE04, 0x9DD80, 0x9DED0, 0x9DEFC, 0x9DF2C,
            ],
        ),
        (
            0x3F84C,
            0x7E,
            [
                0x41B00, 0x41B74, 0x41BB8, 0x41C0C, 0x41C60, 0x41CA4, 0x41CE0, 0x41D28, 0x41D70,
                0x41DB8, 0x41DF4, 0x41E3C, 0x41E80, 0x41ED4,
            ],
            [
                0xA4A14, 0xA2F10, 0xA2F44, 0xA2F6C, 0xA2F14, 0xA2F70, 0xA2F24, 0xA2F2C, 0xA2F34,
                0xA2FA0, 0xA2F1C, 0xA306C, 0xA3098, 0xA30C8,
            ],
        ),
    ]) {
        let rom = fs::read(local_rom_path(case.rom)).unwrap();
        for (index, handler) in handlers.into_iter().enumerate() {
            let slot = first_slot + index;
            assert_eq!(
                u32::from_le_bytes(
                    rom[dispatch + slot * 4..dispatch + slot * 4 + 4]
                        .try_into()
                        .unwrap()
                ),
                0x0800_0000 + handler as u32,
                "{} NPC relationship/love slot {slot:#x}",
                case.name
            );
        }

        let leaf_offsets = [
            0x3C, 0x36, 0x48, 0x48, 0x36, 0x2E, 0x36, 0x36, 0x36, 0x2E, 0x36, 0x36, 0x48, 0x48,
        ];
        for ((handler, leaf), offset) in handlers.into_iter().zip(leaves).zip(leaf_offsets) {
            assert_eq!(
                thumb_call_target(&rom, handler + offset),
                leaf,
                "{} NPC relationship/love native leaf",
                case.name
            );
        }
    }
}

#[test]
fn native_shop_and_gift_wrapping_slots_match_all_targets() {
    for (case, (dispatch, first_slot, handlers, leaves)) in CASES.iter().zip([
        (
            0x3F904,
            0x8B,
            [
                0x41FF4, 0x4200C, 0x4206A, 0x42082, 0x4209A, 0x420B2, 0x420CA, 0x420E2, 0x420FA,
                0x42112, 0x4212A, 0x4203A, 0x42052,
            ],
            [
                0x12BBC, 0x12C50, 0x12CF4, 0x12D88, 0x12E1C, 0x12EC0, 0x12F54, 0x12FE8, 0x13080,
                0x13114, 0x131A8, 0x1323C, 0x132D0,
            ],
        ),
        (
            0x3FAF0,
            0x8E,
            [
                0x42240, 0x42258, 0x422B6, 0x422CE, 0x422E6, 0x422FE, 0x42316, 0x4232E, 0x42346,
                0x4235E, 0x42376, 0x42286, 0x4229E,
            ],
            [
                0x12D48, 0x12DDC, 0x12E80, 0x12F14, 0x12FA8, 0x1304C, 0x130E0, 0x13174, 0x1320C,
                0x132A0, 0x13334, 0x133C8, 0x1345C,
            ],
        ),
        (
            0x3F578,
            0x8B,
            [
                0x41C68, 0x41C80, 0x41CDE, 0x41CF6, 0x41D0E, 0x41D26, 0x41D3E, 0x41D56, 0x41D6E,
                0x41D86, 0x41D9E, 0x41CAE, 0x41CC6,
            ],
            [
                0x12A8C, 0x12B20, 0x12BC4, 0x12C58, 0x12CEC, 0x12D90, 0x12E24, 0x12EB8, 0x12F50,
                0x12FE4, 0x13078, 0x13110, 0x131A4,
            ],
        ),
        (
            0x3F84C,
            0x8E,
            [
                0x41F9C, 0x41FB4, 0x42012, 0x4202A, 0x42042, 0x4205A, 0x42072, 0x4208A, 0x420A2,
                0x420BA, 0x420D2, 0x41FE2, 0x41FFA,
            ],
            [
                0x12BEC, 0x12C80, 0x12D24, 0x12DB8, 0x12E4C, 0x12EF0, 0x12F84, 0x13018, 0x130B0,
                0x13144, 0x131D8, 0x13270, 0x13304,
            ],
        ),
    ]) {
        let rom = fs::read(local_rom_path(case.rom)).unwrap();
        for (index, handler) in handlers.into_iter().enumerate() {
            let slot = first_slot + index;
            assert_eq!(
                u32::from_le_bytes(
                    rom[dispatch + slot * 4..dispatch + slot * 4 + 4]
                        .try_into()
                        .unwrap()
                ),
                0x0800_0000 + handler as u32,
                "{} shop/gift-wrapping slot {slot:#x}",
                case.name
            );
        }

        for (index, (handler, leaf)) in handlers.into_iter().zip(leaves).enumerate() {
            let offset = if index == 1 { 0x26 } else { 0x10 };
            assert_eq!(
                thumb_call_target(&rom, handler + offset),
                leaf,
                "{} shop/gift-wrapping native constructor {index}",
                case.name
            );
        }
    }
}

#[test]
fn native_reference_and_farmhouse_ui_slots_match_all_targets() {
    for (case, (dispatch, first_slot, handlers, leaves)) in CASES.iter().zip([
        (
            0x3F904,
            0x98,
            [
                0x42142, 0x42170, 0x42188, 0x421A0, 0x421B8, 0x421D0, 0x421E8, 0x42200, 0x424F0,
                0x42508,
            ],
            [
                0x13364, 0x13408, 0x134A4, 0x13544, 0x135D8, 0x1366C, 0x13700, 0x13794, 0x13828,
                0x138C0,
            ],
        ),
        (
            0x3FAF0,
            0x9B,
            [
                0x4238E, 0x423BC, 0x423D4, 0x423EC, 0x42404, 0x4241C, 0x42434, 0x4244C, 0x42736,
                0x4274E,
            ],
            [
                0x134F0, 0x13594, 0x13630, 0x136D0, 0x13764, 0x137F8, 0x1388C, 0x13920, 0x139B4,
                0x13A4C,
            ],
        ),
        (
            0x3F578,
            0x98,
            [
                0x41DB6, 0x41DE4, 0x41DFC, 0x41E14, 0x41E2C, 0x41E44, 0x41E5C, 0x41E74, 0x42164,
                0x4217C,
            ],
            [
                0x13238, 0x132DC, 0x13378, 0x13418, 0x134AC, 0x13540, 0x135D4, 0x13668, 0x136FC,
                0x13794,
            ],
        ),
        (
            0x3F84C,
            0x9B,
            [
                0x420EA, 0x42118, 0x42130, 0x42148, 0x42160, 0x42178, 0x42190, 0x421A8, 0x42492,
                0x424AA,
            ],
            [
                0x13398, 0x1343C, 0x134D8, 0x13578, 0x1360C, 0x136A0, 0x13734, 0x137C8, 0x1385C,
                0x138F4,
            ],
        ),
    ]) {
        let rom = fs::read(local_rom_path(case.rom)).unwrap();
        for (index, handler) in handlers.into_iter().enumerate() {
            let slot = first_slot + index;
            assert_eq!(
                u32::from_le_bytes(
                    rom[dispatch + slot * 4..dispatch + slot * 4 + 4]
                        .try_into()
                        .unwrap()
                ),
                0x0800_0000 + handler as u32,
                "{} reference/farmhouse UI slot {slot:#x}",
                case.name
            );
        }

        for (index, (handler, leaf)) in handlers.into_iter().zip(leaves).enumerate() {
            let offset = if index == 0 { 0x26 } else { 0x10 };
            assert_eq!(
                thumb_call_target(&rom, handler + offset),
                leaf,
                "{} reference/farmhouse UI native constructor {index}",
                case.name
            );
        }
    }
}

#[test]
fn native_link_name_entry_and_opening_ui_slots_match_all_targets() {
    for (case, (dispatch, first_slot, handlers, constructors, flashback_selector)) in
        CASES.iter().zip([
            (
                0x3F904,
                0xA2,
                [0x42520, 0x43532, 0x42538, 0x42550],
                [0x1395C, 0x13F88, 0x13BDC, 0x13C70],
                0x29u8,
            ),
            (
                0x3FAF0,
                0xA5,
                [0x42766, 0x43782, 0x4277E, 0x42796],
                [0x13AE8, 0x14114, 0x13D68, 0x13DFC],
                0x29,
            ),
            (
                0x3F578,
                0xA2,
                [0x42194, 0x431A6, 0x421AC, 0x421C4],
                [0x13830, 0x13E5C, 0x13AB0, 0x13B44],
                0x28,
            ),
            (
                0x3F84C,
                0xA5,
                [0x424C2, 0x434DE, 0x424DA, 0x424F2],
                [0x13990, 0x13FBC, 0x13C10, 0x13CA4],
                0x28,
            ),
        ])
    {
        let rom = fs::read(local_rom_path(case.rom)).unwrap();
        for (index, handler) in handlers.into_iter().enumerate() {
            let slot = first_slot + index;
            assert_eq!(
                u32::from_le_bytes(
                    rom[dispatch + slot * 4..dispatch + slot * 4 + 4]
                        .try_into()
                        .unwrap()
                ),
                0x0800_0000 + handler as u32,
                "{} link/name/opening UI slot {slot:#x}",
                case.name
            );
        }

        for (index, (handler, constructor)) in handlers.into_iter().zip(constructors).enumerate() {
            let offset = if index == 1 { 0x3E } else { 0x10 };
            assert_eq!(
                thumb_call_target(&rom, handler + offset),
                constructor,
                "{} link/name/opening UI constructor {index}",
                case.name
            );
        }

        let flashback_handler = handlers[2];
        let flashback_constructor = constructors[2];
        // The callable is exactly one 24-byte, parameterless blocking stage;
        // the immediately following block is a distinct dispatch-table slot.
        assert_eq!(
            handlers[3],
            flashback_handler + 0x18,
            "{} slot boundary",
            case.name
        );
        assert_eq!(
            &rom[flashback_handler..flashback_handler + 0x0C],
            &[0xD5, 0x24, 0xA4, 0x00, 0x28, 0x19, 0x00, 0x68, 0x00, 0x28, 0x01, 0xD1],
            "{} parameterless flashback-state check",
            case.name
        );
        assert_eq!(
            &rom[flashback_constructor..flashback_constructor + 0x0C],
            &[
                0xF0,
                0xB5,
                0x87,
                0xB0,
                0x46,
                0x68,
                flashback_selector,
                0x20,
                0x00,
                0x90,
                0x14,
                0x20,
            ],
            "{} regional flashback-state selector",
            case.name
        );
    }
}

#[test]
fn native_modal_menu_and_transition_slots_match_all_targets() {
    for (case, (dispatch, first_slot, handlers, leaves)) in CASES.iter().zip([
        (
            0x3F904,
            0xA6,
            [
                0x42568, 0x4259C, 0x425CA, 0x425E2, 0x42218, 0x42230, 0x42248, 0x42260, 0x42278,
                0x42290,
            ],
            [
                0x13D04, 0x13DC8, 0x13E6C, 0x13EF8, 0x14034, 0x1404C, 0x14064, 0x1407C, 0x14094,
                0x140AC,
            ],
        ),
        (
            0x3FAF0,
            0xA9,
            [
                0x427AE, 0x427E2, 0x42810, 0x42828, 0x42464, 0x4247C, 0x42494, 0x424AC, 0x424C4,
                0x424DC,
            ],
            [
                0x13E90, 0x13F54, 0x13FF8, 0x14084, 0x141C0, 0x141D8, 0x141F0, 0x14208, 0x14220,
                0x14238,
            ],
        ),
        (
            0x3F578,
            0xA6,
            [
                0x421DC, 0x42210, 0x4223E, 0x42256, 0x41E8C, 0x41EA4, 0x41EBC, 0x41ED4, 0x41EEC,
                0x41F04,
            ],
            [
                0x13BD8, 0x13C9C, 0x13D40, 0x13DCC, 0x13F08, 0x13F20, 0x13F38, 0x13F50, 0x13F68,
                0x13F80,
            ],
        ),
        (
            0x3F84C,
            0xA9,
            [
                0x4250A, 0x4253E, 0x4256C, 0x42584, 0x421C0, 0x421D8, 0x421F0, 0x42208, 0x42220,
                0x42238,
            ],
            [
                0x13D38, 0x13DFC, 0x13EA0, 0x13F2C, 0x14068, 0x14080, 0x14098, 0x140B0, 0x140C8,
                0x140E0,
            ],
        ),
    ]) {
        let rom = fs::read(local_rom_path(case.rom)).unwrap();
        for (index, handler) in handlers.into_iter().enumerate() {
            let slot = first_slot + index;
            assert_eq!(
                u32::from_le_bytes(
                    rom[dispatch + slot * 4..dispatch + slot * 4 + 4]
                        .try_into()
                        .unwrap()
                ),
                0x0800_0000 + handler as u32,
                "{} modal-menu/transition slot {slot:#x}",
                case.name
            );
        }

        let offsets = [0x2C, 0x26, 0x10, 0x26, 0x10, 0x10, 0x10, 0x10, 0x10, 0x10];
        for ((handler, leaf), offset) in handlers.into_iter().zip(leaves).zip(offsets) {
            assert_eq!(
                thumb_call_target(&rom, handler + offset),
                leaf,
                "{} modal-menu/transition native leaf",
                case.name
            );
        }

        for (leaf, callback_offset) in leaves
            .into_iter()
            .skip(4)
            .zip([0x84, 0x88, 0x8C, 0x90, 0x94, 0x98])
        {
            assert_eq!(
                &rom[leaf + 0x0A..leaf + 0x0C],
                &[callback_offset, 0x31],
                "{} modal transition virtual callback {callback_offset:#x}",
                case.name
            );
        }
    }
}

#[test]
fn native_record_player_fireplace_and_vase_slots_match_all_targets() {
    for (case, (dispatch, first_slot, handlers, leaves, offsets)) in CASES.iter().zip([
        (
            0x3F904,
            0xB0,
            [
                0x42436, 0x4245E, 0x424CE, 0x43648, 0x43676, 0x43718, 0x43746,
            ],
            [
                0x0BB74, 0x1424C, 0x14234, 0x14694, 0x0BFC4, 0x146B0, 0x0BFA4,
            ],
            [0x12, 0x28, 0x0E, 0x26, 0x52, 0x26, 0x0E],
        ),
        (
            0x3FAF0,
            0xB3,
            [
                0x42682, 0x426AA, 0x4271A, 0x438A4, 0x438D2, 0x4397C, 0x439AA,
            ],
            [
                0x0BBE4, 0x143D8, 0x143C0, 0x14820, 0x0C038, 0x1483C, 0x0C018,
            ],
            [0x12, 0x28, 0x12, 0x26, 0x58, 0x26, 0x0E],
        ),
        (
            0x3F578,
            0xB0,
            [
                0x420AA, 0x420D2, 0x42142, 0x432BC, 0x432EA, 0x4338C, 0x433BA,
            ],
            [
                0x0BB54, 0x14120, 0x14108, 0x14568, 0x0BFA4, 0x14584, 0x0BF84,
            ],
            [0x12, 0x28, 0x0E, 0x26, 0x52, 0x26, 0x0E],
        ),
        (
            0x3F84C,
            0xB3,
            [
                0x423DE, 0x42406, 0x42476, 0x43600, 0x4362E, 0x436D8, 0x43706,
            ],
            [
                0x0BB98, 0x14280, 0x14268, 0x146C8, 0x0BFEC, 0x146E4, 0x0BFCC,
            ],
            [0x12, 0x28, 0x12, 0x26, 0x58, 0x26, 0x0E],
        ),
    ]) {
        let rom = fs::read(local_rom_path(case.rom)).unwrap();
        for (index, handler) in handlers.into_iter().enumerate() {
            let slot = first_slot + index;
            assert_eq!(
                u32::from_le_bytes(
                    rom[dispatch + slot * 4..dispatch + slot * 4 + 4]
                        .try_into()
                        .unwrap()
                ),
                0x0800_0000 + handler as u32,
                "{} record-player/fireplace/vase slot {slot:#x}",
                case.name
            );
        }
        for ((handler, leaf), offset) in handlers.into_iter().zip(leaves).zip(offsets) {
            assert_eq!(
                thumb_call_target(&rom, handler + offset),
                leaf,
                "{} record-player/fireplace/vase native leaf",
                case.name
            );
        }

        // RecordPlayer::HasAlbum is an exact one-bit read in every target.
        // Pinning the complete leaf prevents this slot from drifting into the
        // separate Van-shop unlocked/available album state.
        assert_eq!(
            &rom[leaves[0]..leaves[0] + 8],
            &[0x00, 0x68, 0xC0, 0x07, 0xC0, 0x0F, 0x70, 0x47],
            "{} inserted record-player album bit",
            case.name
        );

        let fireplace_query = handlers[4];
        for (offset, map_id) in [(0x20, 0x1D), (0x28, 0x12), (0x2E, 0x27)] {
            assert_eq!(
                u16::from_le_bytes(
                    rom[fireplace_query + offset..fireplace_query + offset + 2]
                        .try_into()
                        .unwrap()
                ),
                0x2C00 | map_id,
                "{} fireplace query map selector {map_id:#x}",
                case.name
            );
        }
        assert_eq!(
            u16::from_le_bytes(
                rom[fireplace_query + 0x40..fireplace_query + 0x42]
                    .try_into()
                    .unwrap()
            ),
            0x2100,
            "{} unsupported fireplace map returns false",
            case.name
        );
    }
}

#[test]
fn native_name_entry_preserves_kind_and_target_byte_on_all_targets() {
    fn call_target(rom: &[u8], pc: usize) -> usize {
        let hi = u16::from_le_bytes(rom[pc..pc + 2].try_into().unwrap());
        let lo = u16::from_le_bytes(rom[pc + 2..pc + 4].try_into().unwrap());
        assert_eq!(hi & 0xF800, 0xF000);
        assert_eq!(lo & 0xF800, 0xF800);
        let raw = (((hi & 0x7FF) as i32) << 12) | (((lo & 0x7FF) as i32) << 1);
        (pc as i32 + 4 + ((raw << 9) >> 9)) as usize
    }

    for (case, (table, slot, handler, native)) in CASES.iter().zip([
        (0x3F904, 0xA3, 0x43532, 0x13F88),
        (0x3FAF0, 0xA6, 0x43782, 0x14114),
        (0x3F578, 0xA3, 0x431A6, 0x13E5C),
        (0x3F84C, 0xA6, 0x434DE, 0x13FBC),
    ]) {
        let rom = fs::read(local_rom_path(case.rom)).unwrap();
        assert_eq!(
            u32::from_le_bytes(
                rom[table + slot * 4..table + slot * 4 + 4]
                    .try_into()
                    .unwrap()
            ),
            0x08000000u32 + handler as u32,
            "{} physical name-entry callable",
            case.name
        );
        assert_eq!(
            call_target(&rom, handler + 0x3E),
            native,
            "{} native name-entry constructor",
            case.name
        );
        // kind is preserved as a word at task+0x0C; target_index is always
        // truncated and stored as a byte at task+0x10. Construction does not
        // discard the target merely because a singleton naming kind is used.
        assert_eq!(
            &rom[native + 0x58..native + 0x60],
            &[0x49, 0x46, 0xC1, 0x60, 0x43, 0x46, 0x03, 0x74],
            "{} name-entry operand stores",
            case.name
        );
    }
}

#[test]
fn native_rucksack_additions_use_unsigned_request_and_return_remainder() {
    fn call_target(rom: &[u8], pc: usize) -> usize {
        let hi = u16::from_le_bytes(rom[pc..pc + 2].try_into().unwrap());
        let lo = u16::from_le_bytes(rom[pc + 2..pc + 4].try_into().unwrap());
        assert_eq!(hi & 0xF800, 0xF000);
        assert_eq!(lo & 0xF800, 0xF800);
        let raw = (((hi & 0x7FF) as i32) << 12) | (((lo & 0x7FF) as i32) << 1);
        (pc as i32 + 4 + ((raw << 9) >> 9)) as usize
    }

    for (case, (table, first_slot, handlers, natives)) in CASES.iter().zip([
        (
            0x3F904,
            0x57,
            [0x4196C, 0x419B4, 0x419FC],
            [0x0FDC4, 0x0FD50, 0x0FEC8],
        ),
        (
            0x3FAF0,
            0x58,
            [0x41BB4, 0x41BFC, 0x41C44],
            [0x0FEA0, 0x0FE2C, 0x0FFA4],
        ),
        (
            0x3F578,
            0x57,
            [0x415E0, 0x41628, 0x41670],
            [0x0FDA4, 0x0FD30, 0x0FEA8],
        ),
        (
            0x3F84C,
            0x58,
            [0x41910, 0x41958, 0x419A0],
            [0x0FE54, 0x0FDE0, 0x0FF58],
        ),
    ]) {
        let rom = fs::read(local_rom_path(case.rom)).unwrap();
        for index in 0..3 {
            assert_eq!(
                u32::from_le_bytes(
                    rom[table + (first_slot + index) * 4..table + (first_slot + index) * 4 + 4]
                        .try_into()
                        .unwrap()
                ),
                0x08000000u32 + handlers[index] as u32,
                "{} physical rucksack-add callable {}",
                case.name,
                index
            );
            assert_eq!(
                call_target(&rom, handlers[index] + 0x3A),
                natives[index],
                "{} native rucksack-add target {}",
                case.name,
                index
            );
        }

        // Article and food additions retain requested_count in r5, decrement it
        // once per empty item slot, test it only against zero, and return r5.
        for (native, loop_test, decrement, result) in [
            (natives[0], 0x40, 0x3C, 0x48),
            (natives[1], 0x5C, 0x58, 0x64),
        ] {
            assert_eq!(
                &rom[native + decrement..native + decrement + 2],
                &[0x01, 0x3D]
            );
            assert_eq!(
                &rom[native + loop_test..native + loop_test + 2],
                &[0x00, 0x2D]
            );
            assert_eq!(&rom[native + result..native + result + 2], &[0x28, 0x1C]);
        }

        // Tool addition stores requested_count as a word, uses the unsigned
        // `bls` branch when selecting min(request, 99), subtracts the amount
        // inserted, and returns the remaining word unchanged on exit.
        let tool = natives[2];
        assert_eq!(&rom[tool + 0x08..tool + 0x0A], &[0x01, 0x92]);
        assert_eq!(&rom[tool + 0x20..tool + 0x24], &[0x01, 0x98, 0x00, 0x28]);
        assert_eq!(
            &rom[tool + 0x36..tool + 0x42],
            &[0x63, 0x20, 0x02, 0x90, 0x02, 0xA9, 0x01, 0x98, 0x01, 0xAC, 0x63, 0x28]
        );
        assert_eq!(rom[tool + 0x43], 0xD9, "{} unsigned min branch", case.name);
        assert_eq!(&rom[tool + 0xA8..tool + 0xAC], &[0x01, 0x98, 0x00, 0x1B]);
        assert_eq!(&rom[tool + 0xBA..tool + 0xBC], &[0x05, 0xB0]);
    }
}

#[test]
fn native_random_range_uses_signed_inclusive_width_on_all_targets() {
    fn call_target(rom: &[u8], pc: usize) -> usize {
        let hi = u16::from_le_bytes(rom[pc..pc + 2].try_into().unwrap());
        let lo = u16::from_le_bytes(rom[pc + 2..pc + 4].try_into().unwrap());
        assert_eq!(hi & 0xF800, 0xF000);
        assert_eq!(lo & 0xF800, 0xF800);
        let raw = (((hi & 0x7FF) as i32) << 12) | (((lo & 0x7FF) as i32) << 1);
        (pc as i32 + 4 + ((raw << 9) >> 9)) as usize
    }
    for (case, offset) in CASES.iter().zip([0x4110C, 0x41338, 0x40D80, 0x41094]) {
        let rom = fs::read(local_rom_path(case.rom)).unwrap();
        let random = call_target(&rom, offset + 0x32);
        // Complete native rand with state pointer, LCG constants and a
        // 31-bit result mask (lsl 1; lsr 1), not a 15-bit sample.
        assert_eq!(
            &rom[random..random + 32],
            &[
                0x04, 0xA0, 0x0E, 0xC8, 0x08, 0x68, 0x50, 0x43, 0xC0, 0x18, 0x08, 0x60, 0x40, 0x00,
                0x40, 0x08, 0x70, 0x47, 0x00, 0x00, 0x50, 0x0A, 0x00, 0x03, 0x6D, 0x4E, 0xC6, 0x41,
                0x39, 0x30, 0x00, 0x00,
            ],
            "{} native random generator",
            case.name
        );
        let remainder = call_target(&rom, offset + 0x3A);
        // Nonzero divisor uses BIOS signed division and returns r1 (remainder).
        assert_eq!(
            &rom[remainder..remainder + 10],
            &[0x00, 0x29, 0xF5, 0xD0, 0x06, 0xDF, 0x08, 0x1C, 0x70, 0x47]
        );
        // The separate RandomU15 callable alone adds a 0x7FFF mask.
        assert_eq!(call_target(&rom, offset - 0x1C), random);
        assert_eq!(
            &rom[offset - 0x18..offset - 0x12],
            &[0x04, 0x1C, 0x04, 0x48, 0x04, 0x40]
        );
        assert_eq!(&rom[offset - 4..offset], &[0xFF, 0x7F, 0x00, 0x00]);
        // Inverted range jumps to +44: check stack capacity, then push zero.
        assert_eq!(
            &rom[offset + 0x44..offset + 0x4A],
            &[0x22, 0x68, 0x63, 0x2A, 0x01, 0xD9]
        );
        assert_eq!(
            &rom[offset + 0x4E..offset + 0x54],
            &[0x90, 0x00, 0x30, 0x18, 0x00, 0x21]
        );
        let push_zero = call_target(&rom, offset + 0x54);
        assert_eq!(
            &rom[push_zero..push_zero + 6],
            &[0x01, 0x60, 0x50, 0x1C, 0x20, 0x60]
        );
        // cmp min,max; bgt invalid-range path. Then sub max,min; add 1;
        // after the signed remainder helper, add min to its result.
        // Relocated calls are deliberately outside these arithmetic checks;
        // the FoMT source identifies them as rand and __modsi3.
        for (relative, expected) in [
            (0x2E, &[0xBD, 0x42, 0x08, 0xDC][..]),
            (0x36, &[0x79, 0x1B, 0x01, 0x31][..]),
            (0x3E, &[0x42, 0x19][..]),
        ] {
            assert_eq!(
                &rom[offset + relative..offset + relative + expected.len()],
                expected,
                "{} random range arithmetic {relative:#x}",
                case.name
            );
        }
    }
}

#[test]
fn native_call_script_uses_target_table_without_runtime_id_bounds_check() {
    for (case, (dispatch, slot, handler, leaf, resolver, parser, runner, table)) in
        CASES.iter().zip([
            (
                0x3F904,
                0x37,
                0x40FBC,
                0x12B04,
                0x3F878,
                0x3EEEC,
                0xD7F3C,
                0x080F89D4u32,
            ),
            (
                0x3FAF0, 0x38, 0x411E8, 0x12C90, 0x3FA64, 0x3F0D8, 0xE0374, 0x081014BC,
            ),
            (
                0x3F578, 0x37, 0x40C30, 0x129D4, 0x3F4EC, 0x3EB60, 0xD76F4, 0x080F8230,
            ),
            (
                0x3F84C, 0x38, 0x40F44, 0x12B34, 0x3F7C0, 0x3EE34, 0xDFE88, 0x0810145C,
            ),
        ])
    {
        let rom = fs::read(local_rom_path(case.rom)).unwrap();
        assert_eq!(
            u32::from_le_bytes(
                rom[dispatch + slot * 4..dispatch + slot * 4 + 4]
                    .try_into()
                    .unwrap()
            ),
            0x0800_0000 + handler as u32,
            "{} CallScript physical slot",
            case.name
        );
        assert_eq!(
            thumb_call_target(&rom, handler + 0x28),
            leaf,
            "{} CallScript leaf",
            case.name
        );
        assert_eq!(
            thumb_call_target(&rom, leaf + 0x0A),
            resolver,
            "{} script-table resolver",
            case.name
        );
        assert_eq!(
            thumb_call_target(&rom, leaf + 0x10),
            runner,
            "{} called-script runner",
            case.name
        );

        // lsl id,2; add table; ldr script pointer. There is deliberately no
        // comparison with the target's script count before this lookup.
        assert_eq!(
            &rom[resolver + 0x08..resolver + 0x10],
            &[0x89, 0x00, 0x09, 0x18, 0x09, 0x68, 0x20, 0x1C],
            "{} unchecked script-table lookup",
            case.name
        );
        assert_eq!(
            u32::from_le_bytes(rom[resolver + 0x24..resolver + 0x28].try_into().unwrap()),
            table,
            "{} target script table",
            case.name
        );
        assert_eq!(
            thumb_call_target(&rom, resolver + 0x10),
            parser,
            "{} RIFF/script parser",
            case.name
        );

        // The runner dispatches through the event controller and then writes
        // state 2 at +0x9C. CallScript is therefore a VM/event transition,
        // not a native C ABI call with arguments or a return value.
        assert_eq!(
            &rom[runner + 0x04..runner + 0x12],
            &[0xA8, 0x30, 0x00, 0x68, 0x01, 0x68, 0x92, 0x22, 0x52, 0x00, 0x89, 0x18, 0x09, 0x68]
        );
        assert_eq!(
            &rom[runner + 0x16..runner + 0x1C],
            &[0x9C, 0x34, 0x02, 0x20, 0x20, 0x60]
        );
    }
}

#[test]
fn native_audio_start_guard_matches_all_targets() {
    // m4aMPlayStart: ident guard, optional priority gate, current-track/start
    // and pause checks, followed by the beginning of an accepted start.
    // Pin the FoMT-US instructions rather than only comparing ROMs to each
    // other. This does not claim that runtime code can never enable the gate.
    let expected: &[u8] = &[
        0xF0, 0xB5, 0x47, 0x46, 0x80, 0xB4, 0x05, 0x1C, 0x0F, 0x1C, 0x69, 0x6B, 0x34, 0x48, 0x81,
        0x42, 0x61, 0xD1, 0xE8, 0x7A, 0xBA, 0x78, 0x00, 0x28, 0x13, 0xD0, 0x28, 0x68, 0x00, 0x28,
        0x05, 0xD0, 0xE9, 0x6A, 0x40, 0x20, 0x09, 0x78, 0x08, 0x40, 0x00, 0x28, 0x05, 0xD1, 0x69,
        0x68, 0xA8, 0x88, 0x00, 0x28, 0x06, 0xD0, 0x00, 0x29, 0x04, 0xDB, 0xB8, 0x78, 0x02, 0x1C,
        0x68, 0x7A, 0x90, 0x42, 0x49, 0xD8, 0x68, 0x6B, 0x01, 0x30, 0x68, 0x63, 0x00, 0x21, 0x69,
        0x60, 0x2F, 0x60, 0x78, 0x68,
    ];
    for (case, offset) in CASES.iter().zip([0xD2A0C, 0xDA634, 0xD21C4, 0xDA148]) {
        let rom = fs::read(local_rom_path(case.rom)).unwrap();
        assert_eq!(
            &rom[offset..offset + expected.len()],
            expected,
            "{} audio start guard",
            case.name
        );
        // The PC-relative literal loaded at +12 is the player's identity tag.
        let literal = ((offset + 12 + 4) & !3) + 0x34 * 4;
        assert_eq!(
            u32::from_le_bytes(rom[literal..literal + 4].try_into().unwrap()),
            0x68736D53,
            "{} audio player identity tag",
            case.name
        );
    }
}

#[test]
fn native_audio_callables_keep_physical_slots_and_player_domains_on_all_targets() {
    for (case, (dispatch, handlers, leaves, leaf_call_offsets)) in CASES.iter().zip([
        (
            0x3F904,
            [0x4041C, 0x40480, 0x40498, 0x404DE, 0x45484],
            [0x123CC, 0x12454, 0x12464, 0x124C4, 0x16784],
            [0x5C, 0x10, 0x3E, 0x10, 0x0C],
        ),
        (
            0x3FAF0,
            [0x40630, 0x40694, 0x406AC, 0x406F2, 0x45754],
            [0x124BC, 0x12544, 0x12554, 0x125B4, 0x16804],
            [0x5C, 0x10, 0x3E, 0x10, 0x0E],
        ),
        (
            0x3F578,
            [0x40090, 0x400F4, 0x4010C, 0x40152, 0x450F8],
            [0x1229C, 0x12324, 0x12334, 0x12394, 0x16518],
            [0x5C, 0x10, 0x3E, 0x10, 0x0C],
        ),
        (
            0x3F84C,
            [0x4038C, 0x403F0, 0x40408, 0x4044E, 0x454B0],
            [0x12360, 0x123E8, 0x123F8, 0x12458, 0x16678],
            [0x5C, 0x10, 0x3E, 0x10, 0x0E],
        ),
    ]) {
        let rom = fs::read(local_rom_path(case.rom)).unwrap();
        for (index, handler) in handlers.into_iter().enumerate() {
            let slot = 0x19 + index;
            assert_eq!(
                u32::from_le_bytes(
                    rom[dispatch + slot * 4..dispatch + slot * 4 + 4]
                        .try_into()
                        .unwrap()
                ),
                0x0800_0000 + handler as u32,
                "{} audio callable slot {slot:#x}",
                case.name
            );
            assert_eq!(
                thumb_call_target(&rom, handler + leaf_call_offsets[index]),
                leaves[index],
                "{} audio callable slot {slot:#x} leaf",
                case.name
            );
        }

        // The dedicated BGM operations load the event controller's +0xBC
        // player. PlaySong instead walks the independent pointer range at
        // +0xD0/+0xD4; StopAllSongs deliberately bypasses both and calls the
        // global five-player-table wrapper.
        assert_eq!(
            &rom[leaves[0] + 4..leaves[0] + 14],
            &[0x44, 0x68, 0x12, 0x04, 0x12, 0x0C, 0x20, 0x1C, 0xBC, 0x30]
        );
        assert_eq!(
            &rom[leaves[1] + 2..leaves[1] + 6],
            &[0x40, 0x68, 0xBC, 0x30]
        );
        assert_eq!(
            &rom[leaves[2] + 6..leaves[2] + 16],
            &[0x40, 0x68, 0xD0, 0x30, 0x04, 0x68, 0x45, 0x68, 0xAC, 0x42]
        );
        assert_eq!(
            &rom[leaves[4] + 2..leaves[4] + 10],
            &[0x40, 0x68, 0x04, 0x1C, 0xBC, 0x34, 0x20, 0x1C]
        );
    }
}

#[test]
fn native_pause_menu_selector_uses_the_same_seven_pages_in_all_four_roms() {
    const ENGLISH: [&[u8]; 7] = [
        b"Diary",
        b"Rucksack",
        b"World Map",
        b"Farm Map",
        b"Earnings",
        b"Memo",
        b"Tutorial",
    ];
    const JAPANESE: [&[u8]; 7] = [
        b"\x93\xFA\x8B\x4C",
        b"\x91\x95\x94\xF5\x81\x45\x8E\x9D\x82\xBF\x95\xA8",
        b"\x91\x53\x91\xCC\x83\x7D\x83\x62\x83\x76",
        b"\x96\x71\x8F\xEA\x83\x7D\x83\x62\x83\x76",
        b"\x89\xC6\x8C\x76\x95\xEB",
        b"\x82\xA2\x82\xEB\x82\xA2\x82\xEB",
        b"\x97\x56\x82\xD1\x95\xFB",
    ];

    for (case, ids_offset) in CASES.iter().zip([0xF19CC, 0xFA1F4, 0xF1228, 0xFA194]) {
        let rom = fs::read(local_rom_path(case.rom)).unwrap();
        let labels = if case.target.ends_with("_US") {
            &ENGLISH
        } else {
            &JAPANESE
        };
        for id in 0..7usize {
            assert_eq!(
                u32::from_le_bytes(
                    rom[ids_offset + id * 4..ids_offset + id * 4 + 4]
                        .try_into()
                        .unwrap()
                ),
                id as u32,
                "{} pause-menu page ID {}",
                case.name,
                id
            );
        }

        let mut label_offset = ids_offset + 0x1C;
        let pointer_offset = ids_offset + 0x68;
        for (id, expected) in labels.iter().enumerate() {
            let pointer = u32::from_le_bytes(
                rom[pointer_offset + id * 4..pointer_offset + id * 4 + 4]
                    .try_into()
                    .unwrap(),
            );
            assert_eq!(
                pointer,
                0x0800_0000 + label_offset as u32,
                "{} pause-menu label pointer {}",
                case.name,
                id
            );
            assert_eq!(
                &rom[label_offset..label_offset + expected.len()],
                *expected,
                "{} pause-menu label {}",
                case.name,
                id
            );
            assert_eq!(
                rom[label_offset + expected.len()],
                0,
                "{} pause-menu label terminator {}",
                case.name,
                id
            );
            label_offset = (label_offset + expected.len() + 1 + 3) & !3;
        }
        assert_eq!(
            label_offset, pointer_offset,
            "{} pause-menu label pool extent",
            case.name
        );
    }
}

#[test]
fn native_audio_player_tables_disable_priority_gate_by_default() {
    for (case, table) in CASES.iter().zip([0x13ABB4, 0x144FB8, 0x13BCF8, 0x146A28]) {
        let rom = fs::read(local_rom_path(case.rom)).unwrap();
        for player in 0..5 {
            let entry = table + player * 12;
            assert_eq!(
                rom[entry + 8],
                8,
                "{} player {player} track capacity",
                case.name
            );
            assert_eq!(
                u16::from_le_bytes(rom[entry + 10..entry + 12].try_into().unwrap()),
                0,
                "{} player {player} initial priority gate",
                case.name
            );
        }
    }
}

#[test]
fn native_audio_fade_update_and_completion_match_all_targets() {
    let reference = fs::read(local_rom_path(CASES[0].rom)).unwrap();
    for (case, offset) in CASES.iter().zip([0xD2B30, 0xDA758, 0xD22E8, 0xDA26C]) {
        let rom = fs::read(local_rom_path(case.rom)).unwrap();
        // Complete FadeOutBody through bx r0, including its local mask
        // literal and TrackStop call. The relative call encoding also agrees.
        // Covers countdown reload, +/-16 packed-volume changes, final track
        // cleanup, temporary-fade distinction, pause and volume propagation.
        // Does not establish how frequently the engine invokes this updater.
        assert_eq!(
            &rom[offset..offset + 0xC8],
            &reference[0xD2B30..0xD2BF8],
            "{} complete fade update",
            case.name
        );
    }
}

#[test]
fn native_bgm_fade_initialization_resets_countdown_and_volume() {
    let reference = fs::read(local_rom_path(CASES[0].rom)).unwrap();
    for (case, offset) in CASES.iter().zip([0xD2234, 0xD9E5C, 0xD19EC, 0xD9970]) {
        let rom = fs::read(local_rom_path(case.rom)).unwrap();
        // Complete MPlayFadeOut including its player-ident guard and literal.
        // Accepted requests write both interval/countdown and initial volume.
        assert_eq!(
            &rom[offset..offset + 32],
            &reference[0xD2234..0xD2254],
            "{} fade initialization",
            case.name
        );
    }
}

#[test]
fn native_bgm_fade_uses_fixed_five_parameter() {
    let reference = fs::read(local_rom_path(CASES[0].rom)).unwrap();
    for (case, offset) in CASES.iter().zip([0x16794, 0x16814, 0x16528, 0x16688]) {
        let rom = fs::read(local_rom_path(case.rom)).unwrap();
        // Status-result test and mov r1,5 immediately before the fade call.
        assert_eq!(
            &rom[offset..offset + 10],
            &reference[0x16794..0x1679E],
            "{} BGM fade request",
            case.name
        );
    }
}

#[test]
fn native_stop_all_songs_reaches_global_five_player_loop() {
    let reference = fs::read(local_rom_path(CASES[0].rom)).unwrap();
    for (case, (handler, wrapper, stop, table)) in CASES.iter().zip([
        (0x124C4, 0x8DE8, 0xD240C, 0x0813ABB4u32),
        (0x125B4, 0x8E50, 0xDA034, 0x08144FB8),
        (0x12394, 0x8DF0, 0xD1BC4, 0x0813BCF8),
        (0x12458, 0x8E04, 0xD9B48, 0x08146A28),
    ]) {
        let rom = fs::read(local_rom_path(case.rom)).unwrap();
        for (entry, target) in [(handler, wrapper), (wrapper, stop)] {
            let pc = entry + 2;
            let hi = u16::from_le_bytes(rom[pc..pc + 2].try_into().unwrap());
            let lo = u16::from_le_bytes(rom[pc + 2..pc + 4].try_into().unwrap());
            assert_eq!(hi & 0xF800, 0xF000);
            assert_eq!(lo & 0xF800, 0xF800);
            let raw = (i32::from(hi & 0x7FF) << 12) | (i32::from(lo & 0x7FF) << 1);
            assert_eq!(
                pc as i64 + 4 + i64::from((raw << 9) >> 9),
                target as i64,
                "{} stop call chain",
                case.name
            );
        }
        // Entire loop and count literal (five), excluding the relocated table.
        assert_eq!(
            &rom[stop..stop + 40],
            &reference[0xD240C..0xD2434],
            "{} global stop loop",
            case.name
        );
        assert_eq!(
            u32::from_le_bytes(rom[stop + 40..stop + 44].try_into().unwrap()),
            table,
            "{} global player table",
            case.name
        );
    }
}

#[test]
fn native_audio_weak_start_checks_identity_and_stop_bit() {
    let reference = fs::read(local_rom_path(CASES[0].rom)).unwrap();
    for (case, offset) in CASES.iter().zip([0x8B88, 0x8BF0, 0x8B90, 0x8BA4]) {
        let rom = fs::read(local_rom_path(case.rom)).unwrap();
        // Compares sequence pointers, then tests the player's status sign bit.
        // No volume parameter or volume write occurs in this decision block.
        assert_eq!(
            &rom[offset..offset + 26],
            &reference[0x8B88..0x8BA2],
            "{} weak audio start decision",
            case.name
        );
    }
}

#[test]
fn native_play_song_pool_falls_back_to_last_player() {
    let reference = fs::read(local_rom_path(CASES[0].rom)).unwrap();
    for (case, offset) in CASES.iter().zip([0x12464, 0x12554, 0x12334, 0x123F8]) {
        let rom = fs::read(local_rom_path(case.rom)).unwrap();
        for relative in 0..0x28 {
            if (0x14..0x18).contains(&relative) {
                continue;
            }
            assert_eq!(
                rom[offset + relative],
                reference[0x12464 + relative],
                "{} audio pool +{relative:#x}",
                case.name
            );
        }
        let pc = offset + 0x14;
        let hi = u16::from_le_bytes(rom[pc..pc + 2].try_into().unwrap());
        let lo = u16::from_le_bytes(rom[pc + 2..pc + 4].try_into().unwrap());
        assert_eq!(hi & 0xF800, 0xF000);
        assert_eq!(lo & 0xF800, 0xF800);
        let raw = (i32::from(hi & 0x7FF) << 12) | (i32::from(lo & 0x7FF) << 1);
        let target = usize::try_from(pc as i64 + 4 + i64::from((raw << 9) >> 9)).unwrap();
        // Complete predicate: returns the inverse of the player's stop bit.
        assert_eq!(
            &rom[target..target + 12],
            &reference[0x8CD0..0x8CDC],
            "{} audio player status",
            case.name
        );
    }
}

#[test]
fn native_play_song_truncates_sequence_to_halfword_on_all_targets() {
    let reference = fs::read(local_rom_path(CASES[0].rom)).unwrap();
    for (case, offset) in CASES.iter().zip([0x1248C, 0x1257C, 0x1235C, 0x12420]) {
        let rom = fs::read(local_rom_path(case.rom)).unwrap();
        // lsl r0,r7,16 / lsr r1,r0,16, followed by start-mode dispatch.
        assert_eq!(
            &rom[offset..offset + 18],
            &reference[0x1248C..0x1249E],
            "{} PlaySong sequence truncation",
            case.name
        );
    }
}

#[test]
fn native_camera_pan_setup_matches_except_relocated_calls() {
    let reference = fs::read(local_rom_path(CASES[0].rom)).unwrap();
    for (case, offset) in CASES.iter().zip([0xA59BC, 0xAABF0, 0xA53F4, 0xAA630]) {
        let rom = fs::read(local_rom_path(case.rom)).unwrap();
        // Resolve each relocated call and compare its complete helper body.
        // sqrt uses BIOS SWI 8; signed division uses BIOS SWI 6; unsigned
        // division contains its own high-bit path and a signed fast path.
        for (relative, reference_target, length) in [
            (0x7E, 0xD3774, 4),
            (0x8A, 0xD0EDA, 0x74),
            (0xA2, 0xD0EC8, 8),
            (0xB6, 0xD0EC8, 8),
        ] {
            let pc = offset + relative;
            let hi = u16::from_le_bytes(rom[pc..pc + 2].try_into().unwrap());
            let lo = u16::from_le_bytes(rom[pc + 2..pc + 4].try_into().unwrap());
            assert_eq!(hi & 0xF800, 0xF000);
            assert_eq!(lo & 0xF800, 0xF800);
            let raw = (i32::from(hi & 0x7FF) << 12) | (i32::from(lo & 0x7FF) << 1);
            let displacement = (raw << 9) >> 9;
            let target = usize::try_from(pc as i64 + 4 + i64::from(displacement)).unwrap();
            assert_eq!(
                &rom[target..target + length],
                &reference[reference_target..reference_target + length],
                "{} camera helper +{relative:#x}",
                case.name
            );
        }
        // These four Thumb BL instructions call sqrt/division helpers. Their
        // destinations relocate; this comparison proves the surrounding
        // setup logic, not the implementations of those helper functions.
        for relative in 0..0xE0 {
            if [0x7E..0x82, 0x8A..0x8E, 0xA2..0xA6, 0xB6..0xBA]
                .iter()
                .any(|range| range.contains(&relative))
            {
                continue;
            }
            assert_eq!(
                rom[offset + relative],
                reference[0xA59BC + relative],
                "{} camera setup +{relative:#x}",
                case.name
            );
        }
    }
}

#[test]
fn native_camera_completion_uses_countdown_on_all_targets() {
    let reference = fs::read(local_rom_path(CASES[0].rom)).unwrap();
    for (case, (step, done)) in CASES.iter().zip([
        (0xA4BEC, 0xA5A9C),
        (0xA9E20, 0xAACD0),
        (0xA4624, 0xA54D4),
        (0xA9860, 0xAA710),
    ]) {
        let rom = fs::read(local_rom_path(case.rom)).unwrap();
        // Adds both fixed-point increments and decrements +0x8C once.
        assert_eq!(
            &rom[step..step + 52],
            &reference[0xA4BEC..0xA4C20],
            "{} camera step",
            case.name
        );
        // Completion query checks only +0x8C == 0, not destination equality.
        assert_eq!(
            &rom[done..done + 20],
            &reference[0xA5A9C..0xA5AB0],
            "{} camera completion",
            case.name
        );
    }
}

#[test]
fn native_camera_center_conversion_and_bounds_match_all_targets() {
    let reference = fs::read(local_rom_path(CASES[0].rom)).unwrap();
    for (case, offset) in CASES.iter().zip([0xA58EC, 0xAAB20, 0xA5324, 0xAA560]) {
        let rom = fs::read(local_rom_path(case.rom)).unwrap();
        // Complete helper: converts desired center to viewport origin, reads
        // dimensions in 8-pixel units, bounds both axes, returns two halfwords.
        // Even the GetMapData relative call is identical on these four ROMs.
        assert_eq!(
            &rom[offset..offset + 0x74],
            &reference[0xA58EC..0xA5960],
            "{} camera center and bounds",
            case.name
        );
    }
}

#[test]
fn native_camera_zero_speed_defaults_to_one_on_all_targets() {
    let reference = fs::read(local_rom_path(CASES[0].rom)).unwrap();
    for (case, offset) in CASES.iter().zip([0xA59DC, 0xAAC10, 0xA5414, 0xAA650]) {
        let rom = fs::read(local_rom_path(case.rom)).unwrap();
        // Includes cmp r7,0 / bne / mov r7,1 followed by coordinate deltas.
        assert_eq!(
            &rom[offset..offset + 40],
            &reference[0xA59DC..0xA5A04],
            "{} camera zero-speed fallback",
            case.name
        );
    }
}

#[test]
fn native_interacting_animal_index_comes_from_player_entity_virtual_state() {
    fn call_target(rom: &[u8], pc: usize) -> usize {
        let hi = u16::from_le_bytes(rom[pc..pc + 2].try_into().unwrap());
        let lo = u16::from_le_bytes(rom[pc + 2..pc + 4].try_into().unwrap());
        assert_eq!(hi & 0xF800, 0xF000);
        assert_eq!(lo & 0xF800, 0xF800);
        let raw = (((hi & 0x7FF) as i32) << 12) | (((lo & 0x7FF) as i32) << 1);
        (pc as i32 + 4 + ((raw << 9) >> 9)) as usize
    }

    let reference = fs::read(local_rom_path(CASES[0].rom)).unwrap();
    for (case, (dispatch, slot, handler, handler_call, leaf)) in CASES.iter().zip([
        (0x3F904, 0x103, 0x44472, 0x12, 0x14D5C),
        (0x3FAF0, 0x106, 0x446E2, 0x0E, 0x14EE8),
        (0x3F578, 0x103, 0x440E6, 0x12, 0x14C30),
        (0x3F84C, 0x106, 0x4443E, 0x0E, 0x14D90),
    ]) {
        let rom = fs::read(local_rom_path(case.rom)).unwrap();
        assert_eq!(
            u32::from_le_bytes(
                rom[dispatch + slot * 4..dispatch + slot * 4 + 4]
                    .try_into()
                    .unwrap()
            ),
            0x08000000 + handler as u32,
            "{} interacting-animal physical slot",
            case.name
        );
        assert_eq!(
            call_target(&rom, handler + handler_call),
            leaf,
            "{} interacting-animal leaf",
            case.name
        );

        // Entity manager lookup: scene +A8, virtual +40, key 0 (player).
        assert_eq!(
            &rom[leaf..leaf + 0x0E],
            &reference[0x14D5C..0x14D6A],
            "{} player lookup",
            case.name
        );
        // The returned player entity is immediately dispatched through
        // virtual +78. No mask, comparison, or clamp touches its result.
        assert_eq!(
            &rom[leaf + 0x12..leaf + 0x16],
            &reference[0x14D6E..0x14D72],
            "{} target getter",
            case.name
        );
    }
}

#[test]
fn native_does_animal_exist_is_a_resolver_presence_check_on_all_targets() {
    fn call_target(rom: &[u8], pc: usize) -> usize {
        let hi = u16::from_le_bytes(rom[pc..pc + 2].try_into().unwrap());
        let lo = u16::from_le_bytes(rom[pc + 2..pc + 4].try_into().unwrap());
        assert_eq!(hi & 0xF800, 0xF000);
        assert_eq!(lo & 0xF800, 0xF800);
        let raw = (((hi & 0x7FF) as i32) << 12) | (((lo & 0x7FF) as i32) << 1);
        (pc as i32 + 4 + ((raw << 9) >> 9)) as usize
    }

    let reference = fs::read(local_rom_path(CASES[0].rom)).unwrap();
    for (case, (dispatch, slot, handler, resolver)) in CASES.iter().zip([
        (0x3F904, 0xFF, 0x442EA, 0x4E3D8),
        (0x3FAF0, 0x102, 0x4455A, 0x50994),
        (0x3F578, 0xFF, 0x43F5E, 0x4E200),
        (0x3F84C, 0x102, 0x442B6, 0x506F0),
    ]) {
        let rom = fs::read(local_rom_path(case.rom)).unwrap();
        assert_eq!(
            u32::from_le_bytes(
                rom[dispatch + slot * 4..dispatch + slot * 4 + 4]
                    .try_into()
                    .unwrap()
            ),
            0x08000000 + handler as u32,
            "{} animal-existence physical slot",
            case.name
        );
        assert_eq!(
            call_target(&rom, handler + 0x30),
            resolver,
            "{} shared animal resolver",
            case.name
        );
        // rsb/or/lsr turns exactly zero into FALSE and every non-null pointer
        // into TRUE; there is no additional life-state field read.
        assert_eq!(
            &rom[handler + 0x34..handler + 0x3A],
            &reference[0x4431E..0x44324],
            "{} pointer-to-boolean normalization",
            case.name
        );
    }
}

#[test]
fn native_animal_daily_condition_and_age_accessors_use_exact_record_fields() {
    const LEAF_BYTES: [&[u8]; 7] = [
        &[0x40, 0x7E, 0xC0, 0x06, 0xC0, 0x0F, 0x70, 0x47],
        &[
            0x00, 0xB5, 0x02, 0x1C, 0x51, 0x7E, 0xC8, 0x06, 0x00, 0x28, 0x02, 0xDB, 0x10, 0x20,
            0x08, 0x43, 0x50, 0x76, 0x01, 0xBC, 0x00, 0x47,
        ],
        &[0x00, 0x8B, 0x80, 0x05, 0x80, 0x0D, 0x70, 0x47],
        &[0x00, 0x7F, 0x40, 0x06, 0xC0, 0x0F, 0x70, 0x47],
        &[0x00, 0x7F, 0xC0, 0x09, 0x70, 0x47],
        &[0x24, 0x30, 0x00, 0x78, 0xC0, 0x07, 0xC0, 0x0F, 0x70, 0x47],
        &[
            0x00, 0xB5, 0x01, 0x1C, 0x24, 0x30, 0x00, 0x78, 0xC0, 0x07, 0x00, 0x28, 0x03, 0xD0,
            0x88, 0x8C, 0x40, 0x05, 0xC0, 0x0E, 0x00, 0xE0, 0x00, 0x20, 0x02, 0xBC,
        ],
    ];

    for (case, (dispatch, first_slot, resolver, handlers, leaves)) in CASES.iter().zip([
        (
            0x3F904,
            0x104,
            0x4E3D8,
            [
                0x4448C, 0x444D6, 0x4451A, 0x445BA, 0x44604, 0x4464E, 0x44698,
            ],
            [
                0x9B238, 0x9B290, 0x9B220, 0x9B504, 0x9B50C, 0x9B8B0, 0x9B8D4,
            ],
        ),
        (
            0x3FAF0,
            0x107,
            0x50994,
            [
                0x44704, 0x4474E, 0x44792, 0x44830, 0x4487A, 0x448C4, 0x4490E,
            ],
            [
                0xA01A4, 0xA01FC, 0xA018C, 0xA0470, 0xA0478, 0xA081C, 0xA0840,
            ],
        ),
        (
            0x3F578,
            0x104,
            0x4E200,
            [
                0x44100, 0x4414A, 0x4418E, 0x4422E, 0x44278, 0x442C2, 0x4430C,
            ],
            [
                0x9AC70, 0x9ACC8, 0x9AC58, 0x9AF3C, 0x9AF44, 0x9B2E8, 0x9B30C,
            ],
        ),
        (
            0x3F84C,
            0x107,
            0x506F0,
            [
                0x44460, 0x444AA, 0x444EE, 0x4458C, 0x445D6, 0x44620, 0x4466A,
            ],
            [
                0x9FBE4, 0x9FC3C, 0x9FBCC, 0x9FEB0, 0x9FEB8, 0xA025C, 0xA0280,
            ],
        ),
    ]) {
        let rom = fs::read(local_rom_path(case.rom)).unwrap();
        // AddAnimalAffection occupies the gap after the talked-to setter. The
        // remaining entries are age, unhappy, sick, pregnant, and healthy
        // pregnancy days in their physical callable-table order.
        for (index, (slot_offset, handler)) in [0usize, 1, 7, 3, 4, 5, 6]
            .into_iter()
            .zip(handlers)
            .enumerate()
        {
            assert_eq!(
                u32::from_le_bytes(
                    rom[dispatch + (first_slot + slot_offset) * 4
                        ..dispatch + (first_slot + slot_offset) * 4 + 4]
                        .try_into()
                        .unwrap()
                ),
                0x08000000 + handler as u32,
                "{} animal state callable {index}",
                case.name
            );
            assert_eq!(
                thumb_call_target(&rom, handler + 0x30),
                resolver,
                "{} guarded animal resolver {index}",
                case.name
            );
            let leaf_call = if index == 1 { 0x3C } else { 0x3E };
            assert_eq!(
                thumb_call_target(&rom, handler + leaf_call),
                leaves[index],
                "{} animal state leaf {index}",
                case.name
            );
            assert_eq!(
                &rom[leaves[index]..leaves[index] + LEAF_BYTES[index].len()],
                LEAF_BYTES[index],
                "{} exact animal state field extractor {index}",
                case.name
            );
        }
    }
}

#[test]
fn native_animal_growth_stage_dispatches_all_five_families_on_all_targets() {
    for (case, (dispatch, slot, handler, table_literal, branches)) in CASES.iter().zip([
        (
            0x3F904,
            0x10D,
            0x44728,
            0x4476C,
            [0x44784, 0x447B4, 0x447D4, 0x447F4, 0x4479E],
        ),
        (
            0x3FAF0,
            0x110,
            0x4499A,
            0x449DC,
            [0x449F4, 0x44A20, 0x44A3E, 0x44A5C, 0x44A0C],
        ),
        (
            0x3F578,
            0x10D,
            0x4439C,
            0x443E0,
            [0x443F8, 0x44428, 0x44448, 0x44468, 0x44412],
        ),
        (
            0x3F84C,
            0x110,
            0x446F6,
            0x44738,
            [0x44750, 0x4477C, 0x4479A, 0x447B8, 0x44768],
        ),
    ]) {
        let rom = fs::read(local_rom_path(case.rom)).unwrap();
        assert_eq!(
            u32::from_le_bytes(
                rom[dispatch + slot * 4..dispatch + slot * 4 + 4]
                    .try_into()
                    .unwrap()
            ),
            0x08000000 + handler as u32,
            "{} animal-growth physical slot",
            case.name
        );

        // The handler accepts only ANIMAL_KIND_HORSE through ANIMAL_KIND_DOG
        // and indexes a five-entry Thumb jump table in that exact order.
        let table = u32::from_le_bytes(rom[table_literal..table_literal + 4].try_into().unwrap())
            as usize
            - 0x08000000;
        for (kind, expected) in branches.into_iter().enumerate() {
            assert_eq!(
                u32::from_le_bytes(
                    rom[table + kind * 4..table + kind * 4 + 4]
                        .try_into()
                        .unwrap()
                ),
                0x08000000 + expected as u32,
                "{} animal-growth branch {kind}",
                case.name
            );
        }
    }
}

#[test]
fn native_is_sheep_sheared_uses_the_guarded_sheep_resolver_on_all_targets() {
    fn call_target(rom: &[u8], pc: usize) -> usize {
        let hi = u16::from_le_bytes(rom[pc..pc + 2].try_into().unwrap());
        let lo = u16::from_le_bytes(rom[pc + 2..pc + 4].try_into().unwrap());
        assert_eq!(hi & 0xF800, 0xF000);
        assert_eq!(lo & 0xF800, 0xF800);
        let raw = (((hi & 0x7FF) as i32) << 12) | (((lo & 0x7FF) as i32) << 1);
        (pc as i32 + 4 + ((raw << 9) >> 9)) as usize
    }

    for (case, (dispatch, slot, handler, resolver_call, resolver, state_call, state_getter)) in
        CASES.iter().zip([
            (0x3F904, 0x10E, 0x44816, 0x20, 0x4E3D8, 0x2E, 0x9BF00),
            (0x3FAF0, 0x111, 0x44A8E, 0x20, 0x50994, 0x2E, 0xA0E6C),
            (0x3F578, 0x10E, 0x4448A, 0x20, 0x4E200, 0x2E, 0x9B938),
            (0x3F84C, 0x111, 0x447EA, 0x20, 0x506F0, 0x2E, 0xA08AC),
        ])
    {
        let rom = fs::read(local_rom_path(case.rom)).unwrap();
        assert_eq!(
            u32::from_le_bytes(
                rom[dispatch + slot * 4..dispatch + slot * 4 + 4]
                    .try_into()
                    .unwrap()
            ),
            0x08000000 + handler as u32,
            "{} sheep-sheared physical slot",
            case.name
        );
        assert_eq!(
            call_target(&rom, handler + resolver_call),
            resolver,
            "{} sheep resolver",
            case.name
        );
        assert_eq!(
            call_target(&rom, handler + state_call),
            state_getter,
            "{} native sheared-state getter",
            case.name
        );
    }
}

#[test]
fn native_count_animals_by_life_state_scans_only_livestock_capacity() {
    fn call_target(rom: &[u8], pc: usize) -> usize {
        let hi = u16::from_le_bytes(rom[pc..pc + 2].try_into().unwrap());
        let lo = u16::from_le_bytes(rom[pc + 2..pc + 4].try_into().unwrap());
        assert_eq!(hi & 0xF800, 0xF000);
        assert_eq!(lo & 0xF800, 0xF800);
        let raw = (((hi & 0x7FF) as i32) << 12) | (((lo & 0x7FF) as i32) << 1);
        (pc as i32 + 4 + ((raw << 9) >> 9)) as usize
    }

    for (case, (dispatch, slot, handler, table_literal, branches, state_calls, state_getter)) in
        CASES.iter().zip([
            (
                0x3F904,
                0x10F,
                0x44912,
                0x44958,
                [0x44A1C, 0x44970, 0x449AA, 0x449E4, 0x44A1C],
                [0x7A, 0xB4, 0xEE],
                0x9B514,
            ),
            (
                0x3FAF0,
                0x112,
                0x44B88,
                0x44BCC,
                [0x44C90, 0x44BE4, 0x44C1E, 0x44C58, 0x44C90],
                [0x78, 0xB2, 0xEC],
                0xA0480,
            ),
            (
                0x3F578,
                0x10F,
                0x44586,
                0x445CC,
                [0x44690, 0x445E4, 0x4461E, 0x44658, 0x44690],
                [0x7A, 0xB4, 0xEE],
                0x9AF4C,
            ),
            (
                0x3F84C,
                0x112,
                0x448E4,
                0x44928,
                [0x449EC, 0x44940, 0x4497A, 0x449B4, 0x449EC],
                [0x78, 0xB2, 0xEC],
                0x9FEC0,
            ),
        ])
    {
        let rom = fs::read(local_rom_path(case.rom)).unwrap();
        assert_eq!(
            u32::from_le_bytes(
                rom[dispatch + slot * 4..dispatch + slot * 4 + 4]
                    .try_into()
                    .unwrap()
            ),
            0x08000000 + handler as u32,
            "{} life-state count physical slot",
            case.name
        );
        let table = u32::from_le_bytes(rom[table_literal..table_literal + 4].try_into().unwrap())
            as usize
            - 0x08000000;
        for (kind, expected) in branches.into_iter().enumerate() {
            assert_eq!(
                u32::from_le_bytes(
                    rom[table + kind * 4..table + kind * 4 + 4]
                        .try_into()
                        .unwrap()
                ),
                0x08000000 + expected as u32,
                "{} life-state branch {kind}",
                case.name
            );
        }
        for call in state_calls {
            assert_eq!(
                call_target(&rom, handler + call),
                state_getter,
                "{} shared livestock life-state getter",
                case.name
            );
        }
    }
}

#[test]
fn native_livestock_death_summary_and_cleanup_slots_match_state_specific_flows() {
    fn call_target(rom: &[u8], pc: usize) -> usize {
        let hi = u16::from_le_bytes(rom[pc..pc + 2].try_into().unwrap());
        let lo = u16::from_le_bytes(rom[pc + 2..pc + 4].try_into().unwrap());
        assert_eq!(hi & 0xF800, 0xF000);
        assert_eq!(lo & 0xF800, 0xF800);
        let raw = (((hi & 0x7FF) as i32) << 12) | (((lo & 0x7FF) as i32) << 1);
        (pc as i32 + 4 + ((raw << 9) >> 9)) as usize
    }

    for (case, (dispatch, first_slot, handlers, leaves)) in CASES.iter().zip([
        (
            0x3F904,
            0x110,
            [0x44A34, 0x44A4C, 0x44A64, 0x44A7C],
            [0x14D9C, 0x15210, 0x1531C, 0x15790],
        ),
        (
            0x3FAF0,
            0x113,
            [0x44CA4, 0x44CBC, 0x44CD4, 0x44CEC],
            [0x14F28, 0x15330, 0x1543C, 0x15810],
        ),
        (
            0x3F578,
            0x110,
            [0x446A8, 0x446C0, 0x446D8, 0x446F0],
            [0x14C70, 0x15044, 0x15150, 0x15524],
        ),
        (
            0x3F84C,
            0x113,
            [0x44A00, 0x44A18, 0x44A30, 0x44A48],
            [0x14DD0, 0x151A4, 0x152B0, 0x15684],
        ),
    ]) {
        let rom = fs::read(local_rom_path(case.rom)).unwrap();
        for (index, (handler, leaf)) in handlers.into_iter().zip(leaves).enumerate() {
            assert_eq!(
                u32::from_le_bytes(
                    rom[dispatch + (first_slot + index) * 4
                        ..dispatch + (first_slot + index) * 4 + 4]
                        .try_into()
                        .unwrap()
                ),
                0x08000000 + handler as u32,
                "{} livestock-death callable slot {:X}",
                case.name,
                first_slot + index
            );
            assert_eq!(
                call_target(&rom, handler + 0x10),
                leaf,
                "{} livestock-death native leaf {index}",
                case.name
            );
        }

        // Cleanup functions 1 and 3 scan both the coop and barn records. At
        // the same two instruction offsets they select life-state 2 (death
        // from neglect) or life-state 1 (natural death), respectively.
        for (leaf_index, state) in [(1, 2u16), (3, 1u16)] {
            let leaf = leaves[leaf_index];
            for offset in [0x3C, 0xD8] {
                assert_eq!(
                    u16::from_le_bytes(rom[leaf + offset..leaf + offset + 2].try_into().unwrap()),
                    0x2800 | state,
                    "{} cleanup leaf {leaf_index} state comparison at +{offset:X}",
                    case.name
                );
            }
        }
    }
}

#[test]
fn native_barn_species_query_and_animal_counts_use_typed_record_presence() {
    fn call_target(rom: &[u8], pc: usize) -> usize {
        let hi = u16::from_le_bytes(rom[pc..pc + 2].try_into().unwrap());
        let lo = u16::from_le_bytes(rom[pc + 2..pc + 4].try_into().unwrap());
        assert_eq!(hi & 0xF800, 0xF000);
        assert_eq!(lo & 0xF800, 0xF800);
        let raw = (((hi & 0x7FF) as i32) << 12) | (((lo & 0x7FF) as i32) << 1);
        (pc as i32 + 4 + ((raw << 9) >> 9)) as usize
    }

    for (case, (dispatch, first_slot, handlers, leaves)) in CASES.iter().zip([
        (
            0x3F904,
            0x114,
            [0x44CAE, 0x441AC, 0x441C2, 0x441E4],
            [0xD3D0, 0xCFC4, 0xCFF4, 0xC630],
        ),
        (
            0x3FAF0,
            0x117,
            [0x44F08, 0x4441E, 0x44440, 0x44462],
            [0xD444, 0xD038, 0xD068, 0xC6A4],
        ),
        (
            0x3F578,
            0x114,
            [0x44922, 0x43E20, 0x43E36, 0x43E58],
            [0xD3B0, 0xCFA4, 0xCFD4, 0xC610],
        ),
        (
            0x3F84C,
            0x117,
            [0x44C64, 0x4417A, 0x4419C, 0x441BE],
            [0xD3F8, 0xCFEC, 0xD01C, 0xC658],
        ),
    ]) {
        let rom = fs::read(local_rom_path(case.rom)).unwrap();
        for (index, (handler, leaf)) in handlers.into_iter().zip(leaves).enumerate() {
            assert_eq!(
                u32::from_le_bytes(
                    rom[dispatch + (first_slot + index) * 4
                        ..dispatch + (first_slot + index) * 4 + 4]
                        .try_into()
                        .unwrap()
                ),
                0x08000000 + handler as u32,
                "{} animal-count group slot {:X}",
                case.name,
                first_slot + index
            );
            assert_eq!(
                call_target(&rom, handler + if index == 0 { 0x2E } else { 0x0E }),
                leaf,
                "{} animal-count group native query {index}",
                case.name
            );
        }
    }
}

#[test]
fn native_contest_animal_state_uses_family_selectors_and_transfer_routines() {
    fn call_target(rom: &[u8], pc: usize) -> usize {
        let hi = u16::from_le_bytes(rom[pc..pc + 2].try_into().unwrap());
        let lo = u16::from_le_bytes(rom[pc + 2..pc + 4].try_into().unwrap());
        assert_eq!(hi & 0xF800, 0xF000);
        assert_eq!(lo & 0xF800, 0xF800);
        let raw = (((hi & 0x7FF) as i32) << 12) | (((lo & 0x7FF) as i32) << 1);
        (pc as i32 + 4 + ((raw << 9) >> 9)) as usize
    }

    for (case, (dispatch, first_slot, handlers, native_routines)) in CASES.iter().zip([
        (
            0x3F904,
            0x118,
            [0x44850, 0x44896, 0x448C4],
            [0x16834, 0x168D4, 0x16AFC],
        ),
        (
            0x3FAF0,
            0x11B,
            [0x44AC8, 0x44B0E, 0x44B3C],
            [0x168B4, 0x16954, 0x16B7C],
        ),
        (
            0x3F578,
            0x118,
            [0x444C4, 0x4450A, 0x44538],
            [0x165C8, 0x16668, 0x16890],
        ),
        (
            0x3F84C,
            0x11B,
            [0x44824, 0x4486A, 0x44898],
            [0x16728, 0x167C8, 0x169F0],
        ),
    ]) {
        let rom = fs::read(local_rom_path(case.rom)).unwrap();
        for (index, (handler, native)) in handlers.into_iter().zip(native_routines).enumerate() {
            assert_eq!(
                u32::from_le_bytes(
                    rom[dispatch + (first_slot + index) * 4
                        ..dispatch + (first_slot + index) * 4 + 4]
                        .try_into()
                        .unwrap()
                ),
                0x08000000 + handler as u32,
                "{} contest-animal slot {:X}",
                case.name,
                first_slot + index
            );
            let call_offset = [0x3E, 0x26, 0x2C][index];
            assert_eq!(
                call_target(&rom, handler + call_offset),
                native,
                "{} contest-animal native routine {index}",
                case.name
            );
        }
    }
}

#[test]
fn native_current_catch_callables_share_one_record_and_exact_fields() {
    fn call_target(rom: &[u8], pc: usize) -> usize {
        let hi = u16::from_le_bytes(rom[pc..pc + 2].try_into().unwrap());
        let lo = u16::from_le_bytes(rom[pc + 2..pc + 4].try_into().unwrap());
        assert_eq!(hi & 0xF800, 0xF000);
        assert_eq!(lo & 0xF800, 0xF800);
        let raw = (((hi & 0x7FF) as i32) << 12) | (((lo & 0x7FF) as i32) << 1);
        (pc as i32 + 4 + ((raw << 9) >> 9)) as usize
    }

    for (case, (dispatch, first_slot, handlers, record_getter, getter_calls, max_field)) in
        CASES.iter().zip([
            (
                0x3F904,
                0x11B,
                [0x44D58, 0x44D90, 0x44DB0, 0x44DD0],
                0x167AC,
                [0x28, 0x0E, 0x0E, 0x0E],
                0x12,
            ),
            (
                0x3FAF0,
                0x11E,
                [0x44FB6, 0x44FEA, 0x4500C, 0x45028],
                0x1682C,
                [0x28, 0x0E, 0x12, 0x0E],
                0x16,
            ),
            (
                0x3F578,
                0x11B,
                [0x449CC, 0x44A04, 0x44A24, 0x44A44],
                0x16540,
                [0x28, 0x0E, 0x0E, 0x0E],
                0x12,
            ),
            (
                0x3F84C,
                0x11E,
                [0x44D12, 0x44D46, 0x44D68, 0x44D84],
                0x166A0,
                [0x28, 0x0E, 0x12, 0x0E],
                0x16,
            ),
        ])
    {
        let rom = fs::read(local_rom_path(case.rom)).unwrap();
        for (index, handler) in handlers.into_iter().enumerate() {
            assert_eq!(
                u32::from_le_bytes(
                    rom[dispatch + (first_slot + index) * 4
                        ..dispatch + (first_slot + index) * 4 + 4]
                        .try_into()
                        .unwrap()
                ),
                0x08000000 + handler as u32,
                "{} current-catch slot {:X}",
                case.name,
                first_slot + index
            );
            assert_eq!(
                call_target(&rom, handler + getter_calls[index]),
                record_getter,
                "{} shared current-catch getter {index}",
                case.name
            );
        }
        assert_eq!(&rom[handlers[1] + 0x12..handlers[1] + 0x14], &[0x84, 0x88]);
        assert_eq!(
            &rom[handlers[2] + max_field..handlers[2] + max_field + 2],
            &[0x04, 0x7D]
        );
        assert_eq!(&rom[handlers[3] + 0x12..handlers[3] + 0x14], &[0x44, 0x7B]);
    }
}

#[test]
fn vanilla_fishing_reward_interprets_current_catch_size_as_centimeters() {
    for (case, script_id, size_call) in [
        (&CASES[0], 976usize, 0x11C),
        (&CASES[1], 1044, 0x11F),
        (&CASES[2], 976, 0x11C),
        (&CASES[3], 1044, 0x11F),
    ] {
        let rom = fs::read(local_rom_path(case.rom)).unwrap();
        let entries = get_script_table(&rom).unwrap();
        let entry = entries
            .into_iter()
            .find(|entry| entry.id() == script_id)
            .unwrap_or_else(|| panic!("{} fishing reward script {script_id}", case.name));
        let ScriptTableEntry::Script { data, backing, .. } = entry else {
            panic!("{} fishing reward slot is null", case.name);
        };
        let script = decode_script_with_backing(data, backing).unwrap();
        assert!(
            script
                .instructions
                .contains(&Ins::Call(mary::ir::CallId(size_call))),
            "{} fishing reward reads current catch size",
            case.name
        );
        assert_eq!(
            script
                .instructions
                .iter()
                .filter(|instruction| **instruction == Ins::PushInt(100))
                .count(),
            2,
            "{} fishing reward splits centimeters by 100",
            case.name
        );
        assert!(
            script.instructions.contains(&Ins::Div),
            "{} meter quotient",
            case.name
        );
        assert!(
            script.instructions.contains(&Ins::Mod),
            "{} centimeter remainder",
            case.name
        );
    }
}

#[test]
fn native_mfomt_speakerless_talk_open_replaces_the_text_window_in_both_regions() {
    for (case, (dispatch, normal_handler, speakerless_handler, normal_leaf, speakerless_leaf)) in [
        (&CASES[1], (0x3FAF0, 0x40722, 0x4073A, 0x125C0, 0x126DC)),
        (&CASES[3], (0x3F84C, 0x4047E, 0x40496, 0x12464, 0x12580)),
    ] {
        let rom = fs::read(local_rom_path(case.rom)).unwrap();
        for (slot, handler, leaf, label) in [
            (0x1F, normal_handler, normal_leaf, "normal"),
            (0x20, speakerless_handler, speakerless_leaf, "speakerless"),
        ] {
            assert_eq!(
                u32::from_le_bytes(
                    rom[dispatch + slot * 4..dispatch + slot * 4 + 4]
                        .try_into()
                        .unwrap()
                ),
                0x08000000 + handler as u32,
                "{} {label} talk-open slot",
                case.name
            );
            assert_eq!(
                thumb_call_target(&rom, handler + 0x10),
                leaf,
                "{} {label} talk-open leaf",
                case.name
            );
        }

        // Both leaves destroy and null the two adjacent UI components at
        // offsets +0xB4 and +0xB8. The speakerless variant then always creates
        // one new text-window component at +0xAC.
        assert_eq!(
            &rom[speakerless_leaf + 0x08..speakerless_leaf + 0x18],
            &[
                0xB4, 0x34, 0x00, 0x26, 0x21, 0x68, 0x8E, 0x42, 0x07, 0xD0, 0x00, 0x29, 0x05, 0xD0,
                0x48, 0x68
            ],
            "{} first attached component",
            case.name
        );
        assert_eq!(
            &rom[speakerless_leaf + 0x26..speakerless_leaf + 0x36],
            &[
                0xB8, 0x34, 0x00, 0x26, 0x21, 0x68, 0x8E, 0x42, 0x07, 0xD0, 0x00, 0x29, 0x05, 0xD0,
                0x48, 0x68
            ],
            "{} second attached component",
            case.name
        );

        let speakerless_creator = thumb_call_target(&rom, speakerless_leaf + 0x6A);
        assert_eq!(
            thumb_call_target(&rom, normal_leaf + 0x7E),
            speakerless_creator,
            "{} normal conditional creator branch one",
            case.name
        );
        assert_eq!(
            thumb_call_target(&rom, normal_leaf + 0xD0),
            speakerless_creator,
            "{} normal conditional creator branch two",
            case.name
        );
        assert_eq!(
            &rom[normal_leaf + 0x50..normal_leaf + 0x56],
            &[0x00, 0x78, 0x00, 0x28, 0x29, 0xD1],
            "{} normal open reuses an existing text window",
            case.name
        );
        assert_ne!(
            &rom[speakerless_leaf + 0x50..speakerless_leaf + 0x56],
            &[0x00, 0x78, 0x00, 0x28, 0x29, 0xD1],
            "{} speakerless open must not reuse the text window",
            case.name
        );
    }
}

#[test]
fn native_talk_window_open_close_and_reset_have_distinct_lifecycles() {
    for (case, (dispatch, reset_leaf, open_leaf, close_slot, close_leaf)) in CASES.iter().zip([
        (0x3F904, 0x125EC, 0x124D0, 0x20, 0x12658),
        (0x3FAF0, 0x12778, 0x125C0, 0x21, 0x127E4),
        (0x3F578, 0x124BC, 0x123A0, 0x20, 0x12528),
        (0x3F84C, 0x1261C, 0x12464, 0x21, 0x12688),
    ]) {
        let rom = fs::read(local_rom_path(case.rom)).unwrap();
        for (slot, leaf, label) in [
            (0x1E, reset_leaf, "reset"),
            (0x1F, open_leaf, "open"),
            (close_slot, close_leaf, "close"),
        ] {
            let handler = (u32::from_le_bytes(
                rom[dispatch + slot * 4..dispatch + slot * 4 + 4]
                    .try_into()
                    .unwrap(),
            ) & !1) as usize
                - 0x08000000;
            assert_eq!(
                thumb_call_target(&rom, handler + 0x10),
                leaf,
                "{} {label} talk-window leaf",
                case.name
            );
        }

        // Reset destroys and nulls the portrait/nameplate attachments at B4/B8,
        // then the text-window object at AC, and clears active-state bit 0x0100.
        assert_eq!(
            &rom[reset_leaf + 0x06..reset_leaf + 0x0A],
            &[0xB4, 0x35, 0x00, 0x26]
        );
        assert_eq!(
            &rom[reset_leaf + 0x24..reset_leaf + 0x28],
            &[0xB8, 0x35, 0x00, 0x26]
        );
        assert_eq!(
            &rom[reset_leaf + 0x42..reset_leaf + 0x46],
            &[0xAC, 0x35, 0x00, 0x26]
        );
        assert_eq!(
            &rom[reset_leaf + 0x68..reset_leaf + 0x6C],
            &[0xFF, 0xFE, 0, 0]
        );

        // Normal open performs the same B4/B8 cleanup, but the AC pointer is
        // tested and reused when non-null rather than unconditionally replaced.
        assert_eq!(
            &rom[open_leaf + 0x08..open_leaf + 0x0C],
            &[0xB4, 0x34, 0x00, 0x25]
        );
        assert_eq!(
            &rom[open_leaf + 0x26..open_leaf + 0x2A],
            &[0xB8, 0x34, 0x00, 0x25]
        );
        assert_eq!(
            &rom[open_leaf + 0x42..open_leaf + 0x4E],
            &[0x38, 0x1C, 0xAC, 0x30, 0x05, 0x68, 0x06, 0x1C, 0x00, 0x2D, 0x55, 0xD1],
            "{} normal open AC reuse branch",
            case.name
        );

        // Close checks AC and schedules four close/cleanup operations, then
        // writes state 0x0F. It contains no AC-pointer nulling store.
        assert_eq!(
            &rom[close_leaf..close_leaf + 0x0E],
            &[0x10, 0xB5, 0x44, 0x68, 0x20, 0x1C, 0xAC, 0x30, 0x00, 0x68, 0x00, 0x28, 0x0E, 0xD0],
            "{} close AC presence check",
            case.name
        );
        assert_eq!(
            &rom[close_leaf + 0x24..close_leaf + 0x2A],
            &[0x21, 0x1C, 0x9C, 0x31, 0x0F, 0x20],
            "{} close state request",
            case.name
        );
    }
}

#[test]
fn native_talk_message_variants_share_a_leaf_but_use_distinct_text_rates() {
    for (case, (dispatch, first_slot, handlers, common_leaf)) in CASES.iter().zip([
        (0x3F904, 0x21, [0x405DC, 0x4053E, 0x4058C], 0x1268C),
        (0x3FAF0, 0x22, [0x40808, 0x4076A, 0x407B8], 0x12818),
        (0x3F578, 0x21, [0x40250, 0x401B2, 0x40200], 0x1255C),
        (0x3F84C, 0x22, [0x40564, 0x404C6, 0x40514], 0x126BC),
    ]) {
        let rom = fs::read(local_rom_path(case.rom)).unwrap();
        for (index, handler) in handlers.into_iter().enumerate() {
            assert_eq!(
                u32::from_le_bytes(
                    rom[dispatch + (first_slot + index) * 4
                        ..dispatch + (first_slot + index) * 4 + 4]
                        .try_into()
                        .unwrap()
                ) & !1,
                0x08000000 + handler as u32,
                "{} message variant slot {}",
                case.name,
                first_slot + index
            );
            let call_offset = if index == 0 { 0x44 } else { 0x42 };
            assert_eq!(
                thumb_call_target(&rom, handler + call_offset),
                common_leaf,
                "{} message variant shared leaf",
                case.name
            );
        }

        // Normal advances text at 0x0100, slow at 0x0040, and the visible
        // continuation variant uses zero for immediate segment completion.
        assert_eq!(
            &rom[handlers[0] + 0x3E..handlers[0] + 0x44],
            &[0x80, 0x22, 0x52, 0x00, 0x20, 0x1C]
        );
        assert_eq!(
            &rom[handlers[1] + 0x3E..handlers[1] + 0x42],
            &[0x10, 0x1C, 0x40, 0x22]
        );
        assert_eq!(
            &rom[handlers[2] + 0x3E..handlers[2] + 0x42],
            &[0x10, 0x1C, 0x00, 0x22]
        );

        // The common leaf stores dialogue state 0x10 after forwarding the
        // selected text pointer and rate to the same text-window object.
        assert_eq!(
            &rom[common_leaf + 0x20..common_leaf + 0x28],
            &[0x21, 0x1C, 0x9C, 0x31, 0x10, 0x20, 0x08, 0x60]
        );
    }
}

#[test]
fn native_talk_choice_families_keep_target_slots_and_distinct_layout_leaves() {
    let handler_lengths = [0xBC, 0xEC, 0x118, 0x90, 0xBC, 0xEC, 0x11C, 0x148];
    for (case, (dispatch, first_slot, handlers, prompt_leaf, choice_leaf)) in CASES.iter().zip([
        (
            0x3F904,
            0x24,
            [
                0x406C0, 0x4077C, 0x40868, 0x40980, 0x40A10, 0x40ACC, 0x40BB8, 0x40CD4,
            ],
            0x12860,
            0x128D0,
        ),
        (
            0x3FAF0,
            0x25,
            [
                0x408EC, 0x409A8, 0x40A94, 0x40BAC, 0x40C3C, 0x40CF8, 0x40DE4, 0x40F00,
            ],
            0x129EC,
            0x12A5C,
        ),
        (
            0x3F578,
            0x24,
            [
                0x40334, 0x403F0, 0x404DC, 0x405F4, 0x40684, 0x40740, 0x4082C, 0x40948,
            ],
            0x12730,
            0x127A0,
        ),
        (
            0x3F84C,
            0x25,
            [
                0x40648, 0x40704, 0x407F0, 0x40908, 0x40998, 0x40A54, 0x40B40, 0x40C5C,
            ],
            0x12890,
            0x12900,
        ),
    ]) {
        let rom = fs::read(local_rom_path(case.rom)).unwrap();
        for (index, (handler, length)) in handlers.into_iter().zip(handler_lengths).enumerate() {
            assert_eq!(
                u32::from_le_bytes(
                    rom[dispatch + (first_slot + index) * 4
                        ..dispatch + (first_slot + index) * 4 + 4]
                        .try_into()
                        .unwrap()
                ) & !1,
                0x08000000 + handler as u32,
                "{} choice-family slot {}",
                case.name,
                first_slot + index
            );
            let expected_leaf = if index < 3 { prompt_leaf } else { choice_leaf };
            let matching_calls = (handler..handler + length)
                .step_by(2)
                .filter(|&pc| {
                    let high = u16::from_le_bytes(rom[pc..pc + 2].try_into().unwrap());
                    let low = u16::from_le_bytes(rom[pc + 2..pc + 4].try_into().unwrap());
                    high & 0xF800 == 0xF000
                        && low & 0xF800 == 0xF800
                        && thumb_call_target(&rom, pc) == expected_leaf
                })
                .count();
            assert_eq!(
                matching_calls,
                1,
                "{} choice-family slot {} layout leaf",
                case.name,
                first_slot + index
            );
        }

        // Prompt+choice layouts and choice-only layouts both request the same
        // asynchronous dialogue state, but use different native constructors.
        assert_eq!(
            &rom[prompt_leaf + 0x60..prompt_leaf + 0x68],
            &[0x21, 0x1C, 0x9C, 0x31, 0x14, 0x20, 0x08, 0x60]
        );
        assert_eq!(
            &rom[choice_leaf + 0x9C..choice_leaf + 0xA4],
            &[0x21, 0x1C, 0x9C, 0x31, 0x14, 0x20, 0x08, 0x60]
        );
    }
}

#[test]
fn native_talk_choice_selection_pipeline_is_one_based_on_all_targets() {
    for (case, (selection_mapper, input_update_call, result_accessor)) in CASES.iter().zip([
        (0x50868usize, 0x4F970usize, 0x50DF0usize),
        (0x52E20, 0x51F28, 0x533A8),
        (0x505F4, 0x4F6FC, 0x50B7C),
        (0x52AE0, 0x51BE8, 0x53068),
    ]) {
        let rom = fs::read(local_rom_path(case.rom)).unwrap();

        // The live menu-input update calls the target's selection mapper and
        // immediately stores its return value into the active-selection word.
        assert_eq!(
            thumb_call_target(&rom, input_update_call),
            selection_mapper,
            "{} choice input mapper",
            case.name
        );
        assert_eq!(
            &rom[input_update_call + 4..input_update_call + 8],
            &[0x20, 0x60, 0xA8, 0x42],
            "{} mapped choice is retained as the active result",
            case.name
        );

        // Every mapper has the same six terminal returns. Their control-flow
        // order is 4, 5, 6, 2, 3, then the default/first entry 1; importantly,
        // there is no successful zero-valued visible entry.
        assert_eq!(
            &rom[selection_mapper + 0x254..selection_mapper + 0x270],
            &[
                0x04, 0x20, 0x08, 0xE0, 0x05, 0x20, 0x06, 0xE0, 0x06, 0x20, 0x04, 0xE0, 0x02, 0x20,
                0x02, 0xE0, 0x03, 0x20, 0x00, 0xE0, 0x01, 0x20, 0x02, 0xBC, 0x08, 0x47, 0x00, 0x00,
            ],
            "{} complete one-based choice-result terminals",
            case.name
        );

        // The UI result adapter rejects only teardown mode 6; otherwise it
        // returns the retained word at object offset 0x158 unchanged.
        assert_eq!(
            &rom[result_accessor..result_accessor + 0x18],
            &[
                0x00, 0xB5, 0x01, 0x68, 0x88, 0x68, 0x06, 0x28, 0x04, 0xD0, 0xAC, 0x22, 0x52, 0x00,
                0x88, 0x18, 0x00, 0x68, 0x00, 0xE0, 0x00, 0x20, 0x02, 0xBC,
            ],
            "{} choice-result accessor",
            case.name
        );
    }
}

#[test]
fn native_talk_nameplate_slots_resolve_character_or_bounded_text_and_clear() {
    for (
        case,
        (
            dispatch,
            character_slot,
            text_slot,
            clear_slot,
            character_handler,
            text_handler,
            clear_handler,
            character_leaf,
            text_leaf,
            clear_leaf,
            text_adapter,
            clear_adapter,
            bounded_renderer,
            gate_offset,
        ),
    ) in CASES.iter().zip([
        (
            0x3F904, 0x2C, 0x2D, 0x2E, 0x4062C, 0x4065A, 0x406A8, 0x126E4, 0x12720, 0x1274C,
            0x50E68, 0x50E74, 0x50B50, 0x2210,
        ),
        (
            0x3FAF0, 0x2D, 0x2E, 0x2F, 0x40858, 0x40886, 0x408D4, 0x12870, 0x128AC, 0x128D8,
            0x53420, 0x5342C, 0x53108, 0x224C,
        ),
        (
            0x3F578, 0x2C, 0x2D, 0x2E, 0x402A0, 0x402CE, 0x4031C, 0x125B4, 0x125F0, 0x1261C,
            0x50BF4, 0x50C00, 0x508DC, 0x2210,
        ),
        (
            0x3F84C, 0x2D, 0x2E, 0x2F, 0x405B4, 0x405E2, 0x40630, 0x12714, 0x12750, 0x1277C,
            0x530E0, 0x530EC, 0x52DC8, 0x224C,
        ),
    ]) {
        let rom = fs::read(local_rom_path(case.rom)).unwrap();
        let pointer = |slot: usize| {
            u32::from_le_bytes(
                rom[dispatch + slot * 4..dispatch + slot * 4 + 4]
                    .try_into()
                    .unwrap(),
            )
        };
        for (slot, handler, label) in [
            (character_slot, character_handler, "character"),
            (text_slot, text_handler, "text"),
            (clear_slot, clear_handler, "clear"),
        ] {
            assert_eq!(
                pointer(slot),
                0x08000000 + handler as u32,
                "{} nameplate {label} slot",
                case.name
            );
        }

        assert_eq!(
            thumb_call_target(&rom, character_handler + 0x26),
            character_leaf,
            "{} character-name leaf",
            case.name
        );
        assert_eq!(
            thumb_call_target(&rom, text_handler + 0x40),
            text_leaf,
            "{} direct-text leaf",
            case.name
        );
        assert_eq!(
            thumb_call_target(&rom, clear_handler + 0x10),
            clear_leaf,
            "{} clear-nameplate leaf",
            case.name
        );

        // The character leaf resolves its numeric character argument first,
        // then forwards the resulting text through the same adapter used by
        // the direct-text leaf.
        assert_eq!(
            thumb_call_target(&rom, character_leaf + 0x2A),
            text_adapter,
            "{} resolved character text adapter",
            case.name
        );
        assert_eq!(
            thumb_call_target(&rom, text_leaf + 0x20),
            text_adapter,
            "{} direct nameplate text adapter",
            case.name
        );

        // Character and direct-text setters both require bit 3 of the same
        // family-specific UI state byte. Explicit clear is deliberately not
        // gated, just as the adjacent portrait-clear operation is not gated.
        assert_eq!(
            &rom[text_leaf..text_leaf + 0x20],
            &[
                0x00, 0xB5, 0x42, 0x68, 0x10, 0x1C, 0xAC, 0x30, 0x03, 0x68, 0x00, 0x2B, 0x0A, 0xD0,
                0x20, 0x38, 0x00, 0x68, 0x05, 0x4A, 0x80, 0x18, 0x00, 0x78, 0x00, 0x07, 0x00, 0x28,
                0x02, 0xDA, 0x18, 0x1C,
            ],
            "{} nameplate object and bit-3 gate",
            case.name
        );
        assert_eq!(
            u32::from_le_bytes(rom[text_leaf + 0x28..text_leaf + 0x2C].try_into().unwrap()),
            gate_offset,
            "{} nameplate gate field",
            case.name
        );
        assert_eq!(
            &rom[clear_leaf..clear_leaf + 0x0C],
            &[0x00, 0xB5, 0x40, 0x68, 0xAC, 0x30, 0x00, 0x68, 0x00, 0x28, 0x01, 0xD0],
            "{} ungated nameplate clear",
            case.name
        );
        assert_eq!(
            thumb_call_target(&rom, clear_leaf + 0x0C),
            clear_adapter,
            "{} nameplate clear adapter",
            case.name
        );
        assert_eq!(
            thumb_call_target(&rom, text_adapter + 0x04),
            bounded_renderer,
            "{} bounded nameplate renderer",
            case.name
        );

        // strlen(text) must be in 1..=12. The renderer subtracts one and
        // compares against 11; null, empty, and longer strings call the same
        // lower-level clear operation reached through ClearTalkNameplate's
        // adapter.
        assert_eq!(
            &rom[bounded_renderer + 0x20..bounded_renderer + 0x26],
            &[0x70, 0x1E, 0x0B, 0x28, 0x03, 0xD9],
            "{} one-through-twelve-byte limit",
            case.name
        );
        assert_eq!(
            thumb_call_target(&rom, bounded_renderer + 0x28),
            thumb_call_target(&rom, clear_adapter + 0x04),
            "{} invalid-length lower-level clear path",
            case.name
        );
    }
}

#[test]
fn native_talk_portrait_slots_forward_id_under_ui_gate_and_clear_independently() {
    for (
        case,
        (
            dispatch,
            set_slot,
            set_handler,
            clear_handler,
            set_leaf,
            clear_leaf,
            set_adapter,
            clear_adapter,
            gate_offset,
        ),
    ) in CASES.iter().zip([
        (
            0x3F904, 0x2F, 0x40E1C, 0x40E4A, 0x12760, 0x1278C, 0x50E50, 0x50E5C, 0x2210,
        ),
        (
            0x3FAF0, 0x30, 0x41048, 0x41076, 0x128EC, 0x12918, 0x53408, 0x53414, 0x224C,
        ),
        (
            0x3F578, 0x2F, 0x40A90, 0x40ABE, 0x12630, 0x1265C, 0x50BDC, 0x50BE8, 0x2210,
        ),
        (
            0x3F84C, 0x30, 0x40DA4, 0x40DD2, 0x12790, 0x127BC, 0x530C8, 0x530D4, 0x224C,
        ),
    ]) {
        let rom = fs::read(local_rom_path(case.rom)).unwrap();
        let pointer = |slot: usize| {
            u32::from_le_bytes(
                rom[dispatch + slot * 4..dispatch + slot * 4 + 4]
                    .try_into()
                    .unwrap(),
            )
        };
        assert_eq!(
            pointer(set_slot),
            0x08000000 + set_handler as u32,
            "{} set-portrait slot",
            case.name
        );
        assert_eq!(
            pointer(set_slot + 1),
            0x08000000 + clear_handler as u32,
            "{} clear-portrait slot",
            case.name
        );
        assert_eq!(
            thumb_call_target(&rom, set_handler + 0x26),
            set_leaf,
            "{} set-portrait leaf",
            case.name
        );
        assert_eq!(
            thumb_call_target(&rom, clear_handler + 0x10),
            clear_leaf,
            "{} clear-portrait leaf",
            case.name
        );

        // Set obtains the +AC talk object, tests bit 2 of a target-family state
        // byte, then forwards the script portrait ID unchanged. This proves a
        // real UI gate without assigning an unverified public meaning to it.
        assert_eq!(
            &rom[set_leaf..set_leaf + 0x20],
            &[
                0x00, 0xB5, 0x42, 0x68, 0x10, 0x1C, 0xAC, 0x30, 0x03, 0x68, 0x00, 0x2B, 0x0A, 0xD0,
                0x20, 0x38, 0x00, 0x68, 0x05, 0x4A, 0x80, 0x18, 0x00, 0x78, 0x40, 0x07, 0x00, 0x28,
                0x02, 0xDA, 0x18, 0x1C,
            ],
            "{} portrait object and bit-2 gate",
            case.name
        );
        assert_eq!(
            u32::from_le_bytes(rom[set_leaf + 0x28..set_leaf + 0x2C].try_into().unwrap()),
            gate_offset,
            "{} portrait gate field",
            case.name
        );
        assert_eq!(
            thumb_call_target(&rom, set_leaf + 0x20),
            set_adapter,
            "{} portrait set adapter",
            case.name
        );

        // Clear uses the same +AC object but has no corresponding gate test or
        // script argument; it reaches a distinct lower-level adapter.
        assert_eq!(
            &rom[clear_leaf..clear_leaf + 0x0C],
            &[0x00, 0xB5, 0x40, 0x68, 0xAC, 0x30, 0x00, 0x68, 0x00, 0x28, 0x01, 0xD0],
            "{} portrait clear object guard",
            case.name
        );
        assert_eq!(
            thumb_call_target(&rom, clear_leaf + 0x0C),
            clear_adapter,
            "{} portrait clear adapter",
            case.name
        );
        assert_ne!(
            set_adapter, clear_adapter,
            "{} set and clear portrait adapters",
            case.name
        );
    }
}

#[test]
fn native_display_option_rows_map_buttons_clock_face_and_name_to_bits_zero_through_three() {
    let layouts = [
        ("fomt-us", "rom/fomt.gba", 0x4904, 0x2210, 0xE86D4, 0xE87A4),
        (
            "mfomt-us",
            "rom/mfomt.gba",
            0x4918,
            0x224C,
            0xF0C14,
            0xF0CE4,
        ),
        (
            "fomt-jp",
            "rom/fomtjp.gba",
            0x491C,
            0x2210,
            0xE7AF4,
            0xE7BD8,
        ),
        (
            "mfomt-jp",
            "rom/mfomtjp.gba",
            0x4930,
            0x224C,
            0xF0714,
            0xF07F8,
        ),
    ];
    let mut normalized_writer = None;
    for (name, path, writer, field, labels, help) in layouts {
        let rom = fs::read(local_rom_path(path)).unwrap();
        let mut body = rom[writer..writer + 0x88].to_vec();
        for literal in [0x1C, 0x38, 0x54, 0x78] {
            assert_eq!(
                u32::from_le_bytes(body[literal..literal + 4].try_into().unwrap()),
                field,
                "{name} option-byte field literal"
            );
            body[literal..literal + 4].fill(0);
        }
        if let Some(expected) = &normalized_writer {
            assert_eq!(&body, expected, "{name} four-row bit writer");
        } else {
            normalized_writer = Some(body);
        }

        if name.ends_with("us") {
            assert_eq!(&rom[labels..labels + 7], b"Buttons", "{name} row 0 label");
            assert_eq!(
                &rom[labels + 0x10..labels + 0x15],
                b"Clock",
                "{name} row 1 label"
            );
            assert_eq!(
                &rom[labels + 0x20..labels + 0x24],
                b"Face",
                "{name} row 2 label"
            );
            assert_eq!(
                &rom[labels + 0x28..labels + 0x2C],
                b"Name",
                "{name} row 3 label"
            );
            assert_eq!(&rom[help..help + 8], b"Controls", "{name} control help");
            assert!(
                rom[help..help + 0xD0]
                    .windows(22)
                    .any(|text| text == b"Show or hide the Clock"),
                "{name} clock help"
            );
        } else {
            assert_eq!(
                &rom[labels..labels + 0x40],
                &[
                    0x88, 0xDA, 0x93, 0xAE, 0x90, 0xDD, 0x92, 0xE8, 0, 0, 0, 0, 0x8E, 0x9E, 0x8C,
                    0x76, 0x82, 0xCC, 0x95, 0x5C, 0x8E, 0xA6, 0, 0, 0x82, 0xA0, 0x82, 0xE8, 0, 0,
                    0, 0, 0x82, 0xC8, 0x82, 0xB5, 0, 0, 0, 0, 0x8A, 0xE7, 0x82, 0xCC, 0x95, 0x5C,
                    0x8E, 0xA6, 0, 0, 0, 0, 0x96, 0xBC, 0x91, 0x4F, 0x82, 0xCC, 0x95, 0x5C, 0x8E,
                    0xA6, 0, 0,
                ],
                "{name} movement/clock/face/name labels"
            );
            assert_eq!(
                &rom[help..help + 4],
                &[0x91, 0x80, 0x8D, 0xEC],
                "{name} control help"
            );
        }
    }
}

#[test]
fn native_talk_heart_indicator_slots_read_love_and_hide_independently() {
    for (
        case,
        (dispatch, show_slot, hide_slot, show_handler, hide_handler, show_leaf, hide_leaf),
    ) in CASES.iter().zip([
        (0x3F904, 0x31, 0x32, 0x40E62, 0x40E90, 0x127A0, 0x1284C),
        (0x3FAF0, 0x32, 0x33, 0x4108E, 0x410BC, 0x1292C, 0x129D8),
        (0x3F578, 0x31, 0x32, 0x40AD6, 0x40B04, 0x12670, 0x1271C),
        (0x3F84C, 0x32, 0x33, 0x40DEA, 0x40E18, 0x127D0, 0x1287C),
    ]) {
        let rom = fs::read(local_rom_path(case.rom)).unwrap();
        let pointer = |slot: usize| {
            u32::from_le_bytes(
                rom[dispatch + slot * 4..dispatch + slot * 4 + 4]
                    .try_into()
                    .unwrap(),
            )
        };
        assert_eq!(
            pointer(show_slot),
            0x08000000 + show_handler as u32,
            "{} show-heart slot",
            case.name
        );
        assert_eq!(
            pointer(hide_slot),
            0x08000000 + hide_handler as u32,
            "{} hide-heart slot",
            case.name
        );

        // The show opcode consumes one script argument before calling the
        // target-specific leaf. The hide opcode consumes none and calls its
        // own no-argument leaf, so it is not a speaker-selection operation.
        assert_eq!(
            &rom[show_handler..show_handler + 0x12],
            &[
                0xDA, 0x24, 0x64, 0x00, 0x2B, 0x19, 0x1A, 0x68, 0x90, 0x00, 0x40, 0x19, 0x01, 0x6A,
                0x00, 0x2A, 0x01, 0xD0,
            ],
            "{} show-heart argument pop",
            case.name
        );
        assert_eq!(
            &rom[hide_handler..hide_handler + 0x0C],
            &[0xD5, 0x21, 0x89, 0x00, 0x68, 0x18, 0x00, 0x68, 0x00, 0x28, 0x01, 0xD1],
            "{} hide-heart no-argument dispatch",
            case.name
        );
        assert_eq!(
            thumb_call_target(&rom, show_handler + 0x26),
            show_leaf,
            "{} show-heart leaf",
            case.name
        );
        assert_eq!(
            thumb_call_target(&rom, hide_handler + 0x10),
            hide_leaf,
            "{} hide-heart leaf",
            case.name
        );

        // The show leaf maps love points into seven display levels. These are
        // the six inclusive upper bounds immediately below 10k..60k.
        for (offset, upper_bound) in [
            (0x48, 59_999u32),
            (0x58, 49_999),
            (0x68, 39_999),
            (0x78, 29_999),
            (0x88, 19_999),
            (0xA8, 9_999),
        ] {
            assert_eq!(
                u32::from_le_bytes(
                    rom[show_leaf + offset..show_leaf + offset + 4]
                        .try_into()
                        .unwrap()
                ),
                upper_bound,
                "{} love-level bound at +{offset:#X}",
                case.name
            );
        }
    }
}

#[test]
fn native_moon_viewing_selector_uses_physical_slots_threshold_and_fallbacks() {
    for (case, (dispatch, slot, handler, duplicate, candidates, fallback_offset, fallback)) in
        CASES.iter().zip([
            (
                0x3F904,
                0x11F,
                0x42E34,
                false,
                [3, 12, 19, 21, 25],
                0x254,
                3,
            ),
            (0x3FAF0, 0x122, 0x43054, true, [26, 7, 2, 20, 23], 0x25C, 2),
            (
                0x3F578,
                0x11F,
                0x42AA8,
                false,
                [3, 12, 19, 21, 25],
                0x254,
                3,
            ),
            (0x3F84C, 0x122, 0x42DB0, true, [26, 7, 2, 20, 23], 0x25C, 2),
        ])
    {
        let rom = fs::read(local_rom_path(case.rom)).unwrap();
        let pointer = |slot: usize| {
            u32::from_le_bytes(
                rom[dispatch + slot * 4..dispatch + slot * 4 + 4]
                    .try_into()
                    .unwrap(),
            )
        };
        assert_eq!(
            pointer(slot),
            0x08000000 + handler as u32,
            "{} selector slot",
            case.name
        );
        if duplicate {
            assert_eq!(
                pointer(slot + 1),
                pointer(slot),
                "{} duplicate selector slot",
                case.name
            );
        }

        assert_eq!(
            u32::from_le_bytes(rom[handler + 0x11C..handler + 0x120].try_into().unwrap()),
            30000,
            "{} strict-love baseline",
            case.name
        );
        let rival_limit_checks = rom[handler..handler + 0x11C]
            .chunks_exact(2)
            .filter(|halfword| **halfword == [0x04, 0x28])
            .count();
        assert_eq!(
            rival_limit_checks, 5,
            "{} rival-count eligibility checks",
            case.name
        );

        let candidate_table = handler + 0x15C;
        assert_eq!(
            u32::from_le_bytes(rom[handler + 0x158..handler + 0x15C].try_into().unwrap()),
            0x08000000 + candidate_table as u32,
            "{} candidate jump table",
            case.name
        );
        for (index, expected_character) in candidates.into_iter().enumerate() {
            let target = u32::from_le_bytes(
                rom[candidate_table + index * 4..candidate_table + index * 4 + 4]
                    .try_into()
                    .unwrap(),
            ) as usize
                - 0x08000000;
            let mov = u16::from_le_bytes(rom[target..target + 2].try_into().unwrap());
            assert_eq!(
                mov & 0xF800,
                0x2000,
                "{} candidate {index} immediate",
                case.name
            );
            assert_eq!(
                (mov & 0xFF) as u8,
                expected_character,
                "{} candidate {index}",
                case.name
            );
        }

        let fallback_mov = u16::from_le_bytes(
            rom[handler + fallback_offset..handler + fallback_offset + 2]
                .try_into()
                .unwrap(),
        );
        assert_eq!(
            fallback_mov & 0xF800,
            0x2000,
            "{} fallback immediate",
            case.name
        );
        assert_eq!(
            (fallback_mov & 0xFF) as u8,
            fallback,
            "{} fallback character",
            case.name
        );

        // After rand(), the selector tests bit 0 of rand() >> 8 and replaces
        // the current candidate only when that bit is zero.
        assert_eq!(
            &rom[handler + 0x132..handler + 0x138],
            &[0x00, 0x12, 0x01, 0x21, 0x08, 0x40],
            "{} equal-score replacement test",
            case.name
        );
    }
}

#[test]
fn native_thomas_stocking_gift_selector_has_the_same_exact_weights_on_all_targets() {
    for (case, (dispatch, slot, handler, table_pointer_offset, table)) in CASES.iter().zip([
        (0x3F904, 0x121, 0x43458, 0x1C, 0xF9ED9),
        (0x3FAF0, 0x125, 0x436A4, 0x28, 0x102B1D),
        (0x3F578, 0x121, 0x430CC, 0x1C, 0xF9735),
        (0x3F84C, 0x125, 0x43400, 0x28, 0x102ABD),
    ]) {
        let rom = fs::read(local_rom_path(case.rom)).unwrap();
        assert_eq!(
            u32::from_le_bytes(
                rom[dispatch + slot * 4..dispatch + slot * 4 + 4]
                    .try_into()
                    .unwrap(),
            ),
            0x08000000 + handler as u32,
            "{} stocking-gift callable slot",
            case.name
        );
        assert_eq!(
            u32::from_le_bytes(
                rom[handler + table_pointer_offset..handler + table_pointer_offset + 4]
                    .try_into()
                    .unwrap(),
            ),
            0x08000000 + table as u32,
            "{} stocking-gift weight pointer",
            case.name
        );
        assert_eq!(
            &rom[table..table + 5],
            &[125, 43, 43, 43, 1],
            "{} stocking-gift weights",
            case.name
        );
        assert_eq!(
            &rom[handler + 4..handler + 12],
            &[0x01, 0x11, 0xFF, 0x20, 0x01, 0x40, 0x00, 0x22],
            "{} uses (rand() >> 4) & 0xFF and starts at entry zero",
            case.name
        );
        assert_eq!(
            &rom[handler + 0x16..handler + 0x18],
            &[0x54, 0x1C],
            "{} returns selected table index plus one",
            case.name
        );
    }
}

#[test]
fn native_random_spouse_gift_returns_the_same_weighted_article_domain_on_all_targets() {
    for (case, (dispatch, slot, handler, table_pointer_offset, table)) in CASES.iter().zip([
        (0x3F904, 0x122, 0x43484, 0x24, 0xF9EE0),
        (0x3FAF0, 0x126, 0x436DC, 0x1C, 0x102B24),
        (0x3F578, 0x122, 0x430F8, 0x24, 0xF973C),
        (0x3F84C, 0x126, 0x43438, 0x1C, 0x102AC4),
    ]) {
        let rom = fs::read(local_rom_path(case.rom)).unwrap();
        assert_eq!(
            u32::from_le_bytes(
                rom[dispatch + slot * 4..dispatch + slot * 4 + 4]
                    .try_into()
                    .unwrap(),
            ),
            0x08000000 + handler as u32,
            "{} spouse-gift callable slot",
            case.name
        );
        assert_eq!(
            u32::from_le_bytes(
                rom[handler + table_pointer_offset..handler + table_pointer_offset + 4]
                    .try_into()
                    .unwrap(),
            ),
            0x08000000 + table as u32,
            "{} spouse-gift table pointer",
            case.name
        );
        let expected = [
            (125u8, 0x24u16),
            (43, 0x20),
            (43, 0x1F),
            (43, 0x1D),
            (1, 0x1A),
        ];
        for (index, (weight, article)) in expected.into_iter().enumerate() {
            let entry = table + index * 4;
            assert_eq!(
                rom[entry], weight,
                "{} spouse-gift weight {index}",
                case.name
            );
            assert_eq!(
                rom[entry + 1],
                0,
                "{} spouse-gift padding {index}",
                case.name
            );
            assert_eq!(
                u16::from_le_bytes(rom[entry + 2..entry + 4].try_into().unwrap()),
                article,
                "{} spouse-gift article {index}",
                case.name
            );
        }
        assert_eq!(
            &rom[handler + 4..handler + 12],
            &[0x01, 0x11, 0xFF, 0x20, 0x01, 0x40, 0x00, 0x23],
            "{} uses (rand() >> 4) & 0xFF and starts at entry zero",
            case.name
        );
    }
}

#[test]
fn native_van_album_unlock_and_availability_share_the_exact_state_layout() {
    let unlock_body = [
        0x00, 0xB5, 0x01, 0x1C, 0x88, 0x7D, 0x0A, 0x28, 0x01, 0xD1, 0x00, 0x20, 0x02, 0xE0, 0x01,
        0x30, 0x88, 0x75, 0x01, 0x20, 0x02, 0xBC, 0x08, 0x47,
    ];
    let availability_body = [
        0x00, 0xB5, 0x01, 0x22, 0x8A, 0x40, 0x83, 0x7D, 0x99, 0x42, 0x05, 0xD2, 0x80, 0x8A, 0x10,
        0x40, 0x00, 0x28, 0x01, 0xD1, 0x01, 0x20, 0x00, 0xE0, 0x00, 0x20, 0x02, 0xBC, 0x08, 0x47,
    ];
    for (
        case,
        (
            dispatch,
            first_slot,
            unlock_handler,
            availability_handler,
            unlock_leaf,
            availability_leaf,
        ),
    ) in CASES.iter().zip([
        (0x3F904, 0x123, 0x434BA, 0x434E0, 0x9EED0, 0x9EEA4),
        (0x3FAF0, 0x127, 0x4370A, 0x43730, 0xA4064, 0xA4038),
        (0x3F578, 0x123, 0x4312E, 0x43154, 0x9E908, 0x9E8DC),
        (0x3F84C, 0x127, 0x43466, 0x4348C, 0xA3AA4, 0xA3A78),
    ]) {
        fn call_target(rom: &[u8], pc: usize) -> usize {
            let hi = u16::from_le_bytes(rom[pc..pc + 2].try_into().unwrap());
            let lo = u16::from_le_bytes(rom[pc + 2..pc + 4].try_into().unwrap());
            assert_eq!(hi & 0xF800, 0xF000);
            assert_eq!(lo & 0xF800, 0xF800);
            let raw = (((hi & 0x7FF) as i32) << 12) | (((lo & 0x7FF) as i32) << 1);
            (pc as i32 + 4 + ((raw << 9) >> 9)) as usize
        }

        let rom = fs::read(local_rom_path(case.rom)).unwrap();
        for (slot, handler) in [
            (first_slot, unlock_handler),
            (first_slot + 1, availability_handler),
        ] {
            assert_eq!(
                u32::from_le_bytes(
                    rom[dispatch + slot * 4..dispatch + slot * 4 + 4]
                        .try_into()
                        .unwrap(),
                ),
                0x08000000 + handler as u32,
                "{} Van-album slot {slot:#x}",
                case.name
            );
        }
        assert_eq!(
            call_target(&rom, unlock_handler + 0x0C),
            unlock_leaf,
            "{} unlock leaf",
            case.name
        );
        assert_eq!(
            call_target(&rom, availability_handler + 0x2E),
            availability_leaf,
            "{} availability leaf",
            case.name
        );
        assert_eq!(
            &rom[unlock_leaf..unlock_leaf + unlock_body.len()],
            &unlock_body,
            "{} unlock body",
            case.name
        );
        assert_eq!(
            &rom[availability_leaf..availability_leaf + availability_body.len()],
            &availability_body,
            "{} availability body",
            case.name
        );
        // The wrapper tests exactly ten indices: 0 through 9.
        assert_eq!(
            &rom[availability_handler + 0x24..availability_handler + 0x28],
            &[0x01, 0x34, 0x09, 0x2C],
            "{} ten-album loop",
            case.name
        );
    }
}

#[test]
fn native_spouse_nickname_setter_uses_player_token_and_fixed_string_field() {
    for (case, (dispatch, slot, handler, literal_offset, fallback_text, farmer_offset)) in
        CASES.iter().zip([
            (0x3F904, 0x125, 0x4358E, 0x62, 0xF9EB8, 0x1BD8),
            (0x3FAF0, 0x129, 0x437E8, 0x64, 0x102AFC, 0x1BE8),
            (0x3F578, 0x125, 0x43202, 0x62, 0xF9714, 0x1BD8),
            (0x3F84C, 0x129, 0x43544, 0x64, 0x102A9C, 0x1BE8),
        ])
    {
        let rom = fs::read(local_rom_path(case.rom)).unwrap();
        assert_eq!(
            u32::from_le_bytes(
                rom[dispatch + slot * 4..dispatch + slot * 4 + 4]
                    .try_into()
                    .unwrap(),
            ),
            0x08000000 + handler as u32,
            "{} spouse-nickname slot",
            case.name
        );
        assert_eq!(
            u32::from_le_bytes(
                rom[handler + literal_offset..handler + literal_offset + 4]
                    .try_into()
                    .unwrap(),
            ),
            0x08000000 + fallback_text as u32,
            "{} invalid-text fallback",
            case.name
        );
        assert_eq!(
            u32::from_le_bytes(
                rom[handler + literal_offset + 4..handler + literal_offset + 8]
                    .try_into()
                    .unwrap(),
            ),
            0xFF21,
            "{} player-name control token",
            case.name
        );
        assert_eq!(
            u32::from_le_bytes(
                rom[handler + literal_offset + 8..handler + literal_offset + 12]
                    .try_into()
                    .unwrap(),
            ),
            farmer_offset,
            "{} Farmer field offset for token branch",
            case.name
        );
        assert_eq!(
            u32::from_le_bytes(
                rom[handler + literal_offset + 0x24..handler + literal_offset + 0x28]
                    .try_into()
                    .unwrap(),
            ),
            farmer_offset,
            "{} Farmer field offset for literal branch",
            case.name
        );
        // strlen(source) must exceed one byte before the first two bytes are
        // combined as source[0] << 8 | source[1] and compared with FF21.
        let prefix_check = 0x32;
        assert_eq!(
            &rom[handler + prefix_check..handler + prefix_check + 14],
            &[
                0x01,
                0x28,
                if literal_offset == 0x62 { 0x1B } else { 0x1C },
                0xD9,
                0x20,
                0x78,
                0x00,
                0x02,
                0x61,
                0x78,
                0x40,
                0x18,
                0x09 + (literal_offset != 0x62) as u8,
                0x49
            ],
            "{} encoded player-token prefix check",
            case.name
        );
    }
}

#[test]
fn native_farmhouse_bed_placement_changes_map_and_positions_player() {
    fn call_target(rom: &[u8], pc: usize) -> usize {
        let hi = u16::from_le_bytes(rom[pc..pc + 2].try_into().unwrap());
        let lo = u16::from_le_bytes(rom[pc + 2..pc + 4].try_into().unwrap());
        assert_eq!(hi & 0xF800, 0xF000);
        assert_eq!(lo & 0xF800, 0xF800);
        let raw = (((hi & 0x7FF) as i32) << 12) | (((lo & 0x7FF) as i32) << 1);
        (pc as i32 + 4 + ((raw << 9) >> 9)) as usize
    }

    for (case, (dispatch, slot, handler, upgrade_getter, change_map, set_position, x_table)) in
        CASES.iter().zip([
            (0x3F904, 0x126, 0x43F2C, 0x0BF14, 0x122E0, 0x12064, 0xF9EF4),
            (0x3FAF0, 0x12A, 0x44194, 0x0BF88, 0x123D0, 0x1214C, 0x102B38),
            (0x3F578, 0x126, 0x43BA0, 0x0BEF4, 0x121B0, 0x11F34, 0xF9750),
            (0x3F84C, 0x12A, 0x43EF0, 0x0BF3C, 0x12274, 0x11FF0, 0x102AD8),
        ])
    {
        let rom = fs::read(local_rom_path(case.rom)).unwrap();
        assert_eq!(
            u32::from_le_bytes(
                rom[dispatch + slot * 4..dispatch + slot * 4 + 4]
                    .try_into()
                    .unwrap(),
            ),
            0x08000000 + handler as u32,
            "{} farmhouse-bed slot",
            case.name
        );
        assert_eq!(
            call_target(&rom, handler + 0x1E),
            upgrade_getter,
            "{} upgrade getter",
            case.name
        );
        assert_eq!(
            call_target(&rom, handler + 0x34),
            change_map,
            "{} map transition",
            case.name
        );
        assert_eq!(
            call_target(&rom, handler + 0x44),
            set_position,
            "{} player placement",
            case.name
        );
        assert_eq!(
            u32::from_le_bytes(rom[handler + 0x4C..handler + 0x50].try_into().unwrap()),
            0x08000000 + x_table as u32,
            "{} bed-X table pointer",
            case.name
        );
        assert_eq!(
            &rom[x_table..x_table + 6],
            &[143, 0, 7, 1, 71, 1],
            "{} upgrade-indexed bed X coordinates",
            case.name
        );
        // r1=MAP_FARMHOUSE (0x1D), r3=Y(112), followed by
        // r1=ENTITY_PLAYER (0) and stack facing=FACING_LEFT (2).
        assert_eq!(
            &rom[handler + 0x2C..handler + 0x34],
            &[0x10, 0x1C, 0x1D, 0x21, 0x22, 0x1C, 0x70, 0x23]
        );
        assert_eq!(
            &rom[handler + 0x38..handler + 0x44],
            &[0x30, 0x68, 0x02, 0x21, 0x00, 0x91, 0x00, 0x21, 0x22, 0x1C, 0x70, 0x23]
        );
    }
}

#[test]
fn native_shipped_amount_uses_display_gate_and_complete_product_domain() {
    fn call_target(rom: &[u8], pc: usize) -> usize {
        let hi = u16::from_le_bytes(rom[pc..pc + 2].try_into().unwrap());
        let lo = u16::from_le_bytes(rom[pc + 2..pc + 4].try_into().unwrap());
        assert_eq!(hi & 0xF800, 0xF000);
        assert_eq!(lo & 0xF800, 0xF800);
        let raw = (((hi & 0x7FF) as i32) << 12) | (((lo & 0x7FF) as i32) << 1);
        (pc as i32 + 4 + ((raw << 9) >> 9)) as usize
    }

    for (case, (dispatch, slot, handler, display_query, amount_query, stat_display, stat_amount)) in
        CASES.iter().zip([
            (0x3F904, 0x12C, 0x44164, 0x0B124, 0x0B140, 0x0B280, 0x0B288),
            (0x3FAF0, 0x130, 0x443D8, 0x0B194, 0x0B1B0, 0x0B2F0, 0x0B2F8),
            (0x3F578, 0x12C, 0x43DD8, 0x0B104, 0x0B120, 0x0B260, 0x0B268),
            (0x3F84C, 0x130, 0x44134, 0x0B148, 0x0B164, 0x0B2A4, 0x0B2AC),
        ])
    {
        let rom = fs::read(local_rom_path(case.rom)).unwrap();
        assert_eq!(
            u32::from_le_bytes(
                rom[dispatch + slot * 4..dispatch + slot * 4 + 4]
                    .try_into()
                    .unwrap(),
            ),
            0x08000000 + handler as u32,
            "{} shipped-count slot",
            case.name
        );
        assert_eq!(
            call_target(&rom, handler + 0x28),
            display_query,
            "{} display query",
            case.name
        );
        assert_eq!(
            call_target(&rom, handler + 0x3E),
            amount_query,
            "{} amount query",
            case.name
        );
        assert_eq!(
            call_target(&rom, display_query + 0x12),
            stat_display,
            "{} display field",
            case.name
        );
        assert_eq!(
            call_target(&rom, amount_query + 0x12),
            stat_amount,
            "{} amount field",
            case.name
        );
        // Both public accessors use signed `product_id <= 0x66`. This accepts
        // the complete product enum, returns zero above it, but does not reject
        // negative raw integers.
        assert_eq!(
            &rom[display_query + 4..display_query + 8],
            &[0x66, 0x29, 0x01, 0xDD]
        );
        assert_eq!(
            &rom[amount_query + 4..amount_query + 8],
            &[0x66, 0x29, 0x01, 0xDD]
        );
        assert_eq!(
            &rom[stat_display..stat_display + 6],
            &[0x00, 0x68, 0xC0, 0x0F, 0x70, 0x47]
        );
        assert_eq!(
            &rom[stat_amount..stat_amount + 20],
            &[
                0x00, 0xB5, 0x00, 0x68, 0x00, 0x28, 0x01, 0xDB, 0x00, 0x20, 0x01, 0xE0, 0x40, 0x00,
                0x40, 0x08, 0x02, 0xBC, 0x08, 0x47
            ]
        );
    }
}

#[test]
fn native_sunrise_and_sparkle_effects_share_fixed_host_but_distinct_child_slots() {
    fn call_target(rom: &[u8], pc: usize) -> usize {
        let hi = u16::from_le_bytes(rom[pc..pc + 2].try_into().unwrap());
        let lo = u16::from_le_bytes(rom[pc + 2..pc + 4].try_into().unwrap());
        assert_eq!(hi & 0xF800, 0xF000);
        assert_eq!(lo & 0xF800, 0xF800);
        let raw = (((hi & 0x7FF) as i32) << 12) | (((lo & 0x7FF) as i32) << 1);
        (pc as i32 + 4 + ((raw << 9) >> 9)) as usize
    }

    for (case, (dispatch, first_slot, handlers, leaves, entity_ops, resolver)) in
        CASES.iter().zip([
            (
                0x3F904,
                0x12F,
                [0x44F48, 0x44F5C, 0x44F74, 0x44F88],
                [0x16C2C, 0x16C48, 0x16C6C, 0x16C88],
                [0x387B8, 0x387C8, 0x387EC, 0x387FC],
                0xD3914,
            ),
            (
                0x3FAF0,
                0x133,
                [0x451BC, 0x451D4, 0x451EC, 0x45204],
                [0x16CAC, 0x16CC8, 0x16CEC, 0x16D08],
                [0x38A80, 0x38A90, 0x38AB4, 0x38AC4],
                0xDB53C,
            ),
            (
                0x3F578,
                0x12F,
                [0x44BBC, 0x44BD0, 0x44BE8, 0x44BFC],
                [0x169C0, 0x169DC, 0x16A00, 0x16A1C],
                [0x3854C, 0x3855C, 0x38580, 0x38590],
                0xD30CC,
            ),
            (
                0x3F84C,
                0x133,
                [0x44F18, 0x44F30, 0x44F48, 0x44F60],
                [0x16B20, 0x16B3C, 0x16B60, 0x16B7C],
                [0x388F4, 0x38904, 0x38928, 0x38938],
                0xDB050,
            ),
        ])
    {
        let rom = fs::read(local_rom_path(case.rom)).unwrap();
        for index in 0..4 {
            assert_eq!(
                u32::from_le_bytes(
                    rom[dispatch + (first_slot + index) * 4
                        ..dispatch + (first_slot + index) * 4 + 4]
                        .try_into()
                        .unwrap(),
                ),
                0x08000000 + handlers[index] as u32,
                "{} effect slot {}",
                case.name,
                index
            );
            let handler_call = if case.name.starts_with("fomt-") && (index == 0 || index == 2) {
                handlers[index] + 0x0E
            } else {
                handlers[index] + 0x10
            };
            assert_eq!(
                call_target(&rom, handler_call),
                leaves[index],
                "{} effect leaf {}",
                case.name,
                index
            );
            let host_immediate = if index == 0 || index == 2 {
                leaves[index] + 0x0C
            } else {
                leaves[index] + 0x0E
            };
            assert_eq!(
                &rom[host_immediate..host_immediate + 2],
                &[0x5D, 0x21],
                "{} host entity {}",
                case.name,
                index
            );
            assert_eq!(
                call_target(&rom, host_immediate + 2),
                resolver,
                "{} host resolver {}",
                case.name,
                index
            );
            assert_eq!(
                call_target(&rom, host_immediate + 6),
                entity_ops[index],
                "{} entity operation {}",
                case.name,
                index
            );
        }
        assert_eq!(
            &rom[leaves[1] + 0x1A..leaves[1] + 0x1E],
            &[0x1C, 0x20, 0x20, 0x60],
            "{} sunrise wait state",
            case.name
        );
        assert_eq!(
            &rom[leaves[3] + 0x1A..leaves[3] + 0x1E],
            &[0x1B, 0x20, 0x20, 0x60],
            "{} sparkle wait state",
            case.name
        );
    }
}

#[test]
fn native_mine_floor_generation_excludes_matching_mobile_actor_footprints() {
    fn call_target(rom: &[u8], pc: usize) -> usize {
        let hi = u16::from_le_bytes(rom[pc..pc + 2].try_into().unwrap());
        let lo = u16::from_le_bytes(rom[pc + 2..pc + 4].try_into().unwrap());
        assert_eq!(hi & 0xF800, 0xF000);
        assert_eq!(lo & 0xF800, 0xF800);
        let raw = (((hi & 0x7FF) as i32) << 12) | (((lo & 0x7FF) as i32) << 1);
        (pc as i32 + 4 + ((raw << 9) >> 9)) as usize
    }

    for (case, (dispatch, slot, handler, leaf, generator)) in CASES.iter().zip([
        (0x3F904, 0x134, 0x43FF2, 0x159B0, 0x9CF34),
        (0x3FAF0, 0x138, 0x44264, 0x15A30, 0xA20BC),
        (0x3F578, 0x134, 0x43C66, 0x15744, 0x9C96C),
        (0x3F84C, 0x138, 0x43FC0, 0x158A4, 0xA1AFC),
    ]) {
        let rom = fs::read(local_rom_path(case.rom)).unwrap();
        assert_eq!(
            u32::from_le_bytes(
                rom[dispatch + slot * 4..dispatch + slot * 4 + 4]
                    .try_into()
                    .unwrap(),
            ),
            0x08000000 + handler as u32,
            "{} mine-layout callable slot",
            case.name
        );
        assert_eq!(
            call_target(&rom, handler + 0x3E),
            leaf,
            "{} mine-layout leaf",
            case.name
        );

        // The leaf resolves three existing runtime actors in this exact order:
        // basket (74), farm dog (43), and the still-unidentified entity 75.
        // Each matching actor contributes its four tile-corner coordinates.
        for (offset, entity_id) in [(0x1C, 74u8), (0x174, 43), (0x2D2, 75)] {
            assert_eq!(
                &rom[leaf + offset..leaf + offset + 2],
                &[entity_id, 0x21],
                "{} occupied-actor immediate at {offset:#x}",
                case.name
            );
        }

        assert_eq!(
            call_target(&rom, leaf + 0x446),
            generator,
            "{} procedural mine generator",
            case.name
        );
    }
}

#[test]
fn native_mine_descent_builds_the_next_floor_with_target_actor_exclusions() {
    fn call_target(rom: &[u8], pc: usize) -> usize {
        let hi = u16::from_le_bytes(rom[pc..pc + 2].try_into().unwrap());
        let lo = u16::from_le_bytes(rom[pc + 2..pc + 4].try_into().unwrap());
        assert_eq!(hi & 0xF800, 0xF000);
        assert_eq!(lo & 0xF800, 0xF800);
        let raw = (((hi & 0x7FF) as i32) << 12) | (((lo & 0x7FF) as i32) << 1);
        (pc as i32 + 4 + ((raw << 9) >> 9)) as usize
    }

    for (case, (dispatch, slot, handler, core, spring_base, spring_before)) in CASES.iter().zip([
        (0x3F904, 0x135, 0x44D40, 0x15E30, 0x34u8, 0x33u8),
        (0x3FAF0, 0x139, 0x44F9E, 0x15EB0, 0x3A, 0x39),
        (0x3F578, 0x135, 0x449B4, 0x15BC4, 0x34, 0x33),
        (0x3F84C, 0x139, 0x44CFA, 0x15D24, 0x3A, 0x39),
    ]) {
        let rom = fs::read(local_rom_path(case.rom)).unwrap();
        assert_eq!(
            u32::from_le_bytes(
                rom[dispatch + slot * 4..dispatch + slot * 4 + 4]
                    .try_into()
                    .unwrap(),
            ),
            0x08000000 + handler as u32,
            "{} mine-descent callable slot",
            case.name
        );
        assert_eq!(
            call_target(&rom, handler + 0x10),
            core,
            "{} mine-descent core",
            case.name
        );

        assert_eq!(&rom[core + 0x2A..core + 0x2C], &[spring_base, 0x39]);
        assert_eq!(&rom[core + 0x34..core + 0x36], &[0x50, 0x1C]);
        assert_eq!(&rom[core + 0x3A..core + 0x3C], &[spring_before, 0x3E]);

        for (offset, entity_id) in [
            (0x44, 74u8),
            (0x176, 43),
            (0x2A8, 75),
            (0x43A, 74),
            (0x56E, 43),
            (0x6A2, 75),
        ] {
            assert_eq!(
                &rom[core + offset..core + offset + 2],
                &[entity_id, 0x21],
                "{} descent actor at {offset:#x}",
                case.name
            );
        }
    }
}

#[test]
fn native_cursed_tool_callables_preserve_mapping_and_lift_method_domains() {
    fn call_target(rom: &[u8], pc: usize) -> usize {
        let hi = u16::from_le_bytes(rom[pc..pc + 2].try_into().unwrap());
        let lo = u16::from_le_bytes(rom[pc + 2..pc + 4].try_into().unwrap());
        assert_eq!(hi & 0xF800, 0xF000);
        assert_eq!(lo & 0xF800, 0xF800);
        let raw = (((hi & 0x7FF) as i32) << 12) | (((lo & 0x7FF) as i32) << 1);
        (pc as i32 + 4 + ((raw << 9) >> 9)) as usize
    }

    for (case, (dispatch, first_slot, handlers, cores, wrapper_call)) in CASES.iter().zip([
        (
            0x3F904,
            0x136,
            [0x44EAC, 0x44EE0, 0x44F14],
            [0x9C304, 0x9C444, 0x9C474],
            0x2A,
        ),
        (
            0x3FAF0,
            0x13A,
            [0x45114, 0x4514C, 0x45184],
            [0xA1274, 0xA13B4, 0xA13E4],
            0x28,
        ),
        (
            0x3F578,
            0x136,
            [0x44B20, 0x44B54, 0x44B88],
            [0x9BD3C, 0x9BE7C, 0x9BEAC],
            0x2A,
        ),
        (
            0x3F84C,
            0x13A,
            [0x44E70, 0x44EA8, 0x44EE0],
            [0xA0CB4, 0xA0DF4, 0xA0E24],
            0x28,
        ),
    ]) {
        let rom = fs::read(local_rom_path(case.rom)).unwrap();
        for index in 0..3 {
            assert_eq!(
                u32::from_le_bytes(
                    rom[dispatch + (first_slot + index) * 4
                        ..dispatch + (first_slot + index) * 4 + 4]
                        .try_into()
                        .unwrap(),
                ),
                0x08000000 + handlers[index] as u32,
                "{} cursed-tool slot {index}",
                case.name
            );
            assert_eq!(
                call_target(&rom, handlers[index] + wrapper_call),
                cores[index],
                "{} cursed-tool core {index}",
                case.name
            );
        }

        // The daily advance core accepts only mapped indices 0 (sickle) and
        // 3 (hammer). The church core advances 1 (hoe) and 4 (watering can),
        // while its later 0/3 checks reset the consecutive-day counters.
        assert_eq!(&rom[cores[1] + 0x14..cores[1] + 0x16], &[0x00, 0x29]);
        assert_eq!(&rom[cores[1] + 0x18..cores[1] + 0x1A], &[0x03, 0x29]);
        assert_eq!(&rom[cores[2] + 0x14..cores[2] + 0x16], &[0x01, 0x2C]);
        assert_eq!(&rom[cores[2] + 0x18..cores[2] + 0x1A], &[0x04, 0x2C]);
        assert_eq!(&rom[cores[2] + 0x28..cores[2] + 0x2A], &[0x00, 0x2C]);
        assert_eq!(&rom[cores[2] + 0x2C..cores[2] + 0x2E], &[0x03, 0x2C]);
    }
}

#[test]
fn native_tool_cycle_skips_cursed_tools_instead_of_selecting_one() {
    fn call_target(rom: &[u8], pc: usize) -> usize {
        let hi = u16::from_le_bytes(rom[pc..pc + 2].try_into().unwrap());
        let lo = u16::from_le_bytes(rom[pc + 2..pc + 4].try_into().unwrap());
        assert_eq!(hi & 0xF800, 0xF000);
        assert_eq!(lo & 0xF800, 0xF800);
        let raw = (((hi & 0x7FF) as i32) << 12) | (((lo & 0x7FF) as i32) << 1);
        (pc as i32 + 4 + ((raw << 9) >> 9)) as usize
    }

    for (case, (dispatch, slot, handler, cycle_back)) in CASES.iter().zip([
        (0x3F904, 0x146, 0x41794, 0x0EC4C),
        (0x3FAF0, 0x14A, 0x419D8, 0x0ECEC),
        (0x3F578, 0x146, 0x41408, 0x0EC2C),
        (0x3F84C, 0x14A, 0x41734, 0x0ECA0),
    ]) {
        let rom = fs::read(local_rom_path(case.rom)).unwrap();
        assert_eq!(
            u32::from_le_bytes(
                rom[dispatch + slot * 4..dispatch + slot * 4 + 4]
                    .try_into()
                    .unwrap(),
            ),
            0x08000000 + handler as u32,
            "{} tool-cycle callable slot",
            case.name
        );
        for (offset, cursed_tool_id) in [
            (0x26, 0x05u8),
            (0x3C, 0x0D),
            (0x50, 0x15),
            (0x66, 0x1D),
            (0x7C, 0x25),
            (0x92, 0x2D),
        ] {
            assert_eq!(
                &rom[handler + offset..handler + offset + 2],
                &[cursed_tool_id, 0x28],
                "{} cursed-tool skip ID at {offset:#x}",
                case.name
            );
        }
        assert_eq!(&rom[handler + 0x9C..handler + 0x9E], &[0x09, 0x2E]);
        assert_eq!(
            call_target(&rom, handler + 0xA6),
            cycle_back,
            "{} cycle-back operation",
            case.name
        );
    }
}

#[test]
fn native_refresh_all_npc_schedules_scans_ids_one_through_thirty_five() {
    fn call_target(rom: &[u8], pc: usize) -> usize {
        let hi = u16::from_le_bytes(rom[pc..pc + 2].try_into().unwrap());
        let lo = u16::from_le_bytes(rom[pc + 2..pc + 4].try_into().unwrap());
        assert_eq!(hi & 0xF800, 0xF000);
        assert_eq!(lo & 0xF800, 0xF800);
        let raw = (((hi & 0x7FF) as i32) << 12) | (((lo & 0x7FF) as i32) << 1);
        (pc as i32 + 4 + ((raw << 9) >> 9)) as usize
    }

    for (case, (dispatch, slot, handler, leaf)) in CASES.iter().zip([
        (0x3F904, 0xFE, 0x442D2, 0x14BD8),
        (0x3FAF0, 0x101, 0x44542, 0x14D64),
        (0x3F578, 0xFE, 0x43F46, 0x14AAC),
        (0x3F84C, 0x101, 0x4429E, 0x14C0C),
    ]) {
        let rom = fs::read(local_rom_path(case.rom)).unwrap();
        assert_eq!(
            u32::from_le_bytes(
                rom[dispatch + slot * 4..dispatch + slot * 4 + 4]
                    .try_into()
                    .unwrap()
            ),
            0x08000000 + handler as u32,
            "{} schedule-refresh physical slot",
            case.name
        );
        assert_eq!(
            call_target(&rom, handler + 0x10),
            leaf,
            "{} schedule-refresh leaf",
            case.name
        );

        let halfword = |offset: usize| {
            u16::from_le_bytes(rom[leaf + offset..leaf + offset + 2].try_into().unwrap())
        };
        assert_eq!(halfword(0x04), 0x2401, "{} first NPC ID", case.name);
        assert_eq!(halfword(0x22), 0x2103, "{} refresh mode", case.name);
        assert_eq!(halfword(0x28), 0x3401, "{} NPC increment", case.name);
        assert_eq!(halfword(0x2A), 0x2C23, "{} last NPC ID", case.name);
        assert_eq!(
            halfword(0x2C) & 0xFF00,
            0xD900,
            "{} inclusive loop branch",
            case.name
        );
    }
}

#[test]
fn native_mailbox_bitmaps_follow_each_games_physical_letter_domain() {
    fn call_target(rom: &[u8], pc: usize) -> usize {
        let hi = u16::from_le_bytes(rom[pc..pc + 2].try_into().unwrap());
        let lo = u16::from_le_bytes(rom[pc + 2..pc + 4].try_into().unwrap());
        assert_eq!(hi & 0xF800, 0xF000);
        assert_eq!(lo & 0xF800, 0xF800);
        let raw = (((hi & 0x7FF) as i32) << 12) | (((lo & 0x7FF) as i32) << 1);
        (pc as i32 + 4 + ((raw << 9) >> 9)) as usize
    }

    for (case, (dispatch, first_slot, handlers, leaves, last_index, map_bytes)) in
        CASES.iter().zip([
            (
                0x3F904,
                0xF5,
                [0x43E30, 0x43E66, 0x43E9C, 0x43EC8, 0x43EF4, 0x43F0A],
                [0xBD14, 0xBD40, 0xBCB0, 0xBCD0, 0xBD6C, 0xBDEC],
                59u8,
                8u8,
            ),
            (
                0x3FAF0,
                0xF8,
                [0x4408C, 0x440C2, 0x440F8, 0x44124, 0x44150, 0x44172],
                [0xBD84, 0xBDB0, 0xBD20, 0xBD40, 0xBDDC, 0xBE5C],
                112u8,
                16u8,
            ),
            (
                0x3F578,
                0xF5,
                [0x43AA4, 0x43ADA, 0x43B10, 0x43B3C, 0x43B68, 0x43B7E],
                [0xBCF4, 0xBD20, 0xBC90, 0xBCB0, 0xBD4C, 0xBDCC],
                59u8,
                8u8,
            ),
            (
                0x3F84C,
                0xF8,
                [0x43DE8, 0x43E1E, 0x43E54, 0x43E80, 0x43EAC, 0x43ECE],
                [0xBD38, 0xBD64, 0xBCD4, 0xBCF4, 0xBD90, 0xBE10],
                112u8,
                16u8,
            ),
        ])
    {
        let rom = fs::read(local_rom_path(case.rom)).unwrap();
        for (index, handler) in handlers.into_iter().enumerate() {
            assert_eq!(
                u32::from_le_bytes(
                    rom[dispatch + (first_slot + index) * 4
                        ..dispatch + (first_slot + index) * 4 + 4]
                        .try_into()
                        .unwrap()
                ),
                0x08000000 + handler as u32,
                "{} mailbox callable slot {}",
                case.name,
                first_slot + index
            );
            let call_offset = match index {
                0 | 1 => 0x2A,
                2 | 3 => 0x24,
                _ => 0x0E,
            };
            assert_eq!(
                call_target(&rom, handler + call_offset),
                leaves[index],
                "{} mailbox leaf {index}",
                case.name
            );
        }

        // All four leaf families subtract the physical page-table base 77.
        assert_eq!(&rom[leaves[0] + 6..leaves[0] + 8], &[0x4D, 0x3A]);
        assert_eq!(&rom[leaves[2] + 2..leaves[2] + 4], &[0x4D, 0x39]);
        // Getter/setter bounds are inclusive: 59 => 60 FoMT letters,
        // 112 => 113 MFoMT letters. The second bitmap begins after 8/16 bytes.
        assert_eq!(rom[leaves[0] + 10], last_index);
        assert_eq!(rom[leaves[2] + 4], last_index);
        assert_eq!(rom[leaves[1] + 8], map_bytes);
        assert_eq!(rom[leaves[5] + 10], map_bytes);
    }
}

#[test]
fn native_flash_rgb_packing_does_not_clamp_channels_on_any_target() {
    // Thumb: lsl r0,r6,5; orr r1,r0; lsl r0,r7,10; orr r1,r0;
    // mov r0,r2. There is no channel mask between the shifts and ORs.
    let expected = [0x70, 0x01, 0x01, 0x43, 0xB8, 0x02, 0x01, 0x43, 0x10, 0x1C];
    for (case, offset) in CASES.iter().zip([0x43FCC, 0x44234, 0x43C40, 0x43F90]) {
        let rom = fs::read(local_rom_path(case.rom)).unwrap();
        assert_eq!(
            &rom[offset..offset + expected.len()],
            &expected,
            "{} RGB packing",
            case.name
        );
    }
}

#[test]
fn native_flash_lifetime_and_strength_sequence_match_all_targets() {
    let reference = fs::read(local_rom_path(CASES[0].rom)).unwrap();
    for (case, (init, update, table)) in CASES.iter().zip([
        (0x14AB8, 0xDA814, 0xF05D9),
        (0x14C44, 0xE2D38, 0xF8DD1),
        (0x1498C, 0xD9FC8, 0xEFE2B),
        (0x14AEC, 0xE2848, 0xF8D9B),
    ]) {
        let rom = fs::read(local_rom_path(case.rom)).unwrap();
        // Resets the counter to zero, blend strength to 16, and state to 26.
        assert_eq!(
            &rom[init..init + 56],
            &reference[0x14AB8..0x14AF0],
            "{} flash init",
            case.name
        );
        // Checks counter == 120, clears the active flag and blending state,
        // and returns to state 2 before reading another strength entry.
        assert_eq!(
            &rom[update..update + 0x54],
            &reference[0xDA814..0xDA868],
            "{} flash end",
            case.name
        );
        // Reads strengths[counter], writes the blend coefficient and advances
        // the counter exactly once. Exclude the following relocated literal.
        assert_eq!(
            &rom[update + 0x1EA..update + 0x234],
            &reference[0xDA9FE..0xDAA48],
            "{} flash strength application and counter advance",
            case.name
        );
        // Validate the updater's actual table pointer, not just similar data.
        assert_eq!(
            u32::from_le_bytes(rom[update + 0x238..update + 0x23C].try_into().unwrap()),
            0x08000000 + table as u32,
            "{} flash table pointer",
            case.name
        );
        assert_eq!(
            &rom[table..table + 120],
            &reference[0xF05D9..0xF0651],
            "{} flash strengths",
            case.name
        );
    }
}

#[test]
fn native_entity_effect_lifetime_uses_animation_wrap_on_all_targets() {
    let reference = fs::read(local_rom_path(CASES[0].rom)).unwrap();
    for (case, (lifetime, stepper)) in CASES.iter().zip([
        (0x32670, 0x5E8F0),
        (0x32A2C, 0x6166C),
        (0x32404, 0x5E634),
        (0x328A0, 0x612E4),
    ]) {
        let rom = fs::read(local_rom_path(case.rom)).unwrap();
        // Consumer tests wrap-status bit 2, then exempts persistent mode 2
        // before clearing the active mode. No fixed lifetime counter is used.
        assert_eq!(
            &rom[lifetime..lifetime + 26],
            &reference[0x32670..0x3268A],
            "{} effect clear condition",
            case.name
        );
        // Producer sets that bit when stepping past the animation endpoints;
        // a zero delay/speed can prevent reaching the wrap notification.
        assert_eq!(
            &rom[stepper..stepper + 0xA8],
            &reference[0x5E8F0..0x5E998],
            "{} animation wrap producer",
            case.name
        );
    }
}

#[test]
fn native_entity_emote_animation_8_is_sleep_on_all_targets() {
    for (case, definition) in CASES.iter().zip([0x71D51C, 0x6C2F38, 0x4A3678, 0x6BF8B0]) {
        let rom = fs::read(local_rom_path(case.rom)).unwrap();
        assert_eq!(
            u32::from_le_bytes(rom[definition..definition + 4].try_into().unwrap()),
            11,
            "{} emote animation count",
            case.name
        );
        let animation_8 = definition + 4 + 8 * 4;
        assert_eq!(
            &rom[animation_8..animation_8 + 4],
            &[6, 0, 28, 0],
            "{} sleep animation range",
            case.name
        );

        let frame_count_offset = definition + 4 + 11 * 4;
        let frame_count = u32::from_le_bytes(
            rom[frame_count_offset..frame_count_offset + 4]
                .try_into()
                .unwrap(),
        );
        let frame_metadata_end = frame_count_offset + 4 + frame_count as usize * 16;
        let oam_count = u32::from_le_bytes(
            rom[frame_metadata_end..frame_metadata_end + 4]
                .try_into()
                .unwrap(),
        ) as usize;
        let tile_count_offset = frame_metadata_end + 4 + oam_count * 8;
        let tile_count = u32::from_le_bytes(
            rom[tile_count_offset..tile_count_offset + 4]
                .try_into()
                .unwrap(),
        ) as usize;
        let palette_count_offset = tile_count_offset + 4 + tile_count * 32;
        let palette_count = u32::from_le_bytes(
            rom[palette_count_offset..palette_count_offset + 4]
                .try_into()
                .unwrap(),
        ) as usize;
        let affine_count_offset = palette_count_offset + 4 + palette_count * 32;
        let affine_count = u32::from_le_bytes(
            rom[affine_count_offset..affine_count_offset + 4]
                .try_into()
                .unwrap(),
        ) as usize;
        let instruction_count_offset = affine_count_offset + 4 + affine_count * 8;
        let instruction_pool = instruction_count_offset + 4;
        let frames: Vec<u16> = (28..34)
            .map(|index| {
                u16::from_le_bytes(
                    rom[instruction_pool + index * 4..instruction_pool + index * 4 + 2]
                        .try_into()
                        .unwrap(),
                )
            })
            .collect();
        assert_eq!(
            frames,
            [23, 20, 21, 22, 21, 20],
            "{} sleep frames",
            case.name
        );
    }
}

#[test]
fn native_mfomt_festival_count_missing_result_paths_match_both_regions() {
    for (case, table, entry, kind_check, horse_check, push, exit) in [
        (
            &CASES[1], 0x3FAF0, 0x45ADC, 0x45AFC, 0x45B46, 0x45C78, 0x45C88,
        ),
        (
            &CASES[3], 0x3F84C, 0x45838, 0x45858, 0x458A2, 0x459D4, 0x459E4,
        ),
    ] {
        let rom = fs::read(local_rom_path(case.rom)).unwrap();
        let slot = table + 0x152 * 4;
        assert_eq!(
            u32::from_le_bytes(rom[slot..slot + 4].try_into().unwrap()) & !1,
            0x08000000 + entry,
            "{} physical callable slot",
            case.name
        );
        // Unsigned kind > 4 (including negative values) bypasses VM push.
        assert_eq!(
            &rom[kind_check..kind_check + 6],
            &[0x04, 0x2C, 0x00, 0xD9, 0xC2, 0xE0]
        );
        // Existing horse with winner bit clear also exits without a result.
        assert_eq!(
            &rom[horse_check..horse_check + 8],
            &[0x00, 0x06, 0x00, 0x28, 0x00, 0xD1, 0x9C, 0xE0]
        );
        for branch in [kind_check + 4, horse_check + 6] {
            let instruction = u16::from_le_bytes(rom[branch..branch + 2].try_into().unwrap());
            let displacement = ((instruction & 0x7FF) as i32) << 21 >> 20;
            assert_eq!((branch as i32 + 4 + displacement) as usize, exit);
        }
        // The successful path stores r2 and increments VM depth. The exit
        // merely sets native r0: this must not be mistaken for a VM return.
        assert_eq!(
            &rom[push..exit],
            &[
                0x39, 0x68, 0x63, 0x29, 0x04, 0xD8, 0x88, 0x00, 0x40, 0x44, 0x02, 0x60, 0x48, 0x1C,
                0x38, 0x60
            ]
        );
        assert_eq!(
            &rom[exit..exit + 16],
            &[
                0x00, 0x20, 0x17, 0xB0, 0x38, 0xBC, 0x98, 0x46, 0xA1, 0x46, 0xAA, 0x46, 0xF0, 0xBC,
                0x02, 0xBC
            ]
        );
    }
}

#[test]
fn native_mfomt_tool_experience_and_festival_count_are_unused_by_vanilla_scripts() {
    for case in [&CASES[1], &CASES[3]] {
        let rom = fs::read(local_rom_path(case.rom)).unwrap();
        let entries = get_script_table(&rom).unwrap();
        assert_eq!(entries.len(), case.slots);
        let mut checked = 0;
        for entry in entries {
            if let ScriptTableEntry::Script { id, data, backing } = entry {
                let script = decode_script_with_backing(data, backing).unwrap();
                assert!(
                    !script
                        .instructions
                        .contains(&mary::ir::Ins::Call(mary::ir::CallId(0x151))),
                    "{} script {id} calls tool experience",
                    case.name
                );
                assert!(
                    !script
                        .instructions
                        .contains(&mary::ir::Ins::Call(mary::ir::CallId(0x152))),
                    "{} script {id} calls festival count",
                    case.name
                );
                checked += 1;
            }
        }
        assert_eq!(checked, 1415, "{} non-null scripts checked", case.name);
    }
}

#[test]
fn native_mfomt_tool_experience_preserves_public_to_storage_order() {
    for (case, dispatch, entry, table, getter, read, container_literal) in [
        (
            &CASES[1], 0x3FAF0, 0x45A08, 0x45A40, 0xEBCC, 0xF044, 0x45AD8,
        ),
        (
            &CASES[3], 0x3F84C, 0x45764, 0x4579C, 0xEB80, 0xEFF8, 0x45834,
        ),
    ] {
        let rom = fs::read(local_rom_path(case.rom)).unwrap();
        let word = |offset| u32::from_le_bytes(rom[offset..offset + 4].try_into().unwrap());
        assert_eq!(word(dispatch + 0x151 * 4), 0x08000000 + entry);
        assert_eq!(word(table - 4), 0x08000000 + table as u32);
        for (kind, internal) in [1u16, 0, 2, 3, 4, 5].iter().enumerate() {
            let branch = word(table + kind * 4) as usize - 0x08000000;
            assert_eq!(branch, table + 24 + 20 * kind);
            let instruction = u16::from_le_bytes(rom[branch + 12..branch + 14].try_into().unwrap());
            assert_eq!(instruction, 0x2100 | internal, "{} kind {kind}", case.name);
        }
        // Fixed four-byte records; experience is the first unsigned halfword.
        assert_eq!(word(container_literal), 0x1BE8);
        assert_eq!(
            &rom[getter..getter + 8],
            &[0x89, 0x00, 0x2C, 0x31, 0x40, 0x18, 0x70, 0x47]
        );
        assert_eq!(&rom[read..read + 4], &[0x00, 0x88, 0x70, 0x47]);
    }
}

#[test]
fn native_tool_upgrade_material_sources_are_article_ids_on_all_targets() {
    for (
        case,
        held_source_call,
        held_source,
        held_article_call,
        rucksack_source_call,
        rucksack_source,
        rucksack_article_call,
        article_getter,
    ) in [
        (
            &CASES[0], 0x91D40, 0xF258, 0x91D4A, 0x91E32, 0xF0E8, 0x91E3E, 0xDF54,
        ),
        (
            &CASES[1], 0x96C20, 0xF334, 0x96C2A, 0x96D12, 0xF1C4, 0x96D1E, 0xDFC8,
        ),
        (
            &CASES[2], 0x91880, 0xF238, 0x9188A, 0x91972, 0xF0C8, 0x9197E, 0xDF34,
        ),
        (
            &CASES[3], 0x967A0, 0xF2E8, 0x967AA, 0x96892, 0xF178, 0x9689E, 0xDF7C,
        ),
    ] {
        let rom = fs::read(local_rom_path(case.rom)).unwrap();
        assert_eq!(thumb_call_target(&rom, held_source_call), held_source);
        assert_eq!(thumb_call_target(&rom, held_article_call), article_getter);
        assert_eq!(
            thumb_call_target(&rom, rucksack_source_call),
            rucksack_source
        );
        assert_eq!(
            thumb_call_target(&rom, rucksack_article_call),
            article_getter
        );
        assert_eq!(
            &rom[article_getter..article_getter + 4],
            &[0x00, 0x78, 0x70, 0x47],
            "{} article-id leaf",
            case.name
        );
    }
}

#[test]
fn native_tool_experience_addition_has_unsigned_upper_clamp_on_all_targets() {
    let reference = fs::read(local_rom_path(CASES[0].rom)).unwrap();
    for (case, offset) in CASES.iter().zip([0xEF88, 0xF064, 0xEF68, 0xF018]) {
        let rom = fs::read(local_rom_path(case.rom)).unwrap();
        // Whole leaf including 0xFFFF literal: LDRH, 32-bit ADD, unsigned
        // comparison/BLS, STRH. This is not a signed lower-and-upper clamp.
        assert_eq!(
            &rom[offset..offset + 44],
            &reference[0xEF88..0xEFB4],
            "{} experience writer",
            case.name
        );
        assert_eq!(
            u32::from_le_bytes(rom[offset + 40..offset + 44].try_into().unwrap()),
            65535
        );
        assert_eq!(&rom[offset + 6..offset + 10], &[0x20, 0x88, 0x40, 0x18]);
        assert_eq!(&rom[offset + 22..offset + 24], &[0x00, 0xD9]);
    }
    // Arithmetic model of the pinned instructions, not an emulator run.
    let add = |old: u16, delta: i32| (old as u32).wrapping_add(delta as u32).min(65535) as u16;
    for (old, delta, expected) in [
        (0, -1, 65535),
        (10, -1, 9),
        (65530, 100, 65535),
        (0, 50, 50),
        (0, 100, 100),
    ] {
        assert_eq!(add(old, delta), expected);
    }
}

#[test]
fn native_offset_entity_position_adds_signed_deltas_on_all_four_targets() {
    for (case, dispatch, handler, leaf, resolver, updater) in [
        (&CASES[0], 0x3F904, 0x3FF6C, 0x1223C, 0xD3914, 0x33B84),
        (&CASES[1], 0x3FAF0, 0x40182, 0x1232C, 0xDB53C, 0x33F50),
        (&CASES[2], 0x3F578, 0x3FBE0, 0x1210C, 0xD30CC, 0x33918),
        (&CASES[3], 0x3F84C, 0x3FEDE, 0x121D0, 0xDB050, 0x33DC4),
    ] {
        let rom = fs::read(local_rom_path(case.rom)).unwrap();
        assert_eq!(
            u32::from_le_bytes(
                rom[dispatch + 0x15 * 4..dispatch + 0x15 * 4 + 4]
                    .try_into()
                    .unwrap()
            ),
            0x0800_0000 + handler as u32,
            "{} callable slot 0x15",
            case.name
        );

        // The interpreter pops delta_y into r3, delta_x into r2, and entity_id
        // into r1 before forwarding all three to the native leaf.
        assert_eq!(
            [
                u16::from_le_bytes(rom[handler + 0x12..handler + 0x14].try_into().unwrap()),
                u16::from_le_bytes(rom[handler + 0x24..handler + 0x26].try_into().unwrap()),
                u16::from_le_bytes(rom[handler + 0x36..handler + 0x38].try_into().unwrap()),
            ],
            [0x6803, 0x6802, 0x6801],
            "{} argument pop order",
            case.name
        );
        assert_eq!(thumb_call_target(&rom, handler + 0x50), leaf);

        // The leaf preserves r2/r3 as signed deltas, resolves entity_id from
        // r1, loads signed halfword X/Y at +0x0A/+0x0E, adds the deltas, then
        // calls the position updater with the existing facing byte at +0x30.
        assert_eq!(&rom[leaf..leaf + 6], &[0x70, 0xB5, 0x15, 0x1C, 0x1E, 0x1C]);
        assert_eq!(thumb_call_target(&rom, leaf + 0x10), resolver);
        assert_eq!(
            &rom[leaf + 0x1A..leaf + 0x2C],
            &[
                0x0A, 0x20, 0x22, 0x5E, 0x0E, 0x20, 0x23, 0x5E, 0x52, 0x19, 0x9B, 0x19, 0x20, 0x1C,
                0x30, 0x30, 0x01, 0x78,
            ],
            "{} signed coordinate loads and additions",
            case.name
        );
        assert_eq!(thumb_call_target(&rom, leaf + 0x2E), updater);
    }
}

#[test]
fn native_entity_coordinate_getters_return_signed_q16_integer_parts() {
    for (case, dispatch, x_handler, y_handler, x_leaf, y_leaf, resolver) in [
        (
            &CASES[0], 0x3F904, 0x3FEB2, 0x3FEEC, 0x120A4, 0x120C4, 0xD3914,
        ),
        (
            &CASES[1], 0x3FAF0, 0x400CC, 0x40104, 0x1218C, 0x121AC, 0xDB53C,
        ),
        (
            &CASES[2], 0x3F578, 0x3FB26, 0x3FB60, 0x11F74, 0x11F94, 0xD30CC,
        ),
        (
            &CASES[3], 0x3F84C, 0x3FE28, 0x3FE60, 0x12030, 0x12050, 0xDB050,
        ),
    ] {
        let rom = fs::read(local_rom_path(case.rom)).unwrap();
        for (slot, handler, leaf) in [(3usize, x_handler, x_leaf), (4, y_handler, y_leaf)] {
            assert_eq!(
                u32::from_le_bytes(
                    rom[dispatch + slot * 4..dispatch + slot * 4 + 4]
                        .try_into()
                        .unwrap()
                ),
                0x0800_0000 + handler as u32,
                "{} coordinate slot {slot}",
                case.name
            );
            assert_eq!(thumb_call_target(&rom, handler + 0x30), leaf);
            assert_eq!(thumb_call_target(&rom, leaf + 0x0C), resolver);
            assert_eq!(
                &rom[leaf + 0x10..leaf + 0x18],
                &[
                    0x00,
                    0x21,
                    0x00,
                    0x28,
                    0x00,
                    0xD0,
                    if slot == 3 { 0x81 } else { 0xC1 },
                    0x68
                ],
                "{} coordinate field and absent-entity zero fallback",
                case.name
            );
            // ASR r0,r1,#16 returns the signed integer part of Q16.16.
            assert_eq!(&rom[leaf + 0x18..leaf + 0x1A], &[0x08, 0x14]);
        }
    }
}

#[test]
fn native_mfomt_fishing_record_getters_use_count_and_whole_centimeters() {
    const LEAVES: &[u8] = &[
        0xC9, 0x00, 0x40, 0x18, 0x00, 0x68, 0x70, 0x47, 0xC9, 0x00, 0x04, 0x30, 0x40, 0x18, 0x00,
        0x68, 0x70, 0x47,
    ];
    for (case, dispatch, handlers, leaves, screen, divide, remainder) in [
        (
            &CASES[1],
            0x3FAF0,
            [0x458A8, 0x458DC],
            [0xA1FA0, 0xA1FA8],
            0x76FD8,
            0xD8B76,
            0xD8B02,
        ),
        (
            &CASES[3],
            0x3F84C,
            [0x45604, 0x45638],
            [0xA19E0, 0xA19E8],
            0x76BBC,
            0xD868A,
            0xD8616,
        ),
    ] {
        let rom = fs::read(local_rom_path(case.rom)).unwrap();
        for (slot, handler, leaf) in [
            (0x14Eusize, handlers[0], leaves[0]),
            (0x14Fusize, handlers[1], leaves[1]),
        ] {
            assert_eq!(
                u32::from_le_bytes(
                    rom[dispatch + slot * 4..dispatch + slot * 4 + 4]
                        .try_into()
                        .unwrap()
                ),
                0x0800_0000 + handler as u32,
                "{} fishing-record slot {slot:#x}",
                case.name
            );
            assert_eq!(thumb_call_target(&rom, handler + 0x28), leaf);
        }
        assert_eq!(&rom[leaves[0]..leaves[0] + LEAVES.len()], LEAVES);

        // The record-book screen consumes the size getter, divides by 100 for
        // meters, and takes remainder 100 for the centimeter component.
        assert_eq!(thumb_call_target(&rom, screen), leaves[1]);
        assert_eq!(&rom[screen + 0x0C..screen + 0x0E], &[0x64, 0x21]);
        assert_eq!(thumb_call_target(&rom, screen + 0x0E), divide);
        assert_eq!(&rom[screen + 0x46..screen + 0x48], &[0x64, 0x21]);
        assert_eq!(thumb_call_target(&rom, screen + 0x48), remainder);
    }
}

#[test]
fn native_mfomt_maker_installation_queries_follow_blacksmith_order_completion() {
    const COOP_BIT_3: &[u8] = &[0x40, 0x78, 0x00, 0x07, 0xC0, 0x0F, 0x70, 0x47];
    const BARN_BIT_3: &[u8] = &[0x40, 0x78, 0x00, 0x07, 0xC0, 0x0F, 0x70, 0x47];
    const BARN_BIT_4: &[u8] = &[0x40, 0x78, 0xC0, 0x06, 0xC0, 0x0F, 0x70, 0x47];

    for (case, dispatch, handlers, leaves, completion, setters) in [
        (
            &CASES[1],
            0x3FAF0,
            [0x45830, 0x45852, 0x45874],
            [0xC638, 0xCF74, 0xCF7C],
            0x1140E,
            [0xC8FC, 0xD414, 0xD42C],
        ),
        (
            &CASES[3],
            0x3F84C,
            [0x4558C, 0x455AE, 0x455D0],
            [0xC5EC, 0xCF28, 0xCF30],
            0x113C2,
            [0xC8B0, 0xD3C8, 0xD3E0],
        ),
    ] {
        let rom = fs::read(local_rom_path(case.rom)).unwrap();
        for (index, (handler, leaf)) in handlers.into_iter().zip(leaves).enumerate() {
            let slot = 0x14B + index;
            assert_eq!(
                u32::from_le_bytes(
                    rom[dispatch + slot * 4..dispatch + slot * 4 + 4]
                        .try_into()
                        .unwrap()
                ),
                0x0800_0000 + handler as u32,
                "{} maker-installation slot {slot:#x}",
                case.name
            );
            assert_eq!(thumb_call_target(&rom, handler + 0x0E), leaf);
            assert_eq!(
                &rom[leaf..leaf + 8],
                [COOP_BIT_3, BARN_BIT_3, BARN_BIT_4][index],
                "{} maker-installation leaf {index}",
                case.name
            );
        }

        // The wrappers resolve the Coop at +0x420 or Barn at +0x600 before
        // calling the field getter. They do not resolve an indexed incubator
        // or pregnancy-stall record.
        assert_eq!(
            &rom[handlers[0] + 8..handlers[0] + 12],
            &[0x84, 0x24, 0xE4, 0x00]
        );
        assert_eq!(
            &rom[handlers[1] + 8..handlers[1] + 12],
            &[0xC0, 0x22, 0xD2, 0x00]
        );
        assert_eq!(
            &rom[handlers[2] + 8..handlers[2] + 12],
            &[0xC0, 0x26, 0xF6, 0x00]
        );

        // The completed blacksmith-order object selects the Mayonnaise,
        // Cheese and Yarn Maker orders (35, 36 and 37), rejects an already
        // installed maker, and then sets the matching persistent facility bit.
        assert_eq!(
            u16::from_le_bytes(
                rom[completion + 0x16..completion + 0x18]
                    .try_into()
                    .unwrap()
            ),
            0x2824
        );
        assert_eq!(
            u16::from_le_bytes(
                rom[completion + 0x1E..completion + 0x20]
                    .try_into()
                    .unwrap()
            ),
            0x2823
        );
        assert_eq!(
            u16::from_le_bytes(
                rom[completion + 0x2A..completion + 0x2C]
                    .try_into()
                    .unwrap()
            ),
            0x2825
        );
        for (call_offset, setter) in [0x44, 0x5E, 0x78].into_iter().zip(setters) {
            assert_eq!(
                thumb_call_target(&rom, completion + call_offset),
                setter,
                "{} maker installation setter at +{call_offset:X}",
                case.name
            );
        }
    }
}

#[test]
fn native_entity_facing_accessors_and_opposite_mapping_match_all_targets() {
    for (
        case,
        dispatch,
        set_handler,
        get_handler,
        opposite_handler,
        set_leaf,
        get_leaf,
        resolver,
        facing_setter,
        opposite_value_offsets,
    ) in [
        (
            &CASES[0],
            0x3F904,
            0x3FF26,
            0x4000A,
            0x411C4,
            0x120E4,
            0x12114,
            0xD3914,
            0x32198,
            [0x46, 0x5A, 0x6E, 0x82, 0x96],
        ),
        (
            &CASES[1],
            0x3FAF0,
            0x4013C,
            0x40220,
            0x413EE,
            0x121CC,
            0x121FC,
            0xDB53C,
            0x32554,
            [0x46, 0x5E, 0x76, 0x8E, 0xA6],
        ),
        (
            &CASES[2],
            0x3F578,
            0x3FB9A,
            0x3FC7E,
            0x40E38,
            0x11FB4,
            0x11FE4,
            0xD30CC,
            0x31F2C,
            [0x46, 0x5A, 0x6E, 0x82, 0x96],
        ),
        (
            &CASES[3],
            0x3F84C,
            0x3FE98,
            0x3FF7C,
            0x4114A,
            0x12070,
            0x120A0,
            0xDB050,
            0x323C8,
            [0x46, 0x5E, 0x76, 0x8E, 0xA6],
        ),
    ] {
        let rom = fs::read(local_rom_path(case.rom)).unwrap();
        for (slot, handler) in [
            (5usize, set_handler),
            (6, get_handler),
            (0x13, opposite_handler),
        ] {
            assert_eq!(
                u32::from_le_bytes(
                    rom[dispatch + slot * 4..dispatch + slot * 4 + 4]
                        .try_into()
                        .unwrap()
                ),
                0x0800_0000 + handler as u32,
                "{} facing slot {slot:#x}",
                case.name
            );
        }

        assert_eq!(thumb_call_target(&rom, set_handler + 0x3E), set_leaf);
        assert_eq!(thumb_call_target(&rom, get_handler + 0x30), get_leaf);
        assert_eq!(thumb_call_target(&rom, set_leaf + 0x0E), resolver);
        assert_eq!(thumb_call_target(&rom, set_leaf + 0x24), facing_setter);
        assert_eq!(thumb_call_target(&rom, get_leaf + 0x0C), resolver);

        // Setter rejects only absent/unchanged entities, then forwards the
        // unmasked byte. Getter reads byte +0x20 or returns zero when absent.
        assert_eq!(
            &rom[set_leaf + 0x12..set_leaf + 0x24],
            &[
                0x01, 0x1C, 0x00, 0x29, 0x07, 0xD0, 0x20, 0x30, 0x00, 0x78, 0xA0, 0x42, 0x03, 0xD0,
                0x08, 0x1C, 0x21, 0x1C
            ]
        );
        assert_eq!(
            &rom[get_leaf + 0x10..get_leaf + 0x1C],
            &[0x00, 0x28, 0x02, 0xD0, 0x20, 0x30, 0x00, 0x78, 0x00, 0xE0, 0x00, 0x20]
        );

        // Inputs 0,1,2,3 map to 1,0,3,2. The handler's default arm also
        // writes 1, so out-of-domain values are not preserved or rejected.
        for (offset, value) in opposite_value_offsets.into_iter().zip([1u8, 0, 3, 2, 1]) {
            assert_eq!(
                &rom[opposite_handler + offset..opposite_handler + offset + 2],
                &[value, 0x21]
            );
        }
    }
}

#[test]
fn native_entity_sprite_priority_stores_byte_and_renders_low_two_bits() {
    const RENDER_PRIORITY_PACKING: &[u8] = &[
        0x40, 0x46, 0x21, 0x30, 0x00, 0x78, 0x03, 0x21, 0x08, 0x40, 0x84, 0x00, 0x04, 0x43, 0x01,
        0x01, 0x0C, 0x43, 0x80, 0x01, 0x04, 0x43,
    ];
    for (case, dispatch, handler, leaf, resolver, renderer) in [
        (&CASES[0], 0x3F904, 0x3FFC4, 0x12154, 0xD3914, 0x327D4),
        (&CASES[1], 0x3FAF0, 0x401DA, 0x12240, 0xDB53C, 0x32B90),
        (&CASES[2], 0x3F578, 0x3FC38, 0x12024, 0xD30CC, 0x32568),
        (&CASES[3], 0x3F84C, 0x3FF36, 0x120E4, 0xDB050, 0x32A04),
    ] {
        let rom = fs::read(local_rom_path(case.rom)).unwrap();
        assert_eq!(
            u32::from_le_bytes(
                rom[dispatch + 7 * 4..dispatch + 7 * 4 + 4]
                    .try_into()
                    .unwrap()
            ),
            0x0800_0000 + handler as u32,
            "{} priority slot",
            case.name
        );
        assert_eq!(thumb_call_target(&rom, handler + 0x3E), leaf);
        assert_eq!(thumb_call_target(&rom, leaf + 0x0E), resolver);
        // If lookup succeeds, the native leaf stores the unmodified low byte
        // at AActorEntity +0x21; no range check or immediate refresh occurs.
        assert_eq!(
            &rom[leaf + 0x12..leaf + 0x1A],
            &[0x00, 0x28, 0x01, 0xD0, 0x21, 0x30, 0x04, 0x70]
        );
        // The render path masks that byte with 3 and replicates the two-bit
        // OBJ priority into the packed attributes of all four actor sprites.
        assert_eq!(
            &rom[renderer..renderer + RENDER_PRIORITY_PACKING.len()],
            RENDER_PRIORITY_PACKING
        );
    }
}

#[test]
fn native_entity_movement_normal_raw_and_wait_contracts_match_all_targets() {
    for (case, dispatch, handlers, leaves, resolver, setup_x, setup_y) in [
        (
            &CASES[0],
            0x3F904,
            [0x40044, 0x4009E, 0x400F6, 0x40150, 0x401A8],
            [0x12174, 0x1219C, 0x121C4],
            0xD3914,
            0x32308,
            0x3233C,
        ),
        (
            &CASES[1],
            0x3FAF0,
            [0x40258, 0x402B2, 0x4030A, 0x40364, 0x403BC],
            [0x12260, 0x12288, 0x122B0],
            0xDB53C,
            0x326C4,
            0x326F8,
        ),
        (
            &CASES[2],
            0x3F578,
            [0x3FCB8, 0x3FD12, 0x3FD6A, 0x3FDC4, 0x3FE1C],
            [0x12044, 0x1206C, 0x12094],
            0xD30CC,
            0x3209C,
            0x320D0,
        ),
        (
            &CASES[3],
            0x3F84C,
            [0x3FFB4, 0x4000E, 0x40066, 0x400C0, 0x40118],
            [0x12104, 0x1212C, 0x12154],
            0xDB050,
            0x32538,
            0x3256C,
        ),
    ] {
        let rom = fs::read(local_rom_path(case.rom)).unwrap();
        for (index, handler) in handlers.into_iter().enumerate() {
            let slot = 8 + index;
            assert_eq!(
                u32::from_le_bytes(
                    rom[dispatch + slot * 4..dispatch + slot * 4 + 4]
                        .try_into()
                        .unwrap()
                ),
                0x0800_0000 + handler as u32,
                "{} movement slot {slot:#x}",
                case.name
            );
        }

        // Normal X/Y handlers shift the speed operand left by 16 before the
        // shared leaf; Raw handlers pass the original integer unchanged.
        assert_eq!(&rom[handlers[0] + 0x50..handlers[0] + 0x52], &[0x1B, 0x04]);
        assert_eq!(thumb_call_target(&rom, handlers[0] + 0x52), leaves[0]);
        assert_eq!(thumb_call_target(&rom, handlers[1] + 0x50), leaves[0]);
        assert_eq!(&rom[handlers[2] + 0x50..handlers[2] + 0x52], &[0x1B, 0x04]);
        assert_eq!(thumb_call_target(&rom, handlers[2] + 0x52), leaves[1]);
        assert_eq!(thumb_call_target(&rom, handlers[3] + 0x50), leaves[1]);

        for leaf in leaves {
            let call_offset = if leaf == leaves[2] { 0x0E } else { 0x10 };
            assert_eq!(thumb_call_target(&rom, leaf + call_offset), resolver);
        }
        assert_eq!(thumb_call_target(&rom, leaves[0] + 0x1C), setup_x);
        assert_eq!(thumb_call_target(&rom, leaves[1] + 0x1C), setup_y);
        assert_eq!(thumb_call_target(&rom, handlers[4] + 0x26), leaves[2]);

        // Wait stores the resolved entity pointer at event state +0xE4. Only
        // a live entity causes interpreter state 0x16 to be written at +0x9C.
        assert_eq!(
            &rom[leaves[2] + 0x12..leaves[2] + 0x24],
            &[
                0x21, 0x1C, 0xE4, 0x31, 0x08, 0x60, 0x00, 0x28, 0x02, 0xD0, 0x48, 0x39, 0x16, 0x20,
                0x08, 0x60, 0x10, 0xBC
            ]
        );
    }
}

#[test]
fn native_entity_aux_profile_and_effect_slots_match_all_targets() {
    for (case, dispatch, handlers, leaves, resolver, native) in [
        (
            &CASES[0],
            0x3F904,
            [0x402FE, 0x40278, 0x402D0],
            [0x122BC, 0x12274, 0x122A0],
            0xD3914,
            [0x323E0, 0x32384, 0x323C8],
        ),
        (
            &CASES[1],
            0x3FAF0,
            [0x40512, 0x4048C, 0x404E4],
            [0x123AC, 0x12364, 0x12390],
            0xDB53C,
            [0x3279C, 0x32740, 0x32784],
        ),
        (
            &CASES[2],
            0x3F578,
            [0x3FF72, 0x3FEEC, 0x3FF44],
            [0x1218C, 0x12144, 0x12170],
            0xD30CC,
            [0x32174, 0x32118, 0x3215C],
        ),
        (
            &CASES[3],
            0x3F84C,
            [0x4026E, 0x401E8, 0x40240],
            [0x12250, 0x12208, 0x12234],
            0xDB050,
            [0x32610, 0x325B4, 0x325F8],
        ),
    ] {
        let rom = fs::read(local_rom_path(case.rom)).unwrap();
        for (index, handler) in handlers.into_iter().enumerate() {
            let slot = 0x10 + index;
            assert_eq!(
                u32::from_le_bytes(
                    rom[dispatch + slot * 4..dispatch + slot * 4 + 4]
                        .try_into()
                        .unwrap()
                ),
                0x0800_0000 + handler as u32,
                "{} entity effect slot {slot:#x}",
                case.name
            );
        }
        assert_eq!(thumb_call_target(&rom, handlers[0] + 0x3E), leaves[0]);
        assert_eq!(thumb_call_target(&rom, handlers[1] + 0x50), leaves[1]);
        assert_eq!(thumb_call_target(&rom, handlers[2] + 0x26), leaves[2]);
        for (leaf, native_leaf, call_offset) in [
            (leaves[0], native[0], 0x18),
            (leaves[1], native[1], 0x20),
            (leaves[2], native[2], 0x14),
        ] {
            let resolver_offset = if leaf == leaves[2] {
                0x0C
            } else {
                if leaf == leaves[1] {
                    0x10
                } else {
                    0x0E
                }
            };
            assert_eq!(thumb_call_target(&rom, leaf + resolver_offset), resolver);
            assert_eq!(thumb_call_target(&rom, leaf + call_offset), native_leaf);
        }

        // Start converts every nonzero third operand to one before the native
        // routine converts false/true into active modes 1/2.
        assert_eq!(
            &rom[leaves[1] + 0x18..leaves[1] + 0x1E],
            &[0x62, 0x42, 0x22, 0x43, 0xD2, 0x0F]
        );
        assert_eq!(
            &rom[native[1]..native[1] + 6],
            &[0x30, 0xB5, 0x12, 0x06, 0x15, 0x0E]
        );
        assert_eq!(
            &rom[native[1] + 0x26..native[1] + 0x32],
            &[0x01, 0x23, 0x00, 0x2D, 0x00, 0xD0, 0x02, 0x23, 0x22, 0x1C, 0x8A, 0x32]
        );

        // Stop immediately clears active-mode bits 0-1 at controller +0x8A.
        assert_eq!(
            &rom[native[2]..native[2] + 0x14],
            &[
                0x00, 0xB5, 0x02, 0x69, 0x00, 0x2A, 0x05, 0xD0, 0x8A, 0x32, 0x11, 0x78, 0x04, 0x20,
                0x40, 0x42, 0x08, 0x40, 0x10, 0x70
            ]
        );
        // Auxiliary profile stores its unvalidated low byte at controller +0x88.
        assert_eq!(
            &rom[native[0]..native[0] + 0x0E],
            &[0x00, 0xB5, 0x00, 0x69, 0x00, 0x28, 0x01, 0xD0, 0x88, 0x30, 0x01, 0x70, 0x01, 0xBC]
        );
    }
}

#[test]
fn native_entity_location_and_change_map_use_packed_local_coordinates() {
    let reference = fs::read(local_rom_path(CASES[0].rom)).unwrap();
    for (
        case,
        dispatch,
        get_handler,
        get_leaf,
        change_handler,
        change_leaf,
        resolver,
        transition,
    ) in [
        (
            &CASES[0], 0x3F904, 0x41260, 0x12134, 0x40344, 0x122E0, 0xD3914, 0xD7F08,
        ),
        (
            &CASES[1], 0x3FAF0, 0x4149E, 0x1221C, 0x40558, 0x123D0, 0xDB53C, 0xE0340,
        ),
        (
            &CASES[2], 0x3F578, 0x40ED4, 0x12004, 0x3FFB8, 0x121B0, 0xD30CC, 0xD76C0,
        ),
        (
            &CASES[3], 0x3F84C, 0x411FA, 0x120C0, 0x402B4, 0x12274, 0xDB050, 0xDFE54,
        ),
    ] {
        let rom = fs::read(local_rom_path(case.rom)).unwrap();
        for (slot, handler) in [(0x14usize, get_handler), (0x16, change_handler)] {
            assert_eq!(
                u32::from_le_bytes(
                    rom[dispatch + slot * 4..dispatch + slot * 4 + 4]
                        .try_into()
                        .unwrap()
                ),
                0x0800_0000 + handler as u32,
                "{} map callable slot {slot:#x}",
                case.name
            );
        }
        assert_eq!(thumb_call_target(&rom, get_handler + 0x30), get_leaf);
        assert_eq!(thumb_call_target(&rom, get_leaf + 0x0C), resolver);
        let missing_map = if case.target.contains("MFOMT") {
            0x23A
        } else {
            0x234
        };
        if case.target.contains("MFOMT") {
            assert_eq!(
                &rom[get_leaf + 0x12..get_leaf + 0x18],
                &[0x00, 0x28, 0x00, 0xD0, 0x81, 0x88]
            );
            assert_eq!(
                u32::from_le_bytes(rom[get_leaf + 0x20..get_leaf + 0x24].try_into().unwrap()),
                missing_map
            );
        } else {
            assert_eq!(
                &rom[get_leaf + 0x10..get_leaf + 0x14],
                &[0x8D, 0x21, 0x89, 0x00]
            );
            assert_eq!(
                &rom[get_leaf + 0x14..get_leaf + 0x1A],
                &[0x00, 0x28, 0x00, 0xD0, 0x81, 0x88]
            );
        }

        assert_eq!(thumb_call_target(&rom, change_handler + 0x4C), change_leaf);
        // Before the target-specific transition call, all targets use the
        // same packing operations: map10, signed-x16 and signed-y16.
        assert_eq!(
            &rom[change_leaf..change_leaf + 0x8A],
            &reference[0x122E0..0x1236A]
        );
        assert_eq!(
            u32::from_le_bytes(
                rom[change_leaf + 0xA4..change_leaf + 0xA8]
                    .try_into()
                    .unwrap()
            ),
            0x3FF
        );
        assert_eq!(
            u32::from_le_bytes(
                rom[change_leaf + 0xA8..change_leaf + 0xAC]
                    .try_into()
                    .unwrap()
            ),
            0xFFFFFC00
        );
        assert_eq!(thumb_call_target(&rom, change_leaf + 0x90), transition);
    }
}

#[test]
fn native_mfomt_map_registry_query_scans_counted_records_in_both_regions() {
    let mut handlers = Vec::new();
    let mut leaves = Vec::new();

    for (case, dispatch, expected_handler, leaf) in [
        (&CASES[1], 0x3FAF0usize, 0x45924usize, 0x9F890usize),
        (&CASES[3], 0x3F84C, 0x45680, 0x9F2D0),
    ] {
        let rom = fs::read(local_rom_path(case.rom)).unwrap();
        let entry = dispatch + 0x150 * 4;
        assert_eq!(
            u32::from_le_bytes(rom[entry..entry + 4].try_into().unwrap()),
            0x0800_0000 + expected_handler as u32,
            "{} map-registry callable slot",
            case.name
        );

        // The handler pops one halfword map ID, scans a counted collection,
        // copies each 20-byte record through the leaf, compares field +0,
        // and pushes one on a match or zero after exhausting the collection.
        assert_eq!(
            &rom[expected_handler + 0x0C..expected_handler + 0x16],
            &[0x98, 0x00, 0x04, 0x38, 0x10, 0x18, 0x00, 0x88, 0x14, 0x90]
        );
        assert_eq!(
            &rom[expected_handler + 0x84..expected_handler + 0x9C],
            &[
                0x90, 0x42, 0x0F, 0xD1, 0x3A, 0x68, 0x63, 0x2A, 0x14, 0xD8, 0x90, 0x00, 0x40, 0x44,
                0x01, 0x21, 0x01, 0x60, 0x50, 0x1C, 0x38, 0x60, 0x0D, 0xE0
            ]
        );
        assert_eq!(
            &rom[expected_handler + 0xCC..expected_handler + 0xE0],
            &[
                0x3B, 0x68, 0x63, 0x2B, 0x00, 0xD9, 0x47, 0xE1, 0x98, 0x00, 0x40, 0x44, 0x00, 0x21,
                0x01, 0x60, 0x58, 0x1C, 0x40, 0xE1
            ]
        );
        assert_eq!(
            [
                u32::from_le_bytes(
                    rom[expected_handler + 0x9C..expected_handler + 0xA0]
                        .try_into()
                        .unwrap(),
                ),
                u32::from_le_bytes(
                    rom[expected_handler + 0xA0..expected_handler + 0xA4]
                        .try_into()
                        .unwrap(),
                ),
                u32::from_le_bytes(
                    rom[expected_handler + 0xA4..expected_handler + 0xA8]
                        .try_into()
                        .unwrap(),
                ),
            ],
            [0x2C57, 0xFFFFF000, 0x2250],
            "{} registry count/record-mask/record-base literals",
            case.name
        );
        assert_eq!(thumb_call_target(&rom, expected_handler + 0x7A), leaf);

        handlers.push(rom[expected_handler..expected_handler + 0xE4].to_vec());
        leaves.push(rom[leaf..leaf + 74].to_vec());
    }

    // The only regional handler difference is the relocated BL encoding.
    let jp_call = handlers[1][0x7C..0x7E].to_vec();
    handlers[0][0x7C..0x7E].copy_from_slice(&jp_call);
    assert_eq!(handlers[0], handlers[1]);
    assert_eq!(leaves[0], leaves[1]);
    assert_eq!(
        &leaves[0][..10],
        &[0x10, 0xB5, 0x93, 0x00, 0x9B, 0x18, 0x9B, 0x00, 0xC0, 0x18],
        "record address is base + index * 20"
    );
    assert_eq!(&leaves[0][10..14], &[0x02, 0x88, 0x0A, 0x80]);
}

#[test]
fn native_link_milestone_callables_keep_separate_local_and_received_bitsets() {
    let mut bodies = Vec::new();
    for (case, dispatch, first_slot, starts, local_offset, received_offset) in [
        (
            &CASES[0],
            0x3F904usize,
            0x77usize,
            [0x41A44usize, 0x41A8C, 0x41AD4, 0x41B18],
            0x21DCu32,
            0x21D4u32,
        ),
        (
            &CASES[2],
            0x3F578,
            0x77,
            [0x416B8, 0x41700, 0x41748, 0x4178C],
            0x21DC,
            0x21D4,
        ),
        (
            &CASES[1],
            0x3FAF0,
            0x7A,
            [0x41C8C, 0x41CD4, 0x41D1C, 0x41D60],
            0x2215,
            0x220C,
        ),
        (
            &CASES[3],
            0x3F84C,
            0x7A,
            [0x419E8, 0x41A30, 0x41A78, 0x41ABC],
            0x2215,
            0x220C,
        ),
    ] {
        let rom = fs::read(local_rom_path(case.rom)).unwrap();
        for (index, &start) in starts.iter().enumerate() {
            let entry = dispatch + (first_slot + index) * 4;
            assert_eq!(
                u32::from_le_bytes(rom[entry..entry + 4].try_into().unwrap()),
                0x0800_0000 + start as u32,
                "{} link-milestone slot {:#x}",
                case.name,
                first_slot + index
            );
        }

        assert_eq!(
            u32::from_le_bytes(rom[starts[0] + 0x44..starts[0] + 0x48].try_into().unwrap()),
            local_offset,
            "{} local getter bitset",
            case.name
        );
        assert_eq!(
            u32::from_le_bytes(rom[starts[1] + 0x44..starts[1] + 0x48].try_into().unwrap()),
            received_offset,
            "{} received getter bitset",
            case.name
        );
        for start in [starts[2], starts[3]] {
            assert_eq!(
                u32::from_le_bytes(rom[start + 0x40..start + 0x44].try_into().unwrap()),
                local_offset,
                "{} local writer bitset",
                case.name
            );
        }

        // Both getters compute byte=id/8 and bit=id%8, then normalize to 0/1.
        for start in [starts[0], starts[1]] {
            assert_eq!(
                &rom[start + 0x24..start + 0x40],
                &[
                    0x1A, 0x1C, 0x00, 0x2B, 0x00, 0xDA, 0xDA, 0x1D, 0xD2, 0x10, 0xD1, 0x00, 0x59,
                    0x1A, 0x04, 0x4B, 0xC0, 0x18, 0x80, 0x18, 0x02, 0x78, 0x0A, 0x41, 0x01, 0x20,
                    0x02, 0x40,
                ],
                "{} link getter bit arithmetic",
                case.name
            );
        }
        // The writers target only the local bitset: OR sets, BIC clears.
        assert_eq!(
            &rom[starts[2] + 0x32..starts[2] + 0x3C],
            &[0x01, 0x20, 0x88, 0x40, 0x11, 0x78, 0x08, 0x43, 0x10, 0x70]
        );
        assert_eq!(
            &rom[starts[3] + 0x32..starts[3] + 0x3C],
            &[0x01, 0x21, 0x81, 0x40, 0x10, 0x78, 0x88, 0x43, 0x10, 0x70]
        );

        bodies.push([
            rom[starts[0]..starts[0] + 72].to_vec(),
            rom[starts[1]..starts[1] + 72].to_vec(),
            rom[starts[2]..starts[2] + 68].to_vec(),
            rom[starts[3]..starts[3] + 68].to_vec(),
        ]);
    }

    assert_eq!(bodies[0], bodies[1], "FoMT regional implementations");
    assert_eq!(bodies[2], bodies[3], "MFoMT regional implementations");
}

#[test]
fn native_farming_tutorial_callable_forwards_one_kind_and_waits_on_all_targets() {
    for (case, dispatch, slot, handler, lifecycle, tutorial_task) in [
        (
            &CASES[0],
            0x3F904usize,
            0xA9usize,
            0x425E2usize,
            0x4168Ausize,
            0x13EF8usize,
        ),
        (&CASES[2], 0x3F578, 0xA9, 0x42256, 0x412FE, 0x13DCC),
        (&CASES[1], 0x3FAF0, 0xAC, 0x42828, 0x418CE, 0x14084),
        (&CASES[3], 0x3F84C, 0xAC, 0x42584, 0x4162A, 0x13F2C),
    ] {
        let rom = fs::read(local_rom_path(case.rom)).unwrap();
        let entry = dispatch + slot * 4;
        assert_eq!(
            u32::from_le_bytes(rom[entry..entry + 4].try_into().unwrap()),
            0x0800_0000 + handler as u32,
            "{} farming-tutorial slot",
            case.name
        );
        // Exactly one VM word is loaded and the stack depth is decremented.
        assert_eq!(
            &rom[handler..handler + 0x16],
            &[
                0xDA, 0x26, 0x76, 0x00, 0xAB, 0x19, 0x1A, 0x68, 0x90, 0x00, 0x40, 0x19, 0x01, 0x6A,
                0x00, 0x2A, 0x01, 0xD0, 0x50, 0x1E, 0x18, 0x60,
            ],
            "{} one tutorial-kind operand",
            case.name
        );
        // If no modal task is active, create/run the tutorial task, then pass
        // through the same lifecycle helper again to wait for its completion.
        assert_eq!(thumb_call_target(&rom, handler + 0x22), lifecycle);
        assert_eq!(thumb_call_target(&rom, handler + 0x26), tutorial_task);
        assert_eq!(thumb_call_target(&rom, handler + 0x2A), lifecycle);
    }
}

#[test]
fn native_staff_credits_installs_region_state_and_yields_on_all_targets() {
    for (
        case,
        dispatch,
        slot,
        handler,
        lifecycle,
        transition_task,
        selector,
        task_update,
        transition_object_ctor,
        credits_viewer_ctor,
        credits_lines,
    ) in [
        (
            &CASES[0],
            0x3F904usize,
            0xA8usize,
            0x425CAusize,
            0x4168Ausize,
            0x13E6Cusize,
            0x2Eu8,
            0xDBC0Cusize,
            0x77C40usize,
            0x773E8usize,
            0x0FC4B4usize,
        ),
        (
            &CASES[2], 0x3F578, 0xA8, 0x4223E, 0x412FE, 0x13D40, 0x2D, 0xDB3C0, 0x777C8, 0x77020,
            0x0FBC88,
        ),
        (
            &CASES[1], 0x3FAF0, 0xAB, 0x42810, 0x418CE, 0x13FF8, 0x2E, 0xE4130, 0x7CA04, 0x7C248,
            0x105150,
        ),
        (
            &CASES[3], 0x3F84C, 0xAB, 0x4256C, 0x4162A, 0x13EA0, 0x2D, 0xE3C40, 0x7C5C4, 0x7BE0C,
            0x10506C,
        ),
    ] {
        let rom = fs::read(local_rom_path(case.rom)).unwrap();
        let entry = dispatch + slot * 4;
        assert_eq!(
            u32::from_le_bytes(rom[entry..entry + 4].try_into().unwrap()),
            0x0800_0000 + handler as u32,
            "{} staff-credits slot",
            case.name
        );

        // This handler consumes no VM operand. It checks the active state
        // object, yields if necessary, installs the regional state, and yields.
        assert_eq!(
            &rom[handler..handler + 0x0C],
            &[0xD5, 0x24, 0xA4, 0x00, 0x28, 0x19, 0x00, 0x68, 0x00, 0x28, 0x01, 0xD1],
            "{} parameterless state-object check",
            case.name
        );
        assert_eq!(thumb_call_target(&rom, handler + 0x0C), lifecycle);
        assert_eq!(thumb_call_target(&rom, handler + 0x10), transition_task);
        assert_eq!(thumb_call_target(&rom, handler + 0x14), lifecycle);

        // The native task constructor embeds the regional selector and writes
        // 24 to the game-state transition field after replacing the object.
        // The internal selector is 0x2E in US and 0x2D in JP because the
        // region-specific native state enum is shifted; it is not a Mary ID.
        assert_eq!(
            &rom[transition_task..transition_task + 0x0C],
            &[0xF0, 0xB5, 0x87, 0xB0, 0x46, 0x68, selector, 0x20, 0x00, 0x90, 0x14, 0x20],
            "{} regional state constructor prefix",
            case.name
        );
        assert_eq!(
            &rom[transition_task + 0x62..transition_task + 0x68],
            &[0x31, 0x1C, 0x9C, 0x31, 0x18, 0x20],
            "{} transition value 24",
            case.name
        );

        // The installed task is not a passive marker. Its vtable update method
        // transfers ownership of the wrapped state object into a dedicated
        // credits viewer. The wrapper allocates 0xE6 << 3 = 0x730 bytes and
        // the viewer constructor binds the target's staff-credit line table.
        let task_vtable = u32::from_le_bytes(
            rom[transition_task + 0x88..transition_task + 0x8C]
                .try_into()
                .unwrap(),
        );
        let task_vtable = (task_vtable - 0x0800_0000) as usize;
        assert_eq!(
            u32::from_le_bytes(
                rom[task_vtable + 0x0C..task_vtable + 0x10]
                    .try_into()
                    .unwrap()
            ),
            0x0800_0001 + task_update as u32,
            "{} transition-task update vtable",
            case.name
        );
        assert_eq!(
            thumb_call_target(&rom, task_update + 0x16),
            transition_object_ctor,
            "{} dedicated transition-object constructor",
            case.name
        );
        assert_eq!(
            &rom[transition_object_ctor + 0x0A..transition_object_ctor + 0x0E],
            &[0xE6, 0x20, 0xC0, 0x00],
            "{} transition-object allocation size 0x730",
            case.name
        );
        assert_eq!(
            thumb_call_target(&rom, transition_object_ctor + 0x12),
            credits_viewer_ctor,
            "{} credits-viewer constructor",
            case.name
        );
        assert_eq!(
            u32::from_le_bytes(
                rom[credits_viewer_ctor + 0xA4..credits_viewer_ctor + 0xA8]
                    .try_into()
                    .unwrap()
            ),
            0x0800_0000 + credits_lines as u32,
            "{} staff-credits line table",
            case.name
        );
        let first_credit_line =
            u32::from_le_bytes(rom[credits_lines..credits_lines + 4].try_into().unwrap());
        assert!(
            (0x0800_0000..0x0A00_0000).contains(&first_credit_line),
            "{} first staff-credit line pointer",
            case.name
        );
    }
}

#[test]
fn native_can_discard_held_article_checks_presence_kind_and_article_policy() {
    for (case, dispatch, slot, handler, held_none, held_kind, article_id, can_discard) in [
        (
            &CASES[0],
            0x3F904usize,
            0x4Busize,
            0x412B8usize,
            0x0F190usize,
            0x0F204usize,
            0x0F258usize,
            0x0DFB0usize,
        ),
        (
            &CASES[2], 0x3F578, 0x4B, 0x40F2C, 0x0F170, 0x0F1E4, 0x0F238, 0x0DF90,
        ),
        (
            &CASES[1], 0x3FAF0, 0x4C, 0x414FC, 0x0F26C, 0x0F2E0, 0x0F334, 0x0E024,
        ),
        (
            &CASES[3], 0x3F84C, 0x4C, 0x41258, 0x0F220, 0x0F294, 0x0F2E8, 0x0DFD8,
        ),
    ] {
        let rom = fs::read(local_rom_path(case.rom)).unwrap();
        let entry = dispatch + slot * 4;
        assert_eq!(
            u32::from_le_bytes(rom[entry..entry + 4].try_into().unwrap()),
            0x0800_0000 + handler as u32,
            "{} held-article discard slot",
            case.name
        );

        // The wrapper has no VM operand. It queries the held-item object in
        // this order: empty, kind, article ID, then Article::CanBeDiscarded.
        assert_eq!(thumb_call_target(&rom, handler + 0x0E), held_none);
        assert_eq!(thumb_call_target(&rom, handler + 0x1A), held_kind);
        assert_eq!(thumb_call_target(&rom, handler + 0x26), article_id);
        assert_eq!(thumb_call_target(&rom, handler + 0x30), can_discard);
        assert_eq!(
            &rom[handler + 0x12..handler + 0x1A],
            &[0x00, 0x06, 0x00, 0x28, 0x21, 0xD1, 0x20, 0x1C],
            "{} empty-held-item gate",
            case.name
        );
        assert_eq!(
            &rom[handler + 0x1E..handler + 0x24],
            &[0x06, 0x1C, 0x01, 0x2E, 0x1B, 0xD1],
            "{} article-kind gate",
            case.name
        );
        assert_eq!(
            &rom[handler + 0x34..handler + 0x3A],
            &[0x00, 0x06, 0x00, 0x28, 0x10, 0xD0],
            "{} discard-policy boolean gate",
            case.name
        );
        // Success pushes the already-proven kind value 1; every rejected path
        // joins the common false path that pushes 0.
        assert_eq!(
            &rom[handler + 0x4E..handler + 0x54],
            &[0x98, 0x00, 0x10, 0x18, 0x06, 0x60]
        );
        assert_eq!(
            &rom[handler + 0x5C..handler + 0x60],
            &[0x29, 0x1C, 0x24, 0x31]
        );
    }
}

#[test]
fn all_four_roms_mary_c_round_trip() -> Result<(), Box<dyn std::error::Error>> {
    for case in CASES {
        verify(case)?;
    }
    Ok(())
}
