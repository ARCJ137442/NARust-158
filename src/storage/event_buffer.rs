//! 模仿复刻自PyNARS/`EventBuffer.py`
//! * 🔗参考自: <https://github.com/bowen-xu/PyNARS/blob/72091454adc676fae7d40aad418eb9e8e728c51a/pynars/NARS/DataStructures/MC/EventBuffer.py>
//!   * `Utils.py`: <https://github.com/bowen-xu/PyNARS/blob/72091454adc676fae7d40aad418eb9e8e728c51a/pynars/NARS/DataStructures/MC/Utils.py>
//!   * `EventBuffer.py`: <https://github.com/bowen-xu/PyNARS/blob/72091454adc676fae7d40aad418eb9e8e728c51a/pynars/NARS/DataStructures/MC/EventBuffer.py>
//! * ℹ️原作者: **Tory Li**
//!
//! ! ⚠️【2024-07-25 15:15:11】目前不包含Python源码中任何有关"cheating"的内容

use crate::{
    control::DEFAULT_PARAMETERS,
    entity::{BudgetValue, JudgementV1, Sentence, ShortFloat, Stamp, Task, TruthValue},
    global::Float,
    inference::{BudgetFunctions, Evidential, Truth, TruthFunctions},
    io::symbols::{IMPLICATION_RELATION, PREDICTIVE_IMPLICATION_RELATION},
    language::Term,
    storage::Memory,
};
use std::collections::VecDeque;

mod utils {
    use crate::{
        control::DEFAULT_PARAMETERS,
        entity::{BudgetValue, Judgement, JudgementV1, Sentence, Stamp, Task, TruthValue},
        global::Float,
        inference::{Budget, BudgetInference, Evidential, Truth, TruthFunctions},
        language::Term,
        storage::Memory,
    };
    use std::ops::{Index, IndexMut};

    pub type PqItem<T> = (T, Float);
    /// It is not a heap, it is a sorted array by insertion sort.
    /// Since we need to
    /// 1) access the largest item,
    /// 2) access the smallest item,
    /// 3) access an item in the middle.
    #[derive(Debug, Clone)]
    pub struct PriorityQueue<T> {
        vec: Vec<PqItem<T>>,
        size: usize,
    }

    impl<T> PriorityQueue<T> {
        pub fn new(size: usize) -> Self {
            Self {
                vec: Vec::with_capacity(size),
                size,
            }
        }

        pub fn len(&self) -> usize {
            self.vec.len()
        }

        pub fn is_empty(&self) -> bool {
            self.vec.is_empty()
        }

