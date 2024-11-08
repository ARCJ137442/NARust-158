//! 🎯复刻OpenNARS `nars.inference.TruthFunctions`
//! * 🚩【2024-06-21 00:31:46】现在基于[`Truth`]使用静态分派，并限定返回值为具体类型[`TruthValue`]
//!   * 📝若使用`-> impl Truth`，会导致生命周期问题
/// * 📝所有函数均【返回新真值对象】且【不修改所传入参数】
use crate::{
    entity::{ShortFloat, TruthValue},
    global::OccurrenceTime,
    inference::Truth,
};

/// 真值函数
/// * 🚩【2024-05-02 20:46:50】不同于OpenNARS中「直接创建新值」，此处许多「真值函数」仅改变自身
///   * ✅若需「创建新值」可以通过「事先`clone`」实现
/// * 🚩现在只为「具体的值」（带有「构造/转换」函数的类型）实现
pub trait TruthFunctions: Truth + Sized {
    /* ----- Single argument functions, called in MatchingRules ----- */

    /// 🆕恒等真值函数，用于转换推理
    /// * 🎯维护「真值计算」的一致性：所有真值计算均通过真值函数
    ///
    /// # 📄OpenNARS
    ///
    /// {<(*, A, B) --> R>} |- <A --> (/, R, _, B)>
    ///
    /// @param v1 Truth value of the premise
    /// @return Truth value of the conclusion
    fn identity(&self) -> TruthValue {
        let [f1, c1] = self.fc();
        // * 📝频率=旧频率
        // * 📝信度=旧信度
        TruthValue::new_fc(f1, c1)
    }

    /// 模拟`TruthFunctions.conversion`
    /// * 🚩转换
    ///
    /// # 📄OpenNARS
    ///
    /// {<A ==> B>} |- <B ==> A>
    ///
    /// @param v1 Truth value of the premise
    /// @return Truth value of the conclusion
    fn conversion(&self) -> TruthValue {
        let [f1, c1] = self.fc();
        // * 📝总频数=频率、信度之合取
        // * 📝频率=1（完全正面之猜测）
        // * 📝信度=总频数转换（保证弱推理）
        let w = f1 & c1;
        let c = ShortFloat::w2c(w.to_float());
        TruthValue::new_fc(ShortFloat::ONE, c)
    }

    /* ----- Single argument functions, called in StructuralRules ----- */

    /// 模拟`TruthFunctions.negation`
    /// * 🚩否定
    ///
    /// # 📄OpenNARS
    ///
    /// {A} |- (--A)
    ///
    /// @param v1 Truth value of the premise
    /// @return Truth value of the conclusion
    fn negation(&self) -> TruthValue {
        // * 📝频率相反，信度相等
        let f = !self.frequency();
        let c = self.confidence();
        TruthValue::new_fc(f, c)
    }

    /// 模拟`TruthFunctions.contraposition`
    /// * 🚩逆否
    ///
    /// # 📄OpenNARS
    ///
    /// {<A ==> B>} |- <(--, B) ==> (--, A)>
    ///
    /// @param v1 Truth value of the premise
    /// @return Truth value of the conclusion
    fn contraposition(&self) -> TruthValue {
        // * 📝频率为零，信度是弱
        let [f1, c1] = self.fc();
        let w = !f1 & c1;
        let c = ShortFloat::w2c(w.to_float());
        TruthValue::new_fc(ShortFloat::ZERO, c)
    }

    /* ----- double argument functions, called in SyllogisticRules ----- */

