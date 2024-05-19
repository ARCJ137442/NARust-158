//! 🎯复刻OpenNARS `nars.entity.Concept`
//!
//! * ✅【2024-05-08 15:46:28】目前已初步实现方法API

use super::{
    BudgetValue, Item, SentenceConcrete, StampConcrete, TaskConcrete, TaskLinkConcrete,
    TermLinkConcrete, TruthValueConcrete,
};
use crate::{
    entity::*, global::Float, language::Term, nars::DEFAULT_PARAMETERS, storage::*,
    ToDisplayAndBrief,
};

/// 「概念」的所有可变字段
/// * 🎯证明「这些字段都不会相互冲突」
/// * ❌【2024-05-19 10:21:40】不能直接反向引用「概念」，会导致「概念」需要[`Sized`]
pub struct ConceptFieldsMut<'s, TaskLinkBag, TermLinkBag, Task, Sentence> {
    pub task_links: &'s mut TaskLinkBag,
    pub term_links: &'s mut TermLinkBag,
    // pub term_link_templates: &'s mut [TermLink],
    pub questions: &'s mut Vec<Task>,
    pub beliefs: &'s mut Vec<Sentence>,
    // pub beliefs: &'s mut [Sentence],
}

/// 模拟`nars.entity.Concept`
/// * 🚩【2024-05-04 17:28:30】「概念」首先能被作为「Item」使用
pub trait Concept: Item {
    /// 绑定的「时间戳」类型
    /// * 📌必须是「具体」类型
    type Stamp: StampConcrete;

    /// 绑定的「真值」类型
    /// * 📌必须是「具体」类型
    /// * 🚩【2024-05-07 18:53:40】目前认为，必须限定其「短浮点」类型与[「预算值」](Item::Budget)一致
    type Truth: TruthValueConcrete<E = <Self::Budget as BudgetValue>::E>;

    // * ✅至于「元素id」与「预算值」，已在Item约束中绑定

    // * 🚩【2024-05-06 11:23:27】从「语句」到「任务」再到「任务链」，逐个实现关联类型

    /// 绑定的「语句」
    /// * 🎯每个实现中只会实现一种类型，用于统一多个函数的参数
    /// * ⚠️【2024-05-06 21:19:01】必须是「具体特征」，不然无法使用「复制」「判等」等方法
    ///   * 💭实际上「复制」是否就意味着「信息就那些」？或许可以考虑移回「抽象特征」？
    ///   TODO: 【2024-05-06 21:20:15】留给以后考量
    type Sentence: SentenceConcrete<Truth = Self::Truth, Stamp = Self::Stamp>;

    /// 绑定的「任务」
    /// * 🎯每个实现中只会实现一种类型，用于统一多个函数的参数
    /// * ⚠️【2024-05-06 21:19:01】必须是「具体特征」，不然无法使用「复制」「判等」等方法
    ///   * 💭实际上「复制」是否就意味着「信息就那些」？或许可以考虑移回「抽象特征」？
    ///   TODO: 【2024-05-06 21:20:15】留给以后考量
    type Task: TaskConcrete<Sentence = Self::Sentence, Key = Self::Key, Budget = Self::Budget>;

    /// 绑定的「词项链」
    /// * 🎯每个实现中只会实现一种类型，用于统一多个函数的参数
    type TermLink: TermLinkConcrete<Key = Self::Key, Budget = Self::Budget>;

    /// 绑定的「任务链」
    /// * 🎯每个实现中只会实现一种类型，用于统一多个函数的参数
    type TaskLink: TaskLinkConcrete<Task = Self::Task, Key = Self::Key, Budget = Self::Budget>;

    /// 绑定的「词项链袋」
    /// * 🎯每个实现中只会实现一种类型，用于统一多个函数的参数
    type TermLinkBag: TermLinkBag<Link = Self::TermLink>;

    /// 绑定的「任务链袋」
    /// * 🎯每个实现中只会实现一种类型，用于统一多个函数的参数
    type TaskLinkBag: TaskLinkBag<Link = Self::TaskLink>;

    /// 🆕获取所有可变引用
    /// * 🎯关键在于告诉编译器「能获取到这些值，证明在外部同时修改是没问题的」
    ///   * 📌亦即「可并行修改」
    /// * 🚩用法：获取⇒解构⇒分别使用
    fn fields_mut(
        &mut self,
    ) -> ConceptFieldsMut<'_, Self::TaskLinkBag, Self::TermLinkBag, Self::Task, Self::Sentence>;

