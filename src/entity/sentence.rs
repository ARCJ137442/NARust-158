//! 复刻改版OpenNARS「语句」
//! * ❌不附带「真值」假定——仅「判断」具有「真值」属性

use super::{Judgement, Question, Stamp, TruthValue};
use crate::{
    global::ClockTime, inference::Evidential, io::symbols::*, language::Term,
    util::ToDisplayAndBrief,
};
use anyhow::Result;
use narsese::lexical::Sentence as LexicalSentence;
use std::fmt::{Display, Formatter, Result as FmtResult};

/// NARust特制的「标点」类型
/// * 📌相比旧版的`SentenceType`，此处仅提供简单枚举，不附带字段
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Punctuation {
    Judgement,
    Question,
}
impl Punctuation {
    pub fn from_char(c: char) -> Option<Self> {
        match c {
            JUDGMENT_MARK => Some(Self::Judgement),
            QUESTION_MARK => Some(Self::Question),
            _ => None,
        }
    }
    pub fn to_char(&self) -> char {
        use Punctuation::*;
        match self {
            Judgement => JUDGMENT_MARK,
            Question => QUESTION_MARK,
        }
    }
}
impl Display for Punctuation {
    fn fmt(&self, f: &mut Formatter<'_>) -> FmtResult {
        use Punctuation::*;
        match self {
            Judgement => write!(f, "{JUDGMENT_MARK}"),
            Question => write!(f, "{QUESTION_MARK}"),
        }
    }
}

/// 模拟`nars.entity.Sentence`
/// * 📌【2024-05-10 20:17:04】此处不加入对[`PartialEq`]的要求：会将要求传播到上层的「词项链」「任务链」
///
/// # 📄OpenNARS
///
/// A Sentence is an abstract class, mainly containing a Term, a TruthValue, and a Stamp.
///
/// It is used as the premises and conclusions of all inference rules.
pub trait Sentence: ToDisplayAndBrief + Evidential {
    // 所有抽象字段

    /// 模拟`Sentence.revisable`、`Sentence.getRevisable`
    /// * 📝OpenNARS只在「解析任务」时会设置值
    ///   * 🎯使用目的：「包含因变量的合取」不可被修正
    ///   * 🚩【2024-05-19 13:01:57】故无需让其可变，构造后只读即可
    /// * 🚩【2024-05-24 12:05:54】现在将「是否可修正」放进「判断」标点中
    ///   * 📝根据OpenNARS逻辑，只有「判断」才有「是否可被修正」属性
    ///   * ✅现在无需再依靠具体结构来实现了
    /// * 🚩【2024-06-21 14:54:59】现在成为抽象方法
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
    fn punctuation(&self) -> Punctuation;

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
    /// * ❌【2024-06-21 15:02:36】无法外置到其它「给语句自动添加功能」的特征中去
    ///   * 📌瓶颈：冲突的默认实现
    ///
    /// # 📄OpenNARS
    ///
    /// Distinguish Judgement from Goal ("instanceof Judgement" doesn't work)
    ///
    /// @return Whether the object is a Judgement
    fn is_judgement(&self) -> bool;
    fn as_judgement(&self) -> Option<&impl Judgement>;

    /// 模拟`Sentence.isQuestion`
    /// * ❌【2024-06-21 15:02:36】无法外置到其它「给语句自动添加功能」的特征中去
    ///   * 📌瓶颈：冲突的默认实现
    ///
    /// # 📄OpenNARS
    ///
    /// Distinguish Question from Quest ("instanceof Question" doesn't work)
    ///
    /// @return Whether the object is a Question
    fn is_question(&self) -> bool;
    fn as_question(&self) -> Option<&impl Question>;

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
    fn to_key_string(&self) -> String;

    /// 模拟`Sentence.toString`
    /// * 🚩【2024-05-08 23:34:34】现在借道[`ToDisplayAndBrief`]予以实现
    /// * 🚩与[`Sentence::to_key_string`]不同的是：会纳入时间戳，并且全都是「详细信息」
    ///
    /// # 📄OpenNARS
    ///
    /// Get a String representation of the sentence
    ///
    /// @return The String
    fn sentence_to_display(&self) -> String;

    /// 模拟`Sentence.toStringBrief`
    /// * 🚩【2024-05-08 23:37:44】现在借道[`Sentence::to_key_string`]予以实现
    ///
    /// # 📄OpenNARS
    ///
    /// Get a String representation of the sentence, with 2-digit accuracy
    ///
    /// @return The String
    fn sentence_to_display_brief(&self) -> String {
        /* 📄OpenNARS源码：
        return toKey() + stamp.toString(); */
        self.to_key_string() + &self.stamp_to_display()
    }

    /// 🆕原版没有，此处仅重定向
    fn sentence_to_display_long(&self) -> String {
        self.sentence_to_display()
    }

    /// 🆕用于「新任务建立」
    /// * 🚩使用最大并集（可设置为空）建立同标点新语句
    /// * 📄判断⇒判断，问题⇒问题
    fn sentence_clone_with_same_punctuation(
        content: Term,
        new_content: Term,
        new_truth: TruthValue,
        new_stamp: Stamp,
        revisable: bool,
    ) -> Self;

    /// 🆕与OpenNARS改版不同：从「词法语句」解析
    /// * ℹ️原有的「内部语句」可能不存在标点信息，故只能上移至此
    fn from_lexical(lexical: LexicalSentence) -> Self;

    /// 🆕与OpenNARS改版不同：转换为「词法语句」
    /// * ℹ️原有的「内部语句」可能不存在标点信息，故只能上移至此
    fn sentence_to_lexical(&self) -> LexicalSentence;
    // TODO: 代码交给后续实现者实现
    /* {
        // LexicalSentence {
        //     term: self.content().into(),
        //     // 标点：采用字符串形式
        //     punctuation: self.punctuation().to_char().to_string(),
        //     stamp: self.stamp_to_lexical(),
        //     // 真值可能有、可能无
        //     truth: self
        //         .truth()
        //         .map(TruthValueConcrete::to_lexical)
        //         .unwrap_or_default(), // * 没有真值则创建一个空数组
        // }
    } */
}

/// 🆕一个用于「复用共有字段」的内部对象
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct SentenceInner {
    /// 内部词项
    content: Term,
    /// 内部「时间戳」字段
    stamp: Stamp,
}

impl SentenceInner {
    pub fn content(&self) -> &Term {
        &self.content
    }

    pub fn content_mut(&mut self) -> &mut Term {
        &mut self.content
    }

    pub fn stamp(&self) -> &Stamp {
        &self.stamp
    }

    pub fn stamp_mut(&mut self) -> &mut Stamp {
        &mut self.stamp
    }
}

/// impl<T: TruthValueConcrete, S: StampConcrete> SentenceConcrete for SentenceV1
impl SentenceInner {
    pub fn new(content: Term, stamp: Stamp) -> Self {
        Self { content, stamp }
    }

    pub fn from_lexical(
        lexical: LexicalSentence,
        stamp_current_serial: ClockTime,
        stamp_time: ClockTime,
    ) -> Result<Self> {
        // 直接解构
        let LexicalSentence { term, stamp, .. } = lexical;
        // 词项
        let content = Term::try_from(term)?;
        // 解析时间戳
        let stamp = Stamp::from_lexical(stamp, stamp_current_serial, stamp_time)?;
        // 构造
        Ok(Self::new(content, stamp))
    }
}
