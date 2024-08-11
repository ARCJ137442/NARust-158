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
#[derive(Debug, PartialEq, Serialize, Deserialize)]
pub struct Memory {
    /// 概念袋
    ///
    /// # 📄OpenNARS
    ///
    /// Concept bag. Containing all Concepts of the system
    #[serde(deserialize_with = "Memory::deserialize_concepts")]
    concepts: Bag<Concept>,

    /// 🆕统一所有「超参数」的存储
    ///
    /// TODO: 【2024-08-11 23:46:10】后续尽可能跟「推理器」的超参数字段合并
    parameters: Parameters,
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
                parameters.concept_forgetting_cycle,
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
            self.task_link_forgetting_rate(),
            self.term_link_forgetting_rate(),
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
        let all_task_rcs = bag.iter_mut().flat_map(Concept::iter_tasks_mut);
        RCTask::unify_rcs(all_task_rcs);
        // 返回归一化后的概念袋
        Ok(bag)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{ok, test_term as term, util::AResult};

    #[test]
    fn test_soundness() -> AResult {
        fn test(memory: &Memory) -> AResult {
            let ser = serde_json::to_string(memory)?;
            let de = serde_json::from_str::<Memory>(&ser)?;
            assert_eq!(*memory, de); // 应该相等

            // let ser2 = serde_json::to_string(&de)?;
            // assert_eq!(ser, ser2); // ! 可能会有无序对象

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
