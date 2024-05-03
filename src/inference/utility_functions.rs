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
//! * ✅【2024-05-03 19:28:13】基本完成所有单元测试

use crate::entity::ShortFloat;
use crate::global::Float;
use crate::nars::DEFAULT_PARAMETERS;
use nar_dev_utils::pipe;

/// 【派生】用于「短浮点」的实用方法
///
/// # 📄OpenNARS `nars.inference.UtilityFunctions`
///
/// Common functions on real numbers, mostly in [0,1].
pub trait UtilityFunctions: ShortFloat {
    // * 🚩现在直接使用[`ShortFloat`]基于的[`std::ops::Not`]特征
    // /// 🆕扩展逻辑「非」
    // /// * 📄这个在OpenNARS中直接用`1 - v`表示了，但此处仍然做出抽象
    // /// * 📝在使用了`Copy`并且是「按值传参」的情况下，才可省略[`clone`](Clone::clone)
    // ///   * ⚠️要分清是在「拷贝值」还是在「拷贝引用」
    // #[inline(always)]
    // fn not(self) -> Self {
    //     // Self::one() - self
    //     !self
    // }

    /// 模拟`UtilityFunctions.and`
    /// * 🚩扩展逻辑「与」
    /// * 🚩现在直接使用[`ShortFloat`]基于的[`std::ops::BitAnd`]特征
    ///
    /// # 📄OpenNARS
    ///
    /// A function where the output is conjunctively determined by the inputs
    ///
    ///  @param arr The inputs, each in [0, 1]
    ///  @return The output that is no larger than each input
    #[inline(always)]
    fn and(self, value: Self) -> Self {
        self & value
    }

    /// 🆕多个值相与
    /// * 🚩直接派生自「两个值相与」
    ///   * 📝「扩展逻辑与」遵循交换律和结合律
    fn and_multi(values: impl IntoIterator<Item = Self>) -> Self {
        values
            // 逐个迭代值的迭代器
            .into_iter()
            // 从「1」开始不断取「与」
            .fold(Self::ONE, Self::and)
    }

