-- component _main:

-- ftd.column:

-- string message: hello
-- ftd.text: $message

-- end: ftd.column
-- end: _main

-- _main:




-- string message: hello
-- ftd.text: $message



(module
    (import "fastn" "create_column" (func $create_column (result externref)))
    (import "fastn" "set_column_width_px" (func $set_column_width_px (param externref i32)))
    (import "fastn" "set_column_height_px" (func $set_column_height_px (param externref i32)))

    (table 20 funcref)
    (elem (i32.const 0) $foo_click $foo_width $foo_z)

    ;; fastn.add_child(parent: NodeKey, child: NodeKey)
    (import "fastn" "add_child" (func $add_child (param externref externref)))

    (func $malloc (param $size i32) (result i32)
        (global.set 0 (i32.add (global.get 0) (local.get $size)))
        (i32.add (global.get 0) (local.get $size))
    )

    (func (export "main")
        (param $root externref)
        (local $x_ptr i32)

        ;; body
        (local.set $x_ptr (call create_var (i32.const 10))

        ;; -- foo:
        (call $foo (local.get $root) (i32.const 100) (i32.const 100) (local $x_ptr))

        ;; -- foo:
        (call $foo (local.get $root) (i32.const 200) (i32.const 300) (local $x_ptr))
    )

    ;; $on-click$: { $i = $i + 1 }
    (func $foo_click
        (param $arr i32)
        (local $i_ptr i32)
        ;; body

        (local.set $i_ptr
            (array_resolve (local.get $arr) (i32.const 0))
        )

        (call $set_var_value_i32
            (local.get $i_ptr)
            (i32.add (global.get $i_ptr) (i32.const 1))
        )
    )

    (func $set_var_value_i32 (param $var_ptr i32) (param $value i32)
        (global.set (local.get $var_ptr) (local.get $value))
        ;; update data
        (call $update_data_for_var
            (global.get (i32.add (local.get $var_ptr) (i32.const 2)))
        )

        ;; notify host that ui has changed
        (call $update_ui_for_var
            (global.get (i32.add (local.get $var_ptr) (i32.const 1)))
        )
    )

    (type $ui_func_type (func (param externref) (param i32)))

    (call $update_ui_for_var (param $ui_arr i32)
        (for ($item in $ui_arr)
            (array_get $item 2) ;; func_data
            (array_get $item 1) ;; element
            (call_indirect (array_get $item 0) (type $ui_func_type))
        )
    )

    (func $add_var_ui_dependency
        (param $var_ptr i32)
        (param $ui_func i32)
        (param $element externref)
        (param $ui_func_args i32)

        ;; body

        (array_push (i32.add (global.get (local $var_ptr)) (i32.const 1))
            (create_array_3 (local.get $ui_func) (local.get $element) (local.get $ui_func_args))
        )
    )

    ;; private integer z: { $i * $j }
    (func $foo_z (param $i_ptr i32) (param $j_ptr i32) (result i32)
        (i32.mul (global.get $i_ptr) (global.get $j_ptr))
    )

    ;; height.fixed.px:  { $i * 100 }
    (func $foo_height (param $element externref) (param $func_data i32)
        (call $set_column_height_px
            (local.get $element)
            (i32.mult (array_resolve (local.get $func_data) (i32.const 0)) (i32.const 100))
        )
    )

    (func $foo_width (param $element externref) (param $func_data i32)
        ???
    )

    ;; integer x: 10

    ;; -- component foo:
    ;; private integer $i: $x
    ;; private integer $j: 0
    ;; private integer z: { $i * $j }
    ;;
    ;; -- ftd.column:
    ;; $on-click$: { $i = $i + 1 }
    ;; $on-mouse-over$: { $j = $EVENT.x }
    ;; width.fixed.px: { $z }
    ;; height.fixed.px:  { $i * 100 }
    ;; background.solid: red
    ;;
    ;; -- end: ftd.column
    ;;
    ;; -- end: foo



    ;; struct Var {
    ;;    value: i32,
    ;;    ui_deps: Vec<UIData>, // function pointer, element, variables used by that function
    ;;    data_deps: Vec<Ref<VarData>>,
    ;; }

    ;; struct UIData {
    ;;     func: i32,
    ;;     elem: ExternRef,
    ;;     vars: Vec<Ref<Var>>,
    ;; }

    ;; struct VarData {
    ;;     func: i32,
    ;;     vars: Vec<Ref<Var>>,
    ;; }


    (func $foo
        (param $root externref)
        (param $width i32)
        (param $height i32)
        (param $x_ptr)

        (local $column externref)
        (local $i_ptr i32)
        (local $j_ptr i32)
        (local $z_formula i32)
        (local $z_ij_arr i32)
        ;; (local $fun_width_ptr i32)

        ;; body
        ;; all vars are i32

        ;; private integer $i: $x
        (local.set $i_ptr (call $create_var_with (global.get $x_ptr))
        ;; x is not mutable so we do not create a dependency from x to i.

        ;; private integer $j: 0
        (local.set $j_ptr (call $create_var_with_constant (i32.const 0))

        ;; private integer z: { $i * $j }
        (local.set $z_ij_arr (call create_array))
        (array_push (local.get $z_ij_arr) (local.get $i_ptr))
        (array_push (local.get $z_ij_arr) (local.get $j_ptr))

        (local.set $z_formula (call $create_foruma 2 (local.get $z_ij_arr))

        (var_add_dep (local.get $i_ptr)
            2 ;; the pointer to foo_z in the table
            (local.get $z_ij_arr)
        )
        (var_add_dep (local.get $j_ptr)
            2 ;; the pointer to foo_z in the table
            (local.get $z_ij_arr)
        )

        ;; -- ftd.column:
        (local.set $column (call $create_column))

        ;; $on-click$: { $i = $i + 1 }

        (local $click_i_arr (call create_array))
        (array_push (local.get $click_i_arr) (local.get $i_ptr))

        (call $on_click
            (local.get $column)
            0 ;; foo_click's pointer in the table
            (local.get $click_i_arr)
        )

        ;; height.fixed.px:  { $i * 100 }
        (local $height_arr (call create_array))
        (array_push (local.get $height_arr) (local.get $i_ptr))

        (call $add_var_ui_dependency
            (local.get $i_ptr)
            1 ;; $foo_height
            (local.get $column)
            (local.get $height_arr)
        )










        (add_var_dependency (local.get $i_ptr)
            1 ;; $foo_width
        )


        (call $add_child (local.get $root) (local.get $column))
        (call $set_column_width_px (local.get $column) (local.get $width))
        (call $set_column_height_px (local.get $column) (local.get $height))

        (call $add_on_click (local.get $column) (i32.const 0) (local.get $i_ptr))
    )

    (type $call_func_with_array_type (func (param i32)))

    (func (export call_func_with_array)
        (param $func i32)
        (param $arr i32)

        ;; body
        (local.get $arr)

        (call_indirect (type $call_func_with_array_type) (local.get $func))
    )
)