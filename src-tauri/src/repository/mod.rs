pub mod user_repository;
pub mod session_repository;
pub mod clipboard_repository;
pub mod encryption_repository;
pub mod init;

// 重新导出初始化函数
pub use init::init_tables;