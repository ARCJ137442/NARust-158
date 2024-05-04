//! 🆕NARust的NAVM接口
//! * 🎯接入NAVM，在源码层实现统一输入输出
// TODO: 有待整理（泛型化、参数化，脱实现化）
/*
    use crate::{global::tests::AResult, ok};
use navm::{
    cmd::Cmd,
    output::Output,
    vm::{VmLauncher, VmRuntime},
};

use crate::{
    entity::{BagItem, Budget, BudgetValue},
    storage::bag::{Bag, BagV1},
};


// enum Todo {}
// impl BagItem for Todo {}

// #[derive(Debug, Clone, Default)]
// struct Vm<Memory, Budget>
// where
//     // TODO: 占位符（后续将添加更多新特性）
//     Memory: Bag<Key = String, Item = Todo, Budget = Budget>,
//     Budget: BudgetValue,
// {
//     cached_outputs: Vec<Output>,
//     memory: Memory,
// }

// struct NarsLauncher {
//     // TODO: 根据OpenNARS增加启动配置选项（如构造函数参数）
// }

// impl VmLauncher for NarsLauncher {
//     type Runtime = Vm<BagV1<Todo>, Budget>;

//     fn launch(self) -> Result<Self::Runtime> {
//         todo!()
//     }
// }

// impl<Memory, Budget> VmRuntime for Vm<Memory, Budget>
// where
//     Memory: Bag<Key = String, Item = Todo, Budget = Budget>,
//     Budget: BudgetValue,
// {
//     fn input_cmd(&mut self, cmd: Cmd) -> AResult {
//         todo!()
//     }

//     fn fetch_output(&mut self) -> Result<Output> {
//         todo!()
//     }

//     fn try_fetch_output(&mut self) -> Result<Option<Output>> {
//         todo!()
//     }

//     fn status(&self) -> &navm::vm::VmStatus {
//         todo!()
//     }

//     fn terminate(&mut self) -> AResult {
//         todo!()
//     }
// }
*/
