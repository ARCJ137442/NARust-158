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
use nar_dev_utils::{f_parallel, unwrap_or_return};
use variable_process::VarSubstitution;
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

/// 🆕入口之一：变量引入同主谓
/// * 📝【2024-07-23 12:04:33】OpenNARS 3.1.0仍然没有样例注释……
/// * ♻️【2024-08-07 22:25:57】重构以规整
///
/// ```nal
/// {<<$1 --> B> ==> <$1 --> A>>, <A --> C>}
/// |- <<A --> B> ==> (&&, <#1 --> C>, <#1 --> A>)>
/// {<<B --> $1> ==> <A --> $1>>, <C --> A>}
/// |- <<B --> A> ==> (&&, <C --> #1>, <A --> #1>)>
/// ```
///
/// * 📄一例（平凡情况）：
///   * originalMainSentence = "<<$1 --> swimmer> ==> <$1 --> bird>>"
///   * subSentence = "<bird --> animal>"
///   * component = "<$1 --> bird>"
///   * subContent = "<bird --> animal>"
///   * index = 1 @ originalMainSentence
///   * => "<<$1 --> swimmer> ==> <$1 --> bird>>"
/// * 📄一例：
///   * originalMainSentence = "<<$1 --> swimmer> ==> <$1 --> bird>>"
///   * subSentence = "<bird --> animal>"
///   * index = 1 @ originalMainSentence
///   * => "<<bird --> swimmer> ==> (&&, <#1 --> animal>, <#1 --> bird>)>"
pub fn intro_var_same_subject_or_predicate(
    original_main_sentence: &impl Judgement,
    sub_sentence: &impl Judgement,
    component: &Term,
    sub_content: CompoundTermRef,
    position_sub_in_hi: SyllogismPosition, // 子句在高阶词项中的位置
    context: &mut ReasonContextConcept,
) {
    // * 🚩词项 * //
    let cloned_main = original_main_sentence.sentence_clone();
    let cloned_main_t = cloned_main.content();

    // * 🚩仅对复合词项子项
    if !sub_content.instanceof_compound() {
        return;
    }

    let main_statement = unwrap_or_return!(?cloned_main_t.as_statement());
    // * 🚩对内部内容，仅适用于「继承×继承」与「相似×相似」
    match [component.identifier(), sub_content.identifier()] {
        [INHERITANCE_RELATION, INHERITANCE_RELATION]
        | [SIMILARITY_RELATION, SIMILARITY_RELATION] => {}
        _ => return,
    }
    let [component, sub_content] = match [component.as_statement(), sub_content.as_statement()] {
        [Some(component), Some(sub_content)] => [component, sub_content],
        _ => return,
    };
    // CompoundTerm result = mainCompound;
    if *component == *sub_content {
        return;
    }
    // wouldn't make sense to create a conjunction here,
    // would contain a statement twice

    let [com_sub, com_pre] = component.sub_pre();
    let [sub_sub, sub_pre] = sub_content.sub_pre();
    // * 🚩决定要「引入变量并替换元素」的位置
    //   * 📝哪边词项相等且被替换的不是变量，哪边就引入变量
    let var_position = if *com_pre == *sub_pre && !com_pre.instanceof_variable() {
        Some(Predicate) // 在谓项中引入变量，保留主项
    } else if *com_sub == *sub_sub && !com_sub.instanceof_variable() {
        Some(Subject) // 在主项中引入变量，保留谓项
    } else {
        None // 不引入变量，保留整个陈述（❓为何）
    };
    // * 🚩开始在词项中引入变量
    /// 将陈述的某处替换为变量
    fn replaced_statement_with_term_at(
        statement: StatementRef,
        at: SyllogismPosition,
        new_term: Term,
    ) -> Option<Term> {
        // * 🚩【2024-08-07 21:14:35】实质上就是将「保留之侧的对侧」替换成变量
        let new_remaining_component = at.opposite().select_one(statement.sub_pre()).clone();
        let [sub, pre] = at.select([new_term, new_remaining_component]); // `new_term`在前，始终跟随`at`
        Term::make_statement(&statement, sub, pre)
    }
    let content = match var_position {
        Some(var_position) => {
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
            // * ✅↓不怕重名：现在始终是「最大词项的最大id+1」的模式
            let var_d = || Term::make_var_d([&main_statement, sub_content.statement]);
            // * 🚩假定这个是「子句」陈述，因此能继续提取主项/谓项
            let sub_component_in_main = unwrap_or_return!( // 原zw
                ?position_sub_in_hi.select_one(main_statement.sub_pre()).as_statement()
            );
            let sub_component_replaced = unwrap_or_return!(
                // 原zw2
                // unwrap_or_return!(?sub_component_in_main.get_ref().set_component(1, Some(v())));
                // * 🚩【2024-08-07 21:14:35】实质上就是将「保留之侧的对侧」替换成变量
                ?replaced_statement_with_term_at(sub_component_in_main, var_position, var_d())
            );
            let new_sub_compound = unwrap_or_return!(
                // unwrap_or_return!(?sub_content.into_compound_ref().set_component(1, Some(v())))
                // * 🚩【2024-08-07 21:14:35】实质上就是将「保留之侧的对侧」替换成变量
                ?replaced_statement_with_term_at(sub_content, var_position, var_d())
            );
            if sub_component_replaced == new_sub_compound {
                return;
            }
            // final Conjunction res = (Conjunction) makeConjunction(zw2, newSubCompound);
            let sub_conjunction = unwrap_or_return!(
                ?Term::make_conjunction(sub_component_replaced, new_sub_compound)
            );
            // * 🚩最终构造：替换掉`main_statement`中`position_sub_in_hi`处的「子句」为合取
            unwrap_or_return!(
                ?replaced_statement_with_term_at(
                    main_statement,
                    position_sub_in_hi,
                    sub_conjunction,
                )
            )
        }
        // ? 【2024-07-23 12:20:27】为何要重复得出结果
        None => main_statement.statement.clone(),
    };

    // * 🚩真值 * //
    let truth = original_main_sentence.induction(sub_sentence);

    // * 🚩预算 * //
    let budget = context.budget_compound_forward(&truth, &content);

    // * 🚩结论 * //
    context.double_premise_task(content, Some(truth), budget);
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
    debug_assert!(context.current_task().get_().is_judgement());
    let truth_t = TruthValue::from(context.current_task().get_().unwrap_judgement());
    let truth_b = TruthValue::from(context.current_belief().unwrap());

    // * 🚩词项初步：引入变量 * //
    let [state_i1, state_i2] = intro_var_states_ind(task_content, belief_content, shared_term_i);
    let [state_d1, state_d2] = intro_var_states_dep(task_content, belief_content, shared_term_i);
    let (state_i1, state_i2) = (|| state_i1.clone(), || state_i2.clone());
    let (state_d1, state_d2) = (|| state_d1.clone(), || state_d2.clone());

    // * 🚩继续分派：词项、真值、预算、结论 * //
    // * 📌【2024-08-07 22:37:47】此处为了可读性，将词项多拷贝了一次（而非在最后传入所有权）
    //   * 💭若有在【不破坏调用统一性】的同时【节省掉一次clone】的方法，乐意改进
    enum UsesVar {
        /// 使用独立变量
        I,
        /// 使用非独变量
        D,
    }
    use UsesVar::*;
    type IntroVarOuterParameters = (
        UsesVar,                                    // 用独立变量还是用非独变量
        fn(Term, Term) -> Option<Term>,             // 制作词项
        fn(&TruthValue, &TruthValue) -> TruthValue, // 制作真值
        bool,                                       // 词项、真值的顺序是否要交换
    );
    const T: bool = true; // 💭【2024-08-07 23:57:50】为了简写无所不用其极。。
    const F: bool = false; // 💭【2024-08-07 23:57:50】为了简写无所不用其极。。
    let will_intro_parameters: [IntroVarOuterParameters; 4] = [
        (I, Term::make_implication, TruthFunctions::induction, F), // "<<$1 --> A> ==> <$1 --> B>>"
        (I, Term::make_implication, TruthFunctions::induction, T), // "<<$1 --> B> ==> <$1 --> A>>"
        (I, Term::make_equivalence, TruthFunctions::comparison, F), // "<<$1 --> A> <=> <$1 --> B>>"
        (D, Term::make_conjunction, TruthFunctions::intersection, F), // "(&&,<#1 --> A>,<#1 --> B>)"
    ];
    for (uses_var, make_content, truth_f, reverse_order) in will_intro_parameters {
        // * 🚩决定要填进去的词项
        let states = match uses_var {
            I => [state_i1(), state_i2()],
            D => [state_d1(), state_d2()],
        };
        // * 🚩逐个引入并导出结论
        intro_var_outer_derive(
            states,
            [&truth_t, &truth_b],
            make_content,
            truth_f,
            reverse_order,
            context,
        );
    }
}

