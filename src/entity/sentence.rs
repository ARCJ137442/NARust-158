//! 🎯复刻OpenNARS `nars.entity.Sentence`
//! * 🚩🆕一并复刻「标点」，不仅仅是[`char`]
//!   * ✨能反映「判断有真值，问题无真值」的约束
//! * ✅【2024-05-05 18:27:41】所有方法基本复刻完毕
//! * ✅【2024-05-05 19:41:04】基本完成初代实现
//!
//! ? 是否需要与之对应的解析器
//! * 💭这里的「解析器」有可能是特定的
//!   * 📄时间戳需要结合推理器自身，以及「记忆区」「概念」等

use super::{Stamp, StampConcrete, TruthValueConcrete};
use crate::{io::symbols, language::Term, ToDisplayAndBrief};
use std::hash::{Hash, Hasher};

// /// 🆕模拟`nars.entity.Sentence.punctuation`
// /// * 📌作为一个枚举，相比「字符」更能指定其范围
// /// * 🚩【2024-05-05 17:08:35】目前直接复用[「枚举Narsese」](narsese::enum_narsese)的工作
// pub type Punctuation = narsese::enum_narsese::Punctuation;

/// 模拟`nars.entity.Sentence.punctuation`和OpenNARS`nars.entity.Sentence.truth`
/// * 🎯应对「判断有真值，问题无真值」的情况
#[doc(alias = "Punctuation")]
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum SentenceType<T: TruthValueConcrete> {
    /// 🆕「判断」有真值
    Judgement(T),
    /// 🆕「问题」无真值
    Question,
    // ! 其它类型暂且不表
}

impl<T: TruthValueConcrete> SentenceType<T> {
    /// 将自身与「标点字符」作转换
    /// * 🎯用于生成[`super::Item`]的（字符串）id
    fn punctuation_char(&self) -> char {
        use SentenceType::*;
        match self {
            Judgement(_) => symbols::JUDGMENT_MARK,
            Question => symbols::QUESTION_MARK,
        }
    }
}

/// 模拟`nars.entity.Sentence`
///
/// # 📄OpenNARS
///
/// A Sentence is an abstract class, mainly containing a Term, a TruthValue, and a Stamp.
///
/// It is used as the premises and conclusions of all inference rules.
pub trait Sentence: ToDisplayAndBrief {
    // TODO: 可能后续统一要求`Display`
    /// 绑定的「真值」类型
    type Truth: TruthValueConcrete;

    /// 绑定的「时间戳」类型
    type Stamp: StampConcrete;

    /// 模拟`Sentence.content`、`Sentence.getContent`
    /// * 🚩读写：出现了两个方法
    ///
    /// # 📄OpenNARS
    ///
    /// ## `content`
    ///
    /// The content of a Sentence is a Term
    ///
    /// ## `getContent`
    ///
    /// Get the content of the sentence
    ///
    /// @return The content Term
    fn content(&self) -> &Term;
    /// 模拟`Sentence.setContent`
    /// * 📌[`Sentence::content`]的可变版本
    ///
    /// # 📄OpenNARS
    ///
    /// Set the content Term of the Sentence
    ///
    /// @param t The new content
    fn content_mut(&mut self) -> &mut Term;

    /// 模拟
    /// * `Sentence.punctuation`、`Sentence.getPunctuation`
    /// * `Sentence.truth`、`Sentence.getTruth`
    /// * 🚩【2024-05-05 18:08:26】双属性合一，旨在表示「判断有真值，问题无真值」的约束关系
    /// * 📝OpenNARS中的使用情况
    ///   * `getPunctuation`仅在「构造」「赋值」「判等」中使用，无需直接模拟
    ///
    /// # 📄OpenNARS
    ///
    /// ## `punctuation`
    ///
    /// The punctuation also indicates the type of the Sentence: Judgement,
    /// Question, or Goal
    ///
    /// ## `getPunctuation`
    ///
    /// Get the punctuation of the sentence
    ///
    /// @return The character '.' or '?'
    #[doc(alias = "type")]
    #[doc(alias = "sentence_type")]
    fn punctuation(&self) -> &SentenceType<Self::Truth>;
    /// [`Sentence::punctuation`]的可变版本
    /// * 🚩【2024-05-05 18:13:47】[`Sentence::truth_mut`]需要
    fn punctuation_mut(&mut self) -> &mut SentenceType<Self::Truth>;

