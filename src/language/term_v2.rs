//! 表征NARust 158所用的「词项」
//! * 📄功能上参照OpenNARS
//! * 🚩实现方式上更Rusty，同时亦有其它妥协/加强
//! * ❓【2024-04-20 22:00:44】「统一结构体+用『可选字段』实现多态」的方法，会导致「性能臃肿」问题
//!   * ❗此举需要提前考虑「所有类型词项的所有功能」，并且要做到最大程度兼容
//!   * 📌即便使用「作为枚举的专用字段」也会因为「要适应某种复合词项类型」而导致让步
//!     * 而这种「只会在某个类型上产生让步」的方法，会导致「本该耦合而未耦合」的情形
//!     * 这种「看似通用，实则仍需『专用情况专用对待』」的方法，不利于后续维护
//!   * ❓【2024-04-20 23:53:15】或许也可行：是否可以`match (self.identifier, &*self.components)`
//! * 🚩【2024-04-20 22:05:09】目前将此方案搁置
//!   * ⇒尝试探索「直接基于『枚举Narsese』」的方法

use crate::io::symbols::*;
use nar_dev_utils::manipulate;

/// 作为「结构」的词项
/// * 🚩更多通过「复合」而非「抽象特征-具体实现」复用代码
///   * 📍【2024-04-20 21:13:20】目前只需实现OpenNARS 1.5.8的东西
///
///  ! ⚠️【2024-04-20 21:47:08】暂不实现「变量 < 原子 < 复合」的逻辑
/// * 🎯OpenNARS中有关「词项顺序」的概念，目的是保证「无序不重复集合」的唯一性
///   * 🚩然而此实现的需求用「派生[`Ord`]」虽然造成逻辑不同，但可以满足需求
///   * 📌核心逻辑：实现需求就行，没必要（也很难）全盘照搬
///
/// # 📄OpenNARS
///
/// Term is the basic component of Narsese, and the object of processing in NARS.
/// A Term may have an associated Concept containing relations with other Terms.
/// It is not linked in the Term, because a Concept may be forgot while the Term exists. Multiple objects may represent the same Term.
///
/// ## 作为特征的「实现」
///
/// ### Cloneable => [`Clone`]
///
/// Make a new Term with the same name.
///
/// ### equals => [`Eq`]
///
/// Equal terms have identical name, though not necessarily the same reference.
///
/// ### hashCode => [`Hash`]
///
/// Produce a hash code for the term
///
/// ### compareTo => [`Ord`]
///
/// Orders among terms: variable < atomic < compound
///
/// ### toString => [`Display`]
///
/// The same as getName by default, used in display only.
///
/// @return The name of the term as a String
#[derive(Debug, Clone, Hash, Eq, PartialEq, Ord, PartialOrd)]
pub struct Term {
    /// 标识符
    /// * 🎯决定词项的「类型」
    /// * 🚩使用不同词项类型独有的「标识符」
    ///   * 📄原子词项⇒原子词项前缀
    ///   * 📄复合词项⇒复合词项连接词
    ///   * 📄陈述⇒陈述系词
    /// * ❌【2024-04-21 00:57:39】不能使用「静态字串」固定
    ///   * ⚠️需要针对「用户输入」作一定妥协
    ///     * 此刻通过「词法折叠」等途径获得的「词项」就不一定是「静态引用」了
    ///   * 📌即便标识符的类型尽可能「固定」（就那么几种）
    identifier: String,

    /// 组分
    /// * 🎯表示「词项包含词项」的功能
    /// * 🚩通过单一的「复合组分」实现「组合」功能
    /// * 🚩此处加上[`Box`]，便不会造成「循环包含」
    components: Box<TermComponents>,
}

/// 复合词项组分
#[derive(Debug, Clone, Hash, Eq, PartialEq, Ord, PartialOrd)]
pub enum TermComponents {
    /// 不包含任何组分
    /// * 📄占位符
    Empty,

    /// 仅包含一个字符串作为「名称」
    /// * 📄词语，变量
    Named(String),

    /// 单一组分
    /// * 📄否定
    Unary(Term),

    /// 双重组分（有序）
    /// * 📄外延差、内涵差
    /// * 📄继承、蕴含
    /// * 🚩通过「构造时自动去重并排序」实现「集合无序性」
    ///   * 📄相似、等价
    Binary(Term, Term),

    /// 多重组分
    /// * 📄乘积
    /// * 🚩通过「构造时自动去重并排序」实现「集合无序性」
    ///   * 📄外延集、内涵集
    ///   * 📄外延交、内涵交
    ///   * 📄合取、析取
    Multi(Vec<Term>),

    /// 多重组分（有序）+索引
    /// * 📄外延像、内涵像
    /// * ❓【2024-04-20 21:57:35】日后需要通过「像」使用时，会造成「像-MultiIndexed」绑定
    ///   * ⚡那时候若使用「断言」是否会导致不稳定
    ///   * ❓若不使用「断言」而是静默失败，是否会增加排查难度
    ///   * ❓若不使用「断言」而是发出警告，那是否会导致性能问题
    /// * 🚩可行的解决方案：`match (self.identifier, self.components) { ('/', MultiIndexed(i, v))}`
    MultiIndexed(usize, Vec<Term>),
}

/// 实现 / 构造
mod construct {
    use super::*;

    impl Term {
        /// 构造函数
        /// * ⚠️有限性：仅限在「内部」使用，不希望外部以此构造出「不符范围」的词项
        pub(super) fn new(identifier: impl Into<String>, components: TermComponents) -> Self {
            Self {
                identifier: identifier.into(),
                components: Box::new(components),
            }
        }

        // 原子词项 //

        /// NAL-1 / 词语
        pub fn new_word(name: impl Into<String>) -> Self {
            Self::new(WORD, TermComponents::Named(name.into()))
        }

        /// NAL-4 / 占位符
        /// * 📌【2024-04-21 00:36:27】需要一个「占位符」词项，以便和「词法Narsese」打交道
        /// * 🚩仅使用「占位符标识符+空组分」表示
        pub fn new_placeholder() -> Self {
            Self::new(PLACEHOLDER, TermComponents::Empty)
        }

        /// NAL-6 / 自变量
        pub fn new_var_i(name: impl Into<String>) -> Self {
            Self::new(VAR_INDEPENDENT, TermComponents::Named(name.into()))
        }

        /// NAL-6 / 因变量
        pub fn new_var_d(name: impl Into<String>) -> Self {
            Self::new(VAR_DEPENDENT, TermComponents::Named(name.into()))
        }

        /// NAL-6 / 查询变量
        pub fn new_var_q(name: impl Into<String>) -> Self {
            Self::new(VAR_QUERY, TermComponents::Named(name.into()))
        }

        // 复合词项 //

