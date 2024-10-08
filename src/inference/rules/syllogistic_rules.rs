//! 三段论规则
//! * 🚩【2024-07-11 00:07:34】目前只包含「具体规则处理」两部分
//!   * 📝OpenNARS中「规则表」可能会在某些地方直接分派规则
//!   * 📄条件三段论系列
//!
//! ## Logs
//!
//! * ♻️【2024-07-11 00:07:52】开始根据改版OpenNARS重写
//! * ✅【2024-08-05 17:33:06】基本功能重写完成

use crate::{
    control::*,
    entity::*,
    inference::{
        rules::{cast_statement, utils::*},
        *,
    },
    language::*,
    symbols::CONJUNCTION_OPERATOR,
};
use nar_dev_utils::{unwrap_or_return, RefCount};
use ReasonDirection::*;

/// 🆕演绎规则
pub fn deduction(
    sub: Term,
    pre: Term,
    task: &impl Sentence,
    belief: &impl Judgement,
    context: &mut ReasonContextConcept,
) {
    // * 🚩词项
    let content = unwrap_or_return!(
        ?Term::make_statement(task.content(), sub, pre)
    );
    // * 🚩真值
    let truth = match context.reason_direction() {
        Forward => Some(task.unwrap_judgement().deduction(belief)),
        Backward => None,
    };
    // * 🚩预算
    let budget = match context.reason_direction() {
        Forward => context.budget_forward(truth.as_ref()),
        Backward => context.budget_backward_weak(belief),
    };
    // * 🚩结论
    context.double_premise_task(content, truth, budget);
}

/// 🆕举例规则
pub fn exemplification(
    sub: Term,
    pre: Term,
    task: &impl Sentence,
    belief: &impl Judgement,
    context: &mut ReasonContextConcept,
) {
    // * 🚩词项
    let content = unwrap_or_return!(
        ?Term::make_statement(task.content(), pre, sub)
    );
    // * 🚩真值
    let truth = match context.reason_direction() {
        Forward => Some(task.unwrap_judgement().exemplification(belief)),
        Backward => None,
    };
    // * 🚩预算
    let budget = match context.reason_direction() {
        Forward => context.budget_forward(truth.as_ref()),
        Backward => context.budget_backward_weak(belief),
    };
    // * 🚩结论
    context.double_premise_task(content, truth, budget);
}

/// 🆕归因规则
pub fn abduction(
    sub: Term,
    pre: Term,
    task: &impl Sentence,
    belief: &impl Judgement,
    context: &mut ReasonContextConcept,
) {
    // * 🚩词项
    let content = unwrap_or_return!(
        ?Term::make_statement(task.content(), sub, pre)
    );
    // * 🚩真值
    let truth = match context.reason_direction() {
        Forward => Some(task.unwrap_judgement().abduction(belief)),
        Backward => None,
    };
    // * 🚩预算
    let budget = match context.reason_direction() {
        Forward => context.budget_forward(truth.as_ref()),
        Backward => context.budget_backward(belief),
    };
    // * 🚩结论
    context.double_premise_task(content, truth, budget);
}

/// 🆕归纳规则
pub fn induction(
    sub: Term,
    pre: Term,
    task: &impl Sentence,
    belief: &impl Judgement,
    context: &mut ReasonContextConcept,
) {
    // * 🚩词项
    let content = unwrap_or_return!(
        ?Term::make_statement(task.content(), pre, sub)
    );
    // * 🚩真值
    let truth = match context.reason_direction() {
        Forward => Some(task.unwrap_judgement().induction(belief)),
        Backward => None,
    };
    // * 🚩预算
    let budget = match context.reason_direction() {
        Forward => context.budget_forward(truth.as_ref()),
        Backward => context.budget_backward_weak(belief),
    };
    // * 🚩结论
    context.double_premise_task(content, truth, budget);
}

/// 🆕比较规则
pub fn comparison(
    sub: Term,
    pre: Term,
    task: &impl Sentence,
    belief: &impl Judgement,
    context: &mut ReasonContextConcept,
) {
    // * 🚩词项
    let content = unwrap_or_return!(
        ?Term::make_statement_symmetric(task.content(), sub, pre)
    );
    // * 🚩真值
    let truth = match context.reason_direction() {
        Forward => Some(task.unwrap_judgement().comparison(belief)),
        Backward => None,
    };
    // * 🚩预算
    let budget = match context.reason_direction() {
        Forward => context.budget_forward(truth.as_ref()),
        Backward => context.budget_backward(belief),
    };
    // * 🚩结论
    context.double_premise_task(content, truth, budget);
}

