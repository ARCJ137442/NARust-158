//! 🎯复刻OpenNARS `nars.entity.Bag`

use super::{
    distributor::{Distribute, Distributor},
    BagItemTable, BagItemTableV1, BagNameTable, BagNameTableV1, Bagging,
};
use crate::{__impl_to_display_and_display, entity::Item, nars::DEFAULT_PARAMETERS};

// ! 删除「具体类型」特征：能直接`struct`就直接`struct`

/// 第一版「袋」
/// * 仅用作功能测试，不用作实际功能
///   * 💭【2024-05-04 16:24:13】一些诸如「遗忘时长」的「超参数」仍然需要让具体实现去处理
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
    item_map: BagNameTableV1<E>,

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
    level_map: BagItemTableV1,

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
    // ! ❌不作`memory: Memory`循环引用：所有涉及memory的方法，均移动到Memory中解决（另外，OpenNARS中也没多少地方用到这个引用）
    // memory: Memory,

    // ! ❌不作`bagObserver: BagObserver<Item>`观察者：不引入Java的「观察者模式」
    // ! ❌不作`showLevel: usize`显示用变量：不用于显示
}

impl<E: Item> Default for Bag<E> {
    /// * �【2024-05-04 16:26:53】默认当「概念袋」使
    fn default() -> Self {
        Self::new(
            DEFAULT_PARAMETERS.concept_bag_size,
            DEFAULT_PARAMETERS.concept_forgetting_cycle,
        )
    }
}

