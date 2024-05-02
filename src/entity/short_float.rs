//! 复刻OpenNARS `nars.entity.ShortFloat`
//! * 🚩使用`u16`完整覆盖

use crate::global::Float;
use thiserror::Error;

/// 用作「短浮点」的整数类型
/// * 🚩使用0~65536的「十六位无符号整数」覆盖`0~10000`
type UShort = u16;

/// 用作「短浮点」的范围上界
/// * 🚩表示区间`0~10000`
const SHORT_MAX: UShort = 10000;

/// 用作「整数→浮点」的转换倍率
/// * 🚩【2024-05-02 09:27:03】目前相当于「直接除以一万」
const MULTIPLIER_TO_FLOAT: Float = 0.0001;

/// 用作「浮点→整数」的转换倍率
/// * 🚩【2024-05-02 09:27:03】目前相当于「直接乘以一万」
const MULTIPLIER_TO_UINT: Float = 10000.0;

// TODO: 【2024-05-02 00:58:31】对标复刻内容
/// 模拟OpenNARS `nars.entity.ShortFloat`
/// * 🚩使用`u16`0~65536的范围覆盖
///
/// # 📄OpenNARS
///
/// A float value in [0, 1], with 4 digits accuracy.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub struct ShortFloat {
    /// 0~65536的「实际值」
    ///
    /// # 📄OpenNARS
    ///
    /// To save space, the values are stored as short integers (-32768 to 32767, only
    /// 0 to 10000 used),
    /// but used as float
    value: UShort,
}

/// 用于表示「短浮点」可能产生的错误
#[derive(Debug, Clone, Error)]
pub enum ShortFloatError {
    #[error("value out of range: {0}")]
    OutOfRange(Float),
}

impl ShortFloat {
    /// 以0~10000的整数创建
    #[inline(always)]
    pub fn new(value: UShort) -> Self {
        Self { value }
    }

    /// 模拟OpenNARS`getValue`
    ///
    /// # 📄OpenNARS
    ///
    /// To access the value as float
    ///
    /// @return The current value in float
    /// * 🚩获取浮点值
    #[inline(always)]
    pub fn value(&self) -> Float {
        self.value as Float * MULTIPLIER_TO_FLOAT
    }

    /// 🆕判断浮点数是否在范围内
    /// * 📝判断「是否在范围外」直接使用「不在范围内」的逻辑
    ///   * 📄clippy提示「manual `!RangeInclusive::contains` implementation」
    /// * ✅对`NaN`会默认返回`false`，故无需担心
    #[inline(always)]
    pub fn is_in_range(value: Float) -> bool {
        (0.0..=1.0).contains(&value)
    }

    /// 模拟OpenNARS`ShortFloat.setValue`
    /// * 🚩设置浮点值（有检查）
    pub fn set_value(&mut self, value: Float) -> Result<(), ShortFloatError> {
        // 转换、检查并设置值
        self.value = Self::float_to_short_value(value)?;
        // 返回
        Ok(())
    }

    /// 🆕设置浮点值（无检查）
    /// * ⚠️必须确保值在范围内
    ///
    /// # 📄OpenNARS
    ///
    /// Set new value, rounded, with validity checking
    ///
    /// @param v The new value
    #[inline(always)]
    pub fn set_value_unchecked(&mut self, value: Float) {
        self.value = Self::float_to_short_value_unchecked(value)
    }

    /// 🆕浮点转换为「短整数」（有检查）
    /// * 🎯提取共用逻辑，以同时用于「构造」和「赋值」
    /// * ✅无需考虑「NaN」「无限」等值：[`Self::is_in_range`]会自动判断
    pub fn float_to_short_value(value: Float) -> Result<UShort, ShortFloatError> {
        match Self::is_in_range(value) {
            // 检查通过⇒转换值
            true => Ok(Self::float_to_short_value_unchecked(value)),
            // 检查不通过⇒返回错误
            false => Err(ShortFloatError::OutOfRange(value)),
        }
    }

    /// 🆕浮点转换为「短整数」（无检查）
    /// * 🎯提取共用逻辑，以同时用于「构造」和「赋值」
    /// * ⚠️必须确保值在范围内
    pub fn float_to_short_value_unchecked(value: Float) -> UShort {
        (value * MULTIPLIER_TO_UINT).round() as UShort
    }

    // ! ✅对`equals`、`hashCode`、`clone`均已通过宏自动生成
}

