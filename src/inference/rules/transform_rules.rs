//! 转换推理/转换规则

use crate::{
    control::{
        ContextDerivationConcept, ReasonContext, ReasonContextTransform, ReasonContextWithLinks,
        ReasonDirection,
    },
    entity::{Sentence, TLink, TruthValue},
    inference::BudgetInferenceContext,
    io::symbols::*,
    language::{CompoundTermRef, StatementRef, Term},
    util::RefCount,
};
use nar_dev_utils::unwrap_or_return;

/// 推理引擎「转换推理」的唯一入口
/// * 📝【2024-05-20 11:46:32】在「直接推理」之后、「概念推理」之前使用
///
/// # 📄OpenNARS
///
/// The TaskLink is of type TRANSFORM,
/// and the conclusion is an equivalent transformation
pub fn transform_task(context: &mut ReasonContextTransform) {
    // * 🚩预处理 | 📌【2024-06-07 23:12:34】断定其中的「tLink」就是「当前任务链」
    let t_link = context.current_task_link();
    let task_rc = t_link.target_rc();
    let task = task_rc.get_();
    debug_assert!(
        task.content().is_compound(),
        "// ! 此处必须假定其为复合词项：转换规则的任务均为复合词项"
    );
    let task_content = unwrap_or_return! {
        ?task.content().as_compound()
    };
    let indexes = t_link.indexes();

    // * 🚩获取有待转换的「继承」陈述（引用）
    let inh = unwrap_or_return! {
        ?unwrap_or_return! {
            ?get_inheritance_to_be_transform(
                task_content,
                indexes
            )
        // * 🚩提取出了继承项⇒开始转换
        // * 🚩【2024-07-03 11:35:40】修改：传入时复制
        }.as_statement_type(INHERITANCE_RELATION)
    };
    // * 🚩拷贝词项以隔离修改
    let old_content = task_content.inner.clone();
    let inheritance_to_be_transform = inh.statement.clone();
    drop(task);
    drop(task_rc);

    // * 🚩预先分派 @ 转换的是整体
    match old_content == inheritance_to_be_transform {
        // * 🚩待转换词项为整体（自身）⇒特别分派（可能转换多次）
        true => {
            // * 🚩需要重新拿到「陈述引用」
            let inh = unwrap_or_return!(?inheritance_to_be_transform.as_statement());
            let inh_subject = inh.subject;
            let inh_predicate = inh.predicate;
            // * 🚩转换前项
            if let Some(inh_subject) = inh_subject.as_compound() {
                transform_subject_product_image(inh_subject, inh_predicate, context);
            }
            // * 🚩转换后项
            if let Some(inh_predicate) = inh_predicate.as_compound() {
                transform_predicate_product_image(inh_subject, inh_predicate, context);
            }
        }
        // * 🚩复合词项⇒尝试转换内部的「继承」系词
        // * 📌【2024-07-05 18:22:05】此处不传递indexes，避免借用冲突
        false => {
            if let Some(old_content) = old_content.as_compound() {
                transform_product_image(
                    inheritance_to_be_transform
                        .as_statement()
                        .expect("此处一定是陈述"),
                    old_content,
                    context,
                );
            }
        }
    }
}

/// 🆕获取【需要参与后续「转换」操作】的「继承」陈述
fn get_inheritance_to_be_transform<'t>(
    task_content: CompoundTermRef<'t>,
    indexes: &[usize],
) -> Option<&'t Term> {
    match indexes.len() {
        // * 🚩本身是乘积 | <(*, term, #) --> #>
        2 if task_content.instanceof_inheritance() => Some(task_content.inner),
        // * 🚩乘积在蕴含里边 | <<(*, term, #) --> #> ==> #>
        3 => task_content.component_at(indexes[0]),
        // * 🚩乘积在蕴含的条件中 | <(&&, <(*, term, #) --> #>, #) ==> #>
        4 => task_content
            .as_conditional()?
            // * 🚩提取其中的条件项
            .1
            // * 🚩按索引提取「条件」中的继承陈述
            .component_at(indexes[1]),
        // * 🚩其它⇒返回
        _ => None,
    }
}

