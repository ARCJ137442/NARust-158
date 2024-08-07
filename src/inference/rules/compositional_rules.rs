//! 🎯复刻OpenNARS `nars.inference.CompositionalRules`
//!
//! * ✅【2024-05-12 00:47:43】初步复现方法API
//! * ♻️【2024-08-05 17:31:37】开始根据改版OpenNARS重写

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

/* -------------------- intersections and differences -------------------- */

/// 🆕作为「集合」操作：组合交并差
/// * 📝继承の交并差：外延交、内涵交、外延差/内涵差
///   * 同主项→外延差，同谓项→内涵差
/// * 📝蕴含の交并差：合取、析取、否定
///   * ⚠️其中「否定」不在此出现
///   * ❓是否可以`{<S ==> M>, <P ==> M>} |- {<(--,S) ==> M>, <(--,P) ==> M>}`
///
/// # 📄OpenNARS
///
/// ```nal
/// {<S ==> M>, <P ==> M>} |- {
/// <(S|P) ==> M>, <(S&P) ==> M>,
/// <(S-P) ==> M>, <(P-S) ==> M>
/// }
/// ```
pub fn compose_as_set(
    task_content: StatementRef,
    belief_content: StatementRef,
    shared_term_i: SyllogismPosition,
    component_common: &Term,
    component_t: &Term,
    component_b: &Term,
    context: &mut ReasonContextConcept,
) {
    // ! 📌分派上级「构造复合词项」已断言此处为「前向推理」
    debug_assert_eq!(context.reason_direction(), Forward);

    let truth_t = TruthValue::from(context.current_task().get_().unwrap_judgement());
    let truth_b = context.current_belief().unwrap();
    let truth_or = Some(truth_t.nal_union(truth_b)); // 后续统一类型
    let truth_and = Some(truth_t.intersection(truth_b)); // 后续统一类型
    let truth_dif;
    let [term_or, term_and, term_dif];

    // 俩闭包，调用时复制相应的词项（取得新所有权）
    let component_t = || component_t.clone();
    let component_b = || component_b.clone();
    type MakeTermFrom2 = fn(Term, Term) -> Option<Term>;

    // * 🚩根据「任务内容的类型」分派
    //   * ♻️【2024-08-07 16:48:56】现在「共有词项的位置」融入到更细节的`select`方法中
    match task_content.identifier() {
        // * 继承 * //
        // * 🚩共有在主项 ⇒ 内涵交，外延交，外延差
        //   * 📄"<M --> S>", "<M --> P>"
        // * 🚩共有在谓项 ⇒ 外延交，内涵交，内涵差
        //   * 📄"<S --> M>", "<P --> M>"
        INHERITANCE_RELATION => {
            let [make_term_and, make_term_or]: [MakeTermFrom2; 2] =
                shared_term_i.select([Term::make_intersection_ext, Term::make_intersection_int]);
            // * 🚩「与」：主⇒外延，谓⇒内涵
            term_and = make_term_or(component_t(), component_b());
            // * 🚩「或」：主⇒内涵，谓⇒外延
            term_or = make_term_and(component_t(), component_b());
            // * 🚩「差」的类型：主⇒外延差，谓⇒内涵差
            let make_term_dif: MakeTermFrom2 =
                shared_term_i.select_one([Term::make_difference_ext, Term::make_difference_int]);
            // * 🚩根据「真值负面情况」（极化情况）决定「差」的真值
            //   * 📝永远是「正面-负面」
            (term_dif, truth_dif) = match [truth_t.is_positive(), truth_b.is_positive()] {
                // * 🚩同正/同负 ⇒ 非极性 ⇒ 不予生成
                [true, true] | [false, false] => (None, None),
                // * 🚩任务正，信念负 ⇒ 词项="(任务-信念)"，真值=任务 ∩ ¬信念
                // * 📝正负流向：任务→信念
                [true, false] => (
                    make_term_dif(component_t(), component_b()),
                    Some(truth_t.intersection(&truth_b.negation())),
                ),
                // * 🚩任务负，信念正 ⇒ 词项="(信念-任务)"，真值=信念 ∩ ¬任务
                // * 📝正负流向：信念→任务
                [false, true] => (
                    make_term_dif(component_b(), component_t()),
                    Some(truth_b.intersection(&truth_t.negation())),
                ),
            }
        }
        // * 蕴含 * //
        // * 🚩共有在主项 ⇒ 合取、析取
        //   * 📄"<M ==> S>", "<M ==> P>"
        // * 🚩共有在谓项 ⇒ 析取、合取
        //   * 📄"<S ==> M>", "<P ==> M>"
        IMPLICATION_RELATION => {
            let [make_term_and, make_term_or]: [MakeTermFrom2; 2] =
                shared_term_i.select([Term::make_conjunction, Term::make_disjunction]);
            // * 🚩「与」主⇒合取，谓⇒析取
            term_and = make_term_and(component_t(), component_b());
            // * 🚩「或」主⇒析取，谓⇒合取
            term_or = make_term_or(component_t(), component_b());
            // * 🚩没有「差」
            (term_dif, truth_dif) = (None, None);
        }
        // * 🚩其它情况都没有⇒直接返回
        _ => return,
    }

    // 下面开始统一构造结论
    let component_common = || component_common.clone();
    let mut term_truths = [
        (term_or, truth_or),
        (term_and, truth_and),
        (term_dif, truth_dif),
    ]
    .into_iter();
    // * 🚩遍历并跳过空值
    while let Some((Some(term), Some(truth))) = term_truths.next() {
        // * 🚩统一导出结论
        //   * 主项 ⇒ "<公共项 ==> 新词项>"
        //   * 谓项 ⇒ "<新词项 ==> 公共项>"
        let [subject, predicate] = shared_term_i.select([component_common(), term]);
        // * 🚩词项：不能跟任务、信念 内容相同
        let content = unwrap_or_return!(
            ?Term::make_statement(&task_content, subject, predicate)
            => continue
        );
        if content == *task_content || content == *belief_content {
            continue;
        }

        // * 🚩预算：复合前向
        let budget = context.budget_compound_forward(&truth, &content);

        // * 🚩结论
        context.double_premise_task(content, Some(truth), budget);
    }
}

