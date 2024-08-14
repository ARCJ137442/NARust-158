use crate::control::Reasoner;
use nar_dev_utils::macro_once;

/// 处理指令[`Cmd::HLP`]
pub fn hlp_dispatch(_reasoner: &mut Reasoner, query: impl AsRef<str>) -> Result<String, String> {
    macro_once! {
        macro ( $( $query:literal => $message:expr )* ) => {
            /// 所有非空查询的列表
            /// * 📌格式：Markdown无序列表
            const ALL_QUERIES_LIST: &str = concat!($( "\n- ", $query, )*);
            match query.as_ref() {
                // 特殊/空字串：列举已有的所有参数
                // ! ⚠️【2024-08-09 17:48:15】不能放外边：会被列入非空查询列表中
                "" => Ok(format!("Available help queries: {ALL_QUERIES_LIST}")),
                // 所有固定模式的分派
                $( $query => Ok($message.to_string()), )*
                // 未知的查询关键词
                other => return Err(format!("Unknown help query: {other:?}\nAvailable help queries: {ALL_QUERIES_LIST}")),
            }
        }

        // * 🚩普通帮助查询
        "inf" => CMD_INF            // 展示有关命令`INF`的帮助
        "examples" => EXAMPLES_CMD  // 有关各类指令的输入示例
    }
}

/// 有关指令 [`INF`](Cmd::INF) 的帮助
const CMD_INF: &str = "# cmd `INF`
- Format: `INF <qualifier><target>`
- qualifiers:
  - `#`: Detailed info
- targets:
  - `tasks`: Tasks in reasoner, or derivation chain on detailed mode
  - `concepts`: Concepts in memory
  - `links`: Task-links and term-links in each concepts
  - `parameters`: View reasoner parameters
  - `beliefs`: Beliefs in memory
  - `questions`: Questions in memory
  - `summary`: The summary of status of reasoner, no detailed mode yet
";

/// 有关「示例输入」的帮助
const EXAMPLES_CMD: &str = "# NAVM Cmd examples

## Inputting narseses, tuning the volume, running cycles and querying information
```navm-cmd
NSE <A --> B>.
NSE <A --> C>.
VOL 99
CYC 10
INF tasks
```

## Comments
```navm-cmd
REM This is a comment, it will be ignored
REM For multi-line comments, use `REM` to start each line
```

## Getting help
```navm-cmd
HLP
```
";
