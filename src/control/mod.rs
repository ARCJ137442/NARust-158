//! 🆕NARS控制机制
//! * 🎯集中实现以「工作周期/推理循环」为入口的「推理控制」功能
//!   * 📌推理上下文：直接推理/转换推理/概念推理
//!   * 📌工作周期
//!   * 📌推理过程：直接推理/概念推理
//! * ⚠️此处代码与[原版OpenNARS 1.5.8](https://github.com/patham9/opennars_declarative_core)已有很大不同，不建议完全参考其源码
//!
//! TODO: 【2024-05-22 02:11:59】🚧按照改版重写此模块

// 超参数
mod parameters;
pub use parameters::*;

// 推理器
mod reasoner;
pub use reasoner::*;

// 上下文
mod context;
pub use context::*;

// 概念链接
mod concept_linking;
pub use concept_linking::*;

// 工作过程
mod process;
pub use process::*;
