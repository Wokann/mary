# Mary-C 语言与无损工作流

[English](MARY_C_LANGUAGE.md) | [简体中文](MARY_C_LANGUAGE.zh-CN.md)

Mary-C 是 Mary 虚拟栈脚本的 C 形前端。能与 C 保持相同语义的部分使用标准 C 写法；虚拟机独有且会影响栈或字节布局的行为使用 `mary_` 前缀。`.mary.c`、`.mary.h` 可以获得常见编辑器的 C 高亮，但不是 ISO C，必须由 `mary` 编译。

“成功反编译”必须满足 ROM 字节码转换成 `.mary.c` 后能够重新解析、编译并逐字节还原原 RIFF。批量、具名、单脚本及公共默认入口都使用严格结构化反编译；遇到低级 `ir`、未高级化的 `jump next` 或无法唯一恢复的控制流时会报错，不会静默降级并伪装成高级语言成功。`--print-ir` 只会把底层指令作为明确的审计注释附加到成功输出，不参与回编，也不替代高级化。

## 文件与唯一数据来源

```text
fomt_callables.mary.h / mfomt_callables.mary.h
                               各自作品的原生 callable 有序 ID 表和类型声明
fomt_constants.mary.h / mfomt_constants.mary.h
                               各自作品的固定 ID 常量及参数类型
fomt_scripts_text.mary.sym / mfomt_scripts_text.mary.sym
                               各自作品仅供反编译使用的脚本/文本符号数据库
fomt_scripts.mary.h / mfomt_scripts.mary.h
                               针对作品从 ROM 槽位和 .sym 名称生成的脚本表
EventScript_NNNN.mary.c        单个脚本、脚本内文本表和正文
```

`.mary.sym` 不会被 `.mary.c` include，也不参与回编。批量反编译会复制固定常量头与 callable 头，并从 ROM 指针槽和所选 `.sym` 名称生成本地 `fomt_scripts.mary.h`，因此脚本表与输出函数名来自同一来源。单脚本反编译写入文件时同样复制固定头；若提供脚本符号表或脚本表，则同时生成本地脚本表头。

## 目标选择

每个 `.mary.c` 必须显式定义且只定义一个目标：

```c
#define MARY_FOMT_JP
#include "fomt_callables.mary.h"
#include "fomt_scripts.mary.h"
```

支持目标按统一顺序为：

- `MARY_FOMT_JP`
- `MARY_FOMT_US`
- `MARY_FOMT_EU`
- `MARY_FOMT_DE`
- `MARY_MFOMT_JP`
- `MARY_MFOMT_US`

FoMT 源码必须配套 `fomt_*` 文件，MFoMT 源码必须配套 `mfomt_*` 文件。解析器由正式目标自动派生 `REGION_JP`、`REGION_US`、`REGION_EU` 或 `REGION_DE`；作品间不同的 ID 序列已经由文件边界分开，表内不再重复目标校验、派生宏或 `MARY_FOMT`/`MARY_MFOMT` 大分支。

指针表模式（`--all` 与 `--script-id`）会校验该目标是否与 ROM 头的游戏标题和游戏代码一致；任意二进制 `--offset` 模式不作此要求。

相应 callable、constants 和 `.mary.sym` 只在实际本地化差异处判断地区宏，例如：

```c
#if defined(REGION_JP)
    JapaneseLayoutEntry,
#elif defined(REGION_US)
    EnglishLayoutEntry,
#endif
```

## callable ID

callable 表的预处理后顺序就是 `Call(id)` 的物理 ID：

```c
mary_callable_table
{
    /* 0x000 */ NULL,
    /* 0x001 */ NULL,
    /* 0x002 */ SetEntityPosition,
    /* 0x003 */ GetEntityX,
};

void SetEntityPosition(int entity, int x, int y, int facing);
int GetEntityX(int entity);
```

`void` 原型是无返回值过程，`int` 原型是有返回值函数；参数支持 `int`、`const char *` 及固定 ID 类型。`NULL` 明确保留 VM 内部、不可调用或无法作为独立脚本接口公开的槽位，不能删除。具名的 `NoOp*` 与 `NULL` 不同：它们是原版脚本实际调用、且经四套 ROM 原生处理函数确认不产生效果的零操作；声明仍保留原字节码压入的参数数量，才能严格逐字节往返。比如教程中的 `NoOpTutorialFieldTile(...)` 看似携带坐标和对象参数，但原生入口直接返回，不能再伪装成“设置农田格”的高级功能。

物理 callable 表会保留原生变体。例如 `FadeInScreenWithoutSceneHook(...)` 与 `FadeInScreen(...)` 使用相同的渐变参数域和核心过渡流程，但前者跳过后者在过渡前执行的当前场景虚函数；两者各自占用真实的 `Call(id)` 槽，不能错误合并为别名。ROM 的脚本调用 callable 已根据源码中的 `ScriptEngine::LoadById` 路径确认为 `CallScript(MaryScriptId script_id)`。

