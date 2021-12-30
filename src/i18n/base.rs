const RES: &str = "base";
lazy_static! {
    pub static ref BASE: fpm::i18n::Base = fpm::i18n::new_base(RES);
}

pub fn site_title(lang: &realm_lang::Language) -> String {
    fpm::i18n::message(&BASE, lang, RES, "site-title")
}

pub fn stats_title(lang: &realm_lang::Language, name: String) -> String {
    let mut args = fluent::FluentArgs::new();
    args.set("collection-name", fluent::FluentValue::from(name));

    fpm::i18n::lookup(&BASE, lang, RES, "stats-title", None, Some(&args))
}
