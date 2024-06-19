//! 🎯复刻OpenNARS `nars.entity.ShortFloat`
//! * 🚩核心逻辑：一个前提，多个派生，多方聚合
//!   * 前提：通过实现[`EvidenceReal`]得到「基本操作」
//!   * 派生：通过实现各类`XXXFunctions`得到「派生操作」
//!   * 聚合：通过统一的「自动实现」得到「所有操作汇聚于一体」的静态功能增强（真值函数@数值）
//!     * 📝Rust允许「在外部调用『看似没有实现派生操作的结构』时，允许使用『自动实现了的派生操作』」
//! * 🕒最后更新：【2024-06-19 23:36:50】
//!
//! * ✅【2024-05-02 21:41:48】（初代实现）基本复刻完毕
//! * ♻️【2024-06-19 23:36:56】删繁就简：删去「抽象特征」以免去后续诸多泛型引入的代码复杂性

use crate::{global::Float, impl_display_from_to_display, ToDisplayAndBrief};
use narsese::api::EvidentNumber;
use std::ops::{BitAnd, BitOr, Not};
use thiserror::Error;

/// 用作「短浮点」的整数类型
/// * 🚩使用0~4294967296的「三十二位无符号整数」覆盖`0~10000`与（相乘时的）`0~100000000`
/// * 🎯在「短浮点乘法」处避免重复的`as`转换（以提升性能⚡）
///   * 📄【2024-05-02 11:38:12】总测试时间从原先`(3.5+x)s`变为`3.23s`（用空间换时间后）
type UShort = u32;

/// 用作「短浮点」的范围上界
/// * 🚩表示区间`0~10000`
const SHORT_MAX: UShort = 10000;

/// 用作「整数→浮点」的转换倍率
/// * 🚩【2024-05-02 09:27:03】目前相当于「直接除以一万」
const MULTIPLIER_TO_FLOAT: Float = 0.0001;

/// 用作「浮点→整数」的转换倍率
/// * 🚩【2024-05-02 09:27:03】目前相当于「直接乘以一万」
const MULTIPLIER_TO_UINT: Float = 10000.0;

/// 模拟`nars.entity.ShortFloat`
/// * 🚩使用`u32`0~4294967296的范围覆盖`0~10000²`
/// * ✨原生支持四则运算
/// * 🎯在基本的[「证据数」](EvidentNumber)基础上，添加更多NAL细节功能
///   * 📄原[`nars.inference.UtilityFunctions`](crate::inference::UtilityFunctions)的「扩展逻辑与或非」
/// * 🚩【2024-05-02 16:05:04】搬迁自[`crate::entity::BudgetValue`]
/// * 🚩【2024-05-02 17:48:30】现在全部抛弃基于「不可变引用」的运算
///   * ⚠️混合「传可变引用」和「直接传值」的代码将过于冗杂（并且造成接口不统一）
///   * 📌在实现了[`Copy`]之后，将值的复制看作是「随处可用」的
/// * 🚩【2024-05-03 11:11:48】现在将其概念与「短浮点」合并
///
/// ## ⚠️与OpenNARS不同的一点：浮点舍入问题
///
/// !📝OpenNARS的实现是「四舍五入」，而NARust的实现是「向下截断」
/// * ❗即便在构造时采用了[`Float::round`]，但实际效果仍然与OpenNARS不同
///   * ⚡为性能考量，许多运算最后的舍入操作仍然是四舍五入（整数除法，避免转换为浮点）
/// * 📄这导致`0.1 * 0.0005`在OpenNARS中等于`0.0001`而在NARust中为`0`
///
/// OpenNARS中可行的推理：
///
/// ```plaintext
/// IN: <A --> B>. %1.00;0.10% {6 : 3}
/// IN: <B --> C>. %1.00;0.01% {6 : 4}
/// 1
/// OUT: <A --> C>. %1.00;0.00% {7 : 4;3}
/// OUT: <C --> A>. %1.00;0.00% {7 : 4;3}
/// ```
///
/// # 📄OpenNARS
///
/// A float value in [0, 1], with 4 digits accuracy.
#[derive(Debug, Default, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct ShortFloat {
    /// 0~4294967296的「实际值」
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
/// 符合[`Result`]的「短浮点结果」
pub type ShortFloatResult = Result<ShortFloat, ShortFloatError>;

