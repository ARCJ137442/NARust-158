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
    // 🆕特征：真值、预算、证据基……
    pub use traits;

    // ♻️数值函数：真值函数、预算函数……
    pub use functions;

    // 🛠️预算推理：预算×推理上下文
    pub use budget_inference;

    // 📥本地推理：增删信念、答问……
    pub use local_inference;

    // 🏗️推理引擎：可配置推理功能表
    pub use engine;

    // ♻️具体规则：直接推理、转换推理、匹配推理、概念推理……
    pub use rules;
}

/// 单元测试 通用函数
#[cfg(test)]
pub(super) mod test_inference {
    use super::*;
    use crate::{
        control::{Parameters, DEFAULT_PARAMETERS},
        language::Term,
        vm::{Launcher, Runtime},
    };
    use nar_dev_utils::{list, unwrap_or_return};
    use narsese::api::GetTerm;
    use navm::{
        cmd::Cmd,
        output::Output,
        vm::{VmLauncher, VmRuntime},
    };

    /// 预期输出词项相等
    /// * 🎯独立的「输出内容与预期词项判等」方法
    pub fn expect_output_eq_term(output: &Output, expected: &Term) -> bool {
        let lexical_term = unwrap_or_return!(
            ?output.get_narsese().map(GetTerm::get_term).cloned()
            => false // 输出没有词项⇒直接不等
        );
        let out = Term::from_lexical(lexical_term).expect("要预期的词法不正确");
        // 直接判等：使用内置词项类型
        out == *expected
    }

    pub fn expect_output_eq_term_lexical(output: &Output, lexical: narsese::lexical::Term) -> bool {
        let expected = Term::from_lexical(lexical).expect("要预期的词法不正确");
        expect_output_eq_term(output, &expected)
    }

    /// 预期其中的Narsese词项
    #[macro_export]
    macro_rules! expect_narsese_term {
        // * 🚩模式：【类型】 【内容】 in 【输出】
        ($type:ident $term:literal in outputs) => {
            move |output|
                matches!(output, navm::output::Output::$type {..}) // ! 📌【2024-08-07 15:15:22】类型匹配必须放宏展开式中
                && $crate::inference::test_inference::expect_output_eq_term_lexical(
                    // * 🚩【2024-07-15 00:04:43】此处使用了「词法Narsese」的内部分派
                    output, narsese::lexical_nse_term!(@PARSE $term)
                )
        };
    }

    /// 从「超参数」与「推理引擎」创建虚拟机
    pub fn create_vm(parameters: Parameters, engine: InferenceEngine) -> Runtime {
        let launcher = Launcher::new("test", parameters, engine);
        launcher.launch().expect("推理器虚拟机 启动失败")
    }

    /// 设置虚拟机到「最大音量」
    /// * 🎯使虚拟机得以输出尽可能详尽的信息
    pub fn set_max_volume(vm: &mut impl VmRuntime) {
        vm.input_cmd(Cmd::VOL(100)).expect("输入指令失败");
        let _ = vm.try_fetch_output(); // 📌丢掉其输出
    }

    /// 从「推理引擎」创建虚拟机
    /// * 📜使用默认参数
    /// * 🚩【2024-08-01 14:34:19】默认最大音量
    pub fn create_vm_from_engine(engine: InferenceEngine) -> Runtime {
        let mut vm = create_vm(DEFAULT_PARAMETERS, engine);
        set_max_volume(&mut vm);
        vm
    }

    /// 增强虚拟机运行时的特征
    pub trait VmRuntimeBoost: VmRuntime {
        /// 输入NAVM指令到虚拟机
        fn input_cmds(&mut self, cmds: &str) {
            for cmd in cmds
                .lines()
                .map(str::trim)
                .filter(|line| !line.is_empty())
                .map(|line| Cmd::parse(line).expect("NAVM指令{line}解析失败"))
            {
                let cmd_s = cmd.to_string();
                self.input_cmd(cmd)
                    .unwrap_or_else(|_| panic!("NAVM指令「{cmd_s}」输入失败"));
            }
        }

        /// 输入NAVM指令到虚拟机，但忽略解析错误
        /// * 🎯向后兼容：解析成功则必须稳定，解析失败视作「暂未支持」
        fn input_cmds_soft(&mut self, cmds: &str) {
            for cmd in cmds
                .lines()
                .map(str::trim)
                .filter(|line| !line.is_empty())
                .filter_map(|line| Cmd::parse(line).ok())
            // ! 此处不一样：解析失败后不会panic
            {
                let cmd_s = cmd.to_string();
                self.input_cmd(cmd)
                    .unwrap_or_else(|_| eprintln!("【警告】NAVM指令「{cmd_s}」输入失败"));
                // ! 此处不一样：输入失败后不会panic
            }
        }

        /// 拉取虚拟机的输出
        fn fetch_outputs(&mut self) -> Vec<Output> {
            list![
                output
                while let Some(output) = (self.try_fetch_output().expect("拉取输出失败"))
            ]
        }

        /// 输入指令并拉取输出
        #[must_use]
        fn input_cmds_and_fetch_out(&mut self, cmds: &str) -> Vec<Output> {
            self.input_cmds(cmds);
            self.fetch_outputs()
        }

        /// 拉取输出并预期其中的输出
        fn fetch_expected_outputs(&mut self, expect: impl Fn(&Output) -> bool) -> Vec<Output> {
            let outputs = self.fetch_outputs();
            expect_outputs(&outputs, expect);
            outputs
        }

        /// 输入指令、拉取、打印并预期输出
        fn input_fetch_print_expect(
            &mut self,
            cmds: &str,
            expect: impl Fn(&Output) -> bool,
        ) -> Vec<Output> {
            // 输入
            self.input_cmds(cmds);
            // 拉取
            let outs = self.fetch_outputs();
            // 打印
            print_outputs(&outs);
            // 预期
            expect_outputs(&outs, expect);
            // 返回
            outs
        }
    }
    impl<T: VmRuntime> VmRuntimeBoost for T {}

