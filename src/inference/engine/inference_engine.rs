//! 作为特征定义的「推理引擎」
//!
//! ## Logs
//!
//! * ♻️【2024-06-26 12:11:03】开始根据改版OpenNARS编写

/// 作为通用接口的「推理引擎」特征
/// * 📌只负责处理「推理上下文」
/// * 📌需要能作为**特征对象**
pub trait InferenceEngine {
    // TODO: 等待「推理上下文」完成
    /// 概念推理 入口函数
    fn reason(&mut self, context: &mut ());

    /// 转换推理 入口函数
    fn transform(&mut self, context: &mut ());
}

pub type InferenceEngineObj = Box<dyn InferenceEngine>;
