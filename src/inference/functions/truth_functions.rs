//! 🎯复刻OpenNARS `nars.inference.TruthFunctions`

use super::UtilityFunctions;
use crate::entity::ShortFloat;
use crate::entity::TruthValueConcrete;

/// 真值函数
/// * 🚩【2024-05-02 20:46:50】不同于OpenNARS中「直接创建新值」，此处许多「真值函数」仅改变自身
///   * ✅若需「创建新值」可以通过「事先`clone`」实现
/// * 🚩现在只为「具体的值」（带有「构造/转换」函数的类型）实现
pub trait TruthFunctions: TruthValueConcrete {
    /* ----- Single argument functions, called in MatchingRules ----- */

    /// 模拟`TruthFunctions.conversion`
    /// * 🚩转换
    ///
    /// # 📄OpenNARS
    ///
    /// {<A ==> B>} |- <B ==> A>
    ///
    /// @param v1 Truth value of the premise
    /// @return Truth value of the conclusion
    fn conversion(&self) -> Self {
        /* 📄OpenNARS源码：
        float f1 = v1.getFrequency();
        float c1 = v1.getConfidence();
        float w = and(f1, c1);
        float c = w2c(w);
        return new TruthValue(1, c); */
        let f1 = self.frequency();
        let c1 = self.confidence();
        let w = f1 & c1;
        let c = Self::E::w2c(w.to_float());
        Self::new_fc(Self::E::ONE, c)
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
    fn negation(&self) -> Self {
        /* 📄OpenNARS源码：
        float f = 1 - v1.getFrequency();
        float c = v1.getConfidence();
        return new TruthValue(f, c); */
        let f = !self.frequency();
        let c = self.confidence();
        Self::new_fc(f, c)
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
    fn contraposition(&self) -> Self {
        /* 📄OpenNARS源码：
        float f1 = v1.getFrequency();
        float c1 = v1.getConfidence();
        float w = and(1 - f1, c1);
        float c = w2c(w);
        return new TruthValue(0, c); */
        let f1 = self.frequency();
        let c1 = self.confidence();
        let w = !f1 & c1;
        let c = Self::E::w2c(w.to_float());
        Self::new_fc(Self::E::ZERO, c)
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
    fn revision(&self, v2: &Self) -> Self {
        /* 📄OpenNARS源码：
        float f1 = v1.getFrequency();
        float f2 = v2.getFrequency();
        float c1 = v1.getConfidence();
        float c2 = v2.getConfidence();
        float w1 = c2w(c1);
        float w2 = c2w(c2);
        float w = w1 + w2;
        float f = (w1 * f1 + w2 * f2) / w;
        float c = w2c(w);
        return new TruthValue(f, c); */
        let f1 = self.frequency().to_float();
        let f2 = v2.frequency().to_float();
        let c1 = self.confidence();
        let c2 = v2.confidence();
        let w1 = Self::E::c2w(&c1);
        let w2 = Self::E::c2w(&c2);
        let w = w1 + w2;
        let f = Self::E::from_float((w1 * f1 + w2 * f2) / w);
        let c = Self::E::w2c(w);
        Self::new_fc(f, c)
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
    fn deduction(&self, v2: &Self) -> Self {
        /* 📄OpenNARS源码：
        float f1 = v1.getFrequency();
        float f2 = v2.getFrequency();
        float c1 = v1.getConfidence();
        float c2 = v2.getConfidence();
        float f = and(f1, f2);
        float c = and(c1, c2, f);
        return new TruthValue(f, c); */
        let f1 = self.frequency();
        let f2 = v2.frequency();
        let c1 = self.confidence();
        let c2 = v2.confidence();
        let f = f1 & f2;
        let c = c1 & c2 & f;
        Self::new_fc(f, c)
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
    fn deduction_reliance(&self, reliance: Self::E) -> Self {
        /* 📄OpenNARS源码：
        float f1 = v1.getFrequency();
        float c1 = v1.getConfidence();
        float c = and(f1, c1, reliance);
        return new TruthValue(f1, c, true); */
        let f1 = self.frequency();
        let c1 = self.confidence();
        let c = f1 & c1 & reliance;
        Self::new(f1, c, true)
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
    fn analogy(&self, v2: &Self) -> Self {
        /* 📄OpenNARS源码：
        float f1 = v1.getFrequency();
        float f2 = v2.getFrequency();
        float c1 = v1.getConfidence();
        float c2 = v2.getConfidence();
        float f = and(f1, f2);
        float c = and(c1, c2, f2);
        return new TruthValue(f, c); */
        let f1 = self.frequency();
        let f2 = v2.frequency();
        let c1 = self.confidence();
        let c2 = v2.confidence();
        let f = f1 & f2;
        let c = c1 & c2 & f2;
        Self::new_fc(f, c)
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
    fn resemblance(&self, v2: &Self) -> Self {
        /* 📄OpenNARS源码：
        float f1 = v1.getFrequency();
        float f2 = v2.getFrequency();
        float c1 = v1.getConfidence();
        float c2 = v2.getConfidence();
        float f = and(f1, f2);
        float c = and(c1, c2, or(f1, f2));
        return new TruthValue(f, c); */
        let f1 = self.frequency();
        let f2 = v2.frequency();
        let c1 = self.confidence();
        let c2 = v2.confidence();
        let f = f1 & f2;
        let c = c1 & c2 & (f1 | f2);
        Self::new_fc(f, c)
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
    fn abduction(&self, v2: &Self) -> Self {
        /* 📄OpenNARS源码：
        if (v1.getAnalytic() || v2.getAnalytic()) {
            return new TruthValue(0.5f, 0f);
        }
        float f1 = v1.getFrequency();
        float f2 = v2.getFrequency();
        float c1 = v1.getConfidence();
        float c2 = v2.getConfidence();
        float w = and(f2, c1, c2);
        float c = w2c(w);
        return new TruthValue(f1, c); */
        if self.is_analytic() || v2.is_analytic() {
            return Self::new_analytic_default();
        }
        let f1 = self.frequency();
        let f2 = v2.frequency();
        let c1 = self.confidence();
        let c2 = v2.confidence();
        let w = f2 & c1 & c2;
        let c = Self::E::w2c(w.to_float());
        Self::new_fc(f1, c)
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
    fn abduction_reliance(&self, reliance: Self::E) -> Self {
        /* 📄OpenNARS源码：
        if (v1.getAnalytic()) {
            return new TruthValue(0.5f, 0f);
        }
        float f1 = v1.getFrequency();
        float c1 = v1.getConfidence();
        float w = and(c1, reliance);
        float c = w2c(w);
        return new TruthValue(f1, c, true); */
        if self.is_analytic() {
            return Self::new_analytic_default();
        }
        let f1 = self.frequency();
        let c1 = self.confidence();
        let w = c1 & reliance;
        let c = Self::E::w2c(w.to_float());
        Self::new(f1, c, true)
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
    fn induction(&self, v2: &Self) -> Self {
        self.abduction(v2)
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
    fn exemplification(&self, v2: &Self) -> Self {
        /* 📄OpenNARS源码：
        if (v1.getAnalytic() || v2.getAnalytic()) {
            return new TruthValue(0.5f, 0f);
        }
        float f1 = v1.getFrequency();
        float f2 = v2.getFrequency();
        float c1 = v1.getConfidence();
        float c2 = v2.getConfidence();
        float w = and(f1, f2, c1, c2);
        float c = w2c(w);
        return new TruthValue(1, c); */
        if self.is_analytic() || v2.is_analytic() {
            return Self::new_analytic_default();
        }
        let f1 = self.frequency();
        let f2 = v2.frequency();
        let c1 = self.confidence();
        let c2 = v2.confidence();
        let w = f1 & f2 & c1 & c2;
        let c = Self::E::w2c(w.to_float());
        Self::new_fc(Self::E::ONE, c)
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
    fn comparison(&self, v2: &Self) -> Self {
        /* 📄OpenNARS源码：
        float f1 = v1.getFrequency();
        float f2 = v2.getFrequency();
        float c1 = v1.getConfidence();
        float c2 = v2.getConfidence();
        float f0 = or(f1, f2);
        float f = (f0 == 0) ? 0 : (and(f1, f2) / f0);
        float w = and(f0, c1, c2);
        float c = w2c(w);
        return new TruthValue(f, c); */
        let f1 = self.frequency();
        let f2 = v2.frequency();
        let c1 = self.confidence();
        let c2 = v2.confidence();
        let f0 = f1 | f2;
        let f = match f0.is_zero() {
            true => Self::E::ZERO,
            false => (f1 & f2) / f0,
        };
        let w = f0 & c1 & c2;
        let c = Self::E::w2c(w.to_float());
        Self::new_fc(f, c)
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
    fn desire_strong(&self, v2: &Self) -> Self {
        /* 📄OpenNARS源码：
        float f1 = v1.getFrequency();
        float f2 = v2.getFrequency();
        float c1 = v1.getConfidence();
        float c2 = v2.getConfidence();
        float f = and(f1, f2);
        float c = and(c1, c2, f2);
        return new TruthValue(f, c); */
        let f1 = self.frequency();
        let f2 = v2.frequency();
        let c1 = self.confidence();
        let c2 = v2.confidence();
        let f = f1 & f2;
        let c = c1 & c2 & f2;
        Self::new_fc(f, c)
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
    fn desire_weak(&self, v2: &Self) -> Self {
        /* 📄OpenNARS源码：
        float f1 = v1.getFrequency();
        float f2 = v2.getFrequency();
        float c1 = v1.getConfidence();
        float c2 = v2.getConfidence();
        float f = and(f1, f2);
        float c = and(c1, c2, f2, w2c(1.0f));
        return new TruthValue(f, c); */
        let f1 = self.frequency();
        let f2 = v2.frequency();
        let c1 = self.confidence();
        let c2 = v2.confidence();
        let f = f1 & f2;
        let c = c1 & c2 & f2 & Self::E::w2c(1.0);
        Self::new_fc(f, c)
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
    fn desire_ded(&self, v2: &Self) -> Self {
        /* 📄OpenNARS源码：
        float f1 = v1.getFrequency();
        float f2 = v2.getFrequency();
        float c1 = v1.getConfidence();
        float c2 = v2.getConfidence();
        float f = and(f1, f2);
        float c = and(c1, c2);
        return new TruthValue(f, c); */
        let f1 = self.frequency();
        let f2 = v2.frequency();
        let c1 = self.confidence();
        let c2 = v2.confidence();
        let f = f1 & f2;
        let c = c1 & c2;
        Self::new_fc(f, c)
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
    fn desire_ind(&self, v2: &Self) -> Self {
        /* 📄OpenNARS源码：
        float f1 = v1.getFrequency();
        float f2 = v2.getFrequency();
        float c1 = v1.getConfidence();
        float c2 = v2.getConfidence();
        float w = and(f2, c1, c2);
        float c = w2c(w);
        return new TruthValue(f1, c); */
        let f1 = self.frequency();
        let f2 = v2.frequency();
        let c1 = self.confidence();
        let c2 = v2.confidence();
        let w = f2 & c1 & c2;
        let c = Self::E::w2c(w.to_float());
        Self::new_fc(f1, c)
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
    fn nal_union(&self, v2: &Self) -> Self {
        /* 📄OpenNARS源码：
        float f1 = v1.getFrequency();
        float f2 = v2.getFrequency();
        float c1 = v1.getConfidence();
        float c2 = v2.getConfidence();
        float f = or(f1, f2);
        float c = and(c1, c2);
        return new TruthValue(f, c); */
        let f1 = self.frequency();
        let f2 = v2.frequency();
        let c1 = self.confidence();
        let c2 = v2.confidence();
        let f = f1 | f2;
        let c = c1 & c2;
        Self::new_fc(f, c)
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
    fn intersection(&self, v2: &Self) -> Self {
        /* 📄OpenNARS源码：
        float f1 = v1.getFrequency();
        float f2 = v2.getFrequency();
        float c1 = v1.getConfidence();
        float c2 = v2.getConfidence();
        float f = and(f1, f2);
        float c = and(c1, c2);
        return new TruthValue(f, c); */
        let f1 = self.frequency();
        let f2 = v2.frequency();
        let c1 = self.confidence();
        let c2 = v2.confidence();
        let f = f1 & f2;
        let c = c1 & c2;
        Self::new_fc(f, c)
    }

    /// 模拟`TruthFunctions.reduceDisjunction`
    /// * 🚩消去性析取
    /// * 💭亦即数理逻辑中的「消解律」
    ///
    /// # 📄OpenNARS
    fn reduce_disjunction(&self, v2: &Self) -> Self {
        /* 📄OpenNARS源码：
        TruthValue v0 = intersection(v1, negation(v2));
        return deduction(v0, 1f); */
        let v0 = self.intersection(&v2.negation());
        v0.deduction_reliance(Self::E::ONE)
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
    fn reduce_conjunction(&self, v2: &Self) -> Self {
        /* 📄OpenNARS源码：
        TruthValue v0 = intersection(negation(v1), v2);
        return negation(deduction(v0, 1f)); */
        let v0 = self.negation().intersection(v2);
        v0.deduction_reliance(Self::E::ONE).negation()
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
    fn reduce_conjunction_neg(&self, v2: &Self) -> Self {
        /* 📄OpenNARS源码：
        return reduceConjunction(v1, negation(v2)); */
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
    fn anonymous_analogy(&self, v2: &Self) -> Self {
        /* 📄OpenNARS源码：
        float f1 = v1.getFrequency();
        float c1 = v1.getConfidence();
        TruthValue v0 = new TruthValue(f1, w2c(c1));
        return analogy(v2, v0); */
        let f1 = self.frequency();
        let c1 = self.confidence();
        let v0 = Self::new_fc(f1, Self::E::w2c(c1.to_float()));
        v2.analogy(&v0)
    }
}

/// 为「真值」自动实现「真值函数」
impl<T: TruthValueConcrete> TruthFunctions for T {}

/// TODO: 单元测试
#[cfg(test)]
mod tests {
    use super::*;
}
