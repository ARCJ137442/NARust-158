//! 有关「推理器报告」或「推理器记录」
//! * 🎯承载原`Memory.report`、`Memory.exportStrings`逻辑
//! * 🎯推理器（原记忆区）输出信息
//! * 🚩【2024-05-06 09:35:37】复用[`navm`]中的「NAVM输出」

use super::Reasoner;
use crate::{entity::Task, util::ToDisplayAndBrief};
use narsese::api::NarseseValue;
use navm::output::Output;
use std::collections::VecDeque;

#[derive(Debug, Clone, Default)]
pub(in super::super) struct ReasonRecorder {
    /// 缓存的NAVM输出
    cached_outputs: VecDeque<Output>,
}

impl ReasonRecorder {
    // /// 长度大小
    // pub fn len_output(&self) -> usize {
    //     self.cached_outputs.len()
    // }

    // /// 是否为空
    // pub fn no_output(&self) -> bool {
    //     self.cached_outputs.is_empty()
    // }

    /// 置入NAVM输出（在末尾）
    pub fn put(&mut self, output: Output) {
        self.cached_outputs.push_back(output)
    }

    /// 取出NAVM输出（在开头）
    /// * ⚠️可能没有（空缓冲区）
    pub fn take(&mut self) -> Option<Output> {
        self.cached_outputs.pop_front()
    }

    /// 清空
    /// * 🎯用于推理器「向外输出并清空内部结果」备用
    ///   * 🚩【2024-05-13 02:13:21】现在直接用`while let Some(output) = self.take()`型语法
    pub fn reset(&mut self) {
        self.cached_outputs.clear()
    }
}

/// 简化「添加常用类型输出」功能
impl ReasonRecorder {
    /// 「注释」输出
    /// * 📌一般用于「推理过程debug记录」
    /// * 🎯快捷生成并使用[`Output::COMMENT`]
    pub fn output_comment(message: impl ToString) -> Output {
        Output::COMMENT {
            content: message.to_string(),
        }
    }

    /// 「错误」输出
    /// * 📌一般用于「提醒用户系统内部错误」
    /// * 🎯快捷生成并使用[`Output::ERROR`]
    pub fn output_error(description: impl ToString) -> Output {
        Output::ERROR {
            description: description.to_string(),
        }
    }

    /// 「信息」输出
    /// * 📌一般用于「反馈告知用户系统状态」
    /// * 🎯快捷生成并使用[`Output::INFO`]
    pub fn output_info(message: impl ToString) -> Output {
        Output::INFO {
            message: message.to_string(),
        }
    }

    /// 「导出结论」输出（任务）
    /// * 📌一般用于「推理导出结论」
    /// * 🎯快捷生成并使用[`Output::OUT`]
    /// * 🚩【2024-06-28 15:41:53】目前统一消息输出格式，仅保留Narsese
    pub fn output_out(narsese: &Task) -> Output {
        Output::OUT {
            // * 🚩此处使用「简短结论」以对齐OpenNARS两位数
            content_raw: format!("Derived: {}", narsese.to_display_brief()),
            narsese: Some(NarseseValue::Task(narsese.to_lexical())),
        }
    }

    /// 「输入任务」输出（任务）
    /// * 📌一般用于「推理导出结论」
    /// * 🎯快捷生成并使用[`Output::IN`]
    /// * 🚩【2024-06-28 15:41:53】目前统一消息输出格式，仅保留Narsese
    pub fn output_in(narsese: &Task) -> Output {
        Output::IN {
            // * 🚩此处使用「简短结论」以对齐OpenNARS两位数
            content: format!("In: {}", narsese.to_display_brief()),
            narsese: Some(NarseseValue::Task(narsese.to_lexical())),
        }
    }
}

/// 为「推理器」扩展方法
impl Reasoner {
    /// 报告输出
    pub fn report(&mut self, output: Output) {
        self.recorder.put(output);
    }

    pub fn report_comment(&mut self, message: impl ToString) {
        self.report(ReasonRecorder::output_comment(message));
    }

    pub fn report_info(&mut self, message: impl ToString) {
        self.report(ReasonRecorder::output_info(message));
    }

    #[doc(alias = "report_input")]
    pub fn report_in(&mut self, narsese: &Task) {
        self.report(ReasonRecorder::output_in(narsese));
    }

    #[doc(alias = "report_derived")]
    pub fn report_out(&mut self, narsese: &Task) {
        self.report(ReasonRecorder::output_out(narsese));
    }

    pub fn report_error(&mut self, description: impl ToString) {
        self.report(ReasonRecorder::output_error(description));
    }
}
