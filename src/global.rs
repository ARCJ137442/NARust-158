//! 存储一些「全局」参数
//! * 🎯参数类型如「浮点数」（选择哪个精度）
//! * ⚠️【2024-04-27 10:47:59】尽量不要用来存储常量

/// 全局浮点数类型
pub type Float = f64;

/// 全局「时钟数」类型
/// * 🎯NARS内部推理时间
/// * 🎯时间戳[`crate::entity::Stamp`]
/// * 🚩【2024-05-04 17:41:49】目前设定为无符号整数，对标OpenNARS中的`long`长整数类型
///   * 📝OpenNARS中也是将其作为无符号整数（非负整数）用的
pub type ClockTime = usize;

mod time {
    use super::{ClockTime, Float};
    use serde::{Deserialize, Serialize};
    use std::{
        cmp::Ordering,
        fmt::{Display, Formatter},
        num::ParseIntError,
        ops::{Add, Sub},
        str::FromStr,
    };

    /// 全局「时钟时间」类型
    /// * 📝在ONA中是`long`类型
    ///   * 📌基础大小：**32位**
    ///   * 🔗<https://www.tutorialspoint.com/cprogramming/c_data_types.htm>
    /// * 🚩【2024-10-01 23:44:16】目前仍然需要是`iXX`类型，而非「用`0`替代『永恒』」的方式
    ///   * 📌一些地方仍然依赖`0`值，如`Memory`的单测
    /// * 🚩【2024-10-19 15:41:51】目前改为枚举形式
    ///   * 📌以「永恒」为默认值
    #[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Hash, Serialize, Deserialize)]
    pub enum OccurrenceTime {
        /// 永恒
        #[default]
        Eternal,
        /// 已存在的时间
        Time(ClockTime),
    }
    use OccurrenceTime::{Eternal, Time};

    impl OccurrenceTime {
        /// 绝对时间差
        /// * 🚩【2024-10-19 15:30:04】返回自身以处理「永恒」时间
        /// * 📌双方含有永恒 ⇒ 永恒
        /// * 📌双方都有时间 ⇒ 时间差
        pub fn abs_diff(self, other: Self) -> Self {
            match (self, other) {
                (Eternal, _) | (_, Eternal) => Eternal,
                (Time(a), Time(b)) => Time(a.abs_diff(b)),
            }
        }

        /// 绝对时间差（数值版）
        /// * 🚩【2024-10-19 15:33:02】用于向前兼容ONA的数值版本（永恒 as -1）
        ///
        /// TODO: 考虑后续移除（显化调用处对于「永恒时间」的处理）
        pub fn abs_diff_int(self, other: Self) -> ClockTime {
            match (self, other) {
                // 两个永恒⇒0
                (Eternal, Eternal) => 0,
                // 一边永恒⇒时间+1
                (Eternal, Time(t)) | (Time(t), Eternal) => t + 1,
                // 正常时间差
                (Time(a), Time(b)) => a.abs_diff(b),
            }
        }

        /// 浮点版本
        /// * 🚩【2024-10-19 15:33:02】用于向前兼容ONA的数值版本（永恒 as -1）
        pub fn into_float(self) -> Float {
            match self {
                Eternal => -1.0,
                Time(t) => t as f64,
            }
        }
    }

    /// 显示呈现
    /// * 🚩永恒 => 特殊标识
    /// * 🚩有时间 => 时间数值
    impl Display for OccurrenceTime {
        fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
            /// 有关「永恒」的呈现
            /// * 🚩【2024-10-19 15:59:18】向前兼容ONA的「永恒 as -1」呈现
            const ETERNAL_DISPLAY: &str = "-1";

