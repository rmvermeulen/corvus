use cfg_if::cfg_if;
use smol_str::SmolStr;

#[derive(Clone, Debug)]
pub struct NavigationIconConfig {
    pub back: SmolStr,
    pub next: SmolStr,
    pub up: SmolStr,
    pub reload: SmolStr,
}

#[derive(Clone, Debug)]
pub struct FsIconConfig {
    pub file: SmolStr,
    pub directory: SmolStr,
    pub symlink: SmolStr,
    pub unknown: SmolStr,
}

#[derive(Clone, Debug)]
pub struct IconConfig {
    pub navigation: NavigationIconConfig,
    pub fs: FsIconConfig,
}

const fn s(s: &str) -> SmolStr {
    SmolStr::new_inline(s)
}

cfg_if! {
    if #[cfg(feature = "emoji")] {
        pub const ICON_CONFIG: IconConfig = IconConfig {
            navigation: NavigationIconConfig {
                back: s("🔙"),
                next: '🔜',
                up: '🔝',
                reload: '🔄',
            },
            fs: FsIconConfig {
                file: '📄',
                directory: '📁',
                symlink: '🔗',
                unknown: '❓',
            },
        };
    } else {
        pub const ICON_CONFIG: IconConfig = IconConfig {
            navigation: NavigationIconConfig {
                back: s("[B]"),
                next: s("[N]"),
                up: s("[U]"),
                reload: s("[R]"),
            },
            fs: FsIconConfig {
                file: s("[F]"),
                directory: s("[D]"),
                symlink: s("[S]"),
                unknown: s("[?]"),
            },
        };
    }
}
