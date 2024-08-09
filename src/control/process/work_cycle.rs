//! 基于「推理器」「推理上下文」有关「推理周期」的操作
//! * 🎯从「记忆区」中解耦分离
//! * 🎯在更「现代化」的同时，也使整个过程真正Rusty
//!   * 📌【2024-05-15 01:38:39】至少，能在「通过编译」的条件下复现OpenNARS
//! * 🎯将其中有关「推理周期」的代码摘录出来
//!   * 📌工作周期 from 推理器
//!   * 📌吸收推理上下文(新)
//! * 🚩【2024-05-17 21:35:04】目前直接基于「推理器」而非「记忆区」
//! * ⚠️【2024-05-18 01:25:09】目前这里所参考的「OpenNARS源码」已基本没有「函数对函数」的意义
//!   * 📌许多代码、逻辑均已重构重组
//!
//! ## Logs
//!
//! * ✅【2024-05-12 16:10:24】基本从「记忆区」迁移完所有功能
//! * ♻️【2024-06-26 11:59:58】开始根据改版OpenNARS重写

use crate::{
    control::Reasoner,
    entity::{Concept, Sentence, TLink, Task},
    global::ClockTime,
    inference::{Budget, Evidential},
    util::{RefCount, ToDisplayAndBrief},
};
use cmd_hlp::hlp_dispatch;
use nar_dev_utils::{join, list, JoinTo};
use navm::cmd::Cmd;

impl Reasoner {
    /* 时钟相关 */

    /// 获取时钟时间
    pub fn time(&self) -> ClockTime {
        self.clock
    }

    pub fn init_timer(&mut self) {
        self.set_timer(0);
    }

    pub fn tick_timer(&mut self) {
        self.timer += 1;
    }

    pub fn timer(&self) -> usize {
        self.timer
    }

    pub fn set_timer(&mut self, timer: usize) {
        self.timer = timer;
    }
}

// ! 弃用
// /// 推理器时钟控制
// impl Reasoner {
//     /// # 📄OpenNARS
//     ///
//     /// Start the inference process
//     pub fn run(&mut self) {
//         self.running = true;
//     }

//     /// # 📄OpenNARS
//     ///
//     /// Will carry the inference process for a certain number of steps
//     pub fn walk(&mut self, steps: usize) {
//         self.walking_steps = steps;
//     }

//     /// # 📄OpenNARS
//     ///
//     /// Will stop the inference process
//     pub fn stop(&mut self) {
//         self.running = false;
//     }
// }

/// 推理器步进
impl Reasoner {
    /// 推理循环
    /// * 🚩只负责推理，不处理输入输出
    ///   * 📌在「处理输入」的同时，也可能发生「推理循环」（`CYC`指令）
    pub fn cycle(&mut self, steps: usize) {
        for _ in 0..steps {
            self.handle_work_cycle();
        }
    }

    /// 处理输入输出
    /// * 🚩负责处理输入输出，并**有可能触发推理循环**
    ///   * 📌输入的`CYC`指令 会【立即】触发工作周期
    ///   * 💭【2024-06-29 01:41:03】这样的机制仍有其必要性
    ///     * 💡不同通道的指令具有执行上的优先级
    ///     * 💡每个操作都是【原子性】的，执行过程中顺序先后往往影响最终结果
    pub fn handle_io(&mut self) {
        // * 🚩处理输入（可能会有推理器步进）
        self.handle_input();
        // * 🚩处理输出
        self.handle_output();
    }

    /// 处理输入：遍历所有通道，拿到指令
    fn handle_input(&mut self) {
        // * 🚩遍历所有通道，拿到要执行的指令（序列）
        let input_cmds = self.fetch_cmd_from_input();
        // * 🚩在此过程中执行指令，相当于「在通道中调用`textInputLine`」
        for cmd in input_cmds {
            self.input_cmd(cmd);
        }
    }

