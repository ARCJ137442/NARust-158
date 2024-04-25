//! 抽象的「物品」特征

use super::Budget;

/// # 📄OpenNARS
/// An item is an object that can be put into a Bag,
/// to participate in the resource competition of the system.
///
/// It has a key and a budget. Cannot be cloned
pub trait BagItem<Key> {
    fn key(&self) -> f64;

    fn budget(&self) -> Budget;
}