        /// 🆕像数组一样迭代内部元素
        /// * 🎯封装接口，并用于「预测性蕴含应用」
        pub fn iter_vec(&self) -> impl Iterator<Item = &'_ PqItem<T>> {
            self.vec.iter()
        }
        pub fn iter_mut_vec(&mut self) -> impl Iterator<Item = &'_ mut PqItem<T>> {
            self.vec.iter_mut()
        }

        /// Add a new one, regardless whether there are duplicates.
        pub fn push(&mut self, item: T, priority: Float) {
            let element = (item, priority);
            let mut index = self.vec.len();
            for i in 0..self.vec.len() {
                if priority <= self.vec[i].1 {
                    index = i;
                    break;
                }
            }
            if index == self.vec.len() {
                self.vec.push(element);
            } else {
                self.vec.insert(index, element);
            }
            if self.vec.len() > self.size {
                self.vec.remove(0);
            }
        }

        /// Replacement.
        pub fn edit(&mut self, item: &T, new_priority: Float, identifier: impl Fn(&T, &T) -> bool) {
            let mut found = None;
            for i in 0..self.vec.len() {
                if identifier(&self.vec[i].0, item) {
                    let value = self.vec.remove(i);
                    let _ = found.insert(value);
                    break;
                }
            }
            if let Some((item, _)) = found {
                self.push(item, new_priority);
            }
        }

        /// Pop the highest.
        pub fn pop(&mut self) -> Option<PqItem<T>> {
            self.vec.pop()
        }

        /// 🆕在「预测生成」阶段，要拿出指定索引的元素
        pub fn pop_at(&mut self, index: usize) -> PqItem<T> {
            self.vec.remove(index)
        }

        /// Based on the priority (not budget.priority), randomly pop one buffer task.
        /// The higher the priority, the higher the probability to be popped.
        ///
        /// Design this function is mainly for the prediction generation in the conceptual design.
        ///
        /// It only gives the item, not the value.
        pub fn random_pop(&mut self) -> Option<PqItem<T>> {
            if self.vec.is_empty() {
                return None;
            }
            for i in (0..self.len()).rev() {
                if rand::random::<Float>() < self.vec[i].1 {
                    return Some(self.vec.remove(i));
                }
            }
            None
        }

        // ! 🚩【2024-07-25 16:17:49】暂不使用
        // /// ! 此方法和PyNARS不完全一致
        // /// * 📌对于PyNARS中「间隔」的展示，完全可以放在`identifier`参数中
        // ///
        // /// Show each item in the priority queue.
        // /// Since it may contain items other than BufferTasks,
        // /// you can design you own identifier to show what you want to show.
        // pub fn show<D>(&self, identifier: impl Fn(&T) -> D) -> String
        // where
        //     D: std::fmt::Display,
        // {
        //     let mut vec_to_show = self.vec.iter().collect::<Vec<_>>();
        //     vec_to_show.sort_by(|(_, p_a), (_, p_b)| p_a.total_cmp(p_b));
        //     let mut result = String::new();
        //     for (item, priority) in vec_to_show.iter() {
        //         result += &format!("{} | {}", priority, identifier(item));
        //         result += "\n";
        //     }
        //     result
        // }
    }

    /// 🆕从优先级中拿到元素
    /// * ⚠️不包括其优先级
    impl<T> Index<usize> for PriorityQueue<T> {
        type Output = T;
        fn index(&self, index: usize) -> &Self::Output {
            &self.vec[index].0
        }
    }

    /// 🆕从优先级中拿到元素
    /// * ⚠️不包括其优先级
    impl<T> IndexMut<usize> for PriorityQueue<T> {
        fn index_mut(&mut self, index: usize) -> &mut Self::Output {
            &mut self.vec[index].0
        }
    }

    /// When a task is input to a buffer, its budget might be different from the original.
    /// But this is effected by many factors, and each factor might be further revised.
    /// Therefore, we restore each factor independently; only when everything is decided, the final budget is given.
    #[derive(Debug, Clone)]
    pub struct BufferTask {
        /// the original task
        pub task: Task,
        /// the influence of a channel, currently, all channels have parameter 1 and can't be changed
        pub channel_parameter: usize,
        pub preprocess_effect: Float,
        pub is_component: usize,
        pub expiration_effect: usize,
    }

    impl BufferTask {
        /// 🆕取代原class`BufferTask`的构造函数
        /// * 🎯无需「在构造后赋值」减少借用冲突
        pub fn new(task: Task, preprocess_effect: Float) -> Self {
            Self {
                task,
                channel_parameter: 1,
                preprocess_effect,
                expiration_effect: 1,
                is_component: 0,
            }
        }

        pub fn priority(&self) -> Float {
            self.task.priority().to_float()
                * self.channel_parameter as Float
                * self.preprocess_effect
                * self.expiration_effect as Float
                * ((2 - self.is_component) as Float / 2.0)
        }

        /// 🆕解包出[`Task`]
        /// * 🎯从「缓冲区任务」解包回「任务」
        /// * ⚠️解包时丢失其它信息
        pub fn unwrap_to_task(self) -> Task {
            self.task
        }
    }

    pub fn preprocessing(task: &Task, memory: &Memory) -> Float {
        let term = task.content();
        let concept_in_memory = memory.term_to_concept(term);
        match concept_in_memory {
            Some(concept) => (task.priority() | concept.priority()).to_float(),
            None => 1.0 / ((1 + term.complexity()) as Float),
        }
    }

    pub fn satisfaction_level(truth_1: &impl Truth, truth_2: &impl Truth) -> Float {
        (truth_1.frequency() - truth_2.frequency()).to_float().abs()
    }

    /// 🆕从旧任务构建【预算值完全相同】的新任务，**只有词项和真值不同**
    /// * 🎯在「预测性蕴含结论」中用到
    pub fn same_task_with_different_content_and_truth(
        task: &Task,
        content: Term,
        truth: TruthValue,
    ) -> Task {
        // * 🚩复制语句，仅修改其中的「词项」「真值」并锁定「标点」
        let sentence = JudgementV1::new(
            content,
            truth,
            Stamp::from_evidential(task),
            task.as_judgement().unwrap().revisable(),
        );
        Task::from_input(sentence.into(), BudgetValue::from(task))
    }

    /// 🆕从旧任务构建【内容、预算值完全相同】的新任务，**只有真值不同**
    /// * 🎯在「预测性蕴含结论」与「检查预期」中用到
    pub fn same_task_with_different_truth(task: &Task, truth: TruthValue) -> Task {
        same_task_with_different_content_and_truth(task, task.clone_content(), truth)
    }

    /// 🆕专用于「事件缓冲区」的「修正」规则
    /// * 🎯与记忆区无关，也不支持「反馈到链接」功能
    /// * ⚠️【2024-07-24 16:08:20】目前不支持目标推理
    /// * 🚩【2024-07-25 21:39:42】有可能修正失败
    ///
    /// ```nal
    /// S. %f1;c1%
    /// S. %f2;c2%
    /// |-
    /// S. %F_rev%
    /// ```
    pub fn revision(task: &Task, belief: &Task) -> Option<Task> {
        let truth1 = task.as_judgement().unwrap();
        let truth2 = belief.as_judgement().unwrap();
        /* if Enable.temporal_reasoning:
        # boolean useNewBeliefTerm = intervalProjection(nal, newBelief.getTerm(), oldBelief.getTerm(), beliefConcept.recent_intervals, newTruth);
        raise  */
        let truth = truth1.revision(truth2);

        let budget =
            BudgetValue::revise_direct(truth1, truth2, &truth, &mut BudgetValue::default());

        let term = task.clone_content();

        // stamp: Stamp = deepcopy(task.sentence.stamp) # Stamp(Global.time, task.sentence.stamp.t_occurrence, None, (j1.stamp.evidential_base | j2.stamp.evidential_base))
        // stamp.evidential_base.extend(premise2.evidential_base)
        let creation_time = task.creation_time(); // ? 【2024-07-24 15:11:08】这实际上在PyNARS是没有的
        let stamp = Stamp::from_merge(
            task,
            belief,
            creation_time,
            DEFAULT_PARAMETERS.maximum_stamp_length,
        )?;

        // if task.is_judgement:
        // task = Task(Judgement(term, stamp, truth), budget)
        // elif task.is_goal:
        //     task = Task(Goal(term, stamp, truth), budget)
        // else:
        //     raise "Invalid case."
        let sentence = JudgementV1::new(term, truth, stamp, true);

        let task = Task::from_input(sentence.into(), budget);
        Some(task)
    }
}
use nar_dev_utils::{list, unwrap_or_return};
use utils::*;

