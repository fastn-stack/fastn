#[derive(Debug, Hash, Eq, PartialEq, Clone, Copy)]
pub struct DynamicProperty {
    pub node: fastn_runtime::NodeKey,
    pub property: fastn_runtime::UIProperty,
    pub closure: fastn_runtime::ClosurePointer,
}

#[derive(Debug)]
pub struct TextRole {
    font_size: fastn_runtime::PointerKey,
    line_height: fastn_runtime::PointerKey,
}

// -- integer $x: 20

// -- ftd.text-role r:
// font-size: $x + 20

// def x_modified_update_r(r, x):
//     mem.set_list_item(r, 0, x)

// def x_modified_update_r(r, x):
//      mem.update_text_role(r, TextRoleField::FontSize.to_i32(), x)

#[derive(Debug)]
pub struct ResponsiveProperty<T> {
    desktop: T,
    mobile: T,
}

#[derive(Debug)]
pub struct LengthRole {}

#[derive(Debug)]
pub struct DarkModeProperty<T> {
    pub light: T,
    pub dark: Option<T>,
}

impl<T> From<T> for DarkModeProperty<T> {
    fn from(light: T) -> Self {
        DarkModeProperty { light, dark: None }
    }
}

#[repr(C)]
#[derive(Copy, Clone, Default, Debug)]
pub struct Color {
    pub red: fastn_runtime::PointerKey,
    pub green: fastn_runtime::PointerKey,
    pub blue: fastn_runtime::PointerKey,
    pub alpha: fastn_runtime::PointerKey,
}

#[derive(Debug, Copy, Hash, Eq, PartialEq, Clone)]
pub enum UIProperty {
    WidthFixedPx,
    HeightFixedPx,
    HeightFixedPercentage,
    BackgroundSolid,
    SpacingFixedPx,
    MarginFixedPx,
    Event,
}

impl From<i32> for UIProperty {
    fn from(i: i32) -> UIProperty {
        match i {
            0 => UIProperty::WidthFixedPx,
            1 => UIProperty::HeightFixedPx,
            2 => UIProperty::HeightFixedPercentage,
            3 => UIProperty::BackgroundSolid,
            4 => UIProperty::SpacingFixedPx,
            5 => UIProperty::MarginFixedPx,
            6 => UIProperty::Event,
            _ => panic!("Unknown UIProperty: {}", i),
        }
    }
}

impl From<UIProperty> for i32 {
    fn from(v: UIProperty) -> i32 {
        match v {
            UIProperty::WidthFixedPx => 0,
            UIProperty::HeightFixedPx => 1,
            UIProperty::HeightFixedPercentage => 2,
            UIProperty::BackgroundSolid => 3,
            UIProperty::SpacingFixedPx => 4,
            UIProperty::MarginFixedPx => 5,
            UIProperty::Event => 6,
        }
    }
}

impl UIProperty {
    pub(crate) fn into_dynamic_property(
        self,
        node: fastn_runtime::NodeKey,
        closure_pointer: fastn_runtime::ClosurePointer,
    ) -> DynamicProperty {
        DynamicProperty {
            property: self,
            node,
            closure: closure_pointer,
        }
    }
}
