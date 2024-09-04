use crate::control::Reasoner;
use nar_dev_utils::{macro_once, ResultS};

/// 指令[`Cmd::SAV`]的入口函数
/// * 📌传入的`query`默认为小写字串引用
/// * 📌输出仅为JSON字符串；若返回[错误值](Err)，则视为「报错」
pub fn sav_dispatch(
    reasoner: &mut Reasoner,
    query: impl AsRef<str>,
    path: impl AsRef<str>,
) -> ResultS<Output> {
    macro_once! {
        macro ( $( $query:literal => $output:expr )* ) => {
            /// 所有非空查询的列表
            /// * 📌格式：Markdown无序列表
            const ALL_QUERIES_LIST: &str = concat!($( "\n- ", $query, )*);
            match query.as_ref() {
                // * 🚩特殊/空字串：列举所有query并转接`HLP INF`
                // ! ⚠️【2024-08-09 17:48:15】不能放外边：会被列入非空查询列表中
                "" => Ok(Output::INFO { message: format!("Available save target: {ALL_QUERIES_LIST}") }),
                // 所有固定模式的分派
                // * 🚩【2024-08-18 00:55:40】现在需要传回自定义输出
                $( $query => Ok($output), )*
                // * 🚩其它⇒告警
                other => Err(format!("Unknown save target: {other:?}")),
            }
        }

        // 记忆区
        "memory" => generate_sav_callback(reasoner, query, path, memory_to_json)?
        // 推理器整体状态
        "status" => generate_sav_callback(reasoner, query, path, status_to_json)?
    }
}

/// 通用的「SAV」callback格式
///
/// ## 📌格式
///
/// ```plaintext
/// [指定消息头] 保存的目标类型:
/// 数据
/// ```
/// * 📄消息头参见[`SAV_INFO_HEAD`]
///   * 📌【2024-08-15 17:11:43】目前为`SAV`
fn generate_sav_callback(
    reasoner: &Reasoner,
    target: impl AsRef<str>,
    path: impl AsRef<str>,
    to_json: fn(&Reasoner) -> anyhow::Result<String>,
) -> ResultS<Output> {
    let [target, path] = [target.as_ref(), path.as_ref()];
    let data = to_json(reasoner)
        .map_err(|e| format!("Failed to serialize {target:?} at {path:?}: {e}"))?;
    let output = Output::format_sav_callback(path, data);
    Ok(output)
}

/// 便于外部调用的API
pub mod public {
    use nar_dev_utils::SplitOwned;
    use navm::output::Output;

    /// 在[`SAV`](Cmd::SAV)指令调用后，推理器输出的消息头
    /// * 🎯用于区分「一般消息」与「SAV回调」
    /// * ✅【2024-08-17 23:57:57】现在有了[`nar_dev_utils::SplitOwned`]，可以尝试更精确的格式
    /// * 📌【2024-08-18 00:14:05】留存空格，以便在后续[`str::strip_prefix`]时省去额外的判断
    const SAV_INFO_HEAD: &str = "SAV ";

    pub(super) fn format_sav_callback(path: impl Into<String>, data: impl Into<String>) -> String {
        let [path, data] = [path.into(), data.into()];
        // * 🚩此处将`SAV_INFO_HEAD`之后的空格算入`SAV_INFO_HEAD`中，以提高解析时性能
        // ? ↓【2024-08-18 00:09:04】此处会发生拷贝吗？后续可能需要性能优化
        format!("{SAV_INFO_HEAD}{path}:\n{data}")
        // [format!("{SAV_INFO_HEAD}{path}:\n"), data].concat()
    }

    /// 验证某条消息是否为有效回调，兼顾返回`(目标, 数据)`二元组
    /// * 🚩【2024-08-17 15:00:08】目前方法：验证整条消息**是否为合法JSON**
    /// * 📝兼顾性能的「验证某个字符串是否为JSON」方法：使用[「立即抛弃」](serde::de::IgnoredAny)类型
    ///   * 🔗<https://github.com/serde-rs/json/issues/579>
    ///   * ✅结合[`is_ok`](Result::is_ok)实现「语法正确⇒是回调消息」
    ///   * 🎯最终目的：精确锁定回调消息，防止范围扩大的**误报**
    /// * 🚩【2024-08-18 00:01:18】目前基于「带所有权split可行」重新启用「定制化格式」的方法
    ///   * ✅【2024-08-18 00:02:16】现不再需要检查JSON：有别的格式约束
    fn as_sav_callback(message: &str) -> Option<(&str, &str)> {
        // 元数据和数据分离
        let (meta, data) = message.split_once('\n')?;
        // 验证元数据：两头都有
        // serde_json::from_str::<serde::de::IgnoredAny>(data).is_ok()
        meta.trim()
            .strip_prefix(SAV_INFO_HEAD)
            .and_then(|stripped| stripped.strip_suffix(':'))
            .map(|path| (path, data))
    }

