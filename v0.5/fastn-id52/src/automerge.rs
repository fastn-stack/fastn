impl autosurgeon::Reconcile for crate::PublicKey {
    type Key<'a> = &'a str;

    fn reconcile<R: autosurgeon::Reconciler>(&self, reconciler: R) -> Result<(), R::Error> {
        self.id52().reconcile(reconciler)
    }
}

impl autosurgeon::Hydrate for crate::PublicKey {
    fn hydrate<D: autosurgeon::ReadDoc>(
        doc: &D,
        obj: &automerge::ObjId,
        prop: autosurgeon::Prop<'_>,
    ) -> Result<Self, autosurgeon::HydrateError> {
        let id52_str: String = autosurgeon::Hydrate::hydrate(doc, obj, prop)?;
        std::str::FromStr::from_str(&id52_str)
            .map_err(|e| autosurgeon::HydrateError::unexpected("PublicKey", format!("{e}")))
    }
}

impl autosurgeon::Reconcile for crate::SecretKey {
    type Key<'a> = &'a str;

    fn reconcile<R: autosurgeon::Reconciler>(&self, reconciler: R) -> Result<(), R::Error> {
        self.to_string().reconcile(reconciler)
    }
}

impl autosurgeon::Hydrate for crate::SecretKey {
    fn hydrate<D: autosurgeon::ReadDoc>(
        doc: &D,
        obj: &automerge::ObjId,
        prop: autosurgeon::Prop<'_>,
    ) -> Result<Self, autosurgeon::HydrateError> {
        let hex_str: String = autosurgeon::Hydrate::hydrate(doc, obj, prop)?;
        std::str::FromStr::from_str(&hex_str)
            .map_err(|e| autosurgeon::HydrateError::unexpected("SecretKey", format!("{e}")))
    }
}

impl autosurgeon::Reconcile for crate::Signature {
    type Key<'a> = &'a str;

    fn reconcile<R: autosurgeon::Reconciler>(&self, reconciler: R) -> Result<(), R::Error> {
        self.to_string().reconcile(reconciler)
    }
}

impl autosurgeon::Hydrate for crate::Signature {
    fn hydrate<D: autosurgeon::ReadDoc>(
        doc: &D,
        obj: &automerge::ObjId,
        prop: autosurgeon::Prop<'_>,
    ) -> Result<Self, autosurgeon::HydrateError> {
        let hex_str: String = autosurgeon::Hydrate::hydrate(doc, obj, prop)?;
        std::str::FromStr::from_str(&hex_str)
            .map_err(|e| autosurgeon::HydrateError::unexpected("Signature", format!("{e}")))
    }
}
