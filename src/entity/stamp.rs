//! 🎯复刻OpenNARS `nars.entity.Stamp`
//! * ✅【2024-05-05 15:50:54】基本特征功能复刻完成
//! * ✅【2024-05-05 17:03:34】单元测试初步完成
//! * ♻️【2024-06-21 00:05:34】基本依OpenNARS改版 重复刻（去特征化）完成

use crate::__impl_to_display_and_display;
use crate::{global::ClockTime, inference::Evidential};
use anyhow::Result;
use narsese::lexical::Stamp as LexicalStamp;
use serde::{Deserialize, Serialize};
use std::hash::{Hash, Hasher};

/// [时间戳](Stamp)初代实现
#[derive(Debug, Clone, Eq, Serialize, Deserialize)]
pub struct Stamp {
    evidential_base: Box<[ClockTime]>,
    creation_time: ClockTime,
}

impl Evidential for Stamp {
    #[inline(always)]
    fn evidential_base(&self) -> &[ClockTime] {
        &self.evidential_base
    }

    #[inline(always)]
    fn creation_time(&self) -> ClockTime {
        self.creation_time
    }

    /// 🆕自身到「词法」的转换
    /// * 🎯标准Narsese输出需要（Narsese内容）
    /// * 🚩【2024-05-12 14:48:31】此处跟随OpenNARS，使用空字串
    ///   * 时态暂均为「永恒」
    fn stamp_to_lexical(&self) -> LexicalStamp {
        LexicalStamp::new()
    }
}

__impl_to_display_and_display! {
    // * 🚩【2024-05-09 00:37:24】只实现一个方法（其它默认）
    @(stamp_to_display;;)
    Stamp as Evidential
}

/// 模拟`equals`
impl PartialEq for Stamp {
    /// 模拟`Stamp.toSet`、`Stamp.equals`
    /// * 🎯用于方便实现者用其统一实现[`PartialEq`]
    /// * 🚩证据基集合判等
    ///
    /// # 📄OpenNARS
    ///
    /// Check if two stamps contains the same content
    ///
    /// @param that The Stamp to be compared
    /// @return Whether the two have contain the same elements
    #[inline(always)]
    fn eq(&self, other: &Self) -> bool {
        self.evidential_eq(other)
    }
}

/// 模拟`hashCode`
impl Hash for Stamp {
    /// 模拟`Stamp.hashCode`
    /// * 🎯用于方便实现者用其统一实现[`Hash`]
    /// * ⚠️🆕此处仅对「证据基」作散列化，以保证「散列码相等⇔时间戳相等」
    /// * 📝OpenNARS是通过「证据基+创建时间 → 字符串 → 散列码」转换的
    ///   * 📌但这样会破坏上述的一致性
    ///   * 💭【2024-05-05 17:39:19】似乎仍然只能保证「散列码相等⇒时间戳相等」，顺序因素无法保证
    /// * 🚩证据基集合散列化
    ///
    /// # 📄OpenNARS
    ///
    /// The hash code of Stamp
    ///
    /// @return The hash code
    #[inline(always)]
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.evidential_base().hash(state);
    }
}

/// impl StampConcrete for StampV1
impl Stamp {
    /// 空的、内部的构造函数
    /// * 🚩⚠️【2024-05-05 15:48:24】仅直接安放数值，不负责任何语义处理
    /// * 📌与`current_serial`无关
    pub fn new(creation_time: ClockTime, evidential_base: impl Into<Vec<ClockTime>>) -> Self {
        Self {
            evidential_base: evidential_base.into().into_boxed_slice(),
            creation_time,
        }
    }

    /// 模拟`new Stamp(long time)`
    /// * 🎯一致的对外构造函数
    /// * 🚩【2024-05-05 14:28:49】参数`current_serial`意味着**其自增要在调用方处管理**
    /// * 📌`current_serial`对应[`Self::evidential_base`]的第一个值
    /// * 📌`time`就对应[`Self::creation_time`]
    ///
    /// # 📄OpenNARS
    ///
    /// Generate a new stamp, with a new serial number, for a new Task
    ///
    /// @param time Creation time of the stamp
    pub fn with_time(current_serial: ClockTime, time: ClockTime) -> Self {
        /* 📄OpenNARS源码：
        currentSerial++;
        baseLength = 1;
        evidentialBase = new long[baseLength];
        evidentialBase[0] = currentSerial;
        creationTime = time; */
        let evidential_base = vec![current_serial];
        Self::new(time, evidential_base)
    }

