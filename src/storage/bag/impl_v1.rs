//! 🎯复刻OpenNARS `nars.entity.Bag`

use super::{BagItemTable, BagNameTable, Distribute, Distributor, NameValue};
use crate::{
    control::DEFAULT_PARAMETERS,
    entity::{Item, ShortFloat},
    global::Float,
    inference::{Budget, BudgetFunctions, BudgetInference},
    util::ToDisplayAndBrief,
};

// ! 删除「具体类型」特征：能直接`struct`就直接`struct`

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
/// * 🚩【2024-05-01 23:17:26】暂且按照OpenNARS的命名来：
///   * 📌因为直接使用`Item`而非`BagItem`，故相应地改其中的`Item`为`E`
///   * 📝此中之`E`其实亦代表「Entity」（首字母）
/// * 🚩【2024-06-22 15:19:14】目前基于OpenNARS改版，将特征窄化为具体结构，以简化代码
///
/// TODO: 【2024-05-08 17:25:24】🏗️日后需要统一所有的「DEFAULT_PARAMETERS」：考虑引用计数
///
/// * ✅【2024-05-04 16:38:16】初步完成设计与测试

/// 复刻 `nars.storage.bag`
///
/// # 📄OpenNARS
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
#[derive(Debug, Clone)]
pub struct Bag<E: Item> {
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
    distributor: Distributor,

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
    /// # 📄OpenNARS
    ///
    /// `mapping from key to item`
    item_map: BagNameTable<E>,

    /// 层级映射
    /// * 📝OpenNARS中主要用到的操作
    ///   * 创建 `new`
    ///   * 添加（到末尾） `add`
    ///   * 获取（在指定层级） `get`
    ///   * 获取指定层级是否为空 `get(n).isEmpty`
    ///   * 在指定层级增加 `get(n).add`
    ///   * 获取指定层级第一个 `get(n).getFirst`
    ///   * 移除指定层级第一个 `get(n).removeFirst`
    ///   * 移除指定层级某元素 `get(n).remove`
    /// * 📌【2024-04-27 14:13:36】目前对外层用[`Vec`]，内层用[`VecDeque`]
    ///   * 📌并且，仅存储键，避免复制与额外引用
    ///
    /// # 📄OpenNARS
    ///
    /// array of lists of items, for items on different level
    level_map: BagItemTable,

    /// 袋容量
    /// * 📌在不同地方有不同的定义
    /// * 📝是一个「构造时固定」的属性
    ///
    /// # 📄OpenNARS
    ///
    /// - defined in different bags
    /// - To get the capacity of the concrete subclass
    ///
    /// @return Bag capacity, in number of Items allowed
    capacity: usize,

    /// 遗忘速率
    /// * 📌在不同地方有不同的定义
    /// * 📝是一个「构造时固定」的属性
    /// * 📝OpenNARS用于[`Bag::put_back`]的「放回时遗忘」中
    ///
    /// # 📄OpenNARS
    ///
    /// Get the item decay rate, which differs in difference subclass, and can be
    /// changed in run time by the user, so not a constant.
    ///
    /// @return The number of times for a decay factor to be fully applied
    forget_rate: usize,

    /// 质量
    /// * ❓暂且不能完全明白其含义
    ///
    /// # 📄OpenNARS
    ///
    /// current sum of occupied level
    mass: usize,

    /// 层级索引
    /// * ❓暂且不能完全明白其含义
    ///
    /// # 📄OpenNARS
    ///
    /// index to get next level, kept in individual objects
    level_index: usize,

    /// 当前层级
    /// * ❓暂且不能完全明白其含义
    ///
    /// # 📄OpenNARS
    ///
    /// current take out level
    current_level: usize,

    /// 当前层级
    /// * ❓暂且不能完全明白其含义
    ///
    /// # 📄OpenNARS
    ///
    /// maximum number of items to be taken out at current level
    current_counter: usize,

    /// 🆕决定「预算合并顺序」的函数指针
    /// * 🎯根据元素决定「预算合并」的顺序：新→旧 or 旧→新
    /// * 🚩目前采用函数指针
    merge_order_f: MergeOrderF<E>,
}

/// 🆕决定「预算合并顺序」的函数指针类型
pub type MergeOrderF<E> = fn(&E, &E) -> MergeOrder;

/// 预算合并顺序（枚举）
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub enum MergeOrder {
    /// 从「将移出的Item」合并到「新进入的Item」
    /// * 📌修改「新进入的Item」
    /// * 📜亦为默认值
    #[default]
    OldToNew,
    /// 从「新进入的Item」合并到「将移出的Item」
    /// * 📌修改「将移出的Item」
    NewToOld,
}

impl MergeOrder {
    /// 默认的「合并顺序」：旧→新
    pub fn default_order<E>(_: &E, _: &E) -> Self {
        Self::default()
    }
}

impl<E: Item> Default for Bag<E> {
    /// * 🚩【2024-05-04 16:26:53】默认当「概念袋」使
    fn default() -> Self {
        Self::new(
            DEFAULT_PARAMETERS.concept_bag_size,
            DEFAULT_PARAMETERS.concept_forgetting_cycle,
        )
    }
}

