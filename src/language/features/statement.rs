//! 📄OpenNARS `nars.language.Statement`
//! * 📌NAL底层的「陈述」逻辑，对应`Statement`及其所有子类
//! * ⚠️不包括与记忆区有关的`make`系列方法
//! * ⚠️不包括只和语法解析有关的`isRelation`、`makeName`、`makeStatementName`等方法
//! * ✅【2024-06-14 14:53:10】基本完成方法复刻
//!
//! # 方法列表
//! 🕒最后更新：【2024-06-14 14:53:18】
//!
//! * `Statement`
//!   * `invalidStatement` => `is_invalid_statement`
//!   * `invalidReflexive`
//!   * `invalidPair`
//!   * `invalid`
//!   * `getSubject`
//!   * `getPredicate`
//!
//! # 📄OpenNARS
//!
//! A statement is a compound term, consisting of a subject, a predicate, and a relation symbol in between.
//! It can be of either first-order or higher-order.

use super::compound_term::CompoundTermRef;
use crate::language::*;
use crate::symbols::*;
use nar_dev_utils::{if_return, matches_or};
use std::{
    fmt::{Display, Formatter},
    ops::{Deref, DerefMut},
};

impl Term {
    /// 🆕用于判断是否为「陈述词项」
    /// * 📄OpenNARS `instanceof Statement` 逻辑
    #[inline(always)]
    pub fn instanceof_statement(&self) -> bool {
        Self::is_statement_identifier(self.identifier())
    }

    /// 🆕抽象出来的「标识符（对应的词项类型）是否『可交换』」
    /// * 🎯同时用于「词项属性」与「词项转换」
    ///   * 📄参见[`super::_dialect`]中的`reform_term`函数
    pub fn is_statement_identifier(identifier: &str) -> bool {
        matches!(
            identifier,
            // 四大主要系词 + 时间系词
            INHERITANCE_RELATION
                | SIMILARITY_RELATION
                | IMPLICATION_RELATION
                | EQUIVALENCE_RELATION
                | TEMPORAL_IMPLICATION_RELATION
                // ↓下边都是派生系词（实际上不会出现，OpenNARS也一样）
                | INSTANCE_RELATION
                | PROPERTY_RELATION
                | INSTANCE_PROPERTY_RELATION
        )
    }

    /// 🆕用于判断是否为「继承」
    /// * 📄OpenNARS`instanceof Inheritance`逻辑
    /// * 📝OpenNARS中「继承」与「实例」「属性」「实例属性」没有继承关系
    /// * 🎯[`crate::inference::RuleTables`]推理规则分派
    #[inline(always)]
    pub fn instanceof_inheritance(&self) -> bool {
        self.identifier() == INHERITANCE_RELATION
    }

    /// 🆕用于判断是否为「相似」
    /// * 📄OpenNARS`instanceof Similarity`逻辑
    /// * 🎯[`crate::inference::RuleTables`]推理规则分派
    #[inline(always)]
    pub fn instanceof_similarity(&self) -> bool {
        self.identifier() == SIMILARITY_RELATION
    }

    /// 🆕用于判断是否为「蕴含」
    /// * 📄OpenNARS`instanceof Implication`逻辑
    /// * 🎯[`crate::inference::RuleTables`]推理规则分派
    #[inline(always)]
    pub fn instanceof_implication(&self) -> bool {
        self.identifier() == IMPLICATION_RELATION
    }

    /// 🆕用于判断是否为「等价」
    /// * 📄OpenNARS`instanceof Equivalence`逻辑
    /// * 🎯[`crate::inference::RuleTables`]推理规则分派
    #[inline(always)]
    pub fn instanceof_equivalence(&self) -> bool {
        self.identifier() == EQUIVALENCE_RELATION
    }

    /// 🆕用于判断是否为「时序蕴含」
    /// * 📄遵从ONA逻辑
    #[inline(always)]
    pub fn instanceof_temporal_implication(&self) -> bool {
        self.identifier() == TEMPORAL_IMPLICATION_RELATION
    }

    /// 🆕判断一个词项是否为「陈述词项」
    /// * 🚩判断其「内部元素」的个数是否为2，并且要判断其标识符
    /// * 🚩【2024-09-07 14:59:00】现在采用更严格的条件——需要判断是否为「陈述系词」
    pub fn is_statement(&self) -> bool {
        self.instanceof_statement()
            && matches!(self.components(), TermComponents::Compound(terms) if terms.len() == 2)
    }

    /// 🆕将一个复合词项转换为「陈述词项」（不可变引用）
    /// * 🚩转换为Option
    /// * 🚩【2024-09-07 14:59:00】现在采用更严格的条件——需要判断是否为「陈述系词」
    #[must_use]
    pub fn as_statement(&self) -> Option<StatementRef> {
        matches_or!(
            ?self.components(),
            TermComponents::Compound(ref terms)
            if self.instanceof_statement() && terms.len() == 2
            => StatementRef {
                statement: self,
                subject: &terms[0],
                predicate: &terms[1],
            }
        )
    }

