//! 抽象的「物品」特征

use super::BudgetValue;

/// 袋中的「物品」类型
/// * 📝实际上其「键」和其「预算」都应只限于在「袋」内
///   * 📌即便实际上其自身有存储，也不过是在一种「特殊条件」下进行
/// * 🚩【2024-04-28 08:38:15】目前仍然先参照OpenNARS的方法来
///   * 在`Item`类中，有存在「不通过『袋』访问『预算』」的情况
///
/// # 📄OpenNARS `nars.entity.Item`
/// An item is an object that can be put into a Bag,
/// to participate in the resource competition of the system.
///
/// It has a key and a budget. Cannot be cloned
pub trait BagItem {
    /// 「唯一标识」类型
    /// * 🎯一个类型只有一种
    type Key;

    /// 「预算值」类型
    /// * 🎯一个类型只有一种
    type Budget: BudgetValue;

    /// 获取其唯一标识符
    /// * 🎯应该只与自身数据绑定
    ///   * 📄概念的「词项名」
    fn key(&self) -> &Self::Key;

    /// 获取其预算值
    /// * 🎯便于「物品」之间访问
    ///   * 📄在「概念」中`linkToTask`需要访问其预算值
    fn budget(&self) -> &Self::Budget;
}