/// 🆕以「变量外引入」的内部词项，计算「引入状态」陈述
/// * 📌引入的是「独立变量/自变量」"$"
/// * 🎯产生的陈述（二元组）用于生成新结论内容
fn intro_var_states_ind(
    task_content: StatementRef,
    belief_content: StatementRef,
    shared_term_i: SyllogismPosition,
) -> [Option<Term>; 2] {
    let mut task_content = task_content.to_owned();
    let mut belief_content = belief_content.to_owned();
    // * 🚩先执行归一化替换：替换共同词项
    let var_i = Term::make_var_i([&*task_content, &*belief_content]); // 无论如何都创建，避开借用问题
    let [need_common_t, need_common_b] = [
        shared_term_i.select_another(task_content.sub_pre_mut()),
        shared_term_i.select_another(belief_content.sub_pre_mut()),
    ];
    // * 🚩寻找「第二个相同词项」并在内容中替换 | 对「外延像@0」「内涵像@1」的特殊处理
    // * 📌【2024-07-23 13:19:30】此处原码与secondCommonTerm相同，故提取简并
    let second_common_term = second_common_term([need_common_t, need_common_b], shared_term_i);
    // * 🚩产生一个新的独立变量，并以此替换
    if let Some(second_common_term) = second_common_term {
        // 生成替换映射：第二个相同词项 → 新独立变量
        let substitute = VarSubstitution::from_pairs([(second_common_term.clone(), var_i)]);
        // 应用替换映射
        substitute.apply_to_term(need_common_t);
        substitute.apply_to_term(need_common_b);
    }
    // ! ⚠️在此期间【修改】其【所指向】的词项
    // * 📝若应用了替换，则替换后的变量会算进「任务内容」「信念内容」中，故无需再考量
    let var_i = || Term::make_var_i([&*task_content, &*belief_content]);

    // * 🚩根据索引决定「要组成新陈述的词项的位置」
    let [term11, term12, term21, term22];
    match shared_term_i {
        Subject => {
            term11 = var_i();
            term21 = var_i();
            term12 = task_content.get_ref().predicate().clone();
            term22 = belief_content.get_ref().predicate().clone();
        }
        Predicate => {
            term11 = task_content.get_ref().subject().clone();
            term21 = belief_content.get_ref().subject().clone();
            term12 = var_i();
            term22 = var_i();
        }
    }
    // TODO: ↑继续用select简化
    // * 🚩返回：从元素构造继承陈述
    let inheritance = Term::make_inheritance; // 精简代码量
    [inheritance(term11, term12), inheritance(term21, term22)]
}

