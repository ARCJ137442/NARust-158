//! 🎯复刻OpenNARS `nars.inference.BudgetFunctions`

use super::UtilityFunctions;
use crate::{
    entity::{BudgetValue, ShortFloat, TruthValue},
    global::Float,
    language::Term,
};

/// 预算函数
/// * 🚩【2024-05-03 14:48:13】现在仍依照OpenNARS原意「直接创建新值」
///   * 📝本身复制值也没多大性能损耗
///   * 📌「直接创建新值」会更方便后续调用
///     * 📄减少无谓的`.clone()`
pub trait BudgetFunctions: BudgetValue {
    /* ----------------------- Belief evaluation ----------------------- */

    /// 模拟`BudgetFunctions.truthToQuality`
    ///
    /// # 📄OpenNARS
    ///
    /// Determine the quality of a judgment by its truth value alone
    ///
    /// Mainly decided by confidence, though binary judgment is also preferred
    ///
    /// @param t The truth value of a judgment
    /// @return The quality of the judgment, according to truth value only
    fn truth_to_quality(t: &impl TruthValue) -> Self::E {
        /* 📄OpenNARS源码：
        float exp = t.getExpectation();
        return (float) Math.max(exp, (1 - exp) * 0.75); */
        let exp = t.expectation();
        Self::E::from_float(exp.max((1.0 - exp) * 0.75))
    }

    /// 模拟`BudgetFunctions.rankBelief`
    /// * 🚩🆕【2024-05-03 21:46:17】仅传入「语句」中的「真值」与「时间戳长度」，而非「语句」本身
    ///   * 🚩`judgment.getTruth()` => `truth`
    ///   * 🚩`judgment.getStamp().length()` => `stamp_len`
    /// * 📝在使用该函数返回值的地方，仅为「比较大小」
    ///   * 但[`Self::E`]已经实现了[`Ord`]并且需要[`UtilityFunctions::or`]
    ///
    /// # 📄OpenNARS
    ///
    /// Determine the rank of a judgment by its quality and originality (stamp length), called from Concept
    ///
    /// @param judgment The judgment to be ranked
    /// @return The rank of the judgment, according to truth value only
    fn rank_belief(truth: &impl TruthValue<E = Self::E>, stamp_len: usize) -> Self::E {
        /* 📄OpenNARS源码：
        float confidence = judgment.getTruth().getConfidence();
        float originality = 1.0f / (judgment.getStamp().length() + 1);
        return or(confidence, originality); */
        let confidence = truth.confidence();
        let originality = Self::E::from_float(1.0 / (stamp_len as Float + 1.0));
        confidence | originality
    }

    /* ----- Functions used both in direct and indirect processing of tasks ----- */

