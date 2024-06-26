//! 🆕「推理上下文」
//! * 🎯承载并迁移OpenNARS「记忆区」中的「临时推理状态」变量组
//! * 📄亦仿自OpenNARS 3.x（3.0.4）`DerivationContext`
//! * 📝【2024-05-12 02:17:38】基础数据结构可以借鉴OpenNARS 1.5.8，但涉及「推理」的部分，建议采用OpenNARS 3.0.4的架构来复刻
//!
//! * ♻️【2024-05-22 02:09:10】基本已按照改版重构，但仍需拆分代码到不同文件中
//! * ♻️【2024-06-26 11:47:13】现将按改版OpenNARS架构重写
//!   * 🚩【2024-06-26 11:47:30】仍然可能与旧版不同
#![doc(alias = "derivation_context")]

use navm::output::Output;

use crate::{
    control::Parameters,
    entity::{Concept, RCTask},
    global::{ClockTime, Float},
    language::Term,
    storage::Memory,
};

/// 🆕新的「推理上下文」对象
/// * 📄仿自OpenNARS 3.1.0
pub trait ReasonContext {
    /// 🆕获取记忆区（不可变引用）
    fn memory(&self) -> &Memory;

    /// 🆕访问「当前时间」
    /// * 🎯用于在推理过程中构建「新时间戳」
    /// * ️📝可空性：非空
    /// * 📝可变性：只读
    fn time(&self) -> ClockTime;

    /// 🆕访问「当前超参数」
    /// * 🎯用于在推理过程中构建「新时间戳」（作为「最大长度」参数）
    /// * ️📝可空性：非空
    /// * 📝可变性：只读
    fn parameters(&self) -> &Parameters;

    fn max_evidence_base_length(&self) -> usize {
        self.parameters().maximum_stamp_length
    }

    /// 获取「静默值」
    /// * 🎯在「推理上下文」中无需获取「推理器」`getReasoner`
    /// * ️📝可空性：非空
    /// * 📝可变性：只读
    fn silence_percent(&self) -> Float;

    /// 获取「新任务」的数量
    fn num_new_tasks(&self) -> usize;

    /// 添加「新任务」
    /// * 🎯添加推理导出的任务
    /// * 🚩需要是「共享引用」
    fn add_new_task(&mut self, task_rc: RCTask);

    /// 🆕添加「导出的NAVM输出」
    /// * ⚠️不同于OpenNARS，此处集成NAVM中的 [NARS输出](navm::out::Output) 类型
    /// * 📌同时复刻`addExportString`与`addStringToRecord`两个方法
    #[doc(alias = "add_export_string")]
    #[doc(alias = "add_string_to_record")]
    fn add_output(&mut self, output: Output);

    /// 获取「当前概念」（不可变）
    fn current_concept(&self) -> &Concept;

    /// 获取「当前概念」（可变）
    /// * 📄需要在「概念链接」中使用（添加任务链）
    fn current_concept_mut(&mut self) -> &mut Concept;

    /// 获取「当前词项」
    /// * 🚩获取「当前概念」对应的词项
    fn current_term(&self) -> &Term {
        self.current_concept().term()
    }

    /// 获取「当前任务」（不变）
    /// * 📌共享引用
    ///
    /// # 📄OpenNARS
    ///
    /// The selected task
    fn current_task(&self) -> &RCTask;
    /// 获取「当前任务」（可变）
    /// * 📌共享引用
    fn current_task_mut(&mut self) -> &mut RCTask;

    /// 重置全局状态
    /// * 🚩重置「全局随机数生成器」
    ///
    /// TODO: 功能实装
    #[doc(alias = "init")]
    fn init_global();

    // TODO: 通用功能の默认实现、Core对象
}
