"use strict";
let hello = "hello";
window.ftd = (function () {
    let exports = {};
    exports.init = function () {
        console.log(hello);
    };
    return exports;
})();
