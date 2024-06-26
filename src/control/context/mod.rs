//! 🆕新的「推理上下文」类型
//! * 🎯用于统一在NARS推理中采用的「推理对象」的特征实现
//! * 🎯提供实际推理控制所用到的、单周期内有效的「临时推理结构」
//! * ✨允许「相同的上层接口，不同的底层实现」
//!
//! * ♻️【2024-06-26 11:45:46】开始根据改版OpenNARS重写

nar_dev_utils::mods! {
    // 推理上下文/导出上下文
    pub use reason_context;

    // 直接推理上下文
    pub use context_direct;

    // 转换推理上下文
    pub use context_transform;

    // 概念推理上下文
    pub use context_concept;
}
