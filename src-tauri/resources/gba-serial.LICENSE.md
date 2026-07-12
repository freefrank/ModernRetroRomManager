# GBA Serial 数据来源

`gba-serial.csv` 由 [libretro/libretro-database](https://github.com/libretro/libretro-database)
中的 `Nintendo - Game Boy Advance.dat` 生成，仅保留 Serial、发行名称与区域字段。

- 上游数据版本：2026.05.02
- 上游提交：9aec58983a73ba4370ba6fd7c1b7d915ec56dda6
- 许可证：Creative Commons Attribution-ShareAlike 4.0 International（CC BY-SA 4.0）
- 生成命令：`python scripts/build_gba_serial_db.py <DAT文件> src-tauri/resources/gba-serial.csv`

本项目对数据进行的修改是删除当前识别流程不使用的哈希、文件大小和 DAT 描述字段；
游戏、区域及语言版本的全球发行覆盖不作裁剪。
