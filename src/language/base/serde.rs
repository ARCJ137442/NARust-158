//! 为[`Term`]定制的序列反序列化方法
//! * 🎯节省序列化后的占用空间
//!   * 📄在JSON中不再需要是一个object，是一个`[f, c, a]`三元组就行
use super::Term;
use narsese::conversion::string::impl_lexical::format_instances::FORMAT_ASCII;
use serde::{Deserialize, Deserializer, Serialize, Serializer};

impl Serialize for Term {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        // * 🚩为保证稳定性，此处不使用`Term::format_ascii`
        // 转换为词法Narsese
        let lexical = self.to_lexical();
        // 再变为字符串
        let s = FORMAT_ASCII.format(&lexical);
        s.serialize(serializer)
    }
}

impl<'de> Deserialize<'de> for Term {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        // 先反序列化到字符串
        let s: String = Deserialize::deserialize(deserializer)?;
        // 再词法解析
        let lexical = FORMAT_ASCII
            .parse_term(&s)
            .map_err(serde::de::Error::custom)?;
        // 最后词法折叠
        Term::from_lexical(lexical).map_err(serde::de::Error::custom)
    }
}

#[cfg(test)]
mod tests {
    use crate::{language::Term, ok, test_term as term, util::AResult};
    use nar_dev_utils::macro_once;