固定 ID 使用带显式边界的 C 风格枚举：

```c
typedef enum MaryCharacterId
{
    CHARACTER_KAREN = 19,
    CHARACTER_DOCTOR = 20,
} MaryCharacterId;
```

枚举类型只在编译期标明参数所属的编号域；成员和原始整数都会在发射虚拟栈指令前消解为相同整数，因此 `SetTalkNameplateCharacter(CHARACTER_KAREN)` 与 `SetTalkNameplateCharacter(19)` 生成完全相同的字节。没有自身固定成员、由其他有序表提供符号的类型（例如 `MaryScriptId`）仍使用 `typedef int`。

`fomt_constants.mary.h` 与 `mfomt_constants.mary.h` 中的食品、物品、工具、地图、人物、对话头像、产品、资料页、音频序列、钓鱼记录、联动里程碑及屏幕渐变选择器，均覆盖已经确认的完整物理编号域，不以原版脚本是否使用为筛选条件。其他编号域也必须以原生表的完整范围为完成标准；只收录脚本曾用值不算完成。`MaryVarId` 明确保留 FoMT 的 0–589 和 MFoMT 的 0–727 每一个物理槽：已经确认的槽位使用语义名；尚未证实者使用 `VAR_UNKNOWN_SLOT_nnn` 显式占位，直到原生读取者、写入者、生命周期和值域得到独立确认。未知占位表示“身份仍待证实”，不表示该槽未被使用。

并非每个整数参数都是固定 ID 域。数量、时间和计数由当前运行状态决定，不存在一张跨脚本稳定的全局成员表，因此有意保留为 `int`。开放坐标域也不是有限枚举，但使用 `MaryMapSpaceX`、`MaryMapSpaceY` 及恒等包装 `X(...)`、`Y(...)` 区分轴向；包装不改变数值或生成指令。类别内部的家畜索引不同：其物理范围已经得到证明，因此牛羊舍槽位和鸡舍槽位使用两个独立类型，并根据动物类别参数选择。只有原生有序表、位域或有限分派器能够证明完整成员集合时，才建立对应的 `Mary*Id`、`Mary*Kind` 或状态枚举；不能仅根据原版脚本碰巧出现过的几个数值制造不完整枚举。

部分 VM 整数只有在另一个值确定后才能缩小到具体语义域。常量头通过编译器元数据记录这种关系：`mary_callable_return_type_when(...)` 根据某个参数选择返回值域，`mary_callable_return_type_when_callable(...)` 关联两个已经保存的 callable 返回值，`mary_callable_parameter_type_when(...)` 根据一个参数选择另一个参数的类型，`mary_type_subset(Sub, Super)` 则声明两个已证实枚举域的包含关系，使同时用于宽、窄域的局部值保留更窄的语义符号。这些声明不生成字节码。Mary 加载 callable 表时会校验其中的函数名、参数位置和类型名称，在局部变量及控制流中传播选定类型，并在没有已声明包含关系的路径发生类型冲突时保留原始数字。符号替换也会保留物理编码。普通负数字面量及具名负常量（例如 `-1` 和 `ITEM_TOOL_NOT_PRESENT`）默认直接压入负值；如果输入字节码采用“先压入正数，再执行 Neg”的另一种物理编码，反编译会输出 `mary_negated_int(ITEM_TOOL_NOT_PRESENT)`。这一显式 Mary-C 写法既保留语义常量名，也明确要求通过 Neg 指令产生相同的负值。

`mary_type_subset` 的关系必须是严格、无环的；重复声明、自包含以及直接或间接循环都会在加载常量头时被拒绝，避免“更窄类型”的选择依赖表达式顺序。

## script ID

脚本 ID 由 `fomt_scripts.mary.h` 的有序表决定：

```c
mary_script_table
{
    /* 0x0000 */ NULL,
    /* 0x0001 */ EventScript_OpeningEvent,
    /* 0x0002 */ EventScript_MayorExplainsFarm,
};
```

脚本定义本身不再写 `mary_script(id)`：

```c
void EventScript_MayorExplainsFarm(void)
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
    const char gText_MayorGreeting[] =
        "第一行\r\n"
        "第二行{Press}";

    const char gText_GrandfatherWill[] =
        "下一页\p"
        "继续。{Press}";
};
```

表内声明顺序直接决定 text ID，不需要在表头重复列出名称。同一符号被多次使用仍只有一个 text ID。内容相同但需要两个物理 ID 时，应保留两个不同名称的声明；编译器不会擅自合并。旧的“表中名称列表、表外字符串声明”格式仅保留读取兼容，新反编译固定输出表内声明。

