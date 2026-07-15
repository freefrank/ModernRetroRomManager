# Privacy Policy / 隐私政策

**Effective date: July 14, 2026**
**生效日期：2026 年 7 月 14 日**

[English](#english) | [简体中文](#简体中文)

## English

### 1. Overview

ModernRetroRomManager ("MRRM", "the application") is an open-source desktop application for organizing local ROM libraries, matching game metadata, downloading artwork, translating metadata, and exporting frontend-compatible library files.

MRRM does not operate a developer-controlled server. The developer does not collect application telemetry, analytics, crash reports, ROM files, account credentials, or other user data. Processing is performed locally except when you choose to use a third-party online service.

### 2. Data processed and stored locally

MRRM may process and store the following data on your device:

- ROM library locations, folder names, filenames, file sizes, platform information, and file hashes;
- game metadata, artwork, screenshots, videos, manuals, search results, provider identifiers, and caches;
- application preferences, library configuration, scraper settings, and provider statistics;
- credentials that you enter for third-party services, including usernames, passwords, and API keys;
- custom AI endpoint, model, target-language settings, and translation previews; and
- archive passwords entered for an extraction operation. Archive passwords are used locally for that operation and are not sent to the MRRM developer.

Local application data is normally stored in the MRRM configuration directory under your operating system's application-data location. If a `config` directory exists beside a portable executable, MRRM uses that directory instead. Files exported to a ROM library or another location selected by you remain in that location.

Credentials are stored in the local application configuration. MRRM does not claim to encrypt those configuration files. Access to them depends on your device, operating-system account, filesystem permissions, backup software, and synchronization services. Do not use credentials on a shared or untrusted device unless you accept those risks.

### 3. Data sent to third-party services

MRRM connects to an online service only when a feature requiring that service is enabled or invoked. Depending on the provider and operation, MRRM may send:

- a game title, platform, provider game identifier, filename, file size, and ROM hashes;
- a ScreenScraper member username and password together with the application's ScreenScraper developer credentials;
- a SteamGridDB or TheGamesDB API key and game search information;
- game metadata, translation instructions, model name, and an API key to the OpenAI-compatible endpoint configured by you; and
- normal network information automatically visible to the receiving service, such as your IP address and request headers, when searching or downloading media.

Current integrations include ScreenScraper, SteamGridDB, TheGamesDB, user-configured OpenAI-compatible endpoints, and content hosts returned by those services. Optional data updates may contact GitHub. These services are independent third parties and process information under their own terms and privacy policies. MRRM does not control their retention, logging, security, or international transfer practices.

MRRM does not upload the contents of your ROM files to the developer. Search providers may receive ROM-derived identifiers such as hashes when the corresponding matching feature is used.

### 4. No sale, advertising, or developer profiling

The developer does not sell, rent, share, or use your data for advertising or behavioral profiling. MRRM contains no developer-operated advertising or analytics service.

### 5. Retention and deletion

MRRM retains local settings, caches, metadata, and downloaded assets until you remove them. You can remove credentials through the application where that control is available, or delete the MRRM configuration directory. Uninstalling the packaged application may remove its packaged application data, but files exported to locations you selected and portable `config` directories may need to be deleted separately.

Requests already sent to a third-party service are subject to that service's retention and deletion policies. Contact the relevant provider for requests concerning data held by that provider.

### 6. Children's privacy

MRRM is a general-purpose library-management tool and is not directed to children. The developer does not knowingly collect personal information from children. Use of third-party services remains subject to their age requirements and terms.

### 7. Security

MRRM uses the network protocols provided by the configured service. You are responsible for selecting trustworthy providers and endpoints, preferably HTTPS endpoints, protecting your device and credentials, and reviewing the privacy terms of each service you enable. No storage or transmission method can be guaranteed to be completely secure.

### 8. Changes to this policy

This policy may be updated when MRRM's features or data practices change. The effective date at the top of this document identifies the latest revision. Material changes will be documented in the project repository or release notes.

### 9. Contact

For privacy questions or requests concerning MRRM, open an issue at:

<https://github.com/freefrank/ModernRetroRomManager/issues>

For information sent directly to a third-party provider, contact that provider.

---

## 简体中文

### 1. 概述

ModernRetroRomManager（以下简称“MRRM”或“本应用”）是一款开源桌面应用，用于整理本地 ROM 游戏库、匹配游戏元数据、下载美术资源、翻译元数据，以及导出兼容前端的游戏库文件。

MRRM 不运营由开发者控制的服务器。开发者不收集应用遥测、分析数据、崩溃报告、ROM 文件、账户凭据或其他用户数据。除非你主动选择使用第三方在线服务，否则所有处理均在本地完成。

### 2. 本地处理和存储的数据

MRRM 可能在你的设备上处理并保存以下数据：

- ROM 游戏库位置、文件夹名称、文件名、文件大小、平台信息和文件哈希；
- 游戏元数据、美术资源、截图、视频、手册、搜索结果、Provider 标识和缓存；
- 应用偏好、游戏库配置、抓取设置和 Provider 统计；
- 你填写的第三方服务凭据，包括用户名、密码和 API Key；
- 自定义 AI 端点、模型、目标语言设置和翻译预览；
- 解压操作中输入的压缩包密码。压缩包密码仅在本地用于该次操作，不会发送给 MRRM 开发者。

应用本地数据通常保存在操作系统应用数据目录下的 MRRM 配置目录中。如果 portable 程序旁已经存在 `config` 目录，MRRM 会优先使用该目录。你导出到 ROM 游戏库或其他自选位置的文件将继续保留在相应位置。

凭据保存在本地应用配置中。MRRM 不声明这些配置文件经过加密；其访问安全性取决于你的设备、操作系统账户、文件系统权限、备份软件和同步服务。除非你接受相关风险，否则请勿在共享或不可信设备上保存凭据。

### 3. 发送给第三方服务的数据

只有在启用或主动调用需要联网的功能时，MRRM 才会连接相应在线服务。根据 Provider 和具体操作，MRRM 可能发送：

- 游戏名称、平台、Provider 游戏标识、文件名、文件大小和 ROM 哈希；
- ScreenScraper 会员用户名和密码，以及应用内置的 ScreenScraper 开发者凭据；
- SteamGridDB 或 TheGamesDB API Key 和游戏搜索信息；
- 游戏元数据、翻译指令、模型名称和 API Key，发送至你配置的 OpenAI-compatible 端点；
- 搜索或下载媒体时，接收服务通常能够看到的网络信息，例如你的 IP 地址和请求头。

当前集成包括 ScreenScraper、SteamGridDB、TheGamesDB、用户配置的 OpenAI-compatible 端点，以及这些服务返回的内容托管地址。可选的数据更新功能可能访问 GitHub。这些服务是独立第三方，并按照各自的条款和隐私政策处理信息。MRRM 无法控制其保留、日志记录、安全或跨境传输行为。

MRRM 不会把 ROM 文件内容上传给开发者。使用相应匹配功能时，搜索服务可能收到由 ROM 计算得到的哈希等标识。

### 4. 不出售数据、不投放广告、不建立用户画像

开发者不会出售、出租、分享你的数据，也不会将其用于广告或行为画像。MRRM 不包含由开发者运营的广告或分析服务。

### 5. 保留与删除

本地设置、缓存、元数据和下载资源会保留到你主动删除为止。你可以通过应用中提供的相关功能移除凭据，也可以删除 MRRM 配置目录。卸载打包版应用可能会移除其打包应用数据，但导出到自选位置的文件以及 portable `config` 目录可能需要另行删除。

已经发送给第三方服务的请求受该服务的数据保留和删除政策约束。有关第三方持有数据的请求，请直接联系相应 Provider。

### 6. 儿童隐私

MRRM 是通用游戏库管理工具，并非面向儿童设计。开发者不会有意收集儿童的个人信息。第三方服务的使用仍须遵守其年龄要求和服务条款。

### 7. 安全

MRRM 使用所配置服务提供的网络协议。你有责任选择可信的 Provider 和端点（建议使用 HTTPS）、保护设备和凭据，并查看所启用服务的隐私条款。任何存储或传输方式都无法保证绝对安全。

### 8. 本政策的变更

当 MRRM 功能或数据处理方式发生变化时，本政策可能更新。文档顶部的生效日期表示最新修订时间。重大变化将记录在项目仓库或发行说明中。

### 9. 联系方式

如对 MRRM 的隐私处理有疑问或请求，请在以下地址提交 issue：

<https://github.com/freefrank/ModernRetroRomManager/issues>

如信息已直接发送给第三方 Provider，请联系相应 Provider。
