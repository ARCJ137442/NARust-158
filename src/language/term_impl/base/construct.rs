//! 实现 / 构造

use super::structs::*;
use crate::io::symbols::*;
use anyhow::Result;
use nar_dev_utils::*;

impl Term {
    /// 构造函数
    /// * ⚠️有限性：仅限在「内部」使用，不希望外部以此构造出「不符范围」的词项
    pub(in crate::language) fn new(
        identifier: impl Into<String>,
        components: TermComponents,
    ) -> Self {
        Self {
            identifier: identifier.into(),
            components,
        }
    }

    // 原子词项 //
    // * ℹ️此处一系列构造方法对标OpenNARS中各「词项」的构造函数
    // * ⚠️在MakeTerm中另有一套方法（参见term_making.rs）

    /// NAL-1 / 词语
    pub(in crate::language) fn new_word(name: impl Into<String>) -> Self {
        Self::new(WORD, TermComponents::Word(name.into()))
    }

    /// NAL-4 / 占位符
    /// * 📌【2024-04-21 00:36:27】需要一个「占位符」词项，以便和「词法Narsese」打交道
    /// * 🚩仅使用「占位符标识符+空组分」表示
    /// * 🎯仅在解析时临时出现
    /// * ⚠️【2024-04-25 09:45:51】不允许外部直接创建
    pub(in crate::language) fn new_placeholder() -> Self {
        Self::new(PLACEHOLDER, TermComponents::Empty)
    }

    /// NAL-6 / 独立变量
    pub(in crate::language) fn new_var_i(id: impl Into<usize>) -> Self {
        Self::new(VAR_INDEPENDENT, TermComponents::Variable(id.into()))
    }

    /// NAL-6 / 非独变量
    pub(in crate::language) fn new_var_d(id: impl Into<usize>) -> Self {
        Self::new(VAR_DEPENDENT, TermComponents::Variable(id.into()))
    }

    /// NAL-6 / 查询变量
    pub(in crate::language) fn new_var_q(id: impl Into<usize>) -> Self {
        Self::new(VAR_QUERY, TermComponents::Variable(id.into()))
    }

    /// NAL-7 / 间隔
    pub(crate) fn new_interval(n_time: impl Into<usize>) -> Self {
        Self::new(INTERVAL, TermComponents::Interval(n_time.into()))
    }