文本参数必须引用当前脚本中已经声明的 `gText_*` 符号；不存在的名称会在编译期报错。原版四个 ROM 的运行时 `GetString` 都有 `id <= string_count` 的越界判断错误，但 Mary-C 不开放这个不安全的额外索引：合法 ID 始终是 `0` 至 `string_count - 1`。

未提供符号时，反编译器在内存中生成 `gText_脚本名_文本ID`。这些默认名只写入 `.mary.c`，不会反写进 `.mary.h`。

## 反编译符号库

`fomt_scripts_text.mary.sym` 是有序的反编译符号表。目标预处理后的外层位置自动成为 script ID，脚本内部名称的位置自动成为 text ID：

```c
mary_script_symbols
{
    NULL,

    EventScript_MayorExplainsFarm
    {
        mary_local_type(var_0, MaryBool),
        gText_MayorGreeting,
        gText_GrandfatherWill,
#if defined(REGION_JP)
        gText_JapaneseOnlyExplanation,
#elif defined(REGION_US)
        gText_EnglishOnlyExplanation,
#endif
    },

    EventScript_RegionSensitiveEvent
    {
#if defined(REGION_US)
        gText_EnglishOnlyMessage,
#elif defined(REGION_JP)
        gText_JapaneseOnlyMessage,
#endif
    },
};
```

文件中不记录数字 ID。插入或删除脚本/文本后，后续 ID 随顺序自动调整；`NULL` 保留空脚本槽。连续且适用版本相同的独占脚本共用一个外层条件块；脚本内部仍可嵌套更窄的条件来控制文本槽。公共项只写一次，版本差异只包住实际不同的范围。重复脚本名、同一脚本中的重复文本名、无效标识符和错误的表结构都会报错。

`mary_local_type(var_N, Type)` 是只供反编译使用的局部类型标注，不占用文本槽，也不写入生成的 `.mary.c`。它用于脚本把裸整数先存入局部变量、之后只在条件或 `switch` 中使用，因而无法从 callable 参数或返回值自动恢复语义域的情况。例如上面的 `var_0` 若只取 `0/1`，反编译会输出 `FALSE/TRUE`。类型名必须来自所选目标的常量头，`var_N` 必须确实存在于该脚本；未知类型、拼错的局部名和同一局部的重复标注都会报错。只有整个脚本生命周期中始终保持同一含义的局部量才应使用此标注；被不同功能复用的局部量继续保留原数字。

### 头文件与符号库中的 Mary 元数据

下列形式不是 ISO C 声明，只供 Mary-C 加载表格和恢复语义类型；它们均不生成 VM 指令或 RIFF 字节：

```c
mary_var_type(VAR_SEASON, MarySeason);
mary_typed_identity(MaryMapSpaceX, X);
mary_type_subset(MaryCharacterId, MaryEntityId);
mary_callable_return_type_when(GetAnimalGrowthStage, 0, ANIMAL_KIND_COW, MaryAnimalCowGrowthStage);
mary_callable_return_type_when_callable(GetPresentedItemId, GetPresentedItemKind, HELD_ITEM_KIND_FOOD, MaryItemFoodId);
mary_callable_parameter_type_when(OpenNameEntry, 0, NAME_ENTRY_COW, 1, MaryAnimalSlotIndex);
```

- `mary_var_type` 把某个全局游戏变量绑定到已声明的枚举域，使读取、写入、比较和 `switch case` 能输出语义常量。
- `mary_typed_identity` 声明一个带类型但数值恒等的包装器，例如 `X(120)`；包装器不会产生调用或附加指令。
- `mary_type_subset` 声明已证实的严格类型包含关系，用于控制流合流时保留最窄且仍然正确的枚举域。
- `mary_callable_return_type_when` 根据同一次调用的参数值选择返回值类型。
- `mary_callable_return_type_when_callable` 根据另一个已捕获 callable 的返回值选择当前返回值类型。
- `mary_callable_parameter_type_when` 根据一个参数的值选择另一个参数的类型。
- `mary_local_type` 只存在于 `.mary.sym`，为无法从 callable 数据流推断的整个脚本局部变量指定稳定类型。

这些元数据必须引用已经声明的变量、callable、参数位置、常量和类型。重复、未知引用、错误参数索引、循环类型包含关系或与脚本局部生命周期不符的标注都会报错，而不会猜测或静默退回错误符号。

## 支持的 C 子集

Mary-C 只接受能够确定映射到 Mary 虚拟栈的语法。当前支持：