        /// NAL-3 / 外延集
        /// * 🚩【2024-04-21 13:39:28】使用统一的「无序不重复集合」构造组分
        pub fn new_set_ext(terms: impl Into<Vec<Term>>) -> Self {
            Self::new(
                SET_EXT_OPERATOR,
                TermComponents::new_multi_set(terms.into()),
            )
        }

        /// NAL-3 / 内涵集
        /// * 🚩【2024-04-21 13:39:28】使用统一的「无序不重复集合」构造组分
        pub fn new_set_int(terms: impl Into<Vec<Term>>) -> Self {
            Self::new(
                SET_INT_OPERATOR,
                TermComponents::new_multi_set(terms.into()),
            )
        }

        /// NAL-3 / 外延交
        /// * 🚩【2024-04-21 13:39:28】使用统一的「无序不重复集合」构造组分
        pub fn new_intersect_ext(terms: impl Into<Vec<Term>>) -> Self {
            Self::new(
                INTERSECTION_EXT_OPERATOR,
                TermComponents::new_multi_set(terms.into()),
            )
        }

        /// NAL-3 / 内涵交
        /// * 🚩【2024-04-21 13:39:28】使用统一的「无序不重复集合」构造组分
        pub fn new_intersect_int(terms: impl Into<Vec<Term>>) -> Self {
            Self::new(
                INTERSECTION_INT_OPERATOR,
                TermComponents::new_multi_set(terms.into()),
            )
        }

        /// NAL-3 / 外延差
        pub fn new_diff_ext(term1: Term, term2: Term) -> Self {
            Self::new(
                DIFFERENCE_EXT_OPERATOR,
                TermComponents::Binary(term1, term2),
            )
        }

        /// NAL-3 / 内涵差
        pub fn new_diff_int(term1: Term, term2: Term) -> Self {
            Self::new(
                DIFFERENCE_INT_OPERATOR,
                TermComponents::Binary(term1, term2),
            )
        }

        /// NAL-4 / 乘积
        pub fn new_product(terms: impl Into<Vec<Term>>) -> Self {
            Self::new(PRODUCT_OPERATOR, TermComponents::Multi(terms.into()))
        }

        /// NAL-4 / 外延像
        pub fn new_image_ext(i_placeholder: usize, terms: impl Into<Vec<Term>>) -> Self {
            Self::new(
                IMAGE_EXT_OPERATOR,
                TermComponents::MultiIndexed(i_placeholder, terms.into()),
            )
        }

        /// NAL-4 / 内涵像
        pub fn new_image_int(i_placeholder: usize, terms: impl Into<Vec<Term>>) -> Self {
            Self::new(
                IMAGE_INT_OPERATOR,
                TermComponents::MultiIndexed(i_placeholder, terms.into()),
            )
        }

        /// NAL-5 / 合取
        /// * 🚩【2024-04-21 13:39:28】使用统一的「无序不重复集合」构造组分
        pub fn new_conjunction(terms: impl Into<Vec<Term>>) -> Self {
            Self::new(
                CONJUNCTION_OPERATOR,
                TermComponents::new_multi_set(terms.into()),
            )
        }

        /// NAL-5 / 析取
        /// * 🚩【2024-04-21 13:39:28】使用统一的「无序不重复集合」构造组分
        pub fn new_disjunction(terms: impl Into<Vec<Term>>) -> Self {
            Self::new(
                DISJUNCTION_OPERATOR,
                TermComponents::new_multi_set(terms.into()),
            )
        }

        /// NAL-5 / 否定
        pub fn new_negation(term: Term) -> Self {
            Self::new(NEGATION_OPERATOR, TermComponents::Unary(term))
        }

        // 陈述 //

        /// NAL-1 / 继承
        pub fn new_inheritance(subject: Term, predicate: Term) -> Self {
            Self::new(
                INHERITANCE_RELATION,
                TermComponents::Binary(subject, predicate),
            )
        }

        /// NAL-3 / 相似
        pub fn new_similarity(subject: Term, predicate: Term) -> Self {
            Self::new(
                SIMILARITY_RELATION,
                TermComponents::new_binary_unordered(subject, predicate),
            )
        }

        /// NAL-5 / 蕴含
        pub fn new_implication(subject: Term, predicate: Term) -> Self {
            Self::new(
                IMPLICATION_RELATION,
                TermComponents::Binary(subject, predicate),
            )
        }

        /// NAL-5 / 等价
        pub fn new_equivalence(subject: Term, predicate: Term) -> Self {
            Self::new(
                EQUIVALENCE_RELATION,
                TermComponents::new_binary_unordered(subject, predicate),
            )
        }
    }

    impl TermComponents {
        /// 多元无序不重复组分
        /// * 🎯用于【无序不重复】的集合类组分
        /// * 📄外延集、内涵集
        /// * 📄外延交、内涵交
        pub fn new_multi_set(terms: Vec<Term>) -> Self {
            Self::Multi(manipulate!(
                terms
              => .sort() // 先排序
              => .dedup() // 再去重 | 📝`dedup`即`delete duplicated`，去除连续的重复元素
            ))
        }

        /// 二元无序组分
        /// * 🎯用于【双元素对称性】复合词项
        /// * ⚠️无法去重：元素数量固定为`2`
        /// * 📄相似、等价
        /// * 🚩使用「临时数组切片」实现（较为简洁）
        pub fn new_binary_unordered(term1: Term, term2: Term) -> Self {
            let [term1, term2] = manipulate!(
                [term1, term2]
              => .sort()
            );
            // 构造
            TermComponents::Binary(term1, term2)
        }
    }
}

/// 实现 / 内建
/// * 🎯非OpenNARS所定义之「属性」「方法」
///   * 📌至少并非OpenNARS原先所定义的
mod property {
    use super::*;

    /// 内建属性
    impl Term {
        /// 只读的「标识符」属性
        pub fn identifier(&self) -> &str {
            &self.identifier
        }

        /// 只读的「组分」属性
        pub fn components(&self) -> &TermComponents {
            &self.components
        }

        /// 判断其是否为「占位符」
        /// * 🎯【2024-04-21 01:04:17】在「词法折叠」中首次使用
        pub fn is_placeholder(&self) -> bool {
            self.identifier == PLACEHOLDER
        }

        /// 快捷获取「标识符-组分」二元组
        /// * 🎯用于很多地方的「类型匹配」
        pub fn id_comp(&self) -> (&str, &TermComponents) {
            (&self.identifier, &*self.components)
        }

        /// 快捷获取「标识符-组分」二元组，并提供可变机会
        /// * 🚩【2024-04-21 00:59:20】现在正常返回其两重可变引用
        /// * 📝【2024-04-21 00:58:58】当「标识符」为「静态字串」时，不能对其内部的`&str`属性进行修改
        ///   * 📌使用`&mut &str`会遇到生命周期问题
        ///   * 📌实际上「修改类型」本身亦不常用
        pub fn id_comp_mut(&mut self) -> (&mut str, &mut TermComponents) {
            (&mut self.identifier, &mut *self.components)
        }

