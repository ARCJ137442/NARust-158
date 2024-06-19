//! 🆕新的「缓冲区」类型
//! * 📌复刻自OpenNARS改版

use crate::util::Iterable;

/// 🆕新的 缓冲区 抽象类型
/// * 📌本质上是一个先进先出队列
/// * 🚩抽象的「添加元素」「弹出元素」
pub trait Buffer<T>: Iterable<T> {
    /// 【内部】添加元素（到队尾）
    fn __push(&mut self, item: T);

    /// 【内部】弹出元素（队首）
    fn __pop(&mut self) -> Option<T>;

    /// 获取已有元素数量
    fn size(&self) -> usize;

    /// 获取容量
    fn capacity(&self) -> usize;

    /// 添加元素（到队尾）
    /// * 🚩先添加元素到队尾，再弹出队首元素
    fn add(&mut self, new_item: T) -> Option<T> {
        // * 🚩添加元素到队尾
        self.__push(new_item);
        // * 🚩缓冲区机制 | 📝断言：只在变动时处理
        match self.size() > self.capacity() {
            true => self.__pop(), // FIFO
            false => None,
        }
    }
}

/// 🆕使用「变长数组」实现的「缓冲区」类型
#[derive(Debug, Clone)]
pub struct ArrayBuffer<T> {
    /// 内部数组
    inner: Vec<T>,

    /// 缓冲区容量
    capacity: usize,
}

impl<T> ArrayBuffer<T> {
    /// 构造函数：初始化一个容量固定的空缓冲区
    pub fn new(capacity: usize) -> Self {
        Self {
            inner: vec![],
            capacity,
        }
    }
}

/// 实现「可迭代对象」
impl<T> Iterable<T> for ArrayBuffer<T> {
    type Iter<'a> = core::slice::Iter<'a, T>
    where
        Self: 'a,
        T: 'a;

    fn iter(&self) -> Self::Iter<'_> {
        self.inner.iter()
    }

    type IterMut<'a> = core::slice::IterMut<'a, T>
    where
        Self: 'a,
        T: 'a;

    fn iter_mut(&mut self) -> Self::IterMut<'_> {
        self.inner.iter_mut()
    }
}

/// 实现「缓冲区」
impl<T> Buffer<T> for ArrayBuffer<T> {
    fn __push(&mut self, item: T) {
        self.inner.push(item)
    }

    fn __pop(&mut self) -> Option<T> {
        self.inner.pop()
    }

    fn size(&self) -> usize {
        self.inner.len()
    }

    fn capacity(&self) -> usize {
        self.capacity
    }
}
