//! 🎯复刻OpenNARS `nars.inference.BudgetFunctions`

use crate::{
    entity::*,
    global::*,
    inference::{Budget, Truth},
};

/// 预算函数
/// * 🚩【2024-05-03 14:48:13】现在仍依照OpenNARS原意「直接创建新值」
///   * 📝本身复制值也没多大性能损耗
///   * 📌「直接创建新值」会更方便后续调用
///     * 📄减少无谓的`.clone()`
///
/// * ⚠️【2024-06-20 19:56:05】此处仅存储「纯函数」：不在其中修改传入量的函数
pub trait BudgetFunctions: Budget {
    /* ----------------------- Belief evaluation ----------------------- */

    /// 模拟`BudgetFunctions.truthToQuality`
    ///
    /// # 📄OpenNARS
    ///
    /// Determine the quality of a judgement by its truth value alone
    ///
    /// Mainly decided by confidence, though binary judgement is also preferred
    ///
    /// @param t The truth value of a judgement
    /// @return The quality of the judgement, according to truth value only
    fn truth_to_quality(t: &impl Truth) -> ShortFloat {
        // * 🚩真值⇒质量：期望与「0.75(1-期望)」的最大值
        // * 📝函数：max(c * (f - 0.5) + 0.5, 0.375 - 0.75 * c * (f - 0.5))
        // * 📍最小值：当exp=3/7时，全局最小值为3/7（max的两端相等）
        // * 🔑max(x,y) = (x+y+|x-y|)/2
        let exp = t.expectation();
        ShortFloat::from_float(exp.max((1.0 - exp) * 0.75))
    }

    /// 模拟`BudgetFunctions.rankBelief`
    /// * 🚩🆕【2024-05-03 21:46:17】仅传入「语句」中的「真值」与「时间戳长度」，而非「语句」本身
    ///   * 🚩`judgement.getTruth()` => `truth`
    ///   * 🚩`judgement.getStamp().length()` => `stamp_len`
    /// * 📝在使用该函数返回值的地方，仅为「比较大小」
    ///   * 但[`ShortFloat`]已经实现了[`Ord`]并且需要[`UtilityFunctions::or`]
    ///
    /// # 📄OpenNARS
    ///
    /// Determine the rank of a judgement by its quality and originality (stamp length), called from Concept
    ///
    /// @param judgement The judgement to be ranked
    /// @return The rank of the judgement, according to truth value only
    fn rank_belief(truth: &impl Truth, judgement: &impl Judgement) -> ShortFloat {
        // * 🚩两个指标：信度 + 原创性（时间戳长度）
        // * 📝与信度正相关，与「时间戳长度」负相关；二者有一个好，那就整体好
        let confidence = truth.confidence();
        let originality =
            ShortFloat::from_float(1.0 / (judgement.evidence_length() as Float + 1.0));
        confidence | originality
    }

    /* ----- Functions used both in direct and indirect processing of tasks ----- */

    // TODO: 有待「概念」完工
    // /// 概念的「总体优先级」
    // /// * 📝用于概念的「激活」函数上
    // /// Recalculate the quality of the concept [to be refined to show extension/intension balance]
    // fn concept_total_quality()

