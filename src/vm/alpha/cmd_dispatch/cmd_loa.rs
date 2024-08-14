use crate::{control::Reasoner, storage::Memory};
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
        vm::alpha::RuntimeAlpha,
    };
    use nar_dev_utils::*;
    use navm::{cmd::Cmd, output::Output};

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