/// 解构交并差
/// * ️📝其规则正好是上头「[建构交并差](compose_as_set)」的逆
///
/// # 📄OpenNARS
///
/// ```nal
/// {<(S|P) ==> M>, <P ==> M>} |- <S ==> M>
/// ```
#[doc(alias = "decompose_compound")]
pub fn decompose_as_set(
    task_content: StatementRef,
    compound: CompoundTermRef,
    component: &Term,
    component_common: &Term,
    side: SyllogismPosition,
    compound_from: PremiseSource,
    context: &mut ReasonContextConcept,
) {
    // * 🚩「参考的复合词项」是 陈述/像 ⇒ 不解构
    if compound.instanceof_statement() || compound.instanceof_image() {
        return;
    }

    // ! 只能是判断句、正向推理
    // * 📝【2024-08-07 17:10:20】上游调用者已经限制了「仅判断句」
    debug_assert!(context.current_task().get_().is_judgement());

    // * 🚩将当前元素从复合词项中移除
    let term2 = unwrap_or_return!(
        ?compound.reduce_components(component)
    );

    // * 🚩词项 * //
    // * 🚩共有前项：[共同元素, term2]
    // * 🚩共有后项：[term2, 共同元素]
    let [subject, predicate] = side.select([component_common.clone(), term2.clone()]);
    let content = unwrap_or_return!(?Term::make_statement(&task_content, subject, predicate));

    // * 🚩真值 * //
    let belief_truth: TruthValue = context.current_belief().unwrap().into();
    let task_truth: TruthValue = context.current_task().get_().unwrap_judgement().into();
    let [v1, v2] = compound_from.select([task_truth, belief_truth]);

    /// 反向的「合取消去」
    /// * 🎯格式整齐——让后边直接使用真值函数（指针）而无需凑表达式
    fn reduce_disjunction_rev(v1: &impl Truth, v2: &impl Truth) -> TruthValue {
        v2.reduce_disjunction(v1)
    }

    // * 🚩预先获取各个上下文「主项/谓项」的「与或非」真值函数
    let [truth_f_and, truth_f_or]: [TruthFDouble; 2] = side.select([
        TruthFunctions::reduce_conjunction,
        TruthFunctions::reduce_disjunction,
    ]);
    let truth_f_not = match *compound.component_at(0).unwrap() == *component {
        // * 🚩内容正好为被减项 ⇒ 析取（反向）
        true => reduce_disjunction_rev,
        // * 🚩其它 ⇒ 合取否定
        false => TruthFunctions::reduce_conjunction_neg,
    };

    // * 🚩根据各词项类型分派
    let task_content_type = task_content.identifier();
    let compound_type = compound.identifier();
    let truth_f: TruthFDouble = match task_content_type {
        // * 🚩任务内容 <: 继承
        INHERITANCE_RELATION => match compound_type {
            // * 🚩外延交 ⇒ 合取/析取
            INTERSECTION_EXT_OPERATOR => truth_f_and,
            // * 🚩内涵交 ⇒ 析取/合取
            INTERSECTION_INT_OPERATOR => truth_f_or,
            // * 🚩外延集-外延集 ⇒ 析取/合取
            SET_EXT_OPERATOR if component.instanceof_set_ext() => truth_f_or,
            // * 🚩内涵集-内涵集 ⇒ 合取/析取
            SET_INT_OPERATOR if component.instanceof_set_int() => truth_f_and,
            // * 🚩外延差 @ 主项 ⇒ 差
            DIFFERENCE_EXT_OPERATOR if side == Subject => truth_f_not,
            // * 🚩内涵差 @ 谓项 ⇒ 差
            DIFFERENCE_INT_OPERATOR if side == Predicate => truth_f_not,
            // * 🚩其它 ⇒ 否决
            _ => return,
        },
        // * 🚩任务内容 <: 蕴含
        IMPLICATION_RELATION => match compound_type {
            // * 🚩合取 ⇒ 合取/析取
            CONJUNCTION_OPERATOR => truth_f_and,
            // * 🚩析取 ⇒ 析取/合取
            DISJUNCTION_OPERATOR => truth_f_or,
            // * 🚩其它 ⇒ 否决
            _ => return,
        },
        // * 🚩其它 ⇒ 否决
        _ => return,
    };
    let truth = truth_f(&v1, &v2);

    // * 🚩预算 * //
    let budget = context.budget_compound_forward(&truth, &content);

    // * 🚩结论 * //
    context.double_premise_task(content, Some(truth), budget);
}

