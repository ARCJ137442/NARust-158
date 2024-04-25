//! 实现 / 构造

use super::*;
use anyhow::Result;
use nar_dev_utils::if_return;

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
    /// * 🎯仅在解析时临时出现
    /// * ⚠️【2024-04-25 09:45:51】不允许外部直接创建
    pub(super) fn new_placeholder() -> Self {
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
    /// * 📝占位符索引≠关系词项索引（in OpenNARS）
    ///   * ⚠️占位符索引=0 ⇒ 不被允许
    pub fn new_image_ext(i_placeholder: usize, terms: impl Into<Vec<Term>>) -> Result<Self> {
        Ok(Self::new(
            IMAGE_EXT_OPERATOR,
            Self::_process_image_terms(i_placeholder, terms)?,
        ))
    }

    /// NAL-4 / 内涵像
    /// * 📝占位符索引≠关系词项索引（in OpenNARS）
    ///   * ⚠️占位符索引=0 ⇒ 不被允许
    pub fn new_image_int(i_placeholder: usize, terms: impl Into<Vec<Term>>) -> Result<Self> {
        Ok(Self::new(
            IMAGE_INT_OPERATOR,
            Self::_process_image_terms(i_placeholder, terms)?,
        ))
    }

    /// 代码复用之工具函数：处理像占位符和词项列表
    /// * 🚩将词项列表转换为`Vec<Term>`
    /// * 🚩检查占位符索引范围
    /// * 🚩返回构造好的「词项组分」
    /// * ⚠️会返回错误
    #[inline(always)]
    fn _process_image_terms(
        i_placeholder: usize,
        terms: impl Into<Vec<Term>>,
    ) -> Result<TermComponents> {
        // 转换词项列表
        let terms = terms.into();
        // 检查占位符索引范围
        if_return! {
            i_placeholder == 0
                => Err(anyhow::anyhow!("占位符不能压在「关系词项」的位置上"))
            i_placeholder > terms.len()
                => Err(anyhow::anyhow!("占位符索引超出范围"))
        }
        // 构造 & 返回
        Ok(TermComponents::MultiIndexed(i_placeholder, terms))
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

/// 单元测试
#[cfg(test)]
mod tests {
    use super::super::*;
    use super::*;
    use crate::test_term as term;
    use nar_dev_utils::fail_tests;

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
        // 直接从内部构造函数中构造一个词项
        let im_ext = Term::new(
            IMAGE_EXT_OPERATOR,
            TermComponents::MultiIndexed(1, vec![Term::new_word("word")]),
        );
        detect(&im_ext);
        // 从「词法Narsese」中解析词项
        detect(&term!("<A --> B>"));
        detect(&term!("(--, [C, B, A, 0, 1, 2])"));
        detect(&term!(
            "{<B <-> A>, <D <=> C>, (&&, <A --> B>, <B --> C>), $i, #d, ?q}"
        ));
        detect(&term!("(/, _, A, B)"));
        detect(&term!("(/, A, _, B)"));
        detect(&term!("(/, A, B, _)"));
        detect(&term!(r"(\, _, A, B)"));
        detect(&term!(r"(\, A, _, B)"));
        detect(&term!(r"(\, A, B, _)"));
        // 返回成功
        Ok(())
    }

    // 失败测试
    fail_tests! {
        组分数不对_二元_外延差1 term!(unwrap "(-, A)");
        组分数不对_二元_外延差3 term!(unwrap "(-, A, B, C)");
        组分数不对_一元_否定 term!(unwrap "(--, A, B)");
        空集_外延集 term!(unwrap "{}");
        空集_内涵集 term!(unwrap "[]");
        空集_外延像 term!(unwrap r"(/, _)");
        空集_内涵像 term!(unwrap r"(\, _)");
    }
}