| 类别 | 支持内容 |
| --- | --- |
| 类型 | 局部 `int`、`const int`、文本 `const char []`、callable 参数中的 `const char *`，以及头文件中用于 callable/常量传播的命名 ID 类型 |
| 字面量 | 十进制/十六进制整数（可编码范围 `-2147483648` 至 `0xFFFFFFFF`）、由 charmap 编码的字符串、用于保留原字节的 `\xNN` |
| 表达式 | `+ - * / %`、一元 `-`/`!`、`== != < <= > >=`、非短路 `&& ||`、括号和嵌套 callable 调用 |
| 修改 | `= += -= *= /= %=`，以及前置/后置 `++`、`--` |
| 语句 | 局部声明、调用、赋值、`if/else if/else`、`for`、`do/while`、`switch`、`return;` |
| switch | 多个 `case` 共用正文、显式贯穿、`default`、标准 `break;`，并允许嵌套控制流 |

典型结构如下：

```c
if (condition) { ... } else { ... }
for (int i = 0; i < 10; i++) { ... }
do { ... } while (condition);
switch (value) { case 1: ... break; default: ... break; }
return;
```

`return;` 对应 VM 的脚本退出，而不是从普通 C 函数返回。case 末尾没有 `break;` 时保留 C 的贯穿语义。当前前端只接受直接所属于 `switch` 的标准 `break;`；普通循环 break 尚未建模，因此会明确报错，但这不是 VM 跳转能力的限制。

局部 `int` 遵循块作用域：内层允许声明同名变量并遮蔽外层变量，离开该块后恢复
外层绑定。编译器会为仍同时存活的祖先／子块变量分配不同 VM 槽；只有生命周期
不重叠的兄弟块临时量才可能复用槽位。该规则同样适用于 `if`、`for`、`do/while`
和 `switch` 内的嵌套块。

命名 ID 类型是 Mary-C 读取头文件时使用的静态语义标注，例如 `MaryCharacterId`；当前脚本局部声明仍写作 `int character`，不能写成 `MaryCharacterId character`。反编译器会根据 callable 参数、返回值和数据流选择相应枚举符号，但 VM 中的物理值仍是整数。

`&&` 和 `||` 目前直接对应 VM 的逻辑运算指令，左右操作数都会求值；它们不具有 ISO C 的短路副作用语义。因此只能在两边都无副作用时当作 C 式逻辑条件阅读：

```c
if (hour >= 6 && hour < 18) { ... }       // 可用：两边都是纯计算
if (IsReady() && ConsumeItem()) { ... }   // 不能假定 ConsumeItem() 会被短路
```

### 当前前端未支持，但现有 IR 可以表达

- `while`、`continue`、循环 `break`、`goto`、用户标签和任意跳转：IR 已有 `Label`、`Jmp` 及条件分支；缺少的是 C AST、跳转目标回填和反编译结构恢复。
- 三元运算符：可以由条件分支和共同结果栈位表达；当前只是语法和反编译器没有建模。
- 逗号运算符：可以按顺序求值、丢弃前项并保留末项；当前前端未接受该语法。
- 字符字面量：VM 可接受整数，但 Mary-C 尚未定义字符应映射为 charmap 编码、Unicode 码点还是单字节整数，因此暂不接受。
- 有限的整数类型转换和 `sizeof`：对单一 VM 整数类型可做编译期处理，但当前没有 ISO C 类型系统、对象布局或相应语法。
- 宏函数：属于预处理层缺失，不是 VM 限制；展开后的内容若属于 Mary-C 子集，IR 本身可以表达。

### 现有 IR 没有通用直接支撑

- `return value;`、带参数或非 `void` 的脚本入口、用户自定义并调用的普通 C 函数：VM 只有退出当前脚本的 `Exit` 和按 ID 调用游戏原生 callable 的 `Call`，没有通用的脚本参数、返回值和局部函数调用约定。
- 取地址/解引用、普通数组下标、结构体/联合和成员访问：IR 只能操作局部整数栈位、文本 ID 和 callable，没有通用内存读写指令或 C 对象模型。地址数值本身可当整数计算，但无法在脚本中普遍解引用。
- 浮点数：IR 只有整数栈和整数算术；除非游戏提供特定 callable，否则只能手动设计定点数协议，不等价于 C 浮点。
- 位与/或/异或/取反和移位：现有 `LogicalAnd`/`LogicalOr` 是逻辑指令，不是 C 位运算；IR 没有对应的通用位指令。特定取值范围下可用算术改写，但不能宣称为任意整数的等价支持。
- 动态内存、C 标准库、普通目标文件链接：Mary VM 不是机器码 C ABI，不能直接链接或执行普通 C 库。

### 语义相似但不能当作 ISO C

- `&&`/`||` 的真值计算已有 IR，但当前两边都会求值，不能依赖 ISO C 短路语义。真正短路可用条件跳转实现，但暂未建模。
- 指针运算只能被视为整数地址计算，且计算结果仍无法通用解引用；脚本文本表是构建期 RIFF 字符串表，不是运行时 C 数组。
- 普通头文件中的任意 ISO C 声明及完整 C 副作用顺序不是当前语言契约。Mary-C 只读取自己支持的表、枚举、callable 声明和目标条件。

