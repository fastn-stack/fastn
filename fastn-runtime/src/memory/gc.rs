impl fastn_runtime::Memory {
    pub fn insert_in_frame(
        &mut self,
        pointer: fastn_runtime::PointerKey,
        kind: fastn_runtime::PointerKind,
    ) {
        // using .unwrap() so we crash on a bug instead of silently ignoring it
        let frame = self.stack.last_mut().unwrap();
        let pointer = fastn_runtime::Pointer { pointer, kind };
        for p in frame.pointers.iter() {
            if p == &pointer {
                panic!();
            }
        }
        frame.pointers.push(pointer);
    }

    pub fn add_parent(&mut self, target: fastn_runtime::Pointer, parent: fastn_runtime::Pointer) {
        let branches = parent.get_branches(self);
        let m = target.get_branches_mut(self);

        for a in branches {
            dbg!(a);
            // TODO: this has to be done recursively. We will also need a vec to ensure we only
            //       visit each node only once
            m.insert(a);
        }
    }

    pub fn drop_pointer(
        &mut self,
        pointer: fastn_runtime::Pointer,
        dropped_so_far: &mut Vec<fastn_runtime::Pointer>,
        parent_vec: Option<fastn_runtime::Pointer>,
    ) -> bool {
        println!("consider dropping {:?} {:?}", pointer, parent_vec);
        if dropped_so_far.contains(&pointer) {
            println!("pointer already dropped, ignoring: {:?}", pointer);
            return true;
        }

        // TODO: rewrite this function completely

        let (dependents, values, ui_properties) = match pointer.kind {
            fastn_runtime::PointerKind::Boolean => {
                let b = self.boolean.get(pointer.pointer).unwrap();
                (&b.parents, vec![], &b.ui_properties)
            }
            fastn_runtime::PointerKind::Integer => {
                let b = self.i32.get(pointer.pointer).unwrap();
                (&b.parents, vec![], &b.ui_properties)
            }
            fastn_runtime::PointerKind::Record | fastn_runtime::PointerKind::List => {
                let b = self.vec.get(pointer.pointer).unwrap();
                (&b.parents, b.value.value().to_vec(), &b.ui_properties)
            }
            fastn_runtime::PointerKind::OrType => {
                let b = self.or_type.get(pointer.pointer).unwrap();
                (&b.parents, vec![], &b.ui_properties)
            }
            fastn_runtime::PointerKind::Decimal => {
                let b = self.f32.get(pointer.pointer).unwrap();
                (&b.parents, vec![], &b.ui_properties)
            }
            fastn_runtime::PointerKind::String => {
                let b = self.string.get(pointer.pointer).unwrap();
                (&b.parents, vec![], &b.ui_properties)
            }
        };

        if !ui_properties.is_empty() {
            return false;
        }

        let mut drop = true;

        for d in dependents.clone() {
            if let Some(parent_vec) = parent_vec {
                if d.eq(&parent_vec) {
                    continue;
                }
            }
            if !self.drop_pointer(d, dropped_so_far, None) {
                drop = false;
                break;
            }
        }

        for d in values {
            if !self.drop_pointer(d, dropped_so_far, Some(pointer)) {
                drop = false;
                break;
            }
        }

        if drop {
            println!("dropping {:?} {:?}", pointer, parent_vec);
            dropped_so_far.push(pointer);
            self.delete_pointer(pointer);
        }

        drop
    }

    pub fn delete_pointer(&mut self, pointer: fastn_runtime::Pointer) {
        match pointer.kind {
            fastn_runtime::PointerKind::Boolean => {
                self.boolean.remove(pointer.pointer);
            }
            fastn_runtime::PointerKind::Integer => {
                self.i32.remove(pointer.pointer);
            }
            fastn_runtime::PointerKind::Record | fastn_runtime::PointerKind::List => {
                self.vec.remove(pointer.pointer);
            }
            fastn_runtime::PointerKind::OrType => {
                self.or_type.remove(pointer.pointer);
            }
            fastn_runtime::PointerKind::Decimal => {
                self.f32.remove(pointer.pointer);
            }
            fastn_runtime::PointerKind::String => {
                self.string.remove(pointer.pointer);
            }
        };
    }
}
