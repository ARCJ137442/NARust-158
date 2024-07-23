use std::fmt::Display;

use crate::{
    entity::{BudgetValue, Judgement, Punctuation, Sentence, SentenceV1, Stamp, Task},
    global::Float,
    inference::{Truth, TruthFunctions},
    language::Term,
};

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

    pub fn pop(&mut self) -> Option<(T, Float)> {
        self.vec.pop()
    }

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

    /// ! 此方法和PyNARS不一样
    pub fn show(&self) -> String
    where
        T: std::fmt::Debug,
    {
        let mut result = String::new();
        for (i, (item, priority)) in self.vec.iter().enumerate() {
            result += &format!("{}: {:?} ({})", i, item, priority);
            result += "\n";
        }
        result
    }
}

/// 📝时间窗口
#[derive(Debug, Clone)]
struct Slot {
    events: PriorityQueue<Task>,
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

    pub fn push(&mut self, item: Task, priority: Float) {
        self.events.push(item, priority);
    }

    pub fn pop(&mut self) -> Option<(Task, Float)> {
        self.events.pop()
    }

    pub fn random_pop(&mut self) -> Option<(Task, Float)> {
        self.events.random_pop()
    }
}

pub struct EventBuffer {
    // TODO: 【2024-07-24 01:15:01】待复刻 @ https://github.com/bowen-xu/PyNARS/blob/72091454adc676fae7d40aad418eb9e8e728c51a/pynars/NARS/DataStructures/MC/EventBuffer.py#L70
}
