//! 存放推理器的「推理数据」
//! * 🎯存储有关「新任务队列」「新近任务袋」的数据
//! * 📄新任务队列
//! * 📄新近任务袋
//! * ⚠️不缓存「NAVM输出」：输出保存在[「推理记录器」](super::report)中

use crate::{
    control::{Parameters, DEFAULT_PARAMETERS},
    entity::{RCTask, Sentence, Task},
    global::Float,
    inference::Truth,
    storage::{Bag, Memory},
    util::{IterInnerRcSelf, ToDisplayAndBrief},
};
use serde::{Deserialize, Serialize};
use std::collections::VecDeque;

/// 🚀推理器的「任务缓冲区」
/// * 📝在整个NARS架构中承担「统一接收并筛选分发任务」的职责
///   * 🚩从各个「输入通道」中接收Narsese任务
///   * 🚩在推理周期中「给出待推理的任务」
///   * 📄「任务缓冲区：所有的新语句（包括经通道输入的和系统推导出的）都作为待处理的任务在缓冲区中汇集并接受简单处理。这些任务竞争系统的注意力，而只有其中的少数任务会被选中进入记忆区。这有些像心理学中所讨论的“工作记忆”或“短期记忆”」
/// * 📌【2024-08-12 20:26:44】内部所存储的「任务」暂时无需考虑「任务共享引用归一化」问题
///   * ⚠️本来要考虑的「任务共享引用」：在每个「任务」内部的「父任务」
///   * 📌【2024-09-03 12:25:36】目前假定「输入进其中的任务不会被其它 任务/概念 链接」
///
/// > [!note]
/// > 在开源纳思158的架构中，采取「新任务队列」与「新近任务袋」的处理方式。
#[derive(Debug, Serialize, Deserialize)]
pub struct TaskBuffer {
    /// 新任务队列
    /// * 🚩没有上限，不适合作为「缓冲区」使用
    ///
    /// > [!note]
    /// > 「新任务队列」是外部纳思语任务的入口。
    /// >
    /// > 「新任务队列」没有固定的容量，在「获取待处理任务」的过程中，「新任务队列」会通过预算值被筛选输出，而未通过标准的进入「新近任务袋」——此机制可被理解为「新加入的任务倾向于被立即处理」
    ///
    /// # 📄OpenNARS
    ///
    /// List of new tasks accumulated in one cycle, to be processed in the next cycle
    new_tasks: VecDeque<Task>,

    /// 新近任务袋
    /// * 📌因「进来的任务不会被其它任务/记忆区所引用」故**不设置为共享引用**
    ///
    /// > [!note]
    /// >
    /// > 暂存入「新近任务袋」的任务，在「获取待处理任务」时被按优先级随机取出一个，可被理解为「具备一定随机兼顾性的注意力过程」。
    /// >
    /// > 「新近任务袋」具有容量，此意味着「若新任务量过多，相对不优先的任务将被抛弃」，可被理解为「短期工作记忆的遗忘机制」
    novel_tasks: Bag<Task>,

    /// 🆕相关的「参数变量」
    #[serde(default)] // 🎯向下兼容旧有序列反序列化机制
    parameters: TaskBufferParameters,
}

/// 🆕有关「任务缓冲区」的参数变量
/// * 🎯拆分「存储结构」与「参数变量」
/// * 📌基本在创建后不改变
/// * 🚩【2024-09-03 13:05:00】
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
struct TaskBufferParameters {
    /// # 📄OpenNARS
    ///
    /// Default expectation for confirmation.
    creation_expectation: Float,
}

impl TaskBufferParameters {
    fn new(parameters: &Parameters) -> Self {
        Self {
            creation_expectation: parameters.default_creation_expectation,
        }
    }
}

/// 以默认参数初始化
/// * 🎯向下兼容旧有序列反序列化架构
impl Default for TaskBufferParameters {
    fn default() -> Self {
        Self::new(&DEFAULT_PARAMETERS)
    }
}

