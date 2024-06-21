//! 复刻OpenNARS的「预算」类型
//! * 📄OpenNARS改版 `Budget`接口
//! * 🎯只复刻外部读写方法，不限定内部数据字段
//!   * ❌不迁移「具体类型」特征

use crate::{entity::ShortFloat, io::symbols::*, util::ToDisplayAndBrief};
use nar_dev_utils::join;

/// 模拟`nars.inference.Budget`
/// * 🎯实现最大程度的抽象与通用
///   * 💭后续可以在底层用各种「证据值」替换，而不影响整个推理器逻辑
/// * 🚩不直接使用「获取可变引用」的方式
///   * 📌获取到的「证据值」可能另有一套「赋值」的方法：此时需要特殊定制
///   * 🚩【2024-05-02 00:11:20】目前二者并行，`set_`复用`_mut`的逻辑（`_mut().set(..)`）
/// * 🚩【2024-05-03 14:46:52】要求[`Sized`]是为了使用构造函数
///
/// # 📄OpenNARS
///
/// A triple of priority (current), durability (decay), and quality (long-term average).
pub trait Budget: ToDisplayAndBrief {
    /// 模拟`BudgetValue.getPriority`
    /// * 🚩获取优先级
    /// * 🚩【2024-05-02 18:21:38】现在统一获取值：对「实现了[`Copy`]的类型」直接复制
    ///
    /// # 📄OpenNARS
    ///
    /// Get priority value
    ///
    /// @return The current priority
    fn priority(&self) -> ShortFloat;
    /// 获取优先级（可变）
    /// * 📌【2024-05-03 17:39:04】目前设置为内部方法
    fn __priority_mut(&mut self) -> &mut ShortFloat;

    /// 设置优先级
    /// * 🚩现在统一输入值，[`Copy`]保证无需过于担心性能损失
    #[inline(always)]
    fn set_priority(&mut self, new_p: ShortFloat) {
        self.__priority_mut().set(new_p)
    }

    /// 模拟`BudgetValue.getDurability`
    /// * 🚩获取耐久度
    /// * 🚩【2024-05-02 18:21:38】现在统一获取值：对「实现了[`Copy`]的类型」直接复制
    ///
    /// # 📄OpenNARS
    ///
    /// Get durability value
    ///
    /// @return The current durability
    fn durability(&self) -> ShortFloat;
    /// 获取耐久度（可变）
    /// * 📌【2024-05-03 17:39:04】目前设置为内部方法
    fn __durability_mut(&mut self) -> &mut ShortFloat;

    /// 设置耐久度
    /// * 🚩现在统一输入值，[`Copy`]保证无需过于担心性能损失
    #[inline(always)]
    fn set_durability(&mut self, new_d: ShortFloat) {
        self.__durability_mut().set(new_d)
    }

    /// 模拟`BudgetValue.getQuality`
    /// * 🚩获取质量
    /// * 🚩【2024-05-02 18:21:38】现在统一获取值：对「实现了[`Copy`]的类型」直接复制
    ///
    /// # 📄OpenNARS
    ///
    /// Get quality value
    ///
    /// @return The current quality
    fn quality(&self) -> ShortFloat;
    /// 获取质量（可变）
    /// * 📌【2024-05-03 17:39:04】目前设置为内部方法
    fn __quality_mut(&mut self) -> &mut ShortFloat;

    /// 设置质量
    /// * 🚩现在统一输入值，[`Copy`]保证无需过于担心性能损失
    #[inline(always)]
    fn set_quality(&mut self, new_q: ShortFloat) {
        self.__quality_mut().set(new_q)
    }

    /// 🆕从其它预算值处拷贝值
    /// * 🚩拷贝优先级、耐久度与质量
    fn copy_budget_from(&mut self, from: &impl Budget) {
        self.set_priority(from.priority());
        self.set_durability(from.durability());
        self.set_quality(from.quality());
    }

    // TODO: merge
    // fn merge_budget(&mut self, from: &impl Budget)

    /// 模拟`BudgetValue.summary`
    /// * 🚩📜统一采用「几何平均值」估计（默认）
    ///
    /// # 📄OpenNARS
    ///
    /// To summarize a BudgetValue into a single number in [0, 1]
    #[inline(always)]
    fn summary(&self) -> ShortFloat {
        // 🚩三者几何平均值
        ShortFloat::geometrical_average([self.priority(), self.durability(), self.quality()])
    }

    /// 模拟 `BudgetValue.aboveThreshold`
    /// * 🆕【2024-05-02 00:51:31】此处手动引入「阈值」，以避免使用「全局类の常量」
    ///   * 🚩将「是否要用『全局类の常量』」交给调用方
    /// * 📌常量`budget_threshold`对应OpenNARS`Parameters.BUDGET_THRESHOLD`
    ///
    /// # 📄OpenNARS
    ///
    /// Whether the budget should get any processing at all
    ///
    /// to be revised to depend on how busy the system is
    ///
    /// @return The decision on whether to process the Item
    #[inline(always)]
    fn above_threshold(&self, budget_threshold: ShortFloat) -> bool {
        self.summary() >= budget_threshold
    }

    // ! ❌【2024-05-08 21:53:30】不进行「自动实现」而是「提供所需的默认实现」
    //   * 📌情况：若直接使用「自动实现」则Rust无法分辨「既实现了『预算值』又实现了『真值』的类型所用的方法」
    //   * 📝解决方案：提供一套`__`内部默认实现，后续在「结构」实现时可利用这俩「默认实现方法」通过方便的「宏」自动实现[`ToDisplayAndBrief`]

    /// 模拟`toString`
    /// * 🚩【2024-05-08 22:12:42】现在鉴于实际情况，仍然实现`toString`、`toStringBrief`方法
    ///   * 🚩具体方案：实现一个统一的、内部的、默认的`__to_display(_brief)`，再通过「手动嫁接」完成最小成本实现
    /// * 🚩【2024-06-21 19:29:46】目前方案：明确是「作为不同类型的『字符串呈现』方法」，并在具体类型中手动指定映射
    ///   * 🎯一个是「明确具体的类型」一个是「避免使用混乱」
    ///   * ❓【2024-06-21 19:31:12】或许后续将不再需要[`ToDisplayAndBrief`]
    ///
    /// # 📄OpenNARS
    ///
    /// Fully display the BudgetValue
    ///
    /// @return String representation of the value
    fn budget_to_display(&self) -> String {
        join!(
            => MARK.to_string()
            => &self.priority().to_display()
            => SEPARATOR
            => &self.durability().to_display()
            => SEPARATOR
            => &self.quality().to_display()
            => MARK
        )
    }

    /// 模拟`toStringBrief`
    ///
    /// # 📄OpenNARS
    ///
    /// Briefly display the BudgetValue
    ///
    /// @return String representation of the value with 2-digit accuracy
    fn budget_to_display_brief(&self) -> String {
        MARK.to_string()
            + &self.priority().to_display_brief()
            + SEPARATOR
            + &self.durability().to_display_brief()
            + SEPARATOR
            + &self.quality().to_display_brief()
            + MARK
    }
}

/// * 🚩【2024-05-09 00:56:52】改：统一为字符串
/// # 📄OpenNARS
///
/// The character that marks the two ends of a budget value
const MARK: &str = BUDGET_VALUE_MARK;

/// * 🚩【2024-05-09 00:56:52】改：统一为字符串
/// # 📄OpenNARS
///
/// The character that separates the factors in a budget value
const SEPARATOR: &str = VALUE_SEPARATOR;
