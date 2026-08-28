# mary

[English](README.md) | [简体中文](README.zh-CN.md)

`mary` is a script compiler and decompiler for the US and Japanese GBA releases of *Harvest Moon: Friends of Mineral Town* (FoMT) and *Harvest Moon: More Friends of Mineral Town* (MFoMT).

```text
RIFF bytecode in ROM
        ↓ decompile
Editable structured source (UTF-8)
        ↓ compile
Byte-exact RIFF/HEX
```

All 5,486 valid vanilla scripts from the four supported ROMs pass the strict source round trip: decode, structured decompile, print source, parse source, compile, encode, and compare byte-for-byte with the original RIFF. None currently requires low-level `ir` or retains `jump next`.

| Version | Valid scripts | Structured source | Byte-exact round trip |
| --- | ---: | ---: | ---: |
| FoMT US | 1,328 | 1,328/1,328 | 1,328/1,328 |
| MFoMT US | 1,415 | 1,415/1,415 | 1,415/1,415 |
| FoMT JP | 1,328 | 1,328/1,328 | 1,328/1,328 |
| MFoMT JP | 1,415 | 1,415/1,415 | 1,415/1,415 |

> This 100% figure describes the four tested vanilla corpora. Arbitrary handwritten programs must still use structures supported by the VM and pass compiler validation.

## Build

Install the Rust toolchain, then run:

```console
cargo build --release
```

The release executable is written to `target/release/mary.exe`.

## Command-line usage

### Decompile every script

```console
mary decompile ROM LIBRARY --all --charmap charmap_jp.txt -o OUTPUT_DIRECTORY
```

Example:

```console
mary decompile rom/fomtjp.gba goodies/lib_fomt.txt --all --charmap charmap_jp.txt -o decompiled_text/fomt_jp
```

`--all` writes one file per pointer-table slot, named `EventScript_0000.mary`, `EventScript_0001.mary`, and so on. Null pointers become explicit placeholder files, so later script IDs never shift.

### Decompile one script

```console
mary decompile ROM LIBRARY --script-id ID --charmap charmap_jp.txt -o OUTPUT.mary
```

An arbitrary RIFF offset can also be decoded:

```console
mary decompile BINARY LIBRARY --offset OFFSET --charmap charmap_jp.txt -o OUTPUT.mary
```

`--script-id` and `--offset` are mutually exclusive.

### Compile source

Generate C data definitions:

```console
mary compile INPUT.mary --charmap charmap_jp.txt -o OUTPUT.c
```

Generate one binary RIFF directly:

```console
mary compile INPUT.mary --binary --charmap charmap_jp.txt -o OUTPUT.riff
```

Binary mode accepts exactly one script. The `#include` line in generated source identifies its callable library; `mary` is not itself a C preprocessor. Expand the include before compiling a standalone extracted file, for example:

```console
cpp INPUT.mary | mary compile --binary --charmap charmap_jp.txt -o OUTPUT.riff
```

Use `mary compile --help` and `mary decompile --help` for all options.

## Character map

All ordinary text—including US English and Japanese Shift-JIS text—is converted through the replaceable UTF-8 `charmap_jp.txt` file.

```text
HEX=text or control token
```

Examples:

```text
05={Press}
0A=\n
0D=\r
0C=\p
8140=　
FF21={Player}
FF22={Horse}
```

Rules:

- The left side is a byte sequence in file order. `8140` means bytes `81 40`; it is never endian-swapped. Numeric ROM pointers and RIFF fields remain little-endian integers.
- Encoding and decoding use longest matching. Multi-byte entries such as `FF21={Player}` stay intact.
- Control names, byte prefixes, and lengths come exclusively from the loaded map. The parser does not hardcode byte prefixes such as `FF` as controls.
- If several byte sequences map to the same text, the first entry is the canonical encoding for handwritten source. Ambiguous original bytes are printed as `\xNN` so round trips cannot silently select another encoding.
- Unmapped bytes are also printed as `\xNN`. This syntax bypasses the map and writes one raw byte.
- `\"` and `\\` are source-level escapes required to place a quote or backslash inside a string. Their ROM bytes still come from the map.
- `HEX=` reserves an unassigned code point and is skipped when the map is loaded. This differs from an entry such as `20= `, whose mapped text is an actual space.

Blank lines and full-line comments beginning with `#` are allowed:

```text
# Message controls
0C=\p
```

Inline comments are not supported. `23=#` remains a valid mapping because the line does not begin with `#`.

## Generated text layout

Every string constant starts on the line after `=`. To match the game's display behavior, generated source splits after `\n` or `\p` and keeps fragments aligned. `\r` only returns the game's horizontal cursor to the start of the line and does not cause a layout break by itself; neither does `{Press}` or any other control:

```c
const MESSAGE_12 =
    "あんたも男なんだから、\r\n"
    "気前がいいところをみせてよ。{Press}"
const MESSAGE_13 =
    "わ…わかったよ。{Press}"
```

This uses C-style adjacent string concatenation. Physical line breaks and indentation outside the quotes add no ROM bytes; the compiler merges the fragments into one string.

## RIFF, CODE, JUMP, and STR

Each script is a `RIFF`/`SCR ` container that normally holds a `CODE` chunk, an optional `JUMP` chunk, and a `STR ` chunk. Lengths, counts, offsets, and instruction operands are read as little-endian integers.

- `CODE`: data is bounded by the chunk length and its internal code length is validated. Branch targets are byte offsets within CODE.
- `JUMP`: the decoder reads the switch-table count and each table offset, then resolves every case/default table through that offset. It does not assume tables are physically contiguous in ID order.
- `STR `: the decoder reads the string count and offset table. Every text ID resolves as “string-pool start + that ID's offset”; physical appearance order is not guessed.

