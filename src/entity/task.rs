//! 🎯复刻OpenNARS `nars.entity.Task`
//! * ✅【2024-05-05 21:38:53】基本方法复刻完毕
//! * ♻️【2024-06-21 23:33:24】基于OpenNARS改版再次重写

use super::{BudgetValue, Item, JudgementV1, Sentence, SentenceV1, Token};
use crate::{
    entity::MergeOrder,
    global::{ClockTime, RC},
    inference::{Budget, Evidential},
    util::{RefCount, ToDisplayAndBrief},
};
use nar_dev_utils::join;
use narsese::lexical::{Sentence as LexicalSentence, Task as LexicalTask};
use serde::{Deserialize, Serialize};

/// A task to be processed, consists of a Sentence and a BudgetValue
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Task {
    /// The sentence of the Task
    /// * 📝任务的「内容」
    sentence: SentenceV1,

    /// 🆕Item令牌
    token: Token,

    /// 父任务
    /// * 📌采用「共享引用」结构，以便实现「共享派生树」
    ///
    /// # 📄OpenNARS
    ///
    /// Task from which the Task is derived, or null if input
    parent_task: Option<RCTask>,

    /// 派生所源自的信念
    ///
    /// # 📄OpenNARS
    ///
    /// Belief from which the Task is derived, or null if derived from a theorem
    parent_belief: Option<JudgementV1>,

    /// 最优解
    /// * 📌需要被迭代性改变
    ///
    /// # 📄OpenNARS
    ///
    /// For Question and Goal: best solution found so far
    best_solution: Option<JudgementV1>,
}

/// 拥有「序列号」的共享引用
/// * 🎯【2024-08-11 16:16:44】用于实现序列反序列化，独立成一个特殊的类型
/// * 📌设计上「序列号」用于在「序列反序列化」前后承担「唯一标识」的角色
///   * 📝内容的地址会变，但序列号在序列反序列化中能（相对多个可遍历的引用而言）保持不变
///   * 💡核心想法：通过「序列号」实现「内容归一化」——序列号相同的「序列共享引用」可以实现「统一」操作
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct SerialRef<T> {
    /// 内部引用
    rc: RC<T>,
    /// 所存储的，作为「唯一标识」的「序列号」
    serial: usize,
}

/// 「任务」的共享引用版本
pub type RCTask = SerialRef<Task>;

impl<T: Clone> SerialRef<T> {
    /// 从「内容」对象生成一个【随数据位置唯一】的「序列号」
    /// * 📌这个「序列号」必须对[`clone`](Clone::clone)敏感，即：
    ///   * `clone`之后的序列号必须与原始序列号【不同】
    ///   * 若被移入了类似[`RC`]这样的共享引用结构，不会因为[`RC`]的`clone`而改变
    /// * 🚩【2024-08-11 16:23:11】目前使用自身的指针地址
    ///
    /// ! 📝【2024-08-11 16:47:37】Rust中「移动语义」的含义：**移动后地址改变**
    ///   * 在`let t1 = inner(); let t2 = t1`时，`t1`和`t2`指向不同的内存地址
    fn get_serial(inner: &T) -> usize {
        // 取自身指针地址地址作为序列号
        inner as *const T as usize
    }

    /// 从一个[`RC`]中获取序列号
    fn get_serial_rc(inner: &RC<T>) -> usize {
        Self::get_serial(&*inner.get_())
    }

    /// 使用所传入内容的地址创建一个[`RCTask`]
    /// * 📌这个内容的地址将被[`RCTask`]固定
    pub fn new(inner: T) -> Self {
        let rc = RC::new_(inner);
        let serial = Self::get_serial_rc(&rc);
        Self { rc, serial }
    }

    /// 获取自身存储的序列号（字段）
    fn serial(&self) -> usize {
        self.serial
    }

    /// 获取内部[`Task`]的序列号
    fn inner_serial(&self) -> usize {
        Self::get_serial(&*self.get_())
    }

    /// 同步化
    /// * 🚩将自身的序列号变为内部内容的指针地址
    ///   * 📝后者不会因为引用的拷贝而改变
    fn sync_serial(&mut self) {
        self.serial = self.inner_serial();
    }
}

/// 委托内部rc: RC<Task>字段
impl<T: Clone> RefCount<T> for SerialRef<T> {
    // 直接委托
    type Ref<'r> = <RC<T> as RefCount<T>>::Ref<'r> where T: 'r;
    type RefMut<'r> = <RC<T> as RefCount<T>>::RefMut<'r> where T: 'r;

