//! 🎯复刻OpenNARS `nars.inference.UtilityFunctions`

use super::EvidenceReal;
use nar_dev_utils::pipe;

/// 【派生】用于「证据数值」的实用方法
///
/// # 📄OpenNARS `nars.inference.UtilityFunctions`
///
/// Common functions on real numbers, mostly in [0,1].
pub trait UtilityFunctions: EvidenceReal {
    /// 模拟`UtilityFunctions.not`
    /// * 🚩扩展逻辑「非」
    /// * 📝在使用了`Copy`并且是「按值传参」的情况下，才可省略[`clone`](Clone::clone)
    ///   * ⚠️要分清是在「拷贝值」还是在「拷贝引用」
    #[inline(always)]
    fn not(self) -> Self {
        Self::one() - self
    }

    /// 模拟`UtilityFunctions.and`
    /// * 🚩扩展逻辑「与」
    ///
    /// # 📄OpenNARS
    ///
    /// A function where the output is conjunctively determined by the inputs
    ///
    ///  @param arr The inputs, each in [0, 1]
    ///  @return The output that is no larger than each input
    #[inline(always)]
    fn and(self, value: Self) -> Self {
        self * value
    }

    /// 🆕多个值相与
    /// * 🚩直接派生自「两个值相与」
    ///   * 📝「扩展逻辑与」遵循交换律和结合律
    fn and_multi(values: impl IntoIterator<Item = Self>) -> Self {
        values
            // 逐个迭代值的迭代器
            .into_iter()
            // 从「1」开始不断取「与」
            .fold(Self::one(), Self::and)
    }

    /// 模拟`UtilityFunctions.or`
    /// * 🚩扩展逻辑「或」
    /// * 🚩【2024-05-02 17:53:22】利用德摩根律行事
    ///   * 💭可能会有性能损失
    ///
    /// # 📄OpenNARS
    ///
    /// A function where the output is disjunctively determined by the inputs
    ///
    /// @param arr The inputs, each in [0, 1]
    /// @return The output that is no smaller than each input
    fn or(self, value: Self) -> Self {
        // a ∨ b = ¬(¬a ∧ ¬b)
        // (self.not().and(value.not())).not()
        pipe! {
            // 非
            self.not()
            // 与
            => .and(value.not())
            // 非
            => .not()
        }
    }

    /// 🆕多个值相或
    /// * 🚩直接派生自「多个值相与」
    ///   * 📝「扩展逻辑或」遵循交换律和结合律
    ///   * ⚡优化：无需重复进行逻辑非
    fn or_multi(values: impl IntoIterator<Item = Self>) -> Self {
        pipe! {
            // 逐个迭代值的迭代器
            values.into_iter()
            // 逐个取逻辑非
            => .map(Self::not)
            // 所有值取逻辑与
            => Self::and_multi
            // 最后再取逻辑非
            => .not()
        }
    }

    /// 复刻OpenNARS `nars.inference.UtilityFunctions.aveGeo`
    /// * 🚩求几何平均值
    /// * 🎯🔬实验用：直接以「统一的逻辑」要求，而非将「真值函数」的语义赋予此特征
    /// * ❌不能用`impl IntoIterator<Item = Self>`：要计算长度
    fn geometrical_average(values: &[Self]) -> Self {
        // * 💭【2024-05-02 00:44:41】大概会长期存留，因为与「真值函数」无关而无需迁移
        /* 📄OpenNARS源码：
        float product = 1;
        for (float f : arr) {
            product *= f;
        }
        return (float) Math.pow(product, 1.00 / arr.length); */
        values
            // 逐个迭代值的迭代器
            .iter()
            .cloned()
            // 所有值的乘积（从`1`开始）
            .fold(Self::one(), Self::mul)
            // 值个数次开根
            .root(values.len())
    }

    /// 其它用途

    /// 🆕「增长」值
    /// * 🎯用于（统一）OpenNARS`incPriority`系列方法
    /// * 📝核心逻辑：自己的值和对面取「或」，越取越多
    /// * ❓【2024-05-02 00:31:19】是否真的要放到这儿来，在「数据结构定义」中引入「真值函数」的概念
    fn inc(&mut self, value: Self) {
        // self.set(UtilityFunctions.or(priority.getValue(), v));
        self.set(self.or(value))
    }

    /// 🆕「减少」值
    /// * 🎯用于（统一）OpenNARS`incPriority`系列方法
    /// * 📝核心逻辑：自己的值和对面取「与」，越取越少
    /// * ❓【2024-05-02 00:31:19】是否真的要放到这儿来，在「数据结构定义」中引入「真值函数」的概念
    fn dec(&mut self, value: Self) {
        // self.set(UtilityFunctions.and(priority.getValue(), v));
        self.set(self.and(value))
    }

    /// 🆕「合并」值
    /// * 🎯用于（统一）OpenNARS`merge`的重复调用
    /// * 🚩⚠️统一逻辑：`max(self, other)`
    /// * ❓是否可转换为`max`或使用`Ord`约束
    ///
    /// TODO: 🏗️【2024-05-02 18:24:53】不是这里的，需要移到其它地方（💭预算函数？）
    fn merge(&mut self, other: Self) {
        let max = (*self).max(other);
        self.set(max);
    }
}

/// 直接自动实现，附带所有默认方法
impl<T: EvidenceReal> UtilityFunctions for T {}

// ! ❌type parameter `T` must be used as the type parameter for some local type (e.g., `MyStruct<T>`)
// impl<T: EvidenceReal> std::ops::BitAnd for T {
//     type Output;

//     fn bitand(self, rhs: Self) -> Self::Output {
//         unimplemented!()
//     }
// }
