//! 🎯复刻OpenNARS `nars.inference.UtilityFunctions`
//! * 🚩一些【与原OpenNARS不太相关，但的确通用】的函数，也放在这里
//!   * 📄如[`UtilityFunctions::max_from`]对[`super::BudgetFunctions::merge`]的抽象
//! * ✅【2024-05-02 21:17:31】基本实现所有OpenNARS原有功能
//!   * `and`
//!   * `or`
//!   * `aveAri`
//!   * `aveGeo`
//!   * `w2c`
//!   * `c2w`

use super::EvidenceReal;
use crate::global::Float;
use nar_dev_utils::pipe;
use std::ops::Div;

/// 【派生】用于「证据数值」的实用方法
///
/// # 📄OpenNARS `nars.inference.UtilityFunctions`
///
/// Common functions on real numbers, mostly in [0,1].
pub trait UtilityFunctions: EvidenceReal {
    /// 🆕扩展逻辑「非」
    /// * 📄这个在OpenNARS中直接用`1 - v`表示了，但此处仍然做出抽象
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

    /// 复刻OpenNARS `nars.inference.UtilityFunctions.aveAri`
    /// * 🚩求代数平均值
    /// * ❌不能用`impl IntoIterator<Item = Self>`：要计算长度
    ///
    /// # 📄OpenNARS
    ///
    /// A function where the output is the arithmetic average the inputs
    ///
    /// @param arr The inputs, each in [0, 1]
    /// @return The arithmetic average the inputs
    #[doc(alias = "ave_ari")]
    fn arithmetical_average(values: &[Self]) -> Self {
        // * 💭【2024-05-02 00:44:41】大概会长期存留，因为与「真值函数」无关而无需迁移
        /* 📄OpenNARS源码：
        float product = 1;
        for (float f : arr) {
            product *= f;
        }
        return (float) Math.pow(product, 1.00 / arr.length); */
        pipe! {
            values
            // 逐个迭代值的迭代器
            => .iter()
            // ! 必须先转换为浮点数：连续加和会越界
            => .map(Self::to_float)
            // 所有值的和（从`1`开始）
            => {.sum::<Float>()}#
            // 除以值的个数
            => .div(values.len() as Float)
            // 转换回「证据数值」（保证不越界）
            => Self::from_float
        }
    }

    /// 复刻OpenNARS `nars.inference.UtilityFunctions.aveGeo`
    /// * 🚩求几何平均值
    /// * ❌不能用`impl IntoIterator<Item = Self>`：要计算长度
    ///
    /// # 📄OpenNARS
    ///
    /// A function where the output is the geometric average the inputs
    ///
    /// @param arr The inputs, each in [0, 1]
    /// @return The geometric average the inputs
    #[doc(alias = "ave_geo")]
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

    /// 从真值的「w值」到「c值」
    ///
    /// # 📄OpenNARS
    ///
    /// A function to convert weight to confidence
    ///
    /// @param w Weight of evidence, a non-negative real number
    /// @return The corresponding confidence, in [0, 1)
    fn w2c(w: Float, horizon: usize) -> Self {
        /* 📄OpenNARS源码：
        return w / (w + Parameters.HORIZON); */
        Self::from_float(w / (w + horizon as Float))
    }

    /// 从真值的「c值」到「w值」
    /// * 📌此处的`c`就是`self`
    ///
    /// # 📄OpenNARS
    ///
    /// A function to convert confidence to weight
    ///
    /// @param c confidence, in [0, 1)
    /// @return The corresponding weight of evidence, a non-negative real number
    fn c2w(&self, horizon: usize) -> Float {
        /* 📄OpenNARS源码：
        return Parameters.HORIZON * c / (1 - c); */
        let c = self.to_float();
        horizon as Float * c / (1.0 - c)
    }

    // 其它用途 //
    // ! 🆕这些都不是原OpenNARS「实用函数」中的，而是为了代码统一加上的
    //   * 📄如：`merge`是为了在「预算函数」中减少重复而统一设计的

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

    /// 🆕「最大值合并」
    /// * 🎯用于（统一）OpenNARS`merge`的重复调用
    /// * 🚩现在已经在[「证据数值」](EvidenceReal)中要求了[`Ord`]
    fn max_from(&mut self, other: Self) {
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