/// 📝预测性蕴含（缓冲区专用）
#[derive(Debug, Clone)]
pub struct PredictiveImplication {
    condition: Term,
    /// As explained the conceptual design, "+1, +2" cannot be used in buffers,
    /// thus the interval is only kept in the prediction.
    /// Which might cause a trouble, but you may use +1, +2 as terms if you want.
    /// I will let you know how to do it the referred function.
    interval: usize,
    conclusion: Term,
    /// The expiration of predictions are different from expirations in buffers.
    /// It is a non-negative integer.
    /// It means how many cycles this prediction has not been used.
    to_memory_cooldown: usize,
    expiration: usize,
    task: Task,
}

impl PredictiveImplication {
    pub fn new(condition: Term, interval: usize, conclusion: Term, task: Task) -> Self {
        Self {
            condition,
            interval,
            conclusion,
            expiration: 0,
            to_memory_cooldown: 0,
            task,
        }
    }

    pub fn get_conclusion(&self, condition_task: &Task) -> Option<(usize, Task)> {
        // when "A" is matched with "A =/> B", return B with truth deduction
        let task_judgement = &self.task.as_judgement().unwrap();
        let truth = task_judgement.deduction(condition_task.as_judgement().unwrap());
        if truth.confidence().to_float() < 0.3 {
            return None;
        }
        Some((
            self.interval,
            same_task_with_different_content_and_truth(&self.task, self.conclusion.clone(), truth),
        ))
    }
}

/// 📝预期 @ 缓冲区
#[derive(Debug, Clone)]
pub struct Anticipation {
    matched: bool,
    task: Task,
    prediction: PredictiveImplication,
}

impl Anticipation {
    pub fn new(task: Task, prediction: PredictiveImplication) -> Self {
        Self {
            matched: false,
            task,
            prediction,
        }
    }
}

/// 📝时间窗口
#[derive(Debug, Clone)]
struct Slot {
    events: PriorityQueue<BufferTask>,
    anticipations: Vec<Anticipation>,
    // * 🚩【2024-07-25 16:19:16】取消暂未实装的字段
    // num_anticipations: usize,
    // operations: Vec<()>,
    // num_operations: usize,
}

impl Slot {
    pub fn new(num_events: usize, num_anticipations: usize, _num_operations: usize) -> Self {
        Self {
            events: PriorityQueue::new(num_events),
            anticipations: Vec::with_capacity(num_anticipations),
            // num_anticipations,
            // operations: Vec::with_capacity(num_operations),
            // num_operations,
        }
    }

    pub fn push_event(&mut self, item: BufferTask, priority: Float) {
        self.events.push(item, priority);
    }

    /// 🆕根据被[`push`](Self::push)进的「缓冲区任务」计算其优先级
    /// * 🎯避免借用问题，简化操作
    pub fn push_with_its_priority(&mut self, item: BufferTask) {
        // 计算优先级
        let priority = item.priority();
        // 按所计算的优先级push
        self.push_event(item, priority);
    }

    pub fn pop(&mut self) -> Option<PqItem<BufferTask>> {
        self.events.pop()
    }

    pub fn random_pop(&mut self) -> Option<BufferTask> {
        self.events.random_pop().map(|(task, _)| task)
    }

    pub fn is_empty_events(&self) -> bool {
        self.events.is_empty()
    }

    pub fn iter_events(&self) -> impl Iterator<Item = &'_ PqItem<BufferTask>> {
        self.events.iter_vec()
    }
}

#[derive(Debug, Clone)]
pub struct EventBuffer {
    num_events: usize,
    num_anticipations: usize,
    num_operations: usize,
    slots: VecDeque<Slot>,
    current_slot: usize,
    predictive_implications: PriorityQueue<PredictiveImplication>,
    // reactions
    n: usize,
}

/// 缓冲区循环周期的上下文
pub trait BufferCycleContext {
    /// 获取记忆区
    fn memory(&self) -> &Memory;

    /// 输出任务（到记忆区，或其它地方）
    fn output_task(&mut self, task: Task);

    /// 弹出任务（到别的地方）
    fn popped_task(&mut self, task: Task);

    /// 循环参数
    fn parameters(&self) -> &BufferCycleParameters;
}

