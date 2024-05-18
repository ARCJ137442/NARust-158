//! 🆕「推导上下文」
//! * 🎯承载并迁移OpenNARS「记忆区」中的「临时推理状态」变量组
//! * 📄亦仿自OpenNARS 3.x（3.0.4）`DerivationContext`
//! * 📝【2024-05-12 02:17:38】基础数据结构可以借鉴OpenNARS 1.5.8，但涉及「推理」的部分，建议采用OpenNARS 3.0.4的架构来复刻

use super::ReasonContext;
use crate::{
    entity::Concept,
    global::{ClockTime, Float},
    language::Term,
    *,
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
/// * 🚩【2024-05-17 15:22:46】重要决议：通过「泛型」而非「派生」进行「自动实现」
///   * 📌不堪各种`Self::Type`歧义
///   * 📌**不堪「在其它无『推导上下文』的地方使用『推导上下文』」时「要用大量关联类型对齐」的烦扰
///     * 📄如：`trait T: ReasonContext { fn f(context: DerivationContext<ShortFloat=Self::ShortFloat, 【...】>) }`
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
pub trait DerivationContext<C: ReasonContext>: ReasonContext + Sized {
    /* ---------- Short-term workspace for a single cycle ---------- */

    /// 🆕跟随OpenNARS 3.0.4，要求存储对「记忆区」的引用
    /// * 🚩至于这个「引用」如何存储（带生命周期的内部指针等），可自由决定
    /// * 🎯目前首次用于[「预算推理」](super::BudgetFunctions::__budget_inference)，上游是「组合规则通过词项优先级调整策略」
    /// * 🎯目前仅只读
    fn memory(&self) -> &C::Memory;

    /// 模拟`Memory.currentBelief`
    /// * 🚩【2024-05-08 11:49:37】为强调「引用」需要，此处返回[`RC`]而非引用
    /// * 🚩经OpenNARS`long_term_stability.nal`测试，仍有可能为空
    ///   * 🚩【2024-05-08 14:33:03】见[`Memory::single_premise_task`]
    /// * 📝【2024-05-17 17:34:53】经OpenNARS研究：多个⇒需要可变版本
    ///   * 📝OpenNARS的`Concept.fire`中的确可空，不论是在「开始推理」还是「转换」还是「正式开始」时
    ///   * 📝OpenNARS的`Concept.fire`中需要「一个任务链同多个词项链做推理」
    ///
    /// # 📄OpenNARS
    ///
    /// The selected belief
    fn current_belief(&self) -> &Option<C::Sentence>;
    /// [`Memory::current_belief`]的可变版本
    fn current_belief_mut(&mut self) -> &mut Option<C::Sentence>;

    /// 模拟`Memory.newStamp`
    /// * 🚩【2024-05-12 17:49:18】即便此处可空，也不应是`Option<&>`而应该是`&Option<>`
    ///   * 📌理由：方便复制，性能开销少（不会新创`Option`）且转换容易（[`Option::as_ref`]）
    /// * ⚠️在推理开始时，此值可能为空
    ///   * 📄【2024-05-12 19:34:42】已经过`long_term_stability.nal`测试
    ///
    /// # 📄OpenNARS
    ///
    fn new_stamp(&self) -> &Option<C::Stamp>;
    /// [`Memory::new_stamp`]的可变版本
    fn new_stamp_mut(&mut self) -> &mut Option<C::Stamp>;

    /// 模拟`Memory.newTasks`
    /// * 🚩读写：OpenNARS中要读写对象
    ///   * 🚩【2024-05-12 14:38:58】决议：两头都有
    ///     * 在「记忆区回收上下文」时从「上下文的『新任务』接收」
    ///   * 📌作为一个「推理之后要做的事情」而非「推理期间要做的事情」看待
    /// * 🚩此处仅作为临时变量
    ///   * 📄持久变量参考[`crate::nars::Reasoner::__new_tasks`]
    fn new_tasks(&self) -> &[C::Task];
    /// [`DerivationContext::new_tasks`]的可变版本
    /// * 🚩【2024-05-07 21:13:39】暂时用[`Vec`]代替
    ///   * 📌在「推导上下文」中只会增加，不会被移除
    fn __new_tasks_mut(&mut self) -> &mut Vec<C::Task>;

    // ! ❌【2024-05-07 21:16:10】不复刻`Memory.exportStrings`：🆕使用新的输出系统，不用OpenNARS那一套

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
    /// * 🚩【2024-05-17 16:22:29】现在将作为一个真正的「局部变量」缓存
    ///   * 📌直接从「推理器」中构建
    ///   * ❌不能借用推理器：这样会导致无法再可变借用任何推理器内的东西（包括「记忆区」）
    ///
    /// # 📄OpenNARS
    ///
    /// 🈚
    #[doc(alias = "get_time")]
    fn time(&self) -> ClockTime;

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
    fn new_outputs(&self) -> &[Output];
    fn new_outputs_mut(&mut self) -> &mut Vec<Output>;

    /// 🆕缓存一条「推理输出」
    /// * 📌功能类似OpenNARS`Memory.report`
    #[inline(always)]
    fn report(&mut self, output: Output) {
        self.new_outputs_mut().push(output);
    }
}

/// 「推导上下文」的「具体推理版本」
/// * 🎯作为「具体类型」与「`RuleTables.reason`中使用的『上下文』类型」
///   * 📌拥有构造函数
///   * 📌拥有与「立即本地推理」中与「可空项」不同的字段：一些原「可空」标记为「不可空」
pub trait DerivationContextReason<C: ReasonContext>: DerivationContext<C> {
    /* ========独有内容======== */
    // 构造函数
    fn __new() -> Self {
        todo!("// TODO: 结合所有属性构建功能");
    }

    /* ========承接自「推导上下文」======== */

    /// 模拟`Memory.currentTerm`
    /// * 🚩经OpenNARS`long_term_stability.nal`测试，非空
    /// * 📝经OpenNARS研究，无需可变版本
    ///   * 📝仅在`processConcept`、`immediateProcess`这两个「推理开始前方法」中被修改
    ///   * 🚩【2024-05-17 14:27:24】构造后即不可变
    /// * 📝【2024-05-18 02:39:48】经OpenNARS研究发现：始终代表「当前概念所对应的词项」
    ///   * 📌只要代码稍作变换，不管是「直接推理」还是「概念推理」均以此逻辑为主
    ///   * 🚩【2024-05-18 02:41:36】所以根本无需可变版本
    ///
    /// # 📄OpenNARS
    ///
    /// The selected Term
    fn current_term<'a>(&'a self) -> &'a Term
    where
        C::Concept: 'a,
    {
        self.current_concept().term()
    }

    /// 模拟`Memory.currentConcept`
    /// * 🚩经OpenNARS`long_term_stability.nal`测试，非空
    /// * 📝经OpenNARS研究，无需可变版本
    ///   * 📝仅在`processConcept`、`immediateProcess`这两个「推理开始前方法」中被修改
    ///   * 🚩【2024-05-17 14:27:24】构造后即不可变（不会被整个替换掉）
    /// * 🚩【2024-05-18 11:47:49】仍保留可变版本
    ///   * 📝仍然需要修改：将词项链放回「概念」中
    ///
    /// # 📄OpenNARS
    ///
    /// The selected Concept
    fn current_concept(&self) -> &C::Concept;
    /// [`Memory::current_concept`]的可变版本
    fn current_concept_mut(&mut self) -> &mut C::Concept;

    /// 模拟`Memory.currentTaskLink`
    /// * 🚩经OpenNARS`long_term_stability.nal`测试，非空
    /// * 📝经OpenNARS研究，仍需可变版本：仅引用不变，但内部可变
    ///   * 📄溯源：在「推理规则」的「预算推理」中，需要调用各种「调整预算」的方法
    ///
    /// # 📄OpenNARS
    ///
    fn current_task_link(&self) -> &C::TaskLink;
    /// [`Memory::current_task_link`]的可变版本
    fn current_task_link_mut(&mut self) -> &mut C::TaskLink;

    /// 模拟`Memory.currentTask`
    /// * 🚩【2024-05-08 11:17:37】为强调「引用」需要，此处返回[`RC`]而非引用
    /// * 🚩经OpenNARS`long_term_stability.nal`测试，非空
    /// * 📝经OpenNARS研究，仍需可变版本
    ///   * 📝不仅在`Memory.immediateProcess`与`Concept.fire`这俩「推理开始前方法」中被修改，
    ///   * ❗还会在`CompositionRules.decomposeStatement`中被修改
    ///   * 📄参见[`DerivationContextDirect::current_task`]
    ///   * ❓后续是否仍有可能要修改
    ///
    /// # 📄OpenNARS
    ///
    /// The selected Task
    fn current_task(&self) -> &C::Task;
    /// [`Memory::current_task`]的可变版本
    fn current_task_mut(&mut self) -> &mut C::Task;

    /// 模拟`Memory.currentBeliefLink`
    /// * 🚩【2024-05-08 14:33:03】仍有可能为空：见[`Memory::__fire_concept`]
    /// * 🚩经OpenNARS`long_term_stability.nal`测试，非空
    /// * 📝经OpenNARS研究，无需可变版本
    ///   * 📝仅在`Concept.fire`这个「推理开始前方法」中被修改
    ///   * 📌为了复用，必须修改
    ///
    /// # 📄OpenNARS
    ///
    /// The selected TermLink
    fn current_belief_link(&self) -> &C::TermLink;
    /// [`Memory::current_belief_link`]的可变版本
    fn current_belief_link_mut(&mut self) -> &mut C::TermLink;
}

