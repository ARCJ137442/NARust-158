//! 🆕「推理上下文」
//! * 🎯承载并迁移OpenNARS「记忆区」中的「临时推理状态」变量组
//! * 📄亦仿自OpenNARS 3.x（3.0.4）`DerivationContext`
//! * 📝【2024-05-12 02:17:38】基础数据结构可以借鉴OpenNARS 1.5.8，但涉及「推理」的部分，建议采用OpenNARS 3.0.4的架构来复刻
//!
//! * ♻️【2024-05-22 02:09:10】基本已按照改版重构，但仍需拆分代码到不同文件中
//! * ♻️【2024-06-26 11:47:13】现将按改版OpenNARS架构重写
//!   * 🚩【2024-06-26 11:47:30】仍然可能与旧版不同
#![doc(alias = "derivation_context")]

use crate::{
    control::{Parameters, Reasoner},
    entity::{
        BudgetValue, Concept, JudgementV1, RCTask, Sentence, SentenceV1, Stamp, Task, TaskLink,
        TermLink, TruthValue,
    },
    global::{ClockTime, Float, RC},
    inference::Budget,
    language::Term,
    storage::Memory,
    util::{RefCount, ToDisplayAndBrief},
};
use narsese::api::NarseseValue;
use navm::output::Output;

/// 🆕新的「推理上下文」对象
/// * 📄仿自OpenNARS 3.1.0
pub trait ReasonContext {
    /// 🆕获取记忆区（不可变引用）
    fn memory(&self) -> &Memory;

    /// 🆕访问「当前时间」
    /// * 🎯用于在推理过程中构建「新时间戳」
    /// * ️📝可空性：非空
    /// * 📝可变性：只读
    fn time(&self) -> ClockTime;

    /// 🆕访问「当前超参数」
    /// * 🎯用于在推理过程中构建「新时间戳」（作为「最大长度」参数）
    /// * ️📝可空性：非空
    /// * 📝可变性：只读
    fn parameters(&self) -> &Parameters;

    fn max_evidence_base_length(&self) -> usize {
        self.parameters().maximum_stamp_length
    }

    /// 获取「静默值」
    /// * 🎯在「推理上下文」中无需获取「推理器」`getReasoner`
    /// * ️📝可空性：非空
    /// * 📝可变性：只读
    fn silence_percent(&self) -> Float;

    /// 获取「新任务」的数量
    fn num_new_tasks(&self) -> usize;

    /// 添加「新任务」
    /// * 🎯添加推理导出的任务
    /// * 🚩【2024-06-26 20:51:20】目前固定为「实际值」
    ///   * 📌后续在「被推理器吸收」时，才变为「共享引用」
    fn add_new_task(&mut self, task: Task);

    /// 🆕添加「导出的NAVM输出」
    /// * ⚠️不同于OpenNARS，此处集成NAVM中的 [NARS输出](navm::out::Output) 类型
    /// * 📌同时复刻`addExportString`、`report`与`addStringToRecord`几个方法
    #[doc(alias = "add_export_string")]
    #[doc(alias = "add_string_to_record")]
    #[doc(alias = "report")]
    fn add_output(&mut self, output: Output);

    /// 获取「当前概念」（不可变）
    fn current_concept(&self) -> &Concept;

    /// 获取「当前概念」（可变）
    /// * 📄需要在「概念链接」中使用（添加任务链）
    fn current_concept_mut(&mut self) -> &mut Concept;

    /// 获取「当前词项」
    /// * 🚩获取「当前概念」对应的词项
    fn current_term(&self) -> &Term {
        self.current_concept().term()
    }

    /// 获取「已存在的概念」
    /// * 🎯让「概念推理」可以在「拿出概念」的时候运行，同时不影响具体推理过程
    /// * 🚩先与「当前概念」做匹配，若没有再在记忆区中寻找
    /// * 📌【2024-05-24 22:07:42】目前专供「推理规则」调用
    fn term_to_concept(&self, term: &Term) -> Option<&Concept> {
        match term == self.current_term() {
            true => Some(self.current_concept()),
            false => self.memory().term_to_concept(term),
        }
    }

    /// 获取「当前任务」（不变）
    /// * 📌共享引用
    ///
    /// # 📄OpenNARS
    ///
    /// The selected task
    fn current_task(&self) -> &RCTask;
    /// 获取「当前任务」（可变）
    /// * 📌共享引用
    fn current_task_mut(&mut self) -> &mut RCTask;

