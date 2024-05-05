//! 🎯复刻OpenNARS `nars.entity.ConceptBag`
//! * 📌「概念袋」
//! * ✅【2024-05-04 17:50:50】基本功能复刻完成

use super::Bag;
use crate::entity::Concept;

/// 模拟OpenNARS `nars.entity.ConceptBag`
/// * 📌【2024-05-04 17:30:35】实际上就是「袋+概念+特定参数」
///   * 📌目前不限制构造过程（即 不覆盖方法）
/// * 🚩有关「固定容量」与「遗忘时长」交给构造时决定
///   * ✅这也能避免冗余的对「记忆区」的引用
pub trait ConceptBag: Bag<Self::Concept> {
    /// 绑定的「概念」类型
    /// * 🎯一种实现只能对应一种「概念袋」
    type Concept: Concept;
}

/// TODO: 初代实现（等待[`Concept`]）
mod impl_v1 {
    use super::*;
    use crate::storage::{BagKeyV1, BagV1};

    /// 自动为「概念+[`BagKeyV1`]+[`BagV1`]」实现「新近任务袋」
    impl<C: Concept<Key = BagKeyV1>> ConceptBag for BagV1<C> {
        type Concept = C;
    }

    // TODO: type别名 ConceptV1
}
pub use impl_v1::*;

// * ✅单元测试参见`super::Bag`
