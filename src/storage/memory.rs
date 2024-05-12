//! 🎯复刻OpenNARS `nars.entity.Memory`
//! * 📌「记忆区」
//! * 🚧【2024-05-07 18:52:42】目前复现方法：先函数API（提供函数签名），再翻译填充函数体代码
//!
//! * ✅【2024-05-08 15:46:28】目前已初步实现方法API，并完成部分方法模拟
//! * ✅【2024-05-08 17:17:41】目前已初步完成所有方法的模拟

use crate::{
    entity::*, inference::*, language::Term, nars::DEFAULT_PARAMETERS, storage::*,
    ToDisplayAndBrief,
};
use narsese::api::NarseseValue;
use std::collections::VecDeque;

/// 有关「记忆区报告」或「记忆区记录」
/// * 🎯记忆区输出信息
/// * 🚩【2024-05-06 09:35:37】复用[`navm`]中的「NAVM输出」
mod report {
    use navm::output::Output;
    use std::collections::VecDeque;

    /// 🆕记忆区记忆者
    /// * 📄等价于OpenNARS`nars.inference.IInferenceRecorder`
    pub trait MemoryRecorder {
        /// 缓存的输出缓冲区
        /// * 🚩【2024-05-07 20:09:49】目前使用[`VecDeque`]队列实现
        fn cached_outputs(&self) -> &VecDeque<Output>;
        /// [`MemoryRecorder::cached_outputs`]的可变版本
        fn __cached_outputs_mut(&mut self) -> &mut VecDeque<Output>;

        /// 长度大小
        #[inline]
        fn len_output(&self) -> usize {
            self.cached_outputs().len()
        }

        /// 是否为空
        #[inline]
        fn no_output(&self) -> bool {
            self.cached_outputs().is_empty()
        }

        /// 置入NAVM输出（在末尾）
        #[inline]
        fn put(&mut self, output: Output) {
            self.__cached_outputs_mut().push_back(output)
        }

        /// 取出NAVM输出（在开头）
        /// * ⚠️可能没有（空缓冲区）
        #[inline]
        fn take(&mut self) -> Option<Output> {
            self.__cached_outputs_mut().pop_front()
        }

        /// 清空
        /// * 🎯用于推理器「向外输出并清空内部结果」备用
        ///   * 🚩【2024-05-13 02:13:21】现在直接用`while let Some(output) = self.take()`型语法
        #[inline]
        fn clear(&mut self) {
            self.__cached_outputs_mut().clear()
        }
    }

    /// 🆕[`MemoryRecorder`]的具体特征
    /// * ✅统一的构造函数
    pub trait MemoryRecorderConcrete: MemoryRecorder + Sized {
        /// 🆕构造函数
        /// * 🚩构造一个空的「记忆区记录者」
        fn new() -> Self;
    }

    /// 「记忆区记录器」初代实现
    /// * 🚩使用「NAVM输出」表示
    #[derive(Debug, Clone, Default)]
    pub struct MemoryRecorderV1 {
        /// 输出缓冲区
        cached_outputs: VecDeque<Output>,
    }

    /// 实现「记忆区记录器」（字段对应）
    impl MemoryRecorder for MemoryRecorderV1 {
        fn cached_outputs(&self) -> &VecDeque<Output> {
            &self.cached_outputs
        }

        fn __cached_outputs_mut(&mut self) -> &mut VecDeque<Output> {
            &mut self.cached_outputs
        }
    }

    impl MemoryRecorderConcrete for MemoryRecorderV1 {
        // 构造函数
        // * 🚩默认构造空数组
        #[inline]
        fn new() -> Self {
            Self::default()
        }
    }
}
use super::{ConceptBag, NovelTaskBag};
use navm::output::Output;
pub use report::*;

/// 模拟`nars.entity.Memory`
/// * 🚩直接通过「要求[『推理上下文』](ReasonContext)」获得完整的「类型约束」
///   * ✅一并解决「上下文各种完全限定语法」的语法噪音问题
/// * 🚩【2024-05-08 16:34:15】因为"<as [`RuleTables`]>"的需要，增加约束[`Sized`]
///
/// # 📄OpenNARS
///
/// The memory of the system.
pub trait Memory: ReasonContext<Memory = Self> + Sized {
    // /// 绑定的「概念」类型
    // type Concept: ConceptConcrete;

