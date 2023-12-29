fastn.test_result = [];

fastn.assert = {
    eq: function (a, b) {
        a = fastn_utils.getStaticValue(a);
        b = fastn_utils.getStaticValue(b);
        fastn.test_result.push(a === b);
    },
    ne: function (a, b) {
        a = fastn_utils.getStaticValue(a);
        b = fastn_utils.getStaticValue(b);
        fastn.test_result.push(a !== b);
    },
    exists: function(a) {
        a = fastn_utils.getStaticValue(a);
        fastn.test_result.push(a !== undefined);
    },
    not_empty: function(a) {
        a = fastn_utils.getStaticValue(a);
        if (Array.isArray(a)) {
            fastn.test_result.push(a.length > 0);
        }
        if (a instanceof String) {
            fastn.test_result.push(a.length > 0);
        }
        fastn.test_result.push(a !== undefined);
    }
};
