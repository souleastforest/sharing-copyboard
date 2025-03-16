

# Sharing-Copyboard 需求文档

---

## 一、项目概述
实现跨平台（macOS/Windows）的剪贴板同步工具，支持便签式内容管理，提供实时同步、多端操作和本地化数据缓存。

---

## 二、核心需求
### 1. 基础功能
- **剪贴板同步**
  - 自动监听剪贴板变化（支持手动关闭）
  - 复制内容自动存入云端并同步其他设备
  - 可选仅同步文本内容（过滤图片/文件）

- **便签管理** 
  - 增删改查（支持批量删除）
  - 便签重命名（默认"未命名"+时间戳）
  - 按创建时间倒序排列（支持手动置顶）

### 2. 扩展功能
- 离线缓存（断网时保存至本地）
- 快捷搜索（标题/内容关键词）
- 多设备绑定（同一账号最多5设备）
- 数据加密（AES-256传输与存储）

### 3. 账户系统
- **用户注册**
  - 支持邮箱注册
  - 密码强度校验
  - 邮箱验证码确认
- **用户登录**
  - 账号密码登录
  - 记住登录状态
  - 多设备登录管理
- **账户管理**
  - 修改密码
  - 找回密码
  - 账户信息设置

---

---

## **技术选型调整方案**

### **核心架构**
| 模块                | 技术方案                           | 优势说明                                                                 |
|---------------------|----------------------------------|--------------------------------------------------------------------------|
| **桌面框架**         | Tauri (Rust + Web前端)            | 比 Electron 更轻量（安装包可控制在 10MB 内），内存占用低，无 Node.js 依赖     |
| **剪贴板监听**       | `tauri-plugin-clipboard`          | 官方插件，支持 macOS/Windows 的剪贴板读写监听                               |
| **本地存储**         | SQLx + SQLite                     | 通过 Rust 原生操作 SQLite，支持异步读写                                      |
| **网络同步**         | Rust + **自建同步服务** 或 Supabase | 若需完全控制数据：用 Rust 实现 gRPC/WebSocket 服务；快速开发可用 Supabase 替代 Firebase |
| **前端UI**           | React + Ant Design               | 沿用原有设计，通过 `@tauri-apps/api` 与 Rust 交互                            |
| **加密方案**         | RustCrypto (AES-GCM-SIV)          | 比 JS 实现更安全的加密，支持硬件加速                                         |
| **打包工具**         | Tauri CLI                        | 自动生成签名安装包（Windows MSI/macOS DMG）                                  |

---

## **关键技术实现细节**

### 1. **剪贴板监听优化**
```rust
// 在 Rust 后端实现防抖监听
use tauri::Manager;
use std::time::Duration;

#[tauri::command]
async fn start_clipboard_monitor(app: tauri::AppHandle) {
    let mut last_content = String::new();
    loop {
        let content = app.clipboard().read_text().await.unwrap_or_default();
        if !content.is_empty() && content != last_content {
            app.emit_all("clipboard_update", content).unwrap();
            last_content = content;
        }
        tokio::time::sleep(Duration::from_millis(500)).await;
    }
}
```

### 2. **跨平台差异处理**
- **macOS 权限**：需在 `tauri.conf.json` 声明权限：
  ```json
  {
    "tauri": {
      "macOSPrivateApi": true,
      "bundle": {
        "resources": ["NSPasteboardUsageDescription: 需要访问剪贴板以同步内容"]
      }
    }
  }
  ```

### 3. **数据同步策略**
```rust
// 使用 CRDT 数据结构解决多端冲突（示例简化版）
#[derive(Debug, Serialize, Deserialize)]
struct ClipboardItem {
    id: Uuid,
    content: String,
    timestamp: i64,  // 使用物理时钟 + 逻辑时钟混合
    device_id: String,
}

// 同步优先级：timestamp > device_id 字典序
fn conflict_resolver(a: &ClipboardItem, b: &ClipboardItem) -> ClipboardItem {
    if a.timestamp > b.timestamp || (a.timestamp == b.timestamp && a.device_id > b.device_id) {
        a.clone()
    } else {
        b.clone()
    }
}
```

---

## **调整后的开发计划（6-8周）**

### **阶段一：核心模块搭建（3周）**
1. **Tauri 项目初始化**
   - 配置 Rust 工具链
   - 实现 Rust ↔ JS 通信基础框架

2. **剪贴板引擎**
   - 实现防抖监听（500ms）
   - 文本内容过滤（正则表达式去重）

3. **本地数据库**
   - 使用 `sqlx` 创建 SQLite 表结构
   - 实现离线缓存队列

### **阶段二：同步服务（2周）**
- **Option A - 自建服务**：
  用 Actix-Web 实现 REST API + WebSocket 双通道同步
- **Option B - Supabase**：
  直接调用 Supabase 的 Realtime Database

### **阶段三：联调测试（2周）**
- 压力测试：模拟 10 设备同时操作
- 安全审计：Clipboard 内容加密传输验证
- 异常处理测试：断网恢复/冲突合并

---

## **风险与应对**

### 1. **Rust 学习曲线**
- **应对**：优先使用 `tauri-plugin-*` 官方插件简化开发
- **推荐工具链**：`cargo-edit`（依赖管理）+ `rust-analyzer`（IDE 支持）

### 2. **剪贴板性能问题**
- **优化方案**：在 Rust 层实现内容哈希比对，避免重复同步相同内容

### 3. **多端同步延迟**
- **实测数据**：局域网内延迟可压到 200ms 内，公网依赖 WebSocket 长连接

---

## **Electron vs Tauri 对比**
| 指标                | Electron          | Tauri + Rust       |
|---------------------|-------------------|--------------------|
| 安装包体积          | ~80MB (含 Chromium) | ~5MB (系统 WebView) |
| 内存占用            | 150MB+            | 30MB~50MB          |
| 剪贴板监听稳定性    | 依赖第三方库       | 官方插件维护       |
| 多线程支持          | Worker 有限       | 原生支持 async/多线程 |

如果追求极致的性能和轻量化，Tauri 是更好的选择。需要补充其他细节可以继续探讨。