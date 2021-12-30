const RES: &str = "translation";
lazy_static! {
    pub static ref TRANSLATION: fpm::i18n::Base = fpm::i18n::new_base(RES);
}

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

pub fn cta_switch_to_primary(
    lang: &realm_lang::Language,
    primary_lang: &realm_lang::Language,
) -> String {
    let mut args = fluent::FluentArgs::new();
    args.set(
        "primary-lang",
        fluent::FluentValue::from(primary_lang.human()),
    );
    args.set("lang-code", fluent::FluentValue::from(primary_lang.id()));
    fpm::i18n::lookup(
        &TRANSLATION,
        lang,
        RES,
        "cta-switch-to-primary",
        None,
        Some(&args),
    )
}

pub fn cta_switch_to_local(
    lang: &realm_lang::Language,
    target_lang: &realm_lang::Language,
) -> String {
    let mut args = fluent::FluentArgs::new();
    args.set(
        "target-lang",
        fluent::FluentValue::from(target_lang.human()),
    );
    fpm::i18n::lookup(
        &TRANSLATION,
        lang,
        RES,
        "cta-switch-to-local",
        None,
        Some(&args),
    )
}

pub fn cta_add_translation(lang: &realm_lang::Language) -> String {
    fpm::i18n::message(&TRANSLATION, lang, RES, "cta-add-translation")
}

pub fn cta_edit_translation(lang: &realm_lang::Language) -> String {
    fpm::i18n::message(&TRANSLATION, lang, RES, "cta-edit-translation")
}

pub fn cta_mark_up_to_date(lang: &realm_lang::Language) -> String {
    fpm::i18n::message(&TRANSLATION, lang, RES, "cta-mark-up-to-date")
}

pub fn heading(lang: &realm_lang::Language) -> String {
    fpm::i18n::message(&TRANSLATION, lang, RES, "heading")
}

pub fn crs_heading(lang: &realm_lang::Language, cr_count: usize) -> String {
    let mut args = fluent::FluentArgs::new();
    args.set("cr-count", fluent::FluentValue::from(cr_count));
    fpm::i18n::lookup(&TRANSLATION, lang, RES, "crs-heading", None, Some(&args))
}

pub fn unapproved_heading(lang: &realm_lang::Language) -> String {
    fpm::i18n::message(&TRANSLATION, lang, RES, "unapproved-heading")
}

pub fn body_local(
    lang: &realm_lang::Language,
    target_lang: &realm_lang::Language,
    primary_lang: &realm_lang::Language,
    last_merged_on: String,
) -> String {
    let mut args = fluent::FluentArgs::new();
    args.set(
        "target-lang",
        fluent::FluentValue::from(target_lang.human()),
    );
    args.set(
        "primary-lang",
        fluent::FluentValue::from(primary_lang.human()),
    );
    args.set("last-merged-on", fluent::FluentValue::from(last_merged_on));
    fpm::i18n::lookup(&TRANSLATION, lang, RES, "body-local", None, Some(&args))
}

pub fn body_primary(lang: &realm_lang::Language, primary_lang: &realm_lang::Language) -> String {
    let mut args = fluent::FluentArgs::new();
    args.set(
        "primary-lang",
        fluent::FluentValue::from(primary_lang.human()),
    );
    fpm::i18n::lookup(&TRANSLATION, lang, RES, "body-primary", None, Some(&args))
}

pub fn translation_not_available(
    lang: &realm_lang::Language,
    target_lang: &realm_lang::Language,
    primary_lang: &realm_lang::Language,
) -> String {
    let mut args = fluent::FluentArgs::new();
    args.set(
        "target-lang",
        fluent::FluentValue::from(target_lang.human()),
    );
    args.set(
        "primary-lang",
        fluent::FluentValue::from(primary_lang.human()),
    );
    fpm::i18n::lookup(
        &TRANSLATION,
        lang,
        RES,
        "translation-not-available",
        None,
        Some(&args),
    )
}