impl EventBuffer {
    pub fn new(
        num_slot: usize,
        num_events: usize,
        num_anticipations: usize,
        num_operations: usize,
        num_predictive_implications: usize,
        n: usize, // default: 1
    ) -> Self {
        let n_slots = 1 + 2 * num_slot;
        let slots = (0..n_slots)
            .map(|_| Slot::new(num_events, num_anticipations, num_operations))
            .collect();
        Self {
            num_events,
            num_anticipations,
            num_operations,
            slots,
            current_slot: num_slot,
            predictive_implications: PriorityQueue::new(num_predictive_implications),
            n,
        }
    }

    pub fn with_n_1(
        num_slot: usize,
        num_events: usize,
        num_anticipations: usize,
        num_operations: usize,
        num_predictive_implications: usize,
    ) -> Self {
        Self::new(
            num_slot,
            num_events,
            num_anticipations,
            num_operations,
            num_predictive_implications,
            1,
        )
    }

    /// 🆕获取「当前时间窗」
    fn current_slot(&self) -> &Slot {
        &self.slots[self.current_slot]
    }
    fn current_slot_mut(&mut self) -> &mut Slot {
        &mut self.slots[self.current_slot]
    }

    pub fn push(&mut self, tasks: impl IntoIterator<Item = Task>, memory: &Memory) {
        for task in tasks {
            // 先计算效果参数
            let preprocess_effect = preprocessing(&task, memory);
            let buffer_task = BufferTask::new(task, preprocess_effect);
            // 推送到当前时间窗
            self.current_slot_mut().push_with_its_priority(buffer_task);
        }
    }

    pub fn pop(&mut self, popped_task: impl FnMut(Task)) {
        (0..self.n) // 重复n次尝试
            .filter_map(|_| {
                // 每次尝试弹出「当前时间窗」的一个任务
                self.current_slot_mut()
                    .pop()
                    // 弹出后解包
                    .map(|(b_task, _)| b_task.unwrap_to_task())
            })
            // 所有非空结果放入数组中
            .for_each(popped_task)
    }

