diesel::table! {
    systems (id) {
        id -> Text,
        name -> Text,
        short_name -> Text,
        manufacturer -> Nullable<Text>,
        release_year -> Nullable<Integer>,
        extensions -> Text,
        igdb_platform_id -> Nullable<Integer>,
        thegamesdb_platform_id -> Nullable<Integer>,
        screenscraper_id -> Nullable<Integer>,
    }
}

diesel::table! {
    roms (id) {
        id -> Text,
        filename -> Text,
        path -> Text,
        system_id -> Text,
        size -> BigInt,
        crc32 -> Nullable<Text>,
        md5 -> Nullable<Text>,
        sha1 -> Nullable<Text>,
        created_at -> Text,
        updated_at -> Text,
    }
}

diesel::table! {
    rom_metadata (rom_id) {
        rom_id -> Text,
        name -> Text,
        description -> Nullable<Text>,
        release_date -> Nullable<Text>,
        developer -> Nullable<Text>,
        publisher -> Nullable<Text>,
        genre -> Nullable<Text>,
        players -> Nullable<Integer>,
        rating -> Nullable<Double>,
        region -> Nullable<Text>,
        scraper_source -> Nullable<Text>,
        scraped_at -> Nullable<Text>,
    }
}

diesel::table! {
    media_assets (id) {
        id -> Text,
        rom_id -> Text,
        asset_type -> Text,
        path -> Text,
        width -> Nullable<Integer>,
        height -> Nullable<Integer>,
        file_size -> Nullable<BigInt>,
        source_url -> Nullable<Text>,
        downloaded_at -> Text,
    }
}

diesel::table! {
    api_configs (id) {
        id -> Text,
        provider -> Text,
        api_key -> Nullable<Text>,
        api_secret -> Nullable<Text>,
        username -> Nullable<Text>,
        password -> Nullable<Text>,
        enabled -> Bool,
        priority -> Integer,
    }
}

diesel::table! {
    scan_directories (id) {
        id -> Text,
        path -> Text,
        system_id -> Nullable<Text>,
        recursive -> Bool,
        enabled -> Bool,
        last_scan -> Nullable<Text>,
    }
}

diesel::joinable!(roms -> systems (system_id));
diesel::joinable!(rom_metadata -> roms (rom_id));
diesel::joinable!(media_assets -> roms (rom_id));
diesel::joinable!(scan_directories -> systems (system_id));

diesel::allow_tables_to_appear_in_same_query!(
    systems,
    roms,
    rom_metadata,
    media_assets,
    api_configs,
    scan_directories,
);
