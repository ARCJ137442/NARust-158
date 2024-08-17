//! 集中管理有关「推理器分派处理指令」的函数

use super::RuntimeAlpha;
use navm::cmd::Cmd;

/// 输入指令
impl RuntimeAlpha {
    /// 模拟`ReasonerBatch.textInputLine`
    /// * 🚩🆕【2024-05-13 02:27:07】从「字符串输入」变为「NAVM指令输入」
    /// * 🚩【2024-06-29 01:42:46】现在不直接暴露「输入NAVM指令」：全权交给「通道」机制
    ///   * 🚩由「通道」的「处理IO」引入
    pub(super) fn input_cmd(&mut self, cmd: Cmd) {
        use Cmd::*;
        match cmd {
            SAV { target, path } => self.cmd_sav(target, path),
            LOA { target, path } => self.cmd_loa(target, path),
            // * 🚩重置：推理器复位
            RES { .. } => self.reasoner.reset(),
            // * 🚩Narsese：输入任务（但不进行推理）
            NSE(narsese) => self.cmd_nse(narsese),
            // NEW { target } => (),
            // DEL { target } => (),
            // * 🚩工作周期：只执行推理，不处理输入输出
            CYC(cycles) => self.reasoner.cycle(cycles),
            // * 🚩音量：设置音量 & 提示
            VOL(volume) => self.cmd_vol(volume),
            // REG { name } => (),
            INF { source } => self.cmd_inf(source),
            HLP { name } => self.cmd_hlp(name),
            // * 🚩【2024-05-13 12:21:37】注释：不做任何事情
            REM { .. } => (),
            // * 🚩退出⇒处理完所有输出后直接退出
            EXI { reason } => self.cmd_exi(reason),
            // Custom { head, tail } => (),
            // * 🚩未知指令⇒输出提示
            _ => self.reasoner.report_error(format!("Unknown cmd: {cmd}")),
        }
    }

    /// 处理指令[`Cmd::NSE`]
    fn cmd_nse(&mut self, narsese: narsese::lexical::Task) {
        self.reasoner.input_task(narsese)
    }

    /// 处理指令[`Cmd::VOL`]
    fn cmd_vol(&mut self, volume: usize) {
        self.reasoner
            .report_info(format!("volume: {} => {volume}", self.reasoner.volume()));
        self.reasoner.set_volume(volume);
    }

    /// 处理指令[`Cmd::EXI`]
    ///
    /// ? ❓【2024-07-23 16:10:13】是否一定要主程序退出
    ///   * 💭还是说，NARS本身并没有个实际上的「退出」机制
    fn cmd_exi(&mut self, reason: String) {
        // * 🚩最后的提示性输出
        self.reasoner
            .report_info(format!("Program exited with reason {reason:?}"));
        // * 🚩处理所有输出
        self.handle_output();
        // * 🚩最终退出程序
        std::process::exit(0);
    }

    /// 处理一个[`Result`]消息
    /// * 📌根据变体决定消息类型
    ///   * [`Ok`] => `INFO`
    ///   * [`Err`] => `ERROR`
    fn report_result(&mut self, result: Result<String, String>) {
        // 消息分派 | 📌只在此处涉及「报告输出」
        match result {
            // 正常信息⇒报告info
            Ok(message) => self.reasoner.report_info(message),
            // 错误信息⇒报告error
            Err(message) => self.reasoner.report_error(message),
        }
    }

    /// 处理指令[`Cmd::INF`]
    fn cmd_inf(&mut self, source: String) {
        // 查询
        let query = source.to_lowercase();
        // 消息分派 | 📌只在此处涉及「报告输出」
        let result = inf_dispatch(&mut self.reasoner, query);
        self.report_result(result)
    }

    /// 处理指令[`Cmd::HLP`]
    fn cmd_hlp(&mut self, name: String) {
        // 查询
        let query = name.to_lowercase();
        // 获取并报告消息
        let result = hlp_dispatch(&mut self.reasoner, query);
        self.report_result(result)
    }

    /// 处理指令[`Cmd::SAV`]
    fn cmd_sav(&mut self, target: String, path: String) {
        // 查询
        let query = target.to_lowercase();
        // 获取并报告消息
        let result = sav_dispatch(&mut self.reasoner, query, path);
        // 消息分派 | 🚩【2024-08-18 00:56:40】现在需要特殊考虑
        match result {
            // 正常信息⇒报告消息 // ! 一般不会是「COMMENT」注释
            // * 🎯【2024-08-18 00:57:34】用于锁定「格式化『保存回调』」的消息类型
            Ok(output) => self.reasoner.report(output),
            // 错误信息⇒报告error
            Err(message) => self.reasoner.report_error(message),
        }
    }

    /// 处理指令[`Cmd::LOA`]
    fn cmd_loa(&mut self, target: String, data: String) {
        // 查询
        let query = target.to_lowercase();
        // 获取并报告消息
        let result = loa_dispatch(&mut self.reasoner, query, data);
        self.report_result(result)
    }
}

/// 专用于指令[`Cmd::HLP`]的处理函数
mod cmd_hlp;
use cmd_hlp::*;

/// 专用于指令[`Cmd::INF`]的处理函数
mod cmd_inf;
use cmd_inf::*;

/// 专用于指令[`Cmd::SAV`]的处理函数
mod cmd_sav;
pub use cmd_sav::public::*;
use cmd_sav::*;

/// 专用于指令[`Cmd::LOA`]的处理函数
mod cmd_loa;
use cmd_loa::*;