    /// 🆕用于判断词项是否为「指定类型的复合词项」，并尝试返回「复合词项」的引用信息
    /// * 📌包括陈述
    /// * 🚩模式匹配后返回一个[`Option`]，只在其为「符合指定类型的词项」时为[`Some`]
    /// * 🚩返回不可变引用
    #[must_use]
    pub fn as_statement_type(&self, statement_class: impl AsRef<str>) -> Option<StatementRef> {
        matches_or! {
            ?self.as_statement(),
            Some(statement)
                // * 🚩标识符相等
                if statement_class.as_ref() == self.identifier()
                // * 🚩内部（类型相等）的复合词项
                => statement
        }
    }

    /// 🆕将一个复合词项转换为「陈述词项」（可变引用）
    /// * 🚩转换为Option
    #[must_use]
    pub fn as_statement_mut(&mut self) -> Option<StatementRefMut> {
        matches_or!(
            ?self.components_mut(),
            TermComponents::Compound(ref mut terms) if terms.len() == 2
            => StatementRefMut {
                // * 🚩均转换为裸指针
                subject: &mut terms[0] as *mut Term,
                predicate: &mut terms[1] as *mut Term,
                statement: self,
            }
        )
    }

    /// 🆕用于判断词项是否为「陈述」并解包其中的主项和谓项
    /// * 🚩模式匹配后返回一个[`Option`]，只在其为「符合指定类型的词项」时为[`Some`]
    /// * 🚩返回内部所有元素的所有权
    #[must_use]
    pub fn unwrap_statement_components(self) -> Option<[Term; 2]> {
        matches_or! {
            ?self.unwrap_compound_components(),
            // * 🚩匹配到（语句所作为的）复合词项，同时长度合规
            Some(terms) if terms.len() == 2
            // * 🚩返回内容
            => {
                // ? 💭后续或许能提取出一个统一的逻辑
                let mut terms = terms.into_vec();
                let predicate = terms.pop().expect("已经假定了长度为2");
                let subject = terms.pop().expect("已经假定了长度为2");
                [subject, predicate]
            }
        }
    }

    /// 🆕用于判断词项是否为「陈述」并解包其中的主项、系词和谓项
    /// * 🚩模式匹配后返回一个[`Option`]，只在其为「符合指定类型的词项」时为[`Some`]
    /// * 🚩返回标识符与内部所有元素的所有权
    #[must_use]
    pub fn unwrap_statement_id_components(self) -> Option<(Term, String, Term)> {
        matches_or! {
            ?self.unwrap_compound_id_components(),
            // * 🚩匹配到（语句所作为的）复合词项，同时长度合规
            Some((copula, terms)) if terms.len() == 2
            // * 🚩返回内容
            => {
                // ? 💭后续或许能提取出一个统一的逻辑
                let mut terms = terms.into_vec();
                let predicate = terms.pop().expect("已经假定了长度为2");
                let subject = terms.pop().expect("已经假定了长度为2");
                (subject, copula, predicate)
            }
        }
    }

    /// 🆕用于判断词项是否为「指定类型的陈述」，并解包其中的主项和谓项
    /// * 🚩模式匹配后返回一个[`Option`]，只在其为「符合指定类型的词项」时为[`Some`]
    /// * 🚩返回内部所有元素的所有权
    #[must_use]
    pub fn unwrap_statement_type_components(
        self,
        statement_class: impl AsRef<str>,
    ) -> Option<[Term; 2]> {
        matches_or! {
            ?self.unwrap_compound_type_components(statement_class),
            // * 🚩匹配到（语句所作为的）复合词项，同时长度合规
            Some(terms) if terms.len() == 2
            // * 🚩返回内容
            => {
                // ? 💭后续或许能提取出一个统一的逻辑
                let mut terms = terms.into_vec();
                let predicate = terms.pop().expect("已经假定了长度为2");
                let subject = terms.pop().expect("已经假定了长度为2");
                [subject, predicate]
            }
        }
    }
}

/// 为「复合词项」添加「转换到陈述」的方法
/// * 📌依据：陈述 ⊂ 复合词项
impl<'s> CompoundTermRef<'s> {
    /// 🆕判断一个复合词项是否为「陈述词项」
    /// * 🚩判断其「内部元素」的个数是否为2
    /// * 📌与[`Term::is_statement`]一致
    pub fn is_statement(&self) -> bool {
        self.components.len() == 2
    }

    /// 🆕将一个复合词项转换为「陈述词项」（不可变引用）
    /// * 🚩转换为Option
    /// * 📌与[`Term::as_statement`]一致
    pub fn as_statement(self) -> Option<StatementRef<'s>> {
        matches_or!(
            ?self.components,
            [ref subject, ref predicate]
            => StatementRef {
                statement: self.inner,
                subject,
                predicate,
            }
        )
    }

    // ! ❌【2024-06-14 14:47:26】没必要添加一个额外的`unchecked`方法：可以使用`unwrap`现场解包
}

/// 为「复合词项」添加「转换到陈述」的方法（可变引用）
/// * 📌依据：陈述 ⊂ 复合词项
impl CompoundTermRefMut<'_> {
    /// 🆕将一个复合词项转换为「陈述词项」（可变引用）
    /// * 🚩转换为Option
    /// * 📌与[`Term::as_statement`]一致
    pub fn as_statement(&mut self) -> Option<StatementRef> {
        matches_or!(
            // * 📝此处必须内联`self.components()`，以告诉借用检查器「并非使用整个结构」
            // ! SAFETY: 此处保证对整体（整个复合词项）拥有引用
            ? unsafe { &mut *self.components },
            [ref mut subject, ref mut predicate]
            => StatementRef {
                statement: self.inner,
                subject,
                predicate,
            }
        )
    }

    // ! ❌【2024-06-14 14:47:26】没必要添加一个额外的`unchecked`方法：可以使用`unwrap`现场解包
}

