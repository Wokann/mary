# mary

[English](README.md) | [简体中文](README.zh-CN.md)

`mary` decompiles, recompiles, and safely imports event scripts for:

- *Harvest Moon: Friends of Mineral Town*: JP, US, European English, and German
- *Harvest Moon: More Friends of Mineral Town*: JP and US

Mary-C is the canonical source format. Its `.mary.c` and `.mary.h` suffixes
enable C highlighting, but the files must be compiled by `mary`, not a normal C
compiler. C-compatible constructs keep C semantics; VM or physical-byte forms
use an explicit `mary_` prefix.

```text
ROM pointer table and RIFF scripts
              ↓ decompile
Named UTF-8 Mary-C source and local headers
              ↓ edit / import
Validated ROM copy with rebuilt RIFFs and pointers
```

All 8,142 non-null scripts in the six vanilla ROMs pass strict
ROM → Mary-C → RIFF byte-for-byte round trips. This proves the tested vanilla
corpus; handwritten source must still obey the documented language and VM
limits.

| Target | Non-null scripts | Strict round trip |
| --- | ---: | ---: |
| FoMT JP | 1,328 | 1,328/1,328 |
| FoMT US | 1,328 | 1,328/1,328 |
| FoMT EU | 1,328 | 1,328/1,328 |
| FoMT DE | 1,328 | 1,328/1,328 |
| MFoMT JP | 1,415 | 1,415/1,415 |
| MFoMT US | 1,415 | 1,415/1,415 |

See the [Mary-C language reference](docs/MARY_C_LANGUAGE.md)
([简体中文](docs/MARY_C_LANGUAGE.zh-CN.md)) for the exact C subset, Mary-only
syntax, callable/constant tables, text layout, and RIFF structure.

## Mary-C example

The FoMT-JP form of Ann's New Year's Eve Noodle Festival dialogue shows the
generated target declaration, Japanese text, control codes, callable names,
and typed constants:

```c
#define MARY_FOMT_JP
#include "fomt_constants.mary.h"
#include "fomt_callables.mary.h"
#include "fomt_scripts.mary.h"

mary_text_table
{
    const char gText_FestivalEvent_NewYearsEve_NoodleFestivalDialogue_Ann[] =
        "おそば食べるの楽しみだね。\r\n"
        "わたし、好きなんだー。{Press}";
};

void EventScript_FestivalEvent_NewYearsEve_NoodleFestivalDialogue_Ann(void)
{
    if (HasMetNpc(CHARACTER_ANN) == FALSE)
    {
        MarkNpcSpokenTo(CHARACTER_ANN);
    }
    SetEntityFacing(ENTITY_ANN, GetOppositeFacing(GetEntityFacing(ENTITY_PLAYER)));
    TalkOpen();
    SetTalkPortrait(TALK_PORTRAIT_ANN_HAPPY);
    SetTalkNameplateCharacter(CHARACTER_ANN);
    if (!(VarGet(VAR_KAREN_MARRIAGE_STATE) == MARRIAGE_STATE_MARRIED || VarGet(VAR_POPURI_MARRIAGE_STATE) == MARRIAGE_STATE_MARRIED || VarGet(VAR_MARY_MARRIAGE_STATE) == MARRIAGE_STATE_MARRIED || VarGet(VAR_ELLI_MARRIAGE_STATE) == MARRIAGE_STATE_MARRIED || VarGet(VAR_HARVEST_GODDESS_WEDDING_AND_NICKNAME_EVENT_STATE) == EVENT_LIFECYCLE_COMPLETED || VarGet(VAR_ANN_CLIFF_WEDDING_EVENT_STATE) == EVENT_LIFECYCLE_COMPLETED || VarGet(VAR_ANN_CLIFF_RIVAL_MARRIAGE_STATE) == MARRIAGE_STATE_MARRIED))
    {
        ShowTalkHeartIndicator(CHARACTER_ANN);
    }
    TalkMessage(gText_FestivalEvent_NewYearsEve_NoodleFestivalDialogue_Ann);
    ClearTalkPortrait();
    TalkClose();
    if (WasNpcSpokenToToday(CHARACTER_ANN) == FALSE)
    {
        AddNpcFriendship(CHARACTER_ANN, 5);
    }
    MarkNpcSpokenTo(CHARACTER_ANN);
    SetEntityFacing(ENTITY_ANN, FACING_LEFT);
}
```

