//! 🆕「推理上下文」
//! * 🎯承载并迁移OpenNARS「记忆区」中的「临时推理状态」变量组
//! * 📄亦仿自OpenNARS 3.x（3.0.4）`DerivationContext`
//! * 📝【2024-05-12 02:17:38】基础数据结构可以借鉴OpenNARS 1.5.8，但涉及「推理」的部分，建议采用OpenNARS 3.0.4的架构来复刻
//!
//! * ♻️【2024-05-22 02:09:10】基本已按照改版重构，但仍需拆分代码到不同文件中

use crate::{
    entity::Concept,
    global::{ClockTime, Float},
    language::Term,
    types::TypeContext,
};
use navm::output::Output;

/// 「推理上下文」的所有可变字段
/// * 🎯证明「这些字段都不会相互冲突」
/// * ❌【2024-05-19 10:21:40】不能直接反向引用「概念」，会导致「概念」需要[`Sized`]
/// * 📌【2024-05-19 10:27:01】作为后续「直接推理上下文」与「概念推理上下文」的共用基础
/// * ✅【2024-05-19 10:31:03】因为有「推理上下文」统一类型，此处不再需要额外加泛型参数
///
/// ! ⚠️【2024-05-19 10:28:03】后续可能会因为「共享引用」等问题重写其内部字段类型
pub struct DerivationContextFieldsMut<'s, C: TypeContext> {
    // * 🚩推理用变量
    pub current_concept: &'s mut C::Concept,
    pub current_task: &'s mut C::Task,
    pub current_belief: &'s mut Option<C::Sentence>,
    pub new_stamp: &'s mut Option<C::Stamp>,
    // * 🚩输出用变量
    pub new_tasks: &'s mut Vec<C::Task>,
    pub new_outputs: &'s mut Vec<Output>,
}

/// 🆕「推理上下文」
/// * 🎯承载状态变量，解耦「记忆区」
///   * 🚩替代在「真值函数」「预算函数」「推理规则」中的「记忆区」引用
///   * 📌有利于在Rust中实现「数据解耦」
///   * 💭可能经此无需再考虑RC等「共享引用」类型
/// * 🎯实现「开始推理⇒创建上下文⇒具体推理⇒回收上下文」的新「推理过程」
///   * 💭基于「概念+词项链+任务链」的【可并行化】推理
/// * 🚩【2024-05-12 16:09:29】不堪`<Self as XXX>`其扰，要求实现特征[`Sized`]
/// * 🚩【2024-05-17 15:22:46】重要决议：通过「泛型」而非「派生」进行「自动实现」
///   * 📌不堪各种`Self::Type`歧义
///   * 📌**不堪「在其它无『推理上下文』的地方使用『推理上下文』」时「要用大量关联类型对齐」的烦扰
///     * 📄如：`trait T: TypeContext { fn f(context: DerivationContext<ShortFloat=Self::ShortFloat, 【...】>) }`
/// * 🚩【2024-05-19 10:24:55】现在通过「所有可变字段」结构体，绕过「获取多部可变引用」的借用问题
///   * 📌通过「同一结构体中可以同时拥有各部分可变引用而不会产生『重复借用』问题」实现
///   * 📝亦为一种搬迁「Java OOP」的「字段继承」之设计模式
///
/// ## 所有权/可空性 笔记
///
/// * 📝⚠️【2024-05-12 20:22:11】经OpenNARS`long_term_stability.nal`测试，基本确定「空/非空」情形
///   * 📌仅`currentBelief`、`newStamp`两个字段可空
///   * 🚩【2024-05-12 20:22:41】暂时全部使用`Option`+`unwrap`代替
/// * ❓【2024-05-12 20:46:30】目前对「所有权」仍然存疑
///   * 🚩【2024-05-12 20:46:38】暂时按「具备所有权」的形式做
///   * 💭后续可能仍然要换RC，比如「先放回再推理」「引用整体又引用部分」的情形
/// * 🚩【2024-05-21 19:03:46】目前经过对OpenNARS的更进一步研究，对其「可空性」验证如下：
///   * 📝有效字段 @ 直接推理（以`Concept.directProcess`为入口）
///     * currentConcept (& currentTerm)
///     * currentTask
///     * currentBelief? | 用于中途推理
///     * newStamp? | 用于中途推理
///   * 📝有效字段 @ 转换推理（以`RuleTables.transformTask`为入口）
///     * currentConcept (& currentTerm)
///     * currentTask
///     * currentBelief? | 总为空，以免产生更复杂的类型（并且也可能用于「父信念」）
///     * currentTaskLink (+)
///     * newStamp?
///   * 📝有效字段 @ 概念推理（以`RuleTables.reason`为入口）
///     * currentConcept (& currentTerm)
///     * currentTask
///     * currentBelief? | 可能非空
///     * currentTaskLink
///     * currentBeliefLink (+) | 词项链
///     * newStamp?
///   * 🚩因此可如下设计：
///     * [`DerivationContext`] | 非空=当前概念、当前任务，可空=当前信念、新时间戳
///     * [`DerivationContextDirect`]: [`DerivationContext`] | 构造函数
///     * [`CurrentTaskLink`]: [`DerivationContext`] | 当前任务链
///     * [`DerivationContextTransform`]: [`DerivationContext`] + [`CurrentTaskLink`] | 构造函数
///     * [`DerivationContextReason`]: [`CurrentTaskLink`] | 构造函数，非空=当前信念链
///
/// ## 概念设计笔记
///
/// * 💭【2024-05-08 17:21:00】大致方案：
///   * 📌「记忆区」应该作为一个纯粹的「概念/新任务/新近任务 存储器」来使用
///   * 🚩建立「推理上下文」：其中的数据从「记忆区」取出，经过「推理」生成派生任务与信息，最终「归还」记忆区
///   * 🚩原属于「记忆区」的推理过程有关函数（如`cycle`），应该放在更顶层的「Reasoner」即「推理器」中，统一调度
///     * 🚩并且「推理上下文」应该与「记忆区」平级，统一受「推理器」主控调用
pub trait DerivationContext<C: TypeContext>: TypeContext + Sized {
    /// 🆕获取所有可变引用（这些引用互不干扰）
    fn __fields_mut(&mut self) -> DerivationContextFieldsMut<C>;

