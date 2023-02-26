pub mod translation;

type Bundle = fluent::bundle::FluentBundle<
    fluent::FluentResource,
    intl_memoizer::concurrent::IntlLangMemoizer,
>;
type Map = std::collections::HashMap<realm_lang::Language, Option<Bundle>>;
pub type Base = std::sync::Arc<antidote::Mutex<(Bundle, Map)>>;

#[derive(serde::Serialize)]
#[allow(clippy::upper_case_acronyms)]
pub struct HTML {
    pub text: String,
}

#[derive(serde::Serialize)]
struct Integer {
    value: i64,
    localised: String,
}

#[derive(serde::Serialize)]
struct Float {
    value: f64,
    localised: String,
}

fn new_bundle(lang: &realm_lang::Language, res: String) -> Bundle {
    let i = issue(lang, res.as_str(), None);

    let mut b = fluent::bundle::FluentBundle::new_concurrent(vec![lang
        .to_2_letter_code()
        .parse()
        .unwrap_or_else(|_| panic!("{}", i))]);
    b.add_resource(fluent::FluentResource::try_new(res).unwrap_or_else(|_| panic!("{}", i)))
        .unwrap_or_else(|_| panic!("{}", i));
    b
}

pub fn new_base(id: &'static str) -> Base {
    let default = realm_lang::Language::English;
    std::sync::Arc::new(antidote::Mutex::new((
        new_bundle(
            &default,
            read_file(&default, id).unwrap_or_else(|| panic!("cant read english resource: {}", id)),
        ),
        std::collections::HashMap::new(),
    )))
}

// fn bundle<'a, 'b>(
//     base: &'a Base,
//     lang: &realm_lang::Language,
// ) -> (antidote::MutexGuard<'b, (Bundle, crate::Map)>, &'b Bundle)
// where
//     'a: 'b,
// {
//     use std::ops::DerefMut;
//
//     let mut lock = base.lock();
//     let (en, ref mut m) = lock.deref_mut();
//     let b = match m.get(lang) {
//         Some(Some(v)) => v,
//         Some(None) => en,
//         None => {
//             todo!()
//         }
//     };
//
//     (lock, b)
// }

fn issue(lang: &realm_lang::Language, res: &str, id: Option<&str>) -> String {
    format!("issue with {}/{}/{:?}", lang.to_2_letter_code(), res, id)
}

/*pub fn html(base: &Base, lang: &realm_lang::Language, res: &'static str, id: &'static str) -> HTML {
    assert!(id.ends_with("-html"));
    HTML {
        text: message(base, lang, res, id),
    }
}

pub fn message(
    base: &Base,
    lang: &realm_lang::Language,
    res: &'static str,
    id: &'static str,
) -> String {
    lookup(base, lang, res, id, None, None)
}

// message_with_args

pub fn attribute(
    base: &Base,
    lang: &realm_lang::Language,
    res: &'static str,
    id: &'static str,
    attr: &'static str,
) -> String {
    lookup(base, lang, res, id, Some(attr), None)
}*/

// message_with_args

pub fn lookup(
    base: &Base,
    lang: &realm_lang::Language,
    res: &'static str,
    id: &'static str,
    attribute: Option<&'static str>,
    args: Option<&fluent::FluentArgs>,
) -> String {
    use std::ops::DerefMut;

    let i = issue(lang, res, Some(id));

    let mut lock = base.lock();
    let (en, ref mut m) = lock.deref_mut();
    if m.get(lang).is_none() {
        match read_file(lang, res) {
            Some(v) => {
                m.insert(*lang, Some(new_bundle(lang, v)));
            }
            None => {
                m.insert(*lang, None);
            }
        }
    };

    let b: &Bundle = match m.get(lang) {
        Some(Some(v)) => v,
        Some(None) => en,
        None => unreachable!(),
    };

    let msg = b
        .get_message(id)
        .or_else(|| en.get_message(id))
        .unwrap_or_else(|| panic!("{}", i));

    let mut errors = vec![];

    let pattern = match attribute {
        Some(key) => msg
            .get_attribute(key)
            .unwrap_or_else(|| panic!("{}", i))
            .value(),
        None => msg.value().unwrap_or_else(|| panic!("{}", i)),
    };

    let s = b.format_pattern(pattern, args, &mut errors);

    if !errors.is_empty() {
        panic!("errors found in {}: {:?}", i, errors)
    }

    s.into()
}

fn read_file(lang: &realm_lang::Language, res: &'static str) -> Option<String> {
    let string = match (lang, res) {
        (&realm_lang::Language::Hindi, "translation") => {
            include_str!("../../i18n/hi/translation.ftl")
        }
        (_, "translation") => {
            include_str!("../../i18n/en/translation.ftl")
        }
        _ => panic!(),
    };
    Some(string.to_string())
}