impl std::fmt::Display for ShortFloat {
    #[inline]
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        // 对`1`的特别处理
        if self.value == SHORT_MAX {
            return write!(f, "1.0000");
        }
        // 自身值转换为字符串
        let value_s = self.value.to_string();
        // 左边补0到四位
        let pad_0_s = "0".repeat(4 - value_s.len());
        // 格式化
        write!(f, "0.{pad_0_s}{value_s}")
    }
}

/// 实现「从浮点到『短浮点』的直接转换」
/// 🚩直接通过「构造函数+尝试转换」实现
impl TryFrom<Float> for ShortFloat {
    type Error = ShortFloatError;

    #[inline]
    fn try_from(value: Float) -> Result<Self, Self::Error> {
        Ok(Self::new(Self::float_to_short_value(value)?))
    }
}

/// 单元测试
#[cfg(test)]
mod tests {
    use super::*;
    use anyhow::Result;
    use nar_dev_utils::macro_once;

    /// 📜默认浮点判等精度：1e-6
    /// * 🎯解决「浮点判等」因精度不够失效的问题
    const DEFAULT_EPSILON: Float = 1.0E-6;

    /// 断言约等
    /// * 🎯解决「浮点判等」因精度不够失效的问题
    macro_rules! assert_approx_eq {
        // * 🚩模式：@精度 值1, 值2
        ($epsilon:expr; $v1:expr, $v2:expr) => {
            assert!(
                ($v1 - $v2).abs() < $epsilon,
                "{} !≈ {} @ {}",
                $v1,
                $v2,
                $epsilon
            )
        };
        ($v1:expr, $v2:expr) => {
            assert_approx_eq!(DEFAULT_EPSILON; $v1, $v2)
        };
    }

    /// 测试/new
    #[test]
    fn new() -> Result<()> {
        macro_once! {
            // * 🚩模式：短整数（作为构造函数参数）
            macro test($( $short:expr )*) {
                $(
                    let _ = ShortFloat::new($short);
                )*
            }
            0
            10000
            90
            9000
            1024
            8192
        }
        Ok(())
    }

    /// 测试/value
    #[test]
    fn value() -> Result<()> {
        macro_once! {
            // * 🚩模式：短整数（构造用）⇒预期值
            macro test($( $short:expr => $expected:expr )*) {
                $(
                    let sf = ShortFloat::new($short);
                    // ! ⚠️此处必须使用「约等」判断，否则会出现`0.009 != 0.009000000000000001`的情形
                    assert_approx_eq!(sf.value(), $expected);
                )*
            }
            0 => 0.0
            10000 => 1.0
            90 => 0.009
            9000 => 0.9
            1024 => 0.1024
            8192 => 0.8192
        }
        Ok(())
    }

    /// 测试/is_in_range
    #[test]
    fn is_in_range() -> Result<()> {
        Ok(())
    }

    /// 测试/set_value
    #[test]
    fn set_value() -> Result<()> {
        use ShortFloatError::*;
        macro_once! {
            // * 🚩模式：短整数（构造用） -> 浮点数（赋值用）⇒预期值（短整数） @ 返回的模式
            macro test($( $short:literal -> $float:expr => $expected:literal @ $pattern:pat)*) {
                $(
                    let mut sf = ShortFloat::new($short);
                    let result = sf.set_value($float);
                    // 检查返回值
                    assert_eq!(sf.value, $expected);
                    assert!(matches!(result, $pattern));
                )*
            }
            // 正常赋值
            0     -> 0.0                 => 0     @ Ok(..)
            0     -> 1.0                 => 10000 @ Ok(..)
            0     -> 0.009               => 90    @ Ok(..)
            0     -> 0.9                 => 9000  @ Ok(..)
            0     -> 0.1024              => 1024  @ Ok(..)
            0     -> 0.8192              => 8192  @ Ok(..)
            // 四舍五入
            0     -> 0.00001             => 0     @ Ok(..)
            0     -> 0.00002             => 0     @ Ok(..)
            0     -> 0.00003             => 0     @ Ok(..)
            0     -> 0.00004             => 0     @ Ok(..)
            0     -> 0.00005             => 1     @ Ok(..)
            0     -> 0.00006             => 1     @ Ok(..)
            0     -> 0.00007             => 1     @ Ok(..)
            0     -> 0.00008             => 1     @ Ok(..)
            0     -> 0.00009             => 1     @ Ok(..)
            // 异常赋值：超出范围
            0     -> -0.1                => 0     @ Err(OutOfRange(..))
            10000 ->  2.0                => 10000 @ Err(OutOfRange(..))
            10000 -> Float::INFINITY     => 10000 @ Err(OutOfRange(..))
            0     -> Float::NEG_INFINITY => 0     @ Err(OutOfRange(..))
            // 异常赋值：无效值
            10000 -> Float::NAN          => 10000 @ Err(..)
        }
        Ok(())
    }

