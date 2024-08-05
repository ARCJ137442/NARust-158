//! 🎯复刻OpenNARS `nars.inference.StructuralRules`
//!
//! * ✅【2024-05-11 15:10:00】初步复现方法API
//! * ♻️【2024-08-05 17:32:20】开始根据改版OpenNARS重写

use super::SyllogismPosition;
use crate::{
    control::*,
    entity::*,
    inference::{rules::utils::*, BudgetInferenceContext, TruthFunctions},
    language::*,
    util::RefCount,
};
use nar_dev_utils::unwrap_or_return;
use ReasonDirection::*;

/// 📝根据复合词项与索引，确定「是否在构建时交换」
///
/// # 📄OpenNARS
///
/// List the cases where the direction of inheritance is revised in conclusion
fn switch_order(compound: CompoundTermRef, index: usize) -> bool {
    // * 🚩外延差/内涵差 且 索引【在右侧】
    // * 📝原理：减法的性质
    // * 📄"<A --> B>" => "<(~, C, B) --> (~, C, A)>"
    // * 💭"<A --> B>" => "<(~, A, C) --> (~, B, C)>"
    // * ✅【2024-07-22 14:51:00】上述例子均以ANSWER验证
    (compound.instanceof_difference() && index == 1)
        // * 🚩外延像/内涵像 且 索引【不是关系词项】
        //   * ⚠️【2024-08-05 22:43:23】纠正：索引为关系词项时，不交换
        // * 📄"<A --> B>" => "<(/, R, _, B) --> (/, R, _, A)>"
        // * 💭"<A --> B>" => "<(/, A, _, C) --> (/, B, _, C)>"
        // * ✅【2024-07-22 14:49:59】上述例子均以ANSWER验证
        || (compound.instanceof_image() && index > 0)
}

/// 🆕根据「是否在构建时交换」交换两项（一般是词项）
fn switch_by_order<T>(compound: CompoundTermRef, index: usize, [sub, pre]: [T; 2]) -> [T; 2] {
    match switch_order(compound, index) {
        true => [pre, sub],
        false => [sub, pre],
    }
}

/// * 📝双侧建构
///
/// # 📄OpenNARS
///
/// ```nal
/// {<S --> P>, S@(S&T)} |- <(S&T) --> (P&T)>
/// {<S --> P>, S@(M-S)} |- <(M-P) --> (M-S)>
/// ```
pub fn structural_compose_both(
    compound: CompoundTerm,
    index: usize,
    statement: Statement,
    side: SyllogismPosition,
    context: &mut ReasonContextConcept,
) {
    let direction = context.reason_direction();

    // * 🚩预筛 * //
    let indicated = side.select(statement.sub_pre());
    if *compound == *indicated {
        // * 📄compound="(&,glasses,[black])" @ 0 = "glasses"
        // * * statement="<sunglasses --> (&,glasses,[black])>" @ 1 = compound
        // * * ⇒不处理（❓为何如此）
        return;
    }

    // * 🚩词项 * //
    let (statement_sub, copula, statement_pre) = statement.unwrap();
    let sub_pre = [&statement_sub, &statement_pre];
    let mut components = compound.get_ref().clone_components();
    let [term_self_side, other_statement_component] = side.select_and_other(sub_pre); // 同侧词项 & 异侧词项
    if components.contains(other_statement_component) {
        // * 📝复合词项包含陈述的另一侧词项 ⇒ 中止
        // * 📄compound = "(*,{tom},(&,glasses,[black]))" @ 1 => "(&,glasses,[black])"
        // * * statement = "<(&,glasses,sunglasses) --> (&,glasses,[black])>" @ 0
        // * * components = ["{tom}", "(&,glasses,[black])"]
        // * * ⇒不处理（❓为何如此）
        return;
    }
    // 先决条件：是否包含同侧词项
    let [sub, pre] = match components.contains(term_self_side) {
        true => side.select_and_other([
            // * 🚩主项/谓项：原来的复合词项
            compound.get_ref().inner.clone(),
            // * 🚩谓项/主项：替换后的复合词项
            {
                let term_opposite = side.opposite().select([statement_sub, statement_pre]); // 提取出异侧词项
                components[index] = term_opposite; // 将对应位置换成异侧词项
                unwrap_or_return!(
                    ?Term::make_compound_term(compound.get_ref(), components)
                )
            },
        ]),
        false => [statement_sub, statement_pre],
    };
    // * 📄compound = "(&,[yellow],{Birdie})" @ 0 => "[yellow]"
    // * * statement = "<{Tweety} --> [yellow]>" @ 1
    // * * components = ["{Tweety}", "{Birdie}"]
    // * * subj = "(&,{Tweety},{Birdie})" = null | 空集
    // * * pred = "(&,[yellow],{Birdie})"
    // * * ⇒制作失败
    // * 🚩根据「复合词项&索引」决定是否要「调换关系」
    let [sub, pre] = switch_by_order(compound.get_ref(), index, [sub, pre]);
    let content = unwrap_or_return!(?Term::make_statement_relation(copula, sub, pre));
    let task_truth = context
        .current_task()
        .get_()
        .as_judgement()
        .map(TruthValue::from);

    // * 🚩真值 * //
    let truth = match direction {
        // * 🚩前向推理
        Forward => match compound.get_ref().size() {
            // * 🚩任务项多于一个元素⇒分析性演绎
            2.. => task_truth.map(|task| task.analytic_deduction(context.reasoning_reliance())),
            // * 🚩其它⇒恒等@当前任务
            _ => task_truth.map(|task| task.identity()),
        },
        // * 🚩反向推理⇒空
        Backward => None,
    };

    // * 🚩预算 * //
    let budget = match direction {
        // * 🚩前向推理⇒复合前向
        Forward => context.budget_compound_forward(truth.as_ref(), &content),
        // * 🚩反向推理⇒复合反向弱
        Backward => context.budget_compound_backward_weak(&content),
    };

    // * 🚩结论 * //
    context.single_premise_task_structural(content, truth, budget);
}

