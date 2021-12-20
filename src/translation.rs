enum TranslatedDocument {
    Missing {
        original: fpm::File,
    },
    NeverMarked {
        original: fpm::File,
        translated: fpm::File,
    },
    Outdated {
        original: fpm::File,
        translated: fpm::File,
        last_marked_on: u128,
        original_latest: u128,
        translated_latest: u128,
    },
    UptoDate {
        translated: fpm::File,
    },
}

impl TranslatedDocument {
    pub fn html(&self) -> String {
        // handle the message
        // render with-fallback or with-message
    }
}
