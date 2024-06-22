//! 存放与内部「映射表」有关的结构

use super::{BagItemLevel, BagItemTable, BagNameTable};
use crate::entity::Item;
use std::{
    collections::{HashMap, VecDeque},
    fmt::Debug,
};

/// 初代「元素映射」实现
#[derive(Debug, Clone, PartialEq)]
pub struct BagNameTableV1<E>(HashMap<String, E>);

impl<E> BagNameTableV1<E> {
    pub fn new() -> Self {
        Self(HashMap::new())
    }
}

/// 默认构造空映射
impl<E> Default for BagNameTableV1<E> {
    #[inline(always)]
    fn default() -> Self {
        Self::new()
    }
}

/// 📜为「散列映射」[`HashMap`]实现「元素映射」
/// * 📝同名方法冲突时，避免「循环调用」的方法：完全限定语法
///   * 🔗<https://rustc-dev-guide.rust-lang.org/method-lookup.html>
///   * ⚠️[`HashMap`]使用[`len`](HashMap::len)而非[`size`](BagNameTable::size)
impl<E: Item> BagNameTable<E> for BagNameTableV1<E> {
    #[inline(always)]
    fn size(&self) -> usize {
        self.0.len()
    }

    #[inline(always)]
    fn get(&self, key: &str) -> Option<&E> {
        self.0.get(key)
    }

    #[inline(always)]
    fn get_mut(&mut self, key: &str) -> Option<&mut E> {
        self.0.get_mut(key)
    }

    #[inline(always)]
    fn put(&mut self, key: &str, item: E) -> Option<E> {
        // * 🚩【2024-05-04 13:06:22】始终尝试插入（在「从无到有」的时候需要）
        self.0.insert(key.to_string(), item)
    }

    #[inline(always)]
    fn remove(&mut self, key: &str) -> Option<E> {
        self.0.remove(key)
    }
}

/// 初代「层级映射」实现
#[derive(Clone, Default, PartialEq)]
pub struct BagItemTableV1(Box<[VecDeque<String>]>);

impl BagItemTableV1 {
    pub fn new(levels: usize) -> Self {
        let inner = vec![VecDeque::new(); levels].into_boxed_slice();
        Self(inner)
    }
}

impl Debug for BagItemTableV1 {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        // 默认做法
        // f.debug_list().entries(self.0.iter()).finish()
        let mut debug_struct = f.debug_struct(std::any::type_name::<Self>());
        for (i, level) in self.0.iter().enumerate() {
            if !level.is_empty() {
                debug_struct.field(&format!("level_{i} ({})", level.len()), &level);
            }
        }
        debug_struct.finish()
    }
}

/// 📜为[`BagItemTableV1`]实现「层级映射」
/// * 🚩基于「元素id」的索引：不存储元素值
///   * 📝Java的情况可被视作`Arc`
impl BagItemTable for BagItemTableV1 // * 需要在「具体值匹配删除」时用到
{
    // 队列
    type Level = VecDeque<String>;

    #[inline(always)]
    fn add_new(&mut self, level: usize) {
        self.0[level] = VecDeque::new()
    }

    #[inline(always)]
    fn get(&self, level: usize) -> &Self::Level {
        &self.0[level]
    }

    #[inline(always)]
    fn get_mut(&mut self, level: usize) -> &mut Self::Level {
        &mut self.0[level]
    }
}

/// 📜为「队列」[`VecDeque`]实现「层级」
impl BagItemLevel for VecDeque<String> // * 需要在「具体值匹配删除」时用到
{
    #[inline(always)]
    fn size(&self) -> usize {
        self.len()
    }

    #[inline(always)]
    fn add(&mut self, key: String) {
        self.push_back(key)
    }

    #[inline(always)]
    fn get(&self, index: usize) -> Option<&String> {
        Self::get(self, index)
    }

    #[inline(always)]
    fn get_mut(&mut self, index: usize) -> Option<&mut String> {
        Self::get_mut(self, index)
    }

    #[inline(always)]
    fn remove_first(&mut self) {
        self.pop_front();
    }

    #[inline(always)]
    fn remove(&mut self, key: &str) {
        if let Some(index) = self.iter().position(|k| k == key) {
            self.remove(index);
        }
    }
}