    /// 模拟`TruthFunctions.revision`
    /// * 🚩修正
    ///
    /// # 📄OpenNARS
    ///
    /// {<S ==> P>, <S ==> P>} |- <S ==> P>
    ///
    /// @param v1 Truth value of the first premise
    /// @param v2 Truth value of the second premise
    /// @return Truth value of the conclusion
    fn revision(&self, v2: &impl Truth) -> TruthValue {
        // * 📝转换为「频数视角」，频数相加，并转换回（频率，信度）二元组
        // * ✅特别兼容「信度为1」的「无穷证据量」情况：覆盖 or 取平均
        let ([f1, c1], [f2, c2]) = self.fc_with(v2);
        let is_inf_1 = c1.is_one();
        let is_inf_2 = c2.is_one();
        let ave_ari = ShortFloat::arithmetical_average;
        // * ✅在Rust中可以直接使用模式匹配
        let [f, c] = match [is_inf_1, is_inf_2] {
            // * 1 & 2
            [true, true] => [ave_ari([f1, f2]), ave_ari([c1, c2])],
            // * 1
            [true, false] => [f1, c1],
            // * 2
            [false, true] => [f2, c2],
            // * _
            [false, false] => {
                let w1 = ShortFloat::c2w(&c1);
                let w2 = ShortFloat::c2w(&c2);
                let w = w1 + w2;
                let f1 = f1.to_float();
                let f2 = f2.to_float();
                [
                    ShortFloat::from_float((w1 * f1 + w2 * f2) / w),
                    ShortFloat::w2c(w),
                ]
            }
        };
        TruthValue::new_fc(f, c)
    }

    /// 模拟`TruthFunctions.deduction`
    /// * 🚩演绎
    ///
    /// # 📄OpenNARS
    ///
    /// {<S ==> M>, <M ==> P>} |- <S ==> P>
    ///
    /// @param v1 Truth value of the first premise
    /// @param v2 Truth value of the second premise
    /// @return Truth value of the conclusion
    fn deduction(&self, v2: &impl Truth) -> TruthValue {
        let ([f1, c1], [f2, c2]) = self.fc_with(v2);
        // * 📝频率二者合取，信度四者合取
        let f = f1 & f2;
        let c = c1 & c2 & f;
        TruthValue::new_fc(f, c)
    }

    /// 模拟`TruthFunctions.deduction`
    /// * 🚩演绎导出
    /// * ⚠️此处会设置「真值」的`is_analytic`为`true`
    ///   * 💭或许此处的「M」就是「定义」的意思，因此与「分析」有关
    ///
    /// # 📄OpenNARS
    ///
    /// {M, <M ==> P>} |- P
    ///
    /// @param v1       Truth value of the first premise
    /// @param reliance Confidence of the second (analytical) premise
    /// @return Truth value of the conclusion
    fn analytic_deduction(&self, reliance: ShortFloat) -> TruthValue {
        let [f1, c1] = self.fc();
        // * 📌对于第二个「分析性前提」使用「依赖度」衡量
        // * 📝频率采用前者，信度合取以前者频率、依赖度，并标明这是「分析性」真值
        let c = f1 & c1 & reliance;
        TruthValue::new(f1, c, true)
    }

    /// 模拟`TruthFunctions.analogy`
    /// * 🚩类比
    ///
    /// # 📄OpenNARS
    ///
    /// {<S ==> M>, <M <=> P>} |- <S ==> P>
    ///
    /// @param v1 Truth value of the first premise
    /// @param v2 Truth value of the second premise
    /// @return Truth value of the conclusion
    fn analogy(&self, v2: &impl Truth) -> TruthValue {
        let ([f1, c1], [f2, c2]) = self.fc_with(v2);
        // * 📝类比：频率为二者合取，信度为双方信度、第二方频率三者合取
        let f = f1 & f2;
        let c = c1 & c2 & f2;
        TruthValue::new_fc(f, c)
    }

    /// 模拟`TruthFunctions.resemblance`
    /// * 🚩相似
    ///
    /// # 📄OpenNARS
    ///
    /// {<S <=> M>, <M <=> P>} |- <S <=> P>
    ///
    /// @param v1 Truth value of the first premise
    /// @param v2 Truth value of the second premise
    /// @return Truth value of the conclusion
    fn resemblance(&self, v2: &impl Truth) -> TruthValue {
        let ([f1, c1], [f2, c2]) = self.fc_with(v2);
        // * 📝类比：频率为二者合取，信度为「双方频率之析取」与「双方信度之合取」之合取
        let f = f1 & f2;
        let c = c1 & c2 & (f1 | f2);
        TruthValue::new_fc(f, c)
    }

