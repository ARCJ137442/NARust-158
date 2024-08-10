//! 预算推理
//! * 📄跟从改版OpenNARS的代码安排
//! * 🎯存储【非纯函数式】【与控制机制直接相关】的预算函数

//! 🎯复刻OpenNARS `nars.inference.BudgetFunctions`

use super::{BudgetInferenceFunction, BudgetInferenceResult, Truth};
use crate::{
    control::ReasonContextWithLinks,
    entity::{BudgetValue, ShortFloat, TLink, TruthValue},
    inference::{Budget, BudgetFunctions, ReviseResult},
    language::Term,
    util::{OptionOrSomeRef, RefCount},
};

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

    /// 修正@直接推理
    /// * 🚩【2024-05-21 10:30:50】现在仅用于直接推理，但逻辑可以共用：「反馈到链接」与「具体任务计算」并不矛盾
    ///
    /// # 📄OpenNARS
    ///
    /// Evaluate the quality of a revision, then de-prioritize the premises
    fn revise_direct(
        new_belief_truth: &impl Truth,
        old_belief_truth: &impl Truth,
        revised_truth: &impl Truth,
        current_task_budget: &mut impl Budget,
    ) -> BudgetValue {
        // * 🚩计算
        let ReviseResult {
            new_budget,
            new_task_budget,
            ..
        } = BudgetValue::revise(
            new_belief_truth,
            old_belief_truth,
            revised_truth,
            current_task_budget,
            None::<(&BudgetValue, &BudgetValue)>,
        );
        // * 🚩应用修改
        current_task_budget.copy_budget_from(&new_task_budget);
        // * 🚩返回
        new_budget
    }
}

/// 自动实现「预算推理」
/// * 🎯直接在「预算值」上加功能
impl<B: Budget> BudgetInference for B {}

use BudgetInferenceFunction::*;
/// 🆕为「推理上下文」实现的「预算推理」系列方法
pub trait BudgetInferenceContext: ReasonContextWithLinks {
    /// 🆕同{@link BudgetInference#revise}，但是「概念推理」专用
    /// * 🚩在「共用逻辑」后，将预算值反馈回「词项链」「任务链」
    fn revise_matching(
        &mut self,
        new_belief_truth: &impl Truth,
        old_belief_truth: &impl Truth,
        revised_truth: &impl Truth,
    ) -> BudgetValue {
        // * 🚩计算
        let current_task = self.current_task();
        let current_task_link = self.current_task_link();
        let current_belief_link = self.belief_link_for_budget_inference();
        let current_links_budget = current_belief_link.map(|b_link| (current_task_link, b_link));
        let result = BudgetValue::revise(
            new_belief_truth,
            old_belief_truth,
            revised_truth,
            &*current_task.get_(),
            current_links_budget,
        );
        // * 🚩应用修改
        // 任务更新
        drop(current_task);
        self.current_task_mut()
            .mut_()
            .copy_budget_from(&result.new_task_budget);
        // 链接更新
        if let Some([new_t_budget, new_b_budget]) = result.new_links_budget {
            let current_task_link = self.current_task_link_mut();
            current_task_link.copy_budget_from(&new_t_budget);
            if let Some(current_belief_link) = self.belief_link_for_budget_inference_mut() {
                current_belief_link.copy_budget_from(&new_b_budget);
            }
        }
        // * 🚩返回
        result.new_budget
    }

    /// # 📄OpenNARS
    ///
    /// Forward inference result and adjustment
    fn budget_forward<T: Truth>(&mut self, truth: impl OptionOrSomeRef<T>) -> BudgetValue {
        self.budget_inference(Forward, truth.or_some(), None)
    }

    /// # 📄OpenNARS
    ///
    /// Backward inference result and adjustment, stronger case
    fn budget_backward<T: Truth>(&mut self, truth: impl OptionOrSomeRef<T>) -> BudgetValue {
        self.budget_inference(Backward, truth.or_some(), None)
    }