    /// 模拟`new Stamp(Stamp old, long time)`
    /// * 🚩【2024-05-05 14:30:28】根据OpenNARS，`current_serial`参数就与[「创建时间」](Stamp::creation_time)对应
    ///   * 因此直接将「创建时间」传入
    ///
    /// # 📄OpenNARS
    ///
    /// Generate a new stamp from an existing one, with the same evidentialBase but
    /// different creation time
    ///
    /// For single-premise rules
    ///
    /// @param old  The stamp of the single premise
    /// @param time The current time
    pub fn with_old(old: &impl Evidential, time: ClockTime) -> Self {
        /* 📄OpenNARS源码：
        baseLength = old.length();
        evidentialBase = old.getBase();
        creationTime = time; */
        Self::new(time, old.evidential_base())
    }

    /// 模拟`new Stamp(Stamp first, Stamp second, long time)`
    /// * 🚩【2024-05-05 14:30:28】根据OpenNARS，`current_serial`参数就与[「创建时间」](Stamp::creation_time)对应
    ///   * 因此直接将「创建时间」传入
    ///
    /// # 📄OpenNARS
    ///
    /// Generate a new stamp for derived sentence by merging the two from parents
    /// the first one is no shorter than the second
    ///
    /// @param first  The first Stamp
    /// @param second The second Stamp
    pub fn from_merge_unchecked(
        first: &impl Evidential,
        second: &impl Evidential,
        time: ClockTime,
        max_evidence_base_length: usize,
    ) -> Self {
        let merged_base = Self::merged_evidential_base(
            first.evidential_base(),
            second.evidential_base(),
            max_evidence_base_length,
        );
        Self::new(time, merged_base)
    }

    /// 模拟`Stamp.make`
    ///
    /// # 📄OpenNARS
    ///
    /// Try to merge two Stamps, return null if have overlap
    ///
    /// By default, the event time of the first stamp is used in the result
    ///
    /// @param first  The first Stamp
    /// @param second The second Stamp
    /// @param time   The new creation time
    /// @return The merged Stamp, or null
    pub fn from_merge(
        first: &impl Evidential,
        second: &impl Evidential,
        time: ClockTime,
        max_evidence_base_length: usize,
    ) -> Option<Self> {
        // * 🚩有重合证据⇒返回空；无重合证据⇒合并证据
        match first.evidential_overlap(second) {
            true => None,
            false => Some(Self::from_merge_unchecked(
                first,
                second,
                time,
                max_evidence_base_length,
            )),
        }
    }

    /// 🆕自「词法Narsese / 解析器」构造
    /// * 🎯模拟`nars.io.StringParser.parseTask`的一部分
    /// * 🚩通过「记忆区内部时钟」从用户输入构造
    ///   * 🔗参考OpenNARS`nars.main_nogui.ReasonerBatch.textInputLine`
    ///   * 🔗参考OpenNARS`nars.io.StringParser.parseExperience`
    /// * 🚩【2024-05-10 19:55:39】改名`from_lexical`，实际上并不使用
    ///   * 📌目前总是返回`Ok`（解析成功）
    ///   * 🎯容许后续补充
    /// * 📝OpenNARS 1.5.8并未有「时间戳」的「时态」机制
    /// * 🚩【2024-05-13 10:04:30】目前恢复独立的`current_serial`参数
    ///   * 📝且这个参数先增后用
    pub fn from_lexical(
        _: LexicalStamp,
        current_serial: ClockTime,
        time: ClockTime,
    ) -> Result<Self> {
        Ok(Self::with_time(current_serial, time))
    }
}

/// 初代「时间戳」的快捷构造宏
/// * 🚩模式：{发生时间: 证据1; 证据2; ...}
#[macro_export]
macro_rules! stamp {
    ({ $creation_time:tt : $($evidence:expr $(;)? )* }) => {
        Stamp::new(
            $creation_time,
            &[ $( $evidence ),* ]
        )
    };
}

/// 单元测试
#[cfg(test)]
mod tests {
    use super::*;
    use crate::util::ToDisplayAndBrief;
    use nar_dev_utils::macro_once;

    /// 测试用「时间戳」类型
    type S = Stamp;

    /// 测试用「时间戳判等」
    /// * 🎯完全比对所有字段，并且按照顺序逐个比对
    macro_rules! assert_s_eq {
        // 对两个「时间戳」完全判等
        ($s1:expr, $s2:expr $(, $($arg:tt)*)?) => {
            assert_eq!($s1.evidential_base(), $s2.evidential_base() $(, $($arg)*)?);
            assert_eq!($s1.creation_time(), $s2.creation_time() $(, $($arg)*)?);
        };
        // 对两个「时间戳Option」完全判等
        (Option $s1:expr, $s2:expr $(, $($arg:tt)*)?) => {
            assert_eq!($s1.is_some(), $s2.is_some() $(, $($arg)*)?);
            if let (Some(s1), Some(s2)) = ($s1, $s2) {
                assert_s_eq!(s1, s2 $(, $($arg)*)?);
            }
        };
    }

