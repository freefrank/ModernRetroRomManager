//! 复古游戏系统映射配置
//!
//! 将用户的 ROM 目录名称映射到：
//! - CSV 数据库文件名
//! - Logo 图片文件名

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SystemMapping {
    /// ROM 目录文件夹名称
    pub folder_name: &'static str,
    /// CSV 数据库文件名（不含路径）
    pub csv_name: Option<&'static str>,
    /// Logo 图片文件名（不含路径）
    pub logo_name: Option<&'static str>,
}

/// 获取所有系统映射
pub fn get_system_mappings() -> Vec<SystemMapping> {
    vec![
        // Nintendo 系列
        SystemMapping {
            folder_name: "3DS",
            csv_name: Some("Nintendo - Nintendo 3DS.csv"),
            logo_name: Some("3DS.png"),
        },
        SystemMapping {
            folder_name: "FC",
            csv_name: Some("Nintendo - Nintendo Entertainment System.csv"),
            logo_name: Some("FC.png"),
        },
        SystemMapping {
            folder_name: "FC hack",
            csv_name: None, // Hack 类不映射
            logo_name: Some("FC hack.png"),
        },
        SystemMapping {
            folder_name: "FC-HD",
            csv_name: None, // HD 类不映射
            logo_name: Some("FC-HD.png"),
        },
        SystemMapping {
            folder_name: "SFC",
            csv_name: Some("Nintendo - Super Nintendo Entertainment System.csv"),
            logo_name: Some("SFC.png"),
        },
        SystemMapping {
            folder_name: "SFC hack",
            csv_name: None,
            logo_name: Some("SFC hack.png"),
        },
        SystemMapping {
            folder_name: "SFC-MSU1",
            csv_name: Some("Nintendo - Super Nintendo Entertainment System.csv"),
            logo_name: Some("SFC-MSU1.png"),
        },
        SystemMapping {
            folder_name: "GB",
            csv_name: Some("Nintendo - Game Boy.csv"),
            logo_name: Some("GB.png"),
        },
        SystemMapping {
            folder_name: "GBC",
            csv_name: Some("Nintendo - Game Boy Color.csv"),
            logo_name: Some("GBC.png"),
        },
        SystemMapping {
            folder_name: "GBA",
            csv_name: Some("Nintendo - Game Boy Advance.csv"),
            logo_name: Some("GBA.png"),
        },
        SystemMapping {
            folder_name: "N64",
            csv_name: Some("Nintendo - Nintendo 64.csv"),
            logo_name: Some("N64.png"),
        },
        SystemMapping {
            folder_name: "NDS",
            csv_name: Some("Nintendo - Nintendo DS.csv"),
            logo_name: Some("NDS.png"),
        },
        SystemMapping {
            folder_name: "NGC",
            csv_name: Some("Nintendo - GameCube.csv"),
            logo_name: Some("NGC.png"),
        },
        SystemMapping {
            folder_name: "WII",
            csv_name: Some("Nintendo - Wii.csv"),
            logo_name: Some("WII.png"),
        },
        SystemMapping {
            folder_name: "WII Ware",
            csv_name: Some("Nintendo - Wii.csv"),
            logo_name: Some("WII Ware.png"),
        },
        SystemMapping {
            folder_name: "Virtual Boy",
            csv_name: Some("Nintendo - Virtual Boy (32) (Chinese).csv"),
            logo_name: Some("VB.png"),
        },
        SystemMapping {
            folder_name: "GAME WATCH",
            csv_name: Some("Nintendo - Game  & Watch.csv"),
            logo_name: Some("GAME WATCH.png"),
        },
        SystemMapping {
            folder_name: "POKE MINI",
            csv_name: Some("Nintendo - Pokemon Mini.csv"),
            logo_name: Some("POKE MINI.png"),
        },

        // Sega 系列
        SystemMapping {
            folder_name: "MD",
            csv_name: Some("Sega - Mega Drive - Genesis.csv"),
            logo_name: Some("MD.png"),
        },
        SystemMapping {
            folder_name: "MD hack",
            csv_name: None,
            logo_name: Some("MD hack.png"),
        },
        SystemMapping {
            folder_name: "MD hack(picodrive)",
            csv_name: None,
            logo_name: Some("MD hack.png"),
        },
        SystemMapping {
            folder_name: "MD-32X",
            csv_name: Some("Sega - 32X.csv"),
            logo_name: Some("MD-32X.png"),
        },
        SystemMapping {
            folder_name: "DC",
            csv_name: Some("Sega - Dreamcast.csv"),
            logo_name: Some("DC.png"),
        },
        SystemMapping {
            folder_name: "DC hack",
            csv_name: None,
            logo_name: Some("DC.png"),
        },
        SystemMapping {
            folder_name: "SS",
            csv_name: Some("Sega - Saturn.csv"),
            logo_name: Some("SS.png"),
        },
        SystemMapping {
            folder_name: "GG",
            csv_name: Some("Sega - Game Gear.csv"),
            logo_name: Some("GG.png"),
        },
        SystemMapping {
            folder_name: "SMS",
            csv_name: Some("Sega - Master System - Mark III.csv"),
            logo_name: Some("SMS.png"),
        },
        SystemMapping {
            folder_name: "NAOMI",
            csv_name: Some("Arcade - NAOMI.csv"),
            logo_name: Some("NAOMI.png"),
        },

        // Sony 系列
        SystemMapping {
            folder_name: "PS1",
            csv_name: Some("Sony - PlayStation.csv"),
            logo_name: Some("PS.png"),
        },
        SystemMapping {
            folder_name: "PS1 hack",
            csv_name: None,
            logo_name: Some("PS.png"),
        },
        SystemMapping {
            folder_name: "PS2",
            csv_name: Some("Sony - PlayStation 2.csv"),
            logo_name: Some("PS2.png"),
        },
        SystemMapping {
            folder_name: "PSP",
            csv_name: Some("Sony - PlayStation Portable.csv"),
            logo_name: Some("PSP.png"),
        },

        // Bandai 系列
        SystemMapping {
            folder_name: "WS",
            csv_name: Some("Bandai - WonderSwan.csv"),
            logo_name: Some("WS.png"),
        },
        SystemMapping {
            folder_name: "WSC",
            csv_name: Some("Bandai - WonderSwan Color.csv"),
            logo_name: Some("WSC.png"),
        },

        // SNK 系列
        SystemMapping {
            folder_name: "NGPC",
            csv_name: Some("SNK - Neo Geo Pocket Color.csv"),
            logo_name: Some("NGPC.png"),
        },

        // NEC 系列
        SystemMapping {
            folder_name: "PCE",
            csv_name: Some("NEC - PC Engine - TurboGrafx-16.csv"),
            logo_name: Some("PCE.png"),
        },
        SystemMapping {
            folder_name: "PCE-CD",
            csv_name: Some("NEC - PC Engine - TurboGrafx-16.csv"),
            logo_name: Some("PCE-CD.png"),
        },
        SystemMapping {
            folder_name: "PC-FX",
            csv_name: Some("NEC - PC-FX.csv"),
            logo_name: Some("PC-FX.png"),
        },

        // Panasonic
        SystemMapping {
            folder_name: "3DO",
            csv_name: Some("Panasonic - 3DO Interactive Multiplayer.csv"),
            logo_name: Some("3DO.png"),
        },

        // Atari 系列
        SystemMapping {
            folder_name: "LYNX",
            csv_name: Some("Atari - Lynx.csv"),
            logo_name: Some("LYNX.png"),
        },
        SystemMapping {
            folder_name: "ATARI",
            csv_name: Some("Atari - Atari 2600.csv"),
            logo_name: Some("ATARI.png"),
        },

        // Arcade 系列
        SystemMapping {
            folder_name: "NEOGEO-CD",
            csv_name: Some("Arcade - NEOGEO.csv"),
            logo_name: Some("NEOGEO-CD.png"),
        },
        SystemMapping {
            folder_name: "MODEL2",
            csv_name: Some("Arcade - MODEL2.csv"),
            logo_name: Some("MODEL2.png"),
        },
        SystemMapping {
            folder_name: "MODEL3",
            csv_name: Some("Arcade - MODEL3.csv"),
            logo_name: Some("MODEL3.png"),
        },

        // Microsoft
        SystemMapping {
            folder_name: "PC",
            csv_name: Some("Microsoft - PC.csv"),
            logo_name: Some("PC.png"),
        },
        SystemMapping {
            folder_name: "DOS",
            csv_name: Some("Microsoft - DOS.csv"),
            logo_name: Some("DOS.png"),
        },

        // 其他
        SystemMapping {
            folder_name: "OPENBOR",
            csv_name: None,
            logo_name: Some("OPENBOR.png"),
        },
        SystemMapping {
            folder_name: "SWITCH",
            csv_name: Some("Nintendo - Switch.csv"),
            logo_name: Some("SWITCH.png"),
        },
        SystemMapping {
            folder_name: "TeknoParrot",
            csv_name: Some("Arcade - TeknoParrot.csv"),
            logo_name: Some("TeknoParrot.png"),
        },

        // FBNEO 街机分类
        SystemMapping {
            folder_name: "FBNEO ACT",
            csv_name: None,
            logo_name: None,
        },
        SystemMapping {
            folder_name: "FBNEO STG",
            csv_name: None,
            logo_name: None,
        },
        SystemMapping {
            folder_name: "FBNEO FTG",
            csv_name: None,
            logo_name: None,
        },
        SystemMapping {
            folder_name: "FBNEO FLY",
            csv_name: None,
            logo_name: None,
        },
        SystemMapping {
            folder_name: "FBNEO RAC",
            csv_name: None,
            logo_name: None,
        },
        SystemMapping {
            folder_name: "FBNEO SPO",
            csv_name: None,
            logo_name: None,
        },
        SystemMapping {
            folder_name: "FBNEO ETC",
            csv_name: None,
            logo_name: None,
        },

        // MAME 街机分类
        SystemMapping {
            folder_name: "MAME ACT",
            csv_name: None,
            logo_name: None,
        },
        SystemMapping {
            folder_name: "MAME STG",
            csv_name: None,
            logo_name: None,
        },
        SystemMapping {
            folder_name: "MAME FTG",
            csv_name: None,
            logo_name: None,
        },
        SystemMapping {
            folder_name: "MAME FLY",
            csv_name: None,
            logo_name: None,
        },
        SystemMapping {
            folder_name: "MAME RAC",
            csv_name: None,
            logo_name: None,
        },
        SystemMapping {
            folder_name: "MAME SPO",
            csv_name: None,
            logo_name: None,
        },
        SystemMapping {
            folder_name: "MAME ETC",
            csv_name: None,
            logo_name: None,
        },

        // 光枪游戏
        SystemMapping {
            folder_name: "Light Gun",
            csv_name: None,
            logo_name: None,
        },
    ]
}