    /// 处理输出
    fn handle_output(&mut self) {
        let outputs = list![
            {output}
            while let Some(output) = (self.take_output())
        ];
        if !outputs.is_empty() {
            // * 🚩先将自身通道中的元素挪出（在此过程中筛除），再从此临时通道中计算与获取输入（以便引用自身）
            let mut channels = list![
                {channel} // * ⚠️注意：此时顺序是倒过来的
                while let Some(channel) = (self.io_channels.output_channels.pop()) // * 此处挪出
                if (!channel.need_remove()) // * 此处筛除
            ];
            // * 🚩逆序纠正
            channels.reverse();
            // * 🚩遍历（并可引用自身）
            for channel_out in channels.iter_mut() {
                // * 🚩在此过程中解读输出
                channel_out.next_output(/* self,  */ &outputs);
            }
            // * 🚩放回
            self.io_channels.output_channels.extend(channels);
        }
    }

    fn handle_work_cycle(&mut self) {
        // * 🚩处理时钟
        self.clock += 1;
        self.tick_timer();
        // * 🚩工作周期
        self.work_cycle();
    }
}

/// 工作周期
impl Reasoner {
    fn work_cycle(&mut self) {
        self.report_comment(format!("--- {} ---", self.time()));

        // * 🚩本地任务直接处理 阶段 * //
        let has_result = self.process_direct();

        // * 🚩内部概念高级推理 阶段 * //
        // * 📝OpenNARS的逻辑：一次工作周期，只能在「直接推理」与「概念推理」中选择一个
        if !has_result {
            self.process_reason();
        }

        // * 🚩最后收尾 阶段 * //
        // * 🚩原「清空上下文」已迁移至各「推理」阶段
        // ! ❌不复刻「显示呈现」相关功能
    }

    /// 从输入通道中拿取一个[NAVM指令](Cmd)
    fn fetch_cmd_from_input(&mut self) -> Vec<Cmd> {
        let mut input_cmds = vec![];
        // * 🚩先将自身通道中的元素挪出（在此过程中筛除），再从此临时通道中计算与获取输入（以便引用自身）
        let mut channels = list![
            {channel} // * ⚠️注意：此时顺序是倒过来的
            while let Some(channel) = (self.io_channels.input_channels.pop()) // * 此处挪出
            if (!channel.need_remove()) // * 此处筛除
        ];
        // * 🚩逆序纠正
        channels.reverse();
        // * 🚩遍历（并可引用自身）
        let mut reasoner_should_run = false;
        for channel_in in channels.iter_mut() {
            // * 📝Java的逻辑运算符也是短路的——此处使用预先条件以避免运算
            // * ❓这是否意味着，一次只有一个通道能朝OpenNARS输入
            if !reasoner_should_run {
                let (run, cmds) = channel_in.next_input(/* self */);
                reasoner_should_run = run;
                // * 🆕直接用其输出扩展
                // * 💭但实际上只有一次
                input_cmds.extend(cmds);
            }
        }
        // * 🚩放回
        self.io_channels.input_channels.extend(channels);
        // * 🚩返回
        input_cmds
    }

    /// 模拟改版`Reasoner.inputTask`
    /// * 🚩【2024-05-07 22:51:11】在此对[`Budget::budget_above_threshold`](crate::inference::Budget::budget_above_threshold)引入[「预算阈值」超参数](crate::control::Parameters::budget_threshold)
    /// * 🚩【2024-05-17 15:01:06】自「记忆区」迁移而来
    ///
    /// # 📄OpenNARS
    ///
    /// Input task processing. Invoked by the outside or inside environment.
    /// Outside: StringParser (input); Inside: Operator (feedback). Input tasks
    /// with low priority are ignored, and the others are put into task buffer.
    ///
    /// @param task The input task
    fn input_task(&mut self, task: Task) {
        let budget_threshold = self.parameters.budget_threshold;
        if task.budget_above_threshold(budget_threshold) {
            // ? 💭【2024-05-07 22:57:48】实际上只需要输出`IN`即可：日志系统不必照着OpenNARS的来
            // * 🚩此处两个输出合而为一
            self.report_in(&task);
            // * 📝只追加到「新任务」里边，并不进行推理
            self.derivation_datas.add_new_task(task);
        } else {
            // 此时还是输出一个「被忽略」好
            self.report_comment(format!("!!! Neglected: {}", task.to_display_long()));
        }
    }

    // ! 🚩【2024-06-28 00:09:12】方法「吸收推理上下文」不再需要被「推理器」实现
    // * 📌原因：现在「推理上下文」内置「推理器」的引用
}

