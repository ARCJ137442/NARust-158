//! 📄OpenNARS `nars.language.MakeTerm`
//! * 🎯复刻原OpenNARS 1.5.8的`make`系列方法
//! * 🚩构造词项前，
//!   * 检查其合法性
//!   * 简化其表达
//! * 🎯用于「制作词项」
//!   * 📝是NARS中「词项逻辑」的重要部分——非推理语义简化
//! * 🚩【2024-09-07 16:09:46】从外部IO解析出一个新词项的流程：词法折叠→语义简化→创建结构体

use super::{Term, TermComponents};
use crate::{
    language::{variable::MaximumVariableId, vec_utils, CompoundTermRef, StatementRef},
    symbols::*,
};

impl Term {
    /* Word */

    /// 制作「词语」
    #[inline]
    pub fn make_word(name: impl Into<String>) -> Term {
        Term::new_word(name)
    }

    /* 🆕Placeholder */

    /// 制作「占位符」
    #[inline]
    pub fn make_placeholder() -> Term {
        Term::new_placeholder()
    }

    /* Variable */

    /// 制作「独立变量」
    #[inline]
    pub fn make_var_i(to_max: impl MaximumVariableId) -> Term {
        Term::new_var_i(to_max.maximum_variable_id() + 1)
    }

    /// 制作「非独变量」
    #[inline]
    pub fn make_var_d(to_max: impl MaximumVariableId) -> Term {
        Term::new_var_d(to_max.maximum_variable_id() + 1)
    }

    /// 制作「查询变量」
    #[inline]
    pub fn make_var_q(to_max: impl MaximumVariableId) -> Term {
        Term::new_var_q(to_max.maximum_variable_id() + 1)
    }

    /// 制作「与现有变量类型相同」的变量
    /// * 🚩类型相同但编号不同
    /// * 🎯用于「变量推理」中的「重命名变量」
    #[inline]
    pub fn make_var_similar(from: &Term, id: impl Into<usize>) -> Term {
        Term::from_var_similar(from.identifier(), id)
    }

    /* 🆕Operator */

    /// 制作「操作符」
    #[inline]
    pub fn make_operator(op: impl Into<String>) -> Term {
        Term::new_operator(op)
    }

    /* CompoundTerm */

    /// 📄OpenNARS `public static Term makeCompoundTerm(CompoundTerm compound, ArrayList<Term> components)`
    pub fn make_compound_term(template: CompoundTermRef, components: Vec<Term>) -> Option<Term> {
        /* 📄OpenNARS
        if (compound instanceof ImageExt)
            // * 🚩外延像
            return makeImageExt(components, ((ImageExt) compound).getRelationIndex());
        else if (compound instanceof ImageInt)
            // * 🚩内涵像
            return makeImageInt(components, ((ImageInt) compound).getRelationIndex());
        else
            // * 🚩其它
            return makeCompoundTerm(compound.operator(), components); */
        let term = template.inner;
        match term.identifier.as_str() {
            IMAGE_EXT_OPERATOR => {
                Self::make_image_ext_arg(components, template.get_placeholder_index())
            }
            IMAGE_INT_OPERATOR => {
                Self::make_image_int_arg(components, template.get_placeholder_index())
            }
            identifier => Self::make_compound_term_from_identifier(identifier, components),
        }
    }

    pub fn make_compound_term_or_statement(
        template: CompoundTermRef,
        mut components: Vec<Term>,
    ) -> Option<Term> {
        match template.as_statement() {
            // * 🚩陈述模板
            Some(statement) => match &components.as_slice() {
                // * 🚩双元素
                &[_, _] => {
                    // * 🚩取出其中仅有的两个元素
                    let predicate = components.pop().unwrap();
                    let subject = components.pop().unwrap();
                    Self::make_statement(&statement, subject, predicate)
                }
                // * 🚩其它⇒无
                _ => None,
            },
            // * 🚩复合词项⇒继续
            _ => Self::make_compound_term(template, components),
        }
    }

    /// 📄OpenNARS `public static Term makeCompoundTerm(String op, ArrayList<Term> arg)`
    pub fn make_compound_term_from_identifier(
        identifier: impl AsRef<str>,
        argument: Vec<Term>,
    ) -> Option<Term> {
        match identifier.as_ref() {
            SET_EXT_OPERATOR => Self::make_set_ext_arg(argument),
            SET_INT_OPERATOR => Self::make_set_int_arg(argument),
            INTERSECTION_EXT_OPERATOR => Self::make_intersection_ext_arg(argument),
            INTERSECTION_INT_OPERATOR => Self::make_intersection_int_arg(argument),
            DIFFERENCE_EXT_OPERATOR => Self::make_difference_ext_arg(argument),
            DIFFERENCE_INT_OPERATOR => Self::make_difference_int_arg(argument),
            PRODUCT_OPERATOR => Self::make_product_arg(argument),
            IMAGE_EXT_OPERATOR => Self::make_image_ext_vec(argument),
            IMAGE_INT_OPERATOR => Self::make_image_int_vec(argument),
            NEGATION_OPERATOR => Self::make_negation_arg(argument),
            CONJUNCTION_OPERATOR => Self::make_conjunction_arg(argument),
            DISJUNCTION_OPERATOR => Self::make_disjunction_arg(argument),
            SEQUENCE_OPERATOR => Self::make_sequence(argument),
            // * 🚩其它⇒未知/域外⇒空
            _ => None,
        }
    }

    // * ℹ️其它与「删改词项」有关的方法，均放在「复合词项引用」中

    // * ✅无需复刻`arguments_to_list`：就是直接构造一个双词项数组，另外还可重定向构造函数
    #[deprecated]
    #[allow(unused)]
    fn arguments_to_list(t1: Term, t2: Term) -> Vec<Term> {
        /* 📄OpenNARS改版
        final ArrayList<Term> list = new ArrayList<>(2);
        list.add(t1);
        list.add(t2);
        return list; */
        vec![t1, t2]
    }

    /* Set */

    /// 制作一个 外延集/内涵集
    /// * 🚩单个词项⇒视作一元数组构造
    fn make_set(t: Term, make_set_arg: fn(Vec<Term>) -> Option<Term>) -> Option<Term> {
        make_set_arg(vec![t])
    }

    /// 制作一个 外延集/内涵集
    /// * 🚩数组⇒统一重排去重⇒构造
    /// * ℹ️相对改版而言，综合「用集合构造」与「用数组构造」
    fn make_set_arg(mut argument: Vec<Term>, new_set: fn(Vec<Term>) -> Term) -> Option<Term> {
        // * 🚩不允许空集
        if argument.is_empty() {
            return None;
        }
        // * 🚩重排去重 | 📌只重排一层：OpenNARS原意如此，并且在外部构建的词项也经过了重排去重
        TermComponents::sort_dedup_term_vec(&mut argument);
        // * 🚩构造
        Some(new_set(argument))
    }

    /* SetExt */

    /// 制作一个外延集
    pub fn make_set_ext(t: Term) -> Option<Term> {
        Self::make_set(t, Self::make_set_ext_arg)
    }

    /// 制作一个外延集
    pub fn make_set_ext_arg(argument: Vec<Term>) -> Option<Term> {
        Self::make_set_arg(argument, Term::new_set_ext)
    }

    /* SetInt */

    /// 制作一个内涵集
    pub fn make_set_int(t: Term) -> Option<Term> {
        Self::make_set(t, Self::make_set_int_arg)
    }

    /// 制作一个内涵集
    pub fn make_set_int_arg(argument: Vec<Term>) -> Option<Term> {
        Self::make_set_arg(argument, Term::new_set_int)
    }

    /* Intersection */

    /// 统一的「外延交/内涵交」制作
    /// * 🔧term1、term2：参与制作的两个词项
    /// * 🚩统一的「外延/内涵」参数前缀：要么统一选左侧，要么统一选右侧
    ///   * 左⇒构造**外延**交
    ///   * 右⇒构造**内涵**交
    #[allow(clippy::too_many_arguments)]
    fn make_intersection(
        term1: Term,
        term2: Term,
        // * 📌有关「同相」的参数：外延→外延，内涵→内涵
        ex_in_set_operator: &str,
        ex_in_intersection_operator: &str,
        ex_in_make_set_arg: fn(Vec<Term>) -> Option<Term>,
        ex_in_make_intersection_vec: fn(Vec<Term>) -> Option<Term>,
        // * 📌有关「反相」的参数：外延→内涵，内涵→外延
        in_ex_set_operator: &str,
        in_ex_make_set_arg: fn(Vec<Term>) -> Option<Term>,
    ) -> Option<Term> {
        // * 🚩预置「词项列表」与「词项制作」
        let mut terms = vec![];
        let make: fn(Vec<Term>) -> Option<Term>;
        // * 🚩两个内涵集取外延交 ⇒ 外延交=内涵并 ⇒ 取并集 | 两个外延集取内涵交 ⇒ 内涵交=外延并 ⇒ 取并集
        // * 📄[A,B] & [C,D] = [A,B,C,D]
        // * 📄{A,B} | {C,D} = {A,B,C,D}
        if let [Some(s1), Some(s2)] = [
            term1.as_compound_type(in_ex_set_operator),
            term2.as_compound_type(in_ex_set_operator),
        ] {
            // * 🚩s1加入最终词项集 | s1加入最终词项集
            terms.extend(s1.components.iter().cloned());
            // * 🚩s2加入最终词项集 | s2加入最终词项集
            terms.extend(s2.components.iter().cloned());
            // * 🚩最终生成内涵集 | 最终生成外延集
            make = in_ex_make_set_arg;
        }
        // * 🚩两个外延集取外延交 ⇒ 取交集 | 两个内涵集取内涵交 ⇒ 取交集
        // * 📄{A,B} & {B,C} = {B}
        // * 📄[A,B] | [B,C] = [B]
        else if let [Some(s1), Some(s2)] = [
            term1.as_compound_type(ex_in_set_operator),
            term2.as_compound_type(ex_in_set_operator),
        ] {
            // * 🚩s1加入最终词项集 | s1加入最终词项集
            terms.extend(s1.components.iter().cloned());
            // * 🚩加入的词项集和s2取交集 | 加入的词项集和s2取交集
            vec_utils::retain_all(&mut terms, s2.components);
            // * 🚩最终生成外延集 | 最终生成内涵集
            make = ex_in_make_set_arg;
        } else {
            // * 🚩均生成外延交 | 注意：在OpenNARS中是传入集合然后重载，此处即改为「直接传递类集合数组」 | 均生成内涵交
            make = ex_in_make_intersection_vec;
            match [
                term1.as_compound_type(ex_in_intersection_operator),
                term2.as_compound_type(ex_in_intersection_operator),
            ] {
                // * 🚩左右都是外延交 ⇒ 取交集 | 左右都是内涵交 ⇒ 取交集
                // * 📄(&,P,Q) & (&,R,S) = (&,P,Q,R,S)
                // * 📄(|,P,Q) | (|,R,S) = (|,P,Q,R,S)
                [Some(s1), Some(s2)] => {
                    terms.extend(s1.components.iter().cloned());
                    terms.extend(s2.components.iter().cloned());
                }
                // * 🚩仅左边是外延交 ⇒ 右边加进左边 | 仅左边是内涵交 ⇒ 右边加进左边
                // * 📄(&,P,Q) & R = (&,P,Q,R)
                // * 📄(|,P,Q) | R = (|,P,Q,R)
                [Some(s1), None] => {
                    terms.extend(s1.components.iter().cloned());
                    terms.push(term2);
                }
                // * 🚩仅右边是外延交 ⇒ 左边加进右边 | 仅右边是内涵交 ⇒ 左边加进右边
                // * 📄R & (&,P,Q) = (&,P,Q,R)
                // * 📄R | (|,P,Q) = (|,P,Q,R)
                [None, Some(s2)] => {
                    terms.extend(s2.components.iter().cloned());
                    terms.push(term1);
                }
                // * 🚩纯默认 ⇒ 直接添加
                // * 📄P & Q = (&,P,Q)
                // * 📄P | Q = (|,P,Q)
                _ => {
                    terms.push(term1);
                    terms.push(term2);
                }
            }
        }

        // * 🚩将「最终词项集」视作「集合」重排去重，然后加入「制作」
        TermComponents::sort_dedup_term_vec(&mut terms);
        make(terms)
    }

    /// * 📝同时包括「用户输入」与「从参数构造」两种来源
    /// * 📄来源1：结构规则「structuralCompose2」
    /// * 🆕现在构造时也会用reduce逻辑尝试合并
    fn make_intersection_arg(
        mut argument: Vec<Term>,
        make_arg: fn(Term, Term) -> Option<Term>,
    ) -> Option<Term> {
        if argument.is_empty() {
            return None;
        }
        // * 🆕🚩做一个reduce的操作
        // ! ❌【2024-06-17 23:52:45】不能「从尾到头」：先后顺序不一样
        let mut term = argument.remove(0);
        // * 🚩取出剩下的
        let mut argument = argument.into_iter();
        while let Some(t) = argument.next() {
            // * 🚩尝试做交集
            term = match make_arg(term, t) {
                // * 🚩成功⇒更新
                Some(new_term) => new_term,
                // * 🚩失败⇒空集⇒跳到下一个
                None => argument.next()?,
            };
        }
        // * 🚩返回
        Some(term)
    }

    /// * 🚩只依照集合数量进行化简
    fn make_intersection_vec(
        mut terms: Vec<Term>,
        new_intersection: fn(Vec<Term>) -> Term,
    ) -> Option<Term> {
        // * 🚩重排去重 | 📌只重排一层：OpenNARS原意如此，并且在外部构建的词项也经过了重排去重
        TermComponents::sort_dedup_term_vec(&mut terms);
        // * 🚩再按照重排后的集合大小分派
        match terms.len() {
            // * 🚩空集⇒空
            0 => None,
            // * 🚩单个元素⇒直接取元素
            1 => terms.pop(),
            // * 🚩其它⇒新建词项
            _ => Some(new_intersection(terms)),
        }
    }

    /* IntersectionExt */

    pub fn make_intersection_ext(term1: Term, term2: Term) -> Option<Term> {
        Self::make_intersection(
            term1,
            term2,
            SET_EXT_OPERATOR,
            INTERSECTION_EXT_OPERATOR,
            Self::make_set_ext_arg,
            Self::make_intersection_ext_vec,
            SET_INT_OPERATOR,
            Self::make_set_int_arg,
        )
    }

    /// * 📝同时包括「用户输入」与「从参数构造」两种来源
    /// * 📄来源1：结构规则「structuralCompose2」
    /// * 🆕现在构造时也会用reduce逻辑尝试合并
    pub fn make_intersection_ext_arg(argument: Vec<Term>) -> Option<Term> {
        Self::make_intersection_arg(argument, Self::make_intersection_ext)
    }

    /// * 🚩只依照集合数量进行化简
    pub fn make_intersection_ext_vec(terms: Vec<Term>) -> Option<Term> {
        Self::make_intersection_vec(terms, Term::new_intersection_ext)
    }

    /* IntersectionInt */

    pub fn make_intersection_int(term1: Term, term2: Term) -> Option<Term> {
        Self::make_intersection(
            term1,
            term2,
            SET_INT_OPERATOR,
            INTERSECTION_INT_OPERATOR,
            Self::make_set_int_arg,
            Self::make_intersection_int_vec,
            SET_EXT_OPERATOR,
            Self::make_set_ext_arg,
        )
    }

    /// * 📝同时包括「用户输入」与「从参数构造」两种来源
    /// * 📄来源1：结构规则「structuralCompose2」
    /// * 🆕现在构造时也会用reduce逻辑尝试合并
    pub fn make_intersection_int_arg(argument: Vec<Term>) -> Option<Term> {
        Self::make_intersection_arg(argument, Self::make_intersection_int)
    }

    /// * 🚩只依照集合数量进行化简
    pub fn make_intersection_int_vec(terms: Vec<Term>) -> Option<Term> {
        Self::make_intersection_vec(terms, Term::new_intersection_int)
    }

    /* Difference */

    fn make_difference(
        left: Term,
        right: Term,
        set_operator: &str,
        make_set_arg: fn(Vec<Term>) -> Option<Term>,
        new_diff: fn(Term, Term) -> Term,
    ) -> Option<Term> {
        // * 🚩自己减自己 ⇒ 空集 ⇒ 空
        if left == right {
            return None;
        }
        match [
            left.as_compound_type(set_operator),
            right.as_compound_type(set_operator),
        ] {
            // * 🚩外延集的差：求差，构造外延集 | {A, B} - {A} = {B}
            // * 🚩内涵集的差：求差，构造内涵集 | [A, B] - [A] = [B]
            [Some(..), Some(..)] => {
                // * 🚩先解包出内部元素（开始丢弃左右所有权）
                let [left, right] = [
                    left.unwrap_compound_components().unwrap(), // ! 先前已假设过复合词项 |
                    right.unwrap_compound_components().unwrap(), // ! 先前已假设过复合词项 |
                ];
                // * 🚩left加入最终词项集 |
                // * 📝to_vec会拷贝元素，故不用之 |
                let mut terms = left.into();
                // * 🚩加入的词项集和right取差集 // set difference |
                vec_utils::remove_all(&mut terms, &right);
                // * 🚩最终生成外延集 |
                make_set_arg(terms)
            }
            // * 🚩否则：直接构造差集
            // * 📄A - B = (-,A,B)
            // * 📄A ~ B = (~,A,B)
            _ => Some(new_diff(left, right)),
        }
    }

    fn make_difference_arg(
        mut argument: Vec<Term>,
        make_difference: fn(Term, Term) -> Option<Term>,
    ) -> Option<Term> {
        match argument.len() {
            // * 🚩单个元素：约简为内部元素（仅在「约简元素」reduceComponent时使用）
            // * 📄(-,A) = A
            // * 📄(~,A) = A
            1 => argument.pop(), // special case from CompoundTerm.reduceComponent
            // * 🚩两个元素⇒进一步判断
            2 => {
                let right = argument.pop().unwrap();
                let left = argument.pop().unwrap();
                make_difference(left, right)
            }
            // * 🚩其它⇒空
            _ => None,
        }
    }

    /* DifferenceExt */

    pub fn make_difference_ext(left: Term, right: Term) -> Option<Term> {
        Self::make_difference(
            left,
            right,
            SET_EXT_OPERATOR,
            Self::make_set_ext_arg,
            Term::new_diff_ext,
        )
    }

    fn make_difference_ext_arg(argument: Vec<Term>) -> Option<Term> {
        Self::make_difference_arg(argument, Self::make_difference_ext)
    }

    /* DifferenceInt */

    pub fn make_difference_int(left: Term, right: Term) -> Option<Term> {
        Self::make_difference(
            left,
            right,
            SET_INT_OPERATOR,
            Self::make_set_int_arg,
            Term::new_diff_int,
        )
    }

    fn make_difference_int_arg(argument: Vec<Term>) -> Option<Term> {
        Self::make_difference_arg(argument, Self::make_difference_int)
    }

    /* Product */

    pub fn make_product_arg(argument: Vec<Term>) -> Option<Term> {
        Some(Term::new_product(argument))
    }

    /// * 🚩从「外延像/内涵像」构造，用某个词项替换掉占位符处的元素，并返回新的关系词项
    ///   * 📄`(/, R, _, b)`, `a` => [`(*, a, b)`, `R`]
    /// * 📝`<a --> (/, R, _, b)>` => `<(*, a, b) --> R>`，其中就要用 a 替换 [R,b] 中的R
    /// * ⚠️【2024-06-16 16:29:18】后续要留意其中与OpenNARS「占位符不作词项」逻辑的不同
    pub fn make_product(image: CompoundTermRef, component: &Term) -> Option<[Term; 2]> {
        let mut terms = vec![];
        let mut image_components = image.components.iter();
        let relation = image_components.next()?.clone();
        for term in image_components {
            // * 🚩占位符⇒跳过
            if term.is_placeholder() {
                // ! ⚠️不递增索引：相当于「先移除占位符，再添加元素」
                terms.push(component.clone());
                continue;
            }
            // * 🚩模拟「替换词项」，但使用「惰性复制」的方式（被替换处的词项不会被复制）
            terms.push(term.clone());
        }
        // * 🚩制作 & 返回
        Self::make_product_arg(terms).map(|product| [product, relation])
    }

    /* Image */

    /// * 📌作为模板的「像」提供「占位符位置」，但作为「组分」的`argument`可能没有占位符
    ///   * ⚠️时刻注意OpenNARS内部存储方式的不同
    ///     * 📄"`(/,neutralization,_,base)` => `[neutralization,base]`+relation_index=0"
    ///     * 📄"`(/,reaction,acid,_)` => `[acid,reaction]`+relation_index=1"
    ///   * ❓【2024-08-05 22:59:21】后续是否要完全革新，不按照OpenNARS的构造方式来
    /// * 📄"(/,num,_)", ["0"] => "(/,0,_)"
    /// * 📄"(/,neutralization,_,base)", ["reaction", "base"] => "(/,reaction,_,base)"
    /// * 📄"(/,reaction,acid,_)", ["acid", "neutralization"] => "(/,neutralization,acid,_)"
    /// * 📄"(/,(*,tim,tom),tom,_)", ["tom", "uncle"] => "(/,uncle,tom,_)";
    fn make_image_arg(
        mut argument: Vec<Term>,
        placeholder_index: usize,
        make_image_vec: fn(Vec<Term>) -> Option<Term>,
    ) -> Option<Term> {
        // * 🚩按占位符位置找到「关系词项」并放在最前边（占位符位置>0）
        debug_assert!(placeholder_index > 0);
        // * 🚩【2024-08-05 22:57:53】补丁：若参数表中有占位符，先移除占位符
        if let Some(old_placeholder_index) = argument.iter().position(|term| term.is_placeholder())
        {
            // * 🚩先移除旧位置的占位符
            argument.remove(old_placeholder_index);
        } else {
            // * 🚩OpenNARS旧情况：先将对应位置的词项当作「关系词项」挪到最开头
            let relation = argument.remove(placeholder_index - 1);
            argument.insert(0, relation);
        }
        // * 🚩再插入占位符
        match placeholder_index >= argument.len() {
            // * 🎯处理edge case: "(/,num,_)", ["0"] => "(/,0,_)"
            true => argument.push(Term::new_placeholder()),
            // * 🚩否则⇒插入
            false => argument.insert(placeholder_index, Term::new_placeholder()),
        }
        // * 🚩制作词项
        make_image_vec(argument)
    }

    fn make_image_vec(
        argument: Vec<Term>,
        new_image: fn(Vec<Term>) -> anyhow::Result<Term>,
    ) -> Option<Term> {
        // * 🚩拒绝元素过少的词项 | 第一个词项需要是「关系」，除此之外必须含有至少一个元素 & 占位符
        if argument.len() < 2 {
            return None;
        }
        // ! 📌【2024-08-05 22:08:05】断言：构造的「像」中只能有正好一个占位符
        debug_assert!(argument.iter().filter(|term| term.is_placeholder()).count() == 1);
        // * 🚩因为「词项中自带占位符」所以无需「特别决定索引」
        new_image(argument).ok()
    }

    /// 共用的「从乘积构造像」逻辑
    /// * ⚠️有关「像」的机制跟OpenNARS实现不一致，将作调整
    ///   * 💭但在效果上是可以一致的
    /// * 🚩整体过程：关系词项插入到最前头，然后在指定的占位符处替换
    ///   * 📌应用「惰性复制」思路
    fn make_image_from_product(
        product: CompoundTermRef,
        relation: &Term,
        index: usize, // * 📝这个指的是「乘积里头挖空」的索引
        make_image_vec: fn(Vec<Term>) -> Option<Term>,
    ) -> Option<Term> {
        // * 🚩关系词项是「乘积」⇒可能可以简化
        if let Some(p2) = relation.as_compound_type(PRODUCT_OPERATOR) {
            // * 🚩对「二元像」作特别的「取索引」简化
            if product.size() == 2 && p2.size() == 2 {
                if index == 0 && product.components[1] == p2.components[1] {
                    // (/,(*,a,b),_,b) with [(*,a,b),b]#0
                    // is reduced to self[0][0] = (*,a,b)[0] = a
                    return Some(p2.components[0].clone());
                }
                if index == 1 && product.components[0] == p2.components[0] {
                    // (/,(*,a,b),a,_) with [a,(*,a,b)]#1
                    // is reduced to self[1][1] = (*,a,b)[1] = b
                    return Some(p2.components[1].clone());
                }
            }
        }
        // * 🚩通过「前插关系词项」与「占位符挖空」构造像
        let mut argument = vec![relation.clone()];
        for (i, term) in product.components.iter().enumerate() {
            let term = match i == index {
                // * 🚩要替换的位置⇒占位符
                true => Term::new_placeholder(),
                // * 🚩其它位置⇒惰性拷贝
                false => term.clone(),
            };
            // * 🚩推送元素
            argument.push(term);
        }
        // * 🚩最终从「装填好的参数」中构造词项
        make_image_vec(argument)
    }

