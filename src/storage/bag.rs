use std::collections::{HashMap, VecDeque};

use super::distributor::Distributor;
use crate::{
    entity::{BagItem, BudgetValue},
    global::Float,
    nars::DEFAULT_PARAMETERS,
};

/// 对应OpenNARS的「袋」
/// * 📝【2024-04-26 23:12:15】核心逻辑：通过称作「预算」的机制，经济地分配内部元素
///   * 📌原理：AIKR
/// * 💭【2024-04-26 23:12:47】实际上「袋」并不需要元素基于「预算」
///   * 📌「预算」本质上不属于「元素」而是「元素×袋=预算」的概念
///   * 🚩换句话说，即：元素在袋内才具有的预算，有「预算映射」`(&袋, &元素id) -> Option<&预算>`
///   * 📌另外，「元素索引」作为元素在「袋」中的唯一标识符，有「元素映射」`(&袋, &元素id) -> Option<&元素>`
///     * 📌用于反查，还有「反查映射」`(&袋, &元素) -> Option<&元素id>`
///   * 🚩【2024-04-28 08:36:04】仍然需要：「元素」和「元素」之间，可能仍然需要访问各自的「预算」
///     * 📄在作为「元素」的「概念」中，需要访问「任务」的「预算」——此举不依赖「袋」对象
///     * 🎯减少迁移压力
/// * 📌对于用「关联类型」还用「泛型参数」的问题
///   * 📝「泛型参数」可以用`'_`省掉生命周期，而「关联类型」不行
///   * 📍原则：长久存在、完全所有权的放在「关联类型」，反之放在「泛型参数」
///   * ✅避免生命周期参数的泛滥，避开[`PhantomData`](std::marker::PhantomData)
///   * ❌【2024-04-27 10:14:41】尽可能全部用关联类型：加了泛型会导致无法使用「泛型实现」
///     * 📄"the type parameter `Item` is not constrained by the impl trait, self type, or predicates"
///     * 🔗<https://stackoverflow.com/questions/69238420/the-type-parameter-t-is-not-constrained-by-the-impl-trait-self-type-or-predi>
///   * 🚩【2024-04-27 11:55:09】目前仍然全部使用关联类型
/// * 📌OpenNARS复刻原则 类⇒特征
///   * 🚩私有访问：使用下划线作前缀
///     * 📄对`protected`统一使用`_`作为前缀
///     * 📄对`private`统一使用`__`作为前缀
///   * 🚩私有属性成员：使用`_(_)【属性名】_【成员名】_`模式
///     * 📌双下划线为分隔
///     * 🚩特殊/构造函数：`_(_)【属性名】_new_`（`new`不可能对应常规方法）
///     * 🚩特殊/赋值：`_(_)【属性名】_mut_`（`mut`不可能对应Rust函数）
///     * 🚩特殊/构造赋值：`_(_)【属性名】_mut_new_`
///       * 💭某些时候不知道也难以表示「被构造值」的类型
///       * 💭某些时候只有「构造赋值」的情形
///
/// # 📄OpenNARS `nars.storage.Bag`
/// A Bag is a storage with a constant capacity and maintains an internal
/// priority distribution for retrieval.
///
/// Each entity in a bag must extend Item, which has a BudgetValue and a key.
///
/// A name table is used to merge duplicate items that have the same key.
///
/// The bag space is divided by a threshold, above which is mainly time
/// management, and below, space management.
///
/// Differences:
///
/// 1. level selection vs. item selection
/// 2. decay rate
pub trait Bag<Item>
where
    // * ↑此处`Item`泛型仿OpenNARS`Bag`
    Item: BagItem<Key = Self::Key, Budget = Self::Budget>,
{
    /// 元素id类型
    /// * ❓要是引用类型还是值类型
    ///   * 后续如何兼容`String`与`&str`
    type Key: BagKey;

    /// 预算值类型
    /// * 🎯一种「袋」只有一种对「预算」的表征方式
    type Budget: BudgetValue;

    /// 分发器类型
    /// * 🎯伪随机数生成
    type Distributor: Distributor;

    /// 【只读常量】总层数
    ///
    /// # 📄OpenNARS `Bag.TOTAL_LEVEL`
    ///
    /// priority levels
    #[inline(always)]
    fn __total_level(&self) -> usize {
        DEFAULT_PARAMETERS.bag_level
    }

    /// 【只读常量】触发阈值
    /// * 📌触发の阈值
    ///
    /// # 📄OpenNARS `Bag.THRESHOLD`
    ///
    /// firing threshold
    #[inline(always)]
    fn __threshold(&self) -> usize {
        DEFAULT_PARAMETERS.bag_threshold
    }

    /// 相对阈值
    /// * 🚩由`触发阈值 / 总层数`计算得来
    ///
    /// # 📄OpenNARS `Bag.RELATIVE_THRESHOLD`
    ///
    /// relative threshold, only calculate once
    #[inline(always)]
    fn __relative_threshold(&self) -> Float {
        self.__threshold() as Float / self.__total_level() as Float
    }

    /// 加载因子
    /// * ❓尚不清楚其含义
    ///
    /// # 📄OpenNARS `Bag.LOAD_FACTOR`
    ///
    /// hash table load factor
    #[inline(always)]
    fn __load_factor(&self) -> Float {
        DEFAULT_PARAMETERS.load_factor
    }

    /// 分发器（只读常量）
    ///
    /// # 📄OpenNARS `Bag.DISTRIBUTOR`
    ///
    /// shared DISTRIBUTOR that produce the probability distribution
    fn __distributor(&self) -> &Self::Distributor;

    /// 模拟`Bag.nameTable`属性
    /// * 🚩【2024-04-28 08:43:25】目前不与任何「映射」类型绑定
    ///   * ❌不打算直接返回[`HashMap`]
    /// # 📄OpenNARS `Bag.nameTable`
    ///
    /// mapping from key to item
    fn __name_table(&self) -> &impl BagNameTable<Self::Key, Item>;
    fn __name_table_mut(&mut self) -> &mut impl BagNameTable<Self::Key, Item>;

    /// 模拟`Bag.nameTable`的「构造赋值」
    /// * 🎯预期是「构造一个映射，并赋值给内部字段」
    /// * 📄出现在`init`方法中
    fn __name_table_mut_new_(&mut self);
    // end `nameTable`

    /// 模拟`Bag.itemTable`属性
    /// * 📝OpenNARS中基于「优先级」的元素获取
    /// * 🚩【2024-04-28 10:47:35】目前只获取「元素id」而非「元素」
    ///   * ⚠️后续直接`unwrap`：通过`name_table`保证元素存在
    ///
    /// # 📄OpenNARS `Bag.itemTable`
    ///
    /// array of lists of items, for items on different level
    fn __item_tale(&self) -> &impl BagItemTable<Self::Key>;
    fn __item_tale_mut(&mut self) -> &mut impl BagItemTable<Self::Key>;

    /// 模拟`Bag.itemTable`的「构造赋值」
    /// * 🎯预期是「构造一个双层数组，并赋值给内部字段」
    /// * 📄出现在`init`方法中
    fn __item_table_mut_new_(&mut self);
    // end `itemTable`

    // TODO: 继续研究OpenNARS，发现并复现更多功能（抽象的）
    // * 🚩逐个字段复刻，从`capacity`继续
    // * ❓后续是要如何做？追溯到全部的使用地点吗

    /// 模拟`Bag.get`
    /// * 🚩转发内部`name_table`成员
    #[inline(always)]
    fn get(&self, key: &Self::Key) -> Option<&Item> {
        self.__name_table().get(key)
    }
    /// [`Self::get`]的可变版本
    /// * 🎯【2024-04-28 09:08:14】备用
    /// * 🚩转发内部`name_table`成员
    #[inline(always)]
    fn get_mut(&mut self, key: &Self::Key) -> Option<&mut Item> {
        self.__name_table_mut().get_mut(key)
    }

    /// 模拟`Bag.size`
    /// * 🎯从模拟`Bag.nameTable`派生
    /// * 🚩转发内部`name_table`成员
    #[inline(always)]
    fn size(&self) -> usize {
        self.__name_table().size()
    }

    /// 模拟`Bag.contains`
    /// * 🎯从模拟`Bag.nameTable.containsValue`派生
    /// * 📜默认使用[`Self::get`]
    #[inline(always)]
    fn contains(&self, item: &Item) -> bool {
        self.get(item.key()).is_some()
    }
}

