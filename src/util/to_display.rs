//! 🆕用于统一实现OpenNARS中各个「实体」的`toString`与`toStringBrief`方法
//! * 📌环境上是对「孤儿规则」的「无法自动派生`Debug`/`Display`」之妥协
//! * 🎯增加[`to_display`](ToDisplayAndBrief::to_display)与[`to_display_brief`](ToDisplayAndBrief::to_display_brief)两个选项
//!   * 📌分别对应`toString`与`toStringBrief`
//! * 🎯用于批量管理对OpenNARS`toString`与`toStringBrief`的实现
//! * 🚩【2024-05-08 23:28:56】现在迁移到项目根目录：[`crate::language`]与[`crate::entity`]同时用到它

/// 转换到字符串，以及简略版本
/// * 🎯用以模拟`Object.toString`与`*.toStringBrief`
///   * 🎯绕过「孤儿规则」统一复刻「转换为字符串」方法
///   * 🎯不占用[`std::fmt::Debug`]与[`std::fmt::Display`]
pub trait ToDisplayAndBrief {
    /// 模拟`Object.toString`
    /// * 🎯显示完整信息
    /// * 📝本质上是完整实现OpenNARS中所有「实体」的`toString`方法
    /// * 🎯各类「以字符串作为『元素id』索引的初代实现」所用
    #[doc(alias = "toString")]
    fn to_display(&self) -> String;

    /// 模拟`*.toStringBrief`
    /// * 🎯显示简略信息
    /// * 📝本质上是完整实现OpenNARS中所有「实体」的`toStringBrief`方法
    /// * 📜默认实现：跟随[`ToDisplayAndBrief::to_display`]
    ///   * 📄参考：[`crate::entity::Stamp`]
    #[doc(alias = "toStringBrief")]
    #[doc(alias = "to_brief")]
    #[inline(always)]
    fn to_display_brief(&self) -> String {
        self.to_display()
    }

    /// 模拟`*.toStringLong`
    /// * 🎯显示简略信息
    /// * 📝本质上是完整实现OpenNARS中所有「实体」的`toStringLong`方法
    /// * 📜默认实现：跟随[`ToDisplayAndBrief::to_display`]
    /// * 🚩【2024-05-09 00:25:41】虽然OpenNARS中许多类型未实现，但此中还是全部为之添加
    ///   * 🎯减少代码复杂性：尽可能不分裂实现
    ///   * ️📝虽然这个方法仅在「推理器」中被调用
    #[doc(alias = "toStringLong")]
    #[doc(alias = "to_string_long")]
    #[doc(alias = "to_display_verbose")]
    #[inline(always)]
    fn to_display_long(&self) -> String {
        self.to_display()
    }
}

/// 仅在内容非空时展示（且自动为标题填充换行符）
pub fn to_display_when_has_content(title: &str, content: impl AsRef<str>) -> String {
    let s = content.as_ref();
    match s.trim().is_empty() {
        true => "".into(),
        false => format!("\n{title}{s}"),
    }
}