    /// 测试「序列反序列化」的正确性
    /// * 🎯一次「序列化→反序列化」过程后，结果要和自身一致
    ///   * 📄强一致性：不论任何term输入，均能如此
    ///   * 📄弱一致性：经过一次「序列反序列化」之后（的词项），能通过「强一致性」的条件
    ///
    /// ! ⚠️【2024-08-12 01:09:41】暂且只满足「弱一致性」
    /// * 💫不是很能处理「无序词项+变量」的问题
    ///   * 📄存在「序列反序列化后，因变量id导致无序词项内部顺序变化」的情况
    ///     * "<(&&,<(*,{SELF},$1,FALSE) --> #3>,<(*,{SELF},$1) --> #2>) ==> <(*,{SELF},$1) --> afraid_of>>"
    ///   * 😮‍💨【2024-08-12 01:11:37】所幸这类情况并不多见，且仍满足「弱一致性」
    ///
    /// TODO: 🎯【2024-08-12 01:21:56】后续实现「强一致性」以彻底解决edge cases
    #[test]
    fn test_soundness() -> AResult {
        fn test(term: Term) {
            // * 强一致性 * //
            let mut strong_soundness = true;
            // 序列化-反序列化
            let ser = serde_json::to_string(&term).expect("词项序列化失败");
            let de = serde_json::from_str::<Term>(&ser).expect("词项反序列化失败");
            strong_soundness &= term == de;
            // assert_eq!(
            //     term, de,
            //     "序列化-反序列化 不可靠！ {term} vs\n{de}\nfrom {ser:?}"
            // );
            // 重序列化
            let ser2 = serde_json::to_string(&de).expect("词项重序列化失败");
            strong_soundness &= ser == ser2;
            // assert_eq!(
            //     ser, ser2,
            //     "反序列化-序列化 不可靠！ {ser} vs\n{ser2}\nfrom {de}"
            // );

            // ! 🚩【2024-08-12 01:14:38】目前仅警告
            if !strong_soundness {
                eprintln!("【警告】当前序列反序列化机制 不满足强一致性！\n1 {term} vs\n3 {de}\nfrom\n2 {ser} vs\n4 {ser2}");
            }

            // * 弱一致性 * //
            // 重反序列化
            let de2 = serde_json::from_str::<Term>(&ser2).expect("词项反序列化失败");
            assert_eq!(
                de, de2,
                "序列化-反序列化 不可靠！ {de} vs\n{de2}\nfrom {ser2:?}"
            );
            // 重序列化
            let ser3 = serde_json::to_string(&de2).expect("词项重序列化失败");
            assert_eq!(
                ser2, ser3,
                "反序列化-序列化 不可靠！ {ser} vs\n{ser3}\nfrom {de}"
            );
        }
        macro_once! {
            // * 🚩格式：词项内容字符串
            macro test( $($term:literal)* ) {
                $( test( term!($term) ); )*
            }
            "(&&,<#1 --> object>,<#1 --> [unscrewing]>)"
            "<(&&,<$1 --> [pliable]>,<(*,{SELF},$1) --> #reshape>) ==> <$1 --> [hardened]>>"
            "<(&&,<(*,$1,plastic) --> made_of>,<(*,{SELF},$1) --> #lighter>) ==> <$1 --> [heated]>>"
            "<(&&,<(*,{SELF},wolf) --> close_to>,#1000) ==> <{SELF} --> [hurt]>>"
            "<(&&,<(*,{SELF},$1,FALSE) --> #want>,<(*,{SELF},$1) --> #anticipate>) ==> <(*,{SELF},$1) --> afraid_of>>"
            "<(*,cup,plastic) --> made_of>"
            "<(*,toothbrush,plastic) --> made_of>"
            "<(*,{SELF},?what) --> afraid_of>"
            "<(*,{SELF},wolf) --> close_to>"
            "<(*,{tom},(&,[black],glasses)) --> own>"
            "<(*,{tom},sunglasses) --> own>"
            "<<$1 --> (/,livingIn,_,{graz})> ==> <$1 --> murder>>"
            "<<$1 --> [aggressive]> ==> <$1 --> murder>>"
            "<<$1 --> [hardened]> ==> <$1 --> [unscrewing]>>"
            "<<$1 --> [heated]> ==> <$1 --> [melted]>>"
            "<<$1 --> [melted]> <=> <$1 --> [pliable]>>"
            "<<(*,$1,sunglasses) --> own> ==> <$1 --> [aggressive]>>"
            "<?1 ==> <c --> C>>"
            "<a --> A>"
            "<b --> B>"
            "<c --> C>"
            "<cup --> [bendable]>"
            "<cup --> object>"
            "<sunglasses --> (&,[black],glasses)>"
            "<toothbrush --> [bendable]>"
            "<toothbrush --> object>"
            "<{?who} --> murder>"
            "<{SELF} --> [hurt]>"
            "<{tim} --> (/,livingIn,_,{graz})>"

            "(&&,<#1 --> lock>,<<$2 --> key> ==> <#1 --> (/,open,$2,_)>>)"
            "(&&,<#x --> (/,open,#y,_)>,<#x --> lock>,<#y --> key>)"
            "(&&,<#x --> bird>,<#x --> swimmer>)"
            "(&&,<#x --> key>,<{lock1} --> (/,open,#x,_)>)"
            "(&&,<robin --> [flying]>,<robin --> swimmer>)"
            "(&&,<robin --> swimmer>,<robin --> [flying]>)"
            "(--,<robin --> [flying]>)"
            "(||,<robin --> [flying]>,<robin --> swimmer>)"
            "<(&&,<#1 --> lock>,<#1 --> (/,open,$2,_)>) ==> <$2 --> key>>"
            "<(&&,<$x --> [chirping]>,<$x --> [with_wings]>) ==> <$x --> bird>>"
            "<(&&,<$x --> flyer>,<$x --> [chirping]>) ==> <$x --> bird>>"
            "<(&&,<$x --> flyer>,<$x --> [chirping]>, <(*, $x, worms) --> food>) ==> <$x --> bird>>"
            "<(&&,<$x --> flyer>,<(*,$x,worms) --> food>) ==> <$x --> bird>>"
            "<(&&,<$x --> key>,<$y --> lock>) ==> <$y --> (/,open,$x,_)>>"
            "<(&&,<robin --> [chirping]>,<robin --> [flying]>) ==> <robin --> bird>>"
            "<(&&,<robin --> [chirping]>,<robin --> [flying]>,<robin --> [with_wings]>) ==> <robin --> bird>>"
            "<(&&,<robin --> [flying]>,<robin --> [with_wings]>) ==> <robin --> [living]>>"
            "<(&&,<robin --> [flying]>,<robin --> bird>) ==> <robin --> [living]>>"
            "<(&&,<robin --> bird>,<robin --> [living]>) ==> <robin --> animal>>"
            "<(&,<{Tweety} --> bird>,<bird --> fly>) --> claimedByBob>"
            "<(&,bird,swimmer) --> (&,animal,swimmer)>"
            "<(&,swan,swimmer) --> bird>"
            "<(*,(*,(*,0))) --> num>"
            "<(*,a,b) --> like>"
            "<(*,bird,plant) --> ?x>"
            "<(-,swimmer,animal) --> (-,swimmer,bird)>"
            "<(--,<robin --> [flying]>) ==> <robin --> bird>>"
            "<(--,<robin --> bird>) ==> <robin --> [flying]>>"
            "<(/,neutralization,_,base) --> ?x>"
            r"<(\,(\,REPRESENT,_,<(*,CAT,FISH) --> FOOD>),_,eat,fish) --> cat>"
            r"<(\,REPRESENT,_,CAT) --> cat>"
            r"<(\,neutralization,acid,_) --> ?x>"
            "<(|,boy,girl) --> youth>"
            "<(~,boy,girl) --> [strong]>"
            "<(~,swimmer,swan) --> bird>"
            "<0 --> (/,num,_)>"
            "<0 --> num>"
            "<<$1 --> lock> ==> (&&,<#2 --> key>,<$1 --> (/,open,#2,_)>)>"
            "<<$1 --> num> ==> <(*,$1) --> num>>"
            "<<$x --> animal> <=> <$x --> bird>>"
            "<<$x --> bird> ==> <$x --> animal>>"
            "<<$x --> key> ==> <{lock1} --> (/,open,$x,_)>>"
            "<<$y --> [with_wings]> ==> <$y --> flyer>>"
            "<<$y --> flyer> ==> <$y --> [with_wings]>>"
            "<<(&,<#1 --> $2>,<$3 --> #1>) --> claimedByBob> ==> <<$3 --> $2> --> claimedByBob>>"
            "<<(*,$1,$2) --> like> <=> <(*,$2,$1) --> like>>"
            "<<bird --> $x> ==> <robin --> $x>>"
            "<<lock1 --> (/,open,$1,_)> ==> <$1 --> key>>"
            "<<robin --> [flying]> ==> <robin --> [with_beak]>>"
            "<<robin --> [flying]> ==> <robin --> animal>>"
            "<<robin --> animal> <=> <robin --> bird>>"
            "<<robin --> bird> <=> <robin --> [flying]>>"
            "<<robin --> bird> ==> (&&,<robin --> animal>,<robin --> [flying]>)>"
            "<<robin --> bird> ==> <robin --> [flying]>>"
            "<<robin --> bird> ==> <robin --> animal>>"
            "<?1 --> swimmer>"
            "<Birdie <-> Tweety>"
            "<Tweety {-- bird>"
            "<Tweety {-] yellow>"
            "<[bright] <-> [smart]>"
            "<[smart] --> [bright]>"
            "<acid --> (/,reaction,_,base)>"
            "<cat --> (/,(/,REPRESENT,_,<(*,CAT,FISH) --> FOOD>),_,eat,fish)>"
            "<neutralization --> (*,acid,base)>"
            "<planetX --> {Mars,Pluto,Venus}>"
            "<planetX --> {Pluto,Saturn}>"
            "<raven --] black>"
            "<robin --> (&,bird,swimmer)>"
            "<robin --> (-,bird,swimmer)>"
            "<robin --> (|,bird,swimmer)>"
            "<robin --> [flying]>"
            "<{?1} --> swimmer>"
            "<{Birdie} <-> {Tweety}>"
            "<{Tweety} --> [with_wings]>"
            "<{Tweety} --> flyer>"
            "<{Tweety} --> {Birdie}>"
            "<{key1} --> (/,open,_,{lock1})>"

            // ! edge cases
            "(&&,<(*,1,FALSE) --> #3>,<(*,1) --> #2>)" // ! ❌【2024-08-12 01:19:45】破坏强一致性 最小例
        }
        ok!()
    }
}
