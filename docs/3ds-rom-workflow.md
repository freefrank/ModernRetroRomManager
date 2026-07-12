# 3DS ROM 整理流程

3DS 样本以 RAR/7Z 内的 CIA 为主。CIA 是安装容器，和可直接启动的 `.3ds`/CCI 卡带镜像不是同一种存储格式；解包压缩文件也不等于解密游戏内容。

MRRM 匹配 CIA 时只读取文件头前 64 KiB，从 Ticket 获取 Title ID；不会解密或计算整文件 Hash。更新（`0004000E`）和 DLC（`0004008C`）会归一到对应本体（`00040000`）后匹配英文发行名。无法在内置 Title ID 数据中唯一命中的内容会继续使用现有中文文件名匹配，不会强行覆盖。

## 推荐目录

```text
3DS/
└── 游戏中文名 [Title ID]/
    ├── 本体.cia
    ├── 更新.cia
    ├── DLC.cia
    └── 补丁说明与原文件/
```

保留原始压缩包作为源文件，整理结果写到另一块目录，避免批量操作损坏唯一副本。Title ID 用于去重；同一 Title ID 的本体、更新和 DLC 不能互相覆盖。

只保留一份源内容：不要为了两个模拟器复制两套 ROM。Azahar 中通过菜单安装本体、更新和 DLC 的 CIA；Panda3DS 不能直接载入 CIA，只有在使用本人合法备份和本人主机密钥提取出 `.app`/`.cxi` 后，才单独保留这一份可加载内容。汉化若已经做进 CIA/ROM，就作为独立版本保留；LayeredFS 补丁保持原目录结构并按 Title ID 管理，不要直接覆盖本体。

## 已解压目录只读盘点

对于 NAS/Samba 上已经解压的目录，使用有限深度清单模式。它只读取文件名、扩展名和路径，不哈希、不读取 ROM 内容，也不修改文件：

```powershell
python scripts/organize_3ds_archives.py "Y:\3ds" --inventory-dir --max-depth 3 --report "3ds-directory-report.csv"
```

Azahar 可安装 CIA，并把更新/DLC安装到其用户 NAND；不要再把安装后的 NAND 复制回 ROM 库。Panda3DS 可直接载入 `.3ds/.cci/.cxi/.app/.ncch/.3dsx`，但当前不能直接载入 CIA。加密 dump 需要用户自行把 `aes_keys.txt` 放入 Panda3DS 应用数据目录下的 `sysdata` 文件夹；本项目不创建、下载或分发密钥。Panda3DS 当前没有与 Azahar 等价、文档化稳定的 CIA 更新/DLC 安装流程，因此清单会保留这些 CIA，但不会声称 Panda3DS 可直接使用。

## 先生成清单

```powershell
python scripts/organize_3ds_archives.py "G:\ROMS\3DSCH(OldmanEmu.net)" --password "oldmanemu.net" --report "G:\ROMS\3ds-report.csv"
```

默认只读取压缩包目录，不移动或解包文件。压缩包解开后才能继续读取 CIA Ticket/NCCH 元数据。

## 复制解包

确认清单后再执行：

```powershell
python scripts/organize_3ds_archives.py "G:\ROMS\3DSCH(OldmanEmu.net)" --password "oldmanemu.net" --report "G:\ROMS\3ds-report.csv" --output-dir "G:\ROMS\3DS-整理后" --apply
```

工具不会覆盖已经存在的目标文件，也不会删除源压缩包。

## 解密边界

CIA 内容通常仍有 3DS 加密层。模拟器需要何种格式取决于模拟器；实机安装和模拟器直接加载不是同一流程。项目不分发密钥，也不自动下载解密程序。需要解密时，应使用本人 3DS 导出的 `boot9`/密钥和合法备份，在整理副本上操作，完成后再让 MRRM 扫描。
