fn main() {
    println!("cargo:rerun-if-env-changed=GROK_UI_LANG");
    let lang = std::env::var("GROK_UI_LANG").unwrap_or_else(|_| "en".into());
    println!("cargo:rustc-env=GROK_UI_LANG={lang}");
}