    /// 📝生成同时性组合：旧任务生成新任务
    /// * 🚩【2024-07-24 12:21:38】采用[`Option`]：构造可能失败
    ///
    /// according to the conceptual design, currently only 2-compounds are allowed,
    /// though in the future, we may have compounds with many components,
    fn contemporary_composition<'t>(events: impl IntoIterator<Item = &'t Task>) -> Option<Task> {
        let events = events.into_iter().collect::<Vec<_>>();
        if events.len() < 2 {
            return None;
        }

        // term
        let each_compound_term = events
            .iter()
            .cloned()
            .map(Task::clone_content)
            .collect::<Vec<_>>();
        let term = Term::make_parallel_conjunction(each_compound_term)?; // 平行合取

        // truth, using the truth with the lowest expectation
        let truth = events
            .iter()
            .map(|&event| event.as_judgement().unwrap())
            // * 🚩全序排序取最小值（可能为空⇒无任务产生）
            .min_by(|t1, t2| t1.expectation().total_cmp(&t2.expectation()))?;

        // stamp, using stamp-merge function
        let creation_time = events[0].creation_time(); // ? 【2024-07-24 15:11:08】这实际上在PyNARS是 // 采用第一个的创建时间没有的
        let stamp = events
            .iter()
            .cloned()
            // 先变成时间戳
            .map(Stamp::from_evidential)
            // 全部合并在一起
            .reduce(|accumulated, evidential| {
                Stamp::from_merge_unchecked(
                    &accumulated,
                    &evidential,
                    creation_time,
                    // 采用默认长度
                    DEFAULT_PARAMETERS.maximum_stamp_length,
                )
            })?;

        // budget, using budget-merge function
        let budget = events
            .iter()
            .cloned()
            .map(BudgetValue::from)
            .reduce(|accumulated, budget| accumulated.merge(&budget))?;

        // sentence
        let sentence = JudgementV1::new(term, truth, stamp, true);

        // task
        let task = Task::from_input(sentence.into(), budget);
        Some(task)
    }

    /// 📝生成序列性组合
    /// * 🚩【2024-07-24 15:16:25】采用[`Option`]：构造可能失败
    fn sequential_composition(event_1: &Task, interval: usize, event_2: &Task) -> Option<Task> {
        // according to the conceptual design, we currently only have "event_1, interval, event_2" schema,
        // though in the future this may also change, but it is too early to decide here
        let interval = Term::make_interval(interval);
        let term = Term::make_sequential_conjunction([
            event_1.clone_content(),
            interval,
            event_2.clone_content(),
        ])?;
        // in some cases, the interval needs not to be displayer in Narsese
        // let term = Term::make_sequential_conjunction(event_1.clone_content(), event_2.clone_content())?
        let truth = event_2.as_judgement().map(TruthValue::from)?;
        // decrease the confidence of a compound based on the length of the interval
        // truth.c *= 1 / int(interval)
        // truth.c *= 0.7 + 0.3 * (0.9 - (0.9 / (self.current_slot * 5)) * interval)

        let creation_time = event_1.creation_time(); // ? 【2024-07-24 15:11:08】这实际上在PyNARS是没有的
        let stamp = Stamp::from_merge_unchecked(
            event_1,
            event_2,
            creation_time,
            DEFAULT_PARAMETERS.maximum_stamp_length,
        );

        let budget = event_1.merge(event_2);

        // sentence
        let sentence = JudgementV1::new(term, truth, stamp, true);

        // task
        let task = Task::from_input(sentence.into(), budget);
        Some(task)
    }

    /// * 🚩【2024-07-24 15:16:25】采用[`Option`]：构造可能失败
    fn generate_prediction_util(
        event_1: &Task,
        interval: usize,
        event_2: &Task,
    ) -> Option<PredictiveImplication> {
        let copula = match interval {
            // currently, this is only allowed in the global buffer,
            // but only for events from different resources
            0 => IMPLICATION_RELATION,
            _ => PREDICTIVE_IMPLICATION_RELATION,
        };
        // If you want to include "interval" as a term, you just need to change "term" on the next line.
        let term = Term::make_statement_relation(
            copula,
            event_1.clone_content(),
            event_2.clone_content(),
        )?;

        // truth, a default truth, with only one positive example
        // ? 是否要视作是「沿用配置中的『默认真值』」
        let truth = TruthValue::from_floats(
            DEFAULT_PARAMETERS.default_judgement_frequency,
            DEFAULT_PARAMETERS.default_judgement_confidence,
            false,
        );

        // stamp, using event_2's stamp
        let stamp = Stamp::from_evidential(event_2);

        // budget, using budget-merge function
        let budget = event_1.merge(event_2);

        // sentence
        let sentence = JudgementV1::new(term, truth, stamp, true);

        // task
        let task = Task::from_input(sentence.into(), budget);
        let predictive_implication = PredictiveImplication::new(
            event_1.clone_content(),
            interval,
            event_2.clone_content(),
            task,
        );
        Some(predictive_implication)
    }

    /// 📝复合词项构建
    ///
    /// After the initial composition, pick the one with the highest priority in the current slot.
    /// Compose it with all other events in the current slot and the previous max events.
    fn compound_composition(&mut self, memory: &Memory) {
        if self.current_slot().is_empty_events() {
            return;
        }

        let (mut current_max, _) = self.current_slot_mut().pop().unwrap();
        let mut current_remaining = vec![];
        let mut current_composition = vec![];
        while let Some((mut remaining, _)) = self.current_slot_mut().pop() {
            remaining.is_component = 1;
            current_max.is_component = 1;
            current_remaining.push(remaining);
            if let Some(composition) = Self::contemporary_composition([
                &current_max.task,
                &current_remaining[current_remaining.len() - 1].task,
            ]) {
                // * 🚩【2024-07-25 21:35:50】若组合成功，才返回
                current_composition.push(composition);
            }
        }

        let mut previous_max = vec![];
        let mut previous_composition = vec![];
        for i in 0..self.current_slot {
            if let Some((temp, _)) = self.slots[i].pop() {
                previous_max.push(Some(temp));
                // don't change previous max's "is_component"
                current_max.is_component = 1;
                let composition = Self::sequential_composition(
                    &previous_max.last().unwrap().as_ref().unwrap().task,
                    self.current_slot - i,
                    &current_max.task,
                )
                .unwrap();
                previous_composition.push(composition);
            } else {
                previous_max.push(None)
            }
        }

        // after get all compositions, put everything back
        for (i, buffer_task) in previous_max.into_iter().enumerate() {
            if let Some(buffer_task) = buffer_task {
                self.slots[i].push_with_its_priority(buffer_task);
            }
        }
        self.current_slot_mut().push_with_its_priority(current_max);
        for remaining in current_remaining {
            self.current_slot_mut().push_with_its_priority(remaining);
        }

        // add all compositions to the current slot
        self.push(
            current_composition.into_iter().chain(previous_composition),
            memory,
        )
    }

    /// 📝检查预期
    ///
    /// Check all anticipations, award or punish the corresponding predictive implications.
    /// If an anticipation does not even exist, apply the lowest satisfaction.
    fn check_anticipation(&mut self, memory: &Memory) {
        // update the predictive implications
        // * 🚩【2024-07-24 17:41:32】必须要用一个闭包
        let prediction_award_penalty =
            |each: &mut PredictiveImplication,
             frequency,
             predictive_implications: &mut PriorityQueue<PredictiveImplication>| {
                let belief = same_task_with_different_truth(
                    &each.task,
                    TruthValue::from_floats(
                        frequency,
                        DEFAULT_PARAMETERS.default_judgement_confidence,
                        false,
                    ),
                );
                let revised = unwrap_or_return!(?revision(&each.task, &belief));
                each.task = revised;
                let new_priority = each.task.as_judgement().unwrap().expectation()
                    * preprocessing(&each.task, memory);
                predictive_implications.edit(each, new_priority, |x, y| {
                    x.task.content() == y.task.content()
                });
            };

        let mut checked_buffer_tasks = vec![];
        while let Some((mut buffer_task, _)) = self.current_slot_mut().pop() {
            // ! ❌【2024-07-24 17:42:30】此处必须用`self.slots[self.current_slot]`，避免借用整个`self`
            for anticipation in &mut self.slots[self.current_slot].anticipations {
                // it is possible for an event satisfying multiple anticipations,
                // e.g., A, +1 =/> B, A =/> B
                if anticipation.task.content() == buffer_task.task.content() {
                    anticipation.matched = true;
                    let revised_new_task = unwrap_or_return! {
                        ?revision(&anticipation.task, &buffer_task.task)
                        => continue
                    };
                    buffer_task.task = revised_new_task;
                    let satisfaction = 1.0
                        - satisfaction_level(
                            anticipation.task.as_judgement().unwrap(),
                            buffer_task.task.as_judgement().unwrap(),
                        );
                    prediction_award_penalty(
                        &mut anticipation.prediction,
                        satisfaction,
                        &mut self.predictive_implications,
                    );
                }
            }
            checked_buffer_tasks.push(buffer_task);
        }

        // if there are some unmatched anticipations, apply the lowest satisfaction
        for anticipation in &mut self.slots[self.current_slot].anticipations {
            if !anticipation.matched {
                prediction_award_penalty(
                    // 此处不可能与上一个地方调用的冲突
                    &mut anticipation.prediction,
                    0.0,
                    &mut self.predictive_implications,
                );
            }
        }

        // print("prediction_award_penalty", prediction_award_penalty) // ? 💭【2024-07-24 16:33:30】这个貌似只是调试性信息

        // put all buffer tasks back, some evaluations may change
        for each in checked_buffer_tasks {
            self.current_slot_mut().push_with_its_priority(each);
        }
    }

    /// 📝预测性蕴含应用
    ///
    /// Check all predictive implications, whether some of them can fire.
    /// If so, calculate the corresponding task of the conclusion and create it as an anticipation in the corresponding
    /// slot in the future.
    /// If some implications cannot fire, increase the expiration of them.
    fn predictive_implication_application(&mut self, memory: &Memory) {
        let mut implications = vec![];
        while let Some((implication, _)) = self.predictive_implications.pop() {
            /// 临时用结构体，表示「预测应用」的结果
            enum Application {
                Applied(usize, Task, PredictiveImplication),
                NotApplied(PredictiveImplication),
            }
            use Application::*;
            let applied = 'apply: {
                for (event, _) in self.current_slot().iter_events() {
                    if &implication.condition != event.task.content() {
                        continue;
                    }
                    // 🚩第一个相等⇒得出结论
                    match implication.get_conclusion(&event.task) {
                        Some((interval, conclusion)) => {
                            break 'apply Applied(interval, conclusion, implication);
                        }
                        /* if interval is None:
                        break */
                        None => {
                            break 'apply NotApplied(implication);
                        }
                    };
                }
                NotApplied(implication)
            };
            match applied {
                Applied(interval, conclusion, mut implication) => {
                    implication.expiration = implication.expiration.saturating_sub(1); // 减到0为止
                    implications.push(implication.clone()); // ? 【2024-07-25 15:11:02】因借用问题，这里需要复制；但问题是 是否一定要复制？复制后是否会效果不同
                    let anticipation = Anticipation::new(conclusion, implication);
                    self.slots[self.current_slot + interval]
                        .anticipations
                        .push(anticipation);
                }
                // if not applied
                NotApplied(mut implication) => {
                    implication.expiration += 1;
                    implications.push(implication)
                }
            }
        }
        for each in implications {
            let new_priority = each.task.unwrap_judgement().expectation()
                * preprocessing(&each.task, memory)
                * (1.0 / (1 + each.expiration) as Float);
            self.predictive_implications.push(each, new_priority);
        }
    }

    /// 📝预测蕴含输出
    /// * 🚩【2024-07-25 15:13:52】泛化整个函数的作用：不一定要放进「记忆区」中
    ///
    /// when a predictive implication reaches a relatively high truth value, it will be forwarded to the memory
    ///   (not the next level)
    /// this does not mean it is removed from the predictive implication pq
    #[doc(alias = "to_memory_predictive_implication")]
    fn output_predictive_implication(
        &mut self,
        mut output_task: impl FnMut(Task),
        threshold_f: &ShortFloat,
        threshold_c: &ShortFloat,
        default_cooldown: usize,
    ) {
        for (each, _) in self.predictive_implications.iter_mut_vec() {
            let [task_f, task_c] = &each.task.unwrap_judgement().fc();
            if task_f >= threshold_f && task_c >= threshold_c {
                match each.to_memory_cooldown {
                    0 => {
                        output_task(each.task.clone());
                        each.to_memory_cooldown = default_cooldown;
                    }
                    _ => each.to_memory_cooldown -= 1,
                }
            }
        }
    }

    /// 📝本地执行
    /// 1. 检查预期
    /// 2. 基于预期生成预测性蕴含
    /// 3. 输出预测性蕴含
    ///
    /// ? ❓【2024-09-04 23:46:13】是否应该将此处的「阈值」「冷却时长」放在缓冲区的参数之中
    /// * TODO: 🏗️后续尝试放在缓冲区自身参数之中，而非总是让「周期上下文」提供
    fn local_evaluation(&mut self, context: &mut impl BufferCycleContext) {
        let threshold_f = context.parameters().threshold_f;
        let threshold_c = context.parameters().threshold_c;
        let default_cooldown = context.parameters().default_cooldown;
        self.check_anticipation(context.memory());
        self.predictive_implication_application(context.memory());
        self.output_predictive_implication(
            |task| context.output_task(task),
            &threshold_f,
            &threshold_c,
            default_cooldown,
        );
    }

    /// 📝基于记忆区的执行
    fn memory_based_evaluation(&mut self, memory: &Memory) {
        let mut evaluated_buffer_tasks = vec![];
        while let Some((mut buffer_task, _)) = self.current_slot_mut().pop() {
            buffer_task.preprocess_effect = preprocessing(&buffer_task.task, memory);
            evaluated_buffer_tasks.push(buffer_task);
        }
        for each in evaluated_buffer_tasks {
            self.current_slot_mut().push_with_its_priority(each);
        }
    }

    /// 📝预测修正
    /// * 🚩【2024-07-25 21:42:23】产生的预期可能因为「证据重复」而落空
    fn prediction_revision(
        mut existed_prediction: PredictiveImplication,
        new_prediction: &PredictiveImplication,
    ) -> Option<PredictiveImplication> {
        existed_prediction.task = revision(&existed_prediction.task, &new_prediction.task)?;
        existed_prediction.expiration = existed_prediction.expiration.saturating_sub(1); // = max(0, existed_prediction.expiration - 1);
        Some(existed_prediction)
    }

    /// 📝预测生成
    ///
    /// For each slot, randomly pop "max events per slot" buffer tasks to generate predictions.
    /// Currently, concurrent predictive implications (==>) are not supported.
    fn prediction_generation(&mut self, max_events_per_slot: usize, memory: &Memory) {
        // get all events needed for prediction generation
        let mut selected_buffer_tasks = list![
            (list![
                task
                for _ in (0..max_events_per_slot)
                if let Some(task) = (self.slots[i].random_pop())
            ])
            for i in (0..(self.current_slot + 1))
        ];

        // for i, each_selected_buffer_tasks in enumerate(selected_buffer_tasks):
        //     print("selected_buffer_tasks", i,
        //           [each_event.task if each_event is not None else "None" for each_event in each_selected_buffer_tasks])
        // print("===")

        // generate predictions based on intervals (=/>)
        for i in 0..self.current_slot {
            for each_current_event in &selected_buffer_tasks[selected_buffer_tasks.len() - 1] {
                for each_previous_event in &selected_buffer_tasks[i] {
                    let tmp2 = Self::generate_prediction_util(
                        &each_previous_event.task,
                        self.current_slot - i,
                        &each_current_event.task,
                    );
                    let mut tmp2 = match tmp2 {
                        Some(tmp2) => tmp2,
                        None => continue,
                    };
                    let existed = 'gen: {
                        for j in 0..self.predictive_implications.len() {
                            let prediction = &self.predictive_implications[j];
                            if prediction.task.content() == tmp2.task.content() {
                                break 'gen Some(self.predictive_implications.pop_at(j).0);
                            }
                        }
                        None
                    };
                    if let Some(existed) = existed {
                        if let Some(revised) = Self::prediction_revision(existed, &tmp2) {
                            tmp2 = revised;
                        }
                    }
                    let new_priority = tmp2.task.unwrap_judgement().expectation()
                        * preprocessing(&tmp2.task, memory);
                    self.predictive_implications.push(tmp2, new_priority);
                }
            }
        }

        // after the prediction generation, put the randomly selected buffer tasks back
        for (i, layer) in selected_buffer_tasks
            .drain(0..=self.current_slot) // ! 🚩【2024-07-25 16:03:07】此处使用`drain`取数组切片的部分
            .enumerate()
        {
            for each in layer {
                self.slots[i].push_with_its_priority(each);
            }
        }
    }

    /// 📝时间窗口轮替
    fn slots_cycle(&mut self) {
        // 最前边的窗口删除
        self.slots.pop_front();
        // 最后边增加新窗口
        let new_slot = Slot::new(self.num_events, self.num_anticipations, self.num_operations);
        self.slots.push_back(new_slot);
    }
}