`#define`、`#if`、`#elif`、`#else`、`#endif` 和本地 `#include` 只用于选择目标及读取 Mary-C 的表/声明文件，不构成完整 C 预处理器。遇到不支持的语法，编译器必须报错，不能静默改变含义或退回伪高级语言。

### 不支持语法的建议改写

#### `while`

不支持：

```c
while (HasWork())
{
    DoWork();
}
```

可用 `for` 明确保留先判断后执行的语义：

```c
for (int keep_running = HasWork(); keep_running; keep_running = HasWork())
{
    DoWork();
}
```

只有在正文必须至少执行一次时才可改用 `do/while`；两者不是普遍等价的。

#### `continue`

不支持：

```c
for (int i = 0; i < 10; i++)
{
    if (ShouldSkip(i))
    {
        continue;
    }
    Process(i);
}
```

把剩余正文放进反向条件：

```c
for (int i = 0; i < 10; i++)
{
    if (!ShouldSkip(i))
    {
        Process(i);
    }
}
```

#### 循环 `break`

不支持：

```c
for (int i = 0; i < 10; i++)
{
    if (ShouldStop(i))
    {
        break;
    }
    Process(i);
}
```

把退出状态写入循环条件：

```c
int keep_running = 1;
for (int i = 0; i < 10 && keep_running; i++)
{
    if (ShouldStop(i))
    {
        keep_running = 0;
    }
    else
    {
        Process(i);
    }
}
```

`mary_break_switch;` 不是普通循环 break 的替代品；它只表达原 ROM 中“位于内层循环内、但目标是外层 switch 末尾”的特殊跳转。

上述限制属于当前 Mary-C 前端和反编译结构化器，不是 Mary VM 指令集的能力上限。VM 已有标签、无条件跳转和条件跳转，因此 `while`、`continue`、循环 `break`、`goto` 和用户标签在指令层都可以实现。要将它们纳入受支持语法，还必须同时完成编译跳转、反编译目标识别、嵌套循环归属和逐字节往返验证；不能只让新源码单向编译成功。

#### 控制转移后的词法不可达语句

同一语句块中，直接 `return;`、`break;`，或两侧都终止的 `if/else` 后面不能继续写语句：

```c
switch (state)
{
    case STATE_DONE:
        break;
        Cleanup(); /* 错误：永远无法执行。 */
}
```

Mary-C 会明确报告 `unreachable statement after return or break`。允许这种输入会生成结构化反编译器无法重新表示的尾部控制流，破坏源码往返。源码格式化器也会拒绝把含有这种尾部语句的旧式 AST 输出为 Mary-C，避免生成无法重新编译的源码。这里限制的是终止语句之后物理排列的代码；由运行时条件造成的原版不可达分支仍会按原结构保留，不会被编译器删除或改写。

#### `goto`、用户标签和任意跳转

当前不接受：

```c
goto retry;
retry:
    RetryOperation();
```

若跳转表达的是普通条件或循环，应重写成受支持的 `if`、`for`、`do/while` 或 `switch`。若原始控制流不能被这些结构无损表示，则当前没有安全的 Mary-C 高级写法；不能用 `mary_dead_jump:` 代替，因为它只保留 switch 内不可达的物理尾跳转，并不是用户标签。

#### 需要短路副作用的 `&&` 和 `||`

不能依赖这种 ISO C 行为：

```c
if (IsReady() && ConsumeItem())
{
    ContinueEvent();
}
```

应显式嵌套条件，保证第二个调用只在第一个条件成立后执行：

```c
if (IsReady())
{
    if (ConsumeItem())
    {
        ContinueEvent();
    }
}
```

当前 `&&`/`||` 只适合两边都是无副作用的数值或查询表达式。

#### 三元运算符

不支持：

```c
int result = condition ? when_true : when_false;
```

改为显式分支：

```c
int result;
if (condition)
{
    result = when_true;
}
else
{
    result = when_false;
}
```

#### 逗号运算符、字符字面量和有限编译期运算

当前不接受：

```c
int result = (First(), Second());
int letter = 'A';
int width = sizeof(int);
```

顺序副作用应拆成独立语句；单字节或 charmap 字符值应使用经过确认的整数或命名枚举；固定大小应写成经过验证的常量：

```c
First();
int result = Second();
int letter = LETTER_A;
int width = MARY_VM_INTEGER_WIDTH;
```

示例名称只有在相应枚举确实已由项目头文件定义时才能使用。Mary-C 不会自行把 Unicode 字符或宿主 C 的 `sizeof(int)` 猜成 ROM 数值。