    /// 模拟`TruthFunctions.abduction`
    /// * 🚩溯因
    ///
    /// # 📄OpenNARS
    ///
    /// {<S ==> M>, <P ==> M>} |- <S ==> P>
    ///
    /// @param v1 Truth value of the first premise
    /// @param v2 Truth value of the second premise
    /// @return Truth value of the conclusion
    fn abduction(&self, v2: &impl Truth) -> TruthValue {
        // * 🚩分析性⇒无意义（信度清零）
        if self.is_analytic() || v2.is_analytic() {
            return TruthValue::new_analytic_default();
        }
        let ([f1, c1], [f2, c2]) = self.fc_with(v2);
        // * 📝总频数=第二方频率与双方信度之合取
        // * 📝频率=第一方频率
        // * 📝信度=总频数转换（总是弱推理）
        let w = f2 & c1 & c2;
        let c = ShortFloat::w2c(w.to_float());
        TruthValue::new_fc(f1, c)
    }

    /// 模拟`TruthFunctions.abduction`
    /// * 🚩溯因导出
    ///
    /// # 📄OpenNARS
    ///
    /// {M, <P ==> M>} |- P
    ///
    /// @param v1       Truth value of the first premise
    /// @param reliance Confidence of the second (analytical) premise
    /// @return Truth value of the conclusion
    fn analytic_abduction(&self, reliance: ShortFloat) -> TruthValue {
        // * 🚩分析性⇒无意义（信度清零） | 只能「分析」一次
        if self.is_analytic() {
            return TruthValue::new_analytic_default();
        }
        let [f1, c1] = self.fc();
        // * 📝总频数=频率与「依赖度」之合取
        // * 📝频率=第一方频率
        // * 📝信度=总频数转换（总是弱推理）
        let w = c1 & reliance;
        let c = ShortFloat::w2c(w.to_float());
        TruthValue::new(f1, c, true)
    }

    /// 模拟`TruthFunctions.induction`
    /// * 🚩归纳
    ///
    /// # 📄OpenNARS
    ///
    /// {<M ==> S>, <M ==> P>} |- <S ==> P>
    ///
    /// @param v1 Truth value of the first premise
    /// @param v2 Truth value of the second premise
    /// @return Truth value of the conclusion
    fn induction(&self, v2: &impl Truth) -> TruthValue {
        // * 📝归纳是倒过来的归因
        v2.abduction(self)
    }

    /// 模拟`TruthFunctions.exemplification`
    /// * 🚩例证
    /// * 📝这实际上就是「演绎」反过来
    ///
    /// # 📄OpenNARS
    ///
    /// {<M ==> S>, <P ==> M>} |- <S ==> P>
    ///
    /// @param v1 Truth value of the first premise
    /// @param v2 Truth value of the second premise
    /// @return Truth value of the conclusion
    fn exemplification(&self, v2: &impl Truth) -> TruthValue {
        // * 🚩分析性⇒无意义（信度清零） | 只能「分析」一次
        if self.is_analytic() || v2.is_analytic() {
            return TruthValue::new_analytic_default();
        }
        let ([f1, c1], [f2, c2]) = self.fc_with(v2);
        // * 📝总频数=四方值综合
        // * 📝频率=1（无中生有）
        // * 📝信度=总频数转换（总是弱推理）
        let w = f1 & f2 & c1 & c2;
        let c = ShortFloat::w2c(w.to_float());
        TruthValue::new_fc(ShortFloat::ONE, c)
    }