impl ShortFloat {
    /// 常量「0」
    pub const ZERO: Self = Self::new_unchecked(0);

    /// 常量「1」
    pub const ONE: Self = Self::new_unchecked(SHORT_MAX);

    /// 常量「1/2」
    pub const HALF: Self = Self::new_unchecked(SHORT_MAX / 2);

    /// 以0~10000的整数创建（有检查）
    #[inline(always)]
    pub fn new(value: UShort) -> Result<Self, ShortFloatError> {
        Self::new_unchecked(value).validate()
    }

    /// 以0~10000的整数创建（无检查）
    /// * ⚠️部分封闭：仅对[`crate::entity`]模块开放
    pub(super) const fn new_unchecked(value: UShort) -> Self {
        Self { value }
    }

    /// 🆕判断浮点数是否在范围内
    /// * 📝判断「是否在范围外」直接使用「不在范围内」的逻辑
    ///   * 📄clippy提示「manual `!RangeInclusive::contains` implementation」
    /// * ✅对`NaN`会默认返回`false`，故无需担心
    #[inline(always)]
    pub fn is_in_range(value: Float) -> bool {
        (0.0..=1.0).contains(&value)
    }

    /// 模拟`getValue`
    /// * 🚩获取浮点值
    /// * 🚩【2024-05-03 10:51:09】更名为`value_float`以暂时避免与「短浮点」的`value`重名
    ///
    /// # 📄OpenNARS
    ///
    /// To access the value as float
    ///
    /// @return The current value in float
    #[inline(always)]
    pub fn value_float(&self) -> Float {
        self.value as Float * MULTIPLIER_TO_FLOAT
    }

    /// 🆕获取短整数（只读）
    /// * 🎯用于在「其它地方的impl实现」中增强性能（直接读取内部数值）
    #[inline(always)]
    pub fn value_short(&self) -> UShort {
        self.value
    }

    /// 模拟`ShortFloat.setValue`
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

    /// 🆕判断短整数是否合法
    /// * �直接判断「是否小于等于最大值」
    #[inline(always)]
    pub fn is_valid_short(short: UShort) -> bool {
        short <= SHORT_MAX
    }

    /// 🆕判断自身值是否合法
    #[inline(always)]
    pub fn is_valid(&self) -> bool {
        Self::is_valid_short(self.value)
    }

    /// 🆕检查自身值是否合法
    /// * 🚩判断自身值是否合法，然后返回[`Result`]
    pub fn check_valid(&self) -> Result<(), ShortFloatError> {
        match self.is_valid() {
            true => Ok(()),
            false => Err(ShortFloatError::OutOfRange(self.value_float())),
        }
    }

    /// 🆕检查自身值是否合法，并返回自身
    /// * 🚩判断自身值是否合法，然后返回[`Result<Self, ShortFloatError>`](Result)
    /// * 🎯用于「构造后立即检查」
    pub fn validate(self) -> Result<Self, ShortFloatError> {
        match self.is_valid() {
            true => Ok(self),
            false => Err(ShortFloatError::OutOfRange(self.value_float())),
        }
    }

    // ! ℹ️【2024-06-19 23:08:00】如下实现源自抽象的「短浮点」特征 ! //

    /// 从浮点到自身转换（不检查，直接panic）
    /// * ❌在实现[`TryFrom`]时，无法通过[`From`]实现：conflicting implementations of trait `std::convert::TryFrom<f64>` for type `entity::short_float::ShortFloat`
    ///
    /// ! ⚠️在「范围越界」时直接panic
    /// * 🎯降低代码冗余量（减少过多的「错误处理」）
    /// conflicting implementation in crate `core`:
    /// - impl<T, U> std::convert::TryFrom<U> for T
    /// where U: std::convert::Into<T>;
    #[inline(always)]
    pub fn from_float(value: Float) -> Self {
        // ! ⚠️【2024-05-02 20:41:19】直接unwrap
        Self::try_from(value).unwrap()
    }