/// 🆕作为「陈述引用」的词项类型
/// * 🎯在程序类型层面表示一个「陈述」（不可变引用）
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct StatementRef<'a> {
    /// 陈述词项本身
    pub statement: &'a Term,
    /// 陈述词项的主项
    pub subject: &'a Term,
    /// 陈述词项的谓项
    pub predicate: &'a Term,
}

impl<'s> StatementRef<'s> {
    /// 📄OpenNARS `getSubject`
    ///
    /// # 📄OpenNARS
    ///
    /// 🈚
    pub fn subject(&self) -> &'s Term {
        self.subject
    }

    /// 📄OpenNARS `getPredicate`
    ///
    /// # 📄OpenNARS
    ///
    /// 🈚
    pub fn predicate(&self) -> &'s Term {
        self.predicate
    }

    /// 🆕主项-谓项 二元数组
    pub fn sub_pre(&self) -> [&'s Term; 2] {
        [self.subject, self.predicate]
    }

    /// 📄OpenNARS `invalidStatement`
    /// * ⚠️必须是「陈述」才能调用
    /// * 🎯检查「无效陈述」
    /// * 🎯基于AIKR，避免定义无用、冗余的陈述
    ///   * 📄如「永远成立」的「重言式」tautology
    /// * 📌无效案例：
    ///   * `<A --> A>`
    ///   * `<A --> [A]>`
    ///   * `<[A] --> A>`
    ///   * `<<A --> B> ==> <B --> A>>`
    ///
    /// # 📄OpenNARS
    ///
    /// Check the validity of a potential Statement. [To be refined]
    pub fn invalid_statement(subject: &Term, predicate: &Term) -> bool {
        if_return! {
            // 重言式⇒无效
            subject == predicate => true
            //自反性检查（双向）
            Self::invalid_reflexive(subject, predicate) => true
            Self::invalid_reflexive(predicate, subject) => true
        }
        // 都是陈述⇒进一步检查
        matches_or! {
            (subject.as_statement(), predicate.as_statement()),
            // 获取各自的主词、谓词，并检查是否相等
            // ! 禁止如下格式： <<A --> B> ==> <B --> A>>
            // * 📄ERR: !!! INVALID INPUT: parseTerm: <<A --> B> ==> <B --> A>> --- invalid statement
            // ? 💭【2024-04-24 15:04:44】目前尚未明确含义，可能是防止「重复推导」
            /* 📄OpenNARS源码：
            if ((subject instanceof Statement) && (predicate instanceof Statement)) {
                Statement s1 = (Statement) subject;
                Statement s2 = (Statement) predicate;
                Term t11 = s1.getSubject();
                Term t12 = s1.getPredicate();
                Term t21 = s2.getSubject();
                Term t22 = s2.getPredicate();
                if (t11.equals(t22) && t12.equals(t21)) {
                    return true;
                }
            } */
            (
                Some(StatementRef { subject:ss, predicate:sp,.. }),
                Some(StatementRef { subject:ps, predicate:pp,.. })
            ) if ss == pp && sp == ps => return  true,
            () // 无效案例⇒继续检查
        }
        // 检查完毕⇒否
        false
    }

    /// 📄OpenNARS `invalidReflexive`
    /// * 🚩主词项是「非像复合词项」并且包括另一词项
    ///   * 📄`<A <-> {A}>`
    ///   * 📄`<A ==> (*, B, C, A)>`
    ///
    /// # 📄OpenNARS
    ///
    /// Check if one term is identical to or included in another one, except in a reflexive relation
    pub fn invalid_reflexive(may_container: &Term, may_component: &Term) -> bool {
        /* 📄OpenNARS源码：
        if (!(t1 instanceof CompoundTerm)) {
            return false;
        }
        CompoundTerm com = (CompoundTerm) t1;
        if ((com instanceof ImageExt) || (com instanceof ImageInt)) {
            return false;
        }
        return com.containComponent(t2);
        */
        /* 📝原样转译的Rust代码：
        if_return! {
            !container.instanceof_compound() => false
            container.instanceof_image() => false
        }
        container.contain_component(maybe_component)
        */
        // 筛查词项类型：复合词项
        // ! 仅在复合词项时继续检查
        if let Some(compound) = may_container.as_compound() {
            // 筛查词项类型
            if_return! {
                compound.inner.instanceof_image() => false
            }
            // 若包含词项，则为「无效」
            return compound.contain_component(may_component);
        }
        // 非复合词项⇒通过
        false
    }

    /// 📄OpenNARS `invalidPair`
    /// * 📝总体逻辑：是否「一边包含独立变量，而另一边不包含」
    ///   * 💭可能是要「避免自由变量」
    /// * 🚩两边「包含独立变量」的情况不一致
    ///
    /// # 📄OpenNARS
    ///
    /// 🈚
    pub fn invalid_pair(subject: &Term, predicate: &Term) -> bool {
        /* 📄OpenNARS源码：
        if (Variable.containVarI(s1) && !Variable.containVarI(s2)) {
            return true;
        } else if (!Variable.containVarI(s1) && Variable.containVarI(s2)) {
            return true;
        }
        return false; */
        subject.contain_var_i() != predicate.contain_var_i()
    }

    /// 📄OpenNARS `invalid`
    ///
    /// # 📄OpenNARS
    ///
    /// 🈚
    pub fn invalid(&self) -> bool {
        Self::invalid_statement(self.subject(), self.predicate())
    }

    /// 🆕作为「条件句」使用
    /// * 🎯用于形如`<(&&, A, B) ==> C>`~~或`<(&&, A, B) <=> C>`~~的Narsese词项
    ///   * ~~📌同时兼容`<S <=> (&&, A, B)>`，即「合取不一定在第一个」~~
    ///   * ✨不仅可以判别，还可解包出其中的元素
    /// * 🚩返回`(陈述自身, 第一个找到的合取词项引用, 这个合取词项所在位置索引)`
    ///
    /// ! ❌【2024-07-05 17:04:02】不再考虑支持「等价」陈述的词项链转换，同时也不再将「等价陈述」视作「条件句」
    ///   * 📌【2024-07-05 17:05:48】目前认知：「等价」陈述完全可以「先转换为蕴含，再参与条件推理」
    ///
    /// ## 📄OpenNARS 参考代码
    ///
    /// ```java
    /// if (taskContent instanceof Equivalence)
    ///     throw new Error("【2024-07-05 17:03:18】简化代码：早已去掉「等价」系词的「复合条件」词项链！");
    /// // ! ❌【2024-07-05 17:04:02】不再考虑支持「等价」陈述的词项链转换
    /// final int conditionIndex = indices[0];
    /// final Term contentCondition = taskContent.componentAt(conditionIndex);
    /// // * 🚩判断「条件句」
    /// // * 选取的「条件项」是「合取」
    /// final boolean conditionCondition = contentCondition instanceof Conjunction;
    /// // * 整体是「等价」或「合取在前头的『蕴含』」
    /// final boolean conditionWhole = (taskContent instanceof Implication && conditionIndex == 0)
    ///         || taskContent instanceof Equivalence;
    /// if (conditionSubject && conditionWhole) {
    ///     /* ... */
    /// }
    /// ```
    pub fn as_conditional(self) -> Option<(StatementRef<'s>, CompoundTermRef<'s>)> {
        // // * 🚩提取其中的继承项
        // let subject = self.subject;
        // let predicate = self.subject;

        // // * 🚩判断「条件句」
        // match self.identifier() {
        //     // * 主项是「合取」的「蕴含」
        //     IMPLICATION_RELATION => {
        //         let subject = subject.as_compound_type(CONJUNCTION_OPERATOR)?;
        //         Some((self, subject, 0))
        //     }
        //     // * 【任一处含有合取】的「等价」
        //     EQUIVALENCE_RELATION => {
        //         // * 🚩优先判断并提取主项
        //         if let Some(subject) = subject.as_compound_type(CONJUNCTION_OPERATOR) {
        //             return Some((self, subject, 0));
        //         }
        //         if let Some(predicate) = predicate.as_compound_type(CONJUNCTION_OPERATOR) {
        //             return Some((self, predicate, 1));
        //         }
        //         None
        //     }
        //     // * 其它⇒空
        //     _ => None,
        // }

        // * 🚩蕴含 | 【2024-07-05 17:08:34】现在只判断「蕴含」陈述
        if !self.instanceof_implication() {
            return None;
        }
        // * 🚩主项是合取
        let subject_conjunction = self.subject.as_compound_type(CONJUNCTION_OPERATOR)?;
        // * 🚩返回
        Some((self, subject_conjunction))
    }

    /// 🆕作为「操作句」使用
    /// * 🎯用于形如`<(*, {SELF}, argument) --> ^operator>`的词项
    /// * 🚩返回值：(自身, 乘积词项, 乘积首个参数外延集, 操作名)
    pub fn as_operation(
        self,
    ) -> Option<(
        StatementRef<'s>,
        CompoundTermRef<'s>,
        CompoundTermRef<'s>,
        &'s str,
    )> {
        // * 🚩蕴含 | 【2024-07-05 17:08:34】现在只判断「蕴含」陈述
        if !self.instanceof_inheritance() {
            return None;
        }

        // * 🚩主项是乘积，且乘积的第一个参数是外延集（`{ SELF } or { VAR }`）
        let subject_product = self.subject.as_compound_type(PRODUCT_OPERATOR)?;
        let first_argument = subject_product
            .components
            .first()?
            .as_compound_type(SET_EXT_OPERATOR)?;

        // * 🚩谓项是操作符
        let predicate_op_name = self.predicate.as_operator()?;

        // * 🚩返回
        Some((self, subject_product, first_argument, predicate_op_name))
    }

    /// 转换为「复合词项引用」
    /// * 🎯不通过额外的「类型判断」（从[`DerefMut`]中来）转换为「复合词项引用」
    /// * ❌【2024-06-15 16:37:07】危险：不能在此【只传引用】，否则将能在「拿出引用」的同时「使用自身」
    ///   * 📝因此不能实现`Deref<Target = CompoundTermRef>`
    pub fn into_compound_ref(self) -> CompoundTermRef<'s> {
        debug_assert!(self.is_statement());
        // SAFETY: 保证「陈述词项」一定从「复合词项」中来
        unsafe { self.statement.as_compound_unchecked() }
    }
}

