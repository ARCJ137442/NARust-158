//! 🎯复刻OpenNARS `nars.entity.Memory`
//! * 📌「记忆区」
//! * 🚧【2024-05-07 18:52:42】目前复现方法：先函数API（提供函数签名），再翻译填充函数体代码
//!
//! * ✅【2024-05-08 15:46:28】目前已初步实现方法API，并完成部分方法模拟
//! * ✅【2024-05-08 17:17:41】目前已初步完成所有方法的模拟
//! * ♻️【2024-06-24 20:40:08】开始基于改版OpenNARS重写

use super::Bag;
use crate::{
    control::{prepare_term_link_templates, Parameters, DEFAULT_PARAMETERS},
    entity::{BudgetValue, Concept, Item, RCTask},
    inference::{Budget, BudgetFunctions},
    language::Term,
};
use serde::{Deserialize, Deserializer, Serialize};

/// 记忆区
#[derive(Debug, Serialize, Deserialize)]
pub struct Memory {
    /// 概念袋
    ///
    /// # 📄OpenNARS
    ///
    /// Concept bag. Containing all Concepts of the system
    #[serde(deserialize_with = "Memory::deserialize_concepts")]
    concepts: Bag<Concept>,

    /// 🆕【内部】统一所有「超参数」的存储
    /// * 🎯便于「不依赖推理器使用参数」
    ///   * 💭【2024-08-12 14:09:21】后续是否可能用共享引用？
    /// * 📌【2024-09-04 10:15:43】目前作为整个「推理器状态」的参数存储
    ///  * ❓TODO: 【2024-09-04 10:17:29】考虑是否特化到「所用参数」并在「推理器状态」中独立存储
    pub(crate) parameters: Parameters,
}

impl Memory {
    /// 获取概念遗忘速率
    /// * 🎯概念构造
    pub fn concept_forgetting_rate(&self) -> usize {
        self.parameters.concept_forgetting_cycle
    }

    /// 获取信念遗忘速率
    /// * 🎯概念构造
    #[doc(alias = "belief_forgetting_rate")]
    pub fn term_link_forgetting_rate(&self) -> usize {
        self.parameters.term_link_forgetting_cycle
    }

    /// 获取任务遗忘速率
    /// * 🎯概念构造
    #[doc(alias = "task_forgetting_rate")]
    pub fn task_link_forgetting_rate(&self) -> usize {
        self.parameters.task_link_forgetting_cycle
    }

    /// 构造函数
    pub fn new(parameters: Parameters) -> Self {
        Self {
            // * 🚩概念袋
            concepts: Bag::new(
                parameters.concept_bag_size,
                parameters.concept_forgetting_cycle,
            ),
            // * 🚩超参数
            parameters,
        }
    }

    /// 初始化记忆区
    /// * 🚩初始化「概念袋」
    pub fn init(&mut self) {
        self.concepts.init();
    }

    /// # 📄OpenNARS
    ///
    /// Get an existing Concept for a given name
    ///
    /// called from Term and ConceptWindow.
    #[doc(alias = "name_to_concept")]
    pub fn key_to_concept(&self, key: &str) -> Option<&Concept> {
        self.concepts.get(key)
    }
    #[doc(alias = "name_to_concept_mut")]
    pub fn key_to_concept_mut(&mut self, key: &str) -> Option<&mut Concept> {
        self.concepts.get_mut(key)
    }

    /// 统一集中「词项→袋索引」的逻辑
    pub fn term_to_key(term: &Term) -> String {
        term.name()
    }

    /// # 📄OpenNARS
    ///
    /// Get an existing Concept for a given Term.
    pub fn term_to_concept(&self, term: &Term) -> Option<&Concept> {
        self.key_to_concept(&Self::term_to_key(term))
    }
    pub fn term_to_concept_mut(&mut self, term: &Term) -> Option<&mut Concept> {
        self.key_to_concept_mut(&Self::term_to_key(term))
    }

    pub fn has_concept(&self, term: &Term) -> bool {
        self.concepts.has(&Self::term_to_key(term))
    }

    /// # 📄OpenNARS
    ///
    /// Get the Concept associated to a Term, or create it.
    pub fn get_concept_or_create(&mut self, term: &Term) -> Option<&mut Concept> {
        // * 🚩不给「非常量词项」新建概念 | 「非常量词项」也不可能作为一个「概念」被放进「记忆区」中
        if !term.is_constant() {
            return None;
        }
        // * 🚩尝试从概念袋中获取「已有概念」，否则尝试创建概念
        let has_concept = self.has_concept(term);
        match has_concept {
            // * ⚠️【2024-06-25 01:15:35】不能通过匹配`term_to_concept_mut`判断：可能会有「重复可变借用」嫌疑
            true => self.term_to_concept_mut(term),
            false => self.make_new_concept(term),
        }
    }