    // TODO: solutionEval | 涉及「语句」
    /// 模拟`BudgetFunctions.solutionEval`
    /// * 🚩🆕【2024-05-04 00:21:53】仍然是脱离有关「记忆区」「词项链」「任务」等「附加点」的
    ///   * ❓后续是不是又要做一次「参数预装填」
    /// * ❓这个似乎涉及到「本地规则」的源码
    ///   * 💫TODO: 到底实际上该不该放这儿（不应该放本地规则去吗？）
    /// * 📝似乎的确只出现在「本地规则」的`trySolution`方法中
    ///   * 💫并且那个方法还要修改记忆区「做出回答」，错综复杂
    /// * 🚩【2024-05-04 00:25:17】暂时搁置
    ///
    /// # 📄OpenNARS
    ///
    /// Evaluate the quality of a belief as a solution to a problem, then reward
    /// the belief and de-prioritize the problem
    ///
    /// @param problem  The problem (question or goal) to be solved
    /// @param solution The belief as solution
    /// @param task     The task to be immediately processed, or null for continued
    ///                 process
    /// @return The budget for the new task which is the belief activated, if
    ///         necessary
    fn solution_eval(
        problem_solution_quality: Self::E, // * 🚩对标`LocalRules.solutionQuality(problem, solution);`
        solution_truth: &impl TruthValue<E = Self::E>, // * 🚩对标`solution.getTruth()`
        task_feedback_to_links: bool,      // * 🚩对标`feedbackToLinks`
        task_sentence_is_judgment: bool,   // * 🚩对标`task.getSentence().isJudgment()`
        task_budget: &mut Self,            // * 🚩对标`task`（在判断完「是否为判断」之后）
        memory_current_task_link_budget: &mut Self, // * 🚩对标`memory.currentTaskLink`
        memory_current_belief_link_budget: &mut Self, // * 🚩对标`memory.currentBeliefLink`
    ) -> Option<Self> {
        /* 📄OpenNARS源码：
        BudgetValue budget = null;
        boolean feedbackToLinks = false;
        if (task == null) { // called in continued processing
            task = memory.currentTask;
            feedbackToLinks = true;
        }
        boolean judgmentTask = task.getSentence().isJudgment();
        float quality = LocalRules.solutionQuality(problem, solution);
        if (judgmentTask) {
            task.incPriority(quality);
        } else {
            float taskPriority = task.getPriority();
            budget = new BudgetValue(or(taskPriority, quality), task.getDurability(),
                    truthToQuality(solution.getTruth()));
            task.setPriority(Math.min(1 - quality, taskPriority));
        }
        if (feedbackToLinks) {
            TaskLink tLink = memory.currentTaskLink;
            tLink.setPriority(Math.min(1 - quality, tLink.getPriority()));
            TermLink bLink = memory.currentBeliefLink;
            bLink.incPriority(quality);
        }
        return budget; */
        let mut budget = None;
        let feedback_to_links = task_feedback_to_links;
        // ! 【2024-05-04 00:40:21】跳过对task的「空值判定」和「判断句判定」
        // * 💭相当于将一些「需要使用高级功能」的「判定逻辑」交给调用方了
        let quality = problem_solution_quality;
        if task_sentence_is_judgment {
            task_budget.inc_priority(problem_solution_quality);
        } else {
            let task_priority = task_budget.priority();
            budget = Some(Self::new(
                task_priority | quality,
                task_budget.durability(),
                Self::truth_to_quality(solution_truth),
            ));
        }
        if feedback_to_links {
            let t_link = memory_current_task_link_budget;
            t_link.set_priority(t_link.priority().min(!quality));
            let b_link = memory_current_belief_link_budget;
            b_link.inc_priority(quality);
        }
        budget
    }

    /// 模拟`BudgetFunctions.revise`
    ///
    /// # 📄OpenNARS
    ///
    /// Evaluate the quality of a revision, then de-prioritize the premises
    ///
    /// @param tTruth The truth value of the judgment in the task
    /// @param bTruth The truth value of the belief
    /// @param truth  The truth value of the conclusion of revision
    /// @return The budget for the new task
    fn revise(
        t_truth: &impl TruthValue<E = Self::E>,
        b_truth: &impl TruthValue<E = Self::E>,
        truth: &impl TruthValue<E = Self::E>,
        feedback_to_links: bool,
        memory_current_task_budget: &mut Self,
        memory_current_task_link_budget: &mut Self,
        memory_current_belief_link_budget: &mut Self,
    ) -> Self {
        /* 📄OpenNARS源码：
        float difT = truth.getExpDifAbs(tTruth);
        Task task = memory.currentTask;
        task.decPriority(1 - difT);
        task.decDurability(1 - difT);
        if (feedbackToLinks) {
            TaskLink tLink = memory.currentTaskLink;
            tLink.decPriority(1 - difT);
            tLink.decDurability(1 - difT);
            TermLink bLink = memory.currentBeliefLink;
            float difB = truth.getExpDifAbs(bTruth);
            bLink.decPriority(1 - difB);
            bLink.decDurability(1 - difB);
        }
        float dif = truth.getConfidence() - Math.max(tTruth.getConfidence(), bTruth.getConfidence());
        float priority = or(dif, task.getPriority());
        float durability = aveAri(dif, task.getDurability());
        float quality = truthToQuality(truth);
        return new BudgetValue(priority, durability, quality); */
        let dif_t = Self::E::from_float(truth.expectation_abs_dif(t_truth));
        let task = memory_current_task_budget;
        task.dec_priority(!dif_t);
        task.dec_durability(!dif_t);
        if feedback_to_links {
            let t_link = memory_current_task_link_budget;
            t_link.dec_priority(!dif_t);
            t_link.dec_durability(!dif_t);
            let b_link = memory_current_belief_link_budget;
            let dif_b = Self::E::from_float(truth.expectation_abs_dif(b_truth));
            b_link.dec_priority(!dif_b);
            b_link.dec_durability(!dif_b);
        }
        let dif = truth.confidence() - t_truth.confidence().max(b_truth.confidence());
        let priority = dif | task.priority();
        let durability = Self::E::arithmetical_average([dif, task.durability()]);
        let quality = Self::truth_to_quality(truth);
        Self::new(priority, durability, quality)
    }