/// 输入指令
impl Reasoner {
    /// 模拟`ReasonerBatch.textInputLine`
    /// * 🚩🆕【2024-05-13 02:27:07】从「字符串输入」变为「NAVM指令输入」
    /// * 🚩【2024-06-29 01:42:46】现在不直接暴露「输入NAVM指令」：全权交给「通道」机制
    ///   * 🚩由「通道」的「处理IO」引入
    fn input_cmd(&mut self, cmd: Cmd) {
        match cmd {
            // Cmd::SAV { target, path } => (),
            // Cmd::LOA { target, path } => (),
            // * 🚩重置：推理器复位
            Cmd::RES { .. } => self.reset(),
            // * 🚩Narsese：输入任务（但不进行推理）
            Cmd::NSE(narsese) => self.cmd_nse(narsese),
            // Cmd::NEW { target } => (),
            // Cmd::DEL { target } => (),
            // * 🚩工作周期：只执行推理，不处理输入输出
            Cmd::CYC(cycles) => self.cycle(cycles),
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
            _ => self.report_error(format!("Unknown cmd: {cmd}")),
        }
    }

    /// 处理指令[`Cmd::NSE`]
    fn cmd_nse(&mut self, narsese: narsese::lexical::Task) {
        // * 🚩更新「当前时间戳序列号」
        let stamp_current_serial = self.updated_stamp_current_serial();
        // * 🚩解析并使用结果
        match self.parse_task(narsese, stamp_current_serial) {
            // * 🚩解析成功⇒输入任务
            // * 🚩【2024-05-17 16:28:53】现在无需输入任务
            Ok(task) => self.input_task(task),
            // * 🚩解析失败⇒报告错误
            Err(e) => self.report_error(format!("Narsese任务解析错误：{e}",)),
        }
    }

    /// 处理指令[`Cmd::VOL`]
    fn cmd_vol(&mut self, volume: usize) {
        self.report_info(format!("volume: {} => {volume}", self.volume));
        self.volume = volume;
    }

    /// 处理指令[`Cmd::EXI`]
    ///
    /// ? ❓【2024-07-23 16:10:13】是否一定要主程序退出
    ///   * 💭还是说，NARS本身并没有个实际上的「退出」机制
    fn cmd_exi(&mut self, reason: String) {
        // * 🚩最后的提示性输出
        self.report_info(format!("Program exited with reason {reason:?}"));
        // * 🚩处理所有输出
        self.handle_output();
        // * 🚩最终退出程序
        std::process::exit(0);
    }

    /// 处理指令[`Cmd::INF`]
    fn cmd_inf(&mut self, source: String) {
        // 查询
        let query = source.to_lowercase();
        // 消息分派 | 📌只在此处涉及「报告输出」
        match inf_dispatch(self, query) {
            // 正常信息⇒报告info
            Ok(message) => self.report_info(message),
            // 错误信息⇒报告error
            Err(message) => self.report_error(message),
        }
    }

    /// 处理指令[`Cmd::HLP`]
    fn cmd_hlp(&mut self, name: String) {
        // 查询
        let query = name.to_lowercase();
        // 消息分派 | 📌只在此处涉及「报告输出」
        match hlp_dispatch(self, query) {
            // 正常信息⇒报告info
            Ok(message) => self.report_info(message),
            // 错误信息⇒报告error
            Err(message) => self.report_error(message),
        }
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
        let message = macro_once! {
            macro ( $( $query:literal => $message:expr )* ) => {
                const HELP_QUERIES_LIST: &str = concat!(
                    $( "\n- ", $query, )*
                );
                match query.as_ref() {
                    /// 特殊/空字串：列举已有的所有参数
                    "" => format!("Available help queries: {HELP_QUERIES_LIST}"),
                    // 所有已有的帮助命令
                    $( $query => $message.to_string(), )*
                    // 未知的查询关键词
                    other => return Err(format!("Unknown help query: {other:?}\nAvailable help queries: {HELP_QUERIES_LIST}")),
                }
            }
            "inf" => CMD_INF // 展示有关命令`INF`的帮助
        };
        Ok(message)
    }

    /// 有关指令 [`INF`](Cmd::INF) 的帮助
    const CMD_INF: &str = "
# cmd `INF`
- Format: `INF <qualifier><target>`
- qualifiers:
  - `#`: Detailed info
- targets:
  - `memory`: Memory
  - `reasoner`: Reasoner
  - `tasks`: Tasks in reasoner
  - `concepts`: Concepts in memory
  - `links`: Task-links and term-links in each concepts
";
}
/// 专用于指令[`Cmd::INF`]的处理函数
mod cmd_inf {
    use super::*;
    use nar_dev_utils::macro_once;