    /// 绑定的「概念袋」类型
    /// * 🎯对应`Self::concepts`
    /// * 🚩【2024-05-07 20:04:25】必须与绑定的「概念」类型一致
    type ConceptBag: ConceptBag<Concept = Self::Concept>;

    /// 绑定的「任务袋」类型
    /// * 🚩【2024-05-07 20:04:25】必须与「概念」中的「任务」一致
    /// * 🎯对应`Self::novel_tasks`
    type NovelTaskBag: NovelTaskBag<Task = Self::Task>;

    /// 绑定的「记录者」类型
    type Recorder: MemoryRecorderConcrete;

    // 字段 //

    // ! ❌【2024-05-07 19:59:14】暂不迁移`reasoner`引用：拆解其用途如「时钟」「音量」等属性
    // * 📝OpenNARS中`Memory`用到`reasoner`的地方：`initTimer`、`getTime`(Reasoner.time)、`silenceValue`、`updateTimer`

    /* ---------- Long-term storage for multiple cycles ---------- */

    /// 模拟`Memory.concepts`
    /// * 🚩私有+读写
    ///
    /// # 📄OpenNARS
    ///
    /// Concept bag. Containing all Concepts of the system
    fn __concepts(&self) -> &Self::ConceptBag;
    /// [`Memory::concepts`]的可变版本
    fn __concepts_mut(&mut self) -> &mut Self::ConceptBag;

    /// 模拟`Memory.novelTasks`
    /// * 🚩私有+读写
    ///
    /// # 📄OpenNARS
    ///
    /// New tasks with novel composed terms, for delayed and selective processing
    fn __novel_tasks(&self) -> &Self::NovelTaskBag;
    /// [`Memory::novel_tasks`]的可变版本
    fn __novel_tasks_mut(&mut self) -> &mut Self::NovelTaskBag;

    /// 模拟`Memory.recorder`、`getRecorder`、`setRecorder`
    /// * 🚩🆕【2024-05-07 20:08:35】目前使用新定义的[`MemoryRecorder`]类型
    /// * 📝OpenNARS中`Memory`用到`recorder`的地方：`init`、`inputTask`、`activatedTask`
    ///
    /// # 📄OpenNARS
    ///
    /// Inference record text to be written into a log file
    fn recorder(&self) -> &Self::Recorder;
    /// [`Memory::recorder`]的可变版本
    fn recorder_mut(&mut self) -> &mut Self::Recorder;

    /// 模拟`Memory.beliefForgettingRate`、`Memory.getBeliefForgettingRate`
    /// * 🚩模拟方法：作为变量属性，在每个[「概念」](Concept)构造时作为参数传入
    ///
    /// # 📄OpenNARS
    ///
    /// 🈚
    fn belief_forgetting_rate(&self) -> usize;
    /// [`Memory::belief_forgetting_rate`]的可变版本
    fn belief_forgetting_rate_mut(&mut self) -> &mut usize;

    /// 模拟`Memory.taskForgettingRate`、`Memory.getTaskForgettingRate`
    ///
    /// # 📄OpenNARS
    ///
    /// 🈚
    fn task_forgetting_rate(&self) -> usize;
    /// [`Memory::task_forgetting_rate`]的可变版本
    fn task_forgetting_rate_mut(&mut self) -> &mut usize;

    /// 模拟`Memory.conceptForgettingRate`、`Memory.getConceptForgettingRate`
    /// ! ❌【2024-05-07 20:21:11】不直接复刻`conceptForgettingRate`：存储在[`super::BagV1`]中
    /// * 🚩用的是[`super::Bag::_forget_rate`]
    ///
    /// # 📄OpenNARS
    ///
    /// 🈚
    fn concept_forgetting_rate(&self) -> usize {
        self.__concepts()._forget_rate()
    }

    /// 模拟`Memory.newTasks`
    /// * 🚩读写：OpenNARS中要读写对象
    /// * 📝虽然OpenNARS中被认作是「短期工作空间」，但实际上是个长期的工作空间
    ///   * 📝并且，只在「记忆区」内部被使用，用作「立即推理」
    ///   * 🚩【2024-05-12 14:38:58】决议：两头都有
    ///     * 在「记忆区回收上下文」时从「上下文的『新任务』接收」
    ///
    /// # 📄OpenNARS
    ///
    /// List of new tasks accumulated in one cycle, to be processed in the next cycle
    fn __new_tasks(&self) -> &[Self::Task];
    /// [`Memory::__new_tasks`]的可变版本
    /// * 🚩【2024-05-07 21:13:39】暂时用[`VecDeque`]代替
    fn __new_tasks_mut(&mut self) -> &mut VecDeque<Self::Task>;