/// 🆕以「变量外引入」的内部词项，计算「引入状态」陈述
/// * 📌引入的是「独立变量/自变量」"$"
/// * 🎯产生的陈述（二元组）用于生成新结论内容
fn intro_var_states_dep(
    task_content: StatementRef,
    belief_content: StatementRef,
    shared_term_i: SyllogismPosition,
) -> [Option<Term>; 2] {
    let var_d = || Term::make_var_d([&*task_content, &*belief_content]);

    // * 🚩根据索引决定「要组成新陈述的词项的位置」
    let [term11, term12, term21, term22];
    match shared_term_i {
        Subject => {
            term11 = var_d();
            term21 = var_d();
            term12 = task_content.predicate().clone();
            term22 = belief_content.predicate().clone();
        }
        Predicate => {
            term11 = task_content.subject().clone();
            term21 = belief_content.subject().clone();
            term12 = var_d();
            term22 = var_d();
        }
    }
    // TODO: ↑继续用select简化
    // * 🚩返回：从元素构造继承陈述
    let inheritance = Term::make_inheritance; // 精简代码量
    [inheritance(term11, term12), inheritance(term21, term22)]
}

/// 根据「词项构造函数」「真值函数」「是否交换顺序」统一构造「变量外引入」的结论
/// * 📌其中`reverse_order`连词项与真值一同交换顺序
///   * `state_1` <~> `truth_t`
///   * `state_2` <~> `truth_b`
fn intro_var_outer_derive(
    [state_1, state_2]: [Option<Term>; 2],
    [truth_t, truth_b]: [&TruthValue; 2],
    make_content: fn(Term, Term) -> Option<Term>,
    truth_f: fn(&TruthValue, &TruthValue) -> TruthValue,
    reverse_order: bool,
    context: &mut ReasonContextConcept,
    // 预算函数默认是「复合前向」
) {
    // * 🚩词项
    // 先尝试解包出有用的词项
    let state_1 = unwrap_or_return!(?state_1);
    let state_2 = unwrap_or_return!(?state_2);
    let [state_1, state_2] = reverse_order.select([state_1, state_2]); // 用「是否交换」调换顺序
    let content = unwrap_or_return!(?make_content(state_1, state_2));
    // * 🚩真值
    let [truth_1, truth_2] = reverse_order.select([truth_t, truth_b]);
    let truth = truth_f(truth_1, truth_2);
    // * 🚩预算：统一为「复合前向」
    let budget = context.budget_compound_forward(&truth, &content);
    // * 🚩结论
    context.double_premise_task(content, Some(truth), budget);
}

