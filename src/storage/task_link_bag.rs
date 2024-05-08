//! 🎯复刻OpenNARS `nars.entity.TaskLinkBag`
//! * 📌「任务链袋」
//! * ✅【2024-05-04 17:50:50】基本功能复刻完成
//! * ✅【2024-05-06 00:13:38】初代实现完成

use super::BagConcrete;
use crate::entity::TaskLinkConcrete;

/// 模拟`nars.entity.TaskLinkBag`
/// * 📌【2024-05-04 17:30:35】实际上就是「袋+任务链+特定参数」
///   * 📌目前不限制构造过程（即 不覆盖方法）
/// * 🚩有关「固定容量」与「遗忘时长」交给构造时决定
///   * ✅这也能避免冗余的对「记忆区」的引用
/// * 🚩【2024-05-07 20:57:36】锁定是「具体特征」
///   * 📌目前必须有构造函数
///   * ⚠️不然会有`ConceptBag: BagConcrete<Self::Concept> + ConceptBag`的「双重叠加」问题
///     * ❌这样会出现两套实现
pub trait TaskLinkBag: BagConcrete<Self::Link> {
    /// 绑定的「任务链」类型
    /// * 🎯一种实现只能对应一种「任务链袋」
    type Link: TaskLinkConcrete;
}

/// 初代实现
mod impl_v1 {
    use super::*;
    use crate::{
        entity::{BudgetV1, SentenceV1, StampV1, TaskLinkV1, TaskV1, TruthV1},
        storage::{BagKeyV1, BagV1},
    };

    /// 自动为「任务链+[`BagKeyV1`]+[`BagV1`]」实现「任务链袋」
    impl<T: TaskLinkConcrete<Key = BagKeyV1>> TaskLinkBag for BagV1<T> {
        type Link = T;
    }

    /// 初代[`TaskLinkBag`]实现
    /// * 🚩【2024-05-05 22:29:47】只需限定一系列类型，而无需再声明新`struct`
    pub type TaskLinkBagV1 =
        BagV1<TaskLinkV1<TaskV1<SentenceV1<TruthV1, StampV1>, BagKeyV1, BudgetV1>>>;
}
pub use impl_v1::*;

// * ✅单元测试参见`super::Bag`
