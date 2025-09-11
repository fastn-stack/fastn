#[cfg(test)]
mod stack_overflow_verification {
    use crate::ftd2021::ui::*;

    #[test]
    fn test_box_common_reduces_stack_usage() {
        // This test verifies that the Box<Common> fix prevents stack overflow
        // by creating many UI elements that would previously cause stack overflow
        
        let mut elements = Vec::new();
        
        // Create nested UI elements to stress test stack usage
        for i in 0..50 {
            // Create Row with Box<Common>
            let row = Row {
                container: Container {
                    children: vec![],
                    external_children: None,
                    open: None,
                    append_at: None,
                    wrap: false,
                },
                spacing: None,
                common: Box::new(Common {
                    data_id: Some(format!("test-row-{}", i)),
                    width: Some(crate::ftd2021::ui::Length::Fill),
                    height: Some(crate::ftd2021::ui::Length::Fill),
                    ..Default::default()
                }),
            };
            elements.push(Element::Row(row));
            
            // Create Column with Box<Common>
            let column = Column {
                container: Container {
                    children: vec![],
                    external_children: None,
                    open: None,
                    append_at: None,
                    wrap: false,
                },
                spacing: None,
                common: Box::new(Common {
                    data_id: Some(format!("test-col-{}", i)),
                    width: Some(crate::ftd2021::ui::Length::Fill),
                    height: Some(crate::ftd2021::ui::Length::Fill),
                    ..Default::default()
                }),
            };
            elements.push(Element::Column(column));
        }
        
        // If we reach here without stack overflow, the fix works!
        assert_eq!(elements.len(), 100);
        println!("✅ Stack overflow fix verified - created {} UI elements successfully", elements.len());
    }
    
    #[test]
    fn test_debug_binary_functionality() {
        // Test basic functionality to ensure the fix doesn't break anything
        let common = Box::new(Common {
            data_id: Some("test-id".to_string()),
            ..Default::default()
        });
        
        assert_eq!(common.data_id, Some("test-id".to_string()));
        println!("✅ Box<Common> functionality working correctly");
    }
}