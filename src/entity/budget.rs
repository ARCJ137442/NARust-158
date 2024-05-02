//! 🎯复刻OpenNARS `nars.entity.BudgetValue`
//! * ✅【2024-05-02 00:52:34】所有方法基本复刻完毕

use super::{ShortFloat, ShortFloatError};
use crate::inference::{EvidenceReal, UtilityFunctions};
use narsese::api::EvidentNumber;

/// 抽象的「预算」特征
/// * 🎯实现最大程度的抽象与通用
///   * 💭后续可以在底层用各种「证据值」替换，而不影响整个推理器逻辑
/// * 🚩不直接使用「获取可变引用」的方式
///   * 📌获取到的「证据值」可能另有一套「赋值」的方法：此时需要特殊定制
///   * 🚩【2024-05-02 00:11:20】目前二者并行，`set_`复用`_mut`的逻辑（`_mut().set(..)`）
///
/// # 📄OpenNARS `nars.entity.BudgetValue`
///
/// A triple of priority (current), durability (decay), and quality (long-term average).
pub trait BudgetValue {
    /// 一种类型只可能有一种「证据值」
    /// * ✅兼容OpenNARS `ShortFloat`
    type E: EvidenceReal;

    /// 获取优先级
    /// * 🚩【2024-05-02 18:21:38】现在统一获取值：对「实现了[`Copy`]的类型」直接复制
    fn priority(&self) -> Self::E;
    fn priority_mut(&mut self) -> &mut Self::E;

    /// 设置优先级
    /// * 🚩仅输入不可变引用：仅在必要时复制值
    fn set_priority(&mut self, new_p: Self::E) {
        self.priority_mut().set(new_p)
    }

    /// 获取耐久度
    /// * 🚩【2024-05-02 18:21:38】现在统一获取值：对「实现了[`Copy`]的类型」直接复制
    fn durability(&self) -> Self::E;
    fn durability_mut(&mut self) -> &mut Self::E;

    /// 设置耐久度
    /// * 🚩仅输入不可变引用：仅在必要时复制值
    fn set_durability(&mut self, new_d: Self::E) {
        self.durability_mut().set(new_d)
    }

    /// 获取质量
    /// * 🚩【2024-05-02 18:21:38】现在统一获取值：对「实现了[`Copy`]的类型」直接复制
    fn quality(&self) -> Self::E;
    fn quality_mut(&mut self) -> &mut Self::E;

    /// 设置质量
    /// * 🚩仅输入不可变引用：仅在必要时复制值
    fn set_quality(&mut self, new_q: Self::E) {
        self.quality_mut().set(new_q)
    }

    /// 检查自身合法性
    /// * 📜分别检查`priority`、`durability`、`quality`的合法性
    fn check_valid(&self) -> bool {
        self.priority().is_valid() && self.durability().is_valid() && self.quality().is_valid()
    }

    /// 模拟`BudgetValue.incPriority`
    fn inc_priority(&mut self, value: Self::E) {
        self.priority_mut().inc(value)
    }

    /// 模拟`BudgetValue.decPriority`
    fn dec_priority(&mut self, value: Self::E) {
        self.priority_mut().dec(value)
    }

    /// 模拟`BudgetValue.incDurability`
    fn inc_durability(&mut self, value: Self::E) {
        self.priority_mut().inc(value)
    }

    /// 模拟`BudgetValue.decDurability`
    fn dec_durability(&mut self, value: Self::E) {
        self.durability_mut().dec(value)
    }

    /// 模拟`BudgetValue.incQuality`
    fn inc_quality(&mut self, value: Self::E) {
        self.priority_mut().inc(value)
    }

    /// 模拟`BudgetValue.decQuality`
    fn dec_quality(&mut self, value: Self::E) {
        self.quality_mut().dec(value)
    }

    /// 模拟`BudgetValue.merge`
    ///
    /// # 📄OpenNARS
    ///
    /// Merge one BudgetValue into another
    fn merge(&mut self, other: &Self);

