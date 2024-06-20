/// 统一表示「可迭代对象」的接口
pub trait Iterable<T> {
    type Iter<'a>: Iterator<Item = &'a T> + 'a
    where
        Self: 'a,
        T: 'a;
    /// 获取不可变迭代器
    fn iter(&self) -> Self::Iter<'_>;

    type IterMut<'a>: Iterator<Item = &'a mut T>
    where
        Self: 'a,
        T: 'a;
    /// 获取可变迭代器
    fn iter_mut(&mut self) -> Self::IterMut<'_>;
}
