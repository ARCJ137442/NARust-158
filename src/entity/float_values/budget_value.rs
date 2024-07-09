//! 🎯复刻OpenNARS `nars.entity.BudgetValue`
//! * ✅【2024-05-02 00:52:34】所有方法基本复刻完毕

use crate::__impl_to_display_and_display;
use crate::entity::ShortFloat;
use crate::{global::Float, inference::Budget, util::ToDisplayAndBrief};
use anyhow::Result;
use narsese::lexical::Budget as LexicalBudget;

/// [预算值](BudgetValue)的初步实现
/// * 🚩直接表示为一个三元组（但并非直接对元组实现）
#[derive(Debug, Default, Clone, Copy, PartialEq, Eq, Hash)]
pub struct BudgetValue(ShortFloat, ShortFloat, ShortFloat);

impl Budget for BudgetValue {
    #[inline(always)]
    fn priority(&self) -> ShortFloat {
        self.0 // * 🚩【2024-05-02 18:24:10】现在隐式`clone`
    }

    #[inline(always)]
    fn durability(&self) -> ShortFloat {
        self.1 // * 🚩【2024-05-02 18:24:10】现在隐式`clone`
    }

    #[inline(always)]
    fn quality(&self) -> ShortFloat {
        self.2 // * 🚩【2024-05-02 18:24:10】现在隐式`clone`
    }

    #[inline(always)]
    fn __priority_mut(&mut self) -> &mut ShortFloat {
        &mut self.0
    }

    #[inline(always)]
    fn __durability_mut(&mut self) -> &mut ShortFloat {
        &mut self.1
    }

    #[inline(always)]
    fn __quality_mut(&mut self) -> &mut ShortFloat {
        &mut self.2
    }
}

impl BudgetValue {
    /* impl BudgetConcrete for BudgetValue */

    #[inline(always)]
    pub fn new(p: ShortFloat, d: ShortFloat, q: ShortFloat) -> Self {
        Self(p, d, q)
    }

    pub fn from_floats(p: Float, d: Float, q: Float) -> Self {
        Self::new(
            ShortFloat::from_float(p),
            ShortFloat::from_float(d),
            ShortFloat::from_float(q),
        )
    }