    /* ---------- Short-term workspace for a single cycle ---------- */

    /// 🆕跟随OpenNARS 3.0.4，要求存储对「记忆区」的引用
    /// * 🚩至于这个「引用」如何存储（带生命周期的内部指针等），可自由决定
    /// * 🎯目前首次用于[「预算推理」](super::BudgetFunctions::__budget_inference)，上游是「组合规则通过词项优先级调整策略」
    /// * 🎯目前仅只读
    fn memory(&self) -> &C::Memory;

    // ! ❌【2024-05-07 21:16:10】不复刻`Memory.exportStrings`：🆕使用`new_outputs`代替之

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
    /// * 🚩现在从「记忆区」迁移至「推理上下文」：实际上与「记忆区」无关
    ///   * 📌【2024-05-12 14:55:34】妥协：不仅会影响「输出」或「输入」，而且仍然影响推理过程
    ///
    #[doc(alias = "get_silence_value")]
    fn silence_value(&self) -> usize;

    /// 🆕简化`self.silence_value() as Float / 100 as Float`逻辑
    /// * 🎯统一表示「音量」的百分比（静音の度）
    #[inline(always)]
    fn silence_percent(&self) -> Float {
        self.silence_value() as Float / 100 as Float
    }

    /* ---------------- 推理状态变量 具体通用参数 ---------------- */

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
    fn current_concept(&self) -> &C::Concept;
    /// [`DerivationContextDirect::current_concept`]的可变版本
    fn current_concept_mut<'s>(&'s mut self) -> &'s mut C::Concept
    where
        C: 's,
    {
        DerivationContext::__fields_mut(self).current_concept
    }

    /// 模拟`Memory.currentTerm`
    /// * 🚩公开读写：因为要被「推理规则」使用
    /// * 🚩【2024-05-18 02:42:19】可简化：始终代表「当前概念之词项」，根本无需可变版本
    ///   * ✅同时也无需作为「状态变量」存储
    ///   * 📄参考[`DerivationContextReason::current_term`]
    ///
    /// # 📄OpenNARS
    ///
    /// The selected Term
    fn current_term<'s>(&'s self) -> &'s Term
    where
        C::Concept: 's,
    {
        self.current_concept().term()
    }
    // /// [`DerivationContextDirect::current_term`]的可变版本
    // fn __current_term_mut(&mut self) -> &mut Option<Term>;

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
    fn current_task(&self) -> &C::Task;
    /// [`DerivationContextDirect::current_task`]的可变版本
    fn current_task_mut<'s>(&'s mut self) -> &mut C::Task
    where
        C: 's,
    {
        DerivationContext::__fields_mut(self).current_task
    }

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
    /// * 🚩【2024-05-19 10:23:01】现在通过「所有可变引用」可将「获取所有可变引用的一部分」作为默认实现
    #[inline(always)]
    fn current_belief_mut<'s>(&'s mut self) -> &mut Option<C::Sentence>
    where
        C: 's,
    {
        DerivationContext::__fields_mut(self).current_belief
    }

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
    /// * 🚩【2024-05-19 10:23:01】现在通过「所有可变引用」可将「获取所有可变引用的一部分」作为默认实现
    #[inline(always)]
    fn new_stamp_mut<'s>(&'s mut self) -> &'s mut Option<C::Stamp>
    where
        C: 's,
    {
        DerivationContext::__fields_mut(self).new_stamp
    }

    /* ---------------- 推理结果缓存与记录 ---------------- */

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
    ///   * 📌在「推理上下文」中只会增加，不会被移除
    /// * 🚩【2024-05-19 10:23:01】现在通过「所有可变引用」可将「获取所有可变引用的一部分」作为默认实现
    #[inline(always)]
    fn __new_tasks_mut<'s>(&'s mut self) -> &'s mut Vec<C::Task>
    where
        C: 's,
    {
        DerivationContext::__fields_mut(self).new_tasks
    }

    /// 🆕新的「推理输出」
    /// * 🚩用于「延迟决定」
    ///   * 📌先在上下文中缓存输出，等到记忆区推理完毕后，再根据其中的结果决定「是否要输出」
    fn new_outputs(&self) -> &[Output];
    /// [`DerivationContext::new_outputs`]的可变版本
    /// * 🚩【2024-05-19 10:23:01】现在通过「所有可变引用」可将「获取所有可变引用的一部分」作为默认实现
    #[inline(always)]
    fn new_outputs_mut<'s>(&'s mut self) -> &'s mut Vec<Output>
    where
        C: 's,
    {
        DerivationContext::__fields_mut(self).new_outputs
    }

    /// 🆕缓存一条「推理输出」
    /// * 📌功能类似OpenNARS`Memory.report`
    #[inline(always)]
    fn report(&mut self, output: Output) {
        self.new_outputs_mut().push(output);
    }
}

