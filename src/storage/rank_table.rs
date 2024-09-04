//! 🆕新的「排行表」类型
//! * 📌复刻自OpenNARS改版

use crate::{global::Float, util::Iterable};
use nar_dev_utils::unwrap_or_return;
use serde::{Deserialize, Serialize};

/// 🆕排行表 抽象类型
/// * 🎯按照一个抽象的「排行函数」确定内部元素的位置
/// * 🎯用于「概念」的「信念表」
/// * 📌其中对「元素遍历顺序」要遵循「优先级从高到低」的原则
///   * ⚠️遍历出的索引要能通过[`RankTable::__get`]方法回查（与之一致）
///   * ℹ️亦即：`self.iter().enumerate().all(|(i, e)| self.__get(i) == e)`
pub trait RankTable<T>: Iterable<T> {
    /// 表内已有元素数量
    fn size(&self) -> usize;

    /// 判断是否为空
    fn is_empty(&self) -> bool {
        self.size() == 0
    }

    /// 表内最大元素数量（容量）
    fn capacity(&self) -> usize;

    /// 【核心】排行函数
    fn rank(&self, item: &T) -> Float;

    /// 【内部】获取指定位置的元素
    fn __get(&self, index: usize) -> Option<&T>;

    /// 【内部】在某处插入元素
    fn __insert(&mut self, index: usize, item: T);

    /// 【内部】在某处插入元素（末尾）
    /// * 📌即改版重载的方法`__insert(E newElement)`
    fn __push(&mut self, item: T);

    /// 【内部】弹出（末尾元素）
    fn __pop(&mut self) -> Option<T>;

    /// 【核心】计算将插入位置
    /// * 🚩需要获取元素排行，并判断新增元素「是否兼容」
    fn rank_index_to_add(&self, item: &T) -> Option<usize> {
        // * 🚩按排行计算排行应处在的位置
        let rank_new = self.rank(item);
        for (i_to_add, existed) in self.iter().enumerate() {
            // * 🚩获取待比较的排行
            let rank_existed = self.rank(existed);
            // * 🚩总体顺序：从大到小（一旦比当前的大，那就在前边插入）
            if rank_new >= rank_existed {
                // * 🚩检查是否兼容
                return match self.is_compatible_to_add(item, existed) {
                    // * 🚩标记待插入的位置
                    true => Some(i_to_add),
                    // * 🚩不兼容
                    false => None,
                };
            }
        }
        // * 🚩否则⇒插入末尾
        Some(self.size())
    }

    /// 检查新元素是否兼容
    /// 🎯用于「筛除重复元素」如「重复语句」
    fn is_compatible_to_add(&self, new_item: &T, existed_item: &T) -> bool;

    /// 加入元素
    /// * 🚩成功加入⇒返回null/旧元素；加入失败⇒返回待加入的元素
    fn add(&mut self, new_item: T) -> Option<T> {
        let i_to_add = unwrap_or_return! {
            // * 🚩将新元素插入到「排行表」的索引i位置（可以是末尾）
            ?self.rank_index_to_add(&new_item)
            // * 🚩添加失败 ⇒ 原路返回元素
            => Some(new_item)
        };
        let table_size = self.size();
        // * 🚩根据「是否在末尾」「是否超出容量」判断
        match (i_to_add == table_size, table_size == self.capacity()) {
            // * 🚩末尾 & 超出容量 ⇒ 添加失败
            (true, true) => return Some(new_item),
            // * 🚩末尾 & 未超出容量 ⇒ 追加
            (true, false) => self.__push(new_item),
            // * 🚩非末尾 ⇒ 插入中间
            (false, _) => self.__insert(i_to_add, new_item),
        }

        // * 🚩排行表溢出 | 📌一次只增加一个
        let new_size = self.size();
        match new_size > self.capacity() {
            true => {
                // * 🚩缩减容量到限定的容量
                debug_assert!(
                    new_size - self.capacity() == 1,
                    "【2024-06-08 10:07:31】断言：一次只会添加一个，并且容量不会突然变化"
                );
                // * 🚩从末尾移除，返回移除后的元素
                self.__pop()
            }
            // * 🚩最终添加成功，且没有排行被移除
            false => None,
        }
    }
}

/// 🆕「排行函数」类型
pub type RankF<T> = for<'a> fn(&'a T) -> Float;

/// 🆕「兼容性检查」类型
pub type IsCompatibleToAddF<T> = for<'a> fn(&'a T, &'a T) -> bool;

