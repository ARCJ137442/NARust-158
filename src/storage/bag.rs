use super::distributor::{Distribute, DistributorV1};
use crate::{
    entity::{BagItem, Budget, BudgetValue},
    global::Float,
    nars::DEFAULT_PARAMETERS,
};

/// 对应OpenNARS的「包」
/// * 📝【2024-04-26 23:12:15】核心逻辑：通过称作「预算」的机制，经济地分配内部元素
///   * 📌原理：AIKR
/// * 💭【2024-04-26 23:12:47】实际上「包」并不需要元素基于「预算」
///   * 📌「预算」本质上不属于「元素」而是「元素×包=预算」的概念
///   * 🚩换句话说，即：元素在包内才具有的预算，有「预算映射」`(&包, &元素id) -> Option<&预算>`
///   * 📌另外，「元素索引」作为元素在「包」中的唯一标识符，有「元素映射」`(&包, &元素id) -> Option<&元素>`
///     * 📌用于反查，还有「反查映射」`(&包, &元素) -> Option<&元素id>`
/// * 📌对于用「关联类型」还用「泛型参数」的问题
///   * 📝「泛型参数」可以用`'_`省掉生命周期，而「关联类型」不行
///   * 📍原则：长久存在、完全所有权的放在「关联类型」，反之放在「泛型参数」
///   * ✅避免生命周期参数的泛滥，避开[`PhantomData`](std::marker::PhantomData)
///   * ❌【2024-04-27 10:14:41】尽可能全部用关联类型：加了泛型会导致无法使用「泛型实现」
///     * 📄"the type parameter `Item` is not constrained by the impl trait, self type, or predicates"
///     * 🔗<https://stackoverflow.com/questions/69238420/the-type-parameter-t-is-not-constrained-by-the-impl-trait-self-type-or-predi>
///   * 🚩【2024-04-27 11:55:09】目前仍然全部使用关联类型
/// * 📌OpenNARS复刻原则 类⇒特征
///   * 🚩私有访问：对`private`/`protected`统一使用`_`作为前缀
///   * TODO: 有待扩充
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
pub trait Bag {
    // /// 元素id类型
    // /// * ❓要是引用类型还是值类型
    // ///   * 后续如何兼容`String`与`&str`
    type Key;

    /// 元素类型
    type Item: BagItem;

    /// 预算值类型
    type Budget: BudgetValue;

    /// 分发器类型
    /// * 🎯伪随机数生成
    type Distributor: Distribute;

    /// 【只读常量】总层数
    ///
    /// # 📄OpenNARS `Bag.TOTAL_LEVEL`
    ///
    /// priority levels
    #[inline(always)]
    fn _total_level(&self) -> usize {
        DEFAULT_PARAMETERS.bag_level
    }

    /// 【只读常量】触发阈值
    /// * 📌触发の阈值
    ///
    /// # 📄OpenNARS `Bag.THRESHOLD`
    ///
    /// firing threshold
    #[inline(always)]
    fn _threshold(&self) -> usize {
        DEFAULT_PARAMETERS.bag_threshold
    }

    /// 相对阈值
    /// * 🚩由`触发阈值 / 总层数`计算得来
    ///
    /// # 📄OpenNARS `Bag.RELATIVE_THRESHOLD`
    ///
    /// relative threshold, only calculate once
    #[inline(always)]
    fn _relative_threshold(&self) -> Float {
        self._threshold() as Float / self._total_level() as Float
    }

    /// 加载因子
    /// * ❓尚不清楚其含义
    ///
    /// # 📄OpenNARS `Bag.LOAD_FACTOR`
    ///
    /// hash table load factor
    #[inline(always)]
    fn _load_factor(&self) -> Float {
        DEFAULT_PARAMETERS.load_factor
    }

    /// 分发器（只读常量）
    ///
    /// # 📄OpenNARS `Bag.DISTRIBUTOR`
    ///
    /// shared DISTRIBUTOR that produce the probability distribution
    fn _distributor(&self) -> &Self::Distributor;

    // TODO: 继续研究OpenNARS，发现并复现更多功能（抽象的）
    // * 🚩逐个字段复刻

    /// 「元素映射」：从元素id获取元素
    fn get_item_from_key(&self, key: &Self::Key) -> Option<&Self::Item>;

    /// 「预算映射」：从元素id获取预算
    fn get_budget_from_key(&self, key: &Self::Key) -> Option<&Self::Budget>;
}

pub struct BagV1<Item: BagItem> {
    items: Vec<Item>,
}

impl<Item> Bag for BagV1<Item>
where
    Item: BagItem,
{
    type Distributor = DistributorV1;

    type Key = String;

    type Item = Item; // TODO: 占位符

    type Budget = Budget;

    fn _distributor(&self) -> &Self::Distributor {
        todo!()
    }

    fn get_item_from_key(&self, key: &String) -> Option<&Self::Item> {
        todo!()
    }

    fn get_budget_from_key(&self, key: &String) -> Option<&Self::Budget> {
        todo!()
    }
}
