//! 三段论推理中的「子分派」
//! * 🎯包括「不直接涉及推理结论」的诸多方法

use crate::{
    control::*,
    entity::*,
    inference::rules::{
        intro_var_inner, intro_var_same_subject_or_predicate, syllogistic_rules, utils::*,
    },
    io::symbols::*,
    language::{
        variable_process::{has_unification_q, unify_find_i, unify_find_q},
        *,
    },
    util::*,
};
use syllogistic_figures::*;
use syllogistic_rules::*;
use ReasonDirection::*;

use super::compositional::compose_compound;

/// 索引⇒图式
fn index_to_figure<T, U>(link1: &impl TLink<T>, link2: &impl TLink<U>) -> SyllogismFigure {
    let side1 = SyllogismPosition::from_index(*link1.get_index(0).unwrap());
    let side2 = SyllogismPosition::from_index(*link2.get_index(0).unwrap());
    side1.build_figure(side2)
}

pub fn syllogisms(
    task_term: Statement,
    belief_term: Statement,
    t_index: usize,
    b_index: usize,
    belief: impl Judgement,
    context: &mut ReasonContextConcept,
) {
    // * 🚩提取参数
    let t_link = context.current_task_link();
    let b_link = context.current_belief_link();
    let task_sentence = context.current_task().get_().sentence_clone();
    match [task_term.identifier(), belief_term.identifier()] {
        // * 🚩非对称×非对称：继承×继承 | 蕴含×蕴含
        [INHERITANCE_RELATION, INHERITANCE_RELATION]
        | [IMPLICATION_RELATION, IMPLICATION_RELATION] => asymmetric_asymmetric(
            task_sentence,
            belief,
            index_to_figure(t_link, b_link),
            context,
        ),
        // * 🚩非对称×对称：继承×相似 | 蕴含×等价
        [INHERITANCE_RELATION, SIMILARITY_RELATION]
        | [IMPLICATION_RELATION, EQUIVALENCE_RELATION] => asymmetric_symmetric(
            task_sentence,
            belief,
            index_to_figure(t_link, b_link),
            context,
        ),
        // * 🚩非对称×对称：继承×相似 | 蕴含×等价
        [SIMILARITY_RELATION, INHERITANCE_RELATION]
        | [EQUIVALENCE_RELATION, IMPLICATION_RELATION] => asymmetric_symmetric(
            belief,
            task_sentence,
            index_to_figure(b_link, t_link),
            context,
        ),
        // * 🚩对称×对称：相似×相似 | 等价×等价
        [SIMILARITY_RELATION, SIMILARITY_RELATION]
        | [EQUIVALENCE_RELATION, EQUIVALENCE_RELATION] => symmetric_symmetric(
            task_sentence,
            belief,
            index_to_figure(t_link, b_link),
            context,
        ),
        // * 🚩分离：继承 + | 继承 × 蕴含/等价
        [INHERITANCE_RELATION, IMPLICATION_RELATION | EQUIVALENCE_RELATION] => {
            detachment_with_var(
                task_sentence, // ! 📌【2024-08-01 18:26:04】需要传递所有权：直接统一语句中的变量
                belief,        // ! 📌【2024-08-01 18:26:04】需要传递所有权：直接统一语句中的变量
                PremiseSource::Belief,
                SyllogismPosition::from_index(b_index),
                context,
            )
        }
        // * 🚩分离：蕴含 + | 蕴含/等价 × 继承
        [IMPLICATION_RELATION | EQUIVALENCE_RELATION, INHERITANCE_RELATION] => {
            detachment_with_var(
                task_sentence, // ! 📌【2024-08-01 18:26:04】需要传递所有权：直接统一语句中的变量
                belief,        // ! 📌【2024-08-01 18:26:04】需要传递所有权：直接统一语句中的变量
                PremiseSource::Task,
                SyllogismPosition::from_index(t_index),
                context,
            )
        }
        // * 🚩无果匹配：相似×高阶 | 高阶×相似
        [SIMILARITY_RELATION, IMPLICATION_RELATION | EQUIVALENCE_RELATION]
        | [IMPLICATION_RELATION | EQUIVALENCE_RELATION, SIMILARITY_RELATION] => {}
        // * ❌域外情况
        [t_id, b_id] => unimplemented!("未知的陈述类型：{t_id:?}, {b_id:?}"),
    }
}

