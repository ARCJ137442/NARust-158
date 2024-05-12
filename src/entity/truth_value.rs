//! 🎯复刻OpenNARS `nars.entity.TruthValue`
//! * 📌【2024-05-02 21:30:40】从「预算函数」来：一些地方必须用到「真值」及其方法
//! * ✅【2024-05-03 16:21:02】所有方法基本复刻完毕

use super::{ShortFloat, ShortFloatV1};
use crate::{
    global::Float,
    io::{TRUTH_VALUE_MARK, VALUE_SEPARATOR},
    ToDisplayAndBrief,
};
use anyhow::Result;
use nar_dev_utils::join;
use narsese::lexical::Truth as LexicalTruth;
use std::{fmt::Debug, hash::Hash};

/// 模拟`nars.entity.TruthValue`
///
/// # 📄OpenNARS
///
/// Frequency and confidence.
pub trait TruthValue: ToDisplayAndBrief {
    /// 一种类型只可能有一种「证据值」
    /// * ✅兼容OpenNARS `ShortFloat`
    type E: ShortFloat;

    // ! 🚩【2024-05-04 17:12:30】现在有关「构造」「转换」的方法，均被迁移至[`TruthValueConcrete`]特征中

    /// 模拟`TruthValue.frequency`、`getFrequency`
    /// * 📌此处仍然直接返回（新的）「证据值」而非浮点
    fn frequency(&self) -> Self::E;
    fn frequency_mut(&mut self) -> &mut Self::E;

    /// 模拟`TruthValue.confidence`、`getConfidence`
    /// * 📌此处仍然直接返回（新的）「证据值」而非浮点
    fn confidence(&self) -> Self::E;
    fn confidence_mut(&mut self) -> &mut Self::E;

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
    /// [`TruthValue::is_analytic`]的内部可变版本
    /// * 🎯用于[`TruthValue::set_analytic`]
    fn __is_analytic_mut(&mut self) -> &mut bool;

    /// 模拟`TruthValue.setAnalytic`
    /// * 🚩实质上只是「把默认的`false`设置为`true`」而已
    ///
    /// # 📄OpenNARS
    ///
    /// Set the isAnalytic flag
    #[inline(always)]
    fn set_analytic(&mut self) {
        *self.__is_analytic_mut() = true;
    }

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
        self.confidence().value() * (self.frequency().value() - 0.5) + 0.5
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
    fn expectation_abs_dif(&self, other: &impl TruthValue<E = Self::E>) -> Float {
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
        self.frequency() < Self::E::HALF
    }