/// 用于袋的「索引」
/// * 🎯方便后续安插方法
pub trait BagKey {}

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
/// * 🔦预计实现者：`HashMap<String, Item>`
pub trait BagNameTable<Key: BagKey, Item: BagItem<Key = Key>> {
    /// 模拟`Bag.nameTable.size`方法
    fn size(&self) -> usize;

    /// 模拟`Bag.nameTable.containsValue`方法
    /// * 📜默认复用`get`方法
    #[inline(always)]
    fn contains_value(&self, item: &Item) -> bool {
        self.get(item.key()).is_some()
    }

    /// 模拟`Bag.nameTable.containsValue`方法
    /// * 🎯预期是「在映射查找值；找到⇒Some，没找到⇒None」
    fn get(&self, key: &Key) -> Option<&Item>;
    /// [`Self::get`]的可变引用版本
    /// * 🎯【2024-04-28 09:27:23】备用
    fn get_mut(&mut self, key: &Key) -> Option<&mut Item>;

    /// 模拟`Bag.nameTable.put`方法
    /// * 🎯预期是「向映射插入值」
    /// * 📄出现在`putIn`方法中
    fn put(&mut self, key: &Key, item: Item);

    /// 模拟`Bag.nameTable.remove`方法
    /// * 🎯预期是「从映射移除值」
    /// * 📄出现在`putIn`方法中
    fn remove(&mut self, key: &Key);

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
/// # 📄OpenNARS `Bag.itemTable`
///
/// array of lists of items, for items on different level
pub trait BagItemTable<Key: BagKey> {
    /// 「层级」的类型
    /// * 🎯一个类型只有一种「层级」
    type Level: BagItemLevel<Key>;

