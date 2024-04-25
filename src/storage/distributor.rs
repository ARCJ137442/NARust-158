//! # 分发器
//! * 伪随机数生成器
//!
//! # 📄OpenNARS `nars.storage.Distributor`
//!
//! A pseudo-random number generator, used in Bag.

/// 伪随机数生成，用于`Bag`结构
/// * 🎯抽象出「分发」的基本特征
/// * ⚙️其中
///   * `T`作为「分发出的对象」，默认为无符号整数
///   * `I`作为「分发之索引」，默认为无符号整数
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

/// 伪随机数生成器
/// * 🎯实现一个[`Distribute<usize, usize>`](Distribute)
/// * 🎯以更Rusty的方式复刻OpenNARS之Distributor
///   * ⚡性能
///   * ✨通用性
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct Distributor {
    order: Vec<usize>,
    range: usize,
    capacity: usize,
}

impl Distributor {
    /// 构造函数
    pub fn new(range: usize) -> Self {
        // 推导容量与排序
        let (capacity, order) = Self::capacity_and_order_from_range(range);
        // 构造 & 返回
        Self {
            range,
            order,
            capacity,
        }
    }

    /// 从「范围」推导出「容量」与「排序」
    /// * 📄直接源自OpenNARS
    pub fn capacity_and_order_from_range(range: usize) -> (usize, Vec<usize>) {
        let capacity: usize = range * (range + 1) / 2;
        let mut order = vec![0; capacity];
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
        (capacity, order)
    }

    /// 获取其随机的范围
    /// * 📌会随机出的量之区间
    pub fn range(&self) -> std::ops::Range<usize> {
        0..self.range
    }
}

/// 实现「分派」特征
impl Distribute for Distributor {
    fn pick(&self, index: usize) -> usize {
        self.order[index]
    }

    fn next(&self, index: usize) -> usize {
        (index + 1) % self.capacity
    }
}
// pub struct Distributor<const CAPACITY: usize> {
//     /// 内部的排序
//     /// * ⚠️只能直接上常量，不能走常量表达式
//     order: [usize; CAPACITY],
// }

// impl<const CAPACITY: usize> Distributor<CAPACITY> {
//     pub fn capacity(&self) -> usize {
//         CAPACITY
//     }

//     pub fn range() -> usize {
//         range_from_capacity::<CAPACITY>()
//     }

//     pub fn new() -> Self {
//         let mut order = [0; CAPACITY];
//         let range = Self::range();
//         let mut index = CAPACITY - 1;
//         for rank in ((range + 1)..1).rev() {
//             for _ in 0..rank {
//                 // 变换位置
//                 index = ((CAPACITY / rank) + index) % CAPACITY;
//                 while order[index] > 0 {
//                     index = Self::next(index);
//                 }
//                 // 安插
//                 order[index] = rank;
//             }
//         }
//         for order_i in order.iter_mut() {
//             *order_i -= 1;
//         }
//         // 构造 & 返回
//         Self { order }
//     }
// }

// fn sqrt_usize_floor(u: usize) -> usize {
//     match u {
//         0..=1 => u,
//         2 => 1,
//         _ => {
//             for r in 0..u {
//                 if r * r > u {
//                     return r - 1;
//                 }
//             }
//             0
//         }
//     }
// }

// pub fn range_from_capacity<const CAPACITY: usize>() -> usize {
//     // r^2 + r - 2c = 0
//     // delta = 1 + 4*c
//     // r = (-1 + sqrt(1 + 4*c)) / 2
//     sqrt_usize_floor(1 + 4 * CAPACITY).saturating_sub(1) / 2
// }

// pub fn capacity_from_range<const RANGE: usize>() -> usize {
//     // r^2 + r - 2c = 0
//     // delta = 1 + 4*c
//     // r = (-1 + sqrt(1 + 4*c)) / 2
//     RANGE * (RANGE + 1) / 2
// }

// impl<const CAPACITY: usize> Default for Distributor<CAPACITY> {
//     fn default() -> Self {
//         Self::new()
//     }
// }

// impl<const CAPACITY: usize> Distribute for Distributor<CAPACITY> {
//     fn pick(&self, index: usize) -> usize {
//         self.order[index]
//     }

//     fn next(index: usize) -> usize {
//         (index + 1) % CAPACITY
//     }
// }

/// 单元测试
#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::HashMap;

    /// 测试分派器
    #[test]
    fn test_distributor() {
        let d = Distributor::new(10);
        println!("d = {d:?}");
        // 系列测试（总体权重）
        _test_weight(&_weights(d.take_n(0, d.capacity)));
        _test_local_weights(&d, d.range);
    }

    /// 局部权重测试
    /// * 🎯分派器在各个索引之间，需要「整体权重与局部权重相似」
    ///   * 权重不能随「分派次数」的变更而变更
    /// * 🚩固定「扫描区间」的大小为整个capacity，在n×capacity的结果中扫描
    fn _test_local_weights(d: &Distributor, n: usize) {
        let c = d.capacity;
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
        weights_arr.sort_by(|a, b| a.0.cmp(&b.0));
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
}
