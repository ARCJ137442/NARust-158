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
        // * 🆕🚩做一个reduce的操作 | 此版本中是从尾到头，总体逻辑仍然一样
        // * ✅↓此处已含有「列表为空⇒返回空」的逻辑
        let mut term = argument.pop()?;
        // * 🚩取出剩下的
        while let Some(t) = argument.pop() {
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
        Self::make_intersection_vec(terms, Self::new_intersection_ext)
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
        Self::make_intersection_vec(terms, Self::new_intersection_int)
    }

    /* DifferenceExt */

    // TODO: 有待统一逻辑
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

    /* Image */

    fn make_image_arg(
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
        make_image_arg: fn(Vec<Term>) -> Option<Term>,
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
        make_image_arg(argument)
    }

    /// 共用的「从像构造像」逻辑
    /// * 📌从一个已知的外延像中构造新外延像，并切换占位符的位置
    /// * 🚩关系词项位置不变，后头词项改变位置，原占位符填充词项
    fn make_image_from_image(
        old_image: CompoundTermRef,
        component: &Term,
        index: usize,
        make_image_arg: fn(Vec<Term>) -> Option<Term>,
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
        make_image_arg(argument)
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
        Self::make_image_arg(argument, Self::new_image_ext)
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
        Self::make_image_from_product(product, relation, index, Self::make_image_ext_arg)
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
        Self::make_image_from_image(old_image, component, index, Self::make_image_ext_arg)
    }

    /* ImageInt */

    fn make_image_int_arg(argument: Vec<Term>) -> Option<Term> {
        Self::make_image_arg(argument, Self::new_image_int)
    }

    pub fn make_image_int_from_product(
        product: CompoundTermRef,
        relation: &Term,
        index: usize, // * 📝这个指的是「乘积里头挖空」的索引
    ) -> Option<Term> {
        // * 🚩现在统一在一个「『像』构造」逻辑中
        Self::make_image_from_product(product, relation, index, Self::make_image_int_arg)
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
        Self::make_image_from_image(old_image, component, index, Self::make_image_int_arg)
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
                Some(t) => format!("Some({t})"),
                None => "None".to_string(),
            }
        }

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

        /* ImageExt */

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
        fn make_image_int_arg() -> AResult {
            macro_once! {
                // * 🚩模式：词项列表 ⇒ 预期词项
                macro test($($arg_list:tt => $expected:expr;)*) {
                    $(
                        let arg_list: Vec<_> = term!($arg_list).into();
                        let image = Term::make_image_int_arg(arg_list).expect("解析词项失败！");
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