        /// 用于判断是否为「变量词项」
        /// * 📄OpenNARS `instanceof Variable` 逻辑
        pub fn instanceof_variable(&self) -> bool {
            matches!(
                self.identifier.as_str(),
                VAR_INDEPENDENT | VAR_DEPENDENT | VAR_QUERY
            )
        }
    }

    impl TermComponents {
        /// 获取「组分」的大小
        /// * ⚠️对于「带索引序列」不包括「索引」
        ///   * 📄对「像」不包括「像占位符」
        pub fn len(&self) -> usize {
            use TermComponents::*;
            match self {
                // 无组分
                Empty | Named(..) => 0,
                // 固定数目
                Unary(..) => 1,
                Binary(..) => 2,
                // 不定数目
                Multi(terms) | MultiIndexed(_, terms) => terms.len(),
            }
        }

        /// 获取「组分是否为空」
        /// * 🎯自clippy提示而设
        pub fn is_empty(&self) -> bool {
            use TermComponents::*;
            match self {
                // 一定空
                Empty | Named(..) => true,
                // 一定非空
                Unary(..) | Binary(..) => false,
                // 可能空
                Multi(terms) | MultiIndexed(_, terms) => terms.is_empty(),
            }
        }

        /// 获取指定位置的组分（不一定有）
        /// * ⚠️对于「带索引序列」不受「索引」影响
        ///   * 📄对「像」不受「像占位符」影响
        pub fn get(&self, index: usize) -> Option<&Term> {
            use TermComponents::*;
            match (self, index) {
                // 无组分
                (Empty | Named(..), _) => None,
                // 固定数目 @ 固定索引
                (Unary(term), 0) | (Binary(term, _), 0) | (Binary(_, term), 1) => Some(term),
                // 不定数目
                (Multi(terms) | MultiIndexed(_, terms), _) => terms.get(index),
                // 其它情况⇒无
                _ => None,
            }
        }

        /// 获取指定位置的组分（不检查，直接返回元素）
        /// * ⚠️对于「带索引序列」不受「索引」影响
        ///   * 📄对「像」不受「像占位符」影响
        ///
        /// # Safety
        ///
        /// ⚠️只有在「确保索引不会越界」才不会引发panic和未定义行为（UB）
        pub unsafe fn get_unchecked(&self, index: usize) -> &Term {
            use TermComponents::*;
            match (self, index) {
                // 固定数目
                (Unary(term), 0) | (Binary(term, _), 0) | (Binary(_, term), 1) => term,
                // 不定数目
                (Multi(terms) | MultiIndexed(_, terms), _) => terms.get_unchecked(index),
                // 其它情况⇒panic
                _ => panic!("尝试在非法位置 {index} 获取词项：{self:?}"),
            }
        }

        /// 获取其中「所有元素」的迭代器
        /// * 🚩返回一个迭代器，迭代其中所有「元素」
        /// * ⚠️并非「深迭代」：仅迭代自身的下一级词项，不会递归深入
        pub fn iter(&self) -> impl Iterator<Item = &Term> {
            use TermComponents::*;
            // * 📝必须添加类型注释，以便统一不同类型的`Box`，进而统一「迭代器」类型
            let b: Box<dyn Iterator<Item = &Term>> = match self {
                // 一定空
                Empty | Named(..) => Box::new(None.into_iter()),
                // 一定非空
                Unary(term) => Box::new([term].into_iter()),
                Binary(term1, term2) => Box::new([term1, term2].into_iter()),
                // 可能空
                Multi(terms) | MultiIndexed(_, terms) => Box::new(terms.iter()),
            };
            b
        }

        /// 尝试向其中添加元素
        /// * 🚩始终作为其内的「组分」添加，没有「同类⇒组分合并」的逻辑
        /// * 🚩返回「是否添加成功」
        /// * ⚠️不涉及「记忆区」有关`make`的「词项缓存机制」
        pub fn add(&mut self, term: Term) -> bool {
            use TermComponents::*;
            match self {
                // 固定数目的词项⇒必然添加失败
                Empty | Named(..) | Unary(..) | Binary(..) => false,
                // 不定数目⇒添加
                Multi(terms) | MultiIndexed(_, terms) => {
                    terms.push(term);
                    true
                }
            }
        }

        /// 尝试向其中删除元素
        /// * 🚩始终作为其内的「组分」删除，没有「同类⇒删除其中所有组分」的逻辑
        /// * 🚩返回「是否删除成功」
        /// * ⚠️不涉及「记忆区」有关`make`的「词项缓存机制」
        pub fn remove(&mut self, term: &Term) -> bool {
            use TermComponents::*;
            match self {
                // 固定数目的词项⇒必然添加失败
                Empty | Named(..) | Unary(..) | Binary(..) => false,
                // 不定数目⇒尝试移除
                Multi(terms) | MultiIndexed(_, terms) => match terms.iter().position(|t| t == term)
                {
                    // 找到⇒移除
                    Some(index) => {
                        terms.remove(index);
                        true
                    }
                    // 未找到⇒返回false
                    None => false,
                },
            }
        }

        /// 尝试向其中替换元素
        /// * 🚩始终作为其内的「组分」替换
        /// * 🚩返回「是否替换成功」
        /// * ⚠️不涉及「记忆区」有关`make`的「词项缓存机制」
        pub fn replace(&mut self, index: usize, new: Term) -> bool {
            use TermComponents::*;
            match (self, index) {
                // 无组分
                (Empty | Named(..), _) => false,
                // 固定数目 @ 固定索引
                (Unary(term), 0) | (Binary(term, _), 0) | (Binary(_, term), 1) => {
                    *term = new;
                    true
                }
                // 不定数目 & 长度保证
                (Multi(terms) | MultiIndexed(_, terms), _) if index < terms.len() => {
                    terms[index] = new;
                    true
                }
                // 其它情况⇒无
                _ => false,
            }
        }

        /// （作为无序不重复集合）重新排序
        /// * 🎯用作「集合中替换元素后，重新排序（并去重）」
        ///   * ⚠️不会在「固定数目词项」中去重
        ///   * 📄NAL-6「变量替换」
        pub fn reorder_unordered(&mut self) {
            use TermComponents::*;
            match self {
                // 空 | 单个
                Empty | Named(..) | Unary(..) => {}
                // 二元 ⇒ 尝试交换 | ⚠️无法去重
                Binary(term1, term2) => {
                    if term1 > term2 {
                        std::mem::swap(term1, term2);
                    }
                }
                // 不定数目
                Multi(terms) | MultiIndexed(_, terms) => {
                    terms.sort_unstable();
                    terms.dedup();
                }
            }
        }
    }
}

