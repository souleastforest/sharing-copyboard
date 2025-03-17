use thiserror::Error;

#[derive(Error, Debug)]
pub enum AppError {
    #[error("数据库错误: {0}")]
    DatabaseError(String),
    
    #[error("未找到: {0}")]
    NotFound(String),
    
    #[error("无效的凭据")]
    InvalidCredentials,
    
    #[error("加密错误: {0}")]
    CryptoError(String),
    
    #[error("无效的数据: {0}")]
    InvalidData(String),
    
    // 其他错误类型...
}