//! 转换推理/转换规则

use crate::{
    control::{
        ContextDerivationConcept, ReasonContext, ReasonContextTransform, ReasonContextWithLinks,
        ReasonDirection,
    },
    entity::{Sentence, TLink, TruthValue},
    inference::{BudgetInferenceContext, TruthFunctions},
    language::{CompoundTermRef, StatementRef, Term},
    symbols::*,
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

/// 🆕转换推理通用的「真值/预算值」计算
/// * 🎯删减繁琐的真值计算过程
fn truth_transforming(
    context: &mut ReasonContextTransform,
    new_content: &Term,
) -> (Option<TruthValue>, crate::entity::BudgetValue) {
    let direction = context.reason_direction();
    use ReasonDirection::*;
    // * 🚩真值 * //
    let truth = match direction {
        Forward => Some(
            context.current_task().get_().unwrap_judgement().identity(), // 真值函数：恒等
        ),
        Backward => None,
    };
    // * 🚩预算 * //
    let budget = match direction {
        // * 🚩复合前向 | 📝直接复用「转换后的真值」与解包等效
        Forward => context.budget_compound_forward(truth.as_ref(), new_content),
        // * 🚩复合反向
        Backward => context.budget_compound_backward(new_content),
    };
    (truth, budget)
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

    // * 🚩词项 * //
    // * 📝此处针对各类「条件句」等复杂逻辑
    let new_inheritance =
        unwrap_or_return!(?transform_inheritance(inheritance_to_be_transform, indexes));

    // * 🚩用新构造的「继承」产生【在替换旧有内容中替换之后的】新词项
    let content =
        unwrap_or_return!(?replaced_transformed_content(old_content, indexes, new_inheritance));

    // * 🚩真值&预算 | 恒等真值+复合前向/反向 * //
    drop(task);
    drop(task_rc);
    let (truth, budget) = truth_transforming(context, &content);

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
                let new_condition = conditional.set_component(indexes[1], Some(new_inheritance))?;
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
            0 => {
                let [image, outer] = Term::make_image_ext_from_image(
                    inner_compound,
                    inheritance_to_be_transform.subject,
                    index,
                )?;
                [outer, image]
            }
            // * 🚩其它⇒调转占位符位置
            //   * ℹ️新陈述：另一元素 --> 新像
            //   * 📄「关系词项」如"{lock1}" @ "(/,open,_,{lock1})"
            //   * inh="<$1 --> (/,open,_,{lock1})>"
            //   * => "(/,open,$1,_)"
            _ => {
                let [image, outer] = Term::make_image_ext_from_image(
                    inner_compound,
                    inheritance_to_be_transform.subject,
                    index,
                )?;
                [outer, image]
            }
        },
        // * 🚩内涵像@前项⇒乘积/换索引
        IMAGE_INT_OPERATOR if side == 1 => match index {
            // * 🚩链接来源正好是「关系词项」⇒转乘积
            //   * ℹ️新陈述：关系词项 --> 积
            //   * 📄「关系词项」如："open" @ "(\,open,$1,_)" | 始终在第一位，只是存储时放占位符的位置上
            0 => {
                let [image, outer] = Term::make_image_ext_from_image(
                    inner_compound,
                    inheritance_to_be_transform.subject,
                    index,
                )?;
                [outer, image]
            }
            // * 🚩其它⇒调转占位符位置
            //   * ℹ️新陈述：新像 --> 另一元素
            //   * 📄「关系词项」如"neutralization" @ "(\,neutralization,_,$1)"
            //   * inh="<(\,neutralization,acid,_) --> $1>"
            //   * => "<(\,neutralization,_,$1) --> acid>"
            _ => {
                let [image, outer] = Term::make_image_ext_from_image(
                    inner_compound,
                    inheritance_to_be_transform.subject,
                    index,
                )?;
                [outer, image]
            }
        },
        // * 🚩其它⇒无效
        _ => return None,
    };
    // * 🚩最终返回构造好的陈述
    Term::make_inheritance(subject, predicate)
}

