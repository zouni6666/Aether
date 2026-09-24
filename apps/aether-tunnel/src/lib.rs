#![allow(clippy::large_enum_variant)]

// Tunnel 的运行模块作为库暴露给独立集成测试使用；生产二进制仍由
// src/main.rs 负责命令行解析，避免端到端测试把 Gateway dev-dependency
// 带进 Workspace Rest 的默认测试目标。
pub mod app;
pub mod config;
pub mod egress_proxy;
pub mod hardware;
mod net;
pub mod registration;
pub mod runtime;
pub mod setup;
pub mod state;
pub mod target_filter;
pub mod tunnel;
pub mod upstream_client;
