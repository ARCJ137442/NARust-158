//! 🎯复刻OpenNARS `nars.inference.BudgetFunctions`

use crate::{
    debug_assert_matches,
    entity::*,
    global::*,
    inference::{Budget, Truth},
    language::Term,
};

/// 预算函数
/// * 🚩【2024-05-03 14:48:13】现在仍依照OpenNARS原意「直接创建新值」
///   * 📝本身复制值也没多大性能损耗
///   * 📌「直接创建新值」会更方便后续调用
///     * 📄减少无谓的`.clone()`
/// * ❌【2024-06-24 16:30:56】不能使用`impl Budget + Sized`
///   * 📝这会碰上生命周期问题：不能保证返回的值一定不包含对传入参数的借用
///   * 📄首次错误出现位置：[`crate::inference::BudgetInference::merge_from`]
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
    fn truth_to_quality(truth: &impl Truth) -> ShortFloat {
        // * 🚩现在从更原始（无需反复转换）的`_float`函数中来
        ShortFloat::from_float(Self::truth_to_quality_float(truth))
    }
    fn truth_to_quality_float(truth: &impl Truth) -> Float {
        // * 🚩真值⇒质量：期望与「0.75(1-期望)」的最大值
        // * 📝函数：max(c * (f - 0.5) + 0.5, 0.375 - 0.75 * c * (f - 0.5))
        // * 📍最小值：当exp=3/7时，全局最小值为3/7（max的两端相等）
        // * 🔑max(x,y) = (x+y+|x-y|)/2
        let exp = truth.expectation();
        exp.max((1.0 - exp) * 0.75)
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
    fn rank_belief(judgement: &impl Judgement) -> Float {
        // * 🚩两个指标：信度 + 原创性（时间戳长度）
        // * 📝与信度正相关，与「时间戳长度」负相关；二者有一个好，那就整体好
        let confidence = judgement.confidence();
        let originality =
            ShortFloat::from_float(1.0 / (judgement.evidence_length() as Float + 1.0));
        (confidence | originality).to_float()
    }

    /* ----- Functions used both in direct and indirect processing of tasks ----- */

    // TODO: 有待「概念」完工
    // /// 概念的「总体优先级」
    // /// * 📝用于概念的「激活」函数上
    // /// Recalculate the quality of the concept [to be refined to show extension/intension balance]
    fn concept_total_quality(_concept: &()) -> ShortFloat {
        todo!()
    }

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
    /// * ✅【2024-06-23 01:37:36】目前已按照改版OpenNARS设置
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
    ) -> BudgetValue {
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

    /// 统一的「修正规则」预算函数
    /// * 🚩依照改版OpenNARS，从旧稿中重整
    /// * ✅完全脱离「推理上下文」仅有纯粹的「真值/预算值」计算
    /// * ✅其中对「任务链可空性=信念链可空性」做断言：`feedBackToLinks == current_links_budget.is_some()`
    fn revise(
        new_belief_truth: &impl Truth, // from task
        old_belief_truth: &impl Truth, // from belief
        revised_truth: &impl Truth,
        current_task_budget: &impl Budget,
        current_links_budget: Option<[&impl Budget; 2]>,
    ) -> ReviseResult {
        // * 🚩计算落差 | t = task, b = belief
        let dif_to_new_task =
            ShortFloat::from_float(revised_truth.expectation_abs_dif(new_belief_truth));
        let dif_to_old_belief =
            ShortFloat::from_float(revised_truth.expectation_abs_dif(old_belief_truth));
        // * 🚩若有：反馈到 [任务链, 信念链]
        let new_links_budget = current_links_budget.map(|[t_budget, b_budget]| {
            [
                // * 📝当前任务链 降低预算：
                // * * p = link & !difT
                // * * d = link & !difT
                // * * q = link
                BudgetValue::new(
                    t_budget.priority() & !dif_to_new_task,
                    t_budget.durability() & !dif_to_new_task,
                    t_budget.quality(),
                ),
                // * 📝当前信念链 降低预算：
                // * * p = link & !difB
                // * * d = link & !difB
                // * * q = link
                BudgetValue::new(
                    b_budget.priority() & !dif_to_old_belief,
                    b_budget.durability() & !dif_to_old_belief,
                    b_budget.quality(),
                ),
            ]
        });
        // * 🚩用落差降低优先级、耐久度
        // * 📝当前任务 降低预算：
        // * * p = task & !difT
        // * * d = task & !difT
        // * * q = task
        let new_task_budget = BudgetValue::new(
            current_task_budget.priority() & !dif_to_new_task,
            current_task_budget.durability() | !dif_to_new_task,
            current_task_budget.quality(),
        );
        // * 🚩用更新后的值计算新差 | ❓此时是否可能向下溢出？
        // * 📝新差 = 修正后信念.信度 - max(新信念.信度, 旧信念.信度)
        let dif = revised_truth.confidence()
            - old_belief_truth
                .confidence()
                .max(old_belief_truth.confidence());
        // * 🚩计算新预算值
        // * 📝优先级 = 差 | 当前任务
        // * 📝耐久度 = (差 + 当前任务) / 2
        // * 📝质量 = 新真值→质量
        let new_budget = BudgetValue::new(
            dif | current_task_budget.priority(),
            ShortFloat::arithmetical_average([dif, current_task_budget.durability()]),
            Self::truth_to_quality(revised_truth),
        );
        // 返回
        ReviseResult {
            new_budget,
            new_task_budget,
            new_links_budget,
        }
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
        task_truth: &impl Truth,
        task_budget: &mut Self,
        b_truth: &impl Truth,
    ) -> BudgetValue {
        /* 📄OpenNARS源码：
        Truth tTruth = task.getSentence().getTruth();
        float dif = tTruth.getExpDifAbs(bTruth);
        float priority = or(dif, task.getPriority());
        float durability = aveAri(dif, task.getDurability());
        float quality = truthToQuality(bTruth);
        return new BudgetValue(priority, durability, quality); */
        // * 🚩计算落差
        let dif = ShortFloat::from_float(task_truth.expectation_abs_dif(b_truth));
        // * 🚩根据落差计算预算值
        // * 📝优先级 = 落差 | 任务
        // * 📝耐久度 = (落差 + 任务) / 2
        // * 📝质量 = 信念真值→质量
        let priority = dif | task_budget.priority();
        let durability = ShortFloat::arithmetical_average([dif, task_budget.durability()]);
        let quality = Self::truth_to_quality(task_truth);
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
    fn distribute_among_links(&self, n: usize) -> BudgetValue {
        /* 📄OpenNARS源码：
        float priority = (float) (b.getPriority() / Math.sqrt(n));
        return new BudgetValue(priority, b.getDurability(), b.getQuality()); */
        // * 📝优先级 = 原 / √链接数
        // * 📝耐久度 = 原
        // * 📝质量 = 原
        let priority = self.priority().to_float() / (n as Float).sqrt();
        BudgetValue::new(
            ShortFloat::from_float(priority),
            self.durability(),
            self.quality(),
        )
    }

    /* ----------------------- Concept ----------------------- */

    // TODO: 有待更新：要计算「概念」的「总体质量」
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
    /// * 🚩【2024-06-24 16:13:41】现在跟从改版OpenNARS，转为「创建新值」语义
    ///
    /// # 📄OpenNARS
    ///
    /// Decrease Priority after an item is used, called in Bag
    ///
    /// After a constant time, p should become d*p.
    ///
    /// Since in this period, the item is accessed c*p times, each time p-q should multiple d^(1/(c*p)).
    ///
    /// The intuitive meaning of the parameter "forgetRate" is:
    /// after this number of times of access, priority 1 will become d, it is a system parameter adjustable in run time.
    ///
    /// - @param budget            The previous budget value
    /// - @param forgetRate        The budget for the new item
    /// - @param relativeThreshold The relative threshold of the bag
    fn forget(&self, forget_rate: Float, relative_threshold: Float) -> Float {
        /* 📄OpenNARS源码：
        double quality = budget.getQuality() * relativeThreshold; // re-scaled quality
        double p = budget.getPriority() - quality; // priority above quality
        if (p > 0) {
            quality += p * Math.pow(budget.getDurability(), 1.0 / (forgetRate * p));
        } // priority Durability
        budget.setPriority((float) quality); */
        let [p, d, q] = self.pdq_float();
        // * 🚩先放缩「质量」
        let scaled_q = q * relative_threshold;
        // * 🚩计算优先级和「放缩后质量」的差
        let dif_p_q = p - scaled_q;
        // * 🚩计算新的优先级
        match dif_p_q > 0.0 {
            // * 🚩差值 > 0 | 衰减
            true => scaled_q + dif_p_q * d.powf(1.0 / (forget_rate * dif_p_q)),
            // * 🚩差值 < 0 | 恒定
            false => scaled_q,
        }
    }

    /// 模拟`BudgetValue.merge`，亦与`BudgetFunctions.merge`相同
    /// * 📝【2024-05-03 14:55:29】虽然现在「预算函数」以「直接创建新值」为主范式，
    ///   * 但在用到该函数的`merge`方法上，仍然是「修改」语义——需要可变引用
    /// * 🚩【2024-06-24 16:15:22】现在跟从改版OpenNARS，直接创建新值
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
    fn merge(&self, other: &impl Budget) -> BudgetValue {
        let p = self.priority().max(other.priority());
        let d = self.durability().max(other.durability());
        let q = self.quality().max(other.quality());
        BudgetValue::new(p, d, q)
    }

    /// Forward inference result and adjustment
    fn forward(truth: Option<&impl Truth>, content: Option<&Term>) -> BudgetInferenceParameters {
        // * 📝真值转质量，用不到词项
        debug_assert_matches!((truth, content), (Some(..), None));
        let inference_quality = truth.map_or(ShortFloat::ONE, Self::truth_to_quality);
        let complexity = 1;
        BudgetInferenceParameters {
            inference_quality, // 默认值：1
            complexity,
        }
    }

    /// Backward inference result and adjustment, stronger case
    fn backward(truth: Option<&impl Truth>, content: Option<&Term>) -> BudgetInferenceParameters {
        // * 📝真值转质量，用不到词项
        debug_assert_matches!((truth, content), (Some(..), None));
        let inference_quality = truth.map_or(ShortFloat::ONE, Self::truth_to_quality);
        let complexity = 1;
        BudgetInferenceParameters {
            inference_quality, // 默认值：1
            complexity,
        }
    }

    /// Backward inference result and adjustment, weaker case
    fn backward_weak(
        truth: Option<&impl Truth>,
        content: Option<&Term>,
    ) -> BudgetInferenceParameters {
        // * 📝真值转质量，用不到词项
        debug_assert_matches!((truth, content), (Some(..), None));
        let inference_quality =
            ShortFloat::W2C1() * truth.map_or(ShortFloat::ONE, Self::truth_to_quality);
        let complexity = 1;
        BudgetInferenceParameters {
            inference_quality, // 默认值：1
            complexity,
        }
    }

    /// Forward inference with CompoundTerm conclusion
    fn compound_forward(
        truth: Option<&impl Truth>,
        content: Option<&Term>,
    ) -> BudgetInferenceParameters {
        // * 📝真值转质量，用到词项的复杂度
        debug_assert_matches!((truth, content), (Some(..), Some(..)));
        let inference_quality = truth.map_or(ShortFloat::ONE, Self::truth_to_quality);
        let complexity = content.map_or(1, Term::complexity);
        BudgetInferenceParameters {
            inference_quality, // 默认值：1
            complexity,        // 默认值：1
        }
    }

    /// Backward inference with CompoundTerm conclusion, stronger case
    fn compound_backward(
        truth: Option<&impl Truth>,
        content: Option<&Term>,
    ) -> BudgetInferenceParameters {
        // * 📝用到词项的复杂度，用不到真值
        debug_assert_matches!((truth, content), (None, Some(..)));
        let inference_quality = ShortFloat::ONE;
        let complexity = content.map_or(1, Term::complexity);
        BudgetInferenceParameters {
            inference_quality,
            complexity, // 默认值：1
        }
    }

    /// Backward inference with CompoundTerm conclusion, weaker case
    fn compound_backward_weak(
        truth: Option<&impl Truth>,
        content: Option<&Term>,
    ) -> BudgetInferenceParameters {
        // * 📝用到词项的复杂度，用不到真值
        debug_assert_matches!((truth, content), (None, Some(..)));
        let inference_quality = ShortFloat::W2C1();
        let complexity = content.map_or(1, Term::complexity);
        BudgetInferenceParameters {
            inference_quality,
            complexity, // 默认值：1
        }
    }

    /// 从「预算推理函数 枚举」到「预算推理函数指针」
    fn budget_inference_function_from<T: Truth>(
        function_enum: BudgetInferenceFunction,
    ) -> BudgetInferenceF<T> {
        use BudgetInferenceFunction::*;
        match function_enum {
            Forward => Self::forward,
            Backward => Self::backward,
            BackwardWeak => Self::backward_weak,
            CompoundForward => Self::compound_forward,
            CompoundBackward => Self::compound_backward,
            CompoundBackwardWeak => Self::compound_backward_weak,
        }
    }
    /// Common processing for all inference step
    ///
    /// @param inferenceQuality [] Quality of the inference
    /// @param complexity       [] Syntactic complexity of the conclusion
    /// @return [] Budget of the conclusion task
    fn budget_inference<T: Truth>(
        function: BudgetInferenceFunction,
        truth: Option<&T>,
        content: Option<&Term>,
        task_link_budget: &impl Budget,
        belief_link_budget: Option<&impl Budget>,
        target_activation: ShortFloat,
    ) -> BudgetInferenceResult {
        // * 🚩应用函数，提取其中的「推理优先级」和「复杂度」
        let budget_inference_function = Self::budget_inference_function_from::<T>(function);
        let BudgetInferenceParameters {
            inference_quality,
            complexity,
        } = budget_inference_function(truth, content);
        // * 🚩获取「任务链」和「信念链」的优先级（默认0）与耐久度（默认1）
        // * 📝p = self ?? 0
        // * 📝d = self ?? 1
        let [t_link_p, t_link_d] = [task_link_budget.priority(), task_link_budget.durability()];
        let [b_link_p, b_link_d] = match belief_link_budget {
            // * 🚩有信念链⇒取其值
            Some(budget) => [budget.priority(), budget.durability()],
            // * 🚩无信念链⇒默认为[0, 1]
            None => [ShortFloat::ZERO, ShortFloat::ONE],
        };
        // * 🚩更新预算
        // * 📝p = task | belief
        // * 📝d = (task / complexity) & belief
        // * 📝q = inferenceQuality / complexity
        let [p, d, q] = [
            t_link_p | b_link_p,
            (t_link_d / complexity) & b_link_d,
            inference_quality / complexity,
        ];
        // * 🚩有信念链⇒更新信念链预算值
        // * 🚩【2024-06-20 17:11:30】现在返回一个新的预算值
        let new_belief_link_budget = belief_link_budget.map(|b_link_budget| {
            // * 📌此处仅在「概念推理」中出现：能使用可空值处理
            // * 📝p = belief | quality | targetActivation
            // * 📝d = belief | quality
            // * 📝q = belief
            // * 🚩提升优先级
            let [b_link_p, b_link_d, b_link_q] = b_link_budget.pdq();
            BudgetValue::new(b_link_p | q | target_activation, b_link_d | q, b_link_q)
        });
        // * 🚩返回预算值
        BudgetInferenceResult {
            new_budget: BudgetValue::new(p, d, q),
            new_belief_link_budget,
        }
    }
}

/// 修正规则的预算推理结果
/// * 🎯用于[`BudgetFunctions::revise`]
pub struct ReviseResult {
    /// 新预算
    pub new_budget: BudgetValue,
    /// 新任务预算
    pub new_task_budget: BudgetValue,
    /// [新任务链预算, 新信念链预算]（可空）
    /// * 📌左边任务链，右边信念链
    /// * 🎯统一二者的可空性 from `feedbackToLinks`
    pub new_links_budget: Option<[BudgetValue; 2]>,
}

mod budget_inference_functions {
    use super::*;

    pub struct BudgetInferenceParameters {
        /// * 🚩目前只用于「预算推理」的被除数（除以复杂度）上
        pub inference_quality: ShortFloat,
        pub complexity: usize,
    }

    /// 统一的「预算值参数计算函数」指针类型（带泛型）
    pub type BudgetInferenceF<T> = fn(Option<&T>, Option<&Term>) -> BudgetInferenceParameters;

    /// 所有可用的预算值函数
    /// * 🎯统一呈现「在推理过程中计算预算值」的「预算超参数」
    pub enum BudgetInferenceFunction {
        /// 正向推理
        Forward,
        /// 反向强推理
        Backward,
        /// 反向弱推理
        BackwardWeak,
        /// 复合正向推理
        CompoundForward,
        /// 复合反向强推理
        CompoundBackward,
        /// 复合反向弱推理
        CompoundBackwardWeak,
    }

    pub struct BudgetInferenceResult {
        /// 预算推理算出的新预算
        pub new_budget: BudgetValue,
        /// 预算推理算出的「新信念链预算」
        pub new_belief_link_budget: Option<BudgetValue>,
    }
}
pub use budget_inference_functions::*;

/// 自动实现「预算函数」
/// * 🎯直接在「预算值」上加功能
impl<B: Budget> BudgetFunctions for B {}

/// TODO: 单元测试
#[cfg(test)]
mod tests {}
