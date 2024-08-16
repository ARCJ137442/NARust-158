//! 实现「推理器」层面的「序列反序列化」
//! * 🎯基本完整的「推理器状态」数据存储
//! * 🎯「记忆区」加上「推导数据」的序列反序列化
//! * ℹ️有关「记忆区序列反序列化」参见[`crate::storage::Memory`]
//! * 🔗有关`state`与`status`的区别：https://www.quora.com/Whats-the-difference-in-usage-between-state-and-status
//!   * `state`范围更广，且常用于描述【离散】的状态
//!   * `status`大多指「当下状态」并且能用于名词

use super::{Reasoner, ReasonerDerivationData};
use crate::{entity::RCTask, global::ClockTime, storage::Memory, util::Serial};
use serde::{Deserialize, Serialize};

/// 推理器状态
/// * 🎯先反序列化到此类型，再让推理器加载
/// * 🎯基本完整的「推理器状态」数据存储
/// * 🎯「记忆区」加上「推导数据」的序列反序列化
/// * 🔗有关`state`与`status`的区别：https://www.quora.com/Whats-the-difference-in-usage-between-state-and-status
///   * `state`范围更广，且常用于描述【离散】的状态
///   * `status`大多指「当下状态」并且能用于名词
/// * 🚩【2024-08-12 20:25:24】作为与「推导数据」类似的结构，不对外暴露
/// * ❌【2024-08-12 20:44:58】不能手动实现：操作较为复杂
///   * ⚠️[`Deserialize::deserialize`]只能在函数中调用一次，并且会消耗参数所有权
///   * 🔗参见：<https://serde.rs/deserialize-struct.html>
#[derive(Debug, Deserialize)]
pub(super) struct ReasonerStatusStorage {
    /// 记忆区
    pub memory: Memory,

    /// 推导数据
    pub derivation_datas: ReasonerDerivationData,

    /// 系统时钟
    pub clock: ClockTime,

    /// 时间戳序列号（递增序列号）
    pub stamp_current_serial: ClockTime,

    /// 任务序列号（递增序列号）
    pub task_current_serial: Serial,
}

/// 推理器状态的引用
/// * 🎯从「推理器」构造引用，并由此序列化
#[derive(Debug, Clone, Copy, Serialize)]
pub(super) struct ReasonerStatusStorageRef<'s> {
    /// 记忆区
    pub memory: &'s Memory,

    /// 推导数据
    pub derivation_datas: &'s ReasonerDerivationData,

    /// 系统时钟
    pub clock: ClockTime,

    /// 时间戳序列号（递增序列号）
    pub stamp_current_serial: ClockTime,

    /// 任务序列号（递增序列号）
    pub task_current_serial: Serial,
}

impl ReasonerStatusStorage {
    /// 对整个「推理器状态」的共享引用归一化
    fn unify_all_task_rcs(&mut self) {
        let memory_refs = self.memory.all_task_rcs();
        let derivation_datas_refs = self.derivation_datas.iter_mut_task_rcs();
        let refs = memory_refs.chain(derivation_datas_refs);
        RCTask::unify_rcs(refs);
    }
}

/// 推理器具体结构加载
/// * 📄记忆区加载
/// * 📄推理状态加载
impl Reasoner {
    /// 加载新的记忆区
    #[must_use]
    pub fn load_memory(&mut self, mut memory: Memory) -> Memory {
        // 先交换记忆区对象
        std::mem::swap(&mut memory, &mut self.memory);
        // 返回旧记忆区
        memory
    }

    /// 加载「推导数据」
    #[must_use]
    fn load_derivation_datas(
        &mut self,
        mut derivation_datas: ReasonerDerivationData,
    ) -> ReasonerDerivationData {
        // 先交换记忆区对象
        std::mem::swap(&mut derivation_datas, &mut self.derivation_datas);
        // 返回旧记忆区
        derivation_datas
    }

    /// 加载「推理器状态」
    #[must_use]
    fn load_status(&mut self, status: ReasonerStatusStorage) -> ReasonerStatusStorage {
        let ReasonerStatusStorage {
            memory,
            derivation_datas,
            clock,
            stamp_current_serial,
            task_current_serial,
        } = status;
        // 加载记忆区
        let memory = self.load_memory(memory);
        // 加载推导数据
        let derivation_datas = self.load_derivation_datas(derivation_datas);
        // 加载其它基础类型
        let clock_old = self.time();
        self.set_time(clock);
        let stamp_current_serial_old = self.stamp_current_serial();
        self.set_stamp_current_serial(stamp_current_serial);
        let task_current_serial_old = self.task_current_serial();
        self.set_task_current_serial(task_current_serial);
        // 将旧的数据返回
        ReasonerStatusStorage {
            memory,
            derivation_datas,
            clock: clock_old,
            stamp_current_serial: stamp_current_serial_old,
            task_current_serial: task_current_serial_old,
        }
    }
}

