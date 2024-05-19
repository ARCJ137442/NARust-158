//! NARS推理器中有关「任务解析」的功能
//! * 🎯结合推理器自身信息，解析外部传入的「词法Narsese任务」

use super::*;
use crate::{
    entity::{
        BudgetValueConcrete, Sentence, SentenceConcrete, SentenceType, ShortFloat, StampConcrete,
        TaskConcrete,
    },
    inference::{BudgetFunctions, ReasonContext},
    io::symbols::JUDGMENT_MARK,
    language::Term,
    nars::DEFAULT_PARAMETERS,
};
use anyhow::Result;
use narsese::lexical::{Sentence as LexicalSentence, Task as LexicalTask};

pub trait ReasonerParseTask<C: ReasonContext>: Reasoner<C> {
    /// 模拟`StringParser.parseTask`
    /// * 🚩直接模仿`parseTask`而非`parseExperience`
    /// * 📌结合自身信息的「词法折叠」
    /// * 📝OpenNARS在解析时可能会遇到「新词项⇒新建概念」的情形
    ///   * 🚩因此需要`&mut self`
    #[doc(alias = "parse_experience")]
    fn parse_task(&mut self, narsese: LexicalTask) -> Result<C::Task> {
        /* 📄OpenNARS源码：
        StringBuffer buffer = new StringBuffer(s);
        Task task = null;
        try {
            String budgetString = getBudgetString(buffer);
            String truthString = getTruthString(buffer);
            String str = buffer.toString().trim();
            int last = str.length() - 1;
            char punctuation = str.charAt(last);
            Stamp stamp = new Stamp(time);
            TruthValue truth = parseTruth(truthString, punctuation);
            Term content = parseTerm(str.substring(0, last), memory);
            Sentence sentence = new Sentence(content, punctuation, truth, stamp);
            if ((content instanceof Conjunction) && Variable.containVarD(content.getName())) {
                sentence.setRevisable(false);
            }
            BudgetValue budget = parseBudget(budgetString, punctuation, truth);
            task = new Task(sentence, budget);
        } catch (InvalidInputException e) {
            String message = "ERR: !!! INVALID INPUT: parseTask: " + buffer + " --- " + e.getMessage();
            System.out.println(message);
            // showWarning(message);
        }
        return task; */
        // * 🚩判断是要被解析为「判断」还是「问题」
        let is_judgement = narsese.sentence.punctuation == JUDGMENT_MARK;
        // * 🚩生成默认真值与默认预算值
        let zero = ShortFloat::ZERO;
        let truth_default_values = match is_judgement {
            true => [
                ShortFloat::from_float(DEFAULT_PARAMETERS.default_judgement_frequency),
                ShortFloat::from_float(DEFAULT_PARAMETERS.default_judgement_confidence),
            ],
            // * 🚩【2024-05-13 09:44:32】目前「问题」没有真值，所以全部取`0`当占位符
            false => [zero, zero],
        };

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

        // 根据解析出的词项设置「是否可修正」
        // ! 📝这段代码在不同版本间有争议
        // * 📄OpenNARS 3.0.4不再使用`setRevisable`方法，使之变成了【仅构造时可修改】的变量
        let revisable = !(content.instanceof_conjunction() && content.contain_var_d());

        // 时间戳
        let stamp_current_serial = self.get_stamp_current_serial();
        let stamp_time = self.clock();
        let stamp =
            <C::Stamp as StampConcrete>::from_lexical(stamp, stamp_current_serial, stamp_time)?;

        // 标点 & 真值
        let truth_is_analytic = DEFAULT_PARAMETERS.default_truth_analytic;
        let sentence_type = SentenceType::from_lexical(
            punctuation,
            truth,
            truth_default_values,
            truth_is_analytic,
        )?;

        // 构造语句
        let sentence: C::Sentence = SentenceConcrete::new(content, sentence_type, stamp, revisable);

        // * 🚩解析任务

        // 解析预算值：先计算出「默认预算值」再参与「词法解析」（覆盖）
        use SentenceType::*;
        let (priority, durability, quality) = match sentence.punctuation() {
            Judgement(truth) => (
                ShortFloat::from_float(DEFAULT_PARAMETERS.default_judgement_priority),
                ShortFloat::from_float(DEFAULT_PARAMETERS.default_judgement_durability),
                <C::Budget as BudgetFunctions>::truth_to_quality(truth),
            ),
            Question => (
                ShortFloat::from_float(DEFAULT_PARAMETERS.default_question_priority),
                ShortFloat::from_float(DEFAULT_PARAMETERS.default_question_durability),
                ShortFloat::ONE,
            ),
        };
        let default_budget = [priority, durability, quality];
        let budget: C::Budget = BudgetValueConcrete::from_lexical(budget, default_budget)?;

        // 构造任务
        let task = TaskConcrete::from_input(sentence, budget);

        // 返回
        Ok(task)
    }
}

/// 通过「批量实现」自动加功能
impl<C: ReasonContext, T: Reasoner<C>> ReasonerParseTask<C> for T {}
