//! 🎯复刻OpenNARS `nars.entity.TaskLink`
//! * ✅【2024-05-06 00:13:26】基本功能复刻完成

use super::{Item, Task, TermLink, TermLinkConcrete, TermLinkType};
use crate::{
    entity::Sentence,
    global::{ClockTime, RefCount, RC},
    language::Term,
    nars::DEFAULT_PARAMETERS,
    ToDisplayAndBrief,
};

/// 模拟`nars.entity.TaskLink`
///
/// # 📄OpenNARS
///
/// Reference to a Task.
///
/// The reason to separate a Task and a TaskLink is that the same Task can be
/// linked from multiple Concepts, with different BudgetValue.
pub trait TaskLink: TermLink<Target = Self::Task> {
    /// 绑定的「任务」类型
    /// * 🚩【2024-05-07 19:00:30】目前认为，需要与自身所用之「元素id」「预算值」类型相同
    type Task: Task<Budget = Self::Budget, Key = Self::Key>;

    // * ✅无需模拟`TaskLink.targetTask`、`TaskLink.getTargetTask`
    //   * 📌此实现已被特征约束`T: Task`限定
    // /// 模拟`TaskLink.targetTask`、`TaskLink.getTargetTask`
    // fn target_task(&self) -> RC<Self::Target>;

    /// 模拟`TaskLink.recordedLinks`
    /// * 🚩此处使用[`Self::Key`]代替OpenNARS中的`String`
    /// * 📝OpenNARS中，一旦被创建，长度不会更改
    ///   * 🚩【2024-05-18 11:30:57】故使用`Box<[T]>`作为实际数据类型
    ///
    /// # 📄OpenNARS
    ///
    /// Remember the TermLinks that has been used recently with this TaskLink
    fn __recorded_links(&self) -> &[Self::Key];
    /// [`TaskLink::__recorded_links`]的可变版本
    fn __recorded_links_mut(&mut self) -> &mut Box<[Self::Key]>;

    /// 模拟`TaskLink.recordingTime`
    /// * 📝OpenNARS中，一旦被创建，长度不会更改
    ///   * 🚩【2024-05-18 11:30:57】故使用`Box<[T]>`作为实际数据类型
    ///
    /// # 📄OpenNARS
    ///
    /// Remember the time when each TermLink is used with this TaskLink
    fn __recording_time(&self) -> &[ClockTime];
    /// [`TaskLink::__recording_time`]的可变版本
    fn __recording_time_mut(&mut self) -> &mut Box<[ClockTime]>;

    /// 模拟`TaskLink.counter`
    /// * 🚩【2024-05-05 22:51:50】因此变量并未在外部被使用，故现设置为私有变量
    ///   * 🎯保证后续代码安全编写
    ///
    /// # 📄OpenNARS
    ///
    /// Remember the time when each TermLink is used with this TaskLink
    fn __counter(&self) -> usize;
    /// [`TaskLink::__counter`]的可变版本
    fn __counter_mut(&mut self) -> &mut usize;

