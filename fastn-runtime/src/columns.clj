(module
    (import "fastn" "create_frame" (func $create_frame))
    (import "fastn" "end_frame" (func $end_frame))
    (import "fastn" "return_frame" (func return_frame (param externref) (result externref)))
    (import "fastn" "create_kernel" (func $create_kernel (param i32 externref) (result externref)))
    (import "fastn" "create_boolean" (func $create_boolean (param i32) (result externref)))
    (import "fastn" "create_i32" (func $create_i32 (param i32) (result externref)))
    (import "fastn" "create_rgba" (func $create_rgba (param i32 i32 i32 f32) (result externref)))
    (import "fastn" "set_i32_prop" (func $set_i32_prop (param externref i32 i32)))
    (import "fastn" "set_f32_prop" (func $set_f32_prop (param externref i32 f32)))
    (import "fastn" "set_boolean" (func $set_boolean (param externref i32) (result externref)))
    (import "fastn" "attach_event_handler" (func $attach_event_handler (param externref i32 i32 externref)))
    ;; set_dynamic_property_i32(element, prop, func, variables)
    ;; prop = 0 = fixed width in pixels etc
    ;; func = function to call, index in the table, func must return i32
    ;; variables = array containing variables to pass to the function
    (import "fastn" "set_dynamic_property_i32" (func $set_dynamic_property_i32 (param externref i32 i32 externref)))
    (import "fastn" "set_i32_3_prop_func" (func $set_i32_3_prop_func (param externref i32 i32 externref)))
    (import "fastn" "get_func_arg_i32" (func $get_func_arg_i32 (param externref i32) (result i32)))
    (import "fastn" "array_i32_2" (func $array_i32_2 (param externref externref) (result externref)))

    (global $main#any-hover externref)
    (global $main#x externref)

    (table 3 func)
    (elem (i32.const 0) $product $foo#on_mouse_enter $foo#on_mouse_leave $foo#background)
    (type $return_externref (func (param externref) (result externref)))

    (func (export "main") (param $root externref)
        (local $column externref)

        (call $create_frame)

        ;; -- boolean $any-hover: false
        (global.set $main#any-hover (call $create_boolean (i32.const 0)))

        ;; -- integer x: 10
        (global.set $main#x
            (call $create_i32 (i32.const 10))
        )

        ;; -- ftd.column:
        (local.set $column (call $create_kernel (i32.const 0) (local.get $root)))

        ;; width.fixed.px: $product(a=10, b=$x)
        (call $set_dynamic_property_i32
            (local.get $column)
            (i32.const 0) ;; 0 = fixed width in pixels
            (i32.const 0) ;; index in the table for $product function
            (call $array_i32_2 (global.set $main#x) (call $create_integer (i32.const 10)))
        )

        ;; height.fixed.px: 500
        (call $set_i32_prop
            (local.get $column)
            (i32.const 1) ;; 1 = fixed height in pixels
            (i32.const 500) ;; fixed value
        )

        ;; spacing.fixed.px: 100
        (call $set_i32_prop
            (local.get $column)
            (i32.const 2) ;; 2 = fixed spacing in pixels
            (i32.const 100) ;; fixed value
        )

        ;; margin.px: 100
        (call $set_i32_prop
            (local.get $column)
            (i32.const 2) ;; 3 = margin in px
            (i32.const 100) ;; fixed value
        )

        (call $foo (local.get $column))
        (call $foo (local.get $column))

        (call $end_frame)
    )

    (func $foo (param $parent externref)
        (local $column externref)
        (local $on-hover externref)

        (call $create_frame)

        (local.set $on-hover (call $create_boolean (i32.const 0)))

        ;; -- ftd.column:
        (local.set $column (call $create_kernel (i32.const 0) (local.get $parent)))

        ;; $on-mouse-enter$: {
        ;;     $ftd.set-bool($a=$any-hover, v=true)
        ;;     $ftd.set-bool($a=$foo.on-hover, v=true)
        ;; }
        (call $attach_event_handler
            (local.get $column)
            (i32.const 0) ;; 0 = on mouse enter
            (i32.const 1) ;; index in the table
            (call $array_i32_2 (global.get $main#any-hover) (local.get $on-hover))
        )
        ;; $on-mouse-leave$: {
        ;;     $ftd.set-bool($a=$any-hover, v=false)
        ;;     $ftd.set-bool($a=$foo.on-hover, v=false)
        ;; }
        (call $attach_event_handler
              (local.get $column)
              (i32.const 0) ;; 0 = on mouse enter
              (i32.const 2) ;; index in the table
              (call $array_i32_2 (global.get $main#any-hover) (local.get $on-hover))
        )

        ;; width.fixed.px: 500
        (call $set_i32_prop
              (local.get $column)
              (i32.const 0) ;; 1 = fixed height in pixels
              (i32.const 400) ;; fixed value
        )

        ;; width.fixed.px: 500
        (call $set_f32_prop
              (local.get $column)
              (i32.const 2) ;; 2 = fixed height in percentage
              (f32.const 30) ;; fixed value
        )

        ;; background.solid: red
        ;; background.solid if { foo.on-hover }: green
        ;; background.solid if { any-hover }: blue
        (call $set_i32_3_prop_func
                (local.get $column)
                (i32.const 3) ;; 3 = background.solid
                (i32.const 3) ;; index in the table
                (call $array_i32_2 (local.get $on-hover) (global.get $main#any-hover))
        )

        (call $end_frame)
    )

    (func $foo#background (param $func-data externref) (result externref)
        (call $create_frame)
        (if (call $get_func_arg_i32 (local.get $func-data) (i32.const 0))
            (then
                (call $create_rgba (i32.const 0) (i32.const 20) (i32.const 0) (f32.const 1.0))
            )
        (else
            (if (call $get_func_arg_i32 (local.get $func-data) (i32.const 1))
                (then
                    (call $create_rgba (i32.const 0) (i32.const 0) (i32.const 20) (f32.const 1.0))
                )
                (else
                    (call $create_rgba (i32.const 20) (i32.const 0) (i32.const 0) (f32.const 1.0))
                    )
                )
            )
        )
        (call $end_frame)
    )

    (func $foo#on_mouse_enter (param $func-data externref) (result externref)
        (call $create_frame)
        ;;     $ftd.set-bool($a=$any-hover, v=true)
        (call $set_boolean
            (call $get_arg_ref (local.get $func-data) (i32.const 0))
             (i32.const 1)
        )
        ;;     $ftd.set-bool($a=$foo.on-hover, v=true)
        (call $set_boolean
            (call $get_arg_ref (local.get $func-data) (i32.const 1))
             (i32.const 1)
        )
        (call $end_frame)
    )

    (func $foo#on_mouse_leave (param $func-data externref) (result externref)
        (call $create_frame)
       ;;     $ftd.set-bool($a=$any-hover, v=false)
       (call $set_boolean
             (call $get_arg_ref (local.get $func-data) (i32.const 0))
             (i32.const 0)
       )
       ;;     $ftd.set-bool($a=$foo.on-hover, v=false)
       (call $set_boolean
             (call $get_arg_ref (local.get $func-data) (i32.const 1))
             (i32.const 0)
       )
        (call $end_frame)
    )

    (func $product (param $func-data externref) (result externref)
        (call $create_frame)
        (call $return_frame
              (i32.mul
                   (call $get_func_arg_i32 (local.get $func-data) (i32.const 0))
                   (call $get_func_arg_i32 (local.get $func-data) (i32.const 1))
              )
          )
    )

    (func (export "callByIndex") (param i32 externref) (result externref)
       call_indirect (type $return_externref) (local.get 0) (local.get 1)
    )
)
