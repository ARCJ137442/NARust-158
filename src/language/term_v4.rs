//! 表征NARust 158所用的「词项」
//! * 📄功能上参照OpenNARS
//! * 🚩实现方式上更Rusty，同时亦有其它妥协/加强
//! * 🚩【2024-04-20 22:19:40】目前采用「自制枚举」的方法实现自己的一套Narsese
//! * ⚠️但因此难以复用「枚举Narsese」所自动实现的几乎一切结构方法
//!   * 📌或者「勉强可以」并且还有许多性能损失

use std::{cmp::Ordering, collections::HashSet};

use anyhow::Result;
use narsese::{
    api::{GetCategory, TermCategory},
    conversion::string::impl_enum::format_instances::FORMAT_ASCII,
    enum_narsese::Term as EnumTerm,
};

/// 作为「专用枚举」实现的「词项」类型
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
#[derive(Debug, Clone, Hash, PartialEq, Eq)]
pub enum Term {
    // 原子词项
    Word(String),
    VarI(String),
    VarD(String),
    VarQ(String),
    // 复合词项
    // ! 对于「无序者」会在构造时自动排序
    SetExt(Vec<Term>),
    SetInt(Vec<Term>),
    IntersectExt(Vec<Term>),
    IntersectInt(Vec<Term>),
    DiffExt(Box<(Term, Term)>),
    DiffInt(Box<(Term, Term)>),
    Product(Vec<Term>),
    ImageExt(usize, Vec<Term>),
    ImageInt(usize, Vec<Term>),
    Conjunction(Vec<Term>),
    Disjunction(Vec<Term>),
    Negation(Box<Term>),
    // 陈述
    Inheritance(Box<(Term, Term)>),
    Similarity(Box<(Term, Term)>),
    Implication(Box<(Term, Term)>),
    Equivalence(Box<(Term, Term)>),
}

// 后续实现 //

/// 与「枚举Narsese」的转换
impl Term {
    /// 用于处理「词项集合」
    /// * 🚩自动排序
    fn set_from_hash_set(set: &HashSet<EnumTerm>) -> Result<Vec<Self>> {
        let mut v = vec![];
        for term in set {
            v.push(Self::try_from(term)?);
        }
        v.sort();
        Ok(v)
    }

    /// 用于处理「有序词项对」
    fn pair_from_terms(t1: &EnumTerm, t2: &EnumTerm) -> Result<Box<(Self, Self)>> {
        let t1 = Self::try_from(t1)?;
        let t2 = Self::try_from(t2)?;
        let pair = (t1, t2);
        Ok(Box::new(pair))
    }

    /// 用于处理「无序词项对」
    fn unordered_pair_from_terms(t1: &EnumTerm, t2: &EnumTerm) -> Result<Box<(Self, Self)>> {
        // 构造 & 排序 | 为了和上边[`vec_from_terms`]保持一致`]
        let t1 = Self::try_from(t1)?;
        let t2 = Self::try_from(t2)?;
        let mut pair = [t1, t2];
        pair.sort();
        // 模式转换
        let pair = match pair {
            [t1, t2] => (t1, t2),
        };
        Ok(Box::new(pair))
    }

    /// 用于处理「词项列表」
    fn vec_from_terms(vec: &[EnumTerm]) -> Result<Vec<Self>> {
        let mut v = vec![];
        for term in vec {
            v.push(Self::try_from(term)?);
        }
        Ok(v)
    }
}

impl TryFrom<&EnumTerm> for Term {
    type Error = anyhow::Error;
    fn try_from(enum_term: &EnumTerm) -> Result<Self> {
        use EnumTerm::*;
        let term = match enum_term {
            Word(name) => Self::Word(name.clone()),
            VariableIndependent(name) => Self::VarI(name.clone()),
            VariableDependent(name) => Self::VarD(name.clone()),
            VariableQuery(name) => Self::VarQ(name.clone()),
            SetExtension(s) => Self::SetExt(Self::set_from_hash_set(s)?),
            SetIntension(s) => Self::SetInt(Self::set_from_hash_set(s)?),
            IntersectionExtension(s) => Self::IntersectExt(Self::set_from_hash_set(s)?),
            IntersectionIntension(s) => Self::IntersectInt(Self::set_from_hash_set(s)?),
            DifferenceExtension(t1, t2) => Self::DiffExt(Self::pair_from_terms(t1, t2)?),
            DifferenceIntension(t1, t2) => Self::DiffInt(Self::pair_from_terms(t1, t2)?),
            Product(v) => Self::Product(Self::vec_from_terms(v)?),
            ImageExtension(i, v) => Self::ImageExt(*i, Self::vec_from_terms(v)?),
            ImageIntension(i, v) => Self::ImageInt(*i, Self::vec_from_terms(v)?),
            Conjunction(s) => Self::Conjunction(Self::set_from_hash_set(s)?),
            Disjunction(s) => Self::Disjunction(Self::set_from_hash_set(s)?),
            Negation(t) => Self::Negation(Box::new(Self::try_from(&**t)?)),
            Inheritance(t1, t2) => Self::Inheritance(Self::pair_from_terms(t1, t2)?),
            Similarity(t1, t2) => Self::Similarity(Self::unordered_pair_from_terms(t1, t2)?),
            Implication(t1, t2) => Self::Implication(Self::pair_from_terms(t1, t2)?),
            Equivalence(t1, t2) => Self::Equivalence(Self::unordered_pair_from_terms(t1, t2)?),
            _ => return Err(anyhow::anyhow!("不支持的词项类型 from {enum_term:?}")),
        };
        Ok(term)
    }
}