    /// 模拟`Concept.term`、`Concept.getTerm`
    /// * 🚩只读：OpenNARS仅在构造函数中赋值
    ///
    /// # 📄OpenNARS
    ///
    /// ## `term`
    ///
    /// The term is the unique ID of the concept
    ///
    /// ## `getTerm`
    ///
    /// Return the associated term, called from Memory only
    ///
    /// @return The associated term
    fn term(&self) -> &Term;

    /// 模拟`Concept.taskLinks`
    /// * 🚩私有：未对外暴露直接的公开接口
    ///
    /// # 📄OpenNARS
    ///
    /// Task links for indirect processing
    fn __task_links(&self) -> &Self::TaskLinkBag;
    /// [`Concept::__task_links`]的可变版本
    /// * 🚩【2024-05-19 10:23:01】现在通过「所有可变引用」可将「获取所有可变引用的一部分」作为默认实现
    #[inline(always)]
    fn __task_links_mut(&mut self) -> &mut Self::TaskLinkBag {
        self.fields_mut().task_links
    }

    /// 模拟`Concept.termLinks`
    /// * 🚩私有：未对外暴露直接的公开接口
    ///
    /// # 📄OpenNARS
    ///
    /// Term links between the term and its components and compounds
    fn __term_links(&self) -> &Self::TermLinkBag;
    /// [`Concept::__term_links`]的可变版本
    /// * 🚩【2024-05-19 10:23:01】现在通过「所有可变引用」可将「获取所有可变引用的一部分」作为默认实现
    #[inline(always)]
    fn __term_links_mut(&mut self) -> &mut Self::TermLinkBag {
        self.fields_mut().term_links
    }

    /// 模拟`Concept.termLinkTemplates`、`Concept.getTermLinkTemplates`
    /// * 🚩只读：仅在构造函数中被赋值
    ///
    /// # 📄OpenNARS
    ///
    /// ## `termLinkTemplates`
    ///
    /// Link templates of TermLink, only in concepts with CompoundTerm
    ///
    /// ## `getTermLinkTemplates`
    ///
    /// Return the templates for TermLinks, only called in
    /// Memory.continuedProcess
    ///
    /// @return The template get
    fn term_link_templates(&self) -> &[Self::TermLink];

    /// 模拟`Concept.questions`
    /// * 🚩内部读写：仅在内部被使用
    ///
    /// TODO: 考虑作为「共享引用」
    ///
    /// # 📄OpenNARS
    ///
    /// Question directly asked about the term
    fn __questions(&self) -> &[Self::Task];
    /// [`Concept::questions`]的可变版本
    /// * 🚩【2024-05-06 11:49:15】目前使用[`Vec`]：追加、插入、移除
    /// * 🚩【2024-05-19 10:23:01】现在通过「所有可变引用」可将「获取所有可变引用的一部分」作为默认实现
    #[inline(always)]
    fn __questions_mut(&mut self) -> &mut Vec<Self::Task> {
        self.fields_mut().questions
    }

    /// 模拟`Concept.questions`
    /// * 🚩内部读写：仅在内部被使用
    ///
    /// TODO: 考虑作为「共享引用」
    ///
    /// # 📄OpenNARS
    ///
    /// Sentences directly made about the term, with non-future tense
    fn __beliefs(&self) -> &[Self::Sentence];
    /// [`Concept::beliefs`]的可变版本
    /// * 🚩【2024-05-06 11:49:15】目前使用[`Vec`]：追加、插入、移除
    /// * 🚩【2024-05-19 10:23:01】现在通过「所有可变引用」可将「获取所有可变引用的一部分」作为默认实现
    #[inline(always)]
    fn __beliefs_mut(&mut self) -> &mut Vec<Self::Sentence> {
        self.fields_mut().beliefs
    }

    // /// 🆕获取「信念」与「任务」
    // /// * 🎯用于在「处理判断/问题」时表示「信念、问题互不影响」
    // fn __beliefs_and_questions(&self) -> (&[Self::Sentence], &[Self::Task]);
    // * 💡【2024-05-18 20:25:25】似乎可以利用特殊的「引用结构」来强制要求「互不干扰的字段」
    //   * 🚩配套方法：当要获取多个确定是「互不干扰」的字段时，通过「获取引用并立即解构」的方式获取
    //   * ✅其它「获取单字段」的方法，可以使用这种「字段要求」作为「默认参数」行使

    // ! ❌【2024-05-06 11:37:01】不实现`Concept.memory`（仅用于内部「袋」的容量获取）
    // ! ❌【2024-05-06 11:37:01】不实现`Concept.entityObserver`

    /* 🚩【2024-05-12 15:11:24】大量迁移与「推理控制」有关的函数到[`crate::inference::ConceptProcess`]
     * direct_process
     * __process_judgment
     * __process_question
     * __link_to_task
     * __add_to_table
     * __evaluation
     * insert_task_link
     * build_term_links
     * insert_term_link
     * get_belief
     */