    /// 模拟`Bag.itemTable.add(new ...)`
    /// * 📝OpenNARS目的：填充新的「一层」
    ///   * 📄`itemTable.add(new LinkedList<E>());`
    /// * 🆕此处细化重置为`add_new`以避免表示「层」的类型
    fn add_new(&mut self);

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
pub trait BagItemLevel<Key: BagKey> {
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
    fn add(&mut self, key: Key);

    /// 模拟`LinkedList.get`
    /// * ❓不能引入一个新的元素，因为它所有权在「元素映射」里边
    /// * 🚩【2024-04-28 10:38:45】目前直接索引「键」而非「值」
    fn get(&self, index: usize) -> Option<&Key>;
    fn get_mut(&mut self, index: usize) -> Option<&mut Key>;

    /// 模拟`LinkedList.getFirst`
    /// * 📜默认转发[`Self::get`]
    #[inline(always)]
    fn get_first(&self) -> Option<&Key> {
        self.get(0)
    }

    /// 模拟`LinkedList.removeFirst`
    fn remove_first(&mut self);

    /// 模拟`LinkedList.remove`
    fn remove(&mut self, key: &Key);
}

// 默认实现 //

/// 📜为「散列映射」[`HashMap`]实现「元素映射」
/// * 📝同名方法冲突时，避免「循环调用」的方法：完全限定语法
///   * 🔗<https://rustc-dev-guide.rust-lang.org/method-lookup.html>
///   * ⚠️[`HashMap`]使用[`len`](HashMap::len)而非[`size`](BagNameTable::size)
impl<Budget, Item> BagNameTable<String, Item> for HashMap<String, Item>
where
    Budget: BudgetValue,
    Item: BagItem<Key = String, Budget = Budget>,
{
    #[inline(always)]
    fn size(&self) -> usize {
        self.len()
    }

    #[inline(always)]
    fn get(&self, key: &String) -> Option<&Item> {
        Self::get(self, key)
    }

    #[inline(always)]
    fn get_mut(&mut self, key: &String) -> Option<&mut Item> {
        Self::get_mut(self, key)
    }

    #[inline(always)]
    fn put(&mut self, key: &String, item: Item) {
        if !self.contains_key(key) {
            self.insert(key.clone(), item);
        }
    }

    #[inline(always)]
    fn remove(&mut self, key: &String) {
        Self::remove(self, key);
    }
}

/// 📜为「队列列表」[`Vec<VecDeque>`](Vec)实现「层级映射」
/// * 🚩基于「元素id」的索引：不存储元素值
///   * 📝Java的情况可被视作`Arc`
impl<Key> BagItemTable<Key> for Vec<VecDeque<Key>>
where
    Key: BagKey + Eq, // * 需要在「具体值匹配删除」时用到
{
    // 队列
    type Level = VecDeque<Key>;

    #[inline(always)]
    fn add_new(&mut self) {
        self.push(VecDeque::new())
    }

    #[inline(always)]
    fn get(&self, level: usize) -> &Self::Level {
        &self[level]
    }

    #[inline(always)]
    fn get_mut(&mut self, level: usize) -> &mut Self::Level {
        &mut self[level]
    }
}

/// 📜为「队列」[`VecDeque`]实现「层级」
impl<Key> BagItemLevel<Key> for VecDeque<Key>
where
    Key: BagKey + Eq, // * 需要在「具体值匹配删除」时用到
{
    #[inline(always)]
    fn size(&self) -> usize {
        self.len()
    }

    #[inline(always)]
    fn add(&mut self, key: Key) {
        self.push_back(key)
    }

    #[inline(always)]
    fn get(&self, index: usize) -> Option<&Key> {
        Self::get(self, index)
    }

    #[inline(always)]
    fn get_mut(&mut self, index: usize) -> Option<&mut Key> {
        Self::get_mut(self, index)
    }

    #[inline(always)]
    fn remove_first(&mut self) {
        self.pop_front();
    }

    #[inline(always)]
    fn remove(&mut self, key: &Key) {
        if let Some(index) = self.iter().position(|k| k == key) {
            self.remove(index);
        }
    }
}

// 一个实验级实现 //

/// 袋的「元素id」类型
pub type BagKeyV1 = String;
impl BagKey for BagKeyV1 {}

/*
/// 第一版「袋」
pub struct BagV1<Item: BagItem> {
    /// 🆕分派器
    /// * 🚩不再作为全局变量，而是在构造函数中附带
    /// * 📝OpenNARS中主要用到的操作
    ///   * 创建 `new`
    ///   * 取（随机值） `pick`
    ///   * 下一个（随机值） `next`
    ///
    /// # OpenNARS `Bag.DISTRIBUTOR`
    ///
    /// shared DISTRIBUTOR that produce the probability distribution
    distributor: DistributorV1,

    /// 元素映射
    /// * 📝OpenNARS中主要用到的操作
    ///   * 创建 `new`
    ///   * 获取尺寸 `size`
    ///   * 检查是否包含（值） `containsValue`
    ///   * 从键获取值 `get`
    ///   * 插入值 `put`
    ///   * 从键移除值 `remove`
    ///   * 判断是否为空 `isEmpty`
    ///
    /// # 📄OpenNARS `Bag.nameTable`
    ///
    /// `mapping from key to item`
    item_map: HashMap<BagKeyV1, Item>,

    /// 🆕预算映射
    /// * 🎯用于脱离「元素」的「预算值」属性
    ///   * 📌元素只有在「袋」中才具有预算
    budget_map: HashMap<BagKeyV1, Budget>,

    /// 层级映射
    /// * 📝OpenNARS中主要用到的操作
    ///   * 创建 `new`
    ///   * 添加（到末尾） `add`
    ///   * 获取（在指定层级） `get`
    ///   * 获取指定层级是否为空 `get(n).isEmpty`
    ///   * 在指定层级增加 `get(n).add`
    ///   * 获取指定层级第一个 `get(n).getFirst`
    ///   * 移除指定层级第一个 `get(n).removeFirst`
    ///   * 移除指定层级某物品 `get(n).remove`
    /// * 📌【2024-04-27 14:13:36】目前对外层用[`Vec`]，内层用[`VecDeque`]
    ///   * 📌并且，仅存储键，避免复制与额外引用
    ///
    /// # 📄OpenNARS `Bag.itemTable`
    ///
    /// array of lists of items, for items on different level
    level_map: Vec<VecDeque<BagKeyV1>>,

    /// 袋容量
    /// * 📌在不同地方有不同的定义
    /// * 📝是一个「构造时固定」的属性
    ///
    /// # 📄OpenNARS `Bag.capacity`
    ///
    /// - defined in different bags
    /// - To get the capacity of the concrete subclass
    ///
    /// @return Bag capacity, in number of Items allowed
    capacity: usize,

    /// 质量
    /// * ❓暂且不能完全明白其含义
    ///
    /// # 📄OpenNARS `Bag.mass`
    ///
    /// current sum of occupied level
    mass: usize,

    /// 层级索引
    /// * ❓暂且不能完全明白其含义
    ///
    /// # 📄OpenNARS `Bag.levelIndex`
    ///
    /// index to get next level, kept in individual objects
    level_index: usize,

    /// 当前层级
    /// * ❓暂且不能完全明白其含义
    ///
    /// # 📄OpenNARS `Bag.currentLevel`
    ///
    /// current take out level
    current_level: usize,

    /// 当前层级
    /// * ❓暂且不能完全明白其含义
    ///
    /// # 📄OpenNARS `Bag.currentCounter`
    ///
    /// maximum number of items to be taken out at current level
    current_counter: usize,
    // ! ❌不作`memory: Memory`循环引用：所有涉及memory的方法，均移动到Memory中解决
    // memory: Memory,

    // ! ❌不作`bagObserver: BagObserver<Item>`观察者：不引入Java的「观察者模式」
    // ! ❌不作`showLevel: usize`显示用变量：不用于显示
}

// impl<Item> Bag for BagV1<Item>
// where
//     Item: BagItem,
// {
//     type Distributor = DistributorV1;
//     type Key = String;
//     type Item = Item; // TODO: 占位符
//     type Budget = Budget;

//     fn __distributor(&self) -> &Self::Distributor {
//         &self.distributor
//     }

//     fn get(&self, key: &String) -> Option<&Item> {
//         self.item_map.get(key)
//     }
// }
 */
