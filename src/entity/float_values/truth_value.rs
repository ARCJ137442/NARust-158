//! 🎯复刻OpenNARS `nars.entity.TruthValue`
//! * 📌【2024-05-02 21:30:40】从「预算函数」来：一些地方必须用到「真值」及其方法
//! * ✅【2024-05-03 16:21:02】所有方法基本复刻完毕

use super::ShortFloat;
use crate::{
    __impl_to_display_and_display, global::Float, inference::Truth, util::ToDisplayAndBrief,
};
use anyhow::Result;
use narsese::lexical::Truth as LexicalTruth;
use std::{
    fmt::Debug,
    hash::{Hash, Hasher},
};

/// [`TruthValue`]初代实现
/// * 🎯测试特征的效果
/// * 📌[`PartialEq`]、[`Eq`]、[`Hash`]均特别实现
///
/// # 📄OpenNARS
///
/// Frequency and confidence.
#[derive(Debug, Clone, Copy, Default, Eq)]
pub struct TruthValue {
    /// frequency
    f: ShortFloat,
    /// confidence
    c: ShortFloat,
    /// analytic
    a: bool,
}

/// 定制的序列反序列化方法
/// * 🎯节省序列化后的占用空间
///   * 📄在JSON中不再需要是一个object，是一个`[f, c, a]`三元组就行
mod serde {
    use super::TruthValue;
    use serde::{Deserialize, Deserializer, Serialize, Serializer};

    impl Serialize for TruthValue {
        fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
        where
            S: Serializer,
        {
            // 直接委托到内部整数值
            (self.f, self.c, self.a).serialize(serializer)
        }
    }

    impl<'de> Deserialize<'de> for TruthValue {
        fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
        where
            D: Deserializer<'de>,
        {
            // 先反序列化到内部整数值
            let (f, c, a) = Deserialize::deserialize(deserializer)?;
            // 然后尝试创建，并在其中转换Error类型
            Ok(Self { f, c, a })
        }
    }
}

impl Truth for TruthValue {
    #[inline(always)]
    fn frequency(&self) -> ShortFloat {
        self.f
    }

    #[inline(always)]
    fn frequency_mut(&mut self) -> &mut ShortFloat {
        &mut self.f
    }

    #[inline(always)]
    fn confidence(&self) -> ShortFloat {
        self.c
    }

    #[inline(always)]
    fn confidence_mut(&mut self) -> &mut ShortFloat {
        &mut self.c
    }

    #[inline(always)]
    fn is_analytic(&self) -> bool {
        self.a
    }

    #[inline(always)]
    fn set_analytic(&mut self) {
        self.a = true;
    }
}

/* impl TruthValueConcrete for TruthV1 */
impl TruthValue {
    #[inline(always)]
    pub fn new(frequency: ShortFloat, confidence: ShortFloat, is_analytic: bool) -> Self {
        Self {
            f: frequency,
            c: confidence,
            a: is_analytic,
        }
    }

    pub fn from(truth: &impl Truth) -> Self {
        Self::new(truth.frequency(), truth.confidence(), truth.is_analytic())
    }

    pub fn new_fc(frequency: ShortFloat, confidence: ShortFloat) -> Self {
        Self::new(frequency, confidence, false)
    }

    pub fn from_floats(frequency: Float, confidence: Float, is_analytic: bool) -> Self {
        Self::new(
            ShortFloat::from_float(frequency),
            ShortFloat::from_float(confidence),
            is_analytic,
        )
    }

    pub fn from_fc(frequency: Float, confidence: Float) -> Self {
        Self::new_fc(
            ShortFloat::from_float(frequency),
            ShortFloat::from_float(confidence),
        )
    }

    pub fn new_analytic_default() -> Self {
        /* 📄OpenNARS源码 @ TruthFunctions：
        new TruthValue(0.5f, 0f); */
        Self::new(ShortFloat::HALF, ShortFloat::ZERO, false)
    }

    pub fn from_lexical(
        lexical: LexicalTruth,
        mut default_values: [ShortFloat; 2],
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
            let sf = match ShortFloat::try_from(v) {
                Ok(sf) => sf,
                Err(_) => return Err(anyhow::anyhow!("无效短浮点值：{v}")),
            };
            float_s[i] = sf;
        }
        // 构造
        let [f, c] = *float_s;
        Ok(Self::new(f, c, is_analytic))
    }

    pub fn to_lexical(&self) -> LexicalTruth {
        vec![
            self.frequency().to_display_brief(),
            self.confidence().to_display_brief(),
        ]
    }
}