/// 推理器外部「序列化/反序列化」接口
/// * 🎯屏蔽具体序列反序列化数据类型
///   * 📄[JSON](serde_json)
impl Reasoner {
    /// 从推理器序列化出「推理器状态」
    pub fn serialize_memory<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        // 再序列化
        self.memory.serialize(serializer)
    }

    /// 反序列化并加载「推理器状态」
    /// * 🚩【2024-08-12 20:22:42】不返回「推理器状态」数据
    ///   * 💭出于内部使用考虑，不暴露「推理器状态」数据类型
    pub fn load_from_deserialized_memory<'de, D>(&mut self, deserializer: D) -> Result<(), D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        // 先反序列化到结构体
        // * ✅无需做引用归一化：记忆区反序列化时，便已完成
        let memory = Memory::deserialize(deserializer)?;
        // 再加载
        let _ = self.load_memory(memory);
        Ok(())
    }
    /// 从推理器序列化出「推理器状态」
    pub fn serialize_status<S: serde::Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        // 先构造引用
        let storage_ref = ReasonerStatusStorageRef {
            memory: &self.memory,
            derivation_datas: &self.derivation_datas,
            clock: self.time(),
            stamp_current_serial: self.stamp_current_serial(),
            task_current_serial: self.task_current_serial(),
        };
        // 再序列化
        storage_ref.serialize(serializer)
    }

    /// 反序列化并加载「推理器状态」
    /// * 🚩【2024-08-12 20:22:42】不返回「推理器状态」数据
    ///   * 💭出于内部使用考虑，不暴露「推理器状态」数据类型
    pub fn load_from_deserialized_status<'de, D>(&mut self, deserializer: D) -> Result<(), D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        // 先反序列化到结构体
        let mut status = ReasonerStatusStorage::deserialize(deserializer)?;
        // 引用归一化
        status.unify_all_task_rcs();
        // 再加载
        let _ = self.load_status(status);
        Ok(())
    }
}

// * 🚩【2024-08-12 21:16:27】单元测试放在`cmd_dispatch`处，与JSON格式、NAVM指令分派 一同测试
#[cfg(test)]
pub mod test_util_ser_de {
    use super::*;
    use crate::{
        assert_eq_try,
        entity::Task,
        ok,
        storage::{tests_memory::*, Bag},
        util::AResult,
    };
    use std::collections::VecDeque;

    /// 获取记忆区的引用，而不直接暴露字段
    impl GetMemory for Reasoner {
        fn get_memory(&self) -> &Memory {
            &self.memory
        }
    }

    /// 获取记忆区的引用，而不直接暴露字段
    impl GetMemory for ReasonerStatusStorage {
        fn get_memory(&self) -> &Memory {
            &self.memory
        }
    }

    /// 获取记忆区的引用，而不直接暴露字段
    impl GetMemory for ReasonerStatusStorageRef<'_> {
        fn get_memory(&self) -> &Memory {
            self.memory
        }
    }

    /// 用于在外部crate中，从虚拟机引用记忆区
    pub trait GetReasoner {
        fn get_reasoner(&self) -> &Reasoner;
    }

    /// 获取推理器的引用，而不直接暴露字段
    impl GetReasoner for Reasoner {
        fn get_reasoner(&self) -> &Reasoner {
            self
        }
    }

    /// 保存前判断是否同步
    /// * 判断「[`Rc`]是否在『传入值所有权』后仍尝试移动内部值」
    pub fn status_synced(reasoner: &impl GetReasoner) {
        let reasoner = reasoner.get_reasoner();
        memory_synced(&reasoner.memory);
        reasoner
            .derivation_datas
            .iter_task_rcs()
            .for_each(rc_synced);
    }

    /// 判断推理器状态的一致性
    /// * 🚩通过「返回错误」指定「一致性缺失」
    /// * 📌只传入推理器来判断，不暴露内部数据类型
    pub fn status_consistent<R1: GetReasoner, R2: GetReasoner>(a: &R1, b: &R2) -> AResult {
        let [a, b] = [a.get_reasoner(), b.get_reasoner()];
        // 记忆区一致性
        memory_consistent(&a.memory, &b.memory)?;
        // 推导数据一致性
        derivation_datas_consistent(&a.derivation_datas, &b.derivation_datas)?;
        // 其它数据一致性
        assert_eq_try!(
            a.time(),
            b.time(),
            "系统时钟不一致：{} != {}",
            a.time(),
            b.time()
        );
        assert_eq_try!(
            a.stamp_current_serial(),
            b.stamp_current_serial(),
            "系统时间戳序列号不一致"
        );

        ok!()
    }

    fn derivation_datas_consistent(
        a: &ReasonerDerivationData,
        b: &ReasonerDerivationData,
    ) -> AResult {
        // 新任务队列一致性
        task_deque_consistent(&a.new_tasks, &b.new_tasks)?;
        // 任务袋一致性
        task_bag_consistent(&a.novel_tasks, &b.novel_tasks)?;
        // 推导数据一致性
        ok!()
    }

    /// 任务队列一致性
    /// * 🎯新任务队列
    fn task_deque_consistent(a: &VecDeque<Task>, b: &VecDeque<Task>) -> AResult {
        assert_eq_try!(a.len(), b.len(), "任务队列不一致——长度不一致");
        for (a, b) in zip(a, b) {
            task_consistent(a, b)?;
        }
        // 任务一致性
        ok!()
    }

    /// 任务袋一致性
    /// * 🎯新近任务袋
    fn task_bag_consistent(a: &Bag<Task>, b: &Bag<Task>) -> AResult {
        bag_consistent(a, b, task_consistent)?;
        ok!()
    }
}
