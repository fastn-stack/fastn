#[derive(Debug)]
pub struct DynamicProperty {
    pub property: fastn_runtime::UIProperty,
    pub node: fastn_runtime::NodeKey,
    pub closure: fastn_runtime::ClosurePointer,
}

#[derive(Debug, Copy, Clone)]
pub enum UIProperty {
    WidthFixedPx,
    HeightFixedPx,
    HeightFixedPercentage,
    BackgroundSolid,
    SpacingFixedPx,
    MarginFixedPx,
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
        }
    }
}

impl UIProperty {
    pub(crate) fn into_dynamic_property(self, node: fastn_runtime::NodeKey, closure: fastn_runtime::ClosurePointer) -> DynamicProperty {
        DynamicProperty {
            property: self,
            node,
            closure,
        }
    }
}
