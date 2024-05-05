//! 🎯复刻OpenNARS `nars.entity.Stamp`
//! * ✅【2024-05-05 15:50:54】基本特征功能复刻完成
//! * ✅【2024-05-05 17:03:34】单元测试初步完成

use crate::{global::ClockTime, nars::DEFAULT_PARAMETERS};
use std::hash::Hash;

/// 模拟OpenNARS `nars.entity.Stamp`
/// * 🚩🆕【2024-05-05 14:06:13】目前拒绝「全局静态变量」：这些量应该始终有个确切的来源
///   * 📄如：推理器时钟
/// * 🚩用特征约束 [`Hash`]模拟`Stamp.hashCode`
/// * 🚩用特征约束 [`PartialEq`]模拟`Stamp.hashCode`
///   * ⚠️因「孤儿规则」限制，无法统一自动实现
///   * 📌统一的逻辑：**对「证据基」集合判等（无序相等）**
///
/// # 📄OpenNARS
///
/// Each Sentence has a time stamp, consisting the following components:
/// (1) The creation time of the sentence,
/// (2) A evidentialBase of serial numbers of sentence, from which the sentence
/// is derived.
/// Each input sentence gets a unique serial number, though the creation time may
/// be not unique.
/// The derived sentences inherits serial numbers from its parents, cut at the
/// baseLength limit.
pub trait Stamp: Hash {
    // ! ❌【2024-05-05 14:07:05】不模拟`Stamp.currentSerial`，理由同上

    /// 模拟`Stamp.evidentialBase`、`Stamp.getBase`
    /// * 📝译名为「证据基」
    /// * 🚩【2024-05-05 14:09:16】目前仅使用数组切片，所有权应该在`self`内部存储
    ///
    /// # 📄OpenNARS
    ///
    /// serial numbers
    fn evidential_base(&self) -> &[ClockTime];

    /// 模拟`Stamp.baseLength`、`Stamp.length`
    /// * 🚩🆕【2024-05-05 14:11:23】不直接模拟`Stamp.baseLength`：实际上就是[`Stamp::__evidential_base`]的长度
    /// * 📝OpenNARS中在所有「构造方法之外的方法」中均只读
    ///
    /// # 📄OpenNARS
    ///
    /// evidentialBase baseLength
    #[inline(always)]
    fn base_length(&self) -> usize {
        self.evidential_base().len()
    }

    /// 模拟`Stamp.creationTime`、`Stamp.getCreationTime`
    /// * 📝这个「创建时间」是一个特殊的元素
    ///   * ⚠️不一定在[`Stamp::__evidential_base`]中
    ///
    /// # 📄OpenNARS
    ///
    /// creation time of the stamp
    fn creation_time(&self) -> ClockTime;

    // ! ❌【2024-05-05 14:19:27】不模拟`Stamp.init`静态方法，理由同`Stamp.currentSerial`

    /// 模拟`Stamp.get`
    ///
    /// # 📄OpenNARS
    ///
    /// Get a number from the evidentialBase by index, called in this class only
    ///
    /// @param i The index
    /// @return The number at the index
    fn get(&self, i: usize) -> ClockTime {
        self.evidential_base()[i]
    }

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
    fn equals(&self, other: &impl Stamp) -> bool {
        set_vec_eq(self.evidential_base(), other.evidential_base())
    }
}

/// [`Vec`]集合判等
fn set_vec_eq<T: Clone + Ord>(v1: &[T], v2: &[T]) -> bool {
    let mut v1 = v1.to_owned();
    let mut v2 = v2.to_owned();
    v1.sort();
    v2.sort();
    v1 == v2
}