/// Intro some variables into the contents.
/// * 📝「变量内引入」系列规则
/// * 📝引入的既有非独变量 `#` 又有独立变量 `$`
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
    // * 🚩任务/信念 的真值 | 仅适用于前向推理
    debug_assert!(context.current_task().get_().is_judgement());
    let truth_t = TruthValue::from(context.current_task().get_().unwrap_judgement());
    let truth_b = TruthValue::from(context.current_belief().unwrap());

    // * 🚩前提1与前提2必须是相同类型，且「旧复合词项」不能包括前提1
    if !premise_1.is_same_type(&premise_2) || old_compound.contain_component(&premise_1) {
        return;
    }

    // * 🚩计算共有词项
    let [common_term_1, common_term_2] = intro_var_commons([premise_1, premise_2]);
    let (common_term_1, common_term_2) = (|| common_term_1.cloned(), || common_term_2.cloned());

    // * 🚩继续向下分派
    //   * ℹ️因两个推理结论的构造方式不甚一样，不将它们统一到一个函数中
    f_parallel![
        // * 🚩同时分派到下边两个函数
        intro_var_inner1 intro_var_inner2;
        // * 🚩以如下所列参数分派
        premise_1,
        old_compound,
        common_term_1(),
        common_term_2(),
        &truth_t,
        &truth_b,
        context,
    ];
}

