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
};