/// 转发「呈现」方法到「内部词项」
impl Display for StatementRef<'_> {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        self.statement.fmt(f)
    }
}

/// 向词项本身的自动解引用
/// * 🎯让「陈述引用」可以被看作是一个普通的词项
impl Deref for StatementRef<'_> {
    type Target = Term;

    fn deref(&self) -> &Self::Target {
        self.statement
    }
}

/// 🆕作为「陈述引用」的词项类型
/// * 🎯在程序类型层面表示一个「陈述」（可变引用）
/// * 📝【2024-06-15 17:08:26】目前「陈述可变引用」用处不大
///   * 📄OpenNARS中没有与之相关的独有方法（`Statement`类中没有可变的方法）
#[derive(Debug, PartialEq, Eq, Hash)]
pub struct StatementRefMut<'a> {
    /// 陈述词项本身
    statement: &'a mut Term,
    /// 陈述词项的主项
    subject: *mut Term,
    /// 陈述词项的谓项
    predicate: *mut Term,
}

impl<'a> StatementRefMut<'a> {
    /// 获取陈述整体
    #[doc(alias = "inner")]
    pub fn statement(self) -> &'a mut Term {
        self.statement
    }

    /// 🆕同时获取「主项」与「谓项」的可变引用
    /// * ⚠️此处对裸指针解引用
    ///   * 📄安全性保证同[`CompoundTermRefMut::components`]
    /// * 🎯获取陈述的主谓项，在这之后对齐进行变量替换
    pub fn sub_pre(&mut self) -> [&'a mut Term; 2] {
        // SAFETY: 同[`Compound::components`]
        unsafe { [&mut *self.subject, &mut *self.predicate] }
    }

    /// 📄OpenNARS `getSubject`
    /// # 📄OpenNARS
    ///
    /// 🈚
    pub fn subject(&mut self) -> &'a mut Term {
        let [sub, _] = self.sub_pre();
        sub
    }

    /// 📄OpenNARS `getPredicate`
    /// # 📄OpenNARS
    ///
    /// 🈚
    pub fn predicate(&mut self) -> &'a mut Term {
        let [_, pre] = self.sub_pre();
        pre
    }

    /// 生成一个不可变引用
    /// * 🚩将自身的所有字段转换为不可变引用，然后构造一个「不可变引用」结构
    /// * 📌可变引用一定能转换成不可变引用
    /// * ⚠️与[`AsRef`]与[`Deref`]不同：此处需要返回所有权，而非对目标类型（[`Term`]）的引用
    ///   * ❌返回`&CompoundTermRef`会导致「返回临时变量引用」故无法使用
    /// * ❌【2024-06-15 16:37:07】危险：不能在此【只传引用】，否则将能在「拿出引用」的同时「使用自身」
    pub fn into_ref<'s>(self) -> StatementRef<'s>
    where
        Self: 's,
    {
        // * 🚩解引用前（在debug模式下）检查
        debug_assert!(self.statement.is_statement());
        // * 🚩传递引用 & 裸指针解引用
        StatementRef {
            statement: self.statement,
            // SAFETY: 自身相当于对词项的可变引用，同时所有字段均保证有效——那就一定能同时转换
            subject: unsafe { &*self.subject },
            // SAFETY: 自身相当于对词项的可变引用，同时所有字段均保证有效——那就一定能同时转换
            predicate: unsafe { &*self.predicate },
        }
    }

    /// 转换为「复合词项可变引用」
    /// * 🎯不通过额外的「类型判断」（从[`DerefMut`]中来）转换为「复合词项可变引用」
    /// * ❌【2024-06-15 16:37:07】危险：不能在此【只传引用】，否则将能在「拿出引用」的同时「使用自身」
    pub fn into_compound_ref<'s>(self) -> CompoundTermRefMut<'s>
    where
        Self: 's,
    {
        debug_assert!(self.is_statement());
        // SAFETY: 保证「陈述词项」一定从「复合词项」中来
        unsafe { self.statement.as_compound_mut_unchecked() }
    }
}