/// # 📄OpenNARS
///
/// Equivalent transformation between products and images when the subject is a
/// compound
/// `{<(*, S, M) --> P>, S@(*, S, M)} |- <S --> (/, P, _, M)>`
/// `{<(\, P, _, M) --> S>, P@(\, P, _, M)} |- <P --> (*, S, M)>`
/// `{<(\, P, _, M) --> S>, M@(\, P, _, M)} |- <(\, P, S, _) --> M>`
fn transform_subject_product_image(
    inh_subject: CompoundTermRef,
    inh_predicate: &Term,
    context: &mut ReasonContextTransform,
) {
    // * 🚩积⇒外延像
    if let Some(product) = inh_subject.as_compound_type(PRODUCT_OPERATOR) {
        // * 🚩一次多个：遍历所有可能的索引
        for (i, new_subject) in product.components.iter().cloned().enumerate() {
            // * 🚩词项 * //
            let new_predicate = unwrap_or_return!(?Term::make_image_ext_from_product(product, inh_predicate, i) => continue);
            let inheritance =
                unwrap_or_return!(?Term::make_inheritance(new_subject, new_predicate) => continue);
            // * 🚩真值&预算 | 恒等真值+复合前向/反向 * //
            let (truth, budget) = truth_transforming(context, &inheritance);
            // * 🚩结论 * //
            // * 📝「真值」在「导出任务」时（从「当前任务」）自动生成
            context.single_premise_task_structural(inheritance, truth, budget);
        }
    }
    // * 🚩内涵像⇒积/其它内涵像
    else if let Some(image) = inh_subject.as_compound_type(IMAGE_INT_OPERATOR) {
        let placeholder_index = image.get_placeholder_index();
        // * 🚩一次多个：遍历除「关系词项」外所有位置
        for i in 1..image.size() {
            // * 🚩词项 * //
            // * 🚩根据「链接索引」与「关系索引（占位符位置）」的关系决定「积/像」
            let [new_subject, new_predicate] = match i == placeholder_index {
                // * 🚩转换回「积」
                true => {
                    let [product, relation] =
                        unwrap_or_return!(?Term::make_product(image, inh_predicate) => continue);
                    [relation, product] // 此时`component`是占位符
                }
                // * 🚩更改位置
                false => {
                    let [image, outer] = unwrap_or_return!(?Term::make_image_int_from_image(image, inh_predicate, i-1) => continue);
                    [image, outer]
                }
            };
            let inheritance =
                unwrap_or_return!(?Term::make_inheritance(new_subject, new_predicate) => continue);
            // * 🚩真值&预算 | 恒等真值+复合前向/反向 * //
            let (truth, budget) = truth_transforming(context, &inheritance);
            // * 🚩结论 * //
            // * 📝「真值」在「导出任务」时（从「当前任务」）自动生成
            context.single_premise_task_structural(inheritance, truth, budget);
        }
    }
}

/// # 📄OpenNARS
///
/// Equivalent transformation between products and images when the predicate is a
/// compound
/// `{<P --> (*, S, M)>, S@(*, S, M)} |- <(\, P, _, M) --> S>`
/// `{<S --> (/, P, _, M)>, P@(/, P, _, M)} |- <(*, S, M) --> P>`
/// `{<S --> (/, P, _, M)>, M@(/, P, _, M)} |- <M --> (/, P, S, _)>`
fn transform_predicate_product_image(
    inh_subject: &Term,
    inh_predicate: CompoundTermRef,
    context: &mut ReasonContextTransform,
) {
    // * 🚩积⇒内涵像
    if let Some(product) = inh_predicate.as_compound_type(PRODUCT_OPERATOR) {
        // * 🚩一次多个：遍历除「关系词项」外所有位置
        for (i, new_predicate) in product.components.iter().cloned().enumerate() {
            // * 🚩词项 * //
            let new_subject = unwrap_or_return!(?Term::make_image_int_from_product(product, inh_subject, i) => continue);
            let inheritance =
                unwrap_or_return!(?Term::make_inheritance(new_subject, new_predicate) => continue);
            // * 🚩真值&预算 | 恒等真值+复合前向/反向 * //
            let (truth, budget) = truth_transforming(context, &inheritance);
            // * 🚩结论 * //
            // * 📝「真值」在「导出任务」时（从「当前任务」）自动生成
            context.single_premise_task_structural(inheritance, truth, budget);
        }
    }
    // * 🚩外延像⇒积/其它外延像
    else if let Some(image) = inh_predicate.as_compound_type(IMAGE_EXT_OPERATOR) {
        let placeholder_index = image.get_placeholder_index();
        // * 🚩一次多个：遍历除「关系词项」外所有位置
        for i in 1..image.size() {
            // * 🚩词项 * //
            // * 🚩根据「链接索引」与「关系索引（占位符位置）」的关系决定「积/像」
            let [new_subject, new_predicate] = match i == placeholder_index {
                // * 🚩转换回「积」
                true => {
                    let [product, relation] =
                        unwrap_or_return!(?Term::make_product(image, inh_subject) => continue);
                    [product, relation] // 此时`component`是占位符
                }
                // * 🚩更改位置
                false => {
                    let [image, outer] = unwrap_or_return!(?Term::make_image_ext_from_image(image, inh_subject, i-1) => continue);
                    [outer, image]
                }
            };
            let inheritance =
                unwrap_or_return!(?Term::make_inheritance(new_subject, new_predicate) => continue);
            // * 🚩真值&预算 | 恒等真值+复合前向/反向 * //
            let (truth, budget) = truth_transforming(context, &inheritance);
            // * 🚩结论 * //
            // * 📝「真值」在「导出任务」时（从「当前任务」）自动生成
            context.single_premise_task_structural(inheritance, truth, budget);
        }
    }
}

