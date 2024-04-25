use crate::entity::BagItem;

/// # 📄OpenNARS
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
pub trait Bagging<Key, Item>
where
    // Index: std::hash::Hash,
    Item: BagItem<Key>,
{
    // const TOTAL_LEVEL: usize;
}
