**[English](README_EN.md)** | 中文

# 键盘侠 (Keyman)

魔兽争霸 3 改键助手 - 自定义技能和物品栏快捷键映射工具。

## 功能特性

- 自定义技能和物品栏键位映射（动态添加/删除）
- 多方案管理（创建、切换、重命名、删除）
- 自动检测游戏窗口，仅游戏前台时生效
- F11 游戏内快速切换方案，F12 暂停/恢复改键
- 屏蔽 Win 键防止误触
- 中英双语界面，跟随系统语言，可手动切换
- 跨平台支持（Windows、macOS、Linux）

## 安装

### Windows

**系统要求：Windows 10 v1809 或更高版本，需支持 DirectX 11 的 GPU。**

从 [Releases](../../releases) 下载最新版 `keyman.exe`，双击运行。

### macOS

首次运行需在「系统设置 > 隐私与安全性 > 辅助功能」中授权键盘监控。

### Linux

```bash
sudo usermod -a -G input $USER
```

## 构建

```bash
cargo build --release
```

Windows 输出：`target/release/keyman.exe`

## 使用方法

1. 在技能/物品栏表格中点击格子，按下想要映射的键
2. 按 Delete/Backspace 可清除映射
3. 底部可勾选屏蔽 Win 键
4. 游戏中按 F12 暂停/恢复改键，F11 切换方案
5. 右上角按钮可切换中/英文界面

## 项目结构

```
src/main.rs               # 应用入口
crates/
  keyman-core/            # 映射引擎、配置管理、国际化
  keyman-hook/            # 键盘钩子（跨平台）
  keyman-detect/          # 游戏进程和窗口检测
  keyman-ui/              # 用户界面（GPUI）
assets/                   # 应用图标
```

## 技术栈

- [Rust](https://www.rust-lang.org/)
- [GPUI](https://github.com/zed-industries/zed) - GPU 加速 UI 框架
- [gpui-component](https://github.com/huacnlee/gpui-component) - UI 组件库

## 许可证

MIT License