/// # 📄OpenNARS
///
/// ```nal
/// {(||, S, P), P} |- S
// {(&&, S, P), P} |- S
/// ```
pub fn decompose_statement(
    compound: CompoundTermRef,
    component: &Term,
    compound_from: PremiseSource,
    context: &mut ReasonContextConcept,
) {
    let task_truth = context
        .current_task()
        .get_()
        .as_judgement()
        .map(TruthValue::from);
    let belief_truth = TruthValue::from(context.current_belief().unwrap());
    // * 🚩删去指定的那个元素，用删去之后的剩余元素做结论
    let content = unwrap_or_return!(?compound.reduce_components(component));
    let direction = context.reason_direction();

    match direction {
        // * 🚩前向推理：直接用于构造信念
        Forward => {
            let task_truth = task_truth.unwrap();
            // * 🚩选取前提真值 | ⚠️前后件语义不同
            let [v1, v2] = compound_from.select([&task_truth, &belief_truth]);
            // * 🚩选取真值函数
            let truth_f: TruthFDouble = match compound.identifier() {
                CONJUNCTION_OPERATOR => TruthFunctions::reduce_conjunction,
                DISJUNCTION_OPERATOR => TruthFunctions::reduce_disjunction,
                _ => return,
            };
            // * 🚩构造真值、预算值，双前提结论
            let truth = truth_f(v1, v2);
            let budget = context.budget_compound_forward(&truth, &content);
            context.double_premise_task(content, Some(truth), budget)
        }
        // * 🚩反向推理：尝试答问
        Backward => {
            // * 📄(||,A,B)? + A. => B?
            // * 🚩先将剩余部分作为「问题」提出
            // ! 📄原版bug：当输入 (||,A,?1)? 时，因「弹出的变量复杂度为零」预算推理「除以零」爆炸
            if !content.is_zero_complexity() {
                let budget = context.budget_compound_backward(&content);
                context.double_premise_task(content.clone(), None, budget);
            }
            let task_rc = context.current_task(); // ! 这俩后边要手动drop
            let task_ref = task_rc.get_(); // ! 这俩后边要手动drop
            let task = &*task_ref;
            // * 🚩再将对应有「概念」与「信念」的内容作为新的「信念」放出
            // special inference to answer conjunctive questions with query variables
            if !task.content().contain_var_q() {
                return;
            }
            // * 🚩只有在「回答合取问题」时，取出其中的项构建新任务
            let content_concept = unwrap_or_return!(?context.term_to_concept(&content));
            // * 🚩只在「内容对应了概念」时，取出「概念」中的信念
            let content_belief = unwrap_or_return!(
                ?content_concept.get_belief(task)
            );
            // * 🚩只在「概念中有信念」时，以这个信念作为「当前信念」构建新任务
            let new_stamp = Stamp::from_merge_unchecked(
                task,
                content_belief, // * 🚩实际上就是需要与「已有信念」的证据基合并
                context.time(),
                context.max_evidence_base_length(),
            );
            // * 🚩【2024-06-07 13:41:16】现在直接从「任务」构造新的「预算值」
            let content_task = Task::from_input(content_belief.clone(), task);
            // ! 🚩【2024-05-19 20:29:17】现在移除：直接在「导出结论」处指定
            let conjunction = unwrap_or_return!(
                ?Term::make_conjunction(component.clone(), content)
            );
            // * ↓不会用到`context.getCurrentTask()`、`newStamp`
            let truth = content_belief.intersection(&belief_truth);
            // * ↓不会用到`context.getCurrentTask()`、`newStamp`
            drop(task_ref);
            drop(task_rc);
            let budget = context.budget_compound_forward(&truth, &conjunction);
            // ! ⚠️↓会用到`context.getCurrentTask()`、`newStamp`：构建新结论时要用到
            // * ✅【2024-05-21 22:38:52】现在通过「参数传递」抵消了对`context.getCurrentTask`的访问
            context.double_premise_task_compositional(
                &content_task,
                conjunction,
                Some(truth),
                budget,
                new_stamp,
            );
        }
    }
}

