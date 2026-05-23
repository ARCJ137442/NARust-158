//! 🎯复刻OpenNARS `nars.storage.Distributor`
//!
//! # 分派器
//! * 伪随机数生成器
//!
//! # 📄OpenNARS
//!
//! A pseudo-random number generator, used in Bag.
#![allow(dead_code)] // ! 📌允许「暂且不用」的「分派迭代器」：在最新版Rust中有编译警告

use nar_dev_utils::{manipulate, pipe};
use serde::{Deserialize, Serialize};
use std::fmt::Debug;

/// 伪随机数分派
/// * 🎯用于`Bag`结构的伪随机加权分派
/// * 🎯抽象出「分派」的基本特征
/// * ⚙️其中
///   * `T`作为「分派出的对象」，默认为无符号整数
///   * `I`作为「分派之索引」，默认为无符号整数
pub trait Distribute<T = usize, I = usize> {
    /// 基于当前索引，获取下一个随机数
    /// * 🚩返回一个随机数值
    fn pick(&self, index: I) -> T;

    /// 获取当前索引的下一个索引
    /// * 📌仅依赖于自身「容量」
    fn next(&self, index: I) -> I;

    /// 获取「迭代出所随机元素」的迭代器
    /// * 🎯通用实现
    fn iter(&self, start_i: I) -> Iter<'_, T, I, Self>
    where
        Self: Sized,
    {
        Iter {
            distributor: self,
            index: start_i,
            _mark_t: std::marker::PhantomData,
        }
    }

    /// 获取「迭代出所随机元素」的迭代器（使用「默认索引」开始）
    /// * 🎯通用&默认 实现
    fn iter_default(&self) -> Iter<'_, T, I, Self>
    where
        I: Default,
        Self: Sized,
    {
        self.iter(I::default())
    }

    /// 获取「迭代出所随机元素」的迭代器（使用「默认索引」开始）
    /// * 🎯通用&默认 实现
    fn take_n(&self, start_i: I, n: usize) -> impl Iterator<Item = T>
    where
        I: Copy,
        T: Copy,
        Self: Sized,
    {
        self.iter(start_i).take(n)
    }
}

/// 迭代「分派者」的迭代器
pub struct Iter<'a, T, I, D>
where
    D: Distribute<T, I>,
{
    distributor: &'a D,
    index: I,
    _mark_t: std::marker::PhantomData<T>,
}

/// 实现迭代器
impl<T, I, D> Iterator for Iter<'_, T, I, D>
where
    T: Copy,
    I: Copy,
    D: Distribute<T, I>,
{
    type Item = T;

    fn next(&mut self) -> Option<Self::Item> {
        let result = Some(self.distributor.pick(self.index));
        self.index = self.distributor.next(self.index);
        result
    }
}

/// 伪随机数生成器 初代实现
/// * 🎯实现一个[`Distribute<usize, usize>`](Distribute)
/// * 🎯以更Rusty的方式复刻OpenNARS之Distributor
///   * ⚡性能
///   * ✨通用性
#[derive(Clone, PartialEq, Eq, Hash)]
pub struct Distributor {
    /// 🆕缓存的「随机范围」量
    /// * 🚩表示随机数的样本空间大小
    /// * 🎯用于迭代器
    range: usize,

    /// 伪随机索引「顺序」
    /// * 🚩现在使用直接对应「运行时定长数组」的`Box<[T]>`
    ///   * ✅绕开原先`[T; N]`中「`N`只能在运行时确定」的问题
    ///   * 📝Rust中[`Vec`]附带一个`capacity`以便实现「变长数组」，但实际上只需要一块恒定的内存（指针）
    ///   * 🔗<https://johnbsmith.github.io/Informatik/Rust/Dateien/Rust-container-cheat-sheet.pdf>
    ///
    /// # 📄OpenNARS
    ///
    /// Shuffled sequence of index numbers
    order: Box<[usize]>,

    /// 🆕伪随机索引「下一个」
    /// * 🚩现在使用直接对应「运行时定长数组」的`Box<[T]>`
    ///   * ✅绕开原先`[T; N]`中「`N`只能在运行时确定」的问题
    ///   * 📝Rust中[`Vec`]附带一个`capacity`以便实现「变长数组」，但实际上只需要一块恒定的内存（指针）
    ///   * 🔗<https://johnbsmith.github.io/Informatik/Rust/Dateien/Rust-container-cheat-sheet.pdf>
    /// * 🎯用于`next`函数
    /// * 🚩一个大小为[`Self::capacity`]的数组
    /// * ✨直接通过「硬缓存」的方式，省掉一个变量
    next: Box<[usize]>,
}