    fn make_new_concept(&mut self, term: &Term) -> Option<&mut Concept> {
        // the only place to make a new Concept
        // * 🚩创建新概念
        let concept = Concept::new(
            term.clone(),
            (&self.parameters).into(),
            self.concept_initial_budget(),
            prepare_term_link_templates(term),
        );
        let new_key = concept.key().clone();
        // * 🚩将新概念放入「记忆区」
        let old_concept = self.concepts.put_in(concept);
        let make_success = match old_concept {
            None => true,
            Some(old) => old.key() != &new_key,
        };
        // * 🚩根据「是否放入成功」返回「创建后的概念」
        match make_success {
            true => self.key_to_concept_mut(&new_key),
            false => None,
        }
    }

    /// 获取概念的「初始预算」
    /// * 🚩从自身所存储的「超参数」中构建
    fn concept_initial_budget(&self) -> BudgetValue {
        BudgetValue::from_floats(
            self.parameters.concept_initial_priority,
            self.parameters.concept_initial_durability,
            self.parameters.concept_initial_quality,
        )
    }

    /// Adjust the activation level of a Concept
    ///
    /// called in Concept.insertTaskLink only
    /// * 🚩实际上也被「直接推理」调用
    /// * 🚩【2024-06-25 01:46:20】此处为了避免「借用冲突」选择靠「词项」而非「概念」查询
    /// * 🚩【2024-06-25 02:03:57】目前因为「激活时需要使用不可变引用，修改时又需要可变引用」改为「返回新预算值」机制
    #[must_use]
    pub fn activate_concept_calculate(
        &self,
        concept: &Concept,
        incoming_budget: &impl Budget,
    ) -> BudgetValue {
        // * 📝先「激活」
        let mut activated = incoming_budget.activate_to_concept(concept);
        // * 🚩分「是否已有」判断
        match self.has_concept(concept.term()) {
            // * 🚩已有：只需「激活」 | 后续「放回」将由「袋」自己的机制做
            true => activated,
            // * 🚩没有：需要附加「遗忘」 | 在「袋」外边的「概念」需要手动「遗忘」才能让两个分支效果一致
            false => {
                self.concepts.forget(&mut activated);
                activated
            }
        }
    }

    /// * 🚩【2024-06-25 02:22:31】为避免「记忆区和概念同时可变借用」拆分成两块
    ///   * 📍计算：仅负责计算概念词项
    ///   * 📍应用：将计算出的「新预算值」用在实际对「概念」的修改中
    /// * 🎯避免「同时可变借用记忆区和其内的概念」冲突
    pub fn activate_concept_apply(concept: &mut impl Budget, new_budget: BudgetValue) {
        concept.copy_budget_from(&new_budget);
    }

    /// 🆕对外接口：从「概念袋」中拿出一个概念
    pub fn take_out_concept(&mut self) -> Option<Concept> {
        self.concepts.take_out()
    }

    /// 🆕对外接口：从「概念袋」中挑出一个概念
    /// * 🚩用于「直接推理」中的「拿出概念」
    pub fn pick_out_concept(&mut self, key: &str) -> Option<Concept> {
        self.concepts.pick_out(key)
    }

    /// 🆕对外接口：往「概念袋」放回一个概念
    pub fn put_back_concept(&mut self, concept: Concept) -> Option<Concept> {
        self.concepts.put_back(concept)
    }

    /// 🆕对外接口：只读迭代内部所有「概念」
    pub fn iter_concepts(&self) -> impl Iterator<Item = &Concept> {
        self.concepts.iter()
    }
}

impl Default for Memory {
    fn default() -> Self {
        // * 🚩超参数实现了[`Copy`]
        Self::new(DEFAULT_PARAMETERS)
    }
}

/// 针对[`serde`]做特殊调整
/// * 🎯原本需求是「在自动派生之方法的基础上，归一化其中的『任务共享引用』」
/// * 💡目前实际上「任务共享引用」只存在于「概念袋」中，那为何不在「概念袋」处做优化？
///   * 🚩【2024-08-12 01:28:31】当前做法：在反序列化「概念袋」时因【字段】插入「任务引用归一化」代码
///   * ✅这样便可省去「调用方还要再归一一次」的烦恼
impl Memory {
    /// 反序列化「概念袋」
    /// * 🚩在默认反序列化逻辑上，再加对内部所有「任务共享引用」的归一化处理
    fn deserialize_concepts<'de, D>(deserializer: D) -> Result<Bag<Concept>, D::Error>
    where
        D: Deserializer<'de>,
    {
        // 先反序列化到普通概念袋
        let mut bag = Bag::<Concept>::deserialize(deserializer)?;
        // 开始遍历所有「任务共享引用」，并归一化其值
        let all_task_rcs = Self::concept_bag_all_task_rcs(&mut bag);
        RCTask::unify_rcs(all_task_rcs);
        // 返回归一化后的概念袋
        Ok(bag)
    }

