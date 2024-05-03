//! 🎯复刻OpenNARS `nars.inference.BudgetFunctions`

use super::UtilityFunctions;
use crate::{
    entity::{BudgetValue, ShortFloat},
    global::Float,
};

/// 预算函数
/// * 🚩【2024-05-02 20:46:50】不同于OpenNARS中「直接创建新值」，此处许多「预算函数」仅改变自身
///   * ✅若需「创建新值」可以通过「事先`clone`」实现
pub trait BudgetFunctions: BudgetValue {
    // TODO: truthToQuality | 涉及「真值」

    // TODO: rankBelief | 涉及「语句」

    // TODO: solutionEval | 涉及「语句」

    // TODO: revise | 涉及「真值」「记忆区（推理上下文）」

    // TODO: update | 涉及「任务」「真值」

    /// 模拟`BudgetFunctions.distributeAmongLinks`
    ///
    /// # 📄OpenNARS
    /// Distribute the budget of a task among the links to it
    ///
    /// @param b The original budget
    /// @param n Number of links
    /// @return Budget value for each link
    fn distribute_among_links(&mut self, n: usize) {
        /* 📄OpenNARS源码：
        float priority = (float) (b.getPriority() / Math.sqrt(n));
        return new BudgetValue(priority, b.getDurability(), b.getQuality()); */
        let priority = self.priority().to_float() / (n as Float).sqrt();
        *self.priority_mut() = Self::E::from_float(priority);
    }

    /// 模拟`BudgetFunctions.activate`
    /// * 🚩【2024-05-02 20:55:40】虽然涉及「概念」，但实际上只用到了「概念作为预算值的部分」
    /// * 📌【2024-05-02 20:56:11】目前要求「概念」一方使用同样的「短浮点」
    ///
    fn activate<B>(&mut self, concept: &impl BudgetValue<E = Self::E>) {
        /* 📄OpenNARS源码：
        float oldPri = concept.getPriority();
        float priority = or(oldPri, budget.getPriority());
        float durability = aveAri(concept.getDurability(), budget.getDurability());
        float quality = concept.getQuality();
        concept.setPriority(priority);
        concept.setDurability(durability);
        concept.setQuality(quality); */
        let old_pri = concept.priority();
        let priority = old_pri.or(concept.priority());
        let durability = Self::E::arithmetical_average([concept.durability(), self.durability()]);
        // let quality = concept.quality(); // ! 这俩不变，可以抵消
        *self.priority_mut() = priority;
        *self.durability_mut() = durability;
        // *self.quality_mut() = quality; // ! 这俩不变，可以抵消
    }

    /// 模拟`BudgetFunctions.forget`
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
        *self.priority_mut() = Self::E::from_float(quality);
    }

    /// 模拟`BudgetValue.merge`，亦与`BudgetFunctions.merge`相同
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
        self.priority_mut().max_from(other.priority());
        self.durability_mut().max_from(other.durability());
        self.quality_mut().max_from(other.quality());
    }

    // TODO: forward | 需要「记忆区」「真值」 `budgetInference`
    // TODO: backward | 需要「记忆区」「真值」 `budgetInference`
    // TODO: backwardWeak | 需要「记忆区」「真值」 `budgetInference`
    // TODO: compoundForward | 需要「记忆区」「词项」「真值」 `budgetInference`
    // TODO: compoundBackward | 需要「记忆区」「词项」 `budgetInference`
    // TODO: compoundBackwardWeak | 需要「记忆区」「词项」 `budgetInference`

    // TODO: budgetInference | 需要「记忆区」「词项链」作为「不仅仅是预算」的方法`memory.getConceptActivation`、`blink.getTarget`
    // ! 🚩【2024-05-02 21:29:45】搁置
    // /// 模拟`BudgetFunctions.budgetInference`
    // /// * 🚩通用的「预算推理」
    // /// * 🚩【2024-05-02 21:22:22】此处脱离与「词项链」「任务链」的关系，仅看其「预算」部分
    // ///   * 📝OpenNARS源码本质上还是在强调「预算」而非（继承其上的）「词项」「记忆区」
    // ///   * 📝之所以OpenNARS要传入「记忆区」「真值」是因为需要「获取其中某个词项/任务」
    // ///
    // /// # 📄OpenNARS
    // ///
    // /// Common processing for all inference step
    // ///
    // /// @param qual       Quality of the inference
    // /// @param complexity Syntactic complexity of the conclusion
    // /// @param memory     Reference to the memory
    // /// @return Budget of the conclusion task
    // fn budget_inference(
    //     &mut self,
    //     complexity: usize,
    //     current_task_link_or_current_task_budget: &impl BudgetValue<E = Self::E>,
    //     belief_link_budget: Option<&impl BudgetValue<E = Self::E>>,
    // ) {
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
    //     // 参数转换
    //     let qual = self.quality();
    //     // 代码复刻
    //     let priority = current_task_link_or_current_task_budget.priority();
    //     let durability = current_task_link_or_current_task_budget
    //         .durability()
    //         .to_float()
    //         / complexity as Float;
    //     let quality = qual.to_float() / complexity as Float;
    //     if let Some(blink) = belief_link_budget {}
    // }
}

/// 自动实现「预算函数」
/// * 🎯直接在「预算值」上加功能
impl<B: BudgetValue> BudgetFunctions for B {}

/// TODO: 单元测试
#[cfg(test)]
mod tests {
    use super::*;
}