/// 非对称×非对称
fn asymmetric_asymmetric(
    task_sentence: impl Sentence,
    belief_sentence: impl Judgement,
    figure: SyllogismFigure,
    context: &mut ReasonContextConcept,
) {
    // * 🚩非对称🆚非对称
    let mut t_term = cast_statement(task_sentence.clone_content());
    let mut b_term = cast_statement(belief_sentence.clone_content());
    let [rng_seed, rng_seed2] = context.shuffle_rng_seeds();

    // * 🚩尝试获取各大「共同项」与「其它项」的位置
    // * 📝外部传入的「三段论图式」即「共同项的位置」，「其它项」即各处「共同项」的反向
    let [[common_pos_t, common_pos_b], [other_pos_t, other_pos_b]] = figure.and_opposite();
    // * 🚩先尝试统一独立变量
    let unified_i = unify_find_i(
        common_pos_t.select_one(t_term.sub_pre()),
        common_pos_b.select_one(b_term.sub_pre()),
        rng_seed,
    )
    .apply_to(
        t_term.mut_ref().into_compound_ref(),
        b_term.mut_ref().into_compound_ref(),
    );
    // * 🚩不能统一变量⇒终止
    if !unified_i {
        return;
    }
    // * 🚩统一后内容相等⇒终止
    if t_term == b_term {
        return;
    }
    // * 🚩取其中两个不同的项 | 需要在后续「条件类比」中重复使用
    let term_t = other_pos_t.select_one(t_term.sub_pre());
    let term_b = other_pos_b.select_one(b_term.sub_pre());
    // * 📝构造一个闭包，随时根据图式生成（用于NAL-1推理的）主项、谓项
    //   * 📌原因：先执行的「构造复合词项」「条件归纳」可能要使用term_t、term_b
    let lower_level_composition = |term_t, term_b| match figure {
        // * 📌主项 ⇒ sub来自信念，pre来自任务
        SS | SP => [term_b, term_t],
        // * 📌谓项 ⇒ sub来自任务，pre来自信念
        PS | PP => [term_t, term_b],
    };

    // 再分派特有逻辑
    match figure {
        // * 🚩主项×主项 <A --> B> × <A --> C>
        // induction
        SS => {
            // * 🚩构造复合词项
            compose_compound(
                t_term.get_ref(),
                b_term.get_ref(),
                SyllogismPosition::Subject,
                context,
            );
            // * 🚩归因+归纳+比较
            let [sub, pre] = lower_level_composition(term_t, term_b);
            abd_ind_com(
                sub.clone(),
                pre.clone(),
                task_sentence,
                belief_sentence,
                context,
            );
        }
        // * 🚩谓项×谓项 <A --> B> × <C --> B>
        // abduction
        PP => {
            // * 🚩先尝试进行「条件归纳」，有结果⇒返回
            let [[condition_t, _], [condition_b, _]] = [t_term.sub_pre(), b_term.sub_pre()];
            let applied =
                conditional_abduction(condition_t, condition_b, &t_term, &b_term, context);
            if applied {
                // if conditional abduction, skip the following
                return;
            }
            // * 🚩尝试构建复合词项
            compose_compound(
                t_term.get_ref(),
                b_term.get_ref(),
                SyllogismPosition::Predicate,
                context,
            );
            // * 🚩归因+归纳+比较
            let [sub, pre] = lower_level_composition(term_t, term_b);
            abd_ind_com(
                sub.clone(),
                pre.clone(),
                task_sentence,
                belief_sentence,
                context,
            );
        }
        // * 🚩主项×谓项 <A --> B> × <C --> A>
        // * 🚩谓项×主项 <A --> B> × <B --> C>
        // * 📝【2024-07-31 19:52:56】sub、pre已经在先前「三段论图式选取」过程中确定，此两种形式均一致
        // deduction | exemplification
        SP | PS => {
            // * 🚩尝试统一查询变量
            // * ⚠️【2024-07-14 03:13:32】不同@OpenNARS：无需再应用到整个词项——后续已经不再需要t_term与b_term
            // * ⚠️【2024-07-31 21:37:10】激进改良：无需应用变量替换，只需考虑「是否可替换」
            let [sub, pre] = lower_level_composition(term_t, term_b);
            let unified_q = has_unification_q(sub, pre, rng_seed2);
            match unified_q {
                // * 🚩成功统一 ⇒ 匹配反向
                true => match_reverse(task_sentence, belief_sentence, context),
                // * 🚩未有统一 ⇒ 演绎+举例 | 顺序已在先前决定（要换早换了）
                false => ded_exe(sub, pre, task_sentence, belief_sentence, context),
            }
        }
    }
}

