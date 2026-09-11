# mary

[English](README.md) | [简体中文](README.zh-CN.md)

`mary` 用于反编译、重新编译并安全导入以下 GBA 版本的事件脚本：

- 《牧场物语：矿石镇的伙伴们》（FoMT）：日版、美版、欧版英文和德版
- 《牧场物语：矿石镇的伙伴们 女孩版》（MFoMT）：日版和美版

Mary-C 是当前唯一的标准源码格式。`.mary.c` 与 `.mary.h` 可以获得 C 语法高亮，
但必须由 `mary` 编译，不能交给普通 C 编译器。与 C 语义一致的结构采用 C 写法；
虚拟栈或物理字节专用结构显式使用 `mary_` 前缀。

```text
ROM 指针表与 RIFF 脚本
          ↓ 反编译
具名 UTF-8 Mary-C 源码及本地头文件
          ↓ 编辑／导入
重新生成 RIFF 和指针并通过校验的 ROM 副本
```

六个受支持原版 ROM 的 8142 个非空脚本均通过严格的
ROM → Mary-C → RIFF 逐字节往返测试。这证明的是六套原版测试语料；手写源码
仍须遵守文档限定的语言和虚拟机能力边界。

| 目标 | 非空脚本 | 严格往返 |
| --- | ---: | ---: |
| FoMT 日版 | 1328 | 1328/1328 |
| FoMT 美版 | 1328 | 1328/1328 |
| FoMT 欧版英文 | 1328 | 1328/1328 |
| FoMT 德版 | 1328 | 1328/1328 |
| MFoMT 日版 | 1415 | 1415/1415 |
| MFoMT 美版 | 1415 | 1415/1415 |

准确的 C 子集、Mary 专用语法、函数/常量表、文本排版和 RIFF 结构请阅读
[Mary-C 语言说明](docs/MARY_C_LANGUAGE.zh-CN.md)（[English](docs/MARY_C_LANGUAGE.md)）。

## Mary-C 示例

下面是安在除夕荞麦面节日中的 FoMT 日版实际解包脚本，同时展示目标声明、
日文文本、控制符、函数名及带类型的常量宏：

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

FoMT 美版输出使用相同的脚本名和文本符号名，仅将对应文本换成本地化后的英文：

```c
mary_text_table
{
    const char gText_FestivalEvent_NewYearsEve_NoodleFestivalDialogue_Ann[] =
        "It's too bad that New Year\r\n"
        "Noodles only come once \r\n"
        "a year!{Press}";
};
```

源码中的文本保持 UTF-8，编译时由 `charmap.txt` 转换为 ROM 编码。
`ENTITY_CLIFF`、`TALK_PORTRAIT_CLIFF_AFRAID`、`CHARACTER_CLIFF` 和
`FACING_DOWN` 等符号最终会编译为原始数值。

## 安装

普通用户可从 GitHub Actions 构建产物或 Release 下载对应平台的可执行文件并直接
运行。发布的程序无需安装 Rust、Cargo、MinGW 或其他第三方运行库。

### 从空白 Windows 环境构建

