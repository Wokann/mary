use std::collections::{HashMap, HashSet};

use thiserror::Error;

#[derive(Clone, Debug, Error, PartialEq, Eq)]
pub enum CharmapError {
    #[error("charmap line {line}: expected HEX=TEXT")]
    MalformedLine { line: usize },
    #[error("charmap line {line}: invalid hexadecimal byte sequence '{value}'")]
    InvalidHex { line: usize, value: String },
    #[error("charmap line {line}: byte sequence {value} is defined more than once")]
    DuplicateBytes { line: usize, value: String },
    #[error("charmap line {line}: mapped text must not be empty")]
    EmptyText { line: usize },
    #[error("charmap line {line}: byte sequence {value} contains reserved STR terminator 00")]
    ReservedNull { line: usize, value: String },
    #[error("text contains a character sequence not present in the active charmap: '{0}'")]
    UnmappedText(String),
    #[error("string literals cannot contain the reserved STR terminator byte 00")]
    EmbeddedNull,
}

/// Replaceable mapping between encoded ROM bytes and UTF-8 source text.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct Charmap {
    decode: HashMap<Vec<u8>, String>,
    encode: HashMap<String, Vec<u8>>,
    ambiguous_text: HashSet<String>,
    multibyte_lengths: HashMap<u8, Vec<usize>>,
    max_bytes: usize,
    max_chars: usize,
}

impl Charmap {
    pub fn parse(source: &str) -> Result<Self, CharmapError> {
        let mut result = Self::default();
        let mut candidates: HashMap<String, Vec<Vec<u8>>> = HashMap::new();

        for (index, raw) in source.lines().enumerate() {
            let line = index + 1;
            let raw = raw.trim_end_matches('\r');
            if raw.trim().is_empty() || raw.trim_start().starts_with('#') {
                continue;
            }

            let (hex, text) = raw
                .split_once('=')
                .ok_or(CharmapError::MalformedLine { line })?;
            if hex.is_empty() || hex.len() % 2 != 0 || !hex.bytes().all(|b| b.is_ascii_hexdigit()) {
                return Err(CharmapError::InvalidHex {
                    line,
                    value: hex.into(),
                });
            }
            // TBL files commonly keep unused code points as `HEX=` slots.
            // They do not describe a character and therefore take no part in
            // either decoding or encoding.
            if text.is_empty() {
                continue;
            }
            let text = text.to_owned();

            let bytes = (0..hex.len())
                .step_by(2)
                .map(|offset| u8::from_str_radix(&hex[offset..offset + 2], 16))
                .collect::<Result<Vec<_>, _>>()
                .map_err(|_| CharmapError::InvalidHex {
                    line,
                    value: hex.into(),
                })?;

            if bytes.contains(&0) {
                return Err(CharmapError::ReservedNull {
                    line,
                    value: hex.into(),
                });
            }

            if result.decode.insert(bytes.clone(), text.clone()).is_some() {
                return Err(CharmapError::DuplicateBytes {
                    line,
                    value: hex.into(),
                });
            }

            result.max_bytes = result.max_bytes.max(bytes.len());
            result.max_chars = result.max_chars.max(text.chars().count());
            if bytes.len() > 1 {
                let lengths = result.multibyte_lengths.entry(bytes[0]).or_default();
                if !lengths.contains(&bytes.len()) {
                    lengths.push(bytes.len());
                    lengths.sort_unstable();
                }
            }
            candidates.entry(text).or_default().push(bytes);
        }

        for (text, encodings) in candidates {
            // The first line is the canonical encoding for source text. Other
            // byte sequences with the same Unicode text stay explicit when
            // decompiled because the glyph alone cannot distinguish them.
            result.encode.insert(text.clone(), encodings[0].clone());
            if encodings.len() > 1 {
                result.ambiguous_text.insert(text);
            }
        }

        Ok(result)
    }

    /// Encode UTF-8 source text using longest textual matches.
    pub fn encode_text(&self, text: &str) -> Result<Vec<u8>, CharmapError> {
        let chars: Vec<char> = text.chars().collect();
        let mut output = Vec::new();
        let mut at = 0;

        while at < chars.len() {
            let mut matched = None;
            for count in (1..=self.max_chars.min(chars.len() - at)).rev() {
                let candidate: String = chars[at..at + count].iter().collect();
                if let Some(bytes) = self.encode.get(&candidate) {
                    matched = Some((count, bytes));
                    break;
                }
            }

            let Some((count, bytes)) = matched else {
                return Err(CharmapError::UnmappedText(chars[at].to_string()));
            };
            output.extend(bytes);
            at += count;
        }

        Ok(output)
    }

    /// Decode one longest byte sequence. Ambiguous Unicode mappings are kept
    /// explicit by the caller so printing and parsing cannot change the bytes.
    pub fn decode_one(&self, bytes: &[u8]) -> Option<(usize, &str)> {
        for count in (1..=self.max_bytes.min(bytes.len())).rev() {
            if let Some(text) = self.decode.get(&bytes[..count]) {
                if !self.ambiguous_text.contains(text) {
                    return Some((count, text));
                }
            }
        }
        None
    }

