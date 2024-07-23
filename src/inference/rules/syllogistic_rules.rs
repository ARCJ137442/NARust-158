//! 三段论规则
//! * 🚩【2024-07-11 00:07:34】目前只包含「具体规则处理」两部分
//!   * 📝OpenNARS中「规则表」可能会在某些地方直接分派规则
//!   * 📄条件三段论系列
//!
//! ## Logs
//!
//! * ♻️【2024-07-11 00:07:52】开始根据改版OpenNARS重写

use crate::{
    control::*, entity::*, inference::rules::cast_statement, inference::*, io::symbols::*,
    language::*, util::*,
};
use nar_dev_utils::unwrap_or_return;
use ReasonDirection::*;

/// 存储规则表之外的结构与方法
mod utils {
    /// 🆕三段论位置
    /// * 🎯用于表征[`RuleTables::index_to_figure`]推导出的「三段论子类型」
    /// * 📝OpenNARS中是在「三段论推理」的「陈述🆚陈述」中表示「位置关系」
    ///   * 📄`<A --> B>`与`<B --> C>`中，`B`就分别在`1`、`0`两个索引位置
    ///     * 📌因此有`SP`或`Subject-Predicate`
    ///     * 📌同时也有了其它三种「三段论图式」
    /// * 🚩两种情况：
    ///   * 主项
    ///   * 谓项
    #[doc(alias = "SyllogismLocation")]
    #[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
    pub enum SyllogismPosition {
        /// 主项（第一项）
        Subject = 0,
        /// 谓项（第二项）
        Predicate = 1,
    }

    impl SyllogismPosition {
        /// 🆕调转到相反位置
        pub fn opposite(self) -> Self {
            match self {
                Subject => Predicate,
                Predicate => Subject,
            }
        }

        /// 🆕从「数组索引」中来
        /// * 🎯[`RuleTables::__index_to_figure`]
        /// * 🚩核心：0→主项，1→谓项，整体`<主项 --> 谓项>`
        pub fn from_index(index: usize) -> Self {
            match index {
                0 => Subject,
                1 => Predicate,
                _ => panic!("无效索引"),
            }
        }

        /// 🆕构造「三段论图式」
        /// * 🎯[`RuleTables::__index_to_figure`]
        /// * 🚩直接构造二元组
        pub fn build_figure(self, other: Self) -> SyllogismFigure {
            [self, other]
        }
    }
    use SyllogismPosition::*;

    /// 三段论图式
    /// * 🎯模拟「三段论推理」中「公共项在两陈述的位置」的四种情况
    /// * 📝左边任务（待处理），右边信念（已接纳）
    /// * 🚩公共词项在两个陈述之中的顺序
    /// * 🚩使用二元组实现，允许更细化的组合
    ///   * ✨基本等同于整数（低开销）类型
    /// * 🚩【2024-07-12 21:17:33】现在改为二元数组
    ///   * 💭相同的效果，更简的表达
    ///   * 📌相同类型的序列，宜用数组表达
    /// * 📝四种主要情况：
    ///   * 主项-主项
    ///   * 主项-谓项
    ///   * 谓项-主项
    ///   * 谓项-谓项
    ///
    /// # 📄OpenNARS
    ///
    /// location of the shared term
    pub type SyllogismFigure = [SyllogismPosition; 2];

    /// 存储「三段论图式」常量
    /// * 🎯可完全引用，可简短使用
    ///   * ⚡长度与OpenNARS的`11`、`12`相近
    /// * 🚩仅四种
    pub mod syllogistic_figures {
        use super::*;

        /// [三段论图式](SyllogismFigure)/常用/主项-主项
        #[doc(alias = "SUBJECT_SUBJECT")]
        pub const SS: SyllogismFigure = [Subject, Subject];

        /// [三段论图式](SyllogismFigure)/常用/主项-谓项
        #[doc(alias = "SUBJECT_PREDICATE")]
        pub const SP: SyllogismFigure = [Subject, Predicate];

        /// [三段论图式](SyllogismFigure)/常用/谓项-主项
        #[doc(alias = "PREDICATE_SUBJECT")]
        pub const PS: SyllogismFigure = [Predicate, Subject];

        /// [三段论图式](SyllogismFigure)/常用/谓项-谓项
        #[doc(alias = "PREDICATE_PREDICATE")]
        pub const PP: SyllogismFigure = [Predicate, Predicate];
    }

    /// 三段论推理中的「某侧」
    /// * 📌包含「主项/谓项/整个词项」
    #[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
    pub enum SyllogismSide {
        /// 主项（第一项）
        Subject = 0,
        /// 谓项（第二项）
        Predicate = 1,
        /// 整个词项（整体）
        Whole = -1,
    }
}
pub use utils::*;

/// 规则分派
mod dispatch {
    use super::*;