    /// 模拟`Sentence.truth`、`Sentence.getTruth`
    /// * 🚩读写：可能在「获取真值」后要改变「真值」对象
    /// * ⚠️依据语句的类型而定
    ///   * 「判断」有真值
    ///   * 「问题」无真值
    ///
    /// # 📄OpenNARS
    ///
    /// ## `truth`
    ///
    /// The truth value of Judgement
    ///
    /// ## `getTruth`
    ///
    /// Get the truth value of the sentence
    ///
    /// @return Truth value, null for question
    fn truth(&self) -> Option<&Self::Truth> {
        // 直接匹配
        match self.punctuation() {
            SentenceType::Judgement(truth) => Some(truth),
            SentenceType::Question => None,
        }
    }
    /// [`Sentence::truth`]的可变版本
    fn truth_mut(&mut self) -> Option<&mut Self::Truth> {
        // 直接匹配
        match self.punctuation_mut() {
            SentenceType::Judgement(truth) => Some(truth),
            SentenceType::Question => None,
        }
    }

    /// 模拟`Sentence.stamp`、`Sentence.getStamp`、`Sentence.setStamp`
    /// * 🚩读写：读写方法均出现
    /// * ✨将会借此直接实现[`super::Stamp`]特征
    ///
    /// # 📄OpenNARS
    ///
    /// Partial record of the derivation path
    fn stamp(&self) -> &Self::Stamp;
    /// [`Sentence::stamp`]的可变版本
    fn stamp_mut(&mut self) -> &mut Self::Stamp;

    /// 模拟`Sentence.revisable`、`Sentence.getRevisable`
    /// * ⚠️读写：需要设置其中的值
    ///
    /// # 📄OpenNARS
    ///
    /// ## `revisable`
    ///
    /// Whether the sentence can be revised
    ///
    /// ## `getRevisable`
    ///
    /// 🈚
    fn revisable(&self) -> bool;
    /// 模拟`Sentence.setRevisable`
    /// * 📌[`Sentence::revisable`]的可变版本
    ///
    /// # 📄OpenNARS
    ///
    /// 🈚
    fn revisable_mut(&mut self) -> &mut bool;

    /// 模拟`Sentence.cloneContent`
    /// * 🚩拷贝内部词项
    ///
    /// # 📄OpenNARS
    ///
    /// Clone the content of the sentence
    ///
    /// @return A clone of the content Term
    #[inline(always)]
    fn clone_content(&self) -> Term {
        self.content().clone()
    }

    /// 模拟`Sentence.isJudgement`
    ///
    /// # 📄OpenNARS
    ///
    /// Distinguish Judgement from Goal ("instanceof Judgement" doesn't work)
    ///
    /// @return Whether the object is a Judgement
    #[inline(always)]
    fn is_judgement(&self) -> bool {
        matches!(self.punctuation(), SentenceType::Judgement(..))
    }

    /// 模拟`Sentence.isQuestion`
    ///
    /// # 📄OpenNARS
    ///
    /// Distinguish Question from Quest ("instanceof Question" doesn't work)
    ///
    /// @return Whether the object is a Question
    #[inline(always)]
    fn is_question(&self) -> bool {
        matches!(self.punctuation(), SentenceType::Question)
    }

    /// 模拟`Sentence.containQueryVar`
    ///
    /// # 📄OpenNARS
    ///
    /// 🈚
    #[inline(always)]
    fn contain_query_var(&self) -> bool {
        /* 📄OpenNARS源码
        return (content.getName().indexOf(Symbols.VAR_QUERY) >= 0); */
        self.content().contain_var_q()
    }

