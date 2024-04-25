//! 表征NARust 158所用的「词项」
//! * 📄功能上参照OpenNARS
//! * 🚩实现方式上更Rusty，同时亦有其它妥协/加强
//! * ❓【2024-04-20 22:00:44】「统一结构体+用『可选字段』实现多态」的方法，会导致「性能臃肿」问题
//!   * ❗此举需要提前考虑「所有类型词项的所有功能」，并且要做到最大程度兼容
//!   * 📌即便使用「作为枚举的专用字段」也会因为「要适应某种复合词项类型」而导致让步
//!     * 而这种「只会在某个类型上产生让步」的方法，会导致「本该耦合而未耦合」的情形
//!     * 这种「看似通用，实则仍需『专用情况专用对待』」的方法，不利于后续维护
//!   * ❓【2024-04-20 23:53:15】或许也可行：是否可以`match (self.identifier, &*self.components)`
//! * 🚩【2024-04-20 22:05:09】目前将此方案搁置
//!   * ⇒尝试探索「直接基于『枚举Narsese』」的方法
//! * 🚩【2024-04-25 08:36:07】在`term_v3`、`term_v4`相继失败后，重启该方法
//!   * 📌通过「限制构造函数」+「只处理特定词项模式」的方法，基本解决堵点

use crate::io::symbols::*;
use nar_dev_utils::manipulate;

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
/// TODO: 🏗️【2024-04-24 15:43:32】`make`系列方法在推理规则中的实现
///
/// * 📝OpenNARS在「记忆区构造词项」时，就会进行各种预处理
///   * 📄`<(-, {A, B}, {A}) --> x>` 会产生 `<{B} --> x>`（外延「差」规则）
/// ? 📝OpenNARS中的词项基本只能通过`make`系列方法（从外部）构造
///   * 💭这似乎意味着它是一种「记忆区专用」的封闭数据类型
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
#[derive(Debug, Clone, Eq, PartialOrd, Ord)]
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
    identifier: String,

    /// 组分
    /// * 🎯表示「词项包含词项」的功能
    /// * 🚩通过单一的「复合组分」实现「组合」功能
    /// * 🚩此处加上[`Box`]，便不会造成「循环包含」
    components: Box<TermComponents>,

    /// 自由属性「是否为常量」
    /// * 🎯用于决定其在记忆区、NAL-6推理中的行为
    /// * ❓为何要设置成「结构属性」：会在系统构造「语句」时改变
    ///   * 📝源自OpenNARS：构造语句时所直接涉及的词项均为「常量词项」，必须进入记忆区
    /// * 📄OpenNARS `isConstant` 属性
    /// * 📜默认为`true`
    /// * 📌此属性影响到「语义判等」的行为
    is_constant: bool,
}

/// 复合词项组分
#[derive(Debug, Clone, Hash, Eq, PartialEq, PartialOrd, Ord)]
pub enum TermComponents {
    /// 不包含任何组分
    /// * 📄占位符
    Empty,

    /// 仅包含一个字符串作为「名称」
    /// * 📄词语，变量
    Named(String),

    /// 单一组分
    /// * 📄否定
    Unary(Term),

    /// 双重组分（有序）
    /// * 📄外延差、内涵差
    /// * 📄继承、蕴含
    /// * 🚩通过「构造时自动去重并排序」实现「集合无序性」
    ///   * 📄相似、等价
    Binary(Term, Term),

    /// 多重组分
    /// * 📄乘积
    /// * 🚩通过「构造时自动去重并排序」实现「集合无序性」
    ///   * 📄外延集、内涵集
    ///   * 📄外延交、内涵交
    ///   * 📄合取、析取
    Multi(Vec<Term>),

    /// 多重组分（有序）+索引
    /// * 📄外延像、内涵像
    /// * ❓【2024-04-20 21:57:35】日后需要通过「像」使用时，会造成「像-MultiIndexed」绑定
    ///   * ⚡那时候若使用「断言」是否会导致不稳定
    ///   * ❓若不使用「断言」而是静默失败，是否会增加排查难度
    ///   * ❓若不使用「断言」而是发出警告，那是否会导致性能问题
    /// * 🚩可行的解决方案：`match (self.identifier, self.components) { ('/', MultiIndexed(i, v))}`
    MultiIndexed(usize, Vec<Term>),
}

// 实现 / 构造
mod construct;

// 与其它类型相互转换
mod conversion;

// 实现 / 属性（内建）
mod property;

// 📄OpenNARS `nars.language.Term`
mod term;

// 📄OpenNARS `nars.language.CompoundTerm`
mod compound;

// 📄OpenNARS `nars.language.Variable`
pub mod variable;

// 📄OpenNARS `nars.language.Statement`
mod statement;

// 📄OpenNARS `nars.language.ImageXXt`
mod image;