/// 「直接推理上下文」的所有可变字段
/// * 🎯证明「这些字段都不会相互冲突」
///
/// ! ⚠️【2024-05-19 10:28:03】后续可能会因为「共享引用」等问题重写其内部字段类型
pub struct DerivationContextDirectFieldsMut<'s, C: TypeContext> {
    // 共有变量 //
    pub context: DerivationContextFieldsMut<'s, C>,
    // 独有变量 //
}

/// 直接推理上下文
/// * 📌只为「具体类型」：必须从构造函数中构建，且此时全空
/// * 🎯面向「直接本地规则」设置
///   * 🎯对标OpenNARS`Concept.directProcess`、`Memory.immediateProcess`与
///   * 📝OpenNARS在`RuleTables.reason`前使用
///   * 📌存储「推理上下文」在「正式开始推导」前**可空**的临时变量
/// * 🚩【2024-05-17 17:19:44】仍然需要独立的字段，只是某些字段可空
///   * 🚩【2024-05-17 17:41:01】目前采用「共同派生自抽象特征」的方式
/// * 📝OpenNARS只对「概念」是「先放回再推理」的；对「词项链」「任务链」都是「先推理再放回」
///   * 💭【2024-05-17 20:38:12】因此可以在「推理上下文」中存储「当前任务链（任务）」与「当前词项链（当前信念）」
/// * 🚩现不再作为「概念推理上下文」的「构建者」存在
///   * 📝目前已经「同义重构」证明：OpenNARS中「直接推理」与「概念推理」之间没有「可空值⇒非空值」的转换需要
///   * 🚩↑上述「可空值→非空值」的转换问题，只需「提前设置变量」即可
pub trait DerivationContextDirect<C: TypeContext>: DerivationContext<C> {
    /* ========独有内容======== */