    /// 模拟`Sentence.toKey`
    /// * 📝这个函数似乎被用来给Task作为「Item」提供索引
    ///   * 📄OpenNARS中没有用到时间戳
    /// * 💭实际上只要「独一无二」即可
    /// * 🚩【2024-05-08 22:18:06】目前直接对接[`ToDisplayAndBrief`]
    /// * 🚩【2024-05-10 01:09:44】现在只会在[`crate::entity::TaskConcrete::__new`]的实现中被用到
    ///   * 具体体现在[`crate::entity::TaskV1`]中
    ///
    /// # 📄OpenNARS
    ///
    /// Get a String representation of the sentence for key of Task and TaskLink
    ///
    /// @return The String
    #[doc(alias = "to_key")]
    fn to_key_string(&self) -> String {
        /* 📄OpenNARS源码：
        StringBuilder s = new StringBuilder();
        s.append(content.toString());
        s.append(punctuation).append(" ");
        if (truth != null) {
            s.append(truth.toStringBrief());
        }
        return s.toString(); */
        let mut s = String::new();
        s += &self.content().to_string();
        s.push(self.punctuation().punctuation_char());
        s.push(' ');
        if let Some(truth) = self.truth() {
            s += &truth.to_display_brief();
        }
        s
    }

    /// 模拟`Sentence.toString`
    /// * 🚩【2024-05-08 23:34:34】现在借道[`ToDisplayAndBrief`]予以实现
    /// * 🚩与[`Sentence::to_key_string`]不同的是：会纳入时间戳，并且全都是「详细信息」
    ///
    /// # 📄OpenNARS
    ///
    /// Get a String representation of the sentence
    ///
    /// @return The String
    fn __to_display(&self) -> String {
        /* 📄OpenNARS源码：
        StringBuilder s = new StringBuilder();
        s.append(content.toString());
        s.append(punctuation).append(" ");
        if (truth != null) {
            s.append(truth.toStringBrief());
        }
        s.append(stamp.toString());
        return s.toString(); */
        let mut s = String::new();
        s += &self.content().to_string();
        s.push(self.punctuation().punctuation_char());
        s.push(' ');
        if let Some(truth) = self.truth() {
            s += &truth.to_display();
        }
        s += &self.stamp().to_display();
        s
    }

    /// 模拟`Sentence.toStringBrief`
    /// * 🚩【2024-05-08 23:37:44】现在借道[`Sentence::to_key_string`]予以实现
    ///
    /// # 📄OpenNARS
    ///
    /// Get a String representation of the sentence, with 2-digit accuracy
    ///
    /// @return The String
    fn __to_display_brief(&self) -> String {
        /* 📄OpenNARS源码：
        return toKey() + stamp.toString(); */
        self.to_key_string() + &self.stamp().to_display()
    }
}

// ! ❌【2024-05-05 18:12:28】由于「真值」不是【每种类型的语句都有】，因此不能自动实现
// ! ❌若通过`unwrap`实现，则很容易在「问题」上panic
/* /// 自动实现「真值」特征
/// * ✨语句代理「真值」的特征，可以被看作「真值」使用
impl<S: Sentence + Eq> TruthValue for S {
    type E = <S::Truth as TruthValue>::E;

    #[inline(always)]
    fn frequency(&self) -> Self::E {
        self.truth().frequency()
    }

    #[inline(always)]
    fn frequency_mut(&mut self) -> &mut Self::E {
        self.truth_mut().frequency_mut()
    }

    #[inline(always)]
    fn confidence(&self) -> Self::E {
        self.truth().confidence()
    }

    #[inline(always)]
    fn confidence_mut(&mut self) -> &mut Self::E {
        self.truth_mut().confidence_mut()
    }

    #[inline(always)]
    fn is_analytic(&self) -> bool {
        self.truth().is_analytic()
    }

    #[inline(always)]
    fn set_analytic(&mut self) {
        self.truth_mut().set_analytic()
    }
} */