impl Distributor {
    /// 构造函数
    pub fn new(range: usize) -> Self {
        // 推导容量与排序
        let (capacity, order) = Self::range_to_capacity_and_order(range);
        // 推导缓存`next`函数值
        let next = Self::capacity_to_next(capacity);
        // 构造 & 返回
        Self { range, order, next }
    }

    /// 从「范围」推导出「下一个」映射
    /// * 🚩【2024-05-01 21:12:46】现在使用固定的`Box<[usize]>`代表「运行时定长数组」
    pub fn capacity_to_next(capacity: usize) -> Box<[usize]> {
        manipulate!(
            // 从0到capacity-1
            (1..capacity).collect::<Vec<_>>()
            // 最后一个必是0
            => .push(0)
        )
        .into_boxed_slice()
        // * 🚩等价代码
        // list![
        //     ((i + 1) % capacity)
        //     for i in (0..capacity)
        // ]
    }

    /// 从「范围」推导出「容量」与「排序」
    /// * 📄直接源自OpenNARS
    pub fn range_to_capacity_and_order(range: usize) -> (usize, Box<[usize]>) {
        // 计算整体容量
        let capacity: usize = range * (range + 1) / 2;
        // * 🚩先创建指定容量的变长数组
        let mut order = vec![0; capacity].into_boxed_slice();
        // * 🚩开始填充内容
        let mut index = capacity - 1;
        for rank in (1..=range).rev() {
            for _ in 0..rank {
                // 变换位置
                index = ((capacity / rank) + index) % capacity;
                while order[index] > 0 {
                    index += 1;
                    index %= capacity;
                }
                // 安插
                order[index] = rank;
            }
        }
        for order_i in order.iter_mut() {
            *order_i -= 1;
        }
        // 最后转换成Box
        (capacity, order)
    }

    /// 获取其内部「容量」
    pub fn capacity(&self) -> usize {
        self.order.len()
    }

    // ! ❌【2024-05-04 12:54:08】无法实现「完全的格式化」
    // * 📄`std::fmt::Formatter::new(&b);`
    // * 📄"use of unstable library feature 'fmt_internals': internal to standard library"
    // fn fmt_full(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
    //     let b = String::new();
    //     let f = std::fmt::Formatter::new(&b);
    //     f.debug_struct("Distributor")
    //         .field("range", &self.range)
    //         .field("order", &self.order)
    //         .field("next", &self.next)
    //         .finish()
    // }
}

/// 用于在[`Debug`]打印时能简要显示信息，但又能复用Rust的格式化器
#[derive(Clone)]
struct RawDebug(String);

impl Debug for RawDebug {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}

impl Debug for Distributor {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Distributor")
            .field("range", &self.range)
            .field("order", &RawDebug(debug_truncated_arr(&self.order, 50)))
            .field(
                "next",
                &RawDebug(format!(
                    "[Array `next[i] = i + 1 % {}` with len = {}]",
                    self.capacity(),
                    self.next.len()
                )),
            )
            .finish()
    }
}

/// 将数组截断展示，对多余的内容用「省略号+长度」代替
fn debug_truncated_arr<T: Debug>(arr: &[T], max_len: usize) -> String {
    if arr.len() <= max_len {
        format!("{:?}", arr)
    } else {
        let mut s = format!("{:?}", &arr[..max_len]);
        s.pop(); // * 🚩换掉
        s.push_str(&format!(", ... (len = {})]", arr.len()));
        s
    }
}

/// 实现「分派」特征
impl Distribute for Distributor {
    /// # Panics
    ///
    /// ⚠️数组越界可能会`panic`
    fn pick(&self, index: usize) -> usize {
        self.order[index]
    }

    /// # Panics
    ///
    /// ⚠️数组越界可能会`panic`
    fn next(&self, index: usize) -> usize {
        self.next[index]
    }
}

/// 用于序列反序列化的专用数据结构
/// * 💡整个「分派器」的序列反序列化 先通过此类型，再由此构造
/// * 📌在此享用[`serde`]中自动派生的好处
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
struct DistributorSerde {
    /// 只有一个必要的参数
    range: usize,
}