#### 带返回值的脚本函数和普通辅助函数

不支持：

```c
int AddOne(int value)
{
    return value + 1;
}
```

Mary VM 的脚本入口只有 `void ScriptName(void)`。简单计算应直接写成表达式；游戏原生功能必须在有序 callable 表中声明后调用：

```c
int result = value + 1;
int affection = GetCharacterLove(CHARACTER_KAREN);
```

需要启动另一个事件脚本时使用 `CallScript(EventScript_Name);`，但被调用脚本不能像 C 函数一样返回一个值。

#### 指针、结构体、普通数组与动态内存

这类写法没有通用无损替代：

```c
Actor *actor = &actors[index];
actor->position.x = 10;
```

必须使用已经由 ROM callable 提供并确认语义的接口，例如：

```c
SetEntityPosition(entity_id, 10, 20, FACING_DOWN);
```

若 VM 没有对应 callable，就不能只靠 Mary-C 源码增加这种能力。脚本文本是唯一受支持的专用数组形式，应放在 `mary_text_table` 中。

#### 位运算、移位和类型转换

下列内容当前没有通用语法：

```c
int masked = value & 0x0F;
int shifted = value << 2;
int narrowed = (unsigned char)value;
```

只有在能证明取值范围和算术重写完全等价时，才可使用 `+ - * / %` 重写；否则必须调用已确认的原生 callable，或保留需求等待 VM/编译器增加明确指令支持。不能为了通过编译擅自换成近似计算。

浮点数同样没有通用替代。若游戏逻辑已有明确的定点比例，可以把比例和取值范围写进命名常量并使用整数运算；否则应使用已确认的原生 callable，不能把浮点字面量静默截断成整数。

#### 宏函数和复杂预处理

不支持把 C 宏当成可展开函数：

```c
#define IS_WEEKEND(day) ((day) == 0 || (day) == 6)
```

直接写表达式，固定编号则放入带类型的枚举：

```c
if (day == DAY_OF_WEEK_SUNDAY || day == DAY_OF_WEEK_SATURDAY)
{
    CallScript(EventScript_WeekendEvent);
}
```

目标差异仍可用受支持的条件块：

```c
#if defined(REGION_JP)
    TalkMessage(gText_JapaneseMessage);
#elif defined(REGION_US)
    TalkMessage(gText_EnglishMessage);
#endif
```

## Mary 专用语法

这些扩展只在标准 C 无法区分原 VM 字节布局时使用。它们不是额外的游戏功能，而是为了让看起来相同的高级语义仍能重建不同的原始字节。

### `mary_negated_int`

普通负数和具名负常量默认直接压入其负值：

```c
SetHeldTool(ITEM_TOOL_NOT_PRESENT);
```

若原字节码先压入正数再执行 `Neg`，反编译器则明确输出：

```c
SetHeldTool(mary_negated_int(ITEM_TOOL_NOT_PRESENT));
```

`mary_negated_int` 只接受一个可在编译期求值的整数常量。两种写法的高级数值都为 `-1`，但普通形式直接压入 `-1`，专用形式压入 `1` 后执行 `Neg`。该语法仅用于逐字节还原，不应作为普通算术或类型转换使用。

VM 的立即数和 `case` 值是 32 位物理字段。Mary-C 接受完整的有符号／无符号位模式
表示范围，即 `-2147483648` 至 `4294967295`（`0xFFFFFFFF`）；超出范围的值会报错，
不会静默截取低 32 位。整数文本本身超过 `i64`、常量运算溢出、除以零或取模零同样会
返回编译诊断，不会导致编译器崩溃。

### `X(...)` 与 `Y(...)`

`MaryMapId` 选择具体地图；`X(...)`、`Y(...)` 标记该地图自身局部像素空间中的坐标轴，不表示把所有地图拼接起来的世界坐标：

```c
ChangeMap(MAP_ZACK_HOUSE, X(120), Y(141));
SetEntityPosition(ENTITY_ZACK, X(130), Y(116), FACING_DOWN);
PanCameraTo(X(120), Y(208), CAMERA_MOVE_SPEED_NOMINAL_2_PIXELS_PER_UPDATE);
```

二者是带类型的恒等包装，不生成 callable 或额外 VM 指令。普通负坐标写作 `Y(-48)`；只有原字节码使用正值后取负结构时，才输出 `Y(mary_negated_int(-48))`。

### `mary_nodisc`

普通赋值会丢弃 VM 栈上的赋值结果：

```c
value = VarGet(VAR_HOUR);
```

如果原字节码在赋值后故意没有执行 `Discard`，反编译器会输出：

```c
mary_nodisc(value = VarGet(VAR_HOUR));
```

两者对变量 `value` 的表面效果相同，但后者还在虚拟栈上保留一份结果，因而字节不同。普通手写脚本不应为了省略指令而主动使用它。