/// 与其它类型相互转换
/// * 🎯转换为「词法Narsese」以便「获取名称」
mod conversion {
    use super::*;
    use anyhow::{anyhow, Result};
    use narsese::{
        conversion::{
            inter_type::lexical_fold::TryFoldInto,
            string::impl_lexical::format_instances::FORMAT_ASCII,
        },
        lexical::Term as TermLexical,
    };
    use std::str::FromStr;

    /// 词项⇒字符串
    /// * 🎯用于更好地打印「词项」名称
    impl Term {
        pub fn format_name(&self) -> String {
            let id = &self.identifier;
            match &*self.components {
                // 空组分
                TermComponents::Empty => id.clone(),
                // 名称 | 原子词项
                TermComponents::Named(name) => id.clone() + name,
                // 一元
                TermComponents::Unary(term) => format!("({id} {})", term.format_name()),
                // 二元
                TermComponents::Binary(term1, term2) => {
                    format!("({} {id} {})", term1.format_name(), term2.format_name())
                }
                // 多元
                TermComponents::Multi(terms) => {
                    let mut s = id.to_string() + "(";
                    let mut terms = terms.iter();
                    if let Some(t) = terms.next() {
                        s += &t.format_name();
                    }
                    for t in terms {
                        s += " ";
                        s += &t.format_name();
                    }
                    s + ")"
                }
                // 多元+索引
                TermComponents::MultiIndexed(index, terms) => {
                    let mut s = id.to_string() + "(";
                    for (i, t) in terms.iter().enumerate() {
                        if i == *index {
                            if i > 0 {
                                s += " ";
                            }
                            s += PLACEHOLDER;
                        }
                        if i > 0 {
                            s += " ";
                        }
                        s += &t.format_name();
                    }
                    s + ")"
                }
            }
        }
    }

    /// 词项⇒词法Narsese
    impl From<&Term> for TermLexical {
        fn from(value: &Term) -> Self {
            use TermComponents::*;
            let (id, comp) = value.id_comp();
            match (id, comp) {
                // 专用 / 集合词项 | 默认已排序
                (SET_EXT_OPERATOR | SET_INT_OPERATOR, Multi(v)) => {
                    let v = v.iter().map(TermLexical::from).collect::<Vec<_>>();
                    Self::new_compound(id, v)
                }
                // 专用 / 陈述
                (
                    INHERITANCE_RELATION | SIMILARITY_RELATION | IMPLICATION_RELATION
                    | EQUIVALENCE_RELATION,
                    Binary(subj, pred),
                ) => Self::new_statement(id, subj.into(), pred.into()),
                // 通用 / 空：仅前缀
                (_, Empty) => Self::new_atom(id, ""),
                // 通用 / 具名：前缀+词项名
                (_, Named(name)) => Self::new_atom(id, name),
                // 通用 / 一元
                (_, Unary(term)) => Self::new_compound(id, vec![term.into()]),
                // 通用 / 二元
                (_, Binary(subj, pred)) => Self::new_compound(id, vec![subj.into(), pred.into()]),
                // 多元
                (_, Multi(terms)) => {
                    Self::new_compound(id, terms.iter().map(TermLexical::from).collect())
                }
                // 通用 / 带索引
                (_, MultiIndexed(i, v)) => {
                    // 逐个转换组分
                    let mut v = v.iter().map(TermLexical::from).collect::<Vec<_>>();
                    // 创建并插入「占位符」
                    let placeholder = Term::new_placeholder();
                    let placeholder = (&placeholder).into();
                    v.insert(*i, placeholder);
                    // 构造 & 返回
                    Self::new_compound(id, v)
                }
            }
        }
    }

    /// 词法折叠 / 获取「标识符」
    /// * 🎯从「词法Narsese」获取「标识符」，以便后续根据「标识符」分发逻辑
    /// * 🚩对「集合」词项：将左右括弧直接拼接，作为新的、统一的「标识符」
    fn get_identifier(term: &TermLexical) -> String {
        match term {
            TermLexical::Atom { prefix, .. } => prefix.clone(),
            TermLexical::Compound { connecter, .. } => connecter.clone(),
            TermLexical::Set {
                left_bracket,
                right_bracket,
                ..
            } => left_bracket.to_string() + right_bracket,
            TermLexical::Statement { copula, .. } => copula.clone(),
        }
    }

    /// 词法折叠 / 从「数组」中转换
    /// * 🎯将「词法Narsese词项数组」转换为「内部词项数组」
    /// * 📌在「无法同时`map`与`?`」时独立成函数
    #[inline]
    fn fold_lexical_terms(terms: Vec<TermLexical>) -> Result<Vec<Term>> {
        let mut v = vec![];
        for term in terms {
            v.push(term.try_into()?);
        }
        Ok(v)
    }

    /// 词法折叠 / 从「数组」中转换成「像」
    /// * 🎯将「词法Narsese词项数组」转换为「像」所需的「带索引词项数组」
    #[inline]
    fn fold_lexical_terms_as_image(terms: Vec<TermLexical>) -> Result<(usize, Vec<Term>)> {
        // 构造「组分」
        let mut v = vec![];
        let mut placeholder_index = 0;
        for (i, term) in terms.into_iter().enumerate() {
            let term: Term = term.try_into()?;
            // 识别「占位符位置」
            // 🆕【2024-04-21 01:12:50】不同于OpenNARS：只会留下（且位置取决于）最后一个占位符
            // 📄OpenNARS在「没找到占位符」时，会将第一个元素作为占位符，然后把「占位符索引」固定为`1`
            match term.is_placeholder() {
                true => placeholder_index = i,
                false => v.push(term),
            }
        }
        Ok((placeholder_index, v))
    }

    /// 词法折叠
    impl TryFoldInto<'_, Term, anyhow::Error> for TermLexical {
        type Folder = ();

