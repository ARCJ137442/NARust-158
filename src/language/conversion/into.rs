//! 词项→其它类型

use super::super::base::*;
use crate::symbols::*;
use nar_dev_utils::*;
use narsese::{
    api::{FormatTo, GetCapacity},
    lexical::Term as TermLexical,
};

/// 词项⇒字符串
/// * 🎯用于更好地打印「词项」名称
/// * 🎯用于从「词法Narsese」中解析
///   * 考虑「变量语义」
impl Term {
    /// 格式化名称
    /// * 🚩以方便打印的「内部方言语法」呈现Narsese
    ///   * 📌括号全用 圆括号
    ///   * 📌无逗号分隔符
    pub fn format_name(&self) -> String {
        // 格式化所用常量
        const OPENER: &str = "(";
        const CLOSER: &str = ")";
        const SEPARATOR: &str = " ";

        use narsese::api::TermCapacity::*;
        use TermComponents::*;
        let id = self.identifier();
        match self.components() {
            // 空组分
            Empty => id.to_string(),
            // 名称 | 原子词项
            Word(name) => format!("{id}{name}"),
            // 名称 | 变量词项
            Variable(n) => format!("{id}{n}"),
            Compound(terms) => {
                match self.get_capacity() {
                    // 一元
                    Unary => {
                        // 📄 "(-- A)"
                        manipulate!(
                            String::new()
                            => {+= OPENER}#
                            => {+= id}#
                            => {+= SEPARATOR}#
                            => {+= &terms[0].format_name()}#
                            => {+= CLOSER}#
                        )
                    }
                    // 二元
                    BinaryVec | BinarySet => {
                        // 📄 "(A --> B)"
                        manipulate!(
                            String::new()
                            => {+= OPENER}#
                            => {+= &terms[0].format_name()}#
                            => {+= SEPARATOR}#
                            => {+= id}#
                            => {+= SEPARATOR}#
                            => {+= &terms[1].format_name()}#
                            => {+= CLOSER}#
                        )
                    }
                    // 多元
                    Vec | Set => {
                        let mut s = id.to_string() + OPENER;
                        let mut terms = terms.iter();
                        if let Some(t) = terms.next() {
                            s += &t.format_name();
                        }
                        for t in terms {
                            s += SEPARATOR;
                            s += &t.format_name();
                        }
                        s + CLOSER
                    }
                    Atom => unreachable!("复合词项只可能是「一元」「二元」或「多元」"),
                }
            }
        }
    }

    /// 从「内部Narsese」转换为「词法Narsese」
    /// * 🚩基本无损转换（无需考虑失败情况）
    pub fn to_lexical(&self) -> TermLexical {
        use TermComponents::*;
        type LTerm = TermLexical;
        let (id, comp) = self.id_comp();
        match (id, comp) {
            // 专用 / 集合词项 | 默认已排序
            (SET_EXT_OPERATOR, Compound(v)) => {
                let v = v.iter().map(Self::to_lexical).collect::<Vec<_>>();
                LTerm::new_set(SET_EXT_OPENER, v, SET_EXT_CLOSER)
            }
            (SET_INT_OPERATOR, Compound(v)) => {
                let v = v.iter().map(Self::to_lexical).collect::<Vec<_>>();
                LTerm::new_set(SET_INT_OPENER, v, SET_INT_CLOSER)
            }
            //  陈述
            (
                INHERITANCE_RELATION | SIMILARITY_RELATION | IMPLICATION_RELATION
                | EQUIVALENCE_RELATION,
                Compound(terms),
            ) if terms.len() == 2 => {
                LTerm::new_statement(id, (&terms[0]).into(), (&terms[1]).into())
            }
            // 通用 / 空：仅前缀
            (_, Empty) => LTerm::new_atom(id, ""),
            // 通用 / 具名：前缀+词项名
            (_, Word(name)) => LTerm::new_atom(id, name),
            // 通用 / 变量：前缀+变量编号
            (_, Variable(num)) => LTerm::new_atom(id, num.to_string()),
            // 通用 / 多元
            (_, Compound(terms)) => {
                LTerm::new_compound(id, terms.iter().map(Self::to_lexical).collect())
            }
        }
    }

    /// 转换为显示呈现上的ASCII格式
    /// * 📌对标OpenNARS的默认呈现
    /// * ⚠️【2024-07-02 00:52:54】目前需要「词法Narsese」作为中间格式，可能会有性能损失
    #[doc(alias = "to_display_ascii")]
    pub fn format_ascii(&self) -> String {
        use narsese::conversion::string::impl_lexical::format_instances::FORMAT_ASCII;
        self.to_lexical().format_to(&FORMAT_ASCII)
    }
}

// * 🚩此处的「变量词项」一开始就应该是个数值，从「具名变量」变为「数字变量」
/// 词项⇒词法Narsese
impl From<&Term> for TermLexical {
    fn from(term: &Term) -> Self {
        term.to_lexical()
    }
}

impl From<TermComponents> for Vec<Term> {
    /// 将「词项组分」转换为「可变数组<词项>」
    /// * 🚩原子词项⇒空数组
    /// * 🚩复合词项⇒其内所有词项构成的数组
    fn from(value: TermComponents) -> Self {
        use TermComponents::*;
        match value {
            Empty | Word(..) | Variable(..) => vec![],
            Compound(terms) => terms.into(),
        }
    }
}

impl From<TermComponents> for Box<[Term]> {
    /// 将「词项组分」转换为「定长数组<词项>」
    /// * 🚩原子词项⇒空数组
    /// * 🚩复合词项⇒其内所有词项构成的数组
    /// * ℹ️与上述对[`Vec`]的转换不同：此处直接使用`Box::new([])`构造空数组
    fn from(value: TermComponents) -> Self {
        use TermComponents::*;
        match value {
            Empty | Word(..) | Variable(..) => Box::new([]),
            Compound(terms) => terms,
        }
    }
}