    /// 模拟`UtilityFunctions.or`
    /// * 🚩扩展逻辑「或」
    /// * 🚩【2024-05-02 17:53:22】利用德摩根律行事
    ///   * 💭可能会有性能损失
    /// * 🚩现在直接使用[`ShortFloat`]基于的[`std::ops::BitOr`]特征
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
        // pipe! {
        //     // 非
        //     self.not()
        //     // 与
        //     => .and(value.not())
        //     // 非
        //     => .not()
        // }
        self | value
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
    /// * ⚠️迭代器不能为空
    ///
    /// # 📄OpenNARS
    ///
    /// A function where the output is the arithmetic average the inputs
    ///
    /// @param arr The inputs, each in [0, 1]
    /// @return The arithmetic average the inputs
    #[doc(alias = "ave_ari")]
    fn arithmetical_average(values: impl IntoIterator<Item = Self>) -> Self {
        // * 💭【2024-05-02 00:44:41】大概会长期存留，因为与「真值函数」无关而无需迁移
        /* 📄OpenNARS源码：
        float product = 1;
        for (float f : arr) {
            product *= f;
        }
        return (float) Math.pow(product, 1.00 / arr.length); */
        let mut sum: Float = 0.0;
        let mut len: usize = 0;
        for v in values.into_iter() {
            sum += v.to_float(); // 转换为浮点并追加 | 因此不担心溢出
            len += 1; // 与此同时，计数
        }
        Self::from_float(sum / len as Float)
        // * 🚩【2024-05-03 12:50:23】边遍历边计数，就能解决问题
        // pipe! {
        //     values
        //     // 逐个迭代值的迭代器
        //     => .iter()
        //     // ! 必须先转换为浮点数：连续加和会越界
        //     => .map(Self::to_float)
        //     // 所有值的和（从`1`开始）
        //     => {.sum::<Float>()}#
        //     // 除以值的个数
        //     => .div(values.len() as Float)
        //     // 转换回「短浮点」（保证不越界）
        //     => Self::from_float
        // }
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
    fn geometrical_average(values: impl IntoIterator<Item = Self>) -> Self {
        // * 💭【2024-05-02 00:44:41】大概会长期存留，因为与「真值函数」无关而无需迁移
        /* 📄OpenNARS源码：
        float product = 1;
        for (float f : arr) {
            product *= f;
        }
        return (float) Math.pow(product, 1.00 / arr.length); */
        let mut product: Float = 1.0;
        let mut len: usize = 0;
        for v in values.into_iter() {
            product *= v.to_float(); // 转换为浮点并追加
            len += 1; // 与此同时，计数
        }
        // 因为乘法在0~1封闭，故无需顾忌panic
        Self::from_float(product.powf(1.0 / len as Float))
        // * ❌【2024-05-03 12:51:44】弃用下述代码：在数值过小时会引发精度丢失
        /* [src\inference\utility_functions.rs:446:52] [sf1, sf2] = [
            ShortFloatV1 {
                value: 3,
            },
            ShortFloatV1 {
                value: 3,
            },
        ]
        thread 'inference::utility_functions::tests::geometrical_average' panicked at src\inference\utility_functions.rs:448:13:
        assertion `left == right` failed
          left: ShortFloatV1 { value: 0 }
         right: ShortFloatV1 { value: 3 } */
        // values
        //     // 逐个迭代值的迭代器
        //     .iter()
        //     .cloned()
        //     // 所有值的乘积（从`1`开始）
        //     .fold(Self::one(), Self::mul)
        //     // 值个数次开根
        //     .root(values.len())
    }

    /// 从真值的「w值」到「c值」
    /// * 📄超参数`Parameters.HORIZON`参见[`crate::nars::Parameters`]
    ///
    /// # 📄OpenNARS
    ///
    /// A function to convert weight to confidence
    ///
    /// @param w Weight of evidence, a non-negative real number
    /// @return The corresponding confidence, in [0, 1)
    fn w2c(w: Float) -> Self {
        /* 📄OpenNARS源码：
        return w / (w + Parameters.HORIZON); */
        Self::from_float(w / (w + DEFAULT_PARAMETERS.horizon))
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
    fn c2w(&self) -> Float {
        /* 📄OpenNARS源码：
        return Parameters.HORIZON * c / (1 - c); */
        let c = self.to_float();
        DEFAULT_PARAMETERS.horizon * c / (1.0 - c)
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
        self.set(*self | value)
    }

    /// 🆕「减少」值
    /// * 🎯用于（统一）OpenNARS`incPriority`系列方法
    /// * 📝核心逻辑：自己的值和对面取「与」，越取越少
    /// * ❓【2024-05-02 00:31:19】是否真的要放到这儿来，在「数据结构定义」中引入「真值函数」的概念
    fn dec(&mut self, value: Self) {
        // self.set(UtilityFunctions.and(priority.getValue(), v));
        self.set(*self & value)
    }

    /// 🆕「最大值合并」
    /// * 🎯用于（统一）OpenNARS`merge`的重复调用
    /// * 🚩现在已经在[「短浮点」](EvidenceReal)中要求了[`Ord`]
    /// * 📝【2024-05-03 14:55:29】虽然现在「预算函数」以「直接创建新值」为主范式，
    ///   * 但在用到该函数的`merge`方法上，仍然是「修改」语义——需要可变引用
    fn max_from(&mut self, other: Self) {
        let max = (*self).max(other);
        self.set(max);
    }
}

/// 直接自动实现，附带所有默认方法
impl<T: ShortFloat> UtilityFunctions for T {}

// ! 对标准库方法的实现受到「孤儿规则」的阻碍

#[cfg(test)]
mod tests {
    use super::*;
    use crate::entity::ShortFloatV1;
    use anyhow::Result;
    use nar_dev_utils::{asserts, for_in_ifs, macro_once};

    /// 定义要测试的「短浮点」类型
    type SF = ShortFloatV1;

    /// 健壮性测试所用到的「测试精度」
    /// * 🎯尽可能多地遍历「短浮点」的所有可能情形
    /// * 🚩测试的案例量
    /// * 🕒2000不到0.5s，5000大约1s，10000要大约7s
    const N: usize = 4000;
    const N_FLOAT: Float = N as Float;

    /// 快捷构造宏
    macro_rules! sf {
        // 0、1、0.5 特殊映射
        (0) => {
            SF::ZERO
        };
        (1) => {
            SF::ONE
        };
        (HALF) => {
            SF::HALF
        };
        (1/2) => {
            SF::HALF
        };
        // 值映射
        ($float:expr) => {
            SF::from_float($float)
        };
    }

    /// 以一定数目遍历从0到1的所有「短浮点」
    /// * 🚩用到常量[`N`]与[`N_FLOAT`]
    fn all_sf() -> impl Iterator<Item = SF> {
        (0..=N).map(|v| sf!(v as Float / N_FLOAT))
    }

    /// 海测/快捷遍历所有「短浮点」（所有组合）
    macro_rules! for_all_sf {
        ( ( $($var:pat $(if $cond:expr)?),* $(,)? ) => $($code:tt)* ) => {
            for_in_ifs! {
                // 遍历时要执行的代码
                { $($code)* }
                // 遍历范围
                $( for $var in (all_sf()) $(if ($cond))? )*
            }
        };
    }

    /// 测试/and
    #[test]
    fn and() -> Result<()> {
        // 海测（健壮性测试） | 🎯确保正常值不会panic
        for_all_sf! {
            (sf1, sf2) =>
            // 直接计算
            let _ = sf1 & sf2;
        }
        // 例侧（案例测试）
        macro_once! {
            /// * 🚩模式：值1 & 值2 ⇒ 预期
            macro test($($f1:tt & $f2:tt => $expected:tt)*) {
                asserts! {
                    $(
                        sf!($f1) & sf!($f2) => sf!($expected)
                    )*
                }
            }
            // 0、1
            0 & 0 => 0
            0 & 1 => 0
            1 & 0 => 0
            1 & 1 => 1
            // 1：幺元
            1 & 0.1 => 0.1
            1 & 0.2 => 0.2
            1 & 0.3 => 0.3
            1 & 0.4 => 0.4
            1 & 0.5 => 0.5
            1 & 0.6 => 0.6
            1 & 0.7 => 0.7
            1 & 0.8 => 0.8
            1 & 0.9 => 0.9
            // 0：零元
            0 & 0.1 => 0
            0 & 0.2 => 0
            0 & 0.3 => 0
            0 & 0.4 => 0
            0 & 0.5 => 0
            0 & 0.6 => 0
            0 & 0.7 => 0
            0 & 0.8 => 0
            0 & 0.9 => 0
            // 乘法语义
            0.5 & 0.5 => 0.25
        }
        Ok(())
    }

    /// 测试/and_multi
    #[test]
    fn and_multi() -> Result<()> {
        // 海测（健壮性测试） // * 🚩验证与二元运算的逻辑一致
        for_all_sf! {
            (sf1, sf2) =>
            // 直接计算
            assert_eq!(sf1 & sf2, SF::and_multi([sf1, sf2]));
        }
        // * 🚩验证多元运算的正常结果（乘方）
        let mut sfs = Vec::new();
        let v = 0.9;
        for n in 1..=4 {
            // ! ❌【2024-05-03 12:42:37】对零次幂处理不善：1.0🆚0.9，但OpenNARS中不会用到
            // ! ⚠️【2024-05-03 12:39:56】目前对五次及以上会有微弱不一致：5904🆚5905
            sfs.push(sf!(v));
            let multi = SF::and_multi(sfs.iter().cloned());
            let pow = sf!(v.powi(n));
            assert_eq!(multi, pow);
        }
        // 例侧（案例测试）
        macro_once! {
            /// * 🚩模式：值1 & 值2 & 值3 & ...;
            macro test($( $($f:tt)&* ;)*) {
                asserts! {
                    $(
                        $(sf!($f))&* => SF::and_multi([$(sf!($f)),*])
                    )*
                }
            }
            // 0、1 & 二元、三元（最常见即如此）
            0 & 0;
            0 & 1;
            1 & 0;
            1 & 1;
            0 & 0 & 0;
            0 & 0 & 1;
            0 & 1 & 0;
            0 & 1 & 1;
            1 & 0 & 0;
            1 & 0 & 1;
            1 & 1 & 0;
            1 & 1 & 1;
            // 0.5的幂次
            0.5;
            0.5 & 0.5;
            0.5 & 0.5 & 0.5;
            0.5 & 0.5 & 0.5 & 0.5;
            0.5 & 0.5 & 0.5 & 0.5 & 0.5;
            0.5 & 0.5 & 0.5 & 0.5 & 0.5 & 0.5;
        }
        Ok(())
    }

    /// 测试/or
    #[test]
    fn or() -> Result<()> {
        // 海测（健壮性测试） | 🎯确保正常值不会panic
        for_all_sf! {
            (sf1, sf2) =>
            // 直接计算
            let _ = sf1 | sf2;
        }
        // 例侧（案例测试）
        macro_once! {
            /// * 🚩模式：值1 | 值2 ⇒ 预期
            macro test($($f1:tt | $f2:tt => $expected:tt)*) {
                asserts! {
                    $(
                        sf!($f1) | sf!($f2) => sf!($expected)
                    )*
                }
            }
            // 0、1
            0 | 0 => 0
            0 | 1 => 1
            1 | 0 => 1
            1 | 1 => 1
            // 1：零元
            1 | 0.1 => 1
            1 | 0.2 => 1
            1 | 0.3 => 1
            1 | 0.4 => 1
            1 | 0.5 => 1
            1 | 0.6 => 1
            1 | 0.7 => 1
            1 | 0.8 => 1
            1 | 0.9 => 1
            // 0：幺元
            0 | 0.1 => 0.1
            0 | 0.2 => 0.2
            0 | 0.3 => 0.3
            0 | 0.4 => 0.4
            0 | 0.5 => 0.5
            0 | 0.6 => 0.6
            0 | 0.7 => 0.7
            0 | 0.8 => 0.8
            0 | 0.9 => 0.9
            // 德摩根 乘法语义
            0.5 | 0.5 => 0.75
        }
        Ok(())
    }

    /// 测试/or_multi
    #[test]
    fn or_multi() -> Result<()> {
        // 海测（健壮性测试） // * 🚩验证与二元运算的逻辑一致
        for_all_sf! {
            (sf1, sf2) =>
            // 直接计算
            assert_eq!(sf1 | sf2, SF::or_multi([sf1, sf2]));
        }
        // 例侧（案例测试）
        macro_once! {
            /// * 🚩模式：值1 | 值2 | 值3 | ...;
            macro test($( $($f:tt)|* ;)*) {
                asserts! {
                    $(
                        $(sf!($f))|* => SF::or_multi([$(sf!($f)),*])
                    )*
                }
            }
            // 0、1 | 二元、三元（最常见即如此）
            0 | 0;
            0 | 1;
            1 | 0;
            1 | 1;
            0 | 0 | 0;
            0 | 0 | 1;
            0 | 1 | 0;
            0 | 1 | 1;
            1 | 0 | 0;
            1 | 0 | 1;
            1 | 1 | 0;
            1 | 1 | 1;
            // 0.5的幂次
            0.5;
            0.5 | 0.5;
            0.5 | 0.5 | 0.5;
            0.5 | 0.5 | 0.5 | 0.5;
            0.5 | 0.5 | 0.5 | 0.5 | 0.5;
            0.5 | 0.5 | 0.5 | 0.5 | 0.5 | 0.5;
        }
        Ok(())
    }

    /// 测试/arithmetical_average
    #[test]
    fn arithmetical_average() -> Result<()> {
        // * 🚩验证与浮点运算的逻辑一致
        for_all_sf! {
            (sf1, sf2) =>
            // 直接计算
            let ave_ari = SF::arithmetical_average([sf1 ,sf2]);
            let float_ari = sf!((sf1.to_float() + sf2.to_float()) / 2.0);
            assert_eq!(ave_ari, float_ari);
        }
        Ok(())
    }

    /// 测试/geometrical_average
    #[test]
    fn geometrical_average() -> Result<()> {
        // * 🚩验证与浮点运算的逻辑一致
        for_all_sf! {
            (sf1, sf2) =>
            // 直接计算
            let ave_geo = SF::geometrical_average([sf1 ,sf2]);
            let float_geo = sf!((sf1.to_float() * sf2.to_float()).sqrt());
            assert_eq!(ave_geo, float_geo);
        }
        Ok(())
    }

    /// 测试/w2c
    #[test]
    fn w2c() -> Result<()> {
        // * 🚩验证与浮点运算的逻辑一致
        const N: usize = 1000;
        for w in 0..=N {
            let w = w as Float;
            let k = DEFAULT_PARAMETERS.horizon;
            let c = SF::w2c(w);
            // ! ⚠️【2024-05-03 19:18:14】与`1 - k / (w + k)`有微小不一致：0.0063🆚0.0062
            assert_eq!(c, sf!(w / (w + k)))
        }
        Ok(())
    }

    /// 测试/c2w
    #[test]
    fn c2w() -> Result<()> {
        // * 🚩验证与浮点运算的逻辑一致
        for_all_sf! {
            // * 📌「1」会导致「除以零」溢出
            (c if !c.is_one()) =>
                let k = DEFAULT_PARAMETERS.horizon;
                let w = c.c2w();
                let c = c.to_float();
                // ! ⚠️【2024-05-03 19:18:14】与`1 - k / (w + k)`有微小不一致：0.0063🆚0.0062
                assert_eq!(w, c * k / (1.0 - c))
        }
        Ok(())
    }

    /// 测试/inc
    #[test]
    fn inc() -> Result<()> {
        // * 🚩验证与逻辑运算的结果一致
        for_all_sf! {
            (mut sf1, sf2) =>
            let expected = sf1 | sf2;
            sf1.inc(sf2);
            assert_eq!(sf1, expected);
        }
        Ok(())
    }

    /// 测试/dec
    #[test]
    fn dec() -> Result<()> {
        // * 🚩验证与逻辑运算的结果一致
        for_all_sf! {
            (mut sf1, sf2) =>
            let expected = sf1 & sf2;
            sf1.dec(sf2);
            assert_eq!(sf1, expected);
        }
        Ok(())
    }

    /// 测试/max_from
    #[test]
    fn max_from() -> Result<()> {
        // * 🚩验证与最大值运算的结果一致
        for_all_sf! {
            (mut sf1, sf2) =>
            let expected = sf1.max(sf2);
            sf1.max_from(sf2);
            assert_eq!(sf1, expected);
        }
        Ok(())
    }
}
