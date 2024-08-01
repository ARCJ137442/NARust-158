//! 🎯复刻OpenNARS `nars.inference.RuleTables`
//! * 🚩直接调用所有具体规则，或调用子分派（如 三段论规则的分派）
//!   * 📌核心：不直接涉及「导出结论」
//!
//! ## Logs
//!
//! * ♻️【2024-07-10 21:44:07】开始根据改版OpenNARS重写
//! * ♻️【2024-08-01 21:02:11】开始再重构「分派部分」与「规则部分」

// 三段论规则分派
mod syllogistic;

// 规则表入口
mod entry;
pub use entry::*;

/// 一些通用函数
#[cfg(test)]
pub(super) mod tests {
    use super::*;
    use crate::inference::{process_direct, transform_task, InferenceEngine};

    /// 概念推理专用测试引擎
    /// * 🚩【2024-07-14 23:51:32】禁掉了转换推理
    pub const ENGINE_REASON: InferenceEngine = InferenceEngine::new(
        process_direct,
        transform_task,
        InferenceEngine::VOID.matching_f(),
        reason,
    );
}
