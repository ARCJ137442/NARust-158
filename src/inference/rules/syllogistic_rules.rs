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
        /// * 🎯在「陈述选择」的过程中使用，同时需要前后两项
        /// * 🚩数组的第一项即为「选中项」
        pub fn select_and_other<T>(self, [subject, predicate]: [T; 2]) -> [T; 2] {
            match self {
                Subject => [subject, predicate],
                Predicate => [predicate, subject],
            }
        }

        /// 根据「三段论位置」从参数中选取一个参数
        /// * 🎯在「陈述解包」的过程中使用
        pub fn select<T>(self, sub_pre: [T; 2]) -> T {
            let [selected, _] = self.select_and_other(sub_pre);
            selected
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

    /// 分离规则中「高阶语句」的位置
    /// * 📄任务句
    /// * 📄信念句
    #[derive(Debug, Clone, Copy)]
    pub enum HighOrderPosition {
        Task,
        Belief,
    }
}
pub use utils::*;

/// 规则分派
mod dispatch {
    use super::*;
    use syllogistic_figures::*;
    use variable_process::{has_unification_q, unify_find_i, unify_find_q};

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
            [INHERITANCE_RELATION, IMPLICATION_RELATION | EQUIVALENCE_RELATION] => {
                detachment_with_var(
                    task_sentence, // ! 📌【2024-08-01 18:26:04】需要传递所有权：直接统一语句中的变量
                    belief, // ! 📌【2024-08-01 18:26:04】需要传递所有权：直接统一语句中的变量
                    HighOrderPosition::Belief,
                    SyllogismPosition::from_index(b_index),
                    context,
                )
            }
            // * 🚩分离：蕴含 + | 蕴含/等价 × 继承
            [IMPLICATION_RELATION | EQUIVALENCE_RELATION, INHERITANCE_RELATION] => {
                detachment_with_var(
                    task_sentence, // ! 📌【2024-08-01 18:26:04】需要传递所有权：直接统一语句中的变量
                    belief, // ! 📌【2024-08-01 18:26:04】需要传递所有权：直接统一语句中的变量
                    HighOrderPosition::Task,
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
        let rng_seed = context.shuffle_rng_seed();
        let rng_seed2 = context.shuffle_rng_seed();

        // * 🚩尝试获取各大「共同项」与「其它项」的位置
        // * 📝外部传入的「三段论图式」即「共同项的位置」，「其它项」即各处「共同项」的反向
        let [[common_pos_t, common_pos_b], [other_pos_t, other_pos_b]] = figure.and_opposite();
        // * 🚩先尝试统一独立变量
        let unified_i = unify_find_i(
            t_term.get_ref().get_at_position(common_pos_t),
            b_term.get_ref().get_at_position(common_pos_b),
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
        let term_t = other_pos_t.select(t_term.clone().unwrap_components());
        let term_b = other_pos_b.select(b_term.clone().unwrap_components());
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
                let unified_q = has_unification_q(&sub, &pre, rng_seed2);
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
            pos_t.select(t_term.sub_pre()),
            pos_b.select(b_term.sub_pre()),
        ];
        let rng_seed = context.shuffle_rng_seed();
        // * 🚩尝试以不同方式统一独立变量 @ 公共词项
        let unified = unify_find_i(common_b, common_t, rng_seed).apply_to(
            t_term.mut_ref().into_compound_ref(),
            b_term.mut_ref().into_compound_ref(),
        );
        // * 🚩成功统一 ⇒ 相似传递
        if unified {
            let [other_t, other_b] = [
                pos_t.opposite().select(t_term.unwrap_components()),
                pos_b.opposite().select(b_term.unwrap_components()),
            ];
            resemblance(other_b, other_t, &belief_sentence, &task_sentence, context);
        }
    }

    /// 分离（可带变量）
    fn detachment_with_var(
        mut task_sentence: impl Sentence,
        mut belief: impl Judgement,
        high_order_position: HighOrderPosition,
        position_sub_in_hi: SyllogismPosition,
        context: &mut ReasonContextConcept,
    ) {
        // * 🚩提取元素
        let [term_t, term_b] = [task_sentence.content(), belief.content()];
        let (main_statement, sub_content) = match high_order_position {
            HighOrderPosition::Task => (term_t.as_statement().unwrap(), term_b),
            HighOrderPosition::Belief => (term_b.as_statement().unwrap(), term_t),
        };
        let component = position_sub_in_hi.select(main_statement.sub_pre()); // * 🚩前件

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
            variable_process::unify_find_i(component, sub_content, context.shuffle_rng_seed());
        let [term_mut_t, term_mut_b] = [task_sentence.content_mut(), belief.content_mut()]; // 获取可变引用并统一
        let [main_content_mut, sub_content_mut] = match high_order_position {
            HighOrderPosition::Task => [term_mut_t, term_mut_b],
            HighOrderPosition::Belief => [term_mut_b, term_mut_t],
        };
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
        let (main_statement, sub_content) = match high_order_position {
            HighOrderPosition::Task => (term_t.as_statement().unwrap(), term_b),
            HighOrderPosition::Belief => (term_b.as_statement().unwrap(), term_t),
        };
        // ! ⚠️【2024-06-10 17:52:44】「当前任务」与「主陈述」可能不一致：主陈述可能源自「当前信念」
        // * * 当前任务="<(*,{tom},(&,glasses,[black])) --> own>."
        // * * 主陈述="<<$1 --> (/,livingIn,_,{graz})> ==> <(*,$1,sunglasses) --> own>>"
        // * * 当前信念="<<$1 --> (/,livingIn,_,{graz})> ==> <(*,$1,sunglasses) --> own>>."
        // * 🚩当前为正向推理（任务、信念皆判断），且主句的后项是「陈述」⇒尝试引入变量
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
                    // TODO: 变量内引入
                }
                // TODO: 变量引入 同主项/谓项
            }
            if main_statement.instanceof_equivalence() {
                // TODO: 变量引入 同主项/谓项
            }
        }
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
/// `{<(&&, S2, S3) ==> P>, <(&&, S1, S3) ==> P>} |- <S1 ==> S2>`
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
fn infer_to_asy(asy: &impl Judgement, sym: &impl Judgement, context: &mut ReasonContextConcept) {
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

/// 相似传递
///
/// # 📄OpenNARS
///
/// `{<S <=> M>, <M <=> P>} |- <S <=> P>`
fn resemblance(
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
    high_order_position: HighOrderPosition,
    position_sub_in_hi: SyllogismPosition,
    context: &mut ReasonContextConcept,
) {
    // * 🚩合法性
    let high_order_statement = match high_order_position {
        HighOrderPosition::Task => task_sentence.content(),
        HighOrderPosition::Belief => belief.content(),
    };
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
    let sub_content = match high_order_position {
        HighOrderPosition::Task => belief.content(),
        HighOrderPosition::Belief => task_sentence.content(),
    };
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
            let [main_sentence_truth, sub_sentence_truth] = match high_order_position {
                HighOrderPosition::Task => [
                    TruthValue::from(task_sentence.unwrap_judgement()),
                    TruthValue::from(belief),
                ],
                HighOrderPosition::Belief => [
                    TruthValue::from(belief),
                    TruthValue::from(task_sentence.unwrap_judgement()),
                ],
            };
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
    }
}
