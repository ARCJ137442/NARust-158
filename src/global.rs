//! 存储一些「全局」参数
//! * 🎯参数类型如「浮点数」（选择哪个精度）
//! * ⚠️【2024-04-27 10:47:59】尽量不要用来存储常量

/// 全局浮点数类型
pub type Float = f64;

/// 全局「时钟数」类型
/// * 🎯NARS内部推理时间
/// * 🎯时间戳[`crate::entity::Stamp`]
/// * 🚩【2024-05-04 17:41:49】目前设定为无符号整数，对标OpenNARS中的`long`长整数类型
///   * 📝OpenNARS中也是将其作为无符号整数（非负整数）用的
pub type ClockTime = usize;

/// 有关对Java「引用计数类型」的复刻
mod rc {
    use std::{
        cell::RefCell,
        ops::{Deref, DerefMut},
        rc::Rc,
    };

    /// 全局引用计数类型
    /// * 🚩【2024-05-04 16:51:11】目前尚未做多线程兼容，此处仅考虑单线程，
    ///   * 故暂且使用 [`std::rc::Rc`] 而非 [`std::sync::Arc`]
    pub type RC<T> = Rc<T>;

    /// 全局引用计数类型，可变版本
    /// * 📄[`RC`]的可变版本
    /// * 📝据[`Rc`]文档，实际上`&mut Rc<T>`也可以通过`get_mut`实现可变
    ///   * ❌但这只能在`Rc`只有**唯一一个引用**时才有效
    /// * 🚩【2024-05-15 11:52:37】目前仍然需要探索`RefCell`方案
    pub type RCMut<T> = RcMut<T>;

    /// 通用的「共享引用」接口
    /// * ✅【2024-05-15 16:07:34】通过「封装`struct`」解决了「共享可变性 or 可变共享」的歧义问题
    pub trait GlobalRc<'this, T>: Sized
    where
        Self: 'this,
    {
        /// 获取到的「引用」类型
        type Ref: Deref<Target = T> + 'this;

        /// 创建
        fn new_(value: T) -> Self;

        /// 获取不可变引用
        fn get_(&'this self) -> Self::Ref;
    }

    /// 通用的「共享可变引用」接口
    /// * 📌[`GlobalRc`]的可变版本
    pub trait GlobalRcMut<'this, T>: GlobalRc<'this, T> {
        /// 获取到的「可变引用」类型
        type RefMut: DerefMut<Target = T> + 'this;

        /// 获取可变引用
        fn mut_(&'this mut self) -> Self::RefMut;
    }

    /// 对[`Rc`]实现不可变共享引用
    impl<'this, T> GlobalRc<'this, T> for std::rc::Rc<T>
    where
        Self: 'this,
    {
        type Ref = &'this T;

        #[inline(always)]
        fn new_(value: T) -> Self {
            Rc::new(value)
        }

        #[inline(always)]
        fn get_(&'this self) -> Self::Ref {
            self.as_ref()
        }
    }

    /// 实现「可变共享引用」
    /// * 🎯提供一个内部实现
    ///   * 🚩通过全局常量予以公开
    ///   * 💭【2024-05-15 16:08:54】后续将实现「[`Arc`]无缝替代」
    mod rc_mut {
        use super::*;

        /// 「可变共享引用」包装类型
        /// * 🎯终结「到底是『T的可变共享引用』还是『RefCell<T>的不可变共享引用』」的问题
        ///   * 🚩【2024-05-15 16:03:39】直接选前者
        #[derive(Debug)]
        pub struct RcMut<T>(Rc<RefCell<T>>);

        /// 手动实现[`Clone`]复制
        /// * 🚩直接复制内部[`Rc`]
        impl<T> Clone for RcMut<T> {
            fn clone(&self) -> Self {
                Self(self.0.clone())
            }
        }

        /// 对包装类型[`RcMut`]实现不可变共享引用
        impl<'this, T> GlobalRc<'this, T> for RcMut<T>
        where
            Self: 'this,
        {
            type Ref = std::cell::Ref<'this, T>;

            #[inline(always)]
            fn new_(value: T) -> Self {
                Self(Rc::new(RefCell::new(value)))
            }

            #[inline(always)]
            fn get_(&'this self) -> Self::Ref {
                self.0.borrow()
            }
        }

        /// 对包装类型[`RcMut`]实现可变共享引用
        impl<'this, T> GlobalRcMut<'this, T> for RcMut<T>
        where
            Self: 'this,
        {
            type RefMut = std::cell::RefMut<'this, T>;

            #[inline(always)]
            fn mut_(&'this mut self) -> Self::RefMut {
                self.0.borrow_mut()
            }
        }
    }
    pub use rc_mut::*;
}
pub use rc::*;

/// 测试用
#[cfg(test)]
pub mod tests {
    use super::RC;

    /// 测试用类型，增强[`anyhow::Result`]
    pub type AResult<T = ()> = anyhow::Result<T>;

    /// 测试用宏，简化`Ok(())`
    #[macro_export]
    macro_rules! ok {
        () => {
            Ok(())
        };
        ($($code:tt)*) => {
            Ok($($code)*)
        };
    }

    #[test]
    fn t() {
        let mut rc = RC::new(0);
        dbg!(rc.clone());
        let r = RC::get_mut(&mut rc).expect("需要引用");
        *r += 1;
        dbg!(rc.clone());
    }
}
