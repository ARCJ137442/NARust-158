//! NARust的NAVM接口
//! * 🎯面向crate内外，提供一套封装成NAVM的API
//! * 📌保存可供NAVM使用的接口
//! * ⚠️除了必要的输出，不直接涉及NAVM指令
//! * ⚠️原则上不绑定[`serde_json`]
//!   * ℹ️只需绑定[`serde`]即可

use super::Reasoner;
use crate::{
    entity::{Concept, Judgement, Sentence, TLink, Task},
    global::Float,
    inference::{Budget, Evidential, Truth},
    language::Term,
    util::{AverageFloat, AverageUsize, ToDisplayAndBrief},
};
use nar_dev_utils::{join, macro_once, JoinTo, RefCount};
use narsese::lexical::Task as LexicalTask;

/// 输入输出
/// * 📄（解析并）输入任务
impl Reasoner {
    /// 模拟改版`Reasoner.inputTask`
    /// * 🚩【2024-05-07 22:51:11】在此对[`Budget::budget_above_threshold`](crate::inference::Budget::budget_above_threshold)引入[「预算阈值」超参数](crate::control::Parameters::budget_threshold)
    /// * 🚩【2024-05-17 15:01:06】自「记忆区」迁移而来
    /// * 📌【2024-08-14 17:34:00】重定位功能：输入一个已经被解析好的任务
    ///   * ⚠️在此中不包括对序列号的更新：可能是新的序列号，也可能是旧序列号
    ///
    /// # 📄OpenNARS
    ///
    /// Input task processing. Invoked by the outside or inside environment.
    /// Outside: StringParser (input); Inside: Operator (feedback). Input tasks
    /// with low priority are ignored, and the others are put into task buffer.
    ///
    /// @param task The input task
    pub fn intake_task(&mut self, task: Task) {
        let budget_threshold = self.parameters.budget_threshold;
        if task.budget_above_threshold(budget_threshold) {
            // ? 💭【2024-05-07 22:57:48】实际上只需要输出`IN`即可：日志系统不必照着OpenNARS的来
            // * 🚩此处两个输出合而为一
            self.report_in(&task);
            // * 📝只追加到「新任务」里边，并不进行推理
            self.task_buffer.add_task(task);
        } else {
            // 此时还是输出一个「被忽略」好
            self.report_comment(format!("!!! Neglected: {}", task.to_display_long()));
        }
    }

    /// 输入一个词法任务，解析成内部任务并送入推理器
    /// * ⚠️会将其视作一个全新的任务，赋予【新的】时间戳序列号
    /// * ℹ️若只需将词法Narsese任务转换为内部任务，参考[`Reasoner::parse_task`]
    pub fn input_task(&mut self, task: LexicalTask) {
        // * 🚩视作新任务解析，并使用结果
        match self.parse_new_task(task) {
            // * 🚩解析成功⇒输入任务
            // * 🚩【2024-05-17 16:28:53】现在无需输入任务
            Ok(task) => self.intake_task(task),
            // * 🚩解析失败⇒报告错误
            Err(e) => self.report_error(format!("Narsese任务解析错误：{e}",)),
        }
    }
}

/// 推理周期
/// * 📄推理器做工作周期
impl Reasoner {
    /// 推理循环
    /// * 🚩只负责推理，不处理输入输出
    ///   * 📌在「处理输入」的同时，也可能发生「推理循环」（`CYC`指令）
    pub fn cycle(&mut self, steps: usize) {
        for _ in 0..steps {
            self.handle_work_cycle();
        }
    }
}

/// 信息输出
/// * 📄报告推理器内部信息，用于外部`INF`指令
mod information_report {
    use super::*;

    impl Reasoner {
        /// 报告推理器超参数
        /// * ⚠️【2024-08-14 21:20:45】不甚稳定：参数也可能在后续发生变化
        pub fn report_parameters(&self) -> String {
            format!("Parameters: {:?}", self.parameters)
        }

        /// 报告推理器超参数（详细）
        /// * ⚠️【2024-08-14 21:20:45】不甚稳定：参数也可能在后续发生变化
        pub fn report_parameters_detailed(&self) -> String {
            format!("Parameters: {:#?}", self.parameters)
        }

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
            self.task_buffer.iter_tasks().for_each(deal_ref); // 添加

            // 输出
            outputs
        }