/// 自动实现「时间戳」特征
/// * ✨语句代理「时间戳」的特征，可以被看作「时间戳」使用
impl<S: Sentence + Hash> Stamp for S {
    #[inline(always)]
    fn evidential_base(&self) -> &[crate::global::ClockTime] {
        self.stamp().evidential_base()
    }

    #[inline(always)]
    fn creation_time(&self) -> crate::global::ClockTime {
        self.stamp().creation_time()
    }
}

/// [`Sentence`]的具体类型版本
/// * 📌假定信息就是「所获取的信息」没有其它外延
/// * 🎯约束构造方法
/// * 📝OpenNARS中`revisable`不参与判等、散列化
/// * 🚩用特征约束 [`Hash`]模拟`Stamp.hashCode`
/// * 🚩用特征约束 [`PartialEq`]模拟`Stamp.hashCode`
///   * ⚠️因「孤儿规则」限制，无法统一自动实现
///   * 📌统一的逻辑：**对「证据基」集合判等（无序相等）**
///
/// * 🚩用[`Clone`]对标Java接口`Cloneable`，并模拟`new Sentence(Stamp)`
pub trait SentenceConcrete: Sentence + Clone + Hash + PartialEq {
    /// 模拟`new Sentence(Term content, char punctuation, TruthValue truth, Stamp stamp, boolean revisable)`
    /// * 📌包含所有字段的构造函数
    /// * 🚩【2024-05-05 18:39:19】现在使用「语句类型」简并「标点」「真值」两个字段
    ///   * 🎯应对「判断有真值，问题无真值」的情形
    ///
    /// # 📄OpenNARS
    ///
    /// Create a Sentence with the given fields
    ///
    /// @param content     The Term that forms the content of the sentence
    /// @param punctuation The punctuation indicating the type of the sentence
    /// @param truth       The truth value of the sentence, null for question
    /// @param stamp       The stamp of the sentence indicating its derivation time and base
    /// @param revisable   Whether the sentence can be revised
    fn new(
        content: Term,
        // punctuation: Punctuation,
        // truth: Self::Truth,
        sentence_type: SentenceType<Self::Truth>,
        stamp: Self::Stamp,
        revisable: bool,
    ) -> Self;

    /// 模拟`new Sentence(Term content, char punctuation, TruthValue truth, Stamp stamp)`
    /// * 📝OpenNARS中默认`revisable`为`true`
    /// * 🚩【2024-05-05 18:39:19】现在使用「语句类型」简并「标点」「真值」两个字段
    ///   * 🎯应对「判断有真值，问题无真值」的情形
    ///
    /// # 📄OpenNARS
    ///
    /// Create a Sentence with the given fields
    ///
    /// @param content     The Term that forms the content of the sentence
    /// @param punctuation The punctuation indicating the type of the sentence
    /// @param truth       The truth value of the sentence, null for question
    /// @param stamp       The stamp of the sentence indicating its derivation time
    fn new_revisable(
        content: Term,
        // punctuation: Punctuation,
        // truth: Self::Truth,
        sentence_type: SentenceType<Self::Truth>,
        stamp: Self::Stamp,
    ) -> Self {
        Self::new(content, sentence_type, stamp, true)
    }

    /// 模拟`Sentence.equals`
    /// * 🎯用于方便实现者用其统一实现[`PartialEq`]
    /// * 📝OpenNARS中「是否可修订」不被纳入「判等」的标准
    ///
    /// # 📄OpenNARS
    ///
    /// To check whether two sentences are equal
    ///
    /// @param that The other sentence
    /// @return Whether the two sentences have the same content
    fn equals(&self, other: &impl Sentence<Truth = Self::Truth, Stamp = Self::Stamp>) -> bool {
        /* 📄OpenNARS源码：
        if (that instanceof Sentence) {
            Sentence t = (Sentence) that;
            return content.equals(t.getContent()) && punctuation == t.getPunctuation() && truth.equals(t.getTruth())
                    && stamp.equals(t.getStamp());
        }
        return false; */
        self.content() == other.content()
            && self.punctuation() == other.punctuation()
            // && self.truth() == other.truth() // ! 📌【2024-05-05 18:36:52】「真值」已经在上边的「标点（语句类型）」中被连带判断了
            && self.stamp() == other.stamp()
    }