    /// 打印输出（基本格式）
    pub fn print_outputs<'a>(outs: impl IntoIterator<Item = &'a Output>) {
        outs.into_iter().for_each(|output| {
            println!(
                "[{}]{}\nas narsese {:?}\n",
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
    pub fn expect_outputs_contains_term<'a>(
        outputs: impl IntoIterator<Item = &'a Output>,
        expected: impl Into<narsese::lexical::Term>,
    ) -> &'a Output {
        let expected = Term::from_lexical(expected.into()).expect("要预期的词法不正确");
        // 预测：所有输出中至少要有一个
        outputs
            .into_iter()
            .find(|&output| expect_output_eq_term(output, &expected))
            .unwrap_or_else(|| panic!("没有找到期望的输出「{expected}」"))
    }

    /// 概念推理专用测试引擎
    /// * 🚩【2024-07-14 23:51:32】禁掉了转换推理
    pub const ENGINE_REASON: InferenceEngine = InferenceEngine::new(
        process_direct,
        transform_task,
        InferenceEngine::VOID.matching_f(),
        reason,
    );

    /// 「预期测试」函数
    pub fn expectation_test(inputs: impl AsRef<str>, expectation: impl Fn(&Output) -> bool) {
        let mut vm = create_vm_from_engine(ENGINE_REASON);
        // * 🚩OUT
        vm.input_fetch_print_expect(
            inputs.as_ref(),
            // * 🚩检查其中是否有导出
            expectation,
        );
    }

    /// 一个「单输出预期」测试
    #[macro_export]
    macro_rules! expectation_test {
        (
            $(#[$attr:meta])*
            $name:ident :
            $inputs:expr
            => $($expectations:tt)*
        ) => {
            $(#[$attr])*
            #[test]
            fn $name() {
                $crate::inference::test_inference::expectation_test(
                    $inputs,
                    // * 🚩检查其中是否有预期输出
                    $crate::expect_narsese_term!($($expectations)*),
                )
            }
        };
    }

    /// 一组「单输出预期」测试
    #[macro_export]
    macro_rules! expectation_tests {
        (
            $(
                $(#[$attr:meta])*
                $name:ident : {
                    $inputs:expr
                    => $($expectations:tt)*
                }
            )*
        ) => {
            $(
                $crate::expectation_test! {
                    $(#[$attr])*
                    $name :
                        $inputs
                        => $($expectations)*
                }
            )*
        };
    }
}

/// 总体性测试
/// * 📌长期稳定性、逻辑稳定性
///   * 🎯不在运行时panic
#[cfg(test)]
mod tests {
    use super::*;
    use crate::inference::test_inference::{create_vm_from_engine, print_outputs, VmRuntimeBoost};
    use crate::{ok, util::AResult};
    use nar_dev_utils::{pipe, JoinTo};

    /// 引擎dev
    /// * 🚩【2024-07-09 16:52:40】目前除了「概念推理」均俱全
    /// * ✅【2024-07-14 23:50:15】现集成所有四大推理函数
    const ENGINE_DEV: InferenceEngine = InferenceEngine::new(
        process_direct,
        transform_task,
        match_task_and_belief,
        reason,
    );

    /// 测试多行NAVM指令（文本形式）输入
    /// * 🚩仅测试文本输入（稳定性），不负责捕获输出等额外操作
    fn test_line_inputs(inputs: impl AsRef<str>) -> AResult {
        // 创建
        let mut runtime = create_vm_from_engine(ENGINE_DEV);
        // 输入指令（软标准，不要求解析成功⇒向后兼容）
        runtime.input_cmds_soft(inputs.as_ref());
        // 打印推理器概要
        let _ = runtime.fetch_outputs(); // 丢掉先前的输出
        pipe! {
            "inf summary" // 指令
            => [runtime.input_cmds_and_fetch_out] // 输入
            => .iter() => print_outputs // 打印输出
        }
        // 完
        ok!()
    }

    /// 集成测试：长期稳定性
    /// * 🎯推理器在大量词项与任务的基础上，保持运行不panic
    #[test]
    fn long_term_stability() -> AResult {
        test_line_inputs(
            r#"
            nse <{tim} --> (/,livingIn,_,{graz})>. %0%
            cyc 100
            nse <<(*,$1,sunglasses) --> own> ==> <$1 --> [aggressive]>>.
            nse <(*,{tom},sunglasses) --> own>.
            nse <<$1 --> [aggressive]> ==> <$1 --> murder>>.
            nse <<$1 --> (/,livingIn,_,{graz})> ==> <$1 --> murder>>.
            nse <{?who} --> murder>?
            nse <{tim} --> (/,livingIn,_,{graz})>.
            nse <{tim} --> (/,livingIn,_,{graz})>. %0%
            cyc 100
            nse <<(*,$1,sunglasses) --> own> ==> <$1 --> [aggressive]>>.
            nse <(*,{tom},(&,[black],glasses)) --> own>.
            nse <<$1 --> [aggressive]> ==> <$1 --> murder>>.
            nse <<$1 --> (/,livingIn,_,{graz})> ==> <$1 --> murder>>.
            nse <sunglasses --> (&,[black],glasses)>.
            nse <{?who} --> murder>?
            nse <(*,toothbrush,plastic) --> made_of>.
            nse <(&/,<(*,$1,plastic) --> made_of>,(^lighter,{SELF},$1)) =/> <$1 --> [heated]>>.
            nse <<$1 --> [heated]> =/> <$1 --> [melted]>>.
            nse <<$1 --> [melted]> <|> <$1 --> [pliable]>>.
            nse <(&/,<$1 --> [pliable]>,(^reshape,{SELF},$1)) =/> <$1 --> [hardened]>>.
            nse <<$1 --> [hardened]> =|> <$1 --> [unscrewing]>>.
            nse <toothbrush --> object>.
            nse (&&,<#1 --> object>,<#1 --> [unscrewing]>)!
            nse <{SELF} --> [hurt]>! %0%
            nse <{SELF} --> [hurt]>. :|: %0%
            nse <(&/,<(*,{SELF},wolf) --> close_to>,+1000) =/> <{SELF} --> [hurt]>>.
            nse <(*,{SELF},wolf) --> close_to>. :|:
            nse <(&|,(^want,{SELF},$1,FALSE),(^anticipate,{SELF},$1)) =|> <(*,{SELF},$1) --> afraid_of>>.
            nse <(*,{SELF},?what) --> afraid_of>?
            nse <a --> A>. :|: %1.00;0.90%
            cyc 8
            nse <b --> B>. :|: %1.00;0.90%
            cyc 8
            nse <c --> C>. :|: %1.00;0.90%
            cyc 8
            nse <a --> A>. :|: %1.00;0.90%
            cyc 100
            nse <b --> B>. :|: %1.00;0.90%
            cyc 100
            nse <?1 =/> <c --> C>>?
            nse <(*,cup,plastic) --> made_of>.
            nse <cup --> object>.
            nse <cup --> [bendable]>.
            nse <toothbrush --> [bendable]>.
            nse <toothbrush --> object>.
            nse <(&/,<(*,$1,plastic) --> made_of>,(^lighter,{SELF},$1)) =/> <$1 --> [heated]>>.
            nse <<$1 --> [heated]> =/> <$1 --> [melted]>>.
            nse <<$1 --> [melted]> <|> <$1 --> [pliable]>>.
            nse <(&/,<$1 --> [pliable]>,(^reshape,{SELF},$1)) =/> <$1 --> [hardened]>>.
            nse <<$1 --> [hardened]> =|> <$1 --> [unscrewing]>>.
            nse (&&,<#1 --> object>,<#1 --> [unscrewing]>)!
            cyc 2000"#,
        )
    }

    /// 「逻辑稳定性」中的NAL测试（源自OpenNARS测试用例）
    const NAL_1_0: &str = r"
        nse $0.80;0.80;0.95$ <bird --> swimmer>. %1.00;0.90%
        nse $0.80;0.80;0.95$ <bird --> swimmer>. %0.10;0.60%";

    /// 「逻辑稳定性」中的NAL测试（源自OpenNARS测试用例）
    const NAL_1_1: &str = r"
        nse $0.80;0.80;0.95$ <bird --> animal>. %1.00;0.90%
        nse $0.80;0.80;0.95$ <robin --> bird>. %1.00;0.90%";

    /// 「逻辑稳定性」中的NAL测试（源自OpenNARS测试用例）
    const NAL_1_2: &str = r"
        nse $0.80;0.80;0.95$ <sport --> competition>. %1.00;0.90%
        nse $0.80;0.80;0.95$ <chess --> competition>. %0.90;0.90%";

    /// 「逻辑稳定性」中的NAL测试（源自OpenNARS测试用例）
    const NAL_1_3: &str = r"
        nse $0.80;0.80;0.95$ <swan --> swimmer>. %0.90;0.90%
        nse $0.80;0.80;0.95$ <swan --> bird>. %1.00;0.90%";

    /// 「逻辑稳定性」中的NAL测试（源自OpenNARS测试用例）
    const NAL_1_4: &str = r"
        nse $0.80;0.80;0.95$ <robin --> bird>. %1.00;0.90%
        nse $0.80;0.80;0.95$ <bird --> animal>. %1.00;0.90%";

    /// 「逻辑稳定性」中的NAL测试（源自OpenNARS测试用例）
    const NAL_1_5: &str = r"
        nse $0.80;0.80;0.95$ <bird --> swimmer>. %1.00;0.90%
        nse $0.90;0.80;1.00$ <swimmer --> bird>?";

    /// 「逻辑稳定性」中的NAL测试（源自OpenNARS测试用例）
    const NAL_1_6: &str = r"
        nse $0.80;0.80;0.95$ <bird --> swimmer>. %1.00;0.90%
        nse $0.90;0.80;1.00$ <bird --> swimmer>?";

    /// 「逻辑稳定性」中的NAL测试（源自OpenNARS测试用例）
    const NAL_1_7: &str = r"
        nse $0.80;0.80;0.95$ <bird --> swimmer>. %1.00;0.80%
        nse $0.90;0.80;1.00$ <?x --> swimmer>?";

    /// 「逻辑稳定性」中的NAL测试（源自OpenNARS测试用例）
    const NAL_1_8: &str = r"
        nse $0.80;0.80;0.95$ <bird --> swimmer>. %1.00;0.80%
        nse $0.90;0.80;1.00$ <?1 --> swimmer>?";

    /// 「逻辑稳定性」中的NAL测试（源自OpenNARS测试用例）
    const NAL_2_0: &str = r"
        nse $0.80;0.80;0.95$ <robin <-> swan>. %1.00;0.90%
        nse $0.80;0.80;0.95$ <robin <-> swan>. %0.10;0.60%";

    /// 「逻辑稳定性」中的NAL测试（源自OpenNARS测试用例）
    const NAL_2_1: &str = r"
        nse $0.80;0.80;0.95$ <swan --> swimmer>. %0.90;0.90%
        nse $0.80;0.80;0.95$ <swan --> bird>. %1.00;0.90%";

    /// 「逻辑稳定性」中的NAL测试（源自OpenNARS测试用例）
    const NAL_2_10: &str = r"
        nse $0.80;0.80;0.95$ <Birdie <-> Tweety>. %0.90;0.90%
        nse $0.90;0.80;1.00$ <{Birdie} <-> {Tweety}>?";

    /// 「逻辑稳定性」中的NAL测试（源自OpenNARS测试用例）
    const NAL_2_11: &str = r"
        nse $0.80;0.80;0.95$ <swan --> bird>. %0.90;0.90%
        nse $0.90;0.80;1.00$ <bird <-> swan>?";

    /// 「逻辑稳定性」中的NAL测试（源自OpenNARS测试用例）
    const NAL_2_12: &str = r"
        nse $0.80;0.80;0.95$ <bird <-> swan>. %0.90;0.90%
        nse $0.90;0.80;1.00$ <swan --> bird>?";

    /// 「逻辑稳定性」中的NAL测试（源自OpenNARS测试用例）
    const NAL_2_13: &str = r"
        nse $0.80;0.80;0.95$ <Tweety {-- bird>. %1.00;0.90%";

    /// 「逻辑稳定性」中的NAL测试（源自OpenNARS测试用例）
    const NAL_2_14: &str = r"
        nse $0.80;0.80;0.95$ <raven --] black>. %1.00;0.90%";

    /// 「逻辑稳定性」中的NAL测试（源自OpenNARS测试用例）
    const NAL_2_15: &str = r"
        nse $0.80;0.80;0.95$ <Tweety {-] yellow>. %1.00;0.90%";

    /// 「逻辑稳定性」中的NAL测试（源自OpenNARS测试用例）
    const NAL_2_16: &str = r"
        nse $0.80;0.80;0.95$ <{Tweety} --> {Birdie}>. %1.00;0.90%";

    /// 「逻辑稳定性」中的NAL测试（源自OpenNARS测试用例）
    const NAL_2_17: &str = r"
        nse $0.80;0.80;0.95$ <[smart] --> [bright]>. %1.00;0.90%";

    /// 「逻辑稳定性」中的NAL测试（源自OpenNARS测试用例）
    const NAL_2_18: &str = r"
        nse $0.80;0.80;0.95$ <{Birdie} <-> {Tweety}>. %1.00;0.90%";

    /// 「逻辑稳定性」中的NAL测试（源自OpenNARS测试用例）
    const NAL_2_19: &str = r"
        nse $0.80;0.80;0.95$ <[bright] <-> [smart]>. %1.00;0.90%";

    /// 「逻辑稳定性」中的NAL测试（源自OpenNARS测试用例）
    const NAL_2_2: &str = r"
        nse $0.80;0.80;0.95$ <bird --> swimmer>. %1.00;0.90%
        nse $0.90;0.80;1.00$ <{?1} --> swimmer>?";

    /// 「逻辑稳定性」中的NAL测试（源自OpenNARS测试用例）
    const NAL_2_3: &str = r"
        nse $0.80;0.80;0.95$ <sport --> competition>. %1.00;0.90%
        nse $0.80;0.80;0.95$ <chess --> competition>. %0.90;0.90%";

    /// 「逻辑稳定性」中的NAL测试（源自OpenNARS测试用例）
    const NAL_2_4: &str = r"
        nse $0.80;0.80;0.95$ <swan --> swimmer>. %1.00;0.90%
        nse $0.80;0.80;0.95$ <gull <-> swan>. %1.00;0.90%";

    /// 「逻辑稳定性」中的NAL测试（源自OpenNARS测试用例）
    const NAL_2_5: &str = r"
        nse $0.80;0.80;0.95$ <gull --> swimmer>. %1.00;0.90%
        nse $0.80;0.80;0.95$ <gull <-> swan>. %1.00;0.90%";

    /// 「逻辑稳定性」中的NAL测试（源自OpenNARS测试用例）
    const NAL_2_6: &str = r"
        nse $0.80;0.80;0.95$ <robin <-> swan>. %1.00;0.90%
        nse $0.80;0.80;0.95$ <gull <-> swan>. %1.00;0.90%";

    /// 「逻辑稳定性」中的NAL测试（源自OpenNARS测试用例）
    const NAL_2_7: &str = r"
        nse $0.80;0.80;0.95$ <swan --> bird>. %1.00;0.90%
        nse $0.80;0.80;0.95$ <bird --> swan>. %0.10;0.90%";

    /// 「逻辑稳定性」中的NAL测试（源自OpenNARS测试用例）
    const NAL_2_8: &str = r"
        nse $0.80;0.80;0.95$ <bright <-> smart>. %0.90;0.90%
        nse $0.90;0.80;1.00$ <[smart] --> [bright]>?";

    /// 「逻辑稳定性」中的NAL测试（源自OpenNARS测试用例）
    const NAL_2_9: &str = r"
        nse $0.80;0.80;0.95$ <swan --> bird>. %0.90;0.90%
        nse $0.80;0.80;0.95$ <bird <-> swan>. %0.10;0.90%";

    /// 「逻辑稳定性」中的NAL测试（源自OpenNARS测试用例）
    const NAL_3_0: &str = r"
        nse $0.80;0.80;0.95$ <swan --> swimmer>. %0.90;0.90%
        nse $0.80;0.80;0.95$ <swan --> bird>. %0.80;0.90%";

    /// 「逻辑稳定性」中的NAL测试（源自OpenNARS测试用例）
    const NAL_3_1: &str = r"
        nse $0.80;0.80;0.95$ <sport --> competition>. %0.90;0.90%
        nse $0.80;0.80;0.95$ <chess --> competition>. %0.80;0.90%";

    /// 「逻辑稳定性」中的NAL测试（源自OpenNARS测试用例）
    const NAL_3_10: &str = r"
        nse $0.80;0.80;0.95$ <swan --> bird>. %0.90;0.90%
        nse $0.90;0.80;1.00$ <swan --> (-,swimmer,bird)>?";

    /// 「逻辑稳定性」中的NAL测试（源自OpenNARS测试用例）
    const NAL_3_11: &str = r"
        nse $0.80;0.80;0.95$ <swan --> bird>. %0.90;0.90%
        nse $0.90;0.80;1.00$ <(~,swimmer,swan) --> bird>?";

    /// 「逻辑稳定性」中的NAL测试（源自OpenNARS测试用例）
    const NAL_3_12: &str = r"
        nse $0.80;0.80;0.95$ <robin --> (&,bird,swimmer)>. %0.90;0.90%";

    /// 「逻辑稳定性」中的NAL测试（源自OpenNARS测试用例）
    const NAL_3_13: &str = r"
        nse $0.80;0.80;0.95$ <robin --> (-,bird,swimmer)>. %0.90;0.90%";

    /// 「逻辑稳定性」中的NAL测试（源自OpenNARS测试用例）
    const NAL_3_14: &str = r"
        nse $0.80;0.80;0.95$ <(|,boy,girl) --> youth>. %0.90;0.90%";

    /// 「逻辑稳定性」中的NAL测试（源自OpenNARS测试用例）
    const NAL_3_15: &str = r"
        nse $0.80;0.80;0.95$ <(~,boy,girl) --> [strong]>. %0.90;0.90%";

    /// 「逻辑稳定性」中的NAL测试（源自OpenNARS测试用例）
    const NAL_3_2: &str = r"
        nse $0.80;0.80;0.95$ <robin --> (|,bird,swimmer)>. %1.00;0.90%
        nse $0.80;0.80;0.95$ <robin --> swimmer>. %0.00;0.90%";

    /// 「逻辑稳定性」中的NAL测试（源自OpenNARS测试用例）
    const NAL_3_3: &str = r"
        nse $0.80;0.80;0.95$ <robin --> swimmer>. %0.00;0.90%
        nse $0.80;0.80;0.95$ <robin --> (-,mammal,swimmer)>. %0.00;0.90%";

    /// 「逻辑稳定性」中的NAL测试（源自OpenNARS测试用例）
    const NAL_3_4: &str = r"
        nse $0.80;0.80;0.95$ <planetX --> {Mars,Pluto,Venus}>. %0.90;0.90%
        nse $0.80;0.80;0.95$ <planetX --> {Pluto,Saturn}>. %0.70;0.90%";

    /// 「逻辑稳定性」中的NAL测试（源自OpenNARS测试用例）
    const NAL_3_5: &str = r"
        nse $0.80;0.80;0.95$ <planetX --> {Mars,Pluto,Venus}>. %0.90;0.90%
        nse $0.80;0.80;0.95$ <planetX --> {Pluto,Saturn}>. %0.10;0.90%";

    /// 「逻辑稳定性」中的NAL测试（源自OpenNARS测试用例）
    const NAL_3_6: &str = r"
        nse $0.80;0.80;0.95$ <bird --> animal>. %0.90;0.90%
        nse $0.90;0.80;1.00$ <(&,bird,swimmer) --> (&,animal,swimmer)>?";

    /// 「逻辑稳定性」中的NAL测试（源自OpenNARS测试用例）
    const NAL_3_7: &str = r"
        nse $0.80;0.80;0.95$ <bird --> animal>. %0.90;0.90%
        nse $0.90;0.80;1.00$ <(-,swimmer,animal) --> (-,swimmer,bird)>?";

    /// 「逻辑稳定性」中的NAL测试（源自OpenNARS测试用例）
    const NAL_3_8: &str = r"
        nse $0.80;0.80;0.95$ <swan --> bird>. %0.90;0.90%
        nse $0.90;0.80;1.00$ <swan --> (|,bird,swimmer)>?";

    /// 「逻辑稳定性」中的NAL测试（源自OpenNARS测试用例）
    const NAL_3_9: &str = r"
        nse $0.80;0.80;0.95$ <swan --> bird>. %0.90;0.90%
        nse $0.90;0.80;1.00$ <(&,swan,swimmer) --> bird>?";

    /// 「逻辑稳定性」中的NAL测试（源自OpenNARS测试用例）
    const NAL_4_0: &str = r"
        nse $0.80;0.80;0.95$ <(*,acid,base) --> reaction>. %1.00;0.90%";

    /// 「逻辑稳定性」中的NAL测试（源自OpenNARS测试用例）
    const NAL_4_1: &str = r"
        nse $0.80;0.80;0.95$ <acid --> (/,reaction,_,base)>. %1.00;0.90%";

    /// 「逻辑稳定性」中的NAL测试（源自OpenNARS测试用例）
    const NAL_4_2: &str = r"
        nse $0.80;0.80;0.95$ <base --> (/,reaction,acid,_)>. %1.00;0.90%";

    /// 「逻辑稳定性」中的NAL测试（源自OpenNARS测试用例）
    const NAL_4_3: &str = r"
        nse $0.80;0.80;0.95$ <neutralization --> (*,acid,base)>. %1.00;0.90%";

    /// 「逻辑稳定性」中的NAL测试（源自OpenNARS测试用例）
    const NAL_4_4: &str = r"
        nse $0.80;0.80;0.95$ <(\,neutralization,_,base) --> acid>. %1.00;0.90%";

    /// 「逻辑稳定性」中的NAL测试（源自OpenNARS测试用例）
    const NAL_4_5: &str = r"
        nse $0.80;0.80;0.95$ <(\,neutralization,acid,_) --> base>. %1.00;0.90%";

    /// 「逻辑稳定性」中的NAL测试（源自OpenNARS测试用例）
    const NAL_4_6: &str = r"
        nse $0.80;0.80;0.95$ <bird --> animal>. %1.00;0.90%
        nse $0.90;0.80;1.00$ <(*,bird,plant) --> ?x>?";

    /// 「逻辑稳定性」中的NAL测试（源自OpenNARS测试用例）
    const NAL_4_7: &str = r"
        nse $0.80;0.80;0.95$ <neutralization --> reaction>. %1.00;0.90%
        nse $0.90;0.80;1.00$ <(\,neutralization,acid,_) --> ?x>?";

    /// 「逻辑稳定性」中的NAL测试（源自OpenNARS测试用例）
    const NAL_4_8: &str = r"
        nse $0.80;0.80;0.95$ <soda --> base>. %1.00;0.90%
        nse $0.90;0.80;1.00$ <(/,neutralization,_,base) --> ?x>?";

    /// 「逻辑稳定性」中的NAL测试（源自OpenNARS测试用例）
    const NAL_5_0: &str = r"
        nse $0.80;0.80;0.95$ <<robin --> [flying]> ==> <robin --> bird>>. %1.00;0.90%
        nse $0.80;0.80;0.95$ <<robin --> [flying]> ==> <robin --> bird>>. %0.00;0.60%";

    /// 「逻辑稳定性」中的NAL测试（源自OpenNARS测试用例）
    const NAL_5_1: &str = r"
        nse $0.80;0.80;0.95$ <<robin --> bird> ==> <robin --> animal>>. %1.00;0.90%
        nse $0.80;0.80;0.95$ <<robin --> [flying]> ==> <robin --> bird>>. %1.00;0.90%";

    /// 「逻辑稳定性」中的NAL测试（源自OpenNARS测试用例）
    const NAL_5_10: &str = r"
        nse $0.80;0.80;0.95$ <robin --> bird>. %1.00;0.90%
        nse $0.80;0.80;0.95$ <<robin --> bird> <=> <robin --> [flying]>>. %0.80;0.90%";

    /// 「逻辑稳定性」中的NAL测试（源自OpenNARS测试用例）
    const NAL_5_11: &str = r"
        nse $0.80;0.80;0.95$ <<robin --> animal> <=> <robin --> bird>>. %1.00;0.90%
        nse $0.80;0.80;0.95$ <<robin --> bird> <=> <robin --> [flying]>>. %0.90;0.90%";

    /// 「逻辑稳定性」中的NAL测试（源自OpenNARS测试用例）
    const NAL_5_12: &str = r"
        nse $0.80;0.80;0.95$ <<robin --> [flying]> ==> <robin --> bird>>. %0.90;0.90%
        nse $0.80;0.80;0.95$ <<robin --> bird> ==> <robin --> [flying]>>. %0.90;0.90%";

    /// 「逻辑稳定性」中的NAL测试（源自OpenNARS测试用例）
    const NAL_5_13: &str = r"
        nse $0.80;0.80;0.95$ <<robin --> bird> ==> <robin --> animal>>. %1.00;0.90%
        nse $0.80;0.80;0.95$ <<robin --> bird> ==> <robin --> [flying]>>. %0.90;0.90%";

    /// 「逻辑稳定性」中的NAL测试（源自OpenNARS测试用例）
    const NAL_5_14: &str = r"
        nse $0.80;0.80;0.95$ <<robin --> bird> ==> <robin --> animal>>. %1.00;0.90%
        nse $0.80;0.80;0.95$ <<robin --> [flying]> ==> <robin --> animal>>. %0.90;0.90%";

    /// 「逻辑稳定性」中的NAL测试（源自OpenNARS测试用例）
    const NAL_5_15: &str = r"
        nse $0.80;0.80;0.95$ <<robin --> bird> ==> (&&,<robin --> animal>,<robin --> [flying]>)>. %0.00;0.90%
        nse $0.80;0.80;0.95$ <<robin --> bird> ==> <robin --> [flying]>>. %1.00;0.90%";

    /// 「逻辑稳定性」中的NAL测试（源自OpenNARS测试用例）
    const NAL_5_16: &str = r"
        nse $0.80;0.80;0.95$ (&&,<robin --> [flying]>,<robin --> swimmer>). %0.00;0.90%
        nse $0.80;0.80;0.95$ <robin --> [flying]>. %1.00;0.90%";

    /// 「逻辑稳定性」中的NAL测试（源自OpenNARS测试用例）
    const NAL_5_17: &str = r"
        nse $0.80;0.80;0.95$ (||,<robin --> [flying]>,<robin --> swimmer>). %1.00;0.90%
        nse $0.80;0.80;0.95$ <robin --> swimmer>. %0.00;0.90%";

    /// 「逻辑稳定性」中的NAL测试（源自OpenNARS测试用例）
    const NAL_5_18: &str = r"
        nse $0.80;0.80;0.95$ <robin --> [flying]>. %1.00;0.90%
        nse $0.90;0.80;1.00$ (||,<robin --> [flying]>,<robin --> swimmer>)?";

    /// 「逻辑稳定性」中的NAL测试（源自OpenNARS测试用例）
    const NAL_5_19: &str = r"
        nse $0.90;0.90;0.86$ (&&,<robin --> swimmer>,<robin --> [flying]>). %0.90;0.90%";

    /// 「逻辑稳定性」中的NAL测试（源自OpenNARS测试用例）
    const NAL_5_2: &str = r"
        nse $0.80;0.80;0.95$ <<robin --> [flying]> ==> <robin --> bird>>. %1.00;0.90%
        nse $0.80;0.80;0.95$ <<robin --> bird> ==> <robin --> animal>>. %1.00;0.90%";

    /// 「逻辑稳定性」中的NAL测试（源自OpenNARS测试用例）
    const NAL_5_20: &str = r"
        nse $0.80;0.80;0.95$ (--,<robin --> [flying]>). %0.10;0.90%";

    /// 「逻辑稳定性」中的NAL测试（源自OpenNARS测试用例）
    const NAL_5_21: &str = r"
        nse $0.80;0.80;0.95$ <robin --> [flying]>. %0.90;0.90%
        nse $0.90;0.80;1.00$ (--,<robin --> [flying]>)?";

    /// 「逻辑稳定性」中的NAL测试（源自OpenNARS测试用例）
    const NAL_5_22: &str = r"
        nse $0.80;0.80;0.95$ <(--,<robin --> bird>) ==> <robin --> [flying]>>. %0.10;0.90%
        nse $0.90;0.80;1.00$ <(--,<robin --> [flying]>) ==> <robin --> bird>>?";

    /// 「逻辑稳定性」中的NAL测试（源自OpenNARS测试用例）
    const NAL_5_23: &str = r"
        nse $0.80;0.80;0.95$ <(&&,<robin --> [flying]>,<robin --> [with_wings]>) ==> <robin --> bird>>. %1.00;0.90%
        nse $0.80;0.80;0.95$ <robin --> [flying]>. %1.00;0.90%";

    /// 「逻辑稳定性」中的NAL测试（源自OpenNARS测试用例）
    const NAL_5_24: &str = r"
        nse $0.80;0.80;0.95$ <(&&,<robin --> [chirping]>,<robin --> [flying]>,<robin --> [with_wings]>) ==> <robin --> bird>>. %1.00;0.90%
        nse $0.80;0.80;0.95$ <robin --> [flying]>. %1.00;0.90%";

    /// 「逻辑稳定性」中的NAL测试（源自OpenNARS测试用例）
    const NAL_5_25: &str = r"
        nse $0.80;0.80;0.95$ <(&&,<robin --> bird>,<robin --> [living]>) ==> <robin --> animal>>. %1.00;0.90%
        nse $0.80;0.80;0.95$ <<robin --> [flying]> ==> <robin --> bird>>. %1.00;0.90%";

    /// 「逻辑稳定性」中的NAL测试（源自OpenNARS测试用例）
    const NAL_5_26: &str = r"
        nse $0.80;0.80;0.95$ <<robin --> [flying]> ==> <robin --> bird>>. %1.00;0.90%
        nse $0.80;0.80;0.95$ <(&&,<robin --> swimmer>,<robin --> [flying]>) ==> <robin --> bird>>. %1.00;0.90%";

    /// 「逻辑稳定性」中的NAL测试（源自OpenNARS测试用例）
    const NAL_5_27: &str = r"
        nse $0.80;0.80;0.95$ <(&&,<robin --> [with_wings]>,<robin --> [chirping]>) ==> <robin --> bird>>. %1.00;0.90%
        nse $0.80;0.80;0.95$ <(&&,<robin --> [flying]>,<robin --> [with_wings]>,<robin --> [chirping]>) ==> <robin --> bird>>. %1.00;0.90%";

    /// 「逻辑稳定性」中的NAL测试（源自OpenNARS测试用例）
    const NAL_5_28: &str = r"
        nse $0.80;0.80;0.95$ <(&&,<robin --> [flying]>,<robin --> [with_wings]>) ==> <robin --> [living]>>. %0.90;0.90%
        nse $0.80;0.80;0.95$ <(&&,<robin --> [flying]>,<robin --> bird>) ==> <robin --> [living]>>. %1.00;0.90%";

    /// 「逻辑稳定性」中的NAL测试（源自OpenNARS测试用例）
    const NAL_5_29: &str = r"
        nse $0.80;0.80;0.95$ <(&&,<robin --> [chirping]>,<robin --> [flying]>) ==> <robin --> bird>>. %1.00;0.90%
        nse $0.80;0.80;0.95$ <<robin --> [flying]> ==> <robin --> [with_beak]>>. %0.90;0.90%";

    /// 「逻辑稳定性」中的NAL测试（源自OpenNARS测试用例）
    const NAL_5_3: &str = r"
        nse $0.80;0.80;0.95$ <<robin --> bird> ==> <robin --> animal>>. %1.00;0.90%
        nse $0.80;0.80;0.95$ <<robin --> bird> ==> <robin --> [flying]>>. %0.80;0.90%";

    /// 「逻辑稳定性」中的NAL测试（源自OpenNARS测试用例）
    const NAL_5_4: &str = r"
        nse $0.80;0.80;0.95$ <<robin --> bird> ==> <robin --> animal>>. %1.00;0.90%
        nse $0.80;0.80;0.95$ <<robin --> [flying]> ==> <robin --> animal>>. %0.80;0.90%";

    /// 「逻辑稳定性」中的NAL测试（源自OpenNARS测试用例）
    const NAL_5_5: &str = r"
        nse $0.80;0.80;0.95$ <<robin --> bird> ==> <robin --> animal>>. %1.00;0.90%
        nse $0.80;0.80;0.95$ <robin --> bird>. %1.00;0.90%";

    /// 「逻辑稳定性」中的NAL测试（源自OpenNARS测试用例）
    const NAL_5_6: &str = r"
        nse $0.80;0.80;0.95$ <<robin --> bird> ==> <robin --> animal>>. %0.70;0.90%
        nse $0.80;0.80;0.95$ <robin --> animal>. %1.00;0.90%";

    /// 「逻辑稳定性」中的NAL测试（源自OpenNARS测试用例）
    const NAL_5_7: &str = r"
        nse $0.80;0.80;0.95$ <<robin --> bird> ==> <robin --> animal>>. %1.00;0.90%
        nse $0.80;0.80;0.95$ <<robin --> bird> ==> <robin --> [flying]>>. %0.80;0.90%";

    /// 「逻辑稳定性」中的NAL测试（源自OpenNARS测试用例）
    const NAL_5_8: &str = r"
        nse $0.80;0.80;0.95$ <<robin --> bird> ==> <robin --> animal>>. %0.70;0.90%
        nse $0.80;0.80;0.95$ <<robin --> [flying]> ==> <robin --> animal>>. %1.00;0.90%";

    /// 「逻辑稳定性」中的NAL测试（源自OpenNARS测试用例）
    const NAL_5_9: &str = r"
        nse $0.80;0.80;0.95$ <<robin --> bird> ==> <robin --> animal>>. %1.00;0.90%
        nse $0.80;0.80;0.95$ <<robin --> bird> <=> <robin --> [flying]>>. %0.80;0.90%";

    /// 「逻辑稳定性」中的NAL测试（源自OpenNARS测试用例）
    const NAL_6_0: &str = r"
        nse $0.80;0.80;0.95$ <<$x --> bird> ==> <$x --> flyer>>. %1.00;0.90%
        nse $0.80;0.80;0.95$ <<$y --> bird> ==> <$y --> flyer>>. %0.00;0.70%";

    /// 「逻辑稳定性」中的NAL测试（源自OpenNARS测试用例）
    const NAL_6_1: &str = r"
        nse $0.80;0.80;0.95$ <<$x --> bird> ==> <$x --> animal>>. %1.00;0.90%
        nse $0.80;0.80;0.95$ <<$y --> robin> ==> <$y --> bird>>. %1.00;0.90%";

    /// 「逻辑稳定性」中的NAL测试（源自OpenNARS测试用例）
    const NAL_6_10: &str = r"
        nse $0.80;0.80;0.95$ (&&,<#x --> bird>,<#x --> swimmer>). %1.00;0.90%
        nse $0.80;0.80;0.95$ <swan --> bird>. %0.90;0.90%";

    /// 「逻辑稳定性」中的NAL测试（源自OpenNARS测试用例）
    const NAL_6_11: &str = r"
        nse $0.80;0.80;0.95$ <{Tweety} --> [with_wings]>. %1.00;0.90%
        nse $0.80;0.80;0.95$ <(&&,<$x --> [chirping]>,<$x --> [with_wings]>) ==> <$x --> bird>>. %1.00;0.90%";

    /// 「逻辑稳定性」中的NAL测试（源自OpenNARS测试用例）
    const NAL_6_12: &str = r"
        nse $0.80;0.80;0.95$ <(&&,<$x --> flyer>,<$x --> [chirping]>, <(*, $x, worms) --> food>) ==> <$x --> bird>>. %1.00;0.90%
        nse $0.80;0.80;0.95$ <{Tweety} --> flyer>. %1.00;0.90%";

    /// 「逻辑稳定性」中的NAL测试（源自OpenNARS测试用例）
    const NAL_6_13: &str = r"
        nse $0.80;0.80;0.95$ <(&&,<$x --> key>,<$y --> lock>) ==> <$y --> (/,open,$x,_)>>. %1.00;0.90%
        nse $0.80;0.80;0.95$ <{lock1} --> lock>. %1.00;0.90%";

    /// 「逻辑稳定性」中的NAL测试（源自OpenNARS测试用例）
    const NAL_6_14: &str = r"
        nse $0.80;0.80;0.95$ <<$x --> lock> ==> (&&,<#y --> key>,<$x --> (/,open,#y,_)>)>. %1.00;0.90%
        nse $0.80;0.80;0.95$ <{lock1} --> lock>. %1.00;0.90%";

    /// 「逻辑稳定性」中的NAL测试（源自OpenNARS测试用例）
    const NAL_6_15: &str = r"
        nse $0.80;0.80;0.95$ (&&,<#x --> lock>,<<$y --> key> ==> <#x --> (/,open,$y,_)>>). %1.00;0.90%
        nse $0.80;0.80;0.95$ <{lock1} --> lock>. %1.00;0.90%";

    /// 「逻辑稳定性」中的NAL测试（源自OpenNARS测试用例）
    const NAL_6_16: &str = r"
        nse $0.80;0.80;0.95$ (&&,<#x --> (/,open,#y,_)>,<#x --> lock>,<#y --> key>). %1.00;0.90%
        nse $0.80;0.80;0.95$ <{lock1} --> lock>. %1.00;0.90%";

    /// 「逻辑稳定性」中的NAL测试（源自OpenNARS测试用例）
    const NAL_6_17: &str = r"
        nse $0.80;0.80;0.95$ <swan --> bird>. %1.00;0.90%
        nse $0.80;0.80;0.95$ <swan --> swimmer>. %0.80;0.90%";

    /// 「逻辑稳定性」中的NAL测试（源自OpenNARS测试用例）
    const NAL_6_18: &str = r"
        nse $0.80;0.80;0.95$ <gull --> swimmer>. %1.00;0.90%
        nse $0.80;0.80;0.95$ <swan --> swimmer>. %0.80;0.90%";

    /// 「逻辑稳定性」中的NAL测试（源自OpenNARS测试用例）
    const NAL_6_19: &str = r"
        nse $0.80;0.80;0.95$ <{key1} --> (/,open,_,{lock1})>. %1.00;0.90%
        nse $0.80;0.80;0.95$ <{key1} --> key>. %1.00;0.90%";

    /// 「逻辑稳定性」中的NAL测试（源自OpenNARS测试用例）
    const NAL_6_2: &str = r"
        nse $0.80;0.80;0.95$ <<$x --> swan> ==> <$x --> bird>>. %1.00;0.80%
        nse $0.80;0.80;0.95$ <<$y --> swan> ==> <$y --> swimmer>>. %0.80;0.90%";

    /// 「逻辑稳定性」中的NAL测试（源自OpenNARS测试用例）
    const NAL_6_20: &str = r"
        nse $0.80;0.80;0.95$ <<$x --> key> ==> <{lock1} --> (/,open,$x,_)>>. %1.00;0.90%
        nse $0.80;0.80;0.95$ <{lock1} --> lock>. %1.00;0.90%";

    /// 「逻辑稳定性」中的NAL测试（源自OpenNARS测试用例）
    const NAL_6_21: &str = r"
        nse $0.80;0.80;0.95$ (&&,<#x --> key>,<{lock1} --> (/,open,#x,_)>). %1.00;0.90%
        nse $0.80;0.80;0.95$ <{lock1} --> lock>. %1.00;0.90%";

    /// 「逻辑稳定性」中的NAL测试（源自OpenNARS测试用例）
    const NAL_6_22: &str = r"
        nse $0.80;0.80;0.95$ <0 --> num>. %1.00;0.90%
        nse $0.80;0.80;0.95$ <<$1 --> num> ==> <(*,$1) --> num>>. %1.00;0.90%
        nse $0.90;0.80;1.00$ <(*,(*,(*,0))) --> num>?";

    /// 「逻辑稳定性」中的NAL测试（源自OpenNARS测试用例）
    const NAL_6_23: &str = r"
        nse $0.80;0.80;0.95$ (&&,<#1 --> lock>,<<$2 --> key> ==> <#1 --> (/,open,$2,_)>>). %1.00;0.90%
        nse $0.80;0.80;0.95$ <{key1} --> key>. %1.00;0.90%";

    /// 「逻辑稳定性」中的NAL测试（源自OpenNARS测试用例）
    const NAL_6_24: &str = r"
        nse $0.80;0.80;0.95$ <<$1 --> lock> ==> (&&,<#2 --> key>,<$1 --> (/,open,#2,_)>)>. %1.00;0.90%
        nse $0.80;0.80;0.95$ <{key1} --> key>. %1.00;0.90%";

    /// 「逻辑稳定性」中的NAL测试（源自OpenNARS测试用例）
    const NAL_6_25: &str = r"
        nse $0.80;0.80;0.95$ <<lock1 --> (/,open,$1,_)> ==> <$1 --> key>>. %1.00;0.90%
        nse $0.80;0.80;0.95$ <lock1 --> lock>. %1.00;0.90%";

    /// 「逻辑稳定性」中的NAL测试（源自OpenNARS测试用例）
    const NAL_6_26: &str = r"
        nse $0.80;0.80;0.95$ <lock1 --> lock>. %1.00;0.90%
        nse $0.80;0.80;0.95$ <(&&,<#1 --> lock>,<#1 --> (/,open,$2,_)>) ==> <$2 --> key>>. %1.00;0.90%";

    /// 「逻辑稳定性」中的NAL测试（源自OpenNARS测试用例）
    const NAL_6_27: &str = r"
        nse $0.80;0.80;0.95$ <<lock1 --> (/,open,$1,_)> ==> <$1 --> key>>. %1.00;0.90%
        nse $0.80;0.80;0.95$ <(&&,<#1 --> lock>,<#1 --> (/,open,$2,_)>) ==> <$2 --> key>>. %1.00;0.90%";

    /// 「逻辑稳定性」中的NAL测试（源自OpenNARS测试用例）
    const NAL_6_3: &str = r"
        nse $0.80;0.80;0.95$ <<bird --> $x> ==> <robin --> $x>>. %1.00;0.90%
        nse $0.80;0.80;0.95$ <<swimmer --> $y> ==> <robin --> $y>>. %0.70;0.90%";

    /// 「逻辑稳定性」中的NAL测试（源自OpenNARS测试用例）
    const NAL_6_4: &str = r"
        nse $0.80;0.80;0.95$ <(&&,<$x --> flyer>,<$x --> [chirping]>) ==> <$x --> bird>>. %1.00;0.90%
        nse $0.80;0.80;0.95$ <<$y --> [with_wings]> ==> <$y --> flyer>>. %1.00;0.90%";

    /// 「逻辑稳定性」中的NAL测试（源自OpenNARS测试用例）
    const NAL_6_5: &str = r"
        nse $0.80;0.80;0.95$ <(&&,<$x --> flyer>,<$x --> [chirping]>, <(*, $x, worms) --> food>) ==> <$x --> bird>>. %1.00;0.90%

        nse $0.80;0.80;0.95$ <(&&,<$x --> [chirping]>,<$x --> [with_wings]>) ==> <$x --> bird>>. %1.00;0.90%";

    /// 「逻辑稳定性」中的NAL测试（源自OpenNARS测试用例）
    const NAL_6_6: &str = r"
        nse $0.80;0.80;0.95$ <(&&,<$x --> flyer>,<(*,$x,worms) --> food>) ==> <$x --> bird>>. %1.00;0.90%
        nse $0.80;0.80;0.95$ <<$y --> flyer> ==> <$y --> [with_wings]>>. %1.00;0.90%";

    /// 「逻辑稳定性」中的NAL测试（源自OpenNARS测试用例）
    const NAL_6_7: &str = r"
        nse $0.80;0.80;0.95$ <<$x --> bird> ==> <$x --> animal>>. %1.00;0.90%
        nse $0.80;0.80;0.95$ <robin --> bird>. %1.00;0.90%";

    /// 「逻辑稳定性」中的NAL测试（源自OpenNARS测试用例）
    const NAL_6_8: &str = r"
        nse $0.80;0.80;0.95$ <<$x --> bird> ==> <$x --> animal>>. %1.00;0.90%
        nse $0.80;0.80;0.95$ <tiger --> animal>. %1.00;0.90%";

    /// 「逻辑稳定性」中的NAL测试（源自OpenNARS测试用例）
    const NAL_6_9: &str = r"
        nse $0.80;0.80;0.95$ <<$x --> animal> <=> <$x --> bird>>. %1.00;0.90%
        nse $0.80;0.80;0.95$ <robin --> bird>. %1.00;0.90%";

    /// 「逻辑稳定性」中的NAL测试（源自OpenNARS测试用例）
    const NAL_6_BIRD_CLAIMED_BY_BOB: &str = r"
        nse $0.80;0.80;0.95$ <(&,<{Tweety} --> bird>,<bird --> fly>) --> claimedByBob>. %1.00;0.90%
        nse $0.80;0.80;0.95$ <<(&,<#1 --> $2>,<$3 --> #1>) --> claimedByBob> ==> <<$3 --> $2> --> claimedByBob>>. %1.00;0.90%
        nse $0.90;0.80;1.00$ <?x --> claimedByBob>?";

    /// 「逻辑稳定性」中的NAL测试（源自OpenNARS测试用例）
    const NAL_6_CAN_OF_WORMS: &str = r"
        nse $0.80;0.80;0.95$ <0 --> num>. %1.00;0.90%
        nse $0.80;0.80;0.95$ <0 --> (/,num,_)>. %1.00;0.90%";

    /// 「逻辑稳定性」中的NAL测试（源自OpenNARS测试用例）
    const NAL_6_NLP1: &str = r"
        nse $0.80;0.80;0.95$ <(\,REPRESENT,_,CAT) --> cat>. %1.00;0.90%
        nse $0.80;0.80;0.95$ <(\,(\,REPRESENT,_,<(*,CAT,FISH) --> FOOD>),_,eat,fish) --> cat>. %1.00;0.90%";

    /// 「逻辑稳定性」中的NAL测试（源自OpenNARS测试用例）
    const NAL_6_NLP2: &str = r"
        nse $0.80;0.80;0.95$ <cat --> (/,(/,REPRESENT,_,<(*,CAT,FISH) --> FOOD>),_,eat,fish)>. %1.00;0.90%
        nse $0.80;0.80;0.95$ <cat --> CAT>. %1.00;0.90%";

    /// 「逻辑稳定性」中的NAL测试（源自OpenNARS测试用例）
    const NAL_6_REDUNDANT: &str = r"
        nse $0.80;0.80;0.95$ <<lock1 --> (/,open,$1,_)> ==> <$1 --> key>>. %1.00;0.90%";

    /// 「逻辑稳定性」中的NAL测试（源自OpenNARS测试用例）
    const NAL_6_SYMMETRY: &str = r"
        nse $0.80;0.80;0.95$ <(*,a,b) --> like>. %1.00;0.90%
        nse $0.80;0.80;0.95$ <(*,b,a) --> like>. %1.00;0.90%
        nse $0.90;0.80;1.00$ <<(*,$1,$2) --> like> <=> <(*,$2,$1) --> like>>?";

    /// 「逻辑稳定性」中的NAL测试（源自OpenNARS测试用例）
    const NAL_6_UNCLE: &str = r"
        nse $0.80;0.80;0.95$ <tim --> (/,uncle,_,tom)>. %1.00;0.90%
        nse $0.80;0.80;0.95$ <tim --> (/,uncle,tom,_)>. %0.00;0.90%";

    const NAL_TESTS: [&str; 119] = [
        NAL_1_0,
        NAL_1_1,
        NAL_1_2,
        NAL_1_3,
        NAL_1_4,
        NAL_1_5,
        NAL_1_6,
        NAL_1_7,
        NAL_1_8,
        NAL_2_0,
        NAL_2_1,
        NAL_2_10,
        NAL_2_11,
        NAL_2_12,
        NAL_2_13,
        NAL_2_14,
        NAL_2_15,
        NAL_2_16,
        NAL_2_17,
        NAL_2_18,
        NAL_2_19,
        NAL_2_2,
        NAL_2_3,
        NAL_2_4,
        NAL_2_5,
        NAL_2_6,
        NAL_2_7,
        NAL_2_8,
        NAL_2_9,
        NAL_3_0,
        NAL_3_1,
        NAL_3_10,
        NAL_3_11,
        NAL_3_12,
        NAL_3_13,
        NAL_3_14,
        NAL_3_15,
        NAL_3_2,
        NAL_3_3,
        NAL_3_4,
        NAL_3_5,
        NAL_3_6,
        NAL_3_7,
        NAL_3_8,
        NAL_3_9,
        NAL_4_0,
        NAL_4_1,
        NAL_4_2,
        NAL_4_3,
        NAL_4_4,
        NAL_4_5,
        NAL_4_6,
        NAL_4_7,
        NAL_4_8,
        NAL_5_0,
        NAL_5_1,
        NAL_5_10,
        NAL_5_11,
        NAL_5_12,
        NAL_5_13,
        NAL_5_14,
        NAL_5_15,
        NAL_5_16,
        NAL_5_17,
        NAL_5_18,
        NAL_5_19,
        NAL_5_2,
        NAL_5_20,
        NAL_5_21,
        NAL_5_22,
        NAL_5_23,
        NAL_5_24,
        NAL_5_25,
        NAL_5_26,
        NAL_5_27,
        NAL_5_28,
        NAL_5_29,
        NAL_5_3,
        NAL_5_4,
        NAL_5_5,
        NAL_5_6,
        NAL_5_7,
        NAL_5_8,
        NAL_5_9,
        NAL_6_0,
        NAL_6_1,
        NAL_6_10,
        NAL_6_11,
        NAL_6_12,
        NAL_6_13,
        NAL_6_14,
        NAL_6_15,
        NAL_6_16,
        NAL_6_17,
        NAL_6_18,
        NAL_6_19,
        NAL_6_2,
        NAL_6_20,
        NAL_6_21,
        NAL_6_22,
        NAL_6_23,
        NAL_6_24,
        NAL_6_25,
        NAL_6_26,
        NAL_6_27,
        NAL_6_3,
        NAL_6_4,
        NAL_6_5,
        NAL_6_6,
        NAL_6_7,
        NAL_6_8,
        NAL_6_9,
        NAL_6_BIRD_CLAIMED_BY_BOB,
        NAL_6_CAN_OF_WORMS,
        NAL_6_NLP1,
        NAL_6_NLP2,
        NAL_6_REDUNDANT,
        NAL_6_SYMMETRY,
        NAL_6_UNCLE,
    ];

    /// 从指定的「分隔符」生成「逻辑稳定性」测试用例
    /// * 🎯简化「重复后缀的语句」并统一「测试用例文本」
    fn generate_logical_stability(sep: impl AsRef<str>) -> String {
        NAL_TESTS.into_iter().join_to_new(sep.as_ref())
    }

    /// 集成测试：逻辑稳定性
    /// * 🎯推理器在所有NAL 1-6的测试用例中，保持运行不panic
    #[test]
    fn logical_stability() -> AResult {
        pipe! {
            // * 🚩生成的最终文本附带「每次输入测试后运行100步」的效果
            "
            cyc 100
            "
            => generate_logical_stability
            => test_line_inputs
        }
    }

    /// 集成测试：逻辑稳定性（分离的）
    /// * 🎯推理器在所有NAL 1-6的测试用例中，保持运行不panic
    /// * 🚩与[原测试](logical_stability)的区别：每运行完一个文件后，重置推理器
    #[test]
    fn logical_stability_separated() -> AResult {
        pipe! {
            // * 🚩生成的最终文本附带「每次输入测试后运行100步，并在运行后重置推理器」的效果
            "
            cyc 100
            res
            "
            => generate_logical_stability
            => test_line_inputs
        }
    }
}
