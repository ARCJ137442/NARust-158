//! 🎯复刻OpenNARS `nars.entity.TruthValue`
//! * 📌【2024-05-02 21:30:40】从「预算函数」来：一些地方必须用到「真值」及其方法

use super::ShortFloat;
use crate::{
    global::Float,
    inference::EvidenceReal,
    io::{TRUTH_VALUE_MARK, VALUE_SEPARATOR},
};
use std::hash::Hash;

pub trait TruthValue: Sized + Clone /* ←构造函数需要，模拟OpenNARS `clone` */ + Eq /* 模拟OpenNARS `equals` */ + Hash /* 模拟OpenNARS `hashCode` */ {
    /// 一种类型只可能有一种「证据值」
    /// * ✅兼容OpenNARS `ShortFloat`
    type E: EvidenceReal;

    /// 🆕不使用「字符」而是用统一的「字符串」
    ///
    /// # 📄OpenNARS `TruthValue.DELIMITER`
    ///
    /// The character that marks the two ends of a truth value
    const DELIMITER: char = TRUTH_VALUE_MARK;

    /// 🆕不使用「字符」而是用统一的「字符串」
    ///
    /// # 📄OpenNARS `TruthValue.SEPARATOR`
    ///
    /// The character that separates the factors in a truth value
    const SEPARATOR: char = VALUE_SEPARATOR;

    /// 模拟OpenNARS `TruthValue.frequency`、`getFrequency`
    /// * 📌此处仍然直接返回（新的）「证据值」而非浮点
    fn frequency(&self) -> Self::E;
    fn frequency_mut(&mut self) -> &mut Self::E;

    /// 模拟OpenNARS `TruthValue.confidence`、`getConfidence`
    /// * 📌此处仍然直接返回（新的）「证据值」而非浮点
    fn confidence(&self) -> Self::E;
    fn confidence_mut(&mut self) -> &mut Self::E;

    /// 模拟OpenNARS `TruthValue.isAnalytic`、`getAnalytic`
    /// * 📌此处仍然直接返回（新的）「证据值」而非浮点
    ///
    /// # 📄OpenNARS
    ///
    /// Get the isAnalytic flag
    ///
    /// @return The isAnalytic value
    fn is_analytic(&self) -> bool;
    fn is_analytic_mut(&mut self) -> &mut bool;

    /// 模拟OpenNARS `TruthValue.setAnalytic`
    /// * 🚩实质上只是「把默认的`false`设置为`true`」而已
    ///
    /// # 📄OpenNARS
    ///
    /// Set the isAnalytic flag
    #[inline(always)]
    fn set_analytic(&mut self) {
        *self.is_analytic_mut() = true;
    }

    /// 模拟OpenNARS 构造函数 (f, c, a)
    /// * ⚠️此处让「f」「c」为浮点数，内部实现时再转换
    fn new(frequency: Float, confidence: Float, is_analytic: bool) -> Self;

    /// 模拟OpenNARS 构造函数 (f, c)
    /// * 🚩默认让参数`is_analytic`为`false`
    ///
    /// # 📄OpenNARS
    ///
    /// Constructor with two ShortFloats
    #[inline(always)]
    fn from_fc(frequency: Float, confidence: Float) -> Self {
        Self::new(frequency, confidence, false)
    }

    /// 模拟OpenNARS `getExpectation`
    /// * 🚩此处返回浮点数，因为可能是负数
    ///
    /// # 📄OpenNARS
    ///
    /// Calculate the expectation value of the truth value
    ///
    /// @return The expectation value
    fn expectation(&self) -> Float {
        /* 📄OpenNARS源码：
        return (float) (confidence.getValue() * (frequency.getValue() - 0.5) + 0.5); */
        self.confidence().value() * (self.frequency().value() - 0.5) + 0.5
    }

    /// 模拟OpenNARS `getExpDifAbs`
    /// * 🎯两个真值期望的绝对差
    /// * 🚩仍然返回浮点数
    ///
    /// # 📄OpenNARS
    ///
    /// Calculate the absolute difference of the expectation value and that of a given truth value
    ///
    /// @param t The given value
    /// @return The absolute difference
    fn get_exp_dif_abs(&self, other: &Self) -> Float {
        /* 📄OpenNARS源码：
        return Math.abs(getExpectation() - t.getExpectation()); */
        (self.expectation() - other.expectation()).abs()
    }

    /// 模拟OpenNARS `isNegative`
    ///
    /// # 📄OpenNARS
    ///
    /// Check if the truth value is negative
    ///
    /// @return True if the frequency is less than 1/2
    fn is_negative(&self) -> bool {
        /* 📄OpenNARS源码：
        return getFrequency() < 0.5; */
        self.frequency().value() < 0.5
    }

    // * ❌【2024-05-03 10:52:10】不实现「仅用于 显示/呈现」的方法，包括所有的`toString` `toStringBrief`
    // ! ⚠️孤儿规则：implementing a foreign trait is only possible if at least one of the types for which it is implemented is local
}

/// 初代实现
/// * 🎯测试特征的效果
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct TruthV1 {
    /// frequency
    f: ShortFloat,
    /// confidence
    c: ShortFloat,
    /// analytic
    a: bool,
}

impl TruthValue for TruthV1 {
    type E = ShortFloat;

    #[inline(always)]
    fn frequency(&self) -> Self::E {
        self.f
    }

    #[inline(always)]
    fn frequency_mut(&mut self) -> &mut Self::E {
        &mut self.f
    }

    #[inline(always)]
    fn confidence(&self) -> Self::E {
        self.c
    }

    #[inline(always)]
    fn confidence_mut(&mut self) -> &mut Self::E {
        &mut self.c
    }

    #[inline(always)]
    fn is_analytic(&self) -> bool {
        self.a
    }

    #[inline(always)]
    fn is_analytic_mut(&mut self) -> &mut bool {
        &mut self.a
    }

    #[inline(always)]
    fn new(frequency: Float, confidence: Float, is_analytic: bool) -> Self {
        Self {
            f: Self::E::from_float(frequency),
            c: Self::E::from_float(confidence),
            a: is_analytic,
        }
    }
}

/// TODO: 单元测试
#[cfg(test)]
mod tests {
    use super::*;
}