    /// 🆕将新的「问题」放进自身的「问题集」中
    /// * 🎯最初从[`Concept.processQuestion`](crate::nars::ReasonerDirectProcess::__process_question)中调用
    /// * 🚩有限大小缓冲区：若加入后大小溢出，则「先进先出」（在Rust语境下任务被销毁）
    ///
    /// TODO: 后续要实现一个「固定大小缓冲区队列」？
    fn __add_new_question(&mut self, question_task: Self::Task) {
        // * 🚩新问题⇒加入「概念」已有的「问题列表」中（有限大小缓冲区）
        self.__questions_mut().push(question_task);
        if self.__questions().len() > DEFAULT_PARAMETERS.maximum_questions_length {
            self.__questions_mut().remove(0);
        }
    }

    /* ---------- access local information ---------- */

    /// 模拟`Concept.toString`
    /// * ❌无法直接「默认实现[`Display`]」：孤儿规则
    /// * ✅通过[别的特征](ToDisplayAndBrief)去实现
    ///
    /// # 📄OpenNARS
    ///
    /// Return a String representation of the Item
    ///
    /// @return The String representation of the full content
    fn __to_display(&self) -> String {
        self.budget().__to_display() + " " + &self.key().to_display()
    }

    /// 模拟`Concept.toStringBrief`
    ///
    /// # 📄OpenNARS
    ///
    /// Return a String representation of the Item after simplification
    ///
    /// @return A simplified String representation of the content
    #[inline(always)]
    fn __to_display_brief(&self) -> String {
        self.budget().__to_display_brief() + " " + &self.key().to_display_brief()
    }

    /// 模拟`Concept.toStringLong`
    ///
    /// # 📄OpenNARS
    ///
    /// 🈚
    #[inline(always)]
    fn __to_display_long(&self) -> String {
        self.to_display()
    }

    // ! ❌【2024-05-06 18:45:48】暂不模拟`toString`与`toStringLong`、`toStringIfNotNull`
    // ? ℹ️似乎`toString`还要用到`NARSBatch.isStandAlone()`这种「全局属性」

    /// 模拟`Concept.total_quality`
    /// * ⚠️覆盖原先对[`BudgetValue::quality`]的实现
    ///   * ❓Rust似乎不太能迁移这类「覆盖」的情形——只能靠「重名歧义」提醒
    ///     * 🚩不过后续可以通过「完全限定语法」指定`<self as Concept>::quality`来调用，并且也能提醒在所用之处实现
    ///   * ✅在「概念袋」中的访问，仍然使用其作为[`Item`]的原始实现（[内部「预算值」](Self::Budget)的[质量](BudgetValue::quality)）
    ///     * ℹ️【2024-05-06 19:01:45】已通过OpenNARS调试得到验证：「概念」有两种预算值
    ///       * 第一种是其作为「Item」访问内部[「预算值」](Item::Budget)所得到的「质量」
    ///       * 第二种即为此处「概念」作为一个「整体」所得到的「质量」
    ///     * 📌【2024-05-06 19:01:37】目前认为此处实际上无需出现「方法覆盖」，因为这种覆盖本身就是无效的
    ///       * 第一种走的是`self.budget.quality()`而非`self.quality()`（在实际推理传参时）
    ///       * ✅【2024-05-06 19:22:27】在OpenNARS 3.0.4中，经过断点调试验证，此处亦同奏效
    /// * 📝OpenNARS只会在「预算函数」的[「激活」](crate::inference::BudgetFunctions::activate)处调用
    ///   * 📝同时这个「激活」函数，只会被[「记忆区」](crate::storage::Memory)的[「激活概念」](crate::storage::Memory::activate_concept)调用
    ///   * 📄OpenNARS 3.0.4中亦是「使用场合单一」
    /// * 🚩【2024-05-06 18:54:21】目前的决策：重命名为`total_quality`，以便和「其作为[`Item`]时的『质量』」相区分
    ///
    /// # 📄OpenNARS
    ///
    /// Recalculate the quality of the concept [to be refined to show
    /// extension/intension balance]
    ///
    /// @return The quality value
    fn total_quality(&self) -> <Self::Budget as BudgetValue>::E {
        /* 📄OpenNARS源码：
        float linkPriority = termLinks.averagePriority();
        float termComplexityFactor = 1.0f / term.getComplexity();
        return UtilityFunctions.or(linkPriority, termComplexityFactor); */
        let from = <<Self::Budget as BudgetValue>::E as ShortFloat>::from_float;
        let link_priority = from(self.__term_links().average_priority());
        let term_complexity_factor = from(1.0 / self.term().complexity() as Float);
        link_priority | term_complexity_factor
    }

