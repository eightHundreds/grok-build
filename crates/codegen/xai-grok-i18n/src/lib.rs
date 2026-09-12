//! Fork-only UI dictionary.
//!
//! English msgids stay in upstream source. At paint time, [`t`] looks up the
//! compiled-in catalog. Missing keys return the msgid unchanged.
//!
//! Locale selection:
//! 1. [`set_locale`] (tests / explicit)
//! 2. `GROK_LANG` at process start
//! 3. compile-time `GROK_UI_LANG` (`cargo` / CI), default `en`

use std::borrow::Cow;
use std::collections::HashMap;
use std::sync::{OnceLock, RwLock};

const ZH_CN_TOML: &str = include_str!("../locales/zh-CN.toml");

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Locale {
    En,
    ZhCn,
}

impl Locale {
    pub fn parse(raw: &str) -> Self {
        let s = raw.trim();
        let lower = s.to_ascii_lowercase();
        if lower == "zh"
            || lower == "zh-cn"
            || lower == "zh_cn"
            || lower == "zh-hans"
            || lower.starts_with("zh-hans")
            || lower.starts_with("zh_hans")
        {
            Locale::ZhCn
        } else {
            Locale::En
        }
    }
}

fn startup_locale() -> Locale {
    if let Ok(v) = std::env::var("GROK_LANG")
        && !v.trim().is_empty()
    {
        return Locale::parse(&v);
    }
    Locale::parse(option_env!("GROK_UI_LANG").unwrap_or("en"))
}

fn locale_cell() -> &'static RwLock<Locale> {
    static CELL: OnceLock<RwLock<Locale>> = OnceLock::new();
    CELL.get_or_init(|| RwLock::new(startup_locale()))
}

/// Active UI locale.
pub fn locale() -> Locale {
    *locale_cell().read().unwrap_or_else(|e| e.into_inner())
}

/// Override the UI locale (tests, or a future settings hook).
pub fn set_locale(raw: &str) {
    *locale_cell().write().unwrap_or_else(|e| e.into_inner()) = Locale::parse(raw);
}

/// Restore the previous locale when dropped.
pub struct LocaleGuard {
    prev: Locale,
}

impl Drop for LocaleGuard {
    fn drop(&mut self) {
        *locale_cell().write().unwrap_or_else(|e| e.into_inner()) = self.prev;
    }
}

/// Pin a locale until the guard drops.
pub fn pin_locale(raw: &str) -> LocaleGuard {
    let prev = locale();
    set_locale(raw);
    LocaleGuard { prev }
}

/// Apply `GROK_LANG` / `GROK_UI_LANG`. Call once at process start.
pub fn init_from_env() {
    if let Ok(v) = std::env::var("GROK_LANG")
        && !v.trim().is_empty()
    {
        set_locale(&v);
        return;
    }
    set_locale(option_env!("GROK_UI_LANG").unwrap_or("en"));
}

fn zh_catalog() -> &'static HashMap<String, String> {
    static MAP: OnceLock<HashMap<String, String>> = OnceLock::new();
    MAP.get_or_init(|| {
        toml::from_str::<HashMap<String, String>>(ZH_CN_TOML).unwrap_or_else(|e| {
            panic!("xai-grok-i18n: zh-CN.toml is invalid: {e}");
        })
    })
}

fn lookup<'a>(msgid: &'a str) -> Option<&'static str> {
    match locale() {
        Locale::En => None,
        Locale::ZhCn => {
            let map = zh_catalog();
            map.get(msgid)
                .or_else(|| map.get(&msgid.to_lowercase()))
                .map(String::as_str)
        }
    }
}

/// Translate `msgid`. Unknown keys and the English locale return `msgid`.
pub fn t(msgid: &str) -> Cow<'_, str> {
    match lookup(msgid) {
        Some(s) => Cow::Borrowed(s),
        None => Cow::Borrowed(msgid),
    }
}

/// [`t`] then substitute `{name}` placeholders from `pairs`.
pub fn t_fmt(msgid: &str, pairs: &[(&str, &str)]) -> String {
    let mut out = t(msgid).into_owned();
    for (key, value) in pairs {
        out = out.replace(&format!("{{{key}}}"), value);
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn english_is_identity() {
        let _g = pin_locale("en");
        assert_eq!(t("send").as_ref(), "send");
        assert_eq!(
            t_fmt("press again to {label}", &[("label", "quit")]),
            "press again to quit"
        );
    }

    #[test]
    fn zh_looks_up_labels_and_fmt() {
        let _g = pin_locale("zh");
        assert_eq!(t("send").as_ref(), "发送");
        assert_eq!(t("Send").as_ref(), "发送");
        assert_eq!(t("shortcuts").as_ref(), "快捷键");
        assert_eq!(
            t_fmt("press again to {label}", &[("label", t("quit").as_ref())]),
            "再按一次以退出"
        );
        assert_eq!(t("no-such-msgid").as_ref(), "no-such-msgid");
        assert_eq!(t("Back to Home").as_ref(), "返回主页");
        assert_eq!(t("Commands").as_ref(), "命令");
        assert_eq!(t("↑/↓ nav").as_ref(), "↑/↓ 导航");
        assert_eq!(t("Help improve Grok").as_ref(), "帮助改进 Grok");
        assert_eq!(t("New worktree").as_ref(), "新建工作树");
        assert_eq!(t("Yes, proceed").as_ref(), "是，继续");
        assert_eq!(t("Build anything").as_ref(), "构建任何东西");
        assert_eq!(t("Tip: ").as_ref(), "提示：");
        assert_eq!(t("Always allow:").as_ref(), "始终允许：");
        assert_eq!(t("Return to the welcome screen").as_ref(), "返回欢迎屏");
    }

    #[test]
    fn locale_aliases() {
        assert_eq!(Locale::parse("zh-CN"), Locale::ZhCn);
        assert_eq!(Locale::parse("zh_CN"), Locale::ZhCn);
        assert_eq!(Locale::parse("en-US"), Locale::En);
    }
}