    /* ---------- Constructor ---------- */

    /// 模拟`Memory.init`
    ///
    /// # 📄OpenNARS
    ///
    /// 🈚
    fn init(&mut self) {
        /* 📄OpenNARS源码：
        concepts.init();
        novelTasks.init();
        newTasks.clear();
        exportStrings.clear();
        reasoner.initTimer();
        randomNumber = new Random(1);
        recorder.append("\n-----RESET-----\n"); */
        self.__concepts_mut().init();
        self.__novel_tasks_mut().init();
        self.__new_tasks_mut().clear();
        // exportStrings.clear();
        // reasoner.initTimer();
        // randomNumber = new Random(1);
        self.recorder_mut().put(Output::INFO {
            message: "-----RESET-----".into(),
        })
    }

    /* ---------- conversion utilities ---------- */

    /// 模拟`Memory.nameToConcept`
    /// * 🚩【2024-05-07 21:31:21】此处抽象为更通用的[`BagKey`]特征类型
    ///
    /// # 📄OpenNARS
    ///
    /// Get an existing Concept for a given name
    ///
    /// called from Term and ConceptWindow.
    ///
    /// @param name the name of a concept
    /// @return a Concept or null
    #[inline]
    #[doc(alias = "name_to_concept")]
    fn key_to_concept(&self, key: &Self::Key) -> Option<&Self::Concept> {
        /* 📄OpenNARS源码：
        return concepts.get(name); */
        self.__concepts().get(key)
    } // ? 是否要加可变版本

    /// 模拟`Memory.nameToListedTerm`
    ///
    /// # 📄OpenNARS
    ///
    /// Get a Term for a given name of a Concept or Operator
    ///
    /// called in StringParser and the make methods of compound terms.
    ///
    /// @param name the name of a concept or operator
    /// @return a Term or null (if no Concept/Operator has this name)
    #[inline]
    #[doc(alias = "name_to_listed_term")]
    fn key_to_listed_term(&self, key: &Self::Key) -> Option<&Term> {
        /* 📄OpenNARS源码：
        Concept concept = concepts.get(name);
        if (concept != null) {
            return concept.getTerm();
        }
        return null; */
        self.key_to_concept(key).map(Concept::term)
    }

    /// 模拟`Memory.termToConcept`
    ///
    /// # 📄OpenNARS
    ///
    /// Get an existing Concept for a given Term.
    ///
    /// @param term The Term naming a concept
    /// @return a Concept or null
    fn term_to_concept(&self, term: &Term) -> Option<&Self::Concept> {
        /* 📄OpenNARS源码：
        return nameToConcept(term.getName()); */
        self.key_to_concept(&<Self::ConceptBag as ConceptBag>::key_from_term(term))
    }