/// [`Stamp`]的具体类型版本
/// * 📌假定信息就是「所获取的信息」没有其它外延
/// * 🎯约束构造方法
///
/// * 🚩用[`Clone`]对标Java接口`Cloneable`，并模拟`new Stamp(Stamp)`
pub trait StampConcrete: Stamp + Clone {
    /// 空的、内部的构造函数
    /// * 🚩⚠️【2024-05-05 15:48:24】仅直接安放数值，不负责任何语义处理
    /// * 📌与`current_serial`无关
    fn __new(creation_time: ClockTime, evidential_base: &[ClockTime]) -> Self;

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
    fn with_time(current_serial: ClockTime, time: ClockTime) -> Self {
        /* 📄OpenNARS源码：
        currentSerial++;
        baseLength = 1;
        evidentialBase = new long[baseLength];
        evidentialBase[0] = currentSerial;
        creationTime = time; */
        let evidential_base = vec![current_serial];
        Self::__new(time, &evidential_base)
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
    fn with_old(old: &impl Stamp, time: ClockTime) -> Self {
        /* 📄OpenNARS源码：
        baseLength = old.length();
        evidentialBase = old.getBase();
        creationTime = time; */
        Self::__new(time, old.evidential_base())
    }

    /// 模拟`new Stamp(Stamp first, Stamp second, long time)`
    /// * 🚩【2024-05-05 14:30:28】根据OpenNARS，`current_serial`参数就与[「创建时间」](Stamp::creation_time)对应
    ///   * 因此直接将「创建时间」传入
    /// * 💫【2024-05-05 16:40:38】目前对此运作逻辑尚不清楚
    ///
    /// # 📄OpenNARS
    ///
    /// Generate a new stamp for derived sentence by merging the two from parents
    /// the first one is no shorter than the second
    ///
    /// @param first  The first Stamp
    /// @param second The second Stamp
    fn __from_merge(first: &impl Stamp, second: &impl Stamp, time: ClockTime) -> Self {
        /* 📄OpenNARS源码：int i1, i2, j;
        i1 = i2 = j = 0;
        baseLength = Math.min(first.length() + second.length(), Parameters.MAXIMUM_STAMP_LENGTH);
        evidentialBase = new long[baseLength];
        while (i2 < second.length() && j < baseLength) {
            evidentialBase[j] = first.get(i1);
            i1++;
            j++;
            evidentialBase[j] = second.get(i2);
            i2++;
            j++;
        }
        while (i1 < first.length() && j < baseLength) {
            evidentialBase[j] = first.get(i1);
            i1++;
            j++;
        }
        creationTime = time; */
        let mut i1 = 0;
        let mut i2 = 0;
        let mut j = 0;
        let base_length = ClockTime::min(
            first.base_length() + second.base_length(),
            DEFAULT_PARAMETERS.maximum_stamp_length,
        );
        let mut evidential_base = vec![0; base_length];
        while i2 < second.base_length() && j < base_length {
            evidential_base[j] = first.get(i1);
            i1 += 1;
            j += 1;
            evidential_base[j] = second.get(i2);
            i2 += 1;
            j += 1;
        }
        while i1 < first.base_length() && j < base_length {
            evidential_base[j] = first.get(i1);
            i1 += 1;
            j += 1;
        }
        Self::__new(time, &evidential_base)
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
    fn from_merge(first: &impl Stamp, second: &impl Stamp, time: ClockTime) -> Option<Self> {
        /* 📄OpenNARS源码：
        for (int i = 0; i < first.length(); i++) {
            for (int j = 0; j < second.length(); j++) {
                if (first.get(i) == second.get(j)) {
                    return null;
                }
            }
        }
        if (first.length() > second.length()) {
            return new Stamp(first, second, time);
        } else {
            return new Stamp(second, first, time);
        } */
        // * 🚩本质逻辑是：包含相同证据基⇒返回空值
        for i in first.evidential_base() {
            if second.evidential_base().contains(i) {
                return None;
            }
        }
        match first.base_length() > second.base_length() {
            true => Some(Self::__from_merge(first, second, time)),
            false => Some(Self::__from_merge(second, first, time)),
        }
    }
}

/// 初代实现
mod impl_v1 {
    use super::*;
    use std::hash::Hasher;

    /// [时间戳](Stamp)初代实现
    #[derive(Debug, Clone)]
    pub struct StampV1 {
        evidential_base: Box<[ClockTime]>,
        creation_time: ClockTime,
    }

    /// 仅比对「证据基」所含元素
    /// * 模拟OpenNARS`equals`
    impl PartialEq for StampV1 {
        fn eq(&self, other: &Self) -> bool {
            self.equals(other)
        }
    }

    /// 模拟OpenNARS`hashCode`
    /// * ⚠️🆕此处仅对「证据基」作散列化，以保证「散列码相等⇔时间戳相等」
    /// * 📝OpenNARS是通过「证据基+创建时间 → 字符串 → 散列码」转换的
    ///   * 📌但这样会破坏上述的一致性
    impl Hash for StampV1 {
        fn hash<H: Hasher>(&self, state: &mut H) {
            self.evidential_base.hash(state);
        }
    }

    impl Stamp for StampV1 {
        #[inline(always)]
        fn evidential_base(&self) -> &[ClockTime] {
            &self.evidential_base
        }

        #[inline(always)]
        fn creation_time(&self) -> ClockTime {
            self.creation_time
        }
    }

    impl StampConcrete for StampV1 {
        fn __new(creation_time: ClockTime, evidential_base: &[ClockTime]) -> Self {
            Self {
                evidential_base: evidential_base.to_vec().into(),
                creation_time,
            }
        }
    }
}
pub use impl_v1::*;

/// 单元测试
#[cfg(test)]
mod tests {
    use super::*;
    use nar_dev_utils::macro_once;

    /// 测试用「时间戳」类型
    type S = StampV1;

    /// 测试/set_vec_eq
    /// * 🎯数组集合判等
    #[test]
    fn set_vec_eq() {
        macro_once! {
            /// * 🚩正例 模式：原数组⇒预期相等
            macro test($($value:expr => $($equivalent:expr $(,)? )* ; )*) {
                $(
                    $(
                        assert!(super::set_vec_eq::<usize>(&$value, &$equivalent));
                    )*
                )*
            }
            [] => [];
            [1] => [1];
            [1, 2] => [2, 1];
            [1, 2, 3] => [2, 3, 1], [3, 2, 1], [1, 3, 2], [3, 1, 2], [2, 1, 3];
        }
        macro_once! {
            /// * 🚩反例 模式：原数组⇒预期相等
            macro test($($value:tt != $($equivalent:expr $(,)? )* ; )*) {
                $(
                    $(
                        assert!(!super::set_vec_eq::<usize>(&$value, &$equivalent));
                    )*
                )*
            }
            [1] != [];
            [1] != [0];
            [1, 2] != [1, 1];
            [1, 2] != [1];
            [1, 2, 3] != [2, 0, 1], [0, 2, 1], [1, 0, 2], [0, 1, 2], [2, 1, 0];
        }
    }

    // * ✅测试/__new 已在后续函数中测试

    /// 快捷构造宏
    /// * 🚩模式：{发生时间: 证据1; 证据2; ...}
    macro_rules! stamp {
        ({$creation_time:tt : $($evidence:expr $(;)? )* }) => {
            S::__new(
                $creation_time,
                &[
                    $(
                        $evidence
                    ),*
                ]
            )
        };
    }

    /// 测试/with_time
    #[test]
    fn with_time() {
        macro_once! {
            /// * 🚩模式：(当前时钟时间, 创建时间) => 预期【时间戳`stamp!`】
            macro test($( ( $current_serial:expr, $time:expr ) => $stamp:tt )*) {
                $(
                    assert_eq!(S::with_time( $current_serial, $time ),stamp!($stamp));
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
                    assert_eq!(S::with_old( &stamp!($old), $time ), stamp!($stamp));
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
                (@SINGLE ( $s1:tt, $s2:tt, $time:expr ) => None ) => {
                    assert_eq!(S::from_merge( &stamp!($s1), &stamp!($s2), $time ), None);
                }
                // 有结果
                (@SINGLE ( $s1:tt, $s2:tt, $time:expr ) => $stamp:tt ) => {
                    assert_eq!(S::from_merge( &stamp!($s1), &stamp!($s2), $time ), Some(stamp!($stamp)));
                }
                // 总模式
                ( $( $parameters:tt => $expected:tt )* ) => {
                    $( test!( @SINGLE $parameters => $expected ); )*
                }
            }
            ({0: 1}, {0: 1}, 1) => None
            ({0: 1}, {0: 2}, 10) => {10: 1; 2}
            ({0: 2}, {0: 1}, 10) => {10: 2; 1}
            ({0: 2; 4; 6}, {0: 1; 3; 5}, 10) => {10: 1; 2; 3; 4; 5; 6}
            ({0 : 1}, {3 : 3}, 4) => {4 : 1;3} // ! 📄来自OpenNARS实际运行过程
            ({4 : 1;3}, {6 : 6}, 7) => {7 : 1;6;3} // ! 📄来自OpenNARS实际运行过程
            ({7 : 1;6;3}, {15 : 15}, 29) => {29 : 1;15;6;3} // ! 📄来自OpenNARS实际运行过程
        }
    }

    /// 测试/evidential_base
    #[test]
    fn evidential_base() {
        macro_once! {
            /// * 🚩模式：【时间戳`stamp!`】 => [证据时间...]
            macro test($( $stamp:tt => [ $($time:expr $(,)? )* ] )*) {
                $(
                    assert_eq!(stamp!($stamp).evidential_base(), [ $($time),* ]);
                )*
            }
            {0: } => []
            {0: 1} => [1]
            {0 : 1;3;4} => [1,3,4]
            {0 : 0xabcd;3;0xfade} => [0xabcd,3,0xfade]
            {7 : 1;6;3} => [1, 6, 3] // ! 📄来自OpenNARS实际运行过程
        }
    }

    /// 测试/base_length
    #[test]
    fn base_length() {
        macro_once! {
            /// * 🚩模式：【时间戳`stamp!`】 => 预期
            macro test($( $stamp:tt => $expected:expr )*) {
                $(
                    assert_eq!(stamp!($stamp).base_length(), $expected);
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

    // * ✅测试/equals 已在先前函数中测试过（断言所必须）
}
