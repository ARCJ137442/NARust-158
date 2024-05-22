//! 🎯复刻OpenNARS `nars.entity.Task`
//! * ✅【2024-05-05 21:38:53】基本方法复刻完毕

use super::{BudgetValue, BudgetValueConcrete, Item, Sentence, SentenceConcrete, TruthValue};
use crate::{global::ClockTime, storage::BagKey, ToDisplayAndBrief};
use anyhow::Result;
use nar_dev_utils::join;
use narsese::lexical::Task as LexicalTask;
use std::ops::Deref;

/// 模拟`nars.entity.Task`
/// * ❓【2024-05-12 19:39:20】仍然不明确其中「父任务」「父信念」的数据形式
///   * 📝OpenNARS 3.0.4的文档中使用的是「弱引用」
///
/// TODO: 🏗️【2024-05-10 20:37:04】或许后续考虑直接让[`Task`]要求派生自[`Sentence`]与[`Budget`]？
///
/// # 📄OpenNARS
///
/// A task to be processed, consists of a Sentence and a BudgetValue
pub trait Task: ToDisplayAndBrief {
    /// 绑定的「语句」类型
    ///
    /// ? 【2024-05-05 19:43:16】是要「直接绑定语句」还是「绑定真值、时间戳等，再由其组装成『语句』」
    /// * 🚩【2024-05-05 19:43:42】目前遵循「依赖封闭」的原则，暂还是使用「直接绑定语句」的方式
    type Sentence: SentenceConcrete;

    /// 绑定的「元素id」类型
    /// * 🎯用于实现[`Item`]
    type Key: BagKey;

    /// 绑定的「预算值」类型
    /// * 🚩【2024-05-07 18:53:40】必须限定其「短浮点」类型与[「真值」](Sentence::Truth)一致
    type Budget: BudgetValueConcrete<E = <<Self::Sentence as Sentence>::Truth as TruthValue>::E>;

    /// 🆕获取内部作为引用的「元素id」
    /// * 🎯用于返回引用而非值
    /// * 📌实现者可能需要在内部缓存一个「元素id」而非「直接从『语句』处获取」
    /// * 📌可用于对接[`Sentence::to_key`]
    fn __key(&self) -> &Self::Key;

    /// 模拟`Task.sentence`、`Task.getSentence`
    ///
    /// # 📄OpenNARS
    ///
    /// The sentence of the Task
    fn sentence(&self) -> &Self::Sentence;
    /// 🆕[`Task::sentence`]的可变版本
    /// * 🎯用于自动实现[`Sentence`]
    fn sentence_mut(&mut self) -> &mut Self::Sentence;

    /// 模拟`Task.budget`、`Task.getBudget`
    /// * 📝OpenNARS中的`Task`直接从`Item`中拿到了`Budget`字段
    ///   * 此处为避免与[`Item::budget`]命名冲突，采用内部化命名
    fn __budget(&self) -> &Self::Budget;
    /// 🆕[`Task::budget`]的可变版本
    /// * 🎯用于自动实现[`super::BudgetValue`]
    fn __budget_mut(&mut self) -> &mut Self::Budget;

    /// 模拟`Task.parentTask`、`Task.getParentTask`
    /// * 📝OpenNARS中仅在「直接推理」与「单前提结论」中使用
    ///   * 📄「直接推理」中的用法：识别时间戳「是否重复」⇒优先级沉底
    ///     * `task.getParentTask().getSentence().isJudgment()`
    ///   * 📄「单前提结论」中的用法：「结论」与「父任务」相同⇒不产生结论
    ///     * `if (parentTask != null && newContent.equals(parentTask.getContent())) return;`
    ///   * 🎯总目标：避免【重复推理】
    /// * 🚩【2024-05-22 16:40:04】故仅需只读，且与「是否需要共享引用」无关
    ///   * ✅最后只要是一个「可以被解引用」的返回值就行
    ///
    /// # 📄OpenNARS
    ///
    /// Task from which the Task is derived, or null if input
    fn parent_task(&self) -> Option<impl Deref<Target = Self>>;

