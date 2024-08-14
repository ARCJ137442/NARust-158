//! 集中管理有关「推理器分派处理指令」的函数

use crate::control::Reasoner;
use navm::cmd::Cmd;

/// 输入指令
impl RuntimeAlpha {
    /// 模拟`ReasonerBatch.textInputLine`
    /// * 🚩🆕【2024-05-13 02:27:07】从「字符串输入」变为「NAVM指令输入」
    /// * 🚩【2024-06-29 01:42:46】现在不直接暴露「输入NAVM指令」：全权交给「通道」机制
    ///   * 🚩由「通道」的「处理IO」引入
    ///
    /// TODO: 🏗️后续拟定一个更为完备的「虚拟机开发方案」
    /// * 💭一个场景定制一个虚拟机，内核不变
    /// * 💭内核暴露给像是「序列化JSON」的API，提供一个「虚拟机默认实现」但允许其它外部库接入内核
    /// * 💭内核不应直接提供「指令输入」并封装「有关各指令的功能」，应该暴露接口以便让外部库使用
    pub(super) fn input_cmd(&mut self, cmd: Cmd) {
        match cmd {
            Cmd::SAV { target, path } => self.cmd_sav(target, path),
            Cmd::LOA { target, path } => self.cmd_loa(target, path),
            // * 🚩重置：推理器复位
            Cmd::RES { .. } => self.reasoner.reset(),
            // * 🚩Narsese：输入任务（但不进行推理）
            Cmd::NSE(narsese) => self.cmd_nse(narsese),
            // Cmd::NEW { target } => (),
            // Cmd::DEL { target } => (),
            // * 🚩工作周期：只执行推理，不处理输入输出
            Cmd::CYC(cycles) => self.reasoner.cycle(cycles),
            // * 🚩音量：设置音量 & 提示
            Cmd::VOL(volume) => self.cmd_vol(volume),
            // Cmd::REG { name } => (),
            Cmd::INF { source } => self.cmd_inf(source),
            Cmd::HLP { name } => self.cmd_hlp(name),
            // * 🚩【2024-05-13 12:21:37】注释：不做任何事情
            Cmd::REM { .. } => (),
            // * 🚩退出⇒处理完所有输出后直接退出
            Cmd::EXI { reason } => self.cmd_exi(reason),
            // Cmd::Custom { head, tail } => (),
            // * 🚩未知指令⇒输出提示
            _ => self.reasoner.report_error(format!("Unknown cmd: {cmd}")),
        }
    }

    /// 处理指令[`Cmd::NSE`]
    fn cmd_nse(&mut self, narsese: narsese::lexical::Task) {
        self.reasoner.input_task(narsese)
    }

    /// 处理指令[`Cmd::VOL`]
    fn cmd_vol(&mut self, volume: usize) {
        self.reasoner
            .report_info(format!("volume: {} => {volume}", self.reasoner.volume()));
        self.reasoner.set_volume(volume);
    }

    /// 处理指令[`Cmd::EXI`]
    ///
    /// ? ❓【2024-07-23 16:10:13】是否一定要主程序退出
    ///   * 💭还是说，NARS本身并没有个实际上的「退出」机制
    fn cmd_exi(&mut self, reason: String) {
        // * 🚩最后的提示性输出
        self.reasoner
            .report_info(format!("Program exited with reason {reason:?}"));
        // * 🚩处理所有输出
        self.handle_output();
        // * 🚩最终退出程序
        std::process::exit(0);
    }

    /// 处理一个[`Result`]消息
    /// * 📌根据变体决定消息类型
    ///   * [`Ok`] => `INFO`
    ///   * [`Err`] => `ERROR`
    fn report_result(&mut self, result: Result<String, String>) {
        // 消息分派 | 📌只在此处涉及「报告输出」
        match result {
            // 正常信息⇒报告info
            Ok(message) => self.reasoner.report_info(message),
            // 错误信息⇒报告error
            Err(message) => self.reasoner.report_error(message),
        }
    }

