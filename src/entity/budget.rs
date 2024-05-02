//! 🎯复刻OpenNARS `nars.entity.Budget`
//! * ✅【2024-05-02 00:52:34】所有方法基本复刻完毕

use super::{ShortFloat, ShortFloatError};
use crate::global::Float;
use narsese::api::EvidentNumber;

/// 抽象的「预算数值」特征
/// * 🚩扩展自「证据值」，并（可）实验性地、敏捷开发地为之添加方法
/// * 💭【2024-05-02 00:46:02】亦有可能替代OpenNARS的`nars.inference.UtilityFunctions`
pub trait BudgetNumber:
    EvidentNumber + Clone + PartialOrd<Self> + TryFrom<Float, Error = Self::TryFromError>
{
    /// * 📌此处对[`Error`](std::fmt::Error)的需求仅仅在于[`Result::unwrap`]需要`Error: Debug`
    /// * 🎯【2024-05-02 12:17:19】引入以兼容[`TryFrom`]的[`try_from`](TryFrom::try_from)
    type TryFromError: std::error::Error;

    /// 转换为浮点数
    /// * 🚩使用「全局浮点数类型」
    /// * 🎯用于【预算数值与普通浮点数之间】【不同的预算数值之间】互相转换
    ///   * 📄`w2c`函数需要从值域 $[0, 1]$ 扩展到 $[0, +\infty)$
    fn to_float(&self) -> Float;

    /// 设置值
    /// * 类似「从其它地方拷贝值」的行为
    fn set(&mut self, new_value: &Self) {
        *self = new_value.clone();
    }

    /// 常数「0」
    fn zero() -> Self;

    /// 常数「1」
    fn one() -> Self;

    /// 扩展逻辑「非」
    /// TODO: 🏗️后续可能需要迁移到别的地方
    fn not(&self) -> Self {
        // 需`clone` | ⚠️即便`Copy`也不能省
        Self::one() - self.clone()
    }

    /// 扩展逻辑「与」
    /// TODO: 🏗️后续可能需要迁移到别的地方
    fn and(&self, value: &Self) -> Self {
        // 均需`clone`
        self.clone() * value.clone()
    }

    /// 扩展逻辑「或」
    /// TODO: 🏗️后续可能需要迁移到别的地方
    fn or(&self, value: &Self) -> Self {
        // 均需`clone`
        self.clone() + value.clone()
    }

    /// 🆕「增长」值
    /// * 🎯用于（统一）OpenNARS`incPriority`系列方法
    /// * 📝核心逻辑：自己的值和对面取「或」，越取越多
    /// * ❓【2024-05-02 00:31:19】是否真的要放到这儿来，在「数据结构定义」中引入「真值函数」的概念
    fn inc(&mut self, value: &Self) {
        // self.set(UtilityFunctions.or(priority.getValue(), v));
        self.set(&self.or(value))
    }

    /// 🆕「减少」值
    /// * 🎯用于（统一）OpenNARS`incPriority`系列方法
    /// * 📝核心逻辑：自己的值和对面取「与」，越取越少
    /// * ❓【2024-05-02 00:31:19】是否真的要放到这儿来，在「数据结构定义」中引入「真值函数」的概念
    fn dec(&mut self, value: &Self) {
        // self.set(UtilityFunctions.and(priority.getValue(), v));
        self.set(&self.and(value))
    }

    /// 🆕「合并」值
    /// * 🎯用于（统一）OpenNARS`merge`的重复调用
    /// * 🚩⚠️统一逻辑：`max(self, other)`
    /// * ❓是否可转换为`max`或使用`Ord`约束
    fn merge(&mut self, other: &Self) {
        if let Some(ord) = (*self).partial_cmp(other) {
            match ord {
                // 若 "self < other" ⇒ 自赋值
                std::cmp::Ordering::Less => *self = other.clone(),
                std::cmp::Ordering::Equal => {}
                std::cmp::Ordering::Greater => {}
            }
        }
    }

    /// 求几何平均值
    /// * 🎯🔬实验用：直接以「统一的逻辑」要求，而非将「真值函数」的语义赋予此特征
    fn geometrical_average(values: &[&Self]) -> Self {
        // * 💭【2024-05-02 00:44:41】大概会长期存留，因为与「真值函数」无关而无需迁移
        /* 📄OpenNARS源码：
        float product = 1;
        for (float f : arr) {
            product *= f;
        }
        return (float) Math.pow(product, 1.00 / arr.length); */
        let mut product: Float = 1.0;
        for f in values {
            // 变为浮点数再相乘
            product *= f.to_float();
        }
        //
        product
            .powf(1.0 / values.len() as Float)
            .try_into() // 尝试转换
            .unwrap() // ! ⚠️一般平均值不会越界（全`ShortFloat`的情况下）
    }
}

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
    /// * 🎯模拟OpenNARS `ShortFloat`
    ///
    /// TODO: 🚧【2024-05-01 23:52:11】一些地方尚缺，或需复刻`ShortFloat`
    type E: BudgetNumber;

    /// 获取优先级
    /// * 🚩仅获取不可变引用：避免复杂结构体被复制
    fn priority(&self) -> &Self::E;
    fn priority_mut(&mut self) -> &mut Self::E;

    /// 设置优先级
    /// * 🚩仅输入不可变引用：仅在必要时复制值
    fn set_priority(&mut self, new_p: &Self::E) {
        self.priority_mut().set(new_p)
    }

    /// 获取耐久度
    /// * 🚩仅获取不可变引用：避免复杂结构体被复制
    fn durability(&self) -> &Self::E;
    fn durability_mut(&mut self) -> &mut Self::E;

    /// 设置耐久度
    /// * 🚩仅输入不可变引用：仅在必要时复制值
    fn set_durability(&mut self, new_d: &Self::E) {
        self.durability_mut().set(new_d)
    }

    /// 获取质量
    /// * 🚩仅获取不可变引用：避免复杂结构体被复制
    fn quality(&self) -> &Self::E;
    fn quality_mut(&mut self) -> &mut Self::E;

    /// 设置质量
    /// * 🚩仅输入不可变引用：仅在必要时复制值
    fn set_quality(&mut self, new_q: &Self::E) {
        self.quality_mut().set(new_q)
    }

    /// 检查自身合法性
    /// * 📜分别检查`priority`、`durability`、`quality`的合法性
    fn check_valid(&self) -> bool {
        self.priority().is_valid() && self.durability().is_valid() && self.quality().is_valid()
    }

    /// 模拟`BudgetValue.incPriority`
    fn inc_priority(&mut self, value: &Self::E) {
        self.priority_mut().inc(value)
    }

    /// 模拟`BudgetValue.decPriority`
    fn dec_priority(&mut self, value: &Self::E) {
        self.priority_mut().dec(value)
    }

    /// 模拟`BudgetValue.incDurability`
    fn inc_durability(&mut self, value: &Self::E) {
        self.priority_mut().inc(value)
    }

    /// 模拟`BudgetValue.decDurability`
    fn dec_durability(&mut self, value: &Self::E) {
        self.durability_mut().dec(value)
    }

    /// 模拟`BudgetValue.incQuality`
    fn inc_quality(&mut self, value: &Self::E) {
        self.priority_mut().inc(value)
    }

    /// 模拟`BudgetValue.decQuality`
    fn dec_quality(&mut self, value: &Self::E) {
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
    fn above_threshold(&self, threshold: &Self::E) -> bool {
        self.summary() >= *threshold
    }

    // * ❌【2024-05-02 00:52:02】不实现「仅用于 显示/呈现」的方法，包括所有的`toString` `toStringBrief`
}