    /// # 📄OpenNARS
    ///
    /// Backward inference result and adjustment, weaker case
    fn budget_backward_weak<T: Truth>(&mut self, truth: impl OptionOrSomeRef<T>) -> BudgetValue {
        self.budget_inference(BackwardWeak, truth.or_some(), None)
    }

    /// # 📄OpenNARS
    ///
    /// Forward inference with CompoundTerm conclusion
    fn budget_compound_forward<T: Truth>(
        &mut self,
        truth: impl OptionOrSomeRef<T>,
        content: impl OptionOrSomeRef<Term>,
    ) -> BudgetValue {
        self.budget_inference(CompoundForward, truth.or_some(), content.or_some())
    }

    /// # 📄OpenNARS
    ///
    /// Backward inference with CompoundTerm conclusion, stronger case
    fn budget_compound_backward(&mut self, content: impl OptionOrSomeRef<Term>) -> BudgetValue {
        self.budget_inference(CompoundBackward, None::<&TruthValue>, content.or_some())
    }

    /// # 📄OpenNARS
    ///
    /// Backward inference with CompoundTerm conclusion, weaker case
    fn budget_compound_backward_weak(
        &mut self,
        content: impl OptionOrSomeRef<Term>,
    ) -> BudgetValue {
        self.budget_inference(CompoundBackwardWeak, None::<&TruthValue>, content.or_some())
    }

    /// # 📄OpenNARS
    ///
    /// Common processing for all inference step
    fn budget_inference(
        &mut self,
        function: BudgetInferenceFunction,
        truth: Option<&impl Truth>,
        content: Option<&Term>,
    ) -> BudgetValue {
        // * 🚩获取有关「词项链」「任务链」的有关参数
        let t_link = self.current_task_link();
        let b_link = self.belief_link_for_budget_inference();
        // * 🚩非空时计算，其它默认为0（转换推理不会用到）
        let target_activation = b_link.map_or(ShortFloat::ZERO, |b_link| {
            self.concept_activation(&b_link.target())
        });
        // * 🚩计算新结果
        let result = BudgetValue::budget_inference(
            function,
            truth,
            content,
            t_link,
            b_link,
            target_activation,
        );
        // * 🚩应用新结果
        let b_link = self.belief_link_for_budget_inference_mut();
        Self::budget_inference_apply(result, b_link)
    }

    /// Get the current activation level of a concept.
    /// * 🚩从「概念」中来
    /// * 🚩【2024-06-22 16:59:34】因涉及控制机制（推理上下文），故放入此中
    fn concept_activation(&self, term: &Term) -> ShortFloat {
        self.term_to_concept(term)
            .map_or(ShortFloat::ZERO, |c| c.priority())
    }

    /// 🆕根据计算出的「预算函数」应用其中的结果
    /// * 🚩覆盖各处预算值，并以此更新
    /// * 🚩返回得出的「新预算值」
    fn budget_inference_apply(
        result: BudgetInferenceResult,
        belief_link_budget: Option<&mut impl Budget>,
    ) -> BudgetValue {
        // * 🚩拿出「新信念链预算」并更新
        if let (Some(b_budget), Some(ref new_budget)) =
            (belief_link_budget, result.new_belief_link_budget)
        {
            b_budget.copy_budget_from(new_budget);
        }
        // * 🚩拿出「新预算」并返回
        result.new_budget
    }
}
impl<C: ReasonContextWithLinks> BudgetInferenceContext for C {}

/// TODO: 单元测试
#[cfg(test)]
mod tests {
    // * merge_from
    // * revise_direct
    // * revise_matching
    // * budget_forward
    // * budget_backward
    // * budget_backward_weak
    // * budget_compound_forward
    // * budget_compound_backward
    // * budget_compound_backward_weak
    // * budget_inference
    // * concept_activation
    // * budget_inference_apply
}