    /// 模拟`Sentence.hashCode`
    /// * 🎯用于方便实现者用其统一实现[`Hash`]
    /// * 🚩散列化除了[`Sentence::revisable`]外的所有值
    ///
    /// # 📄OpenNARS
    ///
    /// To produce the hash-code of a sentence
    ///
    /// @return A hash-code
    #[inline(always)]
    fn __hash<H: Hasher>(&self, state: &mut H) {
        /* 📄OpenNARS源码：
        int hash = 5;
        hash = 67 * hash + (this.content != null ? this.content.hashCode() : 0);
        hash = 67 * hash + this.punctuation;
        hash = 67 * hash + (this.truth != null ? this.truth.hashCode() : 0);
        hash = 67 * hash + (this.stamp != null ? this.stamp.hashCode() : 0);
        return hash; */
        self.content().hash(state);
        self.punctuation().hash(state);
        self.truth().hash(state);
        self.stamp().hash(state);
    }

    /// ! ❌不直接模拟`equivalentTo`方法，重定向自`equals`方法
    /// * 📄OpenNARS中只在`Concept.addToTable`中使用
    /// * ⚠️已弃用：OpenNARS 3.1.0已经将其删除
    ///
    /// # 📄OpenNARS
    ///
    /// Check whether the judgement is equivalent to another one
    ///
    /// The two may have different keys
    ///
    /// @param that The other judgement
    /// @return Whether the two are equivalent
    #[inline(always)]
    fn equivalent_to(
        &self,
        other: &impl Sentence<Truth = Self::Truth, Stamp = Self::Stamp>,
    ) -> bool {
        /* 📄OpenNARS源码：
        assert content.equals(that.getContent()) && punctuation == that.getPunctuation();
        return (truth.equals(that.getTruth()) && stamp.equals(that.getStamp())); */
        self.equals(other)
    }
}

// TODO: 有关解析器的方法：从「词项」转换
// * 📌一个难点是「时间戳」的转换（需要依赖「时钟」等量）

/// 初代实现
/// * 📌需要作为一个**独立对象**使用
///   * 📄[「概念」](super::Concept)中的「信念表」
mod impl_v1 {
    use super::*;
    use crate::__impl_to_display_and_display;

    #[derive(Debug, Clone)]
    pub struct SentenceV1<T: TruthValueConcrete, S: StampConcrete> {
        /// 内部词项
        content: Term,
        /// 内部「标点」（语句类型）
        /// * 🚩标点+真值
        punctuation: SentenceType<T>,
        /// 内部「时间戳」字段
        stamp: S,
        /// 内部「可修订」字段
        revisable: bool,
    }

    // * 【2024-05-05 19:38:47】📌后边都是非常简单的「字段对字段」实现 //

    impl<T, S> PartialEq for SentenceV1<T, S>
    where
        T: TruthValueConcrete,
        S: StampConcrete,
    {
        #[inline(always)]
        fn eq(&self, other: &Self) -> bool {
            self.equals(other)
        }
    }

    impl<T, S> Hash for SentenceV1<T, S>
    where
        T: TruthValueConcrete,
        S: StampConcrete,
    {
        #[inline(always)]
        fn hash<H: Hasher>(&self, state: &mut H) {
            self.__hash(state);
        }
    }

    // * 🚩自动实现`ToDisplayAndBrief`
    __impl_to_display_and_display! {
        {T, S}
        SentenceV1<T, S> as Sentence
        where
            T: TruthValueConcrete,
            S: StampConcrete,
    }