/// # 📄OpenNARS
///
/// Equivalent transformation between products and images
/// {<(*, S, M) --> P>, S@(*, S, M)} |- <S --> (/, P, _, M)>
/// {<S --> (/, P, _, M)>, P@(/, P, _, M)} |- <(*, S, M) --> P>
/// {<S --> (/, P, _, M)>, M@(/, P, _, M)} |- <M --> (/, P, S, _)>
fn transform_product_image(
    inheritance_to_be_transform: StatementRef,
    old_content: CompoundTermRef,
    context: &mut ReasonContextTransform,
) {
    // * 🚩提取参数 * //
    let t_link = context.current_task_link();
    let task_rc = t_link.target_rc();
    let task = task_rc.get_();
    let indexes = t_link.indexes();
    let reason_direction = context.reason_direction();

    // * 🚩词项 * //
    // * 📝此处针对各类「条件句」等复杂逻辑
    let new_inheritance =
        unwrap_or_return!(?transform_inheritance(inheritance_to_be_transform, indexes));

    // * 🚩用新构造的「继承」产生【在替换旧有内容中替换之后的】新词项
    let content =
        unwrap_or_return!(?replaced_transformed_content(old_content, indexes, new_inheritance));

    // * 🚩真值 * //
    let truth = task.get_truth().map(TruthValue::from);

    // * 🚩预算 * //
    drop(task);
    drop(task_rc);
    use ReasonDirection::*;
    let budget = match reason_direction {
        // * 🚩复合前向 | 此处无需unwrap：预算推理处再断言
        Forward => context.compound_forward(truth.as_ref(), &content),
        // * 🚩复合反向
        Backward => context.compound_backward(&content),
    };

    // * 🚩结论 * //
    // * 📝「真值」在「导出任务」时（从「当前任务」）自动生成
    context.single_premise_task_structural(content, truth, budget);
}

/// 🆕使用转换后的「关系继承句」回替词项
/// * 🚩按照词项链索引，在「转换后的词项」中找回其位置，并替换原有的词项
/// * ⚠️返回值可能为空
fn replaced_transformed_content(
    old_content: CompoundTermRef,
    indexes: &[usize],
    new_inheritance: Term,
) -> Option<Term> {
    // * 🚩选择或构建最终内容：模仿链接重构词项
    match indexes.len() {
        // * 🚩只有两层 ⇒ 只有「继承+关系」两层 ⇒ 直接使用
        // * 📄A @ <(*, A, B) --> R>
        2 => Some(new_inheritance.clone()),
        // * 🚩三层 ⇒ 只有「继承+关系」两层 ⇒ 直接使用
        // * 📄A @ <<(*, A, B) --> R> ==> C>
        // * 📄oldContent="<(&&,<$1 --> key>,<$2 --> lock>) ==> <$2 --> (/,open,$1,_)>>"
        //   * indices=[1, 1, 1]
        //   * newInh="<(*,$1,$2) --> open>"
        // *=> content="<(&&,<$1 --> key>,<$2 --> lock>) ==> <(*,$1,$2) --> open>>"
        _ if old_content.is_statement() && indexes[0] == 1 => {
            debug_assert!(
                indexes.len() == 3,
                "【2024-07-03 21:55:34】此处原意是「三层、陈述、在谓项中」"
            );
            debug_assert!(old_content.is_compound(), "原内容必须是复合词项");
            Term::make_statement(
                &old_content,
                old_content
                    .as_compound()
                    .unwrap()
                    .component_at(0)
                    .expect("复合词项必须有元素")
                    .clone(),
                new_inheritance,
            )
        }
        _ => match old_content.as_conditional() {
            Some((statement, conditional)) => {
                // * 🚩复合条件⇒四层：蕴含/等价 ⇒ 条件 ⇒ 关系继承 ⇒ 积/像
                // * 📄oldContent="<(&&,<#1-->lock>,<#1-->(/,open,$2,_)>)==>C>"
                //   * indices=[0, 1, 1, 1]
                //   * newInh="<(*,$2,#1)-->open>"
                // *=> content="<(&&,<#1-->lock>,<(*,$2,#1)-->open>)==>C>"
                debug_assert!(
                    indexes.len() == 4,
                    "【2024-07-03 21:55:34】此处原意是「四层、条件、在条件项中」"
                );
                let new_condition =
                    CompoundTermRef::set_component(conditional, indexes[1], Some(new_inheritance))?;
                Term::make_statement(&statement, new_condition, statement.predicate.clone())
            }
            _ => {
                // * 🚩非条件⇒三层：蕴含/等价/合取 ⇒ 结论=关系继承 ⇒ 积/像
                // * 📄oldContent="(&&,<#1 --> lock>,<#1 --> (/,open,#2,_)>,<#2 --> key>)"
                //   * indices=[1, 1, 1] @ "open"
                //   * newInh="<(*,#2,#1) --> open>"
                // *=> content="(&&,<#1 --> lock>,<#2 --> key>,<(*,#2,#1) --> open>)"
                // * 📄oldContent="<<$1 --> (/,open,_,{lock1})> ==> <$1 --> key>>"
                //   * indices=[0, 1, 0] @ "open"
                //   * newInh="<(*,$1,{lock1}) --> open>"
                // *=> content="<<(*,$1,{lock1}) --> open> ==> <$1 --> key>>"
                let mut components = old_content.clone_components();
                components[indexes[0]] = new_inheritance;
                if let Some(conjunction) = old_content.as_compound_type(CONJUNCTION_OPERATOR) {
                    Term::make_compound_term(conjunction, components)
                } else if let Some(statement) = old_content.as_statement() {
                    let subject = components.remove(0);
                    let predicate = components.remove(0);
                    Term::make_statement(&statement, subject, predicate)
                } else {
                    None
                }
            }
        },
    }
}

