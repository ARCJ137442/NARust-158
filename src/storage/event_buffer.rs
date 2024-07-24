//! 模仿复刻自PyNARS/`EventBuffer.py`
//! * 🔗参考自: <https://github.com/bowen-xu/PyNARS/blob/72091454adc676fae7d40aad418eb9e8e728c51a/pynars/NARS/DataStructures/MC/EventBuffer.py>
//!   * `Utils.py`: <https://github.com/bowen-xu/PyNARS/blob/72091454adc676fae7d40aad418eb9e8e728c51a/pynars/NARS/DataStructures/MC/Utils.py>
//!   * `EventBuffer.py`: <https://github.com/bowen-xu/PyNARS/blob/72091454adc676fae7d40aad418eb9e8e728c51a/pynars/NARS/DataStructures/MC/EventBuffer.py>
//! * ℹ️原作者: **Tory Li**

use crate::{
    control::DEFAULT_PARAMETERS,
    entity::{
        BudgetValue, Judgement, JudgementV1, Punctuation, Sentence, SentenceV1, Stamp, Task,
        TruthValue,
    },
    global::Float,
    inference::{BudgetFunctions, Evidential, Truth, TruthFunctions},
    io::symbols::{IMPLICATION_RELATION, PREDICTIVE_IMPLICATION_RELATION},
    language::Term,
};
use std::collections::VecDeque;

mod utils {
    use crate::{
        entity::{Sentence, Task},
        global::Float,
        inference::{Budget, Truth},
        storage::Memory,
    };

    /// It is not a heap, it is a sorted array by insertion sort.
    /// Since we need to
    /// 1) access the largest item,
    /// 2) access the smallest item,
    /// 3) access an item in the middle.
    #[derive(Debug, Clone)]
    pub struct PriorityQueue<T> {
        vec: Vec<(T, Float)>,
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
        pub fn edit(&mut self, item: &T, identifier: impl Fn(&T, &T) -> bool) {
            let mut found = None;
            for i in 0..self.vec.len() {
                if identifier(&self.vec[i].0, item) {
                    let value = self.vec.remove(i);
                    let _ = found.insert(value);
                    break;
                }
            }
            if let Some((item, priority)) = found {
                self.push(item, priority);
            }
        }

        /// Pop the highest.
        pub fn pop(&mut self) -> Option<(T, Float)> {
            self.vec.pop()
        }

        /// Based on the priority (not budget.priority), randomly pop one buffer task.
        /// The higher the priority, the higher the probability to be popped.
        ///
        /// Design this function is mainly for the prediction generation in the conceptual design.
        ///
        /// It only gives the item, not the value.
        pub fn random_pop(&mut self) -> Option<(T, Float)> {
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

        /// ! 此方法和PyNARS不完全一致
        /// * 📌对于PyNARS中「间隔」的展示，完全可以放在`identifier`参数中
        ///
        /// Show each item in the priority queue.
        /// Since it may contain items other than BufferTasks,
        /// you can design you own identifier to show what you want to show.
        pub fn show<D>(&self, identifier: impl Fn(&T) -> D) -> String
        where
            D: std::fmt::Display,
        {
            let mut vec_to_show = self.vec.iter().collect::<Vec<_>>();
            vec_to_show.sort_by(|(_, p_a), (_, p_b)| p_a.total_cmp(p_b));
            let mut result = String::new();
            for (item, priority) in vec_to_show.iter() {
                result += &format!("{} | {}", priority, identifier(item));
                result += "\n";
            }
            result
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
        pub fn new(task: Task) -> Self {
            Self::with_preprocess_effect(task, 1.0)
        }

        /// 🆕
        /// * 🎯无需「在构造后赋值」减少借用冲突
        pub fn with_preprocess_effect(task: Task, preprocess_effect: Float) -> Self {
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
}
use utils::*;

use super::Memory;

#[derive(Debug, Clone)]
pub struct Anticipation {
    matched: bool,
    task: Task,
    prediction: PredictiveImplication,
}

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
        // * 🚩复制语句，仅修改其中的「真值」并锁定「标点」
        let sentence = SentenceV1::new_sentence_from_punctuation(
            self.task.clone_content(),
            Punctuation::Judgement,
            Stamp::from_evidential(&self.task),
            Some((truth, task_judgement.revisable())),
        )
        .unwrap();
        Some((
            self.interval,
            Task::from_input(sentence, BudgetValue::from(&self.task)),
        ))
    }
}

/// 📝时间窗口
#[derive(Debug, Clone)]
struct Slot {
    events: PriorityQueue<BufferTask>,
    anticipations: Vec<Anticipation>,
    num_anticipations: usize,
    operations: Vec<()>,
    num_operations: usize,
}

impl Slot {
    pub fn new(num_events: usize, num_anticipations: usize, num_operations: usize) -> Self {
        Self {
            events: PriorityQueue::new(num_events),
            anticipations: Vec::with_capacity(num_anticipations),
            num_anticipations,
            operations: Vec::with_capacity(num_operations),
            num_operations,
        }
    }

