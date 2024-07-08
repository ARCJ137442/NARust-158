//! NARS中有关「推理」的内容
//! * 🚩【2024-05-02 15:54:15】计划通过「全有默认实现的模板特征」作为功能实现方法
//! * ♻️【2024-05-16 14:01:02】将混杂的推理控制过程分类放置
//!   * 🚩与「上下文」有关的放在一块：推理上下文、推理上下文……
//!   * 🚩与「概念」「记忆区」有关的放在一块：概念处理、记忆区处理……
//!   * 🚩与「推理规则」有关的放在一块：本地规则、三段论规则……
//!   * 🚩与「推理函数」有关的放在一块：真值函数、预算函数……
//! * 🚩【2024-05-22 01:35:53】现在将与「推理周期」有关的「推理控制机制」移至[`crate::control`]中
//!   * 📌目前将只留下纯粹的「推理规则」与「推导函数」
//!
//! # 📄OpenNARS
//!
//! The entry point of the package is `RuleTables`, which dispatch the premises (a task, and maybe also a belief) to various rules, according to their type combination.
//!
//! There are four major groups of inference rules:
//!
//! 1. `LocalRules`, where the task and belief contains the same pair of terms, and the rules provide direct solutions to problems, revise beliefs, and derive some conclusions;
//! 2. `SyllogisticRules`, where the task and belief share one common term, and the rules derive conclusions between the other two terms;
//! 3. `CompositionalRules`, where the rules derive conclusions by compose or decompose the terms in premises, so as to form new terms that are not in the two premises;
//! 4. `StructuralRules`, where the task derives conclusions all by itself, while the other "premise" serves by indicating a certain syntactic structure in a compound term.
//!
//! In the system, forward inference (the task is a Judgement) and backward inference (the task is a Question) are mostly isomorphic to each other, so that the inference rules produce conclusions with the same content for different types of tasks. However, there are exceptions. For example, backward inference does not generate compound terms.
//!
//! There are three files containing numerical functions:
//!
//! 1. `TruthFunctions`: the functions that calculate the truth value of the derived judgements and the desire value (a variant of truth value) of the derived goals;
//! 2. `BudgetFunctions`: the functions that calculate the budget value of the derived tasks, as well as adjust the budget value of the involved items (concept, task, and links);
//! 3. `UtilityFunctions`: the common basic functions used by the others.
//!
//! In each case, there may be multiple applicable rules, which will be applied in parallel. For each rule, each conclusion is formed in three stages, to determine (1) the content (as a Term), (2) the truth-value, and (3) the budget-value, roughly in that order.

nar_dev_utils::mods! {
    // ♻️数值函数
    pub use functions;

    // 🛠️预算推理
    pub use budget_inference;

    // 📥本地推理
    pub use local_inference;

    // 🆕特征
    pub use traits; // TODO: 一个更好的模块名

    // 🏗️推理引擎
    pub use engine;

    // ♻️具体规则
    pub use rules;
}

/// 单元测试 通用函数
#[cfg(test)]
pub mod test {
    use super::*;
    use crate::{
        control::{Parameters, DEFAULT_PARAMETERS},
        vm::{Launcher, Runtime},
    };
    use nar_dev_utils::list;
    use narsese::api::GetTerm;
    use navm::{
        cmd::Cmd,
        output::Output,
        vm::{VmLauncher, VmRuntime},
    };

    /// 从「超参数」与「推理引擎」创建虚拟机
    pub fn create_vm(parameters: Parameters, engine: InferenceEngine) -> Runtime {
        let launcher = Launcher::new("test", parameters, engine);
        launcher.launch().expect("推理器虚拟机 启动失败")
    }

    /// 从「推理引擎」创建虚拟机
    /// * 📜使用默认参数
    pub fn create_vm_from_engine(engine: InferenceEngine) -> Runtime {
        create_vm(DEFAULT_PARAMETERS, engine)
    }

    /// 输入NAVM指令到虚拟机
    pub fn input_cmds(vm: &mut impl VmRuntime, cmds: &str) {
        for cmd in cmds
            .lines()
            .map(str::trim)
            .filter(|line| !line.is_empty())
            .map(|line| Cmd::parse(line).expect("NAVM指令{line}解析失败"))
        {
            let cmd_s = cmd.to_string();
            vm.input_cmd(cmd)
                .unwrap_or_else(|_| panic!("NAVM指令「{cmd_s}」输入失败"));
        }
    }

    /// 拉取虚拟机的输出
    pub fn fetch_outputs(vm: &mut impl VmRuntime) -> Vec<Output> {
        list![
            output
            while let Some(output) = (vm.try_fetch_output().expect("拉取输出失败"))
        ]
    }

    /// 输入指令并拉取输出
    #[must_use]
    pub fn input_cmds_and_fetch_out(vm: &mut impl VmRuntime, cmds: &str) -> Vec<Output> {
        input_cmds(vm, cmds);
        fetch_outputs(vm)
    }

    /// 打印输出（基本格式）
    pub fn print_outputs<'a>(outs: impl IntoIterator<Item = &'a Output>) {
        outs.into_iter().for_each(|output| {
            println!(
                "[{}] {} as narsese {:?}",
                output.type_name(),
                output.get_content(),
                output.get_narsese()
            )
        })
    }

    /// 预期输出
    pub fn expect_outputs<'a>(
        outputs: impl IntoIterator<Item = &'a Output>,
        expect: impl Fn(&Output) -> bool,
    ) -> &'a Output {
        outputs
            .into_iter()
            .find(|&output| expect(output))
            .expect("没有找到期望的输出")
    }

    /// 预期输出包含
    /// * 🚩精确匹配指定类型的Narsese**词项**
    pub fn expect_outputs_contains<'a>(
        outputs: impl IntoIterator<Item = &'a Output>,
        expected: impl Into<narsese::lexical::Term>,
    ) -> &'a Output {
        let expected = expected.into();
        outputs
            .into_iter()
            .find(|&output| matches!(output.get_narsese().map(GetTerm::get_term), Some(term) if *term == expected) )
            .expect("没有找到期望的输出")
    }

    /// 拉取输出并预期其中的输出
    pub fn fetch_expected_outputs(
        vm: &mut impl VmRuntime,
        expect: impl Fn(&Output) -> bool,
    ) -> Vec<Output> {
        let outputs = fetch_outputs(vm);
        expect_outputs(&outputs, expect);
        outputs
    }
}
