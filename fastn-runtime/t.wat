(module
    (import "fastn" "create_frame" (func $create_frame))
    (import "fastn" "end_frame" (func $end_frame))
    (import "fastn" "return_frame" (func $return_frame (param externref) (result externref)))
    (import "fastn" "get_global" (func $get_global (param i32) (result externref)))
    (import "fastn" "set_global" (func $set_global (param i32) (param externref)))
    (import "fastn" "create_boolean" (func $create_boolean (param i32) (result externref)))
    (import "fastn" "create_list" (func $create_list (result externref)))
    (import "fastn" "create_list_1" (func $create_list_1 (param i32) (param externref) (result externref)))
    (import "fastn" "create_list_2" (func $create_list_2 (param i32) (param externref) (param i32) (param externref) (result externref)))
    (import "fastn" "get_boolean" (func $get_boolean (param externref) (result i32)))
    (import "fastn" "set_boolean" (func $set_boolean (param externref) (param i32)))
    (import "fastn" "create_i32" (func $create_i32 (param i32) (result externref)))
    (import "fastn" "get_i32" (func $get_i32 (param externref) (result i32)))
    (import "fastn" "set_i32" (func $set_i32 (param externref) (param i32)))
    (import "fastn" "create_f32" (func $create_f32 (param f32) (result externref)))
    (import "fastn" "multiply_i32" (func $multiply_i32 (param externref) (param i32) (param i32) (result externref)))
    (import "fastn" "get_f32" (func $get_f32 (param externref) (result f32)))
    (import "fastn" "set_f32" (func $set_f32 (param externref) (param f32)))
    (import "fastn" "array_i32_2" (func $array_i32_2 (param externref) (param externref) (result externref)))
    (import "fastn" "create_kernel" (func $create_kernel (param externref) (param i32) (result externref)))
    (import "fastn" "set_property_i32" (func $set_property_i32 (param externref) (param i32) (param i32)))
    (import "fastn" "set_property_f32" (func $set_property_f32 (param externref) (param i32) (param f32)))
    (import "fastn" "set_dynamic_property_i32" (func $set_dynamic_property_i32 (param externref) (param i32) (param i32) (param externref)))

    (table 1 funcref)
    (elem (i32.const 0) $product)

    (type $return_externref
        (func (param externref) (result externref))
    )

    (func (export "call_by_index")
        (param $idx i32)
        (param $arr externref)

        (result externref)

        (call_indirect (type $return_externref) (local.get $arr) (local.get $idx))
    )

    (func $product
        (param $func-data externref)
        (result externref)

        (call $create_frame)

        (call $return_frame
            (call $multiply_i32
                (local.get $func-data) (i32.const 0) (i32.const 1)
            )
        )
    )

    (func (export "main")
        (param $root externref)
        (local $column externref)

        (call $create_frame ) (call $set_global (i32.const 0) (call $create_boolean (i32.const 0))) (call $set_global (i32.const 1) (call $create_i32 (i32.const 10))) (local.set $column (call $create_kernel (local.get $root) (i32.const 0))) (call $set_dynamic_property_i32 (local.get $column) (i32.const 0) (i32.const 0) (call $array_i32_2 (call $create_i32 (i32.const 10)) (call $get_global (i32.const 1)))) (call $end_frame )
    )
)
