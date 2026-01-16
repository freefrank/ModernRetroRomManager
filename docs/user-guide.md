# RetroRomManager 用户指南

## 快速入门

### 安装

#### Windows
1. 下载 `RetroRomManager_x.x.x_x64.msi`
2. 双击运行安装程序
3. 按提示完成安装

#### macOS
1. 下载 `RetroRomManager_x.x.x_macos.dmg`
2. 拖拽到 Applications 文件夹

#### Linux
```bash
# Debian/Ubuntu
sudo dpkg -i retro-rom-manager_x.x.x_amd64.deb

# Arch
yay -S retro-rom-manager
```

---

## 添加 ROM 库

1. 点击左侧菜单 **"库管理"**
2. 点击 **"添加目录"**
3. 选择你的 ROM 目录
4. 软件会自动扫描并识别 ROM

---

## 导入现有数据

### 从 EmulationStation 导入

1. 进入 **设置 > 导入/导出**
2. 选择 **"导入 gamelist.xml"**
3. 定位到你的 `gamelist.xml` 文件
4. 确认导入

### 支持的格式

| 格式 | 文件 | 来源 |
|------|------|------|
| EmulationStation | gamelist.xml | RetroPie, ES-DE |
| metadata.txt | metadata.txt | Pegasus, Recalbox, Batocera |
| LaunchBox | *.xml | LaunchBox |
| RetroArch | *.lpl | RetroArch |

---

## Scraping 游戏信息

### 配置 API 密钥

1. 进入 **设置 > Scraper**
2. 添加你的 API 密钥:
   - IGDB (需要 Twitch 账号)
   - SteamGridDB (免费注册获取)
   - TheGamesDB
   - MobyGames
   - AI Scraper (可选 OpenAI/Claude API Key，或使用本地 Ollama)

### 单个游戏 Scrape

1. 右键点击游戏
2. 选择 **"Scrape"**
3. 确认或编辑搜索结果
4. 选择要下载的资产

### 批量 Scrape

1. 选择多个游戏 (Ctrl/Cmd + 点击)
2. 右键 > **"批量 Scrape"**
3. 配置选项后开始

---

## 常用快捷键

| 快捷键 | 功能 |
|--------|------|
| `Ctrl+F` | 全局搜索 |
| `Ctrl+R` | 刷新 |
| `Ctrl+E` | 导出 |
| `Del` | 删除选中 |
| `F2` | 重命名 |
| `F5` | Scrape 选中 |

---

## 常见问题

### Q: ROM 无法识别？
A: 尝试以下步骤:
1. 检查文件是否完整
2. 手动选择系统类型
3. 使用 Hash 搜索

### Q: Scrape 速度慢？
A: 建议:
1. 注册付费 API 账户
2. 调整并发请求数
3. 使用缓存功能