    /// 【内部】遍历其概念袋内所有「任务共享引用」
    /// * 🎯在「反序列化概念袋」与「参与反序列化上层」之间 共享代码
    fn concept_bag_all_task_rcs(bag: &mut Bag<Concept>) -> impl Iterator<Item = &mut RCTask> {
        bag.iter_mut().flat_map(Concept::iter_tasks_mut)
    }

    /// 【内部】遍历其内所有「任务共享引用」
    /// * 🎯反序列化时与上层如「推理器状态」一同归一化
    /// * ⚠️不包括各个「任务」的「父任务」字段
    ///   * 后者会在[「任务共享引用归一化」](RCTask::unify_rcs)中处理
    pub(crate) fn all_task_rcs(&mut self) -> impl Iterator<Item = &mut RCTask> {
        Self::concept_bag_all_task_rcs(&mut self.concepts)
    }
}

#[cfg(test)]
pub mod tests_memory {
    use super::*;
    use crate::{
        assert_eq_try,
        control::DEFAULT_PARAMETERS,
        entity::*,
        ok, test_term as term,
        util::{AResult, RefCount, ToDisplayAndBrief},
    };
    use nar_dev_utils::*;

    /// 保存前判断是否同步
    /// * 判断「[`Rc`]是否在『传入值所有权』后仍尝试移动内部值」
    pub fn memory_synced(memory: &impl GetMemory) {
        memory
            .get_memory()
            .iter_concepts()
            .flat_map(Concept::iter_tasks)
            .for_each(rc_synced)
    }

    pub fn rc_synced(rc: &RCTask) {
        assert!(
            rc.is_synced_serial(),
            "共享指针不同步：{} != {}",
            rc.serial_(),
            rc.inner_serial_()
        )
    }

    /// 顶层实用函数：迭代器zip
    /// * 🎯让语法`a.zip(b)`变成`zip(a, b)`
    pub fn zip<'t, T: 't, I1, I2>(a: I1, b: I2) -> impl Iterator<Item = (T, T)>
    where
        I1: IntoIterator<Item = T> + 't,
        I2: IntoIterator<Item = T> + 't,
    {
        a.into_iter().zip(b)
    }

    /// 用于在外部crate中直接用推理器检查记忆区，且不使用[`Reasoner::memory`]字段
    pub trait GetMemory {
        fn get_memory(&self) -> &Memory;
    }

    impl GetMemory for Memory {
        fn get_memory(&self) -> &Memory {
            self
        }
    }

    /// 手动检查俩记忆区是否一致
    /// * 📝对「记忆区」因为「共享引用无法准确判等（按引用）」只能由此验证
    pub fn memory_consistent<M1: GetMemory, M2: GetMemory>(old: &M1, new: &M2) -> AResult {
        let [old, new] = [old.get_memory(), new.get_memory()];
        // 参数一致
        assert_eq_try!(
            &old.parameters,
            &new.parameters,
            "记忆区不一致——超参数不一致"
        );
        // 概念袋一致
        bag_consistent(&old.concepts, &new.concepts, concept_consistent)?;
        ok!()
    }

    /// 检查「袋」是否一致
    /// * 🚩接受一个闭包，以便泛用于各类型的「袋」
    pub fn bag_consistent<T: Item>(
        old: &Bag<T>,
        new: &Bag<T>,
        consistent_t: impl Fn(&T, &T) -> AResult,
    ) -> AResult {
        // 排序好的概念列表
        fn sorted_items<T: Item>(m: &Bag<T>) -> Vec<&T> {
            manipulate! {
                m.iter().collect::<Vec<_>>()
                => .sort_by_key(|&t| t.key())
            }
        }
        let [items_old, items_new] = f_parallel![sorted_items; old; new];
        // 内容量
        assert_eq_try!(items_old.len(), items_new.len(), "袋不一致——内容数量不相等");
        // 袋内每一对内容一致
        for (item_old, item_new) in zip(items_old, items_new) {
            consistent_t(item_old, item_new)?;
        }
        ok!()
    }

