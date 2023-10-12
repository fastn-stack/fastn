const createGlobalRef = () => {
    let ref = null;

    return {
        get: () => ref,
        set: global => ref = global,
    }
};

const __fastn_legacy_global_ref__ = createGlobalRef();

ftd.set_value = function() {
    console.log(__fastn_legacy_global_ref__.get());
}