    /// 模拟`BudgetFunctions.update`
    ///
    /// # 📄OpenNARS
    ///
    /// Update a belief
    ///
    /// @param task   The task containing new belief
    /// @param bTruth Truth value of the previous belief
    /// @return Budget value of the updating task
    fn update(
        task_truth: &impl TruthValue<E = Self::E>,
        task_budget: &mut Self,
        b_truth: &impl TruthValue<E = Self::E>,
    ) -> Self {
        /* 📄OpenNARS源码：
        TruthValue tTruth = task.getSentence().getTruth();
        float dif = tTruth.getExpDifAbs(bTruth);
        float priority = or(dif, task.getPriority());
        float durability = aveAri(dif, task.getDurability());
        float quality = truthToQuality(bTruth);
        return new BudgetValue(priority, durability, quality); */
        let t_truth = task_truth;
        let dif = Self::E::from_float(t_truth.expectation_abs_dif(b_truth));
        let priority = dif | task_budget.priority();
        let durability = Self::E::arithmetical_average([dif, task_budget.durability()]);
        let quality = Self::truth_to_quality(t_truth);
        Self::new(priority, durability, quality)
    }

    /* ----------------------- Links ----------------------- */

    /// 模拟`BudgetFunctions.distributeAmongLinks`
    ///
    /// # 📄OpenNARS
    /// Distribute the budget of a task among the links to it
    ///
    /// @param b The original budget
    /// @param n Number of links
    /// @return Budget value for each link
    fn distribute_among_links(&self, n: usize) -> Self {
        /* 📄OpenNARS源码：
        float priority = (float) (b.getPriority() / Math.sqrt(n));
        return new BudgetValue(priority, b.getDurability(), b.getQuality()); */
        let priority = self.priority().to_float() / (n as Float).sqrt();
        Self::new(
            Self::E::from_float(priority),
            self.durability(),
            self.quality(),
        )
    }

    /* ----------------------- Concept ----------------------- */

    /// 模拟`BudgetFunctions.activate`
    /// * 🚩【2024-05-02 20:55:40】虽然涉及「概念」，但实际上只用到了「概念作为预算值的部分」
    /// * 📌【2024-05-02 20:56:11】目前要求「概念」一方使用同样的「短浮点」
    /// * 🚩【2024-05-03 14:58:03】此处是「修改」语义
    /// * ⚠️参数顺序和OpenNARS仍然保持相同：`self`指代其中的`concept`参数
    ///
    /// # 📄OpenNARS
    ///
    /// Activate a concept by an incoming TaskLink
    ///
    /// @param concept The concept
    /// @param budget  The budget for the new item
    fn activate<B>(&mut self, budget: &impl BudgetValue<E = Self::E>) {
        /* 📄OpenNARS源码：
        float oldPri = concept.getPriority();
        float priority = or(oldPri, budget.getPriority());
        float durability = aveAri(concept.getDurability(), budget.getDurability());
        float quality = concept.getQuality();
        concept.setPriority(priority);
        concept.setDurability(durability);
        concept.setQuality(quality); */
        let old_pri = self.priority();
        let priority = old_pri | budget.priority();
        let durability = Self::E::arithmetical_average([self.durability(), budget.durability()]);
        // let quality = self.quality(); // ! 这俩不变，可以抵消
        self.set_priority(priority);
        self.set_durability(durability);
        // self.set_quality(quality) // ! 这俩不变，可以抵消
    }

    /* ---------------- Bag functions, on all Items ------------------- */