    fn new_(t: T) -> Self {
        Self::new(t)
    }

    #[inline(always)]
    fn get_<'r, 's: 'r>(&'s self) -> Self::Ref<'r> {
        self.rc.get_()
    }

    #[inline(always)]
    fn mut_<'r, 's: 'r>(&'s mut self) -> Self::RefMut<'r> {
        self.rc.mut_()
    }

    fn n_strong_(&self) -> usize {
        self.rc.n_strong_()
    }

    fn n_weak_(&self) -> usize {
        self.rc.n_weak_()
    }

    fn ref_eq(&self, other: &Self) -> bool {
        // 只比对内部rc
        self.rc.ref_eq(&other.rc)
    }
}

impl<T: Clone> From<T> for SerialRef<T> {
    fn from(value: T) -> Self {
        Self::new(value)
    }
}

/// 构造函数
impl Task {
    /// * 🚩【2024-06-21 23:35:53】对传入的参数「零信任」
    ///   * 💭此处全部传递所有权（除了「父任务」的共享引用），避免意料之外的所有权共享
    pub fn new(
        sentence: SentenceV1,
        budget: BudgetValue,
        parent_task: Option<RCTask>,
        parent_belief: Option<JudgementV1>,
        best_solution: Option<JudgementV1>,
    ) -> Self {
        let token = Token::new(sentence.to_key(), budget);
        Self {
            token,
            sentence,
            parent_task,
            parent_belief,
            best_solution,
        }
    }

    /// 从「输入」中构造
    /// * 🎯在「用户输入任务」中解析
    pub fn from_input(sentence: impl Into<SentenceV1>, budget: impl Into<BudgetValue>) -> Self {
        Self::new(sentence.into(), budget.into(), None, None, None)
    }

    /// 从「导出结论」构造
    /// * 🚩默认没有「最优解」
    pub fn from_derived(
        sentence: SentenceV1,
        budget: impl Into<BudgetValue>,
        parent_task: Option<RCTask>,
        parent_belief: Option<JudgementV1>,
    ) -> Self {
        Self::new(sentence, budget.into(), parent_task, parent_belief, None)
    }
}

// 访问类 方法
impl Task {
    /// 获取其「父任务」
    pub fn parent_task(&self) -> Option<&RCTask> {
        self.parent_task.as_ref()
    }

    /// 获取其「父信念」
    pub fn parent_belief(&self) -> Option<&JudgementV1> {
        self.parent_belief.as_ref()
    }

    /// 获取其「最优解」
    pub fn best_solution(&self) -> Option<&JudgementV1> {
        self.best_solution.as_ref()
    }

    /// 设置其「最优解」
    pub fn set_best_solution(&mut self, new_solution: JudgementV1) -> &mut JudgementV1 {
        // * 🚩调试时断言
        debug_assert!(
            self.sentence.is_question(),
            "只有「疑问句」才可能有「最优解」"
        );
        self.best_solution.insert(new_solution)
    }

    /// 判断「是否来自输入」
    /// * 🚩其「父任务」是否为空
    pub fn is_input(&self) -> bool {
        self.parent_task.is_none()
    }

    /// 🆕判断「是否有父任务」
    /// * 🎯语义相比「是否来自输入」更明确
    ///   * 后者可能会在未来被更改
    pub fn has_parent(&self) -> bool {
        self.parent_task.is_some()
    }

    /// 🆕判断「是否有最优解」
    pub fn has_best_solution(&self) -> bool {
        self.best_solution.is_some()
    }

    /// 🆕获取其由[`Self::parent_task`]得来的一系列「父任务+父信念」
    /// * 📌派生关系是下标从小到大「子→父」
    /// * ✨后续若只用到「父任务」的话，可以用「元组提取」方便地构造新函数
    ///   * 💭【2024-08-09 00:11:15】只希望这时编译器能知道「优化掉父信念的复制」
    /// * 📝派生关系是「有父任务才可能有父信念，有父信念一定有父任务（单前提）」
    pub fn parents(&self) -> impl Iterator<Item = (RCTask, Option<JudgementV1>)> {
        let option_iter = if let Some(parent) = self.parent_task() {
            let mut current = Some((parent.clone(), self.parent_belief().cloned()));
            let iter = std::iter::from_fn(move || {
                // 先拿到完整的结果，将缓存的量置空
                let returns = current.take();
                // 然后准备「下一个要迭代出的对象」：尝试从结果中拿到引用
                // * 🚩若当前结果（亦即缓存的「当前量」）都没引用，则直接返回
                let (current_rc, _) = returns.as_ref()?;
                let current_ref = current_rc.get_();
                if let Some(next) = current_ref.parent_task().cloned() {
                    // 若有下一个引用，获取值、删掉引用并更新之
                    let parent_belief = current_ref.parent_belief().cloned();
                    drop(current_ref);
                    current = Some((next, parent_belief));
                } else {
                    // 没有⇒直接抛掉「当前任务」的引用，下一次就退出迭代
                    drop(current_ref);
                }
                // 返回最开始拿到的「当前量」
                returns
            });
            Some(iter)
        } else {
            None
        };
        option_iter.into_iter().flatten()
    }
}

