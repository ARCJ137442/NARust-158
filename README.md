# NARust 158

**简体中文** | [English](README.en.md)

<!-- TODO: 【2024-08-19 00:49:56】英文版有待翻译同步 -->

## 声明

关于术语「OpenNARS」的含义：未经详细区分，默认指代[`OpenNARS 1.5.8`](https://github.com/patham9/opennars_declarative_core)。

## 简介

![NARust Logo](./docs/image/opennars-logo-modified-with-rust.svg)

一个 [非公理推理系统](http://www.opennars.org/) 的 [Rust](https://www.rust-lang.org/) 版本，复刻自 [OpenNARS 1.5.8](https://github.com/patham9/opennars_declarative_core)。

## 在线演示

最新版本请 [点击这里](https://arcj137442.github.io/demo-158-dev/)

<!-- TODO: 【2024-08-19 00:48:50】继续根据其它NARS版本 丰富文档 -->

## 快速开始

### 获取源码

TODO

### 本地编译

TODO

### 构建运行

TODO

## 项目概览

### 系统模块架构

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