    /// 模拟`BudgetFunctions.forget`
    /// * 🚩【2024-05-03 14:57:06】此处是「修改」语义，而非「创建新值」语义
    ///
    /// # 📄OpenNARS
    ///
    /// Decrease Priority after an item is used, called in Bag
    ///
    /// After a constant time, p should become d*p. Since in this period, the
    /// item is accessed c*p times, each time p-q should multiple d^(1/(c*p)).
    /// The intuitive meaning of the parameter "forgetRate" is: after this number
    /// of times of access, priority 1 will become d, it is a system parameter
    /// adjustable in run time.
    ///
    /// @param budget            The previous budget value
    /// @param forgetRate        The budget for the new item
    /// @param relativeThreshold The relative threshold of the bag
    fn forget(&mut self, forget_rate: Float, relative_threshold: Float) {
        /* 📄OpenNARS源码：
        double quality = budget.getQuality() * relativeThreshold; // re-scaled quality
        double p = budget.getPriority() - quality; // priority above quality
        if (p > 0) {
            quality += p * Math.pow(budget.getDurability(), 1.0 / (forgetRate * p));
        } // priority Durability
        budget.setPriority((float) quality); */
        let mut quality = self.quality().to_float() * relative_threshold; // 重新缩放「质量」
        let p = self.priority().to_float() - quality; // 「质量」之上的「优先级」
        if p > 0.0 {
            quality += p * p.powf(1.0 / (forget_rate * p));
        } // priority Durability
        self.set_priority(Self::E::from_float(quality));
    }

    /// 模拟`BudgetValue.merge`，亦与`BudgetFunctions.merge`相同
    /// * 📝【2024-05-03 14:55:29】虽然现在「预算函数」以「直接创建新值」为主范式，
    ///   * 但在用到该函数的`merge`方法上，仍然是「修改」语义——需要可变引用
    ///
    /// # 📄OpenNARS
    ///
    /// ## `BudgetValue`
    ///
    /// Merge one BudgetValue into another
    ///
    /// ## `BudgetFunctions`
    ///
    /// Merge an item into another one in a bag, when the two are identical
    /// except in budget values
    ///
    /// @param baseValue   The budget value to be modified
    /// @param adjustValue The budget doing the adjusting
    fn merge(&mut self, other: &impl BudgetValue<E = Self::E>) {
        // * 🚩【2024-05-02 00:16:50】仅作参考，后续要移动到「预算函数」中
        /* OpenNARS源码 @ BudgetFunctions.java：
        baseValue.setPriority(Math.max(baseValue.getPriority(), adjustValue.getPriority()));
        baseValue.setDurability(Math.max(baseValue.getDurability(), adjustValue.getDurability()));
        baseValue.setQuality(Math.max(baseValue.getQuality(), adjustValue.getQuality())); */
        // 🆕此处就是三者的最大值，并且从右边合并到左边
        // ! ❓是否要就此调用可变引用
        self.__priority_mut().max_from(other.priority());
        self.__durability_mut().max_from(other.durability());
        self.__quality_mut().max_from(other.quality());
    }

    /* ----- Task derivation in LocalRules and SyllogisticRules ----- */

    /// 模拟`BudgetFunctions.forward`
    ///
    /// # 📄OpenNARS
    ///
    /// Forward inference result and adjustment
    ///
    /// @param truth The truth value of the conclusion
    /// @return The budget value of the conclusion
    fn forward(
        truth: &impl TruthValue<E = Self::E>,
        // * 🚩↓对标`Item t`（来自`memory`）
        memory_t_budget: &impl BudgetValue<E = Self::E>,
        // * 🚩↓对标两个变量，其中一个在另一个变量的条件里边，因此为「保证同时存在」将其合并到一个变量里头（具体计算交给调用方）
        memory_current_belief_link_budget_and_target_activation: Option<(
            &mut impl BudgetValue<E = Self::E>,
            Self::E,
        )>,
    ) -> Self {
        /* 📄OpenNARS源码：
        return budgetInference(truthToQuality(truth), 1, memory); */
        Self::__budget_inference(
            Self::truth_to_quality(truth),
            1,
            memory_t_budget,
            memory_current_belief_link_budget_and_target_activation,
        )
    }

