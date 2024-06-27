//! NARS推理器中有关「任务解析」的功能
//! * 🎯结合推理器自身信息，解析外部传入的「词法Narsese任务」

use crate::{
    control::Reasoner,
    entity::{BudgetValue, Punctuation, SentenceV1, ShortFloat, Stamp, Task, TruthValue},
    global::ClockTime,
    inference::BudgetFunctions,
    language::Term,
};
use anyhow::{anyhow, Result};
use narsese::lexical::{Sentence as LexicalSentence, Task as LexicalTask};

/// 为「推理器」扩展功能
impl Reasoner {
    /// 模拟`StringParser.parseTask`
    /// * 🚩直接模仿`parseTask`而非`parseExperience`
    /// * 📌结合自身信息的「词法折叠」
    /// * 📝OpenNARS在解析时可能会遇到「新词项⇒新建概念」的情形
    ///   * 🚩因此需要`&mut self`
    pub fn parse_task(
        &self,
        narsese: LexicalTask,
        stamp_current_serial: ClockTime,
    ) -> Result<Task> {
        use Punctuation::*;

        // * 📌因为OpenNARS中「前后解析依赖」，所以总需要解构——真值→预算值，词项→语句→任务
        let LexicalTask {
            budget,
            sentence:
                LexicalSentence {
                    term,
                    punctuation,
                    stamp,
                    truth,
                },
        } = narsese;

        // * 🚩解析词项
        let content = Term::try_from(term)?;

        // * 🚩解析语句：解析「语句」新有的内容，再通过解析出的词项组装

        // 时间戳
        let stamp_time = self.time();
        let stamp = Stamp::from_lexical(stamp, stamp_current_serial, stamp_time)?;

        // 标点
        let punctuation = Punctuation::from_lexical(punctuation)?;

        // 真值 & 可被修正
        let truth_revisable = match punctuation {
            // * 🚩判断句 ⇒ 生成真值等附加信息
            Judgement => {
                // * 🚩生成默认真值与默认预算值
                let truth_default_values = [
                    ShortFloat::from_float(self.parameters.default_judgement_frequency),
                    ShortFloat::from_float(self.parameters.default_judgement_confidence),
                ];

                // * 🚩解析真值
                let truth_is_analytic = self.parameters.default_truth_analytic;
                let truth =
                    TruthValue::from_lexical(truth, truth_default_values, truth_is_analytic)?;

                // * 🚩解析「是否可参与修正」
                // 根据解析出的词项设置「是否可修正」
                // ! 📝这段代码在不同版本间有争议
                // * 📄OpenNARS 3.0.4不再使用`setRevisable`方法，使之变成了【仅构造时可修改】的变量
                let revisable = !(content.instanceof_conjunction() && content.contain_var_d());

                Some((truth, revisable))
            }
            // * 🚩疑问句 ⇒ 空
            Question => None,
        };

        // 构造语句
        let sentence = SentenceV1::new_sentence_from_punctuation(
            content,
            punctuation,
            stamp,
            truth_revisable,
        )?;

        // * 🚩解析任务

        // 解析预算值：先计算出「默认预算值」再参与「词法解析」（覆盖）
        let [priority, durability, quality] = match (punctuation, truth_revisable) {
            // * 🚩判断
            (Judgement, Some((truth, _))) => [
                ShortFloat::from_float(self.parameters.default_judgement_priority),
                ShortFloat::from_float(self.parameters.default_judgement_durability),
                BudgetValue::truth_to_quality(&truth),
            ],
            (Judgement, None) => {
                return Err(anyhow!("【少见】在解析出判断句后，解析出的真值不应为空"))
            }
            // * 🚩问题
            (Question, _) => [
                ShortFloat::from_float(self.parameters.default_question_priority),
                ShortFloat::from_float(self.parameters.default_question_durability),
                ShortFloat::ONE,
            ],
        };
        let budget = BudgetValue::from_lexical(budget, [priority, durability, quality])?;

        // 构造任务
        let task = Task::from_input(sentence, budget);

        // 返回
        Ok(task)
    }
}