    pub fn push(&mut self, item: BufferTask, priority: Float) {
        self.events.push(item, priority);
    }

    pub fn pop(&mut self) -> Option<(BufferTask, Float)> {
        self.events.pop()
    }

    pub fn random_pop(&mut self) -> Option<(BufferTask, Float)> {
        self.events.random_pop()
    }

    pub fn len_events(&self) -> usize {
        self.events.len()
    }

    pub fn is_empty_events(&self) -> bool {
        self.events.is_empty()
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
            .into_iter()
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
            let buffer_task = BufferTask::with_preprocess_effect(task, preprocess_effect);
            // 计算优先级
            let priority = buffer_task.priority();
            // 推送到当前时间窗
            self.current_slot_mut().push(buffer_task, priority);
        }
    }

    pub fn pop(&mut self) -> Vec<Task> {
        (0..self.n) // 重复n次尝试
            .filter_map(|_| {
                // 每次尝试弹出「当前时间窗」的一个任务
                self.current_slot_mut()
                    .pop()
                    // 弹出后解包
                    .map(|(b_task, _)| b_task.unwrap_to_task())
            })
            // 所有非空结果放入数组中
            .collect()
    }

    /// 📝生成同时性组合：旧任务生成新任务
    /// * 🚩【2024-07-24 12:21:38】采用[`Option`]：构造可能失败
    ///
    /// according to the conceptual design, currently only 2-compounds are allowed,
    /// though in the future, we may have compounds with many components,
    fn contemporary_composition<'t>(events: impl IntoIterator<Item = &'t Task>) -> Option<Task> {
        let events = events.into_iter().collect::<Vec<_>>();
        if events.is_empty() {
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
        while !self.current_slot().is_empty_events() {
            let (mut remaining, _) = self.current_slot_mut().pop().unwrap();
            remaining.is_component = 1;
            current_max.is_component = 1;
            current_remaining.push(remaining);
            let composition = Self::contemporary_composition([&current_max.task]).unwrap();
            current_composition.push(composition);
        }

        let mut previous_max = vec![];
        let mut previous_composition = vec![];
        for i in 0..self.current_slot {
            if !self.slots[i].is_empty_events() {
                let (temp, _) = self.slots[i].pop().unwrap();
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
                let priority = buffer_task.priority();
                self.slots[i].push(buffer_task, priority);
            }
        }
        let priority = current_max.priority();
        self.current_slot_mut().push(current_max, priority);
        for remaining in current_remaining {
            let priority = remaining.priority();
            self.current_slot_mut().push(remaining, priority);
        }

        // add all compositions to the current slot
        self.push(
            current_composition.into_iter().chain(previous_composition),
            memory,
        )
    }
}