    /// 处理指令[`Cmd::INF`]
    fn cmd_inf(&mut self, source: String) {
        // 查询
        let query = source.to_lowercase();
        // 消息分派 | 📌只在此处涉及「报告输出」
        let result = inf_dispatch(&mut self.reasoner, query);
        self.report_result(result)
    }

    /// 处理指令[`Cmd::HLP`]
    fn cmd_hlp(&mut self, name: String) {
        // 查询
        let query = name.to_lowercase();
        // 获取并报告消息
        let result = hlp_dispatch(&mut self.reasoner, query);
        self.report_result(result)
    }

    /// 处理指令[`Cmd::SAV`]
    fn cmd_sav(&mut self, target: String, path: String) {
        // 查询
        let query = target.to_lowercase();
        // 获取并报告消息
        let result = sav_dispatch(&mut self.reasoner, query, path);
        self.report_result(result)
    }

    /// 处理指令[`Cmd::LOA`]
    fn cmd_loa(&mut self, target: String, data: String) {
        // 查询
        let query = target.to_lowercase();
        // 获取并报告消息
        let result = loa_dispatch(&mut self.reasoner, query, data);
        self.report_result(result)
    }
}

/// 专用于指令[`Cmd::HLP`]的处理函数
mod cmd_hlp {
    use super::*;
    use nar_dev_utils::macro_once;

    /// 处理指令[`Cmd::HLP`]
    pub fn hlp_dispatch(
        _reasoner: &mut Reasoner,
        query: impl AsRef<str>,
    ) -> Result<String, String> {
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
}
use cmd_hlp::*;

/// 专用于指令[`Cmd::INF`]的处理函数
mod cmd_inf {
    use super::*;
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
                        cmd_hlp::hlp_dispatch(reasoner, "inf")?
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
}
use cmd_inf::*;

/// 专用于指令[`Cmd::SAV`]的处理函数
mod cmd_sav {
    use super::*;
    use nar_dev_utils::macro_once;

    /// 指令[`Cmd::SAV`]的入口函数
    /// * 📌传入的`query`默认为小写字串引用
    /// * 📌输出仅为JSON字符串；若返回[错误值](Err)，则视为「报错」
    pub fn sav_dispatch(
        reasoner: &mut Reasoner,
        query: impl AsRef<str>,
        _path: impl AsRef<str>,
    ) -> Result<String, String> {
        macro_once! {
            macro ( $( $query:literal => $message:expr )* ) => {
                /// 所有非空查询的列表
                /// * 📌格式：Markdown无序列表
                const ALL_QUERIES_LIST: &str = concat!($( "\n- ", $query, )*);
                match query.as_ref() {
                    // * 🚩特殊/空字串：列举所有query并转接`HLP INF`
                    // ! ⚠️【2024-08-09 17:48:15】不能放外边：会被列入非空查询列表中
                    "" => Ok(format!("Available save target: {ALL_QUERIES_LIST}",)),
                    // 所有固定模式的分派
                    $( $query => Ok($message.to_string()), )*
                    // * 🚩其它⇒告警
                    other => Err(format!("Unknown save target: {other:?}")),
                }
            }

            // 记忆区
            "memory" => memory_to_json(reasoner)
                .map_err(|e| format!("Failed to serialize memory: {e}"))?
            // 推理器整体状态
            "status" => status_to_json(reasoner)
                .map_err(|e| format!("Failed to serialize status: {e}"))?
        }
    }

    /// 将记忆区转换为JSON字符串
    /// * ⚠️可能失败：记忆区数据可能无法被序列化
    pub fn memory_to_json(reasoner: &Reasoner) -> anyhow::Result<String> {
        let mut writer = Vec::<u8>::new();
        let mut ser = serde_json::Serializer::new(&mut writer);
        reasoner.serialize_memory(&mut ser)?;
        let json = String::from_utf8(writer)?;
        Ok(json)
    }

    /// 将「推理状态」转换为JSON字符串
    /// * ⚠️可能失败：记忆区数据可能无法被序列化
    pub fn status_to_json(reasoner: &Reasoner) -> anyhow::Result<String> {
        let mut writer = Vec::<u8>::new();
        let mut ser = serde_json::Serializer::new(&mut writer);
        reasoner.serialize_status(&mut ser)?;
        let json = String::from_utf8(writer)?;
        Ok(json)
    }
}
use cmd_sav::*;

/// 专用于指令[`Cmd::LOA`]的处理函数
mod cmd_loa {
    use super::*;
    use crate::storage::Memory;
    use nar_dev_utils::macro_once;

