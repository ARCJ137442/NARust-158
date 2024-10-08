//! 「词项」的结构体
//! * 🚩【2024-06-12 21:11:15】新迁入作为「定义」mod

/// 作为「结构」的词项
/// * 🚩更多通过「复合」而非「抽象特征-具体实现」复用代码
///   * 📍【2024-04-20 21:13:20】目前只需实现OpenNARS 1.5.8的东西
///
///  ! ⚠️【2024-04-20 21:47:08】暂不实现「变量 < 原子 < 复合」的逻辑
/// * 🎯OpenNARS中有关「词项顺序」的概念，目的是保证「无序不重复集合」的唯一性
///   * 🚩然而此实现的需求用「派生[`Ord`]」虽然造成逻辑不同，但可以满足需求
///   * 📌核心逻辑：实现需求就行，没必要（也很难）全盘照搬
/// * ⚠️[`Hash`]特征不能在手动实现的[`PartialEq`]中实现，否则会破坏「散列一致性」
///
/// * 📝OpenNARS在「记忆区构造词项」时，就会进行各种预处理
///   * 📄`<(-, {A, B}, {A}) --> x>` 会产生 `<{B} --> x>`（外延「差」规则）
/// * 📝OpenNARS中的词项基本只能通过`make`系列方法（从外部）构造
///   * 💭这似乎意味着它是一种「记忆区专用」的封闭数据类型
///
/// * 📌【2024-06-16 11:42:25】目前应手动实现[`Ord`]
///   * ⚠️在「重排唯一化」的需求场景下需要「变量之间均相等」
///     * 🚩目前要对「重排唯一化」做单独实现：[`Ord`]需要与[`PartialEq`]对齐
///   * 💭大多情况下不会用到「比大小」逻辑，场景如「排序」
///   * ⚠️不能破坏「比对为等」和「直接判等」的一致性：原先不比对`is_constant`字段，就已经破坏了这点
///
/// # 📄OpenNARS
///
/// Term is the basic component of Narsese, and the object of processing in NARS.
/// A Term may have an associated Concept containing relations with other Terms.
/// It is not linked in the Term, because a Concept may be forgot while the Term exists. Multiple objects may represent the same Term.
///
/// ## 作为特征的「实现」
///
/// ### Cloneable => [`Clone`]
///
/// Make a new Term with the same name.
///
/// ### equals => [`Eq`]
///
/// Equal terms have identical name, though not necessarily the same reference.
///
/// ### hashCode => [`Hash`]
///
/// Produce a hash code for the term
///
/// ### compareTo => [`Ord`]
///
/// Orders among terms: variable < atomic < compound
///
/// ### toString => [`Display`]
///
/// The same as getName by default, used in display only.
///
/// @return The name of the term as a String
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct Term {
    /// 标识符
    /// * 🎯决定词项的「类型」
    /// * 🚩使用不同词项类型独有的「标识符」
    ///   * 📄原子词项⇒原子词项前缀
    ///   * 📄复合词项⇒复合词项连接词
    ///   * 📄陈述⇒陈述系词
    /// * ❌【2024-04-21 00:57:39】不能使用「静态字串」固定
    ///   * ⚠️需要针对「用户输入」作一定妥协
    ///     * 此刻通过「词法折叠」等途径获得的「词项」就不一定是「静态引用」了
    ///   * 📌即便标识符的类型尽可能「固定」（就那么几种）
    pub(super) identifier: String,
    /// 组分
    /// * 🎯表示「词项名称」「词项包含词项」的功能
    /// * 🚩通过单一的「复合组分」实现「组合」功能
    pub(super) components: TermComponents,
    // 自由属性「是否为常量」
    // * 🎯用于决定其在记忆区、NAL-6推理中的行为
    // * ❓为何要设置成「结构属性」：会在系统构造「语句」时改变
    //   * 📝源自OpenNARS：构造语句时所直接涉及的词项均为「常量词项」，必须进入记忆区
    // * 📄OpenNARS `isConstant` 属性
    // * 📜默认为`true`
    // * 📌此属性影响到「语义判等」的行为
    // * ✅【2024-06-19 02:07:04】现已无用：在OpenNARS改版中验证了「运行时动态判断，不影响单步推理结果」
    // pub(in crate::language) is_constant: bool,
}

/// 复合词项组分
/// * ⚠️
#[derive(Debug, Clone, Hash, Eq, PartialEq, PartialOrd, Ord)]
pub enum TermComponents {
    /// 不包含任何组分
    /// * 📄占位符
    Empty,

    /// 仅包含一个字符串作为「名称」
    /// * 📄词语
    Word(String),

    /// 仅包含一个非负整数作为「变量」
    /// * 📄变量
    ///
    /// ## 有关「变量命名」的笔记
    ///
    /// * 📝在NAL中，变量词项的语义仅在单个词项之内有效
    ///   * 📄这两个词项是**语义等价**的：`<A --> $x>` 和 `<B --> $y>`
    ///   * 📄这两个词项同样是**语义等价**的：`(&&,<#1 --> lock>,<#2 --> key>)` 和 `(&&,<#1 --> key>,<#2 --> lock>)`
    /// * ✨具体原理：单个词项之内，变量名的改变不会影响其逻辑语义，只要名字不发生冲突（重命名后不和任一变量名相同）
    ///   * 📌这意味着：若需确定「带变量词项」的唯一性，就需要一套「自动重命名」机制来保证变量「语义相等」
    ///     * 📄此即：重命名后，可以直接通过「数据判等」实现「语义判等」逻辑
    Variable(usize),