/// 转换到词法Narsese
impl Task {
    pub fn to_lexical(&self) -> LexicalTask {
        let sentence = self.sentence_to_lexical();
        let budget = self.budget_to_lexical();
        LexicalTask { sentence, budget }
    }
}

impl Budget for Task {
    fn priority(&self) -> super::ShortFloat {
        self.token.priority()
    }

    fn __priority_mut(&mut self) -> &mut super::ShortFloat {
        self.token.__priority_mut()
    }

    fn durability(&self) -> super::ShortFloat {
        self.token.durability()
    }

    fn __durability_mut(&mut self) -> &mut super::ShortFloat {
        self.token.__durability_mut()
    }

    fn quality(&self) -> super::ShortFloat {
        self.token.quality()
    }

    fn __quality_mut(&mut self) -> &mut super::ShortFloat {
        self.token.__quality_mut()
    }
}

impl Item for Task {
    fn key(&self) -> &String {
        self.token.key()
    }

    /// 决定两个「任务」之间的「合并顺序」
    /// * 🚩 true ⇒ 改变顺序(self <- newer)，并入newer
    /// * 🚩false ⇒ 维持原样(newer <- self)，并入self
    fn merge_order(&self, newer: &Self) -> MergeOrder {
        match self.creation_time() < newer.creation_time() {
            // * 📝自身「创建时间」早于「要移出的任务」 ⇒ 将「要移出的任务」并入自身 ⇒ 新任务并入旧任务
            true => MergeOrder::NewToOld,
            // * 📝自身「创建时间」晚于「要移出的任务」 ⇒ 将「要移出的任务」并入自身 ⇒ 旧任务并入新任务
            false => MergeOrder::OldToNew,
        }
    }
}

impl Evidential for Task {
    fn evidential_base(&self) -> &[ClockTime] {
        self.sentence.evidential_base()
    }

    fn creation_time(&self) -> ClockTime {
        self.sentence.creation_time()
    }

    fn stamp_to_lexical(&self) -> narsese::lexical::Stamp {
        self.sentence.stamp_to_lexical()
    }
}

impl ToDisplayAndBrief for Task {
    fn to_display(&self) -> String {
        join! {
            => self.budget_to_display()
            => " "
            => self.key().to_string()
            => " "
            => self.stamp_to_display()
            => if let Some(parent_task) = &self.parent_task {
                let task = parent_task.get_();
                join!{
                    => "  \n from task: ".to_string()
                    => task.to_display_brief()
                }
            } else {"".to_string()}
            => if let Some(parent_belief) = &self.parent_belief {
                join!{
                    => "  \n from belief: ".to_string()
                    => parent_belief.to_display_brief()
                }
            } else {"".to_string()}
            => if let Some(best_solution) = &self.best_solution {
                join!{
                    => "  \n solution: ".to_string()
                    => best_solution.to_display_brief()
                }
            } else {"".to_string()}
        }
    }

    fn to_display_brief(&self) -> String {
        join! {
            => self.budget_to_display_brief()
            => " "
            => self.key()
        }
    }
}

impl Sentence for Task {
    fn sentence_clone<'s, 'sentence: 's>(&'s self) -> impl Sentence + 'sentence {
        self.sentence.sentence_clone()
    }

    fn content(&self) -> &crate::language::Term {
        self.sentence.content()
    }

    fn content_mut(&mut self) -> &mut crate::language::Term {
        self.sentence.content_mut()
    }

    type Judgement = <SentenceV1 as Sentence>::Judgement;
    type Question = <SentenceV1 as Sentence>::Question;

    fn as_punctuated_ref(&self) -> super::PunctuatedSentenceRef<Self::Judgement, Self::Question> {
        self.sentence.as_punctuated_ref()
    }

    fn to_key(&self) -> String {
        self.sentence.to_key()
    }

    fn sentence_to_display(&self) -> String {
        self.sentence.sentence_to_display()
    }

    fn sentence_to_lexical(&self) -> LexicalSentence {
        self.sentence.sentence_to_lexical()
    }
}