    /// 共用的「从像构造像」逻辑
    /// * 📌从一个已知的外延像中构造新外延像，并切换占位符的位置
    /// * 🚩关系词项位置不变，后头词项改变位置，原占位符填充词项
    ///   * ℹ️输出`[新像, 被换出来的词项]`
    /// * ⚠️`index`的语义：除了第一个「关系词项」外，新的占位符要处在的相对位置
    ///   * 📌最大值：长度-1
    ///   * 📄`(/, R, a, _)`中`index = 0` => 指代位置`a`（而非`R`）
    ///   * 📄`(/, R, a, _)`中`index = 1` => 指代位置`_`（而非`a`，index最大值）
    ///   * 📄`(/, R, a, _)`中`index = 2` => 超出位置
    ///   * ℹ️【2024-08-01 15:53:40】此设定旨在与OpenNARS方案对齐
    /// * 📝本质上是个「替换&插入」的过程
    ///   * 🚩总体过程：
    ///     * `component`插入到原来占位符的位置上
    ///     * `index + 1`处替换为占位符
    ///   * ⚠️index不会指向占位符位置上：
    ///     * 为兼容考虑，此情况将返回`[拷贝的原像, component]`
    fn make_image_from_image(
        old_image: CompoundTermRef,
        component: &Term,
        index: usize,
        make_image_vec: fn(Vec<Term>) -> Option<Term>,
    ) -> Option<[Term; 2]> {
        // * 🚩提取信息 | `old_placeholder_index`算入了「关系词项」
        let old_placeholder_index = old_image.get_placeholder_index();
        // * 🚩判断index是否指向了占位符：若为占位符，直接弹出
        if index + 1 == old_placeholder_index {
            return Some([old_image.inner.clone(), component.clone()]);
        }
        // ! ⚠️【2024-08-08 15:32:34】防御性代码：索引越界⇒驳回
        /* from：
        println!("old_image = {old_image}, component = {component}, index = {index}");
        old_image = /(open _ {}(lock1)), component = $1, index = 2
            at .\src\language\term_impl\term_making.rs:614
            at .\src\language\term_impl\term_making.rs:692
            at .\src\inference\rules\transform_rules.rs:304
            at .\src\inference\rules\transform_rules.rs:154
            at .\src\inference\rules\transform_rules.rs:75
        TODO: 彻查如上bug | 💭思路：可能是在构建「二层转换索引」时出现了问题
         */
        if index + 1 >= old_image.components.len() {
            return None;
        }
        // * 🚩开始选择性添加词项（关系词项也算在内）
        let mut argument = vec![];
        let outer = old_image.components[index + 1].clone();
        for (i, term) in old_image.components.iter().enumerate() {
            let term = if i == index + 1 {
                // * 🚩要替换的位置（要相对「关系词项」后移）⇒占位符
                Term::new_placeholder()
            } else if i == old_placeholder_index {
                // * 🚩原先占位符的位置⇒新元素
                component.clone()
            } else {
                // * 🚩其它位置⇒原词项
                term.clone()
            };
            argument.push(term);
        }
        // * 🚩构造出新词项
        make_image_vec(argument).map(|image| [image, outer])
    }

    /* ImageExt */

    fn make_image_ext_arg(argument: Vec<Term>, placeholder_index: usize) -> Option<Term> {
        Self::make_image_arg(argument, placeholder_index, Self::make_image_ext_vec)
    }

    /// * 🚩从解析器构造外延像
    /// * ⚠️参数argument中含有「占位符」词项
    ///   * ✅这点和OpenNARS相同
    ///
    /// ## 📄OpenNARS中的例子
    ///
    /// * 📄argList=[reaction, _, base] => argument=[reaction, base], index=0
    /// * * => "(/,reaction,_,base)"
    /// * 📄argList=[reaction, acid, _] => argument=[acid, reaction], index=1
    /// * * => "(/,reaction,acid,_)"
    /// * 📄argList=[neutralization, _, base] => argument=[neutralization, base], index=0
    /// * * => "(/,neutralization,_,base)"
    /// * 📄argList=[open, $120, _] => argument=[$120, open], index=1
    /// * * => "(/,open,$120,_)"
    pub fn make_image_ext_vec(argument: impl Into<Vec<Term>>) -> Option<Term> {
        Self::make_image_vec(argument.into(), Term::new_image_ext)
    }

    /// 从一个「乘积」构造外延像
    ///
    /// ## 📄OpenNARS中的例子
    ///
    /// * 📄product="(*,$1,sunglasses)", relation="own",  index=1 => "(/,own,$1,_)"
    /// * 📄product="(*,bird,plant)",    relation="?1",   index=0 => "(/,?1,_,plant)"
    /// * 📄product="(*,bird,plant)",    relation="?1",   index=1 => "(/,?1,bird,_)"
    /// * 📄product="(*,robin,worms)",   relation="food", index=1 => "(/,food,robin,_)"
    /// * 📄product="(*,CAT,eat,fish)",  relation="R",    index=0 => "(/,R,_,eat,fish)"
    /// * 📄product="(*,CAT,eat,fish)",  relation="R",    index=1 => "(/,R,CAT,_,fish)"
    /// * 📄product="(*,CAT,eat,fish)",  relation="R",    index=2 => "(/,R,CAT,eat,_)"
    /// * 📄product="(*,b,a)", relation="(*,b,(/,like,b,_))", index=1 => "(/,like,b,_)"
    /// * 📄product="(*,a,b)", relation="(*,(/,like,b,_),b)", index=0 => "(/,like,b,_)"
    pub fn make_image_ext_from_product(
        product: CompoundTermRef,
        relation: &Term,
        index: usize, // * 📝这个指的是「乘积里头挖空」的索引
    ) -> Option<Term> {
        // * 🚩现在统一在一个「『像』构造」逻辑中
        Self::make_image_from_product(product, relation, index, Self::make_image_ext_vec)
    }

    /// ## 📄OpenNARS中的例子
    ///
    /// * 📄oldImage="(/,open,{key1},_)",   component="lock",   index=0 => "(/,open,_,lock)"
    /// * 📄oldImage="(/,uncle,_,tom)",     component="tim",    index=1 => "(/,uncle,tim,_)"
    /// * 📄oldImage="(/,open,{key1},_)",   component="$2",     index=0 => "(/,open,_,$2)"
    /// * 📄oldImage="(/,open,{key1},_)",   component="#1",     index=0 => "(/,open,_,#1)"
    /// * 📄oldImage="(/,like,_,a)",        component="b",      index=1 => "(/,like,b,_)"
    /// * 📄oldImage="(/,like,b,_)",        component="a",      index=0 => "(/,like,_,a)"
    pub fn make_image_ext_from_image(
        old_image: CompoundTermRef,
        component: &Term,
        index: usize,
    ) -> Option<[Term; 2]> {
        // * 🚩现在统一在一个「『像』构造」逻辑中
        Self::make_image_from_image(old_image, component, index, Self::make_image_ext_vec)
    }

    /* ImageInt */

    fn make_image_int_arg(argument: Vec<Term>, placeholder_index: usize) -> Option<Term> {
        Self::make_image_arg(argument, placeholder_index, Self::make_image_int_vec)
    }

    pub fn make_image_int_vec(argument: impl Into<Vec<Term>>) -> Option<Term> {
        Self::make_image_vec(argument.into(), Term::new_image_int)
    }

    pub fn make_image_int_from_product(
        product: CompoundTermRef,
        relation: &Term,
        index: usize, // * 📝这个指的是「乘积里头挖空」的索引
    ) -> Option<Term> {
        // * 🚩现在统一在一个「『像』构造」逻辑中
        Self::make_image_from_product(product, relation, index, Self::make_image_int_vec)
    }

    /// ## 📄OpenNARS中的例子
    ///
    /// * 📄oldImage=`(\,X,_,eat,fish)`,          component=`cat`,  index=`2` => `(\,X,cat,eat,_)`
    /// * 📄oldImage=`(\,reaction,acid,_)`,       component=`soda`, index=`0` => `(\,reaction,_,soda)`
    /// * 📄oldImage=`(\,X,_,eat,fish)`,          component=`Y`,    index=`2` => `(\,X,Y,eat,_)`
    /// * 📄oldImage=`(\,neutralization,_,soda)`, component=`acid`, index=`1` => `(\,neutralization,acid,_)`
    /// * 📄oldImage=`(\,neutralization,acid,_)`, component=`$1`,   index=`0` => `(\,neutralization,_,$1)`
    /// * 📄oldImage=`(\,REPRESENT,_,$1)`,        component=`Y`,    index=`1` => `(\,REPRESENT,Y,_)`
    ///
    /// ℹ️更多例子详见单元测试用例
    pub fn make_image_int_from_image(
        old_image: CompoundTermRef,
        component: &Term,
        index: usize,
    ) -> Option<[Term; 2]> {
        // * 🚩现在统一在一个「『像』构造」逻辑中
        Self::make_image_from_image(old_image, component, index, Self::make_image_int_vec)
    }

    /* Junction */

    /// 同时代表「从数组」与「从集合」
    fn make_junction_arg(
        mut argument: Vec<Term>,
        new_junction: fn(Vec<Term>) -> Term,
    ) -> Option<Term> {
        // * 🚩重排去重 | 📌只重排一层：OpenNARS原意如此，并且在外部构建的词项也经过了重排去重
        TermComponents::sort_dedup_term_vec(&mut argument);
        // * 🚩再根据参数数目分派
        match argument.len() {
            // * 🚩不允许空集
            0 => None,
            // * 🚩单元素⇒直接用元素（可提取）
            // special case: single component
            1 => argument.pop(),
            // * 🚩多元素⇒构造新的词项
            _ => Some(new_junction(argument)),
        }
    }

    /// 从推理规则中构建
    fn make_junction(
        term1: Term,
        term2: Term,
        junction_operator: &str,
        make_junction_arg: fn(Vec<Term>) -> Option<Term>,
    ) -> Option<Term> {
        let mut terms: Vec<Term> = vec![];
        match term1.as_compound_type(junction_operator) {
            // * 🚩同类⇒合并
            Some(..) => terms.extend(
                term1
                    .unwrap_compound_components()
                    .expect("已判断是复合词项")
                    .into_vec(),
            ),
            // * 🚩异类⇒加入
            _ => terms.push(term1),
        }
        match term2.as_compound_type(junction_operator) {
            // * 🚩同类⇒合并
            Some(..) => terms.extend(
                term2
                    .unwrap_compound_components()
                    .expect("已判断是复合词项")
                    .into_vec(),
            ),
            // * 🚩异类⇒加入
            _ => terms.push(term2),
        }
        make_junction_arg(terms)
    }

    /* Conjunction */
    // ? 【2024-06-17 23:24:39】单独的单元测试

    pub fn make_conjunction_arg(argument: Vec<Term>) -> Option<Term> {
        Self::make_junction_arg(argument, Term::new_conjunction)
    }

    pub fn make_conjunction(term1: Term, term2: Term) -> Option<Term> {
        Self::make_junction(
            term1,
            term2,
            CONJUNCTION_OPERATOR,
            Self::make_conjunction_arg,
        )
    }

    /* Disjunction */
    // ? 【2024-06-17 23:24:39】单独的单元测试

    pub fn make_disjunction_arg(argument: Vec<Term>) -> Option<Term> {
        Self::make_junction_arg(argument, Term::new_disjunction)
    }

    pub fn make_disjunction(term1: Term, term2: Term) -> Option<Term> {
        Self::make_junction(
            term1,
            term2,
            DISJUNCTION_OPERATOR,
            Self::make_disjunction_arg,
        )
    }

    /* Negation */
    // ? 【2024-06-17 23:24:39】单独的单元测试

    pub fn make_negation(t: Term) -> Option<Term> {
        match t.as_compound_type(NEGATION_OPERATOR) {
            // * 🚩双重否定⇒肯定
            // * 📄-- (--,P) = P
            Some(..) => t
                .unwrap_compound_components()
                .expect("已经假定是复合词项")
                .into_vec()
                .pop(), // * 📌只能使用pop来安全取出元素。。
            // * 🚩其它⇒只有一个参数的「否定」词项
            None => Self::make_negation_arg(vec![t]),
        }
    }

    fn make_negation_arg(mut argument: Vec<Term>) -> Option<Term> {
        match argument.len() {
            // * 🚩仅有一个⇒构造
            1 => Some(Term::new_negation(argument.pop().unwrap())),
            // * 🚩其它⇒空（失败）
            _ => None,
        }
    }

    /* Sequence */

    /// 从「前后」两个词项构建序列
    /// * 📄最初参考自ONA
    /// * 🚩内部带有「序列」⇒展开
    /// * 🚩单个词项⇒展开
    /// * 🚩无词项⇒展开
    pub fn make_sequence(argument: impl IntoIterator<Item = Term>) -> Option<Term> {
        let mut components: Vec<Term> = vec![];
        /// 展开其中的嵌套子序列
        fn flatten_append(components: &mut Vec<Term>, argument: Term) {
            // 内部序列⇒展开
            if argument.instanceof_sequence() {
                let terms = argument
                    .unwrap_compound_components()
                    .expect("已经判断是复合词项");
                for term in terms.into_vec() {
                    flatten_append(components, term);
                }
            }
            // 直接添加
            else {
                components.push(argument);
            }
        }

        // 遍历所有参数，添加
        for argument in argument {
            flatten_append(&mut components, argument);
        }

        // 确定结果
        match components.len() {
            // * 🚩空⇒空
            0 => None,
            // * 🚩仅有一个⇒提取出本身
            1 => Some(components.pop().unwrap()),
            // * 🚩其它⇒构造序列
            _ => Some(Term::new_sequence(components)),
        }
    }

    /* Statement */

    /// 从一个「陈述系词」中构造
    pub fn make_statement_relation(
        copula: impl AsRef<str>,
        subject: Term,
        predicate: Term,
    ) -> Option<Term> {
        // * 🚩无效⇒制作失败
        if StatementRef::invalid_statement(&subject, &predicate) {
            return None;
        }
        // * 🚩按照「陈述系词」分派
        match copula.as_ref() {
            INHERITANCE_RELATION => Self::make_inheritance(subject, predicate),
            SIMILARITY_RELATION => Self::make_similarity(subject, predicate),
            INSTANCE_RELATION => Self::make_instance(subject, predicate),
            PROPERTY_RELATION => Self::make_property(subject, predicate),
            INSTANCE_PROPERTY_RELATION => Self::make_instance_property(subject, predicate),
            IMPLICATION_RELATION => Self::make_implication(subject, predicate),
            EQUIVALENCE_RELATION => Self::make_equivalence(subject, predicate),
            TEMPORAL_IMPLICATION_RELATION => Self::make_temporal_implication(subject, predicate),
            _ => None,
        }
    }

    /// 从模板中制作「陈述」
    /// * 🎯推理规则
    /// * 🚩【2024-07-08 11:45:32】放宽对「词项类型」的限制
    ///   * 📌实际上只需识别标识符
    /// * ♻️【2024-08-05 00:58:29】直接使用[`Self::make_statement_relation`]
    ///   * 📌目前保持「依照『模板词项』的标识符制作陈述」的语义
    ///   * ✅由此也兼容了「实例/属性/实例属性」等外部系词
    pub fn make_statement(template: &Term, subject: Term, predicate: Term) -> Option<Term> {
        // * 🚩直接是`make_statement_relation`的链接
        Term::make_statement_relation(template.identifier(), subject, predicate)
    }

    /// 📄OpenNARS `Statement.makeSym`
    /// * 🚩通过使用「标识符映射」将「非对称版本」映射到「对称版本」
    /// * ⚠️目前只支持「继承」和「蕴含」，其它均会`panic`
    /// * 🚩【2024-07-23 15:35:41】实际上并不需要「复合词项引用」：只是对标识符做分派
    ///
    /// # 📄OpenNARS
    /// Make a symmetric Statement from given components and temporal information,
    /// called by the rules
    pub fn make_statement_symmetric(
        template: &Term,
        subject: Term,
        predicate: Term,
    ) -> Option<Term> {
        match template.identifier() {
            // 继承⇒相似
            INHERITANCE_RELATION => Self::make_similarity(subject, predicate),
            // 蕴含⇒等价
            IMPLICATION_RELATION => Self::make_equivalence(subject, predicate),
            // 其它⇒panic
            identifier => unimplemented!("不支持的标识符：{identifier:?}"),
        }
    }

    /* Inheritance */

    pub fn make_inheritance(subject: Term, predicate: Term) -> Option<Term> {
        // * 🚩检查有效性
        match StatementRef::invalid_statement(&subject, &predicate) {
            true => None,
            false => Some(Term::new_inheritance(subject, predicate)),
        }
    }

    /* Instance */

    /// * 🚩转发 ⇒ 继承 + 外延集
    pub fn make_instance(subject: Term, predicate: Term) -> Option<Term> {
        Self::make_inheritance(Self::make_set_ext(subject)?, predicate)
    }

    /* Property */

    /// * 🚩转发 ⇒ 继承 + 内涵集
    pub fn make_property(subject: Term, predicate: Term) -> Option<Term> {
        Self::make_inheritance(subject, Self::make_set_int(predicate)?)
    }

    /* InstanceProperty */

    /// * 🚩转发 ⇒ 继承 + 外延集 + 内涵集
    pub fn make_instance_property(subject: Term, predicate: Term) -> Option<Term> {
        Self::make_inheritance(Self::make_set_ext(subject)?, Self::make_set_int(predicate)?)
    }

    /* Similarity */

    pub fn make_similarity(subject: Term, predicate: Term) -> Option<Term> {
        // * 🚩检查有效性
        match StatementRef::invalid_statement(&subject, &predicate) {
            true => None,
            // * ✅在创建时自动排序
            false => Some(Term::new_similarity(subject, predicate)),
        }
    }

    /* Implication */

    pub fn make_implication(subject: Term, predicate: Term) -> Option<Term> {
        // TODO: 🚧【2024-09-07 15:28:30】有待继续提取至独立的「检查是否合法」方法
        //   * 🏗️后续继续为「变量替换后检查有效性」做准备
        // * 🚩检查有效性
        if StatementRef::invalid_statement(&subject, &predicate) {
            return None;
        }
        // * 🚩检查主词类型
        if subject.instanceof_implication() || subject.instanceof_equivalence() {
            return None;
        }
        if predicate.instanceof_equivalence() {
            return None;
        }
        // B in <A ==> <B ==> C>>
        if predicate.as_compound_type(IMPLICATION_RELATION).is_some() {
            let [old_condition, predicate_predicate] = predicate
                .unwrap_statement_components()
                .expect("已经假定是复合词项");
            // ! ❌ <A ==> <(&&, A, B) ==> C>>
            // ? ❓为何不能合并：实际上A && (&&, A, B) = (&&, A, B)
            if let Some(conjunction) = old_condition.as_compound_type(CONJUNCTION_OPERATOR) {
                if conjunction.contain_component(&subject) {
                    return None;
                }
            }
            // * ♻️ <A ==> <B ==> C>> ⇒ <(&&, A, B) ==> C>
            let new_condition = Self::make_conjunction(subject, old_condition)?;
            Self::make_implication(new_condition, predicate_predicate)
        } else {
            Some(Term::new_implication(subject, predicate))
        }
    }

    /* Equivalence */

    pub fn make_equivalence(subject: Term, predicate: Term) -> Option<Term> {
        // to be extended to check if subject is Conjunction
        // * 🚩检查非法主谓组合
        // ! <<A ==> B> <=> C> or <<A <=> B> <=> C>
        if subject.instanceof_implication() || subject.instanceof_equivalence() {
            return None;
        }
        // ! <C <=> <C ==> D>> or <C <=> <C <=> D>>
        if subject.instanceof_implication() || subject.instanceof_equivalence() {
            return None;
        }
        // ! <A <=> A>, <<A --> B> <=> <B --> A>>
        // * 🚩检查有效性
        match StatementRef::invalid_statement(&subject, &predicate) {
            true => None,
            // * ✅在创建时自动排序
            false => Some(Term::new_equivalence(subject, predicate)),
        }
    }

    /* TemporalImplication */

    pub fn make_temporal_implication(subject: Term, predicate: Term) -> Option<Term> {
        // TODO: 🚧【2024-09-07 15:28:30】有待继续提取至独立的「检查是否合法」方法
        //   * 🏗️后续继续为「变量替换后检查有效性」做准备
        // * 🚩检查有效性
        if StatementRef::invalid_statement(&subject, &predicate) {
            return None;
        }
        // * 🚩检查主词类型
        if subject.instanceof_temporal_implication() {
            return None;
        }
        // B in <A ==> <B ==> C>>
        if predicate.instanceof_temporal_implication() {
            let [old_condition, predicate_predicate] = predicate
                .unwrap_statement_components()
                .expect("已经假定是复合词项");
            // * ♻️ <A ==> <B ==> C>> ⇒ <(&&, A, B) ==> C>
            let new_condition = Self::make_sequence([subject, old_condition])?;
            Self::make_temporal_implication(new_condition, predicate_predicate)
        } else {
            Some(Term::new_temporal_implication(subject, predicate))
        }
    }
}

#[cfg(test)]
mod tests {
    use super::super::test_term::*;
    use super::*;
    use crate::{ok, option_term, test_term as term, util::AResult};
    use nar_dev_utils::macro_once;

    /// 具体的词项构造
    /// * 📄外延集、内涵集……
    mod concrete_type {
        use super::*;

        fn test_make_one(term: Term, expected: Option<Term>, make: fn(Term) -> Option<Term>) {
            // * 🚩格式化字符串，以备后用
            let term_str = term.to_string();
            // * 🚩传入两个词项所有权，制作新词项
            let out = make(term);
            // * 🚩检验
            assert_eq!(
                out,
                expected,
                "{term_str:?} => {} != {}",
                format_option_term(&out),
                format_option_term(&expected)
            );
        }

        fn test_make_one_f(make: fn(Term) -> Option<Term>) -> impl Fn(Term, Option<Term>) {
            move |term, expected| test_make_one(term, expected, make)
        }

        fn test_make_two(
            term1: Term,
            term2: Term,
            expected: Option<Term>,
            make: fn(Term, Term) -> Option<Term>,
        ) {
            // * 🚩格式化字符串，以备后用
            let term1_str = term1.to_string();
            let term2_str = term2.to_string();
            // * 🚩传入两个词项所有权，制作新词项
            let out = make(term1, term2);
            // * 🚩检验
            assert_eq!(
                out,
                expected,
                "{term1_str:?}, {term2_str:?} => {} != {}",
                format_option_term(&out),
                format_option_term(&expected)
            );
        }

        fn test_make_two_f(
            make: fn(Term, Term) -> Option<Term>,
        ) -> impl Fn(Term, Term, Option<Term>) {
            move |t1, t2, expected| test_make_two(t1, t2, expected, make)
        }

        fn test_make_arg(
            terms: Vec<Term>,
            expected: Option<Term>,
            make: fn(Vec<Term>) -> Option<Term>,
        ) {
            // * 🚩格式化字符串，以备后用
            let terms_str = format!("{terms:?}");
            // * 🚩传入两个词项所有权，制作新词项
            let out = make(terms);
            // * 🚩检验
            assert_eq!(
                out,
                expected,
                "{terms_str:?} => {} != {}",
                format_option_term(&out),
                format_option_term(&expected)
            );
        }

        fn test_make_arg_f(
            make: fn(Vec<Term>) -> Option<Term>,
        ) -> impl Fn(Vec<Term>, Option<Term>) {
            move |argument, expected| test_make_arg(argument, expected, make)
        }

        fn test_make_image_from_product_f(
            make: fn(CompoundTermRef, &Term, usize) -> Option<Term>,
        ) -> impl Fn(Term, Term, usize, Term) {
            move |p, relation, index, expected| {
                let product = p.as_compound().expect("解析出的不是复合词项！");
                let image = make(product, &relation, index).expect("词项制作失败！");
                assert_eq!(
                    image, expected,
                    "{product}, {relation}, {index} => {image} != {expected}"
                );
            }
        }

        fn test_make_image_from_image_f(
            make: fn(CompoundTermRef, &Term, usize) -> Option<[Term; 2]>,
        ) -> impl Fn(Term, Term, usize, Term, Term) {
            move |i, component, index, expected, expected_outer| {
                let old_image = i.as_compound().expect("解析出的不是复合词项！");
                let [image, outer] = make(old_image, &component, index).expect("词项制作失败！");
                assert_eq!(
                    image, expected,
                    "{old_image}, {component}, {index} => {image} != {expected}"
                );
                assert_eq!(
                    outer, expected_outer,
                    "{old_image}, {component}, {index} => {outer} != {expected_outer}"
                );
            }
        }

        /* SetExt */

        #[test]
        fn make_set_ext() -> AResult {
            let test = test_make_one_f(Term::make_set_ext);
            macro_once! {
                // * 🚩模式：参数列表 ⇒ 预期词项
                macro test($($t:tt => $expected:tt;)*) {
                    $( test(term!($t) ,option_term!($expected)); )*
                }
                "tom" => "{tom}";
                "Tweety" => "{Tweety}";
                "Saturn" => "{Saturn}";
                "Venus" => "{Venus}";
                "tim" => "{tim}";
                "Birdie" => "{Birdie}";
                "Pluto" => "{Pluto}";
            }
            ok!()
        }

        #[test]
        fn make_set_ext_arg() -> AResult {
            let test = test_make_arg_f(Term::make_set_ext_arg);
            macro_once! {
                // * 🚩模式：参数列表 ⇒ 预期词项
                macro test($($argument:tt => $expected:tt;)*) {
                    $( test(term!($argument).into(), option_term!($expected)); )*
                }
                [] => None;
                ["?49"] => "{?49}";
                ["Mars", "Pluto", "Venus"] => "{Mars,Pluto,Venus}";
                ["Birdie"] => "{Birdie}";
                ["lock"] => "{lock}";
                ["#1"] => "{#1}";
                ["key1"] => "{key1}";
                ["Pluto", "Saturn"] => "{Pluto,Saturn}";
                ["Mars", "Venus"] => "{Mars,Venus}";
                ["lock1"] => "{lock1}";
                ["Tweety"] => "{Tweety}";
            }
            ok!()
        }

        /* SetInt */

