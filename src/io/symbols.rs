//! 字符串符号
//! * 📄nars.io.Symbols
//! * 🎯用于表示词项关键词、展示用前后缀、变量类型、数值分隔符等

// 语句标点
// * 🚩【2024-05-13 09:38:31】改：统一为字符串（需要判等）
pub const JUDGMENT_MARK: &str = ".";
pub const QUESTION_MARK: &str = "?";

/// 🆕词语
/// * 🚩【2024-04-20 21:53:47】使用空字串作为「词语」的（类型）标识符
pub const WORD: &str = "";

/// 🆕占位符
/// * 🚩【2024-04-21 00:35:50】适应「词法Narsese」
pub const PLACEHOLDER: &str = "_";

// 变量类型
// * 🚩【2024-04-20 20:12:43】改：统一为字符串
pub const VAR_INDEPENDENT: &str = "$";
pub const VAR_DEPENDENT: &str = "#";
pub const VAR_QUERY: &str = "?";

// 数值分隔符，必须与「词项分隔符」相异
// * 🚩【2024-05-09 00:56:34】改：统一为字符串
pub const BUDGET_VALUE_MARK: &str = "$";
pub const TRUTH_VALUE_MARK: &str = "%";
pub const VALUE_SEPARATOR: &str = ";";

// 复合词项括弧
pub const COMPOUND_TERM_OPENER: char = '(';
pub const COMPOUND_TERM_CLOSER: char = ')';
pub const STATEMENT_OPENER: char = '<';
pub const STATEMENT_CLOSER: char = '>';
pub const SET_EXT_OPENER: char = '{';
pub const SET_EXT_CLOSER: char = '}';
pub const SET_INT_OPENER: char = '[';
pub const SET_INT_CLOSER: char = ']';

// 参数列表中的特殊字符
pub const ARGUMENT_SEPARATOR: char = ',';
pub const IMAGE_PLACE_HOLDER: char = '_';

// 复合词项连接词，长度为1
pub const INTERSECTION_EXT_OPERATOR: &str = "&";
pub const INTERSECTION_INT_OPERATOR: &str = "|";
pub const DIFFERENCE_EXT_OPERATOR: &str = "-";
pub const DIFFERENCE_INT_OPERATOR: &str = "~";
pub const PRODUCT_OPERATOR: &str = "*";
pub const IMAGE_EXT_OPERATOR: &str = r"/";
pub const IMAGE_INT_OPERATOR: &str = r"\";

// 复合词项连接词，长度为2
pub const SET_EXT_OPERATOR: &str = "{}"; // 🆕统一到「复合词项」中去，不在语法中搞特殊
pub const SET_INT_OPERATOR: &str = "[]"; // 🆕统一到「复合词项」中去，不在语法中搞特殊
pub const NEGATION_OPERATOR: &str = "--";
pub const DISJUNCTION_OPERATOR: &str = "||";
pub const CONJUNCTION_OPERATOR: &str = "&&";

// 陈述系词，长度为3
pub const INHERITANCE_RELATION: &str = "-->";
pub const SIMILARITY_RELATION: &str = "<->";
pub const INSTANCE_RELATION: &str = "{--";
pub const PROPERTY_RELATION: &str = "--]";
pub const INSTANCE_PROPERTY_RELATION: &str = "{-]";
pub const IMPLICATION_RELATION: &str = "==>";
pub const EQUIVALENCE_RELATION: &str = "<=>";

// 「经验行」前缀
pub const INPUT_LINE: &str = "IN";
pub const OUTPUT_LINE: &str = "OUT";
pub const PREFIX_MARK: char = ':';
pub const RESET_MARK: char = '*';
pub const COMMENT_MARK: char = '/';

// 时间戳 | 展示用
// * 🚩【2024-05-09 00:56:34】改：统一为字符串
pub const STAMP_OPENER: &str = "{";
pub const STAMP_CLOSER: &str = "}";
pub const STAMP_SEPARATOR: &str = ";";
pub const STAMP_STARTER: &str = ":";

// 词项链类型 | 展示用
pub const TO_COMPONENT_1: &str = " @(";
pub const TO_COMPONENT_2: &str = ")_ ";
pub const TO_COMPOUND_1: &str = " _@(";
pub const TO_COMPOUND_2: &str = ") ";
