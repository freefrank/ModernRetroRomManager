# ScreenScraper WebAPI v2 速查

> 本文档是 ModernRetroRomManager 的离线开发速查，不代替 ScreenScraper 官方文档。  
> 官方文档：<https://www.screenscraper.fr/webapi2.php?alpha=0&numpage=0>  
> 最后核对：2026-07-13

## 基础请求

- API 根地址：`https://api.screenscraper.fr/api2/`
- 建议输出：`output=json`
- 每个请求都需要软件开发者鉴权：`devid`、`devpassword`、`softname`
- 需要成员额度或成员资料时增加：`ssid`、`sspassword`
- 开发者密码和成员密码不得写入日志、URL 错误信息或此文档。

ModernRetroRomManager 使用的 `softname` 为 `ModernRetroRomManager`。

## 账户与连接测试

### `ssuserInfos.php`

用于验证开发者凭据和成员账号，不应使用游戏搜索来替代连接测试。

必填参数：

- `devid`
- `devpassword`
- `softname`
- `output`
- `ssid`
- `sspassword`

JSON 成功响应的 `response` 通常包含：

- `serveurs`：ScreenScraper 服务状态
- `ssuser`：成员资料、线程和请求额度

`ssuser` 的常用字段包括 `id`、`numid`、`niveau`、`maxthreads`、`requeststoday`、`maxrequestspermin` 和 `maxrequestsperday`。

## 名称搜索

### `jeuRecherche.php`

按游戏名称搜索，最多返回 30 个按匹配概率排列的游戏。

业务参数：

- `recherche`：搜索关键词
- `systemeid`：ScreenScraper 数字平台 ID，可选但建议传入

JSON 返回结构：

```text
response
├─ serveurs
├─ ssuser
└─ jeux[]
   ├─ id
   ├─ noms[]        { region, text }
   ├─ synopsis[]    { langue, text }
   ├─ dates[]       { region, text }
   ├─ genres[]
   └─ medias[]      { type, region, url, ... }
```

`jeuRecherche.php` 是名称搜索接口；`jeuInfos.php?romnom=...` 是 ROM 识别，不能当作普通关键词搜索。

## 游戏详情与 ROM 识别

### `jeuInfos.php`

用于获取游戏详情、媒体和 ROM 匹配结果。

常用查询方式：

- 已知游戏 ID：`gameid`
- ROM 文件识别：`systemeid` + `romnom`，可附加 `romtaille`、`romtype`
- Hash 精确识别：`crc`、`md5` 或 `sha1`，建议同时提供 `systemeid`
- 光盘游戏序列号：`serialnum`

JSON 主数据位于 `response.jeu`。名称搜索返回的游戏字段与此接口基本一致，但不包含完整 ROM 列表。

## 媒体类型

本项目当前常用的 ScreenScraper `medias[].type` 映射：

| ScreenScraper | 项目资产类型 |
| --- | --- |
| `box-2D`, `box-2d` | `boxfront` |
| `box-3D`, `box-3d` | `box3d` |
| `box-back`, `box-arriere` | `boxback` |
| `ss`, `screenshot` | `screenshot` |
| `sstitle` | `titlescreen` |
| `wheel`, `wheel-hd` | `logo` |
| `video`, `video-normalized` | `video` |
| `manuel` | `manual` |

## HTTP 错误与限流

| 状态码 | 含义 | 处理建议 |
| --- | --- | --- |
| `400` | 参数或请求无效 | 检查必填参数和平台 ID |
| `401` / `403` | 成员或开发者鉴权失败 | 区分账号凭据与应用凭据，不回显密码 |
| `426` | 抓取软件被拉黑或版本过旧 | 停止重试并升级客户端 |
| `429` | 并发线程或每分钟请求数超限 | 降低 threads/rate limit，指数退避 |
| `430` | 当日抓取额度用尽 | 停止该 Provider，保留已完成结果 |
| `431` | 当日未识别 ROM 额度用尽 | 停止无效查询，改善名称/Hash 匹配 |

429 不应影响其他 Provider；单个 Provider 受限时可继续使用其他数据源。430/431 不能当作“未找到游戏”吞掉。

## 实现检查清单

- URL 查询参数必须编码。
- `reqwest` 错误可能包含完整 URL，对外错误不得直接传递原文。
- 测试连接使用 `ssuserInfos.php`。
- 名称搜索使用 `jeuRecherche.php?recherche=...`。
- 详情、Hash 或 ROM 识别使用 `jeuInfos.php`。
- 将 429/430/431 作为限流/额度错误传给 Provider failsafe，不得当作空结果。
- 缓存搜索、metadata 和媒体结果，避免浪费成员额度。