    /// 可复用的「记忆区加载成功」消息
    /// * 🎯用于在测试用例中重用
    const MESSAGE_MEMORY_LOAD_SUCCESS: &str = "Memory loading success";
    const MESSAGE_STATUS_LOAD_SUCCESS: &str = "Status loading success";

    /// 指令[`Cmd::LOA`]的入口函数
    /// * 📌传入的`query`默认为小写字串引用
    /// * 📌输出仅为JSON字符串；若返回[错误值](Err)，则视为「报错」
    pub fn loa_dispatch(
        reasoner: &mut Reasoner,
        query: impl AsRef<str>,
        data: impl AsRef<str>,
    ) -> Result<String, String> {
        macro_once! {
            macro ( $( $query:literal => $message:expr )* ) => {
                /// 所有非空查询的列表
                /// * 📌格式：Markdown无序列表
                const ALL_QUERIES_LIST: &str = concat!($( "\n- ", $query, )*);
                match query.as_ref() {
                    // * 🚩特殊/空字串：列举所有query并转接`HLP INF`
                    // ! ⚠️【2024-08-09 17:48:15】不能放外边：会被列入非空查询列表中
                    "" => Ok(format!("Available load target: {ALL_QUERIES_LIST}",)),
                    // 所有固定模式的分派
                    $( $query => Ok($message.to_string()), )*
                    // * 🚩其它⇒告警
                    other => Err(format!("Unknown load target: {other:?}")),
                }
            }

            // 记忆区
            "memory" => {
                reasoner.load_memory_from_json(data).as_ref().map_err(ToString::to_string)?;
                MESSAGE_MEMORY_LOAD_SUCCESS
            }
            // 推理器整体状态
            "status" => {
                reasoner.load_status_from_json(data).as_ref().map_err(ToString::to_string)?;
                MESSAGE_STATUS_LOAD_SUCCESS
            }
        }
    }

    /// 处理有关JSON的交互
    /// * 🎯让`ser_de`模块无需使用[`serde_json`]
    impl Reasoner {
        /// 从JSON加载记忆区
        /// * ⚠️覆盖自身原本的「记忆区」
        fn load_memory_from_json(&mut self, data: impl AsRef<str>) -> anyhow::Result<Memory> {
            let memory = serde_json::from_str(data.as_ref())?;
            let old_memory = self.load_memory(memory);
            Ok(old_memory)
        }

        /// 从JSON加载状态
        /// * ⚠️覆盖自身原本数据
        /// * 🚩【2024-08-12 20:22:42】不返回「推理器状态」数据
        ///   * 💭出于内部使用考虑，不暴露「推理器状态」数据类型
        fn load_status_from_json(&mut self, data: impl AsRef<str>) -> anyhow::Result<()> {
            let mut deserializer_json = serde_json::Deserializer::from_str(data.as_ref());
            self.load_from_deserialized_status(&mut deserializer_json)?;
            Ok(())
        }
    }

    #[cfg(test)]
    mod tests {
        use super::*;
        use crate::{
            control::{
                test_util_ser_de::{status_consistent, GetReasoner},
                DEFAULT_PARAMETERS,
            },
            inference::{
                match_task_and_belief, process_direct, reason, transform_task, InferenceEngine,
            },
            ok,
            storage::tests_memory::{memory_consistent, GetMemory},
            util::AResult,
        };
        use nar_dev_utils::*;
        use navm::output::Output;

        /// 引擎dev
        /// * 🚩【2024-07-09 16:52:40】目前除了「概念推理」均俱全
        /// * ✅【2024-07-14 23:50:15】现集成所有四大推理函数
        const ENGINE_DEV: InferenceEngine = InferenceEngine::new(
            process_direct,
            transform_task,
            match_task_and_belief,
            reason,
        );

