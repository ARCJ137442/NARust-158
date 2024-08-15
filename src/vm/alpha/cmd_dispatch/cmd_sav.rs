use crate::control::Reasoner;
use nar_dev_utils::{macro_once, ResultS};
use navm::output::Output;

/// 指令[`Cmd::SAV`]的入口函数
/// * 📌传入的`query`默认为小写字串引用
/// * 📌输出仅为JSON字符串；若返回[错误值](Err)，则视为「报错」
pub fn sav_dispatch(
    reasoner: &mut Reasoner,
    query: impl AsRef<str>,
    _path: impl AsRef<str>,
) -> ResultS<String> {
    macro_once! {
        macro ( $( $query:literal => $message:expr )* ) => {
            /// 所有非空查询的列表
            /// * 📌格式：Markdown无序列表
            const ALL_QUERIES_LIST: &str = concat!($( "\n- ", $query, )*);
            match query.as_ref() {
                // * 🚩特殊/空字串：列举所有query并转接`HLP INF`
                // ! ⚠️【2024-08-09 17:48:15】不能放外边：会被列入非空查询列表中
                "" => Ok(format!("Available save target: {ALL_QUERIES_LIST}",)),
                // 所有固定模式的分派
                $( $query => Ok($message.to_string()), )*
                // * 🚩其它⇒告警
                other => Err(format!("Unknown save target: {other:?}")),
            }
        }

        // 记忆区
        "memory" => format_sav(reasoner, query.as_ref(), memory_to_json)?
        // 推理器整体状态
        "status" => format_sav(reasoner, query.as_ref(), status_to_json)?
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
fn format_sav(
    reasoner: &Reasoner,
    target: &str,
    to_json: fn(&Reasoner) -> anyhow::Result<String>,
) -> ResultS<String> {
    let data = to_json(reasoner).map_err(|e| format!("Failed to serialize {target}: {e}"))?;
    let message = Output::format_sav_callback(target, data);
    Ok(message)
}

/// 便于外部调用的API
pub mod public {
    use navm::output::Output;

    // ! ❌【2024-08-15 17:57:18】禁用：不使用「前缀标识」
    // /// 在[`SAV`](Cmd::SAV)指令调用后，推理器输出的消息头
    // /// * 🎯用于区分「一般消息」与「SAV回调」
    // const SAV_INFO_HEAD: &str = "SAV";

    pub trait SavCallback: Sized {
        /// 基于类型、数据构造「SAV」callback消息
        /// * 📌对「数据」采取【传递所有权】的方式，避免太大的拷贝开销
        fn format_sav_callback(_target: &str, data: String) -> String {
            // format!("{SAV_INFO_HEAD} {target}:\n{data}")
            data // * 🚩【2024-08-15 18:01:45】目前直接返回
        }

        /// 从一个NAVM输出中拿到「SAV」callback数据（引用）
        /// * 🎯提供易用的「数据保存」回调API
        fn as_sav_callback(&self) -> Option<&str>;

        /// 从一个NAVM输出中拿到「SAV」callback数据（所有权）
        /// * 🎯提供易用的「数据保存」回调API
        /// * ℹ️可能回调中的数据较大，为避免大量数据拷贝，使用所有权转交避免复制
        fn try_into_sav_callback(self) -> Result<String, Self>;
    }
    impl SavCallback for Output {
        fn as_sav_callback(&self) -> Option<&str> {
            use Output::*;
            match self {
                INFO { ref message } if verify_sav_callback(message) => {
                    // let (_, data) = message.split_once('\n')?;
                    let data = message.as_str();
                    Some(data)
                }
                // 其它均为否
                _ => None,
            }
        }
        fn try_into_sav_callback(self) -> Result<String, Self> {
            use Output::*;
            match self {
                INFO { message } if verify_sav_callback(&message) => {
                    // 💭【2024-08-15 17:54:52】理论上是可以做到「传递所有权的拆分」，但标准库不提供，也需要unsafe代码
                    // * 🤔会下放到更底层的u8数组去
                    // * ⚠️【2024-08-15 17:55:41】出于成本考虑，目前暂不这样做，而是使用「完整消息+识别JSON」的方案
                    // message.message.split_off(at);
                    Ok(message)
                }
                // 其它均返还原输出
                _ => Err(self),
            }
        }
    }

    fn verify_sav_callback(message: &str) -> bool {
        // JSON结构体
        (message.starts_with('{') && message.ends_with('}'))
        // JSON数组
        || (message.starts_with('[') && message.ends_with(']'))
    }
}
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