/// 🆕从「转换 乘积/像」中提取出的「转换继承」函数
/// * ⚠️返回值可能为空
/// * 🚩转换构造新的「继承」
fn transform_inheritance(
    inheritance_to_be_transform: StatementRef,
    indexes: &[usize],
) -> Option<Term> {
    // * 🚩决定前后项（此时已完成对「继承」的转换）
    let index = indexes[indexes.len() - 1]; // * 📝取索引 @ 复合词项内 | 📄B@(/, R, B, _) => 1
    let side = indexes[indexes.len() - 2]; // * 📝取索引 @ 复合词项所属继承句 | (*, A, B)@<(*, A, B) --> R> => 0
    let inner_compound = inheritance_to_be_transform
        .into_compound_ref()
        .component_at(side)?
        .as_compound()?; // * 📝拿到「继承」中的复合词项
    let [subject, predicate] = match inner_compound.identifier() {
        // * 🚩乘积⇒转像
        PRODUCT_OPERATOR => match side {
            // * 🚩乘积在左侧⇒外延像
            // * 📝占位符位置：与词项链位置有关
            0 => [
                inner_compound.component_at(index)?.clone(),
                Term::make_image_ext_from_product(
                    inner_compound,
                    inheritance_to_be_transform.predicate,
                    index,
                )?,
            ],
            // * 🚩乘积在右侧⇒内涵像
            // * 📝占位符位置：与词项链位置有关
            _ => [
                Term::make_image_int_from_product(
                    inner_compound,
                    inheritance_to_be_transform.subject,
                    index,
                )?,
                inner_compound.component_at(index)?.clone(),
            ],
        },
        // * 🚩外延像@后项⇒乘积/换索引
        IMAGE_EXT_OPERATOR if side == 1 => match index {
            // * 🚩链接来源正好是「关系词项」⇒转乘积
            //   * ℹ️新陈述：积 --> 关系词项
            //   * 📝实际情况是「索引在1⇒构造词项」
            //   * 📄「关系词项」如："open" @ "(/,open,$1,_)" | 始终在第一位，只是存储时放占位符的位置上
            0 => [
                inner_compound.component_at(index)?.clone(),
                Term::make_image_ext_from_image(
                    inner_compound,
                    inheritance_to_be_transform.subject,
                    index,
                )?,
            ],
            // * 🚩其它⇒调转占位符位置
            //   * ℹ️新陈述：另一元素 --> 新像
            //   * 📄「关系词项」如"{lock1}" @ "(/,open,_,{lock1})"
            //   * inh="<$1 --> (/,open,_,{lock1})>"
            //   * => "(/,open,$1,_)"
            _ => [
                inner_compound.component_at(index)?.clone(),
                Term::make_image_ext_from_image(
                    inner_compound,
                    inheritance_to_be_transform.subject,
                    index,
                )?,
            ],
        },
        // * 🚩内涵像@前项⇒乘积/换索引
        IMAGE_INT_OPERATOR if side == 1 => match index {
            // * 🚩链接来源正好是「关系词项」⇒转乘积
            //   * ℹ️新陈述：关系词项 --> 积
            //   * 📄「关系词项」如："open" @ "(\,open,$1,_)" | 始终在第一位，只是存储时放占位符的位置上
            0 => [
                inner_compound.component_at(index)?.clone(),
                Term::make_image_ext_from_image(
                    inner_compound,
                    inheritance_to_be_transform.subject,
                    index,
                )?,
            ],
            // * 🚩其它⇒调转占位符位置
            //   * ℹ️新陈述：新像 --> 另一元素
            //   * 📄「关系词项」如"neutralization" @ "(\,neutralization,_,$1)"
            //   * inh="<(\,neutralization,acid,_) --> $1>"
            //   * => "<(\,neutralization,_,$1) --> acid>"
            _ => [
                inner_compound.component_at(index)?.clone(),
                Term::make_image_ext_from_image(
                    inner_compound,
                    inheritance_to_be_transform.subject,
                    index,
                )?,
            ],
        },
        // * 🚩其它⇒无效
        _ => return None,
    };
    // * 🚩最终返回构造好的陈述
    Term::make_inheritance(subject, predicate)
}

fn transform_subject_product_image(
    inh_subject: CompoundTermRef,
    inh_predicate: &Term,
    context: &mut ReasonContextTransform,
) {
    todo!()
}

fn transform_predicate_product_image(
    inh_subject: &Term,
    inh_predicate: CompoundTermRef,
    context: &mut ReasonContextTransform,
) {
    todo!()
}