    /// 模拟`TruthFunctions.comparison`
    /// * 🚩比对
    /// * 📝OpenNARS由此产生「相似」陈述
    ///
    /// # 📄OpenNARS
    ///
    /// {<M ==> S>, <M ==> P>} |- <S <=> P>
    ///
    /// @param v1 Truth value of the first premise
    /// @param v2 Truth value of the second premise
    /// @return Truth value of the conclusion
    fn comparison(&self, v2: &impl Truth) -> TruthValue {
        let ([f1, c1], [f2, c2]) = self.fc_with(v2);
        // * 📝总频数=「双频之析取」与「双信之合取」之合取
        // * 📝频率=「双频之合取」/「双频之析取」（📌根据函数图像，可以取"(0,0) -> 0"为可去间断点）
        // * 📝信度=总频数转换（总是弱推理）
        let f0 = f1 | f2;
        let f = match f0.is_zero() {
            true => ShortFloat::ZERO,
            false => (f1 & f2) / f0,
        };
        let w = f0 & c1 & c2;
        let c = ShortFloat::w2c(w.to_float());
        TruthValue::new_fc(f, c)
    }

    /* ----- desire-value functions, called in SyllogisticRules ----- */

    /// 模拟`TruthFunctions.desireStrong`
    /// * 💭强欲望推理
    ///
    /// # 📄OpenNARS
    ///
    /// A function specially designed for desire value [To be refined]
    ///
    /// @param v1 Truth value of the first premise
    /// @param v2 Truth value of the second premise
    /// @return Truth value of the conclusion
    fn desire_strong(&self, v2: &impl Truth) -> TruthValue {
        // ? 此函数似乎是用在「目标」上的
        let ([f1, c1], [f2, c2]) = self.fc_with(v2);
        // * 📝频率=双频之合取
        // * 📝信度=双方信度 合取 第二方频率
        let f = f1 & f2;
        let c = c1 & c2 & f2;
        TruthValue::new_fc(f, c)
    }

    /// 模拟`TruthFunctions.desireWeak`
    /// * 💭弱欲望推理
    ///
    /// # 📄OpenNARS
    ///
    /// A function specially designed for desire value [To be refined]
    ///
    /// @param v1 Truth value of the first premise
    /// @param v2 Truth value of the second premise
    /// @return Truth value of the conclusion
    fn desire_weak(&self, v2: &impl Truth) -> TruthValue {
        let ([f1, c1], [f2, c2]) = self.fc_with(v2);
        // * 📝频率=双频之合取
        // * 📝信度=双方信度 合取 第二方频率 合取 单位数目信度（保证弱推理）
        let f = f1 & f2;
        let c = c1 & c2 & f2 & ShortFloat::W2C1();
        TruthValue::new_fc(f, c)
    }

    /// 模拟`TruthFunctions.desireDed`
    /// * 🚩欲望演绎
    ///
    /// # 📄OpenNARS
    ///
    /// A function specially designed for desire value [To be refined]
    ///
    /// @param v1 Truth value of the first premise
    /// @param v2 Truth value of the second premise
    /// @return Truth value of the conclusion
    fn desire_deduction(&self, v2: &impl Truth) -> TruthValue {
        let ([f1, c1], [f2, c2]) = self.fc_with(v2);
        // * 📝频率=双频之合取
        // * 📝信度=双信之合取
        let f = f1 & f2;
        let c = c1 & c2;
        TruthValue::new_fc(f, c)
    }

    /// 模拟`TruthFunctions.desireInd`
    /// * 🚩欲望归纳
    ///
    /// # 📄OpenNARS
    ///
    /// A function specially designed for desire value [To be refined]
    ///
    /// @param v1 Truth value of the first premise
    /// @param v2 Truth value of the second premise
    /// @return Truth value of the conclusion
    fn desire_induction(&self, v2: &impl Truth) -> TruthValue {
        let ([f1, c1], [f2, c2]) = self.fc_with(v2);
        // * 📝总频数=第二方频率 合取 双信之合取
        // * 📝频率=第一方频率
        // * 📝信度=总频数转换（保证弱推理）
        let w = f2 & c1 & c2;
        let c = ShortFloat::w2c(w.to_float());
        TruthValue::new_fc(f1, c)
    }

    /* ----- double argument functions, called in CompositionalRules ----- */