    /// 指令[`Cmd::INF`]的入口函数
    /// * 📌传入的`query`默认为小写字串引用
    /// * 📌输出仅为一个消息字符串；若返回[错误值](Err)，则视为「报错」
    pub fn inf_dispatch(reasoner: &mut Reasoner, query: impl AsRef<str>) -> Result<String, String> {
        let message = macro_once! {
            macro ( $( $query:literal => $message:expr )* ) => {
                const INF_QUERIES_LIST: &str = concat!(
                    $( "\n- ", $query, )*
                );
                match query.as_ref() {
                    // * 🚩空消息⇒列举所有query并转接`HLP INF`
                    "" => format!(
                        "Available help queries: {INF_QUERIES_LIST}\n\nAnd more info:{}",
                        cmd_hlp::hlp_dispatch(reasoner, "inf")?
                    ),
                    // 所有已有的帮助命令
                    $( $query => $message.to_string(), )*
                    // * 🚩其它⇒告警
                    other => return Err(format!("Unknown info query: {other:?}")),
                }
            }

            // * 🚩普通信息查询
            "memory" => format!("Memory: {:?}", reasoner.memory) // 整个记忆区
            "reasoner" => format!("Reasoner: {reasoner:?}")      // 整个推理器
            "tasks" => reasoner.report_tasks()                   // 推理器中所有任务
            "concepts" => reasoner.report_concepts()             // 推理器中所有概念
            "links" => reasoner.report_links()                   // 推理器中所有链接

            // * 🚩更详尽的信息
            "#memory" => format!("Memory:\n{:#?}", reasoner.memory) // 具有缩进层级
            "#reasoner" => format!("Reasoner:\n{reasoner:#?}")      // 具有缩进层级
            "#tasks" => reasoner.report_task_detailed()             // 推理器中的任务派生链
            "#concepts" => reasoner.report_concepts_detailed()      // 推理器中所有概念，含任务链、词项链
            "#links" => reasoner.report_links_detailed()            // 推理器中所有链接，含预算值
        };
        Ok(message)
    }

    impl Reasoner {
        /// 收集推理器内所有的「任务」
        /// * 🎯包括如下地方
        ///   * 新任务列表
        ///   * 新近任务袋
        ///   * 任务链目标
        ///   * 问题表
        /// * 📌所有收集到的「任务」不会重复
        ///   * 📝对于[`Rc`]，Rust中使用[`Rc::ptr_eq`]判等
        ///   * 💡亦可【直接从引用取指针】判等
        fn collect_tasks_map<T>(&self, map: impl Fn(&Task) -> T) -> Vec<T> {
            let mut outputs = vec![];
            // 获取所有引用地址：通过地址判断是否引用到了同一任务
            let mut target_locations = vec![];
            /// 判断引用是否唯一
            fn ref_unique(task_refs: &[*const Task], task_location: *const Task) -> bool {
                !task_refs
                    .iter()
                    .any(|ptr_location: &*const Task| *ptr_location == task_location)
            }
            let mut deal_ref = |task_ref: &Task| {
                // 取地址
                let task_location = task_ref as *const Task;
                // 不能有任何一个引用重复
                if ref_unique(&target_locations, task_location) {
                    // 加入被记录在案的地址
                    target_locations.push(task_location);
                    // 添加到输出
                    outputs.push(map(task_ref));
                }
            };

            // 记忆区的「所有任务」
            self.memory
                .iter_concepts()
                .flat_map(Concept::iter_tasks)
                .for_each(|task_cell| deal_ref(&task_cell.get_())); // 取引用并添加

            // 新任务列表、新近任务袋中的「所有任务」
            let new_tasks = self.iter_new_tasks();
            let novel_tasks = self.iter_novel_tasks();
            new_tasks.chain(novel_tasks).for_each(deal_ref); // 添加

            // 输出
            outputs
        }

        /// 报告推理器内的所有「任务」
        pub(super) fn report_tasks(&self) -> String {
            format!(
                "Tasks in reasoner:\n{}", // 开始组织格式化
                self.collect_tasks_map(fmt_task)
                    .into_iter()
                    .join_to_new("\n")
            )
        }

