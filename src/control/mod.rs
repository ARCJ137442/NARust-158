//! 🆕NARS控制机制
//! * 🎯集中实现以「工作周期/推理循环」为入口的「推理控制」功能
//!   * 📌推理上下文：直接推理/转换推理/概念推理
//!   * 📌工作周期
//!   * 📌推理过程：直接推理/概念推理
//! * ⚠️此处代码与[原版OpenNARS 1.5.8](https://github.com/patham9/opennars_declarative_core)已有很大不同，不建议完全参考其源码
//!
//! TODO: 【2024-05-22 02:11:59】🚧按照改版重写此模块

nar_dev_utils::mods! {
    // // 上下文
    // pub use context;

    // 概念链接
    pub use concept_linking;

    // // 全局（工作周期）
    // pub use global;
}