    #[inline(always)]
    pub fn to_float(&self) -> Float {
        self.value_float()
    }

    pub fn set(&mut self, new_value: Self) {
        // self.clone_from(new_value)
        // *self = new_value;
        self.value = new_value.value;
    }

    pub fn is_zero(&self) -> bool {
        self == &Self::ZERO
    }

    pub fn is_one(&self) -> bool {
        self == &Self::ONE
    }

    pub fn is_half(&self) -> bool {
        self == &Self::HALF
    }

    pub fn value(&self) -> Float {
        self.to_float()
    }
}

/// 模拟`ShortFloat.toString`、`ShortFloat.toStringBrief`
impl ToDisplayAndBrief for ShortFloat {
    fn to_display(&self) -> String {
        match self.value {
            // 对`1`的特别处理 | 🆕不同于OpenNARS：会将「异常值」按原样展示
            SHORT_MAX => "1.0000".to_string(),
            // 否则：右对齐，左边补零到四位，前缀添加`0.`格式化
            value => format!("0.{value:0>4}"),
        }
    }

    fn to_display_brief(&self) -> String {
        // * 🚩先尝试舍入，再决定截断
        match self.value + 50 {
            // 对`1`的特别处理（此时可能要大于了：舍入）
            v if v >= SHORT_MAX => "1.00".to_string(),
            // 否则：四舍五入到百分位；右对齐，只取两位，前缀添加`0.`格式化
            value => {
                let s = (value / 100).to_string();
                format!("0.{s:0>2}")
            }
        }
    }
}

// 一行自动实现`Display`
impl_display_from_to_display! { ShortFloat }

/// 实现「从浮点到『短浮点』的直接转换」
/// 🚩直接通过「构造函数+尝试转换」实现
impl TryFrom<Float> for ShortFloat {
    type Error = ShortFloatError;

    #[inline]
    fn try_from(value: Float) -> Result<Self, Self::Error> {
        Ok(Self::new_unchecked(Self::float_to_short_value(value)?))
    }
}

// 数学方法 //
impl std::ops::Add for ShortFloat {
    type Output = Self;

    /// 内部值相加，但会检查越界
    ///
    /// # Panics
    ///
    /// ! ⚠️可能会有「数值溢出」的panic
    fn add(self, rhs: Self) -> Self::Output {
        // 相加、构造、返回
        Self::new(self.value + rhs.value).unwrap()
    }
}

impl std::ops::Sub for ShortFloat {
    type Output = Self;

    /// 内部值相减，无需检查越界
    /// * 📌不会减去负值，只会「小于`0`」越界
    ///
    /// # Panics
    ///
    /// ! ⚠️可能会有「数值溢出」的panic
    fn sub(self, rhs: Self) -> Self::Output {
        Self::new_unchecked(self.value - rhs.value)
    }
}

impl std::ops::Mul for ShortFloat {
    type Output = Self;

    /// 内部值相乘，无需检查越界
    /// * ✅0~1的数对乘法封闭，故无需任何检查
    /// * ⚠️乘法在最后「除以最大值」时，采用「向下取整」的方式
    /// * ⚠️因为乘法可能会造成上界溢出，故需要转换为「双倍位类型」
    ///   * 🚩现在直接设置为「双倍位类型」
    fn mul(self, rhs: Self) -> Self::Output {
        // * 📄逻辑是 (self.value / 10000) * (rhs.value / 10000) => (new.value / 10000)
        // * 📄实际上 (self.value / 10000) * (rhs.value / 10000) =  (new.value / 10000) / 10000
        // * 📌因此 new.value = (self.value * rhs.value) / 10000
        Self::new_unchecked(mul_div(self.value, rhs.value))
    }
}

/// 相乘再归约到 0~SHORT_MAX 范围内
/// * 🚩目前是【向下取整】归约
fn mul_div(x: UShort, y: UShort) -> UShort {
    (x * y) / SHORT_MAX
}

impl std::ops::Div for ShortFloat {
    type Output = Self;

