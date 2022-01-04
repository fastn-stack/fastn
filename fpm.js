(function () {
    const MOBILE_VARIABLE = "fpm-ui#mobile";
    // const PIXEL_RATIO = window.devicePixelRatio || 1;

    let last = is_mobile();
    window.ftd.set_bool_for_all(MOBILE_VARIABLE, last);

    window.onresize = function () {
        let current = is_mobile();
        if (current !== last) {
            window.ftd.set_bool_for_all(MOBILE_VARIABLE, current);
            last = current;
        }
    }

    function is_mobile() {
        // not at all sure about this functions logic.
        let width = window.visualViewport.width;

        // why 1148? this is the smallest width my desktop safari can shrink to.
        // in future we may want to have more than one break points, and then
        // we may also want the theme builders to decide where the breakpoints
        // should go. we should be able to fetch fpm variables here, or maybe
        // simply pass the width, user agent etc to fpm and let people put the
        // checks on width user agent etc, but it would be good if we can
        // standardise few breakpoints. or maybe we should do both, some
        // standard breakpoints and pass the raw data.

        // we would then rename this function to detect_device() which will
        // return one of "desktop", "tablet", "mobile". and also maybe have
        // another function detect_orientation(), "landscape" and "portrait" etc,
        // and instead of setting `fpm-ui#mobile: boolean` we set `fpm-ui#device`
        // and `fpm-ui#viewport-orientation` etc.
        return width <= 1000;
    }

    window.show_main = function () {
        document.getElementById("main").style.display = "block";
        document.getElementById("fallback").style.display = "none";
    }

    window.show_fallback = function () {
        document.getElementById("main").style.display = "none";
        document.getElementById("fallback").style.display = "block";
    }

})();


