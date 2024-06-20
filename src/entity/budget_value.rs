//! 🎯复刻OpenNARS `nars.entity.BudgetValue`
//! * ✅【2024-05-02 00:52:34】所有方法基本复刻完毕

use super::ShortFloat;
use crate::{
    global::Float,
    io::symbols::{BUDGET_VALUE_MARK, VALUE_SEPARATOR},
    ToDisplayAndBrief,
};
use anyhow::Result;
use nar_dev_utils::join;
use narsese::lexical::Budget as LexicalBudget;

/// 模拟`nars.entity.BudgetValue`
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
pub trait BudgetValue: ToDisplayAndBrief {
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
    ///
    /// # 📄OpenNARS
    ///
    /// Fully display the BudgetValue
    ///
    /// @return String representation of the value
    fn __to_display(&self) -> String {
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
    fn __to_display_brief(&self) -> String {
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

/// 预算值的「具体类型」
/// * 🎯有选择地支持「限定的构造函数」
///   * 📄需要构造函数：预算函数中「创建新值的函数」
///   * 📄不要构造函数：具有「预算值属性」但【不可从预算值参数构造】的类型
///     * 📄概念[`super::Concept`]
///     * 📄任务[`super::Task`]
/// * 📌整个特征建立在「预算值就是预算值」，即「实现者本身**只有**p、d、q三元组」的基础上
/// * 🚩包括「构造函数」与「转换函数」
/// * 🚩【2024-05-08 11:21:14】为了在「记忆区」中允许复制值，此处需要要求[`Clone`]特征
pub trait BudgetValueConcrete: BudgetValue + Sized + Clone {
    /// 内置构造函数(p, d, q)
    /// * 🚩直接从「短浮点」构造
    fn new(p: ShortFloat, d: ShortFloat, q: ShortFloat) -> Self;

    /// 模拟 `new BudgetValue(p, d, q)`
    /// * 🚩将浮点数分别转换为「短浮点」
    ///
    /// # 📄OpenNARS
    ///
    /// Constructor with initialization
    ///
    /// @param p Initial priority
    /// @param d Initial durability
    /// @param q Initial quality
    #[inline(always)]
    fn from_floats(p: Float, d: Float, q: Float) -> Self {
        Self::new(
            ShortFloat::from_float(p),
            ShortFloat::from_float(d),
            ShortFloat::from_float(q),
        )
    }

    /// 🆕「词法预算值」到「自身类型」的转换
    /// * 🎯统一的、全面的「词法预算值→预算值」转换方法
    /// * 📌需要手动输入「默认值」
    fn from_lexical(lexical: LexicalBudget, mut default_values: [ShortFloat; 3]) -> Result<Self> {
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

    /// 🆕自身到「词法」的转换
    /// * 🎯标准Narsese输出需要（Narsese内容）
    /// * 🚩【2024-05-12 14:48:31】此处跟随OpenNARS，仅用两位小数
    fn to_lexical(&self) -> LexicalBudget {
        vec![
            self.priority().to_display_brief(),
            self.durability().to_display_brief(),
            self.quality().to_display_brief(),
        ]
    }
}

/// 初代实现
mod impl_v1 {
    use super::*;
    use crate::__impl_to_display_and_display;

    /// [预算值](BudgetValue)的初步实现
    /// * 🚩直接表示为一个三元组（但并非直接对元组实现）
    #[derive(Debug, Default, Clone, Copy, PartialEq, Eq, Hash)]
    pub struct BudgetV1(ShortFloat, ShortFloat, ShortFloat);

    impl BudgetValue for BudgetV1 {
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

    impl BudgetValueConcrete for BudgetV1 {
        #[inline(always)]
        fn new(p: ShortFloat, d: ShortFloat, q: ShortFloat) -> Self {
            Self(p, d, q)
        }
    }

    // 自动派生并实现[`ToDisplayAndBrief`]与[`Display`]
    __impl_to_display_and_display! {
        BudgetV1 as BudgetValue
    }
}
pub use impl_v1::*;

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{global::tests::AResult, ok};
    use nar_dev_utils::macro_once;

    /// 定义要测试的「预算值」类型
    type Budget = BudgetV1;
    type SF = ShortFloat;

    /// 快捷构造宏
    macro_rules! budget {
        // 三参数
        ($p:expr; $d:expr; $q:expr) => {
            Budget::from_floats($p, $d, $q)
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
        macro_once! {
            /// * 🚩模式：[预算值的构造方法] → 要被赋的值 ⇒ 预期「短浮点」浮点值
            macro test($( [ $($budget:tt)* ] -> $new_float:tt => $expected:tt)*) {
                $(
                    let mut t = budget!($($budget)*);
                    t.set_quality(SF::from_float($new_float));
                    // 可变与不可变一致
                    assert_eq!(t.quality(), *t.__quality_mut());
                    // 修改后与所读值一致
                    assert_eq!(*t.__quality_mut(), SF::from_float($expected));
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
        macro_once! {
            /// * 🚩模式：[预算值的构造方法] ⇒ 预期「短浮点」浮点值
            macro test($( [ $($budget:tt)* ] => $expected:tt)*) {
                $(
                    assert_eq!(
                        budget!($($budget)*).summary(),
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
        macro_once! {
            /// * 🚩模式：[预算值的构造方法] @ 阈值 ⇒ 预期
            macro test($( [ $($budget:tt)* ] @ $threshold:expr => $expected:tt)*) {
                $(
                    assert_eq!(
                        budget!($($budget)*).above_threshold(SF::from_float($threshold)),
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
        macro_once! {
            /// * 🚩模式：[词法预算值构造方法] ⇒ 预期[预算值的构造方法]
            macro test($(
                [ $($lexical:tt)* ] @ [$p:expr; $d:expr; $q:expr]
                => [ $($budget:tt)* ] )*
            ) {
                $(
                    // 构造
                    let lexical = narsese::lexical_budget!($($lexical)*);
                    let budget = budget!($($budget)*);
                    // 解析
                    let parsed = Budget::from_lexical(
                        lexical,
                        [ // 默认值（完全限定语法）
                            ShortFloat::from_float($p),
                            ShortFloat::from_float($d),
                            ShortFloat::from_float($q),
                        ],
                    ).unwrap();
                    // 判等
                    assert_eq!(parsed, budget);
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
