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
    control::{Parameters, ReasonRecorder, Reasoner},
    entity::{Concept, JudgementV1, RCTask, Task, TaskLink, TermLink},
    global::{ClockTime, Float},
    language::Term,
    storage::Memory,
};
use navm::output::Output;
use std::ops::{Deref, DerefMut};

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

    /// 复刻自改版`DerivationContext.noNewTask`
    /// * 🚩语义改为「是否有新任务」
    fn has_result(&self) -> bool {
        self.num_new_tasks() > 0
    }

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
    #[doc(alias = "add_output")]
    fn report(&mut self, output: Output);

    /// 派生易用性方法
    fn report_comment(&mut self, message: impl ToString) {
        self.report(ReasonRecorder::output_comment(message));
    }

    /// 派生易用性方法
    fn report_out(&mut self, narsese: &Task) {
        self.report(ReasonRecorder::output_out(narsese));
    }

    /// 派生易用性方法
    fn report_error(&mut self, description: impl ToString) {
        self.report(ReasonRecorder::output_error(description));
    }

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

    /// 获取「已存在的概念」（从「键」出发）
    /// * 🎯让「概念推理」可以在「拿出概念」的时候运行，同时不影响具体推理过程
    /// * 🚩先与「当前概念」做匹配，若没有再在记忆区中寻找
    fn key_to_concept(&self, key: &str) -> Option<&Concept> {
        match key == Memory::term_to_key(self.current_term()) {
            true => Some(self.current_concept()),
            false => self.memory().key_to_concept(key),
        }
    }

    /// 获取「当前任务」（不变）
    /// * 📌共享引用（需要是[`Deref`]）
    ///
    /// # 📄OpenNARS
    ///
    /// The selected task
    fn current_task<'r, 's: 'r>(&'s self) -> impl Deref<Target = RCTask> + 'r;
    /// 获取「当前任务」（可变）
    /// * 📌共享引用
    fn current_task_mut<'r, 's: 'r>(&'s mut self) -> impl DerefMut<Target = RCTask> + 'r;

    /// 让「推理器」吸收「推理上下文」
    /// * 🚩【2024-05-19 18:39:44】现在会在每次「准备上下文⇒推理」的过程中执行
    /// * 🎯变量隔离，防止「上下文串线」与「重复使用」
    /// * 📌传入所有权而非引用
    /// * 🚩【2024-05-21 23:17:57】现在迁移到「推理上下文」处，以便进行方法分派
    /// * 🚩【2024-06-28 00:06:45】现在「内置推理器可变引用」后，不再需要第二个参数
    ///   * ✅「推理器引用」可以从自身中取出来
    fn absorbed_by_reasoner(self);
}

/// 「概念推理上下文+链接」
/// * 🎯用于统一「转换推理」与「概念推理」的逻辑
///   * 🚩统一的「当前信念」（一致可空）、「用于预算推理的当前信念链」等附加要求
///   * ✨更多的「单前提结论」「多前提结论」导出方法
/// * 📝其中「当前信念链」放在「概念推理上下文」独有
pub trait ReasonContextWithLinks: ReasonContext {
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

    // * 📄「转换推理上下文」「概念推理上下文」仅作为「当前任务链之目标」
    // ! 【2024-06-27 00:48:01】但Rust不支持「转换为默认实现」

    /// 获取当前任务链
    fn current_task_link(&self) -> &TaskLink;

    /// 获取当前任务链（可变）
    fn current_task_link_mut(&mut self) -> &mut TaskLink;
}

/// 重置全局状态
/// * 🚩重置「全局随机数生成器」
/// * 📌【2024-06-26 23:36:06】目前计划做一个全局的「伪随机数生成器初始化」
///
#[doc(alias = "init")]
pub fn init_global_reason_parameters() {
    eprintln!("// TODO: 功能实装")
}

/// 🆕内置公开结构体，用于公共读取
#[derive(Debug)]
pub struct ReasonContextCore<'this> {
    /// 对「推理器」的反向引用
    /// * 🚩【2024-05-18 17:00:12】目前需要访问其「输出」「概念」等功能
    ///   * 📌需要是可变引用
    /// * 🚩【2024-06-28 00:00:37】目前需要从「推理上下文」视角 锁定整个「推理器」对象
    ///   * 🎯避免「引用推理器的一部分后，还借用着整个推理器」的借用问题
    pub(in crate::control) reasoner: &'this mut Reasoner,

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
    pub(in crate::control) new_tasks: Vec<Task>,

    /// 🆕新的NAVM输出
    /// * 🚩用以复刻`exportStrings`与`stringsToRecord`二者
    pub(in crate::control) outputs: Vec<Output>,

    /// 当前概念
    ///
    /// # 📄OpenNARS
    ///
    /// The selected Concept
    pub(in crate::control) current_concept: Concept,
}

impl<'this> ReasonContextCore<'this> {
    /// 构造函数 from 推理器
    /// * 📝需要保证「推理器」的生命周期覆盖上下文
    pub fn new<'p: 'this>(reasoner: &'p mut Reasoner, current_concept: Concept) -> Self {
        Self {
            time: reasoner.time(),
            silence_value: reasoner.silence_value(),
            current_concept,
            reasoner,
            new_tasks: vec![],
            outputs: vec![],
        }
    }
}

/// ! ⚠️仅用于「统一委托的方法实现」
/// * ❗某些方法将不实现
impl ReasonContextCore<'_> {
    /// 🆕对「推理器」的可变引用
    /// * 🚩用于「被推理器吸收」
    pub fn reasoner_mut(&mut self) -> &mut Reasoner {
        self.reasoner
    }
    /// 对「记忆区」的不可变引用
    pub fn memory(&self) -> &Memory {
        &self.reasoner.memory
    }

    /// 📝对「记忆区」的可变引用，只在「直接推理」中用到
    pub fn memory_mut(&mut self) -> &mut Memory {
        &mut self.reasoner.memory
    }

    pub fn time(&self) -> ClockTime {
        self.time
    }

    pub fn parameters(&self) -> &Parameters {
        &self.reasoner.parameters
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
    pub fn absorbed_by_reasoner(self) {
        let reasoner = self.reasoner;
        let memory = reasoner.memory_mut();
        // * 🚩将「当前概念」归还到「推理器」中
        memory.put_back_concept(self.current_concept);
        // * 🚩将推理导出的「新任务」添加到自身新任务中（先进先出）
        for new_task in self.new_tasks {
            reasoner.derivation_datas.add_new_task(new_task);
        }
        // * 🚩将推理导出的「NAVM输出」添加进自身「NAVM输出」中（先进先出）
        for output in self.outputs {
            reasoner.report(output);
        }
        // * ✅Rust已在此处自动销毁剩余字段
    }
}

#[macro_export]
macro_rules! __delegate_from_core {
    () => {
        fn memory(&self) -> &Memory {
            self.core.memory()
        }

        fn time(&self) -> ClockTime {
            self.core.time()
        }

        fn parameters(&self) -> &Parameters {
            self.core.parameters()
        }

        fn silence_percent(&self) -> Float {
            self.core.silence_percent()
        }

        fn num_new_tasks(&self) -> usize {
            self.core.num_new_tasks()
        }

        fn add_new_task(&mut self, task: Task) {
            self.core.add_new_task(task)
        }

        fn report(&mut self, output: Output) {
            self.core.add_output(output)
        }

        fn current_concept(&self) -> &Concept {
            self.core.current_concept()
        }

        fn current_concept_mut(&mut self) -> &mut Concept {
            self.core.current_concept_mut()
        }
    };
}