    /// 🆕获取所有可变引用（这些引用互不干扰）
    ///  * ✅【2024-05-21 19:37:10】暂时无需特别实现：提供「直接内联」的内部实现
    #[inline]
    fn fields_mut(&mut self) -> DerivationContextDirectFieldsMut<'_, C> {
        DerivationContextDirectFieldsMut {
            context: DerivationContext::__fields_mut(self),
        }
    }

    /// 构造函数
    /// * 🎯在「推理周期」进入「入口函数」前构造
    /// * 📌参数需求如下：
    ///   * 记忆区
    ///   * 来自推理器的「超参数」：时间、静默值
    ///   * 当前任务
    ///   * 当前概念
    /// * 📌构造时的状态：
    ///   * 任务：仍然处于「完全所有权传参」状态——整个值在调度中接力传递，最终将全所有权落入「上下文」中
    fn new(
        memory: &C::Memory,
        time: ClockTime,
        silence_value: usize,
        current_task: C::Task,
        current_concept: &mut C::Concept,
    ) -> Self;

    /* ========承接自「推理上下文」======== */

    // ! 🈚【2024-05-21 19:33:51】自OpenNARS摸清「可空性」，现在已将全部字段提取到根部「推理上下文」中，不再需要
}

/// 复用「当前任务链」
/// * 🎯用于同时构建「转换推理上下文」与「概念推理上下文」
pub trait CurrentTaskLink<C: TypeContext> {
    /// 模拟`Memory.currentTaskLink`
    /// * 🚩经OpenNARS`long_term_stability.nal`测试，非空
    /// * 📝经OpenNARS研究，仍需可变版本：仅引用不变，但内部可变
    ///   * 📄溯源：在「推理规则」的「预算推理」中，需要调用各种「调整预算」的方法
    ///
    /// # 📄OpenNARS
    ///
    fn current_task_link(&self) -> &C::TaskLink;
    // * 🚩【2024-05-21 22:18:37】暂不于此提供可变版本（因「没必要再加『可变字段结构体』」无法自动实现），而是在两个「下游具体特征」中提供自动实现
    // * ⚠️对单个字段专门实现个结构体，代码将过度冗余
}

/// 「转换推理上下文」的所有可变字段
/// * 🎯证明「这些字段都不会相互冲突」
///
/// ! ⚠️【2024-05-19 10:28:03】后续可能会因为「共享引用」等问题重写其内部字段类型
pub struct DerivationContextTransformFieldsMut<'s, C: TypeContext> {
    // 共有变量 //
    pub context: DerivationContextFieldsMut<'s, C>,
    // 独有变量 //
    pub current_task_link: &'s mut C::TaskLink,
}

/// 转换推理上下文
/// * 🎯作为「具体类型」与「`RuleTables.reason`中使用的『上下文』类型」
///   * 📌拥有构造函数
///   * 📌拥有与「立即本地推理」中与「可空项」不同的字段：一些原「可空」标记为「不可空」
pub trait DerivationContextTransform<C: TypeContext>:
    DerivationContext<C> + CurrentTaskLink<C>
{
    /* ========独有内容======== */

    /// 🆕获取所有可变引用（这些引用互不干扰）
    fn fields_mut(&mut self) -> DerivationContextTransformFieldsMut<'_, C>;

    /// 构造函数
    /// * 🎯在「推理周期」进入「入口函数」前构造
    /// * 📌参数需求如下：
    ///   * 记忆区
    ///   * 来自推理器的「超参数」：时间、静默值
    ///   * 当前概念
    ///   * 当前任务链（包含 当前任务 as 链接目标）
    /// * 📌构造时的状态：
    ///   * 任务链：已从「概念」中拿出，但并未归还——将在「被推理器回收」时归还
    fn new(
        memory: &C::Memory,
        time: ClockTime,
        silence_value: usize,
        current_concept: &mut C::Concept,
        current_task_link: C::TaskLink,
    ) -> Self;

    /* ========承接自「推理上下文」======== */

    /// [`Memory::current_task_link`]的可变版本
    /// * 🚩【2024-05-19 11:07:35】现在通过「所有可变引用」可将「获取所有可变引用的一部分」作为默认实现
    ///
    #[inline(always)]
    fn current_task_link_mut<'s>(&'s mut self) -> &'s mut C::TaskLink
    where
        C: 's,
    {
        DerivationContextTransform::fields_mut(self).current_task_link
    }
}