impl Default for TaskBuffer {
    fn default() -> Self {
        Self::new(&DEFAULT_PARAMETERS)
    }
}

impl TaskBuffer {
    /// 从超参数构造一个空的任务缓冲区
    pub fn new(parameters: &Parameters) -> Self {
        Self {
            new_tasks: Default::default(),
            novel_tasks: Bag::from_parameters(
                parameters.novel_task_bag_size,
                parameters.novel_task_forgetting_cycle,
                parameters,
            ),
            parameters: TaskBufferParameters::new(parameters),
        }
    }
    /// 重置推理导出数据
    /// * 🎯原先是「推理器」代码的一部分
    pub fn reset(&mut self) {
        self.new_tasks.clear();
        self.novel_tasks.init();
    }
}

/// 「任务缓冲区」基础功能
/// * ⚠️【2024-06-27 23:12:13】此处不能为推理器添加
///   * ~~📄在[`crate::control::Reasoner::load_from_new_tasks`]中，需要明确借用以避免借用冲突（冲突with记忆区）~~
impl TaskBuffer {
    /// 向「新任务队列」中添加一个任务
    fn add_new_task(&mut self, task: Task) {
        self.new_tasks.push_back(task);
    }

    /// 从「新任务」中拿出（第）一个任务
    #[must_use]
    fn pop_new_task(&mut self) -> Option<Task> {
        self.new_tasks.pop_front()
    }

    /// 将一个任务放进「新近任务袋」
    /// * 🚩同时返回「溢出的新近任务」
    #[must_use]
    fn put_in_novel_tasks(&mut self, task: Task) -> Option<Task> {
        self.novel_tasks.put_in(task)
    }

    /// 从「新近任务袋」拿出一个任务
    #[must_use]
    fn take_a_novel_task(&mut self) -> Option<Task> {
        self.novel_tasks.take_out()
    }
}

/// 「任务缓冲区」对外暴露的「加载上下文」
/// * 🎯简化在「外部交互」时用到的外部功能
///   * 📄外包「是否有相应概念」逻辑到「特征实现者」手中
///   * ✨可进一步分离「方法提供」与「方法调用」二者逻辑
/// * 📝【2024-09-04 00:32:13】适用于「固定方法」的实现者
///   * ⚠️会限制一定的自由性——若需「不同地方调用不同传参」则需定义新结构体
pub trait TaskBufferLoadingContext {
    /// 对外输出一个任务
    /// ! 📌【2024-09-04 00:49:34】目前要求「对外输出任务」功能
    /// * 💭该功能做成「传入某个闭包」也合适
    /// * 🎯调用者决定任务输出的去向
    fn output_task(&mut self, task: Task);

    /// 输出一条注释信息
    /// * 🎯调用者决定消息的去向
    fn report_comment(&mut self, message: String);

    /// 判断任务是否在记忆区中存在
    /// * 🎯由调用者访问记忆区
    fn has_concept(&self, task: &Task) -> bool;
}

/// 对外暴露的接口
impl TaskBuffer {
    /// 向任务缓冲区中添加任务
    /// * 🚩【2024-06-27 20:32:38】不使用[`RCTask`]，并且尽可能限制「共享引用」的使用
    /// * 🚩过程：向「新任务队列」添加任务
    pub fn add_task(&mut self, task: Task) {
        self.add_new_task(task);
    }

