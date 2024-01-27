{"redirect":"http://localhost:8080/hello/","user":{"created_at":"2024-01-15T10:56:59.885382Z","id":35,"name":"John","username":"john"}}
fastn.test_results = {};

fastn.test_results["1"] = {"data":null,"errors":{"payload":"invalid payload: SerdeJsonError(Error(\"missing field `email`\", line: 1, column: 24))"}};

fastn.http_status = 200;

fastn.http_location = "";



fastn.test_result = [];


fastn.assert = {eq: function (a, b) {n        a = fastn_utils.getStaticValue(a);n        b = fastn_utils.getStaticValue(b);n        fastn.test_result.push(a === b);n    },n    ne: function (a, b) {n        a = fastn_utils.getStaticValue(a);n        b = fastn_utils.getStaticValue(b);n        fastn.test_result.push(a !== b);n    },n    exists: function (a) {n        a = fastn_utils.getStaticValue(a);n        fastn.test_result.push(a !== undefined);n    },n    not_empty: function (a) {n        a = fastn_utils.getStaticValue(a);n        if (Array.isArray(a)) {n            fastn.test_result.push(a.length > 0);n        }n        if (a instanceof String) {n            fastn.test_result.push(a.length > 0);n        }n        fastn.test_result.push(a !== undefined);n    }};

fastn.assert.eq(1, 1);


nfastn.test_result

