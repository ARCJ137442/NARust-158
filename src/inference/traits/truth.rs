//! 复刻OpenNARS的「真值」类型
//! * 📄OpenNARS改版 `Truth`接口
//! * 🎯只复刻外部读写方法，不限定内部数据字段
//!   * ❌不迁移「具体类型」特征

use crate::{entity::ShortFloat, global::Float, symbols::*, util::ToDisplayAndBrief};
use nar_dev_utils::join;
use narsese::lexical::Truth as LexicalTruth;

/// 模拟`nars.entity.TruthValue`
///
/// # 📄OpenNARS
///
/// Frequency and confidence.
pub trait Truth: ToDisplayAndBrief {
    /// 一种类型只可能有一种「证据值」
    /// * ✅兼容OpenNARS `ShortFloat`

    // ! 🚩【2024-05-04 17:12:30】现在有关「构造」「转换」的方法，均被迁移至[`TruthValueConcrete`]特征中

    /// 模拟`TruthValue.frequency`、`getFrequency`
    /// * 📌此处仍然直接返回（新的）「证据值」而非浮点
    fn frequency(&self) -> ShortFloat;
    fn frequency_mut(&mut self) -> &mut ShortFloat;

    /// 模拟`TruthValue.confidence`、`getConfidence`
    /// * 📌此处仍然直接返回（新的）「证据值」而非浮点
    fn confidence(&self) -> ShortFloat;
    fn confidence_mut(&mut self) -> &mut ShortFloat;

    /// 🆕用于快捷获取「频率-信度」对
    fn fc(&self) -> [ShortFloat; 2] {
        [self.frequency(), self.confidence()]
    }

    /// 🆕用于快捷获取双倍的「频率-信度」对
    fn fc_with(&self, other: &impl Truth) -> ([ShortFloat; 2], [ShortFloat; 2]) {
        (self.fc(), other.fc())
    }

    /// 模拟`TruthValue.isAnalytic`、`getAnalytic`
    /// * 📝OpenNARS将其用于「A + <A ==> B> = B」导出的真值中，然后在「下一次据此推导」中「排除结论」
    ///   * 💭【2024-05-03 15:34:29】或许正是为了「只导出一遍」或者「由此导出的结论不能直接使用」
    ///
    /// # 📄OpenNARS
    ///
    /// Get the isAnalytic flag
    ///
    /// @return The isAnalytic value
    fn is_analytic(&self) -> bool;

    /// 模拟`TruthValue.setAnalytic`
    /// * 🚩实质上只是「把默认的`false`设置为`true`」而已
    ///
    /// # 📄OpenNARS
    ///
    /// Set the isAnalytic flag
    fn set_analytic(&mut self);

    /// 模拟`getExpectation`
    /// * 🚩此处返回浮点数，因为中间结果可能是负数
    /// * 📝公式： $c * (f - 0.5) + 0.5$
    /// * ✨保证结果范围在 $[0, 1]$ 内
    /// * 🎯预算值、「答问」机制 等
    ///
    /// # 📄OpenNARS
    ///
    /// Calculate the expectation value of the truth value
    ///
    /// @return The expectation value
    fn expectation(&self) -> Float {
        /* 📄OpenNARS源码：
        return (float) (confidence.getValue() * (frequency.getValue() - 0.5) + 0.5); */
        let [f, c] = self.fc();
        let [f, c] = [f.to_float(), c.to_float()];
        c * (f - 0.5) + 0.5
    }

    /// 模拟`getExpDifAbs`
    /// * 🎯两个真值期望的绝对差
    /// * 🚩仍然返回浮点数
    ///
    /// # 📄OpenNARS
    ///
    /// Calculate the absolute difference of the expectation value and that of a given truth value
    ///
    /// @param t The given value
    /// @return The absolute difference
    #[doc(alias = "get_exp_dif_abs")]
    #[doc(alias = "expectation_absolute_difference")]
    fn expectation_abs_dif(&self, other: &impl Truth) -> Float {
        /* 📄OpenNARS源码：
        return Math.abs(getExpectation() - t.getExpectation()); */
        (self.expectation() - other.expectation()).abs()
    }

    /// 模拟`isNegative`
    ///
    /// # 📄OpenNARS
    ///
    /// Check if the truth value is negative
    ///
    /// @return True if the frequency is less than 1/2
    fn is_negative(&self) -> bool {
        /* 📄OpenNARS源码：
        return getFrequency() < 0.5; */
        self.frequency() < ShortFloat::HALF
    }

    /// 🆕模拟[`isNegative`](Self::is_negative)的反面：真值是否为【正面】
    /// * 🎯让代码更好理解
    #[inline]
    fn is_positive(&self) -> bool {
        !self.is_negative()
    }

    /// 模拟`TruthValue.equals`
    /// * 🎯将两个对象作为「真值」比较
    /// * 🎯用于「判断句」中「真值方面的比较」
    /// * ⚠️不一定等于
    fn truth_eq(&self, other: &impl Truth) -> bool {
        self.frequency() == other.frequency() && self.confidence() == other.frequency()
    }

    /// 模拟`toString`
    /// * 🚩【2024-05-08 22:12:42】现在鉴于实际情况，仍然实现`toString`、`toStringBrief`方法
    ///   * 🚩具体方案：实现一个统一的、内部的、默认的`__to_display(_brief)`，再通过「手动嫁接」完成最小成本实现
    /// * 🚩【2024-06-21 19:29:46】目前方案：明确是「作为不同类型的『字符串呈现』方法」，并在具体类型中手动指定映射
    ///
    /// # 📄OpenNARS
    ///
    /// The String representation of a TruthValue
    ///
    /// @return The String
    fn truth_to_display(&self) -> String {
        join!(
            => MARK.to_string()
            => self.frequency().to_display()
            => SEPARATOR
            => self.confidence().to_display()
            => MARK
        )
    }

    /// 模拟`toStringBrief`
    ///
    /// # 📄OpenNARS
    ///
    /// A simplified String representation of a TruthValue, where each factor is accurate to 1%
    ///
    /// @return The String
    fn truth_to_display_brief(&self) -> String {
        // ! 🆕🚩【2024-05-08 22:16:40】不对`1.00 => 0.99`做特殊映射
        MARK.to_string()
            + &self.frequency().to_display_brief()
            + SEPARATOR
            + &self.confidence().to_display_brief()
            + MARK
    }

    /// 🆕转换为「词法真值」
    /// * 🎯与词法Narsese的转换
    /// * 🚩【2024-06-21 21:08:43】目前方法：真值和信度的字符串
    fn truth_to_lexical(&self) -> LexicalTruth {
        vec![self.frequency().to_string(), self.confidence().to_string()]
    }
}

/// * 🚩【2024-05-09 00:56:52】改：统一为字符串
/// # 📄OpenNARS
///
/// The character that marks the two ends of a budget value
const MARK: &str = TRUTH_VALUE_MARK;

/// * 🚩【2024-05-09 00:56:52】改：统一为字符串
/// # 📄OpenNARS
///
/// The character that separates the factors in a budget value
const SEPARATOR: &str = VALUE_SEPARATOR;