    pub fn from_lexical(
        lexical: LexicalBudget,
        mut default_values: [ShortFloat; 3],
    ) -> Result<Self> {
        let sf_str = match lexical.len() {
            0 => &[],
            1 => &lexical[0..1],
            2 => &lexical[0..2],
            _ => &lexical[0..3],
        };
        // 预先解析默认值
        // ! ⚠️必须合法，否则panic
        let float_s = &mut default_values;
        for (i, s) in sf_str.iter().enumerate() {
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
        let [p, d, q] = *float_s;
        Ok(Self::new(p, d, q))
    }

    pub fn to_lexical(&self) -> LexicalBudget {
        vec![
            self.priority().to_display_brief(),
            self.durability().to_display_brief(),
            self.quality().to_display_brief(),
        ]
    }

    /// 从其它支持「预算」特征的对象引用转换
    pub fn from_other(other: &impl Budget) -> Self {
        Self::new(other.priority(), other.durability(), other.quality())
    }
}

/// 允许将所有[`Budget`]的引用转换为[`BudgetValue`]
/// * 🚩在其中创建新「真值」对象
/// * 📝Rust对[`Into`]分派方法时，能实现「自身类型⇒直接传递自身⇒内联」的「零成本抽象」
impl<T: Budget> From<&T> for BudgetValue {
    fn from(value: &T) -> Self {
        Self::new(value.priority(), value.durability(), value.quality())
    }
}

/// 允许通过「短浮点三数组」转换为预算值
impl<SF: Into<ShortFloat>> From<[SF; 3]> for BudgetValue {
    fn from([p, d, q]: [SF; 3]) -> Self {
        Self::new(p.into(), d.into(), q.into())
    }
}

/// 允许通过「pdq三元组」转换为预算值
impl<P: Into<ShortFloat>, D: Into<ShortFloat>, Q: Into<ShortFloat>> From<(P, D, Q)>
    for BudgetValue
{
    fn from((p, d, q): (P, D, Q)) -> Self {
        Self::new(p.into(), d.into(), q.into())
    }
}

// 自动派生并实现[`ToDisplayAndBrief`]与[`Display`]
__impl_to_display_and_display! {
    @( budget_to_display; budget_to_display_brief;)
    BudgetValue as Budget
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{ok, util::AResult};
    use nar_dev_utils::macro_once;

    /// 定义要测试的「预算值」类型
    type BudgetValue = super::BudgetValue;
    type SF = ShortFloat;

    /// 快捷构造宏
    macro_rules! budget {
        // 三参数
        ($p:expr; $d:expr; $q:expr) => {
            BudgetValue::from_floats($p, $d, $q)
        };
    }

    // * ✅测试/new已在「快捷构造宏」中实现

    // * ✅测试/from_float已在「快捷构造宏」中实现

    /// 测试/priority
    #[test]
    fn priority() -> AResult {
        macro_once! {
            /// * 🚩模式：[预算值的构造方法] ⇒ 预期「短浮点」浮点值
            macro test($( [ $($budget:tt)* ] => $expected:tt)*) {
                $(
                    assert_eq!(
                        budget!($($budget)*).priority(),
                        SF::from_float($expected)
                    );
                )*
            }
            [0.5; 0.5; 0.5] => 0.5
            [0.1; 0.9; 0.5] => 0.1
            [0.0001; 0.9; 0.5] => 0.0001
            [0.1024; 0.0; 0.5] => 0.1024
            [0.2; 0.1; 0.5] => 0.2
        }
        ok!()
    }

    // * ✅测试/__priority_mut已经在`set_priority`中实现

    /// 测试/set_priority
    #[test]
    fn set_priority() -> AResult {
        macro_once! {
            /// * 🚩模式：[预算值的构造方法] → 要被赋的值 ⇒ 预期「短浮点」浮点值
            macro test($( [ $($budget:tt)* ] -> $new_float:tt => $expected:tt)*) {
                $(
                    let mut t = budget!($($budget)*);
                    t.set_priority(SF::from_float($new_float));
                    // 可变与不可变一致
                    assert_eq!(t.priority(), *t.__priority_mut());
                    // 修改后与所读值一致
                    assert_eq!(*t.__priority_mut(), SF::from_float($expected));
                )*
            }
            [1.0; 0.9; 0.5] -> 0.5 => 0.5
            [0.1; 0.9; 0.5] -> 0.2 => 0.2
            [0.0001; 0.9; 0.5] -> 0.8 => 0.8
            [0.1024; 0.0; 0.5] -> 0.0 => 0.0
            [0.2; 0.1; 0.5] -> 1.0 => 1.0
        }
        ok!()
    }

    /// 测试/durability
    #[test]
    fn durability() -> AResult {
        macro_once! {
            /// * 🚩模式：[预算值的构造方法] ⇒ 预期「短浮点」浮点值
            macro test($( [ $($budget:tt)* ] => $expected:tt)*) {
                $(
                    assert_eq!(
                        budget!($($budget)*).durability(),
                        SF::from_float($expected)
                    );
                )*
            }
            [0.5; 0.5; 0.5] => 0.5
            [0.1; 0.9; 0.5] => 0.9
            [0.9; 0.0001; 0.5] => 0.0001
            [0.0; 0.1024; 0.5] => 0.1024
            [0.1; 0.2; 0.5] => 0.2
        }
        ok!()
    }

    // * ✅测试/__durability_mut已经在`set_durability`中实现

    /// 测试/set_durability
    #[test]
    fn set_durability() -> AResult {
        macro_once! {
            /// * 🚩模式：[预算值的构造方法] → 要被赋的值 ⇒ 预期「短浮点」浮点值
            macro test($( [ $($budget:tt)* ] -> $new_float:tt => $expected:tt)*) {
                $(
                    let mut t = budget!($($budget)*);
                    t.set_durability(SF::from_float($new_float));
                    // 可变与不可变一致
                    assert_eq!(t.durability(), *t.__durability_mut());
                    // 修改后与所读值一致
                    assert_eq!(*t.__durability_mut(), SF::from_float($expected));
                )*
            }
            [1.0; 0.9; 0.5] -> 0.5 => 0.5
            [0.1; 0.9; 0.5] -> 0.2 => 0.2
            [0.0001; 0.9; 0.5] -> 0.8 => 0.8
            [0.1024; 0.1; 0.5] -> 0.0 => 0.0
            [0.2; 0.1; 0.5] -> 1.0 => 1.0
        }
        ok!()
    }

    /// 测试/quality
    #[test]
    fn quality() -> AResult {
        macro_once! {
            /// * 🚩模式：[预算值的构造方法] ⇒ 预期「短浮点」浮点值
            macro test($( [ $($budget:tt)* ] => $expected:tt)*) {
                $(
                    assert_eq!(
                        budget!($($budget)*).quality(),
                        SF::from_float($expected)
                    );
                )*
            }
            [0.5; 0.5; 0.5] => 0.5
            [0.1; 0.9; 0.5] => 0.5
            [0.9; 0.5; 0.0001] => 0.0001
            [0.0; 0.5; 0.1024] => 0.1024
            [0.1; 0.2; 0.5] => 0.5
        }
        ok!()
    }

    // * ✅测试/__quality_mut已经在`set_quality`中实现

    /// 测试/set_quality
    #[test]
    fn set_quality() -> AResult {
        fn test(mut budget: BudgetValue, new_float: Float, expected: Float) {
            budget.set_quality(SF::from_float(new_float));
            // 可变与不可变一致
            assert_eq!(budget.quality(), *budget.__quality_mut());
            // 修改后与所读值一致
            assert_eq!(*budget.__quality_mut(), SF::from_float(expected));
        }
        macro_once! {
            /// * 🚩模式：[预算值的构造方法] → 要被赋的值 ⇒ 预期「短浮点」浮点值
            macro test($( [ $($budget:tt)* ] -> $new_float:tt => $expected:tt)*) {
                $(
                    test(
                        budget!($($budget)*),
                        $new_float,
                        $expected
                    );
                )*
            }
            [1.0; 0.9; 0.5] -> 0.5 => 0.5
            [0.1; 0.9; 0.52] -> 0.2 => 0.2
            [0.0001; 0.9; 0.54] -> 0.8 => 0.8
            [0.1024; 0.1; 0.75] -> 0.0 => 0.0
            [0.2; 0.1; 0.15] -> 1.0 => 1.0
        }
        ok!()
    }

    // /// 测试/inc_priority
    // #[test]
    // fn inc_priority() -> AResult {
    //     macro_once! {
    //         /// * 🚩模式：[预算值的构造方法] + 参数 ⇒ 预期「短浮点」浮点值
    //         macro test($( [ $($budget:tt)* ] + $delta:tt => $expected:tt)*) {
    //             $(
    //                 let mut t = budget!($($budget)*);
    //                 t.inc_priority(SF::from_float($delta));
    //                 // 修改后与所读值一致
    //                 assert_eq!(t.priority(), SF::from_float($expected));
    //             )*
    //         }
    //         [1.0; 0.9; 0.5] + 0.5 => 1.0
    //         [0.1; 0.9; 0.52] + 0.2 => (1.0 - (0.9 * 0.8))
    //         [0.5; 0.9; 0.54] + 0.8 => (1.0 - (0.5 * 0.2))
    //         [0.1024; 0.1; 0.75] + 0.0 => 0.1024
    //         [0.2; 0.1; 0.15] + 1.0 => 1.0
    //     }
    //     ok!()
    // }

    // /// 测试/dec_priority
    // #[test]
    // fn dec_priority() -> AResult {
    //     macro_once! {
    //         /// * 🚩模式：[预算值的构造方法] - 参数 ⇒ 预期「短浮点」浮点值
    //         macro test($( [ $($budget:tt)* ] - $delta:tt => $expected:tt)*) {
    //             $(
    //                 let mut t = budget!($($budget)*);
    //                 t.dec_priority(SF::from_float($delta));
    //                 // 修改后与所读值一致
    //                 assert_eq!(t.priority(), SF::from_float($expected));
    //             )*
    //         }
    //         [1.0; 0.9; 0.5] - 0.5 => 0.5
    //         [0.1; 0.9; 0.52] - 0.2 => (0.1 * 0.2)
    //         [0.5; 0.9; 0.54] - 0.8 => (0.5 * 0.8)
    //         [0.1024; 0.1; 0.75] - 0.0 => 0.0
    //         [0.2; 0.1; 0.15] - 1.0 => 0.2
    //     }
    //     ok!()
    // }

    // /// 测试/inc_durability
    // #[test]
    // fn inc_durability() -> AResult {
    //     macro_once! {
    //         /// * 🚩模式：[预算值的构造方法] + 参数 ⇒ 预期「短浮点」浮点值
    //         macro test($( [ $($budget:tt)* ] + $delta:tt => $expected:tt)*) {
    //             $(
    //                 let mut t = budget!($($budget)*);
    //                 t.inc_durability(SF::from_float($delta));
    //                 // 修改后与所读值一致
    //                 assert_eq!(t.durability(), SF::from_float($expected));
    //             )*
    //         }
    //         [0.9; 1.0; 0.5] + 0.5 => 1.0
    //         [0.9; 0.1; 0.52] + 0.2 => (1.0 - (0.9 * 0.8))
    //         [0.9; 0.5; 0.54] + 0.8 => (1.0 - (0.5 * 0.2))
    //         [0.1; 0.1024; 0.75] + 0.0 => 0.1024
    //         [0.1; 0.2; 0.15] + 1.0 => 1.0
    //     }
    //     ok!()
    // }

    // /// 测试/dec_durability
    // #[test]
    // fn dec_durability() -> AResult {
    //     macro_once! {
    //         /// * 🚩模式：[预算值的构造方法] - 参数 ⇒ 预期「短浮点」浮点值
    //         macro test($( [ $($budget:tt)* ] - $delta:tt => $expected:tt)*) {
    //             $(
    //                 let mut t = budget!($($budget)*);
    //                 t.dec_durability(SF::from_float($delta));
    //                 // 修改后与所读值一致
    //                 assert_eq!(t.durability(), SF::from_float($expected));
    //             )*
    //         }
    //         [0.9; 1.0; 0.5] - 0.5 => 0.5
    //         [0.9; 0.1; 0.52] - 0.2 => (0.1 * 0.2)
    //         [0.9; 0.5; 0.54] - 0.8 => (0.5 * 0.8)
    //         [0.1; 0.1024; 0.75] - 0.0 => 0.0
    //         [0.1; 0.2; 0.15] - 1.0 => 0.2
    //     }
    //     ok!()
    // }

    // /// 测试/inc_quality
    // #[test]
    // fn inc_quality() -> AResult {
    //     macro_once! {
    //         /// * 🚩模式：[预算值的构造方法] + 参数 ⇒ 预期「短浮点」浮点值
    //         macro test($( [ $($budget:tt)* ] + $delta:tt => $expected:tt)*) {
    //             $(
    //                 let mut t = budget!($($budget)*);
    //                 t.inc_quality(SF::from_float($delta));
    //                 // 修改后与所读值一致
    //                 assert_eq!(t.quality(), SF::from_float($expected));
    //             )*
    //         }
    //         [0.9; 0.5; 1.0] + 0.5 => 1.0
    //         [0.9; 0.52; 0.1] + 0.2 => (1.0 - (0.9 * 0.8))
    //         [0.9; 0.54; 0.5] + 0.8 => (1.0 - (0.5 * 0.2))
    //         [0.1; 0.75; 0.1024] + 0.0 => 0.1024
    //         [0.1; 0.15; 0.2] + 1.0 => 1.0
    //     }
    //     ok!()
    // }

    // /// 测试/dec_quality
    // #[test]
    // fn dec_quality() -> AResult {
    //     macro_once! {
    //         /// * 🚩模式：[预算值的构造方法] - 参数 ⇒ 预期「短浮点」浮点值
    //         macro test($( [ $($budget:tt)* ] - $delta:tt => $expected:tt)*) {
    //             $(
    //                 let mut t = budget!($($budget)*);
    //                 t.dec_quality(SF::from_float($delta));
    //                 // 修改后与所读值一致
    //                 assert_eq!(t.quality(), SF::from_float($expected));
    //             )*
    //         }
    //         [0.9; 0.5; 1.0] - 0.5 => 0.5
    //         [0.9; 0.52; 0.1] - 0.2 => (0.1 * 0.2)
    //         [0.9; 0.54; 0.5] - 0.8 => (0.5 * 0.8)
    //         [0.1; 0.75; 0.1024] - 0.0 => 0.0
    //         [0.1; 0.15; 0.2] - 1.0 => 0.2
    //     }
    //     ok!()
    // }

    /// 测试/summary
    #[test]
    fn summary() -> AResult {
        fn test(budget: BudgetValue, expected: SF) {
            assert_eq!(budget.budget_summary(), expected);
        }
        macro_once! {
            /// * 🚩模式：[预算值的构造方法] ⇒ 预期「短浮点」浮点值
            macro test($( [ $($budget:tt)* ] => $expected:tt)*) {
                $(
                    test(
                        budget!($($budget)*),
                        SF::from_float($expected)
                    );
                )*
            }
            [0.0; 0.0; 0.0] => 0.0
            [0.5; 0.5; 0.5] => 0.5
            [1.0; 1.0; 1.0] => 1.0
            [0.25; 1.0; 0.5] => 0.5
            [0.81; 0.9; 1.0] => 0.9
            [0.01; 0.1; 1.0] => 0.1
            [0.2; 0.04; 0.008] => 0.04
        }
        ok!()
    }

    /// 测试/above_threshold
    #[test]
    fn above_threshold() -> AResult {
        fn test(budget: BudgetValue, threshold: Float, expected: bool) {
            assert_eq!(budget.budget_above_threshold(threshold), expected);
        }
        macro_once! {
            /// * 🚩模式：[预算值的构造方法] @ 阈值 ⇒ 预期
            macro test($( [ $($budget:tt)* ] @ $threshold:expr => $expected:tt)*) {
                $(
                    test(
                        budget!($($budget)*),
                        $threshold,
                        $expected
                    );
                )*
            }
            // 1.0对任何阈值都是`true`
            [1.0; 1.0; 1.0] @ 0.0 => true
            [1.0; 1.0; 1.0] @ 0.5 => true
            [1.0; 1.0; 1.0] @ 1.0 => true
            // 相等情况
            [0.0; 0.0; 0.0] @ 0.0 => true
            [0.5; 0.5; 0.5] @ 0.5 => true
            [0.25; 1.0; 0.5] @ 0.5 => true
            [0.81; 0.9; 1.0] @ 0.9 => true
            [0.01; 0.1; 1.0] @ 0.1 => true
            [0.2; 0.04; 0.008] @ 0.04 => true
            // 边界情况
            [0.0; 0.0; 0.0] @ 0.001 => false
            [0.5; 0.5; 0.5] @ 0.501 => false
            [0.25; 1.0; 0.5] @ 0.501 => false
            [0.81; 0.9; 1.0] @ 0.901 => false
            [0.01; 0.1; 1.0] @ 0.101 => false
            [0.2; 0.04; 0.008] @ 0.041 => false
        }
        ok!()
    }

    /// 测试/to_display
    #[test]
    fn to_display() -> AResult {
        macro_once! {
            /// * 🚩模式：[预算值的构造方法] ⇒ 预期
            macro test($( [ $($budget:tt)* ] => $expected:tt)*) {
                $(
                    assert_eq!(
                        budget!($($budget)*).to_display(),
                        $expected
                    );
                )*
            }
            // ! 注意：OpenNARS中格式化出的「预算值」没有空格
            // 0
            [0.0   ; 0.0   ; 0.0   ] => "$0.0000;0.0000;0.0000$"
            // 1与非1
            [1.0   ; 1.0   ; 1.0   ] => "$1.0000;1.0000;1.0000$"
            [1.0   ; 1.0   ; 0.9   ] => "$1.0000;1.0000;0.9000$"
            [1.0   ; 0.9   ; 1.0   ] => "$1.0000;0.9000;1.0000$"
            [1.0   ; 0.9   ; 0.9   ] => "$1.0000;0.9000;0.9000$"
            [0.9   ; 1.0   ; 1.0   ] => "$0.9000;1.0000;1.0000$"
            [0.9   ; 1.0   ; 0.9   ] => "$0.9000;1.0000;0.9000$"
            [0.9   ; 0.9   ; 1.0   ] => "$0.9000;0.9000;1.0000$"
            [0.9   ; 0.9   ; 0.9   ] => "$0.9000;0.9000;0.9000$"
            // 各个位数
            [0.1   ; 0.2   ; 0.3   ] => "$0.1000;0.2000;0.3000$"
            [0.10  ; 0.20  ; 0.30  ] => "$0.1000;0.2000;0.3000$"
            [0.13  ; 0.74  ; 0.42  ] => "$0.1300;0.7400;0.4200$"
            [0.137 ; 0.442 ; 0.0   ] => "$0.1370;0.4420;0.0000$"
            [0.0   ; 0.1024; 0.2185] => "$0.0000;0.1024;0.2185$"
        }
        ok!()
    }

    /// 测试/to_display_brief
    #[test]
    fn to_display_brief() -> AResult {
        macro_once! {
            /// * 🚩模式：[预算值的构造方法] ⇒ 预期
            macro test($( [ $($budget:tt)* ] => $expected:tt)*) {
                $(
                    assert_eq!(
                        budget!($($budget)*).to_display_brief(),
                        $expected
                    );
                )*
            }
            // ! 注意：OpenNARS中格式化出的「预算值」没有空格
            // 0
            [0.0   ; 0.0   ; 0.0   ] => "$0.00;0.00;0.00$"
            // 1与非1
            [1.0   ; 1.0   ; 1.0   ] => "$1.00;1.00;1.00$"
            [1.0   ; 1.0   ; 0.9   ] => "$1.00;1.00;0.90$"
            [1.0   ; 0.9   ; 1.0   ] => "$1.00;0.90;1.00$"
            [1.0   ; 0.9   ; 0.9   ] => "$1.00;0.90;0.90$"
            [0.9   ; 1.0   ; 1.0   ] => "$0.90;1.00;1.00$"
            [0.9   ; 1.0   ; 0.9   ] => "$0.90;1.00;0.90$"
            [0.9   ; 0.9   ; 1.0   ] => "$0.90;0.90;1.00$"
            [0.9   ; 0.9   ; 0.9   ] => "$0.90;0.90;0.90$"
            // 各个位数
            [0.1   ; 0.2   ; 0.3   ] => "$0.10;0.20;0.30$"
            [0.10  ; 0.20  ; 0.30  ] => "$0.10;0.20;0.30$"
            [0.13  ; 0.74  ; 0.42  ] => "$0.13;0.74;0.42$"
            [0.137 ; 0.442 ; 0.0   ] => "$0.14;0.44;0.00$" // ! 五入四舍
            [0.0   ; 0.1024; 0.2185] => "$0.00;0.10;0.22$" // ! 四舍五入
            [0.99   ; 0.999; 0.9999] => "$0.99;1.00;1.00$" // ! 舍入到`1`
        }
        ok!()
    }

    /// 测试/from_lexical
    #[test]
    fn from_lexical() -> AResult {
        fn test(budget: BudgetValue, lexical: LexicalBudget, pdq: [Float; 3]) {
            // 解析
            let [p, d, q] = pdq;
            let parsed = BudgetValue::from_lexical(
                lexical,
                [
                    // 默认值（完全限定语法）
                    ShortFloat::from_float(p),
                    ShortFloat::from_float(d),
                    ShortFloat::from_float(q),
                ],
            )
            .unwrap();
            // 判等
            assert_eq!(parsed, budget);
        }
        macro_once! {
            /// * 🚩模式：[词法预算值构造方法] ⇒ 预期[预算值的构造方法]
            macro test($(
                [ $($lexical:tt)* ] @ [$p:expr; $d:expr; $q:expr]
                => [ $($budget:tt)* ] )*
            ) {
                $(
                    test(
                        // 构造
                        budget!($($budget)*),
                        narsese::lexical_budget!($($lexical)*),
                        [ $p, $d, $q ],
                    );
                )*
            }
            // 完全解析
            ["1.0" "0.9" "0.5"] @ [0.0; 0.0; 0.0] => [1.0; 0.9; 0.5]
            ["0.1" "0.2" "0.3"] @ [0.4; 0.5; 0.6] => [0.1; 0.2; 0.3]
            // 缺省
            ["0.1" "0.2"] @ [0.5; 0.5; 0.5] => [0.1; 0.2; 0.5]
            ["0.1"] @ [0.5; 0.5; 0.5] => [0.1; 0.5; 0.5]
            [] @ [0.5; 0.5; 0.5] => [0.5; 0.5; 0.5]
            // 多余
            ["0.1" "0.2" "0.3" "0.4"] @ [0.4; 0.5; 0.6] => [0.1; 0.2; 0.3]
            ["0.1" "0.2" "0.3" "ARCJ" "137442"] @ [0.4; 0.5; 0.6] => [0.1; 0.2; 0.3]
        }
        ok!()
    }
}