    /// 模拟`toString`
    /// * 🚩【2024-05-08 22:12:42】现在鉴于实际情况，仍然实现`toString`、`toStringBrief`方法
    ///   * 🚩具体方案：实现一个统一的、内部的、默认的`__to_display(_brief)`，再通过「手动嫁接」完成最小成本实现
    ///
    /// # 📄OpenNARS
    ///
    /// The String representation of a TruthValue
    ///
    /// @return The String
    fn __to_display(&self) -> String {
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
    fn __to_display_brief(&self) -> String {
        // ! 🆕🚩【2024-05-08 22:16:40】不对`1.00 => 0.99`做特殊映射
        MARK.to_string()
            + &self.frequency().to_display_brief()
            + SEPARATOR
            + &self.confidence().to_display_brief()
            + MARK
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

/// 真值的「具体类型」
/// * 📌前置特征：
///   * [`Sized`]：模拟构造函数
///   * [`Clone`]：模拟`clone`
///   * [`Eq`]：模拟`equals`
///   * [`Hash`]：模拟`hashCode`
/// * 🎯有选择地支持「限定的构造函数」
///   * 📄需要构造函数：真值函数中「创建新值的函数」
///   * 📄不要构造函数：具有「真值属性」但【不可从真值参数构造】的类型
///     * 📄语句[`super::Concept`]
///     * 📄任务[`super::Task`]
/// * 📌整个特征建立在「真值就是真值」，即「实现者本身**只有**f、c、a三元组」的基础上
/// * 🚩包括「构造函数」与「转换函数」
/// * 💭【2024-05-04 17:14:08】这是否有些像Julia中「抽象类型🆚具体类型」的关系
pub trait TruthValueConcrete: TruthValue + Sized + Clone + Eq + Hash {
    /// 🆕最原始的构造函数(f, c, a)
    /// * 🎯用于[`TruthValue::new_analytic_default`]
    /// * 用于接收「内部转换后的结果」
    fn new(frequency: Self::E, confidence: Self::E, is_analytic: bool) -> Self;

    /// 🆕最原始的构造函数(f, c)
    /// * 🎯用于[`TruthValue::new_analytic_default`]
    /// * 用于接收「内部转换后的结果」
    #[inline(always)]
    fn new_fc(frequency: Self::E, confidence: Self::E) -> Self {
        Self::new(frequency, confidence, false)
    }

    /// 模拟构造函数 (f, c, a)
    /// * ⚠️此处让「f」「c」为浮点数，内部实现时再转换
    #[inline(always)]
    fn from_floats(frequency: Float, confidence: Float, is_analytic: bool) -> Self {
        Self::new(
            Self::E::from_float(frequency),
            Self::E::from_float(confidence),
            is_analytic,
        )
    }

    /// 模拟构造函数 (f, c)
    /// * 🚩默认让参数`is_analytic`为`false`
    ///
    /// # 📄OpenNARS
    ///
    /// Constructor with two ShortFloats
    #[inline(always)]
    fn from_fc(frequency: Float, confidence: Float) -> Self {
        Self::new_fc(
            Self::E::from_float(frequency),
            Self::E::from_float(confidence),
        )
    }

    /// 🆕集成OpenNARS`isAnalytic`产生的推理结果
    /// * 🎯消除硬编码 自`return new TruthValue(0.5f, 0f);`
    ///   * f、c、a分别为`0.5f`、`0f`、`false`
    /// * ❓【2024-05-03 13:51:37】到底`isAnalytic`意义何在
    #[inline(always)]
    fn new_analytic_default() -> Self {
        /* 📄OpenNARS源码 @ TruthFunctions：
        new TruthValue(0.5f, 0f); */
        Self::new(Self::E::HALF, Self::E::ZERO, false)
    }

    /// 🆕「词法真值」到「自身类型」的转换
    /// * 🎯统一的、全面的「词法真值→真值」转换方法
    /// * 📌需要手动输入「默认值」
    /// * 📌需要手动输入「是否为『分析真值』」
    fn from_lexical(
        lexical: LexicalTruth,
        mut default_values: [Self::E; 2],
        is_analytic: bool,
    ) -> Result<Self> {
        let truth_s = match lexical.len() {
            0 => &[],
            1 => &lexical[0..1],
            _ => &lexical[0..2],
        };
        // 预先解析默认值
        // ! ⚠️必须合法，否则panic
        let float_s = &mut default_values;
        for (i, s) in truth_s.iter().enumerate() {
            // 浮点解析
            let v = s.parse::<Float>()?;
            // 短浮点解析
            let sf = match Self::E::try_from(v) {
                Ok(sf) => sf,
                Err(_) => return Err(anyhow::anyhow!("无效短浮点值：{v}")),
            };
            float_s[i] = sf;
        }
        // 构造
        let [f, c] = *float_s;
        Ok(Self::new(f, c, is_analytic))
    }

    /// 🆕自身到「词法」的转换
    /// * 🎯标准Narsese输出需要（Narsese内容）
    /// * 🚩【2024-05-12 14:48:31】此处跟随OpenNARS，仅用两位小数
    fn to_lexical(&self) -> LexicalTruth {
        vec![
            self.frequency().to_display_brief(),
            self.confidence().to_display_brief(),
        ]
    }
}

/// 初代实现
mod impl_v1 {
    use super::*;
    use crate::__impl_to_display_and_display;
    use std::hash::Hasher;

    /// [`TruthValue`]初代实现
    /// * 🎯测试特征的效果
    /// * 📌[`PartialEq`]、[`Eq`]、[`Hash`]均特别实现
    #[derive(Debug, Clone, Copy, Default)]
    pub struct TruthV1 {
        /// frequency
        f: ShortFloatV1,
        /// confidence
        c: ShortFloatV1,
        /// analytic
        a: bool,
    }

    /// 模拟`equals`
    /// * ⚠️其中[`Self::a`]即`isAnalytic`不参与判等
    impl PartialEq for TruthV1 {
        #[inline(always)]
        fn eq(&self, other: &Self) -> bool {
            self.f == other.f && self.c == other.c
        }
    }
    impl Eq for TruthV1 {}

    /// 手动实现[`Hash`]
    /// * ⚠️因为[`Self::a`]不参与判等，因此也不能参与到「散列化」中
    impl Hash for TruthV1 {
        fn hash<H: Hasher>(&self, state: &mut H) {
            self.f.hash(state);
            self.c.hash(state);
            // self.a.hash(state);
        }
    }

    impl TruthValue for TruthV1 {
        type E = ShortFloatV1;

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
        fn __is_analytic_mut(&mut self) -> &mut bool {
            &mut self.a
        }
    }

    impl TruthValueConcrete for TruthV1 {
        #[inline(always)]
        fn new(frequency: Self::E, confidence: Self::E, is_analytic: bool) -> Self {
            Self {
                f: frequency,
                c: confidence,
                a: is_analytic,
            }
        }
    }

    __impl_to_display_and_display! {
        TruthV1 as TruthValue
    }
}
pub use impl_v1::*;

/// 转换：涉及「词法Narsese」的解析
/// * 🚩【2024-05-10 09:40:03】不实现「从字符串解析」
///   * 无法仅通过「频率」「信度」确定一个「真值」
///   * [`narsese`]包尚未有简单、直接地解析出「词法真值」的函数
mod conversion {
    // ! ❌【2024-05-10 09:35:35】难以仅通过`TryFrom`实现：需要更多参数
    // ! ❌【2024-05-10 09:35:35】无法批量实现：孤儿规则

    /// 快捷构造宏
    #[macro_export]
    macro_rules! truth {
        // 二参数
        ($f:expr; $c:expr) => {
            TruthV1::from_fc($f, $c)
        };
        // 三参数
        ($f:expr; $c:expr; $a:expr) => {
            TruthV1::from_floats($f, $c, $a)
        };
    }
}

/// 单元测试
#[cfg(test)]
mod tests {
    use super::*;
    use crate::{global::tests::AResult, ok, truth};
    use nar_dev_utils::macro_once;

    /// 定义要测试的「真值」类型
    type Truth = TruthV1;
    type SF = <Truth as TruthValue>::E;

    // * ✅测试/new已在「快捷构造宏」中实现

    // * ✅测试/from_fc已在「快捷构造宏」中实现

    // * ✅测试/from_float已在「快捷构造宏」中实现

    /// 测试/frequency
    #[test]
    fn frequency() -> AResult {
        macro_once! {
            /// * 🚩模式：[真值的构造方法] ⇒ 预期「短浮点」浮点值
            macro test($( [ $($truth:tt)* ] => $expected:tt)*) {
                $(
                    assert_eq!(
                        truth!($($truth)*).frequency(),
                        SF::from_float($expected)
                    );
                )*
            }
            [1.0; 0.9] => 1.0
            [0.1; 0.9] => 0.1
            [0.0001; 0.9] => 0.0001
            [0.1024; 0.0] => 0.1024
            [0.2; 0.1] => 0.2
        }
        ok!()
    }

    /// 测试/frequency_mut
    #[test]
    fn frequency_mut() -> AResult {
        macro_once! {
            /// * 🚩模式：[真值的构造方法] → 要被赋的值 ⇒ 预期「短浮点」浮点值
            macro test($( [ $($truth:tt)* ] -> $new_float:tt => $expected:tt)*) {
                $(
                    let mut t = truth!($($truth)*);
                    *t.frequency_mut() = SF::from_float($new_float);
                    // 可变与不可变一致
                    assert_eq!(t.frequency(), *t.frequency_mut());
                    // 修改后与所读值一致
                    assert_eq!(*t.frequency_mut(), SF::from_float($expected));
                )*
            }
            [1.0; 0.9] -> 0.5 => 0.5
            [0.1; 0.9] -> 0.2 => 0.2
            [0.0001; 0.9] -> 0.8 => 0.8
            [0.1024; 0.0] -> 0.0 => 0.0
            [0.2; 0.1] -> 1.0 => 1.0
        }
        ok!()
    }

    /// 测试/confidence
    #[test]
    fn confidence() -> AResult {
        macro_once! {
            /// * 🚩模式：[真值的构造方法] ⇒ 预期「短浮点」浮点值
            macro test($( [ $($truth:tt)* ] => $expected:tt)*) {
                $(
                    assert_eq!(
                        truth!($($truth)*).confidence(),
                        SF::from_float($expected)
                    );
                )*
            }
            [1.0; 0.9] => 0.9
            [0.1; 0.9] => 0.9
            [0.0001; 0.9] => 0.9
            [0.1024; 0.0] => 0.0
            [0.2; 0.1] => 0.1
        }
        ok!()
    }

    /// 测试/confidence_mut
    #[test]
    fn confidence_mut() -> AResult {
        macro_once! {
            /// * 🚩模式：[真值的构造方法] → 要被赋的值 ⇒ 预期「短浮点」浮点值
            macro test($( [ $($truth:tt)* ] -> $new_float:tt => $expected:tt)*) {
                $(
                    let mut t = truth!($($truth)*);
                    *t.confidence_mut() = SF::from_float($new_float);
                    // 可变与不可变一致
                    assert_eq!(t.confidence(), *t.confidence_mut());
                    // 修改后与所读值一致
                    assert_eq!(*t.confidence_mut(), SF::from_float($expected));
                )*
            }
            [1.0; 0.9] -> 0.5 => 0.5
            [0.1; 0.9] -> 0.2 => 0.2
            [0.0001; 0.9] -> 0.8 => 0.8
            [0.1024; 0.0] -> 0.0 => 0.0
            [0.2; 0.1] -> 1.0 => 1.0
        }
        ok!()
    }

    /// 测试/is_analytic
    #[test]
    fn is_analytic() -> AResult {
        macro_once! {
            /// * 🚩模式：[真值的构造方法] ⇒ 预期
            macro test($( [ $($truth:tt)* ] => $expected:tt)*) {
                $(
                    assert_eq!(
                        truth!($($truth)*).is_analytic(),
                        $expected
                    );
                )*
            }
            // 默认值`false`
            [1.0; 0.9] => false
            // 指定值
            [1.0; 0.9; false] => false
            [1.0; 0.9; true] => true
        }
        ok!()
    }

    // * ✅测试/__is_analytic_mut 已在`set_analytic`中测试过

    /// 测试/set_analytic
    #[test]
    fn set_analytic() -> AResult {
        macro_once! {
            /// * 🚩模式：[真值的构造方法]
            macro test($( [ $($truth:tt)* ])*) {
                $(
                    let mut truth = truth!($($truth)*);
                    truth.set_analytic();
                    assert!(truth.is_analytic());
                )*
            }
            // 不管最开始是什么，均会变成`true`
            [1.0; 0.9]
            [1.0; 0.9; false]
            [1.0; 0.9; true]
        }
        ok!()
    }

    /// 测试/expectation
    #[test]
    fn expectation() -> AResult {
        macro_once! {
            /// * 🚩模式：[真值的构造方法] ⇒ 预期
            macro test($( [ $($truth:tt)* ] => $expected:tt)*) {
                $(
                    assert_eq!(
                        truth!($($truth)*).expectation(),
                        $expected
                    );
                )*
            }
            // * 特殊值矩阵
            [0.0; 0.0] => 0.5   [0.0; 0.5] => 0.25   [0.0; 1.0] => 0.0
            [0.5; 0.0] => 0.5   [0.5; 0.5] => 0.5    [0.5; 1.0] => 0.5
            [1.0; 0.0] => 0.5   [1.0; 0.5] => 0.75   [1.0; 1.0] => 1.0
            // * 📝公式：$c * (f - 0.5) + 0.5$
            [1.0; 0.9] => ((0.9 * (1.0 - 0.5)) + 0.5)
        }
        ok!()
    }

    /// 测试/expectation_abs_dif
    #[test]
    fn expectation_abs_dif() -> AResult {
        macro_once! {
            /// * 🚩模式：| [真值的构造方法] - [真值的构造方法] | ⇒ 预期
            macro test($( | [ $($truth1:tt)* ] - [ $($truth2:tt)* ] | => $expected:tt)*) {
                $(
                    let truth1 = truth!($($truth1)*);
                    let truth2 = truth!($($truth2)*);
                    assert_eq!(
                        truth1.expectation_abs_dif(&truth2),
                        $expected
                    );
                )*
            }
            // * 特殊值矩阵（上述矩阵的对边相差）
            |[0.0; 0.0]-[1.0; 1.0]| => 0.5   |[0.0; 0.5]-[1.0; 0.5]| => 0.5   |[0.0; 1.0]-[1.0; 0.0]| => 0.5
            |[0.5; 0.0]-[0.5; 1.0]| => 0.0   |[0.5; 0.5]-[0.5; 0.5]| => 0.0   |[0.5; 1.0]-[0.5; 0.0]| => 0.0
            |[1.0; 0.0]-[0.0; 1.0]| => 0.5   |[1.0; 0.5]-[0.0; 0.5]| => 0.5   |[1.0; 1.0]-[0.0; 0.0]| => 0.5
            // * 📝公式：
            // *   | (c1 * (f1 - 0.5) + 0.5) - (c2 * (f2 - 0.5) + 0.5) |
            // * = |  c1(f1 - 0.5) - c2(f2 - 0.5) |
            // * = |  c1 f1 - c2 f2 - 0.5(c1 - c2) |
            |[1.0; 0.9] - [0.8; 0.3]| => ((1.0*0.9 - 0.8*0.3 - 0.5*(0.9 - 0.3) as Float).abs())
        }
        ok!()
    }

    /// 测试/is_negative
    #[test]
    fn is_negative() -> AResult {
        macro_once! {
            /// * 🚩模式：[真值的构造方法] ⇒ 预期
            macro test($( [ $($truth:tt)* ] => $expected:tt)*) {
                $(
                    assert_eq!(
                        truth!($($truth)*).is_negative(),
                        $expected
                    );
                )*
            }
            [1.0; 0.9] => false
            [0.9; 0.9] => false
            [0.8; 0.9] => false
            [0.7; 0.9] => false
            [0.6; 0.9] => false
            [0.5; 0.9] => false
            // [0.49995; 0.9] => false // 这个舍入到了0.5 | ❓边缘情况是否真的要纳入「单元测试」
            // 0.5以下均为「负面」
            // [0.49994; 0.9] => true
            [0.4; 0.9] => true
            [0.3; 0.9] => true
            [0.2; 0.9] => true
            [0.1; 0.9] => true
            [0.0; 0.9] => true
        }
        ok!()
    }

    /// 测试/to_display
    #[test]
    fn to_display() -> AResult {
        macro_once! {
            /// * 🚩模式：[真值的构造方法] ⇒ 预期
            macro test($( [ $($truth:tt)* ] => $expected:tt)*) {
                $(
                    assert_eq!(
                        truth!($($truth)*).to_display(),
                        $expected
                    );
                )*
            }
            // ! 注意：OpenNARS中格式化出的「真值」没有空格
            // 0
            [0.0   ; 0.0   ] => "%0.0000;0.0000%"
            // 1与非1
            [1.0   ; 1.0   ] => "%1.0000;1.0000%"
            [1.0   ; 0.9   ] => "%1.0000;0.9000%"
            [0.9   ; 1.0   ] => "%0.9000;1.0000%"
            [0.9   ; 0.9   ] => "%0.9000;0.9000%"
            // 各个位数
            [0.1   ; 0.42  ] => "%0.1000;0.4200%"
            [0.137 ; 0.442 ] => "%0.1370;0.4420%"
            [0.1024; 0.2185] => "%0.1024;0.2185%"
        }
        ok!()
    }

    /// 测试/to_display_brief
    #[test]
    fn to_display_brief() -> AResult {
        macro_once! {
            /// * 🚩模式：[真值的构造方法] ⇒ 预期
            macro test($( [ $($truth:tt)* ] => $expected:tt)*) {
                $(
                    assert_eq!(
                        truth!($($truth)*).to_display_brief(),
                        $expected
                    );
                )*
            }
            // ! 注意：OpenNARS中格式化出的「真值」没有空格
            // 0
            [0.0   ; 0.0   ] => "%0.00;0.00%"
            // 1与非1
            [1.0   ; 1.0   ] => "%1.00;1.00%"
            [1.0   ; 0.9   ] => "%1.00;0.90%"
            [0.9   ; 1.0   ] => "%0.90;1.00%"
            [0.9   ; 0.9   ] => "%0.90;0.90%"
            // 各个位数
            [0.1   ; 0.42  ] => "%0.10;0.42%"
            [0.137 ; 0.442 ] => "%0.14;0.44%" // ! 五入四舍
            [0.1024; 0.2185] => "%0.10;0.22%" // ! 四舍五入
            [0.999 ; 0.9999] => "%1.00;1.00%" // ! 五入到`1`
        }
        ok!()
    }

    /// 测试/from_lexical
    #[test]
    fn from_lexical() -> AResult {
        macro_once! {
            /// * 🚩模式：[词法真值构造方法] ⇒ 预期[真值的构造方法]
            macro test($(
                [ $($lexical:tt)* ] @ [$f:expr; $c:expr; $is_analytic:expr]
                => [ $($truth:tt)* ] )*
            ) {
                $(
                    // 构造
                    let lexical = narsese::lexical_truth!($($lexical)*);
                    let truth = truth!($($truth)*);
                    // 解析
                    let parsed = Truth::from_lexical(
                        lexical,
                        [ // 默认值（完全限定语法）
                            <<Truth as TruthValue>::E as ShortFloat>::from_float($f),
                            <<Truth as TruthValue>::E as ShortFloat>::from_float($c),
                        ],
                        $is_analytic
                    ).unwrap();
                    // 判等
                    assert_eq!(parsed, truth);
                )*
            }
            // 完全解析
            ["1.0" "0.9"] @ [0.0; 0.0; false] => [1.0; 0.9; false]
            ["1.0" "0.9"] @ [0.0; 0.0; true] => [1.0; 0.9; true]
            ["0.0" "0.0"] @ [1.0; 0.9; false] => [0.0; 0.0; false]
            ["0.0" "0.0"] @ [1.0; 0.9; true] => [0.0; 0.0; true]
            // 缺省
            ["0.0"] @ [1.0; 0.9; true] => [0.0; 0.9; true]
            [] @ [1.0; 0.9; true] => [1.0; 0.9; true]
            // 多余
            ["0.0" "0.1" "0.2"] @ [1.0; 0.9; true] => [0.0; 0.1; true]
            ["0.0" "0.1" "0.2" "0.3"] @ [1.0; 0.9; true] => [0.0; 0.1; true]
            ["0.0" "0.1" "ARCJ" "137442"] @ [1.0; 0.9; true] => [0.0; 0.1; true]
        }
        ok!()
    }
}