/// {<S ==> P>, <M <=> P>} |- <S ==> P>
/// * 📌类比
/// * 📝【2024-07-02 13:27:22】弱推理🆚强推理、前向推理🆚反向推理 不是一个事儿
pub fn analogy(
    sub: Term,
    pre: Term,
    asymmetric: impl Sentence,
    symmetric: impl Sentence,
    context: &mut ReasonContextConcept,
) {
    // * 🚩验明合法性
    if StatementRef::invalid_statement(&sub, &pre) {
        return;
    }
    // * 🚩提取参数
    let task_rc = context.current_task();
    let task = task_rc.get_();
    let direction = context.reason_direction();
    let task_content = task.content();
    // * 🚩词项
    // * 📝取「反对称」那个词项的系词
    let asymmetric_statement = asymmetric.content().as_statement().unwrap();
    let content = unwrap_or_return!(?Term::make_statement(&asymmetric_statement, sub, pre));

    // * 🚩真值
    let truth = match direction {
        Forward => Some(
            asymmetric
                .unwrap_judgement()
                .analogy(symmetric.unwrap_judgement()),
        ),
        Backward => None,
    };
    // * 🚩预算
    let is_commutative = task_content.is_commutative();
    drop(task);
    drop(task_rc);
    let budget = match direction {
        Forward => context.budget_forward(truth.as_ref()),
        Backward => {
            match is_commutative {
                // * 🚩可交换⇒弱推理
                true => context.budget_backward_weak(asymmetric.unwrap_judgement()),
                // * 🚩不可交换⇒强推理
                false => context.budget_backward(symmetric.unwrap_judgement()),
            }
        }
    };
    // * 🚩结论
    context.double_premise_task(content, truth, budget);
}