// impl<E: Item> BagConcrete<E> for Bag<E> {
impl<E: Item> Bag<E> {
    pub fn with_merge_order(
        capacity: usize,
        forget_rate: usize,
        merge_order_f: MergeOrderF<E>,
    ) -> Self {
        /* 📄OpenNARS源码：
        self.memory = memory;
        capacity = capacity();
        init(); */
        let mut this = Self {
            // 这两个是「超参数」要因使用者而异
            capacity,
            forget_rate,
            // 后续都是「内部状态变量」
            distributor: Distributor::new(Self::__TOTAL_LEVEL),
            // ? ❓【2024-05-04 12:32:58】因为上边这个不支持[`Default`]，所以就要写这些模板代码吗？
            // * 💭以及，这个`new`究竟要不要照抄OpenNARS的「先创建全空属性⇒再全部init初始化」特性
            //   * 毕竟Rust没有`null`要担心
            item_map: BagNameTable::default(),
            level_map: BagItemTable::default(),
            mass: usize::default(),
            level_index: usize::default(),
            current_level: usize::default(),
            current_counter: usize::default(),
            merge_order_f,
        };
        this.init();
        this
    }

    pub fn new(capacity: usize, forget_rate: usize) -> Self
    where
        Self: Sized,
    {
        Self::with_merge_order(capacity, forget_rate, MergeOrder::default_order::<E>)
    }
}

/// 对「以字符串为索引的袋」实现特征
/// * 🚩【2024-05-04 12:01:15】下面这些就是给出自己的属性，即「属性映射」
// impl<E: Item> Bagging<E> for Bag<E> {
impl<E: Item> Bag<E> {
    // * ↑此处`Item`泛型仿OpenNARS`Bag`
    /// 模拟`Bag.TOTAL_LEVEL`
    /// *📌总层数
    /// * 🚩【2024-05-04 01:44:29】根据OpenNARS中「常量」的定义，在此将其全局化
    ///   * 📌`static final` ⇒ `const`
    ///
    /// # 📄OpenNARS
    ///
    /// priority levels
    const __TOTAL_LEVEL: usize = DEFAULT_PARAMETERS.bag_level;

    /// 模拟`Bag.THRESHOLD`
    /// * 📌触发阈值
    /// * 📝触发の阈值
    ///
    /// # 📄OpenNARS
    ///
    /// firing threshold
    const __THRESHOLD: usize = DEFAULT_PARAMETERS.bag_threshold;

    /// 模拟`Bag.RELATIVE_THRESHOLD`
    /// 相对阈值
    /// * 🚩由`触发阈值 / 总层数`计算得来
    ///
    /// # 📄OpenNARS
    ///
    /// relative threshold, only calculate once
    const __RELATIVE_THRESHOLD: Float = Self::__THRESHOLD as Float / Self::__TOTAL_LEVEL as Float;

    /// 模拟`Bag.LOAD_FACTOR`
    /// * 📌加载因子
    /// * ❓尚不清楚其含义
    ///
    /// # 📄OpenNARS
    ///
    /// hash table load factor
    const __LOAD_FACTOR: Float = DEFAULT_PARAMETERS.load_factor;

    /// 模拟`Bag.capacity`
    /// * 📌一个「袋」的「容量」
    /// * 🚩只读
    ///   * 📄`private final int capacity;`
    /// * 📝OpenNARS中作为「属性」定义，仅仅是为了「缓存数值」并「在子类中分派不同的『大小』作为常数返回值」用
    ///   * 🚩因此无需附带`setter`
    /// * 💭【2024-05-04 01:48:01】实际上可以被定义为「关联常量」
    ///
    /// # 📄OpenNARS
    ///
    /// * 【作为属性】defined in different bags
    /// * 【作为方法】To get the capacity of the concrete subclass
    ///   * @return Bag capacity, in number of Items allowed
    pub fn capacity(&self) -> usize {
        self.capacity
    }

    /// 模拟`Bag.mass`
    /// * 📌一个「袋」已有元素的层数
    /// * 🚩会随着「增删元素」而变
    ///   * 🚩故需要一个「可变」版本
    ///   * 📝Rust允许`*self.__mass_mut() = XXX`的语法：左值可以是表达式
    ///
    /// # 📄OpenNARS
    ///
    /// current sum of occupied level
    pub fn mass(&self) -> usize {
        self.mass
    }

    /// 模拟`Bag.init`
    /// * 🚩初始化「元素映射」「层级映射」
    ///   * 📄对应[`Self::__name_table`]、[`Self::__item_table`]
    ///
    /// # 📄OpenNARS
    ///
    /// 🈚
    pub fn init(&mut self) {
        /* itemTable = new ArrayList<>(TOTAL_LEVEL);
        for (int i = 0; i < TOTAL_LEVEL; i++) {
            itemTable.add(new LinkedList<E>());
        }
        nameTable = new HashMap<>((int) (capacity / LOAD_FACTOR), LOAD_FACTOR);
        currentLevel = TOTAL_LEVEL - 1;
        levelIndex = capacity % TOTAL_LEVEL; // so that different bags start at different point
        mass = 0;
        currentCounter = 0; */
        self.level_map = BagItemTable::new(Self::__TOTAL_LEVEL);
        for level in 0..Self::__TOTAL_LEVEL {
            self.level_map.add_new(level);
        }
        self.item_map = BagNameTable::new();
        self.current_level = Self::__TOTAL_LEVEL - 1;
        self.level_index = self.capacity() % Self::__TOTAL_LEVEL; // 不同的「袋」在分派器中有不同的起点
        self.mass = 0;
        self.current_counter = 0;
    }

