# 卡带产品号数据来源

`md-serial.csv`、`n64-serial.csv`、`sfc-serial.csv` 由
[libretro/libretro-database](https://github.com/libretro/libretro-database) 的 `metadat/serial`
数据生成，仅保留产品号、发行名称与区域字段。

- 上游数据获取日期：2026-07-12
- 许可证：Creative Commons Attribution-ShareAlike 4.0 International（CC BY-SA 4.0）
- 生成脚本：`scripts/build_cartridge_serial_db.py`

本项目删除当前识别流程不使用的 CRC 字段，游戏、区域及语言版本的发行覆盖不作裁剪。
