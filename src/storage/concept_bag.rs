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
pub trait ConceptBag<C: Concept>: Bag<C> {}
