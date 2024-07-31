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
    use super::{StatementRef, Term};

    pub trait Opposite {
        /// 调转到「相反方向」「相反位置」
        /// * 🎯抽象自各个「三段论位置」
        /// * 🎯为「三段论图式」添加方法
        fn opposite(self) -> Self;

        /// 返回自身与「自身的相反位置」
        fn and_opposite(self) -> [Self; 2]
        where
            Self: Clone,
        {
            [self.clone(), self.opposite()]
        }
    }

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

    impl Opposite for SyllogismPosition {
        /// 🆕调转到相反位置
        fn opposite(self) -> Self {
            match self {
                Subject => Predicate,
                Predicate => Subject,
            }
        }
    }

    impl SyllogismPosition {
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

        /// 根据「三段论位置」从参数中选取一个参数
        /// * 🎯在「陈述解包」的过程中使用
        pub fn select<T>(self, [subject, predicate]: [T; 2]) -> T {
            match self {
                Subject => subject,
                Predicate => predicate,
            }
        }
    }
    use SyllogismPosition::*;

    /// 以此扩展到「陈述」的功能
    impl StatementRef<'_> {
        /// 根据「三段论位置」扩展获取「三段论位置」对应的「词项」
        pub fn get_at_position(&self, position: SyllogismPosition) -> &Term {
            match position {
                Subject => self.subject(),
                Predicate => self.predicate(),
            }
        }
    }

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

    impl Opposite for SyllogismFigure {
        /// 🆕调转到相反位置：内部俩均如此
        fn opposite(self) -> Self {
            let [subject, predicate] = self;
            [subject.opposite(), predicate.opposite()]
        }
    }

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

    impl Opposite for SyllogismSide {
        /// 🆕调转到相反位置
        fn opposite(self) -> Self {
            use SyllogismSide::*;
            match self {
                Subject => Predicate,
                Predicate => Subject,
                Whole => Whole, // * 📌整体反过来还是整体
            }
        }
    }
}
pub use utils::*;

/// 规则分派
mod dispatch {
    use super::*;
    use syllogistic_figures::*;
    use variable_process::{unify_find_i, unify_find_q};

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