    // ! 🚩`Bag.capacity`已在`self.__capacity`中实现

    /// 模拟`Bag.size`
    /// * 🎯从模拟`Bag.nameTable`派生
    /// * 🚩转发内部`name_table`成员
    ///
    /// # 📄OpenNARS
    ///
    /// The number of items in the bag
    #[inline(always)]
    pub fn size(&self) -> usize {
        self.item_map.size()
    }

    /// 模拟`Bag.averagePriority`
    ///
    /// # 📄OpenNARS
    ///
    /// Get the average priority of Items
    ///
    /// @return The average priority of Items in the bag
    pub fn average_priority(&self) -> Float {
        /* 📄OpenNARS源码：
        if (size() == 0) {
            return 0.01f;
        }
        float f = (float) mass / (size() * TOTAL_LEVEL);
        if (f > 1) {
            return 1.0f;
        }
        return f; */
        if self.size() == 0 {
            return 0.01;
        }
        Float::min(
            // 复刻最后一个条件判断
            (self.mass() as Float) / (self.size() * Self::__TOTAL_LEVEL) as Float,
            1.0,
        )
    }

    /// 模拟`Bag.contains`
    /// * 🎯从模拟`Bag.nameTable.containsValue`派生
    /// * 📜默认使用[`Self::get`]
    ///
    /// # 📄OpenNARS
    ///
    /// Check if the bag contains the item
    ///
    /// @param item The item to be checked
    /// @return Whether the bag contains the item
    #[inline(always)]
    pub fn contains(&self, item: &E) -> bool {
        self.get(item.key()).is_some()
    }

    /// 模拟`Bag.get`
    /// * 🚩转发内部`name_table`成员
    ///
    /// # 📄OpenNARS
    ///
    /// Get an Item by key
    ///
    /// @param key The key of the Item
    /// @return The Item with the given key
    #[inline(always)]
    #[must_use]
    pub fn get(&self, key: &str) -> Option<&E> {
        self.item_map.get(key).map(|(e, _)| e)
    }
    /// [`Self::get`]的可变版本
    /// * 🎯【2024-04-28 09:08:14】备用
    /// * 🚩转发内部`name_table`成员
    #[inline(always)]
    #[must_use]
    pub fn get_mut(&mut self, key: &str) -> Option<&mut E> {
        self.item_map.get_mut(key).map(|(e, _)| e)
    }

    /// 🆕提供「元素id是否对应值」的功能
    /// * 🎯【2024-05-07 22:19:07】在「记忆区」查找时，为规避「直接带Concept [`Option`]」带来的借用问题，采用「只查询是否有」的方式
    pub fn has(&self, key: &str) -> bool {
        self.item_map.has(key)
    }

    /// 模拟`Bag.putIn`
    /// * 🚩过程「放入」
    /// * 🆕不通过「返回布尔值」验证「是否添加成功」，而是通过「返回一个[`Option`]」表示「添加成功与否」
    ///   * 📌此举虽总是「消耗」，但若需要复用「添加失败时的元素」仍可从返回值中拿取
    /// * 🔗链接到的方法
    ///   * [`intoBase`](Self::into_base)
    ///   * [`outOfBase`](Self::out_of_base)
    ///   * [`BudgetValue.merge`](BudgetValue::merge)
    ///
    /// 📄OpenNARS `Bag.putIn`
    ///
    /// Add a new Item into the Bag
    ///
    /// @param newItem The new Item
    /// @return Whether the new Item is added into the Bag
    #[must_use]
    pub fn put_in(&mut self, new_item: E) -> Option<E> {
        /* String newKey = newItem.getKey();
        E oldItem = nameTable.put(newKey, newItem);
        if (oldItem != null) { // merge duplications
            outOfBase(oldItem);
            newItem.merge(oldItem);
        }
        E overflowItem = intoBase(newItem); // put the (new or merged) item into itemTable
        if (overflowItem != null) { // remove overflow
            String overflowKey = overflowItem.getKey();
            nameTable.remove(overflowKey);
            return (overflowItem != newItem);
        } else {
            return true;
        } */

        // 置入「元素映射」
        let new_key = new_item.key().clone();
        let level = self.calculate_level_for_item(&new_item);
        let old_item = self.item_map.put(&new_key, new_item, level);

        // 若在「元素映射」中重复了：有旧项⇒合并「重复了的新旧项」
        if let Some(old) = old_item {
            // * 在「层级映射」移除旧项 | 🚩【2024-05-04 11:45:02】现在仍需使用「元素」，因为下层调用需要访问元素本身（预算值），并需避免过多的「按键取值」过程
            self.item_out_of_base(&old);
            let (mut old_item, _) = old;

            // * 🚩计算「合并顺序」
            let new_item = self.get(&new_key).unwrap(); // * 🚩🆕重新获取「置入后的新项」（⚠️一定有）
            let merge_order = (self.merge_order_f)(&old_item, new_item); // 此处调用函数指针，一定是不可变引用
            let new_item = self.get_mut(&new_key).unwrap(); // * 🚩🆕重新获取「置入后的新项」（⚠️一定有）

            // * 🚩按照计算出的「合并顺序」合并预算值
            use MergeOrder::*;
            match merge_order {
                OldToNew => new_item.merge_from(&old_item),
                NewToOld => old_item.merge_from(new_item),
            }
        }

        // 置入「层级映射」
        // 若在「层级映射」中溢出了：若有「溢出」则在「元素映射」中移除
        // ! 📌【2024-05-04 11:35:45】↓此处`__into_base`仅传入「元素id」是为了规避借用问题（此时`new_item`已失效）
        if let Some(overflow_key) = self.item_into_base(&new_key) {
            // 直接返回「根据『溢出的元素之id』在『元素映射』中移除」的结果
            // * 🚩若与自身相同⇒返回`Some`，添加失败
            // * 🚩若与自身不同⇒返回`None`，添加仍然成功
            let overflow_item = self.item_map.remove_item(&overflow_key);
            match overflow_key == new_key {
                true => overflow_item,
                false => None, // ! 此时将抛掉溢出的元素
            }
        } else {
            None
        }
    }