        #[test]
        fn make_set_int() -> AResult {
            let test = test_make_one_f(Term::make_set_int);
            macro_once! {
                // * 🚩模式：参数列表 ⇒ 预期词项
                macro test($($t:tt => $expected:tt;)*) {
                    $( test(term!($t) ,option_term!($expected)); )*
                }
                "[1]" => "[[1]]";
                "[{1}]" => "[[{1}]]";
                "{[<[1] --> {1}>]}" => "[{[<[1] --> {1}>]}]";
                // * ℹ️以下用例源自OpenNARS实际运行
                "black" => "[black]";
                "yellow" => "[yellow]";
            }
            ok!()
        }

        #[test]
        fn make_set_int_arg() -> AResult {
            let test = test_make_arg_f(Term::make_set_int_arg);
            macro_once! {
                // * 🚩模式：参数列表 ⇒ 预期词项
                macro test($($argument:tt => $expected:tt;)*) {
                    $( test(term!($argument).into(), option_term!($expected)); )*
                }
                [] => None;
                ["1", "2"] => "[1, 2]";
                ["1", "2", "[1]", "[2]"] => "[1, 2, [1], [2]]";
                ["1", "2", "<1 --> 2>", "<1 --> 2>"] => "[1, 2, <1 --> 2>]"; // 去重
                // * ℹ️以下用例源自OpenNARS实际运行
                ["flying"]     => "[flying]";
                ["unscrewing"] => "[unscrewing]";
                ["with_wings"] => "[with_wings]";
                ["smart"]      => "[smart]";
                ["bright"]     => "[bright]";
                ["strong"]     => "[strong]";
                ["living"]     => "[living]";
                ["chirping"]   => "[chirping]";
                ["aggressive"] => "[aggressive]";
                ["black"]      => "[black]";
                ["bendable"]   => "[bendable]";
                ["hurt"]       => "[hurt]";
                ["with_beak"]  => "[with_beak]";
            }
            ok!()
        }

        /* IntersectionExt */

        #[test]
        fn make_intersection_ext() -> AResult {
            let test = test_make_two_f(Term::make_intersection_ext);
            macro_once! {
                // * 🚩模式：函数参数 ⇒ 预期词项
                macro test($($term1:tt, $term2:tt => $expected:tt;)*) {
                    $( test(term!($term1), term!($term2), option_term!($expected)); )*
                }
                // * ℹ️用例均源自OpenNARS实际运行
                // 集合之间的交集
                "[with_wings]", "[with_wings,yellow]" => "[with_wings,with_wings,yellow]";
                "[with_wings]", "[with_wings]" => "[with_wings,with_wings]";
                "{Mars,Pluto,Venus}", "{Pluto,Saturn}" => "{Pluto}";
                "{Mars,Venus}", "{Pluto,Saturn}" => None;
                "{Pluto,Saturn}", "{Mars,Pluto,Venus}" => "{Pluto}";
                "{Tweety}", "{Birdie}" => None;
                // 其它情形
                "#1", "bird" => "(&,#1,bird)";
                "#1", "{Birdie}" => "(&,#1,{Birdie})";
                "(&,bird,{Birdie})", "[yellow]" => "(&,bird,[yellow],{Birdie})";
                "(&,bird,{Birdie})", "flyer" => "(&,bird,flyer,{Birdie})";
                "(&,flyer,{Birdie})", "(&,bird,[yellow])" => "(&,bird,flyer,[yellow],{Birdie})";
                "(|,bird,flyer)", "#1" => "(&,#1,(|,bird,flyer))";
                "(|,bird,flyer)", "(|,bird,{Birdie})" => "(&,(|,bird,flyer),(|,bird,{Birdie}))";
                "(|,flyer,{Birdie})", "(|,#1,flyer)" => "(&,(|,#1,flyer),(|,flyer,{Birdie}))";
                "(|,flyer,{Birdie})", "[with-wings]" => "(&,[with-wings],(|,flyer,{Birdie}))";
                "<{Tweety} --> bird>", "<bird --> fly>" => "(&,<bird --> fly>,<{Tweety} --> bird>)";
                "[strong]", "(~,youth,girl)" => "(&,[strong],(~,youth,girl))";
                "[yellow]", "bird" => "(&,bird,[yellow])";
                "animal", "bird" => "(&,animal,bird)";
                "bird", "#1" => "(&,#1,bird)";
                "bird", "(|,#1,flyer)" => "(&,bird,(|,#1,flyer))";
                "bird", "[with-wings]" => "(&,bird,[with-wings])";
                "bird", "[yellow]" => "(&,bird,[yellow])";
                "bird", "{Birdie}" => "(&,bird,{Birdie})";
                "flyer", "(&,bird,[yellow])" => "(&,bird,flyer,[yellow])";
                "flyer", "(&,bird,{Birdie})" => "(&,bird,flyer,{Birdie})";
                "{Birdie}", "[with-wings]" => "(&,[with-wings],{Birdie})";
                "{Birdie}", "[with_wings]" => "(&,[with_wings],{Birdie})";
                "{Birdie}", "bird" => "(&,bird,{Birdie})";
                "{Tweety}", "#1" => "(&,#1,{Tweety})";
            }
            ok!()
        }

        /* IntersectionInt */
        #[test]
        fn make_intersection_int() -> AResult {
            let test = test_make_two_f(Term::make_intersection_int);
            macro_once! {
                // * 🚩模式：函数参数 ⇒ 预期词项
                macro test($($term1:tt, $term2:tt => $expected:tt;)*) {
                    $( test(term!($term1), term!($term2), option_term!($expected)); )*
                }
                // * ℹ️用例均源自OpenNARS实际运行
                "#1", "(&,bird,{Birdie})" => "(|,#1,(&,bird,{Birdie}))";
                "#1", "bird" => "(|,#1,bird)";
                "#1", "{Birdie}" => "(|,#1,{Birdie})";
                "(&,#1,{lock1})", "lock1" => "(|,lock1,(&,#1,{lock1}))";
                "(&,[with-wings],{Birdie})", "(&,bird,flyer)" => "(|,(&,bird,flyer),(&,[with-wings],{Birdie}))";
                "(&,bird,{Birdie})", "[yellow]" => "(|,[yellow],(&,bird,{Birdie}))";
                "(&,bird,{Birdie})", "flyer" => "(|,flyer,(&,bird,{Birdie}))";
                "(&,flyer,{Birdie})", "(&,bird,[yellow])" => "(|,(&,bird,[yellow]),(&,flyer,{Birdie}))";
                "(&,flyer,{Birdie})", "(&,bird,{Birdie})" => "(|,(&,bird,{Birdie}),(&,flyer,{Birdie}))";
                "(|,#1,bird)", "{Birdie}" => "(|,#1,bird,{Birdie})";
                "(|,[with-wings],(&,bird,[yellow]))", "flyer" => "(|,flyer,[with-wings],(&,bird,[yellow]))";
                "(|,bird,flyer)", "#1" => "(|,#1,bird,flyer)";
                "(|,bird,flyer)", "(|,bird,{Birdie})" => "(|,bird,flyer,{Birdie})";
                "(|,bird,flyer)", "{Birdie}" => "(|,bird,flyer,{Birdie})";
                "(|,flyer,[with_wings])", "{Birdie}" => "(|,flyer,[with_wings],{Birdie})";
                "(|,flyer,{Birdie})", "(|,#1,flyer)" => "(|,#1,flyer,{Birdie})";
                "(|,flyer,{Birdie})", "[with-wings]" => "(|,flyer,[with-wings],{Birdie})";
                "(|,flyer,{Tweety})", "{Birdie}" => "(|,flyer,{Birdie},{Tweety})";
                "(~,boy,girl)", "(~,youth,girl)" => "(|,(~,boy,girl),(~,youth,girl))";
                "[strong]", "(~,youth,girl)" => "(|,[strong],(~,youth,girl))";
                "[with-wings]", "#1" => "(|,#1,[with-wings])";
                "[with-wings]", "(&,bird,[yellow])" => "(|,[with-wings],(&,bird,[yellow]))";
                "[with-wings]", "(&,bird,flyer)" => "(|,[with-wings],(&,bird,flyer))";
                "[with-wings]", "(&,bird,{Birdie})" => "(|,[with-wings],(&,bird,{Birdie}))";
                "[with-wings]", "(|,bird,flyer)" => "(|,bird,flyer,[with-wings])";
                "[with-wings]", "[with_wings,yellow]" => None;
                "[with-wings]", "{Birdie}" => "(|,[with-wings],{Birdie})";
                "[with_wings]", "(&,bird,[with-wings])" => "(|,[with_wings],(&,bird,[with-wings]))";
                "[with_wings]", "(&,bird,{Birdie})" => "(|,[with_wings],(&,bird,{Birdie}))";
                "[with_wings]", "(|,bird,{Birdie})" => "(|,bird,[with_wings],{Birdie})";
                "[with_wings]", "[with-wings]" => None;
                "[with_wings]", "[yellow]" => None;
                "[with_wings]", "bird" => "(|,bird,[with_wings])";
                "[with_wings]", "{Birdie}" => "(|,[with_wings],{Birdie})";
                "animal", "bird" => "(|,animal,bird)";
                "bird", "#1" => "(|,#1,bird)";
                "bird", "(&,bird,{Birdie})" => "(|,bird,(&,bird,{Birdie}))";
                "bird", "(|,#1,flyer)" => "(|,#1,bird,flyer)";
                "bird", "(|,bird,flyer)" => "(|,bird,flyer)";
                "bird", "(|,flyer,[with-wings])" => "(|,bird,flyer,[with-wings])";
                "bird", "[with-wings]" => "(|,bird,[with-wings])";
                "bird", "[yellow]" => "(|,bird,[yellow])";
                "bird", "{Birdie}" => "(|,bird,{Birdie})";
                "boy", "(~,youth,girl)" => "(|,boy,(~,youth,girl))";
                "flyer", "(&,bird,[with-wings])" => "(|,flyer,(&,bird,[with-wings]))";
                "flyer", "(&,bird,[yellow])" => "(|,flyer,(&,bird,[yellow]))";
                "robin", "(|,#1,{Birdie})" => "(|,#1,robin,{Birdie})";
                "{Birdie}", "(|,[with_wings],(&,bird,[with-wings]))" => "(|,[with_wings],{Birdie},(&,bird,[with-wings]))";
                "{Birdie}", "[with-wings]" => "(|,[with-wings],{Birdie})";
                "{Birdie}", "[with_wings]" => "(|,[with_wings],{Birdie})";
                "{Birdie}", "bird" => "(|,bird,{Birdie})";
                "{Mars,Pluto,Venus}", "{Pluto,Saturn}" => "{Mars,Pluto,Saturn,Venus}";
                "{Mars,Venus}", "{Pluto,Saturn}" => "{Mars,Pluto,Saturn,Venus}";
                "{Pluto,Saturn}", "{Mars,Pluto,Venus}" => "{Mars,Pluto,Saturn,Venus}";
                "{Tweety}", "#1" => "(|,#1,{Tweety})";
                "{Tweety}", "{Birdie}" => "{Birdie,Tweety}";
            }
            ok!()
        }

        /* DifferenceExt */

        #[test]
        fn make_difference_ext_arg() -> AResult {
            let test = test_make_arg_f(Term::make_difference_ext_arg);
            macro_once! {
                // * 🚩模式：参数列表 ⇒ 预期词项
                macro test($($arg_list:tt => $expected:expr;)*) {
                    $( test(term!($arg_list).into(), option_term!($expected)); )*
                }
                // * ℹ️用例均源自OpenNARS实际运行
                ["swimmer", "bird"] => "(-,swimmer,bird)";
                ["mammal", "swimmer"] => "(-,mammal,swimmer)";
                ["bird", "swimmer"] => "(-,bird,swimmer)";
                ["swimmer", "animal"] => "(-,swimmer,animal)";
            }
            ok!()
        }

        #[test]
        fn make_difference_ext() -> AResult {
            let test = test_make_two_f(Term::make_difference_ext);
            macro_once! {
                // * 🚩模式：参数列表 ⇒ 预期词项
                macro test($($term1:tt, $term2:tt => $expected:expr;)*) {
                    $( test(term!($term1), term!($term2), option_term!($expected)); )*
                }
                // * ℹ️用例均源自OpenNARS实际运行
                "(&,bird,(|,[yellow],{Birdie}))", "[with_wings]" => "(-,(&,bird,(|,[yellow],{Birdie})),[with_wings])";
                "(&,bird,flyer)", "[with_wings]" => "(-,(&,bird,flyer),[with_wings])";
                "(&,flyer,[yellow])", "[with_wings]" => "(-,(&,flyer,[yellow]),[with_wings])";
                "(&,flyer,{Birdie})", "[with_wings]" => "(-,(&,flyer,{Birdie}),[with_wings])";
                "(/,open,_,#1)", "(/,open,_,{lock1})" => "(-,(/,open,_,#1),(/,open,_,{lock1}))";
                "(|,[yellow],{Birdie})", "[with_wings]" => "(-,(|,[yellow],{Birdie}),[with_wings])";
                "(|,[yellow],{Birdie})", "bird" => "(-,(|,[yellow],{Birdie}),bird)";
                "(|,bird,flyer)", "[with_wings]" => "(-,(|,bird,flyer),[with_wings])";
                "(|,bird,swimmer)", "animal" => "(-,(|,bird,swimmer),animal)";
                "(|,bird,{Birdie})", "[with_wings]" => "(-,(|,bird,{Birdie}),[with_wings])";
                "(|,chess,competition)", "(|,competition,sport)" => "(-,(|,chess,competition),(|,competition,sport))";
                "(|,flyer,[with_wings])", "[yellow]" => "(-,(|,flyer,[with_wings]),[yellow])";
                "(|,flyer,[yellow])", "{Birdie}" => "(-,(|,flyer,[yellow]),{Birdie})";
                "[yellow]", "[with_wings]" => "(-,[yellow],[with_wings])";
                "[yellow]", "bird" => "(-,[yellow],bird)";
                "[yellow]", "{Birdie}" => "(-,[yellow],{Birdie})";
                "animal", "swimmer" => "(-,animal,swimmer)";
                "bird", "[with_wings]" => "(-,bird,[with_wings])";
                "{Birdie}", "[with_wings]" => "(-,{Birdie},[with_wings])";
                "{Birdie}", "flyer" => "(-,{Birdie},flyer)";
                "{Mars,Pluto,Venus}", "{Pluto,Saturn}" => "{Mars,Venus}";
            }
            ok!()
        }

        /* DifferenceInt */

        #[test]
        fn make_difference_int_arg() -> AResult {
            let test = test_make_arg_f(Term::make_difference_int_arg);
            macro_once! {
                // * 🚩模式：参数列表 ⇒ 预期词项
                macro test($($arg_list:tt => $expected:expr;)*) {
                    $( test(term!($arg_list).into(), option_term!($expected)); )*
                }
                // * ℹ️用例均源自OpenNARS实际运行
                ["(~,boy,girl)", "girl"] => "(~,(~,boy,girl),girl)";
                ["swimmer", "swan"] => "(~,swimmer,swan)";
                ["youth", "girl"] => "(~,youth,girl)";
                ["(|,boy,girl)", "girl"] => "(~,(|,boy,girl),girl)";
                ["boy", "girl"] => "(~,boy,girl)";
                ["(/,(*,tim,tom),tom,_)", "(/,uncle,tom,_)"] => "(~,(/,(*,tim,tom),tom,_),(/,uncle,tom,_))";
                ["[strong]", "girl"] => "(~,[strong],girl)";
            }
            ok!()
        }

        #[test]
        fn make_difference_int() -> AResult {
            let test = test_make_two_f(Term::make_difference_int);
            macro_once! {
                // * 🚩模式：参数列表 ⇒ 预期词项
                macro test($($term1:tt, $term2:tt => $expected:expr;)*) {
                    $( test(term!($term1), term!($term2), option_term!($expected)); )*
                }
                // * ℹ️用例均源自OpenNARS实际运行
                "(&,bird,robin)", "tiger" => "(~,(&,bird,robin),tiger)";
                "(&,flyer,{Birdie})", "(&,flyer,robin)" => "(~,(&,flyer,{Birdie}),(&,flyer,robin))";
                "(&,flyer,{Birdie})", "robin" => "(~,(&,flyer,{Birdie}),robin)";
                "(/,(*,tim,tom),tom,_)", "(/,uncle,tom,_)" => "(~,(/,(*,tim,tom),tom,_),(/,uncle,tom,_))";
                "(/,(*,tim,tom),tom,_)", "tim" => "(~,(/,(*,tim,tom),tom,_),tim)";
                "(/,open,_,lock)", "{key1}" => "(~,(/,open,_,lock),{key1})";
                "(|,bird,robin)", "tiger" => "(~,(|,bird,robin),tiger)";
                "(|,flyer,[with_wings],{Birdie})", "robin" => "(~,(|,flyer,[with_wings],{Birdie}),robin)";
                "(|,flyer,{Birdie})", "robin" => "(~,(|,flyer,{Birdie}),robin)";
                "(~,boy,girl)", "girl" => "(~,(~,boy,girl),girl)";
                "[strong]", "girl" => "(~,[strong],girl)";
                "animal", "bird" => "(~,animal,bird)";
                "bird", "#1" => "(~,bird,#1)";
                "bird", "(|,robin,tiger)" => "(~,bird,(|,robin,tiger))";
                "{Birdie}", "(|,flyer,robin)" => "(~,{Birdie},(|,flyer,robin))";
                "{Birdie}", "robin" => "(~,{Birdie},robin)";
                "{Tweety}", "(&,flyer,robin)" => "(~,{Tweety},(&,flyer,robin))";
                "{Tweety}", "(|,robin,[yellow],{Birdie})" => "(~,{Tweety},(|,robin,[yellow],{Birdie}))";
                "{lock1}", "#1" => "(~,{lock1},#1)";
            }
            ok!()
        }

        /* ImageExt */

        #[test]
        fn make_image_ext_vec() -> AResult {
            let test = test_make_arg_f(Term::make_image_ext_vec);
            macro_once! {
                // * 🚩模式：参数列表 ⇒ 预期词项
                macro test($($arg_list:tt => $expected:expr;)*) {
                    $( test(term!($arg_list).into(), option_term!($expected)); )*
                }
                ["reaction", "_", "base"] => "(/,reaction,_,base)";
                ["reaction", "acid", "_"] => "(/,reaction,acid,_)";
                ["neutralization", "_", "base"] => "(/,neutralization,_,base)";
                ["open", "$120", "_"] => "(/,open,$120,_)";
            }
            ok!()
        }

        #[test]
        fn make_image_ext_from_product() -> AResult {
            let test = test_make_image_from_product_f(Term::make_image_ext_from_product);
            macro_once! {
                // * 🚩模式：参数列表 ⇒ 预期词项
                macro test($($product:tt, $relation:tt, $index:tt => $expected:expr;)*) {
                    $( test( term!($product), term!($relation), $index, term!($expected) ); )*
                }
                // * ℹ️用例均源自OpenNARS实际运行
                "(*,$1,sunglasses)", "own",                1 => "(/,own,$1,_)";
                "(*,bird,plant)",    "?1",                 0 => "(/,?1,_,plant)";
                "(*,bird,plant)",    "?1",                 1 => "(/,?1,bird,_)";
                "(*,robin,worms)",   "food",               1 => "(/,food,robin,_)";
                "(*,CAT,eat,fish)",  "R",                  0 => "(/,R,_,eat,fish)";
                "(*,CAT,eat,fish)",  "R",                  1 => "(/,R,CAT,_,fish)";
                "(*,CAT,eat,fish)",  "R",                  2 => "(/,R,CAT,eat,_)";
                "(*,b,a)",           "(*,b,(/,like,b,_))", 1 => "(/,like,b,_)";
                "(*,a,b)",           "(*,(/,like,b,_),b)", 0 => "(/,like,b,_)";
                // 特别替换
                r"(*,(/,like,b,_),b)",                   r"(*,a,b)",                            0 => r"a";
                r"(*,(&,key,(/,open,_,{lock1})),lock1)", r"(*,{key1},lock1)",                   0 => r"{key1}";
                r"(*,(\,reaction,_,soda),base)",         r"(*,(\,neutralization,_,soda),base)", 0 => r"(\,neutralization,_,soda)";
                r"(*,(&,key,(/,open,_,{lock1})),lock)",  r"(*,{key1},lock)",                    0 => r"{key1}";
                r"(*,b,(/,like,b,_))",                   r"(*,b,a)",                            1 => r"a";
                r"(*,(/,like,_,a),a)",                   r"(*,b,a)",                            0 => r"b";
            }
            ok!()
        }

        #[test]
        fn make_image_ext_from_image() -> AResult {
            let test = test_make_image_from_image_f(Term::make_image_ext_from_image);
            macro_once! {
                // * 🚩模式：参数列表 ⇒ 预期词项
                macro test($($image:tt, $component:tt, $index:tt => [$expected:expr, $expected_outer:expr];)*) {
                    $( test( term!($image), term!($component), $index, term!($expected), term!($expected_outer) ); )*
                }
                // * 📌特殊用例：误打误撞占位符
                "(/,open,{key1},_)",   "lock",   1 => ["(/,open,{key1},_)", "lock"];
                // * ℹ️用例均源自OpenNARS实际运行
                "(/,open,{key1},_)",   "lock",   0 => ["(/,open,_,lock)", "{key1}"];
                "(/,uncle,_,tom)",     "tim",    1 => ["(/,uncle,tim,_)", "tom"];
                "(/,open,{key1},_)",   "$2",     0 => ["(/,open,_,$2)", "{key1}"];
                "(/,open,{key1},_)",   "#1",     0 => ["(/,open,_,#1)", "{key1}"];
                "(/,like,_,a)",        "b",      1 => ["(/,like,b,_)", "a"];
                "(/,like,b,_)",        "a",      0 => ["(/,like,_,a)", "b"];
            }
            ok!()
        }

        /* ImageInt */

        #[test]
        fn make_image_int_vec() -> AResult {
            let test = test_make_arg_f(Term::make_image_int_vec);
            macro_once! {
                // * 🚩模式：参数列表 ⇒ 预期词项
                macro test($($arg_list:tt => $expected:expr;)*) {
                    $( test(term!($arg_list).into(), option_term!($expected)); )*
                }
                // * ℹ️用例均源自OpenNARS实际运行
                ["reaction", "_", "base"]       => r"(\,reaction,_,base)";
                ["reaction", "acid", "_"]       => r"(\,reaction,acid,_)";
                ["neutralization", "_", "base"] => r"(\,neutralization,_,base)";
                ["open", "$120", "_"]           => r"(\,open,$120,_)";
            }
            ok!()
        }

        #[test]
        fn make_image_int_from_product() -> AResult {
            let test = test_make_image_from_product_f(Term::make_image_int_from_product);
            macro_once! {
                // * 🚩模式：参数列表 ⇒ 预期词项
                macro test($($product:tt, $relation:tt, $index:tt => $expected:expr;)*) {
                    $( test( term!($product), term!($relation), $index, term!($expected) ); )*
                }
                // * ℹ️用例均源自OpenNARS实际运行
                r"(*,(/,num,_))",                       "#1",                0 => r"(\,#1,_)";
                r"(*,(\,reaction,_,soda),base)",        "neutralization",    1 => r"(\,neutralization,(\,reaction,_,soda),_)";
                r"(*,(\,reaction,_,soda),base)",        "neutralization",    0 => r"(\,neutralization,_,base)";
                r"(*,(/,num,_))",                       "(*,num)",           0 => r"(\,(*,num),_)";
                r"(*,acid,soda)",                       "reaction",          0 => r"(\,reaction,_,soda)";
                r"(*,(*,num))",                         "(*,(*,(/,num,_)))", 0 => r"(\,(*,(*,(/,num,_))),_)";
                r"(*,(*,(*,num)))",                     "(*,(*,(*,0)))",     0 => r"(\,(*,(*,(*,0))),_)";
                r"(*,(\,reaction,_,soda),base)",        "#1",                1 => r"(\,#1,(\,reaction,_,soda),_)";
                r"(*,(*,num))",                         "(*,(*,0))",         0 => r"(\,(*,(*,0)),_)";
                r"(*,acid,base)",                       "reaction",          0 => r"(\,reaction,_,base)";
                r"(*,b,(/,like,b,_))",                  "(*,b,a)",           0 => r"(\,(*,b,a),_,(/,like,b,_))";
                r"(*,(\,reaction,_,soda),base)",        "#1",                0 => r"(\,#1,_,base)";
                r"(*,(*,(/,num,_)))",                   "(*,(*,0))",         0 => r"(\,(*,(*,0)),_)";
                r"(*,(/,num,_))",                       "(*,0)",             0 => r"(\,(*,0),_)";
                r"(*,(/,num,_))",                       "$1",                0 => r"(\,$1,_)";
                r"(*,num)",                             "(*,0)",             0 => r"(\,(*,0),_)";
                r"(*,acid,soda)",                       "reaction",          1 => r"(\,reaction,acid,_)";
                r"(*,(/,like,_,a),a)",                  "(*,b,a)",           1 => r"(\,(*,b,a),(/,like,_,a),_)";
                r"(*,acid,base)",                       "reaction",          1 => r"(\,reaction,acid,_)";
                r"(*,(&,key,(/,open,_,{lock1})),lock)", "(*,{key1},lock)",   1 => r"(\,(*,{key1},lock),(&,key,(/,open,_,{lock1})),_)";
                r"(*,(/,like,b,_),b)",                  "(*,a,b)",           1 => r"(\,(*,a,b),(/,like,b,_),_)";
                // 特别替换
                r"(*,(\,reaction,_,soda),base)",         r"(*,(\,reaction,_,soda),soda)",       1 => r"soda";
                r"(*,(\,reaction,_,soda),base)",         r"(*,acid,base)",                      0 => r"acid";
                r"(*,acid,(\,neutralization,acid,_))",   r"(*,acid,(\,reaction,acid,_))",       1 => r"(\,reaction,acid,_)";
                r"(*,(&,key,(/,open,_,{lock1})),lock)",  r"(*,{key1},lock)",                    0 => r"{key1}";
                r"(*,(\,neutralization,_,soda),base)",   r"(*,(\,reaction,_,soda),base)",       0 => r"(\,reaction,_,soda)";
                r"(*,(/,open,_,#1),{lock1})",            r"(*,{key1},{lock1})",                 0 => r"{key1}";
                r"(*,key,lock)",                         r"(*,{key1},lock)",                    0 => r"{key1}";
                r"(*,acid,(\,reaction,acid,_))",         r"(*,acid,soda)",                      1 => r"soda";
                r"(*,(|,key,(/,open,_,{lock1})),lock1)", r"(*,{key1},lock1)",                   0 => r"{key1}";
                r"(*,(&,key,(/,open,_,{lock1})),lock1)", r"(*,{key1},lock1)",                   0 => r"{key1}";
            }
            ok!()
        }

