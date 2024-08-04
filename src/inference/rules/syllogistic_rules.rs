//! 三段论规则
//! * 🚩【2024-07-11 00:07:34】目前只包含「具体规则处理」两部分
//!   * 📝OpenNARS中「规则表」可能会在某些地方直接分派规则
//!   * 📄条件三段论系列
//!
//! ## Logs
//!
//! * ♻️【2024-07-11 00:07:52】开始根据改版OpenNARS重写

use crate::{
    control::*,
    entity::*,
    inference::{
        rules::{cast_statement, utils::*},
        *,
    },
    io::symbols::CONJUNCTION_OPERATOR,
    language::*,
    util::*,
};
use nar_dev_utils::unwrap_or_return;
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
///
/// ```nal
/// {<(&&, S1, S2, S3) ==> P>, S1} |- <(&&, S2, S3) ==> P>
/// {<(&&, S2, S3) ==> P>, <S1 ==> S2>} |- <(&&, S1, S3) ==> P>
/// {<(&&, S1, S3) ==> P>, <S1 ==> S2>} |- <(&&, S2, S3) ==> P>
/// ```
pub fn conditional_ded_ind(
    conditional: Statement,
    index_in_condition: usize,
    premise2: Term,
    belief: impl Judgement,
    conditional_position: PremiseSource,
    side: SyllogismSide,
    context: &mut ReasonContextConcept,
) {
    // TODO: 🚩待实现
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
    let copula = task_content.identifier().to_string();
    let [sub_t, pre_t] = task_content.unwrap_components();
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
    let content = unwrap_or_return!(?Term::make_statement_relation(&copula, sub, pre));

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
    use super::*;
    use crate::inference::test_inference::{create_vm_from_engine, VmRuntimeBoost};
    use narsese::api::GetTerm;
    use narsese::lexical_nse_term;
    use navm::output::Output;
    use rules::tests::ENGINE_REASON;

    macro_rules! expect_narsese_term {
        // * 🚩模式：【类型】 【内容】 in 【输出】
        ($type:ident $term:literal in outputs) => {
            |o| matches!(
                o,
                Output::$type { narsese,.. }
                // * 🚩【2024-07-15 00:04:43】此处使用了「词法Narsese」的内部分派
                if *narsese.as_ref().unwrap().get_term() == lexical_nse_term!(@PARSE $term)
            )
        };
    }

    fn expectation_test(inputs: impl AsRef<str>, expectation: impl Fn(&Output) -> bool) {
        let mut vm = create_vm_from_engine(ENGINE_REASON);
        // * 🚩OUT
        vm.input_fetch_print_expect(
            inputs.as_ref(),
            // * 🚩检查其中是否有导出
            expectation,
        );
    }

    /// 一个「单输出预期」测试
    macro_rules! expectation_test {
        (
            $(#[$attr:meta])*
            $name:ident :
            $inputs:expr
            => $($expectations:tt)*
        ) => {
            $(#[$attr])*
            #[test]
            fn $name() {
                expectation_test(
                    $inputs,
                    // * 🚩检查其中是否有预期输出
                    expect_narsese_term!($($expectations)*),
                )
            }
        };
    }

    /// 一组「单输出预期」测试
    macro_rules! expectation_tests {
        (
            $(
                $(#[$attr:meta])*
                $name:ident : {
                    $inputs:expr
                    => $($expectations:tt)*
                }
            )*
        ) => {
            $(
                expectation_test! {
                    $(#[$attr])*
                    $name :
                        $inputs
                        => $($expectations)*
                }
            )*
        };
    }

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
    }
}