    /// 模拟`BudgetFunctions.backward`
    /// * 💭似乎跟「前向推理」[`BudgetFunctions::forward`]一样
    ///
    /// # 📄OpenNARS
    ///
    /// Backward inference result and adjustment, stronger case
    ///
    /// @param truth  The truth value of the belief deriving the conclusion
    /// @param memory Reference to the memory
    /// @return The budget value of the conclusion
    fn backward(
        truth: &impl TruthValue<E = Self::E>,
        // * 🚩↓对标`Item t`（来自`memory`）
        memory_t_budget: &impl BudgetValue<E = Self::E>,
        // * 🚩↓对标两个变量，其中一个在另一个变量的条件里边，因此为「保证同时存在」将其合并到一个变量里头（具体计算交给调用方）
        memory_current_belief_link_budget_and_target_activation: Option<(
            &mut impl BudgetValue<E = Self::E>,
            Self::E,
        )>,
    ) -> Self {
        /* 📄OpenNARS源码：
        return budgetInference(truthToQuality(truth), 1, memory); */
        Self::__budget_inference(
            Self::truth_to_quality(truth),
            1,
            memory_t_budget,
            memory_current_belief_link_budget_and_target_activation,
        )
    }

    /// 模拟`BudgetFunctions.backwardWeak`
    /// ? ❓【2024-05-04 01:18:42】究竟是哪儿「弱」了
    ///   * 📝答：在「质量」前乘了个恒定系数（表示「弱推理」？）
    ///
    /// # 📄OpenNARS
    ///
    /// Backward inference result and adjustment, weaker case
    ///
    /// @param truth  The truth value of the belief deriving the conclusion
    /// @param memory Reference to the memory
    /// @return The budget value of the conclusion
    fn backward_weak(
        truth: &impl TruthValue<E = Self::E>,
        // * 🚩↓对标`Item t`（来自`memory`）
        memory_t_budget: &impl BudgetValue<E = Self::E>,
        // * 🚩↓对标两个变量，其中一个在另一个变量的条件里边，因此为「保证同时存在」将其合并到一个变量里头（具体计算交给调用方）
        memory_current_belief_link_budget_and_target_activation: Option<(
            &mut impl BudgetValue<E = Self::E>,
            Self::E,
        )>,
    ) -> Self {
        /* 📄OpenNARS源码：
        return budgetInference(w2c(1) * truthToQuality(truth), 1, memory); */
        Self::__budget_inference(
            Self::E::w2c(1.0) & Self::truth_to_quality(truth),
            1,
            memory_t_budget,
            memory_current_belief_link_budget_and_target_activation,
        )
    }

    /* ----- Task derivation in CompositionalRules and StructuralRules ----- */

    /// 模拟`BudgetFunctions.compoundForward`
    ///
    /// # 📄OpenNARS
    ///
    /// Forward inference with CompoundTerm conclusion
    ///
    /// @param truth   The truth value of the conclusion
    /// @param content The content of the conclusion
    /// @param memory  Reference to the memory
    /// @return The budget of the conclusion
    fn compound_forward(
        truth: &impl TruthValue<E = Self::E>,
        content: &Term,
        // * 🚩↓对标`Item t`（来自`memory`）
        memory_t_budget: &impl BudgetValue<E = Self::E>,
        // * 🚩↓对标两个变量，其中一个在另一个变量的条件里边，因此为「保证同时存在」将其合并到一个变量里头（具体计算交给调用方）
        memory_current_belief_link_budget_and_target_activation: Option<(
            &mut impl BudgetValue<E = Self::E>,
            Self::E,
        )>,
    ) -> Self {
        /* 📄OpenNARS源码：
        return budgetInference(truthToQuality(truth), content.getComplexity(), memory); */
        Self::__budget_inference(
            Self::truth_to_quality(truth),
            content.get_complexity(),
            memory_t_budget,
            memory_current_belief_link_budget_and_target_activation,
        )
    }

    /// 模拟`BudgetFunctions.compoundBackward`
    ///
    /// # 📄OpenNARS
    ///
    /// Backward inference with CompoundTerm conclusion, stronger case
    ///
    /// @param content The content of the conclusion
    /// @param memory  Reference to the memory
    /// @return The budget of the conclusion
    fn compound_backward(
        content: &Term,
        // * 🚩↓对标`Item t`（来自`memory`）
        memory_t_budget: &impl BudgetValue<E = Self::E>,
        // * 🚩↓对标两个变量，其中一个在另一个变量的条件里边，因此为「保证同时存在」将其合并到一个变量里头（具体计算交给调用方）
        memory_current_belief_link_budget_and_target_activation: Option<(
            &mut impl BudgetValue<E = Self::E>,
            Self::E,
        )>,
    ) -> Self {
        /* 📄OpenNARS源码：
        return budgetInference(1, content.getComplexity(), memory); */
        Self::__budget_inference(
            Self::E::ONE,
            content.get_complexity(),
            memory_t_budget,
            memory_current_belief_link_budget_and_target_activation,
        )
    }