/// * 📝双侧解构
///
/// ```nal
/// {<(S&T) --> (P&T)>, S@(S&T)} |- <S --> P>
/// ```
pub fn structural_decompose_both(
    statement: Statement,
    index: usize,
    context: &mut ReasonContextConcept,
) {
    // * 🚩词项 * //
    let (sub, copula, pre) = statement.unwrap();
    // * 📌必须是「同类复合词项」才有可能解构
    if !sub.is_same_type(&pre) {
        return;
    }
    let [sub, pre]: [CompoundTerm; 2] = match [sub.try_into(), pre.try_into()] {
        [Ok(sub), Ok(pre)] => [sub, pre],
        _ => return,
    };
    // * 📌必须是「同尺寸复合词项」且「索引在界内」
    let [sub_size, pre_size] = [sub.get_ref().size(), pre.get_ref().size()];
    if !(sub_size == pre_size && index < sub_size) {
        return;
    }
    // * 🚩取其中索引所在的词项，按顺序制作相同系词的陈述
    let at_index = |compound: CompoundTermRef| compound.component_at(index).unwrap().clone(); // ! 上边已判断在界内
    let sub_inner = at_index(sub.get_ref());
    let pre_inner = at_index(pre.get_ref());

    // * 🚩尝试调换顺序
    let [content_sub, content_pre] = switch_by_order(sub.get_ref(), index, [sub_inner, pre_inner]);
    let content =
        unwrap_or_return!(?Term::make_statement_relation(copula, content_sub, content_pre));

    // * 🚩预筛
    let direction = context.reason_direction();
    let task_is_judgement = context.current_task().get_().is_judgement();
    let task_truth = context
        .current_task()
        .get_()
        .as_judgement()
        .map(TruthValue::from);
    if !(direction == Forward) // ? 💭【2024-08-05 23:37:40】这个「前向推理又是判断」似乎不可能发生
        && !sub.get_ref().instanceof_product()
        && sub.get_ref().size() > 1
        && task_is_judgement
    {
        return;
    }

    // * 🚩真值 * //
    let truth = match direction {
        // * 🚩前向推理⇒直接用任务的真值
        Forward => task_truth.map(|truth| truth.identity()),
        // * 🚩反向推理⇒空
        Backward => None,
    };

    // * 🚩预算 * //
    let budget = match direction {
        // * 🚩前向推理⇒复合前向
        Forward => context.budget_compound_forward(truth.as_ref(), &content),
        // * 🚩反向推理⇒复合反向
        Backward => context.budget_compound_backward(&content),
    };

    // * 🚩结论 * //
    context.single_premise_task_structural(content, truth, budget);
}

#[cfg(test)]
mod tests {
    use crate::expectation_tests;