// impl<E: Item> BagConcrete<E> for Bag<E> {
impl<E: Item> Bag<E> {
    pub fn new(capacity: usize, forget_rate: usize) -> Self
    where
        Self: Sized,
    {
        /* 📄OpenNARS源码：
        this.memory = memory;
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
            item_map: BagNameTableV1::default(),
            level_map: BagItemTableV1::default(),
            mass: usize::default(),
            level_index: usize::default(),
            current_level: usize::default(),
            current_counter: usize::default(),
        };
        this.init();
        this
    }
}

/// 对「以字符串为索引的袋」实现特征
/// * 🚩【2024-05-04 12:01:15】下面这些就是给出自己的属性，即「属性映射」
impl<E: Item> Bagging<E> for Bag<E> {
    #[inline(always)]
    fn __distributor(&self) -> &impl Distribute {
        &self.distributor
    }

    #[inline(always)]
    fn __name_table(&self) -> &impl BagNameTable<E> {
        // * ⚠️【2024-05-04 11:54:07】目前只有「字符串key」的「散列映射」实现了「名称表」
        &self.item_map
    }

    #[inline(always)]
    fn __name_table_mut(&mut self) -> &mut impl BagNameTable<E> {
        &mut self.item_map
    }

    #[inline(always)]
    fn __name_table_mut_new_(&mut self) {
        self.item_map = BagNameTableV1::new();
    }

    #[inline(always)]
    fn __item_table(&self) -> &impl BagItemTable {
        &self.level_map
    }

    #[inline(always)]
    fn __item_table_mut(&mut self) -> &mut impl BagItemTable {
        &mut self.level_map
    }

    #[inline(always)]
    fn __item_table_mut_new_(&mut self) {
        // * 🚩只在这里初始化
        self.level_map = BagItemTableV1::new(Self::__TOTAL_LEVEL);
    }

    #[inline(always)]
    fn __capacity(&self) -> usize {
        self.capacity
    }

    #[inline(always)]
    fn __mass(&self) -> usize {
        self.mass
    }

    #[inline(always)]
    fn __mass_mut(&mut self) -> &mut usize {
        &mut self.mass
    }

    #[inline(always)]
    fn __level_index(&self) -> usize {
        self.level_index
    }

    #[inline(always)]
    fn __level_index_mut(&mut self) -> &mut usize {
        &mut self.level_index
    }

    #[inline(always)]
    fn __current_level(&self) -> usize {
        self.current_level
    }

    #[inline(always)]
    fn __current_level_mut(&mut self) -> &mut usize {
        &mut self.current_level
    }

    #[inline(always)]
    fn __current_counter(&self) -> usize {
        self.current_counter
    }

    #[inline(always)]
    fn __current_counter_mut(&mut self) -> &mut usize {
        &mut self.current_counter
    }

    #[inline(always)]
    fn _forget_rate(&self) -> usize {
        self.forget_rate
    }
}

// 显示呈现方法：自动分派
__impl_to_display_and_display! {
    @(bag_to_display;;)
    {E: Item}
    Bag<E> as Bagging<E>
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
            bag.__mass() == 0, // 空的
            bag._empty_level(0) => true, // 第0层也是空的
        }

        // 放入元素
        let key1 = "item001";
        let item1 = new_item(key1, 0.0, 0.0, 0.0); // * 🚩固定为「全零预算」
        let overflowed = bag.put_in(dbg!(item1.clone()));
        asserts! {
            overflowed.is_none(), // 没有溢出
            bag.get(key1) == Some(&item1), // 放进「对应id位置」的就是原来的元素
            bag.size() == 1, // 放进了一个
            bag.__get_level(&item1) => 0, // 放进的是第0层（优先级为0.0）
            bag._empty_level(0) => false, // 放进的是第0层
            bag.__mass() == 1, // 放进第0层，获得(0+1)的重量
        }
        dbg!(&bag);

        // 挑出元素
        let picked = bag.pick_out(key1).unwrap();
        asserts! {
            picked == item1, // 挑出的就是所置入的
            bag.size() == 0, // 取走了
            bag.__mass() == 0, // 取走了
            bag._empty_level(0) => true, // 取走的是第0层
        }

        // 放回元素
        bag.put_back(picked);
        asserts! {
            bag.size() == 1, // 放回了
            bag._empty_level(0) => false, // 放入的是第0层
            bag.__mass() == 1, // 放进第0层，获得(0+1)的重量
        }

        // 取出元素
        let mut taken = bag.take_out().unwrap();
        asserts! {
            taken == item1, // 取出的就是放回了的
            bag.size() == 0, // 取走了
            bag.__mass() == 0, // 取走了
            bag._empty_level(0) => true, // 取走的是第0层
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
        bag.put_back(taken);
        asserts! {
            bag.size() == 1, // 放回了
            bag._empty_level(0) => true, // 放入的不再是第0层
            bag._empty_level(Bag1::__TOTAL_LEVEL-1) => false, // 放入的是最高层
            bag.__mass() == Bag1::__TOTAL_LEVEL, // 放进第最高层，获得 层数 的重量
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
            bag._empty_level(0) => true, // 第0层也是空的
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
                bag.__get_level(item) => expected_level(i), // 放进了指定层
                bag._empty_level(expected_level(i)) => false, // 放进的是指定层
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
                bag._empty_level(expected_level(i)) => true, // 取走的是指定层
            }
            picked_items.push(picked);
        }

        // 放回元素
        for (i, picked) in picked_items.into_iter().enumerate() {
            bag.put_back(picked); // 此时预算值也改变了：会衰减
            asserts! {
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
            bag.put_back(taken);
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
            bag.__mass() == 0, // 空的
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
            bag.__mass() >= 1, // 放进了，获得重量
        }
        dbg!(&bag);

        // 多次取出放回 | // * 📝根据[`BudgetFunctions::forget`]，实际上只有「优先级」会变化
        println!("budget trending from {budget_initial}:");
        for _ in 0..N {
            let taken = bag.take_out().unwrap(); // 一定拿得出来

            // 检查、展示
            asserts! {
                bag.size() == 0, // 取出了
                bag.__mass() == 0, // 失去所有重量
            };
            println!("\t{}", taken.budget());

            //放回元素
            bag.put_back(taken);
        }
        println!("{}", bag.to_display_long());

        // 最终完成
        ok!()
    }
}
