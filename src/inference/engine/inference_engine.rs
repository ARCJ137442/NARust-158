//! 作为特征定义的「推理引擎」
//!
//! ## Logs
//!
//! * ♻️【2024-06-26 12:11:03】开始根据改版OpenNARS编写

use crate::control::{ReasonContextConcept, ReasonContextDirect, ReasonContextTransform};

/// 作为通用接口的「推理引擎」特征
/// * 📌只负责处理「推理上下文」
/// * 🚩【2024-06-28 01:24:34】现在从「特征对象」降级到「函数指针集合」
///   * 💡实际上只是需要动态分派几个函数而已——况且，这些函数一般也会静态存在（不是闭包什么的）
///   * 💭【2024-06-28 01:26:37】这个「引擎分派」本身就是个`VTable`嘛……
#[derive(Debug, Clone, Copy)]
pub struct InferenceEngine {
    /// 直接推理 入口函数
    /// * 📌接收 [直接推理上下文](ReasonContextDirect)
    /// * 🚩Rust的「对象安全」要求方法必须带`self`参数
    /// * 📝不建议迁出去当类型别名：生命周期参数需要额外补充
    #[doc(alias = "direct_process")]
    direct: fn(&mut ReasonContextDirect),

    /// 转换推理 入口函数
    /// * 📌接收 [转换推理上下文](ReasonContextTransform)
    /// * 🚩Rust的「对象安全」要求方法必须带`self`参数
    /// * 📝不建议迁出去当类型别名：生命周期参数需要额外补充
    #[doc(alias = "transform_task")]
    transform: fn(&mut ReasonContextTransform),

    /// 概念推理 入口函数
    /// * 📌接收 [概念推理上下文](ReasonContextConcept)
    /// * 🚩Rust的「对象安全」要求方法必须带`self`参数
    /// * 📝不建议迁出去当类型别名：生命周期参数需要额外补充
    #[doc(alias = "concept_reason")]
    reason: fn(&mut ReasonContextConcept),
}

impl InferenceEngine {
    pub fn new(
        direct: fn(context: &mut ReasonContextDirect),
        reason: fn(context: &mut ReasonContextConcept),
        transform: fn(context: &mut ReasonContextTransform),
    ) -> Self {
        Self {
            direct,
            reason,
            transform,
        }
    }

    /// 获取「推理函数 @ 直接推理」
    /// * ✅不会长期借用`self`：允许「推理引擎」作为「推理上下文」的一部分（被引用）
    pub fn direct_f(&self) -> fn(&mut ReasonContextDirect) {
        self.direct
    }

    /// 获取「推理函数 @ 转换推理」
    /// * ✅不会长期借用`self`：允许「推理引擎」作为「推理上下文」的一部分（被引用）
    pub fn transform_f(&self) -> fn(&mut ReasonContextTransform) {
        self.transform
    }

    /// 获取「推理函数 @ 概念推理」
    /// * ✅不会长期借用`self`：允许「推理引擎」作为「推理上下文」的一部分（被引用）
    pub fn reason_f(&self) -> fn(&mut ReasonContextConcept) {
        self.reason
    }
}
