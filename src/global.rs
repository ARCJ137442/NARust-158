//! 存储一些「全局」参数
//! * 🎯参数类型如「浮点数」（选择哪个精度）
//! * ⚠️【2024-04-27 10:47:59】尽量不要用来存储常量

/// 全局浮点数类型
pub type Float = f64;

/// 有关对Java「引用计数类型」的复刻
mod rc {
    use std::{cell::RefCell, rc::Rc};

    /// 全局引用计数类型
    /// * 🚩【2024-05-04 16:51:11】目前尚未做多线程兼容，此处仅考虑单线程，
    ///   * 故暂且使用 [`std::rc::Rc`] 而非 [`std::sync::Arc`]
    pub type RC<T> = Rc<T>;

    /// 全局引用计数类型，可变版本
    /// * 📄[`RC`]的可变版本
    /// * 📝据[`Rc`]文档，实际上`&mut Rc<T>`也可以通过`get_mut`实现可变
    ///   * ❌但这只能在`Rc`只有**唯一一个引用**时才有效
    /// * 🚩【2024-05-15 11:52:37】目前仍然需要探索`RefCell`方案
    pub type RCMut<T> = Rc<RefCell<T>>;

    /// 通用的「共享引用」接口
    ///
    /// TODO: 【2024-05-15 11:54:32】🚧有待调整
    ///   * ⚠️RefCell的`borrow`返回一个新结构，会导致「返回临时引用」问题
    pub trait GlobalRc<T>: Sized {
        /// 创建
        fn new_(value: T) -> Self;

        /// 获取不可变引用
        fn get_(&self) -> &T;

        /// 获取可变引用
        fn mut_(&mut self) -> Option<&mut T>;
    }

    impl<T> GlobalRc<T> for RCMut<T> {
        fn new_(value: T) -> Self {
            Rc::new(RefCell::new(value))
        }

        fn get_(&self) -> &T {
            todo!()
        }

        fn mut_(&mut self) -> Option<&mut T> {
            todo!()
        }
    }
}
pub use rc::*;

/// 全局「时钟数」类型
/// * 🎯NARS内部推理时间
/// * 🎯时间戳[`crate::entity::Stamp`]
/// * 🚩【2024-05-04 17:41:49】目前设定为无符号整数，对标OpenNARS中的`long`长整数类型
///   * 📝OpenNARS中也是将其作为无符号整数（非负整数）用的
pub type ClockTime = usize;

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
