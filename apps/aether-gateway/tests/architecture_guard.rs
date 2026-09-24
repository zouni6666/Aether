//! 架构守卫独立测试目标。
//!
//! 从 lib 的 `cfg(test)` 巨型编译单元迁出：只做源码/manifest 字符串断言，
//! 不启动 AppState、不依赖 gateway 私有类型，用于压低 lib test 编译面与 rustc 峰值。
mod architecture;
