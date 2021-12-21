enum TranslatedDocument {
    Missing {
        original: fpm::File,
    },
    NeverMarked {
        original: fpm::File,   // main
        translated: fpm::File, // fallback
    },
    Outdated {
        original: fpm::File,   // fallback
        translated: fpm::File, // main
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