/// 一个默认实现
/// * 🔬仅作测试用
pub type Budget = [ShortFloat; 3];

/// 为「短浮点」实现「预算数值」
impl BudgetNumber for ShortFloat {
    type TryFromError = ShortFloatError;

    #[inline(always)]
    fn zero() -> Self {
        Self::ZERO
    }

    #[inline(always)]
    fn one() -> Self {
        Self::ONE
    }

    #[inline(always)]
    fn to_float(&self) -> Float {
        self.value()
    }

    fn set(&mut self, new_value: &Self) {
        // 直接将自身设置为「新值的浮点数」
        // * ✅不可能panic：对方亦为合法
        self.set_value(new_value.to_float()).unwrap()
    }

    fn merge(&mut self, other: &Self) {
        // * 🚩【2024-05-02 12:05:13】覆盖默认的`PartialEq`方法
        // * 🚩最大值不会越界，无需检查
        *self = Self::new_unchecked(self.value_short().max(other.value_short()))
    }
}

impl BudgetValue for Budget {
    // 指定为浮点数
    type E = ShortFloat;

    fn priority(&self) -> &ShortFloat {
        &self[0]
    }

    fn durability(&self) -> &ShortFloat {
        &self[1]
    }

    fn quality(&self) -> &ShortFloat {
        &self[2]
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