        /// 💭【2024-04-21 14:44:15】目前此中方法「相较保守」
        /// * 📌与词法Narsese严格对应（ASCII）
        /// * ✅基本保证「解析结果均保证『合法』」
        fn try_fold_into(self, _: &'_ Self::Folder) -> Result<Term> {
            let identifier = get_identifier(&self);
            let self_str = FORMAT_ASCII.format(&self);
            // 在有限的标识符范围内匹配
            use TermLexical::*;
            let term = match (identifier.as_str(), self) {
                // 原子词项 | ⚠️虽然「单独的占位符」在OpenNARS中不合法，但在解析「像」时需要用到 //
                (WORD, Atom { name, .. }) => Term::new_word(name),
                (PLACEHOLDER, Atom { .. }) => Term::new_placeholder(),
                (VAR_INDEPENDENT, Atom { name, .. }) => Term::new_var_i(name),
                (VAR_DEPENDENT, Atom { name, .. }) => Term::new_var_d(name),
                (VAR_QUERY, Atom { name, .. }) => Term::new_var_q(name),
                // 复合词项 //
                (SET_EXT_OPERATOR, Set { terms, .. }) => {
                    Term::new_set_ext(fold_lexical_terms(terms)?)
                }
                (SET_INT_OPERATOR, Set { terms, .. }) => {
                    Term::new_set_int(fold_lexical_terms(terms)?)
                }
                (INTERSECTION_EXT_OPERATOR, Compound { terms, .. }) => {
                    Term::new_intersect_ext(fold_lexical_terms(terms)?)
                }
                (INTERSECTION_INT_OPERATOR, Compound { terms, .. }) => {
                    Term::new_intersect_int(fold_lexical_terms(terms)?)
                }
                (DIFFERENCE_EXT_OPERATOR, Compound { terms, .. }) if terms.len() == 2 => {
                    let mut iter = terms.into_iter();
                    let term1 = iter.next().unwrap().try_into()?;
                    let term2 = iter.next().unwrap().try_into()?;
                    Term::new_diff_ext(term1, term2)
                }
                (DIFFERENCE_INT_OPERATOR, Compound { terms, .. }) if terms.len() == 2 => {
                    let mut iter = terms.into_iter();
                    let term1 = iter.next().unwrap().try_into()?;
                    let term2 = iter.next().unwrap().try_into()?;
                    Term::new_diff_int(term1, term2)
                }
                (PRODUCT_OPERATOR, Compound { terms, .. }) => {
                    Term::new_product(fold_lexical_terms(terms)?)
                }
                (IMAGE_EXT_OPERATOR, Compound { terms, .. }) => {
                    let (i, terms) = fold_lexical_terms_as_image(terms)?;
                    Term::new_image_ext(i, terms)
                }
                (IMAGE_INT_OPERATOR, Compound { terms, .. }) => {
                    let (i, terms) = fold_lexical_terms_as_image(terms)?;
                    Term::new_image_int(i, terms)
                }
                (CONJUNCTION_OPERATOR, Compound { terms, .. }) => {
                    Term::new_conjunction(fold_lexical_terms(terms)?)
                }
                (DISJUNCTION_OPERATOR, Compound { terms, .. }) => {
                    Term::new_disjunction(fold_lexical_terms(terms)?)
                }
                (NEGATION_OPERATOR, Compound { terms, .. }) if terms.len() == 1 => {
                    Term::new_negation(terms.into_iter().next().unwrap().try_into()?)
                }
                // 陈述
                (
                    INHERITANCE_RELATION,
                    Statement {
                        subject, predicate, ..
                    },
                ) => Term::new_inheritance(
                    subject.try_fold_into(&())?,
                    predicate.try_fold_into(&())?,
                ),
                (
                    SIMILARITY_RELATION,
                    Statement {
                        subject, predicate, ..
                    },
                ) => {
                    Term::new_similarity(subject.try_fold_into(&())?, predicate.try_fold_into(&())?)
                }
                (
                    IMPLICATION_RELATION,
                    Statement {
                        subject, predicate, ..
                    },
                ) => Term::new_implication(
                    subject.try_fold_into(&())?,
                    predicate.try_fold_into(&())?,
                ),
                (
                    EQUIVALENCE_RELATION,
                    Statement {
                        subject, predicate, ..
                    },
                ) => Term::new_equivalence(
                    subject.try_fold_into(&())?,
                    predicate.try_fold_into(&())?,
                ),
                // 其它情况⇒不合法
                _ => return Err(anyhow!("非法词项：{self_str:?}")),
            };
            Ok(term)
        }
        /*
        /// 💭【2024-04-21 13:40:40】目前这种方法还是「过于粗放」
        ///   * ⚠️容许系统内没有的词项类型
        ///   * ⚠️容许【即便标识符在定义内，但『组分』类型不同】的情况
        fn try_fold_into(self, _: &'_ Self::Folder) -> Result<Term> {
            let identifier = get_identifier(&self);
            use TermLexical::*;
            let term = match (identifier.as_str(), self) {
                // 专用 / 占位符
                (PLACEHOLDER, _) => Term::new_placeholder(),
                // 专用 / 一元复合词项
                (NEGATION_OPERATOR, Compound { mut terms, .. }) => {
                    // 仅在长度为1时返回成功
                    if terms.len() == 1 {
                        // ! ⚠️若使用`get`会导致「重复引用」
                        let term = unsafe { terms.pop().unwrap_unchecked().try_fold_into(&())? };
                        Term::new_negation(term)
                    } else {
                        return Err(anyhow!("非法的一元复合词项组分：{terms:?}"));
                    }
                }
                // 专用 / 二元复合词项（有序）
                (DIFFERENCE_EXT_OPERATOR | DIFFERENCE_INT_OPERATOR, Compound { mut terms, .. }) => {
                    // 仅在长度为2时返回成功
                    if terms.len() == 2 {
                        // ! ⚠️若使用`get`会导致「重复引用」
                        let term2 = unsafe { terms.pop().unwrap_unchecked().try_fold_into(&())? };
                        let term1 = unsafe { terms.pop().unwrap_unchecked().try_fold_into(&())? };
                        Term::new(identifier, TermComponents::Binary(term1, term2))
                    } else {
                        return Err(anyhow!("非法的二元复合词项组分：{terms:?}"));
                    }
                }
                // 专用 / 无序陈述
                (
                    SIMILARITY_RELATION | EQUIVALENCE_RELATION,
                    Statement {
                        subject, predicate, ..
                    },
                ) => Term::new(
                    identifier,
                    TermComponents::new_binary_unordered(
                        subject.try_fold_into(&())?,
                        predicate.try_fold_into(&())?,
                    ),
                ),
                // 专用 / 无序复合词项 | 不含「词项集」（在「集合词项」中）
                (
                    INTERSECTION_EXT_OPERATOR
                    | INTERSECTION_INT_OPERATOR
                    | CONJUNCTION_OPERATOR
                    | DISJUNCTION_OPERATOR,
                    Compound { terms, .. },
                ) => Term::new(
                    identifier,
                    // 视作「多元集合」：排序 & 去重
                    TermComponents::new_multi_set(vec_from_lexical_terms(terms)?),
                ),
                // 专用 / 像
                (IMAGE_EXT_OPERATOR | IMAGE_INT_OPERATOR, Compound { terms, .. }) => {
                    // 构造「组分」
                    let mut v = vec![];
                    let mut placeholder_index = 0;
                    for (i, term) in terms.into_iter().enumerate() {
                        let term: Term = term.try_fold_into(&())?;
                        // 识别「占位符位置」
                        // 🆕【2024-04-21 01:12:50】不同于OpenNARS：只会留下（且位置取决于）最后一个占位符
                        // 📄OpenNARS在「没找到占位符」时，会将第一个元素作为占位符，然后把「占位符索引」固定为`1`
                        match term.is_placeholder() {
                            true => placeholder_index = i,
                            false => v.push(term),
                        }
                    }
                    // 构造 & 返回
                    Term::new(
                        identifier,
                        TermComponents::MultiIndexed(placeholder_index, v),
                    )
                }
                // 通用 / 原子词项
                // * 📄词语
                // * 📄变量
                (_, Atom { name, .. }) => Term::new(identifier, TermComponents::Named(name)),
                // 通用 / 复合词项 | 默认视作有序
                // * 📄乘积
                (_, Compound { terms, .. }) => Term::new(
                    identifier,
                    TermComponents::Multi(vec_from_lexical_terms(terms)?),
                ),
                // 通用 / 集合词项 | 默认视作无序
                // * 📄外延集、内涵集
                (_, Set { terms, .. }) => Term::new(
                    identifier,
                    // 视作「多元集合」：排序 & 去重
                    TermComponents::new_multi_set(vec_from_lexical_terms(terms)?),
                ),
                // 通用 / 陈述 | 默认视作有序
                // * 📄继承、蕴含
                (
                    _,
                    Statement {
                        subject, predicate, ..
                    },
                ) => Term::new(
                    identifier,
                    TermComponents::Binary(
                        subject.try_fold_into(&())?,
                        predicate.try_fold_into(&())?,
                    ),
                ),
                // // 其它⇒返回错误
                // ! 🚩【2024-04-21 01:38:15】已穷尽
                // _ => return Err(anyhow!("未知词项标识符：{identifier:?}")),
            };
            Ok(term)
        } */
    }

    /// 基于「词法折叠」实现[`TryFrom`]
    impl TryFrom<TermLexical> for Term {
        type Error = anyhow::Error;

        #[inline(always)]
        fn try_from(value: TermLexical) -> Result<Self, Self::Error> {
            value.try_fold_into(&())
        }
    }

    /// 字符串解析路线：词法解析 ⇒ 词法折叠
    /// * 🎯同时兼容[`str::parse`]与[`str::try_into`]
    impl TryFrom<&str> for Term {
        type Error = anyhow::Error;

        fn try_from(s: &str) -> Result<Self, Self::Error> {
            // 词法解析
            let lexical = FORMAT_ASCII.parse(s)?;
            // 词法转换 | ⚠️对「语句」「任务」报错
            let term = lexical.try_into_term()?;
            // 词法折叠
            let term = term.try_into()?;
            // 返回
            Ok(term)
        }
    }

    ///  字符串解析
    /// * 🎯同时兼容[`str::parse`]与[`str::try_into`]
    impl FromStr for Term {
        type Err = anyhow::Error;

        fn from_str(s: &str) -> Result<Self, Self::Err> {
            s.try_into()
        }
    }
}

