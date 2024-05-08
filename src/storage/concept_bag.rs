//! 🎯复刻OpenNARS `nars.entity.ConceptBag`
//! * 📌「概念袋」
//! * ✅【2024-05-04 17:50:50】基本功能复刻完成

use super::BagConcrete;
use crate::{
    entity::{Concept, Item},
    language::Term,
};

/// 模拟`nars.entity.ConceptBag`
/// * 📌【2024-05-04 17:30:35】实际上就是「袋+概念+特定参数」
///   * 📌目前不限制构造过程（即 不覆盖方法）
/// * 🚩有关「固定容量」与「遗忘时长」交给构造时决定
///   * ✅这也能避免冗余的对「记忆区」的引用
/// * 🚩【2024-05-07 20:57:36】锁定是「具体特征」
///   * 📌目前必须有构造函数
///   * ⚠️不然会有`ConceptBag: BagConcrete<Self::Concept> + ConceptBag`的「双重叠加」问题
///     * ❌这样会出现两套实现
pub trait ConceptBag: BagConcrete<Self::Concept> {
    /// 绑定的「概念」类型
    /// * 🎯一种实现只能对应一种「概念袋」
    type Concept: Concept;

    /// 🆕从词项中获取词项的「元素id」
    /// * 🎯记忆区中「从词项提取词项」
    /// * 🚩一个统一的「词项→元素id→概念」
    fn key_from_term(term: &Term) -> <Self::Concept as Item>::Key;
}

/// TODO: 初代实现（等待[`Concept`]）
mod impl_v1 {
    use super::*;
    use crate::storage::{BagKeyV1, BagV1};

    /// 自动为「概念+[`BagKeyV1`]+[`BagV1`]」实现「概念袋」
    impl<C: Concept<Key = BagKeyV1>> ConceptBag for BagV1<C> {
        type Concept = C;

        #[inline(always)]
        fn key_from_term(term: &Term) -> <Self::Concept as Item>::Key {
            // * 🚩直接转换为字符串
            term.name()
        }
    }

    // TODO: type别名 ConceptV1
}
pub use impl_v1::*;

// * ✅单元测试参见`super::Bag`