    /// 模拟`BudgetValue.summary`
    /// * 🚩📜统一采用「几何平均值」估计（默认）
    ///
    /// # 📄OpenNARS
    ///
    /// To summarize a BudgetValue into a single number in [0, 1]
    fn summary(&self) -> Self::E {
        // 🚩三者几何平均值
        Self::E::geometrical_average(&[self.priority(), self.durability(), self.quality()])
    }

    /// 模拟 `BudgetValue.aboveThreshold`
    /// * 🆕【2024-05-02 00:51:31】此处手动引入「阈值」，以避免使用「全局类の常量」
    ///   * 🚩将「是否要用『全局类の常量』」交给调用方
    ///
    /// # 📄OpenNARS
    ///
    /// Whether the budget should get any processing at all
    ///
    /// to be revised to depend on how busy the system is
    ///
    /// @return The decision on whether to process the Item
    fn above_threshold(&self, threshold: Self::E) -> bool {
        self.summary() >= threshold
    }

    // * ❌【2024-05-02 00:52:02】不实现「仅用于 显示/呈现」的方法，包括所有的`toString` `toStringBrief`
}

/// 一个默认实现
/// * 🔬仅作测试用
pub type Budget = [ShortFloat; 3];

/// 为「短浮点」实现「预算数值」
impl EvidenceReal for ShortFloat {
    type TryFromError = ShortFloatError;

    // ! ❌【2024-05-02 18:22:22】不再需要：以`root`代替
    // #[inline(always)]
    // fn to_float(&self) -> Float {
    //     self.value()
    // }

    // ! ❌【2024-05-02 18:22:22】不再需要：默认实现就好
    // fn set(&mut self, new_value: Self) {
    //     // 直接将自身设置为「新值的浮点数」
    //     // * ✅不可能panic：对方亦为合法
    //     self.set_value(new_value).unwrap()
    // }

    // ! ❌【2024-05-02 16:11:21】现在不允许覆盖
    // * 📝Rust「自动实现的特征」不能再通过`impl`覆盖：避免「实现冲突」
    // fn merge(&mut self, other: &Self) {
    //     // * 🚩【2024-05-02 12:05:13】覆盖默认的`PartialEq`方法
    //     // * 🚩最大值不会越界，无需检查
    //     *self = Self::new_unchecked(self.value_short().max(other.value_short()))
    // }
}

impl BudgetValue for Budget {
    // 指定为浮点数
    type E = ShortFloat;

    fn priority(&self) -> ShortFloat {
        self[0] // * 🚩【2024-05-02 18:24:10】现在隐式`clone`
    }

    fn durability(&self) -> ShortFloat {
        self[1] // * 🚩【2024-05-02 18:24:10】现在隐式`clone`
    }

    fn quality(&self) -> ShortFloat {
        self[2] // * 🚩【2024-05-02 18:24:10】现在隐式`clone`
    }

    fn priority_mut(&mut self) -> &mut ShortFloat {
        &mut self[0]
    }

    fn durability_mut(&mut self) -> &mut ShortFloat {
        &mut self[1]
    }

    fn quality_mut(&mut self) -> &mut ShortFloat {
        &mut self[2]
    }

    fn merge(&mut self, other: &Self) {
        // * 🚩【2024-05-02 00:16:50】仅作参考，后续要移动到「预算函数」中
        /* OpenNARS源码 @ BudgetFunctions.java：
        baseValue.setPriority(Math.max(baseValue.getPriority(), adjustValue.getPriority()));
        baseValue.setDurability(Math.max(baseValue.getDurability(), adjustValue.getDurability()));
        baseValue.setQuality(Math.max(baseValue.getQuality(), adjustValue.getQuality())); */
        // 🆕此处直接分派到各个值中
        self.priority_mut().merge(other.priority());
        self.durability_mut().merge(other.durability());
        self.quality_mut().merge(other.quality());
    }
}
