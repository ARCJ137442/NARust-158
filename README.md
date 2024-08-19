# NARust 158

**简体中文** | [English](README.en.md)

🕒最后更新：2024-08-19

## 简介

<!-- 📝尺寸参考：<https://stackoverflow.com/questions/14675913/changing-image-size-in-markdown> -->
![logo](./docs/image/opennars-logo-modified-with-rust.svg)

一个 [非公理推理系统](http://www.opennars.org/) 的 [Rust](https://www.rust-lang.org/) 版本，复刻自 [OpenNARS 1.5.8](https://github.com/patham9/opennars_declarative_core)。

## 在线演示

得益于Rust与WebAssembly的集成技术，该推理系统具有[网页版](https://arcj137442.github.io/demo-158-dev/)，并可直接在支持WebAssembly的浏览器中运行。

（支持WebAssembly的浏览器：Chrome 57+、EDGE 16+、Safari 11+、Firefox 52+、Opera 44+、……）

若需自行将其编译到网页端，可参考 [RustWasm](https://rustwasm.github.io/) 与 [`wasm-pack`](https://rustwasm.github.io/wasm-pack/) 工具，另可自行搜索相关资源。

## 快速开始

### 前置条件

1. 在系统中安装 [**Rust**](https://www.rust-lang.org/tools/install) 编译工具链
2. 确保安装后 `cargo` 命令可用

![安装 Cargo 后，在系统命令行中输入 `cargo` 的预期结果](./docs/image/installed-cargo.png)

### 尝鲜：即刻运行

在**有网络**环境下，直接运行如下命令：

```bash
cargo install narust-158
```

截止至目前（2024-08-19），该命令会在系统中安装如下两个二进制文件：

- `narust_158_shell`：便于用户交互，可直接输入Narsese语句和数值（推理器步进指定周期）
- `narust_158_batch`：便于外部集成，统一输入NAVM指令，并输入格式固定的单行JSON文本

### 进阶：源码编译

#### 获取源码

可以直接从GitHub上的项目仓库中获取源码：

```bash
git clone https://github.com/arcj137442/narust-158.git
```

应该预期到如下反应：Git从GitHub仓库中获取到源码，并下载到特定目录下的 `narust-158` 文件夹中。

在项目发布到**crates.io**后，可在Rust工程目录下通过如下命令获取：

```bash
cargo add narust-158
```

#### 本地编译

从GitHub上获取的源码，可在 `narust-158` 根目录下的命令行中输入如下命令构建：

```bash
cargo build
```

应该预期到Cargo自动下载编译依赖，并最终完成对项目二进制文件的编译：

```bash
[...]> cargo build
   Compiling narust-158 vX.X.X ([...])
    Finished dev [unoptimized + debuginfo] target(s) in X.XXs
```

#### 构建运行

此时可使用命令 `cargo run` 运行构建好的二进制文件：

```bash
cargo run --bin narust_158_shell
```

或

```bash
cargo run --bin narust_158_batch
```

预期：命令行光标开启新的一行，并等待用户输入。

在输入如下NAVM指令后，

```navm-cmd
nse <A --> B>.
nse <B --> C>.
nse <A --> C>?
cyc 20
```

应预期到如下输出：

```plaintext
nse <A --> B>.
[IN] $0.8000;0.8000;0.9500$ <A --> B>. %1.0000;0.9000%
nse <B --> C>.
[IN] $0.8000;0.8000;0.9500$ <B --> C>. %1.0000;0.9000%
nse <A --> C>?
[IN] $0.9000;0.9000;1.0000$ <A --> C>?
cyc 20
[ANSWER] <A --> C>. %1.0000;0.8100%
```

（↑在`shell`中）

```plaintext
nse <A --> B>.
{"type":"IN","content":"In: $0.80;0.80;0.95$ (A --> B). %1.00;0.90%","narsese":"$0.8000;0.8000;0.9500$ <A --> B>. %1.0000;0.9000%"}
nse <B --> C>.
{"type":"IN","content":"In: $0.80;0.80;0.95$ (B --> C). %1.00;0.90%","narsese":"$0.8000;0.8000;0.9500$ <B --> C>. %1.0000;0.9000%"}
nse <A --> C>?
{"type":"IN","content":"In: $0.90;0.90;1.00$ (A --> C)?","narsese":"$0.9000;0.9000;1.0000$ <A --> C>?"}
cyc 20
{"type":"ANSWER","content":"Answer: (A --> C). %1.0000;0.8100%{16 : 2;1}","narsese":"<A --> C>. %1.0000;0.8100%"}
```

（↑在`batch`中）

## 项目概览

### 声明

关于术语「OpenNARS」的含义：未经详细区分，默认指代[`OpenNARS 1.5.8`](https://github.com/patham9/opennars_declarative_core)，另可参考个人的[中文笔记附注定制版](https://github.com/ARCJ137442/OpenNARS-158-dev)

- 📌项目结构主要基于「中文笔记附注定制版」（后称「改版OpenNARS」）
- ⚠️项目中的注释与笔记均使用中文编写

### 系统模块架构

整个系统的主要文件夹结构如下：

```plaintext
narust-158
├── docs
├── src
│   ├── bin:        可执行文件编译入口
│   ├── control:    控制机制
│   ├── entity:     实体型结构
│   ├── inference:  推理机制
│   ├── language:   知识表示语言
│   ├── util:       内用工具函数
│   ├── vm:         虚拟机接口及自带实现
│   ├── global.rs:  全局参数
│   ├── lib.rs:     库编译入口
│   ├── symbols.rs: 全局符号常量，对应OpenNARS `nars.io.Symbols`
│   └── ...
├── Cargo.toml
└── ...
```

#### 知识表示语言

语言模块 `src/language`：有关「词项」的定义及处理逻辑

```plaintext
language
├── term_impl: 词项结构具体实现
│   ├── base: 基础功能
│   │   ├── construct.rs:  构造函数
│   │   ├── conversion.rs: 类型转换
│   │   ├── property.rs:   属性
│   │   ├── serde.rs:      序列反序列化
│   │   ├── structs.rs:    结构体定义
│   │   └── ...
│   ├── dialect: 方言语法
│   │   ├── mod.rs:              方言解析器
│   │   └── narust_dialect.pest: 语法定义
│   ├── features: 对应OpenNARS的特性
│   │   ├── compound_term.rs: 对应OpenNARS类 `CompoundTerm`
│   │   ├── image.rs:         对应OpenNARS类 `Image`
│   │   ├── statement.rs:     对应OpenNARS类 `Statement`
│   │   ├── term.rs:          对应OpenNARS类 `Term`
│   │   ├── variable.rs:      对应OpenNARS类 `Variable`
│   │   └── ...
│   ├── term_making.rs:      对应OpenNARS `MakeTerm.java`
│   ├── variable_process.rs: 对应OpenNARS `VariableProcess.java`
│   └── ...
└── ...
```

实体模块 `src/entity`：「真值」「预算值」「语句」「词项链&任务链」「概念」「时间戳」「任务」等结构的定义

```plaintext
entity
├── float_values: 语言机制、控制机制共用的「浮点值」
│   ├── budget_value.rs: 预算值
│   ├── truth_value.rs:  真值
│   ├── short_float.rs:  短浮点（四位小数）
│   └── ...
├── sentence: 语言机制有关「语句」的定义
│   ├── impls:             初代实现（语句、判断、问题）
│   ├── judgement.rs:      统一的「判断句」接口
│   ├── punctuation.rs:    基于枚举的「标点」定义
│   ├── question.rs:       统一的「疑问句」接口
│   ├── sentence_trait.rs: 统一的「语句」接口
│   └── ...
├── linkages: 控制机制有关「链接」的定义
│   ├── t_link.rs:             统一的「链接」接口
│   ├── t_linkage.rs:          通用的「链接」结构
│   ├── task_link.rs:          任务链
│   ├── term_link_template.rs: 词项链模板
│   ├── term_link.rs:          词项链
│   └── ...
├── concept.rs: 控制机制结构「概念」
├── item.rs:    控制机制接口「物品」
├── stamp.rs:   语言机制结构「时间戳」
├── task.rs:    控制机制结构「任务」
└── ...
```

（此结构源自OpenNARS 1.5.8，代码在推理与控制机制上并未完全分离）

#### 推理控制机制

NARS存储容器 `src/storage`：有关「实体」的存储容器

```plaintext
storage
├── bag: 基于「伪随机优先队列」的控制机制基础容器「袋」
│   ├── distributor.rs: 基于优先级的三角分布伪随机分派器
│   ├── impl_tables.rs: 附属的「名称表」「层级表」结构
│   ├── impl_v1.rs:     最终导出的「初代实现」
│   └── ...
├── buffer.rs:     在「概念」中使用的「缓冲区」结构
├── memory.rs:     存储「概念」的整体容器「记忆区」
├── rank_table.rs: 在「概念」中使用的「排行表」结构
└── ...
```

NARS推理功能 `src/inference`：基于NAL与「知识表示语言」机制，在「容器」中处理各类「实体」的过程

```plaintext
inference
├── engine: 通用的「推理引擎」接口
│   ├── inference_engine.rs: 推理引擎定义与接口
│   └── ...
├── functions: 真值函数、预算值函数等
│   ├── budget_functions.rs:  NAL中有关「预算函数」的代码，对应 `nars.inference.BudgetFunctions`
│   ├── truth_functions.rs:   NAL中的「真值函数」，对应 `nars.inference.TruthFunctions`
│   ├── utility_functions.rs: NAL中有关「扩展逻辑运算」的代码，对应 `nars.inference.UtilityFunctions`
│   └── ...
├── rules: 具体NAL推理规则
│   ├── table: 规则分派表
│   │   ├── entry.rs:         规则分派入口，对应 `nars.inference.RuleTables`
│   │   ├── syllogistic.rs:   有关「三段论规则」的分派
│   │   ├── compositional.rs: 有关「组合规则」的分派
│   │   └── ...
│   ├── compositional_rules.rs: 组合规则，对应 `nars.inference.CompositionalRules`
│   ├── local_rules.rs:         本地规则，对应 `nars.inference.LocalRules`
│   ├── matching_rules.rs:      匹配规则，对应 `nars.inference.MatchingRules`
│   ├── structural_rules.rs:    结构规则，对应 `nars.inference.StructuralRules`
│   ├── syllogistic_rules.rs:   三段论规则，对应 `nars.inference.SyllogisticRules`
│   ├── transform_rules.rs:     转换规则，对应 `nars.inference.TransformRules`
│   └── ...
├── traits
│   ├── budget.rs:     有关「预算」的抽象接口，被「预算值」「任务」「概念」等共用
│   ├── evidential.rs: 有关「证据基」的抽象接口，被「时间戳」「语句」「任务」等共用
│   ├── truth.rs:      有关「真值」的抽象接口，被「真值」「判断句」等共用
│   └── ...
├── budget_inference.rs: 涉及「链接反馈」的「预算推理」
├── local_inference.rs:  涉及「信念修正」「问题解答」的「直接推理」
└── ...
```

NARS控制机制 `src/control`：在「容器」与「推理规则」之上、与「推理器」密切相关的功能

```plaintext
control
├── context: 控制机制中的「推理上下文」功能
│   ├── context_concept.rs:   概念推理上下文
│   ├── context_direct.rs:    直接推理上下文
│   ├── context_transform.rs: 转换推理上下文
│   ├── derivation.rs:        推理导出相关功能
│   ├── reason_context.rs:    统一的「推理上下文」接口
│   └── ...
├── process: 有关「工作周期」的运行时功能
│   ├── concept_linking.rs: 概念链接（构建任务链、词项链）
│   ├── parsing_task.rs:    Narsese任务解析功能
│   ├── process_direct.rs:  涉及「直接推理」的控制过程
│   ├── process_reason.rs:  涉及「概念推理」的控制过程
│   ├── work_cycle.rs:      工作周期控制
│   └── ...
├── reasoner: 推理器自身定义及其外部API
│   ├── definition.rs:       推理器的数据结构定义
│   ├── derivation_datas.rs: 附属数据结构，包括「新任务队列」与「新近任务袋」
│   ├── report.rs:           输出报告功能
│   ├── serde.rs:            序列反序列化功能
│   ├── vm_api.rs:           NAVM虚拟机API
│   └── ...
├── parameters.rs: 推理器超参数
└── ...
```

#### 对外应用接口

NAVM虚拟机自带实现 `src/vm`：基于「推理器」与NAVM API，对外提供一个自带的NAVM实现

```plaintext
vm
├── alpha: 虚拟机自带内核「Alpha」
│   ├── cmd_dispatch: NAVM指令分派
│   │   ├── cmd_hlp.rs: 处理指令 `HLP`
│   │   ├── cmd_inf.rs: 处理指令 `INF`
│   │   ├── cmd_loa.rs: 处理指令 `LOA`
│   │   ├── cmd_sav.rs: 处理指令 `SAV`
│   │   └── mod.rs:     顶层分派功能
│   ├── io: 虚拟机层面的输入输出，对应原OpenNARS的通道机制
│   │   ├── _channel.rs:       有关「通道」的抽象特征
│   │   ├── channel_in.rs:     输入通道初步实现
│   │   ├── channel_out.rs:    输出通道初步实现
│   │   ├── channels.rs:       在推理器之上管理输入输出通道
│   │   ├── handle_io.rs:      有关通道输入输出的实际逻辑
│   │   ├── input_channel.rs:  输入通道抽象特征
│   │   ├── output_channel.rs: 输出通道抽象特征
│   │   └── ...
│   ├── launcher.rs: 虚拟机启动器
│   ├── runtime.rs:  虚拟机运行时
│   └── ...
└── ...
```

### 所用语言特性

📝代码所涉及的主要Rust语言特性（部分有理解难度）：

- 模式匹配
- 泛型（静态分派）
- 特征（抽象接口）
- 闭包（用于简化可重用代码的临时闭包）
- 模块
- 宏（声明宏）

⚠️需要注意的、可能较难理解的特性：

- `unsafe`代码（复合词项可变引用）
- 特征对象（动态分派）

### 潜在应用领域

- Web开发：让NARS在浏览器运行，便于集成NARS到互联网应用中
- 嵌入式：让NARS在嵌入式设备中运行，用于小内核、强专用性的场景
- 工业应用：让NARS在工业设备中运行，用于高性能、低运行开销的场景

## 贡献指南

### 项目分支情况

- `main`：基础版本，长期支持
- `dev`：开发版本，持续开发（ℹ️**PR主要在此分支合并**）
- `dev-XXX`：衍生分支，用于学习或实验
- `archive-XXX`：存档分支，提供长期稳定不变的代码
- `debug-XXX`：临时分支，用于解决问题/漏洞

### 贡献途径

- [Github Issues](https://github.com/ARCJ137442/Narust-158/issues)：反馈问题、建议、bug
- [GitHub Pull Request](https://github.com/ARCJ137442/Narust-158/pulls)：直接向项目贡献代码

## 工程日志

### 【2024-08-19 00:28:53】

📌指导原则：**固定基础，分支延伸**

1. 📍对外提供一个接近[「声明性内核」](https://github.com/patham9/opennars_declarative_core)的**基础版本**（LTS 长期支持）
    - 🏗️为整个NARS工程生态做一份实实在在的贡献
    - 🔦为后人（特别是Rustacean）在工程上研究NARS提供一个奠基石
2. 💡在「基础版本」的基石之上，鉴于自己对Rust的熟悉，继续延伸并扩展，发掘其特有的研究与应用价值
    - 🔬研究：凭借自己对内核的熟悉，向NAL 7~9「事件推理」「过程推理」方向探索
    - 🪛应用：凭借Rust程序在性能、安全性、可集成性方面的优势，发掘其在Rust擅长的高性能工业生产、嵌入式、互联网等方面的应用潜力

### 【2024-06-14 00:41:48】

📌核心原则：**先重构，再迁移，最后独立**

1. 💡先在构建好Java开发环境的基础上，通过「同义重构」最大限度利用Java的开发速度、快速验证优势
    - ❗改版不可怕，可怕的是「迁移到一半又要推翻重来」
2. 🚚迁移前先在Java探明并**稳定**各个类、接口、字段、方法的可空性、可变性、引用共享性、变量所有权情况
    - 🎯力求「多次验证，一次迁移，一次稳定」
3. 🏗️迁移到Rust并基本稳定后，开始对齐已有的测试工具链，并在「对齐OpenNARS 1.5.8」后开源公布
    - 🎯公开发帖，力求在七月前完成
