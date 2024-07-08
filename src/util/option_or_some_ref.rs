//! 传参简化：`&T`⇔`Some(&T)`

/// 工具性trait
/// * 🎯在参数中使用`impl OptionOrSomeRef<'a, T>`同时支持传入`&T`和`Option<&T>`
///   * ✨其中`&T`会自动转换成`Some(&T)`
///
/// ## 用例：
///
/// ```
/// use narust_158::util::OptionOrSomeRef;
///
/// /// 从可选引用转换到`Option<&S>`，传入引用自动转换为`Some(&S)`
/// fn f<S: Clone + Into<String>>(s: impl OptionOrSomeRef<S>) -> Option<String> {
///     let option = s.or_some();
///     option.cloned().map(Into::into)
/// }
///
/// let s = String::from("hello");
/// let expected = Some("hello".to_string());
///
/// assert_eq!(f(&s), expected);
/// assert_eq!(f(Some(&s)), expected);
/// assert_eq!(f(None::<&String>), None);
/// ```
pub trait OptionOrSomeRef<T> {
    /// 将自身转换成`Option`
    /// * ✨`&T`会自动转换成`Some(&T)`
    /// * 📝直接在特征方法中做约束，好过在特征定义中放生命周期参数
    fn or_some<'a>(self) -> Option<&'a T>
    where
        Self: 'a;
}

/// 对引用实现
impl<T> OptionOrSomeRef<T> for &T {
    #[inline(always)]
    fn or_some<'a>(self) -> Option<&'a T>
    where
        Self: 'a,
    {
        Some(self)
    }
}

/// 对可空引用实现
impl<T> OptionOrSomeRef<T> for Option<&T> {
    #[inline(always)]
    fn or_some<'a>(self) -> Option<&'a T>
    where
        Self: 'a,
    {
        self
    }
}

/// [`OptionOrSomeRef`]的可变版本
pub trait OptionOrSomeRefMut<T>: OptionOrSomeRef<T> {
    /// 将自身转换成`Option`
    /// * ✨`&mut T`会自动转换成`Some(&mut T)`
    /// * 📝直接在特征方法中做约束，好过在特征定义中放生命周期参数
    fn or_some_mut<'a>(self) -> Option<&'a mut T>
    where
        Self: 'a;
}

/// 对引用实现不可变引用
impl<T> OptionOrSomeRef<T> for &mut T {
    #[inline(always)]
    fn or_some<'a>(self) -> Option<&'a T>
    where
        Self: 'a,
    {
        Some(self)
    }
}

/// 对可空引用实现不可变引用
impl<T> OptionOrSomeRef<T> for Option<&mut T> {
    #[inline(always)]
    fn or_some<'a>(self) -> Option<&'a T>
    where
        Self: 'a,
    {
        // * 🚩可变引用解引用，编译器能自动展开
        self.map(|r| &*r)
    }
}

/// 对可变引用实现
impl<T> OptionOrSomeRefMut<T> for &mut T {
    #[inline(always)]
    fn or_some_mut<'a>(self) -> Option<&'a mut T>
    where
        Self: 'a,
    {
        Some(self)
    }
}

/// 对可空可变引用实现
impl<T> OptionOrSomeRefMut<T> for Option<&mut T> {
    #[inline(always)]
    fn or_some_mut<'a>(self) -> Option<&'a mut T>
    where
        Self: 'a,
    {
        self
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// 通过引用获取一个值
    fn get(option_ref: impl OptionOrSomeRef<usize>) -> Option<usize> {
        option_ref.or_some().cloned()
    }

    /// 尝试让一个值递增
    fn inc(option_ref: impl OptionOrSomeRefMut<usize>) {
        if let Some(p) = option_ref.or_some_mut() {
            *p += 1
        }
    }

    /// 不可变性测试
    #[test]
    fn test_ref() {
        let mut a = 1_usize;
        let null = None::<usize>;
        assert_eq!(null, None); // 空
        assert_eq!(get(&a), Some(1)); // 不可变引用
        assert_eq!(get(&mut a), Some(1)); // 对可变引用也兼容
        assert_eq!(get(Some(&a)), Some(1)); // 不可变引用
        assert_eq!(get(Some(&mut a)), Some(1)); // 对可变引用也兼容
    }

    /// 可变性测试
    #[test]
    fn test_mut() {
        let mut a = 1_usize;
        // assert_eq!(inc(Some(&a)), Some(2)); // ! 编译错误
        inc(&mut a); // 仅引用
        assert_eq!(a, 2);
        inc(Some(&mut a)); // 用`Option`包裹
        assert_eq!(a, 3);
        inc(None); // 用`Option`包裹
        assert_eq!(a, 3);
    }
}
