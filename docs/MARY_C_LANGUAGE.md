# Mary-C 语言与无损工作流

[简体中文](MARY_C_LANGUAGE.md) | [English](MARY_C_LANGUAGE.en.md)

Mary-C 是 Mary 虚拟栈脚本的 C 形前端。能与 C 保持相同语义的部分使用标准 C 写法；虚拟机独有且会影响栈或字节布局的行为使用 `mary_` 前缀。`.mary.c`、`.mary.h` 可以获得常见编辑器的 C 高亮，但不是 ISO C，必须由 `mary` 编译。

“成功反编译”必须满足 ROM 字节码转换成 `.mary.c` 后能够重新解析、编译并逐字节还原原 RIFF。打印器遇到低级 `ir` 或未高级化的 `jump next` 会报错，不会伪装成高级语言成功。

## 文件与唯一数据来源

```text
mary_callables.mary.h          原生 callable 的有序 ID 表和类型声明
mary_scripts_text.mary.sym     仅供反编译使用的脚本/文本符号数据库
mary_scripts.mary.h            针对目标从 ROM 槽位和 .sym 名称生成的脚本表
EventScript_NNNN.mary.c        单个脚本、脚本内文本表和正文
```

`.mary.sym` 不会被 `.mary.c` include，也不参与回编。批量反编译会从 ROM 指针槽和所选 `.sym` 名称生成本地 `mary_scripts.mary.h`，因此脚本表与输出函数名来自同一来源。

## 目标选择

每个 `.mary.c` 必须显式定义且只定义一个目标：

```c
#define MARY_FOMT_JP
#include "mary_callables.mary.h"
#include "mary_scripts.mary.h"
```

支持：

- `MARY_FOMT_US`
- `MARY_MFOMT_US`
- `MARY_FOMT_JP`
- `MARY_MFOMT_JP`

编译器会自动派生上位宏，源码不需要重复 `#define`：`MARY_FOMT_US` 同时满足 `MARY_FOMT` 和 `MARY_US`，`MARY_MFOMT_US` 同时满足 `MARY_MFOMT` 和 `MARY_US`，JP 两种目标同理满足对应作品宏和 `MARY_JP`。因此脚本级差异通常使用 `MARY_FOMT`/`MARY_MFOMT`，脚本内部的语言文本差异通常使用 `MARY_US`/`MARY_JP`；只有无法按单一维度表达的组合才直接判断四个具体目标宏。

`mary_callables.mary.h` 和独立使用的 `.mary.sym` 会在文件开头显式写出这种关系，例如：

```c
#if defined(MARY_MFOMT_JP)
#define MARY_MFOMT
#define MARY_JP
#endif
```

## callable ID

callable 表的预处理后顺序就是 `Call(id)` 的物理 ID：

```c
mary_callable_table
{
    SetEntityPosition,
#if defined(MARY_FOMT)
    FomtOnlyNative,
#elif defined(MARY_MFOMT)
    MfomtOnlyNative,
#endif
    TalkMessage,
    NULL,
};

void SetEntityPosition(int entity, int x, int y, int facing);
void FomtOnlyNative(void);
void MfomtOnlyNative(void);
void TalkMessage(const char *message);
```

`void` 原型是无返回值过程，`int` 原型是有返回值函数；参数支持 `int` 和 `const char *`。`NULL` 明确保留未知或不可调用槽位，不能删除。函数表没有虚构的 `CallScript`；只有在 ROM callable 被实际确认并声明后，源码才会输出其真实名称。

## script ID

脚本 ID 由 `mary_scripts.mary.h` 的有序表决定：

```c
mary_script_table
{
    NULL,
    OpeningEvent,
    MayorExplainsFarm,
#if defined(MARY_MFOMT)
    GirlVersionEvent,
#endif
};
```

脚本定义本身不再写 `mary_script(id)`：

```c
void MayorExplainsFarm(void)
{
    return;
}
```

脚本符号可作为整数常量使用，因此对已验证为“脚本 ID 参数”的真实 callable，可以写符号或数字；两者生成相同整数。编译器不会把任意整数参数猜成脚本调用，也不会制造隐藏语义。

## text ID

文本 ID 是当前脚本内部 STR 表的顺序：

