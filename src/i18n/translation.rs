const RES: &str = "translation";

pub static TRANSLATION: once_cell::sync::Lazy<fpm::i18n::Base> =
    once_cell::sync::Lazy::new(|| fpm::i18n::new_base(RES));

pub fn search(
    lang: &realm_lang::Language,
    primary_lang: &realm_lang::Language,
    key: &'static str,
    last_modified_on: &Option<String>,
) -> String {
    let mut args = fluent::FluentArgs::new();
    args.set(
        "primary-lang",
        fluent::FluentValue::from(primary_lang.human()),
    );
    args.set(
        "primary-lang-code",
        fluent::FluentValue::from(primary_lang.id()),
    );
    args.set("lang", fluent::FluentValue::from(lang.human()));
    args.set("lang-code", fluent::FluentValue::from(lang.id()));
    let last_modified_on = if let Some(last_modified_on) = last_modified_on {
        last_modified_on.to_string()
    } else {
        "Never Synced".to_string()
    };
    args.set(
        "last-modified-on",
        fluent::FluentValue::from(last_modified_on.as_str()),
    );
    fpm::i18n::lookup(&TRANSLATION, lang, RES, key, None, Some(&args))
}