    impl<T, S> Sentence for SentenceV1<T, S>
    where
        T: TruthValueConcrete,
        S: StampConcrete,
    {
        type Truth = T;

        type Stamp = S;

        fn content(&self) -> &Term {
            &self.content
        }

        fn content_mut(&mut self) -> &mut Term {
            &mut self.content
        }

        fn punctuation(&self) -> &SentenceType<Self::Truth> {
            &self.punctuation
        }

        fn punctuation_mut(&mut self) -> &mut SentenceType<Self::Truth> {
            &mut self.punctuation
        }

        fn stamp(&self) -> &Self::Stamp {
            &self.stamp
        }

        fn stamp_mut(&mut self) -> &mut Self::Stamp {
            &mut self.stamp
        }

        fn revisable(&self) -> bool {
            self.revisable
        }

        fn revisable_mut(&mut self) -> &mut bool {
            &mut self.revisable
        }
    }

    impl<T, S> SentenceConcrete for SentenceV1<T, S>
    where
        T: TruthValueConcrete,
        S: StampConcrete,
    {
        fn new(
            content: Term,
            // punctuation: Punctuation,
            // truth: Self::Truth,
            sentence_type: SentenceType<Self::Truth>,
            stamp: Self::Stamp,
            revisable: bool,
        ) -> Self {
            Self {
                content,
                punctuation: sentence_type,
                stamp,
                revisable,
            }
        }
    }
}
pub use impl_v1::*;

/// TODO: 单元测试
#[cfg(test)]
mod tests {
    use super::*;
    use crate::{
        entity::{StampV1, TruthV1},
        global::tests::AResult,
        ok, short_float, stamp, term,
    };

    /// 用于测试的「语句」类型
    type S = SentenceV1<TruthV1, StampV1>;

    /// 测试/content
    #[test]
    fn content() -> AResult {
        let term = term!(<A --> B>)?;
        let stamp = stamp!({1: 1; 2; 3});
        let punctuation = SentenceType::Question;
        let sf = short_float!(str "0.5");
        let sentence = S::new(term, punctuation, stamp, false);
        dbg!(sentence);
        ok!()
    }

    /// 测试/content_mut
    #[test]
    fn content_mut() -> AResult {
        // TODO: 填充测试内容
        ok!()
    }

    /// 测试/punctuation
    #[test]
    fn punctuation() -> AResult {
        // TODO: 填充测试内容
        ok!()
    }

    /// 测试/punctuation_mut
    #[test]
    fn punctuation_mut() -> AResult {
        // TODO: 填充测试内容
        ok!()
    }

    /// 测试/truth
    #[test]
    fn truth() -> AResult {
        // TODO: 填充测试内容
        ok!()
    }

    /// 测试/truth_mut
    #[test]
    fn truth_mut() -> AResult {
        // TODO: 填充测试内容
        ok!()
    }

    /// 测试/stamp
    #[test]
    fn stamp() -> AResult {
        // TODO: 填充测试内容
        ok!()
    }

    /// 测试/stamp_mut
    #[test]
    fn stamp_mut() -> AResult {
        // TODO: 填充测试内容
        ok!()
    }

    /// 测试/revisable
    #[test]
    fn revisable() -> AResult {
        // TODO: 填充测试内容
        ok!()
    }

    /// 测试/revisable_mut
    #[test]
    fn revisable_mut() -> AResult {
        // TODO: 填充测试内容
        ok!()
    }

    /// 测试/clone_content
    #[test]
    fn clone_content() -> AResult {
        // TODO: 填充测试内容
        ok!()
    }

    /// 测试/is_judgement
    #[test]
    fn is_judgement() -> AResult {
        // TODO: 填充测试内容
        ok!()
    }

    /// 测试/is_question
    #[test]
    fn is_question() -> AResult {
        // TODO: 填充测试内容
        ok!()
    }

    /// 测试/contain_query_var
    #[test]
    fn contain_query_var() -> AResult {
        // TODO: 填充测试内容
        ok!()
    }

    /// 测试/to_key_string
    #[test]
    fn to_key_string() -> AResult {
        // TODO: 填充测试内容
        ok!()
    }

    /// 测试/__to_display
    #[test]
    fn __to_display() -> AResult {
        // TODO: 填充测试内容
        ok!()
    }

    /// 测试/__to_display_brief
    #[test]
    fn __to_display_brief() -> AResult {
        // TODO: 填充测试内容
        ok!()
    }
}
