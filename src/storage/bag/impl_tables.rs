//! 存放与内部「映射表」有关的结构

use crate::entity::Item;
use std::{
    collections::{HashMap, VecDeque},
    fmt::Debug,
};

/// 初代「元素映射」实现
#[derive(Debug, Clone, PartialEq)]
pub struct BagNameTable<E>(HashMap<String, NameValue<E>>);

/// 「元素映射」最终从「名称」映射到的结构
/// * 🎯允许「一个键对多个值」
///   * 💭后续可以将预算值加入进去
///   * ⚠️不允许外部调用者随意通过「修改物品优先级」变更「所在层级信息」
pub type NameValue<E> = (E, usize);

impl<E> BagNameTable<E> {
    pub fn new() -> Self {
        Self(HashMap::new())
    }
}

/// 默认构造空映射
impl<E> Default for BagNameTable<E> {
    fn default() -> Self {
        Self::new()
    }
}

/// 📜为「散列映射」[`HashMap`]实现「元素映射」
/// * 📝同名方法冲突时，避免「循环调用」的方法：完全限定语法
///   * 🔗<https://rustc-dev-guide.rust-lang.org/method-lookup.html>
///   * ⚠️[`HashMap`]使用[`len`](HashMap::len)而非[`size`](BagNameTable::size)
impl<E: Item> BagNameTable<E> {
    /// 模拟`Bag.nameTable.size`方法
    pub fn size(&self) -> usize {
        self.0.len()
    }

    /// 模拟`Bag.nameTable.containsValue`方法
    /// * 🎯预期是「在映射查找值；找到⇒Some，没找到⇒None」
    /// * 🚩【2024-06-30 18:28:02】现在获取指定键下的物品和层级
    ///   * 🎯防止「物品在袋内优先级变化导致mass计算错误」的问题
    pub fn get(&self, key: &str) -> Option<&NameValue<E>> {
        self.0.get(key)
    }

    /// [`Self::get`]的可变引用版本
    /// * 🎯【2024-04-28 09:27:23】备用
    pub fn get_mut(&mut self, key: &str) -> Option<&mut NameValue<E>> {
        self.0.get_mut(key)
    }

    /// 🆕判断「是否包含元素」
    /// * 🎯用于[`Bag`]的[「是否有元素」查询](Bag::has)
    /// * 📜默认实现：`self.get(key).is_some()`
    pub fn has(&self, key: &str) -> bool {
        self.get(key).is_some()
    }

    /// 模拟`Bag.nameTable.put`方法
    /// * 🎯预期是「向映射插入值」
    /// * 📄出现在`putIn`方法中
    /// * 🚩需要返回「被替换出的旧有项」
    pub fn put(&mut self, key: &str, item: E, level: usize) -> Option<NameValue<E>> {
        // * 🚩【2024-05-04 13:06:22】始终尝试插入（在「从无到有」的时候需要）
        let name_value = (item, level);
        self.0.insert(key.to_string(), name_value)
    }

    /// 模拟`Bag.nameTable.remove`方法
    /// * 🎯预期是「从映射移除值」
    /// * 📄出现在`putIn`方法中
    /// * 🚩【2024-05-01 23:03:15】现在需要返回「被移除的元素」作为[`Bag::put_in`]的返回值
    pub fn remove(&mut self, key: &str) -> Option<NameValue<E>> {
        self.0.remove(key)
    }

    /// 移除物品，然后只返回移除出来的物品
    pub fn remove_item(&mut self, key: &str) -> Option<E> {
        self.0.remove(key).map(|(item, _)| item)
    }

    /// 模拟`Bag.nameTable.isEmpty`方法
    /// * 📜默认复用`size`方法
    pub fn is_empty(&self) -> bool {
        self.size() == 0
    }
}

/// 初代「层级映射」实现
#[derive(Clone, Default, PartialEq)]
pub struct BagItemTable(Box<[BagItemLevel]>);

impl BagItemTable {
    pub fn new(levels: usize) -> Self {
        let inner = vec![BagItemLevel::new(); levels].into_boxed_slice();
        Self(inner)
    }
}

impl Debug for BagItemTable {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        // 默认做法
        // f.debug_list().entries(self.0.iter()).finish()
        let mut debug_struct = f.debug_struct(std::any::type_name::<Self>());
        for (i, level) in self.0.iter().enumerate() {
            if !level.is_empty() {
                debug_struct.field(&format!("level_{i} ({})", level.size()), &level);
            }
        }
        debug_struct.finish()
    }
}

/// 📜为[`BagItemTableV1`]实现「层级映射」
/// * 🚩基于「元素id」的索引：不存储元素值
///   * 📝Java的情况可被视作`Arc`
impl BagItemTable // * 需要在「具体值匹配删除」时用到
{
    /// 模拟`Bag.itemTable.add(new ...)`
    /// * 📝OpenNARS目的：填充新的「一层」
    ///   * 📄`itemTable.add(new LinkedList<E>());`
    /// * 🆕此处细化重置为`add_new`以避免表示「层」的类型
    /// * 🆕添加「要新增的层级（范围：`0..层数`）」以允许「散列映射」
    pub fn add_new(&mut self, level: usize) {
        self.0[level] = BagItemLevel::new();
    }

    /// 模拟`Bag.itemTable.get`
    /// * 📝OpenNARS目的：多样
    pub fn get(&self, level: usize) -> &BagItemLevel {
        &self.0[level]
    }

    pub fn get_mut(&mut self, level: usize) -> &mut BagItemLevel {
        &mut self.0[level]
    }
}

/// 实现一个「层级队列」
#[derive(Debug, Clone, PartialEq, Default)]
pub struct BagItemLevel(VecDeque<String>);

/// 📜实现「层级」
impl BagItemLevel // * 需要在「具体值匹配删除」时用到
{
    /// 构造函数（空）
    pub fn new() -> Self {
        Self(VecDeque::new())
    }
    /// 模拟`LinkedList.size`
    pub fn size(&self) -> usize {
        self.0.len()
    }

    /// 模拟`LinkedList.isEmpty`
    /// * 📜默认使用[`Self::size`]
    pub fn is_empty(&self) -> bool {
        self.0.is_empty()
    }

    /// 模拟`LinkedList.add`
    /// * ❓不能引入一个新的元素，因为它所有权在「元素映射」里边
    /// * 🚩【2024-04-28 10:38:45】目前直接索引「键」而非「值」
    pub fn add(&mut self, key: String) {
        self.0.push_back(key)
    }

    /// 模拟`LinkedList.get`
    /// * ❓不能引入一个新的元素，因为它所有权在「元素映射」里边
    /// * 🚩【2024-04-28 10:38:45】目前直接索引「键」而非「值」
    pub fn get(&self, index: usize) -> Option<&String> {
        self.0.get(index)
    }

    /// 模拟`LinkedList.getFirst`
    /// * 📜默认转发[`Self::get`]
    #[inline(always)]
    pub fn get_first(&self) -> Option<&String> {
        self.0.front()
    }

    /// 模拟`LinkedList.removeFirst`
    pub fn remove_first(&mut self) {
        self.0.pop_front();
    }

    /// 模拟`LinkedList.remove`
    /// * 🚩【2024-06-22 16:16:37】避免和实现者的[`VecDeque::remove`]冲突
    pub fn remove_element(&mut self, key: &str) {
        if let Some(index) = self.0.iter().position(|k| k == key) {
            self.0.remove(index);
        }
    }
}
