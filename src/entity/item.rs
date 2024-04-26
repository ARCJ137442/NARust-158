//! 抽象的「物品」特征

/// 包中的「物品」类型
/// * 📝实际上其「键」和其「预算」都应只限于在「包」内
///   * 📌即便实际上其自身有存储，也不过是在一种「特殊条件」下进行
///
/// # 📄OpenNARS `nars.entity.Item`
/// An item is an object that can be put into a Bag,
/// to participate in the resource competition of the system.
///
/// It has a key and a budget. Cannot be cloned
pub trait BagItem {
    // fn key(&self) -> f64;

    // fn budget(&self) -> Budget;
}
