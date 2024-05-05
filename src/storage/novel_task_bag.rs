//! 🎯复刻OpenNARS `nars.entity.NovelTaskBag`
//!
//! * ✅【2024-05-06 00:13:38】基本功能、初代实现完成

use super::Bag;
use crate::entity::Task;

/// 模拟OpenNARS `nars.entity.NovelTaskBag`
/// * 📌【2024-05-04 17:30:35】实际上就是「袋+任务+特定参数」
///   * 📌目前不限制构造过程（即 不覆盖方法）
/// * 🚩有关「固定容量」与「遗忘时长」交给构造时决定
///   * ✅这也能避免冗余的对「记忆区」的引用
pub trait NovelTaskBag: Bag<Self::Task> {
    /// 绑定的「任务」类型
    /// * 🎯一种实现只能对应一种「任务袋」
    type Task: Task;
}

mod impl_v1 {
    use super::*;
    use crate::{
        entity::{BudgetV1, SentenceV1, StampV1, TaskV1, TruthV1},
        storage::{BagKeyV1, BagV1},
    };

    /// 自动为「任务+[`BagKeyV1`]+[`BagV1`]」实现「新近任务袋」
    impl<T: Task<Key = BagKeyV1>> NovelTaskBag for BagV1<T> {
        type Task = T;
    }

    /// 初代[`NovelTaskBag`]实现
    /// * 🚩【2024-05-05 22:29:47】只需限定一系列类型，而无需再声明新`struct`
    pub type NovelTaskBagV1 = BagV1<TaskV1<SentenceV1<TruthV1, StampV1>, BagKeyV1, BudgetV1>>;
}
pub use impl_v1::*;

// * ✅单元测试参见`super::Bag`