/// 🆕使用「变长数组」实现的「排行表」类型
/// * 📌直接使用函数指针类型
/// * ❓包含函数指针类型「不大幅破坏原有结构」下的序列化问题
///   * ❌【2024-08-11 22:27:35】回退为特征的方法不可行
///     * ⚠️模拟继承式分派 不可取：多很多有关字段的方法
///   * 💫【2024-08-11 22:43:01】基于「中间类型」的方式难走通
///     * ❌内部存储有复杂的堆分配对象，而序列化时只拿取引用——中间类型的引用不`clone`就拿不到
///   * 🚩【2024-08-11 22:35:56】目前用「先反序列化到白板，再由调用处填充」的方式实现
#[derive(Debug, PartialEq, Serialize, Deserialize)]
pub struct ArrayRankTable<T> {
    /// 内部数组
    inner: Vec<T>,

    /// 排行表容量
    capacity: usize,

    /// 「计算排行」函数（函数指针）
    #[serde(skip)]
    #[serde(default = "ArrayRankTable::<T>::default_rank_f")]
    rank_f: RankF<T>,

    /// 「计算是否可兼容以添加」（函数指针）
    #[serde(skip)]
    // ! ❌不能`skip_serialize`+`deserialize_with`：区别在于「反序列化时仍然检查字段」
    #[serde(default = "ArrayRankTable::<T>::default_is_compatible_to_add_f")]
    is_compatible_to_add_f: IsCompatibleToAddF<T>,
}

impl<T> ArrayRankTable<T> {
    /// 构造函数：创建一个空排行表，用上两个函数指针
    pub fn new(
        capacity: usize,
        rank_f: RankF<T>,
        is_compatible_to_add_f: IsCompatibleToAddF<T>,
    ) -> Self {
        Self {
            inner: vec![],
            capacity,
            rank_f,
            is_compatible_to_add_f,
        }
    }
}

impl<T> Iterable<T> for ArrayRankTable<T> {
    type Iter<'a> = core::slice::Iter<'a,T>
    where
        Self: 'a,
        T: 'a;

    fn iter(&self) -> Self::Iter<'_> {
        self.inner.iter()
    }

    type IterMut<'a>= core::slice::IterMut<'a,T>
    where
        Self: 'a,
        T: 'a;

    fn iter_mut(&mut self) -> Self::IterMut<'_> {
        self.inner.iter_mut()
    }
}

impl<T> RankTable<T> for ArrayRankTable<T> {
    fn rank(&self, item: &T) -> Float {
        (self.rank_f)(item)
    }

    fn is_compatible_to_add(&self, new_item: &T, existed_item: &T) -> bool {
        (self.is_compatible_to_add_f)(new_item, existed_item)
    }

    fn size(&self) -> usize {
        self.inner.len()
    }

    fn capacity(&self) -> usize {
        self.capacity
    }

    fn __get(&self, index: usize) -> Option<&T> {
        self.inner.get(index)
    }

    fn __insert(&mut self, index: usize, item: T) {
        self.inner.insert(index, item)
    }

    fn __push(&mut self, item: T) {
        self.inner.push(item)
    }

    fn __pop(&mut self) -> Option<T> {
        self.inner.pop()
    }
}

impl<T> ArrayRankTable<T> {
    /// （公开API）覆盖所有的函数指针
    /// * 🎯序列反序列化：反序列化到「白板对象」后，覆写其中的模板函数指针
    pub fn override_fn(&mut self, rank_f: RankF<T>, is_compatible_to_add_f: IsCompatibleToAddF<T>) {
        self.rank_f = rank_f;
        self.is_compatible_to_add_f = is_compatible_to_add_f;
    }

    /// 用于序列化产生「空白函数」
    pub fn default_rank_f() -> RankF<T> {
        /// 🚩【2024-08-11 22:37:05】目前将默认返回值定位为「默认panic」
        /// * 💭【2024-08-11 22:37:23】尽可能提前暴露错误（何处未及时设置函数指针），而非让值蒙混过关
        fn rank_f<T>(_: &T) -> Float {
            panic!("未完全反序列化的`rank_f`函数指针")
        }
        rank_f
    }

    /// 用于序列化产生「空白函数」
    pub fn default_is_compatible_to_add_f() -> IsCompatibleToAddF<T> {
        /// 🚩【2024-08-11 22:37:05】目前将默认返回值定位为「默认panic」
        /// * 💭【2024-08-11 22:37:23】尽可能提前暴露错误（何处未及时设置函数指针），而非让值蒙混过关
        fn is_compatible_to_add_f<T>(_: &T, _: &T) -> bool {
            panic!("未完全反序列化的`is_compatible_to_add_f`函数指针")
        }
        is_compatible_to_add_f
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{ok, util::AResult};

    #[test]
    fn ser() -> AResult {
        let table = ArrayRankTable::new(
            10,
            {
                fn rank_f(item: &i32) -> Float {
                    *item as Float
                }
                rank_f
            },
            {
                fn is_compatible_to_add_f(new_item: &i32, existed_item: &i32) -> bool {
                    *new_item > *existed_item
                }
                is_compatible_to_add_f
            },
        );
        let s = serde_json::to_string(&table)?;

        println!("{table:?} => {s}");

        ok!()
    }
}
