//! 作为特征定义的「推理引擎」
//!
//! ## Logs
//!
//! * ♻️【2024-06-26 12:11:03】开始根据改版OpenNARS编写

use crate::control::{ReasonContextConcept, ReasonContextDirect, ReasonContextTransform};
use serde::{Deserialize, Serialize};

/// 作为通用接口的「推理引擎」特征
/// * 📌只负责处理「推理上下文」
/// * 🚩【2024-06-28 01:24:34】现在从「特征对象」降级到「函数指针集合」
///   * 💡实际上只是需要动态分派几个函数而已——况且，这些函数一般也会静态存在（不是闭包什么的）
///   * 💭【2024-06-28 01:26:37】这个「引擎分派」本身就是个`VTable`嘛……
///
/// TODO: 【2024-08-11 16:04:42】函数指针/闭包 的 序列反序列化
#[derive(Debug, Clone, Copy)]
pub struct InferenceEngine {
    /// 直接推理 入口函数
    /// * 📌接收 [直接推理上下文](ReasonContextDirect)
    /// * 📝不建议迁出去当类型别名：生命周期参数需要额外补充
    #[doc(alias = "direct_process")]
    direct: fn(&mut ReasonContextDirect),

    /// 转换推理 入口函数
    /// * 📌接收 [转换推理上下文](ReasonContextTransform)
    /// * 📝不建议迁出去当类型别名：生命周期参数需要额外补充
    #[doc(alias = "transform_task")]
    transform: fn(&mut ReasonContextTransform),

    /// 匹配推理 入口函数
    /// * 📌接收 [概念推理上下文](ReasonContextConcept)
    /// * 📝不建议迁出去当类型别名：生命周期参数需要额外补充
    #[doc(alias = "match_links")]
    matching: fn(&mut ReasonContextConcept),

    /// 概念推理 入口函数
    /// * 📌接收 [概念推理上下文](ReasonContextConcept)
    /// * 📝不建议迁出去当类型别名：生命周期参数需要额外补充
    #[doc(alias = "concept_reason")]
    reason: fn(&mut ReasonContextConcept),
}

impl InferenceEngine {
    // 使用函数指针构造
    #[inline]
    pub const fn new(
        direct: fn(&mut ReasonContextDirect),
        transform: fn(&mut ReasonContextTransform),
        matching: fn(&mut ReasonContextConcept),
        reason: fn(&mut ReasonContextConcept),
    ) -> Self {
        Self {
            direct,
            transform,
            matching,
            reason,
        }
    }

    /// 空指针引擎
    /// * 📌这个引擎「什么都不做」
    pub const VOID: Self = {
        // 三个空函数
        fn direct(_: &mut ReasonContextDirect) {}
        fn transform(_: &mut ReasonContextTransform) {}
        fn matching(_: &mut ReasonContextConcept) {}
        fn reason(_: &mut ReasonContextConcept) {}
        // 构造自身
        Self::new(direct, transform, matching, reason)
    };

    /// 打印回显的推理引擎
    /// * ✨可用于调试控制机制
    pub const ECHO: Self = {
        use crate::{
            control::{ReasonContext, ReasonContextWithLinks},
            util::{RefCount, ToDisplayAndBrief},
        };
        use nar_dev_utils::OptionBoost;

        /// 直接推理
        fn direct(context: &mut ReasonContextDirect) {
            context.report_comment(format!(
                "#Inference - Direct:\nconcept: {}\ntask: {}",
                context.current_concept().to_display_long(),
                context.current_task().get_().to_display_long(),
            ))
        }

        /// 转换推理
        fn transform(context: &mut ReasonContextTransform) {
            context.report_comment(format!(
                "#Inference - Transform:\nconcept: {}\ntask-link: {}",
                context.current_concept().to_display_long(),
                context.current_task_link().to_display_long(),
            ))
        }

        /// 匹配推理
        fn matching(context: &mut ReasonContextConcept) {
            context.report_comment(format!(
                "#Inference - Matching:\nconcept: {}\ntask-link: {}\nbelief-link: {}",
                context.current_concept().to_display_long(),
                context.current_task_link().to_display_long(),
                context.current_belief_link().to_display_long(),
            ))
        }

        /// 概念推理
        fn reason(context: &mut ReasonContextConcept) {
            context.report_comment(format!(
                "#Inference - Reason:\nconcept: {}\ntask-link: {}\nbelief-link: {}\nbelief: {}",
                context.current_concept().to_display_long(),
                context.current_task_link().to_display_long(),
                context.current_belief_link().to_display_long(),
                context
                    .current_belief()
                    .map_unwrap_or(ToDisplayAndBrief::to_display_long, "None".into()),
            ))
        }

        // 返回
        Self::new(direct, transform, matching, reason)
    };

    /// 获取「推理函数 @ 直接推理」
    /// * ✅不会长期借用`self`：允许「推理引擎」作为「推理上下文」的一部分（被引用）
    /// * 🚩【2024-07-02 17:38:22】四个均可作为「常量函数」被调用
    ///   * 📝Rust中引用字段的函数均可如此
    pub const fn direct_f(&self) -> fn(&mut ReasonContextDirect) {
        self.direct
    }

    /// 获取「推理函数 @ 转换推理」
    /// * ✅不会长期借用`self`：允许「推理引擎」作为「推理上下文」的一部分（被引用）
    pub const fn transform_f(&self) -> fn(&mut ReasonContextTransform) {
        self.transform
    }

    /// 获取「推理函数 @ 匹配推理」
    /// * ✅不会长期借用`self`：允许「推理引擎」作为「推理上下文」的一部分（被引用）
    /// * 🚩【2024-07-02 17:38:22】四个均可作为「常量函数」被调用
    ///   * 📝Rust中引用字段的函数均可如此
    pub const fn matching_f(&self) -> fn(&mut ReasonContextConcept) {
        self.matching
    }

    /// 获取「推理函数 @ 概念推理」
    /// * ✅不会长期借用`self`：允许「推理引擎」作为「推理上下文」的一部分（被引用）
    /// * 🚩【2024-07-02 17:38:22】四个均可作为「常量函数」被调用
    ///   * 📝Rust中引用字段的函数均可如此
    pub const fn reason_f(&self) -> fn(&mut ReasonContextConcept) {
        self.reason
    }
}