/// 有关「序列反序列化」的实用方法
impl RCTask {
    /// 将[`serde`]反序列化后【分散】了的引用按「标识符」重新统一
    pub fn unify_rcs<'t>(refs: impl IntoIterator<Item = &'t mut RCTask>)
    where
        Task: 't,
    {
        use std::collections::HashMap;

        // 构建空映射
        let mut serial_map: HashMap<usize, RCTask> = HashMap::new();

        // 一个用于统一每个「任务共享引用」的闭包
        let mut deal_serial = move |task_rc: &mut SerialRef<Task>| {
            // 先尝试获取已有同序列号的引用
            match serial_map.get(&task_rc.serial()) {
                // 若已有同序列号的引用，则直接归一化
                // * ✅此时归一化后被`clone`的`rc`已经被【同步序列号】了
                Some(rc) => *task_rc = rc.clone(),
                // 若无已有同序列号的引用，则同步序列号，并以旧序列号为键进入表中
                // * ℹ️自身序列号已更新，但旧序列号仍用于映射索引
                None => {
                    let serial_to_identify = task_rc.serial();
                    task_rc.sync_serial();
                    serial_map.insert(serial_to_identify, task_rc.clone());
                }
            }
        };

        // 遍历所有引用，开始归一化
        for task_rc in refs {
            // 遍历「任务」中的所有「任务共享引用」字段
            // * 🎯【2024-08-12 02:15:01】为了避免遗漏「父任务」这个字段
            // TODO: 后续或许能通用化成 `T::遍历内部所有与自身有关的共享引用(&mut self, mut 探针: impl Fn(&mut Self))`
            if let Some(parent) = task_rc.mut_().parent_task.as_mut() {
                deal_serial(parent) // 有父任务⇒处理父任务
            }
            // 总是先处理自身
            deal_serial(task_rc)
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{
        budget,
        entity::{QuestionV1, Stamp},
        ok, stamp, term,
        util::AResult,
    };
    use nar_dev_utils::*;

    /// 样本任务
    /// * 🎯不考虑内部所持有的内容，只考虑其地址与指针位置
    fn task_sample() -> Task {
        Task::from_input(
            QuestionV1::new(term!("A").unwrap(), stamp!({0: 1})),
            budget![1.0; 1.0; 1.0],
        )
    }

    /// 方法式语法糖
    impl Task {
        fn serial(&self) -> usize {
            RCTask::get_serial(self)
        }
    }

    /// 方法式语法糖
    impl RCTask {
        /// 指定序列号创建[`RCTask`]
        /// * 📌序列号需要在`inner`之前：传参时有可能从`inner`中来
        /// * ⚠️构造之后将会出现「序列号字段与现取序列号不一致」的情况
        fn with_serial(serial: usize, inner: Task) -> Self {
            Self {
                rc: RC::new_(inner),
                serial,
            }
        }

        /// 判断序列号是否已同步
        /// * 🚩判断自身序列号是否与内部内容的地址相同
        fn is_synced_serial(&self) -> bool {
            self.serial == self.inner_serial()
        }
    }

    mod task {
        use super::*;

        /// 序列号 特性：clone后改变
        #[test]
        fn serial_clone() -> AResult {
            let t1 = task_sample();
            let t2 = t1.clone();
            let [s1, s2] = [t1.serial(), t2.serial()];
            println!("pointer:\tt1->{:p},\tt2->{:p}", &t1, &t2);
            println!("serial: \tt1#0x{s1:x},\tt2#0x{s2:x}");
            assert_ne!(s1, s2);
            ok!()
        }

        /// 序列号 特性：移动后~~不变~~改变
        ///
        /// ! ⚠️【2024-08-11 16:41:28】移动语义是改变地址的，但需要的是Rc本身不变
        #[test]
        fn serial_move() -> AResult {
            let t1 = task_sample();
            print!("pointer:\tt1->{:p}, \t", &t1);
            let s1 = t1.serial();
            let t2 = t1;
            let s2 = t2.serial();
            println!("t2->{:p}", &t2);
            println!("serial: \tt1#0x{s1:x},\tt2#0x{s2:x}");
            assert_ne!(s1, s2); // ! 移动后地址改变
            ok!()
        }
    }

    /// [样本任务](task_sample)的共享引用
    /// * ✅一并测试了[`RCTask::new`]
    fn task_sample_rc() -> RCTask {
        RCTask::new(task_sample())
    }

    mod rc_task {
        use super::*;

        /// 构造稳定性
        #[test]
        fn new() -> AResult {
            let t = task_sample_rc();
            let s = t.serial(); // 取序列号

            // ! 序列号必须与现取的一致
            assert_eq!(s, t.inner_serial());

            ok!()
        }

        /// 序列号 特性：[`RCTask`]clone后不变
        #[test]
        fn serial_clone() -> AResult {
            let t1 = task_sample_rc();
            let t2 = t1.clone();
            let [s1, s2] = [t1.get_().serial(), t2.get_().serial()];
            println!("pointer:\tt1->{:p},\tt2->{:p}", &t1, &t2);
            println!("serial: \tt1#0x{s1:x},\tt2#0x{s2:x}");
            assert_eq!(s1, s2);
            ok!()
        }

        /// 序列号 特性：移动[`RCTask`]后内部[`Task`]的地址不变
        ///
        /// ! ⚠️【2024-08-11 16:41:28】移动语义改变了[`RCTask`]的地址，但没有改变内部[`Task`]的地址
        #[test]
        fn serial_move() -> AResult {
            let t1 = task_sample_rc();
            print!("pointer:\tt1->{:p}, \t", &t1);
            let s1 = t1.get_().serial();
            let t2 = t1;
            let s2 = t2.get_().serial();
            println!("t2->{:p}", &t2);
            println!("serial: \tt1#0x{s1:x},\tt2#0x{s2:x}");
            assert_eq!(s1, s2); // ! RC移动后，内部Task的地址不变
            ok!()
        }

        #[test]
        fn sync_serial() -> AResult {
            let task = task_sample();
            let t = RCTask::new(task.clone()); // 参照
            let s = t.serial(); // 取序列号
            let mut t1 = t.clone(); // 直接拷贝 | 序列号和引用都不同
            let mut t2 = RCTask::with_serial(s, task.clone()); // 序列号相同的实例，哪怕引用不同
            let mut t3 = RCTask::new(task.clone()); // 完全不相关的实例

            println!("t->{:p}\nt1->{:p}\nt2->{:p}\nt3->{:p}", &t, &t1, &t2, &t3); // 三个共享引用的地址
            println!(
                "*t->{:p}\n*t1->{:p}\n*t2->{:p}\n*t3->{:p}",
                &t.get_(),
                &t1.get_(),
                &t2.get_(),
                &t3.get_(),
            ); // 三个共享引用的内容地址

            // 同步前
            asserts! {
                t.ref_eq(&t1), // 直接clone的仍然是相等的
                !t.ref_eq(&t2), // 另俩都指向不同的任务
                !t.ref_eq(&t3), // 另俩都指向不同的任务

                t.serial() == t1.serial(), // 序列号相同
                t.serial() == t2.serial(), // 序列号相同
                t.serial() != t3.serial(), // 序列号不同
            }

            // 归一
            t1.sync_serial();
            t2.sync_serial();
            t3.sync_serial();

            println!("synced:");
            println!("t->{:p}\nt1->{:p}\nt2->{:p}\nt3->{:p}", &t, &t1, &t2, &t3); // 三个共享引用的地址
            println!(
                "*t->{:p}\n*t->{:p}\n*t2->{:p}\n*t3->{:p}",
                &t.get_(),
                &t1.get_(),
                &t2.get_(),
                &t3.get_(),
            ); // 三个共享引用的内容地址

            // 归一后
            asserts! {
                t.ref_eq(&t1), // 直接clone的仍然是相等的
                !t.ref_eq(&t2), // 本身仍然指向不同的任务
                !t.ref_eq(&t3), // 仍不相同的还指向不同的任务

                t.serial() == t1.serial(), // 序列号仍然相同
                t.serial() != t2.serial(), // 序列号变得不同
                t.serial() != t3.serial(), // 序列号仍然不同
            }
            ok!()
        }

        #[test]
        fn clone_stability() -> AResult {
            const N: usize = 10;
            let t = task_sample_rc();

            let ts = [&t]
                .iter()
                .cycle()
                .map(|&r| r.clone())
                .take(N)
                .collect::<Vec<_>>();
            println!("t->{:p}", &t); // 共享引用的地址
            for (i, t) in ts.iter().enumerate() {
                println!("t{i}->{t:p}");
            }
            println!("*t->{:p}", &t.get_()); // 共享引用的内容地址
            for (i, t) in ts.iter().enumerate() {
                println!("*t{i}->{:p}", &t.get_());
            }

            // 假定：拷贝之后序列号不变
            for t in ts {
                assert_eq!(t.serial(), t.serial());
                assert_eq!(t.serial(), t.inner_serial());
            }

            ok!()
        }
    }

    mod serde {
        use super::*;

        /// 模拟[`serde`]中「将[`RCTask`]序列化又反序列化」后的结构
        fn serde_rc_task(rc: &RCTask) -> RCTask {
            pipe! {
                rc
                => serde_json::to_string(rc) => .unwrap() => .as_ref()
                => serde_json::from_str => .unwrap()
            }
        }

        #[test]
        fn unify_rcs() -> AResult {
            let task = task_sample();
            let mut t = RCTask::new(task.clone()); // 参照
            let s = t.serial(); // 取序列号
            let t1 = t.clone(); // 直接拷贝 | 序列号和引用都不同
            let t2 = RCTask::with_serial(s, task.clone()); // 序列号相同的实例，哪怕引用不同
            let t3 = RCTask::new(task.clone()); // 完全不相关的实例

            /// 展示所有四个引用
            macro_rules! show {
                ($title:expr) => {
                    println!("{}", $title);
                    show! {}
                };
                {} => {
                    println!("t->{:p}\nt1->{:p}\nt2->{:p}\nt3->{:p}", &t, &t1, &t2, &t3); // 三个共享引用的地址
                    println!(
                        "*t->{:p}\n*t->{:p}\n*t2->{:p}\n*t3->{:p}",
                        &t.get_(),
                        &t1.get_(),
                        &t2.get_(),
                        &t3.get_(),
                    ); // 三个共享引用的内容地址
                };
            }

            show! {}

            // 同步前
            asserts! {
                t.ref_eq(&t1), // 直接clone的仍然是相等的
                !t.ref_eq(&t2), // 另俩都指向不同的任务
                !t.ref_eq(&t3), // 另俩都指向不同的任务

                t.serial() == t1.serial(), // 序列号相同
                t.serial() == t2.serial(), // 序列号相同
                t.serial() != t3.serial(), // 序列号不同
            }

            // 破坏引用
            let [mut t1, mut t2, mut t3] = f_parallel![serde_rc_task; &t1; &t2; &t3];

            show!("broken:");

            // 归一
            RCTask::unify_rcs([&mut t, &mut t1, &mut t2, &mut t3]);

            show!("synced:");

            // 归一后
            asserts! {
                t.ref_eq(&t1), // 直接clone的仍然是相等的
                t.ref_eq(&t2), // 应该被统一
                !t.ref_eq(&t3), // 仍然独立

                t.serial() == t1.serial(), // 序列号相同
                t.serial() == t2.serial(), // 序列号相同
                t.serial() != t3.serial(), // 序列号不同
            }
            // 确保序列号均已同步
            for t in [&t, &t1, &t2, &t3] {
                assert!(t.is_synced_serial());
            }
            ok!()
        }

        /// 较大规模的同步
        #[test]
        fn unify_rcs_large() -> AResult {
            /// 测试的规模（单次任务个数）
            const RANGE_N: std::ops::Range<usize> = 100..500;
            const MAX_N_GROUPS: usize = 5;

            /// 检查是否均统一
            fn verify_unified(tasks: &[RCTask]) {
                if tasks.is_empty() {
                    return;
                }
                let t0 = &tasks[0];
                for t in tasks {
                    // 检查「序列号一致」
                    assert!(t.is_synced_serial());
                    // 检查「引用相等⇔序列号相等」
                    let is_serial_eq = t0.serial() == t.serial();
                    assert!(t0.ref_eq(t) == is_serial_eq);
                }
            }

            for n in RANGE_N {
                let n_groups = (n % MAX_N_GROUPS) + 1;
                let tasks = list![
                    (vec![task_sample_rc(); n / n_groups]) // 每次添加 n / n_groups个任务
                    for _ in (0..n_groups) // 此处会重复n_groups次
                ]
                .concat(); // 总共 n 个任务

                // 序列反序列化 破坏引用
                let mut tasks = tasks.iter().map(serde_rc_task).collect::<Vec<_>>();

                // 归一化 修复引用
                RCTask::unify_rcs(tasks.iter_mut());

                // 检验
                verify_unified(&tasks);
            }
            ok!()
        }
    }
}