/// 方便快捷地 自动实现 [`ToDisplayAndBrief`]
/// * 🎯自动使用被实现类型内置的`__to_display`与`__to_display_brief`实现[`ToDisplayAndBrief::to_display`]
///
/// ! ❌【2024-06-21 19:58:58】无法泛化到更广的「任意特征之间的委托」
///   * 📌理由：需要预先知道特征的方法签名，才能自动填充特征方法实现
#[macro_export]
macro_rules! __impl_to_display {
    // 完整版：三种全部支持定制
    // * 🚩【2024-05-08 23:40:31】目前是唯一的：只需在使用它的特征中自动添加一个「内联默认方法」即可
    ( // * 应对各种「定制」要求：缺省`brief`的、多出`long`的
        @( // * 📌必须得要标识符：没有识别出标识符的，无法被对应捕获（并正确重复）
            $( $to_display_name:ident )? ;
            $( $to_display_brief_name:ident )? ;
            $( $to_display_long_name:ident )? $(;)?
        )
        $( { $( $generics:tt )* } )?
        $ty:ty as $ty_as:ty
        $( where $( $where_cond:tt )* )?
    ) => {
        impl< $( $( $generics )* )? > $crate::util::ToDisplayAndBrief for $ty
        $( where $( $where_cond )* )?
        {
            $(
                #[inline(always)]
                fn to_display(&self) -> String {
                    <Self as $ty_as>::$to_display_name(self)
                }
            )?

            $(
                #[inline(always)]
                fn to_display_brief(&self) -> String {
                    <Self as $ty_as>::$to_display_brief_name(self)
                }
            )?

            $(
                #[inline(always)]
                fn to_display_long(&self) -> String {
                    <Self as $ty_as>::$to_display_long_name(self)
                }
            )?
        }
    };
    // * 🚩拦截无效格式，并展示编译错误
    ( @ $inner1:tt  $($inner2:tt)* ) => {
        core::compile_error!(
            concat!(
                "方法自动实现错误：",
                "@", stringify!($inner1),
                " 来源：",
                stringify!($($inner2)*),
            )
        );
    };
    // 简单版：默认支持两种
    // * 🚩方法：直接转发
    ( $($inner:tt)* ) => {
        $crate::__impl_to_display! {
            @( // * 📌【2024-05-09 00:42:33】↓目前最常见就两种
                __to_display;
                __to_display_brief;
                // to_display_long
            )
            $($inner)*
        }
    };
    // // 无`brief`版 | // * 📄OpenNARS中「时间戳」没有`toStringBrief`，以备此况
    // (@NO_BRIEF $ty:ty as $ty_as:ty $( { $( $generics:tt )* } )? ) => {
    //     impl< $( $( $generics )* )? > ToDisplayAndBrief for $ty {
    //         #[inline(always)]
    //         fn to_display(&self) -> String {
    //             <Self as $ty_as>::__to_display(self)
    //         }
    //     }
    // };
}

/// 方便快捷地 自动实现 [`ToDisplayAndBrief`] 和 [`std::fmt::Display`]
/// * 🎯自动使用[`ToDisplayAndBrief::to_display`]派生[`std::fmt::Display`]特征
#[macro_export]
macro_rules! __impl_to_display_and_display {
    // 🚩假定两种都实现了 | 将「预设两种都实现了」交给__impl_to_display
    (
        @( $( $inner_prefix:tt )* )
        $( $inner:tt )*
    ) => {
        // * 🚩直接传给后续：标签树语法已经基本统一了
        $crate::__impl_to_display! { @( $( $inner_prefix )* ) $( $inner )* }
        $crate::impl_display_from_to_display! { $( $inner )* }
    };
    (
        $( $inner:tt )*
    ) => {
        // * 🚩直接传给后续：标签树语法已经基本统一了
        $crate::__impl_to_display! { $( $inner )* }
        $crate::impl_display_from_to_display! { $( $inner )* }
    };
}

/// 方便快捷地 自动实现 [`std::fmt::Display`]
/// * 🎯自动使用[`ToDisplayAndBrief::to_display`]派生[`std::fmt::Display`]特征
#[macro_export]
macro_rules! impl_display_from_to_display {
    ( // * 🚩【2024-05-08 23:44:54】↓此处需要加泛型约束：应对带泛型情况
        $( { $( $generics:tt )* } )?
        $ty:ty $(as $ty_as:ty)? // * ←🚩【2024-05-09 00:34:08】此处仅为了统一语法，为了上边能统一传标签流
        $( where $( $where_cond:tt )* )?
    ) => {
        impl< $( $( $generics )* )? > std::fmt::Display for $ty
        $( where $( $where_cond )* )?
        {
            #[inline(always)]
            fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                f.write_str(&$crate::util::ToDisplayAndBrief::to_display(self))
            }
        }
    };
}