/// 转发「呈现」方法到「内部词项」
impl Display for StatementRefMut<'_> {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        self.statement.fmt(f)
    }
}

/// 可变引用 ⇒ 不可变引用
impl<'s> From<StatementRefMut<'s>> for StatementRef<'s> {
    #[inline]
    fn from(r: StatementRefMut<'s>) -> Self {
        r.into_ref()
    }
}

/// 陈述可变引用 ⇒ 复合词项可变引用
impl<'s> From<StatementRefMut<'s>> for CompoundTermRefMut<'s> {
    #[inline]
    fn from(r: StatementRefMut<'s>) -> Self {
        r.into_compound_ref()
    }
}

/// 向词项本身的自动解引用
/// * 🎯让「陈述可变引用」可以被看作是一个普通的词项
/// * 📌【2024-06-15 15:08:55】安全性保证：在该引用结构使用「元素列表」时，独占引用不允许其再度解引用
/// * ❌【2024-06-15 15:38:58】不能实现「自动解引用到不可变引用」
impl Deref for StatementRefMut<'_> {
    type Target = Term;

    fn deref(&self) -> &Self::Target {
        self.statement
    }
}

/// 向词项本身的自动解引用
/// * 🎯让「陈述可变引用」可以被看作是一个普通的词项（可变引用）
/// * 📌【2024-06-15 15:08:55】安全性保证：在该引用结构使用「元素列表」时，独占引用不允许其再度解引用
impl DerefMut for StatementRefMut<'_> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        self.statement
    }
}

