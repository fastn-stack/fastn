pub(crate) trait KindExt {
    fn is_ftd_responsive_type(&self) -> bool;
    fn is_ftd_type(&self) -> bool;
    fn is_ftd_font_size(&self) -> bool;
    fn is_ftd_background_color(&self) -> bool;
    fn is_ftd_length(&self) -> bool;
    fn is_ftd_image_src(&self) -> bool;
    fn is_ftd_color(&self) -> bool;
    fn is_ftd_resizing(&self) -> bool;
    fn is_ftd_resizing_fixed(&self) -> bool;
}

impl KindExt for fastn_type::Kind {
    fn is_ftd_responsive_type(&self) -> bool {
        matches!(self, fastn_type::Kind::Record { name, .. } if name.eq
            (ftd::interpreter::FTD_RESPONSIVE_TYPE))
    }

    fn is_ftd_type(&self) -> bool {
        matches!(self, fastn_type::Kind::Record { name, .. } if name.eq(ftd::interpreter::FTD_TYPE))
    }

    fn is_ftd_font_size(&self) -> bool {
        matches!(self, fastn_type::Kind::Record { name, .. } if name.eq
            (ftd::interpreter::FTD_FONT_SIZE))
    }

    fn is_ftd_background_color(&self) -> bool {
        matches!(self, fastn_type::Kind::OrType { name, variant, .. } if name.eq
            (ftd::interpreter::FTD_BACKGROUND) &&
            variant.is_some() && variant.as_ref().unwrap().starts_with(ftd::interpreter::FTD_BACKGROUND_SOLID))
    }

    fn is_ftd_length(&self) -> bool {
        matches!(self, fastn_type::Kind::OrType { name, .. } if name.eq
            (ftd::interpreter::FTD_LENGTH))
    }

    fn is_ftd_image_src(&self) -> bool {
        matches!(self, fastn_type::Kind::Record { name, .. } if name.eq
            (ftd::interpreter::FTD_IMAGE_SRC))
    }

    fn is_ftd_color(&self) -> bool {
        matches!(self, fastn_type::Kind::Record { name, .. } if name.eq
            (ftd::interpreter::FTD_COLOR))
    }

    fn is_ftd_resizing(&self) -> bool {
        matches!(self, fastn_type::Kind::OrType { name, .. } if name.eq
            (ftd::interpreter::FTD_RESIZING))
    }

    fn is_ftd_resizing_fixed(&self) -> bool {
        matches!(self, fastn_type::Kind::OrType { name, variant, .. } if name.eq
            (ftd::interpreter::FTD_RESIZING) && variant.is_some() && variant.as_ref().unwrap().starts_with(ftd::interpreter::FTD_RESIZING_FIXED))
    }
}
