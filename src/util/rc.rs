//! 有关对Java「引用计数类型」的复刻
//! * 🎯用于后续对OpenNARS「共享引用」类型的复刻
use std::{
    cell::RefCell,
    ops::{Deref, DerefMut},
    rc::Rc,
    sync::{Arc, Mutex},
};

/// 基于[`Rc`]与[`RefCell`]的可变共享引用
pub type RcCell<T> = Rc<RefCell<T>>;
/// 基于[`Arc`]与[`Mutex`]的可变共享引用
pub type ArcMutex<T> = Arc<Mutex<T>>;

/// 统一[`Rc`]与[`Arc`]的「可变共享引用」特征
/// * 🎯统一「可变共享引用」：只要保证「只调用特征方法」即可「无缝切换[`Rc`]与[`Arc`]」
/// * 📝实际上不需要一个专门的特征去表示「引用」，直接使用[`Deref`]系列即可
/// * 📝为解决「获取引用时，引用的生命周期不超过自身的生命周期」的问题，只能在「特征方法」处用`impl`而不能用「带生命周期的关联类型」
///   * 📌`-> Self::Ref + 'r`不合法：具体类型不能直接加生命周期约束
///   * 📌对于[`Rc`]、[`Arc`]获取引用的[`Ref`]等类型，不能在关联类型中明确指定生命周期
///     * 📝这样会将整个引用的生命周期限定死，导致在使用中出现「活不久」编译报错
/// * 🚩【2024-05-22 15:32:30】目前暂不打算支持「弱引用」类型
///   * 📌目前主要用于「任务链→任务→任务」，任务之间具有树状引用结构，同时「任务链」单向指向任务
pub trait RefCount<T>: Sized + Clone {
    /// 特征方法：获取不可变引用
    /// * 🚩可能是包装类型：[`Rc`]等需要一个特别的「代理类型」封装内部引用
    fn get_<'r, 's: 'r>(&'s self) -> impl Deref<Target = T> + 'r;

    /// 特征方法：获取可变引用（包装类型）
    /// * 🚩可能是包装类型：[`Rc`]等需要一个特别的「代理类型」封装内部引用
    fn mut_<'r, 's: 'r>(&'s mut self) -> impl DerefMut<Target = T> + 'r;

    /// 特征方法：构造函数
    /// * 🎯从实际值中构造一个「可变共享引用」
    fn new_(t: T) -> Self;

    /// 特征方法：强引用数目
    /// * 🎯统一表示「强引用数」
    fn n_strong_(&self) -> usize;

    /// 特征方法：弱引用数目
    /// * 🎯统一表示「弱引用数」
    fn n_weak_(&self) -> usize;

    /// 默认特征方法：返回整个共享引用的拷贝
    /// * 🚩约束：仅在内部元素支持[`Clone`]时使用
    fn clone_(&self) -> Self
    where
        T: Clone,
    {
        self.clone()
    }

    /// 默认特征方法：返回内部元素的拷贝
    /// * 🚩约束：仅在内部元素支持[`Clone`]时使用
    fn clone_inner(&self) -> T
    where
        T: Clone,
    {
        self.get_().clone()
    }

    /// 判断是否引用到相同的对象
    /// * 📌所谓「引用判等」
    /// * ⚠️比「值相等」更严格，并且与[`Eq`]无强关联
    fn ref_eq(&self, other: &Self) -> bool;
}

// impls //

/// 对[`Rc<RefCell<T>>`](Rc)实现
impl<T> RefCount<T> for RcCell<T> {
    #[inline(always)]
    fn get_<'r, 's: 'r>(&'s self) -> impl Deref<Target = T> + 'r {
        RefCell::borrow(self)
    }

    #[inline(always)]
    fn mut_<'r, 's: 'r>(&'s mut self) -> impl DerefMut<Target = T> + 'r {
        RefCell::borrow_mut(self)
    }

    #[inline(always)]
    fn new_(t: T) -> Self {
        Rc::new(RefCell::new(t))
    }

    #[inline(always)]
    fn n_strong_(&self) -> usize {
        Rc::strong_count(self)
    }

    #[inline(always)]
    fn n_weak_(&self) -> usize {
        Rc::weak_count(self)
    }

    #[inline(always)]
    fn ref_eq(&self, other: &Self) -> bool {
        Rc::ptr_eq(self, other)
    }
}

