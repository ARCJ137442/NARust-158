//! 🎯复刻OpenNARS `nars.entity.Item`
//! * ✅【2024-05-02 00:54:15】所有方法基本复刻完毕

use super::BudgetValue;
use crate::storage::bag::BagKey;

/// 袋中的「物品」类型
/// * 📝实际上其「键」和其「预算」都应只限于在「袋」内
///   * 📌即便实际上其自身有存储，也不过是在一种「特殊条件」下进行
/// * 🚩【2024-04-28 08:38:15】目前仍然先参照OpenNARS的方法来
///   * 在`Item`类中，有存在「不通过『袋』访问『预算』」的情况
/// * 🚩【2024-05-01 23:17:26】暂且按照OpenNARS的命名来：直接使用`Item`而非`BagItem`
///
/// # 📄OpenNARS `nars.entity.Item`
/// An item is an object that can be put into a Bag,
/// to participate in the resource competition of the system.
///
/// It has a key and a budget. Cannot be cloned
pub trait Item {
    /// 「唯一标识」类型
    /// * 🎯一个类型只有一种
    /// * 🚩【2024-05-01 22:36:42】在`Bag.putIn`中，需要复制键以置入「元素映射」
    type Key: BagKey;

    /// 「预算值」类型
    /// * 🎯一个类型只有一种
    type Budget: BudgetValue;

    /// 获取其唯一标识符
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
    fn budget_mut(&mut self) -> &mut Self::Budget;

    // 系列内联「预算值」的方法 //
    // * ❌【2024-05-01 23:35:47】无法通过「直接自动实现[`BudgetValue`]」迁移原「[预算值](BudgetValue)」的方法
    //   * ⚠️原因：类型系统中复杂的「生命周期承诺」问题
    //   * 🚩目前解决方案：手动逐一复刻

    /// 模拟`Item.get_priority`
    #[inline(always)]
    fn priority(&self) -> <Self::Budget as BudgetValue>::E {
        self.budget().priority()
    }

    /// 模拟`Item.set_priority`
    #[inline(always)]
    fn set_priority(&mut self, value: <Self::Budget as BudgetValue>::E) {
        self.budget_mut().set_priority(value)
    }

    /// 模拟`Item.inc_priority`
    #[inline(always)]
    fn inc_priority(&mut self, value: <Self::Budget as BudgetValue>::E) {
        self.budget_mut().inc_priority(value)
    }

    /// 模拟`Item.dec_priority`
    #[inline(always)]
    fn dec_priority(&mut self, value: <Self::Budget as BudgetValue>::E) {
        self.budget_mut().dec_priority(value)
    }

    /// 模拟`Item.get_durability`
    #[inline(always)]
    fn durability(&self) -> <Self::Budget as BudgetValue>::E {
        self.budget().durability()
    }

    /// 模拟`Item.set_durability`
    #[inline(always)]
    fn set_durability(&mut self, value: <Self::Budget as BudgetValue>::E) {
        self.budget_mut().set_durability(value)
    }

    /// 模拟`Item.inc_durability`
    #[inline(always)]
    fn inc_durability(&mut self, value: <Self::Budget as BudgetValue>::E) {
        self.budget_mut().inc_durability(value)
    }

    /// 模拟`Item.dec_durability`
    #[inline(always)]
    fn dec_durability(&mut self, value: <Self::Budget as BudgetValue>::E) {
        self.budget_mut().dec_durability(value)
    }

    /// 模拟`Item.get_quality`
    #[inline(always)]
    fn quality(&self) -> <Self::Budget as BudgetValue>::E {
        self.budget().quality()
    }

    /// 模拟`Item.set_quality`
    #[inline(always)]
    fn set_quality(&mut self, value: <Self::Budget as BudgetValue>::E) {
        self.budget_mut().set_quality(value)
    }

    /// 模拟`Item.inc_quality`
    #[inline(always)]
    fn inc_quality(&mut self, value: <Self::Budget as BudgetValue>::E) {
        self.budget_mut().inc_quality(value)
    }

    /// 模拟`Item.dec_quality`
    #[inline(always)]
    fn dec_quality(&mut self, value: <Self::Budget as BudgetValue>::E) {
        self.budget_mut().dec_quality(value)
    }

    /// 模拟`Item.merge`
    /// * 🚩【2024-05-01 23:21:01】实际上就是照搬「预算值」的方法
    ///
    /// # 📄OpenNARS `Item.merge`
    ///
    /// Merge with another Item with identical key
    #[inline(always)]
    fn merge(&mut self, other: &Self) {
        self.budget_mut().merge(other.budget())
    }

    // ! 🚩【2024-05-01 23:43:32】不模拟`Item.toString`、`Item.toStringBrief`
    // * ❌不实现「仅用于 显示/呈现」的方法，包括所有的`toString` `toStringBrief`
    // * 📄所有`toString(Brief)`都仅用于`NARSBatch`或「输出行」中
    // * 📌而这些实际上「一个全局函数+一个抽象特征`ToStringBrief`+集中定义各种实现」就可解决
    // * 💭所以这些本来都不需要内置在「系统内核」之中
    // /// 模拟`Item.toString`
    // /// * ❌无法直接「默认实现[`Display`]」：孤儿规则
    // ///
    // /// # 📄OpenNARS `Item.merge`
    // ///
    // /// Return a String representation of the Item
    // ///
    // /// @return The String representation of the full content
    // fn to_string(&self) -> String
    // where
    //     Self::Budget: Display,
    //     Self::Key: Display,
    // {
    //     format!("{} {}", self.budget(), self.key())
    // }
}
