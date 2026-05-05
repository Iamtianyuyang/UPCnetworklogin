# UPC 校园网登录工具 Rust 重写 实现计划

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** 重写 UPC 校园网登录工具，增加配置文件、自动重连、守护进程、终端美化功能。

**Architecture:** 子命令架构（login/logout/daemon/config），模块化拆分为 7 个文件，配置文件使用 TOML 格式，守护进程使用信号处理 + 指数退避重连。

**Tech Stack:** Rust 2021, reqwest (blocking), clap 4 (derive), serde/serde_json/toml, anyhow, colored, indicatif, local-ip-address, dirs, signal-hook

---

### Task 1: 项目骨架 + Cargo.toml 依赖

**Files:**
- Modify: `Cargo.toml`
- Create: `src/main.rs`, `src/config.rs`, `src/daemon.rs`, `src/network.rs`, `src/output.rs`, `src/portal.rs`, `src/transcode.rs`

- [ ] **Step 1: 更新 Cargo.toml 添加新依赖**
- [ ] **Step 2: 写入 src/main.rs 子命令定义 + 分发逻辑**
- [ ] **Step 3: 创建所有模块的空壳文件（最小编译通过）**
- [ ] **Step 4: cargo build 验证编译**
- [ ] **Step 5: 提交**

---

### Task 2: transcode 模块

**Files:**
- Modify: `src/transcode.rs`

- [ ] **Step 1: 实现 URL 双编码转义函数（从现有代码移植）**
- [ ] **Step 2: cargo build 验证**
- [ ] **Step 3: 提交**

---

### Task 3: network 模块

**Files:**
- Modify: `src/network.rs`

- [ ] **Step 1: 实现 get_interface_ip 和 resolve_base_url（从现有代码移植）**
- [ ] **Step 2: cargo build 验证**
- [ ] **Step 3: 提交**

---

### Task 4: portal 模块

**Files:**
- Modify: `src/portal.rs`

- [ ] **Step 1: 实现 build_client, get_query_string, check_needs_login, login, logout**
- [ ] **Step 2: cargo build 验证**
- [ ] **Step 3: 提交**

---

### Task 5: output 模块

**Files:**
- Modify: `src/output.rs`

- [ ] **Step 1: 实现彩色输出函数（print_login_header, print_logging_in, print_login_success/failure, print_logout 系列等）**
- [ ] **Step 2: cargo build 验证**
- [ ] **Step 3: 提交**

---

### Task 6: config 模块

**Files:**
- Modify: `src/config.rs`

- [ ] **Step 1: 实现 Config 结构体、load()、init_config()、show_config()**
- [ ] **Step 2: cargo build 验证**
- [ ] **Step 3: 提交**

---

### Task 7: daemon 模块

**Files:**
- Modify: `src/daemon.rs`

- [ ] **Step 1: 实现守护进程循环：信号捕获、网络检测、自动登录、指数退避**
- [ ] **Step 2: cargo build 验证**
- [ ] **Step 3: 提交**

---

### Task 8: main.rs 完善 + 集成

**Files:**
- Modify: `src/main.rs`

- [ ] **Step 1: 完善 main.rs 中所有子命令的分发逻辑，确保与各模块接口对齐**
- [ ] **Step 2: cargo build 验证**
- [ ] **Step 3: 更新 README.md**
- [ ] **Step 4: 提交**