    /// 让「推理器」吸收「推理上下文」
    /// * 🚩【2024-05-19 18:39:44】现在会在每次「准备上下文⇒推理」的过程中执行
    /// * 🎯变量隔离，防止「上下文串线」与「重复使用」
    /// * 📌传入所有权而非引用
    /// * 🚩【2024-05-21 23:17:57】现在迁移到「推理上下文」处，以便进行方法分派
    fn absorbed_by_reasoner(self, reasoner: &mut Reasoner);

    // TODO: 将以下逻辑迁移到单独的「自动实现之特征」中
    /// 共用终端逻辑：「激活任务」
    /// # 📄OpenNARS
    ///
    /// Activated task called in MatchingRules.trySolution and
    /// Concept.processGoal
    /// * 📝仅被「答问」调用
    fn activated_task(
        &mut self,
        new_budget: BudgetValue,
        new_task: &JudgementV1,
        candidate_belief: &JudgementV1,
    ) {
        let task = Task::new(
            SentenceV1::JudgementV1(new_task.clone()),
            new_budget,
            Some(self.current_task().clone()),
            Some(new_task.clone()),
            Some(candidate_belief.clone()),
        );
        // * 🚩现在重新改为`COMMENT`，但更详细地展示「任务」本身
        self.add_output(Output::COMMENT {
            content: format!("!!! Activated: {}", task.to_display_long()),
        });
        // // * 🚩若为「问题」⇒输出显著的「导出结论」
        self.add_new_task(task);
    }

    /// 共用终端逻辑：「导出任务」
    ///
    /// # 📄OpenNARS
    ///
    /// Derived task comes from the inference rules.
    fn derived_task(&mut self, new_task: Task) {
        // * 🚩判断「导出的新任务」是否有价值
        if !new_task.budget_above_threshold(self.parameters().budget_threshold) {
            self.add_output(Output::COMMENT {
                content: format!("!!! Ignored: {}", new_task.to_display_long()),
            });
            return;
        }
        // * 🚩报告
        self.add_output(Output::COMMENT {
            content: format!("!!! Derived: {}", new_task.to_display_long()),
        });
        let budget_summary = new_task.budget_summary().to_float();
        if budget_summary > self.silence_percent() {
            // only report significant derived Tasks
            let narsese = Some(NarseseValue::Task(new_task.to_lexical()));
            self.add_output(Output::OUT {
                content_raw: format!("OUT: {}", new_task.to_display_long()),
                narsese,
            });
        }
        // * 🚩将「导出的新任务」添加到「新任务表」中
        self.add_new_task(new_task);
    }

    /// 🆕仅源自「修正规则」调用，没有「父信念」
    fn double_premise_task_revision(
        &mut self,
        new_content: Term,
        new_truth: TruthValue,
        new_budget: BudgetValue,
        new_stamp: Stamp,
    ) {
        // * 🚩仅在「任务内容」可用时构造
        let current_task = self.current_task().get_(); // 不能当场变为引用：后续可能要再借用自身
        let new_punctuation = current_task.punctuation();
        let new_sentence = SentenceV1::new_sentence_from_punctuation(
            new_content,
            new_punctuation,
            new_stamp,
            Some((new_truth, true)),
        );
        drop(current_task); // ! 先抛掉引用代理
        match new_sentence {
            Ok(new_sentence) => {
                let new_task = Task::new(
                    new_sentence,
                    new_budget,
                    Some(self.current_task().clone()),
                    None,
                    None,
                );
                self.derived_task(new_task);
            }
            Err(error) => self.add_output(Output::ERROR {
                description: error.to_string(),
            }),
        }
    }
}

/// 「概念推理（中层）上下文」
/// * 🎯用于统一「转换推理」与「概念推理」的逻辑
/// * * 🚩统一的「当前信念」（一致可空）、「用于预算推理的当前信念链」等附加要求
/// * * ✨更多的「单前提结论」「多前提结论」导出方法
pub trait ReasonContextConcept: ReasonContext {
    /// 获取「当前信念」
    /// * 📌仅在「概念推理」中用到
    /// * 🚩对于用不到的实现者，只需实现为空
    fn current_belief(&self) -> Option<&JudgementV1>;

    /// 🆕实用方法：用于简化「推理规则分派」的代码
    fn has_current_belief(&self) -> bool {
        self.current_belief().is_some()
    }

