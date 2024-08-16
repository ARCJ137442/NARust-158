//! 🎯复刻OpenNARS `nars.entity.Task`
//! * ✅【2024-05-05 21:38:53】基本方法复刻完毕
//! * ♻️【2024-06-21 23:33:24】基于OpenNARS改版再次重写

use super::{BudgetValue, Item, JudgementV1, Sentence, SentenceV1, Token};
use crate::{
    entity::MergeOrder,
    global::ClockTime,
    inference::{Budget, Evidential},
    util::{IterInnerRcSelf, RefCount, SerialRef, ToDisplayAndBrief},
};
use nar_dev_utils::join;
use narsese::lexical::{Sentence as LexicalSentence, Task as LexicalTask};
use serde::{Deserialize, Serialize};

/// A task to be processed, consists of a Sentence and a BudgetValue
#[derive(Debug, Clone, Serialize, Deserialize)]
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

/// 「任务」的共享引用版本
pub type RCTask = SerialRef<Task>;

/// 有关「序列反序列化」的实用方法
impl IterInnerRcSelf for Task {
    fn iter_inner_rc_self(&mut self) -> impl Iterator<Item = &mut SerialRef<Self>> {
        // 遍历「任务」中的所有「任务共享引用」字段
        // * 🎯【2024-08-12 02:15:01】为了避免遗漏「父任务」这个字段
        self.parent_task.as_mut().into_iter()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::util::tests_serial_rc::*;
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
        fn serial(&self) -> Serial_ {
            RCTask::get_serial_(self)
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
            let s = t.serial_(); // 取序列号

            // ! 序列号必须与现取的一致
            assert_eq!(s, t.inner_serial_());

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
            let s = t.serial_(); // 取序列号
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

                t.serial_() == t1.serial_(), // 序列号相同
                t.serial_() == t2.serial_(), // 序列号相同
                t.serial_() != t3.serial_(), // 序列号不同
            }

            // 归一
            t1.sync_serial_();
            t2.sync_serial_();
            t3.sync_serial_();

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

                t.serial_() == t1.serial_(), // 序列号仍然相同
                t.serial_() != t2.serial_(), // 序列号变得不同
                t.serial_() != t3.serial_(), // 序列号仍然不同
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
                assert_eq!(t.serial_(), t.serial_());
                assert_eq!(t.serial_(), t.inner_serial_());
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
            let s = t.serial_(); // 取序列号
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

                t.serial_() == t1.serial_(), // 序列号相同
                t.serial_() == t2.serial_(), // 序列号相同
                t.serial_() != t3.serial_(), // 序列号不同
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

                t.serial_() == t1.serial_(), // 序列号相同
                t.serial_() == t2.serial_(), // 序列号相同
                t.serial_() != t3.serial_(), // 序列号不同
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
                    let is_serial_eq = t0.serial_() == t.serial_();
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
