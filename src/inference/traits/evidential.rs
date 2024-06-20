//! 复刻抽象的「证据基」特征
//! * 🎯以「时间戳」为基本结构，使「语句」「任务」直接支持其中的功能

use crate::{global::ClockTime, io::symbols::*, nars::DEFAULT_PARAMETERS, util::ToDisplayAndBrief};
use nar_dev_utils::{join, JoinTo};

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
    /// * 💫【2024-05-05 16:40:38】目前对此运作逻辑尚不清楚
    ///
    /// # 📄OpenNARS
    ///
    /// Generate a new stamp for derived sentence by merging the two from parents
    /// the first one is no shorter than the second
    ///
    /// @param first  The first Stamp
    /// @param second The second Stamp
    fn merged_evidential_base(first: &[ClockTime], second: &[ClockTime]) -> Vec<ClockTime> {
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
        let base_length =
            ClockTime::min(first.len() + second.len(), Self::MAX_EVIDENCE_BASE_LENGTH);
        let mut evidential_base = vec![0; base_length];
        while i2 < second.len() && j < base_length {
            evidential_base[j] = first[i1];
            i1 += 1;
            j += 1;
            evidential_base[j] = second[i2];
            i2 += 1;
            j += 1;
        }
        while i1 < first.len() && j < base_length {
            evidential_base[j] = first[i1];
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