    /// 从「新任务」与「新近任务」装载「待处理任务」
    /// * 🚩【2024-06-27 22:58:33】现在合并逻辑，一个个处理
    /// * 📝逻辑上不影响：
    /// * 1. 「直接推理」的过程中不会用到「新任务」与「新近任务」
    /// * 2. 仍然保留了「在『从新任务获取将处理任务』时，将部分任务放入『新近任务袋』」的逻辑
    pub fn load_from_tasks<ReportComment>(
        &mut self,
        memory: &Memory,
        report_comment: ReportComment,
    ) -> Vec<Task>
    where
        ReportComment: FnMut(String),
    {
        // * 🚩加载任务 | 新任务/新近任务
        let mut tasks_to_process = vec![];
        // * 🚩构建一次性「上下文」对象，针对性实现「检查是否已有概念」「对外输出消息」功能
        let context = {
            /// * 🚩针对此处功能定义一个结构体并初始化
            struct LoadingContext<'a, ReportComment>
            where
                ReportComment: FnMut(String),
            {
                memory: &'a Memory,
                report_comment: ReportComment,
                tasks_to_process: &'a mut Vec<Task>,
            }
            /// * 🚩实现功能
            impl<'a, ReportComment> TaskBufferLoadingContext for LoadingContext<'a, ReportComment>
            where
                ReportComment: FnMut(String),
            {
                fn output_task(&mut self, task: Task) {
                    // * 🚩向缓存的数组中添加任务
                    self.tasks_to_process.push(task);
                }
                fn report_comment(&mut self, message: String) {
                    // * 🚩向外部数组中添加消息
                    (self.report_comment)(message);
                }
                fn has_concept(&self, task: &Task) -> bool {
                    // * 🚩检查是否已有概念
                    // ! 📝【2024-09-05 00:55:08】「部分闭包」问题：在「结构体功能的一部分要作为闭包执行」时，此实现就变得脆弱
                    self.memory.has_concept(task.content())
                }
            }
            &mut LoadingContext {
                memory,
                report_comment,
                tasks_to_process: (&mut tasks_to_process),
            }
        };
        // * 🚩调用功能
        self.load_from_tasks_with_context(context);
        // * 🚩返回
        tasks_to_process
    }

    /// 基于完整的「上下文对象」的方法
    /// * 🎯对外封装简洁API，同时不失可定制性
    /// * 🚩基于上下文输出「将要处理的任务」与「将要传出的消息」
    pub fn load_from_tasks_with_context(&mut self, context: &mut impl TaskBufferLoadingContext) {
        // 装载「新任务」
        self.load_from_new_tasks(context);
        // 装载「新近任务」
        self.load_from_novel_tasks(context);
    }

    /// 获取「要处理的新任务」列表
    /// * 🎯分离「缓冲区结构」与「推理器逻辑」
    /// * 🚩【2024-09-03 13:09:24】目前将「是否有概念」
    fn load_from_new_tasks(&mut self, context: &mut impl TaskBufferLoadingContext) {
        // * 🚩处理新输入：立刻处理 or 加入「新近任务」 or 忽略
        // don't include new tasks produced in the current workCycle
        // * 🚩处理「新任务缓冲区」中的所有任务
        // * 📝此处因为与「记忆区」借用冲突，故需特化到字段
        while let Some(task) = self.pop_new_task() {
            // * 🚩是输入 或 已有对应概念 ⇒ 取出
            if task.is_input() || context.has_concept(&task) {
                context.output_task(task);
            }
            // * 🚩否则：继续筛选以放进「新近任务」
            else {
                let should_add_to_novel_tasks = match task.as_judgement() {
                    // * 🚩判断句⇒看期望，期望满足⇒放进「新近任务」
                    Some(judgement) => {
                        judgement.expectation() > self.parameters.creation_expectation
                    }
                    // * 🚩其它⇒忽略
                    None => false,
                };
                match should_add_to_novel_tasks {
                    // * 🚩添加
                    true => {
                        if let Some(overflowed) = self.put_in_novel_tasks(task) {
                            // 🆕🚩报告「任务溢出」
                            context.report_comment(format!(
                                "!!! NovelTasks overflowed: {}",
                                overflowed.to_display_long()
                            ))
                        }
                    }
                    // * 🚩忽略
                    false => {
                        context.report_comment(format!("!!! Neglected: {}", task.to_display_long()))
                    }
                }
            }
        }
    }