```c
mary_text_table
{
    gText_MayorGreeting,
    gText_GrandfatherWill,
};

const char gText_MayorGreeting[] =
    "第一行\r\n"
    "第二行{Press}";

const char gText_GrandfatherWill[] =
    "下一页\p"
    "继续。{Press}";
```

同一符号被多次使用仍只有一个 text ID。内容相同但需要两个物理 ID 时，应保留两个表项和两个定义；编译器不会擅自合并。

未提供符号时，反编译器在内存中生成 `gText_脚本名_文本ID`。这些默认名只写入 `.mary.c`，不会反写进 `.mary.h`。

## 反编译符号库

`mary_scripts_text.mary.sym` 是有序的反编译符号表。目标预处理后的外层位置自动成为 script ID，脚本内部名称的位置自动成为 text ID：

```c
mary_script_symbols
{
    NULL,

    MayorExplainsFarm
    {
        gText_MayorGreeting,
        gText_GrandfatherWill,
#if defined(MARY_JP)
        gText_JapaneseOnlyExplanation,
#elif defined(MARY_US)
        gText_EnglishOnlyExplanation,
#endif
    },

#if defined(MARY_MFOMT)
    GirlVersionOnlyEvent
    {
        gText_GirlVersionOnlyEvent_000,
#if defined(MARY_JP)
        gText_GirlVersionJapaneseOnly_001,
#endif
    },
    GirlVersionNextEvent
    {
        gText_GirlVersionNextEvent_000,
    },
#endif
};
```

文件中不记录数字 ID。插入或删除脚本/文本后，后续 ID 随顺序自动调整；`NULL` 保留空脚本槽。连续且适用版本相同的独占脚本共用一个外层条件块；脚本内部仍可嵌套更窄的条件来控制文本槽。公共项只写一次，版本差异只包住实际不同的范围。重复脚本名、同一脚本中的重复文本名、无效标识符和错误的表结构都会报错。

## 支持的 C 子集

支持整数变量与常量、字符串常量、函数调用、嵌套调用、赋值、`++`/`--`、算术与比较/逻辑表达式，以及：

```c
if (condition) { ... } else { ... }
for (int i = 0; i < 10; i++) { ... }
do { ... } while (condition);
switch (value) { case 1: ... break; default: ... break; }
return;
```

`return;` 对应脚本退出。标准 `break;` 当前只用于 switch；VM 后端没有普通循环 break 的等价结构，因此循环中的 `break` 会明确报错。`while`、`continue`、带返回值的 `return`、指针运算、结构体和任意普通 C 函数不在当前子集中。

## Mary 专用语法

- `mary_nodisc(expr);`：明确保留赋值等表达式留在虚拟栈上的结果。
- `mary_switch_compact (expr) { ... }`：保留没有标准尾部死跳转的紧凑 switch 布局。
- `mary_implicit_default:`：保留没有显式 case 表项的默认路径。
- `mary_dead_jump:`：保留原字节码中不可达但影响布局的跳转。
- `mary_break_switch;`：跨过内层循环并退出外层 switch。

这些扩展只在标准 C 无法逐字节表达原 VM 行为时使用。

## 命令行

批量反编译：

```console
mary decompile ROM goodies/mary_callables.mary.h --all --mary-c --symbols goodies/mary_scripts_text.mary.sym -D MARY_FOMT_JP --charmap charmap_jp.txt -o OUTPUT
```

回编单个输出脚本：

```console
mary compile OUTPUT/EventScript_0867.mary.c --mary-c --charmap charmap_jp.txt --binary -o EventScript_0867.riff
```

源文件中的 `#define` 选择目标，两个 `#include` 相对当前 `.mary.c` 解析。`-D` 仍可用于自动化和反编译阶段。

## 验证范围

`tests/mary_c_vanilla.rs` 使用 `charmap_jp.txt` 对四个原版 ROM 执行 ROM → Mary-C 文本 → RIFF 的逐字节严格往返：FoMT US 1328、MFoMT US 1415、FoMT JP 1328、MFoMT JP 1415 个有效脚本全部一致。

`tests/mary_c_stress.rs` 另行覆盖多层 switch/if/do-while、紧凑 switch、case 贯穿、嵌套函数调用、虚拟栈保留结果与非法输入诊断。这里的 100% 只表示上述四个已验证 ROM；手写脚本仍必须属于 VM 可表达的语法范围。