    /// 模拟`TaskLink.novel`
    /// * 💫【2024-05-05 23:40:00】对这段代码的理解尚不明晰
    /// * 🗯️【2024-05-05 23:47:25】并不好的设计：本身方法看似是「读取信息」却有副作用
    ///   * 直接反映在「可变引用」上
    /// * 🚩【2024-05-05 23:57:12】因为「可变引用」的怪异，将其重命名为`update_novel`以突出其「修改」的动作
    ///
    /// TODO: 🏗️【2024-05-05 23:48:17】后续定要修复此中之「可变引用」问题
    ///
    /// # 📄OpenNARS
    ///
    /// To check whether a TaskLink should use a TermLink, return false if they
    /// interacted recently
    ///
    /// called in TermLinkBag only
    ///
    /// @param termLink    The TermLink to be checked
    /// @param currentTime The current time
    /// @return Whether they are novel to each other
    #[doc(alias = "novel")]
    fn update_novel<SelfTermLink>(
        &mut self,
        term_link: &SelfTermLink,
        current_time: ClockTime,
    ) -> bool
    where
        SelfTermLink: TermLinkConcrete<Budget = Self::Budget, Key = <Self as Item>::Key>,
    {
        /* 📄OpenNARS源码：
        Term bTerm = termLink.getTarget();
        if (bTerm.equals(targetTask.getSentence().getContent())) {
            return false;
        }
        String linkKey = termLink.getKey();
        int next, i;
        for (i = 0; i < counter; i++) {
            next = i % Parameters.TERM_LINK_RECORD_LENGTH;
            if (linkKey.equals(recordedLinks[next])) {
                if (currentTime < recordingTime[next] + Parameters.TERM_LINK_RECORD_LENGTH) {
                    return false;
                } else {
                    recordingTime[next] = currentTime;
                    return true;
                }
            }
        }
        next = i % Parameters.TERM_LINK_RECORD_LENGTH;
        recordedLinks[next] = linkKey; // add knowledge reference to recordedLinks
        recordingTime[next] = currentTime;
        if (counter < Parameters.TERM_LINK_RECORD_LENGTH) { // keep a constant length
            counter++;
        }
        return true; */
        let b_term = term_link.target();
        if *b_term == *self.target().content() {
            return false;
        }
        let link_key = term_link.key();
        for i in 0..self.__counter() {
            let next = i % DEFAULT_PARAMETERS.term_link_record_length;
            if *link_key == self.__recorded_links()[next] {
                match current_time
                    < self.__recording_time()[next] + DEFAULT_PARAMETERS.term_link_record_length
                {
                    true => return false,
                    false => {
                        self.__recording_time_mut()[next] = current_time;
                        return true;
                    }
                }
            }
        }
        let next = self.__counter() % DEFAULT_PARAMETERS.term_link_record_length;
        self.__recorded_links_mut()[next] = link_key.clone(); // ? 检查「新近」后，增加到自身记忆中？
        self.__recording_time_mut()[next] = current_time;
        if self.__counter() < DEFAULT_PARAMETERS.term_link_record_length {
            *self.__counter_mut() += 1;
        }
        true
    }
}

/// 「任务链」的具体类型
/// * 🎯【2024-05-06 11:19:52】作为[`TermLinkConcrete`]的对应物
pub trait TaskLinkConcrete: TaskLink + Sized {
    /// 🆕完全参数的构造函数
    ///
    /// TODO: 后续有待斟酌里边`target`的类型
    fn __new(
        // * 📌作为[`Item`]的参数
        key: Self::Key,
        budget: Self::Budget,
        // * 📌作为[`TermLink`]的参数
        term_link_type: TermLinkType,
        // * 📌独有参数
        target: RC<Self::Task>,
        recorded_links: Box<[Self::Key]>,
        recording_time: Box<[ClockTime]>,
        counter: usize,
    ) -> Self;

    /// 模拟`new TaskLink(Task t, TermLink template, BudgetValue v)`
    /// * 📝OpenNARS只有这一个公开的构造函数
    fn new<SelfTermLink>(
        target: RC<Self::Task>,
        template: Option<&SelfTermLink>,
        budget: Self::Budget,
    ) -> Self
    where
        SelfTermLink: TermLinkConcrete<Target = Term, Key = Self::Key, Budget = Self::Budget>,
    {
        /* 📄OpenNARS源码：
        super("", v, template == null ? TermLink.SELF : template.getType(),
                template == null ? null : template.getIndices());
        targetTask = t;
        recordedLinks = new String[Parameters.TERM_LINK_RECORD_LENGTH];
        recordingTime = new long[Parameters.TERM_LINK_RECORD_LENGTH];
        counter = 0;
        super.setKey(); // as defined in TermLink
        key += t.getKey(); */
        let term_link_type = match template {
            Some(link_t) => TermLinkType::from(link_t.type_ref()),
            None => TermLinkType::SELF,
        };
        let recorded_links: Box<[Self::Key]> = Default::default();
        let recording_time: Box<[ClockTime]> = Default::default();
        let counter = 0;
        let key = Self::_generate_key(&target.get_(), term_link_type.to_ref());
        Self::__new(
            key,
            budget,
            term_link_type,
            target,
            recorded_links,
            recording_time,
            counter,
        )
    }
}