    /// 模拟`Bag.putBack`
    /// * 🚩过程「放回」
    // * 📝【2024-05-04 02:07:06】把「预算函数」的「基建」做好了，这里的事就好办了
    ///
    /// # 📄OpenNARS
    ///
    /// Put an item back into the itemTable
    ///
    /// The only place where the forgetting rate is applied
    ///
    /// @param oldItem The Item to put back
    /// @return Whether the new Item is added into the Bag
    #[must_use]
    pub fn put_back(&mut self, mut old_item: E) -> Option<E> {
        self.forget(&mut old_item);
        self.put_in(old_item)
    }

    /// 🆕以一定函数修改某个Item的优先级
    /// * 🚩改成泛型函数，以便适用在所有地方
    pub fn forget(&self, item: &mut impl Budget) {
        let new_priority = item.forget(self.forget_rate as Float, Self::__RELATIVE_THRESHOLD);
        item.set_priority(ShortFloat::from_float(new_priority));
    }

    /// 模拟`Bag.takeOut`
    /// * 🚩过程「取出」
    /// * 📝实际上需要这些函数作为前置功能：
    ///   * [`_empty_level`](Bag::_empty_level)
    ///   * [`take_out_first`](Bag::take_out_first)
    ///   * [`refresh`](Bag::refresh)
    ///
    /// # 📄OpenNARS
    ///
    /// Choose an Item according to priority distribution and take it out of the
    /// Bag
    ///
    /// @return The selected Item
    #[must_use]
    pub fn take_out(&mut self) -> Option<E> {
        /* 📄OpenNARS源码：
        if (nameTable.isEmpty()) { // empty bag
            return null;
        }
        if (emptyLevel(currentLevel) || (currentCounter == 0)) { // done with the current level
            currentLevel = DISTRIBUTOR.pick(levelIndex);
            levelIndex = DISTRIBUTOR.next(levelIndex);
            while (emptyLevel(currentLevel)) { // look for a non-empty level
                currentLevel = DISTRIBUTOR.pick(levelIndex);
                levelIndex = DISTRIBUTOR.next(levelIndex);
            }
            if (currentLevel < THRESHOLD) { // for dormant levels, take one item
                currentCounter = 1;
            } else { // for active levels, take all current items
                currentCounter = itemTable.get(currentLevel).size();
            }
        }
        E selected = takeOutFirst(currentLevel); // take out the first item in the level
        currentCounter--;
        nameTable.remove(selected.getKey());
        refresh();
        return selected; */
        if self.item_map.is_empty() {
            return None;
        }
        let level = self.select_next_level_for_take();
        let selected_key = self.take_out_first(level);
        // * 此处需要对内部可能有的「元素id」进行转换
        match selected_key {
            Some(key) => self.item_map.remove_item(&key),
            None => None,
        }
    }

    /// 为[`Self::take_out`]选择下一个要被取走的level
    /// * 🚩计算并返回「下一个level值」
    fn select_next_level_for_take(&mut self) -> usize {
        if self.empty_level(self.current_level) || (self.current_counter) == 0 {
            self.current_level = self.distributor.pick(self.level_index);
            self.level_index = self.distributor.next(self.level_index);
            while self.empty_level(self.current_level) {
                // * 📝这里实际上就是一个do-while
                self.current_level = self.distributor.pick(self.level_index);
                self.level_index = self.distributor.next(self.level_index);
            }
            self.current_counter = match self.current_level < Self::__THRESHOLD {
                true => 1,
                false => self.level_map.get(self.current_level).size(),
            };
        }
        self.current_counter -= 1;
        self.current_level
    }

    /// 模拟`Bag.pickOut`
    /// * 🚩过程「挑出」
    ///
    /// # 📄OpenNARS
    ///
    /// Pick an item by key, then remove it from the bag
    ///
    /// @param key The given key
    /// @return The Item with the key
    #[must_use]
    pub fn pick_out(&mut self, key: &str) -> Option<E> {
        /* 📄OpenNARS源码：
        E picked = nameTable.get(key);
        if (picked != null) {
            outOfBase(picked);
            nameTable.remove(key);
        }
        return picked; */
        let name_value = self.item_map.remove(key)?;
        self.item_out_of_base(&name_value);
        Some(name_value.0)
    }