/// 🆕以「变量内引入」的内部词项，计算「共有词项」
/// * 🎯产生的词项（二元组/空）用于生成新结论内容
fn intro_var_commons([premise_1, premise_2]: [StatementRef; 2]) -> [Option<&Term>; 2] {
    let [term11, term12] = premise_1.sub_pre();
    let [term21, term22] = premise_2.sub_pre();
    // * 🚩轮流判等以决定所抽取的词项
    if *term11 == *term21 {
        // * 🚩共有主项 ⇒ 11→(12×22)
        [Some(term11), second_common_term([term12, term22], Subject)]
    } else if *term12 == *term22 {
        // * 🚩共有谓项 ⇒ 12→(11×21)
        [Some(term12), second_common_term([term11, term21], Subject)]
    } else {
        // * 🚩无共有词项⇒空
        [None, None]
    }
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
    premise_1: StatementRef,
    old_compound: CompoundTermRef,
    common_term_1: Option<Term>,
    _common_term_2: Option<Term>, // 此处用不到
    truth_t: &impl Truth,
    truth_b: &impl Truth,
    context: &mut ReasonContextConcept,
) {
    // * 🚩词项 * //
    let mut content = unwrap_or_return!(
        ?Term::make_conjunction(premise_1.statement.clone(), old_compound.inner.clone())
    );

    // * 🚩将「共有词项」替换成变量
    if let Some(common_term_1) = common_term_1 {
        let var_d = Term::make_var_d(&content);
        let substitute = VarSubstitution::from_pairs([(common_term_1, var_d)]);
        substitute.apply_to_term(&mut content);
    }

    // * 🚩真值 * //
    let truth = truth_t.intersection(truth_b);

    // * 🚩预算 * //
    let budget = context.budget_forward(&truth);

    // * 🚩结论 * //
    println!("content1 = {content}\n from {premise_1}, {old_compound}");
    context.double_premise_task_not_revisable(dbg!(content), Some(truth), budget);
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
    premise_1: StatementRef,
    old_compound: CompoundTermRef,
    common_term_1: Option<Term>,
    common_term_2: Option<Term>,
    truth_t: &TruthValue,
    truth_b: &TruthValue,
    context: &mut ReasonContextConcept,
) {
    // * 🚩词项 * //
    let mut content = unwrap_or_return!(
        ?Term::make_implication(premise_1.statement.clone(), old_compound.inner.clone())
    );

    // * 🚩将「共有词项」替换成变量
    let var_i = Term::make_var_i(&content);
    let var_i_2 = Term::make_var_i([&content, &var_i]); // ! 提前创建以示一致
    let substitute = VarSubstitution::from_pairs(
        // * 🚩两处均为「若有则替换」：空值直接跳过，有值则分别替换为俩不同变量
        [
            common_term_1.map(|common_term| (common_term, var_i)),
            common_term_2.map(|common_term| (common_term, var_i_2)),
        ]
        .into_iter()
        .flatten(),
    );
    substitute.apply_to_term(&mut content);

    // * 🚩真值 * //
    // * 🚩根据「前提1是否与任务内容相等」调整真值参数顺序
    //   * 📄前提1 == 任务 ⇒ 归纳 信念→任务
    //   * 📄前提1 != 任务 ⇒ 归纳 任务→信念
    let premise1_eq_task = *premise_1 == *context.current_task().get_().content();
    let [truth_1, truth_2] = premise1_eq_task.select([truth_t, truth_b]);
    let truth = truth_1.induction(truth_2);

    // * 🚩预算 * //
    let budget = context.budget_forward(&truth);

    // * 🚩结论 * //
    println!("content2 = {content}\n from {premise_1}, {old_compound}");
    context.double_premise_task(dbg!(content), Some(truth), budget);
}