    /// 获取用于「预算推理」的「当前信念链」
    /// * 📌仅在「概念推理」中非空
    /// * 🚩对于用不到的实现者，只需实现为空
    /// * 🎯【2024-06-09 11:25:14】规避对`instanceof DerivationContextReason`的滥用
    fn belief_link_for_budget_inference(&mut self) -> Option<&mut TermLink>;

    // TODO: 统一迁移到别的模块
    /// 🆕产生新时间戳 from 单前提
    fn generate_new_stamp_single(&self) -> Stamp {
        let current_task = self.current_task().get_();
        match (current_task.is_judgement(), self.current_belief()) {
            // * 🚩「当前任务」是判断句 | 没有「当前信念」
            (true, _) | (_, None) => Stamp::with_old(&*current_task, self.time()),
            // * 🚩其它 ⇒ 时间戳来自信念
            // to answer a question with negation in NAL-5 --- move to activated task?
            (false, Some(belief)) => Stamp::with_old(belief, self.time()),
        }
    }

    /// 🆕产生新时间戳 from 双前提
    fn generate_new_stamp_double(&self) -> Option<Stamp> {
        let current_task = &*self.current_task().get_();
        // * 🚩在具有「当前信念」时返回「与『当前任务』合并的时间戳」
        self.current_belief().map(|belief|
                // * 📌此处的「时间戳」一定是「当前信念」的时间戳
                // * 📄理由：最后返回的信念与「成功时比对的信念」一致（只隔着`clone`）
                 Stamp::from_merge_unchecked(current_task, belief, self.time(), self.max_evidence_base_length()))
    }

    // * 📄「转换推理上下文」「概念推理上下文」仅作为「当前任务链之目标」
    // ! 【2024-06-27 00:48:01】但Rust不支持「转换为默认实现」

    /// 获取当前任务链
    fn current_task_link(&self) -> &TaskLink;

    /// 获取当前任务链（可变）
    fn current_task_link_mut(&mut self) -> &mut TaskLink;

    /// Shared final operations by all double-premise rules, called from the
    /// rules except StructuralRules
    /// * 🚩【2024-05-19 12:44:55】构造函数简化：导出的结论【始终可修正】
    fn double_premise_task(
        &mut self,
        new_content: Term,
        new_truth: Option<TruthValue>,
        new_budget: BudgetValue,
    ) {
        // * 🚩尝试创建「新时间戳」然后使用之
        if let Some(new_stamp) = self.generate_new_stamp_double() {
            let new_truth_revisable = new_truth.map(|truth| (truth, true));
            self.double_premise_task_full(
                None,
                new_content,
                new_truth_revisable,
                new_budget,
                new_stamp,
            )
        }
    }

    /// 🆕其直接调用来自组合规则、匹配规则（修正）
    /// * 🎯避免对`currentTask`的赋值，解耦调用（并让`currentTask`不可变）
    /// * 🎯避免对`newStamp`的复制，解耦调用（让「新时间戳」的赋值止步在「推理开始」之前）
    /// * 🚩【2024-06-27 00:52:39】为避免借用冲突，此处使用[`Option`]区分「传入其它地方引用/使用自身引用」
    ///   * 有值 ⇒ 使用内部的值
    ///   * 空值 ⇒ 从`self`中拿取
    fn double_premise_task_full(
        &mut self,
        current_task: Option<&Task>,
        new_content: Term,
        new_truth_revisable: Option<(TruthValue, bool)>,
        new_budget: BudgetValue,
        new_stamp: Stamp,
    ) {
        // * 🚩参考「传入任务/自身默认任务」构造标点
        let new_punctuation = current_task
            .unwrap_or(&*self.current_task().get_()) // 立即使用的不可变引用
            .punctuation();
        let new_sentence = SentenceV1::new_sentence_from_punctuation(
            new_content,
            new_punctuation,
            new_stamp,
            new_truth_revisable,
        );
        if let Ok(sentence) = new_sentence {
            let new_task = Task::from_derived(
                sentence,
                new_budget,
                Some(self.current_task().clone()),
                self.current_belief().cloned(),
            );
            // * 🚩正式导出结论（在这之前注销代理）
            self.derived_task(new_task);
        }
    }

