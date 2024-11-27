(module
    (import "fastn" "create_column" (func $create_column (result externref)))
    (import "fastn" "root_container" (func $root_container (result externref)))
    (import "fastn" "set_column_width_px" (func $set_column_width_px (param externref i32)))
    (import "fastn" "set_column_height_px" (func $set_column_height_px (param externref i32)))

    ;; fastn.add_child(parent: NodeKey, child: NodeKey)
    (import "fastn" "add_child" (func $add_child (param externref externref)))

    (func (export "main") (local $column externref) (local $root_container_ externref)
        (local.set $root_container_ (call $root_container))

        ;; -- ftd.column:
        (call $foo (local.get $root_container_) (i32.const 100) (i32.const 100))
        drop

        (call $foo (local.get $root_container_) (i32.const 200) (i32.const 800))
        drop
    )

    (func $foo
        (param $root externref)
        (param $width i32)
        (param $height i32)

        (result externref)

        (local $column externref)

        ;; body

        (local.set $column (call $create_column))

        (call $add_child (local.get $root) (local.get $column))
        (call $set_column_width_px (local.get $column) (local.get $width))
        (call $set_column_height_px (local.get $column) (local.get $height))

        (local.get $column)
    )
)