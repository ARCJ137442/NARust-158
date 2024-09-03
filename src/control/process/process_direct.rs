//! 推理器有关「直接推理/立即推理」的功能
//! * 🎯模拟以`Memory.immediateProcess`为入口的「直接推理」
//! * 🎯将其中有关「直接推理」的代码摘录出来
//!   * 📌处理新任务(内部) from 工作周期(@记忆区)
//!   * 📌处理新近任务(内部) from 工作周期(@记忆区)
//!   * 📌立即处理(内部) from 处理新任务/处理新近任务
//!   * 📌直接处理 from 立即处理(@记忆区)
//!   * 📌处理判断(内部 @概念) from 直接处理
//!   * 📌处理问题(内部 @概念) from 直接处理
//!
//! ## 🚩【2024-05-18 14:48:57】有关「复制以防止借用问题」的几个原则
//!
//! * 📌从「词项」到「语句」均为「可复制」的，但只应在「不复制会导致借用问题」时复制
//! * 📌「任务」「概念」一般不应被复制
//! * 📌要被修改的对象**不应**被复制：OpenNARS将修改这些量，以便在后续被使用
//!
//! ## Logs
//! * 🚩【2024-05-17 21:35:04】目前直接基于「推理器」而非「记忆区」
//! * ⚠️【2024-05-18 01:25:09】目前这里所参考的「OpenNARS源码」已基本没有「函数对函数」的意义
//!   * 📌许多代码、逻辑均已重构重组
//! * ♻️【2024-06-26 11:59:58】开始根据改版OpenNARS重写

use crate::{
    control::{ReasonContext, ReasonContextDirect, Reasoner},
    entity::{Item, RCTask, Sentence, Task},
    impl_once,
    inference::Budget,
    storage::{Memory, TaskBufferLoadingContext},
    util::{RefCount, ToDisplayAndBrief},
};
use nar_dev_utils::unwrap_or_return;

/// 为「推理器」添加功能
/// * 📌入口函数
impl Reasoner {
    /// 本地直接推理
    /// * 🚩返回「是否有结果」
    pub(in crate::control) fn process_direct(&mut self) -> bool {
        // * 🚩加载任务 | 新任务/新近任务
        let mut messages = vec![]; // 待输出的消息
        let mut tasks_to_process = vec![];
        // * 🚩构建一次性「上下文」对象，针对性实现「检查是否已有概念」「对外输出消息」功能
        let context = impl_once! {
            /// * 🚩针对此处功能定义一个结构体并初始化
            struct LoadingContext in 'a {
                memory: &'a Memory                  = &self.memory,
                messages: &'a mut Vec<String>       = &mut messages,
                tasks_to_process: &'a mut Vec<Task> = &mut tasks_to_process,
            }
            /// * 🚩实现功能
            impl TaskBufferLoadingContext {
                fn output_task(&mut self, task: Task) {
                    // * 🚩向缓存的数组中添加任务
                    self.tasks_to_process.push(task);
                }

                fn report_comment(&mut self, message: String) {
                    // * 🚩向外部数组中添加消息
                    self.messages.push(message);
                }

                fn has_concept(&self, task: &Task) -> bool {
                    // * 🚩检查是否已有概念
                    self.memory.has_concept(task.content())
                }
            }
        };
        // * 🚩调用功能
        self.task_buffer.load_from_tasks(context);
        // * 🚩报告消息
        for message in messages {
            self.report_comment(message)
        }
        // * 🚩处理任务，收尾返回
        self.immediate_process_tasks(tasks_to_process)
    }

    /// 立即处理（多个任务）
    /// * 🚩返回「是否有结果」
    fn immediate_process_tasks(
        &mut self,
        tasks_to_process: impl IntoIterator<Item = Task>,
    ) -> bool {
        let mut has_result = false;
        for task in tasks_to_process {
            let has_result_single = self.immediate_process(task);
            if has_result_single {
                has_result = true;
            }
        }
        has_result
    }

    /// 立即处理
    /// * 🚩返回「是否有结果」
    fn immediate_process(&mut self, task_to_process: Task) -> bool {
        self.report_comment(format!("!!! Insert: {}", task_to_process.to_display_long()));

        // * 🚩构建「实际上下文」并断言可空性 | 构建失败⇒返回「无结果」
        let mut context =
            unwrap_or_return!(?self.prepare_direct_process_context(task_to_process) => false);

        // * 🚩调整概念的预算值 @「激活」
        // * 📌断言：此处一定是「概念在记忆区之外」
        let new_concept_budget = context
            .memory()
            .activate_concept_calculate(context.current_concept(), &*context.current_task().get_());
        context
            .current_concept_mut()
            .copy_budget_from(&new_concept_budget);

        // * 🔥开始「直接推理」
        context.direct_process();
        let has_result = context.has_result();

        // * 🚩吸收并清空上下文
        context.absorbed_by_reasoner();
        has_result
    }

    /// * ✅【2024-06-28 00:11:12】现在将「推理器可变引用」完全内置到「推理上下文」中，不再发生借用冲突
    fn prepare_direct_process_context<'this: 'context, 'context>(
        &'this mut self,
        task_to_process: Task,
    ) -> Option<ReasonContextDirect<'context>> {
        // * 🚩获取「当前任务」对应的「概念」，复制其键以拿出概念
        let task_term = task_to_process.content();
        let concept_key = self.memory.get_concept_or_create(task_term)?.key().clone();
        let current_concept = self.memory.pick_out_concept(&concept_key)?;
        // * 🚩将「任务」变为共享引用
        let current_task = RCTask::new_(task_to_process);
        // * 🚩构造上下文 | ⚠️在此传入`self: &mut Reasoner`独占引用
        let context = ReasonContextDirect::new(self, current_concept, current_task);
        // * 🚩返回
        Some(context)
    }
}

impl ReasonContextDirect<'_> {
    /// 对于「直接推理上下文」的入口
    /// * 🚩返回「是否有结果」
    fn direct_process(&mut self) {
        // * 🚩原先传入的「任务」就是「推理上下文」的「当前任务」
        // * * 📝在其被唯一使用的地方，传入的`task`只有可能是`context.currentTask`
        // * 🚩所基于的「当前概念」就是「推理上下文」的「当前概念」
        // * * 📝在其被唯一使用的地方，传入的`task`只有可能是`context.currentConcept`
        // * * 📝相比于「概念推理」仅少了「当前词项链」与「当前任务链」，其它基本通用

        // * 🚩委派「推理引擎」分派推理
        // * ✅【2024-06-28 01:25:58】使用了函数指针，所以不存在借用问题
        (self.core.reasoner.inference_engine.direct_f())(self);

        // * 🚩在推理后做链接 | 若预算值够就链接，若预算值不够就丢掉
        self.link_concept_to_task()
    }
}