/* --------------- rules used for variable introduction --------------- */

/// 🆕入口之一：变量引入
/// ! ⚠️【2024-07-23 12:20:18】逻辑未完全被测试覆盖，代码理解度低
/// * 📝【2024-07-23 12:04:33】OpenNARS 3.1.0仍然没有样例注释……
/// * 📄一例（平凡情况）：
///   * originalMainSentence = "<<$1 --> swimmer> ==> <$1 --> bird>>"
///   * subSentence = "<bird --> animal>"
///   * component = "<$1 --> bird>"
///   * subContent = "<bird --> animal>"
///   * index = 1 @ originalMainSentence
///   * => "<<$1 --> swimmer> ==> <$1 --> bird>>"
pub fn intro_var_same_subject_or_predicate(
    original_main_sentence: &impl Judgement,
    sub_sentence: &impl Judgement,
    component: &Term,
    sub_content: CompoundTermRef,
    side: SyllogismPosition,
    context: &mut ReasonContextConcept,
) {
    // * 🚩词项 * //
    // * 🚩仅对复合词项
    // * 🚩对内部内容，仅适用于「继承×继承」与「相似×相似」
    // CompoundTerm result = mainCompound;
    // wouldn't make sense to create a conjunction here,
    // would contain a statement twice
    // ! ⚠️【2024-07-23 12:17:44】目前还没真正触发过此处逻辑
    // ! * 诸多尝试均被「变量分离规则」等 截胡
    // * ✅不怕重名：现在始终是「最大词项的最大id+1」的模式
    // ! ⚠️【2024-07-23 12:17:44】目前还没真正触发过此处逻辑
    // ! * 诸多尝试均被「变量分离规则」等 截胡
    /*
     * 📄已知如下输入无法触发：
     * <swam --> swimmer>.
     * <swam --> bird>.
     * <bird --> swimmer>.
     * <<$1 --> swimmer> ==> <$1 --> bird>>.
     * <<bird --> $1> ==> <swimmer --> $1>>.
     * 1000
     */
    // * ✅不怕重名：现在始终是「最大词项的最大id+1」的模式
    // ? 【2024-07-23 12:20:27】为何要重复得出结果
    // * 🚩真值 * //
    // * 🚩预算 * //
    // * 🚩结论 * //
}

