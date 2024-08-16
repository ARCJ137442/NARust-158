use super::RefCount;
use crate::global::RC;
use serde::{Deserialize, Serialize};

/// 拥有「序列号」的共享引用
/// * 🎯【2024-08-11 16:16:44】用于实现序列反序列化，独立成一个特殊的类型
/// * 📌设计上「序列号」用于在「序列反序列化」前后承担「唯一标识」的角色
///   * 📝内容的地址会变，但序列号在序列反序列化中能（相对多个可遍历的引用而言）保持不变
///   * 💡核心想法：通过「序列号」实现「内容归一化」——序列号相同的「序列共享引用」可以实现「统一」操作
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SerialRef<T> {
    /// 内部引用
    rc: RC<T>,
    /// 所存储的，作为「唯一标识」的「序列号」
    serial: Serial,
}
/// 序列号的类型
/// * 🚩【2024-08-15 17:23:23】锁死在64位：避免「64位下保存的数值，在32位中无法加载」
type Serial = u64;

impl<T: Clone> SerialRef<T> {
    /// 从「内容」对象生成一个【随数据位置唯一】的「序列号」
    /// * 📌这个「序列号」必须对[`clone`](Clone::clone)敏感，即：
    ///   * `clone`之后的序列号必须与原始序列号【不同】
    ///   * 若被移入了类似[`RC`]这样的共享引用结构，不会因为[`RC`]的`clone`而改变
    /// * 🚩【2024-08-11 16:23:11】目前使用自身的指针地址
    ///
    /// ! 📝【2024-08-11 16:47:37】Rust中「移动语义」的含义：**移动后地址改变**
    ///   * 在`let t1 = inner(); let t2 = t1`时，`t1`和`t2`指向不同的内存地址
    fn get_serial(inner: &T) -> Serial {
        // 取自身指针地址地址作为序列号
        inner as *const T as Serial
    }

    /// 从一个[`RC`]中获取序列号
    fn get_serial_rc(inner: &RC<T>) -> Serial {
        Self::get_serial(&*inner.get_())
    }

    /// 使用所传入内容的地址创建一个[`RCTask`]
    /// * 📌这个内容的地址将被[`RCTask`]固定
    pub fn new(inner: T) -> Self {
        let rc = RC::new_(inner);
        let serial = Self::get_serial_rc(&rc) as Serial;
        Self { rc, serial }
    }

    /// 获取自身存储的序列号（字段）
    fn serial(&self) -> Serial {
        self.serial
    }

    /// 获取内部[`Task`]的序列号
    fn inner_serial(&self) -> Serial {
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

/// 工具性特征：可变迭代内部共享引用
pub trait IterInnerRcSelf: Sized {
    /// 可变迭代内部共享引用
    /// * 📄[任务](crate::entity::Task)的「父任务」字段
    fn iter_inner_rc_self(&mut self) -> impl Iterator<Item = &mut SerialRef<Self>>;
}

/// 有关「序列反序列化」的实用方法
impl<'t, T: Clone + IterInnerRcSelf + 't> SerialRef<T> {
    /// 将[`serde`]反序列化后【分散】了的引用按「标识符」重新统一
    pub fn unify_rcs(refs: impl IntoIterator<Item = &'t mut Self>) {
        use std::collections::HashMap;

        // 构建空映射
        let mut serial_map: HashMap<Serial, Self> = HashMap::new();

        // 一个用于统一每个「任务共享引用」的闭包
        let mut deal_serial = move |task_rc: &mut Self| {
            // 先尝试获取已有同序列号的引用
            match serial_map.get(&task_rc.serial()) {
                // 若已有同序列号的引用，则检查引用是否相等并尝试归一化
                // * ✅此时归一化后被`clone`的`rc`已经被【同步序列号】了
                Some(rc) => {
                    // 若引用不相等，则尝试归一化
                    // * 🎯【2024-08-12 20:29:14】在「已归一化后的任务共享引用」中 尽可能避免重复拷贝
                    if !task_rc.ref_eq(rc) {
                        *task_rc = rc.clone()
                    }
                }
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
            // 遍历内部的「自身类型共享引用」字段
            // * 📄任务的「父任务」
            for inner_rc in task_rc.mut_().iter_inner_rc_self() {
                deal_serial(inner_rc)
            }
            // 然后再处理自身
            deal_serial(task_rc)
        }
    }
}

/// 测试用例
/// * 📌【2024-08-16 17:06:41】历史原因，此处公开方法
///   * 🎯最初在[`Task`](crate::entity::Task)中进行的测试
#[cfg(test)]
pub(crate) mod tests_serial_rc {
    use super::*;

    /// 测试用例中公开类型
    pub type Serial_ = Serial;

    impl<T: Clone> SerialRef<T> {
        /// 测试用例中公开获取序列号
        pub fn serial_(&self) -> Serial {
            self.serial
        }

        /// 测试用例中公开生成序列号
        pub fn get_serial_(inner: &T) -> Serial {
            // 取自身指针地址地址作为序列号
            inner as *const T as Serial
        }

        /// 获取内部[`Task`]的序列号
        pub fn inner_serial_(&self) -> Serial {
            Self::get_serial(&*self.get_())
        }

        /// 测试用例中公开同步序列号
        pub fn sync_serial_(&mut self) {
            self.serial = self.inner_serial();
        }

        /// 指定序列号创建共享引用
        /// * 📌序列号需要在`inner`之前：传参时有可能从`inner`中来
        /// * ⚠️构造之后将会出现「序列号字段与现取序列号不一致」的情况
        pub fn with_serial(serial: Serial, inner: T) -> Self {
            Self {
                rc: RC::new_(inner),
                serial,
            }
        }

        /// 判断序列号是否已同步
        /// * 🚩判断自身序列号是否与内部内容的地址相同
        pub fn is_synced_serial(&self) -> bool {
            self.serial == self.inner_serial()
        }
    }
}