        #[test]
        fn make_image_int_from_image() -> AResult {
            let test = test_make_image_from_image_f(Term::make_image_int_from_image);
            macro_once! {
                // * 🚩模式：参数列表 ⇒ 预期词项
                macro test($($image:tt, $component:tt, $index:tt => [$expected:expr, $expected_outer:expr];)*) {
                    $( test( term!($image), term!($component), $index, term!($expected), term!($expected_outer) ); )*
                }
                // * 📌特殊用例：误打误撞占位符
                r"(\,R,_,eat,fish)",           "cat",                       0 => [r"(\,R,_,eat,fish)", "cat"];
                // * ℹ️用例均源自OpenNARS实际运行
                r"(\,R,_,eat,fish)",           "cat",                       2 => [r"(\,R,cat,eat,_)", "fish"];
                r"(\,reaction,acid,_)",        "soda",                      0 => [r"(\,reaction,_,soda)", "acid"];
                r"(\,R,_,eat,fish)",          r"(\,REPRESENT,_,$1)",        2 => [r"(\,R,(\,REPRESENT,_,$1),eat,_)", "fish"];
                r"(\,neutralization,_,soda)",  "acid",                      1 => [r"(\,neutralization,acid,_)", "soda"];
                r"(\,neutralization,acid,_)",  "$1",                        0 => [r"(\,neutralization,_,$1)", "acid"];
                r"(\,REPRESENT,_,$1)",        r"(\,R,_,eat,fish)",          1 => [r"(\,REPRESENT,(\,R,_,eat,fish),_)", "$1"];
                r"(\,neutralization,acid,_)",  "soda",                      0 => [r"(\,neutralization,_,soda)", "acid"];
                r"(\,neutralization,acid,_)",  "?1",                        0 => [r"(\,neutralization,_,?1)", "acid"];
                r"(\,reaction,acid,_)",       r"(\,neutralization,acid,_)", 0 => [r"(\,reaction,_,(\,neutralization,acid,_))", "acid"];
                r"(\,REPRESENT,_,CAT)",        "(/,R,_,eat,fish)",          1 => [r"(\,REPRESENT,(/,R,_,eat,fish),_)", "CAT"];
                r"(\,R,_,eat,fish)",          r"(\,REPRESENT,_,$1)",        1 => [r"(\,R,(\,REPRESENT,_,$1),_,fish)", "eat"];
                r"(\,R,_,eat,fish)",           "cat",                       1 => [r"(\,R,cat,_,fish)", "eat"];
                r"(\,reaction,_,soda)",        "acid",                      1 => [r"(\,reaction,acid,_)", "soda"];
                r"(\,reaction,_,base)",       r"(\,reaction,_,soda)",       1 => [r"(\,reaction,(\,reaction,_,soda),_)", "base"];
                r"(\,neutralization,acid,_)",  "#1",                        0 => [r"(\,neutralization,_,#1)", "acid"];
                r"(\,neutralization,acid,_)",  "base",                      0 => [r"(\,neutralization,_,base)", "acid"];
                r"(\,reaction,_,base)",        "acid",                      1 => [r"(\,reaction,acid,_)", "base"];
                r"(\,neutralization,acid,_)",  "(/,reaction,acid,_)",       0 => [r"(\,neutralization,_,(/,reaction,acid,_))", "acid"];
            }
            ok!()
        }
    }

    mod compound {
        use super::*;

        fn test_make_term_with_identifier_f(
            make: fn(&str, Vec<Term>) -> Option<Term>,
        ) -> impl Fn(&str, Vec<Term>, Option<Term>) {
            move |identifier, terms, expected| {
                let terms_str = terms
                    .iter()
                    .map(|t| format!("\"{t}\""))
                    .collect::<Vec<_>>()
                    .join(", ");
                let out = make(identifier, terms);
                assert_eq!(
                    out,
                    expected,
                    "{identifier:?}, {terms_str} => {} != {}",
                    format_option_term(&out),
                    format_option_term(&expected),
                );
            }
        }

        #[test]
        fn make_compound_term_from_identifier() -> AResult {
            fn make(identifier: &str, terms: Vec<Term>) -> Option<Term> {
                Term::make_compound_term_from_identifier(identifier, terms)
            }
            let test = test_make_term_with_identifier_f(make);
            macro_once! {
                // * 🚩模式：参数列表 ⇒ 预期词项
                macro test($($identifier:tt, $terms:tt => $expected:tt;)*) {
                    $( test($identifier, term!($terms).into(), option_term!($expected)); )*
                }
                // * ℹ️用例均源自OpenNARS实际运行
                "&", ["(&,robin,{Tweety})", "{Birdie}"] => "(&,robin,{Birdie},{Tweety})";
                "&", ["(/,neutralization,_,(\\,neutralization,acid,_))", "acid"] => "(&,acid,(/,neutralization,_,(\\,neutralization,acid,_)))";
                "&", ["(/,neutralization,_,base)", "(/,reaction,_,base)"] => "(&,(/,neutralization,_,base),(/,reaction,_,base))";
                "&", ["(/,neutralization,_,base)", "acid"] => "(&,acid,(/,neutralization,_,base))";
                "&", ["(/,open,_,{lock1})", "(/,open,_,lock)"] => "(&,(/,open,_,lock),(/,open,_,{lock1}))";
                "&", ["(\\,REPRESENT,_,CAT)", "(/,(/,REPRESENT,_,<(*,CAT,FISH) --> FOOD>),_,eat,fish)"] => "(&,(\\,REPRESENT,_,CAT),(/,(/,REPRESENT,_,<(*,CAT,FISH) --> FOOD>),_,eat,fish))";
                "&", ["(\\,reaction,_,soda)", "(\\,neutralization,_,base)"] => "(&,(\\,neutralization,_,base),(\\,reaction,_,soda))";
                "&", ["(|,(/,open,_,lock1),(/,open,_,{lock1}))", "(/,open,_,lock)"] => "(&,(/,open,_,lock),(|,(/,open,_,lock1),(/,open,_,{lock1})))";
                "&", ["(|,bird,{Tweety})", "(|,bird,{Birdie})"] => "(&,(|,bird,{Birdie}),(|,bird,{Tweety}))";
                "&", ["(|,key,(/,open,_,{lock1}))", "(/,open,_,lock)"] => "(&,(/,open,_,lock),(|,key,(/,open,_,{lock1})))";
                "&", ["acid", "(/,reaction,_,base)"] => "(&,acid,(/,reaction,_,base))";
                "&", ["acid", "(\\,neutralization,_,base)"] => "(&,acid,(\\,neutralization,_,base))";
                "&", ["animal", "(&,robin,swan)"] => "(&,animal,robin,swan)";
                "&", ["animal", "(|,animal,swimmer)"] => "(&,animal,(|,animal,swimmer))";
                "&", ["animal", "gull"] => "(&,animal,gull)";
                "&", ["bird", "robin", "{Birdie}", "(|,[yellow],{Birdie})"] => "(&,bird,robin,{Birdie},(|,[yellow],{Birdie}))";
                "&", ["flyer", "[with_wings]"] => "(&,flyer,[with_wings])";
                "&", ["flyer", "{Birdie}", "(|,[with_wings],{Birdie})"] => "(&,flyer,{Birdie},(|,[with_wings],{Birdie}))";
                "&", ["flyer", "{Birdie}"] => "(&,flyer,{Birdie})";
                "&", ["key", "(/,open,_,{lock1})"] => "(&,key,(/,open,_,{lock1}))";
                "&", ["neutralization", "(*,(\\,neutralization,_,base),base)"] => "(&,neutralization,(*,(\\,neutralization,_,base),base))";
                "&", ["neutralization", "(*,acid,(/,reaction,acid,_))"] => "(&,neutralization,(*,acid,(/,reaction,acid,_)))";
                "&", ["neutralization", "(*,acid,base)"] => "(&,neutralization,(*,acid,base))";
                "&", ["num", "(/,num,_)"] => "(&,num,(/,num,_))";
                "&", ["{Birdie}", "(|,flyer,{Tweety})"] => "(&,{Birdie},(|,flyer,{Tweety}))";
                "&", ["{Birdie}", "{Tweety}"] => None;
                "&&", ["<robin --> [chirping]>", "<robin --> [flying]>"] => "(&&,<robin --> [chirping]>,<robin --> [flying]>)";
                "&&", ["<robin --> [chirping]>"] => "<robin --> [chirping]>";
                "&&", ["<robin --> bird>", "(||,(&&,<robin --> [flying]>,<robin --> [with_wings]>),<robin --> bird>)"] => "(&&,<robin --> bird>,(||,(&&,<robin --> [flying]>,<robin --> [with_wings]>),<robin --> bird>))";
                "&&", ["<robin --> bird>", "<robin --> [flying]>", "<robin --> [with_wings]>"] => "(&&,<robin --> bird>,<robin --> [flying]>,<robin --> [with_wings]>)";
                "&&", ["<robin --> bird>", "<robin --> [flying]>"] => "(&&,<robin --> bird>,<robin --> [flying]>)";
                "&&", ["<robin --> bird>"] => "<robin --> bird>";
                "&&", ["<robin --> flyer>", "<(*,robin,worms) --> food>"] => "(&&,<robin --> flyer>,<(*,robin,worms) --> food>)";
                "&&", ["<robin --> flyer>", "<robin --> bird>", "<(*,robin,worms) --> food>"] => "(&&,<robin --> bird>,<robin --> flyer>,<(*,robin,worms) --> food>)";
                "&&", ["<robin --> flyer>", "<robin --> bird>", "<worms --> (/,food,robin,_)>"] => "(&&,<robin --> bird>,<robin --> flyer>,<worms --> (/,food,robin,_)>)";
                "&&", ["<robin --> flyer>", "<robin --> bird>"] => "(&&,<robin --> bird>,<robin --> flyer>)";
                "&&", ["<robin --> flyer>", "<worms --> (/,food,robin,_)>"] => "(&&,<robin --> flyer>,<worms --> (/,food,robin,_)>)";
                "*", ["(&,key,(/,open,_,{lock1}))", "lock"] => "(*,(&,key,(/,open,_,{lock1})),lock)";
                "*", ["(&,num,(/,(*,(/,num,_)),_))"] => "(*,(&,num,(/,(*,(/,num,_)),_)))";
                "*", ["(*,num)"] => "(*,(*,num))";
                "*", ["(/,(*,(/,num,_)),_)"] => "(*,(/,(*,(/,num,_)),_))";
                "*", ["(/,(/,num,_),_)"] => "(*,(/,(/,num,_),_))";
                "*", ["(/,REPRESENT,_,<(*,CAT,FISH) --> FOOD>)", "<(*,CAT,FISH) --> FOOD>"] => "(*,(/,REPRESENT,_,<(*,CAT,FISH) --> FOOD>),<(*,CAT,FISH) --> FOOD>)";
                "*", ["(/,num,_)"] => "(*,(/,num,_))";
                "*", ["(/,open,_,lock)", "lock"] => "(*,(/,open,_,lock),lock)";
                "*", ["(/,open,_,lock)", "{lock1}"] => "(*,(/,open,_,lock),{lock1})";
                "*", ["(/,open,_,{lock1})", "lock"] => "(*,(/,open,_,{lock1}),lock)";
                "*", ["(/,open,_,{lock1})", "{lock1}"] => "(*,(/,open,_,{lock1}),{lock1})";
                "*", ["(\\,neutralization,_,base)", "base"] => "(*,(\\,neutralization,_,base),base)";
                "*", ["(|,(/,open,_,lock1),(/,open,_,{lock1}))", "lock1"] => "(*,(|,(/,open,_,lock1),(/,open,_,{lock1})),lock1)";
                "*", ["(|,key,(/,open,_,{lock1}))", "lock"] => "(*,(|,key,(/,open,_,{lock1})),lock)";
                "*", ["0"] => "(*,0)";
                "*", ["a", "b"] => "(*,a,b)";
                "*", ["acid", "(&,soda,(/,neutralization,acid,_))"] => "(*,acid,(&,soda,(/,neutralization,acid,_)))";
                "*", ["acid", "(/,neutralization,acid,_)"] => "(*,acid,(/,neutralization,acid,_))";
                "*", ["acid", "(\\,neutralization,acid,_)"] => "(*,acid,(\\,neutralization,acid,_))";
                "*", ["acid", "(|,base,(\\,reaction,acid,_))"] => "(*,acid,(|,base,(\\,reaction,acid,_)))";
                "*", ["key", "{lock1}"] => "(*,key,{lock1})";
                "*", ["{key1}", "lock1"] => "(*,{key1},lock1)";
                "[]", ["bright"] => "[bright]";
                "{}", ["Birdie"] => "{Birdie}";
                "{}", ["Mars", "Venus"] => "{Mars,Venus}";
                "|", ["(&,animal,gull)", "swimmer"] => "(|,swimmer,(&,animal,gull))";
                "|", ["(&,flyer,{Birdie})", "(|,[yellow],{Birdie})"] => "(|,[yellow],{Birdie},(&,flyer,{Birdie}))";
                "|", ["(&,flyer,{Birdie})", "{Birdie}"] => "(|,{Birdie},(&,flyer,{Birdie}))";
                "|", ["(/,neutralization,_,base)", "(/,reaction,_,(\\,neutralization,acid,_))"] => "(|,(/,neutralization,_,base),(/,reaction,_,(\\,neutralization,acid,_)))";
                "|", ["(/,neutralization,_,base)", "(/,reaction,_,base)"] => "(|,(/,neutralization,_,base),(/,reaction,_,base))";
                "|", ["(/,neutralization,_,base)", "acid"] => "(|,acid,(/,neutralization,_,base))";
                "|", ["(/,neutralization,acid,_)", "(\\,neutralization,acid,_)"] => "(|,(/,neutralization,acid,_),(\\,neutralization,acid,_))";
                "|", ["(/,num,_)", "0"] => "(|,0,(/,num,_))";
                "|", ["(/,open,_,{lock1})", "(/,open,_,lock)"] => "(|,(/,open,_,lock),(/,open,_,{lock1}))";
                "|", ["(\\,REPRESENT,_,CAT)", "(/,(/,REPRESENT,_,<(*,CAT,FISH) --> FOOD>),_,eat,fish)"] => "(|,(\\,REPRESENT,_,CAT),(/,(/,REPRESENT,_,<(*,CAT,FISH) --> FOOD>),_,eat,fish))";
                "|", ["(|,key,(/,open,_,{lock1}))", "(/,open,_,lock)"] => "(|,key,(/,open,_,lock),(/,open,_,{lock1}))";
                "|", ["(~,boy,girl)", "(~,youth,girl)"] => "(|,(~,boy,girl),(~,youth,girl))";
                "|", ["[with_wings]", "(|,flyer,{Tweety})", "{Birdie}"] => "(|,flyer,[with_wings],{Birdie},{Tweety})";
                "|", ["[with_wings]", "flyer", "{Birdie}"] => "(|,flyer,[with_wings],{Birdie})";
                "|", ["[with_wings]", "{Birdie}", "(|,[with_wings],{Birdie})"] => "(|,[with_wings],{Birdie})";
                "|", ["[with_wings]", "{Tweety}", "{Birdie}"] => "(|,[with_wings],{Birdie},{Tweety})";
                "|", ["[yellow]", "[with_wings]"] => None;
                "|", ["[yellow]", "bird"] => "(|,bird,[yellow])";
                "|", ["[yellow]", "{Tweety}"] => "(|,[yellow],{Tweety})";
                "|", ["acid", "(/,reaction,_,base)"] => "(|,acid,(/,reaction,_,base))";
                "|", ["acid", "(\\,neutralization,_,base)"] => "(|,acid,(\\,neutralization,_,base))";
                "|", ["animal", "robin"] => "(|,animal,robin)";
                "|", ["bird", "[with_wings]"] => "(|,bird,[with_wings])";
                "|", ["bird", "flyer", "{Birdie}"] => "(|,bird,flyer,{Birdie})";
                "|", ["bird", "{Birdie}"] => "(|,bird,{Birdie})";
                "|", ["bird", "{Tweety}", "{Birdie}"] => "(|,bird,{Birdie},{Tweety})";
                "|", ["boy", "(~,youth,girl)"] => "(|,boy,(~,youth,girl))";
                "|", ["chess", "(|,chess,sport)"] => "(|,chess,sport)";
                "|", ["flyer", "(&,flyer,{Birdie})", "{Birdie}"] => "(|,flyer,{Birdie},(&,flyer,{Birdie}))";
                "|", ["flyer", "(&,flyer,{Birdie})"] => "(|,flyer,(&,flyer,{Birdie}))";
                "|", ["flyer", "(|,flyer,{Tweety})", "{Birdie}"] => "(|,flyer,{Birdie},{Tweety})";
                "|", ["flyer", "[yellow]", "{Birdie}"] => "(|,flyer,[yellow],{Birdie})";
                "|", ["flyer", "{Birdie}", "(&,bird,(|,[yellow],{Birdie}))"] => "(|,flyer,{Birdie},(&,bird,(|,[yellow],{Birdie})))";
                "|", ["flyer", "{Birdie}", "(&,flyer,{Birdie})"] => "(|,flyer,{Birdie},(&,flyer,{Birdie}))";
                "|", ["key", "(/,open,_,{lock1})"] => "(|,key,(/,open,_,{lock1}))";
                "|", ["neutralization", "(*,acid,(\\,neutralization,acid,_))"] => "(|,neutralization,(*,acid,(\\,neutralization,acid,_)))";
                "|", ["neutralization", "(*,acid,base)"] => "(|,neutralization,(*,acid,base))";
                "|", ["robin", "(|,flyer,{Tweety})", "{Birdie}"] => "(|,flyer,robin,{Birdie},{Tweety})";
                "|", ["tiger", "(|,animal,swimmer)"] => "(|,animal,swimmer,tiger)";
                "|", ["{Birdie}", "{Tweety}"] => "{Birdie,Tweety}";
                "|", ["{Tweety}", "{Birdie}", "(&,flyer,{Birdie})"] => "(|,(&,flyer,{Birdie}),{Birdie,Tweety})";
                "~", ["(/,(*,tim,tom),tom,_)", "(/,uncle,tom,_)"] => "(~,(/,(*,tim,tom),tom,_),(/,uncle,tom,_))";
                "~", ["(|,boy,girl)", "girl"] => "(~,(|,boy,girl),girl)";
                "~", ["(~,boy,girl)", "girl"] => "(~,(~,boy,girl),girl)";
                "~", ["[strong]", "girl"] => "(~,[strong],girl)";
                "~", ["boy", "girl"] => "(~,boy,girl)";
            }
            ok!()
        }