/// The task and belief match reversely
/// * 📄<A --> B> + <B --> A>
///   * inferToSym: <A --> B>. => <A <-> B>.
///   * conversion: <A --> B>? => <A --> B>.
fn match_reverse(
    task_sentence: impl Sentence,
    belief_sentence: impl Judgement,
    context: &mut ReasonContextConcept,
) {
    match context.reason_direction() {
        // * 🚩前向推理⇒判断句⇒尝试合并成对称形式（继承⇒相似，蕴含⇒等价）
        Forward => infer_to_sym(task_sentence.unwrap_judgement(), &belief_sentence, context),
        // * 🚩反向推理⇒疑问句⇒尝试执行转换规则
        Backward => conversion(&belief_sentence, context),
    }
}

/// 非对称×对称
fn asymmetric_symmetric(
    asymmetric: impl Sentence,
    symmetric: impl Sentence,
    figure: SyllogismFigure,
    context: &mut ReasonContextConcept,
) {
    // * 🚩非对称🆚对称
    let mut asy_s = cast_statement(asymmetric.clone_content());
    let mut sym_s = cast_statement(symmetric.clone_content());
    let [rng_seed, rng_seed2] = context.shuffle_rng_seeds();

    // * 🚩尝试获取各大「共同项」与「其它项」的位置
    // * 📝外部传入的「三段论图式」即「共同项的位置」，「其它项」即各处「共同项」的反向
    let [[common_pos_asy, common_pos_sym], [other_pos_asy, other_pos_sym]] = figure.and_opposite();
    let switch_order = match figure {
        // * 🚩主项×主项 <A --> B> × <A <-> C>
        // * 🚩取其中两个不同的谓项 B + C
        // * 🚩最后类比传参：`analogy(term2, term1, ...)`
        SS => true,
        // * 🚩主项×谓项 <A --> B> × <C <-> A>
        // * 🚩取其中两个不同的主项 B + C
        // * 🚩最后类比传参：`analogy(term2, term1, ...)`
        SP => true,
        // * 🚩谓项×主项 <A --> B> × <B <-> C>
        // * 🚩取其中两个不同的主项 A + C
        // * 🚩最后类比传参：`analogy(term1, term2, ...)`
        PS => false,
        // * 🚩谓项×谓项 <A --> B> × <C <-> B>
        // * 🚩取其中两个不同的主项 A + C
        // * 🚩最后类比传参：`analogy(term1, term2, ...)`
        PP => false,
    };

    // * 🚩先尝试统一独立变量
    let unified_i = unify_find_i(
        common_pos_asy.select_one(asy_s.sub_pre()),
        common_pos_sym.select_one(sym_s.sub_pre()),
        rng_seed,
    )
    .apply_to(
        asy_s.mut_ref().into_compound_ref(),
        sym_s.mut_ref().into_compound_ref(),
    );
    // * 🚩不能统一变量⇒终止
    if !unified_i {
        return;
    }
    // * 🚩再根据「是否可统一查询变量」做分派（可统一⇒已经统一了
    let unified_q = unify_find_q(
        other_pos_asy.select_one(asy_s.sub_pre()),
        other_pos_sym.select_one(sym_s.sub_pre()),
        rng_seed2,
    )
    .apply_to(
        asy_s.mut_ref().into_compound_ref(),
        sym_s.mut_ref().into_compound_ref(),
    );
    // * 🚩能统一 ⇒ 继续分派
    if unified_q {
        match_asy_sym(asymmetric, symmetric, context);
    }
    // * 🚩未有统一 ⇒ 类比
    else {
        // 获取并拷贝相应位置的词项
        let [term_asy, term_sym] = [
            other_pos_asy.select_one(asy_s.sub_pre()).clone(),
            other_pos_sym.select_one(sym_s.sub_pre()).clone(),
        ];
        // 转换顺序：true => [C, B], false => [B, C]
        let [term1, term2] = match switch_order {
            true => [term_sym, term_asy],
            false => [term_asy, term_sym],
        };
        analogy(term1, term2, asymmetric, symmetric, context);
    }
}