/// 直接推导上下文
/// * 📌只为「具体类型」：必须从构造函数中构建，且此时全空
/// * 🎯面向「直接本地规则」设置
///   * 🎯对标OpenNARS`Concept.directProcess`、`Memory.immediateProcess`与
///   * 📝OpenNARS在`RuleTables.reason`前使用
///   * 📌存储「推导上下文」在「正式开始推导」前**可空**的临时变量
/// * 🚩【2024-05-17 17:19:44】仍然需要独立的字段，只是某些字段可空
///   * 🚩【2024-05-17 17:41:01】目前采用「共同派生自抽象特征」的方式
/// * 📝OpenNARS只对「概念」是「先放回再推理」的；对「词项链」「任务链」都是「先推理再放回」
///   * 💭【2024-05-17 20:38:12】因此可以在「推理上下文」中存储「当前任务链（任务）」与「当前词项链（当前信念）」
pub trait DerivationContextDirect<C: ReasonContext>: DerivationContext<C> {
    /* ========独有内容======== */
    /// 构建目标
    type Target: DerivationContextReason<C>;

    /// 构造函数
    /// * 🎯在「推理循环」中自动构造
    /// * 🚩默认为全空
    ///
    /// TODO: 【2024-05-17 16:48:12】后续仍需添加参数
    fn new() -> Self;

