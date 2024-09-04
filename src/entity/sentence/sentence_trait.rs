//! 作为特征的「语句」类型

use crate::{
    entity::{Judgement, PunctuatedSentenceRef, Punctuation, Question, Stamp},
    global::ClockTime,
    inference::Evidential,
    language::Term,
    util::ToDisplayAndBrief,
};
use anyhow::Result;
use nar_dev_utils::matches_or;
use narsese::lexical::Sentence as LexicalSentence;
use serde::{Deserialize, Serialize};

/// 模拟`nars.entity.Sentence`
/// * 📌【2024-05-10 20:17:04】此处不加入对[`PartialEq`]的要求：会将要求传播到上层的「词项链」「任务链」
///
/// # 📄OpenNARS
///
/// A Sentence is an abstract class, mainly containing a Term, a TruthValue, and a Stamp.
///
/// It is used as the premises and conclusions of all inference rules.
pub trait Sentence: ToDisplayAndBrief + Evidential {
    /// 🆕复制其中的「语句」成分
    /// * 🎯为了不让方法实现冲突而构建（复制出一个「纯粹的」语句对象）
    /// * 🚩【2024-07-10 22:12:45】此处假定「复制后语句的生命周期超过引用自身的生命周期」
    ///   * 📌保证「复制后的语句」与自身生命周期无关（独立值）
    fn sentence_clone<'s, 'sentence: 's>(&'s self) -> impl Sentence + 'sentence;

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

    // * ⚠️Rust中必须预先定义其中的「判断句」「疑问句」类型
    //   * 📌直接原因：对于带泛型的`as_XXX`，需要知道其中的类型参数，才能正常参与编译
    type Judgement: Judgement;
    type Question: Question;

    /// 🆕作为【标点类型与内部引用数据兼备】的「带标点引用」
    /// * 🚩【2024-07-09 13:13:23】目前只完成不可变引用
    fn as_punctuated_ref(&self) -> PunctuatedSentenceRef<Self::Judgement, Self::Question>;

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
    #[inline]
    fn punctuation(&self) -> Punctuation {
        // * 🚩现在直接用「带标点引用」转换
        self.as_punctuated_ref().into()
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
    fn is_judgement(&self) -> bool {
        matches!(
            self.as_punctuated_ref(),
            PunctuatedSentenceRef::Judgement(..)
        )
    }
    fn as_judgement(&self) -> Option<&Self::Judgement> {
        // * 🚩【2024-07-09 13:17:25】现在直接复用一个函数
        matches_or! {
            ?self.as_punctuated_ref(),
            PunctuatedSentenceRef::Judgement(j) => j
        }
    }
    /// `as_judgement`的快捷解包
    /// * 🎯推理规则中对「前向推理⇒任务有真值」的使用
    fn unwrap_judgement(&self) -> &Self::Judgement {
        // * 🚩【2024-07-09 13:17:25】现在直接复用一个函数
        self.as_judgement().unwrap()
    }

    /// 模拟`Sentence.isQuestion`
    /// * ❌【2024-06-21 15:02:36】无法外置到其它「给语句自动添加功能」的特征中去
    ///   * 📌瓶颈：冲突的默认实现
    ///
    /// # 📄OpenNARS
    ///
    /// Distinguish Question from Quest ("instanceof Question" doesn't work)
    ///
    /// @return Whether the object is a Question
    fn is_question(&self) -> bool {
        matches!(
            self.as_punctuated_ref(),
            PunctuatedSentenceRef::Question(..)
        )
    }
    fn as_question(&self) -> Option<&Self::Question> {
        // * 🚩【2024-07-09 13:17:25】现在直接复用一个函数
        matches_or! {
            ?self.as_punctuated_ref(),
            PunctuatedSentenceRef::Question(q) => q
        }
    }
    /// `as_question`的快捷解包
    fn unwrap_question(&self) -> &Self::Question {
        // * 🚩【2024-07-09 13:17:25】现在直接复用一个函数
        self.as_question().unwrap()
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
    #[doc(alias = "to_key_string")]
    fn to_key(&self) -> String;

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
        self.to_key() + &self.stamp_to_display()
    }

    /// 🆕原版没有，此处仅重定向
    fn sentence_to_display_long(&self) -> String {
        self.sentence_to_display()
    }

    // /// 🆕与OpenNARS改版不同：从「词法语句」解析
    // /// * ℹ️原有的「内部语句」可能不存在标点信息，故只能上移至此
    // fn from_lexical(lexical: LexicalSentence) -> Self;
    // ! ❌【2024-06-21 19:12:02】暂不实现：留给「任务解析器」

    /// 🆕与OpenNARS改版不同：转换为「词法语句」
    /// * ℹ️原有的「内部语句」可能不存在标点信息，故只能上移至此
    fn sentence_to_lexical(&self) -> LexicalSentence;
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
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
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
