//! 🎯复刻OpenNARS `nars.inference.StructuralRules`
//!
//! * ✅【2024-05-11 15:10:00】初步复现方法API
//! * ♻️【2024-08-05 17:32:20】开始根据改版OpenNARS重写

use super::SyllogismPosition;
use crate::{
    control::*,
    entity::*,
    inference::{rules::utils::*, *},
    io::symbols::*,
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

/// 双侧建构
/// * 📝关于「何时传入整个词项，何时传入引用」的处理办法
///   * 📌【2024-08-06 11:32:03】原则：当【整个流程用不到内部所有字段的所有权】时，采用「选择性拷贝」的方法
///     * 🚩传参只传引用，只在需要时拷贝——而非传参前整个拷贝（而后边浪费）
///
/// # 📄OpenNARS
///
/// ```nal
/// {<S --> P>, S@(S&T)} |- <(S&T) --> (P&T)>
/// {<S --> P>, S@(M-S)} |- <(M-P) --> (M-S)>
/// ```
pub fn structural_compose_both(
    compound: CompoundTermRef,
    index: usize,
    statement: StatementRef,
    side: SyllogismPosition,
    context: &mut ReasonContextConcept,
) {
    let direction = context.reason_direction();

    // * 🚩预筛 * //
    let indicated = side.select(statement.sub_pre());
    if *compound == *indicated {
        // * 📄compound="(&,glasses,[black])" @ 0 = "glasses"
        //   * statement="<sunglasses --> (&,glasses,[black])>" @ 1 = compound
        //   * ⇒不处理（❓为何如此）
        return;
    }

    // * 🚩词项 * //
    let [statement_sub, statement_pre] = statement.sub_pre();
    let sub_pre = [&statement_sub, &statement_pre];
    let mut components = compound.clone_components();
    let [term_self_side, other_statement_component] = side.select_and_other(sub_pre); // 同侧词项 & 异侧词项
    if components.contains(other_statement_component) {
        // * 📝复合词项包含陈述的另一侧词项 ⇒ 中止
        // * 📄compound = "(*,{tom},(&,glasses,[black]))" @ 1 => "(&,glasses,[black])"
        //   * statement = "<(&,glasses,sunglasses) --> (&,glasses,[black])>" @ 0
        //   * components = ["{tom}", "(&,glasses,[black])"]
        //   * ⇒不处理（❓为何如此）
        return;
    }
    // 先决条件：是否包含同侧词项
    let cloned_statement_sub_pre = || [statement_sub.clone(), statement_pre.clone()];
    let [sub, pre] = match components.contains(term_self_side) {
        true => side.select_and_other([
            // * 🚩主项/谓项：原来的复合词项
            compound.inner.clone(),
            // * 🚩谓项/主项：替换后的复合词项
            {
                let term_opposite = side.opposite().select([statement_sub, statement_pre]); // 提取出异侧词项
                components[index] = term_opposite.clone(); // 将对应位置换成异侧词项
                unwrap_or_return!(?Term::make_compound_term(compound, components))
            },
        ]),
        false => cloned_statement_sub_pre(),
    };
    // * 📄compound = "(&,[yellow],{Birdie})" @ 0 => "[yellow]"
    //   * statement = "<{Tweety} --> [yellow]>" @ 1
    //   * components = ["{Tweety}", "{Birdie}"]
    //   * subj = "(&,{Tweety},{Birdie})" = null | 空集
    //   * pred = "(&,[yellow],{Birdie})"
    //   * ⇒制作失败
    // * 🚩根据「复合词项&索引」决定是否要「调换关系」
    let [sub, pre] = switch_by_order(compound, index, [sub, pre]);
    let content = unwrap_or_return!(?Term::make_statement(&statement, sub, pre));
    let task_truth = context
        .current_task()
        .get_()
        .as_judgement()
        .map(TruthValue::from);

    // * 🚩真值 * //
    let truth = match direction {
        // * 🚩前向推理
        Forward => match compound.size() {
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

/// 双侧解构
/// * 📝关于「何时传入整个词项，何时传入引用」的处理办法
///   * 📌【2024-08-06 11:32:03】原则：当【整个流程用不到内部所有字段的所有权】时，采用「选择性拷贝」的方法
///     * 🚩传参只传引用，只在需要时拷贝——而非传参前整个拷贝（而后边浪费）
///
/// ```nal
/// {<(S&T) --> (P&T)>, S@(S&T)} |- <S --> P>
/// ```
pub fn structural_decompose_both(
    statement: StatementRef,
    index: usize,
    context: &mut ReasonContextConcept,
) {
    // * 🚩词项 * //
    let [sub, pre] = statement.sub_pre();
    // * 📌必须是「同类复合词项」才有可能解构
    if !sub.is_same_type(pre) {
        return;
    }
    let [sub, pre] = match [sub.as_compound(), pre.as_compound()] {
        [Some(sub), Some(pre)] => [sub, pre],
        _ => return,
    };
    // * 📌必须是「同尺寸复合词项」且「索引在界内」
    let [sub_size, pre_size] = [sub.size(), pre.size()];
    if !(sub_size == pre_size && index < sub_size) {
        return;
    }
    // * 🚩取其中索引所在的词项，按顺序制作相同系词的陈述
    let at_index = |compound: CompoundTermRef| compound.component_at(index).unwrap().clone(); // ! 上边已判断在界内
    let sub_inner = at_index(sub);
    let pre_inner = at_index(pre);

    // * 🚩尝试调换顺序
    let [content_sub, content_pre] = switch_by_order(sub, index, [sub_inner, pre_inner]);
    let content = unwrap_or_return!(?Term::make_statement(&statement, content_sub, content_pre));

    // * 🚩预筛
    let direction = context.reason_direction();
    let task_is_judgement = context.current_task().get_().is_judgement();
    let task_truth = context
        .current_task()
        .get_()
        .as_judgement()
        .map(TruthValue::from);
    if !(direction == Forward) // ? 💭【2024-08-05 23:37:40】这个「前向推理又是判断」似乎不可能发生
        && !sub.instanceof_product()
        && sub.size() > 1
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

/// * 📝单侧建构
///
/// # 📄OpenNARS
///
/// ```nal
/// {<S --> P>, P@(P&Q)} |- <S --> (P&Q)>
/// ```
pub fn structural_compose_one(
    compound: CompoundTermRef,
    index: usize,
    statement: StatementRef,
    context: &mut ReasonContextConcept,
) {
    // ! 📝此推理只适用于前向推理（目标推理亦不行，refer@304）
    if context.reason_direction() == Backward {
        return;
    }

    // * 🚩预先计算真值
    let task_truth = TruthValue::from(context.current_task().get_().unwrap_judgement());
    let truth_deduction = task_truth.analytic_deduction(context.reasoning_reliance());

    // * 🚩部分计算词项，并向下分派
    //   * 📄"P@(P&Q)" => "P"
    //   * 📄"<S --> P>" => subj="S", pred="P"
    let component = unwrap_or_return!(?compound.component_at(index));
    let compound = compound.inner.clone();
    let [sub, pre] = statement.sub_pre();
    let (sub_pre, to_not_ded) = match [*component == *sub, *component == *pre] {
        // * 🚩复合词项是主项
        // * 📄"S"@"(S&T)" × "<S --> P>"
        [true, _] => match (compound.identifier(), index) {
            // * 🚩外延交
            // * 📄"S"@"(S&T)" × "<S --> P>"
            //   * component=subj="S"
            //   * compound="(S&T)"
            //   * pred="P"
            //   * => "<(S&T) --> P>"
            (INTERSECTION_EXT_OPERATOR, _)
            // * 🚩外延差@第一项 ⇒ "<(S-T) --> P>"
            // * 📄"S"@"(S-T)" × "<S --> P>"
            //   * component=subj="S"
            //   * compound="(S-T)"
            //   * pred="P"
            //   * => "<(S-T) --> P>"
            | (DIFFERENCE_EXT_OPERATOR, 0) => ([compound, pre.clone()], false),
            // * 🚩内涵差@第二项 ⇒ "<(T~S) --> P>"
            // * 📄"S"@"(T~S)" × "<S --> P>"
            //   * component=subj="S"
            //   * compound="(T~S)"
            //   * pred="P"
            //   * => "<(T~S) --> P>"
            // * 📝真值取【否定】
            (DIFFERENCE_INT_OPERATOR, 1) => ([compound, pre.clone()], true),
            // 其它
            _ => return,
        },
        // * 🚩复合词项是谓项
        // * 📄"P"@"(P&Q)" × "<S --> P>"
        [_, true] => match (compound.identifier(), index) {
            // * 🚩内涵交
            // * 📄"P"@"(P|Q)" × "<S --> P>"
            //   * component=pred="P"
            //   * compound="(P|Q)"
            //   * subj="S"
            //   * => "<S --> (P|Q)>"
            (INTERSECTION_INT_OPERATOR, _)
            // * 🚩内涵差@第一项
            // * 📄"P"@"(P~Q)" × "<S --> P>"
            //   * component=pred="P"
            //   * compound="(P~Q)"
            //   * subj="S"
            //   * => "<S --> (P~Q)>"
            | (DIFFERENCE_INT_OPERATOR, 0) => ([sub.clone(), compound], false),
            // * 🚩外延差@第二项
            // * 📄"P"@"(Q-P)" × "<S --> P>"
            //   * component=pred="P"
            //   * compound="(Q-P)"
            //   * subj="S"
            //   * => "<S --> (Q-P)>"
            // * 📝真值取【否定】
            (DIFFERENCE_EXT_OPERATOR, 1) => ([sub.clone(), compound], true),
            // 其它
            _ => return,
        },
        _ => return,
    };
    // * 🚩统一构造陈述
    let truth = match to_not_ded {
        true => truth_deduction.negation(), // 要取否定取否定
        false => truth_deduction,           // 否则就是原样
    };
    structural_statement(sub_pre, truth, context);
}

/// * 📝单侧解构
///
/// # 📄OpenNARS
///
/// ```nal
/// {<(S&T) --> P>, S@(S&T)} |- <S --> P>
/// ```
pub fn structural_decompose_one(
    compound: CompoundTermRef,
    index: usize,
    statement: StatementRef,
    context: &mut ReasonContextConcept,
) {
    // ! 📝此推理只适用于前向推理（目标推理亦不行，refer@304）
    if context.reason_direction() == Backward {
        return;
    }

    // * 🚩预先计算真值
    let task_truth = TruthValue::from(context.current_task().get_().unwrap_judgement());
    let truth_deduction = task_truth.analytic_deduction(context.reasoning_reliance());

    // * 🚩部分计算词项，并向下分派
    //   * 📄"S@(S&T)" => "S"
    //   * 📄"<(S&T) --> P>" => subj="(S&T)", pred="P"
    let [sub, pre] = statement.sub_pre();
    let component = unwrap_or_return!(?compound.component_at(index)).clone(); // 只拷贝指定位置的元素
    let (sub_pre, to_not_ded) = match [*compound == *sub, *compound == *pre] {
        // * 🚩复合词项是主项
        // * 📄"P"@"(P&Q)" × "<(P&Q) --> S>"
        [true, _] => match compound.identifier() {
            // * 🚩内涵交
            // * 📄"S"@"(S|T)" × "<(S|T) --> P>"
            // * * compound=subj="(S|T)"
            // * * component="S"
            // * * pred="P"
            // * * => "<S --> P>"
            INTERSECTION_INT_OPERATOR => ([component, pre.clone()], false),
            // * 🚩多元外延集
            // * 📄"S"@"{S,T}" × "<{S,T} --> P>"
            // * * compound=subj="{S,T}"
            // * * component="S"
            // * * pred="P"
            // * * => "<{S} --> P>"
            // * 📌【2024-07-22 16:01:42】此处`makeSet`不会失败（结果非空）
            SET_EXT_OPERATOR if compound.size() > 1 => (
                [
                    unwrap_or_return!(?Term::make_set_ext(component)),
                    pre.clone(),
                ],
                false,
            ),
            // * 🚩内涵差
            // * 📄"S"@"(S~T)" × "<(S~T) --> P>"
            // * * compound=subj="(S~T)"/"(T~S)"
            // * * component="S"
            // * * pred="P"
            // * * => "<S --> P>"
            // * 📝真值函数方面：若为「减掉的项」则【取否定】处理
            DIFFERENCE_INT_OPERATOR => ([component, pre.clone()], index == 1),
            // 其它
            _ => return,
        },
        // * 🚩复合词项是谓项
        // * 📄"P"@"(P&Q)" × "<S --> (P&Q)>"
        [_, true] => match compound.identifier() {
            // * 🚩外延交
            // * 📄"S"@"(S&T)" × "<(S&T) --> P>"
            // * * compound=subj="(S&T)"
            // * * component="S"
            // * * pred="P"
            // * * => "<S --> P>"
            INTERSECTION_EXT_OPERATOR => ([sub.clone(), component], false),
            // * 🚩多元内涵集
            // * 📄"P"@"[P,Q]" × "<S --> [P,Q]>"
            // * * compound=subj="[S,T]"
            // * * component="S"
            // * * pred="P"
            // * * => "<S --> [P]>"
            // * 📌【2024-07-22 16:01:42】此处`makeSet`不会失败（结果非空）
            SET_INT_OPERATOR if compound.size() > 1 => (
                [
                    sub.clone(),
                    unwrap_or_return!(?Term::make_set_int(component)),
                ],
                false,
            ),
            // * 🚩外延差
            // * 📄"P"@"(P-Q)" × "<S --> (P-Q)>"
            // * * compound=pred="(P-Q)"/"(Q-P)"
            // * * component="P"
            // * * subj="S"
            // * * => "<S --> P>"
            // * 📝真值函数方面：若为「减掉的项」则【取否定】处理
            DIFFERENCE_EXT_OPERATOR => ([sub.clone(), component], index == 1),
            // 其它
            _ => return,
        },
        _ => return,
    };
    // * 🚩统一构造陈述
    let truth = match to_not_ded {
        true => truth_deduction.negation(), // 要取否定取否定
        false => truth_deduction,           // 否则就是原样
    };
    structural_statement(sub_pre, truth, context);
}

/// * 📝共用函数：根据给定的主项、谓项、任务内容（as模板） 构造新任务
///
/// # 📄OpenNARS
///
/// Common final operations of the above two methods
fn structural_statement(
    [sub, pre]: [Term; 2],
    truth: TruthValue,
    context: &mut ReasonContextConcept,
) {
    // * 🚩构造新陈述
    let content = {
        // * 🚩获取旧任务的陈述内容
        let task_ref = context.current_task();
        let task_rc = task_ref.get_();
        let task_statement = unwrap_or_return!(?task_rc.content().as_statement());

        // * 🚩构造新陈述，同时回收「任务陈述内容」的引用
        unwrap_or_return!(?Term::make_statement(&task_statement, sub, pre))
    };

    // * 🚩预算 * //
    let budget = context.budget_compound_forward(&truth, &content);

    // * 🚩结论 * //
    context.single_premise_task_structural(content, Some(truth), budget);
}

/* -------------------- set transform -------------------- */

/// 外延集、内涵集的「定理」
/// * 📝NAL中「外延集」「内涵集」可以理解为「概念继承关系的上界/下界」
///   * 💡整个继承关系可类比性地构造出Hasse图
/// * 💭实际上只需要「关系反转」即可——如此便可脱离「相似系词」而获得等价效果
///
/// # 📄OpenNARS
///
/// ```nal
/// {<S --> {P}>} |- <S <-> {P}>
/// {<[S] --> P>} |- <[S] <-> P>
/// ```
pub fn transform_set_relation(
    compound: CompoundTermRef,
    statement: StatementRef,
    side: SyllogismPosition,
    context: &mut ReasonContextConcept,
) {
    // * 🚩预筛 * //
    // * 🚩仅一元集
    if compound.size() > 1 {
        return;
    }
    // * 🚩不处理其它「继承」的情况
    if statement.instanceof_inheritance() {
        match (compound.identifier(), side) {
            // * 📄"<{S} --> X>"
            // * 📄"<X --> [P]>"
            (SET_EXT_OPERATOR, Subject) | (SET_INT_OPERATOR, Predicate) => return,
            _ => {}
        }
    }

    // * 🚩词项 * //
    let [sub, pre] = statement.sub_pre();
    let [sub, pre] = [sub.clone(), pre.clone()];
    let content = match statement.identifier() {
        // * 📄"<S --> {P}>" => "<S <-> {P}>"
        // * 📄"<[S] --> P>" => "<[S] <-> P>"
        INHERITANCE_RELATION => Term::make_similarity(sub, pre),
        _ => match (compound.identifier(), side) {
            // * 📄"<{S} <-> P>" => "<P --> {S}>"
            // * 📄"<S <-> [P]>" => "<[P] --> S>"
            (SET_EXT_OPERATOR, Subject) | (SET_INT_OPERATOR, Predicate) => {
                Term::make_inheritance(pre, sub)
            }
            // * 📄"<S <-> {P}>" => "<S --> {P}>"
            // * 📄"<[S] <-> P>" => "<[S] --> P>"
            _ => Term::make_inheritance(sub, pre),
        },
    };
    let content = unwrap_or_return!(?content);

    let task_truth = context
        .current_task()
        .get_()
        .as_judgement()
        .map(TruthValue::from);
    let direction = context.reason_direction();

    // * 🚩真值 * //
    let truth = match direction {
        // * 🚩前向 ⇒ 恒等
        Forward => task_truth.map(|truth| truth.identity()),
        // * 🚩反向 ⇒ 空
        Backward => None,
    };

    // * 🚩预算 * //
    let budget = match direction {
        // * 🚩前向⇒复合前向
        Forward => context.budget_compound_forward(truth.as_ref(), &content),
        // * 🚩反向⇒复合反向
        Backward => context.budget_compound_backward(&content),
    };

    // * 🚩结论 * //
    context.single_premise_task_structural(content, truth, budget);
}

/* --------------- Disjunction and Conjunction transform --------------- */

/// 有关「合取」与「析取」的转换
///
/// # 📄OpenNARS
///
/// ```nal
/// {(&&, A, B), A@(&&, A, B)} |- A, or answer (&&, A, B)? using A
/// {(||, A, B), A@(||, A, B)} |- A, or answer (||, A, B)? using A
/// ```
#[doc(alias = "structural_compound")]
pub fn structural_junction(
    compound: CompoundTermRef,
    component: &Term,
    compound_from: PremiseSource,
    context: &mut ReasonContextConcept,
) {
    // * 🚩仅「常量词项」
    if !component.is_constant() {
        return;
    }

    let task_truth = context
        .current_task()
        .get_()
        .as_judgement()
        .map(TruthValue::from);
    let direction = context.reason_direction();

    // * 🚩词项 * //
    let content = match compound_from {
        // * 🚩复合词项从任务中来 ⇒ 元素
        PremiseSource::Task => component.clone(),
        // * 🚩信念 ⇒ 整体
        PremiseSource::Belief => compound.inner.clone(),
    };

    // * 🚩真值 * //
    let truth = match direction {
        // * 🚩前向推理⇒根据「复合词项从任务中来 == 复合词项是合取」决策
        // * 📝from OpenNARS 3.0.4：前向推理同时对「判断」「目标」成立，因此「任务是判断」的条件可省去
        Forward => task_truth.map(|truth| {
            match (compound_from == PremiseSource::Task) == compound.instanceof_conjunction() {
                // * 🚩满足⇒分析性演绎
                true => truth.analytic_deduction(context.reasoning_reliance()),
                // * 🚩满足⇒分析性反演（非⇒演绎⇒非）
                false => {
                    dbg!(dbg!(truth.negation()).analytic_deduction(context.reasoning_reliance()))
                        .negation()
                }
            }
        }),
        Backward => None,
    };

    // * 🚩预算 * //
    let budget = match direction {
        // * 🚩前向⇒前向
        Forward => context.budget_forward(truth.as_ref()),
        // * 🚩反向⇒复合反向
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
            cyc 30
            " // ! ♻️【2024-08-06 12:48:56】此处因为「集合转换」规则，又变慢了
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

        // ! 🚩【2024-08-06 10:53:41】后续对「单侧建构/单侧解构」不再附加「ANSWER测试」
        //   * 📝先前的测试已保证「生成结论后必定有ANSWER」与「能根据疑问句内容（词项链）反推答案」
        compose_one_int_ext: {
            "
            nse <A --> B>.
            nse (&,A,C)?
            cyc 10
            "
            => OUT "<(&,A,C) --> B>" in outputs
        }

        compose_one_int_int: {
            "
            nse <A --> B>.
            nse (|,B,C)?
            cyc 10
            "
            => OUT "<A --> (|,B,C)>" in outputs
        }

        compose_one_diff_ext: {
            "
            nse <A --> B>.
            nse (-,A,C)?
            cyc 10
            "
            => OUT "<(-,A,C) --> B>" in outputs
        }

        compose_one_diff_int: {
            "
            nse <A --> B>.
            nse (~,B,C)?
            cyc 10
            "
            => OUT "<A --> (~,B,C)>" in outputs
        }

        compose_one_diff_ext_neg: {
            "
            nse <A --> B>. %0%
            nse (-,C,B)?
            cyc 10
            "
            => OUT "<A --> (-,C,B)>" in outputs
        }

        compose_one_diff_int_neg: {
            "
            nse <A --> B>. %0%
            nse (~,C,A)?
            cyc 10
            "
            => OUT "<(~,C,A) --> B>" in outputs
        }

        decompose_one_int_ext: { // * 📝没有「主项外延交」只有「主项内涵交」
            "
            nse <A --> (&,B,C)>.
            cyc 10
            "
            => OUT "<A --> B>" in outputs
        }

        decompose_one_int_int: { // * 📝没有「谓项内涵交」只有「谓项外延交」
            "
            nse <(|,A,C) --> B>.
            cyc 10
            "
            => OUT "<A --> B>" in outputs
        }

        decompose_one_set_ext_1: {
            "
            nse <{A,C} --> B>.
            cyc 10
            "
            => OUT "<{A} --> B>" in outputs
        }

        decompose_one_set_ext_2: {
            "
            nse <{A,C} --> B>.
            cyc 10
            "
            => OUT "<{C} --> B>" in outputs
        }

        decompose_one_set_int_1: {
            "
            nse <A --> [B,C]>.
            cyc 10
            "
            => OUT "<A --> [B]>" in outputs
        }

        decompose_one_set_int_2: {
            "
            nse <A --> [B,C]>.
            cyc 10
            "
            => OUT "<A --> [C]>" in outputs
        }

        decompose_one_diff_ext: {
            "
            nse <A --> (-,B,C)>.
            cyc 10
            "
            => OUT "<A --> B>" in outputs
        }

        decompose_one_diff_ext_neg: {
            "
            nse <A --> (-,B,C)>. %0%
            cyc 10
            "
            => OUT "<A --> C>" in outputs
        }

        decompose_one_diff_int: {
            "
            nse <(~,A,C) --> B>.
            cyc 10
            "
            => OUT "<A --> B>" in outputs
        }

        decompose_one_diff_int_neg: {
            "
            nse <(~,A,C) --> B>. %0%
            cyc 10
            "
            => OUT "<C --> B>" in outputs
        }

        transform_set_ext: {
            "
            nse <A --> {B}>.
            cyc 10
            "
            => OUT "<A <-> {B}>" in outputs
        }

        transform_set_int: {
            "
            nse <[A] --> B>.
            cyc 10
            "
            => OUT "<B <-> [A]>" in outputs // ! 🚩【2024-08-06 12:44:16】要调换顺序：比对时仅比对「词法Narsese」
        }

        transform_set_ext_sub: {
            "
            nse <A <-> {B}>.
            cyc 10
            "
            => OUT "<A --> {B}>" in outputs
        }

        transform_set_ext_pre: { // ! ❌【2024-08-06 12:47:04】此情形不会被直接触发：相似系词只会将复合词项单排列至一侧
            "
            nse <A <-> {B}>.
            nse <{B} --> A>?
            cyc 10
            "
            => OUT "<{B} --> A>" in outputs
        }

        transform_set_int_sub: {
            "
            nse <[A] <-> B>.
            cyc 10
            "
            => OUT "<[A] --> B>" in outputs
        }

        transform_set_int_pre: { // ! ❌【2024-08-06 12:47:04】此情形不会被直接触发：相似系词只会将复合词项单排列至一侧
            "
            nse <[A] <-> B>.
            nse <B --> [A]>?
            cyc 10
            "
            => OUT "<B --> [A]>" in outputs
        }

        structural_conjunction: {
            "
            nse (&&, A, B).
            cyc 10
            "
            => OUT "A" in outputs
        }

        structural_disjunction: { // * ℹ️OpenNARS中`ANSWER: A. %1.00;0.00% {9 : 2}`，信度为0是正常的
            "
            nse (||, A, B).
            cyc 10
            "
            => OUT "A" in outputs
        }

        structural_conjunction_backward: { // * ℹ️OpenNARS中`ANSWER: (&&, A, B). %1.00;0.00% {9 : 2}`，信度为0是正常的
            "
            nse A.
            nse (&&, A, B)?
            cyc 10
            "
            => ANSWER "(&&, A, B)" in outputs
        }

        structural_disjunction_backward: {
            "
            nse A.
            nse (||, A, B)?
            cyc 10
            "
            => ANSWER "(||, A, B)" in outputs
        }
    }
}
