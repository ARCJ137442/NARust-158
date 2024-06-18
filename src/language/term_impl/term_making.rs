//! 📄OpenNARS `nars.language.MakeTerm`
//! * 🎯复刻原OpenNARS 1.5.8的`make`系列方法
//! * 🚩构造词项前，
//!   * 检查其合法性
//!   * 简化其表达
//! * 🎯用于「制作词项」

use super::{vec_utils, CompoundTermRef, StatementRef, Term, TermComponents};
use crate::io::symbols::*;

impl Term {
    /* Word */

    /// 制作「词语」
    #[inline]
    pub fn make_word(name: impl Into<String>) -> Term {
        Term::new_word(name)
    }

    /* Variable */

    /// 制作「独立变量」
    #[inline]
    pub fn make_var_i(id: usize) -> Term {
        Term::new_var_i(id)
    }

    /// 制作「非独变量」
    #[inline]
    pub fn make_var_d(id: usize) -> Term {
        Term::new_var_d(id)
    }

    /// 制作「查询变量」
    #[inline]
    pub fn make_var_q(id: usize) -> Term {
        Term::new_var_q(id)
    }

    /// 制作「与现有变量类型相同」的变量
    /// * 🚩类型相同但编号不同
    /// * 🎯用于「变量推理」中的「重命名变量」
    #[inline]
    pub fn make_var_similar(from: &Term, id: impl Into<usize>) -> Term {
        Term::from_var_similar(from.identifier(), id)
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
        if term.instanceof_image_ext() {
            Self::make_image_ext_arg(components, template.get_placeholder_index())
        } else if term.instanceof_image_int() {
            Self::make_image_int_arg(components, template.get_placeholder_index())
        } else {
            Self::make_compound_term_from_identifier(&term.identifier, components)
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
                    Self::make_statement(statement, subject, predicate)
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
        for t in argument {
            // * 🚩尝试做交集：失败⇒返回空
            let new_term = make_arg(term, t)?;
            // * 🚩更新
            term = new_term;
        }
        // * 🚩返回
        Some(term)
    }

    /// * 🚩只依照集合数量进行化简
    fn make_intersection_vec(
        mut terms: Vec<Term>,
        new_intersection: fn(Vec<Term>) -> Term,
    ) -> Option<Term> {
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
    fn make_intersection_ext_arg(argument: Vec<Term>) -> Option<Term> {
        Self::make_intersection_arg(argument, Self::make_intersection_ext)
    }

    /// * 🚩只依照集合数量进行化简
    fn make_intersection_ext_vec(terms: Vec<Term>) -> Option<Term> {
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
    fn make_intersection_int_arg(argument: Vec<Term>) -> Option<Term> {
        Self::make_intersection_arg(argument, Self::make_intersection_int)
    }

    /// * 🚩只依照集合数量进行化简
    fn make_intersection_int_vec(terms: Vec<Term>) -> Option<Term> {
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

    fn make_product_arg(argument: Vec<Term>) -> Option<Term> {
        Some(Term::new_product(argument))
    }

    /// * 🚩从「外延像/内涵像」构造，用某个词项替换掉指定索引处的元素
    /// * 📝<a --> (/, R, _, b)> => <(*, a, b) --> R>，其中就要用 a 替换 [R,b] 中的R
    /// * ⚠️【2024-06-16 16:29:18】后续要留意其中与OpenNARS「占位符不作词项」逻辑的不同
    pub fn make_product(image: CompoundTermRef, component: &Term, index: usize) -> Option<Term> {
        let mut terms = vec![];
        let mut current_i = 0;
        for term in image.components {
            // * 🚩占位符⇒跳过
            if term.is_placeholder() {
                // ! ⚠️不递增索引：相当于「先移除占位符，再添加元素」
                continue;
            }
            // * 🚩模拟「替换词项」，但使用「惰性复制」的方式（被替换处的词项不会被复制）
            match current_i == index {
                // ! 📌只会复制一次，但编译器看不出这个假设，用所有权则报错"use of moved value: `component`"
                // ! 🚩【2024-06-16 16:36:16】目前解决方案：作为引用「惰性使用所有权」
                true => terms.push(component.clone()),
                false => terms.push(term.clone()),
            }
            current_i += 1;
        }
        // * 🚩制作 & 返回
        Self::make_product_arg(terms)
    }

    /* Image */

    /// * 📌作为模板的「像」提供「占位符位置」，但作为「组分」的`argument`可能没有占位符
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
        let relation = argument.remove(placeholder_index - 1);
        argument.insert(0, relation);
        // * 🚩再插入占位符
        // * 🎯处理edge case: "(/,num,_)", ["0"] => "(/,0,_)"
        if placeholder_index >= argument.len() {
            argument.push(Term::new_placeholder());
        }
        // * 🚩否则⇒插入
        else {
            argument.insert(placeholder_index, Term::new_placeholder());
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
    fn make_image_from_image(
        old_image: CompoundTermRef,
        component: &Term,
        index: usize,
        make_image_vec: fn(Vec<Term>) -> Option<Term>,
    ) -> Option<Term> {
        // * 🚩提取信息 | `old_placeholder_index`算入了「关系词项」
        let mut argument = vec![];
        let old_placeholder_index = old_image.get_placeholder_index();
        // * 🚩开始选择性添加词项（关系词项也算在内）
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
        make_image_vec(argument)
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
    fn make_image_ext_vec(argument: Vec<Term>) -> Option<Term> {
        Self::make_image_vec(argument, Term::new_image_ext)
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
    ) -> Option<Term> {
        // * 🚩现在统一在一个「『像』构造」逻辑中
        Self::make_image_from_image(old_image, component, index, Self::make_image_ext_vec)
    }

    /* ImageInt */

    fn make_image_int_arg(argument: Vec<Term>, placeholder_index: usize) -> Option<Term> {
        Self::make_image_arg(argument, placeholder_index, Self::make_image_int_vec)
    }

    fn make_image_int_vec(argument: Vec<Term>) -> Option<Term> {
        Self::make_image_vec(argument, Term::new_image_int)
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
    /// * 📄oldImage=`(\,(\,REPRESENT,_,<(*,CAT,FISH) --> FOOD>),_,eat,fish)`, component=`cat`, index=`2` => `(\,(\,REPRESENT,_,<(*,CAT,FISH) --> FOOD>),cat,eat,_)`
    /// * 📄oldImage=`(\,reaction,acid,_)`, component=`soda`, index=`0` => `(\,reaction,_,soda)`
    /// * 📄oldImage=`(\,(\,REPRESENT,_,<(*,$1,FISH) --> FOOD>),_,eat,fish)`, component=`(\,REPRESENT,_,$1)`, index=`2` => `(\,(\,REPRESENT,_,<(*,$1,FISH) --> FOOD>),(\,REPRESENT,_,$1),eat,_)`
    /// * 📄oldImage=`(\,neutralization,_,soda)`, component=`acid`, index=`1` => `(\,neutralization,acid,_)`
    /// * 📄oldImage=`(\,neutralization,acid,_)`, component=`$1`, index=`0` => `(\,neutralization,_,$1)`
    /// * 📄oldImage=`(\,REPRESENT,_,$1)`, component=`(\,(\,REPRESENT,_,<(*,$1,FISH) --> FOOD>),_,eat,fish)`, index=`1` => `(\,REPRESENT,(\,(\,REPRESENT,_,<(*,$1,FISH) --> FOOD>),_,eat,fish),_)`
    ///
    /// ℹ️更多例子详见单元测试用例
    pub fn make_image_int_from_image(
        old_image: CompoundTermRef,
        component: &Term,
        index: usize,
    ) -> Option<Term> {
        // * 🚩现在统一在一个「『像』构造」逻辑中
        Self::make_image_from_image(old_image, component, index, Self::make_image_int_vec)
    }

    /* Junction */

    /// 同时代表「从数组」与「从集合」
    fn make_junction_arg(
        mut argument: Vec<Term>,
        new_junction: fn(Vec<Term>) -> Term,
    ) -> Option<Term> {
        match argument.len() {
            // * 🚩不允许空集
            0 => None,
            // * 🚩单元素⇒直接用元素
            // special case: single component
            1 => argument.pop(),
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

    fn make_conjunction_arg(argument: Vec<Term>) -> Option<Term> {
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

    fn make_disjunction_arg(argument: Vec<Term>) -> Option<Term> {
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

    /* Statement */

    /// 从一个「陈述系词」中构造
    pub fn make_statement_relation(copula: &str, subject: Term, predicate: Term) -> Option<Term> {
        // * 🚩无效⇒制作失败
        if StatementRef::invalid_statement(&subject, &predicate) {
            return None;
        }
        // * 🚩按照「陈述系词」分派
        match copula {
            INHERITANCE_RELATION => Self::make_inheritance(subject, predicate),
            SIMILARITY_RELATION => Self::make_similarity(subject, predicate),
            INSTANCE_RELATION => Self::make_instance(subject, predicate),
            PROPERTY_RELATION => Self::make_property(subject, predicate),
            INSTANCE_PROPERTY_RELATION => Self::make_instance_property(subject, predicate),
            IMPLICATION_RELATION => Self::make_implication(subject, predicate),
            EQUIVALENCE_RELATION => Self::make_equivalence(subject, predicate),
            _ => None,
        }
    }

    pub fn make_statement(template: StatementRef, subject: Term, predicate: Term) -> Option<Term> {
        // * 🚩无效⇒制作失败
        if StatementRef::invalid_statement(&subject, &predicate) {
            return None;
        }
        // * 🚩按照「陈述系词」分派
        match template.identifier() {
            INHERITANCE_RELATION => Self::make_inheritance(subject, predicate),
            SIMILARITY_RELATION => Self::make_similarity(subject, predicate),
            IMPLICATION_RELATION => Self::make_implication(subject, predicate),
            EQUIVALENCE_RELATION => Self::make_equivalence(subject, predicate),
            // ! ↓这三者不会在实际中出现
            // INSTANCE_RELATION => Self::make_instance(subject, predicate),
            // PROPERTY_RELATION => Self::make_property(subject, predicate),
            // INSTANCE_PROPERTY_RELATION => Self::make_instance_property(subject, predicate),
            _ => None,
        }
    }

    /// 📄OpenNARS `Statement.makeSym`
    /// * 🚩通过使用「标识符映射」将「非对称版本」映射到「对称版本」
    /// * ⚠️目前只支持「继承」和「蕴含」，其它均会`panic`
    ///
    /// # 📄OpenNARS
    /// Make a symmetric Statement from given components and temporal information,
    /// called by the rules
    pub fn new_sym_statement(template: CompoundTermRef, subject: Term, predicate: Term) -> Self {
        let identifier = template.identifier();
        match identifier {
            // 继承⇒相似
            INHERITANCE_RELATION => Term::new_similarity(subject, predicate),
            // 蕴含⇒等价
            IMPLICATION_RELATION => Term::new_equivalence(subject, predicate),
            // 其它⇒panic
            _ => unimplemented!("不支持的标识符：{identifier:?}"),
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
}

impl CompoundTermRef<'_> {
    /// 删去元素
    /// * 🚩从复合词项中删去一个元素，或从同类复合词项中删除所有其内元素，然后尝试约简
    /// * ⚠️结果可空
    #[must_use]
    pub fn reduce_components(
        // to_be_reduce
        self,
        component_to_reduce: &Term,
    ) -> Option<Term> {
        let mut components = self.clone_components();
        // * 🚩试着作为复合词项
        let success = match (self.is_same_type(component_to_reduce), self.as_compound()) {
            // * 🚩同类⇒移除所有
            (
                true,
                Some(CompoundTermRef {
                    components: other_components,
                    ..
                }),
            ) => vec_utils::remove_all(&mut components, other_components),
            // * 🚩异类⇒作为元素移除
            _ => vec_utils::remove(&mut components, component_to_reduce),
        };
        if !success {
            return None;
        }
        // * 🚩尝试约简，或拒绝无效词项
        match components.len() {
            // * 🚩元素数量>1⇒以toBeReduce为模板构造新词项
            // * ✅此处的`self`是共享引用，实现了`Copy`特征
            2.. => Term::make_compound_term(self, components),
            // * 🚩元素数量=1⇒尝试「集合约简」
            1 => match Self::can_extract_to_inner(&self) {
                true => components.pop(),
                // ? 为何对「不可约简」的其它复合词项无效，如 (*, A) 就会返回null
                false => None,
            },
            // * 🚩空集⇒始终失败
            _ => None,
        }
    }

    /// 判断「只有一个元素的复合词项」是否与「内部元素」同义
    /// * 📌即判断该类复合词项是否能做「集合约简」
    /// * 🎯用于 `(&&, A) => A`、`(||, A) => A`等词项的简化
    ///   * ⚠️这个「词项」是在「约简之后」考虑的，
    ///   * 所以可能存在 `(-, A)` 等「整体不合法」的情况
    /// * 📄
    #[inline]
    fn can_extract_to_inner(&self) -> bool {
        matches!(
            self.identifier(),
            CONJUNCTION_OPERATOR
                | DISJUNCTION_OPERATOR
                | INTERSECTION_EXT_OPERATOR
                | INTERSECTION_INT_OPERATOR
                | DIFFERENCE_EXT_OPERATOR
                | DIFFERENCE_INT_OPERATOR
        )
    }

    /// 替换词项
    /// * 🚩替换指定索引处的词项，始终返回替换后的新词项
    /// * 🚩若要替换上的词项为空（⚠️t可空），则与「删除元素」等同
    /// * ⚠️结果可空
    #[must_use]
    pub fn set_component(
        compound: CompoundTermRef,
        index: usize,
        term: Option<Term>,
    ) -> Option<Term> {
        let mut list = compound.clone_components();
        list.remove(index);
        if let Some(term) = term {
            match (compound.is_same_type(&term), term.as_compound()) {
                // * 🚩同类⇒所有元素并入 | (*, 1, a)[1] = (*, 2, 3) => (*, 1, 2, 3)
                (
                    true,
                    Some(CompoundTermRef {
                        components: list2, ..
                    }),
                ) => {
                    // * 🚩【2024-06-16 12:20:14】此处选用惰性复制方法：先遍历再复制
                    for (i, term) in list2.iter().enumerate() {
                        list.insert(index + i, term.clone());
                    }
                }
                // * 🚩非同类⇒直接插入 | (&&, a, b)[1] = (||, b, c) => (&&, a, (||, b, c))
                _ => list.insert(index, term),
            }
        }
        // * 🚩以当前词项为模板构造新词项
        Term::make_compound_term(compound, list)
    }
}
#[cfg(test)]
mod tests {
    use super::*;
    use crate::{global::tests::AResult, ok, test_term as term};
    use nar_dev_utils::macro_once;

    /// 快捷构造[`Option<Term>`](Option)
    macro_rules! option_term {
        () => {
            None
        };
        (None) => {
            None
        };
        ($t:literal) => {
            Some(term!($t))
        };
    }

    /// 快捷格式化[`Option<Term>`](Option)
    fn format_option_term(ot: &Option<Term>) -> String {
        match ot {
            Some(t) => format!("Some(\"{t}\")"),
            None => "None".to_string(),
        }
    }

    /// 具体的词项构造
    /// * 📄外延集、内涵集……
    mod concrete_type {
        use super::*;

        /* SetExt */

        #[test]
        fn make_set_ext() -> AResult {
            macro_once! {
                // * 🚩模式：词项列表 ⇒ 预期词项
                macro test($($t:tt => $expected:tt;)*) {
                    $(
                        let out = Term::make_set_ext(term!($t));
                        let expected = option_term!($expected);
                        assert_eq!(out, expected);
                    )*
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
            macro_once! {
                // * 🚩模式：词项列表 ⇒ 预期词项
                macro test($($argument:tt => $expected:tt;)*) {
                    $(
                        let argument: Vec<_> = term!($argument).into();
                        let set = Term::make_set_ext_arg(argument);
                        let expected = option_term!($expected);
                        assert_eq!(set, expected);
                    )*
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
            macro_once! {
                // * 🚩模式：词项列表 ⇒ 预期词项
                macro test($($t:tt => $expected:expr;)*) {
                    $(
                        let out = Term::make_set_int(term!($t)).expect("解析词项失败！");
                        let expected = term!($expected);
                        assert_eq!(out, expected);
                    )*
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
            macro_once! {
                // * 🚩模式：词项列表 ⇒ 预期词项
                macro test($($argument:tt => $expected:tt;)*) {
                    $(
                        let argument: Vec<_> = term!($argument).into();
                        let set = Term::make_set_int_arg(argument);
                        let expected = option_term!($expected);
                        assert_eq!(set, expected);
                    )*
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
            macro_once! {
                // * 🚩模式：函数参数 ⇒ 预期词项
                macro test($($term1:tt, $term2:tt => $expected:tt;)*) {
                    $(
                        let term1 = term!($term1);
                        let term2 = term!($term2);
                        let out = Term::make_intersection_ext(term1.clone(), term2.clone());
                        let expected = option_term!($expected);
                        assert_eq!(
                            out, expected,
                            "{term1}, {term2} => {} != {}",
                            format_option_term(&out),format_option_term(&expected)
                        );
                    )*
                }
                // * ℹ️用例均源自OpenNARS实际运行
                // 集合之间的交集
                "{Pluto,Saturn}", "{Mars,Pluto,Venus}" => "{Pluto}";
                "{Mars,Pluto,Venus}", "{Pluto,Saturn}" => "{Pluto}";
                "[with_wings]", "[yellow]" => "[with_wings,yellow]";
                "[with_wings]", "[with_wings,yellow]" => "[with_wings,with_wings,yellow]";
                "[yellow]", "[with_wings]" => "[with_wings,yellow]";
                "[with_wings]", "[with_wings]" => "[with_wings,with_wings]";
                "[with_wings]", "[yellow]" => "[with_wings,yellow]";
                "[yellow]", "[with_wings]" => "[with_wings,yellow]";
                "{Mars,Venus}", "{Pluto,Saturn}" => None;
                "{Tweety}", "{Birdie}" => None;
                "{Pluto,Saturn}", "{Mars,Venus}" => None;
                // 其它情形
                "robin", "swan" => "(&,robin,swan)";
                "flyer", "{Birdie}" => "(&,flyer,{Birdie})";
                "{Birdie}", "bird" => "(&,bird,{Birdie})";
                "bird", "(|,#1,flyer)" => "(&,bird,(|,#1,flyer))";
                "#1", "bird" => "(&,#1,bird)";
                "(&,flyer,{Birdie})", "[yellow]" => "(&,flyer,[yellow],{Birdie})";
                "bird", "[yellow]" => "(&,bird,[yellow])";
                "chess", "sport" => "(&,chess,sport)";
                "bird", "{Birdie}" => "(&,bird,{Birdie})";
                "(|,bird,flyer)", "(|,bird,{Birdie})" => "(&,(|,bird,flyer),(|,bird,{Birdie}))";
                "swan", "robin" => "(&,robin,swan)";
                "(&,flyer,{Birdie})", "(&,bird,[yellow])" => "(&,bird,flyer,[yellow],{Birdie})";
                "robin", "bird" => "(&,bird,robin)";
                "robin", "{Tweety}" => "(&,robin,{Tweety})";
                "bird", "[with-wings]" => "(&,bird,[with-wings])";
                "bird", "animal" => "(&,animal,bird)";
                "bird", "swan" => "(&,bird,swan)";
                "competition", "sport" => "(&,competition,sport)";
                "flyer", "[yellow]" => "(&,flyer,[yellow])";
                "flyer", "#1" => "(&,#1,flyer)";
                "bird", "tiger" => "(&,bird,tiger)";
                "#1", "{Tweety}" => "(&,#1,{Tweety})";
                "<{Tweety} --> bird>", "<bird --> fly>" => "(&,<bird --> fly>,<{Tweety} --> bird>)";
                "swimmer", "animal" => "(&,animal,swimmer)";
                "(&,bird,{Birdie})", "[yellow]" => "(&,bird,[yellow],{Birdie})";
                "flyer", "(&,bird,[yellow])" => "(&,bird,flyer,[yellow])";
                "{Birdie}", "[with-wings]" => "(&,[with-wings],{Birdie})";
                "flyer", "[with-wings]" => "(&,flyer,[with-wings])";
                "#1", "{Birdie}" => "(&,#1,{Birdie})";
                "chess", "competition" => "(&,chess,competition)";
                "[strong]", "(~,youth,girl)" => "(&,[strong],(~,youth,girl))";
                "robin", "swimmer" => "(&,robin,swimmer)";
                "sport", "chess" => "(&,chess,sport)";
                "bird", "flyer" => "(&,bird,flyer)";
                "swimmer", "bird" => "(&,bird,swimmer)";
                "animal", "bird" => "(&,animal,bird)";
                "swan", "swimmer" => "(&,swan,swimmer)";
                "flyer", "(&,bird,{Birdie})" => "(&,bird,flyer,{Birdie})";
                "flyer", "bird" => "(&,bird,flyer)";
                "bird", "swimmer" => "(&,bird,swimmer)";
                "(|,flyer,{Birdie})", "[with-wings]" => "(&,[with-wings],(|,flyer,{Birdie}))";
                "animal", "swimmer" => "(&,animal,swimmer)";
                "key", "{key1}" => "(&,key,{key1})";
                "{Birdie}", "[with_wings]" => "(&,[with_wings],{Birdie})";
                "bird", "#1" => "(&,#1,bird)";
                "robin", "tiger" => "(&,robin,tiger)";
                "swimmer", "robin" => "(&,robin,swimmer)";
                "(|,flyer,{Birdie})", "(|,#1,flyer)" => "(&,(|,#1,flyer),(|,flyer,{Birdie}))";
                "(|,bird,flyer)", "#1" => "(&,#1,(|,bird,flyer))";
                "bird", "{Tweety}" => "(&,bird,{Tweety})";
                "robin", "{Birdie}" => "(&,robin,{Birdie})";
                "swan", "bird" => "(&,bird,swan)";
                "bird", "robin" => "(&,bird,robin)";
                "#1", "{lock1}" => "(&,#1,{lock1})";
                "{Tweety}", "#1" => "(&,#1,{Tweety})";
                "(|,bird,flyer)", "(|,bird,{Tweety})" => "(&,(|,bird,flyer),(|,bird,{Tweety}))";
                "lock1", "#1" => "(&,#1,lock1)";
                "[yellow]", "bird" => "(&,bird,[yellow])";
                "(&,bird,{Birdie})", "flyer" => "(&,bird,flyer,{Birdie})";
            }
            ok!()
        }

        /* IntersectionInt */
        #[test]
        fn make_intersection_int() -> AResult {
            macro_once! {
                // * 🚩模式：函数参数 ⇒ 预期词项
                macro test($($term1:tt, $term2:tt => $expected:tt;)*) {
                    $(
                        let term1 = term!($term1);
                        let term2 = term!($term2);
                        let out = Term::make_intersection_int(term1.clone(), term2.clone());
                        let expected = option_term!($expected);
                        assert_eq!(
                            out, expected,
                            "{term1}, {term2} => {} != {}",
                            format_option_term(&out),format_option_term(&expected)
                        );
                    )*
                }
                // * ℹ️用例均源自OpenNARS实际运行"(|,flyer,{Tweety})", "{Birdie}" => "(|,flyer,{Birdie},{Tweety})";
                "(|,#1,bird)", "{Birdie}" => "(|,#1,bird,{Birdie})";
                "[with_wings]", "[yellow]" => None;
                "animal", "bird" => "(|,animal,bird)";
                "[with-wings]", "{Tweety}" => "(|,[with-wings],{Tweety})";
                "{Tweety}", "#1" => "(|,#1,{Tweety})";
                "(&,#1,{lock1})", "lock1" => "(|,lock1,(&,#1,{lock1}))";
                "{Mars,Venus}", "{Pluto,Saturn}" => "{Mars,Pluto,Saturn,Venus}";
                "neutralization", "reaction" => "(|,neutralization,reaction)";
                "[strong]", "(~,youth,girl)" => "(|,[strong],(~,youth,girl))";
                "robin", "[with-wings]" => "(|,robin,[with-wings])";
                "robin", "{Tweety}" => "(|,robin,{Tweety})";
                "[with_wings]", "{Birdie}" => "(|,[with_wings],{Birdie})";
                "bird", "(&,bird,{Birdie})" => "(|,bird,(&,bird,{Birdie}))";
                "bird", "tiger" => "(|,bird,tiger)";
                "(|,flyer,[with_wings])", "{Birdie}" => "(|,flyer,[with_wings],{Birdie})";
                "boy", "girl" => "(|,boy,girl)";
                "chess", "(|,chess,sport)" => "(|,chess,sport)";
                "(&,flyer,{Birdie})", "[yellow]" => "(|,[yellow],(&,flyer,{Birdie}))";
                "sport", "competition" => "(|,competition,sport)";
                "flyer", "(|,bird,flyer)" => "(|,bird,flyer)";
                "bird", "{Birdie}" => "(|,bird,{Birdie})";
                "(&,bird,{Birdie})", "[yellow]" => "(|,[yellow],(&,bird,{Birdie}))";
                "flyer", "[with_wings]" => "(|,flyer,[with_wings])";
                "flyer", "[with-wings]" => "(|,flyer,[with-wings])";
                "robin", "(|,#1,{Birdie})" => "(|,#1,robin,{Birdie})";
                "(|,flyer,{Birdie})", "[with-wings]" => "(|,flyer,[with-wings],{Birdie})";
                "(|,bird,robin)", "{Birdie}" => "(|,bird,robin,{Birdie})";
                "#1", "{lock1}" => "(|,#1,{lock1})";
                "{Birdie}", "bird" => "(|,bird,{Birdie})";
                "swimmer", "animal" => "(|,animal,swimmer)";
                "(~,boy,girl)", "(~,youth,girl)" => "(|,(~,boy,girl),(~,youth,girl))";
                "[with-wings]", "(|,bird,flyer)" => "(|,bird,flyer,[with-wings])";
                "bird", "flyer" => "(|,bird,flyer)";
                "(&,flyer,{Birdie})", "(&,bird,{Birdie})" => "(|,(&,bird,{Birdie}),(&,flyer,{Birdie}))";
                "#1", "(&,bird,{Birdie})" => "(|,#1,(&,bird,{Birdie}))";
                "robin", "[yellow]" => "(|,robin,[yellow])";
                "{Tweety}", "{Birdie}" => "{Birdie,Tweety}";
                "#1", "robin" => "(|,#1,robin)";
                "(&,[with-wings],{Birdie})", "(&,bird,flyer)" => "(|,(&,bird,flyer),(&,[with-wings],{Birdie}))";
                "[with_wings]", "(|,bird,{Birdie})" => "(|,bird,[with_wings],{Birdie})";
                "competition", "chess" => "(|,chess,competition)";
                "[with-wings]", "(&,bird,[yellow])" => "(|,[with-wings],(&,bird,[yellow]))";
                "[with_wings]", "[with-wings]" => None;
                "bird", "(|,flyer,[with-wings])" => "(|,bird,flyer,[with-wings])";
                "flyer", "(&,bird,[yellow])" => "(|,flyer,(&,bird,[yellow]))";
                "{Birdie}", "(|,[with_wings],(&,bird,[with-wings]))" => "(|,[with_wings],{Birdie},(&,bird,[with-wings]))";
                "chess", "competition" => "(|,chess,competition)";
                "[with-wings]", "{Birdie}" => "(|,[with-wings],{Birdie})";
                "swan", "bird" => "(|,bird,swan)";
                "(|,bird,flyer)", "(|,bird,{Birdie})" => "(|,bird,flyer,{Birdie})";
                "[with-wings]", "[with_wings,yellow]" => None;
                "{Pluto,Saturn}", "{Mars,Pluto,Venus}" => "{Mars,Pluto,Saturn,Venus}";
                "flyer", "[yellow]" => "(|,flyer,[yellow])";
                "flyer", "{Birdie}" => "(|,flyer,{Birdie})";
                "bird", "robin" => "(|,bird,robin)";
                "bird", "animal" => "(|,animal,bird)";
                "(|,bird,flyer)", "{Birdie}" => "(|,bird,flyer,{Birdie})";
                "animal", "swimmer" => "(|,animal,swimmer)";
                "robin", "swimmer" => "(|,robin,swimmer)";
                "bird", "(|,#1,flyer)" => "(|,#1,bird,flyer)";
                "{Birdie}", "[with_wings]" => "(|,[with_wings],{Birdie})";
                "swan", "animal" => "(|,animal,swan)";
                "(&,bird,{Birdie})", "flyer" => "(|,flyer,(&,bird,{Birdie}))";
                "boy", "(~,youth,girl)" => "(|,boy,(~,youth,girl))";
                "#1", "{Tweety}" => "(|,#1,{Tweety})";
                "#1", "bird" => "(|,#1,bird)";
                "[with_wings]", "(&,bird,{Birdie})" => "(|,[with_wings],(&,bird,{Birdie}))";
                "flyer", "(&,bird,{Birdie})" => "(|,flyer,(&,bird,{Birdie}))";
                "bird", "{Tweety}" => "(|,bird,{Tweety})";
                "robin", "bird" => "(|,bird,robin)";
                "{Mars,Pluto,Venus}", "{Pluto,Saturn}" => "{Mars,Pluto,Saturn,Venus}";
                "(&,flyer,{Birdie})", "(&,bird,[yellow])" => "(|,(&,bird,[yellow]),(&,flyer,{Birdie}))";
                "robin", "animal" => "(|,animal,robin)";
                "[with-wings]", "(&,bird,flyer)" => "(|,[with-wings],(&,bird,flyer))";
                "robin", "swan" => "(|,robin,swan)";
                "robin", "#1" => "(|,#1,robin)";
                "chess", "sport" => "(|,chess,sport)";
                "robin", "tiger" => "(|,robin,tiger)";
                "youth", "girl" => "(|,girl,youth)";
                "bird", "(&,flyer,{Birdie})" => "(|,bird,(&,flyer,{Birdie}))";
                "swimmer", "bird" => "(|,bird,swimmer)";
                "bird", "(|,bird,flyer)" => "(|,bird,flyer)";
                "lock1", "#1" => "(|,#1,lock1)";
                "robin", "(&,bird,[with-wings])" => "(|,robin,(&,bird,[with-wings]))";
                "bird", "swimmer" => "(|,bird,swimmer)";
                "flyer", "(&,bird,[with-wings])" => "(|,flyer,(&,bird,[with-wings]))";
                "flyer", "bird" => "(|,bird,flyer)";
                "swimmer", "robin" => "(|,robin,swimmer)";
                "bird", "swan" => "(|,bird,swan)";
                "swan", "robin" => "(|,robin,swan)";
                "flyer", "#1" => "(|,#1,flyer)";
                "(|,#1,flyer)", "{Tweety}" => "(|,#1,flyer,{Tweety})";
                "robin", "{Birdie}" => "(|,robin,{Birdie})";
                "(|,bird,flyer)", "#1" => "(|,#1,bird,flyer)";
                "[with-wings]", "(&,bird,{Birdie})" => "(|,[with-wings],(&,bird,{Birdie}))";
                "[yellow]", "bird" => "(|,bird,[yellow])";
                "(|,flyer,{Birdie})", "(|,#1,flyer)" => "(|,#1,flyer,{Birdie})";
                "{Birdie}", "[with-wings]" => "(|,[with-wings],{Birdie})";
                "(|,[with-wings],(&,bird,[yellow]))", "flyer" => "(|,flyer,[with-wings],(&,bird,[yellow]))";
                "bird", "#1" => "(|,#1,bird)";
                "[with_wings]", "bird" => "(|,bird,[with_wings])";
                "bird", "[yellow]" => "(|,bird,[yellow])";
                "{key1}", "key" => "(|,key,{key1})";
                "flyer", "(&,flyer,{Birdie})" => "(|,flyer,(&,flyer,{Birdie}))";
                "[with_wings]", "(&,bird,[with-wings])" => "(|,[with_wings],(&,bird,[with-wings]))";
                "#1", "lock1" => "(|,#1,lock1)";
                "flyer", "{Tweety}" => "(|,flyer,{Tweety})";
                "[with-wings]", "#1" => "(|,#1,[with-wings])";
                "#1", "{Birdie}" => "(|,#1,{Birdie})";
                "competition", "sport" => "(|,competition,sport)";
                "sport", "chess" => "(|,chess,sport)";
                "bird", "[with-wings]" => "(|,bird,[with-wings])";
            }
            ok!()
        }

        /* DifferenceExt */

        #[test]
        fn make_difference_ext_arg() -> AResult {
            macro_once! {
                // * 🚩模式：词项列表 ⇒ 预期词项
                macro test($($arg_list:tt => $expected:expr;)*) {
                    $(
                        let arg_list: Vec<_> = term!($arg_list).into();
                        let out = Term::make_difference_ext_arg(arg_list).expect("解析词项失败！");
                        let expected = term!($expected);
                        assert_eq!(out, expected);
                    )*
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
            macro_once! {
                // * 🚩模式：词项列表 ⇒ 预期词项
                macro test($($term1:tt, $term2:tt => $expected:expr;)*) {
                    $(
                        let term1 = term!($term1);
                        let term2 = term!($term2);
                        let out = Term::make_difference_ext(term1.clone(), term2.clone());
                        let expected = option_term!($expected);
                        assert_eq!(
                            out, expected,
                            "{term1}, {term2} => {} != {}",
                            format_option_term(&out), format_option_term(&expected)
                        );
                    )*
                }
                // * ℹ️用例均源自OpenNARS实际运行
                "[yellow]", "bird" => "(-,[yellow],bird)";
                "(|,bird,{Birdie})", "[with_wings]" => "(-,(|,bird,{Birdie}),[with_wings])";
                "bird", "[yellow]" => "(-,bird,[yellow])";
                "bird", "[with_wings]" => "(-,bird,[with_wings])";
                "[yellow]", "{Birdie}" => "(-,[yellow],{Birdie})";
                "(|,[yellow],{Birdie})", "flyer" => "(-,(|,[yellow],{Birdie}),flyer)";
                "(|,chess,competition)", "(|,competition,sport)" => "(-,(|,chess,competition),(|,competition,sport))";
                "{Mars,Pluto,Venus}", "{Pluto,Saturn}" => "{Mars,Venus}";
                "(|,[yellow],{Birdie})", "bird" => "(-,(|,[yellow],{Birdie}),bird)";
                "swan", "swimmer" => "(-,swan,swimmer)";
                "(|,flyer,{Birdie})", "[with_wings]" => "(-,(|,flyer,{Birdie}),[with_wings])";
                "swan", "flyer" => "(-,swan,flyer)";
                "(|,[yellow],{Birdie})", "[with_wings]" => "(-,(|,[yellow],{Birdie}),[with_wings])";
                "robin", "bird" => "(-,robin,bird)";
                "[yellow]", "[with_wings]" => "(-,[yellow],[with_wings])";
                "swimmer", "swan" => "(-,swimmer,swan)";
                "bird", "swimmer" => "(-,bird,swimmer)";
                "{Birdie}", "flyer" => "(-,{Birdie},flyer)";
                "(&,bird,flyer)", "[with_wings]" => "(-,(&,bird,flyer),[with_wings])";
                "(/,open,_,#1)", "(/,open,_,{lock1})" => "(-,(/,open,_,#1),(/,open,_,{lock1}))";
                "flyer", "[with_wings]" => "(-,flyer,[with_wings])";
                "swan", "animal" => "(-,swan,animal)";
                "(&,bird,(|,[yellow],{Birdie}))", "[with_wings]" => "(-,(&,bird,(|,[yellow],{Birdie})),[with_wings])";
                "bird", "flyer" => "(-,bird,flyer)";
                "mammal", "swimmer" => "(-,mammal,swimmer)";
                "(|,flyer,[yellow])", "{Birdie}" => "(-,(|,flyer,[yellow]),{Birdie})";
                "(&,flyer,{Birdie})", "[with_wings]" => "(-,(&,flyer,{Birdie}),[with_wings])";
                "swimmer", "animal" => "(-,swimmer,animal)";
                "(|,flyer,[with_wings])", "[yellow]" => "(-,(|,flyer,[with_wings]),[yellow])";
                "animal", "swimmer" => "(-,animal,swimmer)";
                "bird", "animal" => "(-,bird,animal)";
                "(|,bird,flyer)", "[with_wings]" => "(-,(|,bird,flyer),[with_wings])";
                "{Birdie}", "[with_wings]" => "(-,{Birdie},[with_wings])";
                "(|,bird,swimmer)", "animal" => "(-,(|,bird,swimmer),animal)";
                "(|,flyer,[yellow])", "[with_wings]" => "(-,(|,flyer,[yellow]),[with_wings])";
                "(&,flyer,[yellow])", "[with_wings]" => "(-,(&,flyer,[yellow]),[with_wings])";
                "(|,bird,{Birdie})", "[yellow]" => "(-,(|,bird,{Birdie}),[yellow])";
                "swimmer", "bird" => "(-,swimmer,bird)";
                "swan", "bird" => "(-,swan,bird)";
                "robin", "animal" => "(-,robin,animal)";
            }
            ok!()
        }

        /* DifferenceInt */

        #[test]
        fn make_difference_int_arg() -> AResult {
            macro_once! {
                // * 🚩模式：词项列表 ⇒ 预期词项
                macro test($($arg_list:tt => $expected:expr;)*) {
                    $(
                        let arg_list: Vec<_> = term!($arg_list).into();
                        let out = Term::make_difference_int_arg(arg_list).expect("解析词项失败！");
                        let expected = term!($expected);
                        assert_eq!(out, expected);
                    )*
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
            macro_once! {
                // * 🚩模式：词项列表 ⇒ 预期词项
                macro test($($term1:tt, $term2:tt => $expected:expr;)*) {
                    $(
                        let term1 = term!($term1);
                        let term2 = term!($term2);
                        let out = Term::make_difference_int(term1.clone(), term2.clone());
                        let expected = option_term!($expected);
                        assert_eq!(
                            out, expected,
                            "{term1}, {term2} => {} != {}",
                            format_option_term(&out), format_option_term(&expected)
                        );
                    )*
                }
                // * ℹ️用例均源自OpenNARS实际运行
                "{Birdie}", "(|,flyer,robin)" => "(~,{Birdie},(|,flyer,robin))";
                "{Tweety}", "(|,flyer,robin)" => "(~,{Tweety},(|,flyer,robin))";
                "swimmer", "bird" => "(~,swimmer,bird)";
                "bird", "robin" => "(~,bird,robin)";
                "tiger", "swan" => "(~,tiger,swan)";
                "sport", "chess" => "(~,sport,chess)";
                "robin", "bird" => "(~,robin,bird)";
                "(&,flyer,{Tweety})", "robin" => "(~,(&,flyer,{Tweety}),robin)";
                "(/,open,_,lock)", "{key1}" => "(~,(/,open,_,lock),{key1})";
                "swan", "robin" => "(~,swan,robin)";
                "tiger", "robin" => "(~,tiger,robin)";
                "{Tweety}", "robin" => "(~,{Tweety},robin)";
                "(&,flyer,{Birdie})", "(&,flyer,robin)" => "(~,(&,flyer,{Birdie}),(&,flyer,robin))";
                "boy", "girl" => "(~,boy,girl)";
                "animal", "robin" => "(~,animal,robin)";
                "(/,(*,tim,tom),tom,_)", "(/,uncle,tom,_)" => "(~,(/,(*,tim,tom),tom,_),(/,uncle,tom,_))";
                "bird", "(|,robin,tiger)" => "(~,bird,(|,robin,tiger))";
                "(/,(*,tim,tom),tom,_)", "tim" => "(~,(/,(*,tim,tom),tom,_),tim)";
                "(&,bird,robin)", "tiger" => "(~,(&,bird,robin),tiger)";
                "youth", "girl" => "(~,youth,girl)";
                "(|,flyer,[with_wings],{Birdie})", "robin" => "(~,(|,flyer,[with_wings],{Birdie}),robin)";
                "(|,bird,robin)", "tiger" => "(~,(|,bird,robin),tiger)";
                "(&,flyer,{Tweety})", "(&,flyer,robin)" => "(~,(&,flyer,{Tweety}),(&,flyer,robin))";
                "swan", "bird" => "(~,swan,bird)";
                "swan", "tiger" => "(~,swan,tiger)";
                "swimmer", "swan" => "(~,swimmer,swan)";
                "chess", "sport" => "(~,chess,sport)";
                "tiger", "bird" => "(~,tiger,bird)";
                "(&,flyer,{Birdie})", "robin" => "(~,(&,flyer,{Birdie}),robin)";
                "(|,boy,girl)", "girl" => "(~,(|,boy,girl),girl)";
                "tiger", "swimmer" => "(~,tiger,swimmer)";
                "flyer", "robin" => "(~,flyer,robin)";
                "{Tweety}", "(&,flyer,robin)" => "(~,{Tweety},(&,flyer,robin))";
                "swimmer", "robin" => "(~,swimmer,robin)";
                "animal", "bird" => "(~,animal,bird)";
                "bird", "#1" => "(~,bird,#1)";
                "{lock1}", "#1" => "(~,{lock1},#1)";
                "{Birdie}", "robin" => "(~,{Birdie},robin)";
                "(~,boy,girl)", "girl" => "(~,(~,boy,girl),girl)";
                "{Tweety}", "(|,robin,[yellow],{Birdie})" => "(~,{Tweety},(|,robin,[yellow],{Birdie}))";
                "swimmer", "tiger" => "(~,swimmer,tiger)";
                "swimmer", "#1" => "(~,swimmer,#1)";
                "[strong]", "girl" => "(~,[strong],girl)";
                "(|,flyer,{Birdie})", "robin" => "(~,(|,flyer,{Birdie}),robin)";
            }
            ok!()
        }

        /* ImageExt */

        #[test]
        fn make_image_ext_vec() -> AResult {
            macro_once! {
                // * 🚩模式：词项列表 ⇒ 预期词项
                macro test($($arg_list:tt => $expected:expr;)*) {
                    $(
                        let arg_list: Vec<_> = term!($arg_list).into();
                        let image = Term::make_image_ext_vec(arg_list).expect("解析词项失败！");
                        let expected = term!($expected);
                        assert_eq!(image, expected);
                    )*
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
            macro_once! {
                // * 🚩模式：词项列表 ⇒ 预期词项
                macro test($($product:tt, $relation:tt, $index:tt => $expected:expr;)*) {
                    $(
                        let p = term!($product);
                        let product = p.as_compound().expect("解析出的不是复合词项！");
                        let relation = term!($relation);
                        let index = $index;
                        let image = Term::make_image_ext_from_product(product, &relation, index).expect("词项制作失败！");
                        let expected = term!($expected);
                        assert_eq!(image, expected, "{product}, {relation}, {index} => {image} != {expected}");
                    )*
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
            macro_once! {
                // * 🚩模式：词项列表 ⇒ 预期词项
                macro test($($image:tt, $component:tt, $index:tt => $expected:expr;)*) {
                    $(
                        let i = term!($image);
                        let image = i.as_compound().expect("解析出的不是复合词项！");
                        let component = term!($component);
                        let index = $index;
                        let image = Term::make_image_ext_from_image(image, &component, index).expect("词项制作失败！");
                        let expected = term!($expected);
                        assert_eq!(image, expected, "{image}, {component}, {index} => {image} != {expected}");
                    )*
                }
                // * ℹ️用例均源自OpenNARS实际运行
                "(/,open,{key1},_)",   "lock",   0 => "(/,open,_,lock)";
                "(/,uncle,_,tom)",     "tim",    1 => "(/,uncle,tim,_)";
                "(/,open,{key1},_)",   "$2",     0 => "(/,open,_,$2)";
                "(/,open,{key1},_)",   "#1",     0 => "(/,open,_,#1)";
                "(/,like,_,a)",        "b",      1 => "(/,like,b,_)";
                "(/,like,b,_)",        "a",      0 => "(/,like,_,a)";
            }
            ok!()
        }

        /* ImageInt */

        #[test]
        fn make_image_int_vec() -> AResult {
            macro_once! {
                // * 🚩模式：词项列表 ⇒ 预期词项
                macro test($($arg_list:tt => $expected:expr;)*) {
                    $(
                        let arg_list: Vec<_> = term!($arg_list).into();
                        let image = Term::make_image_int_vec(arg_list).expect("解析词项失败！");
                        let expected = term!($expected);
                        assert_eq!(image, expected);
                    )*
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
            macro_once! {
                // * 🚩模式：词项列表 ⇒ 预期词项
                macro test($($product:tt, $relation:tt, $index:tt => $expected:expr;)*) {
                    $(
                        let p = term!($product);
                        let product = p.as_compound().expect("解析出的不是复合词项！");
                        let relation = term!($relation);
                        let index = $index;
                        let image = Term::make_image_int_from_product(product, &relation, index).expect("词项制作失败！");
                        let expected = term!($expected);
                        assert_eq!(image, expected, "{product}, {relation}, {index} => {image} != {expected}");
                    )*
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
            macro_once! {
                // * 🚩模式：词项列表 ⇒ 预期词项
                macro test($($image:tt, $component:tt, $index:tt => $expected:expr;)*) {
                    $(
                        let i = term!($image);
                        let image = i.as_compound().expect("解析出的不是复合词项！");
                        let component = term!($component);
                        let index = $index;
                        let image = Term::make_image_int_from_image(image, &component, index).expect("词项制作失败！");
                        let expected = term!($expected);
                        assert_eq!(image, expected, "{image}, {component}, {index} => {image} != {expected}");
                    )*
                }
                // * ℹ️用例均源自OpenNARS实际运行
                r"(\,R,_,eat,fish)",           "cat",                       2 => r"(\,R,cat,eat,_)";
                r"(\,reaction,acid,_)",        "soda",                      0 => r"(\,reaction,_,soda)";
                r"(\,R,_,eat,fish)",          r"(\,REPRESENT,_,$1)",        2 => r"(\,R,(\,REPRESENT,_,$1),eat,_)";
                r"(\,neutralization,_,soda)",  "acid",                      1 => r"(\,neutralization,acid,_)";
                r"(\,neutralization,acid,_)",  "$1",                        0 => r"(\,neutralization,_,$1)";
                r"(\,REPRESENT,_,$1)",        r"(\,R,_,eat,fish)",          1 => r"(\,REPRESENT,(\,R,_,eat,fish),_)";
                r"(\,neutralization,acid,_)",  "soda",                      0 => r"(\,neutralization,_,soda)";
                r"(\,neutralization,acid,_)",  "?1",                        0 => r"(\,neutralization,_,?1)";
                r"(\,reaction,acid,_)",       r"(\,neutralization,acid,_)", 0 => r"(\,reaction,_,(\,neutralization,acid,_))";
                r"(\,REPRESENT,_,CAT)",        "(/,R,_,eat,fish)",          1 => r"(\,REPRESENT,(/,R,_,eat,fish),_)";
                r"(\,R,_,eat,fish)",          r"(\,REPRESENT,_,$1)",        1 => r"(\,R,(\,REPRESENT,_,$1),_,fish)";
                r"(\,R,_,eat,fish)",           "cat",                       1 => r"(\,R,cat,_,fish)";
                r"(\,reaction,_,soda)",        "acid",                      1 => r"(\,reaction,acid,_)";
                r"(\,reaction,_,base)",       r"(\,reaction,_,soda)",       1 => r"(\,reaction,(\,reaction,_,soda),_)";
                r"(\,neutralization,acid,_)",  "#1",                        0 => r"(\,neutralization,_,#1)";
                r"(\,neutralization,acid,_)",  "base",                      0 => r"(\,neutralization,_,base)";
                r"(\,reaction,_,base)",        "acid",                      1 => r"(\,reaction,acid,_)";
                r"(\,neutralization,acid,_)",  "(/,reaction,acid,_)",       0 => r"(\,neutralization,_,(/,reaction,acid,_))";
            }
            ok!()
        }
    }

    mod compound {
        use super::*;

        #[test]
        fn make_compound_term_from_identifier() -> AResult {
            macro_once! {
                // * 🚩模式：参数列表 ⇒ 预期词项
                macro test($($identifier:tt, $terms:tt => $expected:tt;)*) {
                    $(
                        let identifier = $identifier;
                        let terms: Vec<Term> = term!($terms).into();
                        let terms_str = terms.iter().map(|t| format!("\"{t}\"")).collect::<Vec<_>>().join(", ");
                        let out = Term::make_compound_term_from_identifier(
                            identifier,
                            terms
                        );
                        let expected = option_term!($expected);
                        assert_eq!(
                            out, expected,
                            "{identifier:?}, {terms_str} => {} != {}",
                            format_option_term(&out),
                            format_option_term(&expected),
                        );
                    )*
                }
                // * ℹ️用例均源自OpenNARS实际运行
                "&", ["(&,robin,{Tweety})", "{Birdie}"] => "(&,robin,{Birdie},{Tweety})";
                "&", ["(/,neutralization,_,(\\,neutralization,acid,_))", "acid"] => "(&,acid,(/,neutralization,_,(\\,neutralization,acid,_)))";
                "&", ["(/,neutralization,_,base)", "(/,reaction,_,base)"] => "(&,(/,neutralization,_,base),(/,reaction,_,base))";
                "&", ["(/,neutralization,_,base)", "acid"] => "(&,acid,(/,neutralization,_,base))";
                "&", ["(/,open,_,lock)", "key"] => "(&,key,(/,open,_,lock))";
                "&", ["(/,open,_,{lock1})", "(/,open,_,lock)"] => "(&,(/,open,_,lock),(/,open,_,{lock1}))";
                "&", ["(/,reaction,_,soda)", "(/,reaction,_,base)"] => "(&,(/,reaction,_,base),(/,reaction,_,soda))";
                "&", ["(\\,REPRESENT,_,CAT)", "(/,(/,REPRESENT,_,<(*,CAT,FISH) --> FOOD>),_,eat,fish)"] => "(&,(\\,REPRESENT,_,CAT),(/,(/,REPRESENT,_,<(*,CAT,FISH) --> FOOD>),_,eat,fish))";
                "&", ["(\\,reaction,_,soda)", "(\\,neutralization,_,base)"] => "(&,(\\,neutralization,_,base),(\\,reaction,_,soda))";
                "&", ["(|,(/,open,_,lock1),(/,open,_,{lock1}))", "(/,open,_,lock)"] => "(&,(/,open,_,lock),(|,(/,open,_,lock1),(/,open,_,{lock1})))";
                "&", ["(|,bird,{Tweety})", "(|,bird,{Birdie})"] => "(&,(|,bird,{Birdie}),(|,bird,{Tweety}))";
                "&", ["(|,key,(/,open,_,{lock1}))", "(/,open,_,lock)"] => "(&,(/,open,_,lock),(|,key,(/,open,_,{lock1})))";
                "&", ["acid", "(/,reaction,_,base)"] => "(&,acid,(/,reaction,_,base))";
                "&", ["acid", "(\\,neutralization,_,base)"] => "(&,acid,(\\,neutralization,_,base))";
                "&", ["acid", "(\\,neutralization,_,soda)"] => "(&,acid,(\\,neutralization,_,soda))";
                "&", ["animal", "(&,robin,swan)"] => "(&,animal,robin,swan)";
                "&", ["animal", "(|,animal,swimmer)"] => "(&,animal,(|,animal,swimmer))";
                "&", ["animal", "gull"] => "(&,animal,gull)";
                "&", ["animal", "swan"] => "(&,animal,swan)";
                "&", ["animal", "swimmer"] => "(&,animal,swimmer)";
                "&", ["base", "(/,reaction,acid,_)"] => "(&,base,(/,reaction,acid,_))";
                "&", ["base", "(\\,neutralization,acid,_)"] => "(&,base,(\\,neutralization,acid,_))";
                "&", ["base", "soda"] => "(&,base,soda)";
                "&", ["bird", "animal"] => "(&,animal,bird)";
                "&", ["bird", "robin", "{Birdie}", "(|,[yellow],{Birdie})"] => "(&,bird,robin,{Birdie},(|,[yellow],{Birdie}))";
                "&", ["bird", "swimmer"] => "(&,bird,swimmer)";
                "&", ["chess", "competition"] => "(&,chess,competition)";
                "&", ["competition", "sport"] => "(&,competition,sport)";
                "&", ["flyer", "[with_wings]"] => "(&,flyer,[with_wings])";
                "&", ["flyer", "[yellow]"] => "(&,flyer,[yellow])";
                "&", ["flyer", "bird"] => "(&,bird,flyer)";
                "&", ["flyer", "robin"] => "(&,flyer,robin)";
                "&", ["flyer", "{Birdie}", "(|,[with_wings],{Birdie})"] => "(&,flyer,{Birdie},(|,[with_wings],{Birdie}))";
                "&", ["flyer", "{Birdie}"] => "(&,flyer,{Birdie})";
                "&", ["flyer", "{Tweety}", "(|,[with_wings],{Birdie})"] => "(&,flyer,{Tweety},(|,[with_wings],{Birdie}))";
                "&", ["flyer", "{Tweety}"] => "(&,flyer,{Tweety})";
                "&", ["key", "(/,open,_,lock)"] => "(&,key,(/,open,_,lock))";
                "&", ["key", "(/,open,_,{lock1})"] => "(&,key,(/,open,_,{lock1}))";
                "&", ["key", "{key1}"] => "(&,key,{key1})";
                "&", ["neutralization", "(*,(\\,neutralization,_,base),base)"] => "(&,neutralization,(*,(\\,neutralization,_,base),base))";
                "&", ["neutralization", "(*,acid,(/,reaction,acid,_))"] => "(&,neutralization,(*,acid,(/,reaction,acid,_)))";
                "&", ["neutralization", "(*,acid,base)"] => "(&,neutralization,(*,acid,base))";
                "&", ["neutralization", "(*,acid,soda)"] => "(&,neutralization,(*,acid,soda))";
                "&", ["neutralization", "reaction"] => "(&,neutralization,reaction)";
                "&", ["num", "(/,num,_)"] => "(&,num,(/,num,_))";
                "&", ["reaction", "neutralization"] => "(&,neutralization,reaction)";
                "&", ["robin", "animal"] => "(&,animal,robin)";
                "&", ["robin", "bird"] => "(&,bird,robin)";
                "&", ["robin", "swimmer"] => "(&,robin,swimmer)";
                "&", ["robin", "{Birdie}"] => "(&,robin,{Birdie})";
                "&", ["tiger", "animal"] => "(&,animal,tiger)";
                "&", ["tiger", "swimmer"] => "(&,swimmer,tiger)";
                "&", ["{Birdie}", "(|,flyer,{Tweety})"] => "(&,{Birdie},(|,flyer,{Tweety}))";
                "&", ["{Birdie}", "{Tweety}"] => None;
                "&", ["{Tweety}", "(|,bird,{Birdie})"] => "(&,{Tweety},(|,bird,{Birdie}))";
                "&", ["{Tweety}", "{Birdie}"] => None;
                "&&", ["<robin --> [chirping]>", "<robin --> [flying]>"] => "(&&,<robin --> [chirping]>,<robin --> [flying]>)";
                "&&", ["<robin --> [chirping]>", "<robin --> [with_wings]>"] => "(&&,<robin --> [chirping]>,<robin --> [with_wings]>)";
                "&&", ["<robin --> [chirping]>"] => "<robin --> [chirping]>";
                "&&", ["<robin --> [flying]>", "<robin --> [with_wings]>"] => "(&&,<robin --> [flying]>,<robin --> [with_wings]>)";
                "&&", ["<robin --> [flying]>"] => "<robin --> [flying]>";
                "&&", ["<robin --> [living]>"] => "<robin --> [living]>";
                "&&", ["<robin --> [with_wings]>"] => "<robin --> [with_wings]>";
                "&&", ["<robin --> bird>", "(||,(&&,<robin --> [flying]>,<robin --> [with_wings]>),<robin --> bird>)"] => "(&&,<robin --> bird>,(||,(&&,<robin --> [flying]>,<robin --> [with_wings]>),<robin --> bird>))";
                "&&", ["<robin --> bird>", "<robin --> [flying]>", "<robin --> [with_wings]>"] => "(&&,<robin --> bird>,<robin --> [flying]>,<robin --> [with_wings]>)";
                "&&", ["<robin --> bird>", "<robin --> [flying]>"] => "(&&,<robin --> bird>,<robin --> [flying]>)";
                "&&", ["<robin --> bird>", "<robin --> [with_wings]>"] => "(&&,<robin --> bird>,<robin --> [with_wings]>)";
                "&&", ["<robin --> bird>"] => "<robin --> bird>";
                "&&", ["<robin --> flyer>", "<(*,robin,worms) --> food>"] => "(&&,<robin --> flyer>,<(*,robin,worms) --> food>)";
                "&&", ["<robin --> flyer>", "<robin --> bird>", "<(*,robin,worms) --> food>"] => "(&&,<robin --> bird>,<robin --> flyer>,<(*,robin,worms) --> food>)";
                "&&", ["<robin --> flyer>", "<robin --> bird>", "<worms --> (/,food,robin,_)>"] => "(&&,<robin --> bird>,<robin --> flyer>,<worms --> (/,food,robin,_)>)";
                "&&", ["<robin --> flyer>", "<robin --> bird>"] => "(&&,<robin --> bird>,<robin --> flyer>)";
                "&&", ["<robin --> flyer>", "<worms --> (/,food,robin,_)>"] => "(&&,<robin --> flyer>,<worms --> (/,food,robin,_)>)";
                "&&", ["<robin --> flyer>"] => "<robin --> flyer>";
                "&&", ["<robin --> swimmer>"] => "<robin --> swimmer>";
                "*", ["(&,key,(/,open,_,{lock1}))", "lock"] => "(*,(&,key,(/,open,_,{lock1})),lock)";
                "*", ["(&,num,(/,(*,(/,num,_)),_))"] => "(*,(&,num,(/,(*,(/,num,_)),_)))";
                "*", ["(*,num)"] => "(*,(*,num))";
                "*", ["(/,(*,(/,num,_)),_)"] => "(*,(/,(*,(/,num,_)),_))";
                "*", ["(/,(/,num,_),_)"] => "(*,(/,(/,num,_),_))";
                "*", ["(/,REPRESENT,_,<(*,CAT,FISH) --> FOOD>)", "<(*,CAT,FISH) --> FOOD>"] => "(*,(/,REPRESENT,_,<(*,CAT,FISH) --> FOOD>),<(*,CAT,FISH) --> FOOD>)";
                "*", ["(/,num,_)"] => "(*,(/,num,_))";
                "*", ["(/,open,_,lock)", "lock"] => "(*,(/,open,_,lock),lock)";
                "*", ["(/,open,_,lock)", "lock1"] => "(*,(/,open,_,lock),lock1)";
                "*", ["(/,open,_,lock)", "{lock1}"] => "(*,(/,open,_,lock),{lock1})";
                "*", ["(/,open,_,lock1)", "lock1"] => "(*,(/,open,_,lock1),lock1)";
                "*", ["(/,open,_,{lock1})", "lock"] => "(*,(/,open,_,{lock1}),lock)";
                "*", ["(/,open,_,{lock1})", "lock1"] => "(*,(/,open,_,{lock1}),lock1)";
                "*", ["(/,open,_,{lock1})", "{lock1}"] => "(*,(/,open,_,{lock1}),{lock1})";
                "*", ["(/,uncle,tom,_)", "tom"] => "(*,(/,uncle,tom,_),tom)";
                "*", ["(\\,neutralization,_,base)", "base"] => "(*,(\\,neutralization,_,base),base)";
                "*", ["(|,(/,open,_,lock1),(/,open,_,{lock1}))", "lock1"] => "(*,(|,(/,open,_,lock1),(/,open,_,{lock1})),lock1)";
                "*", ["(|,key,(/,open,_,{lock1}))", "lock"] => "(*,(|,key,(/,open,_,{lock1})),lock)";
                "*", ["(|,key,(/,open,_,{lock1}))", "lock1"] => "(*,(|,key,(/,open,_,{lock1})),lock1)";
                "*", ["0"] => "(*,0)";
                "*", ["a", "b"] => "(*,a,b)";
                "*", ["acid", "(&,soda,(/,neutralization,acid,_))"] => "(*,acid,(&,soda,(/,neutralization,acid,_)))";
                "*", ["acid", "(/,neutralization,acid,_)"] => "(*,acid,(/,neutralization,acid,_))";
                "*", ["acid", "(/,reaction,acid,_)"] => "(*,acid,(/,reaction,acid,_))";
                "*", ["acid", "(\\,neutralization,acid,_)"] => "(*,acid,(\\,neutralization,acid,_))";
                "*", ["acid", "(\\,reaction,acid,_)"] => "(*,acid,(\\,reaction,acid,_))";
                "*", ["acid", "(|,base,(\\,reaction,acid,_))"] => "(*,acid,(|,base,(\\,reaction,acid,_)))";
                "*", ["acid", "(|,soda,(\\,neutralization,acid,_))"] => "(*,acid,(|,soda,(\\,neutralization,acid,_)))";
                "*", ["acid", "base"] => "(*,acid,base)";
                "*", ["acid", "soda"] => "(*,acid,soda)";
                "*", ["key", "lock"] => "(*,key,lock)";
                "*", ["key", "lock1"] => "(*,key,lock1)";
                "*", ["key", "{lock1}"] => "(*,key,{lock1})";
                "*", ["num"] => "(*,num)";
                "*", ["{key1}", "lock1"] => "(*,{key1},lock1)";
                "[]", ["bright"] => "[bright]";
                "[]", ["smart"] => "[smart]";
                "{}", ["Birdie"] => "{Birdie}";
                "{}", ["Mars", "Venus"] => "{Mars,Venus}";
                "|", ["(&,animal,gull)", "swimmer"] => "(|,swimmer,(&,animal,gull))";
                "|", ["(&,flyer,{Birdie})", "(|,[yellow],{Birdie})"] => "(|,[yellow],{Birdie},(&,flyer,{Birdie}))";
                "|", ["(&,flyer,{Birdie})", "(|,[yellow],{Tweety})"] => "(|,[yellow],{Tweety},(&,flyer,{Birdie}))";
                "|", ["(&,flyer,{Birdie})", "{Birdie}"] => "(|,{Birdie},(&,flyer,{Birdie}))";
                "|", ["(/,neutralization,_,base)", "(/,reaction,_,(\\,neutralization,acid,_))"] => "(|,(/,neutralization,_,base),(/,reaction,_,(\\,neutralization,acid,_)))";
                "|", ["(/,neutralization,_,base)", "(/,reaction,_,base)"] => "(|,(/,neutralization,_,base),(/,reaction,_,base))";
                "|", ["(/,neutralization,_,base)", "acid"] => "(|,acid,(/,neutralization,_,base))";
                "|", ["(/,neutralization,acid,_)", "(\\,neutralization,acid,_)"] => "(|,(/,neutralization,acid,_),(\\,neutralization,acid,_))";
                "|", ["(/,num,_)", "0"] => "(|,0,(/,num,_))";
                "|", ["(/,open,_,{lock1})", "(/,open,_,lock)"] => "(|,(/,open,_,lock),(/,open,_,{lock1}))";
                "|", ["(/,reaction,_,soda)", "(/,reaction,_,base)"] => "(|,(/,reaction,_,base),(/,reaction,_,soda))";
                "|", ["(/,reaction,acid,_)", "(\\,neutralization,acid,_)"] => "(|,(/,reaction,acid,_),(\\,neutralization,acid,_))";
                "|", ["(\\,REPRESENT,_,CAT)", "(/,(/,REPRESENT,_,<(*,CAT,FISH) --> FOOD>),_,eat,fish)"] => "(|,(\\,REPRESENT,_,CAT),(/,(/,REPRESENT,_,<(*,CAT,FISH) --> FOOD>),_,eat,fish))";
                "|", ["(|,key,(/,open,_,{lock1}))", "(/,open,_,lock)"] => "(|,key,(/,open,_,lock),(/,open,_,{lock1}))";
                "|", ["(~,boy,girl)", "(~,youth,girl)"] => "(|,(~,boy,girl),(~,youth,girl))";
                "|", ["[with_wings]", "(|,flyer,{Tweety})", "{Birdie}"] => "(|,flyer,[with_wings],{Birdie},{Tweety})";
                "|", ["[with_wings]", "flyer", "{Birdie}"] => "(|,flyer,[with_wings],{Birdie})";
                "|", ["[with_wings]", "{Birdie}", "(|,[with_wings],{Birdie})"] => "(|,[with_wings],{Birdie})";
                "|", ["[with_wings]", "{Tweety}", "{Birdie}"] => "(|,[with_wings],{Birdie},{Tweety})";
                "|", ["[yellow]", "[with_wings]"] => None;
                "|", ["[yellow]", "bird"] => "(|,bird,[yellow])";
                "|", ["[yellow]", "flyer"] => "(|,flyer,[yellow])";
                "|", ["[yellow]", "{Tweety}"] => "(|,[yellow],{Tweety})";
                "|", ["acid", "(/,reaction,_,base)"] => "(|,acid,(/,reaction,_,base))";
                "|", ["acid", "(\\,neutralization,_,base)"] => "(|,acid,(\\,neutralization,_,base))";
                "|", ["acid", "(\\,neutralization,_,soda)"] => "(|,acid,(\\,neutralization,_,soda))";
                "|", ["animal", "robin"] => "(|,animal,robin)";
                "|", ["animal", "swan"] => "(|,animal,swan)";
                "|", ["animal", "swimmer"] => "(|,animal,swimmer)";
                "|", ["base", "(/,neutralization,acid,_)"] => "(|,base,(/,neutralization,acid,_))";
                "|", ["base", "(/,reaction,acid,_)"] => "(|,base,(/,reaction,acid,_))";
                "|", ["base", "(\\,neutralization,acid,_)"] => "(|,base,(\\,neutralization,acid,_))";
                "|", ["base", "soda"] => "(|,base,soda)";
                "|", ["bird", "[with_wings]"] => "(|,bird,[with_wings])";
                "|", ["bird", "[yellow]"] => "(|,bird,[yellow])";
                "|", ["bird", "animal"] => "(|,animal,bird)";
                "|", ["bird", "flyer", "{Birdie}"] => "(|,bird,flyer,{Birdie})";
                "|", ["bird", "flyer"] => "(|,bird,flyer)";
                "|", ["bird", "swimmer"] => "(|,bird,swimmer)";
                "|", ["bird", "{Birdie}"] => "(|,bird,{Birdie})";
                "|", ["bird", "{Tweety}", "{Birdie}"] => "(|,bird,{Birdie},{Tweety})";
                "|", ["bird", "{Tweety}"] => "(|,bird,{Tweety})";
                "|", ["boy", "(~,youth,girl)"] => "(|,boy,(~,youth,girl))";
                "|", ["chess", "(|,chess,sport)"] => "(|,chess,sport)";
                "|", ["chess", "competition"] => "(|,chess,competition)";
                "|", ["chess", "sport"] => "(|,chess,sport)";
                "|", ["competition", "chess"] => "(|,chess,competition)";
                "|", ["competition", "sport"] => "(|,competition,sport)";
                "|", ["flyer", "(&,flyer,{Birdie})", "{Birdie}"] => "(|,flyer,{Birdie},(&,flyer,{Birdie}))";
                "|", ["flyer", "(&,flyer,{Birdie})"] => "(|,flyer,(&,flyer,{Birdie}))";
                "|", ["flyer", "(|,flyer,{Tweety})", "{Birdie}"] => "(|,flyer,{Birdie},{Tweety})";
                "|", ["flyer", "[yellow]", "{Birdie}"] => "(|,flyer,[yellow],{Birdie})";
                "|", ["flyer", "robin"] => "(|,flyer,robin)";
                "|", ["flyer", "{Birdie}", "(&,bird,(|,[yellow],{Birdie}))"] => "(|,flyer,{Birdie},(&,bird,(|,[yellow],{Birdie})))";
                "|", ["flyer", "{Birdie}", "(&,flyer,{Birdie})"] => "(|,flyer,{Birdie},(&,flyer,{Birdie}))";
                "|", ["flyer", "{Birdie}"] => "(|,flyer,{Birdie})";
                "|", ["flyer", "{Tweety}", "{Birdie}"] => "(|,flyer,{Birdie},{Tweety})";
                "|", ["flyer", "{Tweety}"] => "(|,flyer,{Tweety})";
                "|", ["key", "(/,open,_,lock)"] => "(|,key,(/,open,_,lock))";
                "|", ["key", "(/,open,_,{lock1})"] => "(|,key,(/,open,_,{lock1}))";
                "|", ["key", "{key1}"] => "(|,key,{key1})";
                "|", ["neutralization", "(*,acid,(\\,neutralization,acid,_))"] => "(|,neutralization,(*,acid,(\\,neutralization,acid,_)))";
                "|", ["neutralization", "(*,acid,base)"] => "(|,neutralization,(*,acid,base))";
                "|", ["neutralization", "reaction"] => "(|,neutralization,reaction)";
                "|", ["reaction", "(*,acid,base)"] => "(|,reaction,(*,acid,base))";
                "|", ["reaction", "neutralization"] => "(|,neutralization,reaction)";
                "|", ["robin", "(|,flyer,{Tweety})", "{Birdie}"] => "(|,flyer,robin,{Birdie},{Tweety})";
                "|", ["robin", "[yellow]", "{Birdie}"] => "(|,robin,[yellow],{Birdie})";
                "|", ["robin", "animal"] => "(|,animal,robin)";
                "|", ["robin", "bird"] => "(|,bird,robin)";
                "|", ["robin", "flyer", "{Birdie}"] => "(|,flyer,robin,{Birdie})";
                "|", ["robin", "swimmer"] => "(|,robin,swimmer)";
                "|", ["robin", "{Birdie}", "(&,bird,(|,[yellow],{Birdie}))"] => "(|,robin,{Birdie},(&,bird,(|,[yellow],{Birdie})))";
                "|", ["robin", "{Birdie}"] => "(|,robin,{Birdie})";
                "|", ["robin", "{Tweety}", "{Birdie}"] => "(|,robin,{Birdie},{Tweety})";
                "|", ["sport", "competition"] => "(|,competition,sport)";
                "|", ["tiger", "(|,animal,swimmer)"] => "(|,animal,swimmer,tiger)";
                "|", ["tiger", "animal"] => "(|,animal,tiger)";
                "|", ["tiger", "swimmer"] => "(|,swimmer,tiger)";
                "|", ["{Birdie}", "{Tweety}"] => "{Birdie,Tweety}";
                "|", ["{Tweety}", "{Birdie}", "(&,flyer,{Birdie})"] => "(|,(&,flyer,{Birdie}),{Birdie,Tweety})";
                "|", ["{Tweety}", "{Birdie}"] => "{Birdie,Tweety}";
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
            macro_once! {
                // * 🚩模式：参数列表 ⇒ 预期词项
                macro test($($template:tt, $terms:tt => $expected:tt;)*) {
                    $(
                        let template = term!($template);
                        let terms: Vec<Term> = term!($terms).into();
                        let terms_str = terms.iter().map(|t| format!("\"{t}\"")).collect::<Vec<_>>().join(", ");
                        let out = Term::make_compound_term(
                            template.as_compound().expect("模板不是复合词项！"),
                            terms
                        );
                        let expected = option_term!($expected);
                        assert_eq!(
                            out, expected,
                            "\"{template}\", {terms_str} => {} != {}",
                            format_option_term(&out),
                            format_option_term(&expected),
                        );
                    )*
                }
                // * ℹ️用例均源自OpenNARS实际运行
                "(&&,<robin --> [chirping]>,<robin --> [flying]>)", ["<robin --> [chirping]>"] => "<robin --> [chirping]>";
                "(&&,<robin --> [chirping]>,<robin --> [flying]>)", ["<robin --> [flying]>"] => "<robin --> [flying]>";
                "(&&,<robin --> [chirping]>,<robin --> [flying]>)", ["<robin --> bird>", "<robin --> [flying]>"] => "(&&,<robin --> bird>,<robin --> [flying]>)";
                "(&&,<robin --> [chirping]>,<robin --> [flying]>,<robin --> [with_wings]>)", ["<robin --> [chirping]>", "<robin --> [flying]>"] => "(&&,<robin --> [chirping]>,<robin --> [flying]>)";
                "(&&,<robin --> [chirping]>,<robin --> [flying]>,<robin --> [with_wings]>)", ["<robin --> [chirping]>", "<robin --> [with_wings]>"] => "(&&,<robin --> [chirping]>,<robin --> [with_wings]>)";
                "(&&,<robin --> [chirping]>,<robin --> [flying]>,<robin --> [with_wings]>)", ["<robin --> [flying]>", "<robin --> [with_wings]>"] => "(&&,<robin --> [flying]>,<robin --> [with_wings]>)";
                "(&&,<robin --> [chirping]>,<robin --> [flying]>,<robin --> [with_wings]>)", ["<robin --> bird>", "<robin --> [flying]>", "<robin --> [with_wings]>"] => "(&&,<robin --> bird>,<robin --> [flying]>,<robin --> [with_wings]>)";
                "(&&,<robin --> [chirping]>,<robin --> [with_wings]>)", ["<robin --> [chirping]>", "<robin --> bird>"] => "(&&,<robin --> bird>,<robin --> [chirping]>)";
                "(&&,<robin --> [chirping]>,<robin --> [with_wings]>)", ["<robin --> [chirping]>"] => "<robin --> [chirping]>";
                "(&&,<robin --> [chirping]>,<robin --> [with_wings]>)", ["<robin --> [with_wings]>"] => "<robin --> [with_wings]>";
                "(&&,<robin --> [chirping]>,<robin --> [with_wings]>)", ["<robin --> bird>", "<robin --> [with_wings]>"] => "(&&,<robin --> bird>,<robin --> [with_wings]>)";
                "(&&,<robin --> [flying]>,<robin --> [with_wings]>)", ["<robin --> [flying]>"] => "<robin --> [flying]>";
                "(&&,<robin --> [flying]>,<robin --> [with_wings]>)", ["<robin --> [with_wings]>"] => "<robin --> [with_wings]>";
                "(&&,<robin --> bird>,<robin --> [flying]>)", ["<robin --> [flying]>"] => "<robin --> [flying]>";
                "(&&,<robin --> bird>,<robin --> [flying]>)", ["<robin --> bird>"] => "<robin --> bird>";
                "(&&,<robin --> bird>,<robin --> [flying]>,<robin --> [with_wings]>)", ["<robin --> [flying]>", "<robin --> [with_wings]>"] => "(&&,<robin --> [flying]>,<robin --> [with_wings]>)";
                "(&&,<robin --> bird>,<robin --> [flying]>,<robin --> [with_wings]>)", ["<robin --> bird>", "<robin --> [flying]>", "<robin --> bird>"] => "(&&,<robin --> bird>,<robin --> [flying]>)";
                "(&&,<robin --> bird>,<robin --> [flying]>,<robin --> [with_wings]>)", ["<robin --> bird>", "<robin --> [flying]>"] => "(&&,<robin --> bird>,<robin --> [flying]>)";
                "(&&,<robin --> bird>,<robin --> [flying]>,<robin --> [with_wings]>)", ["<robin --> bird>", "<robin --> [with_wings]>"] => "(&&,<robin --> bird>,<robin --> [with_wings]>)";
                "(&&,<robin --> bird>,<robin --> [living]>)", ["<robin --> [living]>"] => "<robin --> [living]>";
                "(&&,<robin --> bird>,<robin --> [living]>)", ["<robin --> bird>", "(||,(&&,<robin --> [flying]>,<robin --> [with_wings]>),<robin --> bird>)"] => "(&&,<robin --> bird>,(||,(&&,<robin --> [flying]>,<robin --> [with_wings]>),<robin --> bird>))";
                "(&&,<robin --> bird>,<robin --> [living]>)", ["<robin --> bird>", "<robin --> [flying]>", "<robin --> [with_wings]>"] => "(&&,<robin --> bird>,<robin --> [flying]>,<robin --> [with_wings]>)";
                "(&&,<robin --> bird>,<robin --> [living]>)", ["<robin --> bird>", "<robin --> [flying]>"] => "(&&,<robin --> bird>,<robin --> [flying]>)";
                "(&&,<robin --> bird>,<robin --> [living]>)", ["<robin --> bird>", "<robin --> bird>", "<robin --> [flying]>"] => "(&&,<robin --> bird>,<robin --> [flying]>)";
                "(&&,<robin --> bird>,<robin --> [living]>)", ["<robin --> bird>"] => "<robin --> bird>";
                "(&&,<robin --> flyer>,<(*,robin,worms) --> food>)", ["<robin --> flyer>", "<worms --> (/,food,robin,_)>"] => "(&&,<robin --> flyer>,<worms --> (/,food,robin,_)>)";
                "(&&,<robin --> flyer>,<robin --> [chirping]>)", ["<robin --> flyer>", "<robin --> bird>"] => "(&&,<robin --> bird>,<robin --> flyer>)";
                "(&&,<robin --> flyer>,<robin --> [chirping]>)", ["<robin --> flyer>"] => "<robin --> flyer>";
                "(&&,<robin --> flyer>,<robin --> [chirping]>,<(*,robin,worms) --> food>)", ["<robin --> flyer>", "<(*,robin,worms) --> food>"] => "(&&,<robin --> flyer>,<(*,robin,worms) --> food>)";
                "(&&,<robin --> flyer>,<robin --> [chirping]>,<(*,robin,worms) --> food>)", ["<robin --> flyer>", "<robin --> bird>", "<(*,robin,worms) --> food>"] => "(&&,<robin --> bird>,<robin --> flyer>,<(*,robin,worms) --> food>)";
                "(&&,<robin --> flyer>,<robin --> [chirping]>,<worms --> (/,food,robin,_)>)", ["<robin --> flyer>", "<robin --> bird>", "<worms --> (/,food,robin,_)>"] => "(&&,<robin --> bird>,<robin --> flyer>,<worms --> (/,food,robin,_)>)";
                "(&&,<robin --> flyer>,<robin --> [chirping]>,<worms --> (/,food,robin,_)>)", ["<robin --> flyer>", "<worms --> (/,food,robin,_)>"] => "(&&,<robin --> flyer>,<worms --> (/,food,robin,_)>)";
                "(&&,<robin --> flyer>,<worms --> (/,food,robin,_)>)", ["<robin --> flyer>", "<(*,robin,worms) --> food>"] => "(&&,<robin --> flyer>,<(*,robin,worms) --> food>)";
                "(&&,<robin --> swimmer>,<robin --> [flying]>)", ["<robin --> [flying]>"] => "<robin --> [flying]>";
                "(&&,<robin --> swimmer>,<robin --> [flying]>)", ["<robin --> swimmer>"] => "<robin --> swimmer>";
                "(&,(/,neutralization,_,(\\,neutralization,acid,_)),(/,reaction,_,base))", ["(/,neutralization,_,(\\,neutralization,acid,_))", "acid"] => "(&,acid,(/,neutralization,_,(\\,neutralization,acid,_)))";
                "(&,(/,neutralization,_,(\\,neutralization,acid,_)),(/,reaction,_,base))", ["acid", "(/,reaction,_,base)"] => "(&,acid,(/,reaction,_,base))";
                "(&,(/,neutralization,_,base),(/,reaction,_,soda))", ["(/,neutralization,_,base)", "(/,reaction,_,base)"] => "(&,(/,neutralization,_,base),(/,reaction,_,base))";
                "(&,(/,neutralization,_,base),(/,reaction,_,soda))", ["(/,neutralization,_,base)", "acid"] => "(&,acid,(/,neutralization,_,base))";
                "(&,(/,neutralization,_,soda),(/,reaction,_,base))", ["(/,neutralization,_,base)", "(/,reaction,_,base)"] => "(&,(/,neutralization,_,base),(/,reaction,_,base))";
                "(&,(/,neutralization,_,soda),(/,reaction,_,base))", ["(/,reaction,_,soda)", "(/,reaction,_,base)"] => "(&,(/,reaction,_,base),(/,reaction,_,soda))";
                "(&,(/,neutralization,_,soda),(/,reaction,_,base))", ["acid", "(/,reaction,_,base)"] => "(&,acid,(/,reaction,_,base))";
                "(&,(/,open,_,lock),(/,open,_,{lock1}))", ["(/,open,_,lock)", "key"] => "(&,key,(/,open,_,lock))";
                "(&,(\\,REPRESENT,_,CAT),(\\,(\\,REPRESENT,_,<(*,CAT,FISH) --> FOOD>),_,eat,fish))", ["(\\,REPRESENT,_,CAT)", "(/,(/,REPRESENT,_,<(*,CAT,FISH) --> FOOD>),_,eat,fish)"] => "(&,(\\,REPRESENT,_,CAT),(/,(/,REPRESENT,_,<(*,CAT,FISH) --> FOOD>),_,eat,fish))";
                "(&,(\\,REPRESENT,_,CAT),(\\,(\\,REPRESENT,_,<(*,CAT,FISH) --> FOOD>),_,eat,fish))", ["cat", "(\\,(\\,REPRESENT,_,<(*,CAT,FISH) --> FOOD>),_,eat,fish)"] => "(&,cat,(\\,(\\,REPRESENT,_,<(*,CAT,FISH) --> FOOD>),_,eat,fish))";
                "(&,(\\,reaction,_,soda),(|,acid,(\\,reaction,_,base)))", ["(\\,reaction,_,soda)", "(\\,neutralization,_,base)"] => "(&,(\\,neutralization,_,base),(\\,reaction,_,soda))";
                "(&,(|,bird,flyer),(|,bird,{Birdie}))", ["(|,bird,{Tweety})", "(|,bird,{Birdie})"] => "(&,(|,bird,{Birdie}),(|,bird,{Tweety}))";
                "(&,(|,bird,flyer),(|,bird,{Birdie}))", ["{Tweety}", "(|,bird,{Birdie})"] => "(&,{Tweety},(|,bird,{Birdie}))";
                "(&,[with_wings],{Birdie})", ["(&,robin,{Tweety})", "{Birdie}"] => "(&,robin,{Birdie},{Tweety})";
                "(&,[with_wings],{Birdie})", ["flyer", "{Birdie}"] => "(&,flyer,{Birdie})";
                "(&,[with_wings],{Birdie})", ["robin", "{Birdie}"] => "(&,robin,{Birdie})";
                "(&,[with_wings],{Birdie})", ["{Tweety}", "{Birdie}"] => None;
                "(&,[yellow],{Birdie})", ["{Tweety}", "{Birdie}"] => None;
                "(&,acid,(/,neutralization,_,soda))", ["acid", "(/,reaction,_,base)"] => "(&,acid,(/,reaction,_,base))";
                "(&,acid,(\\,reaction,_,base))", ["acid", "(\\,neutralization,_,base)"] => "(&,acid,(\\,neutralization,_,base))";
                "(&,acid,(\\,reaction,_,soda))", ["acid", "(\\,neutralization,_,soda)"] => "(&,acid,(\\,neutralization,_,soda))";
                "(&,animal,(|,animal,swimmer))", ["animal", "gull"] => "(&,animal,gull)";
                "(&,animal,(|,bird,swimmer))", ["animal", "(&,robin,swan)"] => "(&,animal,robin,swan)";
                "(&,animal,(|,bird,swimmer))", ["animal", "swan"] => "(&,animal,swan)";
                "(&,animal,gull)", ["animal", "(|,animal,swimmer)"] => "(&,animal,(|,animal,swimmer))";
                "(&,animal,gull)", ["animal", "swan"] => "(&,animal,swan)";
                "(&,animal,gull)", ["animal", "swimmer"] => "(&,animal,swimmer)";
                "(&,base,(\\,reaction,acid,_))", ["base", "(/,reaction,acid,_)"] => "(&,base,(/,reaction,acid,_))";
                "(&,base,(\\,reaction,acid,_))", ["base", "(\\,neutralization,acid,_)"] => "(&,base,(\\,neutralization,acid,_))";
                "(&,base,(\\,reaction,acid,_))", ["base", "soda"] => "(&,base,soda)";
                "(&,bird,(|,robin,tiger))", ["bird", "animal"] => "(&,animal,bird)";
                "(&,bird,(|,robin,tiger))", ["bird", "swimmer"] => "(&,bird,swimmer)";
                "(&,bird,[with_wings],{Birdie},(|,[yellow],{Birdie}))", ["bird", "robin", "{Birdie}", "(|,[yellow],{Birdie})"] => "(&,bird,robin,{Birdie},(|,[yellow],{Birdie}))";
                "(&,chess,sport)", ["chess", "competition"] => "(&,chess,competition)";
                "(&,chess,sport)", ["competition", "sport"] => "(&,competition,sport)";
                "(&,flyer,[with_wings])", ["flyer", "(&,robin,{Tweety})"] => "(&,flyer,robin,{Tweety})";
                "(&,flyer,[with_wings])", ["flyer", "robin"] => "(&,flyer,robin)";
                "(&,flyer,[with_wings])", ["flyer", "{Birdie}"] => "(&,flyer,{Birdie})";
                "(&,flyer,[with_wings])", ["flyer", "{Tweety}"] => "(&,flyer,{Tweety})";
                "(&,flyer,[yellow])", ["flyer", "{Birdie}"] => "(&,flyer,{Birdie})";
                "(&,flyer,[yellow])", ["flyer", "{Tweety}"] => "(&,flyer,{Tweety})";
                "(&,flyer,[yellow],(|,[with_wings],{Birdie}))", ["flyer", "{Birdie}", "(|,[with_wings],{Birdie})"] => "(&,flyer,{Birdie},(|,[with_wings],{Birdie}))";
                "(&,flyer,[yellow],(|,[with_wings],{Birdie}))", ["flyer", "{Tweety}", "(|,[with_wings],{Birdie})"] => "(&,flyer,{Tweety},(|,[with_wings],{Birdie}))";
                "(&,flyer,{Birdie})", ["flyer", "[with_wings]"] => "(&,flyer,[with_wings])";
                "(&,flyer,{Birdie})", ["flyer", "[yellow]"] => "(&,flyer,[yellow])";
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
                "(&,reaction,(*,acid,soda))", ["reaction", "neutralization"] => "(&,neutralization,reaction)";
                "(&,robin,tiger)", ["robin", "animal"] => "(&,animal,robin)";
                "(&,robin,tiger)", ["robin", "bird"] => "(&,bird,robin)";
                "(&,robin,tiger)", ["robin", "swimmer"] => "(&,robin,swimmer)";
                "(&,tiger,(|,bird,robin))", ["bird", "(|,bird,robin)"] => "(&,bird,(|,bird,robin))";
                "(&,tiger,(|,bird,robin))", ["tiger", "animal"] => "(&,animal,tiger)";
                "(&,tiger,(|,bird,robin))", ["tiger", "swimmer"] => "(&,swimmer,tiger)";
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
                "(*,(/,num,_))", ["num"] => "(*,num)";
                "(*,(/,open,_,lock1),lock1)", ["{key1}", "lock1"] => "(*,{key1},lock1)";
                "(*,(\\,reaction,_,base),base)", ["(\\,neutralization,_,base)", "base"] => "(*,(\\,neutralization,_,base),base)";
                "(*,(\\,reaction,_,soda),base)", ["(\\,neutralization,_,base)", "base"] => "(*,(\\,neutralization,_,base),base)";
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
                "(*,acid,(/,neutralization,acid,_))", ["acid", "base"] => "(*,acid,base)";
                "(*,acid,(/,reaction,acid,_))", ["acid", "(&,soda,(/,neutralization,acid,_))"] => "(*,acid,(&,soda,(/,neutralization,acid,_)))";
                "(*,acid,(/,reaction,acid,_))", ["acid", "(/,neutralization,acid,_)"] => "(*,acid,(/,neutralization,acid,_))";
                "(*,acid,(/,reaction,acid,_))", ["acid", "(\\,neutralization,acid,_)"] => "(*,acid,(\\,neutralization,acid,_))";
                "(*,acid,(/,reaction,acid,_))", ["acid", "(\\,reaction,acid,_)"] => "(*,acid,(\\,reaction,acid,_))";
                "(*,acid,(/,reaction,acid,_))", ["acid", "(|,base,(\\,reaction,acid,_))"] => "(*,acid,(|,base,(\\,reaction,acid,_)))";
                "(*,acid,(/,reaction,acid,_))", ["acid", "(|,soda,(\\,neutralization,acid,_))"] => "(*,acid,(|,soda,(\\,neutralization,acid,_)))";
                "(*,acid,(/,reaction,acid,_))", ["acid", "base"] => "(*,acid,base)";
                "(*,acid,(/,reaction,acid,_))", ["acid", "soda"] => "(*,acid,soda)";
                "(*,acid,base)", ["acid", "(/,neutralization,acid,_)"] => "(*,acid,(/,neutralization,acid,_))";
                "(*,acid,base)", ["acid", "(\\,neutralization,acid,_)"] => "(*,acid,(\\,neutralization,acid,_))";
                "(*,acid,base)", ["acid", "soda"] => "(*,acid,soda)";
                "(*,acid,soda)", ["(/,neutralization,_,soda)", "soda"] => "(*,(/,neutralization,_,soda),soda)";
                "(*,acid,soda)", ["acid", "(/,neutralization,acid,_)"] => "(*,acid,(/,neutralization,acid,_))";
                "(*,acid,soda)", ["acid", "(/,reaction,acid,_)"] => "(*,acid,(/,reaction,acid,_))";
                "(*,acid,soda)", ["acid", "(\\,neutralization,acid,_)"] => "(*,acid,(\\,neutralization,acid,_))";
                "(*,acid,soda)", ["acid", "base"] => "(*,acid,base)";
                "(*,b,a)", ["b", "(/,like,b,_)"] => "(*,b,(/,like,b,_))";
                "(*,num)", ["(/,num,_)"] => "(*,(/,num,_))";
                "(*,num)", ["0"] => "(*,0)";
                "(*,tim,tom)", ["(/,uncle,tom,_)", "tom"] => "(*,(/,uncle,tom,_),tom)";
                "(*,{key1},lock)", ["(&,key,(/,open,_,{lock1}))", "lock"] => "(*,(&,key,(/,open,_,{lock1})),lock)";
                "(*,{key1},lock)", ["(/,open,_,{lock1})", "lock"] => "(*,(/,open,_,{lock1}),lock)";
                "(*,{key1},lock)", ["(|,key,(/,open,_,{lock1}))", "lock"] => "(*,(|,key,(/,open,_,{lock1})),lock)";
                "(*,{key1},lock)", ["key", "lock"] => "(*,key,lock)";
                "(*,{key1},lock1)", ["(/,open,_,lock)", "lock1"] => "(*,(/,open,_,lock),lock1)";
                "(*,{key1},lock1)", ["(/,open,_,lock1)", "lock1"] => "(*,(/,open,_,lock1),lock1)";
                "(*,{key1},lock1)", ["(/,open,_,{lock1})", "lock1"] => "(*,(/,open,_,{lock1}),lock1)";
                "(*,{key1},lock1)", ["(|,(/,open,_,lock1),(/,open,_,{lock1}))", "lock1"] => "(*,(|,(/,open,_,lock1),(/,open,_,{lock1})),lock1)";
                "(*,{key1},lock1)", ["(|,key,(/,open,_,{lock1}))", "lock1"] => "(*,(|,key,(/,open,_,{lock1})),lock1)";
                "(*,{key1},lock1)", ["key", "lock1"] => "(*,key,lock1)";
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
                "(/,neutralization,_,base)", ["neutralization", "(/,neutralization,acid,_)"] => "(/,neutralization,_,(/,neutralization,acid,_))";
                "(/,neutralization,_,base)", ["neutralization", "(\\,neutralization,acid,_)"] => "(/,neutralization,_,(\\,neutralization,acid,_))";
                "(/,neutralization,_,base)", ["neutralization", "soda"] => "(/,neutralization,_,soda)";
                "(/,neutralization,_,base)", ["reaction", "base"] => "(/,reaction,_,base)";
                "(/,neutralization,_,soda)", ["neutralization", "(/,neutralization,acid,_)"] => "(/,neutralization,_,(/,neutralization,acid,_))";
                "(/,neutralization,_,soda)", ["neutralization", "(/,reaction,acid,_)"] => "(/,neutralization,_,(/,reaction,acid,_))";
                "(/,neutralization,_,soda)", ["neutralization", "base"] => "(/,neutralization,_,base)";
                "(/,neutralization,acid,_)", ["acid", "reaction"] => "(/,reaction,acid,_)";
                "(/,num,_)", ["(*,0)"] => "(/,(*,0),_)";
                "(/,num,_)", ["(/,num,_)"] => "(/,(/,num,_),_)";
                "(/,num,_)", ["0"] => "(/,0,_)";
                "(/,open,_,(|,lock,(/,open,{key1},_)))", ["open", "{lock1}"] => "(/,open,_,{lock1})";
                "(/,open,_,{lock1})", ["open", "(|,lock,(/,open,{key1},_))"] => "(/,open,_,(|,lock,(/,open,{key1},_)))";
                "(/,open,_,{lock1})", ["open", "lock"] => "(/,open,_,lock)";
                "(/,reaction,_,base)", ["(*,acid,soda)", "base"] => "(/,(*,acid,soda),_,base)";
                "(/,reaction,_,base)", ["neutralization", "base"] => "(/,neutralization,_,base)";
                "(/,reaction,_,base)", ["reaction", "(/,neutralization,acid,_)"] => "(/,reaction,_,(/,neutralization,acid,_))";
                "(/,reaction,_,base)", ["reaction", "soda"] => "(/,reaction,_,soda)";
                "(/,reaction,_,soda)", ["neutralization", "soda"] => "(/,neutralization,_,soda)";
                "(/,reaction,_,soda)", ["reaction", "(/,neutralization,acid,_)"] => "(/,reaction,_,(/,neutralization,acid,_))";
                "(/,reaction,_,soda)", ["reaction", "(/,reaction,acid,_)"] => "(/,reaction,_,(/,reaction,acid,_))";
                "(/,reaction,_,soda)", ["reaction", "(\\,neutralization,acid,_)"] => "(/,reaction,_,(\\,neutralization,acid,_))";
                "(/,reaction,_,soda)", ["reaction", "(\\,reaction,acid,_)"] => "(/,reaction,_,(\\,reaction,acid,_))";
                "(/,reaction,_,soda)", ["reaction", "base"] => "(/,reaction,_,base)";
                "(/,reaction,acid,_)", ["acid", "(*,acid,soda)"] => "(/,(*,acid,soda),acid,_)";
                "(/,reaction,acid,_)", ["acid", "neutralization"] => "(/,neutralization,acid,_)";
                "(/,uncle,_,tom)", ["(*,tim,tom)", "tom"] => "(/,(*,tim,tom),_,tom)";
                "(/,uncle,tim,_)", ["(/,uncle,_,tom)", "uncle"] => "(/,uncle,(/,uncle,_,tom),_)";
                "(/,uncle,tim,_)", ["tim", "(*,tim,tom)"] => "(/,(*,tim,tom),tim,_)";
                "(/,uncle,tom,_)", ["tom", "(*,tim,tom)"] => "(/,(*,tim,tom),tom,_)";
                "(\\,(*,b,a),_,(/,like,b,_))", ["like", "(/,like,b,_)"] => "(\\,like,_,(/,like,b,_))";
                "(\\,REPRESENT,_,CAT)", ["REPRESENT", "(\\,REPRESENT,_,CAT)"] => "(\\,REPRESENT,_,(\\,REPRESENT,_,CAT))";
                "(\\,neutralization,_,(/,neutralization,acid,_))", ["neutralization", "soda"] => "(\\,neutralization,_,soda)";
                "(\\,neutralization,_,(/,reaction,acid,_))", ["neutralization", "(/,neutralization,acid,_)"] => "(\\,neutralization,_,(/,neutralization,acid,_))";
                "(\\,neutralization,_,(/,reaction,acid,_))", ["neutralization", "(\\,neutralization,acid,_)"] => "(\\,neutralization,_,(\\,neutralization,acid,_))";
                "(\\,neutralization,_,(/,reaction,acid,_))", ["neutralization", "(|,base,(\\,reaction,acid,_))"] => "(\\,neutralization,_,(|,base,(\\,reaction,acid,_)))";
                "(\\,neutralization,_,(/,reaction,acid,_))", ["neutralization", "base"] => "(\\,neutralization,_,base)";
                "(\\,neutralization,_,(/,reaction,acid,_))", ["neutralization", "soda"] => "(\\,neutralization,_,soda)";
                "(\\,neutralization,_,base)", ["neutralization", "(/,neutralization,acid,_)"] => "(\\,neutralization,_,(/,neutralization,acid,_))";
                "(\\,neutralization,_,base)", ["neutralization", "soda"] => "(\\,neutralization,_,soda)";
                "(\\,neutralization,_,base)", ["reaction", "base"] => "(\\,reaction,_,base)";
                "(\\,neutralization,_,soda)", ["neutralization", "(/,neutralization,acid,_)"] => "(\\,neutralization,_,(/,neutralization,acid,_))";
                "(\\,neutralization,_,soda)", ["neutralization", "(/,reaction,acid,_)"] => "(\\,neutralization,_,(/,reaction,acid,_))";
                "(\\,neutralization,_,soda)", ["neutralization", "(\\,neutralization,acid,_)"] => "(\\,neutralization,_,(\\,neutralization,acid,_))";
                "(\\,neutralization,_,soda)", ["neutralization", "(\\,reaction,acid,_)"] => "(\\,neutralization,_,(\\,reaction,acid,_))";
                "(\\,neutralization,_,soda)", ["neutralization", "base"] => "(\\,neutralization,_,base)";
                "(\\,neutralization,acid,_)", ["(\\,reaction,_,base)", "neutralization"] => "(\\,neutralization,(\\,reaction,_,base),_)";
                "(\\,neutralization,acid,_)", ["acid", "reaction"] => "(\\,reaction,acid,_)";
                "(\\,reaction,(\\,reaction,_,soda),_)", ["(\\,reaction,_,base)", "reaction"] => "(\\,reaction,(\\,reaction,_,base),_)";
                "(\\,reaction,_,base)", ["(*,acid,soda)", "base"] => "(\\,(*,acid,soda),_,base)";
                "(\\,reaction,_,base)", ["neutralization", "base"] => "(\\,neutralization,_,base)";
                "(\\,reaction,_,base)", ["reaction", "soda"] => "(\\,reaction,_,soda)";
                "(\\,reaction,_,soda)", ["neutralization", "soda"] => "(\\,neutralization,_,soda)";
                "(\\,reaction,_,soda)", ["reaction", "(/,neutralization,acid,_)"] => "(\\,reaction,_,(/,neutralization,acid,_))";
                "(\\,reaction,_,soda)", ["reaction", "(/,reaction,acid,_)"] => "(\\,reaction,_,(/,reaction,acid,_))";
                "(\\,reaction,_,soda)", ["reaction", "(\\,neutralization,acid,_)"] => "(\\,reaction,_,(\\,neutralization,acid,_))";
                "(\\,reaction,_,soda)", ["reaction", "(\\,reaction,acid,_)"] => "(\\,reaction,_,(\\,reaction,acid,_))";
                "(\\,reaction,_,soda)", ["reaction", "base"] => "(\\,reaction,_,base)";
                "(\\,reaction,acid,_)", ["acid", "(*,acid,soda)"] => "(\\,(*,acid,soda),acid,_)";
                "(\\,reaction,acid,_)", ["acid", "neutralization"] => "(\\,neutralization,acid,_)";
                "(|,(&,animal,gull),(&,bird,robin))", ["(&,animal,gull)", "swimmer"] => "(|,swimmer,(&,animal,gull))";
                "(|,(&,flyer,{Birdie}),{Birdie,Tweety})", ["(&,flyer,{Birdie})", "(|,[yellow],{Birdie})"] => "(|,[yellow],{Birdie},(&,flyer,{Birdie}))";
                "(|,(&,flyer,{Birdie}),{Birdie,Tweety})", ["(&,flyer,{Birdie})", "(|,[yellow],{Tweety})"] => "(|,[yellow],{Tweety},(&,flyer,{Birdie}))";
                "(|,(/,neutralization,_,(\\,neutralization,acid,_)),(/,reaction,_,base))", ["(/,neutralization,_,base)", "(/,reaction,_,base)"] => "(|,(/,neutralization,_,base),(/,reaction,_,base))";
                "(|,(/,neutralization,_,(\\,neutralization,acid,_)),(/,reaction,_,base))", ["acid", "(/,reaction,_,base)"] => "(|,acid,(/,reaction,_,base))";
                "(|,(/,neutralization,_,base),(/,reaction,_,base))", ["(/,neutralization,_,base)", "acid"] => "(|,acid,(/,neutralization,_,base))";
                "(|,(/,neutralization,_,base),(/,reaction,_,soda))", ["(/,neutralization,_,base)", "(/,neutralization,_,(\\,neutralization,acid,_))"] => "(|,(/,neutralization,_,base),(/,neutralization,_,(\\,neutralization,acid,_)))";
                "(|,(/,neutralization,_,base),(/,reaction,_,soda))", ["(/,neutralization,_,base)", "(/,reaction,_,(\\,neutralization,acid,_))"] => "(|,(/,neutralization,_,base),(/,reaction,_,(\\,neutralization,acid,_)))";
                "(|,(/,neutralization,_,base),(/,reaction,_,soda))", ["(/,neutralization,_,base)", "(/,reaction,_,base)"] => "(|,(/,neutralization,_,base),(/,reaction,_,base))";
                "(|,(/,neutralization,_,base),(/,reaction,_,soda))", ["(/,neutralization,_,base)", "acid"] => "(|,acid,(/,neutralization,_,base))";
                "(|,(/,neutralization,_,soda),(/,reaction,_,base))", ["(/,neutralization,_,base)", "(/,reaction,_,base)"] => "(|,(/,neutralization,_,base),(/,reaction,_,base))";
                "(|,(/,neutralization,_,soda),(/,reaction,_,base))", ["(/,reaction,_,soda)", "(/,reaction,_,base)"] => "(|,(/,reaction,_,base),(/,reaction,_,soda))";
                "(|,(/,neutralization,_,soda),(/,reaction,_,base))", ["acid", "(/,reaction,_,base)"] => "(|,acid,(/,reaction,_,base))";
                "(|,(/,num,_),(/,(*,num),_))", ["(/,num,_)", "0"] => "(|,0,(/,num,_))";
                "(|,(\\,REPRESENT,_,CAT),(\\,(\\,REPRESENT,_,<(*,CAT,FISH) --> FOOD>),_,eat,fish))", ["(\\,REPRESENT,_,CAT)", "(/,(/,REPRESENT,_,<(*,CAT,FISH) --> FOOD>),_,eat,fish)"] => "(|,(\\,REPRESENT,_,CAT),(/,(/,REPRESENT,_,<(*,CAT,FISH) --> FOOD>),_,eat,fish))";
                "(|,(\\,REPRESENT,_,CAT),(\\,(\\,REPRESENT,_,<(*,CAT,FISH) --> FOOD>),_,eat,fish))", ["cat", "(\\,(\\,REPRESENT,_,<(*,CAT,FISH) --> FOOD>),_,eat,fish)"] => "(|,cat,(\\,(\\,REPRESENT,_,<(*,CAT,FISH) --> FOOD>),_,eat,fish))";
                "(|,CAT,(/,(/,REPRESENT,_,<(*,CAT,FISH) --> FOOD>),_,eat,fish))", ["(\\,(\\,REPRESENT,_,<(*,CAT,FISH) --> FOOD>),_,eat,fish)", "(/,(/,REPRESENT,_,<(*,CAT,FISH) --> FOOD>),_,eat,fish)"] => "(|,(/,(/,REPRESENT,_,<(*,CAT,FISH) --> FOOD>),_,eat,fish),(\\,(\\,REPRESENT,_,<(*,CAT,FISH) --> FOOD>),_,eat,fish))";
                "(|,[strong],(~,youth,girl))", ["(~,boy,girl)", "(~,youth,girl)"] => "(|,(~,boy,girl),(~,youth,girl))";
                "(|,[strong],(~,youth,girl))", ["boy", "(~,youth,girl)"] => "(|,boy,(~,youth,girl))";
                "(|,[with_wings],[yellow],{Birdie})", ["[with_wings]", "(|,flyer,{Tweety})", "{Birdie}"] => "(|,flyer,[with_wings],{Birdie},{Tweety})";
                "(|,[with_wings],[yellow],{Birdie})", ["[with_wings]", "flyer", "{Birdie}"] => "(|,flyer,[with_wings],{Birdie})";
                "(|,[with_wings],[yellow],{Birdie})", ["[with_wings]", "{Tweety}", "{Birdie}"] => "(|,[with_wings],{Birdie},{Tweety})";
                "(|,[with_wings],[yellow],{Birdie})", ["flyer", "[yellow]", "{Birdie}"] => "(|,flyer,[yellow],{Birdie})";
                "(|,[with_wings],[yellow],{Birdie})", ["robin", "[yellow]", "{Birdie}"] => "(|,robin,[yellow],{Birdie})";
                "(|,[with_wings],{Birdie})", ["flyer", "{Birdie}"] => "(|,flyer,{Birdie})";
                "(|,[with_wings],{Birdie})", ["robin", "{Birdie}"] => "(|,robin,{Birdie})";
                "(|,[with_wings],{Birdie})", ["{Tweety}", "{Birdie}"] => "{Birdie,Tweety}";
                "(|,[with_wings],{Birdie},(&,bird,(|,[yellow],{Birdie})))", ["flyer", "{Birdie}", "(&,bird,(|,[yellow],{Birdie}))"] => "(|,flyer,{Birdie},(&,bird,(|,[yellow],{Birdie})))";
                "(|,[with_wings],{Birdie},(&,bird,(|,[yellow],{Birdie})))", ["robin", "{Birdie}", "(&,bird,(|,[yellow],{Birdie}))"] => "(|,robin,{Birdie},(&,bird,(|,[yellow],{Birdie})))";
                "(|,[with_wings],{Birdie},(&,flyer,[yellow]))", ["[with_wings]", "{Birdie}", "(|,[with_wings],{Birdie})"] => "(|,[with_wings],{Birdie})";
                "(|,[yellow],{Birdie})", ["(&,flyer,{Birdie})", "{Birdie}"] => "(|,{Birdie},(&,flyer,{Birdie}))";
                "(|,[yellow],{Birdie})", ["[yellow]", "[with_wings]"] => None;
                "(|,[yellow],{Birdie})", ["[yellow]", "bird"] => "(|,bird,[yellow])";
                "(|,[yellow],{Birdie})", ["[yellow]", "flyer"] => "(|,flyer,[yellow])";
                "(|,[yellow],{Birdie})", ["[yellow]", "{Tweety}"] => "(|,[yellow],{Tweety})";
                "(|,[yellow],{Birdie})", ["flyer", "{Birdie}"] => "(|,flyer,{Birdie})";
                "(|,[yellow],{Birdie})", ["{Tweety}", "{Birdie}"] => "{Birdie,Tweety}";
                "(|,[yellow],{Birdie},(&,flyer,{Birdie}))", ["flyer", "{Birdie}", "(&,flyer,{Birdie})"] => "(|,flyer,{Birdie},(&,flyer,{Birdie}))";
                "(|,[yellow],{Birdie},(&,flyer,{Birdie}))", ["{Tweety}", "{Birdie}", "(&,flyer,{Birdie})"] => "(|,(&,flyer,{Birdie}),{Birdie,Tweety})";
                "(|,[yellow],{Tweety})", ["flyer", "{Tweety}"] => "(|,flyer,{Tweety})";
                "(|,[yellow],{Tweety})", ["{Birdie}", "{Tweety}"] => "{Birdie,Tweety}";
                "(|,acid,(/,neutralization,_,soda))", ["acid", "(/,reaction,_,base)"] => "(|,acid,(/,reaction,_,base))";
                "(|,acid,(\\,reaction,_,base))", ["acid", "(\\,neutralization,_,base)"] => "(|,acid,(\\,neutralization,_,base))";
                "(|,acid,(\\,reaction,_,soda))", ["acid", "(\\,neutralization,_,base)"] => "(|,acid,(\\,neutralization,_,base))";
                "(|,acid,(\\,reaction,_,soda))", ["acid", "(\\,neutralization,_,soda)"] => "(|,acid,(\\,neutralization,_,soda))";
                "(|,animal,gull)", ["animal", "robin"] => "(|,animal,robin)";
                "(|,animal,gull)", ["animal", "swan"] => "(|,animal,swan)";
                "(|,animal,gull)", ["animal", "swimmer"] => "(|,animal,swimmer)";
                "(|,base,(/,reaction,acid,_))", ["base", "(/,neutralization,acid,_)"] => "(|,base,(/,neutralization,acid,_))";
                "(|,base,(\\,reaction,acid,_))", ["base", "(/,reaction,acid,_)"] => "(|,base,(/,reaction,acid,_))";
                "(|,base,(\\,reaction,acid,_))", ["base", "(\\,neutralization,acid,_)"] => "(|,base,(\\,neutralization,acid,_))";
                "(|,base,(\\,reaction,acid,_))", ["base", "soda"] => "(|,base,soda)";
                "(|,bird,(&,robin,tiger))", ["bird", "animal"] => "(|,animal,bird)";
                "(|,bird,(&,robin,tiger))", ["bird", "swimmer"] => "(|,bird,swimmer)";
                "(|,bird,[yellow])", ["bird", "flyer"] => "(|,bird,flyer)";
                "(|,bird,[yellow])", ["bird", "{Birdie}"] => "(|,bird,{Birdie})";
                "(|,bird,[yellow])", ["bird", "{Tweety}"] => "(|,bird,{Tweety})";
                "(|,bird,[yellow],{Birdie})", ["bird", "flyer", "{Birdie}"] => "(|,bird,flyer,{Birdie})";
                "(|,bird,[yellow],{Birdie})", ["bird", "{Tweety}", "{Birdie}"] => "(|,bird,{Birdie},{Tweety})";
                "(|,bird,{Birdie})", ["bird", "[with_wings]"] => "(|,bird,[with_wings])";
                "(|,bird,{Birdie})", ["bird", "[yellow]"] => "(|,bird,[yellow])";
                "(|,bird,{Birdie})", ["bird", "flyer"] => "(|,bird,flyer)";
                "(|,bird,{Birdie})", ["bird", "{Tweety}"] => "(|,bird,{Tweety})";
                "(|,bird,{Tweety})", ["bird", "(|,bird,flyer)"] => "(|,bird,flyer)";
                "(|,boy,girl)", ["youth", "girl"] => "(|,girl,youth)";
                "(|,chess,competition)", ["chess", "(|,chess,sport)"] => "(|,chess,sport)";
                "(|,chess,competition)", ["chess", "sport"] => "(|,chess,sport)";
                "(|,chess,competition)", ["sport", "competition"] => "(|,competition,sport)";
                "(|,chess,sport)", ["chess", "competition"] => "(|,chess,competition)";
                "(|,chess,sport)", ["competition", "sport"] => "(|,competition,sport)";
                "(|,competition,sport)", ["chess", "sport"] => "(|,chess,sport)";
                "(|,competition,sport)", ["competition", "chess"] => "(|,chess,competition)";
                "(|,flyer,[with_wings])", ["flyer", "robin"] => "(|,flyer,robin)";
                "(|,flyer,[with_wings])", ["flyer", "{Birdie}"] => "(|,flyer,{Birdie})";
                "(|,flyer,[with_wings])", ["flyer", "{Tweety}"] => "(|,flyer,{Tweety})";
                "(|,flyer,[yellow])", ["flyer", "(&,flyer,{Birdie})"] => "(|,flyer,(&,flyer,{Birdie}))";
                "(|,flyer,[yellow])", ["flyer", "{Birdie}"] => "(|,flyer,{Birdie})";
                "(|,flyer,[yellow])", ["flyer", "{Tweety}"] => "(|,flyer,{Tweety})";
                "(|,flyer,[yellow],(&,flyer,{Birdie}))", ["flyer", "{Birdie}", "(&,flyer,{Birdie})"] => "(|,flyer,{Birdie},(&,flyer,{Birdie}))";
                "(|,flyer,[yellow],{Birdie})", ["flyer", "(&,flyer,{Birdie})", "{Birdie}"] => "(|,flyer,{Birdie},(&,flyer,{Birdie}))";
                "(|,flyer,[yellow],{Birdie})", ["flyer", "(|,flyer,{Tweety})", "{Birdie}"] => "(|,flyer,{Birdie},{Tweety})";
                "(|,flyer,[yellow],{Birdie})", ["flyer", "{Tweety}", "{Birdie}"] => "(|,flyer,{Birdie},{Tweety})";
                "(|,key,(/,open,_,lock))", ["key", "(/,open,_,{lock1})"] => "(|,key,(/,open,_,{lock1}))";
                "(|,key,(/,open,_,lock))", ["key", "{key1}"] => "(|,key,{key1})";
                "(|,neutralization,(*,(\\,reaction,_,soda),base))", ["neutralization", "reaction"] => "(|,neutralization,reaction)";
                "(|,neutralization,(*,acid,soda))", ["neutralization", "(*,acid,(\\,neutralization,acid,_))"] => "(|,neutralization,(*,acid,(\\,neutralization,acid,_)))";
                "(|,neutralization,(*,acid,soda))", ["neutralization", "(*,acid,base)"] => "(|,neutralization,(*,acid,base))";
                "(|,neutralization,(*,acid,soda))", ["neutralization", "reaction"] => "(|,neutralization,reaction)";
                "(|,reaction,(*,acid,soda))", ["reaction", "(*,acid,base)"] => "(|,reaction,(*,acid,base))";
                "(|,reaction,(*,acid,soda))", ["reaction", "neutralization"] => "(|,neutralization,reaction)";
                "(|,robin,[yellow],{Birdie})", ["robin", "(|,flyer,{Tweety})", "{Birdie}"] => "(|,flyer,robin,{Birdie},{Tweety})";
                "(|,robin,[yellow],{Birdie})", ["robin", "flyer", "{Birdie}"] => "(|,flyer,robin,{Birdie})";
                "(|,robin,[yellow],{Birdie})", ["robin", "{Tweety}", "{Birdie}"] => "(|,robin,{Birdie},{Tweety})";
                "(|,robin,tiger)", ["robin", "animal"] => "(|,animal,robin)";
                "(|,robin,tiger)", ["robin", "bird"] => "(|,bird,robin)";
                "(|,robin,tiger)", ["robin", "swimmer"] => "(|,robin,swimmer)";
                "(|,soda,(\\,neutralization,acid,_))", ["(/,neutralization,acid,_)", "(\\,neutralization,acid,_)"] => "(|,(/,neutralization,acid,_),(\\,neutralization,acid,_))";
                "(|,soda,(\\,neutralization,acid,_))", ["(/,reaction,acid,_)", "(\\,neutralization,acid,_)"] => "(|,(/,reaction,acid,_),(\\,neutralization,acid,_))";
                "(|,soda,(\\,neutralization,acid,_))", ["base", "(\\,neutralization,acid,_)"] => "(|,base,(\\,neutralization,acid,_))";
                "(|,tiger,(&,bird,robin))", ["tiger", "(|,animal,swimmer)"] => "(|,animal,swimmer,tiger)";
                "(|,tiger,(&,bird,robin))", ["tiger", "animal"] => "(|,animal,tiger)";
                "(|,tiger,(&,bird,robin))", ["tiger", "swimmer"] => "(|,swimmer,tiger)";
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
                "[smart]", ["bright"] => "[bright]";
                "{Birdie}", ["Tweety"] => "{Tweety}";
                "{Mars,Pluto,Saturn,Venus}", ["Mars", "Venus"] => "{Mars,Venus}";
                "{Tweety}", ["Birdie"] => "{Birdie}";
            }
            ok!()
        }

        #[test]
        fn can_extract() -> AResult {
            macro_once! {
                // * 🚩模式：词项字符串⇒预期
                macro test($($term:expr => $expected:expr)*) {
                    $(
                        assert_eq!(term!($term).as_compound().unwrap().can_extract_to_inner(), $expected);
                    )*
                }
                // * 🚩正例
                "(&&, A)" => true
                "(||, A)" => true
                "(&, A)" => true
                "(|, A)" => true
                "(-, A, B)" => true
                "(~, A, B)" => true
                // * 🚩反例
                "{A}" => false
                "[A]" => false
            }
            ok!()
        }

        #[test]
        fn reduce_components() -> AResult {
            fn test(t: Term, to_reduce: &Term) {
                let c = t.as_compound().unwrap();
                let new_c = c.reduce_components(to_reduce);
                // TODO: 需要等到「完整实现」之后才能测试
            }
            ok!()
        }

        #[test]
        fn set_component() -> AResult {
            // TODO: 等待「制作词项」所有方法均完成
            ok!()
        }
    }

    mod statement {
        use super::*;

        #[test]
        fn make_statement_relation() -> AResult {
            macro_once! {
                // * 🚩模式：参数列表 ⇒ 预期词项
                macro test($($relation:tt, $subject:tt, $predicate:tt => $expected:tt;)*) {
                    $(
                        let relation = $relation; // 字符
                        let subject = term!($subject);
                        let predicate = term!($predicate);
                        let out = Term::make_statement_relation(relation, subject.clone(), predicate.clone());
                        let expected = option_term!($expected);
                        assert_eq!(
                            out, expected,
                            "\"{relation}\", \"{subject}\", \"{predicate}\" => {} != {}",
                            format_option_term(&out),
                            format_option_term(&expected),
                        );
                    )*
                }
                // * ℹ️用例均源自OpenNARS实际运行
                "==>", "(&&,<robin --> bird>,<robin --> [flying]>)", "<robin --> [living]>" => "<(&&,<robin --> bird>,<robin --> [flying]>) ==> <robin --> [living]>>";
                "<->", "{Birdie}", "{Tweety}" => "<{Birdie} <-> {Tweety}>";
                "<->", "bird", "swan" => "<bird <-> swan>";
                "==>", "<robin --> [flying]>", "<robin --> animal>" => "<<robin --> [flying]> ==> <robin --> animal>>";
                "-->", "(-,swimmer,animal)", "(-,swimmer,bird)" => "<(-,swimmer,animal) --> (-,swimmer,bird)>";
                "==>", "(&&,<robin --> [flying]>,<robin --> [with_wings]>)", "<robin --> [living]>" => "<(&&,<robin --> [flying]>,<robin --> [with_wings]>) ==> <robin --> [living]>>";
                "-->", "?120", "claimedByBob" => "<?120 --> claimedByBob>";
                "<->", "[bright]", "[smart]" => "<[bright] <-> [smart]>";
                "-->", "{Tweety}", "bird" => "<{Tweety} --> bird>";
                "-->", "(*,CAT,FISH)", "FOOD" => "<(*,CAT,FISH) --> FOOD>";
                "-->", "?120", "swimmer" => "<?120 --> swimmer>";
                "-->", "neutralization", "(*,acid,base)" => "<neutralization --> (*,acid,base)>";
                "-->", "(*,(*,(*,0)))", "num" => "<(*,(*,(*,0))) --> num>";
                "-->", "{key1}", "(/,open,_,{lock1})" => "<{key1} --> (/,open,_,{lock1})>";
                "-->", "(*,bird,plant)", "?120" => "<(*,bird,plant) --> ?120>";
                "-->", "robin", "animal" => "<robin --> animal>";
                "-->", "gull", "swimmer" => "<gull --> swimmer>";
                "-->", "bird", "swan" => "<bird --> swan>";
                "==>", "(&&,<robin --> [flying]>,<robin --> [with_wings]>)", "<robin --> bird>" => "<(&&,<robin --> [flying]>,<robin --> [with_wings]>) ==> <robin --> bird>>";
                "-->", "swan", "(-,swimmer,bird)" => "<swan --> (-,swimmer,bird)>";
                "-->", "planetX", "{Mars,Pluto,Venus}" => "<planetX --> {Mars,Pluto,Venus}>";
                "-->", "(/,neutralization,_,base)", "?120" => "<(/,neutralization,_,base) --> ?120>";
                "==>", "(&&,<robin --> [chirping]>,<robin --> [flying]>)", "<robin --> bird>" => "<(&&,<robin --> [chirping]>,<robin --> [flying]>) ==> <robin --> bird>>";
                "==>", "<robin --> [flying]>", "<robin --> bird>" => "<<robin --> [flying]> ==> <robin --> bird>>";
                "-->", "(~,swimmer,swan)", "bird" => "<(~,swimmer,swan) --> bird>";
                "<=>", "<robin --> bird>", "<robin --> [flying]>" => "<<robin --> bird> <=> <robin --> [flying]>>";
                "-->", "robin", "[living]" => "<robin --> [living]>";
                "-->", "bird", "animal" => "<bird --> animal>";
                "==>", "<robin --> bird>", "(&&,<robin --> animal>,<robin --> [flying]>)" => "<<robin --> bird> ==> (&&,<robin --> animal>,<robin --> [flying]>)>";
                "==>", "(&&,<robin --> swimmer>,<robin --> [flying]>)", "<robin --> bird>" => "<(&&,<robin --> swimmer>,<robin --> [flying]>) ==> <robin --> bird>>";
                "-->", "0", "(/,num,_)" => "<0 --> (/,num,_)>";
                "-->", "(&,swan,swimmer)", "bird" => "<(&,swan,swimmer) --> bird>";
                "-->", "{key1}", "key" => "<{key1} --> key>";
                "==>", "(--,<robin --> bird>)", "<robin --> [flying]>" => "<(--,<robin --> bird>) ==> <robin --> [flying]>>";
                "==>", "(&&,<robin --> bird>,<robin --> [living]>)", "<robin --> animal>" => "<(&&,<robin --> bird>,<robin --> [living]>) ==> <robin --> animal>>";
                "-->", "swan", "(|,bird,swimmer)" => "<swan --> (|,bird,swimmer)>";
                "-->", "[smart]", "[bright]" => "<[smart] --> [bright]>";
                "-->", "robin", "(-,mammal,swimmer)" => "<robin --> (-,mammal,swimmer)>";
                "--]", "raven", "black" => "<raven --> [black]>";
                "-->", "(&,<bird --> fly>,<{Tweety} --> bird>)", "claimedByBob" => "<(&,<bird --> fly>,<{Tweety} --> bird>) --> claimedByBob>";
                "-->", "(*,b,a)", "like" => "<(*,b,a) --> like>";
                "-->", "{Tweety}", "flyer" => "<{Tweety} --> flyer>";
                "<->", "gull", "swan" => "<gull <-> swan>";
                "-->", "(&,bird,swimmer)", "(&,animal,swimmer)" => "<(&,bird,swimmer) --> (&,animal,swimmer)>";
                "-->", "acid", "(/,reaction,_,base)" => "<acid --> (/,reaction,_,base)>";
                "==>", "<robin --> bird>", "<robin --> animal>" => "<<robin --> bird> ==> <robin --> animal>>";
                "-->", "base", "(/,reaction,acid,_)" => "<base --> (/,reaction,acid,_)>";
                "-->", "swimmer", "bird" => "<swimmer --> bird>";
                "-->", "cat", "(/,(/,REPRESENT,_,<(*,CAT,FISH) --> FOOD>),_,eat,fish)" => "<cat --> (/,(/,REPRESENT,_,<(*,CAT,FISH) --> FOOD>),_,eat,fish)>";
                "<=>", "<robin --> animal>", "<robin --> bird>" => "<<robin --> animal> <=> <robin --> bird>>";
                "==>", "<robin --> [flying]>", "<robin --> [with_beak]>" => "<<robin --> [flying]> ==> <robin --> [with_beak]>>";
                "-->", "{Tweety}", "[with_wings]" => "<{Tweety} --> [with_wings]>";
                "==>", "(&&,<robin --> [chirping]>,<robin --> [with_wings]>)", "<robin --> bird>" => "<(&&,<robin --> [chirping]>,<robin --> [with_wings]>) ==> <robin --> bird>>";
                "{-]", "Tweety", "yellow" => "<{Tweety} --> [yellow]>";
                "-->", "swan", "bird" => "<swan --> bird>";
                "-->", "chess", "competition" => "<chess --> competition>";
                "-->", "robin", "[with_wings]" => "<robin --> [with_wings]>";
                "-->", "robin", "[with_beak]" => "<robin --> [with_beak]>";
                "-->", "tiger", "animal" => "<tiger --> animal>";
                "-->", "bird", "swimmer" => "<bird --> swimmer>";
                "-->", "lock1", "lock" => "<lock1 --> lock>";
                "==>", "<robin --> bird>", "<robin --> [flying]>" => "<<robin --> bird> ==> <robin --> [flying]>>";
                "-->", "robin", "bird" => "<robin --> bird>";
                "-->", "(*,a,b)", "like" => "<(*,a,b) --> like>";
                "-->", "robin", "swimmer" => "<robin --> swimmer>";
                "<->", "bright", "smart" => "<bright <-> smart>";
                "-->", "(~,boy,girl)", "[strong]" => "<(~,boy,girl) --> [strong]>";
                "-->", "robin", "[chirping]" => "<robin --> [chirping]>";
                "-->", "(|,boy,girl)", "youth" => "<(|,boy,girl) --> youth>";
                "-->", "{Tweety}", "{Birdie}" => "<{Tweety} --> {Birdie}>";
                "-->", "{?49}", "swimmer" => "<{?49} --> swimmer>";
                "<->", "robin", "swan" => "<robin <-> swan>";
                "-->", "(*,acid,base)", "reaction" => "<(*,acid,base) --> reaction>";
                "-->", "{lock1}", "lock" => "<{lock1} --> lock>";
                "-->", "neutralization", "reaction" => "<neutralization --> reaction>";
                "-->", "swan", "swimmer" => "<swan --> swimmer>";
                "-->", "sport", "competition" => "<sport --> competition>";
                "-->", "0", "num" => "<0 --> num>";
                "-->", "planetX", "{Pluto,Saturn}" => "<planetX --> {Pluto,Saturn}>";
                "-->", "robin", "(-,bird,swimmer)" => "<robin --> (-,bird,swimmer)>";
                "-->", "tim", "(/,uncle,_,tom)" => "<tim --> (/,uncle,_,tom)>";
                "-->", "bird", "fly" => "<bird --> fly>";
                "{--", "Tweety", "bird" => "<{Tweety} --> bird>";
                "-->", "robin", "(&,bird,swimmer)" => "<robin --> (&,bird,swimmer)>";
                "-->", "?49", "swimmer" => "<?49 --> swimmer>";
                "-->", "cat", "CAT" => "<cat --> CAT>";
                "<->", "Birdie", "Tweety" => "<Birdie <-> Tweety>";
                "-->", "robin", "[flying]" => "<robin --> [flying]>";
                "-->", "soda", "base" => "<soda --> base>";
                "-->", "tim", "(/,uncle,tom,_)" => "<tim --> (/,uncle,tom,_)>";
                "==>", "(--,<robin --> [flying]>)", "<robin --> bird>" => "<(--,<robin --> [flying]>) ==> <robin --> bird>>";
                "==>", "(&&,<robin --> [chirping]>,<robin --> [flying]>,<robin --> [with_wings]>)", "<robin --> bird>" => "<(&&,<robin --> [chirping]>,<robin --> [flying]>,<robin --> [with_wings]>) ==> <robin --> bird>>";
                "-->", "robin", "(|,bird,swimmer)" => "<robin --> (|,bird,swimmer)>";
            }
            ok!()
        }

        #[test]
        fn make_statement() -> AResult {
            macro_once! {
                // * 🚩模式：参数列表 ⇒ 预期词项
                macro test($($template:tt, $subject:tt, $predicate:tt => $expected:tt;)*) {
                    $(
                        let template = term!($template);
                        let subject = term!($subject);
                        let predicate = term!($predicate);
                        let out = Term::make_statement(template.as_statement().unwrap(), subject.clone(), predicate.clone());
                        let expected = option_term!($expected);
                        assert_eq!(
                            out, expected,
                            "\"{template}\", \"{subject}\", \"{predicate}\" => {} != {}",
                            format_option_term(&out),
                            format_option_term(&expected),
                        );
                    )*
                }
                // * ℹ️用例均源自OpenNARS实际运行
                "<[smart] --> [bright]>", "[bright]", "[smart]" => "<[bright] --> [smart]>";
                "<swan --> (&,bird,swimmer)>", "(|,robin,swan)", "(&,bird,swimmer)" => "<(|,robin,swan) --> (&,bird,swimmer)>";
                "<{Tweety} --> flyer>", "(|,[with_wings],{Birdie})", "flyer" => "<(|,[with_wings],{Birdie}) --> flyer>";
                "<(*,0) --> (*,(/,num,_))>", "0", "(/,num,_)" => "<0 --> (/,num,_)>";
                "<{Tweety} --> [with_wings]>", "{Tweety}", "(|,[with_wings],(&,flyer,{Birdie}))" => "<{Tweety} --> (|,[with_wings],(&,flyer,{Birdie}))>";
                "<robin --> animal>", "(|,robin,tiger)", "animal" => "<(|,robin,tiger) --> animal>";
                "<(|,bird,{Tweety}) --> (|,bird,{Birdie})>", "{Tweety}", "{Birdie}" => "<{Tweety} --> {Birdie}>";
                "<{key1} --> (/,open,_,{lock1})>", "(/,open,_,{lock1})", "key" => "<(/,open,_,{lock1}) --> key>";
                "<(&&,<robin --> [chirping]>,<robin --> [flying]>,<robin --> [with_wings]>) ==> <robin --> bird>>", "(&&,<robin --> [flying]>,<robin --> [with_wings]>)", "<robin --> bird>" => "<(&&,<robin --> [flying]>,<robin --> [with_wings]>) ==> <robin --> bird>>";
                "<(*,0) --> (*,(/,num,_))>", "(*,(/,num,_))", "(*,num)" => "<(*,(/,num,_)) --> (*,num)>";
                "<planetX --> {Mars,Venus}>", "{Mars,Venus}", "{Pluto,Saturn}" => "<{Mars,Venus} --> {Pluto,Saturn}>";
                "<robin --> bird>", "animal", "robin" => "<animal --> robin>";
                "<{Tweety} --> (&,bird,flyer)>", "{Tweety}", "flyer" => "<{Tweety} --> flyer>";
                "<(|,boy,girl) --> youth>", "boy", "youth" => "<boy --> youth>";
                "<a --> (/,like,b,_)>", "(/,like,_,(/,like,b,_))", "(/,like,_,a)" => "<(/,like,_,(/,like,b,_)) --> (/,like,_,a)>";
                "<{Tweety} --> flyer>", "{Tweety}", "(|,flyer,[yellow],{Birdie})" => "<{Tweety} --> (|,flyer,[yellow],{Birdie})>";
                "<(|,robin,swimmer) --> bird>", "swimmer", "bird" => "<swimmer --> bird>";
                "<planetX --> {Pluto,Saturn}>", "{Pluto,Saturn}", "{Mars,Pluto,Saturn,Venus}" => "<{Pluto,Saturn} --> {Mars,Pluto,Saturn,Venus}>";
                "<?1 --> swimmer>", "animal", "swimmer" => "<animal --> swimmer>";
                "<<robin --> [with_wings]> ==> <robin --> bird>>", "<robin --> flyer>", "<robin --> bird>" => "<<robin --> flyer> ==> <robin --> bird>>";
                "<(/,open,_,lock) --> (&,key,(/,open,_,{lock1}))>", "(/,open,_,lock)", "key" => "<(/,open,_,lock) --> key>";
                "<{Tweety} --> [with_wings]>", "[with_wings]", "flyer" => "<[with_wings] --> flyer>";
                "<(*,a,b) --> like>", "like", "(*,(/,like,b,_),b)" => "<like --> (*,(/,like,b,_),b)>";
                "<{key1} --> key>", "{key1}", "(/,open,_,{lock1})" => "<{key1} --> (/,open,_,{lock1})>";
                "<{key1} --> (/,open,_,{lock1})>", "{key1}", "(|,key,(/,open,_,{lock1}))" => "<{key1} --> (|,key,(/,open,_,{lock1}))>";
                "<bird --> (&,animal,swimmer)>", "bird", "swimmer" => "<bird --> swimmer>";
                "<flyer <-> [with_wings]>", "(|,flyer,{Birdie})", "(|,[with_wings],{Birdie})" => "<(|,flyer,{Birdie}) <-> (|,[with_wings],{Birdie})>";
                "<(*,(*,0)) --> (*,(*,(/,num,_)))>", "(*,(*,(/,num,_)))", "(*,(*,num))" => "<(*,(*,(/,num,_))) --> (*,(*,num))>";
                "<(&&,<robin --> [flying]>,<robin --> [with_wings]>) ==> <robin --> bird>>", "<robin --> [flying]>", "<robin --> bird>" => "<<robin --> [flying]> ==> <robin --> bird>>";
                "<{Tweety} --> [with_wings]>", "(|,flyer,{Birdie})", "[with_wings]" => "<(|,flyer,{Birdie}) --> [with_wings]>";
                "<gull --> swimmer>", "swan", "swimmer" => "<swan --> swimmer>";
                "<{Tweety} --> bird>", "flyer", "bird" => "<flyer --> bird>";
                "<(*,num) --> (*,(/,num,_))>", "num", "(/,num,_)" => "<num --> (/,num,_)>";
                "<{Tweety} --> [with_wings]>", "(&,flyer,{Tweety})", "(&,flyer,[with_wings])" => "<(&,flyer,{Tweety}) --> (&,flyer,[with_wings])>";
                "<(&&,<robin --> bird>,<robin --> [flying]>) ==> <robin --> [living]>>", "<robin --> bird>", "<robin --> [with_wings]>" => "<<robin --> bird> ==> <robin --> [with_wings]>>";
                "<{Tweety} --> (|,[with_wings],{Birdie})>", "(&,flyer,[yellow])", "(|,[with_wings],{Birdie})" => "<(&,flyer,[yellow]) --> (|,[with_wings],{Birdie})>";
                "<{key1} --> (&,key,(/,open,_,{lock1}))>", "{key1}", "(/,open,_,{lock1})" => "<{key1} --> (/,open,_,{lock1})>";
                "<num <-> (/,num,_)>", "(/,num,_)", "(/,(/,num,_),_)" => "<(/,num,_) <-> (/,(/,num,_),_)>";
                "<(&&,<robin --> bird>,<robin --> [flying]>) ==> <robin --> [living]>>", "<robin --> [flying]>", "<robin --> [living]>" => "<<robin --> [flying]> ==> <robin --> [living]>>";
                "<robin --> swan>", "animal", "robin" => "<animal --> robin>";
                "<{Tweety} --> flyer>", "flyer", "{Birdie}" => "<flyer --> {Birdie}>";
                "<(~,boy,girl) --> (~,youth,girl)>", "boy", "(~,youth,girl)" => "<boy --> (~,youth,girl)>";
                "<bird --> swimmer>", "(|,bird,swan)", "swimmer" => "<(|,bird,swan) --> swimmer>";
                "<bird --> {Birdie}>", "bird", "(|,bird,{Birdie})" => None;
                "<robin --> bird>", "robin", "swan" => "<robin --> swan>";
                "<(*,0) --> num>", "(/,(*,0),_)", "(/,num,_)" => "<(/,(*,0),_) --> (/,num,_)>";
                "<robin --> animal>", "swimmer", "robin" => "<swimmer --> robin>";
                "<robin --> bird>", "(|,robin,swan)", "bird" => "<(|,robin,swan) --> bird>";
                "<{Tweety} --> [with_wings]>", "(|,robin,{Tweety})", "[with_wings]" => "<(|,robin,{Tweety}) --> [with_wings]>";
                "<robin --> animal>", "robin", "swimmer" => "<robin --> swimmer>";
                "<0 --> num>", "num", "(/,num,_)" => "<num --> (/,num,_)>";
                "<bird --> swimmer>", "animal", "swimmer" => "<animal --> swimmer>";
                "<(*,(*,0)) --> (*,(*,(/,num,_)))>", "(*,(*,num))", "(*,(*,(/,num,_)))" => "<(*,(*,num)) --> (*,(*,(/,num,_)))>";
                "<planetX --> {Mars,Pluto,Venus}>", "planetX", "{Pluto}" => "<planetX --> {Pluto}>";
                "<{lock1} --> lock>", "{lock1}", "(&,lock,(/,open,{key1},_))" => "<{lock1} --> (&,lock,(/,open,{key1},_))>";
                "<robin --> bird>", "robin", "(|,animal,bird)" => "<robin --> (|,animal,bird)>";
                "<(/,(*,tim,tom),tom,_) --> (/,uncle,tom,_)>", "tom", "tom" => None;
                "<cat --> CAT>", "CAT", "(/,(/,REPRESENT,_,<(*,CAT,FISH) --> FOOD>),_,eat,fish)" => "<CAT --> (/,(/,REPRESENT,_,<(*,CAT,FISH) --> FOOD>),_,eat,fish)>";
                "<{Tweety} --> flyer>", "flyer", "bird" => "<flyer --> bird>";
                "<swimmer --> animal>", "(&,robin,swimmer)", "animal" => "<(&,robin,swimmer) --> animal>";
                "<{Tweety} --> (&,[with_wings],{Birdie})>", "{Tweety}", "[with_wings]" => "<{Tweety} --> [with_wings]>";
                "<swimmer --> animal>", "swimmer", "robin" => "<swimmer --> robin>";
                "<chess --> competition>", "sport", "competition" => "<sport --> competition>";
                "<cat --> CAT>", "(/,(/,REPRESENT,_,<(*,CAT,FISH) --> FOOD>),_,eat,fish)", "CAT" => "<(/,(/,REPRESENT,_,<(*,CAT,FISH) --> FOOD>),_,eat,fish) --> CAT>";
                "<(*,(*,(*,0))) --> num>", "num", "(*,(*,(*,(/,num,_))))" => "<num --> (*,(*,(*,(/,num,_))))>";
                "<robin --> [with_wings]>", "(|,robin,{Birdie})", "(|,[with_wings],{Birdie})" => "<(|,robin,{Birdie}) --> (|,[with_wings],{Birdie})>";
                "<robin --> bird>", "bird", "swimmer" => "<bird --> swimmer>";
                "<soda --> (/,reaction,acid,_)>", "soda", "(/,neutralization,acid,_)" => "<soda --> (/,neutralization,acid,_)>";
                "<(*,acid,base) --> reaction>", "neutralization", "reaction" => "<neutralization --> reaction>";
                "<{key1} --> (|,key,(/,open,_,{lock1}))>", "{key1}", "(/,open,_,{lock1})" => "<{key1} --> (/,open,_,{lock1})>";
                "<(&&,<robin --> bird>,<robin --> [flying]>) ==> <robin --> [living]>>", "<robin --> bird>", "<robin --> [living]>" => "<<robin --> bird> ==> <robin --> [living]>>";
                "<bird --> animal>", "(&,bird,robin)", "animal" => "<(&,bird,robin) --> animal>";
                "<swimmer --> bird>", "bird", "animal" => "<bird --> animal>";
                "<{lock1} --> lock>", "{lock1}", "(|,lock,(/,open,{key1},_))" => "<{lock1} --> (|,lock,(/,open,{key1},_))>";
                "<(&&,<robin --> [flying]>,<robin --> [with_wings]>) ==> <robin --> [living]>>", "<robin --> [flying]>", "<robin --> [living]>" => "<<robin --> [flying]> ==> <robin --> [living]>>";
                "<acid --> (/,reaction,_,base)>", "(&,acid,(/,neutralization,_,base))", "(/,reaction,_,base)" => "<(&,acid,(/,neutralization,_,base)) --> (/,reaction,_,base)>";
                "<(|,robin,swimmer) --> animal>", "swimmer", "animal" => "<swimmer --> animal>";
                "<swan --> (&,bird,swimmer)>", "robin", "swan" => "<robin --> swan>";
                "<robin --> animal>", "bird", "robin" => "<bird --> robin>";
                "<{Tweety} --> (|,flyer,[yellow])>", "bird", "(|,flyer,[yellow])" => "<bird --> (|,flyer,[yellow])>";
                "<(&,robin,{Tweety}) --> [with_wings]>", "(&,robin,{Birdie},{Tweety})", "(&,[with_wings],{Birdie})" => "<(&,robin,{Birdie},{Tweety}) --> (&,[with_wings],{Birdie})>";
                "<robin <-> swan>", "robin", "bird" => "<bird <-> robin>";
                "<{Tweety} --> flyer>", "{Tweety}", "(&,flyer,[yellow])" => "<{Tweety} --> (&,flyer,[yellow])>";
                "<bird --> animal>", "bird", "tiger" => "<bird --> tiger>";
                "<(/,neutralization,_,base) --> (/,reaction,_,base)>", "neutralization", "reaction" => "<neutralization --> reaction>";
                "<{?1} --> swimmer>", "{?1}", "robin" => "<{?1} --> robin>";
                "<(~,boy,girl) --> [strong]>", "(~,boy,girl)", "(|,[strong],(~,youth,girl))" => "<(~,boy,girl) --> (|,[strong],(~,youth,girl))>";
                "<robin --> [with_wings]>", "(|,flyer,robin)", "(|,flyer,[with_wings])" => "<(|,flyer,robin) --> (|,flyer,[with_wings])>";
                "<{Tweety} --> (&,[with_wings],{Birdie})>", "{Tweety}", "{Birdie}" => "<{Tweety} --> {Birdie}>";
                "<{Tweety} --> (|,bird,flyer)>", "{Tweety}", "(|,bird,flyer,{Birdie})" => "<{Tweety} --> (|,bird,flyer,{Birdie})>";
                "<{Tweety} --> [with_wings]>", "robin", "{Tweety}" => "<robin --> {Tweety}>";
                "<robin --> bird>", "(&,robin,swimmer)", "bird" => "<(&,robin,swimmer) --> bird>";
                "<{Tweety} --> [yellow]>", "(|,flyer,{Tweety})", "(|,flyer,[yellow])" => "<(|,flyer,{Tweety}) --> (|,flyer,[yellow])>";
                "<planetX --> {Mars,Pluto,Venus}>", "planetX", "{Mars,Pluto,Saturn,Venus}" => "<planetX --> {Mars,Pluto,Saturn,Venus}>";
                "<robin --> (|,bird,swimmer)>", "swan", "robin" => "<swan --> robin>";
                "<(&&,<robin --> bird>,<robin --> [flying]>) ==> <robin --> animal>>", "<robin --> bird>", "<robin --> animal>" => "<<robin --> bird> ==> <robin --> animal>>";
                "<planetX --> {Pluto,Saturn}>", "planetX", "{Mars,Pluto,Saturn,Venus}" => "<planetX --> {Mars,Pluto,Saturn,Venus}>";
                "<neutralization --> reaction>", "(/,neutralization,_,base)", "(/,reaction,_,base)" => "<(/,neutralization,_,base) --> (/,reaction,_,base)>";
                "<(&&,<robin --> [flying]>,<robin --> [with_wings]>) ==> <robin --> [living]>>", "<robin --> [with_wings]>", "<robin --> [living]>" => "<<robin --> [with_wings]> ==> <robin --> [living]>>";
                "<(&,bird,swimmer) --> (&,animal,swimmer)>", "swimmer", "swimmer" => None;
                "<cat --> CAT>", "cat", "(&,CAT,(/,(/,REPRESENT,_,<(*,CAT,FISH) --> FOOD>),_,eat,fish))" => "<cat --> (&,CAT,(/,(/,REPRESENT,_,<(*,CAT,FISH) --> FOOD>),_,eat,fish))>";
                "<neutralization <-> reaction>", "(/,neutralization,_,base)", "(/,reaction,_,base)" => "<(/,neutralization,_,base) <-> (/,reaction,_,base)>";
                "<robin --> [with_wings]>", "{Tweety}", "robin" => "<{Tweety} --> robin>";
                "<(*,(*,0)) --> (*,(*,(/,num,_)))>", "(*,0)", "(*,(/,num,_))" => "<(*,0) --> (*,(/,num,_))>";
                "<0 --> num>", "(*,0)", "(*,num)" => "<(*,0) --> (*,num)>";
                "<(|,robin,swan) --> (|,bird,swimmer)>", "robin", "bird" => "<robin --> bird>";
                "<robin --> bird>", "(&,robin,swan)", "bird" => "<(&,robin,swan) --> bird>";
                "<{Tweety} --> bird>", "bird", "{Birdie}" => "<bird --> {Birdie}>";
                "<{Tweety} --> (&,bird,{Birdie})>", "{Tweety}", "{Birdie}" => "<{Tweety} --> {Birdie}>";
                "<(&&,<robin --> [flying]>,<robin --> [with_wings]>) ==> <robin --> animal>>", "(&&,<robin --> [flying]>,<robin --> [with_wings]>)", "(||,<robin --> animal>,<robin --> bird>)" => "<(&&,<robin --> [flying]>,<robin --> [with_wings]>) ==> (||,<robin --> animal>,<robin --> bird>)>";
                "<(/,open,_,lock) --> (/,open,_,{lock1})>", "(/,open,_,lock)", "(&,key,(/,open,_,{lock1}))" => "<(/,open,_,lock) --> (&,key,(/,open,_,{lock1}))>";
                "<(|,chess,sport) --> (|,chess,competition)>", "sport", "competition" => "<sport --> competition>";
                "<(&&,<robin --> [chirping]>,<robin --> [flying]>,<robin --> [with_wings]>) ==> <robin --> bird>>", "(&&,<robin --> bird>,<robin --> [flying]>,<robin --> [with_wings]>)", "<robin --> bird>" => None;
                "<(|,robin,swan) --> (&,bird,swimmer)>", "(|,robin,swan)", "swimmer" => "<(|,robin,swan) --> swimmer>";
                "<(*,0) --> (*,num)>", "(*,(*,0))", "(*,(*,num))" => "<(*,(*,0)) --> (*,(*,num))>";
                "<robin --> bird>", "(~,swimmer,robin)", "bird" => "<(~,swimmer,robin) --> bird>";
                "<{Tweety} --> (|,bird,flyer)>", "(|,bird,{Birdie})", "(|,bird,flyer)" => "<(|,bird,{Birdie}) --> (|,bird,flyer)>";
                "<(/,neutralization,_,base) --> ?1>", "(/,reaction,_,base)", "?1" => "<(/,reaction,_,base) --> ?1>";
                "<(&,robin,swimmer) --> animal>", "(&,robin,swimmer)", "(|,animal,bird)" => "<(&,robin,swimmer) --> (|,animal,bird)>";
                "<{Tweety} --> flyer>", "flyer", "[with_wings]" => "<flyer --> [with_wings]>";
                "<(&&,<robin --> [chirping]>,<robin --> [with_wings]>) ==> <robin --> bird>>", "<robin --> [with_wings]>", "<robin --> bird>" => "<<robin --> [with_wings]> ==> <robin --> bird>>";
                "<{Tweety} --> (&,[yellow],{Birdie})>", "{Tweety}", "{Birdie}" => "<{Tweety} --> {Birdie}>";
                "<robin --> swimmer>", "robin", "bird" => "<robin --> bird>";
                "<robin --> bird>", "bird", "robin" => "<bird --> robin>";
                "<{Tweety} --> [with_wings]>", "{Tweety}", "(&,flyer,[with_wings])" => "<{Tweety} --> (&,flyer,[with_wings])>";
                "<bright <-> smart>", "[bright]", "[smart]" => "<[bright] <-> [smart]>";
                "<(&&,<robin --> [chirping]>,<robin --> [flying]>) ==> <robin --> bird>>", "<robin --> [chirping]>", "<robin --> bird>" => "<<robin --> [chirping]> ==> <robin --> bird>>";
                "<{key1} --> (|,key,(/,open,_,{lock1}))>", "{key1}", "(|,key,(/,open,_,{lock1}))" => "<{key1} --> (|,key,(/,open,_,{lock1}))>";
                "<{Tweety} --> [with_wings]>", "{Tweety}", "(|,[with_wings],{Birdie})" => "<{Tweety} --> (|,[with_wings],{Birdie})>";
                "<tim --> (/,uncle,_,tom)>", "(/,uncle,tom,_)", "(/,uncle,_,tom)" => "<(/,uncle,tom,_) --> (/,uncle,_,tom)>";
                "<(&,robin,swan) --> (&,bird,swimmer)>", "(&,robin,swan)", "bird" => "<(&,robin,swan) --> bird>";
                "<(&,robin,swimmer) --> animal>", "(&,robin,swimmer)", "(&,animal,bird)" => "<(&,robin,swimmer) --> (&,animal,bird)>";
                "<{Tweety} --> {Birdie}>", "{Tweety}", "(&,[with_wings],{Birdie})" => "<{Tweety} --> (&,[with_wings],{Birdie})>";
                "<swimmer --> bird>", "swimmer", "swan" => "<swimmer --> swan>";
                "<tiger --> animal>", "tiger", "swimmer" => "<tiger --> swimmer>";
                "<(*,(*,0)) --> (*,(*,(/,num,_)))>", "(*,(*,0))", "(|,(*,(*,num)),(*,(*,(/,num,_))))" => "<(*,(*,0)) --> (|,(*,(*,num)),(*,(*,(/,num,_))))>";
                "<{Birdie} --> [yellow]>", "{Birdie}", "(|,flyer,[yellow])" => "<{Birdie} --> (|,flyer,[yellow])>";
                "<sport --> chess>", "sport", "(&,chess,competition)" => "<sport --> (&,chess,competition)>";
                "<(&&,<robin --> [chirping]>,<robin --> [flying]>,<robin --> [with_wings]>) ==> <robin --> bird>>", "(&&,<robin --> [chirping]>,<robin --> [with_wings]>)", "<robin --> bird>" => "<(&&,<robin --> [chirping]>,<robin --> [with_wings]>) ==> <robin --> bird>>";
                "<{Tweety} --> [with_wings]>", "(&,robin,{Tweety})", "[with_wings]" => "<(&,robin,{Tweety}) --> [with_wings]>";
                "<?1 --> swimmer>", "animal", "?1" => "<animal --> ?1>";
                "<swimmer --> robin>", "(|,animal,swimmer)", "robin" => "<(|,animal,swimmer) --> robin>";
                "<{Tweety} --> flyer>", "{Tweety}", "(&,flyer,[with_wings])" => "<{Tweety} --> (&,flyer,[with_wings])>";
                "<{Birdie} --> [with_wings]>", "{Tweety}", "[with_wings]" => "<{Tweety} --> [with_wings]>";
                "<(|,robin,swimmer) --> bird>", "animal", "bird" => "<animal --> bird>";
                "<swimmer --> bird>", "animal", "bird" => "<animal --> bird>";
                "<robin --> bird>", "(~,swan,robin)", "bird" => "<(~,swan,robin) --> bird>";
                "<swimmer --> bird>", "swan", "swimmer" => "<swan --> swimmer>";
                "<{Tweety} --> flyer>", "{Tweety}", "(|,flyer,(&,[with_wings],{Birdie}))" => "<{Tweety} --> (|,flyer,(&,[with_wings],{Birdie}))>";
                "<tim --> (/,uncle,tom,_)>", "(~,(/,(*,tim,tom),tom,_),tim)", "(/,uncle,tom,_)" => "<(~,(/,(*,tim,tom),tom,_),tim) --> (/,uncle,tom,_)>";
                "<robin --> [with_wings]>", "(&,flyer,robin)", "(&,flyer,[with_wings])" => "<(&,flyer,robin) --> (&,flyer,[with_wings])>";
                "<planetX --> {Mars,Pluto,Saturn,Venus}>", "{Mars,Pluto,Venus}", "{Mars,Pluto,Saturn,Venus}" => "<{Mars,Pluto,Venus} --> {Mars,Pluto,Saturn,Venus}>";
                "<{Tweety} --> flyer>", "flyer", "[yellow]" => "<flyer --> [yellow]>";
                "<(|,boy,girl) --> youth>", "(|,boy,girl)", "youth" => "<(|,boy,girl) --> youth>";
                "<robin --> [with_wings]>", "(|,robin,{Birdie})", "[with_wings]" => "<(|,robin,{Birdie}) --> [with_wings]>";
                "<(|,robin,swan) --> (|,bird,swimmer)>", "swan", "swimmer" => "<swan --> swimmer>";
                "<robin --> animal>", "(&,robin,swimmer)", "animal" => "<(&,robin,swimmer) --> animal>";
                "<bird --> swimmer>", "robin", "bird" => "<robin --> bird>";
                "<(|,bird,swan) --> swimmer>", "swan", "swimmer" => "<swan --> swimmer>";
                "<{Tweety} --> (&,flyer,(|,[yellow],{Birdie}))>", "{Tweety}", "(|,[yellow],{Birdie})" => "<{Tweety} --> (|,[yellow],{Birdie})>";
                "<(&&,<robin --> [chirping]>,<robin --> [flying]>) ==> <robin --> bird>>", "(&&,<robin --> bird>,<robin --> [flying]>)", "<robin --> bird>" => None;
                "<(-,swimmer,animal) --> (-,swimmer,bird)>", "bird", "animal" => "<bird --> animal>";
                "<(/,neutralization,_,base) --> (/,reaction,_,base)>", "(/,neutralization,_,base)", "acid" => "<(/,neutralization,_,base) --> acid>";
                "<{Tweety} --> {Birdie}>", "bird", "{Birdie}" => "<bird --> {Birdie}>";
                "<{Tweety} --> {Birdie}>", "{Tweety}", "(|,bird,{Birdie})" => "<{Tweety} --> (|,bird,{Birdie})>";
                "<robin --> animal>", "bird", "animal" => "<bird --> animal>";
                "<swan --> swimmer>", "swan", "(|,bird,swimmer)" => "<swan --> (|,bird,swimmer)>";
                "<soda --> base>", "soda", "(/,reaction,acid,_)" => "<soda --> (/,reaction,acid,_)>";
                "<(--,<robin --> [flying]>) ==> <robin --> bird>>", "(--,<robin --> bird>)", "<robin --> [flying]>" => "<(--,<robin --> bird>) ==> <robin --> [flying]>>";
                "<{Tweety} --> (&,bird,flyer)>", "{Tweety}", "bird" => "<{Tweety} --> bird>";
                "<bird --> animal>", "(|,bird,robin)", "animal" => "<(|,bird,robin) --> animal>";
                "<0 --> (/,num,_)>", "(/,num,_)", "num" => "<(/,num,_) --> num>";
                "<robin --> swimmer>", "animal", "robin" => "<animal --> robin>";
                "<robin --> [with_wings]>", "{Birdie}", "robin" => "<{Birdie} --> robin>";
                "<(&,robin,swimmer) --> bird>", "(&,robin,swimmer)", "(&,animal,bird)" => "<(&,robin,swimmer) --> (&,animal,bird)>";
                "<(&,robin,swimmer) --> bird>", "bird", "animal" => "<bird --> animal>";
                "<(|,bird,{Tweety}) --> (|,bird,{Birdie})>", "bird", "bird" => None;
                "<(/,open,_,lock) --> (/,open,_,{lock1})>", "{lock1}", "lock" => "<{lock1} --> lock>";
                "<{Tweety} --> flyer>", "{Tweety}", "(&,flyer,(|,[with_wings],{Birdie}))" => "<{Tweety} --> (&,flyer,(|,[with_wings],{Birdie}))>";
                "<(*,a,b) --> like>", "(*,(/,like,b,_),b)", "like" => "<(*,(/,like,b,_),b) --> like>";
                "<robin --> animal>", "robin", "tiger" => "<robin --> tiger>";
                "<chess --> competition>", "(&,chess,sport)", "competition" => "<(&,chess,sport) --> competition>";
                "<[bright] <-> [smart]>", "bright", "smart" => "<bright <-> smart>";
                "<(/,(*,tim,tom),_,tom) --> (/,uncle,_,tom)>", "tom", "tom" => None;
                "<(&,robin,swan) --> (&,bird,swimmer)>", "(&,robin,swan)", "swimmer" => "<(&,robin,swan) --> swimmer>";
                "<{Tweety} --> flyer>", "flyer", "(|,[yellow],{Birdie})" => "<flyer --> (|,[yellow],{Birdie})>";
                "<(*,0) --> (*,num)>", "(*,(/,num,_))", "(*,num)" => "<(*,(/,num,_)) --> (*,num)>";
                "<{key1} --> key>", "(/,open,_,{lock1})", "key" => "<(/,open,_,{lock1}) --> key>";
                "<tiger --> robin>", "(|,swan,tiger)", "robin" => "<(|,swan,tiger) --> robin>";
                "<(|,boy,girl) --> youth>", "youth", "(|,boy,girl)" => "<youth --> (|,boy,girl)>";
                "<(*,b,a) --> (*,b,(/,like,b,_))>", "a", "(/,like,b,_)" => "<a --> (/,like,b,_)>";
                "<{Tweety} --> (|,bird,flyer)>", "{Tweety}", "(&,(|,bird,flyer),(|,bird,{Birdie}))" => "<{Tweety} --> (&,(|,bird,flyer),(|,bird,{Birdie}))>";
                "<b --> (/,like,_,a)>", "(*,a,b)", "(*,a,(/,like,_,a))" => "<(*,a,b) --> (*,a,(/,like,_,a))>";
                "<tiger --> robin>", "(&,swan,tiger)", "robin" => "<(&,swan,tiger) --> robin>";
                "<swan --> (|,bird,swimmer)>", "swan", "robin" => "<swan --> robin>";
                "<{Tweety} --> {Birdie}>", "(|,bird,{Tweety})", "(|,bird,{Birdie})" => "<(|,bird,{Tweety}) --> (|,bird,{Birdie})>";
                "<(|,robin,swimmer) --> bird>", "(|,robin,swimmer)", "(|,animal,bird)" => "<(|,robin,swimmer) --> (|,animal,bird)>";
                "<robin --> animal>", "(|,bird,robin)", "animal" => "<(|,bird,robin) --> animal>";
                "<bird --> swimmer>", "bird", "(|,animal,swimmer)" => "<bird --> (|,animal,swimmer)>";
                "<tim --> (/,uncle,tom,_)>", "(/,uncle,tom,_)", "(/,uncle,_,tom)" => "<(/,uncle,tom,_) --> (/,uncle,_,tom)>";
                "<tiger --> robin>", "swan", "tiger" => "<swan --> tiger>";
                "<robin --> [with_wings]>", "robin", "{Tweety}" => "<robin --> {Tweety}>";
                "<{Tweety} --> flyer>", "flyer", "(|,[with_wings],{Birdie})" => "<flyer --> (|,[with_wings],{Birdie})>";
                "<{Tweety} --> flyer>", "bird", "flyer" => "<bird --> flyer>";
                "<Birdie <-> Tweety>", "Birdie", "Tweety" => "<Birdie <-> Tweety>";
                "<bird --> swimmer>", "bird", "swan" => "<bird --> swan>";
                "<{Tweety} --> flyer>", "(&,[with_wings],{Birdie})", "flyer" => "<(&,[with_wings],{Birdie}) --> flyer>";
                "<tim --> (/,uncle,tom,_)>", "tim", "(/,(*,tim,tom),tom,_)" => "<tim --> (/,(*,tim,tom),tom,_)>";
                "<robin --> [with_wings]>", "robin", "(|,flyer,[with_wings])" => "<robin --> (|,flyer,[with_wings])>";
                "<[bright] --> [smart]>", "[smart]", "[bright]" => "<[smart] --> [bright]>";
                "<(~,boy,girl) --> (~,youth,girl)>", "boy", "youth" => "<boy --> youth>";
                "<{Birdie} <-> {Tweety}>", "{Tweety}", "bird" => "<bird <-> {Tweety}>";
                "<swan --> (|,bird,swimmer)>", "(&,robin,swan)", "(|,bird,swimmer)" => "<(&,robin,swan) --> (|,bird,swimmer)>";
                "<robin --> bird>", "robin", "animal" => "<robin --> animal>";
                "<{Tweety} --> {Birdie}>", "{Tweety}", "(|,[with_wings],{Birdie})" => "<{Tweety} --> (|,[with_wings],{Birdie})>";
                "<{Birdie} --> flyer>", "{Tweety}", "flyer" => "<{Tweety} --> flyer>";
                "<(&&,<robin --> [chirping]>,<robin --> [with_wings]>) ==> <robin --> bird>>", "(&&,<robin --> bird>,<robin --> [with_wings]>)", "<robin --> bird>" => None;
                "<CAT --> (/,(/,REPRESENT,_,<(*,CAT,FISH) --> FOOD>),_,eat,fish)>", "CAT", "(|,CAT,(/,(/,REPRESENT,_,<(*,CAT,FISH) --> FOOD>),_,eat,fish))" => None;
                "<(&&,<robin --> bird>,<robin --> [living]>) ==> <robin --> animal>>", "(&&,<robin --> bird>,<robin --> [flying]>)", "<robin --> animal>" => "<(&&,<robin --> bird>,<robin --> [flying]>) ==> <robin --> animal>>";
                "<{Tweety} --> (&,flyer,[yellow])>", "{Tweety}", "flyer" => "<{Tweety} --> flyer>";
                "<(|,bird,{Tweety}) --> (|,bird,flyer)>", "{Tweety}", "flyer" => "<{Tweety} --> flyer>";
                "<{Tweety} --> {Birdie}>", "{Tweety}", "(|,flyer,{Birdie})" => "<{Tweety} --> (|,flyer,{Birdie})>";
                "<{Tweety} --> [with_wings]>", "flyer", "[with_wings]" => "<flyer --> [with_wings]>";
                "<{Tweety} --> flyer>", "{Tweety}", "(&,flyer,{Birdie})" => "<{Tweety} --> (&,flyer,{Birdie})>";
                "<(/,neutralization,_,base) --> ?1>", "?1", "(/,reaction,_,base)" => "<?1 --> (/,reaction,_,base)>";
                "<swan --> (&,bird,swimmer)>", "swan", "bird" => "<swan --> bird>";
                "<swan --> swimmer>", "(~,swimmer,swan)", "swimmer" => None;
                "<robin --> bird>", "(|,robin,swimmer)", "bird" => "<(|,robin,swimmer) --> bird>";
                "<bird --> swimmer>", "(&,bird,swan)", "swimmer" => "<(&,bird,swan) --> swimmer>";
                "<(/,(*,tim,tom),tom,_) --> (/,uncle,tom,_)>", "(*,tim,tom)", "uncle" => "<(*,tim,tom) --> uncle>";
                "<(|,robin,swimmer) --> bird>", "bird", "animal" => "<bird --> animal>";
                "<robin --> (-,bird,swimmer)>", "robin", "swimmer" => "<robin --> swimmer>";
                "<(&&,<robin --> flyer>,<robin --> [chirping]>,<(*,robin,worms) --> food>) ==> <robin --> bird>>", "(&&,<robin --> bird>,<robin --> flyer>,<(*,robin,worms) --> food>)", "<robin --> bird>" => None;
                "<planetX --> {Mars,Pluto,Saturn,Venus}>", "{Mars,Pluto,Saturn,Venus}", "{Mars,Pluto,Venus}" => "<{Mars,Pluto,Saturn,Venus} --> {Mars,Pluto,Venus}>";
                "<?1 --> claimedByBob>", "(&,<bird --> fly>,<{Tweety} --> bird>)", "?1" => "<(&,<bird --> fly>,<{Tweety} --> bird>) --> ?1>";
                "<?1 --> swimmer>", "?1", "animal" => "<?1 --> animal>";
                "<robin --> swimmer>", "(&,bird,robin)", "swimmer" => "<(&,bird,robin) --> swimmer>";
                "<{?1} --> swimmer>", "{?1}", "bird" => "<{?1} --> bird>";
                "<(*,acid,base) --> reaction>", "reaction", "neutralization" => "<reaction --> neutralization>";
                "<tim --> (/,uncle,tom,_)>", "(/,uncle,_,tom)", "(/,uncle,tom,_)" => "<(/,uncle,_,tom) --> (/,uncle,tom,_)>";
                "<(*,b,a) --> (*,b,(/,like,b,_))>", "b", "b" => None;
                "<swan --> swimmer>", "gull", "swimmer" => "<gull --> swimmer>";
                "<neutralization --> (*,acid,base)>", "reaction", "neutralization" => "<reaction --> neutralization>";
                "<{Tweety} --> bird>", "{Tweety}", "(|,bird,{Birdie})" => "<{Tweety} --> (|,bird,{Birdie})>";
                "<(*,a,b) --> like>", "(*,a,b)", "(|,like,(*,(/,like,b,_),b))" => "<(*,a,b) --> (|,like,(*,(/,like,b,_),b))>";
                "<(|,bird,{Tweety}) --> (|,bird,flyer)>", "bird", "bird" => None;
                "<reaction --> neutralization>", "(/,reaction,acid,_)", "(/,neutralization,acid,_)" => "<(/,reaction,acid,_) --> (/,neutralization,acid,_)>";
                "<0 --> (/,num,_)>", "0", "num" => "<0 --> num>";
                "<swan --> swimmer>", "(&,swan,swimmer)", "swimmer" => None;
                "<<robin --> [with_wings]> ==> <robin --> bird>>", "<robin --> [with_wings]>", "(&&,<robin --> bird>,<robin --> [living]>)" => "<<robin --> [with_wings]> ==> (&&,<robin --> bird>,<robin --> [living]>)>";
                "<robin --> bird>", "swan", "bird" => "<swan --> bird>";
                "<robin --> bird>", "robin", "swimmer" => "<robin --> swimmer>";
                "<(&,robin,swimmer) --> bird>", "animal", "bird" => "<animal --> bird>";
                "<num <-> (/,num,_)>", "(*,num)", "(*,(/,num,_))" => "<(*,num) <-> (*,(/,num,_))>";
                "<(|,robin,{Tweety}) --> [with_wings]>", "robin", "[with_wings]" => "<robin --> [with_wings]>";
                "<(/,open,_,lock) --> (/,open,_,{lock1})>", "(/,open,_,lock)", "(|,key,(/,open,_,{lock1}))" => "<(/,open,_,lock) --> (|,key,(/,open,_,{lock1}))>";
                "<bird --> swimmer>", "swan", "bird" => "<swan --> bird>";
                "<(/,open,_,lock) --> (/,open,_,{lock1})>", "open", "open" => None;
                "<(*,0) --> (*,num)>", "(*,num)", "(*,(/,num,_))" => "<(*,num) --> (*,(/,num,_))>";
                "<{key1} --> (/,open,_,{lock1})>", "{key1}", "(&,key,(/,open,_,{lock1}))" => "<{key1} --> (&,key,(/,open,_,{lock1}))>";
                "<planetX --> {Mars,Venus}>", "planetX", "{Mars,Pluto,Saturn,Venus}" => "<planetX --> {Mars,Pluto,Saturn,Venus}>";
                "<(/,reaction,acid,_) --> soda>", "(/,reaction,acid,_)", "(&,soda,(/,neutralization,acid,_))" => "<(/,reaction,acid,_) --> (&,soda,(/,neutralization,acid,_))>";
                "<bird --> swimmer>", "robin", "swimmer" => "<robin --> swimmer>";
                "<(&&,<robin --> bird>,<robin --> [living]>) ==> <robin --> animal>>", "<robin --> bird>", "<robin --> animal>" => "<<robin --> bird> ==> <robin --> animal>>";
                "<robin --> animal>", "(|,robin,swan)", "animal" => "<(|,robin,swan) --> animal>";
                "<swimmer --> robin>", "bird", "robin" => "<bird --> robin>";
                "<swan --> swimmer>", "swan", "(&,bird,swimmer)" => "<swan --> (&,bird,swimmer)>";
                "<0 --> num>", "(/,num,_)", "num" => "<(/,num,_) --> num>";
                "<(&&,<robin --> bird>,<robin --> [living]>) ==> <robin --> animal>>", "(&&,<robin --> bird>,<robin --> [flying]>,<robin --> [with_wings]>)", "<robin --> animal>" => "<(&&,<robin --> bird>,<robin --> [flying]>,<robin --> [with_wings]>) ==> <robin --> animal>>";
                "<bird --> swimmer>", "swimmer", "animal" => "<swimmer --> animal>";
                "<{Tweety} --> flyer>", "[yellow]", "flyer" => "<[yellow] --> flyer>";
                "<(/,neutralization,_,base) --> ?1>", "(/,neutralization,_,base)", "(/,reaction,_,base)" => "<(/,neutralization,_,base) --> (/,reaction,_,base)>";
                "<sport --> competition>", "(|,chess,sport)", "competition" => "<(|,chess,sport) --> competition>";
                "<(&&,<robin --> flyer>,<robin --> [chirping]>) ==> <robin --> bird>>", "<robin --> flyer>", "<robin --> bird>" => "<<robin --> flyer> ==> <robin --> bird>>";
                "<(&,chess,sport) --> competition>", "chess", "competition" => "<chess --> competition>";
                "<(&&,<robin --> flyer>,<robin --> [chirping]>,<worms --> (/,food,robin,_)>) ==> <robin --> bird>>", "(&&,<robin --> bird>,<robin --> flyer>,<worms --> (/,food,robin,_)>)", "<robin --> bird>" => None;
                "<{Tweety} --> {Birdie}>", "{Tweety}", "(&,flyer,{Birdie})" => "<{Tweety} --> (&,flyer,{Birdie})>";
                "<robin --> bird>", "swimmer", "bird" => "<swimmer --> bird>";
                "<sport --> competition>", "sport", "chess" => "<sport --> chess>";
                "<{key1} --> (&,key,(/,open,_,{lock1}))>", "{key1}", "key" => "<{key1} --> key>";
                "<{Tweety} --> (&,flyer,[yellow])>", "{Tweety}", "[yellow]" => "<{Tweety} --> [yellow]>";
                "<(|,acid,(/,neutralization,_,base)) --> (/,reaction,_,base)>", "acid", "(/,reaction,_,base)" => "<acid --> (/,reaction,_,base)>";
                "<(|,bird,robin) --> animal>", "bird", "animal" => "<bird --> animal>";
                "<<robin --> [with_wings]> ==> <robin --> bird>>", "<robin --> [with_wings]>", "(||,<robin --> bird>,<robin --> [living]>)" => "<<robin --> [with_wings]> ==> (||,<robin --> bird>,<robin --> [living]>)>";
                "<(*,0) --> (*,(/,num,_))>", "(*,(*,0))", "(*,(*,(/,num,_)))" => "<(*,(*,0)) --> (*,(*,(/,num,_)))>";
                "<(|,boy,girl) --> (|,girl,youth)>", "boy", "girl" => "<boy --> girl>";
                "<sport --> competition>", "sport", "(|,chess,competition)" => "<sport --> (|,chess,competition)>";
                "<tim --> (/,uncle,tom,_)>", "(|,tim,(/,(*,tim,tom),tom,_))", "(/,uncle,tom,_)" => "<(|,tim,(/,(*,tim,tom),tom,_)) --> (/,uncle,tom,_)>";
                "<(&&,<robin --> bird>,<robin --> [flying]>,<robin --> [with_wings]>) ==> <robin --> animal>>", "(&&,<robin --> bird>,<robin --> [with_wings]>)", "<robin --> animal>" => "<(&&,<robin --> bird>,<robin --> [with_wings]>) ==> <robin --> animal>>";
                "<(&&,<robin --> bird>,<robin --> [flying]>) ==> <robin --> animal>>", "<robin --> [flying]>", "<robin --> animal>" => "<<robin --> [flying]> ==> <robin --> animal>>";
                "<(/,open,_,lock) --> key>", "(/,open,_,lock)", "(&,key,(/,open,_,{lock1}))" => "<(/,open,_,lock) --> (&,key,(/,open,_,{lock1}))>";
                "<{Tweety} --> (|,bird,flyer)>", "(|,bird,flyer)", "(|,bird,{Birdie})" => "<(|,bird,flyer) --> (|,bird,{Birdie})>";
                "<(&&,<robin --> bird>,<robin --> [living]>) ==> <robin --> animal>>", "<robin --> [living]>", "<robin --> animal>" => "<<robin --> [living]> ==> <robin --> animal>>";
                "<{Tweety} --> [with_wings]>", "[with_wings]", "(&,flyer,{Birdie})" => "<[with_wings] --> (&,flyer,{Birdie})>";
                "<a --> (/,like,b,_)>", "(*,a,b)", "(*,(/,like,b,_),b)" => "<(*,a,b) --> (*,(/,like,b,_),b)>";
                "<robin --> (&,animal,bird)>", "robin", "bird" => "<robin --> bird>";
                "<(&&,<robin --> flyer>,<robin --> [chirping]>,<(*,robin,worms) --> food>) ==> <robin --> bird>>", "(&&,<robin --> flyer>,<(*,robin,worms) --> food>)", "<robin --> bird>" => "<(&&,<robin --> flyer>,<(*,robin,worms) --> food>) ==> <robin --> bird>>";
                "<swimmer --> robin>", "robin", "swan" => "<robin --> swan>";
                "<(/,(*,tim,tom),tom,_) --> (/,uncle,tom,_)>", "(/,(*,tim,tom),tom,_)", "tim" => "<(/,(*,tim,tom),tom,_) --> tim>";
                "<tiger --> animal>", "(&,robin,tiger)", "animal" => "<(&,robin,tiger) --> animal>";
                "<(/,neutralization,_,base) --> (/,reaction,_,base)>", "(|,acid,(/,neutralization,_,base))", "(/,reaction,_,base)" => "<(|,acid,(/,neutralization,_,base)) --> (/,reaction,_,base)>";
                "<robin --> (&,animal,bird)>", "robin", "animal" => "<robin --> animal>";
                "<robin --> [with_wings]>", "robin", "{Birdie}" => "<robin --> {Birdie}>";
                "<{Tweety} --> (&,flyer,(|,[yellow],{Birdie}))>", "{Tweety}", "flyer" => "<{Tweety} --> flyer>";
                "<(&&,<robin --> swimmer>,<robin --> [flying]>) ==> <robin --> bird>>", "<robin --> swimmer>", "<robin --> bird>" => "<<robin --> swimmer> ==> <robin --> bird>>";
                "<swan --> (|,bird,swimmer)>", "(|,robin,swan)", "(|,bird,swimmer)" => "<(|,robin,swan) --> (|,bird,swimmer)>";
                "<{key1} --> key>", "key", "(/,open,_,{lock1})" => "<key --> (/,open,_,{lock1})>";
                "<robin --> animal>", "(&,bird,robin)", "animal" => "<(&,bird,robin) --> animal>";
                "<boy --> youth>", "(~,boy,girl)", "(~,youth,girl)" => "<(~,boy,girl) --> (~,youth,girl)>";
                "<(&&,<robin --> [flying]>,<robin --> [with_wings]>) ==> <robin --> animal>>", "<robin --> bird>", "<robin --> animal>" => "<<robin --> bird> ==> <robin --> animal>>";
                "<bird --> animal>", "bird", "swimmer" => "<bird --> swimmer>";
                "<tim --> (/,uncle,_,tom)>", "(/,uncle,_,tom)", "(/,uncle,tom,_)" => "<(/,uncle,_,tom) --> (/,uncle,tom,_)>";
                "<(~,boy,girl) --> (&,[strong],(~,youth,girl))>", "(~,boy,girl)", "(&,[strong],(~,youth,girl))" => "<(~,boy,girl) --> (&,[strong],(~,youth,girl))>";
                "<[with_wings] --> {Birdie}>", "[with_wings]", "{Tweety}" => "<[with_wings] --> {Tweety}>";
                "<(&,robin,{Tweety}) --> [with_wings]>", "(&,flyer,robin,{Tweety})", "(&,flyer,[with_wings])" => "<(&,flyer,robin,{Tweety}) --> (&,flyer,[with_wings])>";
                "<tiger --> animal>", "(&,robin,tiger)", "(&,animal,robin)" => "<(&,robin,tiger) --> (&,animal,robin)>";
                "<swan --> (&,bird,swimmer)>", "(&,robin,swan)", "(&,bird,swimmer)" => "<(&,robin,swan) --> (&,bird,swimmer)>";
                "<sport --> chess>", "sport", "(|,chess,competition)" => "<sport --> (|,chess,competition)>";
                "<sport --> chess>", "chess", "competition" => "<chess --> competition>";
                "<{Tweety} --> flyer>", "{Tweety}", "(|,bird,flyer)" => "<{Tweety} --> (|,bird,flyer)>";
                "<(|,boy,girl) --> (~,youth,girl)>", "(~,youth,girl)", "(|,boy,girl)" => "<(~,youth,girl) --> (|,boy,girl)>";
                "<soda --> base>", "(/,reaction,acid,_)", "soda" => "<(/,reaction,acid,_) --> soda>";
                "<{key1} --> (/,open,_,{lock1})>", "key", "(/,open,_,{lock1})" => "<key --> (/,open,_,{lock1})>";
                "<robin --> (-,bird,swimmer)>", "robin", "bird" => "<robin --> bird>";
                "<{Tweety} --> flyer>", "{Tweety}", "(|,flyer,[with_wings])" => "<{Tweety} --> (|,flyer,[with_wings])>";
                "<(~,boy,girl) --> [strong]>", "[strong]", "(~,youth,girl)" => "<[strong] --> (~,youth,girl)>";
                "<robin --> animal>", "tiger", "robin" => "<tiger --> robin>";
                "<robin --> animal>", "(&,robin,swan)", "animal" => "<(&,robin,swan) --> animal>";
                "<{Tweety} --> {Birdie}>", "{Birdie}", "[yellow]" => "<{Birdie} --> [yellow]>";
                "<swimmer --> robin>", "swimmer", "animal" => "<swimmer --> animal>";
                "<bird --> (&,animal,swimmer)>", "bird", "animal" => "<bird --> animal>";
                "<{Tweety} --> {Birdie}>", "{Tweety}", "(&,bird,{Birdie})" => "<{Tweety} --> (&,bird,{Birdie})>";
                "<swimmer --> robin>", "(&,animal,swimmer)", "robin" => "<(&,animal,swimmer) --> robin>";
                "<planetX --> {Pluto,Saturn}>", "{Mars,Pluto,Venus}", "{Pluto,Saturn}" => "<{Mars,Pluto,Venus} --> {Pluto,Saturn}>";
                "<{Tweety} --> {Birdie}>", "{Birdie}", "flyer" => "<{Birdie} --> flyer>";
                "<{Tweety} --> [with_wings]>", "{Tweety}", "(&,[with_wings],(|,flyer,{Birdie}))" => "<{Tweety} --> (&,[with_wings],(|,flyer,{Birdie}))>";
                "<{Mars,Pluto,Saturn,Venus} --> {Mars,Pluto,Venus}>", "{Saturn}", "{Mars,Pluto,Venus}" => "<{Saturn} --> {Mars,Pluto,Venus}>";
                "<{Tweety} --> [with_wings]>", "{Birdie,Tweety}", "(|,[with_wings],{Birdie})" => "<{Birdie,Tweety} --> (|,[with_wings],{Birdie})>";
                "<{Tweety} --> flyer>", "{Tweety}", "(|,flyer,{Birdie})" => "<{Tweety} --> (|,flyer,{Birdie})>";
                "<robin --> [with_wings]>", "(|,robin,{Tweety})", "[with_wings]" => "<(|,robin,{Tweety}) --> [with_wings]>";
                "<acid --> (/,reaction,_,base)>", "(|,acid,(/,neutralization,_,base))", "(/,reaction,_,base)" => "<(|,acid,(/,neutralization,_,base)) --> (/,reaction,_,base)>";
                "<{Tweety} --> {Birdie}>", "{Tweety}", "(|,[yellow],{Birdie})" => "<{Tweety} --> (|,[yellow],{Birdie})>";
                "<{Tweety} --> bird>", "{Tweety}", "(&,bird,{Birdie})" => "<{Tweety} --> (&,bird,{Birdie})>";
                "<{Mars,Pluto,Saturn,Venus} --> {Mars,Pluto,Venus}>", "{Venus}", "{Mars,Pluto,Venus}" => "<{Venus} --> {Mars,Pluto,Venus}>";
                "<tim --> (/,uncle,tom,_)>", "(/,(*,tim,tom),tom,_)", "tim" => "<(/,(*,tim,tom),tom,_) --> tim>";
                "<planetX --> {Pluto,Saturn}>", "planetX", "{Mars,Venus}" => "<planetX --> {Mars,Venus}>";
                "<soda --> (/,reaction,acid,_)>", "(/,neutralization,acid,_)", "soda" => "<(/,neutralization,acid,_) --> soda>";
                "<(&&,<robin --> [chirping]>,<robin --> [flying]>,<robin --> [with_wings]>) ==> <robin --> bird>>", "(&&,<robin --> [chirping]>,<robin --> [flying]>)", "<robin --> bird>" => "<(&&,<robin --> [chirping]>,<robin --> [flying]>) ==> <robin --> bird>>";
                "<(&&,<robin --> [flying]>,<robin --> [with_wings]>) ==> <robin --> animal>>", "(&&,<robin --> [flying]>,<robin --> [with_wings]>)", "(&&,<robin --> animal>,<robin --> bird>)" => "<(&&,<robin --> [flying]>,<robin --> [with_wings]>) ==> (&&,<robin --> animal>,<robin --> bird>)>";
                "<neutralization --> (*,acid,base)>", "neutralization", "reaction" => "<neutralization --> reaction>";
                "<(*,a,b) --> like>", "(*,a,b)", "(&,like,(*,(/,like,b,_),b))" => "<(*,a,b) --> (&,like,(*,(/,like,b,_),b))>";
                "<sport --> competition>", "(&,chess,sport)", "competition" => "<(&,chess,sport) --> competition>";
                "<(/,open,_,lock) --> (&,key,(/,open,_,{lock1}))>", "(/,open,_,lock)", "(/,open,_,{lock1})" => "<(/,open,_,lock) --> (/,open,_,{lock1})>";
                "<[yellow] <-> {Birdie}>", "(|,flyer,[yellow])", "(|,flyer,{Birdie})" => "<(|,flyer,[yellow]) <-> (|,flyer,{Birdie})>";
                "<bird --> swimmer>", "swimmer", "robin" => "<swimmer --> robin>";
                "<{Tweety} --> flyer>", "{Tweety}", "(&,bird,flyer)" => "<{Tweety} --> (&,bird,flyer)>";
                "<{Tweety} --> (&,flyer,{Birdie})>", "{Tweety}", "flyer" => "<{Tweety} --> flyer>";
                "<(/,neutralization,_,base) --> (/,reaction,_,base)>", "acid", "(/,neutralization,_,base)" => "<acid --> (/,neutralization,_,base)>";
                "<{Tweety} --> flyer>", "[with_wings]", "flyer" => "<[with_wings] --> flyer>";
                "<planetX --> {Pluto,Saturn}>", "planetX", "{Pluto}" => "<planetX --> {Pluto}>";
                "<(~,boy,girl) --> [strong]>", "boy", "[strong]" => "<boy --> [strong]>";
                "<(/,reaction,acid,_) --> soda>", "(/,neutralization,acid,_)", "soda" => "<(/,neutralization,acid,_) --> soda>";
                "<(|,robin,{Tweety}) --> [with_wings]>", "{Tweety}", "[with_wings]" => "<{Tweety} --> [with_wings]>";
                "<(|,robin,tiger) --> animal>", "tiger", "animal" => "<tiger --> animal>";
                "<robin --> bird>", "bird", "animal" => "<bird --> animal>";
                "<planetX --> {Mars,Venus}>", "{Pluto,Saturn}", "{Mars,Venus}" => "<{Pluto,Saturn} --> {Mars,Venus}>";
                "<(/,(*,tim,tom),tom,_) --> (/,uncle,tom,_)>", "tim", "(/,(*,tim,tom),tom,_)" => "<tim --> (/,(*,tim,tom),tom,_)>";
                "<<robin --> [with_wings]> ==> <robin --> [living]>>", "<robin --> flyer>", "<robin --> [living]>" => "<<robin --> flyer> ==> <robin --> [living]>>";
                "<chess --> competition>", "(|,chess,sport)", "competition" => "<(|,chess,sport) --> competition>";
                "<swan --> swimmer>", "swimmer", "bird" => "<swimmer --> bird>";
                "<robin --> (-,mammal,swimmer)>", "robin", "swimmer" => "<robin --> swimmer>";
                "<(|,robin,swan) --> (&,bird,swimmer)>", "(|,robin,swan)", "bird" => "<(|,robin,swan) --> bird>";
                "<{Tweety} --> (&,bird,{Birdie})>", "{Tweety}", "bird" => "<{Tweety} --> bird>";
                "<chess --> competition>", "chess", "(|,chess,competition)" => None;
                "<(/,open,_,lock) --> key>", "(/,open,_,{lock1})", "key" => "<(/,open,_,{lock1}) --> key>";
                "<(&&,<robin --> bird>,<robin --> [flying]>,<robin --> [with_wings]>) ==> <robin --> animal>>", "(&&,<robin --> bird>,<robin --> [flying]>)", "<robin --> animal>" => "<(&&,<robin --> bird>,<robin --> [flying]>) ==> <robin --> animal>>";
                "<(/,num,_) --> num>", "(*,(/,num,_))", "(*,num)" => "<(*,(/,num,_)) --> (*,num)>";
                "<robin --> swimmer>", "(|,bird,robin)", "swimmer" => "<(|,bird,robin) --> swimmer>";
                "<(/,open,_,lock) --> key>", "key", "(/,open,_,{lock1})" => "<key --> (/,open,_,{lock1})>";
                "<{lock1} --> lock>", "lock", "(/,open,{key1},_)" => "<lock --> (/,open,{key1},_)>";
                "<[yellow] --> {Birdie}>", "(|,flyer,[yellow])", "(|,flyer,{Birdie})" => "<(|,flyer,[yellow]) --> (|,flyer,{Birdie})>";
                "<chess --> competition>", "(~,sport,chess)", "competition" => "<(~,sport,chess) --> competition>";
                "<(*,a,b) --> (&,like,(*,(/,like,b,_),b))>", "(*,a,b)", "(&,like,(*,(/,like,b,_),b))" => "<(*,a,b) --> (&,like,(*,(/,like,b,_),b))>";
                "<{Tweety} --> [with_wings]>", "{Tweety}", "robin" => "<{Tweety} --> robin>";
                "<chess --> competition>", "chess", "sport" => "<chess --> sport>";
                "<{Birdie} <-> {Tweety}>", "Birdie", "Tweety" => "<Birdie <-> Tweety>";
                "<bird --> swimmer>", "bird", "(&,animal,swimmer)" => "<bird --> (&,animal,swimmer)>";
                "<{Tweety} --> flyer>", "(|,[yellow],{Birdie})", "flyer" => "<(|,[yellow],{Birdie}) --> flyer>";
                "<(|,chess,sport) --> competition>", "sport", "competition" => "<sport --> competition>";
                "<planetX --> {Mars,Pluto,Venus}>", "{Mars,Pluto,Venus}", "{Pluto,Saturn}" => "<{Mars,Pluto,Venus} --> {Pluto,Saturn}>";
                "<robin --> animal>", "(|,robin,swimmer)", "animal" => "<(|,robin,swimmer) --> animal>";
                "<[yellow] <-> {Birdie}>", "[yellow]", "{Tweety}" => "<[yellow] <-> {Tweety}>";
                "<(|,robin,swan) --> (|,bird,swimmer)>", "swan", "(|,bird,swimmer)" => "<swan --> (|,bird,swimmer)>";
                "<{Tweety} --> (&,[yellow],{Birdie})>", "{Tweety}", "[yellow]" => "<{Tweety} --> [yellow]>";
                "<(/,(*,0),_) --> (/,num,_)>", "(*,(/,(*,0),_))", "(*,(/,num,_))" => "<(*,(/,(*,0),_)) --> (*,(/,num,_))>";
                "<{Tweety} --> [with_wings]>", "{Tweety}", "(|,flyer,[with_wings],{Birdie})" => "<{Tweety} --> (|,flyer,[with_wings],{Birdie})>";
                "<(/,neutralization,_,base) --> (/,reaction,_,base)>", "(&,acid,(/,neutralization,_,base))", "(/,reaction,_,base)" => "<(&,acid,(/,neutralization,_,base)) --> (/,reaction,_,base)>";
                "<{Tweety} --> bird>", "bird", "flyer" => "<bird --> flyer>";
                "<(/,reaction,acid,_) --> soda>", "(/,reaction,acid,_)", "(|,soda,(/,neutralization,acid,_))" => "<(/,reaction,acid,_) --> (|,soda,(/,neutralization,acid,_))>";
                "<(&&,<robin --> [flying]>,<robin --> [with_wings]>) ==> <robin --> animal>>", "<robin --> animal>", "<robin --> bird>" => "<<robin --> animal> ==> <robin --> bird>>";
                "<cat --> (&,CAT,(/,(/,REPRESENT,_,<(*,CAT,FISH) --> FOOD>),_,eat,fish))>", "cat", "(/,(/,REPRESENT,_,<(*,CAT,FISH) --> FOOD>),_,eat,fish)" => "<cat --> (/,(/,REPRESENT,_,<(*,CAT,FISH) --> FOOD>),_,eat,fish)>";
                "<{lock1} --> (&,lock,(/,open,{key1},_))>", "{lock1}", "lock" => "<{lock1} --> lock>";
                "<(/,(*,tim,tom),tom,_) --> (/,uncle,tom,_)>", "(|,tim,(/,(*,tim,tom),tom,_))", "(/,uncle,tom,_)" => "<(|,tim,(/,(*,tim,tom),tom,_)) --> (/,uncle,tom,_)>";
                "<{lock1} --> lock>", "(/,open,_,lock)", "(/,open,_,{lock1})" => "<(/,open,_,lock) --> (/,open,_,{lock1})>";
                "<robin --> [with_wings]>", "(&,robin,{Tweety})", "[with_wings]" => "<(&,robin,{Tweety}) --> [with_wings]>";
                "<robin --> swan>", "robin", "bird" => "<robin --> bird>";
                "<{Tweety} --> [with_wings]>", "[with_wings]", "(|,flyer,{Birdie})" => "<[with_wings] --> (|,flyer,{Birdie})>";
                "<?1 --> claimedByBob>", "?1", "(&,<bird --> fly>,<{Tweety} --> bird>)" => "<?1 --> (&,<bird --> fly>,<{Tweety} --> bird>)>";
                "<(|,boy,girl) --> youth>", "girl", "youth" => "<girl --> youth>";
                "<(&,robin,swan) --> (&,bird,swimmer)>", "swan", "swimmer" => "<swan --> swimmer>";
                "<boy --> [strong]>", "(~,boy,girl)", "(~,[strong],girl)" => "<(~,boy,girl) --> (~,[strong],girl)>";
                "<(|,robin,swimmer) --> bird>", "robin", "bird" => "<robin --> bird>";
                "<(&&,<robin --> flyer>,<robin --> [chirping]>) ==> <robin --> bird>>", "(&&,<robin --> bird>,<robin --> flyer>)", "<robin --> bird>" => None;
                "<tim --> (/,uncle,tom,_)>", "(&,tim,(/,(*,tim,tom),tom,_))", "(/,uncle,tom,_)" => "<(&,tim,(/,(*,tim,tom),tom,_)) --> (/,uncle,tom,_)>";
                "<{Tweety} --> [yellow]>", "{Tweety}", "(|,flyer,[yellow])" => "<{Tweety} --> (|,flyer,[yellow])>";
                "<robin --> swimmer>", "robin", "animal" => "<robin --> animal>";
                "<swimmer --> animal>", "(|,robin,swimmer)", "animal" => "<(|,robin,swimmer) --> animal>";
                "<(|,bird,robin) --> animal>", "robin", "animal" => "<robin --> animal>";
                "<(~,boy,girl) --> [strong]>", "(~,youth,girl)", "[strong]" => "<(~,youth,girl) --> [strong]>";
                "<robin --> bird>", "swan", "robin" => "<swan --> robin>";
                "<(~,boy,girl) --> [strong]>", "(~,boy,girl)", "[strong]" => "<(~,boy,girl) --> [strong]>";
                "<swan --> (|,bird,swimmer)>", "robin", "swan" => "<robin --> swan>";
                "<(/,(*,tim,tom),tom,_) --> (/,uncle,tom,_)>", "(&,tim,(/,(*,tim,tom),tom,_))", "(/,uncle,tom,_)" => "<(&,tim,(/,(*,tim,tom),tom,_)) --> (/,uncle,tom,_)>";
                "<{Tweety} --> {Birdie}>", "{Birdie}", "bird" => "<{Birdie} --> bird>";
                "<{Tweety} --> [yellow]>", "{Birdie,Tweety}", "(|,[yellow],{Birdie})" => "<{Birdie,Tweety} --> (|,[yellow],{Birdie})>";
                "<(*,(*,0)) --> (*,(*,(/,num,_)))>", "(*,(*,0))", "(&,(*,(*,num)),(*,(*,(/,num,_))))" => "<(*,(*,0)) --> (&,(*,(*,num)),(*,(*,(/,num,_))))>";
                "<bird --> swimmer>", "(&,bird,robin)", "swimmer" => "<(&,bird,robin) --> swimmer>";
                "<(--,<robin --> bird>) ==> <robin --> [flying]>>", "(--,<robin --> [flying]>)", "<robin --> bird>" => "<(--,<robin --> [flying]>) ==> <robin --> bird>>";
                "<(*,0) --> (*,(/,num,_))>", "(*,num)", "(*,(/,num,_))" => "<(*,num) --> (*,(/,num,_))>";
                "<robin --> bird>", "animal", "bird" => "<animal --> bird>";
                "<(|,chess,sport) --> competition>", "chess", "competition" => "<chess --> competition>";
                "<(|,boy,girl) --> youth>", "(|,boy,girl)", "(~,youth,girl)" => "<(|,boy,girl) --> (~,youth,girl)>";
                "<planetX --> {Mars,Pluto,Venus}>", "{Pluto,Saturn}", "{Mars,Pluto,Venus}" => "<{Pluto,Saturn} --> {Mars,Pluto,Venus}>";
                "<(|,boy,girl) --> youth>", "(~,(|,boy,girl),girl)", "(~,youth,girl)" => "<(~,(|,boy,girl),girl) --> (~,youth,girl)>";
                "<boy --> youth>", "(|,boy,girl)", "(|,girl,youth)" => "<(|,boy,girl) --> (|,girl,youth)>";
                "<sport --> competition>", "(|,chess,sport)", "(|,chess,competition)" => "<(|,chess,sport) --> (|,chess,competition)>";
                "<(&&,<robin --> [chirping]>,<robin --> [with_wings]>) ==> <robin --> bird>>", "(&&,<robin --> bird>,<robin --> [chirping]>)", "<robin --> bird>" => None;
                "<reaction --> neutralization>", "(/,reaction,_,base)", "(/,neutralization,_,base)" => "<(/,reaction,_,base) --> (/,neutralization,_,base)>";
                "<robin --> animal>", "robin", "bird" => "<robin --> bird>";
                "<(*,0) --> (*,num)>", "(*,0)", "(&,(*,num),(*,(/,num,_)))" => "<(*,0) --> (&,(*,num),(*,(/,num,_)))>";
                "<(*,0) --> (*,num)>", "0", "num" => "<0 --> num>";
                "<{Birdie} --> [yellow]>", "(&,flyer,{Birdie})", "(&,flyer,[yellow])" => "<(&,flyer,{Birdie}) --> (&,flyer,[yellow])>";
                "<robin --> swimmer>", "bird", "robin" => "<bird --> robin>";
                "<(&&,<robin --> bird>,<robin --> [flying]>,<robin --> [with_wings]>) ==> <robin --> animal>>", "(&&,<robin --> [flying]>,<robin --> [with_wings]>)", "<robin --> animal>" => "<(&&,<robin --> [flying]>,<robin --> [with_wings]>) ==> <robin --> animal>>";
                "<robin --> (|,bird,swimmer)>", "(|,robin,swan)", "(|,bird,swimmer)" => "<(|,robin,swan) --> (|,bird,swimmer)>";
                "<(-,swimmer,animal) --> (-,swimmer,bird)>", "swimmer", "swimmer" => None;
                "<robin --> bird>", "robin", "(&,animal,bird)" => "<robin --> (&,animal,bird)>";
                "<(&,robin,swimmer) --> bird>", "(&,robin,swimmer)", "(|,animal,bird)" => "<(&,robin,swimmer) --> (|,animal,bird)>";
                "<{Birdie} --> flyer>", "(&,flyer,{Birdie})", "flyer" => None;
                "<acid --> (/,reaction,_,base)>", "(/,neutralization,_,base)", "acid" => "<(/,neutralization,_,base) --> acid>";
                "<(/,neutralization,_,base) --> (/,reaction,_,base)>", "base", "base" => None;
                "<robin --> [with_wings]>", "(&,robin,{Birdie})", "[with_wings]" => "<(&,robin,{Birdie}) --> [with_wings]>";
                "<{Tweety} --> flyer>", "{Birdie}", "flyer" => "<{Birdie} --> flyer>";
                "<sport --> chess>", "competition", "chess" => "<competition --> chess>";
                "<{Tweety} --> (|,[with_wings],{Birdie})>", "{Tweety}", "(&,flyer,[yellow],(|,[with_wings],{Birdie}))" => "<{Tweety} --> (&,flyer,[yellow],(|,[with_wings],{Birdie}))>";
                "<(&&,<robin --> [flying]>,<robin --> [with_wings]>) ==> <robin --> bird>>", "<robin --> [with_wings]>", "<robin --> bird>" => "<<robin --> [with_wings]> ==> <robin --> bird>>";
                "<robin --> swan>", "robin", "gull" => "<robin --> gull>";
                "<num --> (/,num,_)>", "(*,num)", "(*,(/,num,_))" => "<(*,num) --> (*,(/,num,_))>";
                "<(&,robin,swimmer) --> animal>", "bird", "animal" => "<bird --> animal>";
                "<{Birdie} --> [yellow]>", "{Birdie}", "(|,[yellow],{Birdie})" => None;
                "<swimmer --> animal>", "robin", "swimmer" => "<robin --> swimmer>";
                "<planetX --> {Mars,Pluto,Venus}>", "planetX", "{Mars,Venus}" => "<planetX --> {Mars,Venus}>";
                "<robin --> swan>", "robin", "animal" => "<robin --> animal>";
                "<{Tweety} --> {Birdie}>", "flyer", "{Birdie}" => "<flyer --> {Birdie}>";
                "<swimmer --> robin>", "swan", "robin" => "<swan --> robin>";
                "<{Tweety} --> [with_wings]>", "{Tweety}", "(&,flyer,[with_wings],{Birdie})" => "<{Tweety} --> (&,flyer,[with_wings],{Birdie})>";
                "<swimmer --> bird>", "swimmer", "bird" => "<swimmer --> bird>";
                "<robin --> (|,bird,swimmer)>", "robin", "swan" => "<robin --> swan>";
                "<bird --> animal>", "tiger", "bird" => "<tiger --> bird>";
                "<(*,tim,tom) --> uncle>", "(/,(*,tim,tom),_,tom)", "(/,uncle,_,tom)" => "<(/,(*,tim,tom),_,tom) --> (/,uncle,_,tom)>";
                "<{lock1} --> (|,lock,(/,open,{key1},_))>", "(/,open,_,(|,lock,(/,open,{key1},_)))", "(/,open,_,{lock1})" => "<(/,open,_,(|,lock,(/,open,{key1},_))) --> (/,open,_,{lock1})>";
                "<b --> (/,like,_,a)>", "(/,like,(/,like,_,a),_)", "(/,like,b,_)" => "<(/,like,(/,like,_,a),_) --> (/,like,b,_)>";
                "<bird --> animal>", "bird", "robin" => "<bird --> robin>";
                "<(*,tim,tom) --> uncle>", "(/,(*,tim,tom),tim,_)", "(/,uncle,tim,_)" => "<(/,(*,tim,tom),tim,_) --> (/,uncle,tim,_)>";
                "<(/,reaction,acid,_) --> soda>", "soda", "(/,neutralization,acid,_)" => "<soda --> (/,neutralization,acid,_)>";
                "<{Birdie} <-> {Tweety}>", "{Birdie}", "{Tweety}" => "<{Birdie} <-> {Tweety}>";
                "<(/,neutralization,acid,_) <-> (/,reaction,acid,_)>", "acid", "acid" => None;
                "<{Tweety} --> (&,flyer,[with_wings])>", "{Tweety}", "flyer" => "<{Tweety} --> flyer>";
                "<swan --> (&,bird,swimmer)>", "swan", "swimmer" => "<swan --> swimmer>";
                "<bird --> animal>", "(|,bird,tiger)", "animal" => "<(|,bird,tiger) --> animal>";
                "<{Tweety} --> {Birdie}>", "{Tweety}", "(&,[yellow],{Birdie})" => "<{Tweety} --> (&,[yellow],{Birdie})>";
                "<0 --> (/,num,_)>", "(*,0)", "(*,(/,num,_))" => "<(*,0) --> (*,(/,num,_))>";
                "<{Tweety} --> flyer>", "flyer", "(&,[with_wings],{Birdie})" => "<flyer --> (&,[with_wings],{Birdie})>";
                "<swan --> (&,bird,swimmer)>", "swan", "robin" => "<swan --> robin>";
                "<(|,robin,tiger) --> animal>", "robin", "animal" => "<robin --> animal>";
                "<(/,(*,tim,tom),tom,_) --> (/,uncle,tom,_)>", "(~,(/,(*,tim,tom),tom,_),tim)", "(/,uncle,tom,_)" => "<(~,(/,(*,tim,tom),tom,_),tim) --> (/,uncle,tom,_)>";
                "<neutralization <-> reaction>", "(/,neutralization,acid,_)", "(/,reaction,acid,_)" => "<(/,neutralization,acid,_) <-> (/,reaction,acid,_)>";
                "<(~,boy,girl) --> [strong]>", "(~,boy,girl)", "(&,[strong],(~,youth,girl))" => "<(~,boy,girl) --> (&,[strong],(~,youth,girl))>";
                "<lock1 --> lock>", "lock", "lock1" => "<lock --> lock1>";
                "<{Tweety} --> (|,bird,flyer)>", "(|,bird,{Tweety})", "(|,bird,flyer)" => "<(|,bird,{Tweety}) --> (|,bird,flyer)>";
                "<cat --> (&,CAT,(/,(/,REPRESENT,_,<(*,CAT,FISH) --> FOOD>),_,eat,fish))>", "cat", "CAT" => "<cat --> CAT>";
                "<{Tweety} --> (|,[with_wings],{Birdie})>", "{Tweety}", "(|,[with_wings],{Birdie},(&,flyer,[yellow]))" => "<{Tweety} --> (|,[with_wings],{Birdie},(&,flyer,[yellow]))>";
                "<base --> (/,reaction,acid,_)>", "(/,neutralization,acid,_)", "base" => "<(/,neutralization,acid,_) --> base>";
                "<{Tweety} --> (&,flyer,[with_wings])>", "{Tweety}", "[with_wings]" => "<{Tweety} --> [with_wings]>";
                "<swimmer --> bird>", "swimmer", "(&,animal,bird)" => "<swimmer --> (&,animal,bird)>";
                "<(|,bird,swan) --> swimmer>", "bird", "swimmer" => "<bird --> swimmer>";
                "<{Tweety} --> flyer>", "{Tweety}", "(&,flyer,(|,[yellow],{Birdie}))" => "<{Tweety} --> (&,flyer,(|,[yellow],{Birdie}))>";
                "<{Mars,Pluto,Saturn,Venus} --> {Mars,Pluto,Venus}>", "{Pluto}", "{Mars,Pluto,Venus}" => "<{Pluto} --> {Mars,Pluto,Venus}>";
                "<{Tweety} --> (|,[with_wings],{Birdie})>", "(|,[with_wings],{Birdie})", "(&,flyer,[yellow])" => "<(|,[with_wings],{Birdie}) --> (&,flyer,[yellow])>";
                "<{Tweety} --> {Birdie}>", "[with_wings]", "{Birdie}" => "<[with_wings] --> {Birdie}>";
                "<{Tweety} --> flyer>", "{Tweety}", "(|,flyer,[yellow])" => "<{Tweety} --> (|,flyer,[yellow])>";
                "<(|,boy,girl) --> (|,girl,youth)>", "girl", "youth" => "<girl --> youth>";
                "<{Tweety} --> flyer>", "{Tweety}", "(|,flyer,[with_wings],{Birdie})" => "<{Tweety} --> (|,flyer,[with_wings],{Birdie})>";
                "<0 --> (/,num,_)>", "num", "(/,num,_)" => "<num --> (/,num,_)>";
                "<{Tweety} --> flyer>", "{Tweety}", "(&,flyer,[with_wings],{Birdie})" => "<{Tweety} --> (&,flyer,[with_wings],{Birdie})>";
                "<(*,0) --> (*,num)>", "(*,0)", "(|,(*,num),(*,(/,num,_)))" => "<(*,0) --> (|,(*,num),(*,(/,num,_)))>";
                "<acid --> (/,reaction,_,base)>", "acid", "(/,neutralization,_,base)" => "<acid --> (/,neutralization,_,base)>";
                "<(&,bird,swimmer) --> (&,animal,swimmer)>", "bird", "animal" => "<bird --> animal>";
                "<(*,0) --> (*,(/,num,_))>", "(*,0)", "(&,(*,num),(*,(/,num,_)))" => "<(*,0) --> (&,(*,num),(*,(/,num,_)))>";
                "<(*,0) --> (*,(/,num,_))>", "(*,0)", "(|,(*,num),(*,(/,num,_)))" => "<(*,0) --> (|,(*,num),(*,(/,num,_)))>";
                "<robin --> (|,bird,swimmer)>", "(&,robin,swan)", "(|,bird,swimmer)" => "<(&,robin,swan) --> (|,bird,swimmer)>";
                "<(/,open,_,lock) --> (/,open,_,{lock1})>", "key", "(/,open,_,{lock1})" => "<key --> (/,open,_,{lock1})>";
                "<(|,robin,tiger) --> animal>", "(|,robin,tiger)", "animal" => "<(|,robin,tiger) --> animal>";
                "<robin --> animal>", "swan", "robin" => "<swan --> robin>";
                "<cat --> CAT>", "cat", "(|,CAT,(/,(/,REPRESENT,_,<(*,CAT,FISH) --> FOOD>),_,eat,fish))" => "<cat --> (|,CAT,(/,(/,REPRESENT,_,<(*,CAT,FISH) --> FOOD>),_,eat,fish))>";
                "<(|,chess,sport) --> competition>", "(|,chess,sport)", "(|,chess,competition)" => "<(|,chess,sport) --> (|,chess,competition)>";
                "<(/,open,_,lock) --> key>", "(/,open,_,lock)", "(|,key,(/,open,_,{lock1}))" => "<(/,open,_,lock) --> (|,key,(/,open,_,{lock1}))>";
                "<sport --> competition>", "(~,chess,sport)", "competition" => "<(~,chess,sport) --> competition>";
                "<{?1} --> swimmer>", "robin", "{?1}" => "<robin --> {?1}>";
                "<robin --> bird>", "swimmer", "robin" => "<swimmer --> robin>";
                "<(/,open,_,lock) --> (/,open,_,{lock1})>", "(/,open,_,{lock1})", "key" => "<(/,open,_,{lock1}) --> key>";
                "<a --> (/,like,b,_)>", "(*,b,a)", "(*,b,(/,like,b,_))" => "<(*,b,a) --> (*,b,(/,like,b,_))>";
                "<{Tweety} --> bird>", "{Tweety}", "(|,bird,flyer)" => "<{Tweety} --> (|,bird,flyer)>";
                "<{Tweety} --> [yellow]>", "{Tweety}", "(|,[yellow],{Birdie})" => "<{Tweety} --> (|,[yellow],{Birdie})>";
                "<(&&,<robin --> [flying]>,<robin --> [with_wings]>) ==> <robin --> [living]>>", "<robin --> [with_wings]>", "<robin --> bird>" => "<<robin --> [with_wings]> ==> <robin --> bird>>";
                "<robin --> animal>", "robin", "swan" => "<robin --> swan>";
                "<(*,(*,(*,0))) --> num>", "(*,(*,(*,(/,num,_))))", "num" => "<(*,(*,(*,(/,num,_)))) --> num>";
                "<swimmer --> robin>", "animal", "swimmer" => "<animal --> swimmer>";
                "<(&,robin,swan) --> (&,bird,swimmer)>", "robin", "bird" => "<robin --> bird>";
                "<swan --> bird>", "gull", "bird" => "<gull --> bird>";
                "<robin --> [with_wings]>", "robin", "flyer" => "<robin --> flyer>";
                "<planetX --> {Pluto,Saturn}>", "{Pluto,Saturn}", "{Mars,Pluto,Venus}" => "<{Pluto,Saturn} --> {Mars,Pluto,Venus}>";
                "<tiger --> robin>", "tiger", "swan" => "<tiger --> swan>";
                "<planetX --> {Pluto,Saturn}>", "{Mars,Pluto,Saturn,Venus}", "{Pluto,Saturn}" => "<{Mars,Pluto,Saturn,Venus} --> {Pluto,Saturn}>";
                "<(/,(*,tim,tom),_,tom) --> (/,uncle,_,tom)>", "(*,tim,tom)", "uncle" => "<(*,tim,tom) --> uncle>";
                "<{Tweety} --> {Birdie}>", "{Birdie}", "[with_wings]" => "<{Birdie} --> [with_wings]>";
                "<(|,acid,(/,neutralization,_,base)) --> (/,reaction,_,base)>", "(/,neutralization,_,base)", "(/,reaction,_,base)" => "<(/,neutralization,_,base) --> (/,reaction,_,base)>";
                "<swan --> swimmer>", "bird", "swimmer" => "<bird --> swimmer>";
                "<swimmer --> bird>", "swimmer", "(|,animal,bird)" => "<swimmer --> (|,animal,bird)>";
                "<(|,robin,swan) --> (|,bird,swimmer)>", "robin", "(|,bird,swimmer)" => "<robin --> (|,bird,swimmer)>";
                "<(|,chess,sport) --> (|,chess,competition)>", "chess", "chess" => None;
                "<(&&,<robin --> [chirping]>,<robin --> [with_wings]>) ==> <robin --> bird>>", "<robin --> [chirping]>", "<robin --> bird>" => "<<robin --> [chirping]> ==> <robin --> bird>>";
                "<(*,(*,0)) --> (*,(*,(/,num,_)))>", "(*,(*,(*,0)))", "(*,(*,(*,(/,num,_))))" => "<(*,(*,(*,0))) --> (*,(*,(*,(/,num,_))))>";
                "<(|,robin,swimmer) --> bird>", "(|,robin,swimmer)", "(&,animal,bird)" => "<(|,robin,swimmer) --> (&,animal,bird)>";
                "<swan --> robin>", "bird", "robin" => "<bird --> robin>";
                "<{key1} --> key>", "{key1}", "(&,key,(/,open,_,{lock1}))" => "<{key1} --> (&,key,(/,open,_,{lock1}))>";
                "<(&&,<robin --> [chirping]>,<robin --> [flying]>) ==> <robin --> bird>>", "<robin --> [flying]>", "<robin --> bird>" => "<<robin --> [flying]> ==> <robin --> bird>>";
                "<{key1} --> key>", "{key1}", "(|,key,(/,open,_,{lock1}))" => "<{key1} --> (|,key,(/,open,_,{lock1}))>";
                "<chess --> competition>", "sport", "chess" => "<sport --> chess>";
                "<bird --> swimmer>", "(|,bird,robin)", "swimmer" => "<(|,bird,robin) --> swimmer>";
                "<{Tweety} --> bird>", "{Birdie}", "bird" => "<{Birdie} --> bird>";
                "<(*,num) <-> (*,(/,num,_))>", "num", "(/,num,_)" => "<num <-> (/,num,_)>";
                "<(*,tim,tom) --> uncle>", "(/,(*,tim,tom),tom,_)", "(/,uncle,tom,_)" => "<(/,(*,tim,tom),tom,_) --> (/,uncle,tom,_)>";
                "<(&&,<robin --> swimmer>,<robin --> [flying]>) ==> <robin --> bird>>", "<robin --> [flying]>", "<robin --> bird>" => "<<robin --> [flying]> ==> <robin --> bird>>";
                "<(~,boy,girl) --> (~,youth,girl)>", "girl", "(~,youth,girl)" => None;
                "<{Birdie} --> [yellow]>", "(|,flyer,{Birdie})", "(|,flyer,[yellow])" => "<(|,flyer,{Birdie}) --> (|,flyer,[yellow])>";
                "<(|,boy,girl) --> (|,girl,youth)>", "(|,boy,girl)", "(|,girl,youth)" => "<(|,boy,girl) --> (|,girl,youth)>";
                "<bird --> swimmer>", "bird", "robin" => "<bird --> robin>";
                "<sport --> competition>", "chess", "sport" => "<chess --> sport>";
                "<(|,robin,swimmer) --> animal>", "robin", "animal" => "<robin --> animal>";
                "<(&,robin,swimmer) --> animal>", "animal", "bird" => "<animal --> bird>";
                "<{Tweety} --> [with_wings]>", "(|,flyer,{Tweety})", "(|,flyer,[with_wings])" => "<(|,flyer,{Tweety}) --> (|,flyer,[with_wings])>";
                "<(~,boy,girl) --> (~,youth,girl)>", "girl", "girl" => None;
                "<{Tweety} --> [with_wings]>", "{Tweety}", "(|,flyer,[with_wings])" => "<{Tweety} --> (|,flyer,[with_wings])>";
                "<{Tweety} --> [with_wings]>", "(&,flyer,{Birdie})", "[with_wings]" => "<(&,flyer,{Birdie}) --> [with_wings]>";
                "<bird --> animal>", "(&,bird,tiger)", "animal" => "<(&,bird,tiger) --> animal>";
                "<base --> (/,reaction,acid,_)>", "base", "(/,neutralization,acid,_)" => "<base --> (/,neutralization,acid,_)>";
                "<bird --> animal>", "robin", "bird" => "<robin --> bird>";
                "<(~,boy,girl) --> [strong]>", "girl", "[strong]" => "<girl --> [strong]>";
                "<robin --> animal>", "(&,robin,tiger)", "animal" => "<(&,robin,tiger) --> animal>";
                "<{Tweety} --> {Birdie}>", "[yellow]", "{Birdie}" => "<[yellow] --> {Birdie}>";
                "<swan --> robin>", "robin", "bird" => "<robin --> bird>";
                "<{Tweety} --> bird>", "{Tweety}", "(&,bird,flyer)" => "<{Tweety} --> (&,bird,flyer)>";
                "<{lock1} --> lock>", "(/,open,{key1},_)", "lock" => "<(/,open,{key1},_) --> lock>";
                "<robin --> [with_wings]>", "robin", "(|,[with_wings],{Birdie})" => "<robin --> (|,[with_wings],{Birdie})>";
                "<{Tweety} --> (&,flyer,{Birdie})>", "{Tweety}", "{Birdie}" => "<{Tweety} --> {Birdie}>";
                "<robin --> (-,mammal,swimmer)>", "robin", "mammal" => "<robin --> mammal>";
                "<Birdie <-> Tweety>", "{Birdie}", "{Tweety}" => "<{Birdie} <-> {Tweety}>";
            }
            ok!()
        }

        #[cfg(TODO)] // TODO: 有待复用
        #[test]
        fn new_sym_statement() -> AResult {
            asserts! {
                // 继承⇒相似
                Term::new_sym_statement(INHERITANCE_RELATION, term!("A"), term!("B"))
                    => term!("<A <-> B>")
                // 蕴含⇒等价
                Term::new_sym_statement(IMPLICATION_RELATION, term!("A"), term!("B"))
                    => term!("<A <=> B>")
            }
            ok!()
        }
    }
}