        #[test]
        fn make_compound_term() -> AResult {
            fn test(template: Term, terms: Vec<Term>, expected: Option<Term>) {
                let terms_str = terms
                    .iter()
                    .map(|t| format!("\"{t}\""))
                    .collect::<Vec<_>>()
                    .join(", ");
                let out = Term::make_compound_term(
                    template.as_compound().expect("模板不是复合词项！"),
                    terms,
                );
                assert_eq!(
                    out,
                    expected,
                    "\"{template}\", {terms_str} => {} != {}",
                    format_option_term(&out),
                    format_option_term(&expected),
                );
            }
            macro_once! {
                // * 🚩模式：参数列表 ⇒ 预期词项
                macro test($($template:tt, $terms:tt => $expected:tt;)*) {
                    $(
                        test(
                            term!($template),
                            term!($terms).into(),
                            option_term!($expected),
                        );
                    )*
                }
                // * ℹ️用例均源自OpenNARS实际运行
                "(&&,<robin --> [chirping]>,<robin --> [flying]>)", ["<robin --> [chirping]>"] => "<robin --> [chirping]>";
                "(&&,<robin --> [chirping]>,<robin --> [flying]>)", ["<robin --> bird>", "<robin --> [flying]>"] => "(&&,<robin --> bird>,<robin --> [flying]>)";
                "(&&,<robin --> [chirping]>,<robin --> [flying]>,<robin --> [with_wings]>)", ["<robin --> [chirping]>", "<robin --> [flying]>"] => "(&&,<robin --> [chirping]>,<robin --> [flying]>)";
                "(&&,<robin --> [chirping]>,<robin --> [flying]>,<robin --> [with_wings]>)", ["<robin --> bird>", "<robin --> [flying]>", "<robin --> [with_wings]>"] => "(&&,<robin --> bird>,<robin --> [flying]>,<robin --> [with_wings]>)";
                "(&&,<robin --> [chirping]>,<robin --> [with_wings]>)", ["<robin --> [chirping]>", "<robin --> bird>"] => "(&&,<robin --> bird>,<robin --> [chirping]>)";
                "(&&,<robin --> bird>,<robin --> [flying]>)", ["<robin --> [flying]>"] => "<robin --> [flying]>";
                "(&&,<robin --> bird>,<robin --> [flying]>)", ["<robin --> bird>"] => "<robin --> bird>";
                "(&&,<robin --> bird>,<robin --> [flying]>,<robin --> [with_wings]>)", ["<robin --> [flying]>", "<robin --> [with_wings]>"] => "(&&,<robin --> [flying]>,<robin --> [with_wings]>)";
                "(&&,<robin --> bird>,<robin --> [flying]>,<robin --> [with_wings]>)", ["<robin --> bird>", "<robin --> [flying]>", "<robin --> bird>"] => "(&&,<robin --> bird>,<robin --> [flying]>)";
                "(&&,<robin --> bird>,<robin --> [flying]>,<robin --> [with_wings]>)", ["<robin --> bird>", "<robin --> [flying]>"] => "(&&,<robin --> bird>,<robin --> [flying]>)";
                "(&&,<robin --> bird>,<robin --> [living]>)", ["<robin --> bird>", "(||,(&&,<robin --> [flying]>,<robin --> [with_wings]>),<robin --> bird>)"] => "(&&,<robin --> bird>,(||,(&&,<robin --> [flying]>,<robin --> [with_wings]>),<robin --> bird>))";
                "(&&,<robin --> bird>,<robin --> [living]>)", ["<robin --> bird>", "<robin --> [flying]>", "<robin --> [with_wings]>"] => "(&&,<robin --> bird>,<robin --> [flying]>,<robin --> [with_wings]>)";
                "(&&,<robin --> bird>,<robin --> [living]>)", ["<robin --> bird>", "<robin --> [flying]>"] => "(&&,<robin --> bird>,<robin --> [flying]>)";
                "(&&,<robin --> bird>,<robin --> [living]>)", ["<robin --> bird>", "<robin --> bird>", "<robin --> [flying]>"] => "(&&,<robin --> bird>,<robin --> [flying]>)";
                "(&&,<robin --> flyer>,<(*,robin,worms) --> food>)", ["<robin --> flyer>", "<worms --> (/,food,robin,_)>"] => "(&&,<robin --> flyer>,<worms --> (/,food,robin,_)>)";
                "(&&,<robin --> flyer>,<robin --> [chirping]>)", ["<robin --> flyer>", "<robin --> bird>"] => "(&&,<robin --> bird>,<robin --> flyer>)";
                "(&&,<robin --> flyer>,<robin --> [chirping]>,<(*,robin,worms) --> food>)", ["<robin --> flyer>", "<(*,robin,worms) --> food>"] => "(&&,<robin --> flyer>,<(*,robin,worms) --> food>)";
                "(&&,<robin --> flyer>,<robin --> [chirping]>,<(*,robin,worms) --> food>)", ["<robin --> flyer>", "<robin --> bird>", "<(*,robin,worms) --> food>"] => "(&&,<robin --> bird>,<robin --> flyer>,<(*,robin,worms) --> food>)";
                "(&&,<robin --> flyer>,<robin --> [chirping]>,<worms --> (/,food,robin,_)>)", ["<robin --> flyer>", "<robin --> bird>", "<worms --> (/,food,robin,_)>"] => "(&&,<robin --> bird>,<robin --> flyer>,<worms --> (/,food,robin,_)>)";
                "(&&,<robin --> flyer>,<robin --> [chirping]>,<worms --> (/,food,robin,_)>)", ["<robin --> flyer>", "<worms --> (/,food,robin,_)>"] => "(&&,<robin --> flyer>,<worms --> (/,food,robin,_)>)";
                "(&&,<robin --> flyer>,<worms --> (/,food,robin,_)>)", ["<robin --> flyer>", "<(*,robin,worms) --> food>"] => "(&&,<robin --> flyer>,<(*,robin,worms) --> food>)";
                "(&,(/,neutralization,_,(\\,neutralization,acid,_)),(/,reaction,_,base))", ["(/,neutralization,_,(\\,neutralization,acid,_))", "acid"] => "(&,acid,(/,neutralization,_,(\\,neutralization,acid,_)))";
                "(&,(/,neutralization,_,(\\,neutralization,acid,_)),(/,reaction,_,base))", ["acid", "(/,reaction,_,base)"] => "(&,acid,(/,reaction,_,base))";
                "(&,(/,neutralization,_,base),(/,reaction,_,soda))", ["(/,neutralization,_,base)", "(/,reaction,_,base)"] => "(&,(/,neutralization,_,base),(/,reaction,_,base))";
                "(&,(/,neutralization,_,base),(/,reaction,_,soda))", ["(/,neutralization,_,base)", "acid"] => "(&,acid,(/,neutralization,_,base))";
                "(&,(/,neutralization,_,soda),(/,reaction,_,base))", ["acid", "(/,reaction,_,base)"] => "(&,acid,(/,reaction,_,base))";
                "(&,(/,open,_,lock),(/,open,_,{lock1}))", ["(/,open,_,lock)", "key"] => "(&,key,(/,open,_,lock))";
                "(&,(\\,REPRESENT,_,CAT),(\\,(\\,REPRESENT,_,<(*,CAT,FISH) --> FOOD>),_,eat,fish))", ["(\\,REPRESENT,_,CAT)", "(/,(/,REPRESENT,_,<(*,CAT,FISH) --> FOOD>),_,eat,fish)"] => "(&,(\\,REPRESENT,_,CAT),(/,(/,REPRESENT,_,<(*,CAT,FISH) --> FOOD>),_,eat,fish))";
                "(&,(\\,REPRESENT,_,CAT),(\\,(\\,REPRESENT,_,<(*,CAT,FISH) --> FOOD>),_,eat,fish))", ["cat", "(\\,(\\,REPRESENT,_,<(*,CAT,FISH) --> FOOD>),_,eat,fish)"] => "(&,cat,(\\,(\\,REPRESENT,_,<(*,CAT,FISH) --> FOOD>),_,eat,fish))";
                "(&,(\\,reaction,_,soda),(|,acid,(\\,reaction,_,base)))", ["(\\,reaction,_,soda)", "(\\,neutralization,_,base)"] => "(&,(\\,neutralization,_,base),(\\,reaction,_,soda))";
                "(&,(|,bird,flyer),(|,bird,{Birdie}))", ["(|,bird,{Tweety})", "(|,bird,{Birdie})"] => "(&,(|,bird,{Birdie}),(|,bird,{Tweety}))";
                "(&,(|,bird,flyer),(|,bird,{Birdie}))", ["{Tweety}", "(|,bird,{Birdie})"] => "(&,{Tweety},(|,bird,{Birdie}))";
                "(&,[with_wings],{Birdie})", ["(&,robin,{Tweety})", "{Birdie}"] => "(&,robin,{Birdie},{Tweety})";
                "(&,[with_wings],{Birdie})", ["flyer", "{Birdie}"] => "(&,flyer,{Birdie})";
                "(&,[with_wings],{Birdie})", ["{Tweety}", "{Birdie}"] => None;
                "(&,acid,(/,neutralization,_,soda))", ["acid", "(/,reaction,_,base)"] => "(&,acid,(/,reaction,_,base))";
                "(&,acid,(\\,reaction,_,base))", ["acid", "(\\,neutralization,_,base)"] => "(&,acid,(\\,neutralization,_,base))";
                "(&,animal,(|,animal,swimmer))", ["animal", "gull"] => "(&,animal,gull)";
                "(&,animal,(|,bird,swimmer))", ["animal", "(&,robin,swan)"] => "(&,animal,robin,swan)";
                "(&,animal,gull)", ["animal", "(|,animal,swimmer)"] => "(&,animal,(|,animal,swimmer))";
                "(&,animal,gull)", ["animal", "swan"] => "(&,animal,swan)";
                "(&,base,(\\,reaction,acid,_))", ["base", "(/,reaction,acid,_)"] => "(&,base,(/,reaction,acid,_))";
                "(&,base,(\\,reaction,acid,_))", ["base", "soda"] => "(&,base,soda)";
                "(&,bird,[with_wings],{Birdie},(|,[yellow],{Birdie}))", ["bird", "robin", "{Birdie}", "(|,[yellow],{Birdie})"] => "(&,bird,robin,{Birdie},(|,[yellow],{Birdie}))";
                "(&,flyer,[with_wings])", ["flyer", "(&,robin,{Tweety})"] => "(&,flyer,robin,{Tweety})";
                "(&,flyer,[with_wings])", ["flyer", "robin"] => "(&,flyer,robin)";
                "(&,flyer,[with_wings])", ["flyer", "{Birdie}"] => "(&,flyer,{Birdie})";
                "(&,flyer,[yellow],(|,[with_wings],{Birdie}))", ["flyer", "{Birdie}", "(|,[with_wings],{Birdie})"] => "(&,flyer,{Birdie},(|,[with_wings],{Birdie}))";
                "(&,flyer,{Birdie})", ["flyer", "[with_wings]"] => "(&,flyer,[with_wings])";
                "(&,flyer,{Birdie})", ["flyer", "bird"] => "(&,bird,flyer)";
                "(&,flyer,{Birdie})", ["flyer", "{Tweety}"] => "(&,flyer,{Tweety})";
                "(&,key,(/,open,_,lock))", ["key", "(/,open,_,{lock1})"] => "(&,key,(/,open,_,{lock1}))";
                "(&,key,(/,open,_,lock))", ["key", "{key1}"] => "(&,key,{key1})";
                "(&,neutralization,(*,(\\,reaction,_,soda),base))", ["neutralization", "(*,(\\,neutralization,_,base),base)"] => "(&,neutralization,(*,(\\,neutralization,_,base),base))";
                "(&,neutralization,(*,(\\,reaction,_,soda),base))", ["neutralization", "reaction"] => "(&,neutralization,reaction)";
                "(&,neutralization,(*,acid,(\\,neutralization,acid,_)))", ["neutralization", "(*,acid,(/,reaction,acid,_))"] => "(&,neutralization,(*,acid,(/,reaction,acid,_)))";
                "(&,neutralization,(*,acid,(\\,neutralization,acid,_)))", ["neutralization", "(*,acid,soda)"] => "(&,neutralization,(*,acid,soda))";
                "(&,neutralization,(*,acid,soda))", ["neutralization", "(*,acid,base)"] => "(&,neutralization,(*,acid,base))";
                "(&,neutralization,(*,acid,soda))", ["neutralization", "reaction"] => "(&,neutralization,reaction)";
                "(&,num,(/,(*,0),_))", ["num", "(/,num,_)"] => "(&,num,(/,num,_))";
                "(&,tiger,(|,bird,robin))", ["bird", "(|,bird,robin)"] => "(&,bird,(|,bird,robin))";
                "(&,{Birdie},(|,flyer,[yellow]))", ["{Birdie}", "(|,flyer,{Tweety})"] => "(&,{Birdie},(|,flyer,{Tweety}))";
                "(&,{Birdie},(|,flyer,[yellow]))", ["{Birdie}", "{Tweety}"] => None;
                "(&,{key1},(/,open,_,lock))", ["(/,open,_,{lock1})", "(/,open,_,lock)"] => "(&,(/,open,_,lock),(/,open,_,{lock1}))";
                "(&,{key1},(/,open,_,lock))", ["(|,(/,open,_,lock1),(/,open,_,{lock1}))", "(/,open,_,lock)"] => "(&,(/,open,_,lock),(|,(/,open,_,lock1),(/,open,_,{lock1})))";
                "(&,{key1},(/,open,_,lock))", ["(|,key,(/,open,_,{lock1}))", "(/,open,_,lock)"] => "(&,(/,open,_,lock),(|,key,(/,open,_,{lock1})))";
                "(&,{key1},(/,open,_,lock))", ["key", "(/,open,_,lock)"] => "(&,key,(/,open,_,lock))";
                "(*,(*,(*,0)))", ["(*,(*,(/,num,_)))"] => "(*,(*,(*,(/,num,_))))";
                "(*,(*,0))", ["(*,(/,num,_))"] => "(*,(*,(/,num,_)))";
                "(*,(*,0))", ["(*,num)"] => "(*,(*,num))";
                "(*,(*,CAT,eat,fish),<(*,CAT,FISH) --> FOOD>)", ["(/,REPRESENT,_,<(*,CAT,FISH) --> FOOD>)", "<(*,CAT,FISH) --> FOOD>"] => "(*,(/,REPRESENT,_,<(*,CAT,FISH) --> FOOD>),<(*,CAT,FISH) --> FOOD>)";
                "(*,(/,(*,0),_))", ["(/,num,_)"] => "(*,(/,num,_))";
                "(*,(/,(/,num,_),_))", ["(/,num,_)"] => "(*,(/,num,_))";
                "(*,(/,num,_))", ["(/,(/,num,_),_)"] => "(*,(/,(/,num,_),_))";
                "(*,(/,num,_))", ["0"] => "(*,0)";
                "(*,(/,open,_,lock1),lock1)", ["{key1}", "lock1"] => "(*,{key1},lock1)";
                "(*,(\\,reaction,_,base),base)", ["(\\,neutralization,_,base)", "base"] => "(*,(\\,neutralization,_,base),base)";
                "(*,(\\,reaction,_,soda),base)", ["acid", "base"] => "(*,acid,base)";
                "(*,(|,key,(/,open,_,{lock1})),lock)", ["(/,open,_,lock)", "lock"] => "(*,(/,open,_,lock),lock)";
                "(*,0)", ["(&,num,(/,(*,(/,num,_)),_))"] => "(*,(&,num,(/,(*,(/,num,_)),_)))";
                "(*,0)", ["(/,(*,(/,num,_)),_)"] => "(*,(/,(*,(/,num,_)),_))";
                "(*,0)", ["(/,num,_)"] => "(*,(/,num,_))";
                "(*,0)", ["num"] => "(*,num)";
                "(*,a,(/,like,_,a))", ["a", "b"] => "(*,a,b)";
                "(*,a,b)", ["(/,like,b,_)", "b"] => "(*,(/,like,b,_),b)";
                "(*,a,b)", ["a", "(/,like,_,a)"] => "(*,a,(/,like,_,a))";
                "(*,acid,(&,soda,(/,neutralization,acid,_)))", ["acid", "(/,reaction,acid,_)"] => "(*,acid,(/,reaction,acid,_))";
                "(*,acid,(/,reaction,acid,_))", ["acid", "(&,soda,(/,neutralization,acid,_))"] => "(*,acid,(&,soda,(/,neutralization,acid,_)))";
                "(*,acid,(/,reaction,acid,_))", ["acid", "(/,neutralization,acid,_)"] => "(*,acid,(/,neutralization,acid,_))";
                "(*,acid,(/,reaction,acid,_))", ["acid", "(\\,neutralization,acid,_)"] => "(*,acid,(\\,neutralization,acid,_))";
                "(*,acid,(/,reaction,acid,_))", ["acid", "(|,base,(\\,reaction,acid,_))"] => "(*,acid,(|,base,(\\,reaction,acid,_)))";
                "(*,acid,base)", ["acid", "(\\,neutralization,acid,_)"] => "(*,acid,(\\,neutralization,acid,_))";
                "(*,acid,base)", ["acid", "soda"] => "(*,acid,soda)";
                "(*,{key1},lock)", ["(&,key,(/,open,_,{lock1}))", "lock"] => "(*,(&,key,(/,open,_,{lock1})),lock)";
                "(*,{key1},lock)", ["(/,open,_,{lock1})", "lock"] => "(*,(/,open,_,{lock1}),lock)";
                "(*,{key1},lock)", ["(|,key,(/,open,_,{lock1}))", "lock"] => "(*,(|,key,(/,open,_,{lock1})),lock)";
                "(*,{key1},lock)", ["key", "lock"] => "(*,key,lock)";
                "(*,{key1},lock1)", ["(/,open,_,lock)", "lock1"] => "(*,(/,open,_,lock),lock1)";
                "(*,{key1},lock1)", ["(|,(/,open,_,lock1),(/,open,_,{lock1}))", "lock1"] => "(*,(|,(/,open,_,lock1),(/,open,_,{lock1})),lock1)";
                "(*,{key1},{lock1})", ["(/,open,_,lock)", "{lock1}"] => "(*,(/,open,_,lock),{lock1})";
                "(*,{key1},{lock1})", ["(/,open,_,{lock1})", "{lock1}"] => "(*,(/,open,_,{lock1}),{lock1})";
                "(*,{key1},{lock1})", ["key", "{lock1}"] => "(*,key,{lock1})";
                "(/,(*,(/,num,_)),_)", ["(*,num)"] => "(/,(*,num),_)";
                "(/,(*,b,(/,like,b,_)),_,a)", ["(*,b,a)", "a"] => "(/,(*,b,a),_,a)";
                "(/,(*,num),_)", ["(*,0)"] => "(/,(*,0),_)";
                "(/,(*,tim,tom),tom,_)", ["tom", "uncle"] => "(/,uncle,tom,_)";
                "(/,(/,num,_),_)", ["0"] => "(/,0,_)";
                "(/,0,_)", ["(&,num,(/,(*,(/,num,_)),_))"] => "(/,(&,num,(/,(*,(/,num,_)),_)),_)";
                "(/,0,_)", ["(/,num,_)"] => "(/,(/,num,_),_)";
                "(/,0,_)", ["num"] => "(/,num,_)";
                "(/,like,_,a)", ["like", "(/,like,b,_)"] => "(/,like,_,(/,like,b,_))";
                "(/,like,b,_)", ["(/,like,_,a)", "like"] => "(/,like,(/,like,_,a),_)";
                "(/,neutralization,_,base)", ["neutralization", "(\\,neutralization,acid,_)"] => "(/,neutralization,_,(\\,neutralization,acid,_))";
                "(/,neutralization,_,base)", ["neutralization", "soda"] => "(/,neutralization,_,soda)";
                "(/,num,_)", ["(*,0)"] => "(/,(*,0),_)";
                "(/,open,_,(|,lock,(/,open,{key1},_)))", ["open", "{lock1}"] => "(/,open,_,{lock1})";
                "(/,open,_,{lock1})", ["open", "(|,lock,(/,open,{key1},_))"] => "(/,open,_,(|,lock,(/,open,{key1},_)))";
                "(/,open,_,{lock1})", ["open", "lock"] => "(/,open,_,lock)";
                "(/,reaction,_,base)", ["(*,acid,soda)", "base"] => "(/,(*,acid,soda),_,base)";
                "(/,reaction,acid,_)", ["acid", "(*,acid,soda)"] => "(/,(*,acid,soda),acid,_)";
                "(\\,(*,b,a),_,(/,like,b,_))", ["like", "(/,like,b,_)"] => "(\\,like,_,(/,like,b,_))";
                "(\\,REPRESENT,_,CAT)", ["REPRESENT", "(\\,REPRESENT,_,CAT)"] => "(\\,REPRESENT,_,(\\,REPRESENT,_,CAT))";
                "(\\,neutralization,_,(/,neutralization,acid,_))", ["neutralization", "soda"] => "(\\,neutralization,_,soda)";
                "(\\,neutralization,_,(/,reaction,acid,_))", ["neutralization", "(/,neutralization,acid,_)"] => "(\\,neutralization,_,(/,neutralization,acid,_))";
                "(\\,neutralization,_,(/,reaction,acid,_))", ["neutralization", "(\\,neutralization,acid,_)"] => "(\\,neutralization,_,(\\,neutralization,acid,_))";
                "(\\,neutralization,_,(/,reaction,acid,_))", ["neutralization", "(|,base,(\\,reaction,acid,_))"] => "(\\,neutralization,_,(|,base,(\\,reaction,acid,_)))";
                "(\\,neutralization,_,base)", ["neutralization", "(/,neutralization,acid,_)"] => "(\\,neutralization,_,(/,neutralization,acid,_))";
                "(\\,neutralization,_,base)", ["neutralization", "soda"] => "(\\,neutralization,_,soda)";
                "(\\,neutralization,acid,_)", ["(\\,reaction,_,base)", "neutralization"] => "(\\,neutralization,(\\,reaction,_,base),_)";
                "(\\,reaction,(\\,reaction,_,soda),_)", ["(\\,reaction,_,base)", "reaction"] => "(\\,reaction,(\\,reaction,_,base),_)";
                "(\\,reaction,_,base)", ["(*,acid,soda)", "base"] => "(\\,(*,acid,soda),_,base)";
                "(\\,reaction,acid,_)", ["acid", "(*,acid,soda)"] => "(\\,(*,acid,soda),acid,_)";
                "(|,(&,animal,gull),(&,bird,robin))", ["(&,animal,gull)", "swimmer"] => "(|,swimmer,(&,animal,gull))";
                "(|,(&,flyer,{Birdie}),{Birdie,Tweety})", ["(&,flyer,{Birdie})", "(|,[yellow],{Birdie})"] => "(|,[yellow],{Birdie},(&,flyer,{Birdie}))";
                "(|,(/,neutralization,_,(\\,neutralization,acid,_)),(/,reaction,_,base))", ["(/,neutralization,_,base)", "(/,reaction,_,base)"] => "(|,(/,neutralization,_,base),(/,reaction,_,base))";
                "(|,(/,neutralization,_,(\\,neutralization,acid,_)),(/,reaction,_,base))", ["acid", "(/,reaction,_,base)"] => "(|,acid,(/,reaction,_,base))";
                "(|,(/,neutralization,_,base),(/,reaction,_,base))", ["(/,neutralization,_,base)", "acid"] => "(|,acid,(/,neutralization,_,base))";
                "(|,(/,neutralization,_,base),(/,reaction,_,soda))", ["(/,neutralization,_,base)", "(/,neutralization,_,(\\,neutralization,acid,_))"] => "(|,(/,neutralization,_,base),(/,neutralization,_,(\\,neutralization,acid,_)))";
                "(|,(/,neutralization,_,base),(/,reaction,_,soda))", ["(/,neutralization,_,base)", "(/,reaction,_,base)"] => "(|,(/,neutralization,_,base),(/,reaction,_,base))";
                "(|,(/,neutralization,_,soda),(/,reaction,_,base))", ["acid", "(/,reaction,_,base)"] => "(|,acid,(/,reaction,_,base))";
                "(|,(/,num,_),(/,(*,num),_))", ["(/,num,_)", "0"] => "(|,0,(/,num,_))";
                "(|,(\\,REPRESENT,_,CAT),(\\,(\\,REPRESENT,_,<(*,CAT,FISH) --> FOOD>),_,eat,fish))", ["(\\,REPRESENT,_,CAT)", "(/,(/,REPRESENT,_,<(*,CAT,FISH) --> FOOD>),_,eat,fish)"] => "(|,(\\,REPRESENT,_,CAT),(/,(/,REPRESENT,_,<(*,CAT,FISH) --> FOOD>),_,eat,fish))";
                "(|,(\\,REPRESENT,_,CAT),(\\,(\\,REPRESENT,_,<(*,CAT,FISH) --> FOOD>),_,eat,fish))", ["cat", "(\\,(\\,REPRESENT,_,<(*,CAT,FISH) --> FOOD>),_,eat,fish)"] => "(|,cat,(\\,(\\,REPRESENT,_,<(*,CAT,FISH) --> FOOD>),_,eat,fish))";
                "(|,CAT,(/,(/,REPRESENT,_,<(*,CAT,FISH) --> FOOD>),_,eat,fish))", ["(\\,(\\,REPRESENT,_,<(*,CAT,FISH) --> FOOD>),_,eat,fish)", "(/,(/,REPRESENT,_,<(*,CAT,FISH) --> FOOD>),_,eat,fish)"] => "(|,(/,(/,REPRESENT,_,<(*,CAT,FISH) --> FOOD>),_,eat,fish),(\\,(\\,REPRESENT,_,<(*,CAT,FISH) --> FOOD>),_,eat,fish))";
                "(|,[strong],(~,youth,girl))", ["(~,boy,girl)", "(~,youth,girl)"] => "(|,(~,boy,girl),(~,youth,girl))";
                "(|,[strong],(~,youth,girl))", ["boy", "(~,youth,girl)"] => "(|,boy,(~,youth,girl))";
                "(|,X,Y)", ["[with_wings]", "(|,flyer,{Tweety})", "{Birdie}"] => "(|,flyer,[with_wings],{Birdie},{Tweety})"; // ! 📌【2024-09-07 14:17:33】为避免左侧词项被自动约简，将「模板词项」简化
                "(|,X,Y)", ["[with_wings]", "flyer", "{Birdie}"] => "(|,flyer,[with_wings],{Birdie})";                       // ! 📌【2024-09-07 14:17:33】为避免左侧词项被自动约简，将「模板词项」简化
                "(|,X,Y)", ["[with_wings]", "{Tweety}", "{Birdie}"] => "(|,[with_wings],{Birdie},{Tweety})";                 // ! 📌【2024-09-07 14:17:33】为避免左侧词项被自动约简，将「模板词项」简化
                "(|,X,Y)", ["flyer", "[yellow]", "{Birdie}"] => "(|,flyer,[yellow],{Birdie})";                               // ! 📌【2024-09-07 14:17:33】为避免左侧词项被自动约简，将「模板词项」简化
                "(|,[with_wings],{Birdie})", ["flyer", "{Birdie}"] => "(|,flyer,{Birdie})";
                "(|,[with_wings],{Birdie})", ["{Tweety}", "{Birdie}"] => "{Birdie,Tweety}";
                "(|,[with_wings],{Birdie},(&,bird,(|,[yellow],{Birdie})))", ["flyer", "{Birdie}", "(&,bird,(|,[yellow],{Birdie}))"] => "(|,flyer,{Birdie},(&,bird,(|,[yellow],{Birdie})))";
                "(|,[with_wings],{Birdie},(&,flyer,[yellow]))", ["[with_wings]", "{Birdie}", "(|,[with_wings],{Birdie})"] => "(|,[with_wings],{Birdie})";
                "(|,[yellow],{Birdie})", ["(&,flyer,{Birdie})", "{Birdie}"] => "(|,{Birdie},(&,flyer,{Birdie}))";
                "(|,[yellow],{Birdie})", ["[yellow]", "[with_wings]"] => None;
                "(|,[yellow],{Birdie})", ["[yellow]", "bird"] => "(|,bird,[yellow])";
                "(|,[yellow],{Birdie})", ["[yellow]", "{Tweety}"] => "(|,[yellow],{Tweety})";
                "(|,[yellow],{Birdie},(&,flyer,{Birdie}))", ["flyer", "{Birdie}", "(&,flyer,{Birdie})"] => "(|,flyer,{Birdie},(&,flyer,{Birdie}))";
                "(|,[yellow],{Birdie},(&,flyer,{Birdie}))", ["{Tweety}", "{Birdie}", "(&,flyer,{Birdie})"] => "(|,(&,flyer,{Birdie}),{Birdie,Tweety})";
                "(|,acid,(/,neutralization,_,soda))", ["acid", "(/,reaction,_,base)"] => "(|,acid,(/,reaction,_,base))";
                "(|,acid,(\\,reaction,_,base))", ["acid", "(\\,neutralization,_,base)"] => "(|,acid,(\\,neutralization,_,base))";
                "(|,animal,gull)", ["animal", "robin"] => "(|,animal,robin)";
                "(|,base,(\\,reaction,acid,_))", ["base", "(/,reaction,acid,_)"] => "(|,base,(/,reaction,acid,_))";
                "(|,base,(\\,reaction,acid,_))", ["base", "soda"] => "(|,base,soda)";
                "(|,bird,(&,robin,tiger))", ["bird", "animal"] => "(|,animal,bird)";
                "(|,bird,[yellow])", ["bird", "flyer"] => "(|,bird,flyer)";
                "(|,bird,[yellow])", ["bird", "{Birdie}"] => "(|,bird,{Birdie})";
                "(|,bird,[yellow],{Birdie})", ["bird", "flyer", "{Birdie}"] => "(|,bird,flyer,{Birdie})";
                "(|,bird,[yellow],{Birdie})", ["bird", "{Tweety}", "{Birdie}"] => "(|,bird,{Birdie},{Tweety})";
                "(|,bird,{Birdie})", ["bird", "[with_wings]"] => "(|,bird,[with_wings])";
                "(|,bird,{Birdie})", ["bird", "flyer"] => "(|,bird,flyer)";
                "(|,bird,{Birdie})", ["bird", "{Tweety}"] => "(|,bird,{Tweety})";
                "(|,bird,{Tweety})", ["bird", "(|,bird,flyer)"] => "(|,bird,flyer)";
                "(|,chess,competition)", ["chess", "(|,chess,sport)"] => "(|,chess,sport)";
                "(|,flyer,[yellow])", ["flyer", "(&,flyer,{Birdie})"] => "(|,flyer,(&,flyer,{Birdie}))";
                "(|,flyer,[yellow],(&,flyer,{Birdie}))", ["flyer", "{Birdie}", "(&,flyer,{Birdie})"] => "(|,flyer,{Birdie},(&,flyer,{Birdie}))";
                "(|,flyer,[yellow],{Birdie})", ["flyer", "(&,flyer,{Birdie})", "{Birdie}"] => "(|,flyer,{Birdie},(&,flyer,{Birdie}))";
                "(|,flyer,[yellow],{Birdie})", ["flyer", "(|,flyer,{Tweety})", "{Birdie}"] => "(|,flyer,{Birdie},{Tweety})";
                "(|,key,(/,open,_,lock))", ["key", "(/,open,_,{lock1})"] => "(|,key,(/,open,_,{lock1}))";
                "(|,key,(/,open,_,lock))", ["key", "{key1}"] => "(|,key,{key1})";
                "(|,neutralization,(*,(\\,reaction,_,soda),base))", ["neutralization", "reaction"] => "(|,neutralization,reaction)";
                "(|,neutralization,(*,acid,soda))", ["neutralization", "(*,acid,(\\,neutralization,acid,_))"] => "(|,neutralization,(*,acid,(\\,neutralization,acid,_)))";
                "(|,neutralization,(*,acid,soda))", ["neutralization", "(*,acid,base)"] => "(|,neutralization,(*,acid,base))";
                "(|,neutralization,(*,acid,soda))", ["neutralization", "reaction"] => "(|,neutralization,reaction)";
                "(|,robin,[yellow],{Birdie})", ["robin", "(|,flyer,{Tweety})", "{Birdie}"] => "(|,flyer,robin,{Birdie},{Tweety})";
                "(|,soda,(\\,neutralization,acid,_))", ["(/,neutralization,acid,_)", "(\\,neutralization,acid,_)"] => "(|,(/,neutralization,acid,_),(\\,neutralization,acid,_))";
                "(|,tiger,(&,bird,robin))", ["tiger", "(|,animal,swimmer)"] => "(|,animal,swimmer,tiger)";
                "(|,{key1},(/,open,_,lock))", ["(/,open,_,{lock1})", "(/,open,_,lock)"] => "(|,(/,open,_,lock),(/,open,_,{lock1}))";
                "(|,{key1},(/,open,_,lock))", ["(|,key,(/,open,_,{lock1}))", "(/,open,_,lock)"] => "(|,key,(/,open,_,lock),(/,open,_,{lock1}))";
                "(|,{key1},(/,open,_,lock))", ["key", "(/,open,_,lock)"] => "(|,key,(/,open,_,lock))";
                "(~,(/,(*,tim,tom),tom,_),tim)", ["(/,(*,tim,tom),tom,_)", "(/,uncle,tom,_)"] => "(~,(/,(*,tim,tom),tom,_),(/,uncle,tom,_))";
                "(~,[strong],girl)", ["(~,boy,girl)", "girl"] => "(~,(~,boy,girl),girl)";
                "(~,[strong],girl)", ["boy", "girl"] => "(~,boy,girl)";
                "(~,boy,girl)", ["[strong]", "girl"] => "(~,[strong],girl)";
                "(~,boy,girl)", ["youth", "girl"] => "(~,youth,girl)";
                "(~,youth,girl)", ["(|,boy,girl)", "girl"] => "(~,(|,boy,girl),girl)";
                "[bright]", ["smart"] => "[smart]";
                "{Birdie}", ["Tweety"] => "{Tweety}";
                "{Mars,Pluto,Saturn,Venus}", ["Mars", "Venus"] => "{Mars,Venus}";
            }
            ok!()
        }
    }