            match self {
                Eternal => write!(f, "{ETERNAL_DISPLAY}"),
                Time(t) => write!(f, "{t}"),
            }
        }
    }

    /// 大小比较
    impl Ord for OccurrenceTime {
        fn cmp(&self, other: &Self) -> Ordering {
            use Ordering::{Equal, Greater, Less};

            // * 📝按照ONA「永恒是-1」的语义：永恒小于「有时间」，其它情况则比较时间
            match [self, other] {
                // 永恒 == 永恒
                [Eternal, Eternal] => Equal,
                // 永恒 < 有时间
                [Eternal, Time(..)] => Less,
                [Time(..), Eternal] => Greater,
                // 时间之间的比较
                [Time(a), Time(b)] => a.cmp(b),
            }
        }
    }

    impl PartialOrd for OccurrenceTime {
        fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
            Some(self.cmp(other))
        }
    }

    /// 加法运算 & 整数
    impl Add<ClockTime> for OccurrenceTime {
        type Output = Self;

        fn add(self, dt: ClockTime) -> Self::Output {
            match self {
                // 永恒还是永恒
                Eternal => Eternal,
                // 有时间 ⇒ 内部值相加
                Time(a) => Time(a + dt),
            }
        }
    }

    /// 加法运算 & 浮点数
    /// * 🎯抽象自`inference::belief_deduction`
    impl Add<Float> for OccurrenceTime {
        type Output = Self;

        fn add(self, dt: Float) -> Self::Output {
            match self {
                // 永恒还是永恒
                Eternal => Eternal,
                // 有时间 ⇒ 内部值相加
                // * 📝【2024-09-19 21:00:40】↓C语言在下边的式子中会发生「类型提升」，先转换成浮点再最后转换回整型
                Time(t) => Time((t as Float + dt) as ClockTime),
            }
        }
    }

    /// 加法运算
    impl Add for OccurrenceTime {
        type Output = Self;

        fn add(self, that: Self) -> Self::Output {
            match [self, that] {
                // 永恒还是永恒
                [Eternal, _] | [_, Eternal] => Eternal,
                // 两个有时间 ⇒ 内部值相加
                [Time(a), Time(b)] => Time(a + b),
            }
        }
    }

    /// 减法运算
    impl Sub for OccurrenceTime {
        type Output = Self;

        /// # Panics
        ///
        /// ⚠️可能有数值溢出
        fn sub(self, that: Self) -> Self::Output {
            match [self, that] {
                // 永恒减以任何时间都是永恒
                [Eternal, _] | [_, Eternal] => Eternal,
                // 两个有时间 ⇒ 内部值相减
                [Time(a), Time(b)] => Time(a - b),
            }
        }
    }

    /// 为「时钟时间」实现功能
    /// * 🎯提前封装一些【后续可与枚举相对应】的功能
    ///   * 📄检查「是否永恒」
    impl OccurrenceTime {
        /// 「时钟时间」中的「永恒」
        /// * 🚩用于表示「永远」
        pub const ETERNAL: Self = Self::Eternal;

        /// 判断一个「时钟时间」是否永恒
        #[inline]
        pub fn is_eternal(&self) -> bool {
            *self == Self::ETERNAL
        }

        /// 判断一个「时钟时间」是否非永恒
        #[inline]
        pub fn not_eternal(&self) -> bool {
            !self.is_eternal()
        }
    }

    /// 从[`UTime`] 转换为 [`ClockTime`]
    /// * 🎯兼容ONA旧有逻辑
    impl From<ClockTime> for OccurrenceTime {
        #[inline]
        fn from(time: ClockTime) -> Self {
            Self::Time(time)
        }
    }

    /// 从字符串解析
    /// * 🚩【2024-10-19 16:10:11】向前兼容ONA：`-1`代表「永恒」
    impl FromStr for OccurrenceTime {
        type Err = ParseIntError;

        #[inline]
        fn from_str(s: &str) -> Result<Self, Self::Err> {
            Ok(match s {
                // * 🚩`-1` ⇒ 永恒
                "-1" => Self::Eternal,
                // * 🚩其它 ⇒ 具体值
                _ => Self::Time(s.parse()?),
            })
        }
    }
}
pub use time::{OccurrenceTime::Eternal, *};

/// 全局引用计数类型
/// * 🚩【2024-05-22 14:27:34】现在默认为「可变共享引用」，暂不细分「不可变」与「可变」
///   * 📌目前使用情况主要在「任务链」与「任务袋」中，这些情况
pub type RC<T> = nar_dev_utils::RcCell<T>;