    /// 🆕重定向
    fn double_premise_task_not_revisable(
        &mut self,
        new_content: Term,
        new_truth: Option<TruthValue>,
        new_budget: BudgetValue,
    ) {
        todo!("【2024-06-27 01:10:54】后续再弄")
    }

    //     /// Shared final operations by all double-premise rules,
    // /// called from the rules except StructuralRules
    // double_premise_task_
}

/// 重置全局状态
/// * 🚩重置「全局随机数生成器」
/// * 📌【2024-06-26 23:36:06】目前计划做一个全局的「伪随机数生成器初始化」
///
/// TODO: 功能实装
#[doc(alias = "init")]
pub fn init_global() {
    todo!()
}

/// 🆕内置公开结构体，用于公共读取
#[derive(Debug)]
pub struct ReasonContextCore<'this> {
    /// 缓存的「当前时间」
    /// * 🎯与「记忆区」解耦
    time: ClockTime,

    /// 缓存的「静默值」
    /// * 🚩【2024-05-30 09:02:10】现仅在构造时赋值，其余情况不变
    silence_value: usize,

    /// 新增加的「任务列表」
    /// * 📍【2024-06-26 20:54:20】因其本身新创建，故可不用「共享引用」
    ///   * 💭在「被推理器吸收」时，才需要共享引用
    /// * 🚩【2024-05-18 17:29:40】在「记忆区」与「推理上下文」中各有一个，但语义不同
    /// * 📌「记忆区」的跨越周期，而「推理上下文」仅用于存储
    ///
    /// # 📄OpenNARS
    /// List of new tasks accumulated in one cycle, to be processed in the next cycle
    new_tasks: Vec<Task>,

    /// 🆕新的NAVM输出
    /// * 🚩用以复刻`exportStrings`与`stringsToRecord`二者
    outputs: Vec<Output>,

    /// 当前概念
    ///
    /// # 📄OpenNARS
    ///
    /// The selected Concept
    current_concept: Concept,

    /// 🆕引用的「超参数」对象
    parameters: &'this Parameters,
}

impl<'this> ReasonContextCore<'this> {
    /// 构造函数 from 推理器
    /// * 📝需要保证「推理器」的生命周期覆盖上下文
    pub fn new<'p: 'this>(
        current_concept: Concept,
        parameters: &'p Parameters,
        time: ClockTime,
        silence_value: usize,
    ) -> Self {
        Self {
            time,
            silence_value,
            current_concept,
            new_tasks: vec![],
            outputs: vec![],
            parameters,
        }
    }

    /// 与[`Self::new`]不同的是：要借用整个推理器
    pub fn from_reasoner<'r: 'this>(current_concept: Concept, reasoner: &'r Reasoner) -> Self {
        Self::new(
            current_concept,
            reasoner.parameters(),
            reasoner.time(),
            reasoner.silence_value(),
        )
    }
}

/// ! ⚠️仅用于「统一委托的方法实现」
/// * ❗某些方法将不实现
impl ReasonContextCore<'_> {
    pub fn time(&self) -> ClockTime {
        self.time
    }

    pub fn parameters(&self) -> &Parameters {
        self.parameters
    }

    pub fn silence_percent(&self) -> Float {
        self.silence_value as Float / 100.0
    }

    pub fn num_new_tasks(&self) -> usize {
        self.new_tasks.len()
    }

    pub fn add_new_task(&mut self, task: Task) {
        self.new_tasks.push(task);
    }

    pub fn add_output(&mut self, output: Output) {
        self.outputs.push(output);
    }

    pub fn current_concept(&self) -> &Concept {
        &self.current_concept
    }

    pub fn current_concept_mut(&mut self) -> &mut Concept {
        &mut self.current_concept
    }

    /// 共用的方法：被推理器吸收
    pub fn absorbed_by_reasoner(self, reasoner: &mut Reasoner) {
        let memory = reasoner.memory_mut();
        // * 🚩将「当前概念」归还到「推理器」中
        memory.put_back_concept(self.current_concept);
        // * 🚩将推理导出的「新任务」添加到自身新任务中（先进先出）
        for new_task in self.new_tasks {
            let task_rc = RC::new_(new_task);
            reasoner.add_new_task(task_rc);
        }
        // * 🚩将推理导出的「NAVM输出」添加进自身「NAVM输出」中（先进先出）
        for output in self.outputs {
            reasoner.report(output);
        }
        // * ✅Rust已在此处自动销毁剩余字段
    }
}