    mod statement {
        use super::*;

        #[test]
        fn make_statement_relation() -> AResult {
            fn test(relation: &str, subject: Term, predicate: Term, expected: Option<Term>) {
                let out =
                    Term::make_statement_relation(relation, subject.clone(), predicate.clone());
                assert_eq!(
                    out,
                    expected,
                    "\"{relation}\", \"{subject}\", \"{predicate}\" => {} != {}",
                    format_option_term(&out),
                    format_option_term(&expected),
                );
            }
            macro_once! {
                // * 🚩模式：参数列表 ⇒ 预期词项
                macro test($($relation:tt, $subject:tt, $predicate:tt => $expected:tt;)*) {
                    $( test($relation, term!($subject), term!($predicate), option_term!($expected)); )*
                }
                // * ℹ️用例均源自OpenNARS实际运行
                "-->", "(&,<bird --> fly>,<{Tweety} --> bird>)", "claimedByBob" => "<(&,<bird --> fly>,<{Tweety} --> bird>) --> claimedByBob>";
                "-->", "(&,bird,swimmer)", "(&,animal,swimmer)" => "<(&,bird,swimmer) --> (&,animal,swimmer)>";
                "-->", "(&,swan,swimmer)", "bird" => "<(&,swan,swimmer) --> bird>";
                "-->", "(*,(*,(*,0)))", "num" => "<(*,(*,(*,0))) --> num>";
                "-->", "(*,CAT,FISH)", "FOOD" => "<(*,CAT,FISH) --> FOOD>";
                "-->", "(*,bird,plant)", "?120" => "<(*,bird,plant) --> ?120>";
                "-->", "(-,swimmer,animal)", "(-,swimmer,bird)" => "<(-,swimmer,animal) --> (-,swimmer,bird)>";
                "-->", "(/,neutralization,_,base)", "?120" => "<(/,neutralization,_,base) --> ?120>";
                "-->", "(|,boy,girl)", "youth" => "<(|,boy,girl) --> youth>";
                "-->", "(~,boy,girl)", "[strong]" => "<(~,boy,girl) --> [strong]>";
                "-->", "(~,swimmer,swan)", "bird" => "<(~,swimmer,swan) --> bird>";
                "-->", "0", "(/,num,_)" => "<0 --> (/,num,_)>";
                "-->", "0", "num" => "<0 --> num>";
                "-->", "?120", "claimedByBob" => "<?120 --> claimedByBob>";
                "-->", "[smart]", "[bright]" => "<[smart] --> [bright]>";
                "-->", "acid", "(/,reaction,_,base)" => "<acid --> (/,reaction,_,base)>";
                "-->", "cat", "(/,(/,REPRESENT,_,<(*,CAT,FISH) --> FOOD>),_,eat,fish)" => "<cat --> (/,(/,REPRESENT,_,<(*,CAT,FISH) --> FOOD>),_,eat,fish)>";
                "-->", "neutralization", "(*,acid,base)" => "<neutralization --> (*,acid,base)>";
                "-->", "planetX", "{Mars,Pluto,Venus}" => "<planetX --> {Mars,Pluto,Venus}>";
                "-->", "planetX", "{Pluto,Saturn}" => "<planetX --> {Pluto,Saturn}>";
                "-->", "robin", "(&,bird,swimmer)" => "<robin --> (&,bird,swimmer)>";
                "-->", "robin", "(-,bird,swimmer)" => "<robin --> (-,bird,swimmer)>";
                "-->", "robin", "(|,bird,swimmer)" => "<robin --> (|,bird,swimmer)>";
                "-->", "robin", "[chirping]" => "<robin --> [chirping]>";
                "-->", "{?49}", "swimmer" => "<{?49} --> swimmer>";
                "-->", "{Tweety}", "[with_wings]" => "<{Tweety} --> [with_wings]>";
                "-->", "{Tweety}", "bird" => "<{Tweety} --> bird>";
                "-->", "{Tweety}", "{Birdie}" => "<{Tweety} --> {Birdie}>";
                "-->", "{key1}", "(/,open,_,{lock1})" => "<{key1} --> (/,open,_,{lock1})>";
                "--]", "raven", "black" => "<raven --> [black]>";
                "<->", "Birdie", "Tweety" => "<Birdie <-> Tweety>";
                "<->", "[bright]", "[smart]" => "<[bright] <-> [smart]>";
                "<->", "{Birdie}", "{Tweety}" => "<{Birdie} <-> {Tweety}>";
                "<=>", "<robin --> animal>", "<robin --> bird>" => "<<robin --> animal> <=> <robin --> bird>>";
                "<=>", "<robin --> bird>", "<robin --> [flying]>" => "<<robin --> bird> <=> <robin --> [flying]>>";
                "==>", "(&&,<robin --> [chirping]>,<robin --> [flying]>)", "<robin --> bird>" => "<(&&,<robin --> [chirping]>,<robin --> [flying]>) ==> <robin --> bird>>";
                "==>", "(&&,<robin --> [chirping]>,<robin --> [flying]>,<robin --> [with_wings]>)", "<robin --> bird>" => "<(&&,<robin --> [chirping]>,<robin --> [flying]>,<robin --> [with_wings]>) ==> <robin --> bird>>";
                "==>", "(&&,<robin --> [flying]>,<robin --> [with_wings]>)", "<robin --> [living]>" => "<(&&,<robin --> [flying]>,<robin --> [with_wings]>) ==> <robin --> [living]>>";
                "==>", "(&&,<robin --> bird>,<robin --> [flying]>)", "<robin --> [living]>" => "<(&&,<robin --> bird>,<robin --> [flying]>) ==> <robin --> [living]>>";
                "==>", "(&&,<robin --> bird>,<robin --> [living]>)", "<robin --> animal>" => "<(&&,<robin --> bird>,<robin --> [living]>) ==> <robin --> animal>>";
                "==>", "(--,<robin --> [flying]>)", "<robin --> bird>" => "<(--,<robin --> [flying]>) ==> <robin --> bird>>";
                "==>", "(--,<robin --> bird>)", "<robin --> [flying]>" => "<(--,<robin --> bird>) ==> <robin --> [flying]>>";
                "==>", "<robin --> [flying]>", "<robin --> [with_beak]>" => "<<robin --> [flying]> ==> <robin --> [with_beak]>>";
                "==>", "<robin --> [flying]>", "<robin --> animal>" => "<<robin --> [flying]> ==> <robin --> animal>>";
                "==>", "<robin --> bird>", "(&&,<robin --> animal>,<robin --> [flying]>)" => "<<robin --> bird> ==> (&&,<robin --> animal>,<robin --> [flying]>)>";
                "==>", "<robin --> bird>", "<robin --> [flying]>" => "<<robin --> bird> ==> <robin --> [flying]>>";
                "==>", "<robin --> bird>", "<robin --> animal>" => "<<robin --> bird> ==> <robin --> animal>>";
                "{--", "Tweety", "bird" => "<{Tweety} --> bird>";
                "{-]", "Tweety", "yellow" => "<{Tweety} --> [yellow]>";
            }
            ok!()
        }