/// 事件缓冲区循环
impl EventBuffer {
    /// 📝事件缓冲区循环
    pub fn buffer_cycle(
        &mut self,
        tasks: impl IntoIterator<Item = Task>,
        context: &mut impl BufferCycleContext,
        // memory: &Memory,
        // output_task: impl FnMut(Task),
        // popped_task: impl FnMut(Task),
        // parameters: &BufferCycleParameters, // * ✨缓冲区循环参数
    ) {
        // put all tasks to the current slot
        self.push(tasks, context.memory());

        // 组合复合词项
        self.compound_composition(context.memory());

        // 本地执行（此时输出新任务）
        self.local_evaluation(context);

        // 基于记忆区的执行
        self.memory_based_evaluation(context.memory());

        // 生成新预测
        self.prediction_generation(context.parameters().max_events_per_slot, context.memory());

        // 弹出旧任务
        self.pop(|task| context.popped_task(task));

        // 时间窗口轮替
        self.slots_cycle();
    }
}

/// 用于简化「事件缓冲区循环」
/// * 🎯用作[`EventBuffer::buffer_cycle`]的参数
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct BufferCycleParameters {
    pub max_events_per_slot: usize,
    pub threshold_f: ShortFloat,
    pub threshold_c: ShortFloat,
    pub default_cooldown: usize,
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{control::Reasoner, global::ClockTime, util::ToDisplayAndBrief};
    use nar_dev_utils::impl_once;
    use narsese::conversion::string::impl_lexical::format_instances::FORMAT_ASCII;

    fn parse_task(s: impl AsRef<str>, stamp_time: ClockTime) -> Task {
        let task = FORMAT_ASCII
            .parse(s.as_ref())
            .unwrap()
            .try_into_task_compatible()
            .unwrap();
        Reasoner::parse_task_full(&DEFAULT_PARAMETERS, stamp_time, task, stamp_time).unwrap()
    }
    fn parse_task_inc(s: impl AsRef<str>, clock: &mut ClockTime) -> Task {
        parse_task(s, {
            *clock += 1;
            *clock - 1
        })
    }

    fn parse_tasks<'s, S: AsRef<str> + 's>(
        ss: impl IntoIterator<Item = S> + 's,
        clock: &'s mut ClockTime,
    ) -> impl Iterator<Item = Task> + 's {
        ss.into_iter().map(|s| parse_task_inc(s, clock))
    }

    fn test_setup() -> (BufferCycleParameters, EventBuffer, Memory, ClockTime) {
        let cycle_parameters = BufferCycleParameters {
            max_events_per_slot: 5,
            threshold_f: ShortFloat::from_float(0.8),
            threshold_c: ShortFloat::from_float(0.9),
            default_cooldown: 10,
        };
        // 参考自 <https://github.com/ARCJ137442/PyNARS/blob/7a28ad83550b2d28f860f4362994c0a2d686298b/pynars/NARS/DataStructures/MC/Console.py#L148>
        let buffer = EventBuffer::new(2, 50, 50, 5, 50, 1);
        let memory = Memory::new(DEFAULT_PARAMETERS);
        let clock = 1;
        (cycle_parameters, buffer, memory, clock)
    }

    #[test]
    fn test_push() {
        let (_, mut buffer, memory, mut clock) = test_setup();
        let tasks = "ABCDEFGHIJKLMNOPQRSTUVWXYZ";
        let tasks = parse_tasks(tasks.chars().map(|c| c.to_string() + "."), &mut clock);
        buffer.push(tasks, &memory);
        dbg!(buffer);
    }

    /// 共用的「缓冲区循环」逻辑
    fn buffer_cycle_echo(
        buffer: &mut EventBuffer,
        tasks: impl IntoIterator<Item = Task>,
        memory: &Memory,
        parameters: &BufferCycleParameters,
    ) {
        buffer.buffer_cycle(
            tasks,
            impl_once! {
                struct Context in 'a {
                    memory: &'a Memory = memory,
                    parameters: &'a BufferCycleParameters = parameters,
                } impl BufferCycleContext {
                    fn memory(&self) -> &Memory {
                        self.memory
                    }

                    fn output_task(&mut self, task: Task) {
                        println!("output: {}", task.to_display_long())
                    }

                    fn popped_task(&mut self, task: Task) {
                        println!("popped: {}", task.to_display_long())
                    }

                    fn parameters(&self) -> &BufferCycleParameters {
                        self.parameters
                    }
                }
            },
        )
    }

    #[test]
    fn test_cycle_1_per() {
        let (parameters, mut buffer, memory, mut clock) = test_setup();
        let tasks = "ABCDEFGHIJKLMNOPQRSTUVWXYZ";
        let tasks = parse_tasks(tasks.chars().map(|c| c.to_string() + "."), &mut clock);
        for task in tasks {
            buffer_cycle_echo(
                &mut buffer,
                [task], // ! 📌【2024-07-25 17:38:52】单个任务会直接被pop忽略？
                &memory,
                &parameters,
            );
        }
        dbg!(buffer);
    }

    #[test]
    fn test_cycle_multi_single() {
        let (parameters, mut buffer, memory, mut clock) = test_setup();
        let tasks = "ABCDEFGHIJKLMNOPQRSTUVWXYZ";
        let tasks = parse_tasks(tasks.chars().map(|c| c.to_string() + "."), &mut clock);
        for task in tasks {
            buffer_cycle_echo(&mut buffer, [task.clone(), task], &memory, &parameters);
        }
        // dbg!(buffer);
    }

    #[test]
    fn test_cycle_multi_multi() {
        let (parameters, mut buffer, memory, mut clock) = test_setup();
        let tasks1 = "ABCDEFGHIJKLMNOPQRSTUVWXYZ";
        let tasks2 = "abcdefghijklmnopqrstuvwxyz";
        let tasks1 = tasks1.chars().map(|c| c.to_string() + ".");
        let tasks2 = tasks2.chars().map(|c| c.to_string() + ".");
        let tasks = tasks1.zip(tasks2);
        for (task1, task2) in tasks {
            let task1 = parse_task_inc(task1, &mut clock);
            let task2 = parse_task_inc(task2, &mut clock);
            buffer_cycle_echo(
                &mut buffer,
                [task1.clone(), task2.clone(), task1, task2],
                &memory,
                &parameters,
            );
        }
        // dbg!(buffer);
    }
}
