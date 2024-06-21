use super::{JudgementV1, Punctuation, Question, Sentence, SentenceInner};
use crate::{__impl_to_display_and_display, entity::Stamp, inference::Evidential, language::Term};
use narsese::lexical::Sentence as LexicalSentence;

/// 🆕疑问句 初代实现
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct QuestionV1 {
    pub(crate) inner: SentenceInner,
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
    fn sentence_clone(&self) -> impl Sentence {
        self.clone()
    }

    fn content(&self) -> &Term {
        self.inner.content()
    }

    fn content_mut(&mut self) -> &mut Term {
        self.inner.content_mut()
    }

    fn punctuation(&self) -> Punctuation {
        Punctuation::Question
    }

    type Judgement = JudgementV1;
    type Question = Self;

    fn as_judgement(&self) -> Option<&Self::Judgement> {
        None
    }

    fn as_question(&self) -> Option<&Self::Question> {
        Some(self)
    }

    fn to_key(&self) -> String {
        self.question_to_key()
    }

    fn sentence_to_display(&self) -> String {
        self.question_to_display()
    }

    fn sentence_to_lexical(&self) -> LexicalSentence {
        self.question_to_lexical()
    }
}

impl Question for QuestionV1 {}

__impl_to_display_and_display! {
    @(question_to_display;;)
    QuestionV1 as Question
}

// TODO: 单元测试