/// Introduce a dependent variable in an outer-layer conjunction
/// * 📝「变量外引入」系列规则
///
/// * 📌导出结论：「正反似合」
///   * 外延正传递（归因/归纳）
///   * 外延反传递（归因/归纳）
///   * 相似の传递（比较）
///   * 因变量引入（合取）
///
/// * 📄@主项: "<M --> S>" × "<M --> P>"
///   * => "<<$1 --> S> ==> <$1 --> P>>"
///   * => "<<$1 --> P> ==> <$1 --> S>>"
///   * => "<<$1 --> S> <=> <$1 --> P>>"
///   * => "(&&,<#1 --> S>,<#1 --> P>)"
///
/// * 📄@谓项: "<S --> M>" × "<P --> M>"
///   * => "<<S --> $1> ==> <P --> $1>>"
///   * => "<<P --> $1> ==> <S --> $1>>"
///   * => "<<P --> $1> <=> <S --> $1>>"
///   * => "(&&,<P --> #1>,<S --> #1>)"
pub fn intro_var_outer(
    task_content: StatementRef,
    belief_content: StatementRef,
    shared_term_i: SyllogismPosition,
    context: &mut ReasonContextConcept,
) {
    // * 🚩任务/信念 的真值 | 仅适用于前向推理
    // * 🚩词项初步：引入变量 * //
    // * 🚩继续分派：词项、真值、预算、结论 * //
}

/// 🆕以「变量外引入」的内部词项，计算「引入状态」陈述
/// * 📌引入的是「独立变量/自变量」"$"
/// * 🎯产生的陈述（二元组）用于生成新结论内容
fn intro_var_states_ind(
    task_content: Statement,
    belief_content: Statement,
    side: SyllogismPosition,
) -> [Term; 2] {
    // * 🚩根据索引决定「要组成新陈述的词项的位置」
    // index == 1
    // * 🚩寻找「第二个相同词项」并在内容中替换 | 对「外延像@0」「内涵像@1」的特殊处理
    // * 📌【2024-07-23 13:19:30】此处原码与secondCommonTerm相同，故提取简并
    // * 🚩产生一个新的独立变量，并以此替换
    // ! ⚠️在此期间【修改】其【所指向】的词项
    // * 🚩返回：从元素构造继承陈述
    todo!()
}

/// 🆕以「变量外引入」的内部词项，计算「引入状态」陈述
/// * 📌引入的是「独立变量/自变量」"$"
/// * 🎯产生的陈述（二元组）用于生成新结论内容
fn intro_var_states_dep(
    task_content: Statement,
    belief_content: Statement,
    side: SyllogismPosition,
) -> [Term; 2] {
    todo!()
}

/// 「变量外引入」规则 结论1
/// * 📄"<bird --> animal>" × "<bird --> swimmer>"
///   * => "<<$1 --> animal> ==> <$1 --> swimmer>>"
/// * 📄"<sport --> competition>" × "<chess --> competition>"
///   * => "<<sport --> $1> ==> <chess --> $1>>"
fn intro_var_outer1(
    state_1: Term,
    state_2: Term,
    truth_t: &impl Truth,
    truth_b: &impl Truth,
    context: &mut ReasonContextConcept,
) {
    // TODO
}

/// 「变量外引入」规则 结论2
/// * 📄"<bird --> animal>" × "<bird --> swimmer>"
///   * => "<<$1 --> swimmer> ==> <$1 --> animal>>"
/// * 📄"<sport --> competition>" × "<chess --> competition>"
///   * => "<<chess --> $1> ==> <sport --> $1>>"
fn intro_var_outer2(
    state_1: Term,
    state_2: Term,
    truth_t: &impl Truth,
    truth_b: &impl Truth,
    context: &mut ReasonContextConcept,
) {
    // TODO
}