    /// 模拟`Task.parentBelief`、`Task.getParentBelief`
    /// * 🚩【2024-05-05 20:51:48】目前对「共享引用」使用「引用计数」处理
    /// * 🚩【2024-05-17 16:13:50】目前先改回「可空非共享引用」形式
    ///   TODO: 后续仍然要根据实际情况调整：指向「记忆区」中「任务」的唯一链接
    /// * 📝OpenNARS只在「本地规则/问答`trySolution`」中使用
    ///   * 💭实质上是为「本地规则」服务的
    ///   * 📌上游：`LocalRules.trySolution`、`LocalRules.match`/`Concept.processJudgement`/`Concept.processQuestion`
    ///   * 📌下游：`Memory.activatedTask`、`new Task(..., solution)`
    ///
    /// # 📄OpenNARS
    ///
    /// Belief from which the Task is derived, or null if derived from a theorem
    fn parent_belief(&self) -> &Option<Self::Sentence>;
    /// [`Task::parent_belief`]的可变版本
    /// * 📌只能修改「指向哪个[`Sentence`]」，不能修改所指向[`Sentence`]内部的数据
    /// * 📝OpenNARS中的用法是「一旦构造，不再改变」
    ///   TODO: 后续可能不再需要此字段
    fn parent_belief_mut(&mut self) -> &mut Option<Self::Sentence>;

    /// 模拟`Task.bestSolution`
    /// * 🚩【2024-05-05 20:51:48】目前对「共享引用」使用「引用计数」处理
    ///
    /// # 📄OpenNARS
    ///
    /// For Question and Goal: best solution found so far
    fn best_solution(&self) -> &Option<Self::Sentence>;
    /// [`Task::best_solution`]的可变版本
    /// * 📌只能修改「指向哪个[`Sentence`]」，不能修改所指向[`Sentence`]内部的数据
    fn best_solution_mut(&mut self) -> &mut Option<Self::Sentence>;

    // * ✅`getContent`、`getCreationTime`均已通过「自动实现」被自动模拟

    /// 模拟`Task.isInput`
    ///
    /// # 📄OpenNARS
    ///
    /// Check if a Task is a direct input
    ///
    /// @return Whether the Task is derived from another task
    #[inline(always)]
    fn is_input(&self) -> bool {
        /* 📄OpenNARS源码：
        return parentTask == null; */
        self.parent_task().is_none()
    }

    // * ✅`merge`已通过「自动实现」被自动模拟

