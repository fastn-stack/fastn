use crate::ftd_types::{ComponentType, FtdSize, SimpleFtdComponent};
use taffy::{Dimension, FlexDirection, LengthPercentage, LengthPercentageAuto, Rect, Size, Style};

/// Maps FTD properties to Taffy CSS styles
pub struct FtdToCssMapper;

impl FtdToCssMapper {
    pub fn new() -> Self {
        Self
    }

    /// Convert FTD component to Taffy style
    pub fn component_to_style(&self, component: &SimpleFtdComponent) -> Style {
        let mut style = Style::default();

        // Map size properties
        if let Some(width) = &component.width {
            style.size.width = self.map_size(width);
        }

        if let Some(height) = &component.height {
            style.size.height = self.map_size(height);
        }

        // Map padding (simplified to uniform for Week 1)
        if let Some(padding_px) = component.padding {
            let padding_value = LengthPercentage::Length((padding_px as f32).into());
            style.padding = Rect {
                left: padding_value,
                right: padding_value,
                top: padding_value,
                bottom: padding_value,
            };
        }

        // Map margin (simplified to uniform for Week 1)
        if let Some(margin_px) = component.margin {
            let margin_value = LengthPercentageAuto::Length((margin_px as f32).into());
            style.margin = Rect {
                left: margin_value,
                right: margin_value,
                top: margin_value,
                bottom: margin_value,
            };
        }

        // Map border-width (simplified to uniform for Week 1)
        if let Some(border_px) = component.border_width {
            let border_value = LengthPercentage::Length((border_px as f32).into());
            style.border = Rect {
                left: border_value,
                right: border_value,
                top: border_value,
                bottom: border_value,
            };
        }

        // Map container-specific properties
        match component.component_type {
            ComponentType::Column => {
                style.flex_direction = FlexDirection::Column;

                // Map spacing to gap for columns
                if let Some(spacing_px) = component.spacing {
                    style.gap = Size {
                        width: LengthPercentage::Length(0.0.into()),
                        height: LengthPercentage::Length((spacing_px as f32).into()),
                    };
                }
            }
            ComponentType::Row => {
                style.flex_direction = FlexDirection::Row;

                // Map spacing to gap for rows
                if let Some(spacing_px) = component.spacing {
                    style.gap = Size {
                        width: LengthPercentage::Length((spacing_px as f32).into()),
                        height: LengthPercentage::Length(0.0.into()),
                    };
                }
            }
            ComponentType::Text | ComponentType::Container => {
                // Text and container don't have specific flex direction
            }
        }

        style
    }

    /// Convert FTD size to Taffy dimension
    fn map_size(&self, size: &FtdSize) -> Dimension {
        match size {
            FtdSize::Fixed { px } => Dimension::Length((*px as f32).into()),
            FtdSize::FillContainer => Dimension::Percent(1.0),
            FtdSize::HugContent => Dimension::Auto,
            FtdSize::Percent { value } => Dimension::Percent(*value as f32 / 100.0),
        }
    }
}

impl Default for FtdToCssMapper {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_text_component_mapping() {
        let mapper = FtdToCssMapper::new();
        let component = SimpleFtdComponent::text("Hello")
            .with_width(FtdSize::Fixed { px: 100 })
            .with_padding(8);

        let style = mapper.component_to_style(&component);

        assert_eq!(style.size.width, Dimension::Length(100.0.into()));
        assert_eq!(style.padding.left, LengthPercentage::Length(8.0.into()));
    }

    #[test]
    fn test_column_layout_mapping() {
        let mapper = FtdToCssMapper::new();
        let component = SimpleFtdComponent::column().with_spacing(16);

        let style = mapper.component_to_style(&component);

        assert_eq!(style.flex_direction, FlexDirection::Column);
        assert_eq!(style.gap.height, LengthPercentage::Length(16.0.into()));
    }

    #[test]
    fn test_fill_container_mapping() {
        let mapper = FtdToCssMapper::new();
        let component = SimpleFtdComponent::text("Test").with_width(FtdSize::FillContainer);

        let style = mapper.component_to_style(&component);

        assert_eq!(style.size.width, Dimension::Percent(1.0));
    }
}
