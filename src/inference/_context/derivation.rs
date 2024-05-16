//! 🆕「推导上下文」
//! * 🎯承载并迁移OpenNARS「记忆区」中的「临时推理状态」变量组
//! * 📄亦仿自OpenNARS 3.x（3.0.4）`DerivationContext`
//! * 📝【2024-05-12 02:17:38】基础数据结构可以借鉴OpenNARS 1.5.8，但涉及「推理」的部分，建议采用OpenNARS 3.0.4的架构来复刻

use super::ReasonContext;
use crate::{
    global::{ClockTime, Float},
    language::Term,
};
use navm::output::Output;

/// 🆕「推导上下文」
/// * 🎯承载状态变量，解耦「记忆区」
///   * 🚩替代在「真值函数」「预算函数」「推理规则」中的「记忆区」引用
///   * 📌有利于在Rust中实现「数据解耦」
///   * 💭可能经此无需再考虑RC等「共享引用」类型
/// * 🎯实现「开始推理⇒创建上下文⇒具体推理⇒回收上下文」的新「推理过程」
///   * 💭基于「概念+词项链+任务链」的【可并行化】推理
/// * 🚩【2024-05-12 16:09:29】不堪`<Self as XXX>`其扰，要求实现特征[`Sized`]
///
/// ## 所有权/可空性 笔记
///
/// * 📝⚠️【2024-05-12 20:22:11】经OpenNARS`long_term_stability.nal`测试，基本确定「空/非空」情形
///   * 📌仅`currentBelief`、`newStamp`两个字段可空
///   * 🚩【2024-05-12 20:22:41】暂时全部使用`Option`+`unwrap`代替
/// * ❓【2024-05-12 20:46:30】目前对「所有权」仍然存疑
///   * 🚩【2024-05-12 20:46:38】暂时按「具备所有权」的形式做
///   * 💭后续可能仍然要换RC，比如「先放回再推理」「引用整体又引用部分」的情形
///
/// ## 概念设计笔记
///
/// * 💭【2024-05-08 17:21:00】大致方案：
///   * 📌「记忆区」应该作为一个纯粹的「概念/新任务/新近任务 存储器」来使用
///   * 🚩建立「推理上下文」：其中的数据从「记忆区」取出，经过「推理」生成派生任务与信息，最终「归还」记忆区
///   * 🚩原属于「记忆区」的推理过程有关函数（如`cycle`），应该放在更顶层的「Reasoner」即「推理器」中，统一调度
///     * 🚩并且「推理上下文」应该与「记忆区」平级，统一受「推理器」主控调用
pub trait DerivationContext: ReasonContext + Sized {
    /* ---------- Short-term workspace for a single cycle ---------- */

    /// 🆕跟随OpenNARS 3.0.4，要求存储对「记忆区」的引用
    /// * 🚩至于这个「引用」如何存储（带生命周期的内部指针等），可自由决定
    /// * 🎯目前首次用于[「预算推理」](super::BudgetFunctions::__budget_inference)，上游是「组合规则通过词项优先级调整策略」
    /// * 🎯目前仅只读
    fn memory(&self) -> &Self::Memory;

    /// 模拟`Memory.newTasks`
    /// * 🚩读写：OpenNARS中要读写对象
    ///   * 🚩【2024-05-12 14:38:58】决议：两头都有
    ///     * 在「记忆区回收上下文」时从「上下文的『新任务』接收」
    ///   * 📌作为一个「推理之后要做的事情」而非「推理期间要做的事情」看待
    ///
    /// # 📄OpenNARS
    ///
    /// List of new tasks accumulated in one cycle, to be processed in the next cycle
    fn new_tasks(&self) -> &[Self::Task];
    /// [`Memory::__new_tasks`]的可变版本
    /// * 🚩【2024-05-07 21:13:39】暂时用[`Vec`]代替
    ///   * 📌在「推导上下文」中只会增加，不会被移除
    fn __new_tasks_mut(&mut self) -> &mut Vec<Self::Task>;

    // ! ❌【2024-05-07 21:16:10】不复刻`Memory.exportStrings`：🆕使用新的输出系统，不用OpenNARS那一套

