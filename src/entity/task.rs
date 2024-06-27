//! 🎯复刻OpenNARS `nars.entity.Task`
//! * ✅【2024-05-05 21:38:53】基本方法复刻完毕
//! * ♻️【2024-06-21 23:33:24】基于OpenNARS改版再次重写

use super::{BudgetValue, Item, JudgementV1, Sentence, SentenceV1, Token};
use crate::{
    global::{ClockTime, RC},
    inference::{Budget, Evidential},
    util::{RefCount, ToDisplayAndBrief},
};
use nar_dev_utils::join;
use narsese::lexical::{Sentence as LexicalSentence, Task as LexicalTask};

/// 可选的共享指针
/// * 📌类似Java中默认的对象类型
type Orc<T> = Option<RC<T>>;
type OrcRef<'a, T> = Option<&'a RC<T>>;

/// A task to be processed, consists of a Sentence and a BudgetValue
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Task {
    /// The sentence of the Task
    /// * 📝任务的「内容」
    sentence: SentenceV1,

    /// 🆕Item令牌
    token: Token,

    /// 父任务
    /// * 📌采用「共享引用」结构，以便实现「共享派生树」
    ///
    /// # 📄OpenNARS
    ///
    /// Task from which the Task is derived, or null if input
    parent_task: Orc<Task>,

    /// 派生所源自的信念
    ///
    /// # 📄OpenNARS
    ///
    /// Belief from which the Task is derived, or null if derived from a theorem
    parent_belief: Option<JudgementV1>,

    /// 最优解
    /// * 📌需要被迭代性改变
    ///
    /// # 📄OpenNARS
    ///
    /// For Question and Goal: best solution found so far
    best_solution: Option<JudgementV1>,
}

/// 用于实际传递的「任务」共享引用
pub type RCTask = RC<Task>;

/// 构造函数
impl Task {
    /// * 🚩【2024-06-21 23:35:53】对传入的参数「零信任」
    ///   * 💭此处全部传递所有权（除了「父任务」的共享引用），避免意料之外的所有权共享
    pub fn new(
        sentence: SentenceV1,
        budget: BudgetValue,
        parent_task: Orc<Self>,
        parent_belief: Option<JudgementV1>,
        best_solution: Option<JudgementV1>,
    ) -> Self {
        let token = Token::new(sentence.to_key(), budget);
        Self {
            token,
            sentence,
            parent_task,
            parent_belief,
            best_solution,
        }
    }

    pub fn from_input(sentence: SentenceV1, budget: BudgetValue) -> Self {
        Self::new(sentence, budget, None, None, None)
    }

    /// 从「导出结论」构造
    /// * 🚩默认没有「最优解」
    pub fn from_derived(
        sentence: SentenceV1,
        budget: BudgetValue,
        parent_task: Orc<Self>,
        parent_belief: Option<JudgementV1>,
    ) -> Self {
        Self::new(sentence, budget, parent_task, parent_belief, None)
    }
}

// 访问类 方法
impl Task {
    pub fn parent_task(&self) -> OrcRef<Self> {
        self.parent_task.as_ref()
    }

    pub fn parent_belief(&self) -> Option<&JudgementV1> {
        self.parent_belief.as_ref()
    }

    pub fn best_solution(&self) -> Option<&JudgementV1> {
        self.best_solution.as_ref()
    }

    pub fn set_best_solution(&mut self, new_solution: JudgementV1) -> &mut JudgementV1 {
        // * 🚩调试时断言
        debug_assert!(
            self.sentence.is_question(),
            "只有「疑问句」才可能有「最优解」"
        );
        self.best_solution.insert(new_solution)
    }
}

/// 转换到词法Narsese
impl Task {
    pub fn to_lexical(&self) -> LexicalTask {
        let sentence = self.sentence_to_lexical();
        let budget = self.budget_to_lexical();
        LexicalTask { sentence, budget }
    }
}

impl Budget for Task {
    fn priority(&self) -> super::ShortFloat {
        self.token.priority()
    }

    fn __priority_mut(&mut self) -> &mut super::ShortFloat {
        self.token.__priority_mut()
    }

    fn durability(&self) -> super::ShortFloat {
        self.token.durability()
    }

    fn __durability_mut(&mut self) -> &mut super::ShortFloat {
        self.token.__durability_mut()
    }

    fn quality(&self) -> super::ShortFloat {
        self.token.quality()
    }

    fn __quality_mut(&mut self) -> &mut super::ShortFloat {
        self.token.__quality_mut()
    }
}

impl Item for Task {
    fn key(&self) -> &String {
        self.token.key()
    }
}

impl Evidential for Task {
    fn evidential_base(&self) -> &[ClockTime] {
        self.sentence.evidential_base()
    }

    fn creation_time(&self) -> ClockTime {
        self.sentence.creation_time()
    }

    fn stamp_to_lexical(&self) -> narsese::lexical::Stamp {
        self.sentence.stamp_to_lexical()
    }
}

impl ToDisplayAndBrief for Task {
    fn to_display(&self) -> String {
        join! {
            => self.budget_to_display()
            => " "
            => self.key().to_string()
            => " "
            => self.stamp_to_display()
            => if let Some(parent_task) = &self.parent_task {
                let task = parent_task.get_();
                join!{
                    => "  \n from task: ".to_string()
                    => task.to_display_brief()
                }
            } else {"".to_string()}
            => if let Some(parent_belief) = &self.parent_belief {
                join!{
                    => "  \n from belief: ".to_string()
                    => parent_belief.to_display_brief()
                }
            } else {"".to_string()}
            => if let Some(best_solution) = &self.best_solution {
                join!{
                    => "  \n solution: ".to_string()
                    => best_solution.to_display_brief()
                }
            } else {"".to_string()}
        }
    }

    fn to_display_brief(&self) -> String {
        join! {
            => self.budget_to_display_brief()
            => " "
            => self.key()
        }
    }
}

impl Sentence for Task {
    fn sentence_clone(&self) -> impl Sentence {
        self.sentence.sentence_clone()
    }

    fn content(&self) -> &crate::language::Term {
        self.sentence.content()
    }

    fn content_mut(&mut self) -> &mut crate::language::Term {
        self.sentence.content_mut()
    }

    fn punctuation(&self) -> super::Punctuation {
        self.sentence.punctuation()
    }

    type Judgement = <SentenceV1 as Sentence>::Judgement;

    type Question = <SentenceV1 as Sentence>::Question;

    fn as_judgement(&self) -> Option<&Self::Judgement> {
        self.sentence.as_judgement()
    }

    fn as_question(&self) -> Option<&Self::Question> {
        self.sentence.as_question()
    }

    fn to_key(&self) -> String {
        self.sentence.to_key()
    }

    fn sentence_to_display(&self) -> String {
        self.sentence.sentence_to_display()
    }

    fn sentence_to_lexical(&self) -> LexicalSentence {
        self.sentence.sentence_to_lexical()
    }
}