/// 对[`Arc<Mutex<T>>`](Arc)实现
impl<T> RefCount<T> for ArcMutex<T> {
    #[inline(always)]
    fn get_<'r, 's: 'r>(&'s self) -> impl Deref<Target = T> + 'r {
        // * ❓或许后续可以考虑使用`get_try`等
        self.lock().expect("互斥锁已中毒")
    }

    #[inline(always)]
    fn mut_<'r, 's: 'r>(&'s mut self) -> impl DerefMut<Target = T> + 'r {
        self.lock().expect("互斥锁已中毒")
    }

    #[inline(always)]
    fn new_(t: T) -> Self {
        Arc::new(Mutex::new(t))
    }

    #[inline(always)]
    fn n_strong_(&self) -> usize {
        Arc::strong_count(self)
    }

    #[inline(always)]
    fn n_weak_(&self) -> usize {
        Arc::weak_count(self)
    }

    #[inline(always)]
    fn ref_eq(&self, other: &Self) -> bool {
        Arc::ptr_eq(self, other)
    }
}

/// 测试用
#[cfg(test)]
pub mod tests {
    use super::*;

    /// 测试 / 通用
    /// * 🎯只用特征方法，不影响用法地兼容[`Rc`]与[`Arc`]
    fn test_rc<R: std::fmt::Debug + RefCount<i32>>() {
        // * 🚩创建一个可变共享引用，并展示
        let mut rc = R::new_(0);
        dbg!(rc.clone_());
        // * 🚩修改引用，断言，并展示
        let mut r = rc.mut_();
        *r += 1;
        assert_eq!(*r, 1);
        // * 🚩释放引用
        drop(r);

        // * 🚩复制这个可变共享引用，验证「多个不可变引用同时存在」
        let rc2 = dbg!(rc.clone_());

        // ! ⚠️此处不能同时获取：对`Mutex`会导致线程死锁
        let value = *rc.get_();
        let value2 = *rc2.get_();
        assert_eq!(value, value2);
    }

    /// 测试 / [`Rc`] & [`Arc`]
    /// * 🎯两种类型的无缝切换
    #[test]
    fn tests_ref_count() {
        test_rc::<RcCell<i32>>();
        test_rc::<ArcMutex<i32>>();
    }

    // 实例测试 //

    /// 🎯控制使用的「共享可变引用」类型
    /// * ✅【2024-05-22 12:38:32】现在可以无缝在[`Rc`]与[`Arc`]之间切换
    type R<T> = ArcMutex<T>;

    #[derive(Debug, Clone)]
    struct Task {
        pub content: String,
        parent: Option<R<Task>>,
    }

    impl Task {
        pub fn new(content: impl Into<String>, parent_task: Option<&R<Task>>) -> Self {
            Self {
                content: content.into(),
                parent: parent_task.map(R::clone),
            }
        }

        pub fn new_rc(content: impl Into<String>, parent_task: Option<&R<Task>>) -> R<Self> {
            R::new_(Self::new(content, parent_task))
        }

        pub fn parent(&self) -> Option<R<Task>> {
            self.parent.clone()
        }

        /// 设置父任务
        /// * 📝因为其所产生的引用是[自身类型](Task)（且引用目标就是[自身类型](Task)），
        ///   * 有可能发生循环引用
        /// * 📝但若禁止在构造后修改此处的值，则不会有事——不可能在构造时传入自身
        pub fn set_parent(&mut self, parent: &R<Task>) -> &mut R<Task> {
            self.parent.insert(parent.clone())
        }

        /// 🆕删除父任务
        /// * 🎯用于解除循环引用
        pub fn delete_parent(&mut self) -> Option<R<Task>> {
            self.parent.take()
        }
    }

    /// 任务 / 更改父任务
    #[test]
    fn test_set_parent() {
        let task_i = Task::new_rc("input.", None);
        let task_j = Task::new_rc("JnPut.", Some(&task_i));
        let mut task_k = Task::new_rc("KnPut.", Some(&task_i));
        // let r = task_k.borrow_mut(); // ! 启用这行，删掉dbg，就会引发借用panic
        task_k.mut_().set_parent(&task_j);
        dbg!(task_i, task_j, task_k);
        // * ♻️Dropped: task_k
        // * ♻️Dropped: task_j
        // * ♻️Dropped: task_i
    }

    /// 任务 / 循环引用
    #[test]
    fn test_recursive() {
        let mut task_i = Task::new_rc("recursive.", None);
        // let task_j = Task::new_rc("j from i.", Some(&task_i)); // ! 尝试链接到「自引用任务」会爆栈
        // let task_k = Task::new_rc("k from j.", Some(&task_j));
        // * 🚩设置递归
        let task_i_self = task_i.clone();
        task_i.mut_().set_parent(&task_i_self);

        // * 🚩检验递归
        // ! ⚠️若将`parent`内联，则会造成「重复锁定」导致「线程死锁」
        let parent = task_i.get_().parent().unwrap();
        assert_eq!(parent.get_().content, "recursive.");

        // * 🚩删除递归
        task_i.mut_().delete_parent(); // ! 必须先删除循环引用，才能正常删除整体
        dbg!(task_i.n_strong_(), task_i.n_weak_());

        // * ♻️Dropped: task_i
    }
}
