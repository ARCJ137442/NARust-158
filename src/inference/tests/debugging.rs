//! debug的测试
//! * 🎯【2024-08-24 11:38:46】用于安放与issues、bugs有关的测试
//!   * 📝这些测试往往和单个推理规则无关，也可能和控制机制有关

use super::tools::{create_reasoner_from_engine, print_outputs, ENGINE_DEV};
use crate::{expectation_tests, language::Term};
use narsese::{api::GetTerm, conversion::string::impl_lexical::format_instances::FORMAT_ASCII};
use navm::output::Output;

expectation_tests! {

    /// 变量引入「重命名变量」方面的bug
    /// * 📌【2024-08-24 11:50:14】目前的命名格式：
    ///   * 📝【类型：bug/issue/...】_【日期】_（具体函数）_（原因梗概）
    ///   * 📍类型
    ///     * bug：开发者在编写代码时发现的问题
    ///     * issue_【编号】：来自GitHub issue
    ///     * ...
    ///   * ℹ️圆括号的为可选
    ///
    /// ## Logs
    ///
    /// * ✅【2024-08-19 22:12:47】bug解决，测试通过
    bug_20240819_intro_var_inner_loop_substitute: {
        "
        nse <<A --> (*, $1, $2)> ==> <A --> [$1, $2]>>.
        nse <A --> (*, B, C)>.
        nse <A --> (*, B, B)>.
        nse <A --> [B, C]>?
        nse <A --> [B]>?
        cyc 20
        "
        => ANSWER "<A --> [B]>" in outputs
    }

    /// [`bug_20240819_intro_var_inner_loop_substitute`]的另一个例子
    bug_20240819_intro_var_inner_another_example: {
        "
        vol 99
        nse <<A --> [$1, $2]> ==> <A --> (*, $1, $2)>>.
        nse <A --> [B, C]>.
        nse <A --> [B]>.
        nse <A --> (*, B, C)>?
        rem ↓下面这个不行
        nse <A --> (*, B, B)>?
        cyc 1000
        "
        => ANSWER "<A --> (*, B, C)>" in outputs
    }

    /// 🔗https://github.com/ARCJ137442/Narust-158/issues/1
    /// * 📝在Windows环境下运行缓慢
    /// * ⚠️最后堆栈溢出在`variable_process`的「应用替换」上
    ///   * 替换的是一个外延像
    #[ignore = "堆栈溢出类测试 运行时间过长，不便加入cargo test中"]
    issue_001_20240824_apply_substitute_infinite_recurse: {
        "
        nse <{P1} --> P>.
        nse <{L1} --> L>.
        nse <{A1} --> A>.
        nse <{O1} --> O>.

        nse <(*, {A1}, {P1}) -->IN>.

        nse <(*, {O1}, {L1}) --> IN>.

        nse <(*, {A1}, {O1}) --> Hb>.

        nse <(*, {P1}, {L1}) --> Bind>.

        nse <{P2} --> P>.
        nse <{L2} --> L>.
        nse <{A2} --> A>.
        nse <{O2} --> O>.

        nse <(*, {A2}, {P2}) --> IN>.
        nse <(*, {O2}, {L2}) --> IN>.
        nse <(*, {A2}, {O2}) --> Hb>.

        nse <(*, {P2}, {L1}) --> Bind>?
        nse <(*, {P1}, {L2}) --> Bind>?
        nse <(*, {P2}, {L2}) --> Bind>?
        cyc 6000
        "
        => ANSWER "<(*, {P1}, {L2}) --> Bind>" in outputs // TODO 一个测试中预期多个输出
    }
}

#[test]
fn bug_20240908_intro_var_inner_invalid_statement() {
    let mut vm = create_reasoner_from_engine(ENGINE_DEV);
    // * 🚩OUT
    let outputs = vm.input_cmds_and_fetch_out(
        // 示例性文本：产生「无效输出」的地方
        "
        nse <<$1 --> B> ==> <$1 --> A>>.
        nse <A --> C>.
        cyc 5
        ",
    );
    // * 🚩打印输出以便跟踪
    print_outputs(&outputs);
    // * 🚩检查输出中的Narsese
    for narsese_lexical in outputs
        .iter()
        .flat_map(Output::get_narsese)
        .map(GetTerm::get_term)
        .cloned()
    {
        // * 🚩【2024-09-08 16:28:28】目前仅对错误予以警告
        // TODO: 启用并着手解决此类bug（筛选の方法，避免无效结论在NARS内部累积）
        //   * 📄如：「检验是否有效」函数 @ 所有词项
        let str_ascii = FORMAT_ASCII.format(&narsese_lexical);
        if let Err(e) = Term::from_lexical(narsese_lexical) {
            // panic
            eprintln!("推理器产生了无效的内部输出 {str_ascii:?}：{e}")
        }
    }
}