        impl RuntimeAlpha {
            /// 测试用：从字符串输入系列NAVM指令
            fn input_cmds(&mut self, inputs: impl AsRef<str>) {
                inputs
                    .as_ref()
                    .lines()
                    .map(str::trim)
                    .filter(|line| !line.is_empty())
                    .map(|line| Cmd::parse(line).expect("NAVM指令{line}解析失败"))
                    .for_each(|cmd| self.input_cmd(cmd))
            }

            /// 测试用：拉取所有已有输出
            fn fetch_outputs(&mut self) -> Vec<Output> {
                list![
                    out
                    while let Some(out) = (self.reasoner.take_output())
                ]
            }

            /// 测试用：打印所有输出
            fn print_outputs(&mut self) {
                self.fetch_outputs()
                    .iter()
                    .for_each(|o| println!("[{}] {}", o.type_name(), o.get_content()))
            }
        }

        /// 测试用：获取推理器
        impl GetReasoner for RuntimeAlpha {
            fn get_reasoner(&self) -> &Reasoner {
                &self.reasoner
            }
        }

        /// 测试用：获取记忆区
        impl GetMemory for RuntimeAlpha {
            fn get_memory(&self) -> &Memory {
                self.reasoner.get_memory()
            }
        }

        fn vm_after_inputs(inputs: impl AsRef<str>) -> RuntimeAlpha {
            let mut reasoner = default_vm();
            reasoner.input_cmds(inputs);
            reasoner
        }

        fn default_vm() -> RuntimeAlpha {
            RuntimeAlpha::new("test", DEFAULT_PARAMETERS, ENGINE_DEV)
        }

        /// 作为样本的输入
        /// * 🎯构造出「经过一定输入之后的推理器」
        const SAMPLE_INPUTS: &str = "
        nse <A --> B>.
        nse <A --> C>.
        nse <C --> B>?
        vol 99
        cyc 20";

        /// 输入NAVM[`SAV`](Cmd::SAV)指令，并从后续的INFO中取出JSON字符串
        /// * 📄推理器状态
        /// * 📄记忆区
        /// * 🚩同时检验「是否有加载成功」
        fn save_xxx_by_cmd(
            reasoner: &mut RuntimeAlpha,
            target: impl Into<String>,
            path: impl Into<String>,
        ) -> String {
            // SAV指令
            let cmd = Cmd::SAV {
                target: target.into(),
                path: path.into(),
            };
            // 输入之前清空旧输出，以避免其它输出干扰
            let _ = reasoner.fetch_outputs();
            reasoner.input_cmd(cmd);
            let outputs = reasoner.fetch_outputs();
            // 记忆区应该被替换了
            // 找到一条「INFO」内容，就直接返回
            for o in outputs {
                if let Output::INFO { message } = o {
                    return message;
                }
            }
            panic!("未找到序列化后的数据");
        }

        /// 将JSON数据以NAVM指令形式输入推理器，让推理器加载指定数据
        /// * 📄推理器状态
        /// * 📄记忆区
        /// * 🚩同时检验「是否有加载成功」
        fn load_xxx_by_cmd(
            reasoner: &mut RuntimeAlpha,
            target: impl Into<String>,
            data: impl Into<String>,
            target_name: &str,
            success_message: &str,
        ) {
            // 将JSON以指令形式封装
            let cmd = Cmd::LOA {
                target: target.into(),
                path: data.into(),
            };
            // 打包成NAVM指令，加载进推理器
            reasoner.input_cmd(cmd);
            let outputs = reasoner.fetch_outputs();
            // 推理器部分内容应该被替换了
            assert!(
                // 检查是否有一条【类型为INFO】且内容为「加载成功」的输出
                outputs.iter().any(|o| matches!(
                    o,
                    Output::INFO { message }
                    if message == success_message
                )),
                "{target_name}没有被替换: {outputs:?}",
            );
        }

        /// 将JSON数据以NAVM指令形式输入推理器，让推理器加载记忆区
        /// * 🚩同时检验「是否有加载成功」
        fn load_memory_by_cmd(vm: &mut RuntimeAlpha, data: impl Into<String>) {
            load_xxx_by_cmd(vm, "memory", data, "记忆区", MESSAGE_MEMORY_LOAD_SUCCESS)
        }