    /// 消耗自身以构建「推导上下文」
    /// * 🎯核心功能：创建推导上下文所必须
    /// * 🚩【2024-05-18 01:11:31】严格限定「错误」类型
    ///   * 📌核心逻辑：
    ///     * 预先检查，检查失败⇒返回`Err(self)`；
    ///     * 否则⇒解构并构建，并且一定成功
    fn build(
        self,
        memory: &C::Memory,
        time: ClockTime,
        silence_value: usize,
    ) -> Result<Self::Target, Self>;

    // TODO: 【2024-05-17 17:57:39】添加字段的可选版本

    /* ========承接自「推导上下文」======== */

    /// 模拟`Memory.currentTerm`
    /// * 🚩公开读写：因为要被「推理规则」使用
    /// * 🚩【2024-05-18 02:42:19】可简化：始终代表「当前概念之词项」，根本无需可变版本
    ///   * 参考[`DerivationContextReason::current_term`]
    ///
    /// # 📄OpenNARS
    ///
    /// The selected Term
    fn current_term<'a>(&'a self) -> Option<&'a Term>
    where
        C::Concept: 'a,
    {
        self.current_concept().as_ref().map(Concept::term)
    }
    // /// [`DerivationContextDirect::current_term`]的可变版本
    // fn __current_term_mut(&mut self) -> &mut Option<Term>;

    /// 模拟`Memory.currentConcept`
    /// * 🚩经OpenNARS`long_term_stability.nal`测试，非空
    /// * 📝经OpenNARS研究，无需可变版本
    ///   * 📝仅在`processConcept`、`immediateProcess`这两个「推理开始前方法」中被修改
    ///   * 🚩【2024-05-17 14:27:24】构造后即不可变
    ///   * ❓后续是否仍有可能要修改
    ///
    /// # 📄OpenNARS
    ///
    /// The selected Concept
    fn current_concept(&self) -> &Option<C::Concept>;
    /// [`DerivationContextDirect::current_concept`]的可变版本
    fn current_concept_mut(&mut self) -> &mut Option<C::Concept>;

    /// 模拟`Memory.currentTaskLink`
    /// * 🚩经OpenNARS`long_term_stability.nal`测试，非空
    /// * 📝经OpenNARS研究，仍需可变版本：仅引用不变，但内部可变
    ///   * 📄溯源：在「推理规则」的「预算推理」中，需要调用各种「调整预算」的方法
    ///
    /// # 📄OpenNARS
    ///
    fn current_task_link(&self) -> &Option<C::TaskLink>;
    /// [`DerivationContextDirect::current_task_link`]的可变版本
    fn current_task_link_mut(&mut self) -> &mut Option<C::TaskLink>;

    /// 模拟`Memory.currentTask`
    /// * 🚩读写
    /// * 📝经OpenNARS `Memory.immediateProcess`、`Concept.directProcess`、`Concept.linkToTask`，得出结论：
    ///   * ✅可以具备所有权：从「新任务/新近任务 缓冲区」中拿出，一路处理并最终分派到「词项链/任务链」中去
    /// * ❓「当前任务」与「当前任务链」一致吗
    ///   * ❌【2024-05-17 21:03:48】在`CompositionalRules.decomposeStatement`中不一致
    ///   * 📄`Task contentTask = new Task(contentBelief, task.getBudget());`
    ///   * 📄`memory.currentTask = contentTask;`
    ///   * 📌更改后的`currentTask`会在`doublePremiseTask`中被用于「构建新任务」
    ///   * 💭目的似乎是「要从『合取』中的某个内容去推导新词项」
    ///
    /// # 📄OpenNARS
    ///
    /// The selected Task
    fn current_task(&self) -> &Option<C::Task>;
    /// [`DerivationContextDirect::current_task`]的可变版本
    fn current_task_mut(&mut self) -> &mut Option<C::Task>;

    /// 模拟`Memory.currentBeliefLink`
    /// * 🚩读写
    /// * 📝OpenNARS在「开始推理」之前需要这个概念
    ///
    /// # 📄OpenNARS
    ///
    /// The selected TermLink
    fn current_belief_link(&self) -> &Option<C::TermLink>;
    /// [`DerivationContextDirect::current_belief_link`]的可变版本
    fn current_belief_link_mut(&mut self) -> &mut Option<C::TermLink>;
}