    /// 模拟`Task.toString`
    /// * 🚩【2024-05-08 23:56:19】现在借道[`ToDisplayAndBrief`]予以实现
    /// * 🚩🆕【2024-05-11 00:20:54】现在不完全按OpenNARS的来
    ///   * 🚩全`toString`就全`toString`
    ///   * 🚩全`toStringBrief`就全`toStringBrief`
    ///   * ...
    ///
    /// # 📄OpenNARS
    ///
    /// Get a String representation of the Task
    ///
    /// @return The Task as a String
    fn __to_display(&self) -> String
    where
        Self: Sized,
    {
        /* 📄OpenNARS源码：
        StringBuilder s = new StringBuilder();
        s.append(super.toString()).append(" ");
        s.append(getSentence().getStamp());
        if (parentTask != null) {
            s.append("  \n from task: ").append(parentTask.toStringBrief());
            if (parentBelief != null) {
                s.append("  \n from belief: ").append(parentBelief.toStringBrief());
            }
        }
        if (bestSolution != null) {
            s.append("  \n solution: ").append(bestSolution.toStringBrief());
        }
        return s.toString(); */
        join!(
            // 首先作为一个「Item」
            => Item::__to_display(self)
            => ' '
            // 时间戳
            => self.stamp().to_display()
            // 下面这些反正格式化会构造新字符串，为了直观均直接用`format!`代表
            => {# "\n from task: {}" in parent_task.to_display()}
                if let Some(parent_task) = self.parent_task()
            // * 🚩🆕【2024-05-09 00:50:41】此处不采用嵌套：都可能有
            => {# "\n from belief: {}" in parent_belief.to_display()}
                if let Some(parent_belief) = self.parent_belief()
            => {# "\n solution: {}" in best_solution.to_display()}
                if let Some(best_solution) = self.best_solution()
        )
    }

    /// 🆕[`Task::__to_display`]的全简略版本
    fn __to_display_brief(&self) -> String
    where
        Self: Sized,
    {
        join!(
            // 首先作为一个「Item」
            => Item::__to_display_brief(self)
            => ' '
            // 时间戳
            => self.stamp().to_display_brief()
            // 下面这些反正格式化会构造新字符串，为了直观均直接用`format!`代表
            => {# "\n from task: {}" in parent_task.to_display_brief()}
                if let Some(parent_task) = self.parent_task()
            // * 🚩🆕【2024-05-09 00:50:41】此处不采用嵌套：都可能有
            => {# "\n from belief: {}" in parent_belief.to_display_brief()}
                if let Some(parent_belief) = self.parent_belief()
            => {# "\n solution: {}" in best_solution.to_display_brief()}
                if let Some(best_solution) = self.best_solution()
        )
    }

    /// 🆕[`Task::__to_display`]的详尽版本
    fn __to_display_long(&self) -> String
    where
        Self: Sized,
    {
        join!(
            // 首先作为一个「Item」
            => Item::__to_display_long(self)
            => ' '
            // * ✅无需额外添加时间戳（详尽版「语句」已有）
            // 下面这些反正格式化会构造新字符串，为了直观均直接用`format!`代表
            => {# "\n from task: {}" in parent_task.to_display_long()}
                if let Some(parent_task) = self.parent_task()
            // * 🚩🆕【2024-05-09 00:50:41】此处不采用嵌套：都可能有
            => {# "\n from belief: {}" in parent_belief.to_display_long()}
                if let Some(parent_belief) = self.parent_belief()
            => {# "\n solution: {}" in best_solution.to_display_long()}
                if let Some(best_solution) = self.best_solution()
        )
    }
}

pub trait TaskConcrete: Task + Clone + Sized {
    /// 🆕模拟`new Task(Sentence s, BudgetValue b, Task parentTask, Sentence parentBelief, Sentence solution)`
    /// * 🚩完全参数的构造函数
    /// * 🚩【2024-05-08 11:21:58】函数签名与[`Self::from_activate`]相同，但语义并不相似
    ///   * ⚠️私有性：该函数本身应该是【更为内部】【不应被外界直接调用】的
    fn __new(
        sentence: Self::Sentence,
        budget: Self::Budget,
        parent_task: Option<Self>,
        parent_belief: Option<Self::Sentence>,
        solution: Option<Self::Sentence>,
    ) -> Self;

    /// 模拟`new Task(Sentence s, BudgetValue b)`
    ///
    /// # 📄OpenNARS
    ///
    /// Constructor for input task
    ///
    /// @param s The sentence
    /// @param b The budget
    #[inline(always)]
    fn from_input(sentence: Self::Sentence, budget: Self::Budget) -> Self {
        Self::__new(sentence, budget, None, None, None)
    }

    /// 模拟`new Task(Sentence s, BudgetValue b, Task parentTask, Sentence parentBelief)`
    /// * 🚩【2024-05-08 14:33:40】锁定保持[`Option`]：不能再假定为[`Some`]了
    ///   * 📄参见[`crate::storage::Memory::single_premise_task`]
    ///
    /// # 📄OpenNARS
    ///
    /// Constructor for a derived task
    ///
    /// @param s            The sentence
    /// @param b            The budget
    /// @param parentTask   The task from which this new task is derived
    /// @param parentBelief The belief from which this new task is derived
    #[inline(always)]
    fn from_derive(
        sentence: Self::Sentence,
        budget: Self::Budget,
        parent_task: Option<Self>,
        parent_belief: Option<Self::Sentence>,
    ) -> Self {
        Self::__new(sentence, budget, parent_task, parent_belief, None)
    }

    /// 模拟`new Task(Sentence s, BudgetValue b, Task parentTask, Sentence parentBelief, Sentence solution)`
    ///
    /// # 📄OpenNARS
    ///
    /// Constructor for an activated task
    ///
    /// @param s            The sentence
    /// @param b            The budget
    /// @param parentTask   The task from which this new task is derived
    /// @param parentBelief The belief from which this new task is derived
    /// @param solution     The belief to be used in future inference
    fn from_activate(
        sentence: Self::Sentence,
        budget: Self::Budget,
        parent_task: Self,
        parent_belief: Self::Sentence,
        solution: Self::Sentence,
    ) -> Self {
        /* 📄OpenNARS源码：
        this(s, b, parentTask, parentBelief);
        this.bestSolution = solution; */
        let mut this = Self::from_derive(sentence, budget, Some(parent_task), Some(parent_belief));
        *this.best_solution_mut() = Some(solution.clone());
        this // ? 【2024-05-08 11:14:29】💭是否可以直接使用`Self::new`而无需再赋值
             // TODO: 🏗️【2024-05-08 11:15:12】日后在「有足够单元测试」的环境下精简
    }

    /// 🆕从「词法Narsese」中折叠
    /// * 🎯词法折叠；字符串解析器
    /// * 📌附带所有来自「记忆区」「时钟」「真值」「预算值」的超参数
    fn from_lexical(
        lexical: LexicalTask,
        truth_default_values: [<<Self::Sentence as Sentence>::Truth as TruthValue>::E; 2],
        budget_default_values: [<Self::Budget as BudgetValue>::E; 3],
        truth_is_analytic: bool,
        stamp_current_serial: ClockTime,
        stamp_time: ClockTime,
        sentence_revisable: bool,
    ) -> Result<Self> {
        // 直接解构
        let LexicalTask { budget, sentence } = lexical;
        // 语句
        let sentence = <Self::Sentence as SentenceConcrete>::from_lexical(
            sentence,
            truth_default_values,
            truth_is_analytic,
            stamp_current_serial,
            stamp_time,
            sentence_revisable,
        )?;
        // 预算值
        let budget =
            <Self::Budget as BudgetValueConcrete>::from_lexical(budget, budget_default_values)?;
        // 构造
        Ok(Self::from_input(sentence, budget))
    }

    /// 🆕自身到「词法」的转换
    /// * 🎯标准Narsese输出需要（Narsese内容）
    fn to_lexical(&self) -> LexicalTask {
        LexicalTask {
            sentence: self.sentence().to_lexical(),
            budget: self.budget().to_lexical(),
        }
    }
}

/// 自动实现「语句」
/// * ✅同时自动实现「时间戳」[`super::Stamp`]
impl<T: Task> Sentence for T {
    type Truth = <<Self as Task>::Sentence as Sentence>::Truth;
    type Stamp = <<Self as Task>::Sentence as Sentence>::Stamp;

    #[inline(always)]
    fn content(&self) -> &crate::language::Term {
        self.sentence().content()
    }

    #[inline(always)]
    fn content_mut(&mut self) -> &mut crate::language::Term {
        self.sentence_mut().content_mut()
    }

    #[inline(always)]
    fn punctuation(&self) -> &super::SentenceType<Self::Truth> {
        self.sentence().punctuation()
    }

    #[inline(always)]
    fn punctuation_mut(&mut self) -> &mut super::SentenceType<Self::Truth> {
        self.sentence_mut().punctuation_mut()
    }

    #[inline(always)]
    fn stamp(&self) -> &Self::Stamp {
        self.sentence().stamp()
    }

    #[inline(always)]
    fn stamp_mut(&mut self) -> &mut Self::Stamp {
        self.sentence_mut().stamp_mut()
    }

    #[inline(always)]
    fn revisable(&self) -> bool {
        self.sentence().revisable()
    }
}

/// 自动实现「Item」
/// * ✅同时自动实现「预算值」[`super::BudgetValue`]
impl<T: Task> Item for T {
    type Key = <Self as Task>::Key;
    type Budget = <Self as Task>::Budget;

    #[inline(always)]
    fn key(&self) -> &Self::Key {
        self.__key()
    }

    #[inline(always)]
    fn budget(&self) -> &Self::Budget {
        self.__budget()
    }

    #[inline(always)]
    fn budget_mut(&mut self) -> &mut Self::Budget {
        self.__budget_mut()
    }
}

/// 初代实现
mod impl_v1 {
    use super::*;
    use crate::{
        __impl_to_display_and_display,
        global::{RefCount, RC},
        storage::BagKeyV1,
    };
    use std::fmt::Debug;

    /// [`Task`]的初代实现
    /// * ❌【2024-05-22 16:43:35】因`RC`不支持[`Hash`]，故放弃自动派生[`Hash`]
    #[derive(Debug, Clone, PartialEq, Eq)]
    pub struct TaskV1<S, K, B>
    where
        S: SentenceConcrete,
        K: BagKey,
        B: BudgetValueConcrete<E = <S::Truth as TruthValue>::E>,
    {
        sentence: S,
        key: K,
        budget: B,
        parent_task: Option<RC<Self>>,
        parent_belief: Option<S>,
        best_solution: Option<S>,
    }

    // * 🚩自动实现`ToDisplayAndBrief`
    __impl_to_display_and_display! {
        // * 🚩【2024-05-11 00:24:05】现在是全三个版本
        @(to_display; to_display_brief; to_display_long)
        {S, K, B}
        TaskV1<S, K, B> as Task
        where
            S: SentenceConcrete,
            K: BagKey,
            B: BudgetValueConcrete<E = <S::Truth as TruthValue>::E>,
    }

    /// 逐个字段实现
    impl<S, K, B> Task for TaskV1<S, K, B>
    where
        S: SentenceConcrete,
        K: BagKey,
        B: BudgetValueConcrete<E = <S::Truth as TruthValue>::E>,
    {
        type Sentence = S;
        type Key = K;
        type Budget = B;

        #[inline(always)]
        fn __key(&self) -> &Self::Key {
            &self.key
        }

        #[inline(always)]
        fn sentence(&self) -> &Self::Sentence {
            &self.sentence
        }

        #[inline(always)]
        fn sentence_mut(&mut self) -> &mut Self::Sentence {
            &mut self.sentence
        }

        #[inline(always)]
        fn __budget(&self) -> &Self::Budget {
            &self.budget
        }

        #[inline(always)]
        fn __budget_mut(&mut self) -> &mut Self::Budget {
            &mut self.budget
        }

        #[inline(always)]
        fn parent_task(&self) -> Option<impl Deref<Target = Self>> {
            self.parent_task.as_ref().map(RefCount::get_)
        }

        #[inline(always)]
        fn parent_belief(&self) -> &Option<Self::Sentence> {
            &self.parent_belief
        }

        #[inline(always)]
        fn parent_belief_mut(&mut self) -> &mut Option<Self::Sentence> {
            &mut self.parent_belief
        }

        #[inline(always)]
        fn best_solution(&self) -> &Option<Self::Sentence> {
            &self.best_solution
        }

        #[inline(always)]
        fn best_solution_mut(&mut self) -> &mut Option<Self::Sentence> {
            &mut self.best_solution
        }
    }

    /// 直接实现
    impl<S, B> TaskConcrete for TaskV1<S, BagKeyV1, B>
    where
        S: SentenceConcrete,
        S::Truth: Debug,
        B: BudgetValueConcrete<E = <S::Truth as TruthValue>::E>,
    {
        #[inline(always)]
        fn __new(
            s: Self::Sentence,
            b: Self::Budget,
            parent_task: Option<Self>,
            parent_belief: Option<Self::Sentence>,
            solution: Option<Self::Sentence>,
        ) -> Self {
            Self {
                key: s.to_key_string(),
                sentence: s,
                budget: b,
                parent_task: parent_task.map(RC::new_),
                parent_belief,
                best_solution: solution,
            }
        }
    }
}
pub use impl_v1::*;

/// 单元测试
#[cfg(test)]
mod tests {
    use nar_dev_utils::macro_once;
    use narsese::conversion::string::impl_lexical::format_instances::FORMAT_ASCII;

    use super::*;
    use crate::{
        entity::{
            BudgetV1, SentenceType, SentenceV1, ShortFloat, ShortFloatV1, StampConcrete, StampV1,
            TruthV1, TruthValueConcrete,
        },
        global::tests::AResult,
        language::Term,
        ok,
        storage::BagKeyV1,
        test_term,
    };

    /// 测试用具体类型
    type T = TaskV1<SentenceV1<TruthV1, StampV1>, BagKeyV1, BudgetV1>;

    /// 短浮点简写别名
    type SF = ShortFloatV1;

    /// 测试用默认值/真值
    fn truth_default_values() -> [ShortFloatV1; 2] {
        [SF::from_float(1.0), SF::from_float(0.9)]
    }

    /// 测试用默认值/预算值
    fn budget_default_values() -> [ShortFloatV1; 3] {
        [
            SF::from_float(0.8),
            SF::from_float(0.8),
            SF::from_float(0.8),
        ]
    }

    /// 测试用默认值/当前序列（发生时间）
    const CURRENT_SERIAL_DEFAULT: ClockTime = 0;

    /// 测试用默认值/可修订
    const REVISABLE_DEFAULT: bool = true;

    /// 测试用默认值/是否为「分析真值」
    const IS_ANALYTIC_DEFAULT: bool = false;

    /// 快捷构造宏
    /// * 🚩使用「变量遮蔽」的方式，允许「可选参数」的出现
    ///   * 📌虽然这里的「可选参数」仍然需要排序
    macro_rules! l_task {
        (
            // 主参数：文本
            $text:expr $(;
            // 可选参数
            $(current_serial = $current_serial:expr , )?
            $(time = $time:expr , )?
            $(is_analytic = $is_analytic:expr , )?
            $(revisable = $revisable:expr , )?
            $(truth_default_values = $truth_default_values:expr , )?
            $(budget_default_values = $budget_default_values:expr , )? )?
        ) => {{
            let lexical = FORMAT_ASCII.parse($text)?.try_into_task_compatible()?;
            // current_serial
            let current_serial = CURRENT_SERIAL_DEFAULT;
            $( let current_serial = $current_serial; )?
            // time
            let time = CURRENT_SERIAL_DEFAULT;
            $( let time = $time; )?
            // is_analytic
            let is_analytic = IS_ANALYTIC_DEFAULT;
            $( let is_analytic = $is_analytic; )?
            // revisable
            let revisable = REVISABLE_DEFAULT;
            $( let revisable = $revisable; )?
            // truth_default_values
            let truth_default_values = truth_default_values();
            $( let truth_default_values = $truth_default_values; )?
            // budget_default_values
            let budget_default_values = budget_default_values();
            $( let budget_default_values = $budget_default_values; )?
            T::from_lexical(lexical, truth_default_values, budget_default_values, is_analytic, current_serial, time, revisable)?
        }};
    }

    // * ✅测试/new 已在后续函数中测试

    /// 测试/from_input
    /// * 🎯顺带测试「展示类函数」是否正常运行（不检验展示结果）
    #[test]
    fn from_input() -> AResult {
        /// ! 本身「简略」模式下「预算值」仍然是「详细」，OpenNARS如此
        ///   * 📄OpenNARS`s.append(super.toString())`
        ///   * 📄[`Task::__to_display`]
        fn show(task: T) {
            println!("BRIEF:   {}", task.to_display_brief());
            println!("NORMAL:  {}", task.to_display());
            println!("LONG:    {}", task.to_display_long());
        }
        // 构造（一行）
        let text = "$0.8; 0.8; 0.8$ A. :|: %1.0; 0.9%";
        let task = l_task!(text);
        // 展示
        show(task);
        // 构造
        let content = test_term!("A");
        let current_serial = 0;
        let stamp = StampV1::with_time(current_serial, 0);
        let is_analytic = false;
        let truth = TruthV1::from_floats(1.0, 0.9, is_analytic);
        let revisable = false;
        let sentence = SentenceV1::new(content, SentenceType::Judgement(truth), stamp, revisable);
        let budget = BudgetV1::from_floats(0.5, 0.5, 0.5);
        let task = T::from_input(sentence, budget);
        // 展示
        show(task);

        // 完成
        ok!()
    }

    /// 测试/`to_display`、`to_display_brief`、`to_display_long`
    /// * 🎯所有OpenNARS相关的「显示」方法
    #[test]
    fn to_display_xxx() -> AResult {
        // TODO: 单元测试完成实质性内容
        fn show(task: T) -> T {
            println!("BRIEF:   {}", task.to_display_brief());
            println!("NORMAL:  {}", task.to_display());
            println!("LONG:    {}", task.to_display_long());
            task
        }
        macro_once! {
            /// * 🚩模式：任务 => 预期
            macro test( $( $task:literal /* => $expected:tt */ )* ) {
                $(
                    show(l_task!($task));
                )*
            }
            "A."
            "A. :|:"
            "A. %0.5; 0.5%"
            "A. :|: %0.5; 0.5%"
            "$0.1; 0.2; 0.3$ A."
            "$0.1; 0.2; 0.3$ A. :|:"
            "$0.1; 0.2; 0.3$ A. %0.5; 0.5%"
            "$0.1; 0.2; 0.3$ A. :|: %0.5; 0.5%"
        }
        // 完成
        ok!()
    }

    // TODO: 继续有关「父任务」「父信念」的内容
}
