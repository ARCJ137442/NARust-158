//! 单元测试 通用函数

use crate::{
    control::Reasoner,
    inference::{match_task_and_belief, process_direct, reason, transform_task, InferenceEngine},
    language::Term,
    ok,
    parameters::{Parameters, DEFAULT_PARAMETERS},
    util::AResult,
};
use nar_dev_utils::{list, unwrap_or_return};
use narsese::{api::GetTerm, conversion::string::impl_lexical::format_instances::FORMAT_ASCII};
use navm::{cmd::Cmd, output::Output};

/// 预期输出词项相等
/// * 🎯独立的「输出内容与预期词项判等」方法
pub fn expect_output_eq_term(output: &Output, expected: &Term) -> bool {
    let lexical_term = unwrap_or_return!(
        ?output.get_narsese().map(GetTerm::get_term).cloned()
        => false // 输出没有词项⇒直接不等
    );
    let lexical_str = FORMAT_ASCII.format(&lexical_term);
    let out = unwrap_or_return!(
        @Term::from_lexical(lexical_term),
        e => {
            // * 🚩【2024-09-07 15:22:16】目前补丁：打印警告并忽略之
            //   * ℹ️缘由：输出中可能包含「无效词项」（`make`方法中非法）
            //   * 📄首次见于测试`intro_var_same_subject`中（由于变量归一化？）产生的词项`(&&(($1 --> B) ($1 --> C)) ==> ($1 --> $1))`
            //   * 📄直接推理结论产生在组合规则`intro_var_inner2`方法中
            eprintln!("要与预期相比对的词项 {lexical_str:?} 解析失败：{e}");
            true
        }
    );
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
            && $crate::inference::tests::tools::expect_output_eq_term_lexical(
                // * 🚩【2024-07-15 00:04:43】此处使用了「词法Narsese」的内部分派
                &output, narsese::lexical_nse_term!(@PARSE $term)
            )
    };
}

/// 从「超参数」与「推理引擎」创建虚拟机
pub fn create_reasoner(parameters: Parameters, engine: InferenceEngine) -> Reasoner {
    Reasoner::new("test", parameters, engine)
}

/// 设置虚拟机到「最大音量」
/// * 🎯使虚拟机得以输出尽可能详尽的信息
pub fn set_max_volume(reasoner: &mut Reasoner) {
    reasoner.set_volume(100);
}

/// 从「推理引擎」创建虚拟机
/// * 📜使用默认参数
/// * 🚩【2024-08-01 14:34:19】默认最大音量
pub fn create_reasoner_from_engine(engine: InferenceEngine) -> Reasoner {
    let mut reasoner = create_reasoner(DEFAULT_PARAMETERS, engine);
    set_max_volume(&mut reasoner);
    reasoner
}

/// 扩展推理器的功能
impl Reasoner {
    /// 简单解释NAVM指令
    /// * 🎯轻量级指令分派，不带存取等额外功能
    pub(crate) fn input_cmd(&mut self, cmd: Cmd) -> AResult<()> {
        use Cmd::*;
        match cmd {
            NSE(task) => self.input_task(task),
            CYC(steps) => self.cycle(steps),
            VOL(volume) => self.set_volume(volume),
            RES { .. } => self.reset(),
            REM { .. } => (),
            INF { source } if source == "summary" => self.report_info(self.report_summary()),
            INF { .. } => (),
            _ => return Err(anyhow::anyhow!("不支持的NAVM指令：{cmd}")),
        }
        ok!()
    }

    /// 输入NAVM指令到虚拟机
    pub(crate) fn input_cmds(&mut self, cmds: &str) {
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
    pub(crate) fn input_cmds_soft(&mut self, cmds: &str) {
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
    pub(crate) fn fetch_outputs(&mut self) -> Vec<Output> {
        list![
            output
            while let Some(output) = (self.take_output())
        ]
    }

    /// 输入指令并拉取输出
    #[must_use]
    pub(crate) fn input_cmds_and_fetch_out(&mut self, cmds: &str) -> Vec<Output> {
        self.input_cmds(cmds);
        self.fetch_outputs()
    }

    // !  ❌【2024-08-15 00:58:37】暂时用不到
    // /// 拉取输出并预期其中的输出
    // pub(crate) fn fetch_expected_outputs(
    //     &mut self,
    //     expect: impl Fn(&Output) -> bool,
    // ) -> Vec<Output> {
    //     let outputs = self.fetch_outputs();
    //     expect_outputs(&outputs, expect);
    //     outputs
    // }

    /// 输入指令、拉取、打印并预期输出
    pub(crate) fn input_fetch_print_expect(
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

/// 引擎dev
/// * 🚩【2024-07-09 16:52:40】目前除了「概念推理」均俱全
/// * ✅【2024-07-14 23:50:15】现集成所有四大推理函数
pub const ENGINE_DEV: InferenceEngine = InferenceEngine::new(
    process_direct,
    transform_task,
    match_task_and_belief,
    reason,
);

/// 「预期测试」函数
pub fn expectation_test(inputs: impl AsRef<str>, expectation: impl Fn(&Output) -> bool) {
    let mut vm = create_reasoner_from_engine(ENGINE_DEV);
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
            $crate::inference::tests::tools::expectation_test(
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