/// 单元测试
#[cfg(test)]
mod tests {
    use super::*;
    use crate::inference::{process_direct, tools::*, InferenceEngine};
    use narsese::lexical_nse_term;

    const ENGINE: InferenceEngine = InferenceEngine::new(
        process_direct, // ! 必须加：否则无法转换任务
        transform_task,
        InferenceEngine::ECHO.matching_f(),
        InferenceEngine::ECHO.reason_f(),
    );

    /// 通用测试：输入系列指令，拉取打印输出，检查其中的所有词项
    fn test_transform_and_expect_terms(
        cmds: impl AsRef<str>,
        expect_terms: impl IntoIterator<Item = narsese::lexical::Term>,
    ) {
        let mut vm = create_reasoner_from_engine(ENGINE);
        // * 🚩输入指令并拉取输出
        let outs = vm.input_cmds_and_fetch_out(cmds.as_ref());
        // * 🚩打印输出
        print_outputs(&outs);
        // * 🚩检查其中是否有结论
        for expected in expect_terms {
            expect_outputs_contains_term(&outs, expected);
        }
    }

    /// 基础转换推理
    /// * 🚩积⇒像 @ 外延
    #[test]
    fn transform_basic_ext() {
        test_transform_and_expect_terms(
            r"
            nse <(*, A, B) --> R>.
            cyc 10
            ",
            [
                lexical_nse_term!(r"<A --> (/, R, _, B)>"),
                lexical_nse_term!(r"<B --> (/, R, A, _)>"),
            ],
        )
    }

    /// 基础转换推理
    /// * 🚩积⇒像
    #[test]
    fn transform_basic_int() {
        test_transform_and_expect_terms(
            r"
            nse <S --> (*, C, D)>.
            cyc 10
            ",
            [
                lexical_nse_term!(r"<(\, S, _, D) --> C>"),
                lexical_nse_term!(r"<(\, S, C, _) --> D>"),
            ],
        )
    }

    /// 反向转换推理 @ 外延
    /// * 🚩像⇒积/像
    #[test]
    fn transform_backward_ext() {
        test_transform_and_expect_terms(
            r"
            nse <A --> (/, R, _, B)>.
            cyc 10
            ",
            [
                lexical_nse_term!(r"<(*, A, B) --> R>"),
                lexical_nse_term!(r"<B --> (/, R, A, _)>"),
            ],
        )
    }

    /// 反向转换推理 @ 内涵
    /// * 🚩像⇒积/像
    #[test]
    fn transform_backward_int() {
        test_transform_and_expect_terms(
            r"
            nse <(\, S, _, D) --> C>.
            cyc 10
            ",
            [
                lexical_nse_term!(r"<S --> (*, C, D)>"),
                lexical_nse_term!(r"<(\, S, C, _) --> D>"),
            ],
        )
    }

    /// 稳定性
    #[test]
    fn stability() {
        let mut vm = create_reasoner_from_engine(ENGINE);
        // * 🚩输入指令并拉取输出
        let outs = vm.input_cmds_and_fetch_out(
            "
            nse <(*, A, B) --> R>.
            nse <S --> (*, C, D)>.
            cyc 1000
            ",
        );
        // * 🚩打印输出
        print_outputs(&outs);
    }
}
