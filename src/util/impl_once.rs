/// 一次性实现某个结构体
/// * 📝【2024-09-04 01:08:51】目前不提供「一次性实现的`enum`」
///   * 💭理由：极少见，一般用`struct`足以胜任大多场景
/// * 📌其使用语法如下：
/// ```no_run
/// impl_once! {
///     struct 【临时结构体名】 in （生命周期参数） {
///         【字段名】 : 【字段类型】 = 【字段值】,
///     } impl 【实现的特征】 {
///         trait_body
///     }
/// }
/// ```
#[macro_export]
macro_rules! impl_once {
    (
        $(#[$attr:meta])*
        struct $name:ident $(in $life:lifetime)? {
            $(
                $field_name:ident : $field_type:ty = $field_value:expr $(,)?
            )*
        }
        $(#[$attr_impl:meta])*
        impl $trait:ident {
            $($trait_body:tt)*
        }
    ) => {
        { // * 📌整体是一个块表达式
            $(#[$attr])*
            // * 🚩针对功能定义一个结构体
            struct $name $(<$life>)? {
                $(
                    $field_name : $field_type,
                )*
            }
            $(#[$attr_impl])*
            // * 🚩实现功能
            impl $(<$life>)? $trait for $name $(<$life>)? {
                $($trait_body)*
            }
            // * 🚩构建并载入上下文，拿到独占引用
            //   * 📌【2024-09-04 01:16:25】目前总是取可变引用：后续使用时可根据「点方法」灵活调用
            &mut $name {
                $(
                    $field_name : $field_value,
                )*
            }
        }
    };
}
