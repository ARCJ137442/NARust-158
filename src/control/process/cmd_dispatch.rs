//! 集中管理有关「推理器分派处理指令」的函数

use crate::{
    control::Reasoner,
    entity::{Concept, Sentence, TLink, Task},
    inference::{Budget, Evidential},
    util::{RefCount, ToDisplayAndBrief},
};
use nar_dev_utils::{join, JoinTo};
use navm::cmd::Cmd;

/// 输入指令
impl Reasoner {
    /// 模拟`ReasonerBatch.textInputLine`
    /// * 🚩🆕【2024-05-13 02:27:07】从「字符串输入」变为「NAVM指令输入」
    /// * 🚩【2024-06-29 01:42:46】现在不直接暴露「输入NAVM指令」：全权交给「通道」机制
    ///   * 🚩由「通道」的「处理IO」引入
    pub(super) fn input_cmd(&mut self, cmd: Cmd) {
        match cmd {
            Cmd::SAV { target, path } => self.cmd_sav(target, path),
            Cmd::LOA { target, path } => self.cmd_loa(target, path),
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

    /// 处理指令[`Cmd::SAV`]
    fn cmd_sav(&mut self, target: String, path: String) {
        // 查询
        let query = target.to_lowercase();
        // 消息分派 | 📌只在此处涉及「报告输出」
        match sav_dispatch(self, query, path) {
            // 正常信息⇒报告info
            Ok(message) => self.report_info(message),
            // 错误信息⇒报告error
            Err(message) => self.report_error(message),
        }
    }

    /// 处理指令[`Cmd::LOA`]
    fn cmd_loa(&mut self, target: String, data: String) {
        // 查询
        let query = target.to_lowercase();
        // 消息分派 | 📌只在此处涉及「报告输出」
        match loa_dispatch(self, query, data) {
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
  - `memory`: Memory
  - `reasoner`: Reasoner
  - `tasks`: Tasks in reasoner
  - `concepts`: Concepts in memory
  - `links`: Task-links and term-links in each concepts
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
    use crate::{
        entity::Judgement,
        global::Float,
        inference::Truth,
        language::Term,
        util::{AverageFloat, AverageUsize},
    };
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
                    other => Err(format!("Unknown info query: {other:?}")),
                }
            }

            // * 🚩普通信息查询
            "memory" => format!("Memory: {:?}", reasoner.memory)             // 整个记忆区
            "reasoner" => format!("Reasoner: {reasoner:?}")                  // 整个推理器
            "parameters" => format!("Parameters: {:?}", reasoner.parameters) // 推理器的超参数
            "tasks" => reasoner.report_tasks()                               // 推理器中所有任务
            "beliefs" => reasoner.report_beliefs()                           // 推理器中所有信念
            "questions" => reasoner.report_questions()                       // 推理器中所有问题
            "concepts" => reasoner.report_concepts()                         // 推理器中所有概念
            "links" => reasoner.report_links()                               // 推理器中所有链接
            "summary" => reasoner.report_summary()                               // 推理器中所有链接

            // * 🚩更详尽的信息
            "#memory" => format!("Memory:\n{:#?}", reasoner.memory)             // 具有缩进层级
            "#reasoner" => format!("Reasoner:\n{reasoner:#?}")                  // 具有缩进层级
            "#parameters" => format!("Parameters:\n{:#?}", reasoner.parameters) // 具有缩进层级
            "#tasks" => reasoner.report_tasks_detailed()                         // 推理器中的任务派生链
            "#beliefs" => reasoner.report_beliefs_detailed()                     // 推理器中所有信念（详细）
            "#questions" => reasoner.report_questions_detailed()                 // 推理器中所有问题（详细）
            "#concepts" => reasoner.report_concepts_detailed()                  // 推理器中所有概念，含任务链、词项链
            "#links" => reasoner.report_links_detailed()                        // 推理器中所有链接，含预算值
        }
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
        fn report_tasks(&self) -> String {
            format!(
                "Tasks in reasoner:\n{}", // 开始组织格式化
                self.collect_tasks_map(format_task)
                    .into_iter()
                    .join_to_new("\n")
            )
        }

        /// 详尽报告推理器内所有「任务」（的派生关系）
        fn report_tasks_detailed(&self) -> String {
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

        /// 报告推理器内的所有「信念」
        fn report_beliefs(&self) -> String {
            format!(
                "Beliefs in reasoner:\n{}", // 开始组织格式化
                self.memory
                    .iter_concepts()
                    .flat_map(Concept::iter_beliefs)
                    .map(format_belief)
                    .join_to_new("\n")
            )
        }

        /// 详尽报告推理器内所有「信念」
        fn report_beliefs_detailed(&self) -> String {
            format!(
                "Beliefs in reasoner:\n{}", // 开始组织格式化
                self.memory
                    .iter_concepts()
                    .flat_map(Concept::iter_beliefs)
                    .map(format_belief_detailed)
                    .join_to_new("\n")
            )
        }

        /// 报告推理器内的所有「问题」
        fn report_questions(&self) -> String {
            format!(
                "Questions in reasoner:\n{}", // 开始组织格式化
                self.collect_tasks_map(fmt_question(format_task))
                    .into_iter()
                    .flatten()
                    .join_to_new("\n")
            )
        }

        /// 详尽报告推理器内所有「问题」（的派生关系）
        fn report_questions_detailed(&self) -> String {
            format!(
                // 任务派生链
                "Questions in reasoner:\n{}",
                // 开始组织格式化
                self.collect_tasks_map(fmt_question(format_task_chain_detailed))
                    .into_iter()
                    .flatten()
                    .flatten()
                    .join_to_new("\n\n") // 任务之间两行分隔
            )
        }

        /// 按指定函数格式化推理器内的所有「概念」
        fn fmt_concepts(&self, fmt: impl Fn(&Concept) -> String) -> String {
            // 开始组织格式化
            self.memory.iter_concepts().map(fmt).join_to_new("\n\n")
        }

        /// 报告推理器内的所有「概念」
        fn report_concepts(&self) -> String {
            format!(
                "Concepts in memory:\n{}",
                self.memory
                    .iter_concepts()
                    .map(|c| format!("- {}", c.term()))
                    .join_to_new("\n") // 只展示所有词项
            )
        }

        /// 详尽报告推理器内的所有「概念」
        fn report_concepts_detailed(&self) -> String {
            format!(
                "# Concepts in memory\n{}",
                self.fmt_concepts(|c| format!("## Concept @ {}", c.to_display_long()))
            )
        }

        /// 报告内部所有链接（仅词项）
        fn report_links(&self) -> String {
            format!(
                "Links in memory:\n{}",
                self.memory
                    .iter_concepts()
                    .map(format_concept_links)
                    .join_to_new("\n") // 只展示所有词项
            )
        }

        /// 详尽报告内部所有链接
        fn report_links_detailed(&self) -> String {
            format!(
                "Links in memory:\n{}",
                self.memory
                    .iter_concepts()
                    .map(format_concept_links_detailed)
                    .join_to_new("\n") // 只展示所有词项
            )
        }

        /// 报告自身状况概要
        /// * 💡【2024-08-09 18:12:57】灵感源自ONA
        ///   * 📝复现方式：`NAR.exe shell`后 Ctrl+D 触发EOF
        /// * 📌格式：Markdown
        /// * 📝概念：「原生信息/次生信息」
        ///   * 📌「原生信息」：只能从推理器内部信息获得的信息，如「系统内的概念数量」「系统内的任务数量」
        ///   * 📌「次生信息」：可以从其它「原生信息」推算出来的信息，如「系统内每个概念平均持有的任务数量」
        fn report_summary(&self) -> String {
            // 预先计算可重用的统计数据
            let iter_concepts = self.memory.iter_concepts().collect::<Vec<_>>(); // 避免重复计算引用
            let iter_concepts = || iter_concepts.iter().cloned(); // 若复制了整个「概念」则会编译报错
            let iter_beliefs = || iter_concepts().flat_map(Concept::iter_beliefs);
            let iter_questions = || iter_concepts().flat_map(Concept::iter_questions);
            let iter_inputted_questions = || iter_questions().filter(|q| q.get_().is_input()); // 用户输入的问题，用于区分「系统派生的问题」
            let iter_concept_complexity =
                || iter_concepts().map(Concept::term).map(Term::complexity);
            // let iter_tasks = || self.collect_tasks_map(|t| t); // ! 不能这样做：有些任务的引用在Rc里，不能随意脱离生命周期
            let iter_tasks_complexity = || {
                self.collect_tasks_map(|t| t.content().complexity())
                    .into_iter()
            };
            let iter_beliefs_complexity =
                || iter_beliefs().map(Sentence::content).map(Term::complexity);
            let iter_questions_complexity =
                || iter_questions().map(|t| t.get_().content().complexity());

            let n_concepts = iter_concepts().count();
            let n_tasks = self.collect_tasks_map(|_| ()).len(); // * 📌使用ZST闭包统计（不重复的）任务数量
            let n_beliefs = iter_beliefs().count();
            let n_questions = iter_questions().count();
            let n_inputted_questions = iter_inputted_questions().count();
            let n_questions_solved = iter_questions()
                .filter(|q| q.get_().has_best_solution())
                .count();
            let n_questions_answered = iter_inputted_questions() // 「回答」了用户输入的问题
                .filter(|q| q.get_().has_best_solution())
                .count();
            let n_task_links = iter_concepts().flat_map(Concept::iter_task_links).count();
            let n_term_links = iter_concepts().flat_map(Concept::iter_term_links).count();
            let task_parent_sizes = self.collect_tasks_map(|task| task.parents().count());

            // 用一次性宏组织信息
            macro_once! {
                // * 🚩组织格式：`【名称】 => 【值】`
                macro ( $( $name:literal => $value:expr)* ) => {
                    // const NAME_LENS: &[usize] = &[$($name.len()),*];
                    // let max_name_len = NAME_LENS.iter().cloned().max().unwrap_or(0);
                    // ? 💭【2024-08-10 13:59:23】似乎没必要因为「字段名对齐」牺牲concat的性能
                    format!(
                        concat!(
                            "# Statistics",
                            // * 📌所有名称，格式：`- $name: $value`
                            $("\n- ", $name, ":\t{}"),*
                        ),
                        $($value),*
                    )
                }
                // * 🚩当前状态
                "current time" => self.time()
                "current stamp serial" => self.stamp_current_serial
                "current volume" => self.volume
                "current count of new tasks" => self.derivation_datas.new_tasks.len()
                "current count of novel tasks" => self.derivation_datas.novel_tasks.size()
                "current count of in-channels" => self.io_channels.input_channels.len()
                "current count of out-channels" => self.io_channels.output_channels.len()

                // * 🚩总数有关的信息
                "total concepts" => n_concepts
                "total tasks" => n_tasks
                "total beliefs" => n_beliefs
                "total questions" => n_questions
                "total questions inputted" => n_inputted_questions
                "total task-links" => n_task_links
                "total term-links" => n_term_links
                "total questions solved" => n_questions_solved
                "total questions answered" => n_questions_answered

                // * 🚩均值/比值 有关的信息
                // ! ❌【2024-08-10 15:04:17】不要在数目不定的迭代器上用`ShortFloat::arithmetical_average`，会有NAN问题
                "average concept priority" => self.memory.iter_concepts().map(|c| c.priority().to_float()).average_float()
                "average concept quality" => self.memory.iter_concepts().map(|c| c.quality().to_float()).average_float()
                "average concept complexity" => iter_concept_complexity().average_usize()
                "average task complexity" => iter_tasks_complexity().average_usize()
                "average belief complexity" => iter_beliefs_complexity().average_usize()
                "average question complexity" => iter_questions_complexity().average_usize()
                "average confidence by belief" => iter_beliefs().map(|b| b.confidence().to_float()).average_float()
                // ⚠️下边是「次生信息」
                "average tasks by concept" => n_tasks as Float / n_concepts as Float
                "average beliefs by concept" => n_beliefs as Float / n_concepts as Float
                "average questions by concept" => n_questions as Float / n_concepts as Float
                "average task-links by concept" => n_task_links as Float / n_concepts as Float
                "average term-links by concept" => n_term_links as Float / n_concepts as Float
                "average parent counts by task" => task_parent_sizes.iter().sum::<usize>() as Float / n_tasks as Float
                "percentage of problems solved" => n_questions_solved as Float / n_questions as Float
                "percentage of problems answered" => n_questions_answered as Float / n_inputted_questions as Float

                // * 🚩极值有关的信息
                "maximum task parent count" => task_parent_sizes.iter().max().unwrap_or(&0)
                "minimum task parent count" => task_parent_sizes.iter().min().unwrap_or(&0)
                "maximum concept complexity" => iter_concept_complexity().max().unwrap_or(0)
                "minimum concept complexity" => iter_concept_complexity().min().unwrap_or(0)
                "maximum task complexity" => iter_tasks_complexity().max().unwrap_or(0)
                "minimum task complexity" => iter_tasks_complexity().min().unwrap_or(0)
                "maximum belief complexity" => iter_beliefs_complexity().max().unwrap_or(0)
                "minimum belief complexity" => iter_beliefs_complexity().min().unwrap_or(0)
                "maximum question complexity" => iter_questions_complexity().max().unwrap_or(0)
                "minimum question complexity" => iter_questions_complexity().min().unwrap_or(0)
            }
        }
    }

    /// 组织一个[任务](Task)的格式
    fn format_task(task: &Task) -> String {
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

    /// 组织一个[信念](Judgement)的格式
    fn format_belief(belief: &impl Judgement) -> String {
        format!("Belief#{} {}", belief.creation_time(), belief.to_display())
    }

    /// 简略组织一个[任务](Task)的格式
    /// * 🎯需求：所有信息均在一行之内
    fn format_belief_detailed(belief: &impl Judgement) -> String {
        format!(
            "Belief#{} {}",
            belief.creation_time(), // ! 这个不保证不重复
            belief.to_display_long()
        )
    }

    /// 根据「任务是否为『问题』」决定「是否要格式化并展示」
    /// * 📌核心思路：转换成一个可选的String，并在后边用[`Iterator::flatten`]解包
    ///   * ⚠️因为要兼容返回「可选字符串」的「任务派生链」，将其泛型化
    /// * 🚩具体步骤：返回一个包装后的新函数，这个函数「在『任务』为『问题』时调用原格式化函数，否则返回空值」
    /// * ️🚩【2024-08-10 13:00:13】为了节省函数，目前做成一个高阶函数
    ///   * ℹ️返回一个闭包，可以通过`fmt_question(fn_format_task)`获得新闭包
    fn fmt_question<T>(format: impl Fn(&Task) -> T) -> impl Fn(&Task) -> Option<T> {
        move |maybe_question: &Task| match maybe_question.is_question() {
            true => Some(format(maybe_question)),
            false => None,
        }
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
                    " + {}",
                    format_belief(belief)
                )) if let Some(ref belief) = parent_belief
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

/// 专用于指令[`Cmd::SAV`]的处理函数
mod cmd_sav {
    use super::*;
    use nar_dev_utils::macro_once;

    impl Reasoner {
        /// 将记忆区转换为JSON字符串
        /// * ⚠️可能失败：记忆区数据可能无法被序列化
        pub fn memory_to_json(&self) -> Result<String, serde_json::Error> {
            serde_json::to_string(&self.memory)
        }

        /// 将「推理状态」转换为JSON字符串
        /// * ⚠️可能失败：记忆区数据可能无法被序列化
        pub fn status_to_json(&self) -> anyhow::Result<String> {
            let mut writer = Vec::<u8>::new();
            let mut ser = serde_json::Serializer::new(&mut writer);
            self.serialize_status(&mut ser)?;
            let json = String::from_utf8(writer)?;
            Ok(json)
        }
    }

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
            "memory" => reasoner.memory_to_json()
                .map_err(|e| format!("Failed to serialize memory: {e}"))?
            // 推理器整体状态
            "status" => reasoner.status_to_json()
                .map_err(|e| format!("Failed to serialize status: {e}"))?
        }
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
            control::{test_util_ser_de::status_consistent, DEFAULT_PARAMETERS},
            inference::{
                match_task_and_belief, process_direct, reason, transform_task, InferenceEngine,
            },
            ok,
            storage::tests_memory::memory_consistent,
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

        impl Reasoner {
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
                    while let Some(out) = (self.take_output())
                ]
            }

