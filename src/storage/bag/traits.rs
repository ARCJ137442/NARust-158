//! 存放与「袋」有关的特征
//! * 📄袋
//! * 📄元素映射
//! * 📄层级映射

use crate::entity::Item;

/// 袋的「名称映射」
/// * 📄OpenNARS`Bag.nameTable`
/// * 🎯便于表示成员方法
///   * ⚠️仍然不能表达「构造」「赋值」
///     * 调用成员方法时只能返回`impl XXX`，若需「类型稳定」必须显示表示类型
/// * 📝OpenNARS所用到的方法
///   * 创建 `new` => 在`Bag`内部表示`mut_new`
///   * 获取尺寸 `size`
///   * 检查是否包含（值） `containsValue`
///   * 从键获取值 `get`
///   * 插入值 `put`
///   * 从键移除值 `remove`
///   * 判断是否为空 `isEmpty`
/// * 🔦预计实现者：`HashMap<String, E>`
pub trait BagNameTable<E: Item> {
    /// 模拟`Bag.nameTable.size`方法
    fn size(&self) -> usize;

    /// 模拟`Bag.nameTable.containsValue`方法
    /// * 📜默认复用`get`方法
    #[inline(always)]
    fn contains_value(&self, item: &E) -> bool {
        self.get(item.key()).is_some()
    }

    /// 模拟`Bag.nameTable.containsValue`方法
    /// * 🎯预期是「在映射查找值；找到⇒Some，没找到⇒None」
    fn get(&self, key: &str) -> Option<&E>;
    /// [`Self::get`]的可变引用版本
    /// * 🎯【2024-04-28 09:27:23】备用
    fn get_mut(&mut self, key: &str) -> Option<&mut E>;

    /// 🆕判断「是否包含元素」
    /// * 🎯用于[`Bag`]的[「是否有元素」查询](Bag::has)
    /// * 📜默认实现：`self.get(key).is_some()`
    #[inline(always)]
    fn has(&self, key: &str) -> bool {
        self.get(key).is_some()
    }

    /// 模拟`Bag.nameTable.put`方法
    /// * 🎯预期是「向映射插入值」
    /// * 📄出现在`putIn`方法中
    /// * 🚩需要返回「被替换出的旧有项」
    fn put(&mut self, key: &str, item: E) -> Option<E>;

    /// 模拟`Bag.nameTable.remove`方法
    /// * 🎯预期是「从映射移除值」
    /// * 📄出现在`putIn`方法中
    /// * 🚩【2024-05-01 23:03:15】现在需要返回「被移除的元素」作为[`Bag::put_in`]的返回值
    fn remove(&mut self, key: &str) -> Option<E>;

    /// 模拟`Bag.nameTable.isEmpty`方法
    /// * 📜默认复用`size`方法
    #[inline(always)]
    fn is_empty(&self) -> bool {
        self.size() == 0
    }
}

/// 袋的「层级映射」：从层级获取（并修改）元素列表
/// * 📝OpenNARS中基于「优先级」的元素获取
/// * 🆕🚩内部仅存储「元素id」而非「元素」值
///   * 🎯避免复制值，亦避免循环引用
/// * 🎯对应`Bag.itemTable`
/// * 📝OpenNARS所用到的方法
///   * 创建 `new` => 在`Bag`内部表示`mut_new`
///   * 新增空层级 `add(new ...)`
///   * 获取某个层级 `get`（可变）
///   * 遍历所有层级 `for (LinkedList<E> items : itemTable)`（仅呈现）
/// * 🔦预计实现者：`Vec<VecDeque<Item>>`
///
/// # 📄OpenNARS
///
/// array of lists of items, for items on different level
pub trait BagItemTable {
    /// 「层级」的类型
    /// * 🎯一个类型只有一种「层级」
    type Level: BagItemLevel;

    /// 模拟`Bag.itemTable.add(new ...)`
    /// * 📝OpenNARS目的：填充新的「一层」
    ///   * 📄`itemTable.add(new LinkedList<E>());`
    /// * 🆕此处细化重置为`add_new`以避免表示「层」的类型
    /// * 🆕添加「要新增的层级（范围：`0..层数`）」以允许「散列映射」
    fn add_new(&mut self, level: usize);

    /// 模拟`Bag.itemTable.get`
    /// * 📝OpenNARS目的：多样
    fn get(&self, level: usize) -> &Self::Level;
    fn get_mut(&mut self, level: usize) -> &mut Self::Level;
}

/// 袋「层级映射」的一层
/// * 🎯对标Java类型 `LinkedList<E>`
/// * 🚩内部仅存储「元素id」而非「元素」值
///   * 🎯避免复制值，亦避免循环引用
/// * 📝OpenNARS所用到的方法
///   * 创建 `new` => [`BagItemTable::add_new`]
///   * 大小 `size`
///   * 新增 `add`
///   * 获取 `get`
///   * 获取头部 `getFirst`
///   * 移除头部 `removeFirst`
///   * 移除（对某元素(id)） `remove`
/// * 🔦预计实现者：`Vec<VecDeque<Item>>`
pub trait BagItemLevel {
    /// 模拟`LinkedList.size`
    fn size(&self) -> usize;

    /// 模拟`LinkedList.isEmpty`
    /// * 📜默认使用[`Self::size`]
    #[inline(always)]
    fn is_empty(&self) -> bool {
        self.size() == 0
    }

    /// 模拟`LinkedList.add`
    /// * ❓不能引入一个新的元素，因为它所有权在「元素映射」里边
    /// * 🚩【2024-04-28 10:38:45】目前直接索引「键」而非「值」
    fn add(&mut self, key: String);

    /// 模拟`LinkedList.get`
    /// * ❓不能引入一个新的元素，因为它所有权在「元素映射」里边
    /// * 🚩【2024-04-28 10:38:45】目前直接索引「键」而非「值」
    fn get(&self, index: usize) -> Option<&String>;
    fn get_mut(&mut self, index: usize) -> Option<&mut String>;

    /// 模拟`LinkedList.getFirst`
    /// * 📜默认转发[`Self::get`]
    #[inline(always)]
    fn get_first(&self) -> Option<&String> {
        self.get(0)
    }

    /// 模拟`LinkedList.removeFirst`
    fn remove_first(&mut self);

    /// 模拟`LinkedList.remove`
    fn remove(&mut self, key: &str);
}