/// 📄OpenNARS `nars.language.Term`
/// * ⚠️不包含与特定层数Narsese有关的逻辑
///   * 📄事关NAL-6的`isConstant`、`renameVariables`方法，不予在此实现
/// * ⚠️不包含与「记忆区」有关的方法
///   * 📄`make`
///   * 📝OpenNARS中有关`make`的目的：避免在记忆区中**重复构造**词项
///     * 🚩已经在概念区中⇒使用已有「概念」的词项
///     * 📌本质上是「缓存」的需求与作用
mod term {
    use super::*;
    use nar_dev_utils::if_return;
    /// 📄OpenNARS `nars.language.Term`
    impl Term {
        /// 📄OpenNARS `getName` 方法
        /// * 🆕使用自身内建的「获取名称」方法
        ///   * 相较OpenNARS更**短**
        ///   * 仍能满足OpenNARS的需求
        /// * 🎯OpenNARS原有需求
        ///   * 📌保证「词项不同 ⇔ 名称不同」
        ///   * 📌保证「可用于『概念』『记忆区』的索引」
        pub fn get_name(&self) -> String {
            self.format_name()
        }

        /// 📄OpenNARS `getComplexity` 方法
        /// * 🚩逻辑 from OpenNARS
        ///   * 词语 ⇒ 1
        ///   * 变量 ⇒ 0
        ///   * 复合 ⇒ 1 + 所有组分复杂度之和
        ///
        /// # 📄OpenNARS
        ///
        /// - The syntactic complexity, for constant atomic Term, is 1.
        /// - The complexity of the term is the sum of those of the components plus 1
        /// - The syntactic complexity of a variable is 0, because it does not refer to * any concept.
        ///
        /// @return The complexity of the term, an integer
        pub fn get_complexity(&self) -> usize {
            // 对「变量」特殊处理：不引用到任何「概念」
            if_return! {
                self.instanceof_variable() => 0
            }
            // 剩余类型
            use TermComponents::*;
            match &*self.components {
                // 占位符 ⇒ 0
                Empty => 0,
                // 原子 ⇒ 1 | 不包括「变量」
                Named(..) => 1,
                // 一元 ⇒ 1 + 内部词项复杂度
                Unary(term) => 1 + term.get_complexity(),
                // 二元 ⇒ 1 + 内部所有词项复杂度之和
                Binary(term1, term2) => 1 + term1.get_complexity() + term2.get_complexity(),
                // 多元 ⇒ 1 + 内部所有词项复杂度之和
                Multi(terms) | MultiIndexed(_, terms) => {
                    1 + terms.iter().map(Term::get_complexity).sum::<usize>()
                }
            }
        }
    }
}

/// 📄OpenNARS `nars.language.CompoundTerm`
/// * ⚠️不包含与NAL-6有关的「变量」逻辑
///   * 📄`isConstant`、`renameVariables`
/// * ⚠️不包含与「记忆区」有关的方法
///   * 📄`addComponents`、`reduceComponents`
///
/// # 方法列表
/// 🕒最后更新：【2024-04-21 17:10:46】
///
/// * `isCommutative`
/// * `size`
/// * `componentAt`
/// * `componentAt`
/// * `getComponents`
/// * `cloneComponents`
/// * `containComponent`
/// * `containTerm`
/// * `containAllComponents`
///
/// # 📄OpenNARS
///
/// A CompoundTerm is a Term with internal (syntactic) structure
///
/// A CompoundTerm consists of a term operator with one or more component Terms.
///
/// This abstract class contains default methods for all CompoundTerms.
mod compound {
    use super::*;
    impl Term {
        /// 📄OpenNARS `isCommutative` 属性
        ///
        /// # 📄OpenNARS
        ///
        /// Check if the order of the components matters
        ///
        /// Commutative CompoundTerms: Sets, Intersections
        /// Commutative Statements: Similarity, Equivalence (except the one with a temporal order)
        /// Commutative CompoundStatements: Disjunction, Conjunction (except the one with a temporal order)
        pub fn is_commutative(&self) -> bool {
            matches!(
                self.identifier.as_str(),
                // Commutative CompoundTerms
                SET_EXT_OPERATOR
                    | SET_INT_OPERATOR
                    | INTERSECTION_EXT_OPERATOR
                    | INTERSECTION_INT_OPERATOR
                    // Commutative Statements
                    | SIMILARITY_RELATION
                    | EQUIVALENCE_RELATION
                    // Commutative CompoundStatements
                    | DISJUNCTION_OPERATOR
                    | CONJUNCTION_OPERATOR
            )
        }

