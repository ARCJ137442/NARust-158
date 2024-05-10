//! 🎯复刻OpenNARS `nars.entity.Item`
//! * ✅【2024-05-02 00:54:15】所有方法基本复刻完毕

use super::{BudgetValue, BudgetValueConcrete};
use crate::{inference::BudgetFunctions, storage::BagKey, ToDisplayAndBrief};

/// 模拟`nars.entity.Item`
/// * 📌袋中的「物品」类型
/// * 📝实际上其「键」和其「预算」都应只限于在「袋」内
///   * 📌即便实际上其自身有存储，也不过是在一种「特殊条件」下进行
/// * 🚩【2024-04-28 08:38:15】目前仍然先参照OpenNARS的方法来
///   * 在`Item`类中，有存在「不通过『袋』访问『预算』」的情况
/// * 🚩【2024-05-01 23:17:26】暂且按照OpenNARS的命名来：直接使用`Item`而非`BagItem`
/// * 📍示例：只实现「像预算那样『具有p、d、q属性』，但不仅仅有p、d、q属性」且不能直接从p、d、q构造
///   * ℹ️亦即：实现[`BudgetValue`]而未实现[`BudgetValueConcrete`]
///   * ✨并不「继承」预算值，但却可以当预算值一样使用（属性&方法）
///
/// An item is an object that can be put into a Bag,
/// to participate in the resource competition of the system.
///
/// It has a key and a budget. Cannot be cloned
pub trait Item: ToDisplayAndBrief {
    /// 「元素id」类型
    /// * 🎯一个类型只有一种
    /// * 🚩【2024-05-01 22:36:42】在`Bag.putIn`中，需要复制键以置入「元素映射」
    type Key: BagKey;

    /// 「预算值」类型
    /// * 🎯一个类型只有一种
    /// * 必须是「实体」类型
    type Budget: BudgetValueConcrete;

    /// 获取其元素id
    /// * 🎯应该只与自身数据绑定
    ///   * 📄概念的「词项名」
    fn key(&self) -> &Self::Key;
    // ! ⚠️【2024-05-01 22:49:15】临时：仅用于解决借用问题
    fn ____key_cloned(&self) -> Self::Key {
        self.key().clone()
    }

    /// 获取其预算值
    /// * 🎯便于「物品」之间访问
    ///   * 📄在「概念」中`linkToTask`需要访问其预算值
    fn budget(&self) -> &Self::Budget;
    /// 获取其预算值（[`Item::budget`]的可变版本）
    fn budget_mut(&mut self) -> &mut Self::Budget;

    fn quality(&self) -> <Self::Budget as BudgetValue>::E {
        self.budget().quality()
    }

    /// 模拟`Item.merge`
    /// * 🚩【2024-05-01 23:21:01】实际上就是照搬「预算值」的方法
    /// * 🚩【2024-05-02 21:06:22】现在直接使用了「预算函数」[`BudgetFunctions`]的特征方法
    ///
    /// # 📄OpenNARS
    ///
    /// Merge with another Item with identical key
    #[inline(always)]
    fn merge(&mut self, other: &Self) {
        self.budget_mut().merge(other.budget())
    }

    // * 🚩不实现「仅用于 显示/呈现」的方法，除了`toString` `toStringBrief`
    // * ✅现在可以借道[`ToDisplayAndBrief`]近乎无障碍实现二者

    /// 模拟`Item.toString`
    /// * ❌无法直接「默认实现[`Display`]」：孤儿规则
    /// * ✅通过[别的特征](ToDisplayAndBrief)去实现
    ///
    /// # 📄OpenNARS
    ///
    /// Return a String representation of the Item
    ///
    /// @return The String representation of the full content
    fn __to_display(&self) -> String {
        self.budget().__to_display() + " " + &self.key().to_display()
    }

    /// 模拟`Item.toStringBrief`
    ///
    /// # 📄OpenNARS
    ///
    /// Return a String representation of the Item after simplification
    ///
    /// @return A simplified String representation of the content
    #[inline(always)]
    fn __to_display_brief(&self) -> String {
        self.budget().__to_display_brief() + " " + &self.key().to_display_brief()
    }

    /// 模拟`Item.toStringLong`
    ///
    /// # 📄OpenNARS
    ///
    /// 🈚
    #[inline(always)]
    fn __to_display_long(&self) -> String {
        self.to_display()
    }
}

// ! ❌【2024-05-05 21:14:54】无法自动实现「元素id」：不是「具体类型」也没有「具体类型」

/// 自动实现「预算值」
/// * ℹ️具有属性，但不能从这些属性中构造
impl<T: Item> BudgetValue for T {
    type E = <<Self as Item>::Budget as BudgetValue>::E;

    /// 模拟`Item.get_priority`
    #[inline(always)]
    fn priority(&self) -> Self::E {
        self.budget().priority()
    }

    /// 🆕模拟`Item.get_priority`（可变版本）
    #[inline(always)]
    fn __priority_mut(&mut self) -> &mut Self::E {
        self.budget_mut().__priority_mut()
    }

    /// 模拟`Item.get_durability`
    #[inline(always)]
    fn durability(&self) -> Self::E {
        self.budget().durability()
    }

    /// 🆕模拟`Item.get_durability`（可变版本）
    #[inline(always)]
    fn __durability_mut(&mut self) -> &mut Self::E {
        self.budget_mut().__durability_mut()
    }

    /// 模拟`Item.get_quality`
    #[inline(always)]
    fn quality(&self) -> Self::E {
        self.budget().quality()
    }

    /// 🆕模拟`Item.get_quality`（可变版本）
    #[inline(always)]
    fn __quality_mut(&mut self) -> &mut Self::E {
        self.budget_mut().__quality_mut()
    }
}

// * ✅测试代码见[`crate::storage::Bag`]