/// 非对称×对称
///
/// # 📄OpenNARS
///
/// Inheritance/Implication matches Similarity/Equivalence
fn match_asy_sym(
    asymmetric: impl Sentence,
    symmetric: impl Sentence,
    context: &mut ReasonContextConcept,
) {
    match context.reason_direction() {
        // * 🚩前向推理⇒尝试合并到非对称形式（相似⇒继承，等价⇒蕴含）
        // * 🚩若「当前任务」是「判断」，则两个都会是「判断」
        Forward => infer_to_asy(
            asymmetric.unwrap_judgement(),
            symmetric.unwrap_judgement(),
            context,
        ),
        // * 🚩反向推理：尝试「继承⇄相似」「蕴含⇄等价」
        Backward => {
            let task_sentence = &context.current_task().get_().sentence_clone(); // ! 复制以避免借用问题
            convert_relation(task_sentence.unwrap_question(), context)
        }
    }
}

/// 对称×对称
fn symmetric_symmetric(
    task_sentence: impl Sentence,
    belief_sentence: impl Judgement,
    figure: SyllogismFigure,
    context: &mut ReasonContextConcept,
) {
    // * 🚩对称🆚对称
    let mut t_term = cast_statement(task_sentence.clone_content());
    let mut b_term = cast_statement(belief_sentence.clone_content());
    let [pos_t, pos_b] = figure;
    let [common_t, common_b] = [
        pos_t.select_one(t_term.sub_pre()),
        pos_b.select_one(b_term.sub_pre()),
    ];
    let rng_seed = context.shuffle_rng_seeds();
    // * 🚩尝试以不同方式统一独立变量 @ 公共词项
    let unified = unify_find_i(common_b, common_t, rng_seed).apply_to(
        t_term.mut_ref().into_compound_ref(),
        b_term.mut_ref().into_compound_ref(),
    );
    // * 🚩成功统一 ⇒ 相似传递
    if unified {
        let [other_t, other_b] = [
            pos_t.opposite().select_one(t_term.unwrap_components()),
            pos_b.opposite().select_one(b_term.unwrap_components()),
        ];
        resemblance(other_b, other_t, &belief_sentence, &task_sentence, context);
    }
}

/// 分离（可带变量）
pub fn detachment_with_var(
    mut task_sentence: impl Sentence,
    mut belief: impl Judgement,
    high_order_position: PremiseSource,
    position_sub_in_hi: SyllogismPosition,
    context: &mut ReasonContextConcept,
) {
    // * 🚩提取元素
    let [term_t, term_b] = [task_sentence.content(), belief.content()];
    let [main_statement, sub_content] = high_order_position.select([term_t, term_b]); // 先选中高阶陈述（任务⇒顺序不变，信念⇒顺序反转）
    let main_statement = main_statement.as_statement().unwrap();
    let component = position_sub_in_hi.select_one(main_statement.sub_pre()); // * 🚩前件

    // * 🚩非继承或否定⇒提前结束
    if !(component.instanceof_inheritance() || component.instanceof_negation()) {
        return;
    }

    // * 🚩常量词项（没有变量）⇒直接分离
    if component.is_constant() {
        return detachment(
            &task_sentence,
            &belief,
            high_order_position,
            position_sub_in_hi,
            context,
        );
    }

    // * 🚩若非常量（有变量） ⇒ 尝试统一独立变量
    let unification_i =
        variable_process::unify_find_i(component, sub_content, context.shuffle_rng_seeds());
    let [main_content_mut, sub_content_mut] =
        high_order_position.select([task_sentence.content_mut(), belief.content_mut()]); // 选取可变引用并统一
    let unified_i = unification_i.apply_to_term(main_content_mut, sub_content_mut);
    // * 🚩统一成功⇒分离
    if unified_i {
        return detachment(
            &task_sentence, // ! 这时应该统一了变量
            &belief,        // ! 这时应该统一了变量
            high_order_position,
            position_sub_in_hi,
            context,
        );
    }

    // * 🚩重新提取
    let [term_t, term_b] = [task_sentence.content(), belief.content()];
    let [main_statement, sub_content] = high_order_position.select([term_t, term_b]); // 选高阶陈述（任务⇒顺序不变，信念⇒顺序反转）
    let main_statement = main_statement.as_statement().unwrap();
    let sub_content = sub_content.as_compound().unwrap();
    // ! ⚠️【2024-06-10 17:52:44】「当前任务」与「主陈述」可能不一致：主陈述可能源自「当前信念」
    // * * 当前任务="<(*,{tom},(&,glasses,[black])) --> own>."
    // * * 主陈述="<<$1 --> (/,livingIn,_,{graz})> ==> <(*,$1,sunglasses) --> own>>"
    // * * 当前信念="<<$1 --> (/,livingIn,_,{graz})> ==> <(*,$1,sunglasses) --> own>>."
    // * 🚩当前为正向推理（任务、信念皆判断），且主句的后项是「陈述」⇒尝试引入变量

    // * 🚩使用一次性闭包代替重复的「引入变量」操作
    let intro_var_same_s_or_p = |context| {
        let task_judgement = task_sentence.unwrap_judgement(); // 避免重复借用
        let component = position_sub_in_hi.select_one(main_statement.sub_pre());
        // * 🚩【2024-08-06 20:49:18】此处必须分开
        //   * ⚠️不能保证俩`impl Judgement`是一样的类型，难以保证类型一致性
        match high_order_position {
            PremiseSource::Task => intro_var_same_subject_or_predicate(
                task_judgement,
                &belief,
                component,
                sub_content,
                position_sub_in_hi,
                context,
            ),
            PremiseSource::Belief => intro_var_same_subject_or_predicate(
                &belief,
                task_judgement,
                component,
                sub_content,
                position_sub_in_hi,
                context,
            ),
        }
    };

    let direction = context.reason_direction();
    let main_predicate_is_statement = main_statement.predicate.instanceof_statement();
    if direction == Forward && main_predicate_is_statement {
        // ? 💫【2024-06-10 17:50:36】此处逻辑尚未能完全理解
        if main_statement.instanceof_implication() {
            let s2 = main_statement.predicate.as_statement().unwrap();
            let content_subject = sub_content.as_statement().unwrap().subject;
            if s2.subject == content_subject {
                // * 📄【2024-06-10 17:46:02】一例：
                // * Task@838 "<<toothbrush --> $1> ==> <cup --> $1>>.
                // * // from task: $0.80;0.80;0.95$ <toothbrush --> [bendable]>. %1.00;0.90%
                // * // from belief: <cup --> [bendable]>. %1.00;0.90% {460 : 37} "
                // * content="<cup --> toothbrush>"
                // * s2="<cup --> $1>"
                // * mainStatement="<<toothbrush --> $1> ==> <cup --> $1>>"
                intro_var_inner(
                    sub_content.as_statement().unwrap(),
                    s2,
                    main_statement.into_compound_ref(),
                    context,
                )
            }
            intro_var_same_s_or_p(context)
        }
        // * 🚩等价⇒直接引入变量
        else if main_statement.instanceof_equivalence() {
            intro_var_same_s_or_p(context)
        }
    }
}