/// 单元测试
/// * 🎯对结构体本身进行测试
/// * 提供通用的测试用函数
#[cfg(test)]
pub(super) mod test {
    use super::*;
    use anyhow::Result;

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
        // 单个词项
        ($s:expr) => {
            $s.parse::<Term>()?
        };
        // 单个词项
        (unwrap $s:expr) => {
            $s.parse::<Term>().unwrap()
        };
    }
    use crate::test_term as term;

    /// 测试 / [`TermComponents`]
    mod components {
        use super::*;
        use nar_dev_utils::asserts;

        /// 测试/长度
        #[test]
        fn len() -> Result<()> {
            macro_rules! len {
                ($s:expr) => {
                    term!($s).components.len()
                };
            }
            asserts! {
                // 平常情况
                len!("B") => 0
                len!("?quine") => 0
                len!("<A --> B>") => 2
                len!("(*, {SELF}, x, y)") => 3
                len!("(--, [good])") => 1
                // 像：占位符不算
                len!("(/, A, _, B)") => 2
                // 集合：缩并
                len!("[2, 1, 0, 0, 1, 2]") => 3
            }
            Ok(())
        }

        /// 测试/判空
        #[test]
        fn is_empty() -> Result<()> {
            macro_rules! is_empty {
                ($s:expr) => {
                    term!($s).components.is_empty()
                };
            }
            asserts! {
                is_empty!("B") => true
                is_empty!("?quine") => true
                is_empty!("<A --> B>") => false
                is_empty!("(*, {SELF}, x, y)") => false
                is_empty!("(--, [good])") => false
                is_empty!("(/, A, _, B)") => false
                is_empty!("[2, 1, 0, 0, 1, 2]") => false
            }
            Ok(())
        }

        /// 测试/获取
        #[test]
        fn get() -> Result<()> {
            macro_rules! get {
                ($s:expr, $i:expr) => {
                    term!($s).components.get($i)
                };
            }
            asserts! {
                // 平常情况
                get!("B", 0) => None
                get!("?quine", 0) => None
                get!("<A --> B>", 0) => Some(&"A".parse()?)
                get!("<A --> B>", 1) => Some(&"B".parse()?)
                get!("<A --> B>", 2) => None
                get!("{SELF}", 0) => Some(&"SELF".parse()?)
                get!("{SELF}", 1) => None
                get!("(*, {SELF}, x, y)", 0) => Some(&"{SELF}".parse()?)
                get!("(*, {SELF}, x, y)", 1) => Some(&"x".parse()?)
                get!("(*, {SELF}, x, y)", 2) => Some(&"y".parse()?)
                get!("(*, {SELF}, x, y)", 3) => None
                get!("(--, [good])", 0) => Some(&"[good]".parse()?)
                get!("(--, [good])", 1) => None
                // 像：占位符不算
                get!("(/, A, _, B)", 0) => Some(&"A".parse()?)
                get!("(/, A, _, B)", 1) => Some(&"B".parse()?)
                get!("(/, A, _, B)", 2) => None
                // 集合：排序 & 缩并
                get!("[2, 1, 0, 0, 1, 2]", 0) => Some(&"0".parse()?)
                get!("[2, 1, 0, 0, 1, 2]", 1) => Some(&"1".parse()?)
                get!("[2, 1, 0, 0, 1, 2]", 2) => Some(&"2".parse()?)
                get!("[2, 1, 0, 0, 1, 2]", 3) => None
            }
            Ok(())
        }

        /// 测试/获取
        #[test]
        fn get_unchecked() -> Result<()> {
            macro_rules! get_unchecked {
                ($s:expr, $i:expr) => {
                    unsafe { $s.parse::<Term>()?.components.get_unchecked($i) }
                };
            }
            asserts! {
                // 平常情况
                get_unchecked!("<A --> B>", 0) => &term!("A")
                get_unchecked!("<A --> B>", 1) => &term!("B")
                get_unchecked!("{SELF}", 0) => &term!("SELF")
                get_unchecked!("(*, {SELF}, x, y)", 0) => &term!("{SELF}")
                get_unchecked!("(*, {SELF}, x, y)", 1) => &term!("x")
                get_unchecked!("(*, {SELF}, x, y)", 2) => &term!("y")
                get_unchecked!("(--, [good])", 0) => &term!("[good]")
                // 像：占位符不算
                get_unchecked!("(/, A, _, B)", 0) => &term!("A")
                get_unchecked!("(/, A, _, B)", 1) => &term!("B")
                // 集合：排序 & 缩并
                get_unchecked!("[2, 1, 0, 0, 1, 2]", 0) => &term!("0")
                get_unchecked!("[2, 1, 0, 0, 1, 2]", 1) => &term!("1")
                get_unchecked!("[2, 1, 0, 0, 1, 2]", 2) => &term!("2")
            }
            Ok(())
        }

        /// 测试/迭代器
        /// * 🚩转换为数组，然后跟数组比对
        #[test]
        fn iter() -> Result<()> {
            macro_rules! iter {
                ($s:expr) => {
                    term!($s).components.iter().collect::<Vec<_>>()
                };
            }
            asserts! {
                iter!("<A --> B>") => term!(["A", "B"]&)
                // 平常情况
                iter!("{SELF}") => term!(["SELF"]&)
                iter!("(*, {SELF}, x, y)") => term!(["{SELF}", "x", "y"]&)
                iter!("(--, [good])") => term!(["[good]"]&)
                // 像：占位符不算
                iter!("(/, A, _, B)") => term!(["A", "B"]&)
                // 集合：排序 & 缩并
                iter!("[2, 1, 0, 0, 1, 2]") => term!(["0", "1", "2"]&)
            }
            Ok(())
        }

        // TODO: 更多函数的测试
    }
}