/// * 📝条件归因，消去S3、P，可能构造<S1 ==> S2>也可能构造<S2 ==> S1>
/// * 🚩返回「是否应用成功」，用于规则表分派
///
/// # 📄OpenNARS
///
/// `{<(&&, S2, S3) ==> P>, <(&&, S1, S3) ==> P>} |- {<S1 ==> S2>, <S2 ==> S1>}`
pub fn conditional_abduction(
    condition_t: &Term,
    condition_b: &Term,
    statement_t: &Statement,
    statement_b: &Statement,
    context: &mut ReasonContextConcept,
) -> bool {
    // * 🚩检验合法性 * //
    if !statement_t.instanceof_implication() || !statement_b.instanceof_implication() {
        return false;
    }
    // * 📝此中的「条件」可以是单独的词项，也可以是一个合取
    // * 【2024-08-04 22:05:53】或许就直接拿「单独词项/合取词项」来表达？
    let [conjunction_t, conjunction_b] = match [
        condition_t.as_compound_type(CONJUNCTION_OPERATOR),
        condition_b.as_compound_type(CONJUNCTION_OPERATOR),
    ] {
        // OpenNARS原意：除了「俩都不是合取」的情况，都通过（允许不是合取）
        /* [Some(conjunction_t), Some(conjunction_b)] => [conjunction_t, conjunction_b],
        _ => return false, */
        [None, None] => return false,
        options => options,
    };

    // * 🚩提取参数 * //
    let task_truth = context
        .current_task()
        .get_()
        .as_judgement()
        .map(TruthValue::from);
    let belief_truth = TruthValue::from(unwrap_or_return!(
        ?context.current_belief() => false
    ));
    let direction = context.reason_direction();

    // * 🚩预置词项：分别消去彼此间的「内含条件」
    let reduced_t =
        // if ((cond1 instanceof Conjunction) &&
        // !Variable.containVarDep(cond1.getName())) {
        // * 🚩逻辑：若为合取，尝试消去元素并制作新词项；制作新词项失败时，亦为None
        conjunction_t.and_then(|conjunction_t| conjunction_t.reduce_components(condition_b));
    let reduced_b =
        // if ((cond2 instanceof Conjunction) &&
        // !Variable.containVarDep(cond2.getName())) {
        // * 🚩逻辑：若为合取，尝试消去元素并制作新词项；制作新词项失败时，亦为None
        conjunction_b.and_then(|conjunction_b| conjunction_b.reduce_components(condition_t));

    // * 📌【2024-08-04 23:34:14】后续取逻辑或，此处费事再判断一次
    /* // * 🚩都消没了⇒推理失败
    if reduced_t.is_none() && reduced_b.is_none() {
        return false;
    } */

    // * 🚩利用「左右共通逻辑」把代码简化到一个闭包中，后续只需「往返调用」即可
    //   * ℹ️闭包捕获「推理上下文」作为参数，在调用时无需重复声明与附带
    //   * 📝利用「带标签代码块」做逻辑控制
    let mut derive = |other_statement,
                      [self_condition, other_condition]: [&Option<Term>; 2],
                      [self_truth, other_truth]: [&Option<TruthValue>; 2]| 'derive: {
        // * 🚩前提条件 * //
        // OpenNARS源码@信念端：`if (term2 != null)`
        let self_condition = unwrap_or_return! {
            ?self_condition => break 'derive false // 💭若条件没提取出来，还是算了
        };
        // * 🚩词项 * //
        let content = match other_condition {
            // * 🚩仍然是条件句
            // OpenNARS源码@信念端：`makeStatement(st1, term1, term2)`
            Some(other_condition) => unwrap_or_return!(
                ?Term::make_statement(other_statement, other_condition.clone(), self_condition.clone())
                => break 'derive false // 💭制作失败就别求得出啥结论了
            ),
            // * 🚩只剩下条件
            None => self_condition.clone(),
        };
        // * 🚩真值 * //
        let truth = match direction {
            // * 🚩类比
            Forward => {
                // 解包两个真值
                // * 📝不知从任务来，还是从信念来；至少在正向推理时都在
                let [self_truth, other_truth] = [
                    unwrap_or_return!(?self_truth => break 'derive false),
                    unwrap_or_return!(?other_truth => break 'derive false),
                ];
                // 计算 @ 归因
                Some(other_truth.abduction(self_truth))
            }
            Backward => None,
        };
        // * 🚩预算 * //
        let budget = match direction {
            Forward => context.budget_forward(truth.as_ref()),
            // * 🚩反向 ⇒ 弱 | 此处的真值恒取自于信念
            Backward => context.budget_backward_weak(&belief_truth),
        };
        // * 🚩结论 * //
        context.double_premise_task(content, truth, budget);
        // * 🚩匹配成功
        true
    };
    // * 🚩往返调用
    let [derived_t, derived_b] = [
        // 任务→信念
        derive(
            statement_b,
            [&reduced_t, &reduced_b],
            [&task_truth, &Some(belief_truth)],
        ),
        // 信念→任务
        derive(
            statement_t,
            [&reduced_b, &reduced_t],
            [&Some(belief_truth), &task_truth],
        ),
    ];
    // * 🚩其中一个匹配成功才算成功 | ⚠️不同于OpenNARS，此处更为精确
    derived_t || derived_b
}

/// * 📝条件演绎/条件归纳
/// * ♻️【2024-08-05 15:31:25】不再直接传入「信念」句：可能其中的内容是旧的
///   * ⚠️在调用此方法前，有可能经过了「变量归一化」的过程
///
/// ```nal
/// {<(&&, S1, S2, S3) ==> P>, S1} |- <(&&, S2, S3) ==> P>
/// {<(&&, S2, S3) ==> P>, <S1 ==> S2>} |- <(&&, S1, S3) ==> P>
/// {<(&&, S1, S3) ==> P>, <S1 ==> S2>} |- <(&&, S2, S3) ==> P>
/// ```
pub fn conditional_deduction_induction(
    conditional: Statement,
    index_in_condition: usize,
    premise2: Term,
    belief_truth: &impl Truth,
    conditional_from: PremiseSource, // ! 📝【2024-08-05 01:15:51】暂时用不着：「当前任务是否为条件句」不重要
    side: SyllogismSide,
    context: &mut ReasonContextConcept,
) {
    use SyllogismSide::*;
    let [rng_seed, rng_seed2, rng_seed3] = context.shuffle_rng_seeds();

    // * 🚩提取参数 * //
    let task_truth = context
        .current_task()
        .get_()
        .as_judgement()
        .map(TruthValue::from);
    // * 🚩若条件句来自任务，则取premise2作为「信念内容」；否则取来自信念的conditional
    // * ✅【2024-08-05 15:29:10】经测试基本成功
    // println!("{unified_belief_content} 🆚 {}", belief.content());
    let [_, unified_belief_content] = conditional_from.select([&*conditional, &premise2]);
    let conditional_task =
        variable_process::has_unification_i(&premise2, unified_belief_content, rng_seed);
    let direction = context.reason_direction();
    let deduction = side != Subject;

    // * 🚩词项 * //
    // * 🚩获取公共项
    /* 📝此处「互斥性选择」对应以下逻辑：
    if (side == 0) { // * 在主项
        commonComponent = ((Statement) premise2).getSubject();
        newComponent = ((Statement) premise2).getPredicate();
    } else if (side == 1) { // * 在谓项
        commonComponent = ((Statement) premise2).getPredicate();
        newComponent = ((Statement) premise2).getSubject();
    } else { // * 整个词项
        commonComponent = premise2;
        newComponent = null;
    } */
    let [common_component, new_component] = side.select_exclusive(&premise2);
    let common_component = common_component.expect("应该有提取到");
    // * 🚩获取「条件句」的条件
    let old_condition = unwrap_or_return!(
        ?conditional.get_ref().subject.as_compound_type(CONJUNCTION_OPERATOR)
    );
    // * 🚩根据「旧条件」选取元素（或应用「变量统一」）
    let index_2 = old_condition.index_of_component(common_component);
    let index_in_old_condition;
    let conditional_unified; // 经过（潜在的）「变量统一」之后的「前提1」
    if let Some(index_2) = index_2 {
        index_in_old_condition = index_2;
        conditional_unified = conditional.clone();
    } else {
        // * 🚩尝试数次匹配，将其中的变量归一化
        // * 📝两次尝试的变量类型相同，但应用的位置不同
        index_in_old_condition = index_in_condition;
        let condition_to_unify = unwrap_or_return!(
            ?old_condition.component_at(index_in_old_condition)
        );
        let unification_i =
            variable_process::unify_find_i(condition_to_unify, common_component, rng_seed2);
        if unification_i.has_unification {
            let mut to_be_apply = conditional.clone();
            unification_i
                .unify_map_1
                .apply_to(to_be_apply.mut_ref().into_compound_ref());
            conditional_unified = to_be_apply;
        } else if common_component.is_same_type(&old_condition) {
            let common_component_component = unwrap_or_return!(
                ?common_component
                .as_compound()
                .unwrap()
                .component_at(index_in_old_condition)
            );
            // * 🚩尝试寻找并应用变量归一化 @ 共同子项
            let unification_i = variable_process::unify_find_i(
                condition_to_unify,
                common_component_component,
                rng_seed3,
            );
            if unification_i.has_unification {
                let mut to_be_apply = conditional.clone();
                unification_i
                    .unify_map_1
                    .apply_to(to_be_apply.mut_ref().into_compound_ref());
                conditional_unified = to_be_apply;
            } else {
                return;
            }
        } else {
            return;
        }
    }
    // * 🚩构造「新条件」
    let new_condition = match old_condition.inner == common_component {
        true => None,
        false => old_condition.set_component(index_in_old_condition, new_component.cloned()),
    };
    // * 🚩根据「新条件」构造新词项
    let (_, copula, predicate) = conditional_unified.unwrap();
    let content = match new_condition {
        Some(new_condition) => {
            unwrap_or_return!(?Term::make_statement_relation(copula, new_condition, predicate))
        }
        None => predicate,
    };

    // * 🚩真值 * //
    let truth = match direction {
        Forward => Some(match deduction {
            true => task_truth.unwrap().deduction(belief_truth),
            // * 🚩演绎 ⇒ 演绎
            false => match conditional_task {
                // * 🚩任务是条件句 ⇒ 归纳（任务→信念，就是反过来的归因）
                true => belief_truth.induction(&task_truth.unwrap()),
                // * 🚩其它 ⇒ 归纳（信念⇒任务）
                false => task_truth.unwrap().induction(belief_truth),
            },
        }),
        Backward => None,
    };

    // * 🚩预算 * //
    let budget = match direction {
        // * 🚩前向
        Forward => context.budget_forward(&truth.unwrap()),
        // * 🚩反向⇒弱推理
        Backward => context.budget_backward_weak(belief_truth),
    };

    // * 🚩结论 * //
    context.double_premise_task(content, truth, budget);
}

/// {<(&&, S1, S2) <=> P>, (&&, S1, S2)} |- P
/// * 📝条件类比
/// * 💭【2024-07-09 18:18:41】实际上是死代码
///   * 📄禁用「等价⇒复合条件」后，「等价」不再能自`compoundAndCompoundCondition`分派
///   * 📌【2024-08-05 15:57:25】替代式推理路径：等价→蕴含 + 条件演绎/条件归纳
pub fn conditional_analogy(
    mut belief_equivalence: Statement, // 前提1
    index_in_condition: usize,
    mut task_implication: Statement, // 前提2
    common_term_side: SyllogismSide,
    belief_truth: &impl Truth,
    context: &mut ReasonContextConcept,
) {
    let [rng_seed1, rng_seed2, rng_seed3] = context.shuffle_rng_seeds();
    // * 🚩提取参数 * //
    let task_truth: Option<TruthValue> = context
        .current_task()
        .get_()
        .as_judgement()
        .map(TruthValue::from);
    let direction = context.reason_direction();
    let conditional_task =
        variable_process::has_unification_i(&task_implication, &belief_equivalence, rng_seed1);

    // * 🚩词项 * //
    let [common_component, _] = common_term_side.select_exclusive(&task_implication);
    let common_component = common_component.expect("应该有提取到");
    // * 🚩尝试消解条件中的变量，匹配数次未果则返回
    let old_condition = unwrap_or_return!(
        ?belief_equivalence.get_ref().subject.as_compound_type(CONJUNCTION_OPERATOR)
    );
    let common_in_condition = old_condition.component_at(index_in_condition).unwrap();
    let unification_d =
        variable_process::unify_find_d(common_in_condition, common_component, rng_seed2);
    let unification = if unification_d.has_unification {
        unification_d
    } else if common_component.is_same_type(&old_condition) {
        let common_inner = common_component
            .as_compound()
            .unwrap()
            .component_at(index_in_condition)
            .unwrap();
        let unification_d =
            variable_process::unify_find_d(common_in_condition, common_inner, rng_seed3);
        if unification_d.has_unification {
            unification_d
        } else {
            return; // 失败⇒中止
        }
    } else {
        return; // 失败⇒中止
    };
    unification.apply_to(
        belief_equivalence.mut_ref().into_compound_ref(),
        task_implication.mut_ref().into_compound_ref(),
    );
    // 构造新条件词项
    let [common_component, new_component] = common_term_side.select_exclusive(&task_implication);
    let common_component = common_component.expect("应该有提取到");
    let old_condition = unwrap_or_return!(
        ?belief_equivalence.get_ref().subject.as_compound_type(CONJUNCTION_OPERATOR)
    );
    let new_condition = match *old_condition == *common_component {
        true => None,
        false => old_condition.set_component(index_in_condition, new_component.cloned()),
    };
    let (_, copula, premise1_predicate) = belief_equivalence.unwrap();
    let content = match new_condition {
        Some(new_condition) => unwrap_or_return!(
            ?Term::make_statement_relation(copula, new_condition, premise1_predicate)
        ),
        None => premise1_predicate,
    };

    // * 🚩真值 * //
    let truth = match direction {
        Forward => Some(match conditional_task {
            // * 🚩条件性任务 ⇒ 比较
            true => task_truth.unwrap().comparison(belief_truth),
            // * 🚩其它 ⇒ 类比
            false => task_truth.unwrap().analogy(belief_truth),
        }),
        Backward => None,
    };

    // * 🚩预算 * //
    let budget = match direction {
        // * 🚩前向
        Forward => context.budget_forward(&truth.unwrap()),
        // * 🚩反向⇒弱推理
        Backward => context.budget_backward_weak(belief_truth),
    };

    // * 🚩结论 * //
    context.double_premise_task(content, truth, budget);
}

/// {<S --> P>, <P --> S} |- <S <-> p>
/// Produce Similarity/Equivalence from a pair of reversed
/// Inheritance/Implication
/// * 📝非对称⇒对称（前向推理）
pub fn infer_to_sym(
    judgement1: &impl Judgement,
    judgement2: &impl Judgement,
    context: &mut ReasonContextConcept,
) {
    // * 🚩词项 * //
    let [sub, pre] = cast_statement(judgement1.content().clone()).unwrap_components();
    let content = unwrap_or_return!(
        ?Term::make_statement_symmetric(judgement1.content(), sub, pre)
    );

    // * 🚩真值 * //
    let truth = judgement1.intersection(judgement2);

    // * 🚩预算 * //
    let budget = context.budget_forward(&truth);

    // * 🚩结论 * //
    context.double_premise_task(content, Some(truth), budget);
}

/// * 📝对称⇒非对称（前向推理）
///
/// # 📄OpenNARS
///
/// {<S <-> P>, <P --> S>} |- <S --> P> Produce an Inheritance/Implication
/// from a Similarity/Equivalence and a reversed Inheritance/Implication
pub fn infer_to_asy(
    asy: &impl Judgement,
    sym: &impl Judgement,
    context: &mut ReasonContextConcept,
) {
    // * 🚩词项 * //
    // * 🚩提取 | 📄<S --> P> => S, P
    // * 🚩构建新的相反陈述 | 📄S, P => <P --> S>
    let [pre, sub] = cast_statement(asy.content().clone()).unwrap_components();
    let content = unwrap_or_return!(
        ?Term::make_statement(asy.content(), sub, pre)
    );

    // * 🚩真值 * //
    let truth = sym.reduce_conjunction(asy);

    // * 🚩预算 * //
    let budget = context.budget_forward(&truth);

    // * 🚩结论 * //
    context.double_premise_task(content, Some(truth), budget);
}

/// * 📝转换（反向推理，但使用前向预算值）
///
/// # 📄OpenNARS
///
/// {<P --> S>} |- <S --> P> Produce an Inheritance/Implication from a
/// reversed Inheritance/Implication
pub fn conversion(belief: &impl Judgement, context: &mut ReasonContextConcept) {
    // * 🚩真值 * //
    let truth = belief.conversion();

    // * 🚩预算 * //
    let budget = context.budget_forward(&truth);

    // * 🚩转发到统一的逻辑
    converted_judgment(truth, budget, context);
}

/// * 📝非对称⇔对称
///
/// # 📄OpenNARS
///
/// {<S --> P>} |- <S <-> P>
/// {<S <-> P>} |- <S --> P> Switch between
/// Inheritance/Implication and Similarity/Equivalence
pub fn convert_relation(task_question: &impl Question, context: &mut ReasonContextConcept) {
    // * 🚩真值 * //
    // * 🚩基于「当前信念」
    let belief = unwrap_or_return!(
        ?context.current_belief()
    );
    let truth = match task_question.content().is_commutative() {
        // * 🚩可交换（相似/等价）⇒归纳
        true => belief.analytic_abduction(ShortFloat::ONE),
        // * 🚩不可交换（继承/蕴含）⇒演绎
        false => belief.analytic_deduction(ShortFloat::ONE),
    };
    // * 🚩预算 * //
    let budget = context.budget_forward(&truth);
    // * 🚩继续向下分派函数
    converted_judgment(truth, budget, context);
}

/// # 📄OpenNARS
///
/// Convert judgment into different relation
///
/// called in MatchingRules
pub fn converted_judgment(
    new_truth: TruthValue,
    new_budget: BudgetValue,
    context: &mut ReasonContextConcept,
) {
    // * 🚩词项 * //
    let task_content = cast_statement(context.current_task().get_().content().clone());
    let belief_content = cast_statement(
        context
            .current_belief()
            .expect("概念推理一定有当前信念")
            .content()
            .clone(),
    );
    let (sub_t, copula, pre_t) = task_content.unwrap();
    let [sub_b, pre_b] = belief_content.unwrap_components();
    // * 🚩创建内容 | ✅【2024-06-10 10:26:14】已通过「长期稳定性」验证与原先逻辑的稳定
    let [sub, pre] = match [sub_t.contain_var_q(), pre_t.contain_var_q()] {
        // * 🚩谓项有查询变量⇒用「信念主项/信念谓项」替换
        [_, true] => {
            let eq_sub_t = sub_t == sub_b; // ! 欠一致：后初始化的要用到先初始化的，导致需要提取变量
            [
                sub_t,
                match eq_sub_t {
                    true => pre_b,
                    false => sub_b,
                },
            ]
        }
        // * 🚩主项有查询变量⇒用「信念主项/信念谓项」替换
        [true, _] => [
            match pre_t == sub_b {
                true => pre_b,
                false => sub_b,
            },
            pre_t,
        ],
        // * 🚩否则：直接用「任务主项&任务谓项」替换
        _ => [sub_t, pre_t],
    };
    let content = unwrap_or_return!(?Term::make_statement_relation(copula, sub, pre));

    // * 🚩结论 * //
    context.single_premise_task_full(content, Punctuation::Judgement, Some(new_truth), new_budget)
}

/// 相似传递
///
/// # 📄OpenNARS
///
/// `{<S <=> M>, <M <=> P>} |- <S <=> P>`
pub fn resemblance(
    sub: Term,
    pre: Term,
    belief: &impl Judgement,
    task: &impl Sentence,
    context: &mut ReasonContextConcept,
) {
    // * 🚩合法性
    if StatementRef::invalid_statement(&sub, &pre) {
        return;
    }
    // * 🚩提取参数
    let direction = context.reason_direction();
    // * 🚩词项
    let content = unwrap_or_return!(
        ?Term::make_statement(belief.content(), sub, pre)
    );
    // * 🚩真值
    let truth = match direction {
        Forward => Some(belief.resemblance(task.unwrap_judgement())),
        Backward => None,
    };
    // * 🚩预算
    let budget = match direction {
        Forward => context.budget_forward(truth.as_ref()),
        Backward => context.budget_backward(belief),
    };
    // * 🚩结论
    context.double_premise_task(content, truth, budget);
}

/// ```nal
/// {<<M --> S> ==> <M --> P>>, <M --> S>} |- <M --> P>
/// {<<M --> S> ==> <M --> P>>, <M --> P>} |- <M --> S>
/// {<<M --> S> <=> <M --> P>>, <M --> S>} |- <M --> P>
/// {<<M --> S> <=> <M --> P>>, <M --> P>} |- <M --> S>
/// ```
///
/// * 📝分离规则
/// * 🚩由规则表直接分派
pub fn detachment(
    task_sentence: &impl Sentence,
    belief: &impl Judgement,
    high_order_position: PremiseSource,
    position_sub_in_hi: SyllogismPosition,
    context: &mut ReasonContextConcept,
) {
    // * 🚩合法性
    let [high_order_statement, _] =
        high_order_position.select([task_sentence.content(), belief.content()]); // 按位置选取高阶陈述
    if !(high_order_statement.instanceof_implication()
        || high_order_statement.instanceof_equivalence())
    {
        return;
    }

    // * 🚩提取参数
    let high_order_statement = cast_statement(high_order_statement.clone());

    let high_order_symmetric = high_order_statement.is_commutative(); // * 📌用于替代OpenNARS源码后边的「是否为等价」（除了那里其它地方用不到，后边直接unwrap）
    let [sub, pre] = high_order_statement.unwrap_components();
    let direction = context.reason_direction();
    // * 🚩词项
    let [_, sub_content] = high_order_position.select([task_sentence.content(), belief.content()]); // 选取另一侧的子内容
    use SyllogismPosition::*;
    let content = match position_sub_in_hi {
        // * 🚩主项&相等⇒取出
        Subject if *sub_content == sub => pre,
        // * 🚩谓项&相等⇒取出
        Predicate if *sub_content == pre => sub,
        // * 🚩其它⇒无效
        _ => return,
    };
    if let Some(statement) = content.as_statement() {
        // * 📄【2024-06-15 11:39:40】可能存在「变量统一」后词项无效的情况
        // * * main"<<bird --> bird> ==> <bird --> swimmer>>"
        // * * content"<bird --> bird>"
        // * * sub"<bird --> swimmer>"
        if statement.invalid() {
            return;
        }
    }
    // * 🚩真值
    let truth = match direction {
        Forward => {
            // 提取主句、副句
            let [main_sentence_truth, sub_sentence_truth] = high_order_position.select([
                TruthValue::from(task_sentence.unwrap_judgement()),
                TruthValue::from(belief),
            ]);
            // 计算真值
            Some(match (high_order_symmetric, position_sub_in_hi) {
                // * 🚩等价⇒类比
                (true, _) => sub_sentence_truth.analogy(&main_sentence_truth),
                // * 🚩非对称 & 主词 ⇒ 演绎
                (_, Subject) => main_sentence_truth.deduction(&sub_sentence_truth),
                // * 🚩其它 ⇒ 归纳
                (_, Predicate) => sub_sentence_truth.abduction(&main_sentence_truth),
            })
        }
        // * 🚩反向推理⇒空
        Backward => None,
    };

    // * 🚩预算
    let budget = match direction {
        Forward => context.budget_forward(&truth.unwrap()), // 前向推理一定产生了真值
        Backward => match (high_order_symmetric, position_sub_in_hi) {
            // * 🚩等价 | 其它 ⇒ 反向
            (true, _) | (_, Predicate) => context.budget_backward(belief),
            // * 🚩非对称 & 主词 ⇒ 反向弱
            (_, Subject) => context.budget_backward_weak(belief),
        },
    };

    // * 🚩结论
    context.double_premise_task(content, truth, budget);
}

#[cfg(test)]
mod tests {
    use crate::expectation_tests;

    expectation_tests! {
        deduction: {
            "
            nse <A --> B>.
            nse <B --> C>.
            cyc 10
            "
            => OUT "<A --> C>" in outputs
        }

        /// ! 【2024-07-23 17:38:57】❓补完NAL-1后，需要的步数更多了
        deduction_answer: {
            "
            nse <A --> B>.
            nse <B --> C>.
            nse <A --> C>?
            cyc 50
            "
            => ANSWER "<A --> C>" in outputs
        }

        deduction_backward: {
            "
            nse <A --> B>.
            nse <?1 --> B>?
            cyc 10
            "
            => OUT "<?1 --> A>" in outputs
        }

        exemplification: {
            "
            nse <A --> B>.
            nse <B --> C>.
            cyc 10
            "
            => OUT "<C --> A>" in outputs
        }

        exemplification_backward: {
            "
            nse <A --> B>.
            nse <?1 --> B>?
            cyc 10
            "
            => OUT "<A --> ?1>" in outputs
        }

        exemplification_answer: {
            "
            nse <A --> B>.
            nse <B --> C>.
            nse <C --> A>?
            cyc 20
            "
            => ANSWER "<C --> A>" in outputs
        }

        abduction_sub: {
            "
            nse <A --> B>.
            nse <A --> C>.
            cyc 10
            "
            => OUT "<B --> C>" in outputs
        }

        abduction_answer_sub: {
            "
            nse <A --> B>.
            nse <A --> C>.
            nse <B --> C>?
            cyc 20
            "
            => ANSWER "<B --> C>" in outputs
        }

        abduction_backward_sub: {
            "
            nse <A --> B>.
            nse <A --> {?1}>?
            cyc 20
            "
            => OUT "<B --> {?1}>" in outputs
        }

        abduction_pre: {
            "
            nse <B --> A>.
            nse <C --> A>.
            cyc 10
            "
            => OUT "<C --> B>" in outputs
        }

        abduction_answer_pre: {
            "
            nse <B --> A>.
            nse <C --> A>.
            nse <C --> B>?
            cyc 20
            "
            => ANSWER "<C --> B>" in outputs
        }

        induction_sub: {
            "
            nse <A --> B>.
            nse <A --> C>.
            cyc 10
            "
            => OUT "<C --> B>" in outputs
        }

        induction_answer_sub: {
            "
            nse <A --> B>.
            nse <A --> C>.
            nse <C --> B>?
            cyc 20
            "
            => ANSWER "<C --> B>" in outputs
        }

        induction_pre: {
            "
            nse <B --> A>.
            nse <C --> A>.
            cyc 10
            "
            => OUT "<B --> C>" in outputs
        }

        induction_answer_pre: {
            "
            nse <B --> A>.
            nse <C --> A>.
            nse <B --> C>?
            cyc 20
            "
            => ANSWER "<B --> C>" in outputs
        }

        comparison_sub: {
            "
            nse <A --> B>.
            nse <A --> C>.
            cyc 10
            "
            => OUT "<B <-> C>" in outputs
        }

        comparison_answer_sub: {
            "
            nse <A --> B>.
            nse <A --> C>.
            nse <B <-> C>?
            cyc 20
            "
            => ANSWER "<B <-> C>" in outputs
        }

        comparison_pre: {
            "
            nse <B --> A>.
            nse <C --> A>.
            cyc 10
            "
            => OUT "<B <-> C>" in outputs
        }

        comparison_answer_pre: {
            "
            nse <B --> A>.
            nse <C --> A>.
            nse <B <-> C>?
            cyc 20
            "
            => ANSWER "<B <-> C>" in outputs
        }

        analogy_sub: {
            "
            nse <A --> B>.
            nse <C <-> A>.
            cyc 10
            "
            => OUT "<C --> B>" in outputs
        }

        analogy_answer_sub: {
            "
            nse <A --> B>.
            nse <C <-> A>.
            nse <C --> B>?
            cyc 20
            "
            => ANSWER "<C --> B>" in outputs
        }

        analogy_pre: {
            "
            nse <A --> B>.
            nse <C <-> A>.
            cyc 10
            "
            => OUT "<C --> B>" in outputs
        }

        analogy_answer_pre: {
            "
            nse <A --> B>.
            nse <C <-> A>.
            nse <C --> B>?
            cyc 20
            "
            => ANSWER "<C --> B>" in outputs
        }

        conversion: {
            "
            nse <A --> B>.
            nse <B --> A>?
            cyc 10
            "
            => ANSWER "<B --> A>" in outputs
        }

        infer_to_asy: {
            "
            nse <A <-> B>.
            nse <A --> B>?
            cyc 10
            "
            => ANSWER "<A --> B>" in outputs
        }

        infer_to_sym: {
            "
            nse <A --> B>.
            nse <A <-> B>?
            cyc 10
            "
            => ANSWER "<A <-> B>" in outputs
        }

        conversion_high: {
            "
            nse <A ==> B>.
            nse <B ==> A>?
            cyc 10
            "
            => ANSWER "<B ==> A>" in outputs
        }

        infer_to_asy_high: {
            "
            nse <A <=> B>.
            nse <A ==> B>?
            cyc 10
            "
            => ANSWER "<A ==> B>" in outputs
        }

        infer_to_sym_high: {
            "
            nse <A ==> B>.
            nse <A <=> B>?
            cyc 10
            "
            => ANSWER "<A <=> B>" in outputs
        }

        resemblance: {
            "
            nse <A <-> B>.
            nse <B <-> C>.
            cyc 10
            "
            => OUT "<A <-> C>" in outputs
        }

        resemblance_answer: {
            "
            nse <A <-> B>.
            nse <B <-> C>.
            nse <A <-> C>?
            cyc 20
            "
            => ANSWER "<A <-> C>" in outputs
        }

        detachment: {
            "
            nse <A ==> B>.
            nse A.
            cyc 10
            "
            => OUT "B" in outputs
        }

        detachment_answer: {
            "
            nse <A ==> B>.
            nse A.
            nse B?
            cyc 20
            "
            => ANSWER "B" in outputs
        }

        detachment_weak: {
            "
            nse <A ==> B>.
            nse B.
            cyc 10
            "
            => OUT "A" in outputs
        }

        detachment_answer_weak: {
            "
            nse <A ==> B>.
            nse B.
            nse A?
            cyc 20
            "
            => ANSWER "A" in outputs
        }

        detachment_var: {
            "
            nse <<$1 --> A> ==> <$1 --> B>>.
            nse <C --> A>.
            cyc 10
            "
            => OUT "<C --> B>" in outputs
        }

        detachment_var_answer: {
            "
            nse <<$1 --> A> ==> <$1 --> B>>.
            nse <C --> A>.
            nse <C --> B>?
            cyc 20
            "
            => ANSWER "<C --> B>" in outputs
        }

        detachment_var_weak: {
            "
            nse <<$1 --> A> ==> <$1 --> B>>.
            nse <C --> B>.
            cyc 10
            "
            => OUT "<C --> A>" in outputs
        }

        detachment_var_answer_weak: {
            "
            nse <<$1 --> A> ==> <$1 --> B>>.
            nse <C --> B>.
            nse <C --> A>?
            cyc 20
            "
            => ANSWER "<C --> A>" in outputs
        }

        conditional_abduction: {
            "
            nse <(&&, S2, S3) ==> P>.
            nse <(&&, S1, S3) ==> P>.
            cyc 10
            "
            => OUT "<S1 ==> S2>" in outputs
        }

        conditional_abduction_answer: {
            "
            nse <(&&, S2, S3) ==> P>.
            nse <(&&, S1, S3) ==> P>.
            nse <S1 ==> S2>?
            cyc 20
            "
            => ANSWER "<S1 ==> S2>" in outputs
        }

        conditional_abduction_rev: {
            "
            nse <(&&, S2, S3) ==> P>.
            nse <(&&, S1, S3) ==> P>.
            cyc 10
            "
            => OUT "<S2 ==> S1>" in outputs
        }

        conditional_abduction_rev_answer: {
            "
            nse <(&&, S2, S3) ==> P>.
            nse <(&&, S1, S3) ==> P>.
            nse <S2 ==> S1>?
            cyc 20
            "
            => ANSWER "<S2 ==> S1>" in outputs
        }

        conditional_deduction_reduce: {
            "
            nse <(&&, S1, S2, S3) ==> P>.
            nse S1.
            cyc 10
            "
            => OUT "<(&&, S2, S3) ==> P>" in outputs
        }

        conditional_deduction_reduce_answer: {
            "
            nse <(&&, S1, S2, S3) ==> P>.
            nse S1.
            nse <(&&, S2, S3) ==> P>?
            cyc 20
            "
            => ANSWER "<(&&, S2, S3) ==> P>" in outputs
        }

        conditional_deduction_replace: {
            "
            nse <(&&, S2, S3) ==> P>.
            nse <S1 ==> S2>.
            cyc 100
            "
            => OUT "<(&&, S1, S3) ==> P>" in outputs
        }

        conditional_deduction_replace_answer: {
            "
            nse <(&&, S2, S3) ==> P>.
            nse <S1 ==> S2>.
            nse <(&&, S1, S3) ==> P>?
            cyc 200
            "
            => ANSWER "<(&&, S1, S3) ==> P>" in outputs
        }

        conditional_induction: {
            "
            nse <(&&, S1, S3) ==> P>.
            nse <S1 ==> S2>.
            cyc 100
            "
            => OUT "<(&&, S2, S3) ==> P>" in outputs
        }

        conditional_induction_answer: {
            "
            nse <(&&, S1, S3) ==> P>.
            nse <S1 ==> S2>.
            nse <(&&, S2, S3) ==> P>?
            cyc 200
            "
            => ANSWER "<(&&, S2, S3) ==> P>" in outputs
        }

        // ! ❌【2024-08-05 17:33:28】暂不为「条件类比」编写测试：推理规则「条件类比」实际已被禁用
        //   * 📝自有「条件演绎/归纳」为其提供类似实现

        /// 【2024-08-08 15:37:08】测试出现在「条件演绎」时的panic问题
        /// * 📄对应NAL-6.13
        fail_case_image_from_image_from_conditional_ded: {
            "
            nse $0.80;0.80;0.95$ <(&&,<$x --> key>,<$y --> lock>) ==> <$y --> (/,open,$x,_)>>. %1.00;0.90%
            nse $0.80;0.80;0.95$ <{lock1} --> lock>. %1.00;0.90%
            cyc 40
            "
            => OUT "<<$1 --> key> ==> <{lock1} --> (/,open,$1,_)>>" in outputs
        }
    }
}