    // * ✅测试/__new 已在后续函数中测试

    /// 测试/with_time
    #[test]
    fn with_time() {
        macro_once! {
            /// * 🚩模式：(当前时钟时间, 创建时间) => 预期【时间戳`stamp!`】
            macro test($( ( $current_serial:expr, $time:expr ) => $stamp:tt )*) {
                $(
                    assert_s_eq!(S::with_time( $current_serial, $time ), stamp!($stamp));
                )*
            }
            (1, 0) => {0: 1}
            (2, 1) => {1: 2}
            (2147483647, 10000) => {10000: 2147483647}
            (0xfade, 0xabcd) => {0xabcd: 0xfade}
        }
    }

    /// 测试/with_old
    #[test]
    fn with_old() {
        macro_once! {
            /// * 🚩模式：(旧【时间戳`stamp!`】, 创建时间) => 预期【时间戳`stamp!`】
            macro test($( ( $old:tt, $time:expr ) => $stamp:tt )*) {
                $(
                    assert_s_eq!(S::with_old( &stamp!($old), $time ), stamp!($stamp));
                )*
            }
            ({0: 1}, 1) => {1: 1}
            ({0: 2}, 1) => {1: 2}
            ({10000: 2147483647}, 0) => {0: 2147483647}
            ({10000: 0xabcd}, 0xfade) => {0xfade: 0xabcd}
        }
    }

    // * ✅测试/__from_merge 已在后续函数中测试

    /// 测试/from_merge
    #[test]
    fn from_merge() {
        macro_once! {
            /// * 🚩模式：(【时间戳1`stamp!`】, 【时间戳2`stamp!`】, 创建时间) => 预期【时间戳`stamp!`/None】
            macro test {
                // 没结果
                (@SINGLE ( $s1:tt, $s2:tt, $time:expr, $max_base_l:expr ) => None ) => {
                    assert_s_eq!(Option S::from_merge(&stamp!($s1), &stamp!($s2), $time, $max_base_l), None::<S>);
                }
                // 有结果
                (@SINGLE ( $s1:tt, $s2:tt, $time:expr, $max_base_l:expr ) => $stamp:tt ) => {
                    assert_s_eq!(Option S::from_merge(&stamp!($s1), &stamp!($s2), $time, $max_base_l), Some(stamp!($stamp)));
                }
                // 总模式
                ( $( $parameters:tt => $expected:tt )* ) => {
                    $( test!( @SINGLE $parameters => $expected ); )*
                }
            }
            ({0: 1}, {0: 1}, 1, 8) => None
            ({0: 1}, {0: 2}, 10, 8) => {10: 2; 1}
            ({0: 2}, {0: 1}, 10, 8) => {10: 1; 2}
            ({0: 2; 4; 6}, {0: 1; 3; 5}, 10, 8) => {10: 1; 2; 3; 4; 5; 6}
            // ! 📄来自OpenNARS实际运行过程
            ({1 : 2}, {0 : 1}, 2, 8) => {2 : 1;2} // ! ⚠️注意：需要是传入`Stamp.make`处的参数（可能中途调换位置）
            ({13 : 3}, {13 : 1;2}, 13, 8) => {13 : 1;3;2}
            ({34 : 4}, {14 : 1;3;2}, 35, 8) => {35 : 1;4;3;2}
            ({34 : 4}, {14 : 1;3;2}, 35, 4) => {35 : 1;4;3;2}
            ({34 : 4}, {14 : 1;3;2}, 35, 3) => {35 : 1;4;3}
            ({34 : 4}, {14 : 1;3;2}, 35, 2) => {35 : 1;4}
            ({34 : 4}, {14 : 1;3;2}, 35, 1) => {35 : 1}
            ({34 : 4}, {14 : 1;3;2}, 35, 0) => {35 :}
        }
    }

    /// 测试/evidential_base
    #[test]
    fn evidential_base() {
        macro_once! {
            /// * 🚩模式：【时间戳`stamp!`】 => [证据时间...]
            macro test($( $stamp:tt => [ $($time:expr $(,)? )* ] )*) {
                $(
                     // ! ⚠️【2024-05-06 11:30:48】可能的编译错误：在引入`serde_json`后，若对空数组判等，则会导致`&[usize]`与`&[serde_json::Value]`的类型歧义
                     // * 🚩故此处限定「预期」的类型
                    let expected: &[ClockTime] = &[ $($time),* ];
                    assert_eq!(stamp!($stamp).evidential_base(), expected);
                )*
            }
            {0: } => []
            {0: 1} => [1]
            {0 : 1;3;4} => [1,3,4]
            {0 : 0xabcd;3;0xfade} => [0xabcd,3,0xfade]
            {7 : 1;6;3} => [1, 6, 3] // ! 📄来自OpenNARS实际运行过程
        }
    }