            /// 测试用：打印所有输出
            fn print_outputs(&mut self) {
                self.fetch_outputs()
                    .iter()
                    .for_each(|o| println!("[{}] {}", o.type_name(), o.get_content()))
            }
        }

        fn reasoner_after_inputs(inputs: impl AsRef<str>) -> Reasoner {
            let mut reasoner = default_reasoner();
            reasoner.input_cmds(inputs);
            reasoner
        }

        fn default_reasoner() -> Reasoner {
            Reasoner::new("test", DEFAULT_PARAMETERS, ENGINE_DEV)
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
            reasoner: &mut Reasoner,
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
            reasoner: &mut Reasoner,
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
        fn load_memory_by_cmd(reasoner: &mut Reasoner, data: impl Into<String>) {
            load_xxx_by_cmd(
                reasoner,
                "memory",
                data,
                "记忆区",
                MESSAGE_MEMORY_LOAD_SUCCESS,
            )
        }

        /// 将JSON数据以NAVM指令形式输入推理器，让推理器加载状态
        /// * 🚩同时检验「是否有加载成功」
        fn load_status_by_cmd(reasoner: &mut Reasoner, data: impl Into<String>) {
            load_xxx_by_cmd(
                reasoner,
                "status",
                data,
                "推理器状态",
                MESSAGE_STATUS_LOAD_SUCCESS,
            )
        }

        #[test]
        fn load_memory_from_json() -> AResult {
            // 一定推理后的推理器
            let mut reasoner = reasoner_after_inputs(SAMPLE_INPUTS);
            // 记忆区序列化成JSON
            let data = save_xxx_by_cmd(&mut reasoner, "memory", "");
            // 从JSON加载记忆区
            let old_memory = reasoner.load_memory_from_json(&data)?;
            // 旧的记忆区应该与新的一致
            memory_consistent(&old_memory, &reasoner.memory)?;

            // 将JSON以指令形式封装，让推理器从指令中加载记忆区
            load_memory_by_cmd(&mut reasoner, data.clone());

            // 旧的记忆区应该与新的一致
            memory_consistent(&old_memory, &reasoner.memory)?;

            // ✅成功，输出附加信息 | ❌【2024-08-12 13:21:22】下面俩太卡了
            println!("Memory reloading success!");
            println!("data = {data}");
            // println!("old = {old_memory:?}");
            // println!("new = {:?}", reasoner.memory);

            ok!()
        }

        /// 将记忆区加载到其它空推理器中，实现「分支」效果
        #[test]
        fn load_memory_to_other_reasoners() -> AResult {
            // 一定推理后的推理器
            let mut reasoner = reasoner_after_inputs(SAMPLE_INPUTS);
            // 记忆区序列化成JSON
            let data = save_xxx_by_cmd(&mut reasoner, "memory", "");
            // 从JSON加载记忆区
            let old_memory = reasoner.load_memory_from_json(&data)?;
            // 旧的记忆区应该与新的一致
            memory_consistent(&old_memory, &reasoner.memory)?;

            // * 🚩以纯数据形式加载到新的「空白推理器」中 * //
            // 创建新的空白推理器
            let mut reasoner2 = default_reasoner();
            // 从JSON加载记忆区
            let old_memory2 = reasoner2.load_memory_from_json(&data)?;
            let consistent_on_clone = |reasoner2: &Reasoner| -> AResult {
                // 但新的记忆区应该与先前旧的记忆区一致
                memory_consistent(&old_memory, &reasoner2.memory)?;
                // 同时，俩推理器现在记忆区一致
                memory_consistent(&reasoner.memory, &reasoner2.memory)?;
                ok!()
            };
            // 空白的记忆区应该与新的不一致
            memory_consistent(&old_memory2, &reasoner2.memory).expect_err("意外的记忆区一致");
            // 被重复加载的记忆区应该一致
            consistent_on_clone(&reasoner2)?;

            // * 🚩以NAVM指令形式加载到新的「空白推理器」中 * //
            // 创建新的空白推理器
            let mut reasoner3 = default_reasoner();
            // 从JSON加载记忆区
            load_memory_by_cmd(&mut reasoner3, data.clone());
            // 被重复加载的记忆区应该一致
            consistent_on_clone(&reasoner3)?;

            // * 🚩分道扬镳的推理歧路 * //
            // 推理器2
            reasoner2.input_cmds(
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
            reasoner.print_outputs();
            println!("reasoner 2:");
            reasoner2.print_outputs();
            println!("reasoner 3:");
            reasoner3.print_outputs();
            // 现在推理器（的记忆区）应该两两不一致
            memory_consistent(&reasoner.memory, &reasoner2.memory).expect_err("意外的记忆区一致");
            memory_consistent(&reasoner.memory, &reasoner3.memory).expect_err("意外的记忆区一致");
            memory_consistent(&reasoner2.memory, &reasoner3.memory).expect_err("意外的记忆区一致");
            ok!()
        }

        /// 加载状态
        /// ! 💫【2024-08-12 22:23:23】因为「推理器内部类型不暴露在外」，所以「单推理器加载状态后，用旧的状态与新的状态对比」难以安排
        /// * 🚩【2024-08-12 22:23:26】目前采用「创建多个推理器，保留一个作为『旧状态』」的方式
        ///   * 📝核心想法：既然「一致性」比对的是推理器，那多创建两个一样的不就好了……
        #[test]
        fn load_status_from_json() -> AResult {
            // 一定推理后的推理器 样本
            let reasoner_old = reasoner_after_inputs(SAMPLE_INPUTS);
            let mut reasoner = reasoner_after_inputs(SAMPLE_INPUTS);
            // 状态序列化成JSON
            let data = save_xxx_by_cmd(&mut reasoner, "status", "");
            // 从JSON加载状态
            reasoner.load_status_from_json(&data)?;
            // 旧的状态应该与新的一致
            status_consistent(&reasoner_old, &reasoner)?;

            // 将JSON以指令形式封装，让推理器从指令中加载状态
            load_status_by_cmd(&mut reasoner, data.clone());

            // 旧的状态应该与新的一致
            status_consistent(&reasoner_old, &reasoner)?;

            // ✅成功，输出附加信息 | ❌【2024-08-12 13:21:22】下面俩太卡了
            println!("Status reloading success!");
            println!("data = {data}");

            ok!()
        }

        /// 将状态加载到其它空推理器中，实现「分支」效果
        #[test]
        fn load_status_to_other_reasoners() -> AResult {
            // 一定推理后的推理器
            let old_reasoner = reasoner_after_inputs(SAMPLE_INPUTS);
            let mut reasoner = reasoner_after_inputs(SAMPLE_INPUTS);
            // 状态序列化成JSON
            let data = save_xxx_by_cmd(&mut reasoner, "status", "");
            // 从JSON加载状态
            reasoner.load_status_from_json(&data)?;
            // 旧的状态应该与新的一致
            status_consistent(&old_reasoner, &reasoner)?;

            // * 🚩以纯数据形式加载到新的「空白推理器」中 * //
            // 创建新的空白推理器
            let old_reasoner2 = default_reasoner();
            let mut reasoner2 = default_reasoner();
            // 从JSON加载状态
            reasoner2.load_status_from_json(&data)?;
            let consistent_on_clone = |reasoner2: &Reasoner| -> AResult {
                // 但新的状态应该与先前旧的状态一致
                status_consistent(&old_reasoner, reasoner2)?;
                // 同时，俩推理器现在状态一致
                status_consistent(&reasoner, reasoner2)?;
                ok!()
            };
            // 空白的状态应该与新的不一致
            status_consistent(&old_reasoner2, &reasoner2).expect_err("意外的状态一致");
            // 被重复加载的状态应该一致
            consistent_on_clone(&reasoner2)?;

            // * 🚩以NAVM指令形式加载到新的「空白推理器」中 * //
            // 创建新的空白推理器
            let mut reasoner3 = default_reasoner();
            // 从JSON加载状态
            load_status_by_cmd(&mut reasoner3, data.clone());
            // 被重复加载的状态应该一致
            consistent_on_clone(&reasoner3)?;

            // * 🚩分道扬镳的推理歧路 * //
            // 推理器2
            reasoner2.input_cmds(
                "
                nse (&&, <A --> C>, <A --> B>).
                cyc 10
                inf concepts
                inf tasks
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
                inf tasks
                inf summary
                ",
            );
            println!("reasoner:");
            reasoner.print_outputs();
            println!("reasoner 2:");
            reasoner2.print_outputs();
            println!("reasoner 3:");
            reasoner3.print_outputs();
            // 现在推理器（的状态）应该两两不一致
            status_consistent(&reasoner, &reasoner2).expect_err("意外的状态一致");
            status_consistent(&reasoner, &reasoner3).expect_err("意外的状态一致");
            status_consistent(&reasoner2, &reasoner3).expect_err("意外的状态一致");
            ok!()
        }
    }
}
use cmd_loa::*;