    /// 验证某条消息是否为有效回调，兼顾返回`(路径, 数据)`二元组
    /// * 📌[`as_sav_callback`]的带所有权版本
    ///   * 在所有权拆分下，可以避免不必要的拷贝开销
    fn as_sav_callback_owned(message: String) -> Result<(String, String), String> {
        // 元数据和数据分离
        let (meta, data) = message.split_owned_once('\n')?;
        // 验证元数据：两头都有
        // serde_json::from_str::<serde::de::IgnoredAny>(data).is_ok()
        let path = meta
            .trim()
            .strip_prefix(SAV_INFO_HEAD)
            .and_then(|stripped| stripped.strip_suffix(':'))
            .ok_or(format!("{meta}\n{data}"))?;
        Ok((path.to_owned(), data))
    }

    pub trait SavCallback: Sized {
        /// 基于类型、数据构造「SAV」callback消息
        /// * 📌对「数据」采取【传递所有权】的方式，避免太大的拷贝开销
        /// * 📌参数类型：`(路径)`
        ///   * 🚧【2024-08-18 01:01:11】后续或许考虑「目标、路径、数据三者兼备」
        fn format_sav_callback(path: impl Into<String>, data: impl Into<String>) -> Output {
            Output::INFO {
                message: format_sav_callback(path, data),
            }
        }

        /// 从一个NAVM输出中拿到「SAV」callback数据（引用）
        /// * 🎯提供易用的「数据保存」回调API
        fn as_sav_callback(&self) -> Option<(&str, &str)>;

        /// 从一个NAVM输出中拿到「SAV」callback数据（所有权）
        /// * 🎯提供易用的「数据保存」回调API
        /// * ℹ️可能回调中的数据较大，为避免大量数据拷贝，使用所有权转交避免复制
        /// * 📌返回结果：`(保存到的目标, 所保存的数据)`
        fn try_into_sav_callback(self) -> Result<(String, String), Self>;
    }

    impl SavCallback for Output {
        fn as_sav_callback(&self) -> Option<(&str, &str)> {
            use Output::*;
            match self {
                INFO { ref message } => as_sav_callback(message),
                // 其它均为否
                _ => None,
            }
        }
        fn try_into_sav_callback(self) -> Result<(String, String), Self> {
            use Output::*;
            match self {
                INFO { message } => {
                    as_sav_callback_owned(message).map_err(|message| INFO { message })
                }
                // 其它均返还原输出
                _ => Err(self),
            }
        }
    }

    #[cfg(test)]
    mod tests {
        use super::*;
        use nar_dev_utils::f_tensor;
        use navm::output::Operation;

        #[test]
        fn format_verify() {
            // 合法JSON
            f_tensor![
                format_verify_ok;
                "memory.json" "status.json";
                "" // ! 不检查数据侧的JSON合法性
                "[]"
                "{}"
            ];
            let callback_msg = || format_sav_callback("memory.json", "{}");
            // 非法JSON
            f_tensor![
                format_verify_err;
                // 其它类型不是
                Output::COMMENT { content: callback_msg() }
                Output::ANSWER { content_raw: callback_msg(), narsese: None }
                Output::OUT { content_raw: callback_msg(), narsese: None }
                Output::EXE { operation: Operation::new("", []), content_raw: callback_msg() }
                // 其它内容不是
                Output::INFO { message: "()".into() }
                // 多加一行也不是
                Output::INFO { message: "\n".to_owned() + &callback_msg() }
            ];
        }

        fn format_verify_ok(path: impl Into<String>, data: impl Into<String>) {
            let [path, data] = [path.into(), data.into()];
            let out = format_sav_callback(path, data);
            assert!(dbg!(as_sav_callback(&out)).is_some());
            assert!(dbg!(as_sav_callback_owned(out)).is_ok());
        }

        fn format_verify_err(out: impl Into<Output>) {
            let out = out.into();
            assert!(dbg!(out.as_sav_callback()).is_none());
            assert!(dbg!(out.clone().try_into_sav_callback()).is_err_and(|e| e == out));
            // ↑返还的输入与原输入等价
        }
    }
}
use navm::output::Output;
use public::*;

/// 将记忆区转换为JSON字符串
/// * ⚠️可能失败：记忆区数据可能无法被序列化
fn memory_to_json(reasoner: &Reasoner) -> anyhow::Result<String> {
    let mut writer = Vec::<u8>::new();
    let mut ser = serde_json::Serializer::new(&mut writer);
    reasoner.serialize_memory(&mut ser)?;
    let json = String::from_utf8(writer)?;
    Ok(json)
}

/// 将「推理状态」转换为JSON字符串
/// * ⚠️可能失败：记忆区数据可能无法被序列化
fn status_to_json(reasoner: &Reasoner) -> anyhow::Result<String> {
    let mut writer = Vec::<u8>::new();
    let mut ser = serde_json::Serializer::new(&mut writer);
    reasoner.serialize_status(&mut ser)?;
    let json = String::from_utf8(writer)?;
    Ok(json)
}