        #[test]
        fn make_statement() -> AResult {
            fn test(template: Term, subject: Term, predicate: Term, expected: Option<Term>) {
                let out = Term::make_statement(&template, subject.clone(), predicate.clone());
                assert_eq!(
                    out,
                    expected,
                    "\"{template}\", \"{subject}\", \"{predicate}\" => {} != {}",
                    format_option_term(&out),
                    format_option_term(&expected),
                );
            }
            macro_once! {
                // * 🚩模式：参数列表 ⇒ 预期词项
                macro test($($template:tt, $subject:tt, $predicate:tt => $expected:tt;)*) {
                    $( test(term!($template), term!($subject), term!($predicate), option_term!($expected)); )*
                }
                // * ℹ️用例均源自OpenNARS实际运行
                "<(&&,<robin --> [chirping]>,<robin --> [flying]>) ==> <robin --> bird>>", "(&&,<robin --> bird>,<robin --> [flying]>)", "<robin --> bird>" => None;"<(&&,<robin --> [chirping]>,<robin --> [flying]>) ==> <robin --> bird>>", "<robin --> [chirping]>", "<robin --> bird>" => "<<robin --> [chirping]> ==> <robin --> bird>>";
                "<(&&,<robin --> [chirping]>,<robin --> [flying]>,<robin --> [with_wings]>) ==> <robin --> bird>>", "(&&,<robin --> [chirping]>,<robin --> [flying]>)", "<robin --> bird>" => "<(&&,<robin --> [chirping]>,<robin --> [flying]>) ==> <robin --> bird>>";
                "<(&&,<robin --> [chirping]>,<robin --> [flying]>,<robin --> [with_wings]>) ==> <robin --> bird>>", "(&&,<robin --> bird>,<robin --> [flying]>,<robin --> [with_wings]>)", "<robin --> bird>" => None;
                "<(&&,<robin --> [chirping]>,<robin --> [with_wings]>) ==> <robin --> bird>>", "(&&,<robin --> bird>,<robin --> [chirping]>)", "<robin --> bird>" => None;
                "<(&&,<robin --> [flying]>,<robin --> [with_wings]>) ==> <robin --> [living]>>", "<robin --> [flying]>", "<robin --> [living]>" => "<<robin --> [flying]> ==> <robin --> [living]>>";
                "<(&&,<robin --> [flying]>,<robin --> [with_wings]>) ==> <robin --> [living]>>", "<robin --> [with_wings]>", "<robin --> bird>" => "<<robin --> [with_wings]> ==> <robin --> bird>>";
                "<(&&,<robin --> [flying]>,<robin --> [with_wings]>) ==> <robin --> animal>>", "(&&,<robin --> [flying]>,<robin --> [with_wings]>)", "(&&,<robin --> animal>,<robin --> bird>)" => "<(&&,<robin --> [flying]>,<robin --> [with_wings]>) ==> (&&,<robin --> animal>,<robin --> bird>)>";
                "<(&&,<robin --> [flying]>,<robin --> [with_wings]>) ==> <robin --> animal>>", "(&&,<robin --> [flying]>,<robin --> [with_wings]>)", "(||,<robin --> animal>,<robin --> bird>)" => "<(&&,<robin --> [flying]>,<robin --> [with_wings]>) ==> (||,<robin --> animal>,<robin --> bird>)>";
                "<(&&,<robin --> [flying]>,<robin --> [with_wings]>) ==> <robin --> animal>>", "<robin --> animal>", "<robin --> bird>" => "<<robin --> animal> ==> <robin --> bird>>";
                "<(&&,<robin --> bird>,<robin --> [flying]>) ==> <robin --> [living]>>", "<robin --> [flying]>", "<robin --> [living]>" => "<<robin --> [flying]> ==> <robin --> [living]>>";
                "<(&&,<robin --> bird>,<robin --> [flying]>) ==> <robin --> [living]>>", "<robin --> bird>", "<robin --> [living]>" => "<<robin --> bird> ==> <robin --> [living]>>";
                "<(&&,<robin --> bird>,<robin --> [flying]>) ==> <robin --> animal>>", "<robin --> [flying]>", "<robin --> animal>" => "<<robin --> [flying]> ==> <robin --> animal>>";
                "<(&&,<robin --> bird>,<robin --> [flying]>) ==> <robin --> animal>>", "<robin --> bird>", "<robin --> animal>" => "<<robin --> bird> ==> <robin --> animal>>";
                "<(&&,<robin --> bird>,<robin --> [flying]>,<robin --> [with_wings]>) ==> <robin --> animal>>", "(&&,<robin --> [flying]>,<robin --> [with_wings]>)", "<robin --> animal>" => "<(&&,<robin --> [flying]>,<robin --> [with_wings]>) ==> <robin --> animal>>";
                "<(&&,<robin --> bird>,<robin --> [flying]>,<robin --> [with_wings]>) ==> <robin --> animal>>", "(&&,<robin --> bird>,<robin --> [flying]>)", "<robin --> animal>" => "<(&&,<robin --> bird>,<robin --> [flying]>) ==> <robin --> animal>>";
                "<(&&,<robin --> bird>,<robin --> [living]>) ==> <robin --> animal>>", "(&&,<robin --> bird>,<robin --> [flying]>)", "<robin --> animal>" => "<(&&,<robin --> bird>,<robin --> [flying]>) ==> <robin --> animal>>";
                "<(&&,<robin --> bird>,<robin --> [living]>) ==> <robin --> animal>>", "(&&,<robin --> bird>,<robin --> [flying]>,<robin --> [with_wings]>)", "<robin --> animal>" => "<(&&,<robin --> bird>,<robin --> [flying]>,<robin --> [with_wings]>) ==> <robin --> animal>>";
                "<(&&,<robin --> flyer>,<robin --> [chirping]>) ==> <robin --> bird>>", "(&&,<robin --> bird>,<robin --> flyer>)", "<robin --> bird>" => None;
                "<(&&,<robin --> flyer>,<robin --> [chirping]>,<(*,robin,worms) --> food>) ==> <robin --> bird>>", "(&&,<robin --> bird>,<robin --> flyer>,<(*,robin,worms) --> food>)", "<robin --> bird>" => None;
                "<(&&,<robin --> flyer>,<robin --> [chirping]>,<(*,robin,worms) --> food>) ==> <robin --> bird>>", "(&&,<robin --> flyer>,<(*,robin,worms) --> food>)", "<robin --> bird>" => "<(&&,<robin --> flyer>,<(*,robin,worms) --> food>) ==> <robin --> bird>>";
                "<(&&,<robin --> flyer>,<robin --> [chirping]>,<worms --> (/,food,robin,_)>) ==> <robin --> bird>>", "(&&,<robin --> bird>,<robin --> flyer>,<worms --> (/,food,robin,_)>)", "<robin --> bird>" => None;
                "<(&,bird,swimmer) --> (&,animal,swimmer)>", "bird", "animal" => "<bird --> animal>";
                "<(&,bird,swimmer) --> (&,animal,swimmer)>", "swimmer", "swimmer" => None;
                "<(&,chess,sport) --> competition>", "chess", "competition" => "<chess --> competition>";
                "<(&,robin,swan) --> (&,bird,swimmer)>", "(&,robin,swan)", "bird" => "<(&,robin,swan) --> bird>";
                "<(&,robin,swimmer) --> animal>", "(&,robin,swimmer)", "(&,animal,bird)" => "<(&,robin,swimmer) --> (&,animal,bird)>";
                "<(&,robin,swimmer) --> animal>", "(&,robin,swimmer)", "(|,animal,bird)" => "<(&,robin,swimmer) --> (|,animal,bird)>";
                "<(&,robin,{Tweety}) --> [with_wings]>", "(&,flyer,robin,{Tweety})", "(&,flyer,[with_wings])" => "<(&,flyer,robin,{Tweety}) --> (&,flyer,[with_wings])>";
                "<(&,robin,{Tweety}) --> [with_wings]>", "(&,robin,{Birdie},{Tweety})", "(&,[with_wings],{Birdie})" => "<(&,robin,{Birdie},{Tweety}) --> (&,[with_wings],{Birdie})>";
                "<(*,(*,(*,0))) --> num>", "(*,(*,(*,(/,num,_))))", "num" => "<(*,(*,(*,(/,num,_)))) --> num>";
                "<(*,(*,(*,0))) --> num>", "num", "(*,(*,(*,(/,num,_))))" => "<num --> (*,(*,(*,(/,num,_))))>";
                "<(*,(*,0)) --> (*,(*,(/,num,_)))>", "(*,(*,(*,0)))", "(*,(*,(*,(/,num,_))))" => "<(*,(*,(*,0))) --> (*,(*,(*,(/,num,_))))>";
                "<(*,(*,0)) --> (*,(*,(/,num,_)))>", "(*,(*,(/,num,_)))", "(*,(*,num))" => "<(*,(*,(/,num,_))) --> (*,(*,num))>";
                "<(*,(*,0)) --> (*,(*,(/,num,_)))>", "(*,(*,0))", "(&,(*,(*,num)),(*,(*,(/,num,_))))" => "<(*,(*,0)) --> (&,(*,(*,num)),(*,(*,(/,num,_))))>";
                "<(*,(*,0)) --> (*,(*,(/,num,_)))>", "(*,(*,0))", "(|,(*,(*,num)),(*,(*,(/,num,_))))" => "<(*,(*,0)) --> (|,(*,(*,num)),(*,(*,(/,num,_))))>";
                "<(*,(*,0)) --> (*,(*,(/,num,_)))>", "(*,(*,num))", "(*,(*,(/,num,_)))" => "<(*,(*,num)) --> (*,(*,(/,num,_)))>";
                "<(*,(*,0)) --> (*,(*,(/,num,_)))>", "(*,0)", "(*,(/,num,_))" => "<(*,0) --> (*,(/,num,_))>";
                "<(*,0) --> (*,(/,num,_))>", "(*,(*,0))", "(*,(*,(/,num,_)))" => "<(*,(*,0)) --> (*,(*,(/,num,_)))>";
                "<(*,0) --> (*,(/,num,_))>", "(*,(/,num,_))", "(*,num)" => "<(*,(/,num,_)) --> (*,num)>";
                "<(*,0) --> (*,(/,num,_))>", "(*,0)", "(&,(*,num),(*,(/,num,_)))" => "<(*,0) --> (&,(*,num),(*,(/,num,_)))>";
                "<(*,0) --> (*,(/,num,_))>", "(*,0)", "(|,(*,num),(*,(/,num,_)))" => "<(*,0) --> (|,(*,num),(*,(/,num,_)))>";
                "<(*,0) --> (*,(/,num,_))>", "(*,num)", "(*,(/,num,_))" => "<(*,num) --> (*,(/,num,_))>";
                "<(*,0) --> (*,(/,num,_))>", "0", "(/,num,_)" => "<0 --> (/,num,_)>";
                "<(*,0) --> (*,num)>", "(*,(*,0))", "(*,(*,num))" => "<(*,(*,0)) --> (*,(*,num))>";
                "<(*,0) --> (*,num)>", "(*,(/,num,_))", "(*,num)" => "<(*,(/,num,_)) --> (*,num)>";
                "<(*,0) --> (*,num)>", "(*,0)", "(&,(*,num),(*,(/,num,_)))" => "<(*,0) --> (&,(*,num),(*,(/,num,_)))>";
                "<(*,0) --> (*,num)>", "(*,0)", "(|,(*,num),(*,(/,num,_)))" => "<(*,0) --> (|,(*,num),(*,(/,num,_)))>";
                "<(*,0) --> (*,num)>", "(*,num)", "(*,(/,num,_))" => "<(*,num) --> (*,(/,num,_))>";
                "<(*,0) --> (*,num)>", "0", "num" => "<0 --> num>";
                "<(*,0) --> num>", "(/,(*,0),_)", "(/,num,_)" => "<(/,(*,0),_) --> (/,num,_)>";
                "<(*,a,b) --> (&,like,(*,(/,like,b,_),b))>", "(*,a,b)", "(&,like,(*,(/,like,b,_),b))" => "<(*,a,b) --> (&,like,(*,(/,like,b,_),b))>";
                "<(*,a,b) --> like>", "(*,(/,like,b,_),b)", "like" => "<(*,(/,like,b,_),b) --> like>";
                "<(*,a,b) --> like>", "(*,a,b)", "(&,like,(*,(/,like,b,_),b))" => "<(*,a,b) --> (&,like,(*,(/,like,b,_),b))>";
                "<(*,a,b) --> like>", "(*,a,b)", "(|,like,(*,(/,like,b,_),b))" => "<(*,a,b) --> (|,like,(*,(/,like,b,_),b))>";
                "<(*,a,b) --> like>", "like", "(*,(/,like,b,_),b)" => "<like --> (*,(/,like,b,_),b)>";
                "<(*,acid,base) --> reaction>", "neutralization", "reaction" => "<neutralization --> reaction>";
                "<(*,b,a) --> (*,b,(/,like,b,_))>", "a", "(/,like,b,_)" => "<a --> (/,like,b,_)>";
                "<(*,b,a) --> (*,b,(/,like,b,_))>", "b", "b" => None;
                "<(*,num) <-> (*,(/,num,_))>", "num", "(/,num,_)" => "<num <-> (/,num,_)>";
                "<(*,tim,tom) --> uncle>", "(/,(*,tim,tom),_,tom)", "(/,uncle,_,tom)" => "<(/,(*,tim,tom),_,tom) --> (/,uncle,_,tom)>";
                "<(-,swimmer,animal) --> (-,swimmer,bird)>", "bird", "animal" => "<bird --> animal>";
                "<(-,swimmer,animal) --> (-,swimmer,bird)>", "swimmer", "swimmer" => None;
                "<(--,<robin --> [flying]>) ==> <robin --> bird>>", "(--,<robin --> bird>)", "<robin --> [flying]>" => "<(--,<robin --> bird>) ==> <robin --> [flying]>>";
                "<(--,<robin --> bird>) ==> <robin --> [flying]>>", "(--,<robin --> [flying]>)", "<robin --> bird>" => "<(--,<robin --> [flying]>) ==> <robin --> bird>>";
                "<(/,(*,0),_) --> (/,num,_)>", "(*,(/,(*,0),_))", "(*,(/,num,_))" => "<(*,(/,(*,0),_)) --> (*,(/,num,_))>";
                "<(/,(*,tim,tom),_,tom) --> (/,uncle,_,tom)>", "(*,tim,tom)", "uncle" => "<(*,tim,tom) --> uncle>";
                "<(/,(*,tim,tom),_,tom) --> (/,uncle,_,tom)>", "tom", "tom" => None;
                "<(/,(*,tim,tom),tom,_) --> (/,uncle,tom,_)>", "(&,tim,(/,(*,tim,tom),tom,_))", "(/,uncle,tom,_)" => "<(&,tim,(/,(*,tim,tom),tom,_)) --> (/,uncle,tom,_)>";
                "<(/,(*,tim,tom),tom,_) --> (/,uncle,tom,_)>", "(/,(*,tim,tom),tom,_)", "tim" => "<(/,(*,tim,tom),tom,_) --> tim>";
                "<(/,(*,tim,tom),tom,_) --> (/,uncle,tom,_)>", "(|,tim,(/,(*,tim,tom),tom,_))", "(/,uncle,tom,_)" => "<(|,tim,(/,(*,tim,tom),tom,_)) --> (/,uncle,tom,_)>";
                "<(/,(*,tim,tom),tom,_) --> (/,uncle,tom,_)>", "(~,(/,(*,tim,tom),tom,_),tim)", "(/,uncle,tom,_)" => "<(~,(/,(*,tim,tom),tom,_),tim) --> (/,uncle,tom,_)>";
                "<(/,(*,tim,tom),tom,_) --> (/,uncle,tom,_)>", "tim", "(/,(*,tim,tom),tom,_)" => "<tim --> (/,(*,tim,tom),tom,_)>";
                "<(/,neutralization,_,base) --> (/,reaction,_,base)>", "(&,acid,(/,neutralization,_,base))", "(/,reaction,_,base)" => "<(&,acid,(/,neutralization,_,base)) --> (/,reaction,_,base)>";
                "<(/,neutralization,_,base) --> (/,reaction,_,base)>", "(/,neutralization,_,base)", "acid" => "<(/,neutralization,_,base) --> acid>";
                "<(/,neutralization,_,base) --> (/,reaction,_,base)>", "(|,acid,(/,neutralization,_,base))", "(/,reaction,_,base)" => "<(|,acid,(/,neutralization,_,base)) --> (/,reaction,_,base)>";
                "<(/,neutralization,_,base) --> (/,reaction,_,base)>", "acid", "(/,neutralization,_,base)" => "<acid --> (/,neutralization,_,base)>";
                "<(/,neutralization,_,base) --> (/,reaction,_,base)>", "base", "base" => None;
                "<(/,neutralization,_,base) --> (/,reaction,_,base)>", "neutralization", "reaction" => "<neutralization --> reaction>";
                "<(/,neutralization,_,base) --> ?1>", "(/,neutralization,_,base)", "(/,reaction,_,base)" => "<(/,neutralization,_,base) --> (/,reaction,_,base)>";
                "<(/,neutralization,_,base) --> ?1>", "(/,reaction,_,base)", "?1" => "<(/,reaction,_,base) --> ?1>";
                "<(/,neutralization,_,base) --> ?1>", "?1", "(/,reaction,_,base)" => "<?1 --> (/,reaction,_,base)>";
                "<(/,neutralization,acid,_) <-> (/,reaction,acid,_)>", "acid", "acid" => None;
                "<(/,num,_) --> num>", "(*,(/,num,_))", "(*,num)" => "<(*,(/,num,_)) --> (*,num)>";
                "<(/,open,_,lock) --> (&,key,(/,open,_,{lock1}))>", "(/,open,_,lock)", "(/,open,_,{lock1})" => "<(/,open,_,lock) --> (/,open,_,{lock1})>";
                "<(/,open,_,lock) --> (&,key,(/,open,_,{lock1}))>", "(/,open,_,lock)", "key" => "<(/,open,_,lock) --> key>";
                "<(/,open,_,lock) --> (/,open,_,{lock1})>", "(/,open,_,lock)", "(&,key,(/,open,_,{lock1}))" => "<(/,open,_,lock) --> (&,key,(/,open,_,{lock1}))>";
                "<(/,open,_,lock) --> (/,open,_,{lock1})>", "(/,open,_,lock)", "(|,key,(/,open,_,{lock1}))" => "<(/,open,_,lock) --> (|,key,(/,open,_,{lock1}))>";
                "<(/,open,_,lock) --> (/,open,_,{lock1})>", "(/,open,_,{lock1})", "key" => "<(/,open,_,{lock1}) --> key>";
                "<(/,open,_,lock) --> (/,open,_,{lock1})>", "key", "(/,open,_,{lock1})" => "<key --> (/,open,_,{lock1})>";
                "<(/,open,_,lock) --> (/,open,_,{lock1})>", "open", "open" => None;
                "<(/,open,_,lock) --> (/,open,_,{lock1})>", "{lock1}", "lock" => "<{lock1} --> lock>";
                "<(/,open,_,lock) --> key>", "(/,open,_,lock)", "(&,key,(/,open,_,{lock1}))" => "<(/,open,_,lock) --> (&,key,(/,open,_,{lock1}))>";
                "<(/,open,_,lock) --> key>", "(/,open,_,lock)", "(|,key,(/,open,_,{lock1}))" => "<(/,open,_,lock) --> (|,key,(/,open,_,{lock1}))>";
                "<(/,open,_,lock) --> key>", "(/,open,_,{lock1})", "key" => "<(/,open,_,{lock1}) --> key>";
                "<(/,open,_,lock) --> key>", "key", "(/,open,_,{lock1})" => "<key --> (/,open,_,{lock1})>";
                "<(/,reaction,acid,_) --> soda>", "(/,neutralization,acid,_)", "soda" => "<(/,neutralization,acid,_) --> soda>";
                "<(/,reaction,acid,_) --> soda>", "(/,reaction,acid,_)", "(&,soda,(/,neutralization,acid,_))" => "<(/,reaction,acid,_) --> (&,soda,(/,neutralization,acid,_))>";
                "<(/,reaction,acid,_) --> soda>", "(/,reaction,acid,_)", "(|,soda,(/,neutralization,acid,_))" => "<(/,reaction,acid,_) --> (|,soda,(/,neutralization,acid,_))>";
                "<(/,reaction,acid,_) --> soda>", "soda", "(/,neutralization,acid,_)" => "<soda --> (/,neutralization,acid,_)>";
                "<(|,acid,(/,neutralization,_,base)) --> (/,reaction,_,base)>", "(/,neutralization,_,base)", "(/,reaction,_,base)" => "<(/,neutralization,_,base) --> (/,reaction,_,base)>";
                "<(|,acid,(/,neutralization,_,base)) --> (/,reaction,_,base)>", "acid", "(/,reaction,_,base)" => "<acid --> (/,reaction,_,base)>";
                "<(|,bird,robin) --> animal>", "bird", "animal" => "<bird --> animal>";
                "<(|,bird,{Tweety}) --> (|,bird,flyer)>", "bird", "bird" => None;
                "<(|,bird,{Tweety}) --> (|,bird,flyer)>", "{Tweety}", "flyer" => "<{Tweety} --> flyer>";
                "<(|,bird,{Tweety}) --> (|,bird,{Birdie})>", "bird", "bird" => None;
                "<(|,bird,{Tweety}) --> (|,bird,{Birdie})>", "{Tweety}", "{Birdie}" => "<{Tweety} --> {Birdie}>";
                "<(|,boy,girl) --> (|,girl,youth)>", "(|,boy,girl)", "(|,girl,youth)" => "<(|,boy,girl) --> (|,girl,youth)>";
                "<(|,boy,girl) --> (|,girl,youth)>", "boy", "girl" => "<boy --> girl>";
                "<(|,boy,girl) --> (~,youth,girl)>", "(~,youth,girl)", "(|,boy,girl)" => "<(~,youth,girl) --> (|,boy,girl)>";
                "<(|,boy,girl) --> youth>", "(|,boy,girl)", "(~,youth,girl)" => "<(|,boy,girl) --> (~,youth,girl)>";
                "<(|,boy,girl) --> youth>", "(|,boy,girl)", "youth" => "<(|,boy,girl) --> youth>";
                "<(|,boy,girl) --> youth>", "(~,(|,boy,girl),girl)", "(~,youth,girl)" => "<(~,(|,boy,girl),girl) --> (~,youth,girl)>";
                "<(|,boy,girl) --> youth>", "youth", "(|,boy,girl)" => "<youth --> (|,boy,girl)>";
                "<(|,chess,sport) --> (|,chess,competition)>", "chess", "chess" => None;
                "<(|,chess,sport) --> competition>", "(|,chess,sport)", "(|,chess,competition)" => "<(|,chess,sport) --> (|,chess,competition)>";
                "<(|,robin,swan) --> (&,bird,swimmer)>", "(|,robin,swan)", "bird" => "<(|,robin,swan) --> bird>";
                "<(|,robin,swan) --> (|,bird,swimmer)>", "robin", "(|,bird,swimmer)" => "<robin --> (|,bird,swimmer)>";
                "<(|,robin,swimmer) --> bird>", "(|,robin,swimmer)", "(&,animal,bird)" => "<(|,robin,swimmer) --> (&,animal,bird)>";
                "<(|,robin,{Tweety}) --> [with_wings]>", "robin", "[with_wings]" => "<robin --> [with_wings]>";
                "<(|,robin,{Tweety}) --> [with_wings]>", "{Tweety}", "[with_wings]" => "<{Tweety} --> [with_wings]>";
                "<(~,boy,girl) --> (&,[strong],(~,youth,girl))>", "(~,boy,girl)", "(&,[strong],(~,youth,girl))" => "<(~,boy,girl) --> (&,[strong],(~,youth,girl))>";
                "<(~,boy,girl) --> (~,youth,girl)>", "boy", "(~,youth,girl)" => "<boy --> (~,youth,girl)>";
                "<(~,boy,girl) --> (~,youth,girl)>", "boy", "youth" => "<boy --> youth>";
                "<(~,boy,girl) --> (~,youth,girl)>", "girl", "(~,youth,girl)" => None;
                "<(~,boy,girl) --> (~,youth,girl)>", "girl", "girl" => None;
                "<(~,boy,girl) --> [strong]>", "(~,boy,girl)", "(&,[strong],(~,youth,girl))" => "<(~,boy,girl) --> (&,[strong],(~,youth,girl))>";
                "<(~,boy,girl) --> [strong]>", "(~,boy,girl)", "(|,[strong],(~,youth,girl))" => "<(~,boy,girl) --> (|,[strong],(~,youth,girl))>";
                "<(~,boy,girl) --> [strong]>", "(~,boy,girl)", "[strong]" => "<(~,boy,girl) --> [strong]>";
                "<(~,boy,girl) --> [strong]>", "[strong]", "(~,youth,girl)" => "<[strong] --> (~,youth,girl)>";
                "<(~,boy,girl) --> [strong]>", "boy", "[strong]" => "<boy --> [strong]>";
                "<0 --> (/,num,_)>", "(*,0)", "(*,(/,num,_))" => "<(*,0) --> (*,(/,num,_))>";
                "<0 --> (/,num,_)>", "(/,num,_)", "num" => "<(/,num,_) --> num>";
                "<0 --> (/,num,_)>", "0", "num" => "<0 --> num>";
                "<0 --> (/,num,_)>", "num", "(/,num,_)" => "<num --> (/,num,_)>";
                "<0 --> num>", "(*,0)", "(*,num)" => "<(*,0) --> (*,num)>";
                "<0 --> num>", "(/,num,_)", "num" => "<(/,num,_) --> num>";
                "<0 --> num>", "num", "(/,num,_)" => "<num --> (/,num,_)>";
                "<<robin --> [with_wings]> ==> <robin --> [living]>>", "<robin --> flyer>", "<robin --> [living]>" => "<<robin --> flyer> ==> <robin --> [living]>>";
                "<<robin --> [with_wings]> ==> <robin --> bird>>", "<robin --> [with_wings]>", "(&&,<robin --> bird>,<robin --> [living]>)" => "<<robin --> [with_wings]> ==> (&&,<robin --> bird>,<robin --> [living]>)>";
                "<<robin --> [with_wings]> ==> <robin --> bird>>", "<robin --> [with_wings]>", "(||,<robin --> bird>,<robin --> [living]>)" => "<<robin --> [with_wings]> ==> (||,<robin --> bird>,<robin --> [living]>)>";
                "<<robin --> [with_wings]> ==> <robin --> bird>>", "<robin --> flyer>", "<robin --> bird>" => "<<robin --> flyer> ==> <robin --> bird>>";
                "<?1 --> claimedByBob>", "(&,<bird --> fly>,<{Tweety} --> bird>)", "?1" => "<(&,<bird --> fly>,<{Tweety} --> bird>) --> ?1>";
                "<?1 --> claimedByBob>", "?1", "(&,<bird --> fly>,<{Tweety} --> bird>)" => "<?1 --> (&,<bird --> fly>,<{Tweety} --> bird>)>";
                "<?1 --> swimmer>", "?1", "animal" => "<?1 --> animal>";
                "<?1 --> swimmer>", "animal", "?1" => "<animal --> ?1>";
                "<?1 --> swimmer>", "animal", "swimmer" => "<animal --> swimmer>";
                "<Birdie <-> Tweety>", "Birdie", "Tweety" => "<Birdie <-> Tweety>";
                "<Birdie <-> Tweety>", "{Birdie}", "{Tweety}" => "<{Birdie} <-> {Tweety}>";
                "<CAT --> (/,(/,REPRESENT,_,<(*,CAT,FISH) --> FOOD>),_,eat,fish)>", "CAT", "(|,CAT,(/,(/,REPRESENT,_,<(*,CAT,FISH) --> FOOD>),_,eat,fish))" => None;
                "<[bright] --> [smart]>", "[smart]", "[bright]" => "<[smart] --> [bright]>";
                "<[bright] <-> [smart]>", "bright", "smart" => "<bright <-> smart>";
                "<[with_wings] --> {Birdie}>", "[with_wings]", "{Tweety}" => "<[with_wings] --> {Tweety}>";
                "<[yellow] --> {Birdie}>", "(|,flyer,[yellow])", "(|,flyer,{Birdie})" => "<(|,flyer,[yellow]) --> (|,flyer,{Birdie})>";
                "<[yellow] <-> {Birdie}>", "(|,flyer,[yellow])", "(|,flyer,{Birdie})" => "<(|,flyer,[yellow]) <-> (|,flyer,{Birdie})>";
                "<[yellow] <-> {Birdie}>", "[yellow]", "{Tweety}" => "<[yellow] <-> {Tweety}>";
                "<a --> (/,like,b,_)>", "(*,a,b)", "(*,(/,like,b,_),b)" => "<(*,a,b) --> (*,(/,like,b,_),b)>";
                "<a --> (/,like,b,_)>", "(*,b,a)", "(*,b,(/,like,b,_))" => "<(*,b,a) --> (*,b,(/,like,b,_))>";
                "<a --> (/,like,b,_)>", "(/,like,_,(/,like,b,_))", "(/,like,_,a)" => "<(/,like,_,(/,like,b,_)) --> (/,like,_,a)>";
                "<acid --> (/,reaction,_,base)>", "(&,acid,(/,neutralization,_,base))", "(/,reaction,_,base)" => "<(&,acid,(/,neutralization,_,base)) --> (/,reaction,_,base)>";
                "<acid --> (/,reaction,_,base)>", "(/,neutralization,_,base)", "acid" => "<(/,neutralization,_,base) --> acid>";
                "<acid --> (/,reaction,_,base)>", "(|,acid,(/,neutralization,_,base))", "(/,reaction,_,base)" => "<(|,acid,(/,neutralization,_,base)) --> (/,reaction,_,base)>";
                "<acid --> (/,reaction,_,base)>", "acid", "(/,neutralization,_,base)" => "<acid --> (/,neutralization,_,base)>";
                "<b --> (/,like,_,a)>", "(/,like,(/,like,_,a),_)", "(/,like,b,_)" => "<(/,like,(/,like,_,a),_) --> (/,like,b,_)>";
                "<bird --> (&,animal,swimmer)>", "bird", "animal" => "<bird --> animal>";
                "<bird --> animal>", "(&,bird,robin)", "animal" => "<(&,bird,robin) --> animal>";
                "<bird --> animal>", "(|,bird,robin)", "animal" => "<(|,bird,robin) --> animal>";
                "<bird --> animal>", "bird", "robin" => "<bird --> robin>";
                "<bird --> swimmer>", "bird", "(&,animal,swimmer)" => "<bird --> (&,animal,swimmer)>";
                "<bird --> swimmer>", "bird", "(|,animal,swimmer)" => "<bird --> (|,animal,swimmer)>";
                "<bird --> {Birdie}>", "bird", "(|,bird,{Birdie})" => None;
                "<boy --> [strong]>", "(~,boy,girl)", "(~,[strong],girl)" => "<(~,boy,girl) --> (~,[strong],girl)>";
                "<boy --> youth>", "(|,boy,girl)", "(|,girl,youth)" => "<(|,boy,girl) --> (|,girl,youth)>";
                "<boy --> youth>", "(~,boy,girl)", "(~,youth,girl)" => "<(~,boy,girl) --> (~,youth,girl)>";
                "<bright <-> smart>", "[bright]", "[smart]" => "<[bright] <-> [smart]>";
                "<cat --> (&,CAT,(/,(/,REPRESENT,_,<(*,CAT,FISH) --> FOOD>),_,eat,fish))>", "cat", "(/,(/,REPRESENT,_,<(*,CAT,FISH) --> FOOD>),_,eat,fish)" => "<cat --> (/,(/,REPRESENT,_,<(*,CAT,FISH) --> FOOD>),_,eat,fish)>";
                "<cat --> (&,CAT,(/,(/,REPRESENT,_,<(*,CAT,FISH) --> FOOD>),_,eat,fish))>", "cat", "CAT" => "<cat --> CAT>";
                "<cat --> CAT>", "(/,(/,REPRESENT,_,<(*,CAT,FISH) --> FOOD>),_,eat,fish)", "CAT" => "<(/,(/,REPRESENT,_,<(*,CAT,FISH) --> FOOD>),_,eat,fish) --> CAT>";
                "<cat --> CAT>", "CAT", "(/,(/,REPRESENT,_,<(*,CAT,FISH) --> FOOD>),_,eat,fish)" => "<CAT --> (/,(/,REPRESENT,_,<(*,CAT,FISH) --> FOOD>),_,eat,fish)>";
                "<cat --> CAT>", "cat", "(&,CAT,(/,(/,REPRESENT,_,<(*,CAT,FISH) --> FOOD>),_,eat,fish))" => "<cat --> (&,CAT,(/,(/,REPRESENT,_,<(*,CAT,FISH) --> FOOD>),_,eat,fish))>";
                "<cat --> CAT>", "cat", "(|,CAT,(/,(/,REPRESENT,_,<(*,CAT,FISH) --> FOOD>),_,eat,fish))" => "<cat --> (|,CAT,(/,(/,REPRESENT,_,<(*,CAT,FISH) --> FOOD>),_,eat,fish))>";
                "<chess --> competition>", "(~,sport,chess)", "competition" => "<(~,sport,chess) --> competition>";
                "<chess --> competition>", "chess", "(|,chess,competition)" => None;
                "<flyer <-> [with_wings]>", "(|,flyer,{Birdie})", "(|,[with_wings],{Birdie})" => "<(|,flyer,{Birdie}) <-> (|,[with_wings],{Birdie})>";
                "<neutralization --> (*,acid,base)>", "neutralization", "reaction" => "<neutralization --> reaction>";
                "<neutralization --> reaction>", "(/,neutralization,_,base)", "(/,reaction,_,base)" => "<(/,neutralization,_,base) --> (/,reaction,_,base)>";
                "<neutralization <-> reaction>", "(/,neutralization,_,base)", "(/,reaction,_,base)" => "<(/,neutralization,_,base) <-> (/,reaction,_,base)>";
                "<num <-> (/,num,_)>", "(*,num)", "(*,(/,num,_))" => "<(*,num) <-> (*,(/,num,_))>";
                "<num <-> (/,num,_)>", "(/,num,_)", "(/,(/,num,_),_)" => "<(/,num,_) <-> (/,(/,num,_),_)>";
                "<planetX --> {Mars,Pluto,Saturn,Venus}>", "{Mars,Pluto,Saturn,Venus}", "{Mars,Pluto,Venus}" => "<{Mars,Pluto,Saturn,Venus} --> {Mars,Pluto,Venus}>";
                "<planetX --> {Mars,Pluto,Saturn,Venus}>", "{Mars,Pluto,Venus}", "{Mars,Pluto,Saturn,Venus}" => "<{Mars,Pluto,Venus} --> {Mars,Pluto,Saturn,Venus}>";
                "<planetX --> {Mars,Pluto,Venus}>", "planetX", "{Mars,Pluto,Saturn,Venus}" => "<planetX --> {Mars,Pluto,Saturn,Venus}>";
                "<planetX --> {Mars,Pluto,Venus}>", "planetX", "{Mars,Venus}" => "<planetX --> {Mars,Venus}>";
                "<planetX --> {Mars,Pluto,Venus}>", "planetX", "{Pluto}" => "<planetX --> {Pluto}>";
                "<planetX --> {Mars,Pluto,Venus}>", "{Mars,Pluto,Venus}", "{Pluto,Saturn}" => "<{Mars,Pluto,Venus} --> {Pluto,Saturn}>";
                "<planetX --> {Mars,Pluto,Venus}>", "{Pluto,Saturn}", "{Mars,Pluto,Venus}" => "<{Pluto,Saturn} --> {Mars,Pluto,Venus}>";
                "<planetX --> {Mars,Venus}>", "planetX", "{Mars,Pluto,Saturn,Venus}" => "<planetX --> {Mars,Pluto,Saturn,Venus}>";
                "<planetX --> {Mars,Venus}>", "{Mars,Venus}", "{Pluto,Saturn}" => "<{Mars,Venus} --> {Pluto,Saturn}>";
                "<planetX --> {Pluto,Saturn}>", "planetX", "{Mars,Venus}" => "<planetX --> {Mars,Venus}>";
                "<planetX --> {Pluto,Saturn}>", "planetX", "{Pluto}" => "<planetX --> {Pluto}>";
                "<planetX --> {Pluto,Saturn}>", "{Mars,Pluto,Saturn,Venus}", "{Pluto,Saturn}" => "<{Mars,Pluto,Saturn,Venus} --> {Pluto,Saturn}>";
                "<planetX --> {Pluto,Saturn}>", "{Mars,Pluto,Venus}", "{Pluto,Saturn}" => "<{Mars,Pluto,Venus} --> {Pluto,Saturn}>";
                "<planetX --> {Pluto,Saturn}>", "{Pluto,Saturn}", "{Mars,Pluto,Saturn,Venus}" => "<{Pluto,Saturn} --> {Mars,Pluto,Saturn,Venus}>";
                "<planetX --> {Pluto,Saturn}>", "{Pluto,Saturn}", "{Mars,Pluto,Venus}" => "<{Pluto,Saturn} --> {Mars,Pluto,Venus}>";
                "<robin --> (-,bird,swimmer)>", "robin", "bird" => "<robin --> bird>";
                "<robin --> (|,bird,swimmer)>", "(&,robin,swan)", "(|,bird,swimmer)" => "<(&,robin,swan) --> (|,bird,swimmer)>";
                "<robin --> (|,bird,swimmer)>", "(|,robin,swan)", "(|,bird,swimmer)" => "<(|,robin,swan) --> (|,bird,swimmer)>";
                "<robin --> (|,bird,swimmer)>", "robin", "swan" => "<robin --> swan>";
                "<robin --> [with_wings]>", "(&,flyer,robin)", "(&,flyer,[with_wings])" => "<(&,flyer,robin) --> (&,flyer,[with_wings])>";
                "<robin --> [with_wings]>", "(&,robin,{Birdie})", "[with_wings]" => "<(&,robin,{Birdie}) --> [with_wings]>";
                "<robin --> [with_wings]>", "(|,flyer,robin)", "(|,flyer,[with_wings])" => "<(|,flyer,robin) --> (|,flyer,[with_wings])>";
                "<robin --> [with_wings]>", "(|,robin,{Birdie})", "(|,[with_wings],{Birdie})" => "<(|,robin,{Birdie}) --> (|,[with_wings],{Birdie})>";
                "<robin --> [with_wings]>", "(|,robin,{Birdie})", "[with_wings]" => "<(|,robin,{Birdie}) --> [with_wings]>";
                "<robin --> [with_wings]>", "robin", "(|,[with_wings],{Birdie})" => "<robin --> (|,[with_wings],{Birdie})>";
                "<robin --> [with_wings]>", "robin", "(|,flyer,[with_wings])" => "<robin --> (|,flyer,[with_wings])>";
                "<robin --> [with_wings]>", "robin", "flyer" => "<robin --> flyer>";
                "<robin --> [with_wings]>", "robin", "{Birdie}" => "<robin --> {Birdie}>";
                "<robin --> [with_wings]>", "{Birdie}", "robin" => "<{Birdie} --> robin>";
                "<soda --> base>", "(/,reaction,acid,_)", "soda" => "<(/,reaction,acid,_) --> soda>";
                "<soda --> base>", "soda", "(/,reaction,acid,_)" => "<soda --> (/,reaction,acid,_)>";
                "<swan --> (&,bird,swimmer)>", "(&,robin,swan)", "(&,bird,swimmer)" => "<(&,robin,swan) --> (&,bird,swimmer)>";
                "<swan --> (&,bird,swimmer)>", "(|,robin,swan)", "(&,bird,swimmer)" => "<(|,robin,swan) --> (&,bird,swimmer)>";
                "<swan --> swimmer>", "(&,swan,swimmer)", "swimmer" => None;
                "<swan --> swimmer>", "(~,swimmer,swan)", "swimmer" => None;
                "<tiger --> animal>", "(&,robin,tiger)", "(&,animal,robin)" => "<(&,robin,tiger) --> (&,animal,robin)>";
                "<tim --> (/,uncle,_,tom)>", "(/,uncle,_,tom)", "(/,uncle,tom,_)" => "<(/,uncle,_,tom) --> (/,uncle,tom,_)>";
                "<tim --> (/,uncle,tom,_)>", "(&,tim,(/,(*,tim,tom),tom,_))", "(/,uncle,tom,_)" => "<(&,tim,(/,(*,tim,tom),tom,_)) --> (/,uncle,tom,_)>";
                "<tim --> (/,uncle,tom,_)>", "(/,(*,tim,tom),tom,_)", "tim" => "<(/,(*,tim,tom),tom,_) --> tim>";
                "<tim --> (/,uncle,tom,_)>", "(|,tim,(/,(*,tim,tom),tom,_))", "(/,uncle,tom,_)" => "<(|,tim,(/,(*,tim,tom),tom,_)) --> (/,uncle,tom,_)>";
                "<tim --> (/,uncle,tom,_)>", "(~,(/,(*,tim,tom),tom,_),tim)", "(/,uncle,tom,_)" => "<(~,(/,(*,tim,tom),tom,_),tim) --> (/,uncle,tom,_)>";
                "<tim --> (/,uncle,tom,_)>", "tim", "(/,(*,tim,tom),tom,_)" => "<tim --> (/,(*,tim,tom),tom,_)>";
                "<{?1} --> swimmer>", "robin", "{?1}" => "<robin --> {?1}>";
                "<{?1} --> swimmer>", "{?1}", "bird" => "<{?1} --> bird>";
                "<{Birdie} --> [with_wings]>", "{Tweety}", "[with_wings]" => "<{Tweety} --> [with_wings]>";
                "<{Birdie} --> [yellow]>", "(&,flyer,{Birdie})", "(&,flyer,[yellow])" => "<(&,flyer,{Birdie}) --> (&,flyer,[yellow])>";
                "<{Birdie} --> [yellow]>", "(|,flyer,{Birdie})", "(|,flyer,[yellow])" => "<(|,flyer,{Birdie}) --> (|,flyer,[yellow])>";
                "<{Birdie} --> [yellow]>", "{Birdie}", "(|,[yellow],{Birdie})" => None;
                "<{Birdie} --> [yellow]>", "{Birdie}", "(|,flyer,[yellow])" => "<{Birdie} --> (|,flyer,[yellow])>";
                "<{Birdie} --> flyer>", "(&,flyer,{Birdie})", "flyer" => None;
                "<{Birdie} --> flyer>", "{Tweety}", "flyer" => "<{Tweety} --> flyer>";
                "<{Birdie} <-> {Tweety}>", "Birdie", "Tweety" => "<Birdie <-> Tweety>";
                "<{Birdie} <-> {Tweety}>", "{Birdie}", "{Tweety}" => "<{Birdie} <-> {Tweety}>";
                "<{Birdie} <-> {Tweety}>", "{Tweety}", "bird" => "<bird <-> {Tweety}>";
                "<{Mars,Pluto,Saturn,Venus} --> {Mars,Pluto,Venus}>", "{Pluto}", "{Mars,Pluto,Venus}" => "<{Pluto} --> {Mars,Pluto,Venus}>";
                "<{Tweety} --> (&,[with_wings],{Birdie})>", "{Tweety}", "[with_wings]" => "<{Tweety} --> [with_wings]>";
                "<{Tweety} --> (&,[with_wings],{Birdie})>", "{Tweety}", "{Birdie}" => "<{Tweety} --> {Birdie}>";
                "<{Tweety} --> (&,bird,flyer)>", "{Tweety}", "bird" => "<{Tweety} --> bird>";
                "<{Tweety} --> (&,bird,{Birdie})>", "{Tweety}", "bird" => "<{Tweety} --> bird>";
                "<{Tweety} --> (&,bird,{Birdie})>", "{Tweety}", "{Birdie}" => "<{Tweety} --> {Birdie}>";
                "<{Tweety} --> (&,flyer,(|,[yellow],{Birdie}))>", "{Tweety}", "(|,[yellow],{Birdie})" => "<{Tweety} --> (|,[yellow],{Birdie})>";
                "<{Tweety} --> (&,flyer,(|,[yellow],{Birdie}))>", "{Tweety}", "flyer" => "<{Tweety} --> flyer>";
                "<{Tweety} --> (&,flyer,[with_wings])>", "{Tweety}", "[with_wings]" => "<{Tweety} --> [with_wings]>";
                "<{Tweety} --> (&,flyer,[with_wings])>", "{Tweety}", "flyer" => "<{Tweety} --> flyer>";
                "<{Tweety} --> (|,[with_wings],{Birdie})>", "(&,flyer,[yellow])", "(|,[with_wings],{Birdie})" => "<(&,flyer,[yellow]) --> (|,[with_wings],{Birdie})>";
                "<{Tweety} --> (|,[with_wings],{Birdie})>", "(|,[with_wings],{Birdie})", "(&,flyer,[yellow])" => "<(|,[with_wings],{Birdie}) --> (&,flyer,[yellow])>";
                "<{Tweety} --> (|,[with_wings],{Birdie})>", "{Tweety}", "(&,flyer,[yellow],(|,[with_wings],{Birdie}))" => "<{Tweety} --> (&,flyer,[yellow],(|,[with_wings],{Birdie}))>";
                "<{Tweety} --> (|,[with_wings],{Birdie})>", "{Tweety}", "(|,[with_wings],{Birdie},(&,flyer,[yellow]))" => "<{Tweety} --> (|,[with_wings],{Birdie},(&,flyer,[yellow]))>";
                "<{Tweety} --> (|,bird,flyer)>", "(|,bird,flyer)", "(|,bird,{Birdie})" => "<(|,bird,flyer) --> (|,bird,{Birdie})>";
                "<{Tweety} --> (|,bird,flyer)>", "(|,bird,{Birdie})", "(|,bird,flyer)" => "<(|,bird,{Birdie}) --> (|,bird,flyer)>";
                "<{Tweety} --> (|,bird,flyer)>", "{Tweety}", "(&,(|,bird,flyer),(|,bird,{Birdie}))" => "<{Tweety} --> (&,(|,bird,flyer),(|,bird,{Birdie}))>";
                "<{Tweety} --> (|,bird,flyer)>", "{Tweety}", "(|,bird,flyer,{Birdie})" => "<{Tweety} --> (|,bird,flyer,{Birdie})>";
                "<{Tweety} --> (|,flyer,[yellow])>", "bird", "(|,flyer,[yellow])" => "<bird --> (|,flyer,[yellow])>";
                "<{Tweety} --> [with_wings]>", "(&,flyer,{Birdie})", "[with_wings]" => "<(&,flyer,{Birdie}) --> [with_wings]>";
                "<{Tweety} --> [with_wings]>", "(|,flyer,{Birdie})", "[with_wings]" => "<(|,flyer,{Birdie}) --> [with_wings]>";
                "<{Tweety} --> [with_wings]>", "[with_wings]", "(&,flyer,{Birdie})" => "<[with_wings] --> (&,flyer,{Birdie})>";
                "<{Tweety} --> [with_wings]>", "[with_wings]", "(|,flyer,{Birdie})" => "<[with_wings] --> (|,flyer,{Birdie})>";
                "<{Tweety} --> [with_wings]>", "[with_wings]", "flyer" => "<[with_wings] --> flyer>";
                "<{Tweety} --> [with_wings]>", "flyer", "[with_wings]" => "<flyer --> [with_wings]>";
                "<{Tweety} --> [with_wings]>", "robin", "{Tweety}" => "<robin --> {Tweety}>";
                "<{Tweety} --> [with_wings]>", "{Birdie,Tweety}", "(|,[with_wings],{Birdie})" => "<{Birdie,Tweety} --> (|,[with_wings],{Birdie})>";
                "<{Tweety} --> [with_wings]>", "{Tweety}", "(&,[with_wings],(|,flyer,{Birdie}))" => "<{Tweety} --> (&,[with_wings],(|,flyer,{Birdie}))>";
                "<{Tweety} --> [with_wings]>", "{Tweety}", "(&,flyer,[with_wings])" => "<{Tweety} --> (&,flyer,[with_wings])>";
                "<{Tweety} --> [with_wings]>", "{Tweety}", "(&,flyer,[with_wings],{Birdie})" => "<{Tweety} --> (&,flyer,[with_wings],{Birdie})>";
                "<{Tweety} --> [with_wings]>", "{Tweety}", "(|,[with_wings],(&,flyer,{Birdie}))" => "<{Tweety} --> (|,[with_wings],(&,flyer,{Birdie}))>";
                "<{Tweety} --> [with_wings]>", "{Tweety}", "(|,[with_wings],{Birdie})" => "<{Tweety} --> (|,[with_wings],{Birdie})>";
                "<{Tweety} --> [with_wings]>", "{Tweety}", "(|,flyer,[with_wings],{Birdie})" => "<{Tweety} --> (|,flyer,[with_wings],{Birdie})>";
                "<{Tweety} --> [with_wings]>", "{Tweety}", "robin" => "<{Tweety} --> robin>";
                "<{Tweety} --> bird>", "bird", "flyer" => "<bird --> flyer>";
                "<{Tweety} --> bird>", "bird", "{Birdie}" => "<bird --> {Birdie}>";
                "<{Tweety} --> bird>", "{Tweety}", "(&,bird,flyer)" => "<{Tweety} --> (&,bird,flyer)>";
                "<{Tweety} --> bird>", "{Tweety}", "(&,bird,{Birdie})" => "<{Tweety} --> (&,bird,{Birdie})>";
                "<{Tweety} --> bird>", "{Tweety}", "(|,bird,flyer)" => "<{Tweety} --> (|,bird,flyer)>";
                "<{Tweety} --> bird>", "{Tweety}", "(|,bird,{Birdie})" => "<{Tweety} --> (|,bird,{Birdie})>";
                "<{Tweety} --> flyer>", "(&,[with_wings],{Birdie})", "flyer" => "<(&,[with_wings],{Birdie}) --> flyer>";
                "<{Tweety} --> flyer>", "(|,[with_wings],{Birdie})", "flyer" => "<(|,[with_wings],{Birdie}) --> flyer>";
                "<{Tweety} --> flyer>", "[with_wings]", "flyer" => "<[with_wings] --> flyer>";
                "<{Tweety} --> flyer>", "flyer", "(&,[with_wings],{Birdie})" => "<flyer --> (&,[with_wings],{Birdie})>";
                "<{Tweety} --> flyer>", "flyer", "(|,[with_wings],{Birdie})" => "<flyer --> (|,[with_wings],{Birdie})>";
                "<{Tweety} --> flyer>", "flyer", "[with_wings]" => "<flyer --> [with_wings]>";
                "<{Tweety} --> flyer>", "{Tweety}", "(&,flyer,(|,[with_wings],{Birdie}))" => "<{Tweety} --> (&,flyer,(|,[with_wings],{Birdie}))>";
                "<{Tweety} --> flyer>", "{Tweety}", "(&,flyer,[with_wings])" => "<{Tweety} --> (&,flyer,[with_wings])>";
                "<{Tweety} --> flyer>", "{Tweety}", "(&,flyer,[with_wings],{Birdie})" => "<{Tweety} --> (&,flyer,[with_wings],{Birdie})>";
                "<{Tweety} --> flyer>", "{Tweety}", "(|,flyer,(&,[with_wings],{Birdie}))" => "<{Tweety} --> (|,flyer,(&,[with_wings],{Birdie}))>";
                "<{Tweety} --> flyer>", "{Tweety}", "(|,flyer,[with_wings])" => "<{Tweety} --> (|,flyer,[with_wings])>";
                "<{Tweety} --> flyer>", "{Tweety}", "(|,flyer,[with_wings],{Birdie})" => "<{Tweety} --> (|,flyer,[with_wings],{Birdie})>";
                "<{Tweety} --> {Birdie}>", "(|,bird,{Tweety})", "(|,bird,{Birdie})" => "<(|,bird,{Tweety}) --> (|,bird,{Birdie})>";
                "<{Tweety} --> {Birdie}>", "[with_wings]", "{Birdie}" => "<[with_wings] --> {Birdie}>";
                "<{Tweety} --> {Birdie}>", "bird", "{Birdie}" => "<bird --> {Birdie}>";
                "<{Tweety} --> {Birdie}>", "{Birdie}", "[with_wings]" => "<{Birdie} --> [with_wings]>";
                "<{Tweety} --> {Birdie}>", "{Birdie}", "bird" => "<{Birdie} --> bird>";
                "<{Tweety} --> {Birdie}>", "{Tweety}", "(&,[with_wings],{Birdie})" => "<{Tweety} --> (&,[with_wings],{Birdie})>";
                "<{Tweety} --> {Birdie}>", "{Tweety}", "(&,bird,{Birdie})" => "<{Tweety} --> (&,bird,{Birdie})>";
                "<{Tweety} --> {Birdie}>", "{Tweety}", "(|,[with_wings],{Birdie})" => "<{Tweety} --> (|,[with_wings],{Birdie})>";
                "<{Tweety} --> {Birdie}>", "{Tweety}", "(|,bird,{Birdie})" => "<{Tweety} --> (|,bird,{Birdie})>";
                "<{key1} --> (&,key,(/,open,_,{lock1}))>", "{key1}", "(/,open,_,{lock1})" => "<{key1} --> (/,open,_,{lock1})>";
                "<{key1} --> (&,key,(/,open,_,{lock1}))>", "{key1}", "key" => "<{key1} --> key>";
                "<{key1} --> (/,open,_,{lock1})>", "(/,open,_,{lock1})", "key" => "<(/,open,_,{lock1}) --> key>";
                "<{key1} --> (/,open,_,{lock1})>", "key", "(/,open,_,{lock1})" => "<key --> (/,open,_,{lock1})>";
                "<{key1} --> (/,open,_,{lock1})>", "{key1}", "(&,key,(/,open,_,{lock1}))" => "<{key1} --> (&,key,(/,open,_,{lock1}))>";
                "<{key1} --> (/,open,_,{lock1})>", "{key1}", "(|,key,(/,open,_,{lock1}))" => "<{key1} --> (|,key,(/,open,_,{lock1}))>";
                "<{key1} --> (|,key,(/,open,_,{lock1}))>", "{key1}", "(/,open,_,{lock1})" => "<{key1} --> (/,open,_,{lock1})>";
                "<{key1} --> (|,key,(/,open,_,{lock1}))>", "{key1}", "(|,key,(/,open,_,{lock1}))" => "<{key1} --> (|,key,(/,open,_,{lock1}))>";
                "<{key1} --> key>", "(/,open,_,{lock1})", "key" => "<(/,open,_,{lock1}) --> key>";
                "<{key1} --> key>", "key", "(/,open,_,{lock1})" => "<key --> (/,open,_,{lock1})>";
                "<{key1} --> key>", "{key1}", "(&,key,(/,open,_,{lock1}))" => "<{key1} --> (&,key,(/,open,_,{lock1}))>";
                "<{key1} --> key>", "{key1}", "(/,open,_,{lock1})" => "<{key1} --> (/,open,_,{lock1})>";
                "<{key1} --> key>", "{key1}", "(|,key,(/,open,_,{lock1}))" => "<{key1} --> (|,key,(/,open,_,{lock1}))>";
                "<{lock1} --> (&,lock,(/,open,{key1},_))>", "{lock1}", "lock" => "<{lock1} --> lock>";
                "<{lock1} --> (|,lock,(/,open,{key1},_))>", "(/,open,_,(|,lock,(/,open,{key1},_)))", "(/,open,_,{lock1})" => "<(/,open,_,(|,lock,(/,open,{key1},_))) --> (/,open,_,{lock1})>";
                "<{lock1} --> lock>", "(/,open,_,lock)", "(/,open,_,{lock1})" => "<(/,open,_,lock) --> (/,open,_,{lock1})>";
                "<{lock1} --> lock>", "(/,open,{key1},_)", "lock" => "<(/,open,{key1},_) --> lock>";
                "<{lock1} --> lock>", "lock", "(/,open,{key1},_)" => "<lock --> (/,open,{key1},_)>";
                "<{lock1} --> lock>", "{lock1}", "(&,lock,(/,open,{key1},_))" => "<{lock1} --> (&,lock,(/,open,{key1},_))>";
                "<{lock1} --> lock>", "{lock1}", "(|,lock,(/,open,{key1},_))" => "<{lock1} --> (|,lock,(/,open,{key1},_))>";
            }
            ok!()
        }