    /// 模拟`TruthFunctions.union`
    /// * 🚩并集
    /// * 🚩【2024-05-03 14:40:42】目前回避Rust的关键字`union`
    ///
    /// # 📄OpenNARS
    ///
    /// {<M --> S>, <M <-> P>} |- <M --> (S|P)>
    ///
    /// @param v1 Truth value of the first premise
    /// @param v2 Truth value of the second premise
    /// @return Truth value of the conclusion
    #[doc(alias = "union")]
    fn union_(&self, v2: &impl Truth) -> TruthValue {
        let ([f1, c1], [f2, c2]) = self.fc_with(v2);
        // * 📝频率=双频之析取
        // * 📝信度=双信之合取
        let f = f1 | f2;
        let c = c1 & c2;
        TruthValue::new_fc(f, c)
    }

    /// 模拟`TruthFunctions.intersection`
    /// * 🚩交集
    ///
    /// # 📄OpenNARS
    ///
    /// {<M --> S>, <M <-> P>} |- <M --> (S&P)>
    ///
    /// @param v1 Truth value of the first premise
    /// @param v2 Truth value of the second premise
    /// @return Truth value of the conclusion
    fn intersection(&self, v2: &impl Truth) -> TruthValue {
        let ([f1, c1], [f2, c2]) = self.fc_with(v2);
        // * 📝频率=双频之合取
        // * 📝信度=双信之合取
        let f = f1 & f2;
        let c = c1 & c2;
        TruthValue::new_fc(f, c)
    }

    /// 模拟`TruthFunctions.reduceDisjunction`
    /// * 🚩消去性析取
    /// * 💭亦即数理逻辑中的「消解律」
    ///
    /// # 📄OpenNARS
    fn reduce_disjunction(&self, v2: &impl Truth) -> TruthValue {
        // * 🚩演绎（反向交集，依赖度=1）
        let v0 = self.intersection(&v2.negation());
        v0.analytic_deduction(ShortFloat::ONE)
    }

    /// 模拟`TruthFunctions.reduceConjunction`
    /// * 🚩消去性合取
    /// * 📝逻辑：先当作「并入」再进行「否定消去」
    ///
    /// # 📄OpenNARS
    ///
    /// {(--, (&&, A, B)), B} |- (--, A)
    ///
    /// @param v1 Truth value of the first premise
    /// @param v2 Truth value of the second premise
    /// @return Truth value of the conclusion
    fn reduce_conjunction(&self, v2: &impl Truth) -> TruthValue {
        // * 🚩否定演绎（反向交集（内部取反），依赖度=1）
        let v0 = self.negation().intersection(v2);
        v0.analytic_deduction(ShortFloat::ONE).negation()
    }

    /// 模拟`TruthFunctions.reduceConjunctionNeg`
    /// * 🚩消去性合取（否定）
    /// * 📝when 两端都套上了一个否定之时
    ///
    /// # 📄OpenNARS
    ///
    /// {(--, (&&, A, (--, B))), (--, B)} |- (--, A)
    ///
    /// @param v1 Truth value of the first premise
    /// @param v2 Truth value of the second premise
    /// @return Truth value of the conclusion
    fn reduce_conjunction_neg(&self, v2: &impl Truth) -> TruthValue {
        // * 🚩消取，但对第二方套否定
        self.reduce_conjunction(&v2.negation())
    }

    /// 模拟`TruthFunctions.anonymousAnalogy`
    /// * 🚩匿名溯因
    /// * 📝用于NAL-6「非独变量」的推理
    ///
    /// # 📄OpenNARS
    ///
    /// {(&&, <#x() ==> M>, <#x() ==> P>), S ==> M} |- <S ==> P>
    ///
    /// @param v1 Truth value of the first premise
    /// @param v2 Truth value of the second premise
    /// @return Truth value of the conclusion
    fn anonymous_analogy(&self, v2: &impl Truth) -> TruthValue {
        // * 📝中间频率=第一方频
        // * 📝中间信度=第一方信度作为「总频数」（弱推理）
        let [f1, c1] = self.fc();
        let v0 = TruthValue::new_fc(f1, ShortFloat::w2c(c1.to_float()));
        // * 🚩再参与「类比」（弱中之弱）
        v2.analogy(&v0)
    }

    /// 🆕真值永恒化
    fn eternalize(&self) -> TruthValue {
        let [f, c] = self.fc();
        TruthValue::new(f, ShortFloat::w2c(c.to_float()), self.is_analytic())
    }

