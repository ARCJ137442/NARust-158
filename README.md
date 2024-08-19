# NARust 158

**简体中文** | [English](README.en.md)

<!-- TODO: 【2024-08-19 00:49:56】英文版有待翻译同步 -->

## 声明

关于术语「OpenNARS」的含义：未经详细区分，默认指代[`OpenNARS 1.5.8`](https://github.com/patham9/opennars_declarative_core)。

## 简介

![NARust Logo](./docs/image/opennars-logo-modified-with-rust.svg)

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

🕒最后更新：2024-08-19

### 系统模块架构

整个系统的文件夹结构如下：

```text
narust-158/
├── docs/
├── src/

<!-- TODO: 【2024-08-19 00:48:50】继续根据其它NARS版本 丰富文档 -->

#### 知识表示语言

TODO

#### 推理控制机制

TODO

#### 对外应用接口

TODO

### 所用语言特性

TODO

### 潜在应用领域

## 贡献指南

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