        #[test]
        fn make_statement_symmetric() -> AResult {
            fn test(template: Term, subject: Term, predicate: Term, expected: Option<Term>) {
                let out =
                    Term::make_statement_symmetric(&template, subject.clone(), predicate.clone());
                assert_eq!(
                    out,
                    expected,
                    "\"{template}\", \"{subject}\", \"{predicate}\" => {} != {}",
                    format_option_term(&out),
                    format_option_term(&expected),
                );
            }
            macro_once! {
                // * 🚩模式：参数列表 ⇒ 预期词项
                macro test($($template:tt, $subject:tt, $predicate:tt => $expected:tt;)*) {
                    $( test(term!($template), term!($subject), term!($predicate), option_term!($expected)); )*
                }
                "<(&&,<robin --> [flying]>,<robin --> [with_wings]>) ==> <robin --> bird>>", "<robin --> [living]>", "<robin --> bird>" => "<<robin --> bird> <=> <robin --> [living]>>";
                "<(&,bird,[yellow]) --> (&,bird,{Birdie})>", "(&,bird,[yellow])", "{Tweety}" => "<{Tweety} <-> (&,bird,[yellow])>";
                "<(&,robin,swan) --> (&,robin,(|,bird,swimmer))>", "bird", "(&,robin,(|,bird,swimmer))" => "<bird <-> (&,robin,(|,bird,swimmer))>";
                "<(&,robin,swan) --> bird>", "swimmer", "bird" => "<bird <-> swimmer>";
                "<(&,swan,swimmer) --> bird>", "(&,swimmer,(|,bird,robin))", "bird" => "<bird <-> (&,swimmer,(|,bird,robin))>";
                "<(*,(*,(*,0))) --> num>", "(*,(*,(*,0)))", "0" => "<0 <-> (*,(*,(*,0)))>";
                "<(*,b,a) --> like>", "(*,b,(/,like,_,b))", "like" => "<like <-> (*,b,(/,like,_,b))>";
                "<(/,neutralization,_,(/,reaction,acid,_)) --> (/,neutralization,_,base)>", "(/,reaction,_,(/,reaction,acid,_))", "(/,neutralization,_,base)" => "<(/,neutralization,_,base) <-> (/,reaction,_,(/,reaction,acid,_))>";
                "<(/,neutralization,_,base) --> (/,(*,acid,base),_,base)>", "(/,neutralization,_,(/,reaction,acid,_))", "(/,(*,acid,base),_,base)" => "<(/,neutralization,_,(/,reaction,acid,_)) <-> (/,(*,acid,base),_,base)>";
                "<(/,neutralization,_,base) --> (/,reaction,_,base)>", "(/,neutralization,_,base)", "acid" => "<acid <-> (/,neutralization,_,base)>";
                "<(/,neutralization,_,base) --> ?1>", "(/,(*,acid,base),_,base)", "?1" => "<?1 <-> (/,(*,acid,base),_,base)>";
                "<(/,neutralization,_,base) --> ?1>", "(/,neutralization,_,(/,reaction,acid,_))", "?1" => "<?1 <-> (/,neutralization,_,(/,reaction,acid,_))>";
                "<(/,neutralization,_,base) --> ?1>", "(/,reaction,_,base)", "?1" => "<?1 <-> (/,reaction,_,base)>";
                "<(/,open,_,lock) --> (/,open,_,{lock1})>", "(/,open,_,lock)", "{key1}" => "<{key1} <-> (/,open,_,lock)>";
                "<(/,reaction,(/,reaction,_,base),_) --> (/,reaction,acid,_)>", "(/,reaction,(/,reaction,_,base),_)", "base" => "<base <-> (/,reaction,(/,reaction,_,base),_)>";
                "<(\\,neutralization,_,base) --> acid>", "(/,reaction,_,base)", "acid" => "<acid <-> (/,reaction,_,base)>";
                "<(\\,neutralization,acid,_) --> (/,reaction,(/,reaction,_,base),_)>", "base", "(/,reaction,(/,reaction,_,base),_)" => "<base <-> (/,reaction,(/,reaction,_,base),_)>";
                "<(\\,neutralization,acid,_) --> (\\,reaction,acid,_)>", "(\\,neutralization,acid,_)", "(\\,reaction,acid,_)" => "<(\\,neutralization,acid,_) <-> (\\,reaction,acid,_)>";
                "<(\\,neutralization,acid,_) --> ?1>", "(/,reaction,(/,reaction,_,base),_)", "?1" => "<?1 <-> (/,reaction,(/,reaction,_,base),_)>";
                "<(\\,neutralization,acid,_) --> ?1>", "base", "?1" => "<?1 <-> base>";
                "<(\\,neutralization,acid,_) --> base>", "(/,reaction,(/,reaction,_,base),_)", "base" => "<base <-> (/,reaction,(/,reaction,_,base),_)>";
                "<(|,boy,girl) --> youth>", "(|,girl,[strong])", "youth" => "<youth <-> (|,girl,[strong])>";
                "<(|,robin,swan) --> (|,animal,robin)>", "(&,bird,swimmer)", "(|,animal,robin)" => "<(&,bird,swimmer) <-> (|,animal,robin)>";
                "<(|,robin,swan) --> (|,animal,robin)>", "(|,bird,robin)", "(|,animal,robin)" => "<(|,animal,robin) <-> (|,bird,robin)>";
                "<0 --> num>", "(/,num,_)", "num" => "<num <-> (/,num,_)>";
                "<?1 --> claimedByBob>", "?1", "(&,<bird --> fly>,<{Tweety} --> bird>)" => "<?1 <-> (&,<bird --> fly>,<{Tweety} --> bird>)>";
                "<?1 --> swimmer>", "?1", "animal" => "<?1 <-> animal>";
                "<[bright] --> [smart]>", "[bright]", "[smart]" => "<[bright] <-> [smart]>";
                "<bird --> (|,robin,swimmer)>", "bird", "(|,robin,swan)" => "<bird <-> (|,robin,swan)>";
                "<bird --> animal>", "bird", "robin" => "<bird <-> robin>";
                "<bird --> {Birdie}>", "bird", "[yellow]" => "<bird <-> [yellow]>";
                "<bird --> {Birdie}>", "bird", "{Tweety}" => "<bird <-> {Tweety}>";
                "<cat --> CAT>", "(/,(/,REPRESENT,_,<(*,CAT,FISH) --> FOOD>),_,eat,fish)", "CAT" => "<CAT <-> (/,(/,REPRESENT,_,<(*,CAT,FISH) --> FOOD>),_,eat,fish)>";
                "<planetX --> {Mars,Pluto,Saturn,Venus}>", "{Mars,Pluto,Venus}", "{Mars,Pluto,Saturn,Venus}" => "<{Mars,Pluto,Venus} <-> {Mars,Pluto,Saturn,Venus}>";
                "<planetX --> {Mars,Pluto,Saturn,Venus}>", "{Pluto,Saturn}", "{Mars,Pluto,Saturn,Venus}" => "<{Pluto,Saturn} <-> {Mars,Pluto,Saturn,Venus}>";
                "<planetX --> {Mars,Pluto,Venus}>", "{Mars,Venus}", "{Mars,Pluto,Venus}" => "<{Mars,Venus} <-> {Mars,Pluto,Venus}>";
                "<planetX --> {Mars,Venus}>", "{Mars,Pluto,Venus}", "{Mars,Venus}" => "<{Mars,Venus} <-> {Mars,Pluto,Venus}>";
                "<planetX --> {Mars,Venus}>", "{Pluto,Saturn}", "{Mars,Venus}" => "<{Mars,Venus} <-> {Pluto,Saturn}>";
                "<planetX --> {Pluto,Saturn}>", "{Mars,Pluto,Saturn,Venus}", "{Pluto,Saturn}" => "<{Pluto,Saturn} <-> {Mars,Pluto,Saturn,Venus}>";
                "<robin --> (&,bird,swimmer)>", "robin", "swan" => "<robin <-> swan>";
                "<robin --> (|,bird,swimmer)>", "robin", "swan" => "<robin <-> swan>";
                "<robin --> [chirping]>", "robin", "{Tweety}" => "<robin <-> {Tweety}>";
                "<robin --> [with_wings]>", "robin", "bird" => "<bird <-> robin>";
                "<swan --> animal>", "(|,bird,robin)", "animal" => "<animal <-> (|,bird,robin)>";
                "<tim --> (/,uncle,_,tom)>", "(/,uncle,tom,_)", "(/,uncle,_,tom)" => "<(/,uncle,tom,_) <-> (/,uncle,_,tom)>";
                "<tim --> (/,uncle,tom,_)>", "tim", "(/,(*,tim,tom),tom,_)" => "<tim <-> (/,(*,tim,tom),tom,_)>";
                "<{Birdie} --> [yellow]>", "bird", "[yellow]" => "<bird <-> [yellow]>";
                "<{Birdie} --> {Tweety}>", "{Birdie}", "{Tweety}" => "<{Birdie} <-> {Tweety}>";
                "<{Tweety} --> (&,[yellow],{Birdie})>", "bird", "(&,[yellow],{Birdie})" => "<bird <-> (&,[yellow],{Birdie})>";
                "<{Tweety} --> (&,bird,[yellow])>", "{Birdie}", "(&,bird,[yellow])" => "<{Birdie} <-> (&,bird,[yellow])>";
                "<{Tweety} --> (&,bird,{Birdie})>", "(|,bird,[yellow])", "(&,bird,{Birdie})" => "<(&,bird,{Birdie}) <-> (|,bird,[yellow])>";
                "<{Tweety} --> (|,bird,[yellow])>", "{Birdie}", "(|,bird,[yellow])" => "<{Birdie} <-> (|,bird,[yellow])>";
                "<{Tweety} --> [chirping]>", "{Tweety}", "robin" => "<robin <-> {Tweety}>";
                "<{Tweety} --> [yellow]>", "{Birdie}", "[yellow]" => "<[yellow] <-> {Birdie}>";
                "<{Tweety} --> bird>", "[with_wings]", "bird" => "<bird <-> [with_wings]>";
                "<{Tweety} --> bird>", "flyer", "bird" => "<bird <-> flyer>";
                "<{Tweety} --> bird>", "{Tweety}", "bird" => "<bird <-> {Tweety}>";
                "<{Tweety} --> {Birdie}>", "(&,bird,[yellow])", "{Birdie}" => "<{Birdie} <-> (&,bird,[yellow])>";
                "<{Tweety} --> {Birdie}>", "bird", "{Birdie}" => "<bird <-> {Birdie}>";
                "<{Tweety} --> {Birdie}>", "{Tweety}", "[yellow]" => "<[yellow] <-> {Tweety}>";
                "<{Tweety} --> {Birdie}>", "{Tweety}", "bird" => "<bird <-> {Tweety}>";
                "<{key1} --> key>", "(/,open,_,{lock1})", "key" => "<key <-> (/,open,_,{lock1})>";
                "<{lock1} --> (/,open,{key1},_)>", "lock", "(/,open,{key1},_)" => "<lock <-> (/,open,{key1},_)>";
            }
            ok!()
        }
    }
}