    expectation_tests! {
        compose_both_int_ext: {
            "
            nse <A --> B>.
            nse (&,A,C).
            cyc 10
            "
            => OUT "<(&,A,C) --> (&,B,C)>" in outputs
        }

        compose_both_int_ext_answer: {
            "
            nse <A --> B>.
            nse <(&,A,C) --> (&,B,C)>?
            cyc 20
            "
            => ANSWER "<(&,A,C) --> (&,B,C)>" in outputs
        }

        compose_both_int_int: {
            "
            nse <A --> B>.
            nse (|,A,C).
            cyc 10
            "
            => OUT "<(|,A,C) --> (|,B,C)>" in outputs
        }

        compose_both_int_int_answer: {
            "
            nse <A --> B>.
            nse <(|,A,C) --> (|,B,C)>?
            cyc 20
            "
            => ANSWER "<(|,A,C) --> (|,B,C)>" in outputs
        }

        compose_both_diff_ext: {
            "
            nse <A --> B>.
            nse (-,A,C).
            cyc 10
            "
            => OUT "<(-,A,C) --> (-,B,C)>" in outputs
        }

        compose_both_diff_ext_answer: {
            "
            nse <A --> B>.
            nse <(-,A,C) --> (-,B,C)>?
            cyc 20
            "
            => ANSWER "<(-,A,C) --> (-,B,C)>" in outputs
        }

        compose_both_diff_ext_rev: {
            "
            nse <A --> B>.
            nse (-,C,A).
            cyc 10
            "
            => OUT "<(-,C,B) --> (-,C,A)>" in outputs
        }

        compose_both_diff_ext_rev_answer: {
            "
            nse <A --> B>.
            nse <(-,C,B) --> (-,C,A)>?
            cyc 20
            "
            => ANSWER "<(-,C,B) --> (-,C,A)>" in outputs
        }

        compose_both_diff_int: {
            "
            nse <A --> B>.
            nse (~,A,C).
            cyc 10
            "
            => OUT "<(~,A,C) --> (~,B,C)>" in outputs
        }

        compose_both_diff_int_answer: {
            "
            nse <A --> B>.
            nse <(~,A,C) --> (~,B,C)>?
            cyc 20
            "
            => ANSWER "<(~,A,C) --> (~,B,C)>" in outputs
        }

        compose_both_diff_int_rev: {
            "
            nse <A --> B>.
            nse (~,C,A).
            cyc 10
            "
            => OUT "<(~,C,B) --> (~,C,A)>" in outputs
        }

        compose_both_diff_int_rev_answer: {
            "
            nse <A --> B>.
            nse <(~,C,B) --> (~,C,A)>?
            cyc 20
            "
            => ANSWER "<(~,C,B) --> (~,C,A)>" in outputs
        }

        compose_both_product: {
            "
            nse <A --> B>.
            nse (*,C,A).
            cyc 10
            "
            => OUT "<(*,C,A) --> (*,C,B)>" in outputs
        }

        compose_both_product_answer: {
            "
            nse <A --> B>.
            nse <(*,C,A) --> (*,C,B)>?
            cyc 20
            "
            => ANSWER "<(*,C,A) --> (*,C,B)>" in outputs
        }

        compose_both_image_ext_1: { // ? ❓【2024-08-05 22:36:17】为何这里要反过来？仍然不明确
            "
            nse <R --> S>.
            nse (/,R,_,A).
            cyc 10
            "
            => OUT "<(/,R,_,A) --> (/,S,_,A)>" in outputs
        }

        compose_both_image_ext_1_answer: { // ? ❓【2024-08-05 22:36:17】为何这里要反过来？仍然不明确
            "
            nse <R --> S>.
            nse <(/,R,_,A) --> (/,S,_,A)>?
            cyc 20
            "
            => ANSWER "<(/,R,_,A) --> (/,S,_,A)>" in outputs
        }

        compose_both_image_ext_2: {
            "
            nse <A --> B>.
            nse (/,R,_,A).
            cyc 10
            "
            => OUT "<(/,R,_,B) --> (/,R,_,A)>" in outputs
        }

        compose_both_image_ext_2_answer: {
            "
            nse <A --> B>.
            nse <(/,R,_,B) --> (/,R,_,A)>?
            cyc 20
            "
            => ANSWER "<(/,R,_,B) --> (/,R,_,A)>" in outputs
        }

        compose_both_image_int_1: { // ? ❓【2024-08-05 22:36:17】为何这里要反过来？仍然不明确
            r"
            nse <R --> S>.
            nse (\,R,_,A).
            cyc 10
            "
            => OUT r"<(\,R,_,A) --> (\,S,_,A)>" in outputs
        }

        compose_both_image_int_1_answer: { // ? ❓【2024-08-05 22:36:17】为何这里要反过来？仍然不明确
            r"
            nse <R --> S>.
            nse <(\,R,_,A) --> (\,S,_,A)>?
            cyc 20
            "
            => ANSWER r"<(\,R,_,A) --> (\,S,_,A)>" in outputs
        }

        compose_both_image_int_2: {
            r"
            nse <A --> B>.
            nse (\,R,_,A).
            cyc 10
            "
            => OUT r"<(\,R,_,B) --> (\,R,_,A)>" in outputs
        }

        compose_both_image_int_2_answer: {
            r"
            nse <A --> B>.
            nse <(\,R,_,B) --> (\,R,_,A)>?
            cyc 20
            "
            => ANSWER r"<(\,R,_,B) --> (\,R,_,A)>" in outputs
        }

        decompose_both_int_ext: {
            "
            nse <(&,A,C) --> (&,B,C)>.
            cyc 20
            "
            => OUT "<A --> B>" in outputs
        }

        decompose_both_int_ext_answer: {
            "
            nse <(&,A,C) --> (&,B,C)>.
            nse <A --> B>?
            cyc 30
            "
            => ANSWER "<A --> B>" in outputs
        }

        decompose_both_int_int: {
            "
            nse <(|,A,C) --> (|,B,C)>.
            cyc 20
            "
            => OUT "<A --> B>" in outputs
        }

        decompose_both_int_int_answer: {
            "
            nse <(|,A,C) --> (|,B,C)>.
            nse <A --> B>?
            cyc 30
            "
            => ANSWER "<A --> B>" in outputs
        }

        decompose_both_diff_ext: {
            "
            nse <(-,A,C) --> (-,B,C)>.
            cyc 20
            "
            => OUT "<A --> B>" in outputs
        }

        decompose_both_diff_ext_answer: {
            "
            nse <(-,A,C) --> (-,B,C)>.
            nse <A --> B>?
            cyc 30
            "
            => ANSWER "<A --> B>" in outputs
        }

        decompose_both_diff_ext_rev: {
            "
            nse <(-,C,B) --> (-,C,A)>.
            cyc 20
            "
            => OUT "<A --> B>" in outputs
        }

        decompose_both_diff_ext_rev_answer: {
            "
            nse <(-,C,B) --> (-,C,A)>.
            nse <A --> B>?
            cyc 30
            "
            => ANSWER "<A --> B>" in outputs
        }

        decompose_both_diff_int: {
            "
            nse <(~,A,C) --> (~,B,C)>.
            cyc 20
            "
            => OUT "<A --> B>" in outputs
        }

        decompose_both_diff_int_answer: {
            "
            nse <(~,A,C) --> (~,B,C)>.
            nse <A --> B>?
            cyc 30
            "
            => ANSWER "<A --> B>" in outputs
        }

        decompose_both_diff_int_rev: {
            "
            nse <(~,C,B) --> (~,C,A)>.
            cyc 20
            "
            => OUT "<A --> B>" in outputs
        }

        decompose_both_diff_int_rev_answer: {
            "
            nse <(~,C,B) --> (~,C,A)>.
            nse <A --> B>?
            cyc 30
            "
            => ANSWER "<A --> B>" in outputs
        }

        decompose_both_product: {
            "
            nse <(*,C,A) --> (*,C,B)>.
            cyc 20
            "
            => OUT "<A --> B>" in outputs
        }

        decompose_both_product_answer: {
            "
            nse <(*,C,A) --> (*,C,B)>.
            nse <A --> B>?
            cyc 30
            "
            => ANSWER "<A --> B>" in outputs
        }

        decompose_both_image_ext_1: { // ? ❓【2024-08-05 22:36:17】为何这里要反过来？仍然不明确
            "
            nse <(/,R,_,A) --> (/,S,_,A)>.
            cyc 20
            "
            => OUT "<R --> S>" in outputs
        }

        decompose_both_image_ext_1_answer: { // ? ❓【2024-08-05 22:36:17】为何这里要反过来？仍然不明确
            "
            nse <(/,R,_,A) --> (/,S,_,A)>.
            nse <R --> S>?
            cyc 30
            "
            => ANSWER "<R --> S>" in outputs
        }

        decompose_both_image_ext_2: {
            "
            nse <(/,R,_,B) --> (/,R,_,A)>.
            cyc 20
            "
            => OUT "<A --> B>" in outputs
        }

        decompose_both_image_ext_2_answer: {
            "
            nse <(/,R,_,B) --> (/,R,_,A)>.
            nse <A --> B>?
            cyc 30
            "
            => ANSWER "<A --> B>" in outputs
        }

        decompose_both_image_int_1: { // ? ❓【2024-08-05 22:36:17】为何这里要反过来？仍然不明确
            r"
            nse <(\,R,_,A) --> (\,S,_,A)>.
            cyc 20
            "
            => OUT r"<R --> S>" in outputs
        }

        decompose_both_image_int_1_answer: { // ? ❓【2024-08-05 22:36:17】为何这里要反过来？仍然不明确
            r"
            nse <(\,R,_,A) --> (\,S,_,A)>.
            nse <R --> S>?
            cyc 30
            "
            => ANSWER r"<R --> S>" in outputs
        }

        decompose_both_image_int_2: {
            r"
            nse <(\,R,_,B) --> (\,R,_,A)>.
            cyc 20
            "
            => OUT r"<A --> B>" in outputs
        }

        decompose_both_image_int_2_answer: {
            r"
            nse <(\,R,_,B) --> (\,R,_,A)>.
            nse <A --> B>?
            cyc 30
            "
            => ANSWER r"<A --> B>" in outputs
        }

    }
}
