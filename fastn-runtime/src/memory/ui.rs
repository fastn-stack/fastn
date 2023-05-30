#[derive(Debug)]
pub struct DynamicProperty {
    pub property: fastn_runtime::UIProperty,
    pub node: fastn_runtime::NodeKey,
    pub closure: Option<fastn_runtime::ClosureKey>,
}

impl DynamicProperty {
    pub(crate) fn closure(self, closure: fastn_runtime::ClosureKey) -> Self {
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
}

impl From<i32> for UIProperty {
    fn from(i: i32) -> UIProperty {
        match i {
            0 => UIProperty::WidthFixedPx,
            1 => UIProperty::HeightFixedPx,
            2 => UIProperty::HeightFixedPercentage,
            3 => UIProperty::BackgroundSolid,
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
        }
    }
}

impl UIProperty {
    pub(crate) fn into_ui_dependent(self, node: fastn_runtime::NodeKey) -> DynamicProperty {
        DynamicProperty {
            property: self,
            node,
            closure: None,
        }
    }
}