/// 「变量外引入」规则 结论3
/// * 📄"<bird --> animal>" × "<bird --> swimmer>"
///   * => "<<$1 --> animal> <=> <$1 --> swimmer>>"
/// * 📄"<sport --> competition>" × "<chess --> competition>"
///   * => "<<chess --> $1> <=> <sport --> $1>>"
fn intro_var_outer3(
    state_1: Term,
    state_2: Term,
    truth_t: &impl Truth,
    truth_b: &impl Truth,
    context: &mut ReasonContextConcept,
) {
    // TODO
}

/// 「变量外引入」规则 结论4
/// * 📄"<bird --> animal>" × "<bird --> swimmer>"
///   * => "(&&,<#1 --> animal>,<#1 --> swimmer>)"
/// * 📄"<sport --> competition>" × "<chess --> competition>"
///   * => "(&&,<chess --> #1>,<sport --> #1>)"
fn intro_var_outer4(
    state_1: Term,
    state_2: Term,
    truth_t: &impl Truth,
    truth_b: &impl Truth,
    context: &mut ReasonContextConcept,
) {
    // TODO
}

/// * 📝入口2：变量内引入
///
/// # 📄OpenNARS
///
/// ```nal
/// {<M --> S>, <C ==> <M --> P>>} |- <(&&, <#x --> S>, C) ==> <#x --> P>>
/// {<M --> S>, (&&, C, <M --> P>)} |- (&&, C, <<#x --> S> ==> <#x --> P>>)
/// ```
pub fn intro_var_inner(
    premise_1: StatementRef,
    premise_2: StatementRef,
    old_compound: CompoundTermRef,
    context: &mut ReasonContextConcept,
) {
    // * 🚩仅适用于前向推理
    // * 🚩前提1与前提2必须是相同类型，且「旧复合词项」不能包括前提1
    // * 🚩计算共有词项
    // * 🚩继续向下分派
}

/// 🆕以「变量内引入」的内部词项，计算「共有词项」
/// * 🎯产生的词项（二元组/空）用于生成新结论内容
fn intro_var_commons(premise_1: Statement, premise_2: Statement) -> [Term; 2] {
    // * 🚩轮流判等以决定所抽取的词项
    // * 🚩共有主项 ⇒ 11→(12×22)
    // * 🚩共有谓项 ⇒ 12→(11×21)
    // * 🚩无共有词项⇒空
    todo!()
}

/// 「变量内引入」规则 结论1
/// * 📝引入第二个变量，并在替换后产生一个合取
///
/// * 📄"<{lock1} --> lock>" × "<{lock1} --> (/,open,$1,_)>"
/// * * @ "<<$1 --> key> ==> <{lock1} --> (/,open,$1,_)>>"
/// * * => "(&&,<#2 --> lock>,<<$1 --> key> ==> <#2 --> (/,open,$1,_)>>)"
///
/// * 📄"<{Tweety} --> [chirping]>" × "<robin --> [chirping]>"
/// * * @ "(&&,<robin --> [chirping]>,<robin --> [with_wings]>)"
/// * * => "(&&,<robin --> #1>,<robin --> [with_wings]>,<{Tweety} --> #1>)"
fn intro_var_inner1(
    premise_1: Statement,
    old_compound: CompoundTerm,
    truth_t: &impl Truth,
    truth_b: &impl Truth,
    common_term_1: Term,
    common_term_2: Term,
    context: &mut ReasonContextConcept,
) {
    // * 🚩词项 * //
    // * 🚩将「共有词项」替换成变量
    // * 🚩真值 * //
    // * 🚩预算 * //
    // * 🚩结论 * //
}

/// 「变量内引入」规则 结论2
/// * 📝引入第二个变量，并在替换后产生一个蕴含
///
/// * 📄"<{lock1} --> lock>" × "<{lock1} --> (/,open,$1,_)>"
/// * * @ "<<$1 --> key> ==> <{lock1} --> (/,open,$1,_)>>"
/// * * => "<(&&,<$1 --> key>,<$2 --> lock>) ==> <$2 --> (/,open,$1,_)>>"
///
/// * 📄"<{Tweety} --> [chirping]>" × "<robin --> [chirping]>"
/// * * @ "(&&,<robin --> [chirping]>,<robin --> [with_wings]>)"
/// * * => "<<{Tweety} --> $1> ==> (&&,<robin --> $1>,<robin --> [with_wings]>)>"
fn intro_var_inner2(
    premise_1: Statement,
    old_compound: CompoundTerm,
    truth_t: &impl Truth,
    truth_b: &impl Truth,
    common_term_1: Term,
    common_term_2: Term,
    context: &mut ReasonContextConcept,
) {
    // * 🚩词项 * //
    // * 🚩将「共有词项」替换成变量
    // * 🚩真值 * //
    // * 🚩前提 == 任务 ⇒ 归纳 信念→任务
    // * 🚩前提 != 任务 ⇒ 归纳 任务→信念
    // * 🚩预算 * //
    // * 🚩结论 * //
}

