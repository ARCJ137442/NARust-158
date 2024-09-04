use super::cmd_hlp::hlp_dispatch;
use crate::control::Reasoner;
use nar_dev_utils::macro_once;

/// 指令[`Cmd::INF`]的入口函数
/// * 📌传入的`query`默认为小写字串引用
/// * 📌输出仅为一个消息字符串；若返回[错误值](Err)，则视为「报错」
pub fn inf_dispatch(reasoner: &mut Reasoner, query: impl AsRef<str>) -> Result<String, String> {
    macro_once! {
        macro ( $( $query:literal => $message:expr )* ) => {
            /// 所有非空查询的列表
            /// * 📌格式：Markdown无序列表
            const ALL_QUERIES_LIST: &str = concat!($( "\n- ", $query, )*);
            match query.as_ref() {
                // * 🚩特殊/空字串：列举所有query并转接`HLP INF`
                // ! ⚠️【2024-08-09 17:48:15】不能放外边：会被列入非空查询列表中
                "" => Ok(format!(
                    "Available info queries: {ALL_QUERIES_LIST}\n\nAnd more info:\n{}",
                    hlp_dispatch(reasoner, "inf")?
                )),
                // 所有固定模式的分派
                $( $query => Ok($message.to_string()), )*
                // * 🚩其它⇒告警
                other => Err(format!("Unknown info query: {other:?}\nAvailable info queries: {ALL_QUERIES_LIST}")),
            }
        }

        // * 🚩普通信息查询
        "parameters" => reasoner.report_parameters() // 推理器的超参数
        "tasks" => reasoner.report_tasks()           // 推理器中所有任务
        "beliefs" => reasoner.report_beliefs()       // 推理器中所有信念
        "questions" => reasoner.report_questions()   // 推理器中所有问题
        "concepts" => reasoner.report_concepts()     // 推理器中所有概念
        "links" => reasoner.report_links()           // 推理器中所有链接
        "summary" => reasoner.report_summary()       // 推理器中所有链接

        // * 🚩更详尽的信息
        "#parameters" => reasoner.report_parameters_detailed() // 具有缩进层级
        "#tasks" => reasoner.report_tasks_detailed()           // 推理器中的任务派生链
        "#beliefs" => reasoner.report_beliefs_detailed()       // 推理器中所有信念（详细）
        "#questions" => reasoner.report_questions_detailed()   // 推理器中所有问题（详细）
        "#concepts" => reasoner.report_concepts_detailed()     // 推理器中所有概念，含任务链、词项链
        "#links" => reasoner.report_links_detailed()           // 推理器中所有链接，含预算值
    }
}