    /// 模拟`Memory.getConcept`
    /// * 🚩尝试获取现有的概念；若无，则创建新概念
    /// * ⚠️仍然不总是能获取到概念：对于并非「常量」的词项，不予创建新概念
    ///
    /// # 📄OpenNARS
    ///
    /// Get the Concept associated to a Term, or create it.
    ///
    /// @param term indicating the concept
    /// @return an existing Concept, or a new one, or null ( TODO bad smell )
    #[doc(alias = "get_concept")]
    fn get_concept_or_create<'s>(&'s mut self, term: &Term) -> Option<&'s Self::Concept> {
        /* 📄OpenNARS源码：
        if (!term.isConstant()) {
            return null;
        }
        String n = term.getName();
        Concept concept = concepts.get(n);
        if (concept == null) {
            concept = new Concept(term, this); // the only place to make a new Concept
            boolean created = concepts.putIn(concept);
            if (!created) {
                return null;
            }
        }
        return concept; */
        if !term.is_constant() {
            return None;
        }
        let key = <Self::ConceptBag as ConceptBag>::key_from_term(term);
        let has_concept = self.__concepts().has(&key);
        // 暂无概念⇒当即创建
        if !has_concept {
            // * 🚩此处不能省掉`<Self::Concept as ConceptConcrete>`：直接使用类参，会有歧义
            let new_concept = <Self::Concept as ConceptConcrete>::new(term.clone());
            // ! 💫【2024-05-07 21:55:26】借用问题：「获取概念」与「插入新概念」借用冲突
            // * ✅【2024-05-07 23:19:37】已解决：通过「最开始只获取『是否有』，分支之后再获取『概念』」的方式，解决了「一个引用蔓延到两个分支」的生命周期问题
            let created = self.__concepts_mut().put_in(new_concept);
            if created.is_some() {
                return None;
            }
        }
        // 其它⇒直接查询并返回（不管有无，创建了也会被查询到）
        self.__concepts().get(&key)
    }

    /// 模拟`Memory.getConceptActivation`
    /// * 📝被[`BudgetFunctions::__budget_inference`]调用，
    ///   * ⚠️从而被包括「结构规则」在内的所有规则调用
    ///
    /// # 📄OpenNARS
    ///
    /// Get the current activation level of a concept.
    ///
    /// @param t The Term naming a concept
    /// @return the priority value of the concept
    fn get_concept_activation(&self, term: &Term) -> Self::ShortFloat {
        /* 📄OpenNARS源码：
        Concept c = termToConcept(t);
        return (c == null) ? 0f : c.getPriority(); */
        match self.term_to_concept(term) {
            Some(c) => c.priority(),
            None => Self::ShortFloat::ZERO,
        }
    }

    /* ---------- adjustment functions ---------- */

    /// 模拟`Memory.activateConcept`
    /// * 🚩【2024-05-07 22:35:27】此处解耦：使用「元素id」而非「元素」进行操作
    ///   * 🎯避免「在『概念』中调用自身，自身又移动了『概念』的位置」
    ///
    /// # 📄OpenNARS
    ///
    /// Adjust the activation level of a Concept
    ///
    /// called in Concept.insertTaskLink only
    ///
    /// @param c the concept to be adjusted
    /// @param b the new BudgetValue
    fn activate_concept(&mut self, key: &Self::Key, budget: &Self::Budget) {
        /* 📄OpenNARS源码：
        concepts.pickOut(c.getKey());
        BudgetFunctions.activate(c, b);
        concepts.putBack(c); */
        let concept = self.__concepts_mut().pick_out(key);
        // * 🆕仅在「挑出了概念」时「激活」
        if let Some(mut concept) = concept {
            concept.budget_mut().activate(budget);
            self.__concepts_mut().put_back(concept);
        }
    }

    /* ---------- new task entries ---------- */
    /*
     * There are several types of new tasks, all added into the
     * newTasks list, to be processed in the next workCycle.
     * Some of them are reported and/or logged.
     */

    /// 模拟`Memory.inputTask`
    /// * 🚩【2024-05-07 22:51:11】在此对[`BudgetValue::above_threshold`]引入[「预算阈值」超参数](crate::nars::Parameters::budget_threshold)
    ///
    /// # 📄OpenNARS
    ///
    /// Input task processing. Invoked by the outside or inside environment.
    /// Outside: StringParser (input); Inside: Operator (feedback). Input tasks
    /// with low priority are ignored, and the others are put into task buffer.
    ///
    /// @param task The input task
    fn input_task(&mut self, task: Self::Task) {
        /* 📄OpenNARS源码：
        if (task.getBudget().aboveThreshold()) {
            recorder.append("!!! Perceived: " + task + "\n");
            report(task.getSentence(), ReportType.IN); // report input
            newTasks.add(task); // wait to be processed in the next workCycle
        } else {
            recorder.append("!!! Neglected: " + task + "\n");
        } */
        let budget_threshold = DEFAULT_PARAMETERS.budget_threshold;
        // * ✅【2024-05-07 23:22:54】现在通过重命名「真值」「预算值」的相应方法，不再有命名冲突（`from_float`→`from_floats`）
        let budget_threshold = Self::ShortFloat::from_float(budget_threshold);
        if task.budget().above_threshold(budget_threshold) {
            // ? 💭【2024-05-07 22:57:48】实际上只需要输出`IN`即可：日志系统不必照着OpenNARS的来
            // * 🚩此处两个输出合而为一
            let narsese = NarseseValue::from_task(task.to_lexical());
            self.recorder_mut().put(Output::IN {
                content: format!("!!! Perceived: {}", task.to_display_long()),
                narsese: Some(narsese),
            });
            // * 📝只追加到「新任务」里边，并不进行推理
            self.__new_tasks_mut().push_back(task);
        } else {
            // 此时还是输出一个「被忽略」好
            self.recorder_mut().put(Output::COMMENT {
                content: format!("!!! Neglected: {}", task.to_display_long()),
            });
        }
    }

    /* 📝诸多方法现均被置入「推理上下文」而非「记忆区」中
     * activated_task
     * derived_task
     * double_premise_task_revisable
     * double_premise_task
     * single_premise_task_current
     * single_premise_task
     * work_cycle
     * __process_new_task
     * __process_novel_task
     * __process_concept
     * __fire_concept
     * __immediate_process
     */

    /* ---------- display ---------- */
    // ! ❌【2024-05-08 15:42:42】目前不复刻「显示」类方法
    // * conceptsStartPlay
    // * taskBuffersStartPlay
    // * report
    // * toString
    // * toStringLongIfNotNull
    // * toStringLongIfNotNull
    // * toStringIfNotNull

    // * ✅`getTaskForgettingRate`已在开头实现
    // * ✅`getBeliefForgettingRate`已在开头实现
    // * ✅`getConceptForgettingRate`已在开头实现

    // ! ❌【2024-05-08 15:44:26】暂不模拟`Memory.NullInferenceRecorder`
}

