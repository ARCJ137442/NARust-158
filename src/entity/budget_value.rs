//! 🎯复刻OpenNARS `nars.entity.BudgetValue`
//! * ✅【2024-05-02 00:52:34】所有方法基本复刻完毕

use super::{ShortFloat, ShortFloatV1};
use crate::{global::Float, inference::UtilityFunctions};

/// 模拟OpenNARS `nars.entity.BudgetValue`
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
pub trait BudgetValue {
    /// 一种类型只可能有一种「证据值」
    /// * ✅兼容OpenNARS `ShortFloat`
    type E: ShortFloat;

    /// 模拟`BudgetValue.getPriority`
    /// * 🚩获取优先级
    /// * 🚩【2024-05-02 18:21:38】现在统一获取值：对「实现了[`Copy`]的类型」直接复制
    ///
    /// # 📄OpenNARS
    ///
    /// Get priority value
    ///
    /// @return The current priority
    fn priority(&self) -> Self::E;
    /// 获取优先级（可变）
    /// * 📌【2024-05-03 17:39:04】目前设置为内部方法
    fn __priority_mut(&mut self) -> &mut Self::E;

    /// 设置优先级
    /// * 🚩现在统一输入值，[`Copy`]保证无需过于担心性能损失
    #[inline(always)]
    fn set_priority(&mut self, new_p: Self::E) {
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
    fn durability(&self) -> Self::E;
    /// 获取耐久度（可变）
    /// * 📌【2024-05-03 17:39:04】目前设置为内部方法
    fn __durability_mut(&mut self) -> &mut Self::E;

    /// 设置耐久度
    /// * 🚩现在统一输入值，[`Copy`]保证无需过于担心性能损失
    #[inline(always)]
    fn set_durability(&mut self, new_d: Self::E) {
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
    fn quality(&self) -> Self::E;
    /// 获取质量（可变）
    /// * 📌【2024-05-03 17:39:04】目前设置为内部方法
    fn __quality_mut(&mut self) -> &mut Self::E;

    /// 设置质量
    /// * 🚩现在统一输入值，[`Copy`]保证无需过于担心性能损失
    #[inline(always)]
    fn set_quality(&mut self, new_q: Self::E) {
        self.__quality_mut().set(new_q)
    }

    /// 模拟`BudgetValue.incPriority`
    ///
    /// # 📄OpenNARS
    /// Increase priority value by a percentage of the remaining range
    ///
    /// @param v The increasing percent
    #[inline(always)]
    fn inc_priority(&mut self, value: Self::E) {
        self.__priority_mut().inc(value)
    }

    /// 模拟`BudgetValue.decPriority`
    ///
    /// # 📄OpenNARS
    /// Decrease priority value by a percentage of the remaining range
    ///
    /// @param v The decreasing percent
    #[inline(always)]
    fn dec_priority(&mut self, value: Self::E) {
        self.__priority_mut().dec(value)
    }

    /// 模拟`BudgetValue.incDurability`
    ///
    /// # 📄OpenNARS
    ///
    /// Increase durability value by a percentage of the remaining range
    ///
    /// @param v The increasing percent
    #[inline(always)]
    fn inc_durability(&mut self, value: Self::E) {
        self.__durability_mut().inc(value)
    }

    /// 模拟`BudgetValue.decDurability`
    ///
    /// # 📄OpenNARS
    ///
    /// Decrease durability value by a percentage of the remaining range
    ///
    /// @param v The decreasing percent
    #[inline(always)]
    fn dec_durability(&mut self, value: Self::E) {
        self.__durability_mut().dec(value)
    }

    /// 模拟`BudgetValue.incQuality`
    ///
    /// # 📄OpenNARS
    ///
    /// Increase quality value by a percentage of the remaining range
    ///
    /// @param v The increasing percent
    #[inline(always)]
    fn inc_quality(&mut self, value: Self::E) {
        self.__quality_mut().inc(value)
    }

    /// 模拟`BudgetValue.decQuality`
    ///
    /// # 📄OpenNARS
    ///
    /// Decrease quality value by a percentage of the remaining range
    ///
    /// @param v The decreasing percent
    #[inline(always)]
    fn dec_quality(&mut self, value: Self::E) {
        self.__quality_mut().dec(value)
    }

    /// 模拟`BudgetValue.summary`
    /// * 🚩📜统一采用「几何平均值」估计（默认）
    ///
    /// # 📄OpenNARS
    ///
    /// To summarize a BudgetValue into a single number in [0, 1]
    #[inline(always)]
    fn summary(&self) -> Self::E {
        // 🚩三者几何平均值
        Self::E::geometrical_average([self.priority(), self.durability(), self.quality()])
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
    fn above_threshold(&self, budget_threshold: Self::E) -> bool {
        self.summary() >= budget_threshold
    }

    // * ❌【2024-05-02 00:52:02】不实现「仅用于 显示/呈现」的方法，包括所有的`toString` `toStringBrief`
}

/// 预算值的「具体类型」
/// * 🎯有选择地支持「限定的构造函数」
///   * 📄需要构造函数：预算函数中「创建新值的函数」
///   * 📄不要构造函数：具有「预算值属性」但【不可从预算值参数构造】的类型
///     * 📄概念[`super::Concept`]
///     * 📄任务[`super::Task`]
/// * 📌整个特征建立在「预算值就是预算值」，即「实现者本身**只有**p、d、q三元组」的基础上
/// * 🚩包括「构造函数」与「转换函数」
pub trait BudgetValueConcrete: Sized + BudgetValue {
    /// 内置构造函数(p, d, q)
    /// * 🚩直接从「短浮点」构造
    fn new(
        p: <Self as BudgetValue>::E,
        d: <Self as BudgetValue>::E,
        q: <Self as BudgetValue>::E,
    ) -> Self;

    /// 模拟 `BudgetValue` 构造函数(p, d, q)
    /// * 🚩将浮点数分别转换为「短浮点」
    ///
    /// # 📄OpenNARS `BudgetValue`
    ///
    /// Constructor with initialization
    ///
    /// @param p Initial priority
    /// @param d Initial durability
    /// @param q Initial quality
    #[inline(always)]
    fn from_float(p: Float, d: Float, q: Float) -> Self {
        Self::new(
            <Self as BudgetValue>::E::from_float(p),
            <Self as BudgetValue>::E::from_float(d),
            <Self as BudgetValue>::E::from_float(q),
        )
    }
}

/// 初代实现
mod impl_v1 {
    use super::*;

    /// 一个默认实现
    /// * 🔬仅作测试用
    #[derive(Debug, Default, Clone, Copy, PartialEq, Eq, Hash)]
    pub struct BudgetV1(ShortFloatV1, ShortFloatV1, ShortFloatV1);

    impl BudgetValue for BudgetV1 {
        // 指定为浮点数
        type E = ShortFloatV1;

        #[inline(always)]
        fn priority(&self) -> ShortFloatV1 {
            self.0 // * 🚩【2024-05-02 18:24:10】现在隐式`clone`
        }

        #[inline(always)]
        fn durability(&self) -> ShortFloatV1 {
            self.1 // * 🚩【2024-05-02 18:24:10】现在隐式`clone`
        }

        #[inline(always)]
        fn quality(&self) -> ShortFloatV1 {
            self.2 // * 🚩【2024-05-02 18:24:10】现在隐式`clone`
        }

        #[inline(always)]
        fn __priority_mut(&mut self) -> &mut ShortFloatV1 {
            &mut self.0
        }

        #[inline(always)]
        fn __durability_mut(&mut self) -> &mut ShortFloatV1 {
            &mut self.1
        }

        #[inline(always)]
        fn __quality_mut(&mut self) -> &mut ShortFloatV1 {
            &mut self.2
        }
    }

    impl BudgetValueConcrete for BudgetV1 {
        #[inline(always)]
        fn new(p: Self::E, d: Self::E, q: Self::E) -> Self {
            Self(p, d, q)
        }
    }

    impl std::fmt::Display for BudgetV1 {
        fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
            write!(f, "${}; {}; {}$", self.0, self.1, self.2)
        }
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
    type SF = <Budget as BudgetValue>::E;

    /// 快捷构造宏
    macro_rules! budget {
        // 三参数
        ($p:expr; $d:expr; $q:expr) => {
            Budget::from_float($p, $d, $q)
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

    /// 测试/inc_priority
    #[test]
    fn inc_priority() -> AResult {
        macro_once! {
            /// * 🚩模式：[预算值的构造方法] + 参数 ⇒ 预期「短浮点」浮点值
            macro test($( [ $($budget:tt)* ] + $delta:tt => $expected:tt)*) {
                $(
                    let mut t = budget!($($budget)*);
                    t.inc_priority(SF::from_float($delta));
                    // 修改后与所读值一致
                    assert_eq!(t.priority(), SF::from_float($expected));
                )*
            }
            [1.0; 0.9; 0.5] + 0.5 => 1.0
            [0.1; 0.9; 0.52] + 0.2 => (1.0 - (0.9 * 0.8))
            [0.5; 0.9; 0.54] + 0.8 => (1.0 - (0.5 * 0.2))
            [0.1024; 0.1; 0.75] + 0.0 => 0.1024
            [0.2; 0.1; 0.15] + 1.0 => 1.0
        }
        ok!()
    }

    /// 测试/dec_priority
    #[test]
    fn dec_priority() -> AResult {
        macro_once! {
            /// * 🚩模式：[预算值的构造方法] - 参数 ⇒ 预期「短浮点」浮点值
            macro test($( [ $($budget:tt)* ] - $delta:tt => $expected:tt)*) {
                $(
                    let mut t = budget!($($budget)*);
                    t.dec_priority(SF::from_float($delta));
                    // 修改后与所读值一致
                    assert_eq!(t.priority(), SF::from_float($expected));
                )*
            }
            [1.0; 0.9; 0.5] - 0.5 => 0.5
            [0.1; 0.9; 0.52] - 0.2 => (0.1 * 0.2)
            [0.5; 0.9; 0.54] - 0.8 => (0.5 * 0.8)
            [0.1024; 0.1; 0.75] - 0.0 => 0.0
            [0.2; 0.1; 0.15] - 1.0 => 0.2
        }
        ok!()
    }

    /// 测试/inc_durability
    #[test]
    fn inc_durability() -> AResult {
        macro_once! {
            /// * 🚩模式：[预算值的构造方法] + 参数 ⇒ 预期「短浮点」浮点值
            macro test($( [ $($budget:tt)* ] + $delta:tt => $expected:tt)*) {
                $(
                    let mut t = budget!($($budget)*);
                    t.inc_durability(SF::from_float($delta));
                    // 修改后与所读值一致
                    assert_eq!(t.durability(), SF::from_float($expected));
                )*
            }
            [0.9; 1.0; 0.5] + 0.5 => 1.0
            [0.9; 0.1; 0.52] + 0.2 => (1.0 - (0.9 * 0.8))
            [0.9; 0.5; 0.54] + 0.8 => (1.0 - (0.5 * 0.2))
            [0.1; 0.1024; 0.75] + 0.0 => 0.1024
            [0.1; 0.2; 0.15] + 1.0 => 1.0
        }
        ok!()
    }

    /// 测试/dec_durability
    #[test]
    fn dec_durability() -> AResult {
        macro_once! {
            /// * 🚩模式：[预算值的构造方法] - 参数 ⇒ 预期「短浮点」浮点值
            macro test($( [ $($budget:tt)* ] - $delta:tt => $expected:tt)*) {
                $(
                    let mut t = budget!($($budget)*);
                    t.dec_durability(SF::from_float($delta));
                    // 修改后与所读值一致
                    assert_eq!(t.durability(), SF::from_float($expected));
                )*
            }
            [0.9; 1.0; 0.5] - 0.5 => 0.5
            [0.9; 0.1; 0.52] - 0.2 => (0.1 * 0.2)
            [0.9; 0.5; 0.54] - 0.8 => (0.5 * 0.8)
            [0.1; 0.1024; 0.75] - 0.0 => 0.0
            [0.1; 0.2; 0.15] - 1.0 => 0.2
        }
        ok!()
    }

    /// 测试/inc_quality
    #[test]
    fn inc_quality() -> AResult {
        macro_once! {
            /// * 🚩模式：[预算值的构造方法] + 参数 ⇒ 预期「短浮点」浮点值
            macro test($( [ $($budget:tt)* ] + $delta:tt => $expected:tt)*) {
                $(
                    let mut t = budget!($($budget)*);
                    t.inc_quality(SF::from_float($delta));
                    // 修改后与所读值一致
                    assert_eq!(t.quality(), SF::from_float($expected));
                )*
            }
            [0.9; 0.5; 1.0] + 0.5 => 1.0
            [0.9; 0.52; 0.1] + 0.2 => (1.0 - (0.9 * 0.8))
            [0.9; 0.54; 0.5] + 0.8 => (1.0 - (0.5 * 0.2))
            [0.1; 0.75; 0.1024] + 0.0 => 0.1024
            [0.1; 0.15; 0.2] + 1.0 => 1.0
        }
        ok!()
    }

    /// 测试/dec_quality
    #[test]
    fn dec_quality() -> AResult {
        macro_once! {
            /// * 🚩模式：[预算值的构造方法] - 参数 ⇒ 预期「短浮点」浮点值
            macro test($( [ $($budget:tt)* ] - $delta:tt => $expected:tt)*) {
                $(
                    let mut t = budget!($($budget)*);
                    t.dec_quality(SF::from_float($delta));
                    // 修改后与所读值一致
                    assert_eq!(t.quality(), SF::from_float($expected));
                )*
            }
            [0.9; 0.5; 1.0] - 0.5 => 0.5
            [0.9; 0.52; 0.1] - 0.2 => (0.1 * 0.2)
            [0.9; 0.54; 0.5] - 0.8 => (0.5 * 0.8)
            [0.1; 0.75; 0.1024] - 0.0 => 0.0
            [0.1; 0.15; 0.2] - 1.0 => 0.2
        }
        ok!()
    }

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
}
