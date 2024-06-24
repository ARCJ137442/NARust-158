//! 预算推理
//! * 📄跟从改版OpenNARS的代码安排
//! * 🎯存储【非纯函数式】【与控制机制直接相关】的预算函数

//! 🎯复刻OpenNARS `nars.inference.BudgetFunctions`

use crate::inference::{Budget, BudgetFunctions};

/// 预算推理
pub trait BudgetInference: Budget {
    /// 🆕模拟`BudgetValue.merge`，亦与`BudgetInference.merge`相同
    /// * 📌此处承载「预算函数」的修改语义
    /// * 📝若不限定`Self: Sized`，则对`new_budget`的赋值有问题
    fn merge_from(&mut self, other: &impl Budget)
    where
        Self: Sized,
    {
        let this = &*self;
        let new_budget = this.merge(other);
        self.copy_budget_from(&new_budget);
    }

    // TODO: 【2024-06-22 14:50:02】后续拆分到「预算推理」中去
    // /* ----- Task derivation in LocalRules and SyllogisticRules ----- */
    // /// 模拟`BudgetInference.forward`
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

    // /// 模拟`BudgetInference.backward`
    // /// * 💭似乎跟「前向推理」[`BudgetInference::forward`]一样
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

    // /// 模拟`BudgetInference.backwardWeak`
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
    // /// 模拟`BudgetInference.compoundForward`
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

    // /// 模拟`BudgetInference.compoundBackward`
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

    // /// 模拟`BudgetInference.compoundBackwardWeak`
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

    // /// 模拟`BudgetInference.budgetInference`
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
impl<B: Budget> BudgetInference for B {}

/// TODO: 单元测试
#[cfg(test)]
mod tests {}