/// 初代实现
mod impl_v1 {
    use super::*;
    use crate::{
        __impl_to_display_and_display,
        entity::{sentence::Sentence, Item, TaskConcrete, TermLinkRef, TermLinkType, TermLinkV1},
        global::{RefCount, RC},
        storage::BagKeyV1,
    };
    use std::ops::{Deref, DerefMut};

    /// 词项链 初代实现
    /// * 🚩目前不限制其中「预算值」的类型
    /// * ❌【2024-05-22 16:26:39】为保证对[`RcCell`]与[`ArcMutex`]的无缝兼容，不能自动派生[`PartialEq`]
    #[derive(Debug, Clone)]
    pub struct TaskLinkV1<T: TaskConcrete> {
        // * 📌作为[`Item`]的字段
        key: T::Key,
        budget: T::Budget,
        // * 📌作为「词项链」的字段
        type_ref: TermLinkType,
        // * 📌独有字段
        target: RC<T>,
        // TODO: 再增加字段，完成初代实现
    }

    __impl_to_display_and_display! {
        {T: TaskConcrete}
        TaskLinkV1<T> as Item
    }

    impl<T: TaskConcrete> Item for TaskLinkV1<T> {
        type Key = T::Key;
        type Budget = T::Budget;

        fn key(&self) -> &Self::Key {
            &self.key
        }

        fn budget(&self) -> &Self::Budget {
            &self.budget
        }

        fn budget_mut(&mut self) -> &mut Self::Budget {
            &mut self.budget
        }
    }

    /// 实现「词项链」
    /// * 🚩【2024-05-05 23:13:02】目前还是默认其中「元素id」[`BagKey`]的实现为[`String`]
    ///   * 📄因为当前「语句」只能生成[`String`]
    ///
    /// TODO: 【2024-05-05 23:14:49】🏗️后续定要做彻底的抽象化：对「语句」使用「ToKey」等特征方法……
    impl<T> TermLink for TaskLinkV1<T>
    where
        T: TaskConcrete<Key = BagKeyV1>,
    {
        type Target = T;

        #[inline(always)]
        fn target(&self) -> impl Deref<Target = Self::Target> {
            self.target.get_()
        }

        #[inline(always)]
        fn target_mut(&mut self) -> impl DerefMut<Target = Self::Target> {
            self.target.mut_()
        }

        fn type_ref(&self) -> TermLinkRef {
            self.type_ref.to_ref()
        }

        fn __key_mut(&mut self) -> &mut Self::Key {
            &mut self.key
        }

        fn _generate_key(target: &Self::Target, type_ref: TermLinkRef) -> Self::Key {
            // TODO: 【2024-05-05 23:12:08】有关字符串到底要耦合到多少程度，到底多少程度从BagKey抽象……这个还没底
            TermLinkV1::<T::Budget>::_generate_key(target.content(), type_ref)
        }
    }

    // TODO: 🏗️【2024-05-09 01:37:39】实装初代实现
    impl<T> TaskLink for TaskLinkV1<T>
    where
        T: TaskConcrete<Key = BagKeyV1>,
    {
        type Task = T;

        fn __recorded_links(&self) -> &[Self::Key] {
            todo!()
        }

        fn __recorded_links_mut(&mut self) -> &mut Box<[Self::Key]> {
            todo!()
        }

        fn __recording_time(&self) -> &[ClockTime] {
            todo!()
        }

        fn __recording_time_mut(&mut self) -> &mut Box<[ClockTime]> {
            todo!()
        }

        fn __counter(&self) -> usize {
            todo!()
        }

        fn __counter_mut(&mut self) -> &mut usize {
            todo!()
        }
    }

    impl<T> TaskLinkConcrete for TaskLinkV1<T>
    where
        T: TaskConcrete<Key = BagKeyV1>,
    {
        fn __new(
            // * 📌作为[`Item`]的参数
            key: Self::Key,
            budget: Self::Budget,
            // * 📌作为[`TermLink`]的参数
            term_link_type: TermLinkType,
            // * 📌独有参数
            target: RC<Self::Task>,
            recorded_links: Box<[Self::Key]>,
            recording_time: Box<[ClockTime]>,
            counter: usize,
        ) -> Self {
            todo!()
        }
    }
}
pub use impl_v1::*;

/// TODO: 单元测试
#[cfg(test)]
mod tests {
    use super::*;
}
