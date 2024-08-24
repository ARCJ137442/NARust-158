//! 总体性测试
//! * 📌长期稳定性、逻辑稳定性
//!   * 🎯不在运行时panic

use super::tools::{create_reasoner_from_engine, print_outputs, ENGINE_DEV};
use crate::{ok, util::AResult};
use nar_dev_utils::pipe;

/// 测试多行NAVM指令（文本形式）输入
/// * 🚩仅测试文本输入（稳定性），不负责捕获输出等额外操作
/// * 🚩【2024-08-12 22:47:01】为提高「长期测试」效率，将推理器强制静音
///   * ⚠️损失了一部分有关「生成输出」的测试
fn test_line_inputs<S: AsRef<str>>(inputs: impl IntoIterator<Item = S>) -> AResult {
    // 创建
    let mut runtime = create_reasoner_from_engine(ENGINE_DEV);
    // 静音
    runtime.input_cmds("vol 0");
    // 输入指令（软标准，不要求解析成功⇒向后兼容）
    for inputs in inputs {
        runtime.input_cmds_soft(inputs.as_ref());
    }
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

const NAL_LONG_TERM_STABILITY: &str = r#"
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
        cyc 1000"#;

/// 集成测试：长期稳定性
/// * 🎯推理器在大量词项与任务的基础上，保持运行不panic
#[test]
fn long_term_stability() -> AResult {
    test_line_inputs([NAL_LONG_TERM_STABILITY])
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

/// 「逻辑稳定性」中所有的NAL测试（源自OpenNARS测试用例）
/// * 📌【2024-08-10 14:47:27】此中之`119`是为了兼容后续测试
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
fn generate_logical_stability(sep: &str) -> impl Iterator<Item = String> + '_ {
    NAL_TESTS.into_iter().map(|s| s.to_string() + sep)
}

/// 集成测试：逻辑稳定性
/// * 🎯推理器在所有NAL 1-6的测试用例中，保持运行不panic
/// * 🚩【2024-08-12 22:56:38】考虑到单测时间太长，目前压到5步
#[test]
fn logical_stability() -> AResult {
    pipe! {
        // * 🚩生成的最终文本附带「每次输入测试后运行5步」的效果
        "
            cyc 5
            "
        => generate_logical_stability
        => test_line_inputs
    }
}

/// 集成测试：逻辑稳定性（分离的）
/// * 🎯推理器在所有NAL 1-6的测试用例中，保持运行不panic
/// * 🚩与[原测试](logical_stability)的区别：每运行完一个文件后，重置推理器
/// * 🚩【2024-08-12 22:56:38】考虑到单测时间太长，目前压到50步
#[test]
fn logical_stability_separated() -> AResult {
    pipe! {
        // * 🚩生成的最终文本附带「每次输入测试后运行50步，并在运行后重置推理器」的效果
        "
            cyc 50
            res
            "
        => generate_logical_stability
        => test_line_inputs
    }
}
