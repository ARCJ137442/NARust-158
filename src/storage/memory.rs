//! 🎯复刻OpenNARS `nars.entity.Memory`
//! * 📌「记忆区」
//!
//! TODO: 🏗️【2024-05-06 00:19:43】有待着手开始；待[`crate::entity::Concept`]完成之后

use crate::{entity::ConceptConcrete, global::ClockTime};

/// 有关「记忆区报告」
/// * 🎯记忆区输出信息
/// * 🚩【2024-05-06 09:35:37】复用[`navm`]中的「NAVM输出」
mod report {
    use navm::output::Output;
    use std::collections::VecDeque;

    /// 缓存的「记忆区报告」
    /// * 🚩使用「NAVM输出」表示
    #[derive(Debug, Clone, Default)]
    pub struct MemoryReportCache {
        /// 输出缓冲区
        buffer: VecDeque<Output>,
    }

    impl MemoryReportCache {
        /// 构造函数
        /// * 🚩默认构造空数组
        #[inline]
        pub fn new() -> Self {
            Self::default()
        }

        /// 置入NAVM输出（在末尾）
        #[inline]
        pub fn put(&mut self, output: Output) {
            self.buffer.push_back(output)
        }

        /// 取出NAVM输出（在开头）
        /// * ⚠️可能没有（空缓冲区）
        #[inline]
        pub fn take(&mut self) -> Option<Output> {
            self.buffer.pop_front()
        }

        /// 长度大小
        #[inline]
        pub fn len(&self) -> usize {
            self.buffer.len()
        }

        /// 是否为空
        #[inline]
        pub fn is_empty(&self) -> bool {
            self.buffer.is_empty()
        }
    }
}
pub use report::*;

/// 模拟OpenNARS `nars.entity.Memory`
///
/// # 📄OpenNARS
///
/// The memory of the system.
pub trait Memory {
    /// 绑定的「概念」类型
    type Concept: ConceptConcrete;

    /// 模拟`Memory.getTime`
    /// * 🎯【2024-05-06 21:13:48】从[`Concept::get_belief`]来
    ///
    /// TODO: 🏗️【2024-05-06 21:14:33】后续要迁移
    ///
    /// # 📄OpenNARS
    ///
    /// 🈚
    #[doc(alias = "get_time")]
    fn time(&self) -> ClockTime {
        /* 📄OpenNARS源码：
        return reasoner.getTime(); */
        todo!("// TODO: 后续要迁移")
    }
}

/// [`Memory`]的具体版本
/// * 🎯规定「构造函数」「比对判等」等逻辑
pub trait MemoryConcrete: Memory + Sized {}