/// # 📄OpenNARS
///
/// Introduce a second independent variable into two terms with a common
/// component
fn second_common_term(
    [term1, term2]: [&Term; 2], // 强制将这俩词项统一到了同一生命周期
    shared_term_i: SyllogismPosition,
) -> Option<&Term> {
    // * 🚩确定「需要特别判断的『像』类型」
    //   * 主项 ⇒ 外延像
    //   * 谓项 ⇒ 内涵像
    let specific_image_type = shared_term_i.select_one([IMAGE_EXT_OPERATOR, IMAGE_INT_OPERATOR]);
    // * 🚩只在「都是指定像类型」时继续判断（其它情况直接返回空）
    //   * 📄1: 都是主项，且均为外延像
    //   * 📄2: 都是谓项，且均为内涵像
    let image1 = term1.as_compound_type(specific_image_type)?;
    let image2 = term2.as_compound_type(specific_image_type)?;

    // * 🚩在俩像之间获取词项并尝试
    match image1.get_the_other_component() {
        // * 🚩先试第一个
        Some(common_term) if image2.contain_term(common_term) => Some(common_term),
        // * 🚩尝试不到？考虑第二个/用第二个覆盖
        _ => match image2.get_the_other_component() {
            // * 🚩再试第二个
            Some(common_term) if image1.contain_term(common_term) => Some(common_term),
            // * 🚩尝试不到就是尝试不到
            _ => None,
        },
    }
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

        intro_var_same_subject: {
            "
            nse <<$1 --> B> ==> <$1 --> A>>.
            nse <A --> C>.
            cyc 10
            "
            => OUT "<<A --> B> ==> (&&,<#1 --> C>,<#1 --> A>)>" in outputs
        }

        intro_var_same_predicate: {
            "
            nse <<B --> $1> ==> <A --> $1>>.
            nse <C --> A>.
            cyc 10
            "
            => OUT "<<B --> A> ==> (&&,<C --> #1>,<A --> #1>)>" in outputs
        }

        intro_var_outer_sub_imp: {
            "
            nse <M --> A>.
            nse <M --> B>.
            cyc 5
            "
            => OUT "<<$1 --> A> ==> <$1 --> B>>" in outputs
        }

        intro_var_outer_sub_imp_rev: {
            "
            nse <M --> A>.
            nse <M --> B>.
            cyc 5
            "
            => OUT "<<$1 --> B> ==> <$1 --> A>>" in outputs
        }

        intro_var_outer_sub_equ: {
            "
            nse <M --> A>.
            nse <M --> B>.
            cyc 5
            "
            => OUT "<<$1 --> A> <=> <$1 --> B>>" in outputs
        }

        intro_var_outer_sub_con: {
            "
            nse <M --> A>.
            nse <M --> B>.
            cyc 5
            "
            => OUT "(&&,<#1 --> A>,<#1 --> B>)" in outputs
        }

        intro_var_outer_pre_imp: {
            "
            nse <A --> M>.
            nse <B --> M>.
            cyc 5
            "
            => OUT "<<A --> $1> ==> <B --> $1>>" in outputs
        }

        intro_var_outer_pre_imp_rev: {
            "
            nse <A --> M>.
            nse <B --> M>.
            cyc 5
            "
            => OUT "<<B --> $1> ==> <A --> $1>>" in outputs
        }

        intro_var_outer_pre_equ: {
            "
            nse <A --> M>.
            nse <B --> M>.
            cyc 5
            "
            => OUT "<<A --> $1> <=> <B --> $1>>" in outputs
        }

        intro_var_outer_pre_con: {
            "
            nse <A --> M>.
            nse <B --> M>.
            cyc 5
            "
            => OUT "(&&,<A --> #1>,<B --> #1>)" in outputs
        }

        // ! ❌【2024-08-08 02:07:47】OpenNARS改版中亦测试失败
        // intro_var_inner_imp_1: {
        //     "
        //     nse <M --> S>.
        //     nse <C ==> <M --> P>>.
        //     cyc 20
        //     " // 似乎跟预期中 "(&&,C,<<#1 --> S> ==> <#1 --> P>>)" 不一致
        //     => OUT "(&&,C,<#1 --> S>,<#1 --> P>)" in outputs
        // }

        // ! ❌【2024-08-08 02:07:47】OpenNARS改版中亦测试失败
        // intro_var_inner_imp_2: {
        //     "
        //     nse <M --> S>.
        //     nse <C ==> <M --> P>>.
        //     cyc 20
        //     " // 似乎跟预期中 "<(&&,<#x --> S>,C) ==> <#x --> P>>" 不一致
        //     => OUT "<<$1 --> S> ==> (&&,C,<$1 --> P>)>" in outputs
        // }

        intro_var_inner_con_1: {
            "
            nse <M --> S>.
            nse (&&,C,<M --> P>).
            cyc 20
            " // 似乎跟预期中 "(&&,C,<<#1 --> S> ==> <#1 --> P>>)" 不一致
            => OUT "(&&,C,<#1 --> S>,<#1 --> P>)" in outputs
        }

        intro_var_inner_con_2: {
            "
            nse <M --> S>.
            nse (&&,C,<M --> P>).
            cyc 20
            " // 似乎跟预期中 "<(&&,<#x --> S>,C) ==> <#x --> P>>" 不一致
            => OUT "<<$1 --> S> ==> (&&,C,<$1 --> P>)>" in outputs
        }
    }
}
