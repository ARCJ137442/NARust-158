use super::{JudgementV1, Sentence, SentenceInner, Stamp};
use crate::{__impl_to_display_and_display, inference::Evidential, language::Term};
use nar_dev_utils::join;
use narsese::lexical::Sentence as LexicalSentence;

pub trait Question: Sentence {
    // ! ❌不能在此自动实现`isQuestion` `asQuestion`
    // * 📝或者，Rust不允许类似「继承」的「实现一部分，丢给别的类型再实现另一部分」的做法

    // ! ❌不能在此自动实现`toKey` `sentenceToString`
    // * 📝或者，Rust不允许类似「继承」的「实现一部分，丢给别的类型再实现另一部分」的做法
    /// 作为一个[`Sentence::to_key`]的默认【非覆盖性】实现
    fn question_to_key(&self) -> String {
        join! {
            => self.content().to_string()
            => self.punctuation().to_string()
        }
    }

    /// 作为一个[`Sentence::sentence_to_display`]的默认【非覆盖性】实现
    fn question_to_display(&self) -> String {
        join! {
            => self.content().to_string()
            => self.punctuation().to_string()
            => self.stamp_to_display()
        }
    }
}

/// 🆕疑问句 初代实现
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct QuestionV1 {
    inner: SentenceInner,
}

impl QuestionV1 {
    pub fn new(content: Term, stamp: Stamp) -> Self {
        Self {
            inner: SentenceInner::new(content, stamp),
        }
    }
}

impl Evidential for QuestionV1 {
    fn evidential_base(&self) -> &[crate::global::ClockTime] {
        self.inner.stamp().evidential_base()
    }

    fn creation_time(&self) -> crate::global::ClockTime {
        self.inner.stamp().creation_time()
    }

    fn stamp_to_lexical(&self) -> narsese::lexical::Stamp {
        self.inner.stamp().stamp_to_lexical()
    }
}

impl Sentence for QuestionV1 {
    type Judgement = JudgementV1;
    type Question = Self;

    fn content(&self) -> &Term {
        self.inner.content()
    }

    fn content_mut(&mut self) -> &mut Term {
        self.inner.content_mut()
    }

    fn punctuation(&self) -> super::Punctuation {
        super::Punctuation::Question
    }

    fn as_judgement(&self) -> Option<&Self::Judgement> {
        None
    }

    fn as_question(&self) -> Option<&Self::Question> {
        Some(self)
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

impl Question for QuestionV1 {}

__impl_to_display_and_display! {
    QuestionV1 as Sentence
}