    /// 模拟`Bag.emptyLevel`
    ///
    /// # 📄OpenNARS
    ///
    /// Check whether a level is empty
    ///
    /// @param n The level index
    /// @return Whether that level is empty
    pub fn empty_level(&self, level: usize) -> bool {
        /* 📄OpenNARS源码：
        return (itemTable.get(n).isEmpty()); */
        self.level_map.get(level).is_empty()
    }

    /// 模拟`Bag.getLevel`
    /// * 📝Rust中[`usize`]无需考虑负值问题
    /// * 🚩【2024-06-30 17:55:38】现更改计算方法：不能信任物品的「优先级」
    ///   * ⚠️bug：可能物品在袋内变更了优先级，后续拿出时就会mass溢出
    /// * 🆕只在[`Self::item_into_base`]中被调用
    ///
    /// # 📄OpenNARS
    ///
    /// Decide the put-in level according to priority
    ///
    /// @param item The Item to put in
    /// @return The put-in level
    #[doc(alias = "level_from_item")]
    fn calculate_level_for_item(&self, item: &E) -> usize {
        /* 📄OpenNARS源码：
        float fl = item.getPriority() * TOTAL_LEVEL;
        int level = (int) Math.ceil(fl) - 1;
        return (level < 0) ? 0 : level; // cannot be -1 */
        let fl = item.priority().to_float() * Self::__TOTAL_LEVEL as Float;
        let level = (fl.ceil()) as usize; // ! 此处不提前-1，避免溢出
        level.saturating_sub(1) // * 🚩↓相当于如下代码
                                /* if level < 1 {
                                    0
                                } else {
                                    level - 1
                                } */
    }

    /// 模拟`Bag.intoBase`
    /// * 🚩以「元素id」代替「元素自身」在「层级映射」中添加元素
    /// * 🚩若添加成功，将复制「元素id」
    /// * 🚩返回「『溢出』的元素id」
    /// * 🚩【2024-05-01 23:10:46】此处允许【在clippy中被警告】的情形：OpenNARS原装函数
    ///   * ✅【2024-05-04 11:09:39】现在因为「前缀下划线」不再会被警告
    /// * 🚩【2024-05-04 11:13:04】现在仍然使用「元素引用」，因为[`Bag::__get_level`]需要元素的预算值
    /// * 📝【2024-05-04 11:34:43】OpenNARS中只会被[`Bag::put_in`]调用
    /// * 🚩【2024-06-22 16:36:10】改名避嫌
    ///   * ℹ️ clippy: methods called `into_*` usually take `self` by value; consider choosing a less ambiguous name
    ///
    /// # 📄OpenNARS
    ///
    /// Insert an item into the itemTable, and return the overflow
    ///
    /// @param newItem The Item to put in
    /// @return The overflow Item
    fn item_into_base(&mut self, new_key: &str) -> Option<String> {
        /* 📄OpenNARS源码：
        E oldItem = null;
        int inLevel = getLevel(newItem);
        if (size() > capacity) { // the bag is full
            int outLevel = 0;
            while (emptyLevel(outLevel)) {
                outLevel++;
            }
            if (outLevel > inLevel) { // ignore the item and exit
                return newItem;
            } else { // remove an old item in the lowest non-empty level
                oldItem = takeOutFirst(outLevel);
            }
        }
        itemTable.get(inLevel).add(newItem); // FIFO
        mass += (inLevel + 1); // increase total mass
        refresh(); // refresh the window
        return oldItem; */
        let new_item = self.get(new_key).expect("不能没有所要获取的值"); // * 🚩🆕（在调用方处）重新获取「置入后的新项」（⚠️一定有）
        let mut old_item = None;
        let in_level = self.calculate_level_for_item(new_item);

        // 🆕先假设「新元素已被置入」，「先加后减」防止usize溢出
        self.mass += in_level + 1;
        if self.size() > self.capacity() {
            // * 📝逻辑：低优先级溢出——从低到高找到「第一个非空层」然后弹出其中第一个（最先的）元素
            // * 🚩【2024-05-04 13:14:02】实际上与Java代码等同；但若直接按源码来做就会越界
            let out_level = (0..Self::__TOTAL_LEVEL)
                .find(|level| !self.empty_level(*level))
                .unwrap_or(Self::__TOTAL_LEVEL);
            if out_level > in_level {
                // 若到了自身所在层⇒弹出自身（相当于「添加失败」）
                self.mass -= in_level + 1; // 🆕失败，减去原先相加的数
                return Some(new_key.to_string()); // 提早返回
            } else {
                old_item = self.take_out_first(out_level);
            }
        }
        // 继续增加元素
        self.level_map.get_mut(in_level).add(new_key.to_string());
        // self.refresh(); // ! ❌【2024-05-04 11:16:55】不复刻这个有关「观察者」的方法
        old_item
    }