    /// 测试/evidence_length
    #[test]
    fn evidence_length() {
        macro_once! {
            /// * 🚩模式：【时间戳`stamp!`】 => 预期
            macro test($( $stamp:tt => $expected:expr )*) {
                $(
                    assert_eq!(stamp!($stamp).evidence_length(), $expected);
                )*
            }
            {15 : 15} => 1 // ! 📄来自OpenNARS实际运行过程
            {29 : 15} => 1 // ! 📄来自OpenNARS实际运行过程
            {18 : 15;6} => 2 // ! 📄来自OpenNARS实际运行过程
            {7 : 1;6;3} => 3 // ! 📄来自OpenNARS实际运行过程
        }
    }

    /// 测试/creation_time
    #[test]
    fn creation_time() {
        macro_once! {
            /// * 🚩模式：【时间戳`stamp!`】 => 预期
            macro test($( $stamp:tt => $expected:expr )*) {
                $(
                    assert_eq!(stamp!($stamp).creation_time(), $expected);
                )*
            }
            {15 : 15} => 15 // ! 📄来自OpenNARS实际运行过程
            {6 : 6} => 6 // ! 📄来自OpenNARS实际运行过程
            {7 : 1;6;3} => 7 // ! 📄来自OpenNARS实际运行过程
        }
    }

    /// 测试/get
    #[test]
    fn get() {
        macro_once! {
            /// * 🚩模式：【时间戳`stamp!`】 @ 索引 => 预期
            macro test($( $stamp:tt @ $index:expr => $expected:expr )*) {
                $(
                    assert_eq!(stamp!($stamp).get($index), $expected);
                )*
            }
            {15 : 15} @ 0 => 15
            {29 : 15} @ 0 => 15 // ! 📄来自OpenNARS实际运行过程
            {33 : 15;6} @ 0 => 15 // ! 📄来自OpenNARS实际运行过程
            {16 : 1;15;3} @ 1 => 15 // ! 📄来自OpenNARS实际运行过程
        }
    }

    /// 测试/evidential_eq
    #[test]
    fn evidential_eq() {
        macro_once! {
            /// * 🚩模式：(【时间戳1`stamp!`】, 【时间戳2`stamp!`】, 创建时间) => 预期【时间戳`stamp!`/None】
            macro test( $( ($s1:tt, $s2:tt) => $expected:tt )* ) {
                $(
                    // 验证「相等」符合预期
                    assert_eq!(stamp!($s1).evidential_eq(&stamp!($s2)), $expected);
                    // 验证`equals`与`==`一致
                    assert_eq!(stamp!($s1) == stamp!($s2), $expected);
                )*
            }
            // 单个：不一致就是不一致
            ({0: 1}, {0: 1}) => true
            ({0: 1}, {0: 2}) => false
            ({0: 2}, {0: 1}) => false
            // 只比较「证据基」而不比较「创建时间」
            ({0: 1}, {1: 1}) => true
            // 多个：无序比较证据基
            ({0: 1; 2}, {0: 1; 2}) => true
            ({0: 1; 2}, {0: 2; 1}) => true
            ({1000: 1; 2}, {0: 2; 1}) => true // 忽略创建时间
            ({0: 1; 2; 3}, {0: 2; 1; 3}) => true
            ({0: 1; 2; 3}, {0: 1; 3; 2}) => true
            ({0: 1; 2; 3}, {0: 3; 2; 1}) => true
            ({0: 1; 2; 3}, {0: 2; 3; 1}) => true
            ({0: 1; 2; 3}, {0: 3; 1; 2}) => true
        }
    }

    /// 测试/to_display
    #[test]
    fn to_display() {
        macro_once! {
            /// * 🚩模式：【时间戳`stamp!`】 => 预期
            macro test($( $stamp:tt => $expected:expr )*) {
                $(
                    assert_eq!(stamp!($stamp).to_display(), $expected);
                )*
            }
            {15 : 15} => "{15 : 15}" // ! 📄来自OpenNARS实际运行过程
            {29 : 15} => "{29 : 15}" // ! 📄来自OpenNARS实际运行过程
            {18 : 15;6} => "{18 : 15;6}" // ! 📄来自OpenNARS实际运行过程
            {7 : 1;6;3} => "{7 : 1;6;3}" // ! 📄来自OpenNARS实际运行过程
        }
    }
}
