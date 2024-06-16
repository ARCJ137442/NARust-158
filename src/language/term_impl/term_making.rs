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
            Self::make_image_ext_arg(components)
        } else if term.instanceof_image_int() {
            Self::make_image_int_arg(components)
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
            IMAGE_EXT_OPERATOR => Self::make_image_ext_arg(argument),
            IMAGE_INT_OPERATOR => Self::make_image_int_arg(argument),
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

    /* SetExt */

    /// 制作一个外延集
    /// * 🚩单个词项⇒视作一元数组构造
    pub fn make_set_ext(t: Term) -> Option<Term> {
        Self::make_set_ext_arg(vec![t])
    }

    /// 制作一个外延集
    /// * 🚩数组⇒统一重排去重⇒构造
    /// * ℹ️相对改版而言，综合「用集合构造」与「用数组构造」
    pub fn make_set_ext_arg(mut argument: Vec<Term>) -> Option<Term> {
        // * 🚩不允许空集
        if argument.is_empty() {
            return None;
        }
        // * 🚩重排去重 | 📌只重排一层：OpenNARS原意如此，并且在外部构建的词项也经过了重排去重
        TermComponents::sort_dedup_term_vec(&mut argument);
        // * 🚩构造
        Some(Term::new_set_ext(argument))
    }

    /* SetInt */

    /// 制作一个内涵集
    /// * 🚩单个词项⇒视作一元数组构造
    pub fn make_set_int(t: Term) -> Option<Term> {
        Self::make_set_int_arg(vec![t])
    }

    /// 制作一个内涵集
    /// * 🚩数组⇒统一重排去重⇒构造
    /// * ℹ️相对改版而言，综合「用集合构造」与「用数组构造」
    pub fn make_set_int_arg(mut argument: Vec<Term>) -> Option<Term> {
        // * 🚩不允许空集
        if argument.is_empty() {
            return None;
        }
        // * 🚩重排去重 | 📌只重排一层：OpenNARS原意如此，并且在外部构建的词项也经过了重排去重
        TermComponents::sort_dedup_term_vec(&mut argument);
        // * 🚩构造
        Some(Term::new_set_int(argument))
    }

    /* IntersectionExt */

    pub fn make_intersection_ext(term1: Term, term2: Term) -> Option<Term> {
        // * 🚩预置「词项列表」与「词项制作」
        let mut terms = vec![];
        let make: fn(Vec<Term>) -> Option<Term>;
        // * 🚩两个内涵集取外延交 ⇒ 外延交=内涵并 ⇒ 取并集
        // * 📄[A,B] & [C,D] = [A,B,C,D]
        if let [Some(s1), Some(s2)] = [
            term1.as_compound_type(SET_INT_OPERATOR),
            term2.as_compound_type(SET_INT_OPERATOR),
        ] {
            // * 🚩s1加入最终词项集
            terms.extend(s1.components.iter().cloned());
            // * 🚩s2加入最终词项集
            terms.extend(s2.components.iter().cloned());
            // * 🚩最终生成内涵集
            make = Self::make_set_int_arg;
        }
        // * 🚩两个外延集取外延交 ⇒ 取交集
        // * 📄{A,B} & {B,C} = {B}
        else if let [Some(s1), Some(s2)] = [
            term1.as_compound_type(SET_EXT_OPERATOR),
            term2.as_compound_type(SET_EXT_OPERATOR),
        ] {
            // * 🚩s1加入最终词项集
            terms.extend(s1.components.iter().cloned());
            // * 🚩加入的词项集和s2取交集
            vec_utils::retain_all(&mut terms, s2.components);
            // * 🚩最终生成外延集
            make = Self::make_set_ext_arg;
        } else {
            // * 🚩均生成外延交 | 注意：在OpenNARS中是传入集合然后重载，此处即改为「直接传递类集合数组」
            make = Self::make_intersection_ext_vec;
            match [
                term1.as_compound_type(INTERSECTION_EXT_OPERATOR),
                term2.as_compound_type(INTERSECTION_EXT_OPERATOR),
            ] {
                // * 🚩左右都是外延交 ⇒ 取交集
                // * 📄(&,P,Q) & (&,R,S) = (&,P,Q,R,S)
                [Some(s1), Some(s2)] => {
                    terms.extend(s1.components.iter().cloned());
                    terms.extend(s2.components.iter().cloned());
                }
                // * 🚩仅左边是外延交 ⇒ 右边加进左边
                // * 📄(&,P,Q) & R = (&,P,Q,R)
                [Some(s1), None] => {
                    terms.extend(s1.components.iter().cloned());
                    terms.push(term2);
                }
                // * 🚩仅右边是外延交 ⇒ 左边加进右边
                // * 📄R & (&,P,Q) = (&,P,Q,R)
                [None, Some(s2)] => {
                    terms.extend(s2.components.iter().cloned());
                    terms.push(term1);
                }
                // * 🚩纯默认 ⇒ 直接添加
                // * 📄P & Q = (&,P,Q)
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
    fn make_intersection_ext_arg(mut argument: Vec<Term>) -> Option<Term> {
        // * 🆕🚩做一个reduce的操作 | 此版本中是从尾到头，总体逻辑仍然一样
        // * ✅↓此处已含有「列表为空⇒返回空」的逻辑
        let mut term = argument.pop()?;
        // * 🚩取出剩下的
        while let Some(t) = argument.pop() {
            // * 🚩尝试做交集：失败⇒返回空
            let new_term = Self::make_intersection_ext(term, t)?;
            // * 🚩更新
            term = new_term;
        }
        // * 🚩返回
        Some(term)
    }

    /// * 🚩只依照集合数量进行化简
    fn make_intersection_ext_vec(mut terms: Vec<Term>) -> Option<Term> {
        match terms.len() {
            // * 🚩空集⇒空
            0 => None,
            // * 🚩单个元素⇒直接取元素
            1 => terms.pop(),
            // * 🚩其它⇒新建词项
            _ => Some(Term::new_intersection_ext(terms)),
        }
    }

    /* IntersectionInt */

    pub fn make_intersection_int(term1: Term, term2: Term) -> Option<Term> {
        // TODO: 或可与「制作外延交」归一化？
        // * 🚩预置「词项列表」与「词项制作」
        let mut terms = vec![];
        let make: fn(Vec<Term>) -> Option<Term>;
        // * 🚩两个外延集取内涵交 ⇒ 内涵交=外延并 ⇒ 取并集
        // * 📄{A,B} | {C,D} = {A,B,C,D}
        if let [Some(s1), Some(s2)] = [
            term1.as_compound_type(SET_EXT_OPERATOR),
            term2.as_compound_type(SET_EXT_OPERATOR),
        ] {
            // * 🚩s1加入最终词项集
            terms.extend(s1.components.iter().cloned());
            // * 🚩s2加入最终词项集
            terms.extend(s2.components.iter().cloned());
            // * 🚩最终生成外延集
            make = Self::make_set_ext_arg;
        }
        // * 🚩两个内涵集取内涵交 ⇒ 取交集
        // * 📄[A,B] | [B,C] = [B]
        else if let [Some(s1), Some(s2)] = [
            term1.as_compound_type(SET_INT_OPERATOR),
            term2.as_compound_type(SET_INT_OPERATOR),
        ] {
            // * 🚩s1加入最终词项集
            terms.extend(s1.components.iter().cloned());
            // * 🚩加入的词项集和s2取交集
            vec_utils::retain_all(&mut terms, s2.components);
            // * 🚩最终生成内涵集
            make = Self::make_set_int_arg;
        } else {
            // * 🚩均生成内涵交
            make = Self::make_intersection_int_vec;
            match [
                term1.as_compound_type(INTERSECTION_INT_OPERATOR),
                term2.as_compound_type(INTERSECTION_INT_OPERATOR),
            ] {
                // * 🚩左右都是内涵交 ⇒ 取交集
                // * 📄(|,P,Q) | (|,R,S) = (|,P,Q,R,S)
                [Some(s1), Some(s2)] => {
                    terms.extend(s1.components.iter().cloned());
                    terms.extend(s2.components.iter().cloned());
                }
                // * 🚩仅左边是内涵交 ⇒ 右边加进左边
                // * 📄(|,P,Q) | R = (|,P,Q,R)
                [Some(s1), None] => {
                    terms.extend(s1.components.iter().cloned());
                    terms.push(term2);
                }
                // * 🚩仅右边是内涵交 ⇒ 左边加进右边
                // * 📄R | (|,P,Q) = (|,P,Q,R)
                [None, Some(s2)] => {
                    terms.extend(s2.components.iter().cloned());
                    terms.push(term1);
                }
                // * 🚩纯默认 ⇒ 直接添加
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
    fn make_intersection_int_arg(mut argument: Vec<Term>) -> Option<Term> {
        // * 🆕🚩做一个reduce的操作 | 此版本中是从尾到头，总体逻辑仍然一样
        // * ✅↓此处已含有「列表为空⇒返回空」的逻辑
        let mut term = argument.pop()?;
        // * 🚩取出剩下的
        while let Some(t) = argument.pop() {
            // * 🚩尝试做交集：失败⇒返回空
            let new_term = Self::make_intersection_int(term, t)?;
            // * 🚩更新
            term = new_term;
        }
        // * 🚩返回
        Some(term)
    }

    /// * 🚩只依照集合数量进行化简
    fn make_intersection_int_vec(mut argument: Vec<Term>) -> Option<Term> {
        match argument.len() {
            // * 🚩空集⇒空
            0 => None,
            // * 🚩单个元素⇒直接取元素
            1 => argument.pop(),
            // * 🚩其它⇒新建词项
            _ => Some(Term::new_intersection_int(argument)),
        }
    }

    /* DifferenceExt */

    pub fn make_difference_ext(left: Term, right: Term) -> Option<Term> {
        // * 🚩自己减自己 ⇒ 空集 ⇒ 空
        if left == right {
            return None;
        }
        match [
            left.as_compound_type(SET_EXT_OPERATOR),
            right.as_compound_type(SET_EXT_OPERATOR),
        ] {
            // * 🚩外延集的差：求差，构造外延集 | {A, B} - {A} = {B}
            [Some(..), Some(..)] => {
                // * 🚩先解包出内部元素（开始丢弃左右所有权）
                let [left, right] = [
                    left.unwrap_compound_components().unwrap(), // ! 先前已假设过复合词项
                    right.unwrap_compound_components().unwrap(), // ! 先前已假设过复合词项
                ];
                // * 🚩left加入最终词项集
                // * 📝to_vec会拷贝元素，故不用之
                let mut terms = left.into();
                // * 🚩加入的词项集和right取差集 // set difference
                vec_utils::remove_all(&mut terms, &right);
                // * 🚩最终生成外延集
                Self::make_set_int_arg(terms)
            }
            // * 🚩否则：直接构造外延差 | A - B = (-,A,B)
            _ => Some(Self::new_diff_ext(left, right)),
        }
    }

    fn make_difference_ext_arg(mut argument: Vec<Term>) -> Option<Term> {
        match argument.len() {
            // * 🚩单个元素：约简为内部元素 | (-,A) = A
            1 => argument.pop(), // special case from CompoundTerm.reduceComponent
            // * 🚩两个元素⇒进一步判断
            2 => {
                let right = argument.pop().unwrap();
                let left = argument.pop().unwrap();
                Self::make_difference_ext(left, right)
            }
            // * 🚩其它⇒空
            _ => None,
        }
    }

    /* DifferenceInt */

    pub fn make_difference_int(left: Term, right: Term) -> Option<Term> {
        // * 🚩自己减自己 ⇒ 空集 ⇒ 空
        if left == right {
            return None;
        }
        match [
            left.as_compound_type(SET_INT_OPERATOR),
            right.as_compound_type(SET_INT_OPERATOR),
        ] {
            // * 🚩内涵集的差：求差，构造内涵集 | [A, B] - [A] = [B]
            [Some(..), Some(..)] => {
                // * 🚩先解包出内部元素（开始丢弃左右所有权）
                let [left, right] = [
                    left.unwrap_compound_components().unwrap(), // ! 先前已假设过复合词项
                    right.unwrap_compound_components().unwrap(), // ! 先前已假设过复合词项
                ];
                // * 🚩left加入最终词项集
                // * 📝to_vec会拷贝元素，故不用之
                let mut terms = left.into();
                // * 🚩加入的词项集和right取差集 // set difference
                vec_utils::remove_all(&mut terms, &right);
                // * 🚩最终生成内涵集
                Self::make_set_int_arg(terms)
            }
            // * 🚩否则：直接构造内涵差 | A - B = (-,A,B)
            _ => Some(Self::new_diff_int(left, right)),
        }
    }

    fn make_difference_int_arg(mut argument: Vec<Term>) -> Option<Term> {
        match argument.len() {
            // * 🚩单个元素：约简为内部元素 | (-,A) = A
            1 => argument.pop(), // special case from CompoundTerm.reduceComponent
            // * 🚩两个元素⇒进一步判断
            2 => {
                let right = argument.pop().unwrap();
                let left = argument.pop().unwrap();
                Self::make_difference_int(left, right)
            }
            // * 🚩其它⇒空
            _ => None,
        }
    }

    /* Product */

    fn make_product_arg(argument: Vec<Term>) -> Option<Term> {
        Some(Self::new_product(argument))
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

    /* ImageExt */

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
    fn make_image_ext_arg(argument: Vec<Term>) -> Option<Term> {
        // * 🚩拒绝元素过少的词项 | 第一个词项需要是「关系」，除此之外必须含有至少一个元素 & 占位符
        if argument.len() < 2 {
            return None;
        }
        // * 🚩因为「词项中自带占位符」所以无需「特别决定索引」
        Self::new_image_ext(argument).ok()
    }

    /// 从一个「乘积」构造外延像
    /// * ⚠️有关「像」的机制跟OpenNARS实现不一致，将作调整
    ///   * 💭但在效果上是可以一致的
    /// * 🚩整体过程：关系词项插入到最前头，然后在指定的占位符处替换
    ///   * 📌应用「惰性复制」思路
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
        // * 🚩关系词项是「乘积」⇒可能可以简化
        if let Some(p2) = relation.as_compound_type(PRODUCT_OPERATOR) {
            // * 🚩对「二元外延像」作特别的「取索引」简化
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
                // TODO: 后续可以通用化？
            }
        }
        // * 🚩通过「前插关系词项」与「占位符挖空」构造外延像
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
        Self::make_image_ext_arg(argument)
    }

    /// 从一个已知的外延像中构造新外延像，并切换占位符的位置
    /// * 🚩关系词项位置不变，后头词项改变位置，原占位符填充词项
    ///
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
        Self::make_image_ext_arg(argument)
    }

    /* ImageInt */

    fn make_image_int_arg(mut argument: Vec<Term>) -> Option<Term> {
        todo!("// TODO: 有待复刻")
    }

    pub fn make_image_int(argument: Vec<Term>, placeholder_index: usize) -> Option<Term> {
        todo!("// TODO: 有待复刻")
    }

    /* Conjunction */

    fn make_conjunction_arg(mut argument: Vec<Term>) -> Option<Term> {
        todo!("// TODO: 有待复刻")
    }

    fn make_disjunction_arg(mut argument: Vec<Term>) -> Option<Term> {
        todo!("// TODO: 有待复刻")
    }

    fn make_negation_arg(mut argument: Vec<Term>) -> Option<Term> {
        todo!("// TODO: 有待复刻")
    }

    /* Statement */

    pub fn make_statement(template: StatementRef, subject: Term, predicate: Term) -> Option<Term> {
        todo!("// TODO: 有待复刻")
    }

    #[cfg(TODO)] // TODO: 有待复用
    /// 📄OpenNARS `Statement.makeSym`
    /// * 🚩通过使用「标识符映射」将「非对称版本」映射到「对称版本」
    /// * ⚠️目前只支持「继承」和「蕴含」，其它均会`panic`
    ///
    /// # 📄OpenNARS
    /// Make a symmetric Statement from given components and temporal information,
    /// called by the rules
    pub fn new_sym_statement(identifier: &str, subject: Term, predicate: Term) -> Self {
        match identifier {
            // 继承⇒相似
            INHERITANCE_RELATION => Term::new_similarity(subject, predicate),
            // 蕴含⇒等价
            IMPLICATION_RELATION => Term::new_equivalence(subject, predicate),
            // 其它⇒panic
            _ => unimplemented!("不支持的标识符：{identifier:?}"),
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
    use crate::{global::tests::AResult, ok, test_compound as compound, test_term as term};
    use nar_dev_utils::macro_once;

    /// 具体的词项构造
    /// * 📄外延集、内涵集……
    mod concrete_type {
        use super::*;

        #[test]
        fn make_image_ext_arg() -> AResult {
            macro_once! {
                // * 🚩模式：词项列表 ⇒ 预期词项
                macro test($($arg_list:tt => $expected:expr;)*) {
                    $(
                        let arg_list: Vec<_> = term!($arg_list).into();
                        let image = Term::make_image_ext_arg(arg_list).expect("解析词项失败！");
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
                "(*,$1,sunglasses)", "own",  1 => "(/,own,$1,_)";
                "(*,bird,plant)",    "?1",   0 => "(/,?1,_,plant)";
                "(*,bird,plant)",    "?1",   1 => "(/,?1,bird,_)";
                "(*,robin,worms)",   "food", 1 => "(/,food,robin,_)";
                "(*,CAT,eat,fish)",  "R",    0 => "(/,R,_,eat,fish)";
                "(*,CAT,eat,fish)",  "R",    1 => "(/,R,CAT,_,fish)";
                "(*,CAT,eat,fish)",  "R",    2 => "(/,R,CAT,eat,_)";
                "(*,b,a)", "(*,b,(/,like,b,_))", 1 => "(/,like,b,_)";
                "(*,a,b)", "(*,(/,like,b,_),b)", 0 => "(/,like,b,_)";
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
                "(/,open,{key1},_)",   "lock",   0 => "(/,open,_,lock)";
                "(/,uncle,_,tom)",     "tim",    1 => "(/,uncle,tim,_)";
                "(/,open,{key1},_)",   "$2",     0 => "(/,open,_,$2)";
                "(/,open,{key1},_)",   "#1",     0 => "(/,open,_,#1)";
                "(/,like,_,a)",        "b",      1 => "(/,like,b,_)";
                "(/,like,b,_)",        "a",      0 => "(/,like,_,a)";
            }
            ok!()
        }
    }

    mod compound {
        use super::*;

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