    /// 索引⇒图式
    fn index_to_figure<T, U>(link1: &impl TLink<T>, link2: &impl TLink<U>) -> SyllogismFigure {
        let side1 = SyllogismPosition::from_index(*link1.get_index(0).unwrap());
        let side2 = SyllogismPosition::from_index(*link2.get_index(0).unwrap());
        [side1, side2]
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
            [INHERITANCE_RELATION, IMPLICATION_RELATION]
            | [INHERITANCE_RELATION, EQUIVALENCE_RELATION] => {
                detachment_with_var(belief, task_sentence, b_index, context)
            }
            // * 🚩分离：蕴含 + | 蕴含/等价 × 继承
            [IMPLICATION_RELATION | EQUIVALENCE_RELATION, INHERITANCE_RELATION] => {
                detachment_with_var(task_sentence, belief, t_index, context)
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
        let rng_seed = context.shuffle_rng_seed();
        let rng_seed2 = context.shuffle_rng_seed();
        use syllogistic_figures::*;
        match figure {
            // * 🚩主项×主项 <A --> B> × <A --> C>
            // induction
            SS => {
                // * 🚩先尝试统一独立变量
                let unified_i = variable_process::unify_find_i(
                    t_term.get_ref().subject(),
                    b_term.get_ref().subject(),
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
                // * 🚩取其中两个不同的谓项 B + C
                let ([_, term2], [_, term1]) =
                    (t_term.unwrap_components(), b_term.unwrap_components());
                // * 🚩构造复合词项
                // TODO
                // * 🚩归因+归纳+比较
                abd_ind_com(term1, term2, task_sentence, belief_sentence, context);
            }
            // * 🚩主项×谓项 <A --> B> × <C --> A>
            // deduction
            SP => {
                // * 🚩先尝试统一独立变量
                let unified_i = variable_process::unify_find_i(
                    t_term.get_ref().subject(),
                    b_term.get_ref().predicate(),
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
                // * 🚩取其中两个不同的主项和谓项 C + B
                let ([_, mut term2], [mut term1, _]) =
                    (t_term.unwrap_components(), b_term.unwrap_components());
                // * 🚩尝试统一查询变量
                // * ⚠️【2024-07-14 03:13:32】不同@OpenNARS：无需再应用到整个词项——后续已经不再需要t_term与b_term
                let unified_q = variable_process::unify_find_q(&term1, &term2, rng_seed2)
                    .apply_to_term(&mut term1, &mut term2);
                if unified_q {
                    // * 🚩成功统一 ⇒ 匹配反向
                    // TODO
                }
                // * 🚩未有统一 ⇒ 演绎+举例
                else {
                    ded_exe(term1, term2, task_sentence, belief_sentence, context);
                }
            }
            // * 🚩谓项×主项 <A --> B> × <B --> C>
            // exemplification
            PS => {
                // * 🚩先尝试统一独立变量
                // * 📝统一之后，原先的变量就丢弃了
                // * 🚩不能统一变量⇒终止
                // * 🚩统一后内容相等⇒终止
                // * 🚩取其中两个不同的主项和谓项 A + C
                // * 🚩尝试统一查询变量
                // * 🚩成功统一 ⇒ 匹配反向
                // * 🚩未有统一 ⇒ 演绎+举例
            }
            // * 🚩谓项×谓项 <A --> B> × <C --> B>
            // abduction
            PP => {
                // * 🚩先尝试统一独立变量
                // * 🚩不能统一变量⇒终止
                // * 🚩统一后内容相等⇒终止
                // * 🚩取其中两个不同的主项和谓项 A + C
                // * 🚩先尝试进行「条件归纳」，有结果⇒返回
                // if conditional abduction, skip the following
                // * 🚩尝试构建复合词项
                // * 🚩归因+归纳+比较
            }
        }
    }

    /// 非对称×对称
    fn asymmetric_symmetric(
        asymmetric: impl Sentence,
        symmetric: impl Sentence,
        figure: SyllogismFigure,
        context: &mut ReasonContextConcept,
    ) {
        // TODO
    }

    /// 对称×对称
    fn symmetric_symmetric(
        task_sentence: impl Sentence,
        belief_sentence: impl Judgement,
        figure: SyllogismFigure,
        context: &mut ReasonContextConcept,
    ) {
        // TODO
    }

    /// 分离（可带变量）
    fn detachment_with_var(
        high_order_sentence: impl Sentence,
        sub_sentence: impl Sentence,
        index: usize,
        context: &mut ReasonContextConcept,
    ) {
    }

    /// ```nal
    /// {<S ==> M>, <M ==> P>} |- {<S ==> P>, <P ==> S>}
    /// ```
    ///
    /// 演绎&举例
    /// * 📝一个强推理，一个弱推理
    ///
    fn ded_exe(
        sub: Term,
        pre: Term,
        task_sentence: impl Sentence,
        belief_sentence: impl Judgement,
        context: &mut ReasonContextConcept,
    ) {
        // * 🚩陈述有效才行
        if StatementRef::invalid_statement(&sub, &pre) {
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
}
pub use dispatch::*;

/// 🆕演绎规则
fn deduction(
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
fn exemplification(
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
fn abduction(
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
fn induction(
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
fn comparison(
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

    #[test]
    fn deduction() {
        let mut vm = create_vm_from_engine(ENGINE_REASON);
        // * 🚩OUT
        vm.input_fetch_print_expect(
            "
            nse <A --> B>.
            nse <B --> C>.
            cyc 10
            ",
            // * 🚩检查其中是否有导出
            expect_narsese_term!(OUT "<A --> C>" in outputs),
        );
    }

    #[test]
    fn deduction_answer() {
        let mut vm = create_vm_from_engine(ENGINE_REASON);
        // * 🚩ANSWER
        vm.input_fetch_print_expect(
            "
            nse <A --> B>.
            nse <B --> C>.
            nse <A --> C>?
            cyc 20
            ",
            // * 🚩检查其中是否有导出
            expect_narsese_term!(ANSWER "<A --> C>" in outputs),
        );
    }
}