/// 具备所有权的复合词项
/// * 🎯初步决定用于「推理规则」向下分派
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct Statement {
    /// 内部词项
    term: Term,
}

impl Statement {
    /// 获取不可变引用
    /// * 🚩【2024-07-10 23:51:54】此处使用[`Option::unwrap`]代替`unsafe`操作
    pub fn get_ref(&self) -> StatementRef {
        self.term.as_statement().unwrap()
    }

    /// 获取可变引用
    /// * 🚩【2024-07-10 23:51:54】此处使用[`Option::unwrap`]代替`unsafe`操作
    pub fn mut_ref(&mut self) -> StatementRefMut {
        self.term.as_statement_mut().unwrap()
    }

    /// 🆕同时快捷获取`[主项, 谓项]`
    /// * 🚩【2024-07-31 22:24:07】现场解包[`StatementRef`]中的引用，避免「临时对象dropped」
    pub fn sub_pre(&self) -> [&Term; 2] {
        let StatementRef {
            subject, predicate, ..
        } = self.get_ref();
        [subject, predicate]
    }

    /// 🆕同时快捷获取`[主项, 谓项]`的可变引用
    /// * 🎯用于场景「获取 主项/谓项，然后对齐进行变量替换」
    pub fn sub_pre_mut(&mut self) -> [&mut Term; 2] {
        self.mut_ref().sub_pre()
    }

    /// 解包为内部元素（主项、谓项）
    /// * 🎯用于「推理规则」中的新词项生成
    pub fn unwrap_components(self) -> [Term; 2] {
        self.term.unwrap_statement_components().unwrap()
    }

    /// 解包为内部成分（主项、系词、谓项）
    /// * 🎯用于「推理规则」中的新词项生成
    pub fn unwrap(self) -> (Term, String, Term) {
        self.term.unwrap_statement_id_components().unwrap()
    }
}

/// 仅有的一处入口：从[词项](Term)构造
impl TryFrom<Term> for Statement {
    /// 转换失败时，返回原始词项
    type Error = Term;

    fn try_from(term: Term) -> Result<Self, Self::Error> {
        // * 🚩仅在是复合词项时转换成功
        match term.is_statement() {
            true => Ok(Self { term }),
            false => Err(term),
        }
    }
}

/// 出口（转换成词项）
impl From<Statement> for Term {
    fn from(value: Statement) -> Self {
        value.term
    }
}

/// 方便直接作为词项使用
/// * ❓是否要滥用此种「类似继承的模式」
impl Deref for Statement {
    type Target = Term;

    fn deref(&self) -> &Self::Target {
        &self.term
    }
}

/// 方便直接作为词项使用（可变）
impl DerefMut for Statement {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.term
    }
}

/// 内联「显示呈现」
impl Display for Statement {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        self.term.fmt(f)
    }
}

/// 陈述引用⇒陈述
impl StatementRef<'_> {
    /// 从「陈述引用」转换为陈述（获得所有权）
    /// * ✅对于「陈述可变引用」可以先转换为「不可变引用」使用
    pub fn to_owned(&self) -> Statement {
        debug_assert!(self.statement.is_statement()); // 转换前检验是否为陈述类词项
        Statement {
            term: self.statement.clone(),
        }
    }
}

/// 单元测试
#[cfg(test)]
mod tests {
    use super::*;
    use crate::test_term as term;
    use crate::{ok, util::AResult};
    use nar_dev_utils::{asserts, macro_once};

    macro_rules! statement {
        // 具所有权/新常量
        (box $term:literal) => {
            statement!(box term!($term))
        };
        // 具所有权/原有变量
        (box $term:expr) => {
            Statement::try_from($term).unwrap()
        };
        // 可变引用/新常量
        (mut $term:literal) => {
            statement!(mut term!($term))
        };
        // 可变引用/原有变量
        (mut $term:expr) => {
            $term.as_statement_mut().unwrap()
        };
        // 不可变引用 解包
        (unwrap $term:literal) => {
            statement!(term!(unwrap $term))
        };
        // 不可变引用
        ($term:literal) => {
            statement!(term!($term))
        };
        // 不可变引用
        ($term:expr) => {
            $term.as_statement().unwrap()
        };
    }

    /// 不可变引用
    mod statement_ref {
        use super::*;
        use nar_dev_utils::fail_tests;

        /// 陈述有效性
        /// * 🎯一并测试
        ///   * `invalid`
        ///   * `invalid_statement`
        ///   * `invalid_reflexive`
        ///   * `invalid_pair`
        #[test]
        fn invalid() -> AResult {
            asserts! {
                // 合法
                !statement!("<A --> B>").invalid()
                !statement!("<A --> [B]>").invalid()
                !statement!("<[A] --> B>").invalid()
                !statement!("<<A --> B> ==> <B --> C>>").invalid()
                !statement!("<<A --> B> ==> <C --> A>>").invalid()
                !statement!("<<A --> B> ==> <C --> D>>").invalid()
            }
            ok!()
        }

