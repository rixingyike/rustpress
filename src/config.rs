//! 配置管理模块
//! 
//! 处理配置文件的读取和解析

use crate::error::{Error, Result};
use std::path::Path;

/// 站点配置
#[derive(Debug, Clone)]
pub struct Config {
    /// 原始配置数据
    pub data: toml::Value,
}

impl Config {
    /// 从文件加载配置
    pub fn from_file<P: AsRef<Path>>(path: P) -> Result<Self> {
        let path = path.as_ref();
        // 直接读取传入的配置路径（项目根 config.toml）
        let content = std::fs::read_to_string(path)
            .map_err(|e| Error::Config(format!("无法读取配置文件 {:?}: {}", path, e)))?;
        
        let data: toml::Value = toml::from_str(&content)
            .map_err(|e| Error::Config(format!("配置文件格式错误: {}", e)))?;
        
        Ok(Config { data })
    }
    
    /// 获取站点配置
    pub fn site(&self) -> toml::Value {
        self.data.get("site")
            .cloned()
            .unwrap_or_else(|| toml::Value::Table(toml::map::Map::new()))
    }
    
    /// 获取分类法配置
    pub fn taxonomies(&self) -> Option<&toml::Value> {
        self.data.get("taxonomies")
    }
    
    /// 获取主题配置
    pub fn theme(&self) -> Option<&toml::Value> {
        self.data.get("theme")
    }

    /// 获取主题名称（默认 "default"）
    pub fn theme_name(&self) -> String {
        self.theme()
            .and_then(|v| v.get("name"))
            .and_then(|v| v.as_str())
            .unwrap_or("default")
            .to_string()
    }
    
    /// 获取作者配置
    pub fn author(&self) -> Option<toml::Value> {
        self.site().get("author").cloned()
    }
    
    /// 获取社交链接配置
    pub fn social(&self) -> Option<toml::Value> {
        self.site().get("social").cloned()
    }
}