        // * 🚩尝试获取各大「共同项」与「其它项」的位置
        // * 📝外部传入的「三段论图式」即「共同项的位置」，「其它项」即各处「共同项」的反向
        let [[common_position_t, common_position_b], [other_position_t, other_position_b]] =
            figure.and_opposite();
        // * 🚩先尝试统一独立变量
        let unified_i = variable_process::unify_find_i(
            t_term.get_ref().get_at_position(common_position_t),
            b_term.get_ref().get_at_position(common_position_b),
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
        let term_t = other_position_t.select(t_term.clone().unwrap_components());
        let term_b = other_position_b.select(b_term.clone().unwrap_components());
        let [sub, pre] = match figure {
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
                // TODO
                // * 🚩归因+归纳+比较
                abd_ind_com(sub, pre, task_sentence, belief_sentence, context);
            }
            // * 🚩谓项×谓项 <A --> B> × <C --> B>
            // abduction
            PP => {
                // * 🚩先尝试进行「条件归纳」，有结果⇒返回
                let applied = conditional_abd(sub.clone(), pre.clone(), t_term, b_term, context);
                if applied {
                    // if conditional abduction, skip the following
                    return;
                }
                // * 🚩尝试构建复合词项
                // TODO
                // * 🚩归因+归纳+比较
                abd_ind_com(sub, pre, task_sentence, belief_sentence, context);
            }
            // * 🚩主项×谓项 <A --> B> × <C --> A>
            // * 🚩谓项×主项 <A --> B> × <B --> C>
            // * 📝【2024-07-31 19:52:56】sub、pre已经在先前「三段论图式选取」过程中确定，此两种形式均一致
            // deduction | exemplification
            SP | PS => {
                // * 🚩尝试统一查询变量
                // * ⚠️【2024-07-14 03:13:32】不同@OpenNARS：无需再应用到整个词项——后续已经不再需要t_term与b_term
                // * ⚠️【2024-07-31 21:37:10】激进改良：无需应用变量替换，只需考虑「是否可替换」
                let unified_q = variable_process::has_unification_q(&sub, &pre, rng_seed2);
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
    ///
    /// @param context Reference to the derivation context
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
        let rng_seed = context.shuffle_rng_seed();
        let rng_seed2 = context.shuffle_rng_seed();

        // * 🚩尝试获取各大「共同项」与「其它项」的位置
        // * 📝外部传入的「三段论图式」即「共同项的位置」，「其它项」即各处「共同项」的反向
        let [[common_pos_asy, common_pos_sym], [other_pos_asy, other_pos_sym]] =
            figure.and_opposite();
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
            asy_s.get_ref().get_at_position(common_pos_asy),
            sym_s.get_ref().get_at_position(common_pos_sym),
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
            asy_s.get_ref().get_at_position(other_pos_asy),
            sym_s.get_ref().get_at_position(other_pos_sym),
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
                asy_s.get_ref().get_at_position(other_pos_asy).clone(),
                sym_s.get_ref().get_at_position(other_pos_sym).clone(),
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
    ///
    /// @param asym    A Inheritance/Implication sentence
    /// @param sym     A Similarity/Equivalence sentence
    /// @param figure  location of the shared term
    /// @param context Reference to the derivation context
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
        // TODO
    }

    /// 分离（可带变量）
    fn detachment_with_var(
        high_order_sentence: impl Sentence,
        sub_sentence: impl Sentence,
        index: usize,
        context: &mut ReasonContextConcept,
    ) {
        // TODO
    }

    /// ```nal
    /// {<S ==> M>, <M ==> P>} |- {<S ==> P>, <P ==> S>}
    /// ```
    ///
    /// 演绎&举例
    /// * 📝一个强推理，一个弱推理
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

/// {<S ==> P>, <M <=> P>} |- <S ==> P>
/// * 📌类比
/// * 📝【2024-07-02 13:27:22】弱推理🆚强推理、前向推理🆚反向推理 不是一个事儿
fn analogy(
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
/// {<(&&, S2, S3) ==> P>, <(&&, S1, S3) ==> P>} |- <S1 ==> S2>
///
/// @param cond1   The condition of the first premise
/// @param cond2   The condition of the second premise
/// @param st1     The first premise
/// @param st2     The second premise
/// @param context Reference to the derivation context
/// @return Whether there are derived tasks
fn conditional_abd(
    sub: Term,
    pre: Term,
    t_term: Statement,
    b_term: Statement,
    context: &mut ReasonContextConcept,
) -> bool {
    // TODO: 🚩待实现
    false
}

/// {<S --> P>, <P --> S} |- <S <-> p>
/// Produce Similarity/Equivalence from a pair of reversed
/// Inheritance/Implication
/// * 📝非对称⇒对称（前向推理）
fn infer_to_sym(
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
///
/// @param asym    The asymmetric premise
/// @param sym     The symmetric premise
/// @param context Reference to the derivation context
fn infer_to_asy(asy: &impl Judgement, sym: &impl Judgement, context: &mut ReasonContextConcept) {
    // * 🚩词项 * //
    // * 🚩提取 | 📄<S --> P> => S, P
    // * 🚩构建新的相反陈述 | 📄S, P => <P --> S>
    let [sub, pre] = cast_statement(asy.content().clone()).unwrap_components();
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
///
/// @param context Reference to the derivation context
fn conversion(belief: &impl Judgement, context: &mut ReasonContextConcept) {
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
///
/// @param context Reference to the derivation context
fn convert_relation(task_question: &impl Question, context: &mut ReasonContextConcept) {
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
fn converted_judgment(
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
    }
}
