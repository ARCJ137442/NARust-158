/// 一次性实现某个结构体
/// * 🎯构建一个一次性结构体，实现所需求的功能（并在后续传入参数）
///   * ✅相比「多个闭包」可规避部分所有权问题
///     * 📄`call(|| self.get(X), |x, y| self.set(x, y))` => `call(context)`
/// * 📝【2024-09-04 01:08:51】目前不提供「一次性实现的`enum`」
///   * 💭理由：极少见，一般用`struct`足以胜任大多场景
/// * 📌【2024-09-04 10:52:26】兼容泛型，但不推荐使用
///   * 📝一般而言，在具体产生上下文时，均能得到其中的类型信息
///   * 🚧目前不兼容太复杂的泛型用法
///
/// ## 使用语法
/// ```no_run
/// impl_once! {
///     struct 【临时结构体名】 （in 【生命周期参数】） {
///         【字段名】 : 【字段类型】 = 【字段值】,
///     } impl 【实现的特征】 {
///         trait_body
///     }
/// }
/// ```
#[macro_export]
macro_rules! impl_once {
    ( // 无泛型：转发
        $(#[$attr:meta])*
        struct $name:ident $(in $life:lifetime)? {
            $(
                $field_name:ident : $field_type:ty = $field_value:expr $(,)?
            )*
        }
        $(#[$attr_impl:meta])*
        impl $trait:ty {
            $($trait_body:tt)*
        }
    ) => {
        impl_once! {
            $(#[$attr])*
            struct $name [ $($life)? ] {
                $(
                    $field_name : $field_type = $field_value
                )*
            }
            $(#[$attr_impl])*
            impl $trait {
                $($trait_body)*
            }
        }
    };
    ( // 有泛型：具体展开
        $(#[$attr:meta])*
        struct $name:ident [ $($generics:tt)* ] {
            $(
                $field_name:ident : $field_type:ty = $field_value:expr $(,)?
            )*
        }
        $(#[$attr_impl:meta])*
        impl $trait:ty {
            $($trait_body:tt)*
        }
    ) => {
        { // * 📌整体是一个块表达式
            $(#[$attr])*
            // * 🚩针对功能定义一个结构体
            struct $name <$($generics)*> {
                $(
                    $field_name : $field_type,
                )*
            }
            $(#[$attr_impl])*
            // * 🚩实现功能
            impl <$($generics)*> $trait for $name <$($generics)*> {
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

#[cfg(test)]
mod tests {

    #[test]
    fn test() {
        use std::collections::HashMap;
        trait Context<K, V> {
            fn get(&self, key: &K) -> Option<V>;
            fn set(&mut self, key: &K, value: V);
        }

        fn inc_or_one<K>(context: &mut impl Context<K, usize>, key: &K) {
            let value = context.get(key).map_or(1, |value| value + 1);
            context.set(key, value);
        }

        let mut map: HashMap<String, usize> = HashMap::new();
        let context = impl_once! {
            struct MapRef in 'a {
                map: &'a mut HashMap<String, usize> = &mut map,
            } impl Context<String, usize> {
                fn get(&self, key: &String) -> Option<usize> {
                    self.map.get(key).copied()
                }

                fn set(&mut self, key: &String, value: usize) {
                    self.map.insert(key.clone(), value);
                }
            }
        };
        let key = "key".to_string();
        inc_or_one(context, &key);
        inc_or_one(context, &key);
        let key2 = "key2".to_string();
        inc_or_one(context, &key2);
        assert_eq!(map.get(&key), Some(&2));
        assert_eq!(map.get(&key2), Some(&1));
    }
}
