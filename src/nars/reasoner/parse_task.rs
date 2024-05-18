//! NARS推理器中有关「任务解析」的功能
//! * 🎯结合推理器自身信息，解析外部传入的「词法Narsese任务」

use super::*;
use crate::{
    entity::{
        BudgetValueConcrete, Sentence, SentenceConcrete, SentenceType, ShortFloat, TaskConcrete,
    },
    inference::{BudgetFunctions, ReasonContext},
    io::symbols::JUDGMENT_MARK,
    nars::DEFAULT_PARAMETERS,
};
use anyhow::Result;
use narsese::lexical::Task as LexicalTask;

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
        let LexicalTask { budget, sentence } = narsese;

        // * 🚩解析语句：先解析出「语句」再设置其中的「可修正」属性
        let stamp_current_serial = self.get_stamp_current_serial();
        let stamp_time = self.clock();
        let truth_is_analytic = DEFAULT_PARAMETERS.default_truth_analytic;
        let mut sentence: C::Sentence = SentenceConcrete::from_lexical(
            sentence,
            truth_default_values,
            truth_is_analytic,
            stamp_current_serial,
            stamp_time,
            false, // ! 🚩暂时设置为`false`，后续要通过「解析出来的词项」判断「是否可修正」
        )?;
        let term = sentence.content();
        *sentence.revisable_mut() = !(term.instanceof_conjunction() && term.contain_var_d());

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