    /// 从旧的原子词项构造，但使用新的名称
    /// * 🎯重命名变量时，将变量「换名复制」
    /// * 🚩使用旧词项的标识符，但产生新的变量
    /// * ⚠️【2024-04-25 23:08:20】内部使用：会导致产生无效类型（改变了组分类型）
    pub(in crate::language) fn from_var_similar(
        var_type: impl Into<String>,
        new_id: impl Into<usize>,
    ) -> Self {
        Self::new(var_type.into(), TermComponents::Variable(new_id.into()))
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
    pub fn new_intersection_ext(terms: impl Into<Vec<Term>>) -> Self {
        Self::new(
            INTERSECTION_EXT_OPERATOR,
            TermComponents::new_multi_set(terms.into()),
        )
    }

    /// NAL-3 / 内涵交
    /// * 🚩【2024-04-21 13:39:28】使用统一的「无序不重复集合」构造组分
    pub fn new_intersection_int(terms: impl Into<Vec<Term>>) -> Self {
        Self::new(
            INTERSECTION_INT_OPERATOR,
            TermComponents::new_multi_set(terms.into()),
        )
    }

    /// NAL-3 / 外延差
    pub fn new_diff_ext(term1: Term, term2: Term) -> Self {
        Self::new(
            DIFFERENCE_EXT_OPERATOR,
            TermComponents::new_binary(term1, term2),
        )
    }

    /// NAL-3 / 内涵差
    pub fn new_diff_int(term1: Term, term2: Term) -> Self {
        Self::new(
            DIFFERENCE_INT_OPERATOR,
            TermComponents::new_binary(term1, term2),
        )
    }

    /// NAL-4 / 乘积
    pub fn new_product(terms: impl Into<Vec<Term>>) -> Self {
        Self::new(PRODUCT_OPERATOR, TermComponents::new_multi(terms.into()))
    }

    /// NAL-4 / 外延像
    /// * 📝占位符索引≠关系词项索引（in OpenNARS）
    ///   * ⚠️占位符索引=0 ⇒ 不被允许
    ///
    /// ! ⚠️【2024-06-16 16:50:23】现在传入的「词项列表」将附带「像占位符」词项
    pub fn new_image_ext(terms: impl Into<Vec<Term>>) -> Result<Self> {
        Ok(Self::new(
            IMAGE_EXT_OPERATOR,
            Self::_process_image_terms(terms)?,
        ))
    }

    /// NAL-4 / 内涵像
    /// * 📝占位符索引≠关系词项索引（in OpenNARS）
    ///   * ⚠️占位符索引=0 ⇒ 不被允许
    ///
    /// ! ⚠️【2024-06-16 16:50:23】现在传入的「词项列表」将附带「像占位符」词项
    pub fn new_image_int(terms: impl Into<Vec<Term>>) -> Result<Self> {
        Ok(Self::new(
            IMAGE_INT_OPERATOR,
            Self::_process_image_terms(terms)?,
        ))
    }

    /// 代码复用之工具函数：处理像占位符和词项列表
    /// * 🚩将词项列表转换为`Vec<Term>`
    /// * 🚩检查占位符索引范围
    /// * 🚩返回构造好的「词项组分」
    /// * ⚠️会返回错误
    ///
    /// ! ⚠️【2024-06-16 16:50:23】现在传入的「词项列表」将附带「像占位符」词项
    #[inline(always)]
    fn _process_image_terms(terms: impl Into<Vec<Term>>) -> Result<TermComponents> {
        // 转换词项列表
        let terms = terms.into();
        // 检索像占位符位置
        let i_placeholder = terms.iter().position(Term::is_placeholder);
        // 检查占位符索引范围
        match i_placeholder {
            Some(i_placeholder) => {
                if_return! {
                    i_placeholder == 0
                        => Err(anyhow::anyhow!("占位符不能压在「关系词项」的位置上"))
                    i_placeholder > terms.len()
                        => Err(anyhow::anyhow!("占位符索引超出范围"))
                }
            }
            None => return Err(anyhow::anyhow!("未在像的元素中找到占位符")),
        }
        // 构造 & 返回
        // * 🚩【2024-06-12 22:48:33】现在不再附带额外字段，统一使用一个枚举变种
        Ok(TermComponents::new_multi(terms))
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
        Self::new(NEGATION_OPERATOR, TermComponents::new_unary(term))
    }

    /// NAL-7 / 序列合取
    pub fn new_sequential_conjunction(terms: impl Into<Vec<Term>>) -> Self {
        Self::new(
            SEQUENTIAL_CONJUNCTION_OPERATOR,
            TermComponents::new_multi(terms.into()),
        )
    }

    /// NAL-7 / 平行合取
    /// * 🚩【2024-04-21 13:39:28】使用统一的「无序不重复集合」构造组分
    pub fn new_parallel_conjunction(terms: impl Into<Vec<Term>>) -> Self {
        Self::new(
            PARALLEL_CONJUNCTION_OPERATOR,
            TermComponents::new_multi_set(terms.into()),
        )
    }

    // 陈述 //

    /// NAL-1 / 继承
    pub fn new_inheritance(subject: Term, predicate: Term) -> Self {
        Self::new(
            INHERITANCE_RELATION,
            TermComponents::new_binary(subject, predicate),
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
            TermComponents::new_binary(subject, predicate),
        )
    }

    /// NAL-5 / 等价
    pub fn new_equivalence(subject: Term, predicate: Term) -> Self {
        Self::new(
            EQUIVALENCE_RELATION,
            TermComponents::new_binary_unordered(subject, predicate),
        )
    }

    /// NAL-7 / 预测性蕴含
    pub fn new_predicative_implication(subject: Term, predicate: Term) -> Self {
        Self::new(
            PREDICTIVE_IMPLICATION_RELATION,
            TermComponents::new_binary(subject, predicate),
        )
    }
}

impl TermComponents {
    /// 一元组分
    /// * 🚩【2024-06-12 22:43:34】现在封装「内部枚举变种」接口
    pub fn new_unary(term: Term) -> Self {
        Self::Compound(Box::new([term]))
    }

    /// 二元有序组分
    /// * 🚩【2024-06-12 22:43:34】现在封装「内部枚举变种」接口
    pub fn new_binary(term1: Term, term2: Term) -> Self {
        Self::Compound(Box::new([term1, term2]))
    }

    /// 二元无序组分
    /// * 🎯用于【双元素对称性】复合词项
    /// * ⚠️无法去重：元素数量固定为`2`
    /// * 📄相似、等价
    /// * 🚩使用「临时数组切片」实现（较为简洁）
    pub fn new_binary_unordered(term1: Term, term2: Term) -> Self {
        pipe! {
            // 排序
            manipulate!(
                [term1, term2]
                => .sort()
            )
            // 构造
            => Box::new
            => Self::Compound
        }
    }