    fn solution_quality(query: &impl Sentence, solution: &impl Judgement) -> ShortFloat {
        // * 🚩根据「一般疑问 | 特殊疑问/目标」拆解
        // * 📝一般疑问 ⇒ 解の信度
        // * 📝特殊疑问 ⇒ 解の期望 / 解の复杂度
        let has_query_var = query.content().contain_var_q();
        match has_query_var {
            // * 🚩【特殊疑问/目标】 "what" question or goal
            true => ShortFloat::from_float(
                solution.expectation() / solution.content().complexity() as Float,
            ),
            // * 🚩【一般疑问】 "yes/no" question
            false => solution.confidence(),
        }
    }

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
        problem: &impl Question,
        solution: &impl Judgement,
        question_task_budget: &impl Budget,
    ) -> impl Budget + Sized {
        /* 📄OpenNARS改版：
        final float newP = or(questionTaskBudget.getPriority(), solutionQuality(problem, solution));
        final float newD = questionTaskBudget.getDurability();
        final float newQ = truthToQuality(solution);
        return new BudgetValue(newP, newD, newQ); */
        // * ️📝新优先级 = 任务优先级 | 解决方案质量
        let p = question_task_budget.priority() | Self::solution_quality(problem, solution);
        // * 📝新耐久度 = 任务耐久度
        let d = question_task_budget.durability();
        // * ️📝新质量 = 解决方案の真值→质量
        let q = Self::truth_to_quality(solution);
        // 返回
        BudgetValue::new(p, d, q)
    }

    // TODO: 修正规则（更新旧预算值，但发送到「新预算值」中）

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
        task_truth: &impl Truth,
        task_budget: &mut Self,
        b_truth: &impl Truth,
    ) -> impl Budget + Sized {
        /* 📄OpenNARS源码：
        Truth tTruth = task.getSentence().getTruth();
        float dif = tTruth.getExpDifAbs(bTruth);
        float priority = or(dif, task.getPriority());
        float durability = aveAri(dif, task.getDurability());
        float quality = truthToQuality(bTruth);
        return new BudgetValue(priority, durability, quality); */
        let t_truth = task_truth;
        let dif = ShortFloat::from_float(t_truth.expectation_abs_dif(b_truth));
        let priority = dif | task_budget.priority();
        let durability = ShortFloat::arithmetical_average([dif, task_budget.durability()]);
        let quality = Self::truth_to_quality(t_truth);
        BudgetValue::new(priority, durability, quality)
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
    fn distribute_among_links(&self, n: usize) -> impl Budget + Sized {
        /* 📄OpenNARS源码：
        float priority = (float) (b.getPriority() / Math.sqrt(n));
        return new BudgetValue(priority, b.getDurability(), b.getQuality()); */
        let priority = self.priority().to_float() / (n as Float).sqrt();
        BudgetValue::new(
            ShortFloat::from_float(priority),
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
    fn activate(&mut self, budget: &impl Budget) {
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
        let durability = ShortFloat::arithmetical_average([self.durability(), budget.durability()]);
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
            quality += p * self.durability().to_float().powf(1.0 / (forget_rate * p));
        } // 优先级耐久 | q' = q * relativeThreshold + p * d^(1 / forgetRate*p)
        self.set_priority(ShortFloat::from_float(quality));
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
    fn merge(&mut self, other: &impl Budget) {
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

    // TODO: 【2024-06-22 14:50:02】后续拆分到「预算推理」中去
    // /* ----- Task derivation in LocalRules and SyllogisticRules ----- */
    // /// 模拟`BudgetFunctions.forward`
    // ///
    // /// # 📄OpenNARS
    // ///
    // /// Forward inference result and adjustment
    // ///
    // /// @param truth The truth value of the conclusion
    // /// @return The budget value of the conclusion
    // fn forward<C>(
    //     truth: &impl Truth,
    //     // * 🚩【2024-05-12 15:48:37】↓对标`memory`
    //     context: &mut impl DerivationContextReason<C>,
    //     memory: &impl Memory<ShortFloat = ShortFloat>,
    // ) -> impl Budget + Sized
    // where
    //     C: TypeContext<ShortFloat = ShortFloat, Budget = Self>,
    // {
    //     /* 📄OpenNARS源码：
    //     return budgetInference(truthToQuality(truth), 1, memory); */
    //     Self::__budget_inference(Self::truth_to_quality(truth), 1, context, memory)
    // }

    // /// 模拟`BudgetFunctions.backward`
    // /// * 💭似乎跟「前向推理」[`BudgetFunctions::forward`]一样
    // ///
    // /// # 📄OpenNARS
    // ///
    // /// Backward inference result and adjustment, stronger case
    // ///
    // /// @param truth  The truth value of the belief deriving the conclusion
    // /// @param memory Reference to the memory
    // /// @return The budget value of the conclusion
    // fn backward<C>(
    //     truth: &impl Truth,
    //     // * 🚩【2024-05-12 15:48:37】↓对标`memory`
    //     context: &mut impl DerivationContextReason<C>,
    //     memory: &impl Memory<ShortFloat = ShortFloat>,
    // ) -> impl Budget + Sized
    // where
    //     C: TypeContext<ShortFloat = ShortFloat, Budget = Self>,
    // {
    //     /* 📄OpenNARS源码：
    //     return budgetInference(truthToQuality(truth), 1, memory); */
    //     Self::__budget_inference(Self::truth_to_quality(truth), 1, context, memory)
    // }

    // /// 模拟`BudgetFunctions.backwardWeak`
    // /// ? ❓【2024-05-04 01:18:42】究竟是哪儿「弱」了
    // ///   * 📝答：在「质量」前乘了个恒定系数（表示「弱推理」？）
    // ///
    // /// # 📄OpenNARS
    // ///
    // /// Backward inference result and adjustment, weaker case
    // ///
    // /// @param truth  The truth value of the belief deriving the conclusion
    // /// @param memory Reference to the memory
    // /// @return The budget value of the conclusion
    // fn backward_weak<C>(
    //     truth: &impl Truth,
    //     // * 🚩【2024-05-12 15:48:37】↓对标`memory`
    //     context: &mut impl DerivationContextReason<C>,
    //     memory: &impl Memory<ShortFloat = ShortFloat>,
    // ) -> impl Budget + Sized
    // where
    //     C: TypeContext<ShortFloat = ShortFloat, Budget = Self>,
    // {
    //     /* 📄OpenNARS源码：
    //     return budgetInference(w2c(1) * truthToQuality(truth), 1, memory); */
    //     Self::__budget_inference(
    //         ShortFloat::w2c(1.0) & Self::truth_to_quality(truth),
    //         1,
    //         context,
    //         memory,
    //     )
    // }

    // /* ----- Task derivation in CompositionalRules and StructuralRules ----- */
    // /// 模拟`BudgetFunctions.compoundForward`
    // ///
    // /// # 📄OpenNARS
    // ///
    // /// Forward inference with CompoundTerm conclusion
    // ///
    // /// @param truth   The truth value of the conclusion
    // /// @param content The content of the conclusion
    // /// @param memory  Reference to the memory
    // /// @return The budget of the conclusion
    // fn compound_forward<C>(
    //     truth: &impl Truth,
    //     content: &Term,
    //     // * 🚩【2024-05-12 15:48:37】↓对标`memory`
    //     context: &mut impl DerivationContextReason<C>,
    //     memory: &impl Memory<ShortFloat = ShortFloat>,
    // ) -> impl Budget + Sized
    // where
    //     C: TypeContext<ShortFloat = ShortFloat, Budget = Self>,
    // {
    //     /* 📄OpenNARS源码：
    //     return budgetInference(truthToQuality(truth), content.getComplexity(), memory); */
    //     Self::__budget_inference(
    //         Self::truth_to_quality(truth),
    //         content.complexity(),
    //         context,
    //         memory,
    //     )
    // }

    // /// 模拟`BudgetFunctions.compoundBackward`
    // ///
    // /// # 📄OpenNARS
    // ///
    // /// Backward inference with CompoundTerm conclusion, stronger case
    // ///
    // /// @param content The content of the conclusion
    // /// @param memory  Reference to the memory
    // /// @return The budget of the conclusion
    // fn compound_backward<C>(
    //     content: &Term,
    //     // * 🚩【2024-05-12 15:48:37】↓对标`memory`
    //     context: &mut impl DerivationContextReason<C>,
    //     memory: &impl Memory<ShortFloat = ShortFloat>,
    // ) -> impl Budget + Sized
    // where
    //     C: TypeContext<ShortFloat = ShortFloat, Budget = Self>,
    // {
    //     /* 📄OpenNARS源码：
    //     return budgetInference(1, content.getComplexity(), memory); */
    //     Self::__budget_inference(ShortFloat::ONE, content.complexity(), context, memory)
    // }

    // /// 模拟`BudgetFunctions.compoundBackwardWeak`
    // ///
    // /// # 📄OpenNARS
    // fn compound_backward_weak<C>(
    //     content: &Term,
    //     // * 🚩【2024-05-12 15:48:37】↓对标`memory`
    //     context: &mut impl DerivationContextReason<C>,
    //     memory: &impl Memory<ShortFloat = ShortFloat>,
    // ) -> impl Budget + Sized
    // where
    //     C: TypeContext<ShortFloat = ShortFloat, Budget = Self>,
    // {
    //     /* 📄OpenNARS源码：
    //     return budgetInference(w2c(1), content.getComplexity(), memory); */
    //     Self::__budget_inference(ShortFloat::w2c(1.0), content.complexity(), context, memory)
    // }

    // /// 模拟`BudgetFunctions.budgetInference`
    // /// * 🚩通用的「预算推理」
    // /// * 🚩【2024-05-02 21:22:22】此处脱离与「词项链」「任务链」的关系，仅看其「预算」部分
    // ///   * 📝OpenNARS源码本质上还是在强调「预算」而非（继承其上的）「词项」「记忆区」
    // ///   * 📝之所以OpenNARS要传入「记忆区」「真值」是因为需要「获取其中某个词项/任务」
    // /// * 🚩【2024-05-12 15:55:37】目前在实现「记忆区」「推理上下文」的API之下，可以按逻辑无损复刻
    // ///   * ❓后续是否要将「记忆区」的引用代入「推理上下文」
    // /// * 📝【2024-05-17 15:41:10】经OpenNARS基本论证：`t`不可能为`null`
    // ///   * 📌「直接推理（任务+概念）」从来不会调用此函数
    // ///     * 📄证据：`processJudgement`与`processQuestion`均除了本地规则「修正/问答」外没调用别的
    // ///   * 🚩【2024-05-18 01:58:44】故因此只会从「概念推理」被调用，
    // ///   * ✅使用[`DerivationContextReason`]解决
    // ///
    // ///
    // /// # 📄OpenNARS
    // ///
    // /// Common processing for all inference step
    // ///
    // /// @param qual       Quality of the inference
    // /// @param complexity Syntactic complexity of the conclusion
    // /// @param memory     Reference to the memory
    // /// @return Budget of the conclusion task
    // fn __budget_inference<C>(
    //     qual: ShortFloat,
    //     complexity: usize,
    //     context: &mut impl DerivationContextReason<C>,
    //     memory: &impl Memory<ShortFloat = ShortFloat>,
    // ) -> impl Budget + Sized
    // where
    //     C: TypeContext<ShortFloat = ShortFloat, Budget = Self>,
    // {
    //     /* 📄OpenNARS源码：
    //     Item t = memory.currentTaskLink;
    //     if (t == null) {
    //         t = memory.currentTask;
    //     }
    //     float priority = t.getPriority();
    //     float durability = t.getDurability() / complexity;
    //     float quality = qual / complexity;
    //     TermLink bLink = memory.currentBeliefLink;
    //     if (bLink != null) {
    //         priority = or(priority, bLink.getPriority());
    //         durability = and(durability, bLink.getDurability());
    //         float targetActivation = memory.getConceptActivation(bLink.getTarget());
    //         bLink.incPriority(or(quality, targetActivation));
    //         bLink.incDurability(quality);
    //     }
    //     return new BudgetValue(priority, durability, quality); */
    //     let t_budget = context.current_task_link().budget();
    //     let mut priority = t_budget.priority();
    //     let mut durability =
    //         ShortFloat::from_float(t_budget.durability().to_float() / complexity as Float);
    //     let quality = ShortFloat::from_float(qual.to_float() / complexity as Float);
    //     let b_link = context.current_belief_link_mut();
    //     let activation = memory.get_concept_activation(&b_link.target());
    //     priority = priority | b_link.priority();
    //     durability = durability & b_link.durability();
    //     let target_activation = activation;
    //     b_link.inc_priority(quality | target_activation);
    //     b_link.inc_durability(quality);
    //     BudgetValue::new(priority, durability, quality)
    // }
}

/// 自动实现「预算函数」
/// * 🎯直接在「预算值」上加功能
/// * 🚩现在只为「具体的值」（带有「构造/转换」函数的类型）实现
impl<B: Budget> BudgetFunctions for B {}

/// TODO: 单元测试
#[cfg(test)]
mod tests {}