/// # 📄OpenNARS
///
/// Introduce a second independent variable into two terms with a common
/// component
fn second_common_term([term1, term2]: [&Term; 2], side: SyllogismPosition) -> &Term {
    // * 📄1: 都是主项，且均为外延像
    // * 📄2: 都是谓项，且均为内涵像
    // * 🚩先试第一个
    // * 🚩尝试不到？考虑第二个/用第二个覆盖
    // * 🚩再试第二个
    // * 🚩尝试不到就是尝试不到
    // * 🚩根据中间条件多次覆盖，最终拿到一个引用
    todo!()
}

/// 因变量消元
/// * 📝用于处理类似「存在变量」的情况
///
/// # 📄OpenNARS
///
/// ```nal
/// {(&&, <#x() --> S>, <#x() --> P>), <M --> P>} |- <M --> S>
/// ```
pub fn eliminate_var_dep(
    compound: CompoundTermRef,
    component: &Term,
    compound_from: PremiseSource,
    context: &mut ReasonContextConcept,
) {
    // * 🚩提取参数 * //
    // * 🚩词项 * //
    // * 🚩真值 * //
    // * 🚩复合词项来自任务 ⇒ 任务，信念
    // * 🚩否则 ⇒ 信念，任务
    // * 🚩预算 * //
    // * 🚩复合词项来自任务 ⇒ 反向
    // * 🚩其它 ⇒ 反向弱推理
    // * 🚩前向推理
    // * 🚩结论 * //
}

#[cfg(test)]
mod tests {
    use crate::expectation_tests;