    /* ---------- main loop ---------- */

    // ! ❌【2024-05-08 15:06:45】不在此处模拟`Concept.fire`：本该是记忆区干的事
    // * 📄参考[`Memory::__fire_concept`]

    // ! ❌【2024-05-06 21:23:00】暂不实现与「呈现」「观察」有关的方法
    // * 📄有关`toString`在上头`access local information`中
}

/// 「概念」的具体类型
pub trait ConceptConcrete: Concept + Sized {
    /* ---------- constructor and initialization ---------- */

    /// 模拟`new Concept(Term tm, Memory memory)`
    /// * 🚩具体的「创建空数组」「创建空袋」交由「初代实现」实现
    ///
    /// # 📄OpenNARS
    ///
    /// Constructor, called in Memory.getConcept only
    ///
    /// @param tm     A term corresponding to the concept
    /// @param memory A reference to the memory
    fn new(term: Term) -> Self;
}

/// 初代实现
mod impl_v1 {
    use super::*;
    use crate::{
        __impl_to_display_and_display,
        entity::{StampV1, TaskV1, TermLinkV1, TruthV1},
    };

    /// TODO: 初代实现
    pub struct ConceptV1 {
        // TODO: 添加字段
    }

    /*
    impl Item for ConceptV1 {
        type Key = BagKeyV1;
        type Budget = BudgetV1;

        fn key(&self) -> &Self::Key {
            todo!()
        }

        fn budget(&self) -> &Self::Budget {
            todo!()
        }

        fn budget_mut(&mut self) -> &mut Self::Budget {
            todo!()
        }
    }

    impl Concept for ConceptV1 {
        type Stamp = StampV1;

        type Truth = TruthV1;

        type Sentence = SentenceV1<Self::Truth, Self::Stamp>;

        type Task = TaskV1<Self::Sentence, Self::Key, Self::Budget>;

        type TermLink = TermLinkV1<Self::Budget>;

        type TaskLink = TaskLinkV1<Self::Task>;

        type TaskLinkBag = TaskLinkBagV1;

        // ! ❌【2024-05-09 01:43:32】the trait bound `entity::term_link::impl_v1::TermLinkV1<entity::task::impl_v1::TaskV1<entity::sentence::impl_v1::SentenceV1<entity::truth_value::impl_v1::TruthV1, entity::stamp::impl_v1::StampV1>, std::string::String, entity::budget_value::impl_v1::BudgetV1>>: entity::item::Item` is not satisfied
        // ! the trait `entity::item::Item` is implemented for `entity::term_link::impl_v1::TermLinkV1<B>`
        type TermLinkBag = TermLinkBagV1;

        fn term(&self) -> &Term {
            todo!()
        }

        fn __task_links(&self) -> &Self::TaskLinkBag {
            todo!()
        }

        fn __task_links_mut(&mut self) -> &mut Self::TaskLinkBag {
            todo!()
        }

        fn __term_links(&self) -> &Self::TermLinkBag {
            todo!()
        }

        fn __term_links_mut(&mut self) -> &mut Self::TermLinkBag {
            todo!()
        }

        fn term_link_templates(&self) -> &[Self::TermLink] {
            todo!()
        }

        fn __questions(&self) -> &[Self::Task] {
            todo!()
        }

        fn __questions_mut(&mut self) -> &mut Vec<Self::Task> {
            todo!()
        }

        fn __beliefs(&self) -> &[Self::Sentence] {
            todo!()
        }

        fn __beliefs_mut(&mut self) -> &mut Vec<Self::Sentence> {
            todo!()
        }
    }

    __impl_to_display_and_display! {
        ConceptV1 as Concept
    }

    // TODO: 有待迁移到`ConceptConcrete`实现
    impl ConceptConcrete for ConceptV1 {
        fn new(term: Term) -> Self {
            /* 📄OpenNARS源码：
            super(tm.getName());
            term = tm;
            this.memory = memory;
            questions = new ArrayList<>();
            beliefs = new ArrayList<>();
            taskLinks = new TaskLinkBag(memory);
            termLinks = new TermLinkBag(memory);
            if (tm instanceof CompoundTerm) {
                termLinkTemplates = ((CompoundTerm) tm).prepareComponentLinks();
            } */
            // TODO: 复刻逻辑
            Self {}
        }
    } */
}
pub use impl_v1::*;

/// TODO: 单元测试
#[cfg(test)]
mod tests {
    use super::*;
}