    /// 内部值相除，会检查越界
    ///
    /// # Panics
    ///
    /// ! ⚠️可能会有「数值溢出」的panic
    fn div(self, rhs: Self) -> Self::Output {
        // * 📄逻辑是 (self.value / 10000) / (rhs.value / 10000) => (new.value / 10000)
        // * 📄实际上 (self.value / 10000) * (rhs.value / 10000) =  self.value / rhs.value
        // * 📌因此 new.value = (self.value / rhs.value) * 10000 = (self.value * 10000) / rhs.value
        // * 📝↑采用「先乘后除」的方法，最大保留精度
        // 相除、构造、返回
        Self::new((self.value * SHORT_MAX) / rhs.value).unwrap()
    }
}

// NAL相关 //
// * 🚩【2024-05-02 11:44:12】有关「真值」「预算值」的函数，均在其它文件中

/// 实现「证据数值」
impl EvidentNumber for ShortFloat {
    #[inline(always)]
    fn zero() -> Self {
        Self::ZERO
    }

    #[inline(always)]
    fn one() -> Self {
        Self::ONE
    }

    fn root(self, n: usize) -> Self {
        // * �【2024-05-02 18:23:31】开根不会越界，故直接`unwrap`
        self.value_float()
            .powf(1.0 / (n as Float))
            .try_into()
            .unwrap()
    }
}

/// 实现「NAL逻辑非」
/// ? 💭是否可以自动派生（主要是受到「孤儿规则」的限制）
impl Not for ShortFloat {
    type Output = Self;

    fn not(self) -> Self::Output {
        Self::ONE - self
    }
}

/// 实现「NAL逻辑与」
/// * 🚩【2024-05-03 11:31:18】对`clippy`允许「令人疑惑的代数实现」
/// ? 💭是否可以自动派生（主要是受到「孤儿规则」的限制）
#[allow(clippy::suspicious_arithmetic_impl)]
impl BitAnd for ShortFloat {
    type Output = Self;

    fn bitand(self, rhs: Self) -> Self::Output {
        self * rhs
    }
}

/// 实现「NAL逻辑或」
/// * 🚩【2024-05-03 11:31:18】对`clippy`允许「令人疑惑的代数实现」
/// ? 💭是否可以自动派生（主要是受到「孤儿规则」的限制）
#[allow(clippy::suspicious_arithmetic_impl)]
impl BitOr for ShortFloat {
    type Output = Self;

    fn bitor(self, rhs: Self) -> Self::Output {
        // pipe! {
        //     // 非
        //     self.not()
        //     // 与
        //     => .and(value.not())
        //     // 非
        //     => .not()
        // }
        // !(!self & !rhs)
        // * 🚩【2024-05-03 12:27:21】做如下代数简化，仍然能通过测试 并且结果一致
        //   1 - (1 - a)(1 - b)
        // = 1 - (1 - a - b + ab)
        // = 1 - 1 + a + b - ab
        // = a + b - ab
        // ↑仅在`ab`引入小数，故最终舍入不会受其影响
        Self::new_unchecked(self.value + rhs.value - ((self.value * rhs.value) / SHORT_MAX))
    }
}

/// [「短浮点」](ShortFloat)的快捷构造宏
#[macro_export]
macro_rules! short_float {
    // 从浮点数构造
    ($float:expr) => {
        ShortFloat::from_float($float)
    };
    // 从字符串构造（保留「结果」）
    (str? $float:expr) => {
        $s.parse::<$crate::global::Float>()
            .map($crate::entity::ShortFloat::try_from)
    };
    // 从字符串构造（一路解包）
    (str $s:expr) => {
        $crate::entity::ShortFloat::try_from($s.parse::<$crate::global::Float>().unwrap()).unwrap()
    };
}

/// 单元测试
#[cfg(test)]
mod tests {
    use super::*;
    use crate::{global::tests::AResult, ok};
    use nar_dev_utils::macro_once;

    /// 用于测试的类型简写
    type SF = ShortFloat;

    // 基本功能 //

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
    fn new() -> AResult {
        macro_once! {
            // * 🚩模式：短整数（作为构造函数参数）
            macro test($( $short:expr )*) {
                $(
                    let _ = SF::new($short);
                )*
            }
            0
            10000
            90
            9000
            1024
            8192
        }
        ok!()
    }