    /// 模拟`Bag.takeOutFirst`
    ///
    /// # 📄OpenNARS
    ///
    /// Take out the first or last E in a level from the itemTable
    ///
    /// @param level The current level
    /// @return The first Item
    fn take_out_first(&mut self, level: usize) -> Option<String> {
        /* 📄OpenNARS源码：
        E selected = itemTable.get(level).getFirst();
        itemTable.get(level).removeFirst();
        mass -= (level + 1);
        refresh();
        return selected; */
        let selected = self.level_map.get(level).get_first().cloned();
        if selected.is_some() {
            // * 🚩仅在「有选择到」时移除 | ✅【2024-05-04 14:31:30】此举修复了「mass溢出」的bug！
            self.level_map.get_mut(level).remove_first();
            self.mass -= level + 1;
        }
        selected
    }

    /// 模拟`Bag.outOfBase`
    /// * 🚩【2024-06-22 16:37:07】跟从[`Self::item_into_base`]一同改名
    ///
    /// # 📄OpenNARS
    ///
    /// Remove an item from itemTable, then adjust mass
    ///
    /// @param oldItem The Item to be removed
    fn item_out_of_base(&mut self, (old_item, level): &NameValue<E>) {
        /* 📄OpenNARS源码：
        int level = getLevel(oldItem);
        itemTable.get(level).remove(oldItem);
        mass -= (level + 1);
        refresh(); */
        self.level_map
            .get_mut(*level)
            .remove_element(old_item.key());
        self.mass -= level + 1;
    }

    /// 模拟`Bag.toString`
    /// * 🚩🆕一次显示所有层，避开`showLevel`
    ///
    /// # 📄OpenNARS
    ///
    /// Collect Bag content into a String for display
    ///
    /// @return A String representation of the content
    pub fn bag_to_display(&self) -> String {
        /* 📄OpenNARS源码：
        StringBuffer buf = new StringBuffer(" ");
        for (int i = TOTAL_LEVEL; i >= showLevel; i--) {
            if (!emptyLevel(i - 1)) {
                buf = buf.append("\n --- Level ").append(i).append(":\n ");
                for (int j = 0; j < itemTable.get(i - 1).size(); j++) {
                    buf = buf.append(itemTable.get(i - 1).get(j).toStringBrief()).append("\n ");
                }
            }
        }
        return buf.toString(); */
        let mut buf = String::new();
        // * 🚩倒序遍历所有非空层
        for level in (0..Self::__TOTAL_LEVEL)
            .rev()
            .filter(|&level| !self.empty_level(level))
        {
            buf += "\n --- Level ";
            buf += &level.to_string();
            buf += ":\n ";
            let level_size = self.level_map.get(level).size();
            for i in 0..level_size {
                let key = self.level_map.get(level).get(i);
                if let Some(key) = key {
                    let item = self.get(key).unwrap(); // ! 📌【2024-05-09 01:27:59】不可能没有
                    buf += &item.to_display_brief();
                    buf += "\n "
                }
            }
        }
        buf
    }
}