/// [`Memory`]的具体版本
/// * 🎯规定「构造函数」「比对判等」等逻辑
pub trait MemoryConcrete: Memory + Sized {
    /// 🆕包含所有参数的内部构造函数
    fn __new(
        recorder: Self::Recorder,
        concepts: Self::ConceptBag,
        novel_tasks: Self::NovelTaskBag,
        new_tasks: VecDeque<Self::Task>,
        belief_forgetting_rate: usize,
        task_forgetting_rate: usize,
        // concept_forgetting_rate: usize, // * 🚩【2024-05-07 20:35:46】目前直接存到「概念袋」中
    ) -> Self;

    /// 模拟`new Memory(ReasonerBatch reasoner)`
    /// * 🚩【2024-05-07 20:32:33】目前拆解所有来自`ReasonerBatch`的参数
    ///
    /// # 📄OpenNARS
    ///
    /// Create a new memory
    ///
    /// Called in Reasoner.reset only
    ///
    /// @param reasoner
    fn new(
        belief_forgetting_rate: usize,
        task_forgetting_rate: usize,
        concept_forgetting_rate: usize,
    ) -> Self {
        /* 📄OpenNARS源码：
        this.reasoner = reasoner;
        recorder = new NullInferenceRecorder();
        concepts = new ConceptBag(this);
        novelTasks = new NovelTaskBag(this);
        newTasks = new LinkedList<>();
        exportStrings = new ArrayList<>(); */
        Self::__new(
            <Self::Recorder as MemoryRecorderConcrete>::new(),
            <Self::ConceptBag as BagConcrete<Self::Concept>>::new(
                // * 🚩复刻`nars.storage.ConceptBag.capacity`
                DEFAULT_PARAMETERS.concept_bag_size,
                // * 🚩复刻`nars.storage.ConceptBag.forgetRate`
                concept_forgetting_rate,
            ),
            <Self::NovelTaskBag as BagConcrete<Self::Task>>::new(
                // * 🚩复刻`nars.storage.NovelTaskBag.capacity`
                DEFAULT_PARAMETERS.task_buffer_size,
                // * 🚩复刻`nars.storage.NovelTaskBag.forgetRate`
                DEFAULT_PARAMETERS.new_task_forgetting_cycle,
            ),
            VecDeque::new(), // TODO: 🏗️【2024-05-07 21:09:58】日后是否可独立成一个`add`、`size`、`get`的特征？
            belief_forgetting_rate,
            task_forgetting_rate,
        )
    }
}

/// TODO: 初代实现
mod impl_v1 {
    use super::*;
}
pub use impl_v1::*;

/// TODO: 单元测试
#[cfg(test)]
mod tests {
    use super::*;
}