    /// 测试/set_value_unchecked
    #[test]
    fn set_value_unchecked() -> Result<()> {
        macro_once! {
            // * 🚩模式：短整数（构造用） -> 浮点数（赋值用）⇒预期值（短整数）
            macro test($( $short:literal -> $float:expr => $expected:literal)*) {
                $(
                    let mut sf = ShortFloat::new($short);
                    sf.set_value_unchecked($float);
                    // 检查返回值
                    assert_eq!(sf.value, $expected, "设置值`{sf:?} -> {}`不符预期`{}`", $float, $expected);
                )*
            }
            // 异常值仍可以赋值 | ⚠️负值会重置为`0`
            0     -> 1.0001              => 10001
            0     -> 2.0                 => 20000
            0     -> 6.5535              => 65535
            0     -> -0.1                => 0
            0     -> -2.0                => 0
            // 异常值正常四舍五入
            0     -> 1.00001             => 10000
            0     -> 1.00002             => 10000
            0     -> 1.00003             => 10000
            0     -> 1.00004             => 10000
            0     -> 1.00005             => 10001
            0     -> 1.00006             => 10001
            0     -> 1.00007             => 10001
            0     -> 1.00008             => 10001
            0     -> 1.00009             => 10001
            // 无穷值会被重置为 最大/最小 值：正无穷⇒最大，负无穷⇒最小
            0     -> Float::INFINITY     => 65535
            10000 -> Float::NEG_INFINITY => 0
            // NaN会被重置为`0`
            10000 -> Float::NAN          => 0
        }
        Ok(())
    }

    // 测试/float_to_short_value
    // * ✅已在`set_value`中连带测试过

    // 测试/float_to_short_value_unchecked
    // * ✅已在`set_value`中连带测试过

    /// 测试/fmt
    #[test]
    fn fmt() -> Result<()> {
        macro_once! {
            // * 🚩模式：短整数（构造用） => 预期值（字符串）
            macro test($( $short:expr => $expected:expr)*) {
                $(
                    let mut sf = ShortFloat::new($short);
                    let formatted = format!("{sf}");
                    // 检查返回值
                    assert_eq!(formatted, $expected);
                )*
            }
            // 1
            10000 => "1.0000"
            // 正常
            1024  => "0.1024"
            8192  => "0.8192"
            // 不足位补全
            0     => "0.0000"
            90    => "0.0090"
            900   => "0.0900"
        }
        Ok(())
    }

    /// 测试/try_from
    #[test]
    fn try_from() -> Result<()> {
        use ShortFloatError::*;
        macro_once! {
            // * 🚩模式：浮点数（转换用）⇒预期值（短整数） @ 返回的模式
            macro test($( $float:expr => $pattern:pat)*) {
                $(
                    // 尝试转换
                    let mut result: Result<ShortFloat, ShortFloatError> = $float.try_into();
                    // 检查返回值（兼检查转换结果）
                    assert!(matches!(result, $pattern));
                )*
            }
            // 正常转换
            0.0                 => Ok(ShortFloat {value: 0})
            1.0                 => Ok(ShortFloat {value: 10000})
            0.009               => Ok(ShortFloat {value: 90})
            0.9                 => Ok(ShortFloat {value: 9000})
            0.1024              => Ok(ShortFloat {value: 1024})
            0.8192              => Ok(ShortFloat {value: 8192})
            // 四舍五入
            0.00001             => Ok(ShortFloat {value: 0})
            0.00002             => Ok(ShortFloat {value: 0})
            0.00003             => Ok(ShortFloat {value: 0})
            0.00004             => Ok(ShortFloat {value: 0})
            0.00005             => Ok(ShortFloat {value: 1})
            0.00006             => Ok(ShortFloat {value: 1})
            0.00007             => Ok(ShortFloat {value: 1})
            0.00008             => Ok(ShortFloat {value: 1})
            0.00009             => Ok(ShortFloat {value: 1})
            // 异常转换：超出范围
            -0.1                => Err(OutOfRange(..))
             2.0                => Err(OutOfRange(..))
            Float::INFINITY     => Err(OutOfRange(..))
            Float::NEG_INFINITY => Err(OutOfRange(..))
            // 异常转换：无效值
            Float::NAN          => Err(..)
        }
        Ok(())
    }
}
