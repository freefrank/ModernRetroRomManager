# 3DS ROM 整理流程

3DS 样本以 RAR/7Z 内的 CIA 为主。CIA 是安装容器，和可直接启动的 `.3ds`/CCI 卡带镜像不是同一种存储格式；解包压缩文件也不等于解密游戏内容。

## 推荐目录

```text
3DS/
└── 游戏中文名 [Title ID]/
    └── game.cia
```

保留原始压缩包作为源文件，整理结果写到另一块目录，避免批量操作损坏唯一副本。Title ID 用于去重；同一 Title ID 的本体、更新和 DLC 不能互相覆盖。

## 先生成清单

```powershell
python scripts/organize_3ds_archives.py "G:\ROMS\3DSCH(OldmanEmu.net)" --report "G:\ROMS\3ds-report.csv"
```

默认只读取压缩包目录，不移动或解包文件。压缩包解开后才能继续读取 CIA Ticket/NCCH 元数据。

## 复制解包

确认清单后再执行：

```powershell
python scripts/organize_3ds_archives.py "G:\ROMS\3DSCH(OldmanEmu.net)" --report "G:\ROMS\3ds-report.csv" --output-dir "G:\ROMS\3DS-整理后" --apply
```

工具不会覆盖已经存在的目标文件，也不会删除源压缩包。

## 解密边界

CIA 内容通常仍有 3DS 加密层。模拟器需要何种格式取决于模拟器；实机安装和模拟器直接加载不是同一流程。项目不分发密钥，也不自动下载解密程序。需要解密时，应使用本人 3DS 导出的 `boot9`/密钥和合法备份，在整理副本上操作，完成后再让 MRRM 扫描。