    /// 模拟`Memory.currentTerm`
    /// * 🚩公开读写：因为要被「推理规则」使用
    /// * 🚩经OpenNARS`long_term_stability.nal`测试，非空
    ///
    /// # 📄OpenNARS
    ///
    /// The selected Term
    fn current_term(&self) -> &Term;
    /// [`Memory::current_term`]的可变版本
    fn __current_term_mut(&mut self) -> &mut Term;

    /// 模拟`Memory.currentConcept`
    /// * 🚩经OpenNARS`long_term_stability.nal`测试，非空
    ///
    /// # 📄OpenNARS
    ///
    /// The selected Concept
    fn current_concept(&self) -> &Option<Self::Concept>;
    /// [`Memory::current_concept`]的可变版本
    fn current_concept_mut(&mut self) -> &mut Option<Self::Concept>;

    /// 模拟`Memory.currentTaskLink`
    /// * 🚩经OpenNARS`long_term_stability.nal`测试，非空
    ///
    /// # 📄OpenNARS
    ///
    fn current_task_link(&self) -> &Option<Self::TaskLink>;
    /// [`Memory::current_task_link`]的可变版本
    fn current_task_link_mut(&mut self) -> &mut Option<Self::TaskLink>;

    /// 模拟`Memory.currentTask`
    /// * 🚩【2024-05-08 11:17:37】为强调「引用」需要，此处返回[`RC`]而非引用
    /// * 🚩经OpenNARS`long_term_stability.nal`测试，非空
    ///
    /// # 📄OpenNARS
    ///
    /// The selected Task
    fn current_task(&self) -> &Self::Task;
    /// [`Memory::current_task`]的可变版本
    fn current_task_mut(&mut self) -> &mut Self::Task;

    /// 模拟`Memory.currentBeliefLink`
    /// * 🚩【2024-05-08 14:33:03】仍有可能为空：见[`Memory::__fire_concept`]
    /// * 🚩经OpenNARS`long_term_stability.nal`测试，非空
    ///
    /// # 📄OpenNARS
    ///
    /// The selected TermLink
    fn current_belief_link(&self) -> &Option<Self::TermLink>;
    /// [`Memory::current_belief_link`]的可变版本
    fn current_belief_link_mut(&mut self) -> &mut Option<Self::TermLink>;

    /// 模拟`Memory.currentBelief`
    /// * 🚩【2024-05-08 11:49:37】为强调「引用」需要，此处返回[`RC`]而非引用
    /// * 🚩经OpenNARS`long_term_stability.nal`测试，仍有可能为空
    ///   * 🚩【2024-05-08 14:33:03】见[`Memory::single_premise_task`]
    ///
    /// # 📄OpenNARS
    ///
    /// The selected belief
    fn current_belief(&self) -> &Option<Self::Sentence>;
    /// [`Memory::current_belief`]的可变版本
    fn current_belief_mut(&mut self) -> &mut Option<Self::Sentence>;

    /// 模拟`Memory.newStamp`
    /// * 🚩【2024-05-12 17:49:18】即便此处可空，也不应是`Option<&>`而应该是`&Option<>`
    ///   * 📌理由：方便复制，性能开销少（不会新创`Option`）且转换容易（[`Option::as_ref`]）
    /// * ⚠️在推理开始时，此值可能为空
    ///   * 📄【2024-05-12 19:34:42】已经过`long_term_stability.nal`测试
    ///
    /// # 📄OpenNARS
    ///
    fn new_stamp(&self) -> &Option<Self::Stamp>;
    /// [`Memory::new_stamp`]的可变版本
    fn new_stamp_mut(&mut self) -> &mut Option<Self::Stamp>;

    // ! ❌【2024-05-07 21:26:49】暂不使用
    // 📄OpenNARS："TODO unused"
    // /// 模拟`Memory.substitute`
    // ///
    // /// # 📄OpenNARS
    // ///
    // fn substitute(&self) -> &VarSubstitution;
    // /// [`Memory::substitute`]的可变版本
    // fn substitute_mut(&mut self) -> &mut VarSubstitution;

    // ! ❌【2024-05-07 21:25:23】暂不模拟`Memory.randomNumber`
    //   * 📝OpenNARS中仅在「可交换复合词项匹配」`find_substitute`用到

    /* ---------------- 推理 超参数 ---------------- */

    /* ---------- access utilities ---------- */