        // ! 📌【2024-09-07 13:40:39】现在无效的词项本身就不能被构建
        fail_tests! {
            invalid_非陈述词项 statement!(unwrap "(*, A, B)"); // ! 📌【2024-09-07 15:00:45】二元复合词项本该不是陈述词项
            invalid_重言式 term!(unwrap "<A --> A>");
            invalid_被包含的重言式_主项包含谓项 term!(unwrap "<[A] --> A>");
            invalid_被包含的重言式_谓项包含主项 term!(unwrap "<A --> [A]>");
            invalid_蕴含重言式 term!(unwrap "<<A --> B> ==> <B --> A>>");
        }

        #[test]
        fn subject_predicate() -> AResult {
            macro_once! {
                // * 🚩模式：陈述 ⇒ [主词, 谓词]
                macro test($($statement:expr => [$subject:literal, $predicate:literal])*) {
                    asserts! {$(
                        statement!($statement).subject() => &term!($subject)
                        statement!($statement).predicate() => &term!($predicate)
                    )*}
                }
                "<A --> B>"             => ["A", "B"]
                "<あ ==> α>"            => ["あ", "α"]
                "<{SELF} --> [good]>"   => ["{SELF}", "[good]"]
                "<<a --> b> ==> {C}>"   => ["<a --> b>", "{C}"]
                "<$1 --> [2]>"         => ["$1", "[2]"] // ! 变量词项可能会被重排编号
                "<#2 --> {1}>"         => ["#2", "{1}"] // ! 变量词项可能会被重排编号
                "<(*, 1, 2, 3) ==> 4>"  => ["(*, 1, 2, 3)", "4"]
                // ! 实例、属性、实例属性 ⇒ 继承
                "<A {-- B>"             => ["{A}",  "B"]
                "<A --] B>"             => [ "A",  "[B]"]
                "<A {-] B>"             => ["{A}", "[B]"]
            }
            ok!()
        }
    }

    /// 可变引用
    mod statement_ref_mut {
        use super::*;

        #[test]
        fn subject_predicate() -> AResult {
            macro_once! {
                // * 🚩模式：陈述 ⇒ [主词, 谓词]
                macro test($($statement:expr => [$subject:literal, $predicate:literal])*) {
                    asserts! {$(
                        statement!(mut $statement).subject() => &term!($subject)
                        statement!(mut $statement).predicate() => &term!($predicate)
                    )*}
                }
                "<A --> B>"             => ["A", "B"]
                "<あ ==> α>"            => ["あ", "α"]
                "<{SELF} --> [good]>"   => ["{SELF}", "[good]"]
                "<<a --> b> ==> {C}>"   => ["<a --> b>", "{C}"]
                "<$1 --> [2]>"         => ["$1", "[2]"] // ! 变量词项可能会被重排编号
                "<#2 --> {1}>"         => ["#2", "{1}"] // ! 变量词项可能会被重排编号
                "<(*, 1, 2, 3) ==> 4>"  => ["(*, 1, 2, 3)", "4"]
                // ! 实例、属性、实例属性 ⇒ 继承
                "<A {-- B>"             => ["{A}",  "B"]
                "<A --] B>"             => [ "A",  "[B]"]
                "<A {-] B>"             => ["{A}", "[B]"]
            }
            ok!()
        }

        #[test]
        fn to_ref() -> AResult {
            fn test(mut term: Term) {
                // * 🚩非陈述⇒返回 | 🎯检验「检验函数」
                if !term.is_statement() {
                    return;
                }
                // * 🚩构建陈述的可变引用
                let mut statement = term.as_statement_mut().expect("是陈述了还转换失败");
                // * 🚩测试/Deref
                assert!(!statement.as_statement().unwrap().invalid());
                // * 🚩假定陈述有效
                let (id, _) = statement.subject().id_comp_mut();
                *id = "".into();
                // * 🚩转换为不可变引用
                let statement = statement.into_ref();
                assert!(!statement.invalid());
            }
            macro_once! {
                macro test($($term:expr)*) {
                    $(test(term!($term));)*
                }
                // !
                "A"
                "A"
            }
            ok!()
        }
    }
    /// 具所有权
    mod statement {
        use super::*;
        use std::str::FromStr;

        /// 词项之间的类型转换
        /// * 📄[`Term::try_into`] / [`Statement::try_from`]
        /// * 📄[`Term::from`] / [`Statement::into`]
        #[test]
        fn from_into() -> AResult {
            /// 通用测试函数
            fn test(compound: Statement) {
                // * 🚩首先是一个陈述
                assert!(compound.is_compound());

                // * 🚩从内部拷贝一个词项后，仍可无损转换为陈述
                let term: Term = (*compound).clone();
                let _: Statement = term.try_into().expect("应该是陈述！");

                // * 🚩解包成普通词项后，仍可无损转换为陈述
                let term: Term = compound.into();
                let _: Statement = term.try_into().expect("应该是陈述！");
            }
            macro_once! {
                // * 🚩模式：词项字符串 ⇒ 预期
                macro test($( $term:literal )*) {$(
                    test(statement!(box $term));
                )*}
                // 单层
                "<A --> B>"
                "<A <-> B>"
                "<A ==> B>"
                "<A <=> B>"
                // 组合
                "<(*, A, B) --> P>"
                "<(*, A, B) <-> P>"
                "<(*, A, B) ==> P>"
                "<(*, A, B) <=> P>"
                "<S --> (*, A, B)>"
                "<S <-> (*, A, B)>"
                "<S ==> (*, A, B)>"
                "<S <=> (*, A, B)>"
                // 多层
                "<X --> <A ==> B>>"
                "<X <-> <A <=> B>>"
                "<<A --> B> ==> X>"
                "<<A <-> B> <=> X>"
                "<<A ==> B> --> <C ==> D>>"
                "<<A <=> B> <-> <C <=> D>>"
                "<<A --> B> ==> <C --> D>>"
                "<<A <-> B> <=> <C <-> D>>"
                r"<(/, R, A, _) --> (\, R, _, B)>"
                r"<(/, R, A, _) <-> (\, R, _, B)>"
                r"<(/, R, A, _) ==> (\, R, _, B)>"
                r"<(/, R, A, _) <=> (\, R, _, B)>"
            }
            ok!()
        }