Some modified ROMs may retain the original STR/RIFF declared lengths, place text beyond the RIFF, and point STR offsets at that relocated text. ROM-mode decompilation keeps a backing view from the current RIFF to the remainder of the ROM, so these external strings remain addressable; CODE and JUMP stay bounded by their own chunk lengths.

Compilation produces a standard self-contained RIFF: strings are written consecutively in text-ID order, and the STR offsets and chunk lengths are rebuilt. A modified ROM that uses external strings can therefore be decoded and normalized, but its rebuilt RIFF is not guaranteed to reproduce the modified ROM's unusual physical layout byte-for-byte. The four vanilla ROMs continue to pass strict byte-exact round-trip tests.

## ROM pointer tables

| Version | ROM address | File offset | Slots |
| --- | ---: | ---: | --- |
| FoMT US | `0x080F89D4` | `0x0F89D4` | null ID 0, then 1,328 scripts |
| MFoMT US | `0x081014BC` | `0x1014BC` | null ID 0, then 1,415 scripts |
| FoMT JP | `0x080F8230` | `0x0F8230` | null ID 0, then 1,328 scripts |
| MFoMT JP | `0x0810145C` | `0x10145C` | null ID 0, then 1,415 scripts |

Table length is determined from pointer validity and RIFF data, not by dropping null entries. Empty slots are retained.

## Callable libraries

The VM invokes native game code by numeric ID. `mary` needs a callable “shape” for each ID: whether it is a value-returning `func` or a `proc`, plus its parameters.

- [FoMT library](goodies/lib_fomt.txt)
- [MFoMT library](goodies/lib_mfomt.txt)

FoMT uses MFoMT-oriented names for easier comparison, so digits in a name do not necessarily equal the FoMT ROM ID. The declaration in the selected library is authoritative.

## Script syntax

The language uses C-like structured syntax, but it is not full C. It represents only semantics that the stack VM can encode losslessly.

### Top-level declarations

```c
func 0x106 Func106()
proc 0x022 TalkMessage(message : string)
const NAME = VALUE

script 19 EventScript_19
{
    // body
}
```

### Values and calls

```c
const MESSAGE_0 =
    "First line\r\n"
    "Second line\p"
    "Next page{Press}"

var value = Func106()
var other
other = value + 1
TalkMessage(MESSAGE_0)
```

Top-level and local constants are compile-time only and add no VM instructions.

### Control flow

```c
if condition
{
    Proc001()
}
else
{
    Proc002()
}

for var i = 0; i < 10; i++
{
    Proc003(i)
}

do
{
    i++
} while i < 10

switch value
{
    case 1
    {
        Proc004()
        break
    }
    case 2 fallthrough
    {
        Proc005()
    }
    default
    {
        Proc006()
    }
}
```

Lossless VM-specific constructs include `nodisc`, `discard`, `switch compact`, `default implicit`, `dead`, and `jump next`. If a future valid script cannot be structured without changing its bytecode, an editable instruction block remains available:

```c
ir
{
    PushInt(1)
    Label(0)
    Jmp(0)
    Exit()
}
```

The four current vanilla corpora need no `ir` fallback.

## Verification

`tests/` contains Rust integration tests and a manual verification helper:

- `structured_stress.rs` constructs nested switches, loops, conditionals, breaks, and stack-heavy expressions. Each case performs three compile/decompile/print/recompile rounds and compares the resulting bytes; pairwise and three-level control-flow matrices are included.
- `vanilla_symmetry.rs` reads every pointer-table slot from all four vanilla ROMs. Each version gets both a direct decode/encode test and a printed high-level source round trip; RIFF bytes must match exactly, with no residual low-level `ir` or `jump next`.
- `common/mod.rs` provides recursive AST inspection shared by the integration tests and is not an independently executed test.
- `decompile_all_roms.bat` manually decompiles all four ROMs for inspection and is not run by `cargo test`.

Run ordinary tests that do not require ROMs with:

```console
cargo test --all-targets
```

Before running the complete ROM suite, place the four files at these fixed paths:

```text
rom/fomt.gba
rom/mfomt.gba
rom/fomtjp.gba
rom/mfomtjp.gba
```

The ROM and decompiled-output directories are ignored by Git. Callable declarations come directly from the tracked `goodies/lib_fomt.txt` and `goodies/lib_mfomt.txt`; no environment variables are required. Run:

```console
cargo test --all-targets --features test_with_roms
```

To run only the four-ROM strict suite and show per-version statistics:

```console
cargo test --features test_with_roms --test vanilla_symmetry -- --nocapture
```

On failure, original and rebuilt RIFF files are stored under the ignored `test_failures/<variant>/` directory. Success means byte-for-byte equality for every regenerated RIFF—not merely successful parsing.

On Windows, generate decompiled source for all four ROMs from the repository root with:

```console
tests\decompile_all_roms.bat
```

The helper builds the current code and writes `decompiled_text/fomt_us`, `mfomt_us`, `fomt_jp`, and `mfomt_jp`. It is intended for inspecting decompiled output and does not replace the strict round-trip suite.

## Related projects

- [StanHash/mary_old](https://github.com/StanHash/mary_old) — earlier C++ implementation.
- [StanHash/FOMT-DOC](https://github.com/StanHash/FOMT-DOC) — FoMT event-script research.
- [StanHash/fomt](https://github.com/StanHash/fomt) — FoMT decompilation project.