    /// 多元有序组分
    pub fn new_multi(terms: Vec<Term>) -> Self {
        pipe! {
            terms
            // 转换
            => .into_boxed_slice()
            // 构造
            => Self::Compound
        }
    }

    /// 多元无序不重复组分
    /// * 🎯用于【无序不重复】的集合类组分
    /// * 📄外延集、内涵集
    /// * 📄外延交、内涵交
    pub fn new_multi_set(terms: Vec<Term>) -> Self {
        pipe! {
            manipulate!(
                terms
                // 重排 & 去重
                => .sort()
                => .dedup()
            )
            => .into_boxed_slice()
            => Self::Compound
        }
    }
}

/// [「词项」](Term)的快捷构造宏
#[macro_export]
macro_rules! term {
    // 单个词项（字符串）
    ($s:literal) => {
        $s.parse::<$crate::language::Term>()
    };
    // 单个词项，但unwrap
    (unwrap $s:expr) => {
        $s.parse::<$crate::language::Term>().unwrap()
    };
    // 单个词项，无需引号
    ($($t:tt)*) => {
        stringify!($($t)*).parse::<$crate::language::Term>()
    };
}

/// 单元测试
#[cfg(test)]
mod tests {
    use super::*;
    use crate::{ok, test_term as t, util::AResult};
    use nar_dev_utils::fail_tests;
    // ! ❌使用`test_term as t`避免`term`重名：即便不导入，也会ambiguous

    /// 测试/词项
    #[test]
    fn test_term() -> AResult {
        // 测试一个词项
        fn detect(term: &Term) {
            use TermComponents::*;
            match term.id_comp() {
                (WORD, Word(name)) => {
                    println!("word with {name:?}");
                }
                (IMAGE_EXT_OPERATOR, Compound(v)) => {
                    let i = v.iter().position(Term::is_placeholder).unwrap();
                    println!("ext_image '/' with {i}");
                    println!("<components>");
                    for term in v.iter() {
                        detect(term);
                    }
                    println!("</components>");
                }
                _ => println!("term {:?}: {}", term.identifier, term.format_name()),
            }
        }
        // 直接从内部构造函数中构造一个词项
        let im_ext = Term::new(
            IMAGE_EXT_OPERATOR,
            TermComponents::new_multi(vec![Term::new_word("word"), Term::new_placeholder()]),
        );
        detect(&im_ext);
        // 从「词法Narsese」中解析词项
        detect(&t!("<A --> B>"));
        detect(&t!("(--, [C, B, A, 0, 1, 2])"));
        detect(&t!(
            "{<B <-> A>, <D <=> C>, (&&, <A --> B>, <B --> C>), $i, #d, ?q}"
        ));
        detect(&t!("(/, _, A, B)"));
        detect(&t!("(/, A, _, B)"));
        detect(&t!("(/, A, B, _)"));
        detect(&t!(r"(\, _, A, B)"));
        detect(&t!(r"(\, A, _, B)"));
        detect(&t!(r"(\, A, B, _)"));
        // 返回成功
        ok!()
    }

    // 失败测试
    fail_tests! {
        组分数不对_二元_外延差1 t!(unwrap "(-, A)");
        组分数不对_二元_外延差3 t!(unwrap "(-, A, B, C)");
        组分数不对_一元_否定 t!(unwrap "(--, A, B)");
        空集_外延集 t!(unwrap "{}");
        空集_内涵集 t!(unwrap "[]");
        空集_外延像 t!(unwrap r"(/, _)");
        空集_内涵像 t!(unwrap r"(\, _)");
    }

    #[test]
    #[cfg(弃用_20240614000254_对后续变量命名等机制无用)]
    #[deprecated]
    fn from_var_clone() -> AResult {
        macro_once! {
            // * 🚩模式：词项字符串 ⇒ 预期词项字符串
            macro from_var_clone($($origin:literal x $new_name:expr => $expected:expr )*) {
                asserts! {$(
                    Term::from_var_clone(&t!($origin), $new_name) => t!($expected)
                    // 比对
                    // dbg!(&term);
                    // assert_eq!(term, t!($expected));
                )*}
            }
            // 原子词项
            "A" x "B" => "B"
            "$A" x "B" => "$B"
            "#A" x "B" => "#B"
            "?A" x "B" => "?B"
        }
        ok!()
    }
}
