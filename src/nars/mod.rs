//! NARS中有关「主控」的内容
//! * 🚩【2024-05-02 21:51:38】此处将会对接[NAVM.rs](navm)与[BabelNAR.rs](babel_nar)
//!   * ⚠️后续布局会和OpenNARS有很大不同
//!
//! # 📄OpenNARS `main`
//!
//! This package contains the top classes of the system.
//!
//! `NARS`: defines the application and applet.
//!
//! `Reasoner`: controls the interaction between the memory and the communication channels.
//!
//! # 📄OpenNARS `main_nogui`
//!
//! This package contains the top classes of the system.
//!
//! `NARSBatch`: defines the application.
//!
//! `Parameters`: collects all system parameters, which can be edited before compiling.
//!
//! `CommandLineParameters`: system parameters that used in the command-line version
//!
//! `ReasonerBatch`: controls the interaction between the memory and the communication channels.

nar_dev_utils::mods! {
    // 超参数
    pub use parameters;
    // 推理器
    pub use reasoner;
}