/// 「概念推理上下文」的所有可变字段
/// * 🎯证明「这些字段都不会相互冲突」
///
/// ! ⚠️【2024-05-19 10:28:03】后续可能会因为「共享引用」等问题重写其内部字段类型
pub struct DerivationContextReasonFieldsMut<'s, C: TypeContext> {
    // 共有变量 //
    pub context: DerivationContextFieldsMut<'s, C>,
    // 独有变量 //
    pub current_task_link: &'s mut C::TaskLink,
    pub current_belief_link: &'s mut C::TermLink, // * 📌相比「转换推理上下文」新增
}

/// 概念推理上下文
/// * 🎯作为「具体类型」与「`RuleTables.reason`中使用的『上下文』类型」
///   * 📌拥有构造函数
///   * 📌拥有「当前词项链（信念链）」与「当前任务链」两者
pub trait DerivationContextReason<C: TypeContext>:
    DerivationContext<C> + CurrentTaskLink<C>
{
    /* ========独有内容======== */

    /// 🆕获取所有可变引用（这些引用互不干扰）
    fn fields_mut(&mut self) -> DerivationContextReasonFieldsMut<'_, C>;

    /// 构造函数
    /// * 🎯在「推理周期」进入「入口函数」前构造
    /// * 📌参数需求如下：
    ///   * 记忆区
    ///   * 来自推理器的「超参数」：时间、静默值
    ///   * 当前概念
    ///   * 当前任务链（包含 当前任务 as 链接目标）
    /// * 📌构造时的状态：
    ///   * 任务链：已从「概念」中拿出，但并未归还——将在「被推理器回收」时归还
    fn new(
        memory: &C::Memory,
        time: ClockTime,
        silence_value: usize,
        current_concept: &mut C::Concept,
        current_task_link: C::TaskLink,
        current_belief_link: C::TermLink,
    ) -> Self;

    // TODO: 根据新的「推理上下文」更新架构（吸收上下文 等）

    /* ========承接自「推理上下文」======== */

    /// [`Memory::current_task_link`]的可变版本
    /// * 🚩【2024-05-19 11:07:35】现在通过「所有可变引用」可将「获取所有可变引用的一部分」作为默认实现
    ///
    #[inline(always)]
    fn current_task_link_mut<'s>(&'s mut self) -> &'s mut C::TaskLink
    where
        C: 's,
    {
        DerivationContextReason::fields_mut(self).current_task_link
    }

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
    /// * 🚩【2024-05-19 11:07:35】现在通过「所有可变引用」可将「获取所有可变引用的一部分」作为默认实现
    #[inline(always)]
    fn current_belief_link_mut<'s>(&'s mut self) -> &'s mut C::TermLink
    where
        C: 's,
    {
        DerivationContextReason::fields_mut(self).current_belief_link
    }
}

/// 初代实现
///
/// TODO: 【2024-05-21 22:09:32】🚧封存：先做好上层API，再开发底层实现（+单测）
#[allow(unused)]
mod impl_v1 {
    use super::*;

    // /// 初代「概念推理上下文」结构
    // /// * 🚩【2024-05-17 13:37:30】目前均带上引用
    // /// * 🎯【2024-05-12 20:59:09】用在[`RuleTables::reason`]之中
    // ///   * 📌此即为「概念推理」上下文
    // pub struct DerivationContextReasonV1<'s, C: TypeContext> {
    //     /* ========从「构造函数」中来======== */
    //     /// 记忆区
    //     pub memory: &'s C::Memory,
    //     /// 当前时间
    //     pub time: ClockTime,
    //     /// 当前「静默值」
    //     pub silence_value: usize,