The FoMT-US output keeps the same script and symbol names while emitting its
English localized text:

```c
mary_text_table
{
    const char gText_FestivalEvent_NewYearsEve_NoodleFestivalDialogue_Ann[] =
        "It's too bad that New Year\r\n"
        "Noodles only come once \r\n"
        "a year!{Press}";
};
```

The text remains UTF-8 in source and is encoded through `charmap.txt` during
compilation. Symbols such as `ENTITY_CLIFF`, `TALK_PORTRAIT_CLIFF_AFRAID`,
`CHARACTER_CLIFF`, and `FACING_DOWN` compile to their original numeric values.

## Install

End users should download the executable for their platform from a GitHub
Actions artifact or a release and run it directly. The distributed executable
does not require Rust, Cargo, MinGW, or a third-party runtime to be installed.

### Windows from a clean system

Download `rustup-init.exe` from the
[official Rust installation page](https://www.rust-lang.org/tools/install) and
run it. It is an interactive console installer, not a graphical wizard. Both
Windows toolchain families are supported:

- **MSVC:** accept the default installation. If necessary, let the installer
  add Microsoft C++ Build Tools and the Windows SDK.
- **MinGW:** select **Customize installation** and set:

```text
Default host triple: x86_64-pc-windows-gnu
Default toolchain: stable
Profile: minimal
Modify PATH variable: yes
```

Use `i686-pc-windows-gnu` as the host triple on a 32-bit Windows system. Close
and reopen the terminal after installation. On recent Windows 10/11 systems,
WinGet is an optional shortcut instead of downloading the installer manually:

```console
winget install --exact --id Rustlang.Rustup
rustup toolchain install stable-x86_64-pc-windows-gnu --profile minimal
rustup default stable-x86_64-pc-windows-gnu
```

An ordinary build uses the installed default ABI and host architecture. A
64-bit host therefore produces Win64 by default, while a 32-bit host produces
Win32:

```console
cargo build --locked --release
```

The Makefile exposes all four Windows combinations explicitly:

```console
make windows-x86-mingw
make windows-x86_64-mingw
make windows-x86-msvc
make windows-x86_64-msvc
```

The equivalent generic form accepts an explicit Rust toolchain and target:

```console
make release TOOLCHAIN=stable-x86_64-pc-windows-gnu TARGET=i686-pc-windows-gnu
make release TOOLCHAIN=stable-x86_64-pc-windows-msvc TARGET=x86_64-pc-windows-msvc
```

The MinGW targets use Rust's official GNU toolchain and do not require MSYS2 or
Visual Studio for this pure-Rust project. The MSVC targets require Microsoft
C++ Build Tools and the Windows SDK. Outputs are under `target/TARGET/release`.

Win32 describes the i686 architecture, not Windows XP compatibility. Current
Rust Windows targets require Windows 10 or newer; supporting XP would require a
separate frozen legacy toolchain and dependency set.

### Linux from a clean system

Install the host tools, rustup, and the MUSL target matching the machine:

```console
sudo apt-get update
sudo apt-get install --yes curl build-essential musl-tools
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh -s -- -y --profile minimal --default-toolchain stable
. "$HOME/.cargo/env"

# Use aarch64-unknown-linux-musl on an ARM64 machine.
rustup target add x86_64-unknown-linux-musl
cargo build --locked --release --target x86_64-unknown-linux-musl
```

The output is in `target/TARGET/release/mary`. Cross-building for the other
Linux architecture additionally requires that architecture's MUSL linker;
the supplied Actions workflow avoids that extra setup by using native x86_64
and ARM64 runners.

### macOS from a clean system

Install Apple's command-line tools and rustup:

```console
xcode-select --install
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh -s -- -y --profile minimal --default-toolchain stable
. "$HOME/.cargo/env"
rustup target add x86_64-apple-darwin aarch64-apple-darwin
make macos-universal
```

This creates the Universal 2 binary at
`target/universal-apple-darwin/release/mary`. Xcode supplies the Apple SDK,
linker, GNU Make, and `lipo`.

## Build

```console
cargo build --release
```

`cargo` is the fundamental build command. GNU Make is an optional convenience
frontend used by the targets below.

GNU Make provides the same local entry point:

```console
make release
make test
make lint
```

After `rustup` and Make exist, `make setup` installs the current default
toolchain plus `rustfmt` and Clippy. On Windows, `make setup-windows` is the
MinGW default; use `make setup-windows-msvc` only when MSVC is wanted.
`make setup-linux` and `make setup-macos` add their platform targets. These
setup targets cannot install the operating-system package manager, Apple SDK,
or native linker; those prerequisites are listed above.

Architecture targets are `linux-x86_64`, `linux-aarch64`, the four explicit
Windows MinGW/MSVC targets above, `macos-x86_64`, and `macos-aarch64`.
`windows-x86` and `windows-x86_64` are MinGW shorthand targets.
`macos-universal`
builds both macOS architectures and combines them with `lipo`; it must run on
macOS with Xcode command-line tools. A target still needs its platform linker,
so the GitHub Actions workflow builds each platform on a matching native
runner rather than assuming every host can cross-link every target.

`.github/workflows/build.yml` tests every push and pull request and uploads:

- statically linked Linux x86_64 and ARM64 executables
- Win32 x86 and Win64 x86_64 executables built with both MinGW and MSVC
- one macOS Universal 2 executable plus its Intel and Apple Silicon thin files

These artifacts need no third-party runtime installed on the destination
system. They may still use operating-system libraries and APIs supplied by
Windows or macOS.

## Targets and generated files

Select exactly one target when decompiling:

```text
MARY_FOMT_JP  MARY_FOMT_US  MARY_FOMT_EU  MARY_FOMT_DE
MARY_MFOMT_JP  MARY_MFOMT_US
```

FoMT uses `goodies/fomt_callables.mary.h`,
`goodies/fomt_constants.mary.h`, and
`goodies/fomt_scripts_text.mary.sym`; MFoMT uses the corresponding `mfomt_*`
files. `.mary.sym` is decompiler-only naming metadata and is never a compiler
or linker-bundle input.

A complete decompile writes one `.mary.c` per physical pointer-table slot and
three local headers: constants, callables, and the generated script table.
Null pointers get explicit placeholder files and remain `NULL` in that table,
so later IDs never shift. Each generated source contains its target `#define`
and includes all three headers.

## Decompile

Complete FoMT-JP example:

```console
mary decompile ROM goodies/fomt_callables.mary.h --all --mary-c \
  --symbols goodies/fomt_scripts_text.mary.sym -D MARY_FOMT_JP \
  --charmap charmap.txt -o OUTPUT_DIRECTORY
```

One script ID:

```console
mary decompile ROM goodies/fomt_callables.mary.h --script-id ID --mary-c \
  --symbols goodies/fomt_scripts_text.mary.sym -D MARY_FOMT_JP \
  --charmap charmap.txt -o OUTPUT.mary.c
```

An arbitrary RIFF file offset does not require a recognized ROM header:

```console
mary decompile BINARY goodies/fomt_callables.mary.h --offset OFFSET --mary-c \
  -D MARY_FOMT_JP --charmap charmap.txt -o OUTPUT.mary.c
```

`OFFSET` may be decimal or `0x`-prefixed hexadecimal and cannot be combined
with `--script-id`. For a relocated table that cannot be found from Mary
metadata, provide both manual recovery values:

```console
mary decompile ROM CALLABLES --all --mary-c -D TARGET --charmap charmap.txt \
  --pointer-table TABLE_ADDRESS --pointer-count SLOT_COUNT -o OUTPUT_DIRECTORY
```

## Edit and import into a ROM copy

The input ROM is always read-only and `-o` must name a different file.

Replace one script in its original allocation:

```console
mary import ROM INPUT.mary.c --script-id ID --charmap charmap.txt -o OUTPUT.gba
```

The rebuilt RIFF including alignment may not cross the next script. If it is
shorter, the whole unused remainder of its old allocation is cleared to `00`.

Rebuild a complete or partial source directory at the original address:

```console
mary import ROM SOURCE_DIRECTORY --charmap charmap.txt -o OUTPUT.gba
```

Provided `.mary.c` files are compiled. An existing ROM ID omitted from the
directory keeps its original RIFF bytes; a newly added slot without source
remains `NULL`. Scripts are packed with four-byte zero alignment and must fit
the original contiguous area. Its unused old tail is cleared to `00`.

Relocate a script or packed directory:

```console
mary import ROM SOURCE --address SCRIPT_ADDRESS --charmap charmap.txt -o OUTPUT.gba
```

For one script, also pass `--script-id`. Addresses may be file offsets or
`0x08xxxxxx` GBA addresses and must be four-byte aligned. Mary checks the whole
destination. Non-`00`/`FF` data requires interactive confirmation or `--force`
in non-interactive use. `--dry-run` validates the complete plan without output.

### Expand or relocate the pointer table

When the maintained script table gains slots, Mary first tries to extend the
current ROM pointer table in place. If every newly occupied byte is `00` or
`FF`, the extension is written directly. If the extension contains other data,
the import stops with the conflicting range and byte; pass `--force` only when
that overwrite is intentional. Shrinking a table clears its abandoned pointer
entries to zero. On a later import Mary recognizes an earlier in-place
extension by scanning contiguous null/blank entries and valid pointers to RIFF
headers, so those existing pointers are not mistaken for unrelated data.

The packed RIFF block is checked independently. If it no longer fits its old
area, relocate the script data with `--address`. To relocate the pointer table
itself as well, use:

```console
mary import ROM SOURCE_DIRECTORY --address SCRIPT_ADDRESS \
  --relocate-pointer-table TABLE_ADDRESS --charmap charmap.txt -o OUTPUT.gba
```

For example, these entries add a null ID 1416 followed by a real ID 1417. A
null slot never terminates later indexing:

```c
    /* 0x0588 */ NULL,
    /* 0x0589 */ EventScript_ExpansionProbe,
```

The named script must be defined by a `.mary.c` file. Mary writes the enlarged
table, patches the three verified native table references, and stores checked
discovery metadata in the abandoned original table. Later commands reuse the
relocated table automatically; damaged or unavailable metadata can be bypassed
with `--pointer-table` and `--pointer-count`.

Script data, tables, native references, and metadata cannot overlap. Output is
installed atomically through a unique temporary file only after compilation,
boundary checks, occupancy checks, and any confirmation all succeed.

## Compile without importing

Raw RIFF:

```console
mary compile INPUT.mary.c --mary-c --binary --charmap charmap.txt -o OUTPUT.riff
```

C byte-array definition:

```console
mary compile INPUT.mary.c --mary-c --charmap charmap.txt -o OUTPUT.c
```

`--print-ir` may append non-compiling VM audit comments to textual decompiler
or C-array output. It cannot accompany raw binary output and never substitutes
for structured Mary-C recovery.

Use `mary decompile --help`, `mary import --help`, and `mary compile --help`
for every option.

## Build linker inputs for a decompilation project

`bundle` compiles a complete Mary-C source directory and exposes its scripts,
pointer table, and individual STR strings as assembler/linker symbols. Script
IDs come from the script-table header; text names and physical STR order come
directly from each source file's `mary_text_table`. The decompiler-only
`.mary.sym` database does not participate in this path.

Generate one contiguous, four-byte-aligned RIFF block:

```console
mary bundle decompiled_text/fomt_jp -o build/data/scripts --layout packed \
  --library goodies/fomt_callables.mary.h \
  --script-table decompiled_text/fomt_jp/fomt_scripts.mary.h \
  --charmap charmap.txt -D MARY_FOMT_JP
```

This writes:

```text
scripts.bin       all non-null RIFFs in script-slot order
scripts.s         script and STR text symbols bound to offsets in scripts.bin
script_table.s    ordered GBA pointers, with .word 0 for null slots
scripts.d         Make dependencies for all participating Mary-C inputs
```

To keep each RIFF as a separate file, change `--layout packed` to
`--layout split`. The output then uses `riff/<script-name>.riff`; `scripts.s`
includes those files in slot order and still presents one contiguous linker
section.

Assemble the two generated sources normally:

```console
arm-none-eabi-as build/data/scripts/scripts.s -o build/data/scripts/scripts.o
arm-none-eabi-as build/data/scripts/script_table.s -o build/data/scripts/script_table.o
```

They provide `.rodata.mary_scripts` and `.rodata.mary_script_table`. Place the
two input sections at the desired ROM file offsets in the project's linker
script. `scripts.o` defines every script and text symbol at its real RIFF/STR
byte address; `script_table.o` keeps relocations against those script symbols,
so the linker writes the final pointers.

`scripts.d` participates only in Make's dependency stage—it is not assembled,
linked, or stored in the ROM. Include it from the project Makefile so editing
any `.mary.c`, included `.mary.h`, or charmap reruns `mary bundle`:

```make
MARY_SOURCES := $(wildcard data/scripts/*.mary.c)

-include build/data/scripts/scripts.d

build/data/scripts/scripts.s \
build/data/scripts/script_table.s \
build/data/scripts/scripts.bin: $(MARY_SOURCES)
	mary bundle data/scripts -o build/data/scripts --layout packed \
	  --library data/scripts/fomt_callables.mary.h \
	  --script-table data/scripts/fomt_scripts.mary.h \
	  --charmap data/scripts/charmap.txt -D MARY_FOMT_JP
```

The script-table header is the sole authority for script IDs. Directory builds
compile every named slot and preserve each explicit `NULL` exactly as written.
A `.mary.c` whose script name is absent from the table is ignored and does not
enter the generated dependency file or linker output. Conversely, every
non-null table entry must have a source definition; a directory build stops
before producing output and lists every missing ID and name. Add, remove,
reorder, or null scripts by editing the table deliberately. A single-script
import checks only its specified file and does not require the remaining table
sources to be present.

The wildcard remains useful because it reruns Mary when a file is added, but
the new file is still ignored until its script name is registered in the table.

Every declaration in `mary_text_table` becomes one STR entry in declaration
order, even when CODE does not reference it. Equal text declarations remain
separate physical IDs. Adding or removing declarations therefore dynamically
rebuilds that RIFF's STR count, offset table, and string pool without editing
`.mary.sym`.
Use `mary bundle --help` for all bundle options.

## Character map

All localized text is encoded and decoded through UTF-8 `charmap.txt`:

```text
05={Press}
0A=\n
0D=\r
0C=\p
8140=　
FF21={Player}
```

Hexadecimal keys are byte sequences in file order, not little-endian integers.
Encoding and decoding use longest matching; controls and lengths come from the
map. Unmapped or physically ambiguous bytes become `\xNN`; `HEX=` reserves an
unmapped code. Blank lines and full-line `#` comments are accepted, but inline
comments are not. Generated strings split for readability after `\n` or `\p`;
`\r`, `{Press}`, and other controls do not split source lines by themselves.

## Verification

```console
cargo test --all-targets
cargo clippy --all-targets -- -D warnings
```

For ROM-backed tests, place the files at:

```text
rom/fomt_jp.gba
rom/fomt_us.gba
rom/fomt_eu.gba
rom/fomt_de.gba
rom/mfomt_jp.gba
rom/mfomt_us.gba
```

Then run:

```console
cargo test --all-targets --features test_with_roms
```

ROMs are ignored by Git. `decompiled_text/` is generated workspace output and
is not committed. Failures go to ignored `test_failures/`. On Windows, the
tracked `tests\decompile_all_roms.bat` rebuilds the debug executable and
regenerates all six output directories for inspection.

## Related projects

- [StanHash/mary_old](https://github.com/StanHash/mary_old) — earlier C++ implementation
- [StanHash/FOMT-DOC](https://github.com/StanHash/FOMT-DOC) — FoMT event-script research
- [StanHash/fomt](https://github.com/StanHash/fomt) — FoMT decompilation project
