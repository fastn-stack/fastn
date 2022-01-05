(function () {
    const FPM_MOBILE = "fpm-ui#mobile";
    const FPM_DARK_MODE = "fpm-ui#dark-mode"
    const FPM_FOLLOW_SYSTEM_DARK_MODE = "fpm-ui#follow-system-dark-mode"

    let last_device;

    initialise_device();
    initialise_dark_mode();

    function initialise_device() {
        last_device = is_mobile();
        window.ftd.set_bool_for_all(FPM_MOBILE, last_device);
    }

    window.onresize = function () {
        let current = is_mobile();
        if (current !== last_device) {
            window.ftd.set_bool_for_all(FPM_MOBILE, current);
            last_device = current;
        }
    }

    function is_mobile() {
        // not at all sure about this functions logic.
        let width = window.visualViewport.width;

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
        return width <= 1200;
    }

    window.show_main = function () {
        document.getElementById("main").style.display = "block";
        document.getElementById("fallback").style.display = "none";
    }

    window.show_fallback = function () {
        document.getElementById("main").style.display = "none";
        document.getElementById("fallback").style.display = "block";
    }

    /*
        fpm-ui.dark-mode behaviour:

        fpm-ui.dark-mode is a boolean, default false, it tells the UI to show
        the UI in dark or light mode. Themes should use this variable to decide
        which mode to show in UI.

        fpm-ui.dark-mode-follow-system, boolean, default true, keeps track if
        we are reading the value of `dark-mode` from system preference, or user
        has overridden the system preference.

        These two variables must not be set by ftd code directly, but they must
        use `$event-click$: message-host enable-dark-mode`, to ignore system
        preference and use dark mode. `$event-click$: message-host
        disable-dark-mode` to ignore system preference and use light mode and
        `$event-click$: message-host follow-system-dark-mode` to ignore user
        preference and start following system preference.

        we use a cookie: `fpm-dark-mode` to store the preference. The cookie can
        have three values:

           cookie missing /          user wants us to honour system preference
               system-light          and currently its light.

           system-dark               follow system and currently its dark.

           light:                    user prefers light

           dark:                     user prefers light

        We use cookie instead of localstorage so in future `fpm-repo` can see
        users preferences up front and renders the HTML on service wide
        following user's preference.

     */

    const DARK_MODE_COOKIE = "fpm-dark-mode";
    const COOKIE_SYSTEM_LIGHT = "system-light";
    const COOKIE_SYSTEM_DARK = "system-dark";
    const COOKIE_DARK_MODE = "dark";
    const COOKIE_LIGHT_MODE = "light";

    window.enable_dark_mode = function () {
        // TODO: coalesce the two set_bool-s into one so there is only one DOM
        //       update
        window.ftd.set_bool_for_all(FPM_DARK_MODE, true);
        window.ftd.set_bool_for_all(FPM_FOLLOW_SYSTEM_DARK_MODE, false);
        set_cookie(DARK_MODE_COOKIE, COOKIE_DARK_MODE);
    }

    window.enable_light_mode = function () {
        // TODO: coalesce the two set_bool-s into one so there is only one DOM
        //       update
        window.ftd.set_bool_for_all(FPM_DARK_MODE, false);
        window.ftd.set_bool_for_all(FPM_FOLLOW_SYSTEM_DARK_MODE, false);
        set_cookie(DARK_MODE_COOKIE, COOKIE_LIGHT_MODE);
    }

    window.enable_system_mode = function () {
        // TODO: coalesce the two set_bool-s into one so there is only one DOM
        //       update
        window.ftd.set_bool_for_all(FPM_FOLLOW_SYSTEM_DARK_MODE, true);
        if (system_dark_mode()) {
            window.ftd.set_bool_for_all(FPM_DARK_MODE, true);
            set_cookie(DARK_MODE_COOKIE, COOKIE_SYSTEM_DARK)
        } else {
            window.ftd.set_bool_for_all(FPM_DARK_MODE, false);
            set_cookie(DARK_MODE_COOKIE, COOKIE_SYSTEM_LIGHT)
        }
    }


    function initialise_dark_mode() {
        update_dark_mode();
        start_watching_dark_mode_system_preference();
    }

    function update_dark_mode() {
        let current_dark_mode_cookie = get_cookie(DARK_MODE_COOKIE, COOKIE_SYSTEM_LIGHT);

        switch (current_dark_mode_cookie) {
            case COOKIE_SYSTEM_LIGHT:
            case COOKIE_SYSTEM_DARK:
                window.enable_system_mode();
                break;
            case COOKIE_LIGHT_MODE:
                window.enable_light_mode();
                break;
            case COOKIE_DARK_MODE:
                window.enable_dark_mode();
                break;
            default:
                console.log("cookie value is wrong", current_dark_mode_cookie);
                window.enable_system_mode();
        }
    }

    function get_cookie(name, def) {
        // source: https://stackoverflow.com/questions/5639346/
        return document.cookie.match('(^|;)\\s*' + name + '\\s*=\\s*([^;]+)')?.pop() || def
    }

    function set_cookie(name, value) {
        document.cookie = name + "=" + value;
    }

    function system_dark_mode() {
        return !!(window.matchMedia && window.matchMedia('(prefers-color-scheme: dark)').matches)
    }

    function start_watching_dark_mode_system_preference() {
        window.matchMedia && window.matchMedia('(prefers-color-scheme: dark)').addEventListener(
            "change", update_dark_mode
        );
    }
})();


