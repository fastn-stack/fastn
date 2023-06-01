#[derive(Debug)]
pub struct DynamicProperty {
    pub property: fastn_runtime::UIProperty,
    pub node: fastn_runtime::NodeKey,
    pub closure: Option<fastn_runtime::ClosurePointer>,
}

impl DynamicProperty {
    pub(crate) fn closure(self, closure: fastn_runtime::ClosurePointer) -> Self {
        let mut ui_dependent = self;
        ui_dependent.closure = Some(closure);
        ui_dependent
    }
}

#[derive(Debug, Copy, Clone)]
pub enum UIProperty {
    WidthFixedPx,
    HeightFixedPx,
    HeightFixedPercentage,
    BackgroundSolid,
    SpacingFixedPx,
}

impl From<i32> for UIProperty {
    fn from(i: i32) -> UIProperty {
        match i {
            0 => UIProperty::WidthFixedPx,
            1 => UIProperty::HeightFixedPx,
            2 => UIProperty::HeightFixedPercentage,
            3 => UIProperty::BackgroundSolid,
            4 => UIProperty::SpacingFixedPx,
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
        }
    }
}

impl UIProperty {
    pub(crate) fn into_dynamic_property(self, node: fastn_runtime::NodeKey) -> DynamicProperty {
        DynamicProperty {
            property: self,
            node,
            closure: None,
        }
    }
}
