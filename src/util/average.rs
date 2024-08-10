//! 用于从迭代器中获取其代数均值
//! * 🎯推理器`INF`指令呈现「概要信息」时需要

use crate::global::Float;

/// 对[`usize`]迭代器求均值
/// * ✨此处的`U`支持所有「能[转换](Into)到[`usize`]的类型」
/// * ✨此处的「迭代器」支持像数组那样的[`IntoIterator`]泛类型
pub trait AverageUsize<U: Into<usize>>: IntoIterator<Item = U> + Sized {
    /// 对usize迭代器求均值
    /// * 📝【2024-08-10 13:22:07】关键不能省的代码就在`for`内部：迭代时要同时更新两者
    ///   * ❌【2024-08-10 13:26:35】不能使用[`Iterator::unzip`]
    ///     * ⚠️该函数要返回两个能`collect`到的对象
    ///     * ⚠️但不希望除了俩[`usize`]之外的空间分配
    fn average_usize(self) -> Float {
        let mut sum: usize = 0;
        let mut count: usize = 0;
        for n in self {
            sum += n.into();
            count += 1;
        }
        sum as Float / count as Float
    }
}

/// 对所有[`usize`]迭代器实现
impl<U: Into<usize>, T> AverageUsize<U> for T where T: IntoIterator<Item = U> {}
