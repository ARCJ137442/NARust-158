use super::{QuestionV1, Sentence, SentenceInner, Stamp, TruthValue};
use crate::{
    __impl_to_display_and_display,
    global::ClockTime,
    inference::{Evidential, Truth},
    language::Term,
};
use nar_dev_utils::join;
use narsese::lexical::Sentence as LexicalSentence;

pub trait Judgement: Sentence + Truth {
    /// 📄改版OpenNARS `static revisable`
    fn revisable_to(&self, other: &Self) -> bool {
        let content_eq = self.content() == other.content();
        let other_revisable = other.revisable();
        content_eq && other_revisable
    }

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

    // ! ❌不能在此自动实现`isJudgement` `asJudgement`
    // * 📝或者，Rust不允许类似「继承」的「实现一部分，丢给别的类型再实现另一部分」的做法

    fn is_belief_equivalent(&self, other: &impl Judgement) -> bool {
        self.truth_eq(other) && self.evidential_eq(other)
    }

    // ! ❌不能在此自动实现`toKey` `sentenceToString`
    // * 📝或者，Rust不允许类似「继承」的「实现一部分，丢给别的类型再实现另一部分」的做法
    /// 作为一个[`Sentence::to_key`]的默认【非覆盖性】实现
    fn judgement_to_key(&self) -> String {
        join! {
            => self.content().to_string()
            => self.punctuation().to_string() + " "
            => self.truth_to_display_brief()
        }
    }

    /// 作为一个[`Sentence::sentence_to_display`]的默认【非覆盖性】实现
    fn judgement_to_display(&self) -> String {
        join! {
            => self.content().to_string()
            => self.punctuation().to_string() + " "
            => self.truth_to_display()
            => self.stamp_to_display()
        }
    }
}

/// 🆕判断句 初代实现
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct JudgementV1 {
    /// 🆕内部存储的「语句」实现
    inner: SentenceInner,
    /// Whether the sentence can be revised
    revisable: bool,
    /// The truth value of Judgment
    truth: TruthValue,
}

impl JudgementV1 {
    pub fn new(content: Term, truth: &impl Truth, stamp: Stamp, revisable: bool) -> Self {
        Self {
            inner: SentenceInner::new(content, stamp),
            revisable,
            truth: TruthValue::from(truth),
        }
    }
}

impl Evidential for JudgementV1 {
    fn evidential_base(&self) -> &[ClockTime] {
        self.inner.stamp().evidential_base()
    }

    fn creation_time(&self) -> ClockTime {
        self.inner.stamp().creation_time()
    }

    fn stamp_to_lexical(&self) -> narsese::lexical::Stamp {
        self.inner.stamp().stamp_to_lexical()
    }
}

impl Sentence for JudgementV1 {
    type Judgement = Self;
    type Question = QuestionV1;
    fn content(&self) -> &Term {
        self.inner.content()
    }

    fn content_mut(&mut self) -> &mut Term {
        self.inner.content_mut()
    }

    fn punctuation(&self) -> super::Punctuation {
        super::Punctuation::Judgement
    }

    fn is_judgement(&self) -> bool {
        true
    }

    fn as_judgement(&self) -> Option<&Self::Judgement> {
        Some(self)
    }

    fn is_question(&self) -> bool {
        false
    }

    fn as_question(&self) -> Option<&Self::Question> {
        None
    }

    fn to_key(&self) -> String {
        todo!()
    }

    fn sentence_to_display(&self) -> String {
        todo!()
    }

    fn sentence_to_lexical(&self) -> LexicalSentence {
        todo!()
    }
}

impl Truth for JudgementV1 {
    #[inline(always)]
    fn frequency(&self) -> super::ShortFloat {
        self.truth.frequency()
    }

    #[inline(always)]
    fn frequency_mut(&mut self) -> &mut super::ShortFloat {
        self.truth.frequency_mut()
    }

    #[inline(always)]
    fn confidence(&self) -> super::ShortFloat {
        self.truth.confidence()
    }

    #[inline(always)]
    fn confidence_mut(&mut self) -> &mut super::ShortFloat {
        self.truth.confidence_mut()
    }

    #[inline(always)]
    fn is_analytic(&self) -> bool {
        self.truth.is_analytic()
    }

    #[inline(always)]
    fn set_analytic(&mut self) {
        self.truth.set_analytic()
    }
}

impl Judgement for JudgementV1 {
    fn revisable(&self) -> bool {
        self.revisable
    }
}

__impl_to_display_and_display! {
    @(judgement_to_display;;)
    JudgementV1 as Judgement
}
