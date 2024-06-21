//! 复刻抽象的「证据基」特征
//! * 🎯以「时间戳」为基本结构，使「语句」「任务」直接支持其中的功能

use crate::{global::ClockTime, io::symbols::*, nars::DEFAULT_PARAMETERS, util::ToDisplayAndBrief};
use nar_dev_utils::{join, JoinTo};
use narsese::lexical::Stamp as LexicalStamp;

/// [`Vec`]集合判等
fn set_vec_eq<T: Clone + Ord>(v1: &[T], v2: &[T]) -> bool {
    v1.len() == v2.len() && v1.iter().all(|i| v2.contains(i))
    // let mut v1 = v1.to_owned();
    // let mut v2 = v2.to_owned();
    // v1.sort();
    // v2.sort();
    // v1 == v2
}

/// 🆕证据（基）
/// * 🎯抽象描述「时间戳」的特征
/// * 📝核心：记载一系列「证据时间」，提供「证据是否重复」方法，以避免「重复推理」
pub trait Evidential: ToDisplayAndBrief {
    /// 🆕提取出的「最大长度」常量
    const MAX_EVIDENCE_BASE_LENGTH: usize = DEFAULT_PARAMETERS.maximum_stamp_length;

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
    #[doc(alias = "base_length")]
    #[inline(always)]
    fn evidence_length(&self) -> usize {
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
    fn merged_evidential_base(first: &[ClockTime], second: &[ClockTime]) -> Vec<ClockTime> {
        /* 📄OpenNARS
        // * 🚩计算新证据基长度：默认长度相加，一定长度后截断
        final int baseLength = Math.min( // * 📝一定程度上允许重复推理：在证据复杂时遗漏一定数据
                base1.length + base2.length,
                maxEvidenceBaseLength);
        // * 🚩计算长短证据基
        final long[] longer, shorter;
        if (base1.length > base2.length) {
            longer = base1;
            shorter = base2;
        } else {
            longer = base2;
            shorter = base1;
        }
        // * 🚩开始构造并填充数据：拉链式填充，1-2-1-2……
        int i1, i2, j;
        i1 = i2 = j = 0;
        final long[] evidentialBase = new long[baseLength];
        while (i2 < shorter.length && j < baseLength) {
            evidentialBase[j] = longer[i1];
            i1++;
            j++;
            evidentialBase[j] = shorter[i2];
            i2++;
            j++;
        }
        // * 🚩2的长度比1小，所以此后随1填充
        while (i1 < longer.length && j < baseLength) {
            evidentialBase[j] = longer[i1];
            i1++;
            j++;
        }
        // * 🚩返回构造好的新证据基
        return evidentialBase; */
        // * 🚩计算新证据基长度：默认长度相加，一定长度后截断
        let base_length =
            ClockTime::min(first.len() + second.len(), Self::MAX_EVIDENCE_BASE_LENGTH);
        // * 🚩计算长短证据基
        let [longer, shorter] = match first.len() > second.len() {
            true => [first, second],
            false => [second, first],
        };
        // * 🚩开始构造并填充数据：拉链式填充，1-2-1-2……
        let mut i1 = 0;
        let mut i2 = 0;
        let mut j = 0;
        let mut evidential_base = vec![0; base_length];
        let shorter_len = shorter.len();
        let longer_len = longer.len();
        while i2 < shorter_len && j < base_length {
            evidential_base[j] = longer[i1];
            i1 += 1;
            j += 1;
            evidential_base[j] = shorter[i2];
            i2 += 1;
            j += 1;
        }
        // * 🚩2的长度比1小，所以此后随1填充
        while i1 < longer_len && j < base_length {
            evidential_base[j] = longer[i1];
            i1 += 1;
            j += 1;
        }
        evidential_base
    }

    /// 🆕判断两个「时间戳」是否含有相同证据
    /// * 🎯用于「概念处理」中的「获取信念」，并反映到后续「推理上下文」的分派中
    ///   * 🎯深层目的：防止重复推理
    /// * 🚩包含相同证据基⇒返回空值
    /// * 🚩【2024-06-20 23:47:41】现在按照OpenNARS改版的来：名之曰「证据上重合」
    fn evidential_overlap(&self, second: &impl Evidential) -> bool {
        self.evidential_base()
            .iter()
            .any(|i| second.evidential_base().contains(i))
    }

    /// 判断是否【在证据上】相等
    fn evidential_eq(&self, other: &impl Evidential) -> bool {
        set_vec_eq(self.evidential_base(), other.evidential_base())
    }

    /// 🆕与OpenNARS改版不同：将其中的「证据基」成分转换为「词法时间戳」
    fn stamp_to_lexical(&self) -> LexicalStamp;

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
    fn stamp_to_display(&self) -> String {
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
    fn __to_display(&self) -> String {
        self.stamp_to_display()
    }
}

#[cfg(test)]
mod tests {
    use nar_dev_utils::macro_once;

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
}