    /// 🆕真值投影
    fn projection(
        &self,
        original_time: impl Into<OccurrenceTime>,
        target_time: impl Into<OccurrenceTime>,
        decay: ShortFloat,
    ) -> TruthValue {
        let [original_time, target_time] = [original_time.into(), target_time.into()];
        let [f, c] = self.fc();
        if original_time.is_eternal() {
            TruthValue::new(f, c, self.is_analytic())
        } else {
            let difference = OccurrenceTime::abs_diff_int(target_time, original_time);
            TruthValue::new(f, c * decay.pow(difference), self.is_analytic())
        }
    }

    /// 目标演绎
    /// * 🚩使用「直接演绎」和「反演演绎」再从信度中挑高的一个
    fn goal_deduction(&self, v2: &impl Truth) -> TruthValue {
        let res1 = self.deduction(v2);
        let res2 = self.negation().deduction(v2).negation();
        if res1.confidence() >= res2.confidence() {
            res1
        } else {
            res2
        }
    }
}

/// 为「真值」自动实现「真值函数」
impl<T: Truth + Sized> TruthFunctions for T {}

/// 单真值函数
pub type TruthFSingle = fn(&TruthValue) -> TruthValue;
/// 双真值函数
pub type TruthFDouble = fn(&TruthValue, &TruthValue) -> TruthValue;
/// 单真值依赖函数（分析性函数）
pub type TruthFAnalytic = fn(&TruthValue, ShortFloat) -> TruthValue;

/// TODO: 对每个真值函数的单元测试
#[cfg(test)]
mod tests {
    use super::*;

    /// 🆕函数表
    /// * 🎯示例性存储表示「真值函数」的引用（函数指针）
    /// * 🚩无需真正创建实例
    #[test]
    fn function_table() {
        // * 📌单真值函数
        let conversion: TruthFSingle = TruthValue::conversion;
        let negation: TruthFSingle = TruthValue::negation;
        let contraposition: TruthFSingle = TruthValue::contraposition;
        // * 📌双真值函数
        let revision: TruthFDouble = TruthValue::revision;
        let deduction: TruthFDouble = TruthValue::deduction;
        let analogy: TruthFDouble = TruthValue::analogy;
        let resemblance: TruthFDouble = TruthValue::resemblance;
        let abduction: TruthFDouble = TruthValue::abduction;
        let induction: TruthFDouble = TruthValue::induction;
        let exemplification: TruthFDouble = TruthValue::exemplification;
        let desire_strong: TruthFDouble = TruthValue::desire_strong;
        let desire_weak: TruthFDouble = TruthValue::desire_weak;
        let desire_deduction: TruthFDouble = TruthValue::desire_deduction;
        let desire_induction: TruthFDouble = TruthValue::desire_induction;
        let nal_union: TruthFDouble = TruthValue::union_;
        let intersection: TruthFDouble = TruthValue::intersection;
        let reduce_disjunction: TruthFDouble = TruthValue::reduce_disjunction;
        let reduce_conjunction: TruthFDouble = TruthValue::reduce_conjunction;
        let reduce_conjunction_neg: TruthFDouble = TruthValue::reduce_conjunction_neg;
        let anonymous_analogy: TruthFDouble = TruthValue::anonymous_analogy;
        // * 📌单真值依赖函数（分析性函数）
        let analytic_deduction: TruthFAnalytic = TruthValue::analytic_deduction;
        let analytic_abduction: TruthFAnalytic = TruthValue::analytic_abduction;

        let _ = [conversion, negation, contraposition];
        let _ = [
            revision,
            deduction,
            analogy,
            resemblance,
            abduction,
            induction,
            exemplification,
            desire_strong,
            desire_weak,
            desire_deduction,
            desire_induction,
            nal_union,
            intersection,
            reduce_disjunction,
            reduce_conjunction,
            reduce_conjunction_neg,
            anonymous_analogy,
        ];
        let _ = [analytic_deduction, analytic_abduction];
    }
}