/// 初代实现
mod impl_v1 {
    use super::*;

    /// 初代「推理推导上下文」结构
    /// * 🚩【2024-05-17 13:37:30】目前均带上引用
    /// * 🎯【2024-05-12 20:59:09】用在[`RuleTables::reason`]之中
    ///   * 📌此即为「概念推理」上下文
    pub struct DerivationContextReasonV1<'s, C: ReasonContext> {
        /* ========从「构造函数」中来======== */
        /// 记忆区
        pub memory: &'s C::Memory,
        /// 当前时间
        pub time: ClockTime,
        /// 当前「静默值」
        pub silence_value: usize,

        /* ========从「直接推导上下文」中来======== */
        // * 🚩目前为方便，对此处变量直接使用「带所有权变量」
        // TODO: 【2024-05-17 16:11:38】🏗️后续仍然会调整：需要「控制机制」的顶层设计先敲定
        //   * ❓如：是否要在【取出概念】的同时做推理，是否要【取出任务】后推理，「词项链」「任务链」如何设计……
        // /// 当前词项
        // pub current_term: &'s Term,
        // * 📝OpenNARS源码
        /// 当前概念
        pub current_concept: &'s mut C::Concept,
        /// 当前任务链
        pub current_task_link: C::TaskLink,
        /// 当前任务
        /// * 🚩【2024-05-18 02:45:04】目前持有所有权：就是从「任务缓冲区」中拿出的
        ///
        /// TODO: 后续可能还要分「直接推理」与「概念推理」两种情况
        pub current_task: C::Task,
        /// 当前信念链（词项链）
        /// * 🚩【2024-05-18 02:45:04】目前持有所有权：就是从「词项链」中拿出的
        pub current_belief_link: C::TermLink,
        /// 当前信念
        /// * 📌可空：无信念单任务推理
        /// * 🚩【2024-05-17 14:49:55】目前持有所有权：特征定义需要
        ///   * ⚠️后续仍有可能更改
        pub current_belief: Option<C::Sentence>,