        #[test]
        fn get_ref() -> AResult {
            /// 通用测试函数
            fn test(statement: Statement) {
                // * 🚩首先是一个陈述
                assert!(statement.is_compound());

                // * 🚩获取主谓项
                let ref_statement = statement.get_ref();
                let subject = ref_statement.subject();
                let predicate = ref_statement.predicate();
                println!("{statement} => [{subject}, {predicate}]");

                // * 🚩遍历所有元素 as 复合词项
                statement
                    .get_ref()
                    .components()
                    .iter()
                    .enumerate()
                    .for_each(|(i, component)| println!("    [{i}] => {component}"))
            }
            macro_once! {
                // * 🚩模式：词项字符串 ⇒ 预期
                macro test($( $term:literal )*) {$(
                    test(statement!(box $term));
                )*}
                // 单层
                "<A --> B>"
                "<A <-> B>"
                "<A ==> B>"
                "<A <=> B>"
                // 组合
                "<(*, A, B) --> P>"
                "<(*, A, B) <-> P>"
                "<(*, A, B) ==> P>"
                "<(*, A, B) <=> P>"
                "<S --> (*, A, B)>"
                "<S <-> (*, A, B)>"
                "<S ==> (*, A, B)>"
                "<S <=> (*, A, B)>"
                // 多层
                "<X --> <A ==> B>>"
                "<X <-> <A <=> B>>"
                "<<A --> B> ==> X>"
                "<<A <-> B> <=> X>"
                "<<A ==> B> --> <C ==> D>>"
                "<<A <=> B> <-> <C <=> D>>"
                "<<A --> B> ==> <C --> D>>"
                "<<A <-> B> <=> <C <-> D>>"
                r"<(/, R, A, _) --> (\, R, _, B)>"
                r"<(/, R, A, _) <-> (\, R, _, B)>"
                r"<(/, R, A, _) ==> (\, R, _, B)>"
                r"<(/, R, A, _) <=> (\, R, _, B)>"
            }
            ok!()
        }

        #[test]
        fn mut_ref() -> AResult {
            /// 通用测试函数
            fn test(mut statement: Statement) -> AResult {
                // * 🚩首先是一个陈述
                assert!(statement.is_compound());

                // * 🚩修改：更改主项
                let old_s = statement.to_string();
                let mut mut_ref = statement.mut_ref();
                let subject = mut_ref.subject();
                let x = term!("X");
                *subject = x.clone();
                println!("modification: {old_s:?} => \"{statement}\"");
                assert_eq!(*statement.get_ref().subject(), x); // 假定修改后的结果

                // * 🚩修改：更改谓项
                let old_s = statement.to_string();
                let mut mut_ref = statement.mut_ref();
                let predicate = mut_ref.predicate();
                let y = term!("Y");
                *predicate = y.clone();
                println!("modification: {old_s:?} => \"{statement}\"");
                assert_eq!(*statement.get_ref().predicate(), y); // 假定修改后的结果

                // * 🚩遍历修改所有元素
                statement
                    .mut_ref()
                    .into_compound_ref()
                    .components()
                    .iter_mut()
                    .enumerate()
                    .for_each(|(i, component)| {
                        *component = Term::from_str(&format!("T{i}")).unwrap()
                    });
                print!(" => \"{statement}\"");

                ok!()
            }
            macro_once! {
                // * 🚩模式：词项字符串 ⇒ 预期
                macro test($( $term:literal )*) {$(
                    test(statement!(box $term))?;
                )*}
                // 单层
                "<A --> B>"
                "<A <-> B>"
                "<A ==> B>"
                "<A <=> B>"
                // 组合
                "<(*, A, B) --> P>"
                "<(*, A, B) <-> P>"
                "<(*, A, B) ==> P>"
                "<(*, A, B) <=> P>"
                "<S --> (*, A, B)>"
                "<S <-> (*, A, B)>"
                "<S ==> (*, A, B)>"
                "<S <=> (*, A, B)>"
                // 多层
                "<X --> <A ==> B>>"
                "<X <-> <A <=> B>>"
                "<<A --> B> ==> X>"
                "<<A <-> B> <=> X>"
                "<<A ==> B> --> <C ==> D>>"
                "<<A <=> B> <-> <C <=> D>>"
                "<<A --> B> ==> <C --> D>>"
                "<<A <-> B> <=> <C <-> D>>"
                r"<(/, R, A, _) --> (\, R, _, B)>"
                r"<(/, R, A, _) <-> (\, R, _, B)>"
                r"<(/, R, A, _) ==> (\, R, _, B)>"
                r"<(/, R, A, _) <=> (\, R, _, B)>"
            }
            ok!()
        }
    }
}