    /// 测试/value
    #[test]
    fn value() -> AResult {
        macro_once! {
            // * 🚩模式：短整数（构造用）⇒预期值
            macro test($( $short:expr => $expected:expr )*) {
                $(
                    let sf = SF::new_unchecked($short);
                    // ! ⚠️此处必须使用「约等」判断，否则会出现`0.009 != 0.009000000000000001`的情形
                    assert_approx_eq!(sf.value_float(), $expected);
                )*
            }
            0 => 0.0
            10000 => 1.0
            90 => 0.009
            9000 => 0.9
            1024 => 0.1024
            8192 => 0.8192
        }
        ok!()
    }

    // * ✅测试/is_in_range已在`float_to_short_value`中一并测试过

    /// 测试/set_value
    #[test]
    fn set_value() -> AResult {
        use ShortFloatError::*;
        macro_once! {
            // * 🚩模式：短整数（构造用） -> 浮点数（赋值用）⇒预期值（短整数） @ 返回的模式
            macro test($( $short:literal -> $float:expr => $expected:literal @ $pattern:pat)*) {
                $(
                    let mut sf = SF::new_unchecked($short);
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
        ok!()
    }

    /// 测试/set_value_unchecked
    #[test]
    fn set_value_unchecked() -> AResult {
        macro_once! {
            // * 🚩模式：短整数（构造用） -> 浮点数（赋值用）⇒预期值（短整数）
            macro test($( $short:literal -> $float:expr => $expected:expr)*) {
                $(
                    let mut sf = SF::new_unchecked($short);
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
            0     -> Float::INFINITY     => UShort::MAX
            10000 -> Float::NEG_INFINITY => 0
            // NaN会被重置为`0`
            10000 -> Float::NAN          => 0
        }
        ok!()
    }

    // 测试/float_to_short_value
    // * ✅已在`set_value`中连带测试过

    // 测试/float_to_short_value_unchecked
    // * ✅已在`set_value`中连带测试过

    /// 测试/fmt
    #[test]
    fn fmt() -> AResult {
        macro_once! {
            // * 🚩模式：短整数（构造用） => 预期值（字符串）
            macro test($( $short:expr => $expected:expr)*) {
                $(
                    let mut sf = SF::new_unchecked($short);
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
        ok!()
    }

    /// 测试/try_from
    #[test]
    fn try_from() -> AResult {
        use ShortFloatError::*;
        macro_once! {
            // * 🚩模式：浮点数（转换用） ⇒ 返回的模式
            macro test($( $float:expr => $pattern:pat)*) {
                $(
                    // 尝试转换
                    let mut result: ShortFloatResult = $float.try_into();
                    // 检查返回值（兼检查转换结果）
                    assert!(matches!(result, $pattern));
                )*
            }
            // 正常转换
            0.0                 => Ok(SF {value: 0})
            1.0                 => Ok(SF {value: 10000})
            0.009               => Ok(SF {value: 90})
            0.9                 => Ok(SF {value: 9000})
            0.1024              => Ok(SF {value: 1024})
            0.8192              => Ok(SF {value: 8192})
            // 四舍五入
            0.00001             => Ok(SF {value: 0})
            0.00002             => Ok(SF {value: 0})
            0.00003             => Ok(SF {value: 0})
            0.00004             => Ok(SF {value: 0})
            0.00005             => Ok(SF {value: 1})
            0.00006             => Ok(SF {value: 1})
            0.00007             => Ok(SF {value: 1})
            0.00008             => Ok(SF {value: 1})
            0.00009             => Ok(SF {value: 1})
            // 异常转换：超出范围
            -0.1                => Err(OutOfRange(..))
             2.0                => Err(OutOfRange(..))
            Float::INFINITY     => Err(OutOfRange(..))
            Float::NEG_INFINITY => Err(OutOfRange(..))
            // 异常转换：无效值
            Float::NAN          => Err(..)
        }
        ok!()
    }

    /// 测试/check_valid
    #[test]
    fn check_valid() -> AResult {
        use ShortFloatError::*;
        macro_once! {
            // * 🚩模式：短整数（构造用） ⇒ 返回的模式
            macro test($( $short:expr => $pattern:pat)*) {
                $(
                    // 尝试转换
                    let sf = SF::new_unchecked($short);
                    // 检查返回值（兼检查转换结果）
                    assert!(matches!(sf.check_valid(), $pattern));
                )*
            }
            // 正常值
            0           => Ok(..)
            10000       => Ok(..)
            90          => Ok(..)
            900         => Ok(..)
            9000        => Ok(..)
            1024        => Ok(..)
            8192        => Ok(..)
            // 异常值：超出范围
            10001       => Err(OutOfRange(..))
            20000       => Err(OutOfRange(..))
            65535       => Err(OutOfRange(..))
        }
        ok!()
    }

    /// 快捷构造
    macro_rules! sf {
        ($short:expr) => {
            SF::new_unchecked($short)
        };
    }

    /// 测试/add
    #[test]
    fn add() -> AResult {
        // 加法 | 保证 a + b <= SHORT_MAX
        for a in 0..=SHORT_MAX {
            for b in 0..=(SHORT_MAX - a) {
                assert_eq!(sf!(a) + sf!(b), sf!(a + b))
            }
        }
        ok!()
    }

    /// 测试/sub
    #[test]
    fn sub() -> AResult {
        // 减法 | 保证 a >= b
        for a in 0..=SHORT_MAX {
            for b in 0..=a {
                assert_eq!(sf!(a) - sf!(b), sf!(a - b))
            }
        }
        ok!()
    }

    /// 测试/mul
    #[test]
    fn mul() -> AResult {
        // 乘法
        assert_eq!(sf!(0) * sf!(0), sf!(0));
        assert_eq!(sf!(0) * sf!(SHORT_MAX), sf!(0));
        assert_eq!(sf!(SHORT_MAX) * sf!(SHORT_MAX), sf!(SHORT_MAX));
        assert_eq!(sf!(7) * sf!(9363), sf!(6)); // 边界情况：乘以的临时值`65541`溢出
        for a in 0..=SHORT_MAX {
            for b in 0..=SHORT_MAX {
                assert_eq!(sf!(a) * sf!(b), sf!(a * b / SHORT_MAX))
            }
        }
        ok!()
    }

    /// 测试/div
    #[test]
    fn div() -> AResult {
        // 除法 | 保证 a < b
        for a in 1..=SHORT_MAX {
            for b in a..=SHORT_MAX {
                assert_eq!(sf!(a) / sf!(b), sf!((a * SHORT_MAX) / b))
            }
        }
        ok!()
    }

    /// 测试/__to_display
    #[test]
    fn __to_display() -> AResult {
        macro_once! {
            /// * 🚩模式：短浮点（浮点值） ⇒ 预期
            macro test($( $value:tt => $expected:tt)*) {
                $(
                    assert_eq!(
                        SF::from_float($value).to_display(),
                        $expected
                    );
                )*
            }
            // 0
            0.0    => "0.0000"
            // 1与非1
            1.0    => "1.0000"
            0.9    => "0.9000"
            // 各个位数
            0.1    => "0.1000"
            0.42   => "0.4200"
            0.137  => "0.1370"
            0.442  => "0.4420"
            0.1024 => "0.1024"
            0.2185 => "0.2185"
        }
        ok!()
    }

    /// 测试/__to_display_brief
    #[test]
    fn __to_display_brief() -> AResult {
        macro_once! {
            /// * 🚩模式：短浮点（浮点值） ⇒ 预期
            macro test($( $value:tt => $expected:tt)*) {
                $(
                    assert_eq!(
                        SF::from_float($value).to_display_brief(),
                        $expected
                    );
                )*
            }
            // 0
            0.0    => "0.00"
            // 1与非1
            1.0    => "1.00"
            0.9    => "0.90"
            // 各个位数
            0.1    => "0.10"
            0.42   => "0.42"
            0.137  => "0.14" // ! 五入
            0.442  => "0.44" // ! 四舍
            0.1024 => "0.10" // ! 四舍
            0.2185 => "0.22" // ! 五入
            0.999  => "1.00" // ! 五入到`1`
            0.9999 => "1.00" // ! 五入到`1`
        }
        ok!()
    }
}
