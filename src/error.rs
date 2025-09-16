//! 错误处理模块
//! 
//! 定义了 RustPress 中使用的错误类型

use std::fmt;

/// RustPress 的错误类型
#[derive(Debug)]
pub enum Error {
    /// IO 错误
    Io(std::io::Error),
    /// 配置文件解析错误
    Config(String),
    /// 模板渲染错误
    Template(tera::Error),
    /// Markdown 解析错误
    Markdown(String),
    /// YAML 解析错误
    Yaml(serde_yaml::Error),
    /// JSON 序列化错误
    Json(serde_json::Error),
    /// TOML 解析错误
    Toml(toml::de::Error),
    /// 网络服务器错误
    Server(String),
    /// 通用错误
    Other(String),
}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Error::Io(err) => write!(f, "IO 错误: {}", err),
            Error::Config(msg) => write!(f, "配置错误: {}", msg),
            Error::Template(err) => write!(f, "模板错误: {}", err),
            Error::Markdown(msg) => write!(f, "Markdown 解析错误: {}", msg),
            Error::Yaml(err) => write!(f, "YAML 解析错误: {}", err),
            Error::Json(err) => write!(f, "JSON 序列化错误: {}", err),
            Error::Toml(err) => write!(f, "TOML 解析错误: {}", err),
            Error::Server(msg) => write!(f, "服务器错误: {}", msg),
            Error::Other(msg) => write!(f, "错误: {}", msg),
        }
    }
}

impl std::error::Error for Error {}

impl From<std::io::Error> for Error {
    fn from(err: std::io::Error) -> Self {
        Error::Io(err)
    }
}

impl From<tera::Error> for Error {
    fn from(err: tera::Error) -> Self {
        Error::Template(err)
    }
}

impl From<serde_yaml::Error> for Error {
    fn from(err: serde_yaml::Error) -> Self {
        Error::Yaml(err)
    }
}

impl From<serde_json::Error> for Error {
    fn from(err: serde_json::Error) -> Self {
        Error::Json(err)
    }
}

impl From<toml::de::Error> for Error {
    fn from(err: toml::de::Error) -> Self {
        Error::Toml(err)
    }
}

impl From<anyhow::Error> for Error {
    fn from(err: anyhow::Error) -> Self {
        Error::Other(err.to_string())
    }
}

/// RustPress 的结果类型
pub type Result<T> = std::result::Result<T, Error>;