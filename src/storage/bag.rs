use crate::entity::{BagItem, BudgetValue};

/// 对应OpenNARS的「包」
/// * 📝【2024-04-26 23:12:15】核心逻辑：通过称作「预算」的机制，经济地分配内部元素
///   * 📌原理：AIKR
/// * 💭【2024-04-26 23:12:47】实际上「包」并不需要元素基于「预算」
///   * 📌「预算」本质上不属于「元素」而是「元素×包=预算」的概念
///   * 🚩换句话说，即：元素在包内才具有的预算，有「预算映射」`(&包, &元素id) -> Option<&预算>`
///   * 📌另外，「元素索引」作为元素在「包」中的唯一标识符，有「元素映射」`(&包, &元素id) -> Option<&元素>`
///     * 📌用于反查，还有「反查映射」`(&包, &元素) -> Option<&元素id>`
///
/// # 📄OpenNARS `nars.storage.Bag`
/// A Bag is a storage with a constant capacity and maintains an internal
/// priority distribution for retrieval.
///
/// Each entity in a bag must extend Item, which has a BudgetValue and a key.
///
/// A name table is used to merge duplicate items that have the same key.
///
/// The bag space is divided by a threshold, above which is mainly time
/// management, and below, space management.
///
/// Differences:
///
/// 1. level selection vs. item selection
/// 2. decay rate
pub trait Bagging<Key, Item, Budget>
where
    Item: BagItem,
    Budget: BudgetValue,
{
    /// 「元素映射」：从元素id获取元素
    fn get_item_from_key(&self, key: Key) -> Option<&Item>;

    /// 「预算映射」：从元素id获取预算
    fn get_budget_from_key(&self, key: Key) -> Option<&Budget>;

    // TODO: 继续研究OpenNARS，发现并复现更多功能（抽象的）
}