// 显示呈现方法
impl<E: Item> ToDisplayAndBrief for Bag<E> {
    fn to_display(&self) -> String {
        self.bag_to_display()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::global::Float;
    use crate::{
        entity::{BudgetValue, ShortFloat, Token},
        inference::Budget,
        ok,
        util::{AResult, ToDisplayAndBrief},
    };
    use nar_dev_utils::{asserts, list};

    /// [`Item`]的测试用初代实现
    /// * 💭【2024-05-07 20:50:29】实际上并没有用：真正有用的是「任务」「概念」等「实体类」
    type ItemV1 = Token;

    fn new_item(key: impl Into<String>, p: Float, d: Float, q: Float) -> ItemV1 {
        ItemV1::new(key.into(), BudgetValue::from_floats(p, d, q))
    }

    /// 测试用「袋」的类型
    type Item1 = ItemV1;
    type Bag1 = Bag<Item1>;

    /// 测试/单个元素
    /// * 🎯初始化 [`Bag::init`]
    /// * 🎯尺寸 [`Bag::size`]
    /// * 🎯重量 [`Bag::__mass`]
    /// * 🎯获取 [`Bag::get`]
    /// * 🎯获取层级 [`Bag::__get_level`]
    /// * 🎯判空层级 [`Bag::_empty_level`]
    /// * 🎯放入 [`Bag::put_in`]
    /// * 🎯挑出 [`Bag::pick_out`]
    /// * 🎯放回 [`Bag::put_back`]
    /// * 🎯取出 [`Bag::take_out`]
    #[test]
    fn single_item() -> AResult {
        // 构造测试用「袋」
        let mut bag = Bag1::new(1, 1);
        dbg!(&bag);

        // 初始化 // ? 是否应该自带
        bag.init();
        dbg!(&bag);
        asserts! {
            bag.size() == 0, // 空的
            bag.mass() == 0, // 空的
            bag.empty_level(0) => true, // 第0层也是空的
        }

        // 放入元素
        let key1 = "item001";
        let item1 = new_item(key1, 0.0, 0.0, 0.0); // * 🚩固定为「全零预算」
        let overflowed = bag.put_in(dbg!(item1.clone()));
        asserts! {
            overflowed.is_none(), // 没有溢出
            bag.get(key1) == Some(&item1), // 放进「对应id位置」的就是原来的元素
            bag.size() == 1, // 放进了一个
            bag.calculate_level_for_item(&item1) => 0, // 放进的是第0层（优先级为0.0）
            bag.empty_level(0) => false, // 放进的是第0层
            bag.mass() == 1, // 放进第0层，获得(0+1)的重量
        }
        dbg!(&bag);

        // 挑出元素
        let picked = bag.pick_out(key1).unwrap();
        asserts! {
            picked == item1, // 挑出的就是所置入的
            bag.size() == 0, // 取走了
            bag.mass() == 0, // 取走了
            bag.empty_level(0) => true, // 取走的是第0层
        }

        // 放回元素
        let overflowed = bag.put_back(picked);
        asserts! {
            overflowed => None, // 没有溢出
            bag.size() == 1, // 放回了
            bag.empty_level(0) => false, // 放入的是第0层
            bag.mass() == 1, // 放进第0层，获得(0+1)的重量
        }

        // 取出元素
        let mut taken = bag.take_out().unwrap();
        asserts! {
            taken == item1, // 取出的就是放回了的
            bag.size() == 0, // 取走了
            bag.mass() == 0, // 取走了
            bag.empty_level(0) => true, // 取走的是第0层
        }

        // 修改预算值：优先级"0 => 1"，耐久度"0 => 1"
        // ! 📝如果没有耐久度
        taken.budget_mut().set_priority(ShortFloat::ONE);
        taken.budget_mut().set_durability(ShortFloat::ONE);
        asserts! {
            // 最终增长到 1.0
            taken.budget_mut().priority() == ShortFloat::ONE,
            taken.budget_mut().durability() == ShortFloat::ONE,
        }

        // 放回元素，其中会有「遗忘」的操作
        let overflowed = bag.put_back(taken);
        asserts! {
            overflowed => None, // 没有溢出
            bag.size() == 1, // 放回了
            bag.empty_level(0) => true, // 放入的不再是第0层
            bag.empty_level(Bag1::__TOTAL_LEVEL-1) => false, // 放入的是最高层
            bag.mass() == Bag1::__TOTAL_LEVEL, // 放进第最高层，获得 层数 的重量
        }

        // 最后完成
        ok!()
    }

    /// 测试/多个元素
    /// * 🎯初始化 [`Bag::init`]
    /// * 🎯尺寸 [`Bag::size`]
    /// * 🎯获取 [`Bag::get`]
    /// * 🎯获取层级 [`Bag::__get_level`]
    /// * 🎯判空层级 [`Bag::_empty_level`]
    /// * 🎯放入 [`Bag::put_in`]
    /// * 🎯挑出 [`Bag::pick_out`]
    /// * 🎯放回 [`Bag::put_back`]
    /// * 🎯取出 [`Bag::take_out`]
    #[test]
    fn multi_item() -> AResult {
        // 构造测试用「袋」并初始化
        let mut bag = Bag1::default();
        bag.init();
        dbg!(&bag);
        asserts! {
            bag.size() == 0, // 空的
            bag.empty_level(0) => true, // 第0层也是空的
        }

        /// 测试规模（放入0~10 共**(N+1)**个元素）
        const N: usize = 10;

        // 生成元素
        let key_f = |i| format!("item{:03}", i);
        let priority = |i| i as Float / N as Float;
        // * 📝变换关系：0~1 → [0, 层数] → [0, 层数)
        // * 📝对应关系 @ [0, 层数] → [0, 层数)
        //   * [0, 1] => 0
        //   * (1, 2] => 1
        //   * [层数-1, 层数] => 层数-1
        // * 📌层级计算公式：
        //   * 层级百分比：`i / N`
        //   * 层级：`ceil(百分比 * 层数) - 1`
        let expected_level = |i| {
            let level_percent = priority(i) as Float * Bag1::__TOTAL_LEVEL as Float;
            (level_percent.ceil() as usize).saturating_sub(1)
        };
        let items = list![
            {
                let key = key_f(i);
                let priority = priority(i);
                let durability = 0.5;
                let quality = 0.5;
                let item = new_item(key.clone(), priority, durability, quality);
                (key, item)
            }
            for i in (0..=N)
        ];

        // 放入多个元素
        for (i, (key, item)) in items.iter().enumerate() {
            let overflowed = bag.put_in(item.clone());
            asserts! {
                overflowed.is_none(), // 没有溢出
                bag.get(key) == Some(item), // 放进「对应id位置」的就是原来的元素
                bag.size() == i + 1, // 放进了(i+1)个
                bag.calculate_level_for_item(item) => expected_level(i), // 放进了指定层
                bag.empty_level(expected_level(i)) => false, // 放进的是指定层
            }
        }
        println!("初次放入后：{bag:#?}");

        // 挑出元素
        let mut picked_items = vec![];
        for (i, (key, item)) in items.iter().enumerate() {
            let picked = bag.pick_out(key).unwrap(); // 一定能挑出

            // 计算预期层数
            asserts! {
                picked == *item, // 挑出的就是所置入的
                bag.size() == N - i, // 取走了
                bag.empty_level(expected_level(i)) => true, // 取走的是指定层
            }
            picked_items.push(picked);
        }

        // 放回元素
        for (i, picked) in picked_items.into_iter().enumerate() {
            let overflowed = bag.put_back(picked); // 此时预算值也改变了：会衰减
            asserts! {
                overflowed => None, // 没有溢出
                bag.size() == i + 1, // 放回了
                // bag._empty_level(0) => false, // 放入的是第0层
            }
        }
        println!("第一次放回后：{bag:#?}");

        // 取出元素
        let mut taken_items = vec![];
        for i in 0..=N {
            let taken = bag.take_out().unwrap(); // 一定拿得出来
            asserts! {
                bag.size() == N - i, // 取走了
                // bag._empty_level(0) => true, // 取走的是第0层
            }
            // 添加 & 展示 | 📌此处预算值已根据[`BudgetValue::forget`]衰减
            taken_items.push(dbg!(taken));
        }

        // 放回元素
        for (i, taken) in taken_items.into_iter().enumerate() {
            let _ = bag.put_back(taken);
            asserts! {
                bag.size() == i + 1, // 放回了
                // bag._empty_level(0) => true, // 放入的不再是第0层
                // bag._empty_level(Bag1::__TOTAL_LEVEL-1) => false, // 放入的是最高层
            }
        }

        // 最后完成
        println!("第二次放回后：{bag:#?}");
        ok!()
    }

    /// 测试/长期
    /// * 🎯放入→多次「取出→放回→取出→放回→……」的结果
    #[test]
    fn long_term() -> AResult {
        // 测试规模（重复「取出→放回→」的次数）
        const N: usize = 100;

        // 构造测试用「袋」并初始化
        let mut bag = Bag1::new(10, N);
        bag.init();
        dbg!(&bag);
        asserts! {
            bag.size() == 0, // 空的
            bag.mass() == 0, // 空的
        }

        // 生成元素
        let key = "item";
        // * 🚩固定的初始预算值
        let budget_initial = BudgetValue::new(ShortFloat::ONE, ShortFloat::HALF, ShortFloat::ONE);
        let item = Item1::new(key, budget_initial);

        // 放入元素
        let overflowed = bag.put_in(dbg!(item.clone()));
        asserts! {
            overflowed.is_none(), // 没有溢出
            bag.get(key) == Some(&item), // 放进「对应id位置」的就是原来的元素
            bag.size() == 1, // 放进了一个
            bag.mass() >= 1, // 放进了，获得重量
        }
        dbg!(&bag);

        // 多次取出放回 | // * 📝根据[`BudgetFunctions::forget`]，实际上只有「优先级」会变化
        println!("budget trending from {budget_initial}:");
        for _ in 0..N {
            let taken = bag.take_out().unwrap(); // 一定拿得出来

            // 检查、展示
            asserts! {
                bag.size() == 0, // 取出了
                bag.mass() == 0, // 失去所有重量
            };
            println!("\t{}", taken.budget());

            //放回元素
            let overflowed = bag.put_back(taken);
            assert_eq!(
                overflowed,
                None // 没有溢出
            )
        }
        println!("{}", bag.to_display_long());

        // 最终完成
        ok!()
    }

    /// 测试/物品在袋内优先级变化
    /// * ⚠️测试「袋内优先级发生变化，是否能正确被 挑出/拿出」
    #[test]
    fn modified_level_in_bag() -> AResult {
        // 构造测试用「袋」
        let mut bag = Bag1::new(1, 1);
        bag.init();

        // 放入元素
        let key = "item001";
        let item = new_item(key, 0.0, 0.0, 0.0); // * 🚩固定为「全零预算」
        let overflowed = bag.put_in(dbg!(item.clone()));
        asserts! {
            overflowed.is_none(), // 没有溢出
            bag.get(key) == Some(&item), // 放进「对应id位置」的就是原来的元素
            bag.size() == 1, // 放进了一个
            bag.calculate_level_for_item(&item) => 0, // 放进的是第0层（优先级为0.0）
            bag.empty_level(0) => false, // 放进的是第0层
            bag.mass() == 1, // 放进第0层，获得(0+1)的重量
        }
        dbg!(&bag);

        // ! 在袋内修改优先级
        let item_mut = bag.get_mut(key).expect("此时袋内必须有物品");
        item_mut.set_priority(ShortFloat::ONE);

        // 挑出元素
        let picked = bag.pick_out(key).unwrap();
        asserts! {
            bag.size() == 0, // 取走了
            bag.mass() == 0, // 取走了
            bag.empty_level(0) => true, // 取走的是第0层
        }

        // 放回元素
        let overflowed = bag.put_back(picked);
        asserts! {
            overflowed => None, // 没有溢出
            bag.size() == 1, // 放回了
            bag.empty_level(0) => false, // 放入的是第0层
            bag.mass() == 1, // 放进第0层，获得(0+1)的重量
        }

        // ! 在袋内修改优先级
        let item_mut = bag.get_mut(key).expect("此时袋内必须有物品");
        item_mut.set_priority(ShortFloat::HALF);

        // 取出元素
        let taken = bag.take_out().unwrap();
        asserts! {
            taken.priority() == ShortFloat::HALF,
            bag.size() == 0, // 取走了
            bag.mass() == 0, // 取走了
            bag.empty_level(0) => true, // 取走的是第0层
        }

        // 最后完成
        ok!()
    }
}