impl From<&Term> for EnumTerm {
    fn from(value: &Term) -> Self {
        use EnumTerm::*;
        match value {
            Term::Word(name) => Word(name.clone()),
            Term::VarI(name) => VariableIndependent(name.clone()),
            Term::VarD(name) => VariableDependent(name.clone()),
            Term::VarQ(name) => VariableQuery(name.clone()),
            Term::SetExt(s) => SetExtension(HashSet::from_iter(s.iter().map(Self::from))),
            Term::SetInt(s) => SetIntension(HashSet::from_iter(s.iter().map(Self::from))),
            Term::IntersectExt(s) => {
                IntersectionExtension(HashSet::from_iter(s.iter().map(Self::from)))
            }
            Term::IntersectInt(s) => {
                IntersectionExtension(HashSet::from_iter(s.iter().map(Self::from)))
            }
            Term::DiffExt(b) => {
                DifferenceExtension(Box::new((&b.0).into()), Box::new((&b.1).into()))
            }
            Term::DiffInt(b) => {
                DifferenceExtension(Box::new((&b.0).into()), Box::new((&b.1).into()))
            }
            Term::Product(v) => Product(v.iter().map(Self::from).collect::<Vec<_>>()),
            Term::ImageExt(i, v) => {
                ImageExtension(*i, v.iter().map(Self::from).collect::<Vec<_>>())
            }
            Term::ImageInt(i, v) => {
                ImageExtension(*i, v.iter().map(Self::from).collect::<Vec<_>>())
            }
            Term::Conjunction(s) => Conjunction(HashSet::from_iter(s.iter().map(Self::from))),
            Term::Disjunction(s) => Disjunction(HashSet::from_iter(s.iter().map(Self::from))),
            Term::Negation(t) => Negation(Box::new((&**t).into())),
            Term::Inheritance(b) => Inheritance(Box::new((&b.0).into()), Box::new((&b.1).into())),
            Term::Similarity(b) => Similarity(Box::new((&b.0).into()), Box::new((&b.1).into())),
            Term::Implication(b) => Implication(Box::new((&b.0).into()), Box::new((&b.1).into())),
            Term::Equivalence(b) => Equivalence(Box::new((&b.0).into()), Box::new((&b.1).into())),
        }
    }
}

// 与「属性」相关

impl Term {
    /// 词项的「名称」
    /// * 🎯对应OpenNARS中的「name」属性
    /// * 🎯对应OpenNARS中的「toString」方法
    pub fn name(&self) -> String {
        FORMAT_ASCII.format(&EnumTerm::from(self))
    }

    /// 词项「是否为变量」
    /// * 对应OpenNARS中的`instanceof Variable`
    pub fn is_variable_term(&self) -> bool {
        use Term::*;
        matches!(self, VarI(..) | VarD(..) | VarQ(..))
    }
}

/// 词项类别（用于比对）
impl GetCategory for Term {
    fn get_category(&self) -> TermCategory {
        use Term::*;
        use TermCategory::*;
        match self {
            Word(..) | VarI(..) | VarD(..) | VarQ(..) => Atom,
            SetExt(..) | SetInt(..) | IntersectExt(..) | IntersectInt(..) | DiffExt(..)
            | DiffInt(..) | Product(..) | ImageExt(..) | ImageInt(..) | Conjunction(..)
            | Disjunction(..) | Negation(..) => Compound,
            Inheritance(..) | Similarity(..) | Implication(..) | Equivalence(..) => Statement,
        }
    }
}

impl PartialOrd for Term {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

/// 实现「排序」
/// `compareTo` => [`Ord`]
/// * 🎯实现「变量 < 原子 < 复合（陈述）」
impl Ord for Term {
    fn cmp(&self, other: &Self) -> Ordering {
        // 变量 < 原子
        use Ordering::*;
        if self.is_variable_term() && !other.is_variable_term() {
            return Less;
        } else if !self.is_variable_term() && other.is_variable_term() {
            return Greater;
        }
        // 原子 < 复合（陈述）
        use TermCategory::*;
        match (self.get_category(), other.get_category()) {
            (Atom, Compound | Statement) => return Less,
            (Compound | Statement, Atom) => return Greater,
            _ => (),
        }
        // 最后比名称
        self.name().cmp(&other.name())
    }
}
