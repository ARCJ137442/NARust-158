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
use SyllogismPosition::*;

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
        // * 🚩外延像/内涵像 且 索引【不在占位符上】
        // * 📄"<A --> B>" => "<(/, R, _, B) --> (/, R, _, A)>"
        // * 💭"<A --> B>" => "<(/, A, _, C) --> (/, B, _, C)>"
        // * ✅【2024-07-22 14:49:59】上述例子均以ANSWER验证
        || (compound.instanceof_image() && index != compound.get_placeholder_index())
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
    let copula = statement.identifier().to_owned();
    let [statement_sub, statement_pre] = statement.unwrap_components();
    let mut components = compound.get_ref().clone_components();
    let other_statement_component = side.opposite().select([&statement_sub, &statement_pre]);
    if components.contains(other_statement_component) {
        // * 📝复合词项包含陈述的另一侧词项 ⇒ 中止
        // * 📄compound = "(*,{tom},(&,glasses,[black]))" @ 1 => "(&,glasses,[black])"
        // * * statement = "<(&,glasses,sunglasses) --> (&,glasses,[black])>" @ 0
        // * * components = ["{tom}", "(&,glasses,[black])"]
        // * * ⇒不处理（❓为何如此）
        return;
    }
    /* if match side {
        Subject => components.contains(statement_predicate),
        Predicate => components.contains(statement_subject),
    } {
        return;
    } */
    let [sub, pre] = match side {
        Subject if components.contains(&statement_sub) => [
            // * 🚩主项：原来的复合词项
            compound.get_ref().inner.clone(),
            // * 🚩谓项：替换后的复合词项
            {
                components[index] = statement_pre;
                unwrap_or_return!(
                    ?Term::make_compound_term(compound.get_ref(), components)
                )
            },
        ],
        Predicate if components.contains(&statement_pre) => [
            // * 🚩主项：替换后的复合词项
            {
                components[index] = statement_sub;
                unwrap_or_return!(
                    ?Term::make_compound_term(compound.get_ref(), components)
                )
            },
            // * 🚩谓项：原来的复合词项
            compound.get_ref().inner.clone(),
        ],
        // TODO: 【2024-08-05 17:47:15】后续或可简化
        _ => [statement_sub, statement_pre],
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
        // TODO: 更多测试
    }
}