    //     /* ========从「直接推理上下文」中来======== */
    //     // * 🚩目前为方便，对此处变量直接使用「带所有权变量」
    //     // TODO: 【2024-05-17 16:11:38】🏗️后续仍然会调整：需要「控制机制」的顶层设计先敲定
    //     //   * ❓如：是否要在【取出概念】的同时做推理，是否要【取出任务】后推理，「词项链」「任务链」如何设计……
    //     // /// 当前词项
    //     // pub current_term: &'s Term,
    //     // * 📝OpenNARS源码
    //     /// 当前概念
    //     pub current_concept: &'s mut C::Concept,
    //     /// 当前任务链
    //     pub current_task_link: C::TaskLink,
    //     /// 当前任务
    //     /// * 🚩【2024-05-18 02:45:04】目前持有所有权：就是从「任务缓冲区」中拿出的
    //     ///
    //     /// TODO: 后续可能还要分「直接推理」与「概念推理」两种情况
    //     pub current_task: C::Task,
    //     /// 当前信念链（词项链）
    //     /// * 🚩【2024-05-18 02:45:04】目前持有所有权：就是从「词项链」中拿出的
    //     pub current_belief_link: C::TermLink,
    //     /// 当前信念
    //     /// * 📌可空：无信念单任务推理
    //     /// * 🚩【2024-05-17 14:49:55】目前持有所有权：特征定义需要
    //     ///   * ⚠️后续仍有可能更改
    //     pub current_belief: Option<C::Sentence>,

    //     /* ========缓存的「推导结果」变量======== */
    //     /// 新派生的任务
    //     pub new_tasks: Vec<C::Task>,
    //     /// 新产生的输出
    //     pub new_outputs: Vec<Output>,
    //     /// 新时间戳
    //     /// * 📌可空：创建与使用距离较远之情况
    //     ///   * 📝OpenNARS中：在推理过程中创建，在「最终得出结论」时使用（构造新语句/新任务）
    //     ///     * 📌但总是不会为`null`：在「语句」中并未检查到构造时`stamp == null`的情况
    //     /// * 🚩【2024-05-17 14:49:33】作为「推理产物」放在「推导结果变量」中
    //     pub new_stamp: Option<C::Stamp>,
    // }

    // /// 初代「直接推理上下文」
    // /// * 🎯用于「直接推理」也用于「构建『概念推理上下文』」
    // pub struct DerivationContextDirectV1<'s, C: TypeContext> {
    //     /* ========从「构造函数」中来======== */
    //     /// 记忆区
    //     pub memory: &'s C::Memory,
    //     /// 当前时间
    //     pub time: ClockTime,
    //     /// 当前「静默值」
    //     pub silence_value: usize,

    //     /* ========从「直接推理上下文」中来======== */
    //     /// 当前词项
    //     pub current_term: Option<&'s Term>,
    //     /// 当前概念
    //     pub current_concept: Option<&'s C::Concept>,
    //     /// 当前任务链
    //     pub current_task_link: Option<&'s C::TaskLink>,
    //     /// 当前任务
    //     pub current_task: Option<&'s C::Task>,
    //     /// 当前信念链（词项链）
    //     pub current_belief_link: Option<&'s C::TermLink>,
    //     /// 当前信念
    //     /// * 📌允许保留空值：无信念单任务推理
    //     /// * 🚩【2024-05-17 14:47:40】目前按值存储：特征需要如此
    //     ///   * ⚠️后续仍有可能按「推理规则」的形式更改
    //     pub current_belief: Option<C::Sentence>,
    //     // ! 无需「新时间戳」：这是在「概念推理」过程中产生的新值

    //     /* ========缓存的「推导结果」变量======== */
    //     /// 新派生的任务
    //     pub new_tasks: Vec<C::Task>,
    //     /// 新产生的输出
    //     pub new_outputs: Vec<Output>,
    //     /// 新时间戳
    //     /// * 📌可空：创建与使用距离较远之情况
    //     ///   * 📝OpenNARS中：在推理过程中创建，在「最终得出结论」时使用（构造新语句/新任务）
    //     ///     * 📌但总是不会为`null`：在「语句」中并未检查到构造时`stamp == null`的情况
    //     /// * 🚩【2024-05-17 14:49:33】作为「推理产物」放在「推导结果变量」中
    //     pub new_stamp: Option<C::Stamp>,
    // }

    // // 批量实现「推理上下文」
    // impl_type_context_from_generics! {
    //     C in ['s, C: TypeContext]
    //     for DerivationContextReasonV1<'s, C> => TypeContext

    //     C in ['s, C: TypeContext]
    //     for DerivationContextDirectV1<'s, C> => TypeContext
    // }

    // /// 对「概念推理上下文」的实现
    // mod impl_reason {
    //     use super::*;

