//! 🆕新的「推理上下文」类型
//! * 🎯用于统一在NARS推理中采用的「推理对象」的特征实现
//! * 🎯提供实际推理控制所用到的、单周期内有效的「临时推理结构」
//! * ✨允许「相同的上层接口，不同的底层实现」

nar_dev_utils::mods! {
    // 定义
    pub use definition;
    // 导出（输出、任务）
    pub use derivation;
    // TODO: 直接推理
    // TODO: 转换推理
    // TODO: 概念推理
}