        /// 📄OpenNARS `size` 属性
        /// * 🚩直接链接到[`TermComponents`]的属性
        /// * ⚠️对「像」不包括「像占位符」
        ///   * 📄`(/, A, _, B)`的`size`为`2`而非`3`
        ///
        /// # 📄OpenNARS
        ///
        /// get the number of components
        #[inline]
        pub fn size(&self) -> usize {
            self.components.len()
        }

        /// 📄OpenNARS `componentAt` 方法
        /// * 🚩直接连接到[`TermComponents`]的方法
        /// * ⚠️对「像」不受「像占位符」位置影响
        ///
        /// # 📄OpenNARS
        ///
        /// get a component by index
        #[inline]
        pub fn component_at(&self, index: usize) -> Option<&Term> {
            self.components.get(index)
        }

        /// 📄OpenNARS `componentAt` 方法
        /// * 🆕unsafe版本：若已知词项的组分数，则可经此对症下药
        /// * 🚩直接连接到[`TermComponents`]的方法
        /// * ⚠️对「像」不受「像占位符」位置影响
        ///
        /// # Safety
        ///
        /// ⚠️只有在「确保索引不会越界」才不会引发panic
        ///
        /// # 📄OpenNARS
        ///
        /// get a component by index
        #[inline]
        pub unsafe fn component_at_unchecked(&self, index: usize) -> &Term {
            self.components.get_unchecked(index)
        }

        /// 📄OpenNARS `getComponents` 属性
        /// * 🚩直接连接到[`TermComponents`]的方法
        /// * 🚩【2024-04-21 16:11:59】目前只需不可变引用
        ///   * 🔎OpenNARS中大部分用法是「只读」情形
        ///
        /// # 📄OpenNARS
        ///
        /// Get the component list
        #[inline]
        pub fn get_components(&self) -> impl Iterator<Item = &Term> {
            self.components.iter()
        }

        /// 📄OpenNARS `cloneComponents` 方法
        /// * 🚩直接连接到[`TermComponents`]的方法
        /// * ✅直接使用自动派生的[`TermComponents::clone`]方法，且不需要OpenNARS中的`cloneList`
        ///
        /// # 📄OpenNARS
        ///
        /// Clone the component list
        pub fn clone_components(&self) -> TermComponents {
            *self.components.clone()
        }

        /// 📄OpenNARS `containComponent` 方法
        /// * 🎯检查其是否包含**直接**组分
        /// * 🚩直接基于已有迭代器方法
        ///
        /// # 📄OpenNARS
        ///
        /// Check whether the compound contains a certain component
        pub fn contain_component(&self, component: &Term) -> bool {
            self.get_components().any(|term| term == component)
        }

        /// 📄OpenNARS `containTerm` 方法
        /// * 🎯检查其是否**递归**包含组分
        /// * 🚩直接基于已有迭代器方法
        ///
        /// # 📄OpenNARS
        ///
        /// Recursively check if a compound contains a term
        #[allow(clippy::only_used_in_recursion)]
        pub fn contain_term(&self, term: &Term) -> bool {
            self.get_components()
                .any(|component| component.contain_term(term))
        }

        /// 🆕用于替代Java的`getClass`
        #[inline(always)]
        pub fn get_class(&self) -> &str {
            &self.identifier
        }

        /// 📄OpenNARS `containAllComponents` 方法
        /// * 🎯分情况检查「是否包含所有组分」
        ///   * 📌同类⇒检查其是否包含`other`的所有组分
        ///   * 📌异类⇒检查其是否包含`other`作为整体
        /// * 🚩直接基于已有迭代器方法
        ///
        /// # 📄OpenNARS
        ///
        /// Check whether the compound contains all components of another term, or that term as a whole
        pub fn contain_all_components(&self, other: &Term) -> bool {
            match self.get_class() == other.get_class() {
                true => other
                    .get_components()
                    .all(|should_in| self.contain_component(should_in)),
                false => self.contain_component(other),
            }
        }
    }
}

/// 📄OpenNARS `nars.language.Variable`
/// * 📌与NAL-6有关的「变量」逻辑
///   * 📄`isConstant`、`renameVariables`……
/// * ✨既包括直接与`Variable`有关的方法，也包括来自`nars.language.Term`、`nars.language.CompoundTerm`的方法
///
/// # 方法列表
/// 🕒最后更新：【2024-04-21 17:10:46】
///
/// * `isConstant`
/// * `renameVariables`
/// * `applySubstitute`
/// * `getType` => `getVariableType`
/// * `containVarI`
/// * `containVarD`
/// * `containVarQ`
/// * `containVar`
/// * `unify`
/// * `makeCommonVariable` (内用)
/// * `isCommonVariable` (内用)
/// * `hasSubstitute`
///
/// TODO: 完成实际代码
///
/// # 📄OpenNARS
///
/// A variable term, which does not correspond to a concept
mod variable {}

/// 单元测试
#[cfg(test)]
mod test {
    use super::*;
    use anyhow::Result;
    use narsese::{
        conversion::{
            inter_type::lexical_fold::TryFoldInto,
            string::impl_lexical::format_instances::FORMAT_ASCII,
        },
        lexical::Term as LexicalTerm,
        lexical_nse_term,
    };

    /// 用于批量生成「解析后的词项」
    /// * 🚩使用`?`直接在解析处上抛错误
    macro_rules! term {
        // 词项数组
        ([$($s:expr $(,)?)*]) => {
            [ $( term!($s) ),* ]
        };
        // 词项引用数组（一次性）
        ([$($s:expr $(,)?)*] &) => {
            [ $( &term!($s) ),* ]
        };
        // 单个词项
        ($s:expr) => {
            $s.parse::<Term>()?
        };
    }