    /// 模拟`BudgetFunctions.compoundBackwardWeak`
    ///
    /// # 📄OpenNARS
    fn compound_backward_weak(
        content: &Term,
        // * 🚩↓对标`Item t`（来自`memory`）
        memory_t_budget: &impl BudgetValue<E = Self::E>,
        // * 🚩↓对标两个变量，其中一个在另一个变量的条件里边，因此为「保证同时存在」将其合并到一个变量里头（具体计算交给调用方）
        memory_current_belief_link_budget_and_target_activation: Option<(
            &mut impl BudgetValue<E = Self::E>,
            Self::E,
        )>,
    ) -> Self {
        /* 📄OpenNARS源码：
        return budgetInference(w2c(1), content.getComplexity(), memory); */
        Self::__budget_inference(
            Self::E::w2c(1.0),
            content.get_complexity(),
            memory_t_budget,
            memory_current_belief_link_budget_and_target_activation,
        )
    }

    /// 模拟`BudgetFunctions.budgetInference`
    /// * 🚩通用的「预算推理」
    /// * 🚩【2024-05-02 21:22:22】此处脱离与「词项链」「任务链」的关系，仅看其「预算」部分
    ///   * 📝OpenNARS源码本质上还是在强调「预算」而非（继承其上的）「词项」「记忆区」
    ///   * 📝之所以OpenNARS要传入「记忆区」「真值」是因为需要「获取其中某个词项/任务」
    /// * 🚩【2024-05-04 01:11:52】目前通过「将『非纯预算值计算函数』交给调用方计算」勉强实现处理逻辑
    ///   * 一些逻辑还需交由调用方行使
    ///
    /// # 📄OpenNARS
    ///
    /// Common processing for all inference step
    ///
    /// @param qual       Quality of the inference
    /// @param complexity Syntactic complexity of the conclusion
    /// @param memory     Reference to the memory
    /// @return Budget of the conclusion task
    fn __budget_inference(
        qual: Self::E,
        complexity: usize,
        // * 🚩↓对标`Item t`（来自`memory`）
        memory_t_budget: &impl BudgetValue<E = Self::E>,
        // * 🚩↓对标两个变量，其中一个在另一个变量的条件里边，因此为「保证同时存在」将其合并到一个变量里头（具体计算交给调用方）
        memory_current_belief_link_budget_and_target_activation: Option<(
            &mut impl BudgetValue<E = Self::E>,
            Self::E,
        )>,
    ) -> Self {
        /* 📄OpenNARS源码：
        Item t = memory.currentTaskLink;
        if (t == null) {
            t = memory.currentTask;
        }
        float priority = t.getPriority();
        float durability = t.getDurability() / complexity;
        float quality = qual / complexity;
        TermLink bLink = memory.currentBeliefLink;
        if (bLink != null) {
            priority = or(priority, bLink.getPriority());
            durability = and(durability, bLink.getDurability());
            float targetActivation = memory.getConceptActivation(bLink.getTarget());
            bLink.incPriority(or(quality, targetActivation));
            bLink.incDurability(quality);
        }
        return new BudgetValue(priority, durability, quality); */
        let mut priority = memory_t_budget.priority();
        let mut durability =
            Self::E::from_float(memory_t_budget.durability().to_float() / complexity as Float);
        let quality = Self::E::from_float(qual.to_float() / complexity as Float);
        if let Some((b_link, activation)) = memory_current_belief_link_budget_and_target_activation
        {
            priority = priority | b_link.priority();
            durability = durability & b_link.durability();
            let target_activation = activation;
            b_link.inc_priority(quality | target_activation);
            b_link.inc_durability(quality);
        }
        Self::new(priority, durability, quality)
    }
}

/// 自动实现「预算函数」
/// * 🎯直接在「预算值」上加功能
impl<B: BudgetValue> BudgetFunctions for B {}

/// TODO: 单元测试
#[cfg(test)]
mod tests {
    use super::*;
}