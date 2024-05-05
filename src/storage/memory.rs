//! 🎯复刻OpenNARS `nars.entity.Memory`
//! * 📌「记忆区」
//!
//! TODO: 🏗️【2024-05-06 00:19:43】有待着手开始；待[`crate::entity::Concept`]完成之后

/// 模拟OpenNARS `nars.entity.Memory`
///
/// # 📄OpenNARS
///
/// The memory of the system.
pub trait Memory {}

/// [`Memory`]的具体版本
/// * 🎯规定「构造函数」「比对判等」等逻辑
pub trait MemoryConcrete: Memory + Sized {}