### `mary_switch_compact`

普通 `switch` 按标准 Mary VM 布局生成尾部死跳转：

```c
switch (choice)
{
case 0:
    Accept();
    break;
default:
    Decline();
    break;
}
```

某些原 ROM 脚本省略了该死跳转，同样的高级语义要写成：

```c
mary_switch_compact (choice)
{
case 0:
    Accept();
    break;
default:
    Decline();
    break;
}
```

两段逻辑等价，区别只在 switch 调度区附近的物理指令布局。

### `mary_implicit_default`

标准 `default:` 会在 JUMP 块中生成明确的默认 case 表项：

```c
switch (choice)
{
default:
    Decline();
    break;
case 1:
    Accept();
    break;
}
```

原脚本有默认执行路径、但 JUMP 块没有默认表项时，使用：

```c
switch (choice)
{
mary_implicit_default:
    Decline();
    break;
case 1:
    Accept();
    break;
}
```

它保留“代码顺序中存在默认代码，但 case 表不引用它”的原始结构。

### `mary_dead_jump`

有些 switch 在物理代码中留有没有任何 case 会到达的跳转。标准 C 没有对应语义，Mary-C 用专用标记保留：

```c
switch (choice)
{
case 1:
    Accept();
    break;
mary_dead_jump:
}
```

`mary_dead_jump:` 不是可供 `goto` 调用的用户标签；它不在 case 表中建立入口，只按原样生成影响布局的不可达跳转。

### `mary_break_switch`

在内层循环中的标准 `break;` 只应退出该循环。如果原 VM 跳转直接指向外层 switch 末尾，则明确写成：

```c
switch (choice)
{
case 1:
    do
    {
        if (ShouldStop())
        {
            mary_break_switch;
        }
        Process();
    } while (HasMoreWork());
    Finish();
    break;
default:
    break;
}
```

`mary_break_switch;` 会同时穿过内层循环并到达外层 switch 的结束位置；它与未来支持的普通循环 `break;` 不是同一种跳转。

### 完整对照示例

下面的文件同时展示目标选择、本地头文件、脚本内文本、固定 ID、嵌套调用、条件、循环、switch、其他脚本调用和脚本退出：

```c
#define MARY_FOMT_JP
#include "fomt_constants.mary.h"
#include "fomt_callables.mary.h"
#include "fomt_scripts.mary.h"

mary_text_table
{
    const char gText_MorningGreeting[] =
        "おはよう。\r\n"
        "今日もがんばろう。{Press}";

    const char gText_EveningGreeting[] =
        "こんばんは。{Press}";
};

void EventScript_FarmMorningCheck(void)
{
    int hour = VarGet(VAR_HOUR);

    if (hour < 12)
    {
        TalkMessage(gText_MorningGreeting);
    }
    else
    {
        TalkMessage(gText_EveningGreeting);
    }

    for (int i = 0; i < 3; i++)
    {
        if (VarGet(VAR_WEATHER_TODAY) == WEATHER_RAIN)
        {
            TalkMessage(gText_EveningGreeting);
        }
    }

    switch (VarGet(VAR_DAY_OF_WEEK))
    {
        case DAY_OF_WEEK_SUNDAY:
        case DAY_OF_WEEK_SATURDAY:
            CallScript(EventScript_WeekendEvent);
            break;
        default:
            CallScript(638);
            break;
    }

    return;
}
```

若把上例循环体改成 `break;`，编译器会报告“循环 break 不受支持”。若只是跳过剩余循环体，应重新组织条件；若原 ROM 是“穿过内层循环退出外层 switch”的特殊布局，才使用 `mary_break_switch;`。Mary-C 不会为了看起来像 C 而伪造无法编码的控制流。

脚本调用的数字和符号形式完全等价：

```c
CallScript(638);
CallScript(EventScript_WeekendEvent);
```

前者固定调用物理 ID 638；后者使用当前目标的 `mary_script_table` 查找名称对应的 ID，适合脚本插入或版本表调整。两者最终都只向 VM 压入一个整数。

## 命令行

### 批量反编译 ROM 指针表

```console
mary decompile ROM goodies/fomt_callables.mary.h --all --mary-c --symbols goodies/fomt_scripts_text.mary.sym -D MARY_FOMT_JP --charmap charmap.txt -o OUTPUT
```

输出目录按物理槽位完整生成，空指针也有明确占位文件；同时复制常量/callable 头并生成目标脚本表。

六个 `MARY_FOMT_JP`／`MARY_FOMT_US`／`MARY_FOMT_EU`／`MARY_FOMT_DE`／
`MARY_MFOMT_JP`／`MARY_MFOMT_US` 宏是正式目标，每次必须且只能选择一个。

### 按脚本 ID 反编译

