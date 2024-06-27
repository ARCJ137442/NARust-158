//! 作为特征定义的「推理引擎」
//!
//! ## Logs
//!
//! * ♻️【2024-06-26 12:11:03】开始根据改版OpenNARS编写

use crate::control::ReasonContextConcept;

/// 作为通用接口的「推理引擎」特征
/// * 📌只负责处理「推理上下文」
/// * 📌需要能作为**特征对象**
pub trait InferenceEngine {
    /// 概念推理 入口函数
    /// * 🚩Rust的「对象安全」要求方法必须带`self`参数
    fn reason(&mut self, context: &mut ReasonContextConcept);

    /// 转换推理 入口函数
    /// * 🚩Rust的「对象安全」要求方法必须带`self`参数
    fn transform(&mut self, context: &mut ReasonContextConcept);
}

pub type InferenceEngineObj = Box<dyn InferenceEngine>;