    /// 测试/词项
    #[test]
    fn test_term() -> Result<()> {
        // 测试一个词项
        fn detect(term: &Term) {
            use TermComponents::*;
            match term.id_comp() {
                (WORD, Named(name)) => {
                    println!("word with {name:?}");
                }
                (IMAGE_EXT_OPERATOR, MultiIndexed(i, v)) => {
                    println!("ext_image '/' with {i}");
                    println!("<components>");
                    for term in v {
                        detect(term);
                    }
                    println!("</components>");
                }
                _ => println!("term {:?}: {}", term.identifier, term.format_name()),
            }
        }
        // 构造一个词项
        let im_ext = Term::new(
            IMAGE_EXT_OPERATOR,
            TermComponents::MultiIndexed(1, vec![Term::new_word("word")]),
        );
        detect(&im_ext);
        // 从「词法Narsese」中解析词项
        detect(&term!("<A --> B>"));
        detect(&term!("(--, A)"));
        detect(&term!("(--, (&&, <A --> B>, <B --> C>))"));
        // 返回成功
        Ok(())
    }

    /// 测试 / 词法折叠
    #[test]
    fn test_lexical_fold() -> Result<()> {
        fn fold(t: LexicalTerm) -> Result<Term> {
            print!("{:?} => ", FORMAT_ASCII.format(&t));
            let term: Term = t.try_fold_into(&())?;
            println!("{:?}", term.format_name());
            Ok(term)
        }
        fold(lexical_nse_term!(<A --> B>))?;
        fold(lexical_nse_term!((&&, C, B, A, (/, A, _, B))))?;
        fold(lexical_nse_term!(<(*, {SELF}, x, y) --> ^left>))?;
        fold(lexical_nse_term!([2, 1, 0, $0, #1, ?2]))?;
        fold(lexical_nse_term!(<A <-> {A}>))?;
        fold(lexical_nse_term!(<{B} <=> B>))?;
        fold(lexical_nse_term!(<{SELF} ==> (--, [good])>))?;
        Ok(())
    }

    mod components {
        use super::*;
        use nar_dev_utils::asserts;

        /// 测试/长度
        #[test]
        fn len() -> Result<()> {
            macro_rules! len {
                ($s:expr) => {
                    term!($s).components.len()
                };
            }
            asserts! {
                // 平常情况
                len!("B") => 0
                len!("?quine") => 0
                len!("<A --> B>") => 2
                len!("(*, {SELF}, x, y)") => 3
                len!("(--, [good])") => 1
                // 像：占位符不算
                len!("(/, A, _, B)") => 2
                // 集合：缩并
                len!("[2, 1, 0, 0, 1, 2]") => 3
            }
            Ok(())
        }

        /// 测试/判空
        #[test]
        fn is_empty() -> Result<()> {
            macro_rules! is_empty {
                ($s:expr) => {
                    term!($s).components.is_empty()
                };
            }
            asserts! {
                is_empty!("B") => true
                is_empty!("?quine") => true
                is_empty!("<A --> B>") => false
                is_empty!("(*, {SELF}, x, y)") => false
                is_empty!("(--, [good])") => false
                is_empty!("(/, A, _, B)") => false
                is_empty!("[2, 1, 0, 0, 1, 2]") => false
            }
            Ok(())
        }

        /// 测试/获取
        #[test]
        fn get() -> Result<()> {
            macro_rules! get {
                ($s:expr, $i:expr) => {
                    term!($s).components.get($i)
                };
            }
            asserts! {
                // 平常情况
                get!("B", 0) => None
                get!("?quine", 0) => None
                get!("<A --> B>", 0) => Some(&"A".parse()?)
                get!("<A --> B>", 1) => Some(&"B".parse()?)
                get!("<A --> B>", 2) => None
                get!("{SELF}", 0) => Some(&"SELF".parse()?)
                get!("{SELF}", 1) => None
                get!("(*, {SELF}, x, y)", 0) => Some(&"{SELF}".parse()?)
                get!("(*, {SELF}, x, y)", 1) => Some(&"x".parse()?)
                get!("(*, {SELF}, x, y)", 2) => Some(&"y".parse()?)
                get!("(*, {SELF}, x, y)", 3) => None
                get!("(--, [good])", 0) => Some(&"[good]".parse()?)
                get!("(--, [good])", 1) => None
                // 像：占位符不算
                get!("(/, A, _, B)", 0) => Some(&"A".parse()?)
                get!("(/, A, _, B)", 1) => Some(&"B".parse()?)
                get!("(/, A, _, B)", 2) => None
                // 集合：排序 & 缩并
                get!("[2, 1, 0, 0, 1, 2]", 0) => Some(&"0".parse()?)
                get!("[2, 1, 0, 0, 1, 2]", 1) => Some(&"1".parse()?)
                get!("[2, 1, 0, 0, 1, 2]", 2) => Some(&"2".parse()?)
                get!("[2, 1, 0, 0, 1, 2]", 3) => None
            }
            Ok(())
        }

        /// 测试/获取
        #[test]
        fn get_unchecked() -> Result<()> {
            macro_rules! get_unchecked {
                ($s:expr, $i:expr) => {
                    unsafe { $s.parse::<Term>()?.components.get_unchecked($i) }
                };
            }
            asserts! {
                // 平常情况
                get_unchecked!("<A --> B>", 0) => &term!("A")
                get_unchecked!("<A --> B>", 1) => &term!("B")
                get_unchecked!("{SELF}", 0) => &term!("SELF")
                get_unchecked!("(*, {SELF}, x, y)", 0) => &term!("{SELF}")
                get_unchecked!("(*, {SELF}, x, y)", 1) => &term!("x")
                get_unchecked!("(*, {SELF}, x, y)", 2) => &term!("y")
                get_unchecked!("(--, [good])", 0) => &term!("[good]")
                // 像：占位符不算
                get_unchecked!("(/, A, _, B)", 0) => &term!("A")
                get_unchecked!("(/, A, _, B)", 1) => &term!("B")
                // 集合：排序 & 缩并
                get_unchecked!("[2, 1, 0, 0, 1, 2]", 0) => &term!("0")
                get_unchecked!("[2, 1, 0, 0, 1, 2]", 1) => &term!("1")
                get_unchecked!("[2, 1, 0, 0, 1, 2]", 2) => &term!("2")
            }
            Ok(())
        }

        /// 测试/迭代器
        /// * 🚩转换为数组，然后跟数组比对
        #[test]
        fn iter() -> Result<()> {
            macro_rules! iter {
                ($s:expr) => {
                    term!($s).components.iter().collect::<Vec<_>>()
                };
            }
            asserts! {
                iter!("<A --> B>") => term!(["A", "B"]&)
                // 平常情况
                iter!("{SELF}") => term!(["SELF"]&)
                iter!("(*, {SELF}, x, y)") => term!(["{SELF}", "x", "y"]&)
                iter!("(--, [good])") => term!(["[good]"]&)
                // 像：占位符不算
                iter!("(/, A, _, B)") => term!(["A", "B"]&)
                // 集合：排序 & 缩并
                iter!("[2, 1, 0, 0, 1, 2]") => term!(["0", "1", "2"]&)
            }
            Ok(())
        }

        // TODO: 更多函数的测试
    }
}
