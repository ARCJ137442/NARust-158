//! 🎯复刻OpenNARS `nars.entity.Bag`

use super::distributor::Distributor;
use crate::{
    entity::{BudgetValue, Item, ShortFloat},
    global::Float,
    inference::BudgetFunctions,
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
/// * 🚩【2024-05-01 23:17:26】暂且按照OpenNARS的命名来：
///   * 📌因为直接使用`Item`而非`BagItem`，故相应地改其中的`Item`为`E`
///   * 📝此中之`E`其实亦代表「Entity」（首字母）
///

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
pub trait Bag<E>
where
    // * ↑此处`Item`泛型仿OpenNARS`Bag`
    E: Item,
{
    // ! 🚩【2024-05-04 11:49:53】现在「元素id类型」与「预算值」都包括在「元素」的类型之中
    // /// 元素id类型
    // /// * ❓要是引用类型还是值类型
    // ///   * 后续如何兼容`String`与`&str`
    // type Key: BagKey;

    // /// 预算值类型
    // /// * 🎯一种「袋」只有一种对「预算」的表征方式
    // type Budget: BudgetValue;

    // ! 🚩【2024-05-04 12:02:17】现在「分派器」无需被绑定作为关联类型（只要实现者返回一个就行，使用静态分派）
    // /// 分派器类型
    // /// * 🎯伪随机数生成
    // type Distributor: Distributor;

    /// 总层数
    /// * 🚩【2024-05-04 01:44:29】根据OpenNARS中「常量」的定义，在此将其全局化
    ///   * 📌`static final` ⇒ `const`
    ///
    /// # 📄OpenNARS `Bag.TOTAL_LEVEL`
    ///
    /// priority levels
    const __TOTAL_LEVEL: usize = DEFAULT_PARAMETERS.bag_level;

    /// 触发阈值
    /// * 📌触发の阈值
    ///
    /// # 📄OpenNARS `Bag.THRESHOLD`
    ///
    /// firing threshold
    const __THRESHOLD: usize = DEFAULT_PARAMETERS.bag_threshold;

    /// 相对阈值
    /// * 🚩由`触发阈值 / 总层数`计算得来
    ///
    /// # 📄OpenNARS `Bag.RELATIVE_THRESHOLD`
    ///
    /// relative threshold, only calculate once
    const __RELATIVE_THRESHOLD: Float = Self::__THRESHOLD as Float / Self::__TOTAL_LEVEL as Float;

    /// 加载因子
    /// * ❓尚不清楚其含义
    ///
    /// # 📄OpenNARS `Bag.LOAD_FACTOR`
    ///
    /// hash table load factor
    const __LOAD_FACTOR: Float = DEFAULT_PARAMETERS.load_factor;

    /// 【只读常量】分派器
    /// * ❌【2024-05-04 01:46:06】这个「静态常量」因为`Self::Distributor`没有「常量构造函数」而暂且还是以「特征方法」的形式存在
    /// * 🚩【2024-05-04 12:01:42】实际上并不需要强行把「分派器」绑定在「袋」上作为关联类型
    ///
    /// # 📄OpenNARS `Bag.DISTRIBUTOR`
    ///
    /// shared DISTRIBUTOR that produce the probability distribution
    fn __distributor(&self) -> &impl Distributor;

    /// 模拟`Bag.nameTable`属性
    /// * 🚩【2024-04-28 08:43:25】目前不与任何「映射」类型绑定
    ///   * ❌不打算直接返回[`HashMap`]
    /// # 📄OpenNARS `Bag.nameTable`
    ///
    /// mapping from key to item
    fn __name_table(&self) -> &impl BagNameTable<E>;
    fn __name_table_mut(&mut self) -> &mut impl BagNameTable<E>;

    /// 模拟`Bag.nameTable`的「构造赋值」
    /// * 🎯预期是「构造一个映射，并赋值给内部字段」
    /// * 📄出现在`init`方法中
    fn __name_table_mut_new_(&mut self);
    // end `nameTable`

    /// 模拟`Bag.itemTable`属性
    /// * 📝OpenNARS中基于「优先级」的元素获取
    /// * 🚩【2024-04-28 10:47:35】目前只获取「元素id」而非「元素」
    ///   * ⚠️后续直接`unwrap`：通过`name_table`保证元素存在
    /// * 📝Rust中需要「本体」和「本体_mut」两种函数，以便分别实现属性的「读写」
    ///   * ✅「本体」作为不可变者，允许在「不可变变量」中使用
    ///   * ⚠️若全部将「可变成员」作为可变引用`&mut 成员类型`返回，则这样的成员无法在「不可变变量」中使用
    ///     * 💭【2024-05-01 21:48:56】因此替换不等效
    ///
    /// # 📄OpenNARS `Bag.itemTable`
    ///
    /// array of lists of items, for items on different level
    fn __item_tale(&self) -> &impl BagItemTable<E::Key>;
    fn __item_tale_mut(&mut self) -> &mut impl BagItemTable<E::Key>;

    /// 模拟`Bag.itemTable`的「构造赋值」
    /// * 🎯预期是「构造一个双层数组，并赋值给内部字段」
    /// * 📄出现在`init`方法中
    fn __item_table_mut_new_(&mut self);
    // end `itemTable`

    /// 一个「袋」的「容量」
    /// * 🚩只读
    ///   * 📄`private final int capacity;`
    /// * 📝OpenNARS中作为「属性」定义，仅仅是为了「缓存数值」并「在子类中分派不同的『大小』作为常数返回值」用
    ///   * 🚩因此无需附带`setter`
    /// * 💭【2024-05-04 01:48:01】实际上可以被定义为「关联常量」
    ///
    /// # 📄OpenNARS `Bag.capacity`
    ///
    /// * 【作为属性】defined in different bags
    /// * 【作为方法】To get the capacity of the concrete subclass
    ///   * @return Bag capacity, in number of Items allowed
    fn __capacity(&self) -> usize;

    /// 一个「袋」已有元素的层数
    /// * 🚩会随着「增删元素」而变
    ///   * 🚩故需要一个「可变」版本
    ///   * 📝Rust允许`*self.__mass_mut() = XXX`的语法：左值可以是表达式
    ///
    /// # 📄OpenNARS `Bag.mass`
    ///
    /// current sum of occupied level
    fn __mass(&self) -> usize;
    fn __mass_mut(&mut self) -> &mut usize;

    /// 一个「袋」中用于指示「用于获取下一层级的索引」的状态量
    /// * 🎯用于在「分派器」中调用「下一层级」
    /// * 📄`levelIndex = capacity % TOTAL_LEVEL; // so that different bags start at different point`
    ///
    /// # 📄OpenNARS `Bag.levelIndex`
    ///
    /// index to get next level, kept in individual objects
    fn __level_index(&self) -> usize;
    fn __level_index_mut(&mut self) -> &mut usize;

    /// 一个「袋」中用于指示「当前层级」的状态量
    /// * ❓和`levelIndex`区别何在
    ///
    /// # 📄OpenNARS `Bag.currentLevel`
    ///
    /// current take out level
    fn __current_level(&self) -> usize;
    fn __current_level_mut(&mut self) -> &mut usize;

    /// 一个「袋」中用于指示「当前计数器」的状态量
    /// * 📝【2024-05-01 21:50:09】在OpenNARS中与「层级」有关
    ///
    /// # 📄OpenNARS `Bag.currentCounter`
    ///
    /// maximum number of items to be taken out at current level
    fn __current_counter(&self) -> usize;
    fn __current_counter_mut(&mut self) -> &mut usize;

    // ! ❌不对「记忆区」进行递归引用
    // * 🚩【2024-05-01 21:51:05】相反，将这些函数移除「实例方法」中，作为独立的函数处理
    //   * 🚧有待「记忆区」抽象接口实现
    // 📄在OpenNARS中用于`forgetRate`属性的实现，如`ConceptBag`中：
    // ```java
    // protected int forgetRate() {
    //     return memory.getConceptForgettingRate().get();
    // }
    // ```
    // /// 📄OpenNARS `Bag.memory`
    // ///
    // /// reference to memory
    // fn __memory(&self) -> impl Memory;

    // ! ❌不迁移「袋观察者」模式
    // * 📌【2024-05-01 21:52:26】不能完全照搬Java的设计模式
    // * 💭【2024-05-01 21:54:29】这个变量甚至没有注释……
    // fn __bag_observer(&self) -> impl BagObserver<Item>;

    // ! ❌不迁移「显示用变量」
    // /// 📄OpenNARS `Bag.showLevel`
    // ///
    // /// The display level; initialized at lowest
    // fn __show_level(&self) -> usize;
    // fn __show_level_mut(&mut self) -> &mut usize;

    // ** 属性迁移完毕 ** //

    // ! ❌不迁移「传承的构造函数」
    // 📄OpenNARS源码：
    // ```java
    // /**
    //  * constructor, called from subclasses
    //  *
    //  * @param memory The reference to memory
    //  */
    // protected Bag(Memory memory) {
    //     this.memory = memory;
    //     capacity = capacity();
    //     init();
    // }
    // ```

    /// 模拟`Bag.init`
    ///
    /// # 📄OpenNARS `Bag.init`
    ///
    /// 🈚
    fn init(&mut self) {
        /* itemTable = new ArrayList<>(TOTAL_LEVEL);
        for (int i = 0; i < TOTAL_LEVEL; i++) {
            itemTable.add(new LinkedList<E>());
        }
        nameTable = new HashMap<>((int) (capacity / LOAD_FACTOR), LOAD_FACTOR);
        currentLevel = TOTAL_LEVEL - 1;
        levelIndex = capacity % TOTAL_LEVEL; // so that different bags start at different point
        mass = 0;
        currentCounter = 0; */
        self.__item_table_mut_new_(); // 🚩「添加新层级的代码」亦在其中，以实现功能解耦
        self.__name_table_mut_new_();
        *self.__current_level_mut() = Self::__TOTAL_LEVEL - 1;
        *self.__level_index_mut() = self.__capacity() % Self::__TOTAL_LEVEL; // 不同的「袋」在分派器中有不同的起点
        *self.__mass_mut() = 0;
        *self.__current_counter_mut() = 0;
    }

    // ! 🚩`Bag.capacity`已在`self.__capacity`中实现

    /// 模拟`Bag.forgetRate`
    /// * 📝用于并体现AIKR所衍生的「资源竞争」思想
    /// * 🚩【2024-05-04 12:00:04】OpenNARS中该值不可变，且多为常量（任务链袋中还与「记忆区」相关）
    ///
    /// # 📄OpenNARS `Bag.forgetRate`
    ///
    /// Get the item decay rate,
    /// which differs in difference subclass,
    /// and **can be changed in run time by the user**, so not a constant.
    ///
    /// @return The number of times for a decay factor to be fully applied
    fn _forget_rate(&self) -> usize;

    /// 模拟`Bag.size`
    /// * 🎯从模拟`Bag.nameTable`派生
    /// * 🚩转发内部`name_table`成员
    ///
    /// # 📄OpenNARS `Bag.size`
    ///
    /// The number of items in the bag
    #[inline(always)]
    fn size(&self) -> usize {
        self.__name_table().size()
    }

    /// 模拟`Bag.averagePriority`
    ///
    /// # 📄OpenNARS `Bag.averagePriority`
    ///
    /// Get the average priority of Items
    ///
    /// @return The average priority of Items in the bag
    fn average_priority(&self) -> Float {
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
            (self.__mass() as Float) / (self.size() * Self::__TOTAL_LEVEL) as Float,
            1.0,
        )
    }

    /// 模拟`Bag.contains`
    /// * 🎯从模拟`Bag.nameTable.containsValue`派生
    /// * 📜默认使用[`Self::get`]
    ///
    /// # 📄OpenNARS `Bag.contains`
    ///
    /// Check if the bag contains the item
    ///
    /// @param item The item to be checked
    /// @return Whether the bag contains the item
    #[inline(always)]
    fn contains(&self, item: &E) -> bool {
        self.get(item.key()).is_some()
    }

    /// 模拟`Bag.get`
    /// * 🚩转发内部`name_table`成员
    ///
    /// # 📄OpenNARS `Bag.get`
    ///
    /// Get an Item by key
    ///
    /// @param key The key of the Item
    /// @return The Item with the given key
    #[inline(always)]
    fn get(&self, key: &E::Key) -> Option<&E> {
        self.__name_table().get(key)
    }
    /// [`Self::get`]的可变版本
    /// * 🎯【2024-04-28 09:08:14】备用
    /// * 🚩转发内部`name_table`成员
    #[inline(always)]
    fn get_mut(&mut self, key: &E::Key) -> Option<&mut E> {
        self.__name_table_mut().get_mut(key)
    }

    /// 模拟`Bag.putIn`
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
    fn put_in(&mut self, new_item: E) -> Option<E> {
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
        // ! ❓【2024-05-01 22:44:45】此处内联`key_cloned`会出现莫名其妙的借用问题：`clone`了还说「已被借用」
        /* 📝亦有一个使用`unsafe`的解决方案：
        let new_key = unsafe {
            let this: *const Item = &new_item;
            this.as_ref().unwrap().key()
        };
        let old_item = self.__name_table_mut().put(new_key, new_item);
        */
        let new_key = new_item.____key_cloned();
        let old_item = self.__name_table_mut().put(&new_key, new_item);
        let new_item = self.get_mut(&new_key).unwrap(); // * 🚩🆕重新获取「置入后的新项」（⚠️一定有）

        // 若在「元素映射」中重复了：有旧项⇒合并「重复了的新旧项」
        if let Some(old_item) = old_item {
            // 将旧项（的预算值）并入新项 | 🆕⚠️必须在前：`new_item`可变借用了`self`，而下一句中不能出现`new_item`
            new_item.merge(&old_item);
            // 在「层级映射」移除旧项 | 🚩【2024-05-04 11:45:02】现在仍需使用「元素」，因为下层调用需要访问元素本身（预算值），并需避免过多的「按键取值」过程
            self._out_of_base(&old_item);
        }

        // 置入「层级映射」
        // 若在「层级映射」中溢出了：若有「溢出」则在「元素映射」中移除
        // ! 📌【2024-05-04 11:35:45】↓此处`__into_base`仅传入「元素id」是为了规避借用问题（此时`new_item`已失效）
        if let Some(overflow_key) = self.__into_base(&new_key) {
            // 直接返回「根据『溢出的元素之id』在『元素映射』中移除」的结果
            // * 🚩若与自身相同⇒返回`Some`，添加失败
            // * 🚩若与自身不同⇒返回`None`，添加仍然成功
            let overflow_item = self.__name_table_mut().remove(&overflow_key);
            match overflow_key == new_key {
                true => overflow_item,
                false => None, // ! 此时将
            }
        } else {
            None
        }
    }

    /// 模拟`Bag.putBack`
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
    fn put_back(&mut self, mut old_item: E) -> Option<E> {
        /* 📄OpenNARS源码：
        BudgetFunctions.forget(oldItem.getBudget(), forgetRate(), RELATIVE_THRESHOLD);
        return putIn(oldItem); */
        old_item
            .budget_mut()
            .forget(self._forget_rate() as Float, Self::__RELATIVE_THRESHOLD);
        self.put_in(old_item)
    }

    /// 模拟`Bag.takeOut`
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
    fn take_out(&mut self) -> Option<E> {
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
        if self.__name_table().is_empty() {
            return None;
        }
        if self._empty_level(self.__current_level()) || self.__current_counter() == 0 {
            *self.__current_level_mut() = self.__distributor().pick(self.__level_index());
            *self.__level_index_mut() = self.__distributor().next(self.__level_index());
            while self._empty_level(self.__current_level()) {
                // * 📝这里实际上就是一个do-while
                *self.__current_level_mut() = self.__distributor().pick(self.__level_index());
                *self.__level_index_mut() = self.__distributor().next(self.__level_index());
            }
            if self.__current_level() < Self::__THRESHOLD {
                *self.__current_counter_mut() = 1;
            } else {
                *self.__current_counter_mut() =
                    self.__item_tale().get(self.__current_level()).size();
            }
        }
        let selected_key = self.__take_out_first(self.__current_level());
        *self.__current_counter_mut() -= 1;
        // * 此处需要对内部可能有的「元素id」进行转换
        let selected;
        if let Some(key) = selected_key {
            selected = self.__name_table_mut().remove(&key)
        } else {
            selected = None
        }
        // self.refresh(); // ! ❌【2024-05-04 11:16:55】不复刻这个有关「观察者」的方法
        selected
    }

    /// 模拟`Bag.pickOut`
    ///
    /// # 📄OpenNARS
    ///
    /// Pick an item by key, then remove it from the bag
    ///
    /// @param key The given key
    /// @return The Item with the key
    fn pick_out(&mut self, key: &E::Key) -> Option<E> {
        /* 📄OpenNARS源码：
        E picked = nameTable.get(key);
        if (picked != null) {
            outOfBase(picked);
            nameTable.remove(key);
        }
        return picked; */
        let picked_key = self.__name_table().get(key).map(E::key).cloned();
        let picked;
        if let Some(key) = picked_key {
            let item = self.__name_table_mut().remove(&key).unwrap(); // 此时一定有
            self._out_of_base(&item);
            picked = Some(item);
        } else {
            picked = None
        }
        picked
    }

    /// 模拟`Bag.emptyLevel`
    ///
    /// # 📄OpenNARS
    ///
    /// Check whether a level is empty
    ///
    /// @param n The level index
    /// @return Whether that level is empty
    fn _empty_level(&self, level: usize) -> bool {
        /* 📄OpenNARS源码：
        return (itemTable.get(n).isEmpty()); */
        self.__item_tale().get(level).is_empty()
    }

    /// 模拟`Bag.getLevel`
    /// * 📝Rust中[`usize`]无需考虑负值问题
    /// *
    ///
    /// # 📄OpenNARS
    ///
    /// Decide the put-in level according to priority
    ///
    /// @param item The Item to put in
    /// @return The put-in level
    fn __get_level(&self, item: &E) -> usize {
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
    ///
    /// # 📄OpenNARS
    ///
    /// Insert an item into the itemTable, and return the overflow
    ///
    /// @param newItem The Item to put in
    /// @return The overflow Item
    fn __into_base(&mut self, new_key: &E::Key) -> Option<E::Key> {
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
        return oldItem; // TODO return null is a bad smell */
        let new_item = self.get(new_key).expect("不能没有所要获取的值"); // * 🚩🆕（在调用方处）重新获取「置入后的新项」（⚠️一定有）
        let mut old_item = None;
        let in_level = self.__get_level(new_item);
        if self.size() > self.__capacity() {
            let mut out_level = 0;
            while self._empty_level(out_level) {
                out_level += 1;
            }
            if out_level > in_level {
                return Some(new_item.____key_cloned());
            } else {
                old_item = self.__take_out_first(out_level);
            }
        }
        self.__item_tale_mut()
            .get_mut(in_level)
            .add(new_key.clone());
        *self.__mass_mut() += in_level + 1;
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
    fn __take_out_first(&mut self, level: usize) -> Option<E::Key> {
        /* 📄OpenNARS源码：
        E selected = itemTable.get(level).getFirst();
        itemTable.get(level).removeFirst();
        mass -= (level + 1);
        refresh();
        return selected; */
        let selected = self.__item_tale().get(level).get_first().cloned();
        self.__item_tale_mut().get_mut(level).remove_first();
        *self.__mass_mut() -= level + 1;
        selected
    }

    /// 模拟`Bag.outOfBase`
    ///
    /// # 📄OpenNARS
    ///
    /// Remove an item from itemTable, then adjust mass
    ///
    /// @param oldItem The Item to be removed
    fn _out_of_base(&mut self, old_item: &E) {
        /* 📄OpenNARS源码：
        int level = getLevel(oldItem);
        itemTable.get(level).remove(oldItem);
        mass -= (level + 1);
        refresh(); */
        let level = self.__get_level(old_item);
        self.__item_tale_mut().get_mut(level).remove(old_item.key());
        *self.__mass_mut() -= level + 1;
        // self.refresh() // ! ❌【2024-05-04 11:46:09】不复刻这个有关「观察者」的方法
    }

    // ! ❌【2024-05-04 01:57:00】有关「观察者」「呈现用」的方法，此处暂且不进行复刻

    // ! ❌addBagObserver
    // ! ❌play
    // ! ❌stop
    // ! ❌refresh
    // ! ❌toString
    // ! ❌toStringLong
}

/// 用于袋的「索引」
/// * 🎯方便后续安插方法
pub trait BagKey: Clone + Eq {}

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
    fn get(&self, key: &E::Key) -> Option<&E>;
    /// [`Self::get`]的可变引用版本
    /// * 🎯【2024-04-28 09:27:23】备用
    fn get_mut(&mut self, key: &E::Key) -> Option<&mut E>;

    /// 模拟`Bag.nameTable.put`方法
    /// * 🎯预期是「向映射插入值」
    /// * 📄出现在`putIn`方法中
    /// * 🚩需要返回「被替换出的旧有项」
    fn put(&mut self, key: &E::Key, item: E) -> Option<E>;

    /// 模拟`Bag.nameTable.remove`方法
    /// * 🎯预期是「从映射移除值」
    /// * 📄出现在`putIn`方法中
    /// * 🚩【2024-05-01 23:03:15】现在需要返回「被移除的元素」作为[`Bag::put_in`]的返回值
    fn remove(&mut self, key: &E::Key) -> Option<E>;

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

/// 初代实现
mod impl_v1 {
    use super::*;
    use crate::storage::DistributorV1;

    // 默认实现 //
    use std::collections::{HashMap, VecDeque};

    /// 📜为字符串实现「元素id」
    impl BagKey for String {}

    /// 📜为「散列映射」[`HashMap`]实现「元素映射」
    /// * 📝同名方法冲突时，避免「循环调用」的方法：完全限定语法
    ///   * 🔗<https://rustc-dev-guide.rust-lang.org/method-lookup.html>
    ///   * ⚠️[`HashMap`]使用[`len`](HashMap::len)而非[`size`](BagNameTable::size)
    impl<E> BagNameTable<E> for HashMap<String, E>
    where
        E: Item<Key = String>,
    {
        #[inline(always)]
        fn size(&self) -> usize {
            self.len()
        }

        #[inline(always)]
        fn get(&self, key: &String) -> Option<&E> {
            Self::get(self, key)
        }

        #[inline(always)]
        fn get_mut(&mut self, key: &String) -> Option<&mut E> {
            Self::get_mut(self, key)
        }

        #[inline(always)]
        fn put(&mut self, key: &String, item: E) -> Option<E> {
            match self.contains_key(key) {
                true => self.insert(key.clone(), item),
                false => None,
            }
        }

        #[inline(always)]
        fn remove(&mut self, key: &String) -> Option<E> {
            Self::remove(self, key)
        }
    }

    /// 📜为「队列列表」[`Vec<VecDeque>`](Vec)实现「层级映射」
    /// * 🚩基于「元素id」的索引：不存储元素值
    ///   * 📝Java的情况可被视作`Arc`
    impl<Key> BagItemTable<Key> for Vec<VecDeque<Key>>
    where
        Key: BagKey, // * 需要在「具体值匹配删除」时用到
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
        Key: BagKey, // * 需要在「具体值匹配删除」时用到
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

    /// 第一版「袋」
    pub struct BagV1<E: Item> {
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
        item_map: HashMap<E::Key, E>,

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
        level_map: Vec<VecDeque<E::Key>>,

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

        /// 遗忘速率
        /// * 📌在不同地方有不同的定义
        /// * 📝是一个「构造时固定」的属性
        /// * 📝OpenNARS用于[`Bag::put_back`]的「放回时遗忘」中
        ///
        /// # 📄OpenNARS `Bag.forgetRate`
        ///
        /// Get the item decay rate, which differs in difference subclass, and can be
        /// changed in run time by the user, so not a constant.
        ///
        /// @return The number of times for a decay factor to be fully applied
        forget_rate: usize,

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

    /// 对「以字符串为索引的袋」实现特征
    /// * 🚩【2024-05-04 12:01:15】下面这些就是给出自己的属性，即「属性映射」
    impl<E: Item<Key = String>> Bag<E> for BagV1<E> {
        fn __distributor(&self) -> &impl Distributor {
            &self.distributor
        }

        fn __name_table(&self) -> &impl BagNameTable<E> {
            // * ⚠️【2024-05-04 11:54:07】目前只有「字符串key」的「散列映射」实现了「名称表」
            &self.item_map
        }

        fn __name_table_mut(&mut self) -> &mut impl BagNameTable<E> {
            &mut self.item_map
        }

        fn __name_table_mut_new_(&mut self) {
            self.item_map = HashMap::new();
        }

        fn __item_tale(&self) -> &impl BagItemTable<<E as Item>::Key> {
            &self.level_map
        }

        fn __item_tale_mut(&mut self) -> &mut impl BagItemTable<<E as Item>::Key> {
            &mut self.level_map
        }

        fn __item_table_mut_new_(&mut self) {
            self.level_map = Vec::new();
        }

        fn __capacity(&self) -> usize {
            self.capacity
        }

        fn __mass(&self) -> usize {
            self.mass
        }

        fn __mass_mut(&mut self) -> &mut usize {
            &mut self.mass
        }

        fn __level_index(&self) -> usize {
            self.level_index
        }

        fn __level_index_mut(&mut self) -> &mut usize {
            &mut self.level_index
        }

        fn __current_level(&self) -> usize {
            self.current_level
        }

        fn __current_level_mut(&mut self) -> &mut usize {
            &mut self.current_level
        }

        fn __current_counter(&self) -> usize {
            self.current_counter
        }

        fn __current_counter_mut(&mut self) -> &mut usize {
            &mut self.current_counter
        }

        fn _forget_rate(&self) -> usize {
            self.forget_rate
        }
    }
}
pub use impl_v1::*;

/// TODO: 单元测试
#[cfg(test)]
mod tests {
    use super::*;
    use crate::{global::tests::AResult, ok};

    #[test]
    fn test_bag() -> AResult {
        ok!()
    }
}