    /// 模拟`Memory.getTime`
    /// * 🎯【2024-05-06 21:13:48】从[`Concept::get_belief`]来
    /// * 🚩【2024-05-12 14:57:37】现在从[「记忆区」](crate::storage::Memory)中迁移而来：实际上与「记忆区」无关
    ///
    /// # 📄OpenNARS
    ///
    /// 🈚
    #[doc(alias = "get_time")]
    fn time(&self) -> ClockTime {
        /* 📄OpenNARS源码：
        return reasoner.getTime(); */
        todo!("// TODO: 后续要迁移")
    }

    /// 模拟`Memory.noResult`
    /// * 🚩【2024-05-12 14:57:37】现在从[「记忆区」](crate::storage::Memory)中迁移而来：实际上与「记忆区」无关
    ///
    /// # 📄OpenNARS
    ///
    /// Actually means that there are no new Tasks
    fn no_result(&self) -> bool {
        /* 📄OpenNARS源码：
        return newTasks.isEmpty(); */
        self.new_tasks().is_empty()
    }

    /// 🆕模拟`Memory.reasoner.getSilenceValue().get()`
    /// * 🎯【2024-05-06 21:13:48】从[`Memory::derived_task`]来
    /// * 🚩现在从「记忆区」迁移至「推导上下文」：实际上与「记忆区」无关
    ///   * 📌【2024-05-12 14:55:34】妥协：不仅会影响「输出」或「输入」，而且仍然影响推理过程
    ///
    #[doc(alias = "get_silence_value")]
    fn silence_value(&self) -> usize {
        /* 📄OpenNARS源码：
        return reasoner.getTime(); */
        todo!("// TODO: 后续要迁移")
    }

    /// 🆕简化`self.silence_value() as Float / 100 as Float`逻辑
    /// * 🎯统一表示「音量」的百分比（静音の度）
    #[inline(always)]
    fn silence_percent(&self) -> Float {
        self.silence_value() as Float / 100 as Float
    }

    /* ---------------- 推理结果缓存与记录 ---------------- */

    /// 🆕新的「推理输出」
    /// * 🚩用于「延迟决定」
    ///   * 📌先在上下文中缓存输出，等到记忆区推理完毕后，再根据其中的结果决定「是否要输出」
    fn cached_reports(&self) -> &[Output];
    fn cached_reports_mut(&mut self) -> &mut Vec<Output>;

    /// 🆕缓存一条「推理输出」
    /// * 📌功能类似OpenNARS`Memory.report`
    #[inline(always)]
    fn report(&mut self, output: Output) {
        self.cached_reports_mut().push(output);
    }
}

/// 「推导上下文」的「具体类型」
/// * 🎯构造函数
pub trait DerivationContextConcrete: DerivationContext {
    /// 构造函数
    fn new(
        builder: DerivationContextBuilder<'_, Self>,
        memory: &Self::Memory,
        time: ClockTime,
        silence_value: usize,
    ) -> Self;
}

/// 用来构建「推理上下文」的结构
/// * 🎯打包作为构建「推理上下文」的参数
/// * 🚩【2024-05-12 20:58:07】目前均带上引用，并且不带上超参数
/// * 🚩具体用法尚待商酌
///   * 💭【2024-05-12 20:59:09】计划用在[`RuleTables::reason`]调用之前
///     * 🚩推理之前先构建好此「脚手架」对象
///     * 🚩推理开始之前，尝试转换到正式的上下文
pub struct DerivationContextBuilder<'s, Context: DerivationContext> {
    /// 当前词项
    pub current_term: Option<&'s Term>,
    /// 当前概念
    pub current_concept: Option<&'s Context::Concept>,
    /// 当前任务链
    pub current_task_link: Option<&'s Context::TaskLink>,
    /// 当前任务
    pub current_task: Option<&'s Context::Task>,
    /// 当前信念链（词项链）
    pub current_belief_link: Option<&'s Context::TermLink>,
    /// 当前信念
    /// * 📌允许空值：无信念单任务推理
    pub current_belief: Option<&'s Context::Sentence>,
    /// 新时间戳
    /// * 📌允许空值：创建与使用距离较远之情况
    ///   * 📝OpenNARS中：在推理过程中创建，在「最终得出结论」时使用（构造新语句/新任务）
    pub new_stamp: Option<&'s Context::Stamp>,
}