    /// 获取「要处理的新任务」列表
    fn load_from_novel_tasks(&mut self, context: &mut impl TaskBufferLoadingContext) {
        // * 🚩从「新近任务袋」中拿出一个任务，若有⇒添加进列表
        if let Some(task) = self.take_a_novel_task() {
            context.output_task(task);
        }
    }
}

/// 用于「呈现内部信息」的功能
impl TaskBuffer {
    /// 获取「新任务」数量
    pub fn n_new_tasks(&self) -> usize {
        self.new_tasks.len()
    }

    /// 获取「新近任务」数量
    pub fn n_novel_tasks(&self) -> usize {
        self.novel_tasks.size()
    }

    /// 获取总任务数
    #[doc(alias = "len")]
    pub fn size(&self) -> usize {
        self.n_new_tasks() + self.n_novel_tasks()
    }

    /// 迭代器：迭代「任务缓冲区」中的所有任务
    /// * 🎯用于「呈现任务信息」
    /// * ⚠️不对外公开
    pub fn iter_tasks(&self) -> impl Iterator<Item = &Task> {
        let new_tasks = self.iter_new_tasks();
        let novel_tasks = self.iter_novel_tasks();
        new_tasks.chain(novel_tasks)
    }

    /// 迭代器：迭代「新任务列表」中的所有任务
    /// * 🎯用于「呈现任务信息」
    /// * ⚠️不对外公开
    fn iter_new_tasks(&self) -> impl Iterator<Item = &Task> {
        self.new_tasks.iter()
    }

    /// 迭代器：迭代「新任务列表」中的所有任务
    /// * 🎯用于「呈现任务信息」
    /// * ⚠️不对外公开
    fn iter_novel_tasks(&self) -> impl Iterator<Item = &Task> {
        self.novel_tasks.iter()
    }
}
/// 用于「序列反序列化」的功能
impl TaskBuffer {
    /// 遍历其中所有「共享任务引用」的可变引用
    /// * 🚩若直接存储
    pub(crate) fn iter_mut_task_rcs(&mut self) -> impl Iterator<Item = &mut RCTask> {
        self.new_tasks
            .iter_mut()
            .chain(self.novel_tasks.iter_mut())
            .flat_map(|t| t.iter_inner_rc_self())
    }

    #[cfg(test)]
    pub(crate) fn iter_task_rcs(&self) -> impl Iterator<Item = &RCTask> {
        self.new_tasks
            .iter()
            .chain(self.novel_tasks.iter())
            .flat_map(Task::parent_task)
    }
}

/// 测试用方法
#[cfg(test)]
pub(crate) mod tests_task_buffer {
    use super::*;
    use crate::{
        assert_eq_try, ok,
        storage::tests_memory::{bag_consistent, task_consistent, zip},
        util::AResult,
    };

    /// 任务缓冲区一致性
    pub fn task_buffer_consistent(a: &TaskBuffer, b: &TaskBuffer) -> AResult {
        // 新任务队列一致性
        task_deque_consistent(&a.new_tasks, &b.new_tasks)?;
        // 任务袋一致性
        task_bag_consistent(&a.novel_tasks, &b.novel_tasks)?;
        // 推导数据一致性
        ok!()
    }

    /// 任务队列一致性
    /// * 🎯新任务队列
    pub fn task_deque_consistent(a: &VecDeque<Task>, b: &VecDeque<Task>) -> AResult {
        assert_eq_try!(a.len(), b.len(), "任务队列不一致——长度不一致");
        for (a, b) in zip(a, b) {
            task_consistent(a, b)?;
        }
        // 任务一致性
        ok!()
    }

    /// 任务袋一致性
    /// * 🎯新近任务袋
    pub fn task_bag_consistent(a: &Bag<Task>, b: &Bag<Task>) -> AResult {
        bag_consistent(a, b, task_consistent)?;
        ok!()
    }
}