/// 内部类型到外部类型的相互转换
impl From<&Distributor> for DistributorSerde {
    fn from(d: &Distributor) -> Self {
        Self { range: d.range }
    }
}

/// 外部类型到内部类型的相互转换
impl From<DistributorSerde> for Distributor {
    fn from(d: DistributorSerde) -> Self {
        Self::new(d.range)
    }
}

/// 定制的「序列化」方法
/// ✨只需处理一个range属性，其它均不用
impl Serialize for Distributor {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        pipe! {
            self
            => DistributorSerde::from
            => .serialize(serializer)
        }
    }
}

/// 定制的「反序列化」方法
/// ✨只需处理一个range属性，其它均不用
impl<'de> Deserialize<'de> for Distributor {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        pipe! {
            deserializer
            => DistributorSerde::deserialize => {?}#
            => Distributor::from
            => Ok
        }
    }
}

/// 单元测试
#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::HashMap;

    /// 测试「截断的数组展示」
    #[test]
    fn test_debug_truncated_arr() {
        let arr = [1, 2, 3, 4, 5, 6, 7, 8, 9, 10];
        assert_eq!(
            debug_truncated_arr(&arr, 5),
            // 前五个元素……长度为10
            "[1, 2, 3, 4, 5, ... (len = 10)]"
        );
    }

    /// 测试分派器
    #[test]
    fn test_distributor() {
        // 测试范围
        let range = 10..=20;
        // 范围测试
        for n in range {
            _test_distributor(n);
        }
    }

    /// 含参（大小）
    fn _test_distributor(n: usize) {
        let d = Distributor::new(n);
        println!("d = {d:?}");
        // 系列测试 //
        // next
        _test_next(&d);
        // 总体权重
        _test_weight(&_weights(d.take_n(0, d.capacity())));
        _test_local_weights(&d, d.range);
    }

    /// next测试
    fn _test_next(d: &Distributor) {
        let c = d.capacity();
        // 没有「取模约束」时
        for i in 0..(c - 1) {
            assert_eq!(d.next(i), i + 1);
        }
        // 取模约束
        assert_eq!(d.next(c - 1), 0);
    }

    /// 局部权重测试
    /// * 🎯分派器在各个索引之间，需要「整体权重与局部权重相似」
    ///   * 权重不能随「分派次数」的变更而变更
    /// * 🚩固定「扫描区间」的大小为整个capacity，在n×capacity的结果中扫描
    fn _test_local_weights(d: &Distributor, n: usize) {
        let c = d.capacity();
        let l = c * n;
        let results = d.iter_default().take(l).collect::<Vec<_>>();
        for i in 0..(l - c) {
            let slice = &results[i..(i + c)];
            _test_weight(&_weights(slice.iter().copied()));
        }
    }

    /// 测试分派器的权重
    /// * 🎯越大的索引应该有越大的权重
    fn _test_weight(weights: &HashMap<usize, usize>) {
        let mut weights_arr = weights.iter().map(|(k, v)| (*k, *v)).collect::<Vec<_>>();
        weights_arr.sort_by_key(|a| a.0);
        for (i, (term, w)) in weights_arr.iter().enumerate() {
            if i > 0 {
                let (previous, w_p) = weights_arr[i - 1];
                // 必须顺序一致：越大的索引具有越大的权重
                assert_eq!(
                    *term < previous,
                    *w < w_p,
                    "error with weights = {:?} and (term, w) = ({term}, {w}), (previous, w_p) = ({previous}, {w_p}))",
                    &weights_arr
                );
            }
        }
    }

    /// 获取分派器各个索引对应的权重
    fn _weights(term_iter: impl Iterator<Item = usize>) -> HashMap<usize, usize> {
        let mut weights = HashMap::new();

        for t in term_iter {
            // 自增 or 插入1
            weights.entry(t).and_modify(|u| *u += 1).or_insert(1);
        }

        weights
    }

    /// 测试序列化与反序列化
    #[test]
    fn serde() {
        /// 测试用的[`Distributor::range`]范围
        const TEST_RANGE: std::ops::Range<usize> = 10..100;

        for range in TEST_RANGE {
            let d0 = Distributor::new(range);
            let s = serde_json::to_string(&d0).unwrap();
            let d = serde_json::from_str::<Distributor>(&s).unwrap();
            assert_eq!(d0, d);
        }
    }
}