    expectation_tests! {
        compose_as_sub_inh_and: {
            "
            nse <S --> M>.
            nse <P --> M>.
            cyc 10
            "
            => OUT "<(&,S,P) --> M>" in outputs
        }

        compose_as_sub_inh_or: {
            "
            nse <S --> M>.
            nse <P --> M>.
            cyc 10
            "
            => OUT "<(|,S,P) --> M>" in outputs
        }

        compose_as_sub_inh_not_sp: {
            "
            nse <S --> M>. %1%
            nse <P --> M>. %0%
            cyc 10
            " // 主项：`1` ~ `0`
            => OUT "<(~,S,P) --> M>" in outputs
        }

        compose_as_sub_inh_not_ps: {
            "
            nse <S --> M>. %0%
            nse <P --> M>. %1%
            cyc 10
            " // 主项：`1` ~ `0`
            => OUT "<(~,P,S) --> M>" in outputs
        }

        compose_as_sub_imp_and: {
            "
            nse <S ==> M>.
            nse <P ==> M>.
            cyc 10
            "
            => OUT "<(&&,S,P) ==> M>" in outputs
        }

        compose_as_sub_imp_or: {
            "
            nse <S ==> M>.
            nse <P ==> M>.
            cyc 10
            "
            => OUT "<(||,S,P) ==> M>" in outputs
        }

        compose_as_pre_inh_and: {
            "
            nse <M --> S>.
            nse <M --> P>.
            cyc 10
            "
            => OUT "<M --> (&,S,P)>" in outputs
        }

        compose_as_pre_inh_or: {
            "
            nse <M --> S>.
            nse <M --> P>.
            cyc 10
            "
            => OUT "<M --> (|,S,P)>" in outputs
        }

        compose_as_pre_inh_not_sp: {
            "
            nse <M --> S>. %1%
            nse <M --> P>. %0%
            cyc 10
            " // 谓项：`1` - `0`
            => OUT "<M --> (-,S,P)>" in outputs
        }

        compose_as_pre_inh_not_ps: {
            "
            nse <M --> S>. %0%
            nse <M --> P>. %1%
            cyc 10
            " // 谓项：`1` - `0`
            => OUT "<M --> (-,P,S)>" in outputs
        }

        compose_as_pre_imp_and: {
            "
            nse <M ==> S>.
            nse <M ==> P>.
            cyc 10
            "
            => OUT "<M ==> (||,S,P)>" in outputs
        }

        compose_as_pre_imp_or: {
            "
            nse <M ==> S>.
            nse <M ==> P>.
            cyc 10
            "
            => OUT "<M ==> (&&,S,P)>" in outputs
        }

        decompose_as_sub_inh_and: {
            "
            nse <(&,S,P) --> M>.
            nse <S --> M>.
            cyc 20
            "
            => OUT "<P --> M>" in outputs
        }

        decompose_as_sub_inh_or: {
            "
            nse <(|,S,P) --> M>.
            nse <S --> M>.
            cyc 20
            "
            => OUT "<P --> M>" in outputs
        }

        decompose_as_sub_inh_not_sp: {
            "
            nse <(~,S,P) --> M>. %1%
            nse <S --> M>. %0%
            cyc 20
            " // 主项：`1` ~ `0`
            => OUT "<P --> M>" in outputs
        }

        decompose_as_sub_inh_not_ps: {
            "
            nse <(~,P,S) --> M>. %0%
            nse <S --> M>. %1%
            cyc 20
            " // 主项：`1` ~ `0`
            => OUT "<P --> M>" in outputs
        }

        // ! ❌【2024-08-07 17:59:52】此测试失败：蕴含+合取⇒链接「复合条件」不走组合规则
        // decompose_as_sub_imp_and: {
        //     "
        //     nse <(&&,S,P) ==> M>.
        //     nse <S ==> M>.
        //     cyc 1000
        //     "
        //     => OUT "<P ==> M>" in outputs
        // }

        decompose_as_sub_imp_or: {
            "
            nse <(||,S,P) ==> M>.
            nse <S ==> M>.
            cyc 20
            "
            => OUT "<P ==> M>" in outputs
        }

        decompose_as_pre_inh_and: {
            "
            nse <M --> (&,S,P)>.
            nse <M --> S>.
            cyc 20
            "
            => OUT "<M --> P>" in outputs
        }

        decompose_as_pre_inh_or: {
            "
            nse <M --> (|,S,P)>.
            nse <M --> S>.
            cyc 20
            "
            => OUT "<M --> P>" in outputs
        }

        decompose_as_pre_inh_not_sp: {
            "
            nse <M --> (-,S,P)>. %1%
            nse <M --> S>. %0%
            cyc 20
            " // 谓项：`1` - `0`
            => OUT "<M --> P>" in outputs
        }

        decompose_as_pre_inh_not_ps: {
            "
            nse <M --> (-,P,S)>. %0%
            nse <M --> S>. %1%
            cyc 20
            " // 谓项：`1` - `0`
            => OUT "<M --> P>" in outputs
        }

        decompose_as_pre_imp_and: {
            "
            nse <M ==> (||,S,P)>.
            nse <M ==> S>.
            cyc 20
            "
            => OUT "<M ==> P>" in outputs
        }

        decompose_as_pre_imp_or: {
            "
            nse <M ==> (&&,S,P)>.
            nse <M ==> S>.
            cyc 20
            "
            => OUT "<M ==> P>" in outputs
        }

        decompose_compound_pre_inh_and: {
            "
            nse <M --> (&,S,P)>.
            nse <M --> S>.
            cyc 10
            "
            => OUT "<M --> P>" in outputs
        }

        decompose_statement_conjunction: {
            "
            nse (&&,S,P).
            nse P.
            cyc 10
            "
            => OUT "S" in outputs
        }

        decompose_statement_disjunction: {
            "
            nse (||,S,P).
            nse P.
            cyc 10
            "
            => OUT "S" in outputs
        }

        decompose_statement_conjunction_backward: {
            "
            nse (&&,S,P).
            nse S?
            cyc 10
            "
            => ANSWER "S" in outputs
        }

        decompose_statement_disjunction_backward: {
            "
            nse (||,S,P).
            nse S?
            cyc 10
            "
            => ANSWER "S" in outputs
        }
    }
}