        /// 详尽报告推理器内所有「任务」（的派生关系）
        pub(super) fn report_task_detailed(&self) -> String {
            format!(
                // 任务派生链
                "Tasks in reasoner:\n{}",
                // 开始组织格式化
                self.collect_tasks_map(format_task_chain_detailed)
                    .into_iter()
                    .flatten()
                    .join_to_new("\n\n") // 任务之间两行分隔
            )
        }

        /// 按指定函数格式化推理器内的所有「概念」
        fn format_concepts(&self, fmt: impl Fn(&Concept) -> String) -> String {
            // 开始组织格式化
            self.memory.iter_concepts().map(fmt).join_to_new("\n\n")
        }

        /// 报告推理器内的所有「概念」
        pub(super) fn report_concepts(&self) -> String {
            format!(
                "Concepts in memory:\n{}",
                self.memory
                    .iter_concepts()
                    .map(|c| format!("- {}", c.term()))
                    .join_to_new("\n") // 只展示所有词项
            )
        }

        /// 详尽报告推理器内的所有「概念」
        pub(super) fn report_concepts_detailed(&self) -> String {
            format!(
                "# Concepts in memory\n{}",
                self.format_concepts(|c| format!("## Concept @ {}", c.to_display_long()))
            )
        }

        /// 报告内部所有链接（仅词项）
        pub(super) fn report_links(&self) -> String {
            format!(
                "Links in memory:\n{}",
                self.memory
                    .iter_concepts()
                    .map(format_concept_links)
                    .join_to_new("\n") // 只展示所有词项
            )
        }

        /// 详尽报告内部所有链接
        pub(super) fn report_links_detailed(&self) -> String {
            format!(
                "Links in memory:\n{}",
                self.memory
                    .iter_concepts()
                    .map(format_concept_links_detailed)
                    .join_to_new("\n") // 只展示所有词项
            )
        }
    }

    /// 组织一个[任务](Task)的格式
    fn fmt_task(task: &Task) -> String {
        format!("Task#{} {}", task.creation_time(), task.to_display_long())
    }

    /// 简略组织一个[任务](Task)的格式
    /// * 🎯需求：所有信息均在一行之内
    fn format_task_brief(task: &Task) -> String {
        format!(
            "Task#{} \"{}{}\"",
            task.creation_time(), // ! 这个不保证不重复
            task.content(),
            task.punctuation() // * 🚩【2024-08-09 00:28:05】目前从简：不显示真值、预算值（后两者可从`tasks`中查询）
        )
    }

    /// 详尽展示一条「任务派生链」
    /// * ⚠️可能失败：父任务可能不存在
    fn format_task_chain_detailed(root: &Task) -> Option<String> {
        // 开始组织
        Some(join! {
            // 当前任务
            => format_task_brief(root)
            // 逐个加入其父任务
            => (join! {
                => "\n <- {}".to_string()
                => format_task_brief(&parent_task.get_())
                => (format!(
                    " + Belief#{} \"{}\"",
                    belief.creation_time(), // ! 这个不保证不重复
                    belief.to_display()
                )) if let Some(belief) = parent_belief
            }) for (parent_task, parent_belief) in root.parents()
        })
    }

    /// 展示一个「概念」的链接
    fn format_concept_links(c: &Concept) -> String {
        format!(
            "- {}\n{}\n{}",
            c.term(),
            c.iter_term_links() // 词项链
                .map(|l| format!("  -> {}", &*l.target(),))
                .join_to_new("\n"),
            c.iter_task_links() // 任务链
                .map(|l| format!("  ~> {}", l.target().content(),))
                .join_to_new("\n")
        )
    }

    /// 详尽展示一个「概念」的链接
    fn format_concept_links_detailed(c: &Concept) -> String {
        format!(
            "- {}\n{}\n{}",
            c.term(),
            c.iter_term_links() // 词项链
                .map(|l| format!("  -> {} {}", l.budget_to_display(), &*l.target(),))
                .join_to_new("\n"),
            c.iter_task_links() // 任务链
                .map(|l| format!(
                    "  ~> {} {}{}",
                    l.budget_to_display(),
                    l.target().content(),
                    l.target().punctuation(),
                ))
                .join_to_new("\n")
        )
    }
}
use cmd_inf::*;