```console
mary decompile ROM goodies/fomt_callables.mary.h --script-id 867 --mary-c --symbols goodies/fomt_scripts_text.mary.sym -D MARY_FOMT_JP --charmap charmap.txt -o EventScript_0867.mary.c
```

### 从任意二进制偏移反编译

```console
mary decompile INPUT.bin goodies/fomt_callables.mary.h --offset 0x123456 --mary-c -D MARY_FOMT_JP --charmap charmap.txt -o Extracted.mary.c
```

`--script-id` 使用内置 ROM 版本的指针表；`--offset` 直接把给定文件偏移视作 RIFF 起点，接受十进制或带 `0x` 前缀的十六进制，两者不能同时使用。越过输入文件或指向无效 RIFF 会返回明确错误。

### 编译为单个 RIFF


```console
mary compile OUTPUT/EventScript_0867.mary.c --mary-c --charmap charmap.txt --binary -o EventScript_0867.riff
```

`--binary` 只允许一个脚本，并输出可直接比较或嵌入 ROM 的 RIFF 二进制。

### 编译为 C 字节数组

```console
mary compile OUTPUT/EventScript_0867.mary.c --mary-c --charmap charmap.txt -o EventScript_0867.c
```

不使用 `--binary` 时输出适合 C 工程引用的数据定义。这是输出容器选择，不表示 `.mary.c` 能由普通 C 编译器直接编译。

### 导入 ROM 副本

```console
mary import ROM EventScript.mary.c --script-id ID --charmap charmap.txt -o OUTPUT.gba
mary import ROM OUTPUT_DIRECTORY --charmap charmap.txt -o OUTPUT.gba
```

第一种形式在原分配区替换单个槽位；RIFF 可以使用原有的零值对齐填充，但不能
越过下一条脚本，未使用的原分配区会全部清零。第二种形式编译目录内提供的
文件；已有 ID 缺少文件时保留原 RIFF 字节，新增而未提供源码的槽位保持
`NULL`，其余脚本按四字节零填充规则重新紧凑排列。未指定新地址时，结果必须
容纳在原连续脚本区内，整体末尾未使用的原范围也会全部清零。

脚本块迁移且指针表扩容时使用：

```console
mary import ROM OUTPUT_DIRECTORY --address SCRIPT_ADDRESS \
  --relocate-pointer-table TABLE_ADDRESS --charmap charmap.txt -o OUTPUT.gba
```

地址可使用文件偏移或 GBA 地址，必须四字节对齐、位于 ROM 内，且不能互相
重叠，也不能覆盖原生指针引用或 Mary 元数据。目标区存在非 `00`/`FF` 字节时，
必须交互确认或显式使用 `--force`；`--dry-run` 只校验而不写文件。迁移后的表
通过写在废弃原生表中的带校验元数据自动发现；`--pointer-table` 与
`--pointer-count` 是手动恢复入口。Mary 会修补三处已经验证的原生表引用；
原生 `ScriptEngine::LoadById` 直接按 ID 索引表，不存在另一处硬编码槽数上限。

### 同时查看 IR

```console
mary decompile ROM goodies/fomt_callables.mary.h --script-id 867 --mary-c --print-ir -D MARY_FOMT_JP --charmap charmap.txt -o EventScript_0867.mary.c
mary compile EventScript_0867.mary.c --mary-c --print-ir --charmap charmap.txt -o EventScript_0867.c
```

`--print-ir` 在文本形式的反编译源码或 C 字节数组定义中把对应虚拟栈 IR 作为注释输出，用于审计高级结构和底层指令的对应关系；IR 注释不参与再次编译。`--binary` 是纯 RIFF 字节流，无法携带注释，不应与 `--print-ir` 组合。

源文件中的 `#define` 选择目标，`#include` 相对当前 `.mary.c` 解析。也可以使用 `--library` 和 `--script-table` 显式传入表文件；`-D` 仍可用于自动化和反编译阶段。若源文件已经定义目标，不必在命令行重复 `-D`。

## 验证范围

`tests/mary_c_vanilla.rs` 使用 `charmap.txt` 对六个原版 ROM 执行 ROM → Mary-C 文本 → RIFF 的逐字节严格往返。FoMT JP/US/EU/DE 各保留 1329 个物理槽位（其中 1328 个为非空 RIFF），MFoMT JP/US 各保留 1416 个物理槽位（其中 1415 个为非空 RIFF）；所有非空脚本都必须逐字节一致，空指针槽也必须保持原位。

`tests/mary_c_stress.rs` 另行覆盖多层 switch/if/do-while、紧凑 switch、case 贯穿、嵌套函数调用、虚拟栈保留结果与非法输入诊断。这里的 100% 只表示上述六个已验证 ROM；手写脚本仍必须属于 VM 可表达的语法范围。