        /// 将JSON数据以NAVM指令形式输入推理器，让推理器加载状态
        /// * 🚩同时检验「是否有加载成功」
        fn load_status_by_cmd(vm: &mut RuntimeAlpha, data: impl Into<String>) {
            load_xxx_by_cmd(
                vm,
                "status",
                data,
                "推理器状态",
                MESSAGE_STATUS_LOAD_SUCCESS,
            )
        }

        #[test]
        fn load_memory_from_json() -> AResult {
            // 一定推理后的推理器
            let mut vm = vm_after_inputs(SAMPLE_INPUTS);
            // 记忆区序列化成JSON
            let data = save_xxx_by_cmd(&mut vm, "memory", "");
            // 从JSON加载记忆区
            let old_memory = vm.reasoner.load_memory_from_json(&data)?;
            // 旧的记忆区应该与新的一致
            memory_consistent(&old_memory, &vm)?;

            // 将JSON以指令形式封装，让推理器从指令中加载记忆区
            load_memory_by_cmd(&mut vm, data.clone());

            // 旧的记忆区应该与新的一致
            memory_consistent(&old_memory, &vm)?;

            // ✅成功，输出附加信息 | ❌【2024-08-12 13:21:22】下面俩太卡了
            println!("Memory reloading success!");
            println!("data = {data}");

            ok!()
        }

        /// 将记忆区加载到其它空推理器中，实现「分支」效果
        #[test]
        fn load_memory_to_other_reasoners() -> AResult {
            // 一定推理后的推理器
            let mut vm = vm_after_inputs(SAMPLE_INPUTS);
            // 记忆区序列化成JSON
            let data = save_xxx_by_cmd(&mut vm, "memory", "");
            // 从JSON加载记忆区
            let old_memory = vm.reasoner.load_memory_from_json(&data)?;
            // 旧的记忆区应该与新的一致
            memory_consistent(&old_memory, &vm)?;

            // * 🚩以纯数据形式加载到新的「空白推理器」中 * //
            // 创建新的空白推理器
            let mut vm2 = default_vm();
            // 从JSON加载记忆区
            let old_memory2 = vm2.reasoner.load_memory_from_json(&data)?;
            let consistent_on_clone = |vm2: &RuntimeAlpha| -> AResult {
                // 但新的记忆区应该与先前旧的记忆区一致
                memory_consistent(&old_memory, vm2)?;
                // 同时，俩推理器现在记忆区一致
                memory_consistent(&vm, vm2)?;
                ok!()
            };
            // 空白的记忆区应该与新的不一致
            memory_consistent(&old_memory2, &vm2).expect_err("意外的记忆区一致");
            // 被重复加载的记忆区应该一致
            consistent_on_clone(&vm2)?;

            // * 🚩以NAVM指令形式加载到新的「空白推理器」中 * //
            // 创建新的空白推理器
            let mut reasoner3 = default_vm();
            // 从JSON加载记忆区
            load_memory_by_cmd(&mut reasoner3, data.clone());
            // 被重复加载的记忆区应该一致
            consistent_on_clone(&reasoner3)?;

            // * 🚩分道扬镳的推理歧路 * //
            // 推理器2
            vm2.input_cmds(
                "
                nse (&&, <A --> C>, <A --> B>).
                cyc 10
                inf concepts
                inf summary
                ",
            );
            // 推理器3
            reasoner3.input_cmds(
                "
                nse <C --> D>.
                nse <A --> D>?
                cyc 10
                inf concepts
                inf summary
                ",
            );
            println!("reasoner:");
            vm.print_outputs();
            println!("reasoner 2:");
            vm2.print_outputs();
            println!("reasoner 3:");
            reasoner3.print_outputs();
            // 现在推理器（的记忆区）应该两两不一致
            memory_consistent(&vm, &vm2).expect_err("意外的记忆区一致");
            memory_consistent(&vm, &reasoner3).expect_err("意外的记忆区一致");
            memory_consistent(&vm2, &reasoner3).expect_err("意外的记忆区一致");
            ok!()
        }