    /// 概念一致
    pub fn concept_consistent(concept_old: &Concept, concept_new: &Concept) -> AResult {
        // 词项一致
        let term = Concept::term;
        let [term_old, term_new] = f_parallel![term; concept_old; concept_new];
        assert_eq_try!(term_old, term_new);
        let term = term_old;

        // 预算值一致
        assert_eq_try!(
            BudgetValue::from(concept_old),
            BudgetValue::from(concept_new),
            "概念'{term}'的预算值不一致"
        );

        // 任务链 | ⚠️任务链因内部引用问题，不能直接判等
        fn sorted_task_links(c: &Concept) -> Vec<&TaskLink> {
            manipulate! {
                c.iter_task_links().collect::<Vec<_>>()
                => .sort_by_key(|link| link.key())
            }
        }
        let [task_links_old, task_links_new] =
            f_parallel![sorted_task_links; concept_old; concept_new];
        assert_eq_try!(
            task_links_old.len(),
            task_links_new.len(),
            "概念'{term}'的任务链数量不一致"
        );
        for (old, new) in zip(task_links_old, task_links_new) {
            task_consistent(&old.target(), &new.target())?;
        }

        // 词项链 | ℹ️因为是「词项链袋」所以要调整顺序而非直接zip，但✅词项链可以直接判等
        fn sorted_term_links(c: &Concept) -> Vec<&TermLink> {
            manipulate! {
                c.iter_term_links().collect::<Vec<_>>()
                => .sort_by_key(|link| link.key())
            }
        }
        let [links_old, links_new] = f_parallel![sorted_term_links; concept_old; concept_new];
        assert_eq_try!(
            links_old,
            links_new,
            "概念'{term}'的词项链不一致\nold = {links_old:?}\nnew = {links_new:?}",
        );

        // 信念表 | ℹ️顺序也必须一致
        for (old, new) in zip(concept_old.iter_beliefs(), concept_new.iter_beliefs()) {
            assert_eq_try!(
                old,
                new,
                "概念'{term}'的信念列表不一致\nold = {}\nnew = {}",
                old.to_display_long(),
                new.to_display_long(),
            );
        }
        ok!()
    }

    /// 任务一致性
    /// * 🎯应对其中「父任务」引用的「无法判等」
    pub fn task_consistent(a: &Task, b: &Task) -> AResult {
        // 常规属性
        let [ka, kb] = [a.key(), b.key()];
        assert_eq_try!(ka, kb, "任务不一致——key不一致：{ka} != {kb}",);
        let [ca, cb] = [a.content(), b.content()];
        assert_eq_try!(ca, cb, "任务不一致——content不一致：{ca} != {cb}");
        assert_eq_try!(
            a.as_judgement().map(TruthValue::from),
            b.as_judgement().map(TruthValue::from),
            "任务不一致——真值不一致"
        );
        assert_eq_try!(
            BudgetValue::from(a),
            BudgetValue::from(b),
            "任务不一致——预算不一致"
        );
        assert_eq_try!(
            a.punctuation(),
            b.punctuation(),
            "任务不一致——punctuation不一致"
        );
        assert_eq_try!(
            a.parent_belief(),
            b.parent_belief(),
            "任务不一致——parent_belief不一致"
        );
        // 父任务 | ⚠️父任务因内部引用问题，不能直接判等
        match (a.parent_task(), b.parent_task()) {
            (Some(a), Some(b)) => {
                task_consistent(&a.get_(), &b.get_())?;
            }
            (None, None) => {}
            _ => panic!("任务不一致——父任务不一致"),
        };
        ok!()
    }

    /// 对记忆区「序列反序列化」的可靠性测试
    #[test]
    fn test_soundness() -> AResult {
        fn test(memory: &Memory) -> AResult {
            let ser = serde_json::to_string(memory)?;
            let de = serde_json::from_str::<Memory>(&ser)?;
            memory_consistent(memory, &de)?; // 应该相等

            // let ser2 = serde_json::to_string(&de)?;
            // assert_eq_try!(ser, ser2); // ! 可能会有无序对象

            ok!()
        }
        // 测试的总体规模：使用字符当作词项名
        const R_TERM: std::ops::RangeInclusive<char> = 'A'..='Z';
        for t_end in R_TERM {
            // 构造不同大小的记忆区
            let mut memory = Memory::new(DEFAULT_PARAMETERS);
            for t in 'A'..=t_end {
                memory.make_new_concept(&term!(str t.to_string()));
            }
            // 开始测试「序列反序列化」
            test(&memory)?;
        }
        ok!()
    }
}