/// 允许将所有[`Truth`]的引用转换为[`TruthValue`]
/// * 🚩在其中创建新「真值」对象
/// * 📝Rust对[`Into`]分派方法时，能实现「自身类型⇒直接传递自身⇒内联」的「零成本抽象」
impl<T: Truth> From<&T> for TruthValue {
    fn from(value: &T) -> Self {
        Self::new(value.frequency(), value.confidence(), value.is_analytic())
    }
}

__impl_to_display_and_display! {
    @(truth_to_display; truth_to_display_brief;)
    TruthValue as Truth
}

/// 模拟`equals`
/// * ⚠️其中[`Self::a`]即`isAnalytic`不参与判等
impl PartialEq for TruthValue {
    #[inline(always)]
    fn eq(&self, other: &Self) -> bool {
        self.f == other.f && self.c == other.c
    }
}

/// 手动实现[`Hash`]
/// * ⚠️因为[`Self::a`]不参与判等，因此也不能参与到「散列化」中
impl Hash for TruthValue {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.f.hash(state);
        self.c.hash(state);
        // self.a.hash(state);
    }
}

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
            TruthValue::from_fc($f, $c)
        };
        // 三参数
        ($f:expr; $c:expr; $a:expr) => {
            TruthValue::from_floats($f, $c, $a)
        };
    }
}

/// 单元测试
#[cfg(test)]
mod tests {
    use super::*;
    use crate::{ok, truth, util::AResult};
    use nar_dev_utils::macro_once;

    /// 定义要测试的「真值」类型
    type TruthV = TruthValue;
    type SF = ShortFloat;

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
        fn test(mut truth: TruthV) {
            truth.set_analytic();
            assert!(truth.is_analytic());
        }
        macro_once! {
            /// * 🚩模式：[真值的构造方法]
            macro test($( [ $($truth:tt)* ])*) {
                $(
                    test(truth!($($truth)*));
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
        fn test(truth: TruthV, expected: Float) {
            assert_eq!(truth.expectation(), expected);
        }
        macro_once! {
            /// * 🚩模式：[真值的构造方法] ⇒ 预期
            macro test($( [ $($truth:tt)* ] => $expected:tt)*) {
                $(
                    test(truth!($($truth)*), $expected);
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
        fn test(truth1: TruthV, truth2: TruthV, expected: Float) {
            assert_eq!(truth1.expectation_abs_dif(&truth2), expected);
        }
        macro_once! {
            /// * 🚩模式：| [真值的构造方法] - [真值的构造方法] | ⇒ 预期
            macro test($( | [ $($truth1:tt)* ] - [ $($truth2:tt)* ] | => $expected:tt)*) {
                $(
                    test(
                        truth!($($truth1)*),truth!($($truth2)*),
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
        fn test(truth: TruthV, expected: bool) {
            assert_eq!(truth.is_negative(), expected);
        }
        macro_once! {
            /// * 🚩模式：[真值的构造方法] ⇒ 预期
            macro test($( [ $($truth:tt)* ] => $expected:tt)*) {
                $(
                    test(truth!($($truth)*), $expected);
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
        fn test(lexical: LexicalTruth, truth: TruthV, fc: [Float; 2], is_analytic: bool) {
            // 解析
            let [f, c] = fc;
            let parsed = TruthV::from_lexical(
                lexical,
                [
                    // 默认值（完全限定语法）
                    ShortFloat::from_float(f),
                    ShortFloat::from_float(c),
                ],
                is_analytic,
            )
            .unwrap();
            // 判等
            assert_eq!(parsed, truth);
        }
        macro_once! {
            /// * 🚩模式：[词法真值构造方法] ⇒ 预期[真值的构造方法]
            macro test($(
                [ $($lexical:tt)* ] @ [$f:expr; $c:expr; $is_analytic:expr]
                => [ $($truth:tt)* ] )*
            ) {
                $(
                    test(
                        narsese::lexical_truth!($($lexical)*),
                        truth!($($truth)*),
                        [$f, $c],
                        $is_analytic
                    );
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