        /// 加载状态
        /// ! 💫【2024-08-12 22:23:23】因为「推理器内部类型不暴露在外」，所以「单推理器加载状态后，用旧的状态与新的状态对比」难以安排
        /// * 🚩【2024-08-12 22:23:26】目前采用「创建多个推理器，保留一个作为『旧状态』」的方式
        ///   * 📝核心想法：既然「一致性」比对的是推理器，那多创建两个一样的不就好了……
        #[test]
        fn load_status_from_json() -> AResult {
            // 一定推理后的推理器 样本
            let vm_old = vm_after_inputs(SAMPLE_INPUTS);
            let mut vm = vm_after_inputs(SAMPLE_INPUTS);
            // 状态序列化成JSON
            let data = save_xxx_by_cmd(&mut vm, "status", "");
            // 从JSON加载状态
            vm.reasoner.load_status_from_json(&data)?;
            // 旧的状态应该与新的一致
            status_consistent(&vm_old, &vm)?;

            // 将JSON以指令形式封装，让推理器从指令中加载状态
            load_status_by_cmd(&mut vm, data.clone());

            // 旧的状态应该与新的一致
            status_consistent(&vm_old, &vm)?;

            // ✅成功，输出附加信息 | ❌【2024-08-12 13:21:22】下面俩太卡了
            println!("Status reloading success!");
            println!("data = {data}");

            ok!()
        }

        /// 将状态加载到其它空推理器中，实现「分支」效果
        #[test]
        fn load_status_to_other_reasoners() -> AResult {
            // 一定推理后的推理器
            let old_vm = vm_after_inputs(SAMPLE_INPUTS);
            let mut vm = vm_after_inputs(SAMPLE_INPUTS);
            // 状态序列化成JSON
            let data = save_xxx_by_cmd(&mut vm, "status", "");
            // 从JSON加载状态
            vm.reasoner.load_status_from_json(&data)?;
            // 旧的状态应该与新的一致
            status_consistent(&old_vm, &vm)?;

            // * 🚩以纯数据形式加载到新的「空白推理器」中 * //
            // 创建新的空白推理器
            let old_vm2 = default_vm();
            let mut vm2 = default_vm();
            // 从JSON加载状态
            vm2.reasoner.load_status_from_json(&data)?;
            let consistent_on_clone = |vm2: &RuntimeAlpha| -> AResult {
                // 但新的状态应该与先前旧的状态一致
                status_consistent(&old_vm, vm2)?;
                // 同时，俩推理器现在状态一致
                status_consistent(&vm, vm2)?;
                ok!()
            };
            // 空白的状态应该与新的不一致
            status_consistent(&old_vm2, &vm2).expect_err("意外的状态一致");
            // 被重复加载的状态应该一致
            consistent_on_clone(&vm2)?;

            // * 🚩以NAVM指令形式加载到新的「空白推理器」中 * //
            // 创建新的空白推理器
            let mut vm3 = default_vm();
            // 从JSON加载状态
            load_status_by_cmd(&mut vm3, data.clone());
            // 被重复加载的状态应该一致
            consistent_on_clone(&vm3)?;

            // * 🚩分道扬镳的推理歧路 * //
            // 推理器2
            vm2.input_cmds(
                "
                nse (&&, <A --> C>, <A --> B>).
                cyc 10
                inf concepts
                inf tasks
                inf summary
                ",
            );
            // 推理器3
            vm3.input_cmds(
                "
                nse <C --> D>.
                nse <A --> D>?
                cyc 10
                inf concepts
                inf tasks
                inf summary
                ",
            );
            println!("reasoner:");
            vm.print_outputs();
            println!("reasoner 2:");
            vm2.print_outputs();
            println!("reasoner 3:");
            vm3.print_outputs();
            // 现在推理器（的状态）应该两两不一致
            status_consistent(&vm, &vm2).expect_err("意外的状态一致");
            status_consistent(&vm, &vm3).expect_err("意外的状态一致");
            status_consistent(&vm2, &vm3).expect_err("意外的状态一致");
            ok!()
        }
    }
}
use cmd_loa::*;

use super::RuntimeAlpha;
