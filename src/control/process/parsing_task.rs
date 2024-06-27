//! NARS推理器中有关「任务解析」的功能
//! * 🎯结合推理器自身信息，解析外部传入的「词法Narsese任务」

use crate::{control::Reasoner, entity::Task};
use anyhow::Result;
use narsese::lexical::Task as LexicalTask;

impl Reasoner {
    /// 模拟`StringParser.parseTask`
    /// * 🚩直接模仿`parseTask`而非`parseExperience`
    /// * 📌结合自身信息的「词法折叠」
    /// * 📝OpenNARS在解析时可能会遇到「新词项⇒新建概念」的情形
    ///   * 🚩因此需要`&mut self`
    pub fn parse_task(&mut self, narsese: LexicalTask) -> Result<Task> {
        todo!()
    }
}