/// ```nal
/// {<S ==> M>, <M ==> P>} |- {<S ==> P>, <P ==> S>}
/// ```
///
/// 演绎&举例
/// * 📝一个强推理，一个弱推理
/// * 🚩【2024-08-04 21:52:34】仅传入引用，仅在需要时拷贝
fn ded_exe(
    sub: &Term,
    pre: &Term,
    task_sentence: impl Sentence,
    belief_sentence: impl Judgement,
    context: &mut ReasonContextConcept,
) {
    // * 🚩陈述有效才行
    if StatementRef::invalid_statement(sub, pre) {
        return;
    }

    // * 🚩演绎 & 举例
    deduction(
        sub.clone(),
        pre.clone(),
        &task_sentence,
        &belief_sentence,
        context,
    );
    exemplification(
        sub.clone(),
        pre.clone(),
        &task_sentence,
        &belief_sentence,
        context,
    );
}

/// ```nal
/// {<M ==> S>, <M ==> P>} |- {<S ==> P>, <P ==> S>, <S <=> P>}
/// ```
/// * 📝归因 & 归纳 & 比较
fn abd_ind_com(
    sub: Term,
    pre: Term,
    task_sentence: impl Sentence,
    belief_sentence: impl Judgement,
    context: &mut ReasonContextConcept,
) {
    // * 🚩判断结论合法性
    if StatementRef::invalid_statement(&sub, &pre) || StatementRef::invalid_pair(&sub, &pre) {
        return;
    }

    // * 🚩归因 & 归纳 & 比较
    // TODO: 【2024-07-31 11:38:26】可配置推理规则
    abduction(
        sub.clone(),
        pre.clone(),
        &task_sentence,
        &belief_sentence,
        context,
    );
    induction(
        sub.clone(),
        pre.clone(),
        &task_sentence,
        &belief_sentence,
        context,
    );
    comparison(
        sub.clone(),
        pre.clone(),
        &task_sentence,
        &belief_sentence,
        context,
    );
}