        /* ========缓存的「推导结果」变量======== */
        /// 新派生的任务
        pub new_tasks: Vec<C::Task>,
        /// 新产生的输出
        pub new_outputs: Vec<Output>,
        /// 新时间戳
        /// * 📌可空：创建与使用距离较远之情况
        ///   * 📝OpenNARS中：在推理过程中创建，在「最终得出结论」时使用（构造新语句/新任务）
        ///     * 📌但总是不会为`null`：在「语句」中并未检查到构造时`stamp == null`的情况
        /// * 🚩【2024-05-17 14:49:33】作为「推理产物」放在「推导结果变量」中
        pub new_stamp: Option<C::Stamp>,
    }

    /// 用来构建「推导上下文」的结构
    /// * 🎯打包作为构建「推导上下文」的参数
    /// * 🚩【2024-05-12 20:58:07】目前均带上引用，并且均为可空值
    ///   * 💭【2024-05-12 20:59:09】计划用在[`RuleTables::reason`]调用之前
    ///     * 🚩推理之前先构建好此「脚手架」对象
    ///     * 🚩推理开始之前，尝试转换到正式的上下文
    pub struct DerivationContextBuilderV1<'s, C: ReasonContext> {
        /// 当前词项
        pub current_term: Option<&'s Term>,
        /// 当前概念
        pub current_concept: Option<&'s C::Concept>,
        /// 当前任务链
        pub current_task_link: Option<&'s C::TaskLink>,
        /// 当前任务
        pub current_task: Option<&'s C::Task>,
        /// 当前信念链（词项链）
        pub current_belief_link: Option<&'s C::TermLink>,
        /// 当前信念
        /// * 📌允许保留空值：无信念单任务推理
        /// * 🚩【2024-05-17 14:47:40】目前按值存储：特征需要如此
        ///   * ⚠️后续仍有可能按「推理规则」的形式更改
        pub current_belief: Option<C::Sentence>,
        // ! 无需「新时间戳」：这是在「正式推理」过程中产生的新值
    }

    // 批量实现「推导上下文」
    impl_reason_context_from_generics! {
        C in ['s, C: ReasonContext]
        for DerivationContextReasonV1<'s, C> => ReasonContext

        C in ['s, C: ReasonContext]
        for DerivationContextBuilderV1<'s, C> => ReasonContext
    }

    /// 实现「推导上下文」
    impl<'s, C: ReasonContext> DerivationContext<C> for DerivationContextReasonV1<'s, C> {
        fn time(&self) -> ClockTime {
            self.time
        }

        fn memory(&self) -> &C::Memory {
            self.memory
        }

        fn new_tasks(&self) -> &[C::Task] {
            &self.new_tasks
        }

        fn __new_tasks_mut(&mut self) -> &mut Vec<C::Task> {
            &mut self.new_tasks
        }

        fn current_belief(&self) -> &Option<C::Sentence> {
            &self.current_belief
        }

        fn current_belief_mut(&mut self) -> &mut Option<C::Sentence> {
            &mut self.current_belief
        }

        fn new_stamp(&self) -> &Option<C::Stamp> {
            &self.new_stamp
        }

        fn new_stamp_mut(&mut self) -> &mut Option<C::Stamp> {
            &mut self.new_stamp
        }

        fn new_outputs(&self) -> &[Output] {
            &self.new_outputs
        }

        fn new_outputs_mut(&mut self) -> &mut Vec<Output> {
            &mut self.new_outputs
        }
    }

    /// 实现「推理推导上下文」
    impl<'s, C: ReasonContext> DerivationContextReason<C> for DerivationContextReasonV1<'s, C> {
        fn current_concept(&self) -> &C::Concept {
            self.current_concept
        }

        fn current_concept_mut(&mut self) -> &mut C::Concept {
            self.current_concept
        }

        fn current_task_link(&self) -> &C::TaskLink {
            &self.current_task_link
        }

        fn current_task_link_mut(&mut self) -> &mut <C as ReasonContext>::TaskLink {
            &mut self.current_task_link
        }

        fn current_task(&self) -> &C::Task {
            &self.current_task
        }

        fn current_task_mut(&mut self) -> &mut <C as ReasonContext>::Task {
            &mut self.current_task
        }

        fn current_belief_link(&self) -> &C::TermLink {
            &self.current_belief_link
        }

        fn current_belief_link_mut(&mut self) -> &mut <C as ReasonContext>::TermLink {
            &mut self.current_belief_link
        }
    }

    // TODO: 实现「推理上下文构建者」
}
pub use impl_v1::*;
