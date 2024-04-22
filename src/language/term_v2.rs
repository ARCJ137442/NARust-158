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

    /// 自由属性「是否为常量」
    /// * 🎯用于决定其在记忆区、NAL-6推理中的行为
    /// * ❓为何要设置成「结构属性」：会在系统构造「语句」时概改变
    ///   * 📝源自OpenNARS：构造语句时所直接涉及的词项均为「常量词项」，必须进入记忆区
    /// * 📄OpenNARS `isConstant` 属性
    /// * 📜默认为`true`
    is_constant: bool,
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
            // 使用默认值构造
            let mut term = Self {
                identifier: identifier.into(),
                components: Box::new(components),
                is_constant: true, // 取默认值
            };
            // 初始化「是否常量」为「是否不含变量」 | ⚠️后续可能会被修改
            term.is_constant = !term.contain_var();
            // 返回
            term
        }

        /// 从「语句」初始化
        /// * 🎯应对OpenNARS中「语句内初始化词项⇒必定是『常量』」的情形
        /// * 🎯后续遇到异常的「是常量」情况，便于追溯
        pub fn init_from_sentence(&mut self) {
            self.is_constant = true;
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

        /// 判断「是否包含指定类型的词项」
        /// * 🎯支持「词项」中的方法，递归判断「是否含有变量」
        pub fn contain_type(&self, identifier: &str) -> bool {
            self.identifier == identifier || self.components.contain_type(identifier)
        }

        /// 判断和另一词项是否「结构匹配」
        /// * 🎯变量替换中的模式匹配
        #[inline(always)]
        pub fn structural_match(&self, other: &Self) -> bool {
            self.components.structural_match(&other.components)
        }
    }

    /// 内建属性
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

        /// 获取其中「所有元素」的迭代器（可变引用）
        /// * 🚩返回一个迭代器，迭代其中所有「元素」
        /// * 🎯词项的「变量代入」替换
        /// * ⚠️并非「深迭代」：仅迭代自身的下一级词项，不会递归深入
        pub fn iter_mut(&mut self) -> impl Iterator<Item = &mut Term> {
            use TermComponents::*;
            // * 📝必须添加类型注释，以便统一不同类型的`Box`，进而统一「迭代器」类型
            let b: Box<dyn Iterator<Item = &mut Term>> = match self {
                // 一定空
                Empty | Named(..) => Box::new(None.into_iter()),
                // 一定非空
                Unary(term) => Box::new([term].into_iter()),
                Binary(term1, term2) => Box::new([term1, term2].into_iter()),
                // 可能空
                Multi(terms) | MultiIndexed(_, terms) => Box::new(terms.iter_mut()),
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

        /// 判断「是否包含指定类型的词项」
        /// * 🎯支持「词项」中的方法，递归判断「是否含有变量」
        /// * 🚩【2024-04-21 20:35:23】目前直接基于迭代器
        ///   * 📌牺牲一定性能，加快开发速度
        pub fn contain_type(&self, identifier: &str) -> bool {
            self.iter().any(|term| term.contain_type(identifier))
        }

        /// 判断「结构模式上是否匹配」
        /// * 🚩判断二者在「结构大小」与（可能有的）「结构索引」是否符合
        /// * 🎯变量替换中的「相同结构之模式替换」
        /// * 📄`variable::find_substitute`
        pub fn structural_match(&self, other: &Self) -> bool {
            use TermComponents::*;
            match (self, other) {
                // 同类型 / 空 | 同类型 / 二元
                (Empty | Named(..), Empty | Named(..)) | (Binary(..), Binary(..)) => true,
                // 同类型 / 多元
                (Multi(terms1), Multi(terms2)) => terms1.len() == terms2.len(),
                (MultiIndexed(i1, terms1), MultiIndexed(i2, terms2)) => {
                    i1 == i2 && terms1.len() == terms2.len()
                }
                // 其它情形（类型相异）
                _ => false,
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
                (
                    INSTANCE_RELATION, // 派生系词/实例
                    Statement {
                        subject, predicate, ..
                    },
                ) => Term::new_inheritance(
                    Term::new_set_ext(vec![subject.try_fold_into(&())?]),
                    predicate.try_fold_into(&())?,
                ),

                (
                    PROPERTY_RELATION, // 派生系词/属性
                    Statement {
                        subject, predicate, ..
                    },
                ) => Term::new_inheritance(
                    subject.try_fold_into(&())?,
                    Term::new_set_int(vec![predicate.try_fold_into(&())?]),
                ),
                (
                    INSTANCE_PROPERTY_RELATION, // 派生系词/实例属性
                    Statement {
                        subject, predicate, ..
                    },
                ) => Term::new_inheritance(
                    Term::new_set_ext(vec![subject.try_fold_into(&())?]),
                    Term::new_set_int(vec![predicate.try_fold_into(&())?]),
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
        /// 用于判断是否为「变量词项」
        /// * 📄OpenNARS `instanceof Variable` 逻辑
        /// * 🎯判断「[是否内含变量](Self::contain_var)」
        pub fn instanceof_variable(&self) -> bool {
            matches!(
                self.identifier.as_str(),
                VAR_INDEPENDENT | VAR_DEPENDENT | VAR_QUERY
            )
        }

        /// 用于判断是否为「复合词项」
        /// * 📄OpenNARS `instanceof CompoundTerm` 逻辑
        pub fn instanceof_compound(&self) -> bool {
            matches!(
                self.identifier.as_str(),
                SET_EXT_OPERATOR
                    | SET_INT_OPERATOR
                    | INTERSECTION_EXT_OPERATOR
                    | INTERSECTION_INT_OPERATOR
                    | DIFFERENCE_EXT_OPERATOR
                    | DIFFERENCE_INT_OPERATOR
                    | PRODUCT_OPERATOR
                    | IMAGE_EXT_OPERATOR
                    | IMAGE_INT_OPERATOR
                    | CONJUNCTION_OPERATOR
                    | DISJUNCTION_OPERATOR
                    | NEGATION_OPERATOR
            )
        }

        /// 用于判断是否为「陈述词项」
        /// * 📄OpenNARS `instanceof Statement` 逻辑
        pub fn instanceof_statement(&self) -> bool {
            matches!(
                self.identifier.as_str(),
                // 四大主要系词
                INHERITANCE_RELATION
                    | SIMILARITY_RELATION
                    | IMPLICATION_RELATION
                    | EQUIVALENCE_RELATION
                    // ↓下边都是派生系词
                    | INSTANCE_RELATION
                    | PROPERTY_RELATION
                    | INSTANCE_PROPERTY_RELATION
            )
        }

        /// 📄OpenNARS `Term.getName` 方法
        /// * 🆕使用自身内建的「获取名称」方法
        ///   * 相较OpenNARS更**短**
        ///   * 仍能满足OpenNARS的需求
        /// * 🎯OpenNARS原有需求
        ///   * 📌保证「词项不同 ⇔ 名称不同」
        ///   * 📌保证「可用于『概念』『记忆区』的索引」
        pub fn get_name(&self) -> String {
            self.format_name()
        }

        /// 📄OpenNARS `Term.getComplexity` 方法
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
        /// 📄OpenNARS `CompoundTerm.isCommutative` 属性
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

        /// 📄OpenNARS `CompoundTerm.size` 属性
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

        /// 📄OpenNARS `CompoundTerm.componentAt` 方法
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

        /// 📄OpenNARS `CompoundTerm.componentAt` 方法
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

        /// 📄OpenNARS `CompoundTerm.getComponents` 属性
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

        /// 📄OpenNARS `CompoundTerm.cloneComponents` 方法
        /// * 🚩直接连接到[`TermComponents`]的方法
        /// * ✅直接使用自动派生的[`TermComponents::clone`]方法，且不需要OpenNARS中的`cloneList`
        ///
        /// # 📄OpenNARS
        ///
        /// Clone the component list
        pub fn clone_components(&self) -> TermComponents {
            *self.components.clone()
        }

        /// 📄OpenNARS `CompoundTerm.containComponent` 方法
        /// * 🎯检查其是否包含**直接**组分
        /// * 🚩直接基于已有迭代器方法
        ///
        /// # 📄OpenNARS
        ///
        /// Check whether the compound contains a certain component
        pub fn contain_component(&self, component: &Term) -> bool {
            self.get_components().any(|term| term == component)
        }

        /// 📄OpenNARS `CompoundTerm.containTerm` 方法
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

        /// 📄OpenNARS `CompoundTerm.containAllComponents` 方法
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
pub mod variable {
    use super::*;
    use std::collections::HashMap;

    impl Term {
        /// 📄OpenNARS `Term.isConstant` 属性
        /// * 🚩检查其是否为「常量」：自身是否「不含变量」
        /// * 🎯决定其是否能**成为**一个「概念」（被作为「概念」存入记忆区）
        /// * ❓OpenNARS中在「构造语句」时又会将`isConstant`属性置为`true`，这是为何
        ///   * 📝被`Sentence(..)`调用的`CompoundTerm.renameVariables()`会直接将词项「视作常量」
        ///   * 💭这似乎是被认为「即便全是变量，只要是【被作为语句输入过】的，就会被认作是『常量』」
        ///   * 📝然后这个「是否常量」会在「记忆区」中被认作「是否能从中获取概念」的依据：`if (!term.isConstant()) { return null; }`
        /// * 🚩【2024-04-21 23:46:12】现在变为「只读属性」：接受OpenNARS中有关「设置语句时/替换变量后 变为『常量』」的设定
        ///   * 💫【2024-04-22 00:03:10】后续仍然有一堆复杂逻辑要考虑
        ///
        /// # 📄OpenNARS
        ///
        /// Check whether the current Term can name a Concept.
        ///
        /// - A Term is constant by default
        /// - A variable is not constant
        /// - (for `CompoundTerm`) check if the term contains free variable
        #[inline(always)]
        pub fn is_constant(&self) -> bool {
            !self.contain_var()
        }

        /// 📄OpenNARS `Variable.containVar` 方法
        /// * 🚩检查其是否「包含变量」
        ///   * 自身为「变量词项」或者其包含「变量词项」
        /// * 🎯用于决定复合词项是否为「常量」
        /// * 📝OpenNARS中对于复合词项的`isConstant`属性采用「惰性获取」的机制
        ///   * `isConstant`作为`!Variable.containVar(name)`进行初始化
        /// * 🆕实现方法：不同于OpenNARS「直接从字符串中搜索子串」的方式，基于递归方法设计
        ///
        /// # 📄OpenNARS
        ///
        /// Check whether a string represent a name of a term that contains a variable
        #[inline]
        pub fn contain_var(&self) -> bool {
            self.instanceof_variable() || self.components.contain_var()
        }

        /// 📄OpenNARS `Variable.containVarI` 方法
        /// * 🎯判断「是否包含指定类型的变量」
        /// * 🚩通过「判断是否包含指定标识符的词项」完成判断
        pub fn contain_var_i(&self) -> bool {
            self.contain_type(VAR_INDEPENDENT)
        }

        /// 📄OpenNARS `Variable.containVarD` 方法
        /// * 🎯判断「是否包含指定类型的变量」
        /// * 🚩通过「判断是否包含指定标识符的词项」完成判断
        pub fn contain_var_d(&self) -> bool {
            self.contain_type(VAR_DEPENDENT)
        }

        /// 📄OpenNARS `Variable.containVarQ` 方法
        /// * 🎯判断「是否包含指定类型的变量」
        /// * 🚩通过「判断是否包含指定标识符的词项」完成判断
        pub fn contain_var_q(&self) -> bool {
            self.contain_type(VAR_QUERY)
        }

        /// 📄OpenNARS `Term.renameVariables` 方法
        /// * 🚩重命名自身变量为一系列「固定编号」
        ///   * 📌整体逻辑：将其中所有不同名称的「变量」编篡到一个字典中，排序后以编号重命名（抹消具体名称）
        ///   * 📝因为这些变量都位于「词项内部」，即「变量作用域全被约束在词项内」，故无需考虑「跨词项编号歧义」的问题
        /// * 🎯用于将「变量」统一命名成固定的整数编号
        /// * ❓目前对此存疑：必要性何在？
        ///   * ~~不一致性：输入`<$A --> $B>`再输入`<$B --> $A>`会被看作是一样的变量~~
        ///   * 📌既然是「变量作用域对整个词项封闭」那**任意名称都没问题**
        ///
        /// # 📄OpenNARS
        ///
        /// @ Term: Blank method to be override in CompoundTerm
        ///
        /// @ CompoundTerm:
        ///   * Rename the variables in the compound, called from Sentence constructors
        ///   * Recursively rename the variables in the compound
        pub fn rename_variables(&mut self) {
            unimplemented!("【2024-04-21 20:48:33】目前尚不清楚其必要性");
        }

        /// 📄OpenNARS `CompoundTerm.applySubstitute` 方法
        /// * 🚩直接分派给其组分
        /// * 📝OpenNARS中「原子词项」不参与「变量替代」：执行无效果
        ///
        /// # 📄OpenNARS
        ///
        /// Recursively apply a substitute to the current CompoundTerm
        #[inline]
        pub fn apply_substitute(&mut self, substitution: &VarSubstitution) {
            self.components.apply_substitute(substitution)
        }

        /// 📄OpenNARS `Variable.getType` 方法
        /// * 🎯在OpenNARS中仅用于「判断变量类型相等」
        /// * 🚩归并到「判断词项标识符相等」
        ///
        /// # 📄OpenNARS
        ///
        /// Get the type of the variable
        #[inline(always)]
        pub fn get_variable_type(&self) -> &str {
            &self.identifier
        }
    }

    /// 📄OpenNARS `Variable.unify` 方法
    /// * 🚩总体流程：找「可替换的变量」并（两头都）替换之
    /// * 📝⚠️不对称性：从OpenNARS `findSubstitute`中所见，
    ///   * `to_be_unified_1`是「包含变量，将要被消元」的那个（提供键），
    ///   * 而`to_be_unified_2`是「包含常量，将要用于消元」的那个（提供值）
    ///
    /// # 📄OpenNARS
    ///
    /// To unify two terms
    ///
    /// @param type            The type of variable that can be substituted
    /// @param to_be_unified_1 The first term to be unified
    /// @param to_be_unified_2 The second term to be unified
    /// @param unified_in_1    The compound containing the first term
    /// @param unified_in_2    The compound containing the second term
    /// @return Whether the unification is possible
    ///
    /// # 📄案例
    ///
    /// ## 1 from OpenNARS调试 @ 【2024-04-21 21:48:21】
    ///
    /// 传入
    ///
    /// - type: "$"
    /// - to_be_unified_1: "<$1 --> B>"
    /// - to_be_unified_2: "<C --> B>"
    /// - unified_in_1: <<$1 --> A> ==> <$1 --> B>>
    /// - unified_in_2: <C --> B>
    ///
    /// 结果
    /// - to_be_unified_1: "<$1 --> B>"
    /// - to_be_unified_2: "<C --> B>"
    /// - unified_in_1: <<C --> A> ==> <C --> B>>
    /// - unified_in_2: <C --> B>
    ///
    #[allow(unused_variables)]
    pub fn unify(
        var_type: &str,
        to_be_unified_1: &Term,
        to_be_unified_2: &Term,
        unified_in_1: &mut Term,
        unified_in_2: &mut Term,
    ) -> bool {
        // 构造并找出所有「变量替代模式」
        // * 🚩递归找出其中所有「可被替代的变量」装载进「变量替换映射」中
        let mut substitution_1 = VarSubstitution::new();
        let mut substitution_2 = VarSubstitution::new();
        let has_substitute = find_substitute(
            var_type,
            to_be_unified_1,
            to_be_unified_2,
            &mut substitution_1,
            &mut substitution_2,
        );
        // 根据「变量替换映射」在两头相应地替换变量
        // * 🚩若「变量替换映射」为空，本来就不会执行
        unified_in_1.apply_substitute(&substitution_1);
        unified_in_2.apply_substitute(&substitution_2);
        // 返回「是否替换了变量」
        has_substitute
    }

    /// 📄OpenNARS `Variable.findSubstitute` 方法
    /// * 💫【2024-04-21 21:40:45】目前尚未能完全理解此处的逻辑
    /// * 📝【2024-04-21 21:50:42】递归查找一个「同位替代」的「变量→词项」映射
    /// * 🚧缺少注释：逻辑基本照抄OpenNARS的代码
    ///
    /// # 📄OpenNARS
    ///
    /// To recursively find a substitution that can unify two Terms without changing them
    ///
    /// @param type            The type of variable that can be substituted
    /// @param to_be_unified_1 The first term to be unified
    /// @param to_be_unified_2 The second term to be unified
    /// @param substitution_1  The substitution for term1 formed so far
    /// @param substitution_2  The substitution for term2 formed so far
    /// @return Whether the unification is possible
    ///
    /// # 📄案例
    ///
    /// ## 1 from OpenNARS调试 @ 【2024-04-21 21:48:21】
    ///
    /// 传入
    ///
    /// - type: "$"
    /// - to_be_unified_1: "<$1 --> B>"
    /// - to_be_unified_2: "<C --> B>"
    /// - substitution_1: HashMap{}
    /// - substitution_2: HashMap{}
    ///
    /// 结果
    ///
    /// - 返回值 = true
    /// - substitution_1: HashMap{ Term"$1" => Term"C" }
    /// - substitution_2: HashMap{}
    ///
    /// ## 2 from OpenNARS调试 @ 【2024-04-21 22:05:46】
    ///
    /// 传入
    ///
    /// - type: "$"
    /// - to_be_unified_1: "<<A --> $1> ==> <B --> $1>>"
    /// - to_be_unified_2: "<B --> C>"
    /// - substitution_1: HashMap{}
    /// - substitution_2: HashMap{}
    ///
    /// 结果
    ///
    /// - 返回值 = true
    /// - substitution_1: HashMap{ Term"$1" => Term"C" }
    /// - substitution_2: HashMap{}
    pub fn find_substitute(
        var_type: &str,
        to_be_unified_1: &Term,
        to_be_unified_2: &Term,
        substitution_1: &mut VarSubstitution,
        substitution_2: &mut VarSubstitution,
    ) -> bool {
        //==== 内用函数 ====//

        /// 特殊的「共有变量」标识符
        /// * 📄迁移自OpenNARS
        const COMMON_VARIABLE: &str = "COMMON_VARIABLE";

        /// 📄OpenNARS `Variable.makeCommonVariable` 函数
        /// * 🎯用于「变量统一」方法
        fn make_common_variable(v1: &Term, v2: &Term) -> Term {
            Term::new(
                COMMON_VARIABLE,
                TermComponents::Named(v1.get_name() + &v2.get_name()),
            )
        }

        /// 📄OpenNARS `Variable.isCommonVariable` 函数
        fn is_common_variable(v: &Term) -> bool {
            v.identifier() == COMMON_VARIABLE
        }

        //==== 正式开始函数体 ====//
        // 📄 `if ((term1 instanceof Variable) && (((Variable) term1).getType() == type)) {`
        if to_be_unified_1.get_variable_type() == var_type {
            match substitution_1.get(to_be_unified_1).cloned() {
                // already mapped
                Some(new_term) => {
                    // 📄 `return findSubstitute(type, t, term2, map1, map2);`
                    // 在新替换的变量中递归深入
                    find_substitute(
                        var_type,
                        &new_term, // ! 必须复制：否则会存留不可变引用
                        to_be_unified_2,
                        substitution_1,
                        substitution_2,
                    )
                }
                // not mapped yet
                None => {
                    if to_be_unified_2.get_variable_type() == var_type {
                        let common_var = make_common_variable(to_be_unified_1, to_be_unified_2);
                        substitution_1.put(to_be_unified_1, common_var.clone()); // unify
                        substitution_2.put(to_be_unified_2, common_var); // unify
                    } else {
                        substitution_1.put(to_be_unified_1, to_be_unified_2.clone()); // elimination
                        if is_common_variable(to_be_unified_1) {
                            substitution_2.put(to_be_unified_1, to_be_unified_2.clone());
                        }
                    }
                    true
                }
            }
        } else if to_be_unified_2.get_variable_type() == var_type {
            // 📄 `else if ((term2 instanceof Variable) && (((Variable) term2).getType() == type)) {`
            // 📄 `t = map2.get(var2); if (t != null) { .. }`
            match substitution_2.get(to_be_unified_2).cloned() {
                // already mapped
                Some(new_term) => {
                    find_substitute(
                        var_type,
                        to_be_unified_1,
                        &new_term, // ! 必须复制：否则会存留不可变引用
                        substitution_1,
                        substitution_2,
                    )
                }
                // not mapped yet
                None => {
                    /*
                     * 📝【2024-04-22 00:13:19】发生在如下场景：
                     * <(&&, <A-->C>, <B-->$2>) ==> <C-->$2>>.
                     * <(&&, <A-->$1>, <B-->D>) ==> <$1-->D>>.
                     * <(&&, <A-->C>, <B-->D>) ==> <C-->D>>?
                     *
                     * 系列调用：
                     * * `$` `A` `$1`
                     * * `$` `D` `$1`
                     * * `$` `<C --> D>` `<$1 --> D>`
                     * * `$` `<C --> D>` `<C --> $1>`
                     *
                     * 📌要点：可能两边各有「需要被替换」的地方
                     */
                    substitution_2.put(to_be_unified_2, to_be_unified_1.clone()); // elimination
                    if is_common_variable(to_be_unified_2) {
                        substitution_1.put(to_be_unified_2, to_be_unified_1.clone());
                    }
                    true
                }
            }
        } else if to_be_unified_1.instanceof_compound()
            && to_be_unified_1.get_class() == to_be_unified_2.get_class()
            // 必须结构匹配
            // 📄 `if (cTerm1.size() != ...... return false; }`
            && to_be_unified_1.structural_match(to_be_unified_2)
        {
            // 📄 `else if ((term1 instanceof CompoundTerm) && term1.getClass().equals(term2.getClass())) {`
            // ? ❓为何要打乱无序词项
            // 📄 `if (cTerm1.isCommutative()) { Collections.shuffle(list, Memory.randomNumber); }`
            // ! 🚩【2024-04-22 09:43:26】此处暂且不打乱无序词项：疑点重重
            // 对位遍历
            // for (t1, t2) in to_be_unified_1
            //     .get_components()
            //     .zip(to_be_unified_2.get_components())
            // {
            //     if !find_substitute(var_type, t1, t2, substitution_1, substitution_2) {
            //         return false;
            //     }
            // }
            // * 🚩【2024-04-22 09:45:55】采用接近等价的纯迭代器方案，可以直接返回
            to_be_unified_1
                .get_components()
                .zip(to_be_unified_2.get_components())
                .all(|(t1, t2)| find_substitute(var_type, t1, t2, substitution_1, substitution_2))
        } else {
            // for atomic constant terms
            to_be_unified_1 == to_be_unified_2
        }
        // todo!("【2024-04-22 09:19:16】目前尚未能完全理解")
    }

    pub fn has_substitute(var_type: &str, to_be_unified_1: &Term, to_be_unified_2: &Term) -> bool {
        // 📄 `return findSubstitute(type, term1, term2, new HashMap<Term, Term>(), new HashMap<Term, Term>());`
        find_substitute(
            var_type,
            to_be_unified_1,
            to_be_unified_2,
            // 创建一个临时的「变量替换映射」
            &mut VarSubstitution::new(),
            &mut VarSubstitution::new(),
        )
    }

    impl TermComponents {
        /// 判断「是否包含变量（词项）」
        /// * 🎯支持「词项」中的方法，递归判断「是否含有变量」
        /// * 🚩【2024-04-21 20:35:23】目前直接基于迭代器
        ///   * 📌牺牲一定性能，加快开发速度
        pub fn contain_var(&self) -> bool {
            self.iter().any(Term::contain_var)
        }

        /// 📄OpenNARS `CompoundTerm.applySubstitute` 方法
        pub fn apply_substitute(&mut self, substitution: &VarSubstitution) {
            // 遍历其中所有地方的可变引用
            for term in self.iter_mut() {
                // 寻找其「是否有替代」
                match substitution.get(term) {
                    // 有替代⇒直接赋值
                    Some(new_term) => *term = new_term.clone(),
                    // 没替代⇒继续递归替代
                    None => term.apply_substitute(substitution),
                }
            }
        }
    }

    /// 用于表示「变量替换」的字典
    /// * 🎯NAL-6中的「变量替换」「变量代入」
    #[derive(Debug, Default, Clone)]
    #[doc(alias = "VariableSubstitution")]
    pub struct VarSubstitution {
        map: HashMap<Term, Term>,
    }

    impl VarSubstitution {
        /// 构造函数
        pub fn new() -> Self {
            Self::default()
        }

        /// 从其它构造出「散列映射」的地方构造
        pub fn from(map: impl Into<HashMap<Term, Term>>) -> Self {
            Self { map: map.into() }
        }

        /// 从其它构造出「散列映射」的地方构造
        pub fn from_pairs(pairs: impl IntoIterator<Item = (Term, Term)>) -> Self {
            Self {
                map: HashMap::from_iter(pairs),
            }
        }

        /// 尝试获取「替代项」
        /// * 🎯变量替换
        pub fn get(&self, key: &Term) -> Option<&Term> {
            self.map.get(key)
        }

        /// 设置「替代项」
        /// * 🎯寻找可替换变量，并返回结果
        /// * 🚩只在没有键时复制`key`，并且总是覆盖`value`值
        pub fn put(&mut self, key: &Term, value: Term) {
            match self.map.get_mut(key) {
                Some(old_value) => *old_value = value,
                None => {
                    self.map.insert(key.clone(), value);
                }
            }
        }
    }
}

/// 单元测试
#[cfg(test)]
mod test {
    use super::*;
    use anyhow::Result;
    use nar_dev_utils::asserts;
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

    mod variable {
        use super::*;
        use crate::language::variable::VarSubstitution;

        /// 测试/包含变量
        /// * ✨同时包含对「是否常量」的测试
        #[test]
        fn contain_var() -> Result<()> {
            asserts! {
                term!("<A --> var_word>").contain_var() => false
                term!("<A --> $var_word>").contain_var() => true
                term!("<A --> #var_word>").contain_var() => true
                term!("<A --> ?var_word>").contain_var() => true

                term!("<A --> var_word>").is_constant() => true
                term!("<A --> $var_word>").is_constant() => false
                term!("<A --> #var_word>").is_constant() => false
                term!("<A --> ?var_word>").is_constant() => false
                term!("<<A --> $1> ==> <B --> $1>>").is_constant() => true // ! 变量作用域限定在词项之内，被视作「常量」
            }
            Ok(())
        }

        /// 测试/变量替换
        #[test]
        fn apply_substitute() -> Result<()> {
            macro_rules! apply_substitute {
                {
                    $(
                        $term_str:expr, $substitution:expr
                        => $substituted_str:expr
                    )*
                } => {
                    $(
                        let mut term = term!($term_str);
                        term.apply_substitute(&$substitution);
                        assert_eq!(term, term!($substituted_str));
                    )*
                };
            }
            let substitution = VarSubstitution::from_pairs([
                (term!("var_word"), term!("word")),
                (term!("$1"), term!("1")),
            ]);
            apply_substitute! {
                "<A --> var_word>", substitution => "<A --> word>"
                "<<$1 --> A> ==> <B --> $1>>", substitution => "<<1 --> A> ==> <B --> 1>>"
            }
            Ok(())
        }
    }
}
