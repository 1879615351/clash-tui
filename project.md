# Clash TUI（Rust）终端客户端项目设计文档

## 1. 项目目标

开发一个基于 Rust 的跨平台 TUI Clash 客户端，类似于：

* [https://github.com/ratatui/ratatui](https://github.com/ratatui/ratatui)
* [https://github.com/clash-verge-rev/clash-verge-rev](https://github.com/clash-verge-rev/clash-verge-rev)

但采用：

* 纯终端 TUI 界面
* 支持 Linux / Windows
* 支持后台驻留运行
* 支持热键唤出/隐藏
* 支持 Clash 订阅
* 支持节点切换
* 支持系统代理
* 支持日志查看
* 支持连接管理
* 支持多 Clash Core
* 不用切换终端，立即在当前终端生效代理

推荐项目名称：

* Clash-TUI
* Clash-Term
* VergeTUI
* rat-clash
* clash-console

---

# 2. 技术栈设计

## 2.1 UI层

### 推荐：

* ratatui
* crossterm

原因：

* 跨平台
* Windows/Linux/macOS 支持优秀
* 性能好
* 异步兼容
* 社区活跃

---

## 2.2 异步运行时

推荐：

* tokio

用于：

* Clash API 通信
* websocket
* 后台任务
* 日志流
* 热更新
* 订阅下载

---

## 2.3 HTTP客户端

推荐：

* reqwest

用于：

* Clash REST API
* 下载订阅
* GitHub 更新检查

---

## 2.4 配置管理

推荐：

* serde
* serde_yaml
* toml

---

## 2.5 后台守护

Linux：

* daemonize
* systemd 集成

Windows：

* tray-icon
* windows-service

---

# 3. 整体架构

```text
┌──────────────────────────────┐
│           TUI UI             │
│         (ratatui)            │
└──────────────┬───────────────┘
               │
               ▼
┌──────────────────────────────┐
│        Application Core      │
│                              │
│  状态管理                    │
│  事件系统                    │
│  命令调度                    │
│  UI Router                   │
└──────────────┬───────────────┘
               │
     ┌─────────┴─────────┐
     ▼                   ▼
┌──────────────┐   ┌──────────────┐
│ Clash API    │   │ Config       │
│ Client       │   │ Manager      │
└──────────────┘   └──────────────┘
     │
     ▼
┌──────────────────────────────┐
│      Clash Core Runtime      │
│ (mihomo/clash-premium/meta)  │
└──────────────────────────────┘
```

---

# 4. 模块划分

## 4.1 tui/

负责：

* 页面渲染
* 键盘事件
* 状态显示
* 动画
* 布局

推荐页面：

```text
Dashboard
Proxies
Connections
Logs
Rules
Settings
Subscriptions
Profiles
Runtime
```

---

## 4.2 clash/

负责：

* Clash REST API
* websocket
* 配置热更新
* 节点切换
* 代理组操作

支持：

```text
GET /proxies
PUT /proxies/{name}
GET /connections
DELETE /connections
GET /logs
```

---

## 4.3 subscription/

负责：

* 订阅下载
* 自动更新
* yaml解析
* profile管理

支持：

```text
HTTP订阅
Base64订阅
本地配置
GitHub配置
```

---

## 4.4 runtime/

负责：

* Clash Core 生命周期
* 启动/停止
* 自动重启
* 后台运行
* PID 管理

支持：

```text
mihomo
clash-meta
clash-premium
sing-box（后续）
```

---

## 4.5 hotkey/

负责：

* 全局热键
* 唤出/隐藏 TUI
* 快捷切换代理

Linux：

```text
X11
Wayland
```

Windows：

```text
Win32 Hook
```

推荐库：

* global-hotkey

---

## 4.6 system_proxy/

负责：

* 设置系统代理
* PAC
* TUN

Linux：

```text
gsettings
environment
networkmanager
```

Windows：

```text
WinInet
Registry
```

---

# 5. UI设计

## 5.1 Dashboard

```text
┌ Clash-TUI ───────────────────────┐
│ Core: Running                    │
│ Mode: Rule                       │
│ Memory: 132MB                    │
│ Upload: 2.3MB/s                  │
│ Download: 10.8MB/s               │
│ Active Connections: 24           │
└──────────────────────────────────┘
```

---

## 5.2 Proxies页面

```text
┌ Proxy Groups ────────────────────┐
│ GLOBAL                           │
│ AUTO                             │
│ TELEGRAM                         │
│ OPENAI                           │
└──────────────────────────────────┘

┌ Nodes ───────────────────────────┐
│ HK-01  23ms                      │
│ JP-01  48ms                      │
│ SG-01  69ms                      │
└──────────────────────────────────┘
```

支持：

* Enter 切换节点
* 自动测速
* 延迟排序
* 收藏节点

---

## 5.3 Logs页面

```text
[INFO] Clash started
[INFO] Proxy changed -> JP-01
[WARN] Connection timeout
```

支持：

* 实时滚动
* 搜索
* 过滤

---

# 6. 后台运行设计（重点）

这是整个项目最关键的部分。

## 6.1 Linux

实现：

```bash
clash-tui daemon
```

功能：

* fork后台运行
* 保存PID
* 关闭终端继续运行
* UNIX Socket通信

推荐：

```rust
daemonize
```

通信：

```text
TUI Client <-> UNIX SOCKET <-> Daemon
```

---

## 6.2 Windows

实现：

```text
后台service + console client
```

推荐：

* windows-service
* named pipe

结构：

```text
TUI.exe
    │
    ▼
Background Service
    │
    ▼
Clash Core
```

---

# 7. 热键唤出/隐藏

类似：

```text
Ctrl + `
```

功能：

* 隐藏TUI
* 重新唤出
* 后台继续运行

实现：

Linux：

```text
X11 event hook
```

Windows：

```text
RegisterHotKey
```

---

# 8. 配置文件设计

推荐：

```toml
[core]
type = "mihomo"
path = "./core/mihomo"

[api]
port = 9090
secret = ""

[ui]
theme = "tokyo-night"

[hotkey]
toggle = "Ctrl+`"

[subscription]
auto_update = true
interval = 24
```

---

# 9. 推荐目录结构

```text
clash-tui/
├── src/
│   ├── main.rs
│   ├── app/
│   ├── tui/
│   ├── clash/
│   ├── runtime/
│   ├── subscription/
│   ├── hotkey/
│   ├── proxy/
│   ├── config/
│   ├── ipc/
│   └── utils/
│
├── assets/
├── themes/
├── configs/
├── scripts/
├── plugins/
│
├── Cargo.toml
└── README.md
```

---

# 10. Cargo.toml 推荐依赖

```toml
[dependencies]
ratatui = "0.29"
crossterm = "0.28"

tokio = { version = "1", features = ["full"] }

reqwest = { version = "0.12", features = ["json"] }

serde = { version = "1", features = ["derive"] }
serde_yaml = "0.9"
toml = "0.8"

anyhow = "1"
thiserror = "1"

directories = "5"

tracing = "0.1"
tracing-subscriber = "0.3"

daemonize = "0.5"

global-hotkey = "0.6"

tui-input = "0.11"

futures = "0.3"

tokio-tungstenite = "0.24"
```

---

# 11. 推荐运行模式

## 模式1：单体模式

```text
TUI + Clash Core
```

优点：

* 简单

缺点：

* 关闭终端退出

---

## 模式2：Client/Daemon模式（推荐）

```text
┌────────────┐
│ TUI Client │
└─────┬──────┘
      │ IPC
┌─────▼──────┐
│ Daemon     │
│ Clash Core │
└────────────┘
```

优点：

* 真后台
* 可热重连
* 更稳定
* 类似 clash-verge

推荐最终采用。

---

# 12. IPC设计（重要）

Linux：

```text
UNIX DOMAIN SOCKET
```

Windows：

```text
Named Pipe
```

统一抽象：

```rust
trait IPCTransport {
    async fn send();
    async fn recv();
}
```

---

# 13. Clash API设计

推荐封装：

```rust
pub struct ClashClient {
    client: reqwest::Client,
    base_url: String,
}
```

接口：

```rust
async fn get_proxies()
async fn switch_proxy()
async fn get_logs()
async fn get_connections()
```

---

# 14. 主题系统

支持：

```text
Tokyo Night
Catppuccin
Gruvbox
Nord
Dracula
```

实现：

```rust
pub struct Theme {
    background: Color,
    foreground: Color,
    accent: Color,
}
```

---

# 15. 插件系统（高级）

后期可支持：

```text
Lua
WASM
Dynamic Library
```

用于：

* 自定义节点排序
* AI自动选路
* 自动测速

---

# 16. 自动更新

支持：

```text
GitHub Release
Core自动更新
订阅自动更新
```

推荐：

```text
self_update
```

---

# 17. 推荐开发顺序

## 第一阶段

实现：

* TUI基础框架
* Clash API
* Dashboard
* Proxies

---

## 第二阶段

实现：

* Logs
* Connections
* Subscription

---

## 第三阶段

实现：

* 后台Daemon
* IPC
* 热键

---

## 第四阶段

实现：

* 系统代理
* TUN
* 插件

---

# 18. 推荐核心

优先推荐：

## Linux

* mihomo

## Windows

* mihomo + TUN

原因：

* 社区活跃
* 功能完整
* API稳定
* 支持TUN
* 支持Rule Provider

---

# 19. 关键难点

## 难点1

后台运行 + TUI重新附着

解决：

```text
Client/Daemon架构
```

---

## 难点2

Windows 热键

解决：

```text
Win32 API Hook
```

---

## 难点3

异步日志流

解决：

```text
tokio websocket
```

---

## 难点4

Terminal状态恢复

解决：

```rust
disable_raw_mode()
LeaveAlternateScreen
```

---

# 20. 推荐最终架构

最终推荐：

```text
┌─────────────────────┐
│     TUI Client      │
│    (ratatui)        │
└──────────┬──────────┘
           │ IPC
┌──────────▼──────────┐
│   Background Core   │
│                     │
│ Clash Runtime       │
│ Subscription        │
│ Proxy Manager       │
│ Hotkey Manager      │
│ System Proxy        │
└─────────────────────┘
```

---

# 21. 推荐额外功能

## 支持：

### 节点测速

```text
url-test
latency benchmark
```

### 流量统计

```text
upload/download graph
```

### OpenAI分流

```text
规则组快速切换
```

### 快速命令面板

类似：

```text
Ctrl+P
```

---

# 22. 推荐开源项目参考

## TUI

* [https://github.com/ratatui/ratatui/tree/main/examples](https://github.com/ratatui/ratatui/tree/main/examples)

---

## Clash GUI

* [https://github.com/clash-verge-rev/clash-verge-rev](https://github.com/clash-verge-rev/clash-verge-rev)

---

## Terminal UI

* [https://github.com/ClementTsang/bottom](https://github.com/ClementTsang/bottom)
* [https://github.com/jesseduffield/lazygit](https://github.com/jesseduffield/lazygit)

---

# 23. 最终建议

这个项目最合理的路线：

```text
TUI Client
        ↓
Background Daemon
        ↓
mihomo Core
```

不要：

```text
直接把所有东西塞进单个TUI进程
```

否则：

* 后台运行困难
* 热键困难
* terminal恢复困难
* Windows兼容困难

Client/Daemon 是最佳方案。