    //     /// 实现「推理上下文」
    //     impl<C: TypeContext> DerivationContext<C> for DerivationContextReasonV1<'_, C> {
    //         fn fields_mut(&mut self) -> DerivationContextFieldsMut<'_, C> {
    //             DerivationContextFieldsMut {
    //                 new_tasks: &mut self.new_tasks,
    //                 new_outputs: &mut self.new_outputs,
    //             }
    //         }

    //         fn time(&self) -> ClockTime {
    //             self.time
    //         }

    //         fn silence_value(&self) -> usize {
    //             self.silence_value
    //         }

    //         fn memory(&self) -> &C::Memory {
    //             self.memory
    //         }

    //         fn new_tasks(&self) -> &[C::Task] {
    //             &self.new_tasks
    //         }

    //         fn new_outputs(&self) -> &[Output] {
    //             &self.new_outputs
    //         }
    //     }

    //     /// 实现「概念推理上下文」
    //     impl<C: TypeContext> DerivationContextReason<C> for DerivationContextReasonV1<'_, C> {
    //         fn fields_mut(&mut self) -> DerivationContextReasonFieldsMut<'_, C> {
    //             DerivationContextReasonFieldsMut {
    //                 // * 🚩作为「推理上下文」的所有字段（必须内联以避免借用整个`self`）
    //                 context: DerivationContextFieldsMut {
    //                     new_tasks: &mut self.new_tasks,
    //                     new_outputs: &mut self.new_outputs,
    //                 },
    //                 // * 🚩↓此处特殊：直接传字段即可
    //                 current_concept: self.current_concept,
    //                 current_task_link: &mut self.current_task_link,
    //                 current_task: &mut self.current_task,
    //                 current_belief_link: &mut self.current_belief_link,
    //                 current_belief: &mut self.current_belief,
    //                 new_stamp: &mut self.new_stamp,
    //             }
    //         }

    //         fn current_concept(&self) -> &C::Concept {
    //             self.current_concept
    //         }

    //         // * 📝下面这些「可变字段实现」若后续更改了字段类型，直接删除即可，莫为此多耗时间
    //         // * 🚩直接使用默认实现
    //         fn current_concept_mut<'s>(&'s mut self) -> &'s mut C::Concept
    //         where
    //             C: 's,
    //         {
    //             self.current_concept
    //         }

    //         fn current_task_link(&self) -> &C::TaskLink {
    //             &self.current_task_link
    //         }

    //         fn current_task_link_mut<'s>(&'s mut self) -> &'s mut <C as TypeContext>::TaskLink
    //         where
    //             C: 's,
    //         {
    //             &mut self.current_task_link
    //         }

    //         fn current_task(&self) -> &C::Task {
    //             &self.current_task
    //         }

    //         fn current_task_mut<'s>(&'s mut self) -> &'s mut <C as TypeContext>::Task
    //         where
    //             C: 's,
    //         {
    //             &mut self.current_task
    //         }

    //         fn current_belief_link(&self) -> &C::TermLink {
    //             &self.current_belief_link
    //         }

    //         fn current_belief_link_mut<'s>(&'s mut self) -> &'s mut <C as TypeContext>::TermLink
    //         where
    //             C: 's,
    //         {
    //             &mut self.current_belief_link
    //         }

    //         fn current_belief(&self) -> &Option<C::Sentence> {
    //             &self.current_belief
    //         }

    //         fn new_stamp(&self) -> &Option<C::Stamp> {
    //             &self.new_stamp
    //         }
    //     }
    // }

    // mod impl_direct {
    //     use super::*;

    //     /// 实现「推理上下文」
    //     impl<C: TypeContext> DerivationContext<C> for DerivationContextDirectV1<'_, C> {
    //         fn fields_mut(&mut self) -> DerivationContextFieldsMut<'_, C> {
    //             DerivationContextFieldsMut {
    //                 new_tasks: &mut self.new_tasks,
    //                 new_outputs: &mut self.new_outputs,
    //             }
    //         }

    //         fn time(&self) -> ClockTime {
    //             self.time
    //         }

    //         fn silence_value(&self) -> usize {
    //             self.silence_value
    //         }

    //         fn memory(&self) -> &C::Memory {
    //             self.memory
    //         }

    //         fn new_tasks(&self) -> &[C::Task] {
    //             &self.new_tasks
    //         }

    //         fn new_outputs(&self) -> &[Output] {
    //             &self.new_outputs
    //         }
    //     }
    // }
}
pub use impl_v1::*;