    /// 一般复合词项
    /// * 📌一元、二元、多元词项
    /// * 🚩【2024-06-12 21:18:23】现在统一「一元复合词项」「二元复合词项」和「多元复合词项」
    ///   * 📌统一使用「构造后定长数组」实现，无需复杂match匹配（少两个分支）
    ///   * 📌统一「可交换」与「不可交换」两类词项（无需考虑复合词项数目）
    ///   * 📌使用堆分配的「构造后定长数组」规避「循环包含」问题
    ///
    /// 词项类型举例：
    /// * 📄乘积
    /// * 📄否定
    /// * 📄继承、蕴含
    /// * 🚩通过「构造时自动去重并排序」实现「集合无序性」
    ///   * 📄外延差、内涵差
    ///   * 📄外延集、内涵集
    ///   * 📄外延交、内涵交
    ///   * 📄合取、析取
    ///   * 📄相似、等价
    ///
    /// ## 【2024-06-12 22:12:25】有关「像」的结构实现 笔记
    /// * 📝实际上，「有索引」和「使用占位符」是两个相同的方案
    /// * 🎯这两个方案的核心目标皆为「在『外延像/内涵像』中取到『像占位符』的位置」
    /// * 📄差别仅在手段不同
    ///   * 📌「有索引」的方法：多加一个字段，特别标识「像占位符位置」以便快速索引
    ///   * 📌「使用占位符」的方法：允许「占位符」独立作为词项，在「获取像占位符位置」时依靠「索引像占位符」获取位置
    /// * 💭两个方案各有优劣
    ///   * ⚠️「有索引」的方法：以设计换性能——需要多添加字段，在不应「另创新类」的情况下徒增许多模式匹配需要
    ///   * ⚠️「使用占位符」的方法：以性能换设计——在每次「检索像占位符位置」均需要`O(内容词项数)`的时间复杂度
    /// * 🚩【2024-06-12 22:11:47】综合多方考量，最终确定（并唯一选择）第二种「使用占位符」的方案
    Compound(Box<[Term]>),
    // /// 多重组分（有序）+索引
    // /// * ❓【2024-04-20 21:57:35】日后需要通过「像」使用时，会造成「像-MultiIndexed」绑定
    // ///   * ⚡那时候若使用「断言」是否会导致不稳定
    // ///   * ❓若不使用「断言」而是音量失败，是否会增加排查难度
    // ///   * ❓若不使用「断言」而是发出警告，那是否会导致性能问题
    // /// * 🚩可行的解决方案：`match (self.identifier, self.components) { ('/', MultiIndexed(i, v))}`
    // ///
    // /// 词项类型举例：
    // /// * 📄外延像、内涵像
    // MultiIndexed(usize, Box<[Term]>),
}

/// 单元测试
/// * 🎯对结构体本身进行测试（仅结构字段、枚举变种）
/// * 🎯提供通用的测试用函数
#[cfg(test)]
pub(crate) mod test_term {
    use super::*;
    use nar_dev_utils::ResultBoost;

    /// 用于批量生成「解析后的词项」
    /// * 🚩使用`?`直接在解析处上抛错误
    #[macro_export]
    macro_rules! test_term {
        // 词项数组
        ([$($s:expr $(,)?)*]) => {
            [ $( term!($s) ),* ]
        };
        // 词项引用数组（一次性）
        ([$($s:expr $(,)?)*] &) => {
            [ $( &term!($s) ),* ]
        };
        // 单个词项（字符串），问号上抛
        ($s:literal) => {
            $s.parse::<Term>()?
        };
        // 单个词项（字符串），问号上抛
        (str $s:expr) => {
            $s.parse::<Term>()?
        };
        // 单个词项，但unwrap
        (unwrap $s:expr) => {
            $s.parse::<Term>().unwrap()
        };
        // 单个词项，问号上抛
        ($($t:tt)*) => {
            $crate::test_term!(str stringify!($($t)*))
        };
    }

    /// 快捷构造[`Option<Term>`](Option)
    #[macro_export]
    macro_rules! option_term {
        () => {
            None
        };
        (None) => {
            None
        };
        ($t:literal) => {
            parse_option_term($t)
        };
    }

    /// 用于封装作为`result`的方法
    /// * 🚩在解析失败时，打印错误信息并返回`None`
    ///   * 📌一般这时会有「预期比对」能触发失败
    pub fn parse_option_term(t: &str) -> Option<Term> {
        t.parse::<Term>()
            .ok_or_run(|e| eprintln!("!!! 词项 {t:?} 解析失败：{e}"))
    }

    /// 快捷格式化[`Option<Term>`](Option)
    pub fn format_option_term(ot: &Option<Term>) -> String {
        match ot {
            Some(t) => format!("Some(\"{t}\")"),
            None => "None".to_string(),
        }
    }
}