    /// Return the number of bytes that must stay together when a sequence
    /// cannot be rendered as mapped text. Both exact-but-ambiguous mappings
    /// and unknown sequences beginning with a configured multibyte prefix are
    /// emitted as one run of explicit `\\xNN` escapes by the pretty-printer.
    pub fn raw_sequence_len(&self, bytes: &[u8]) -> usize {
        if bytes.is_empty() {
            return 0;
        }

        for count in (2..=self.max_bytes.min(bytes.len())).rev() {
            if self.decode.contains_key(&bytes[..count]) {
                return count;
            }
        }

        self.multibyte_lengths
            .get(&bytes[0])
            .and_then(|lengths| {
                lengths
                    .iter()
                    .copied()
                    .find(|&length| length <= bytes.len())
            })
            .unwrap_or(1)
    }

    /// Whether mapped text is a source-level named escape such as `\\n`,
    /// `\\p`, or `\\l`. Its spelling and byte encoding come exclusively from
    /// the loaded map; the parser does not assign meanings to escape names.
    pub fn is_named_escape(&self, text: &str) -> bool {
        let mut chars = text.chars();
        chars.next() == Some('\\') && chars.next().is_some() && chars.next().is_none()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn unique_text_round_trips_and_ambiguous_bytes_stay_explicit() {
        let map = Charmap::parse("41=A\n8140=　\nFA40=dup\nFA41=dup\n").unwrap();
        assert_eq!(map.encode_text("A　").unwrap(), vec![0x41, 0x81, 0x40]);
        assert_eq!(map.decode_one(&[0x81, 0x40]), Some((2, "　")));
        assert_eq!(map.decode_one(&[0xFA, 0x41]), None);
    }

    #[test]
    fn skips_unassigned_slots_but_keeps_whitespace_mappings() {
        let map = Charmap::parse("41=A\n42=\n20= \n").unwrap();

        assert_eq!(map.decode_one(&[0x41]), Some((1, "A")));
        assert_eq!(map.decode_one(&[0x42]), None);
        assert_eq!(map.decode_one(&[0x20]), Some((1, " ")));
        assert_eq!(map.encode_text("A ").unwrap(), vec![0x41, 0x20]);
    }

    #[test]
    fn rejects_unmapped_source_text() {
        let map = Charmap::parse("41=A\n").unwrap();
        assert_eq!(
            map.encode_text("B"),
            Err(CharmapError::UnmappedText("B".into()))
        );
    }

    #[test]
    fn multi_byte_keys_keep_the_file_byte_order() {
        let map = Charmap::parse("8140=　\n82A0=あ\n").unwrap();
        assert_eq!(
            map.encode_text("　あ").unwrap(),
            vec![0x81, 0x40, 0x82, 0xA0]
        );
        assert_eq!(map.decode_one(&[0x81, 0x40]), Some((2, "　")));
        assert_eq!(map.decode_one(&[0x40, 0x81]), None);
    }

    #[test]
    fn textual_escapes_use_the_map_but_hex_escapes_stay_raw() {
        use std::sync::Arc;

        let map = Arc::new(Charmap::parse("AA=\"\nAB=\\\nFE=\\n\n0102=\\p\n").unwrap());
        let source = r#"
            proc 1 Put(value : string)
            script 1 Escapes
            {
                Put("\"\\\n"
                    "\p\x22")
            }
        "#;
        let parsed = crate::compiler::parse_string_with_charmap(source, map).unwrap();
        assert_eq!(
            parsed.scripts[0].2.strings,
            vec![vec![0xAA, 0xAB, 0xFE, 0x01, 0x02, 0x22]]
        );
    }

    #[test]
    fn named_escapes_are_data_driven() {
        let map = Charmap::parse("0102=\\p\n030405=\\l\n").unwrap();

        assert_eq!(map.encode_text("\\p\\l").unwrap(), vec![1, 2, 3, 4, 5]);
        assert_eq!(map.decode_one(&[1, 2]), Some((2, "\\p")));
        assert!(map.is_named_escape("\\p"));
        assert!(map.is_named_escape("\\l"));
        assert!(!map.is_named_escape("\\"));
    }

    #[test]
    fn raw_sequence_length_is_derived_from_the_loaded_map() {
        let map = Charmap::parse("20= \nAA01={control}\nBB0102={wide}\n").unwrap();

        assert_eq!(map.raw_sequence_len(&[0xAA, 0x20]), 2);
        assert_eq!(map.raw_sequence_len(&[0xBB, 0xDD, 0xEE]), 3);
        assert_eq!(map.raw_sequence_len(&[0xCC, 0x20]), 1);
        assert_eq!(map.raw_sequence_len(&[0xAA]), 1);
    }

    #[test]
    fn rejects_mappings_that_embed_the_str_terminator() {
        assert_eq!(
            Charmap::parse("4100=invalid\n"),
            Err(CharmapError::ReservedNull {
                line: 1,
                value: "4100".into(),
            })
        );
        // Empty TBL slots remain legal because they do not define a mapping.
        assert!(Charmap::parse("00=\n").is_ok());
    }

    #[test]
    fn rejects_raw_null_escape_with_or_without_a_charmap() {
        use std::sync::Arc;

        let source = r#"script 1 Invalid { const MESSAGE = "\x00" }"#;
        assert!(matches!(
            crate::compiler::parse_string(source),
            Err(crate::compiler::ScriptError::Charmap(
                CharmapError::EmbeddedNull
            ))
        ));

        let map = Arc::new(Charmap::parse("41=A\n").unwrap());
        assert!(matches!(
            crate::compiler::parse_string_with_charmap(source, map),
            Err(crate::compiler::ScriptError::Charmap(
                CharmapError::EmbeddedNull
            ))
        ));
    }
}
