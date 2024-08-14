use crate::control::Reasoner;
use nar_dev_utils::macro_once;

/// 指令[`Cmd::SAV`]的入口函数
/// * 📌传入的`query`默认为小写字串引用
/// * 📌输出仅为JSON字符串；若返回[错误值](Err)，则视为「报错」
pub fn sav_dispatch(
    reasoner: &mut Reasoner,
    query: impl AsRef<str>,
    _path: impl AsRef<str>,
) -> Result<String, String> {
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
        "memory" => memory_to_json(reasoner)
            .map_err(|e| format!("Failed to serialize memory: {e}"))?
        // 推理器整体状态
        "status" => status_to_json(reasoner)
            .map_err(|e| format!("Failed to serialize status: {e}"))?
    }
}

/// 将记忆区转换为JSON字符串
/// * ⚠️可能失败：记忆区数据可能无法被序列化
pub fn memory_to_json(reasoner: &Reasoner) -> anyhow::Result<String> {
    let mut writer = Vec::<u8>::new();
    let mut ser = serde_json::Serializer::new(&mut writer);
    reasoner.serialize_memory(&mut ser)?;
    let json = String::from_utf8(writer)?;
    Ok(json)
}

/// 将「推理状态」转换为JSON字符串
/// * ⚠️可能失败：记忆区数据可能无法被序列化
pub fn status_to_json(reasoner: &Reasoner) -> anyhow::Result<String> {
    let mut writer = Vec::<u8>::new();
    let mut ser = serde_json::Serializer::new(&mut writer);
    reasoner.serialize_status(&mut ser)?;
    let json = String::from_utf8(writer)?;
    Ok(json)
}