从 [Rust 官方安装页面](https://www.rust-lang.org/tools/install) 下载并运行
`rustup-init.exe`。它不是图形界面安装向导，而是交互式控制台安装程序。本项目
同时支持两套 Windows 工具链：

- **MSVC：**接受默认安装；如有需要，让安装器补充 Microsoft C++ Build Tools
  和 Windows SDK。
- **MinGW：**选择 **Customize installation（自定义安装）**，并设置：

```text
Default host triple: x86_64-pc-windows-gnu
Default toolchain: stable
Profile: minimal
Modify PATH variable: yes
```

32 位 Windows 将 host triple 改为 `i686-pc-windows-gnu`。安装完成后关闭并重新
打开终端。较新的 Windows 10/11 也可用 WinGet 代替手动下载安装程序：

```console
winget install --exact --id Rustlang.Rustup
rustup toolchain install stable-x86_64-pc-windows-gnu --profile minimal
rustup default stable-x86_64-pc-windows-gnu
```

普通构建默认跟随已安装工具链的 ABI 和宿主架构：64 位宿主生成 Win64，32 位宿主
生成 Win32：

```console
cargo build --locked --release
```

Makefile 提供四种明确的 Windows 组合：

```console
make windows-x86-mingw
make windows-x86_64-mingw
make windows-x86-msvc
make windows-x86_64-msvc
```

也可以通过通用入口手动指定 Rust 工具链和编译目标：

```console
make release TOOLCHAIN=stable-x86_64-pc-windows-gnu TARGET=i686-pc-windows-gnu
make release TOOLCHAIN=stable-x86_64-pc-windows-msvc TARGET=x86_64-pc-windows-msvc
```

MinGW 目标使用 Rust 官方 GNU 工具链；对当前纯 Rust 项目，不要求安装 MSYS2 或
Visual Studio。MSVC 目标需要 Microsoft C++ Build Tools 和 Windows SDK。程序
统一生成在 `target/目标名/release`。

Win32 表示 i686 架构，并不代表兼容 Windows XP。当前 Rust Windows 目标要求
Windows 10 或更新版本；若要支持 XP，必须另行冻结一套历史 Rust 工具链及依赖。

### 从空白 Linux 环境构建

安装宿主工具、rustup，以及与本机架构相同的 MUSL 目标：

```console
sudo apt-get update
sudo apt-get install --yes curl build-essential musl-tools
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh -s -- -y --profile minimal --default-toolchain stable
. "$HOME/.cargo/env"

# ARM64 机器改用 aarch64-unknown-linux-musl
rustup target add x86_64-unknown-linux-musl
cargo build --locked --release --target x86_64-unknown-linux-musl
```

程序位于 `target/目标名/release/mary`。在一种 Linux 架构上交叉构建另一种架构
还需安装相应架构的 MUSL 链接器；本项目的 Actions 直接使用原生 x86_64 和 ARM64
runner，避免这项额外配置。

### 从空白 macOS 环境构建

安装 Apple 命令行工具和 rustup：

```console
xcode-select --install
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh -s -- -y --profile minimal --default-toolchain stable
. "$HOME/.cargo/env"
rustup target add x86_64-apple-darwin aarch64-apple-darwin
make macos-universal
```

Universal 2 程序生成在 `target/universal-apple-darwin/release/mary`。Xcode 会
提供 Apple SDK、链接器、GNU Make 和 `lipo`。

## 构建

```console
cargo build --release
```

`cargo` 是基础构建命令；GNU Make 只是下列目标使用的可选便捷入口。

也可以使用 GNU Make 调用相同的本地构建入口：

```console
make release
make test
make lint
```

当系统已经具备 `rustup` 和 Make 后，`make setup` 会安装当前默认工具链、
`rustfmt` 与 Clippy。Windows 下 `make setup-windows` 默认配置 MinGW；只有明确
需要 MSVC 时才调用 `make setup-windows-msvc`。`make setup-linux` 与
`make setup-macos` 会添加对应平台的 Rust targets。这些配置目标无法反过来安装
操作系统包管理器、Apple SDK 或原生链接器；相关前置依赖仍按上面的首次安装步骤
准备。

架构目标包括 `linux-x86_64`、`linux-aarch64`、上述四种明确的 Windows
MinGW/MSVC 目标、`macos-x86_64` 和 `macos-aarch64`。`windows-x86` 与
`windows-x86_64` 是对应 MinGW 目标的简写。
`macos-universal` 会构建两种 macOS 架构并通过 `lipo` 合并，必须在安装了
Xcode 命令行工具的 macOS 上执行。各目标仍需要对应平台链接器，因此 GitHub
Actions 会在匹配的原生 runner 上构建，而不假设任意系统都能完成所有跨平台
链接。

`.github/workflows/build.yml` 会在每次推送和拉取请求中执行测试并上传：

- 静态链接的 Linux x86_64 与 ARM64 可执行文件
- 分别使用 MinGW 与 MSVC 构建的 Win32 x86、Win64 x86_64 可执行文件
- 一份 macOS Universal 2 程序，以及 Intel 和 Apple Silicon 单架构程序

这些产物不要求目标电脑额外安装第三方运行库，但 Windows/macOS 版本仍会调用
操作系统自带的系统库和 API。

## 目标版本与生成文件

反编译时必须选择且只能选择一个目标：

```text
MARY_FOMT_JP  MARY_FOMT_US  MARY_FOMT_EU  MARY_FOMT_DE
MARY_MFOMT_JP  MARY_MFOMT_US
```

FoMT 使用 `goodies/fomt_callables.mary.h`、
`goodies/fomt_constants.mary.h` 和
`goodies/fomt_scripts_text.mary.sym`；MFoMT 使用相应的 `mfomt_*` 文件。
`.mary.sym` 在反编译及链接打包时提供名称，但不会编译进虚拟机字节码。

完整反编译会按物理指针表的每个槽位生成一个 `.mary.c`，同时生成常量、函数和
脚本表三份本地头文件。空指针会生成明确的占位文件，并在脚本表中保持 `NULL`，
所以后续 ID 不会偏移。每个生成源码都包含目标 `#define` 并 include 三份头文件。

## 反编译

FoMT 日版完整示例：

```console
mary decompile ROM goodies/fomt_callables.mary.h --all --mary-c \
  --symbols goodies/fomt_scripts_text.mary.sym -D MARY_FOMT_JP \
  --charmap charmap.txt -o OUTPUT_DIRECTORY
```

单个脚本 ID：

```console
mary decompile ROM goodies/fomt_callables.mary.h --script-id ID --mary-c \
  --symbols goodies/fomt_scripts_text.mary.sym -D MARY_FOMT_JP \
  --charmap charmap.txt -o OUTPUT.mary.c
```

任意文件偏移处的 RIFF 不要求输入具有可识别的 ROM 头：

```console
mary decompile BINARY goodies/fomt_callables.mary.h --offset OFFSET --mary-c \
  -D MARY_FOMT_JP --charmap charmap.txt -o OUTPUT.mary.c
```

`OFFSET` 接受十进制或带 `0x` 前缀的十六进制文件偏移，不能与 `--script-id`
同时使用。如果迁移表无法通过 Mary 元数据自动找到，可提供两个手动恢复值：

```console
mary decompile ROM CALLABLES --all --mary-c -D TARGET --charmap charmap.txt \
  --pointer-table TABLE_ADDRESS --pointer-count SLOT_COUNT -o OUTPUT_DIRECTORY
```

## 编辑并导入 ROM 副本

输入 ROM 始终只读，`-o` 必须指定另一个文件。

在原分配区替换单个脚本：

```console
mary import ROM INPUT.mary.c --script-id ID --charmap charmap.txt -o OUTPUT.gba
```

重新生成的 RIFF 连同对齐填充不能越过下一条脚本；缩短后，原分配区未使用部分
会全部清为 `00`。

在原地址重建完整或不完整的源码目录：

```console
mary import ROM SOURCE_DIRECTORY --charmap charmap.txt -o OUTPUT.gba
```

目录中提供的 `.mary.c` 会重新编译；缺少原 ROM 已有 ID 时保留原 RIFF 字节；
新增槽位没有源码时保持 `NULL`。脚本以四字节零填充方式紧凑排列，必须容纳在
原连续脚本区中；整体缩短后，未使用的原区尾部全部清为 `00`。

迁移单脚本或整体目录：

```console
mary import ROM SOURCE --address SCRIPT_ADDRESS --charmap charmap.txt -o OUTPUT.gba
```

迁移单脚本时还要传入 `--script-id`。地址可以是文件偏移或 `0x08xxxxxx` GBA
地址，并且必须四字节对齐。Mary 检查完整目标范围；发现 `00`/`FF` 以外的数据
时，交互环境会先询问，非交互环境必须添加 `--force`。`--dry-run` 完成编译和
全部规划校验，但不生成输出 ROM。

### 扩容或迁移指针表

生成脚本表增加槽位后，必须同时迁移整体脚本块和指针表：

```console
mary import ROM SOURCE_DIRECTORY --address SCRIPT_ADDRESS \
  --relocate-pointer-table TABLE_ADDRESS --charmap charmap.txt -o OUTPUT.gba
```

例如，下面会新增空的 ID 1416 和有脚本的 ID 1417；空指针不会终止后续索引：

```c
    /* 0x0588 */ NULL,
    /* 0x0589 */ EventScript_ExpansionProbe,
```

`EventScript_ExpansionProbe` 必须由某个 `.mary.c` 文件定义。Mary 会写入扩容表，
修补三处已经验证的原生表引用，并在废弃的原始表中保存带校验的发现元数据。
后续反编译和导入会自动复用迁移表；元数据损坏或不可用时，使用
`--pointer-table` 与 `--pointer-count` 手动恢复。

脚本数据、指针表、原生引用和元数据之间禁止重叠。只有编译、边界、占用和必要
确认全部通过后，输出才会通过唯一临时文件原子安装。

## 只编译而不导入

生成原始 RIFF：

```console
mary compile INPUT.mary.c --mary-c --binary --charmap charmap.txt -o OUTPUT.riff
```

生成 C 字节数组定义：

```console
mary compile INPUT.mary.c --mary-c --charmap charmap.txt -o OUTPUT.c
```

`--print-ir` 可以在文本反编译输出或 C 字节数组输出中附加不参与编译的虚拟机
审计注释；它不能和原始二进制输出组合，也绝不会替代结构化 Mary-C 恢复。

使用 `mary decompile --help`、`mary import --help` 和
`mary compile --help` 查看全部选项。

## 为反编译工程生成链接输入

`bundle` 会编译完整的 Mary-C 源码目录，并把脚本、指针表和 RIFF 内各条 STR
文本公开为汇编／链接符号。所选 `.mary.sym` 的顺序必须与生成的脚本表及每个
RIFF 的文本表严格一致；不一致会直接报错，避免名称静默绑定到错误地址。

生成一个连续且按四字节对齐的 RIFF 整块：

```console
mary bundle decompiled_text/fomt_jp -o build/data/scripts --layout packed \
  --symbols goodies/fomt_scripts_text.mary.sym \
  --library goodies/fomt_callables.mary.h \
  --script-table decompiled_text/fomt_jp/fomt_scripts.mary.h \
  --charmap charmap.txt -D MARY_FOMT_JP
```

输出内容为：

```text
scripts.bin       按脚本槽顺序连续排列的全部非空 RIFF
scripts.s         绑定到 scripts.bin 偏移的脚本及 STR 文本符号
script_table.s    有序 GBA 指针；空槽写成 .word 0
scripts.d         所有参与编译的 Mary-C 输入文件的 Make 依赖
```

如果希望每个 RIFF 独立保存，把 `--layout packed` 改为 `--layout split`。此时
生成 `riff/<脚本名>.riff`；`scripts.s` 仍会按槽位顺序包含这些文件，并向链接器
提供一个连续 section。

两个汇编文件可由常规 ARM 汇编器处理：

```console
arm-none-eabi-as build/data/scripts/scripts.s -o build/data/scripts/scripts.o
arm-none-eabi-as build/data/scripts/script_table.s -o build/data/scripts/script_table.o
```

它们分别提供 `.rodata.mary_scripts` 和 `.rodata.mary_script_table`。在反编译
工程的链接脚本中，把这两个输入 section 放到目标 ROM 文件偏移即可。
`scripts.o` 会把每个脚本和文本符号绑定到真实 RIFF／STR 字节地址；
`script_table.o` 保留针对脚本符号的重定位，由链接器写入最终指针。

`scripts.d` 只参与 Make 的依赖判断，不会交给汇编器、链接器或写入 ROM。
工程 Makefile 应 include 它，使任何 `.mary.c`、被包含的 `.mary.h`、`.mary.sym`
或码表发生变化时重新运行 `mary bundle`：

```make
MARY_SOURCES := $(wildcard data/scripts/*.mary.c)

-include build/data/scripts/scripts.d

build/data/scripts/scripts.s \
build/data/scripts/script_table.s \
build/data/scripts/scripts.bin: $(MARY_SOURCES)
	mary bundle data/scripts -o build/data/scripts --layout packed \
	  --symbols data/scripts/fomt_scripts_text.mary.sym \
	  --library data/scripts/fomt_callables.mary.h \
	  --script-table data/scripts/fomt_scripts.mary.h \
	  --charmap data/scripts/charmap.txt -D MARY_FOMT_JP
```

这里保留通配依赖是有意的：生成的 `.d` 能追踪已有输入文件，而通配列表还能让
Make 发现刚刚新增的 `.mary.c`。全部选项可通过 `mary bundle --help` 查看。

## 字符码表

所有本地化文本均通过 UTF-8 `charmap.txt` 编码和解码：

```text
05={Press}
0A=\n
0D=\r
0C=\p
8140=　
FF21={Player}
```

左侧十六进制是文件顺序的字节序列，不是小端整数。编码和解码采用最长匹配；
控制符名称和长度来自码表。未映射或物理编码有歧义的原字节输出为 `\xNN`；
`HEX=` 保留未分配码位。允许空行和以 `#` 开头的整行注释，不支持行尾注释。
生成源码只在 `\n` 或 `\p` 后为可读性拆分字符串片段；`\r`、`{Press}` 和其他
控制符本身不会拆分源码行。

## 验证

```console
cargo test --all-targets
cargo clippy --all-targets -- -D warnings
```

ROM 测试使用以下固定路径：

```text
rom/fomt_jp.gba
rom/fomt_us.gba
rom/fomt_eu.gba
rom/fomt_de.gba
rom/mfomt_jp.gba
rom/mfomt_us.gba
```

```console
cargo test --all-targets --features test_with_roms
```

ROM 文件由 Git 忽略。`decompiled_text/` 是工作区生成内容，不属于提交源码；
失败转储保存在被忽略的 `test_failures/` 中。Windows 下，已提交的
`tests\decompile_all_roms.bat` 会构建调试版程序并重新生成六套脚本供检查。

## 相关项目

- [StanHash/mary_old](https://github.com/StanHash/mary_old)：早期 C++ 实现
- [StanHash/FOMT-DOC](https://github.com/StanHash/FOMT-DOC)：FoMT 事件脚本研究
- [StanHash/fomt](https://github.com/StanHash/fomt)：FoMT 反编译项目