        /// 报告推理器内的所有「任务」
        pub fn report_tasks(&self) -> String {
            format!(
                "Tasks in reasoner:\n{}", // 开始组织格式化
                self.collect_tasks_map(format_task)
                    .into_iter()
                    .join_to_new("\n")
            )
        }

        /// 详尽报告推理器内所有「任务」（的派生关系）
        pub fn report_tasks_detailed(&self) -> String {
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
        pub fn report_beliefs(&self) -> String {
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
        pub fn report_beliefs_detailed(&self) -> String {
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
        pub fn report_questions(&self) -> String {
            format!(
                "Questions in reasoner:\n{}", // 开始组织格式化
                self.collect_tasks_map(fmt_question(format_task))
                    .into_iter()
                    .flatten()
                    .join_to_new("\n")
            )
        }

        /// 详尽报告推理器内所有「问题」（的派生关系）
        pub fn report_questions_detailed(&self) -> String {
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
        pub fn report_concepts(&self) -> String {
            format!(
                "Concepts in memory:\n{}",
                self.memory
                    .iter_concepts()
                    .map(|c| format!("- {}", c.term()))
                    .join_to_new("\n") // 只展示所有词项
            )
        }

        /// 详尽报告推理器内的所有「概念」
        pub fn report_concepts_detailed(&self) -> String {
            format!(
                "# Concepts in memory\n{}",
                self.fmt_concepts(|c| format!("## Concept @ {}", c.to_display_long()))
            )
        }

        /// 报告内部所有链接（仅词项）
        pub fn report_links(&self) -> String {
            format!(
                "Links in memory:\n{}",
                self.memory
                    .iter_concepts()
                    .map(format_concept_links)
                    .join_to_new("\n") // 只展示所有词项
            )
        }

        /// 详尽报告内部所有链接
        pub fn report_links_detailed(&self) -> String {
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
        pub fn report_summary(&self) -> String {
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
                "current stamp serial" => self.stamp_current_serial()
                "current volume" => self.volume()
                "current count of new tasks" => self.task_buffer.n_new_tasks()
                "current count of novel tasks" => self.task_buffer.n_novel_tasks()

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
    /// * ⚠️【2024-08-17 09:30:36】截止至目前，没有一个较好的「序列号表示方式」
    ///   * ❌任务序列号：①仅为「序列反序列化」设计；②仅能用于「任务」不能用于「信念」
    ///   * ❌时间戳序列号from证据基：仅在「从词法中解析出的语句」中唯一，重码率反而比「创建时间」更高
    fn format_task(task: &Task) -> String {
        format!("Task#{} {}", task.creation_time(), task.to_display_long())
    }

    /// 简略组织一个[任务](Task)的格式
    /// * 🎯需求：所有信息均在一行之内
    /// * ⚠️【2024-08-17 09:30:36】截止至目前，没有一个较好的「序列号表示方式」
    ///   * ❌任务序列号：①仅为「序列反序列化」设计；②仅能用于「任务」不能用于「信念」
    ///   * ❌时间戳序列号from证据基：仅在「从词法中解析出的语句」中唯一，重码率反而比「创建时间」更高
    fn format_task_brief(task: &Task) -> String {
        format!(
            "Task#{} \"{}{}\"",
            task.creation_time(), // ! 这个不保证不重复
            task.content(),
            task.punctuation() // * 🚩【2024-08-09 00:28:05】目前从简：不显示真值、预算值（后两者可从`tasks`中查询）
        )
    }

    /// 组织一个[信念](Judgement)的格式
    /// * ⚠️【2024-08-17 09:30:36】截止至目前，没有一个较好的「序列号表示方式」
    ///   * ❌任务序列号：①仅为「序列反序列化」设计；②仅能用于「任务」不能用于「信念」
    ///   * ❌时间戳序列号from证据基：仅在「从词法中解析出的语句」中唯一，重码率反而比「创建时间」更高
    fn format_belief(belief: &impl Judgement) -> String {
        format!("Belief#{} {}", belief.creation_time(), belief.to_display())
    }

    /// 简略组织一个[任务](Task)的格式
    /// * 🎯需求：所有信息均在一行之内
    /// * ⚠️【2024-08-17 09:30:36】截止至目前，没有一个较好的「序列号表示方式」
    ///   * ❌任务序列号：①仅为「序列反序列化」设计；②仅能用于「任务」不能用于「信念」
    ///   * ❌时间戳序列号from证据基：仅在「从词法中解析出的语句」中唯一，重码率反而比「创建时间」更高
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
                => "\n <- ".to_string()
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
