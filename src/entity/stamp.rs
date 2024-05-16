//! 🎯复刻OpenNARS `nars.entity.Stamp`
//! * ✅【2024-05-05 15:50:54】基本特征功能复刻完成
//! * ✅【2024-05-05 17:03:34】单元测试初步完成

use crate::{
    global::ClockTime,
    io::symbols::{STAMP_CLOSER, STAMP_OPENER, STAMP_SEPARATOR, STAMP_STARTER},
    nars::DEFAULT_PARAMETERS,
    ToDisplayAndBrief,
};
use anyhow::Result;
use nar_dev_utils::{join, JoinTo};
use narsese::lexical::Stamp as LexicalStamp;
use std::hash::{Hash, Hasher};

/// 模拟`nars.entity.Stamp`
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
pub trait Stamp: ToDisplayAndBrief + PartialEq {
    // ! ❌【2024-05-05 14:07:05】不模拟`Stamp.currentSerial`，理由同上「拒绝全局静态变量」

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

    /// 模拟`toString`
    /// * 🚩【2024-05-08 22:12:42】现在鉴于实际情况，仍然实现`toString`、`toStringBrief`方法
    ///   * 🚩具体方案：实现一个统一的、内部的、默认的`__to_display(_brief)`，再通过「手动嫁接」完成最小成本实现
    /// * ⚠️🆕具体格式化结果相比OpenNARS**没有头尾空白**
    ///
    /// # 📄OpenNARS
    ///
    /// Get a String form of the Stamp for display
    /// Format: {creationTime [: eventTime] : evidentialBase}
    ///
    /// @return The Stamp as a String
    fn __to_display(&self) -> String {
        /* 📄OpenNARS源码：
        StringBuilder buffer = new StringBuilder(" " + Symbols.STAMP_OPENER + creationTime);
        buffer.append(" ").append(Symbols.STAMP_STARTER).append(" ");
        for (int i = 0; i < baseLength; i++) {
            buffer.append(Long.toString(evidentialBase[i]));
            if (i < (baseLength - 1)) {
                buffer.append(Symbols.STAMP_SEPARATOR);
            } else {
                buffer.append(Symbols.STAMP_CLOSER).append(" ");
            }
        }
        return buffer.toString(); */
        join!(
            // 生成头部：`{0:`
            => STAMP_OPENER.to_string()
            => {# self.creation_time()}
            => ' '
            => STAMP_STARTER
            => ' '
            // 循环迭代加入中部：`0;1;2`
            => self.evidential_base()
                .iter().map(ToString::to_string) // 迭代器转换为字符串
                .join_to_new(STAMP_SEPARATOR) // 加入到新字串中
            // 最终加入尾部：`}`
            => STAMP_CLOSER
        )
    }

    /// 🆕判断两个「时间戳」是否含有相同证据
    /// * 🎯用于「概念处理」中的「获取信念」，并反映到后续「推理上下文」的分派中
    ///   * 🎯深层目的：防止重复推理
    /// * 🚩包含相同证据基⇒返回空值
    fn have_common_evidence(&self, second: &impl Stamp) -> bool {
        self.evidential_base()
            .iter()
            .any(|i| second.evidential_base().contains(i))
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
pub trait StampConcrete: Stamp + Clone + Hash + PartialEq {
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
    #[doc(alias = "from_make")]
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
        if first.have_common_evidence(second) {
            return None;
        }
        match first.base_length() > second.base_length() {
            true => Some(Self::__from_merge(first, second, time)),
            false => Some(Self::__from_merge(second, first, time)),
        }
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
    fn __hash<H: Hasher>(&self, state: &mut H) {
        self.evidential_base().hash(state);
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
    #[inline(always)]
    #[doc(alias = "from_input")]
    fn from_lexical(_: LexicalStamp, current_serial: ClockTime, time: ClockTime) -> Result<Self> {
        Ok(Self::with_time(current_serial, time))
    }

    /// 🆕自身到「词法」的转换
    /// * 🎯标准Narsese输出需要（Narsese内容）
    /// * 🚩【2024-05-12 14:48:31】此处跟随OpenNARS，使用空字串
    ///   * 时态暂均为「永恒」
    #[inline(always)]
    fn to_lexical(&self) -> LexicalStamp {
        LexicalStamp::new()
    }
}

/// 初代实现
mod impl_v1 {
    use super::*;
    use crate::__impl_to_display_and_display;

    /// [时间戳](Stamp)初代实现
    #[derive(Debug, Clone)]
    pub struct StampV1 {
        evidential_base: Box<[ClockTime]>,
        creation_time: ClockTime,
    }

    /// 模拟`equals`
    impl PartialEq for StampV1 {
        #[inline(always)]
        fn eq(&self, other: &Self) -> bool {
            self.equals(other)
        }
    }

    /// 模拟`hashCode`
    impl Hash for StampV1 {
        #[inline(always)]
        fn hash<H: Hasher>(&self, state: &mut H) {
            self.__hash(state)
        }
    }

    __impl_to_display_and_display! {
        // * 🚩【2024-05-09 00:37:24】只实现一个方法（其它默认）
        @(to_display;;)
        StampV1 as Stamp
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

    /// 初代「时间戳」的快捷构造宏
    /// * 🚩模式：{发生时间: 证据1; 证据2; ...}
    #[macro_export]
    macro_rules! stamp {
        ({ $creation_time:tt : $($evidence:expr $(;)? )* }) => {
            StampV1::__new(
                $creation_time,
                &[ $( $evidence ),* ]
            )
        };
    }
}
pub use impl_v1::*;

/// 单元测试
#[cfg(test)]
mod tests {
    use super::*;
    use crate::stamp;
    use nar_dev_utils::macro_once;

    /// 测试用「时间戳」类型
    type S = StampV1;

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
                (@SINGLE ( $s1:tt, $s2:tt, $time:expr ) => None ) => {
                    assert_s_eq!(Option S::from_merge(&stamp!($s1), &stamp!($s2), $time), None::<S>);
                }
                // 有结果
                (@SINGLE ( $s1:tt, $s2:tt, $time:expr ) => $stamp:tt ) => {
                    assert_s_eq!(Option S::from_merge(&stamp!($s1), &stamp!($s2), $time), Some(stamp!($stamp)));
                }
                // 总模式
                ( $( $parameters:tt => $expected:tt )* ) => {
                    $( test!( @SINGLE $parameters => $expected ); )*
                }
            }
            ({0: 1}, {0: 1}, 1) => None
            ({0: 1}, {0: 2}, 10) => {10: 2; 1}
            ({0: 2}, {0: 1}, 10) => {10: 1; 2}
            ({0: 2; 4; 6}, {0: 1; 3; 5}, 10) => {10: 1; 2; 3; 4; 5; 6}
            ({1 : 2}, {0 : 1}, 2) => {2 : 1;2} // ! 📄来自OpenNARS实际运行过程 | ⚠️注意：需要是传入`Stamp.make`处的参数（可能中途调换位置）
            ({13 : 3}, {13 : 1;2}, 13) => {13 : 1;3;2} // ! 📄来自OpenNARS实际运行过程
            ({34 : 4}, {14 : 1;3;2}, 35) => {35 : 1;4;3;2} // ! 📄来自OpenNARS实际运行过程
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

    /// 测试/equals
    #[test]
    fn equals() {
        macro_once! {
            /// * 🚩模式：(【时间戳1`stamp!`】, 【时间戳2`stamp!`】, 创建时间) => 预期【时间戳`stamp!`/None】
            macro test( $( ($s1:tt, $s2:tt) => $expected:tt )* ) {
                $(
                    // 验证「相等」符合预期
                    assert_eq!(stamp!($s1).equals(&stamp!($s2)), $expected);
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
