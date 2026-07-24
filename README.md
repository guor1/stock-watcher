# 盯盘小助手

置顶悬浮条形式的股票行情监控桌面小工具，数据来自腾讯财经实时接口。

## 功能

- 大盘指数（上证/深成/创业板）实时展示
- 自选股管理（增删）
- 涨跌预警（价格/涨跌幅阈值触发 Windows 系统通知）
- 老板键（全局快捷键 `Ctrl+`` 一键隐藏/显示）
- 智能刷新（交易时段 3s，非交易时段 60s）

## 技术栈

- **Tauri 2** — Rust 后端 + WebView 前端
- **前端** — 原生 HTML/CSS/JS（无框架）
- **数据源** — 腾讯财经 `qt.gtimg.cn`（GBK 编码，Rust 端解码）

## 开发

```bash
npm install
npm run dev      # 启动 Tauri 开发模式（首次编译需 3-4 分钟）
```

## 构建

```bash
npm run build    # 产出 Windows 安装包 → src-tauri/target/release/bundle/
```

## 使用

1. 启动后桌面顶部出现置顶悬浮条
2. 点击 ⚙ 按钮打开设置面板，添加自选股代码
3. 在设置中配置预警规则
4. 按 `Ctrl+`` 隐藏/显示窗口
