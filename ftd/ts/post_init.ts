window.ftd.post_init = function () {
    const DARK_MODE = "ftd#dark-mode";
    const SYSTEM_DARK_MODE = "ftd#system-dark-mode";
    const FOLLOW_SYSTEM_DARK_MODE = "ftd#follow-system-dark-mode";
    const DARK_MODE_COOKIE = "ftd-dark-mode";
    const COOKIE_SYSTEM_LIGHT = "system-light";
    const COOKIE_SYSTEM_DARK = "system-dark";
    const COOKIE_DARK_MODE = "dark";
    const COOKIE_LIGHT_MODE = "light";
    const DARK_MODE_CLASS = "fpm-dark";
    const MOBILE_CLASS = "ftd-mobile";
    const XL_CLASS = "ftd-xl";
    const FTD_DEVICE = "ftd#device";
    const FTD_BREAKPOINT_WIDTH = "ftd#breakpoint-width";
    let last_device: string;

   function initialise_device() {
        last_device = get_device();
        console_log("last_device", last_device);
        window.ftd.set_string_for_all(FTD_DEVICE, last_device);
    }

    window.onresize = function () {
        let current = get_device();
        if (current === last_device) {
            return;
        }

        window.ftd.set_string_for_all(FTD_DEVICE, current);
        last_device = current;
        console_log("last_device", last_device);
    };

    /*function update_markdown_colors() {
       // remove all colors from ftd.css: copy every deleted stuff in this function
       let markdown_style_sheet = document.createElement('style');


       markdown_style_sheet.innerHTML = `
       .ft_md a {
           color: ${window.ftd.get_value("main", MARKDOWN_COLOR + ".link.light")};
           background-color: ${window.ftd.get_value("main", MARKDOWN_BACKGROUND_COLOR + ".link.light")};
       }
       body.fpm-dark .ft_md a {
           color: ${window.ftd.get_value("main", MARKDOWN_COLOR + ".link.dark")};
           background-color: ${window.ftd.get_value("main", MARKDOWN_BACKGROUND_COLOR + ".link.dark")};
       }

       .ft_md code {
           color: ${window.ftd.get_value("main", MARKDOWN_COLOR + ".code.light")};
           background-color: ${window.ftd.get_value("main", MARKDOWN_BACKGROUND_COLOR + ".code.light")};
       }
       body.fpm-dark .ft_md code {
           color: ${window.ftd.get_value("main", MARKDOWN_COLOR + ".code.dark")};
           background-color: ${window.ftd.get_value("main", MARKDOWN_BACKGROUND_COLOR + ".code.dark")};
       }

       .ft_md a:visited {
           color: ${window.ftd.get_value("main", MARKDOWN_COLOR + ".link-visited.light")};
           background-color: ${window.ftd.get_value("main", MARKDOWN_BACKGROUND_COLOR + ".link-visited.light")};
       }
       body.fpm-dark .ft_md a:visited {
           color: ${window.ftd.get_value("main", MARKDOWN_COLOR + ".link-visited.dark")};
           background-color: ${window.ftd.get_value("main", MARKDOWN_BACKGROUND_COLOR + ".link-visited.dark")};
       }

       .ft_md a code {
           color: ${window.ftd.get_value("main", MARKDOWN_COLOR + ".link-code.light")};
           background-color: ${window.ftd.get_value("main", MARKDOWN_BACKGROUND_COLOR + ".link-code.light")};
       }
       body.fpm-dark .ft_md a code {
           color: ${window.ftd.get_value("main", MARKDOWN_COLOR + ".link-code.dark")};
           background-color: ${window.ftd.get_value("main", MARKDOWN_BACKGROUND_COLOR + ".link-code.dark")};
       }

       .ft_md a:visited code {
           color: ${window.ftd.get_value("main", MARKDOWN_COLOR + ".link-visited-code.light")};
           background-color: ${window.ftd.get_value("main", MARKDOWN_BACKGROUND_COLOR + ".link-visited-code.light")};
       }
       body.fpm-dark .ft_md a:visited code {
           color: ${window.ftd.get_value("main", MARKDOWN_COLOR + ".link-visited-code.dark")};
           background-color: ${window.ftd.get_value("main", MARKDOWN_BACKGROUND_COLOR + ".link-visited-code.dark")};
       }

       .ft_md ul ol li:before {
           color: ${window.ftd.get_value("main", MARKDOWN_COLOR + ".ul-ol-li-before.light")};
           background-color: ${window.ftd.get_value("main", MARKDOWN_BACKGROUND_COLOR + ".ul-ol-li-before.light")};
       }
       body.fpm-dark .ft_md ul ol li:before {
           color: ${window.ftd.get_value("main", MARKDOWN_COLOR + ".ul-ol-li-before.dark")};
           background-color: ${window.ftd.get_value("main", MARKDOWN_BACKGROUND_COLOR + ".ul-ol-li-before.dark")};
       }
       `;

       document.getElementsByTagName('head')[0].appendChild(markdown_style_sheet);
   }*/

   function get_device() {
       // not at all sure about this functions logic.
       let width = window.innerWidth;

       // in future we may want to have more than one break points, and then
       // we may also want the theme builders to decide where the breakpoints
       // should go. we should be able to fetch fpm variables here, or maybe
       // simply pass the width, user agent etc to fpm and let people put the
       // checks on width user agent etc, but it would be good if we can
       // standardize few breakpoints. or maybe we should do both, some
       // standard breakpoints and pass the raw data.

       // we would then rename this function to detect_device() which will
       // return one of "desktop", "tablet", "mobile". and also maybe have
       // another function detect_orientation(), "landscape" and "portrait" etc,
       // and instead of setting `fpm#mobile: boolean` we set `fpm-ui#device`
       // and `fpm#view-port-orientation` etc.
       let mobile_breakpoint = window.ftd.get_value("main", FTD_BREAKPOINT_WIDTH + ".mobile");
       if (width <= mobile_breakpoint) {
           document.body.classList.add(MOBILE_CLASS);
           if (document.body.classList.contains(XL_CLASS)) {
               document.body.classList.remove(XL_CLASS);
           }
           return "mobile";
       }
       /*if (width > desktop_breakpoint) {
           document.body.classList.add(XL_CLASS);
           if (document.body.classList.contains(MOBILE_CLASS)) {
               document.body.classList.remove(MOBILE_CLASS);
           }
           return "xl";
       }*/
       if (document.body.classList.contains(MOBILE_CLASS)) {
           document.body.classList.remove(MOBILE_CLASS);
       }
       /*if (document.body.classList.contains(XL_CLASS)) {
           document.body.classList.remove(XL_CLASS);
       }*/
       return "desktop";
   }

    /*
        ftd.dark-mode behaviour:

        ftd.dark-mode is a boolean, default false, it tells the UI to show
        the UI in dark or light mode. Themes should use this variable to decide
        which mode to show in UI.

        ftd.follow-system-dark-mode, boolean, default true, keeps track if
        we are reading the value of `dark-mode` from system preference, or user
        has overridden the system preference.

        These two variables must not be set by ftd code directly, but they must
        use `$on-click$: message-host enable-dark-mode`, to ignore system
        preference and use dark mode. `$on-click$: message-host
        disable-dark-mode` to ignore system preference and use light mode and
        `$on-click$: message-host follow-system-dark-mode` to ignore user
        preference and start following system preference.

        we use a cookie: `ftd-dark-mode` to store the preference. The cookie can
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

    window.enable_dark_mode = function () {
        // TODO: coalesce the two set_bool-s into one so there is only one DOM
        //       update
        window.ftd.set_bool_for_all(DARK_MODE, true);
        window.ftd.set_bool_for_all(FOLLOW_SYSTEM_DARK_MODE, false);
        window.ftd.set_bool_for_all(SYSTEM_DARK_MODE, system_dark_mode());
        document.body.classList.add(DARK_MODE_CLASS);
        set_cookie(DARK_MODE_COOKIE, COOKIE_DARK_MODE);
    };

    window.enable_light_mode = function () {
        // TODO: coalesce the two set_bool-s into one so there is only one DOM
        //       update
        window.ftd.set_bool_for_all(DARK_MODE, false);
        window.ftd.set_bool_for_all(FOLLOW_SYSTEM_DARK_MODE, false);
        window.ftd.set_bool_for_all(SYSTEM_DARK_MODE, system_dark_mode());
        if (document.body.classList.contains(DARK_MODE_CLASS)) {
            document.body.classList.remove(DARK_MODE_CLASS);
        }
        set_cookie(DARK_MODE_COOKIE, COOKIE_LIGHT_MODE);
    };

    window.enable_system_mode = function () {
        // TODO: coalesce the two set_bool-s into one so there is only one DOM
        //       update
        window.ftd.set_bool_for_all(FOLLOW_SYSTEM_DARK_MODE, true);
        window.ftd.set_bool_for_all(SYSTEM_DARK_MODE, system_dark_mode());
        if (system_dark_mode()) {
            window.ftd.set_bool_for_all(DARK_MODE, true);
            document.body.classList.add(DARK_MODE_CLASS);
            set_cookie(DARK_MODE_COOKIE, COOKIE_SYSTEM_DARK);
        } else {
            window.ftd.set_bool_for_all(DARK_MODE, false);
            if (document.body.classList.contains(DARK_MODE_CLASS)) {
                document.body.classList.remove(DARK_MODE_CLASS);
            }
            set_cookie(DARK_MODE_COOKIE, COOKIE_SYSTEM_LIGHT);
        }
    };

    function set_cookie(name: string, value: string) {
        document.cookie = name + "=" + value + "; path=/";
    }

    function system_dark_mode() {
        return !!(window.matchMedia && window.matchMedia('(prefers-color-scheme: dark)').matches);
    }

    function initialise_dark_mode() {
        update_dark_mode();
        start_watching_dark_mode_system_preference();
    }

    function get_cookie(name: string, def: string) {
        // source: https://stackoverflow.com/questions/5639346/
        let regex = document.cookie.match('(^|;)\\s*' + name + '\\s*=\\s*([^;]+)');
        return regex !== null ? regex.pop() : def;
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
                console_log("cookie value is wrong", current_dark_mode_cookie);
                window.enable_system_mode();
        }
    }

    function start_watching_dark_mode_system_preference() {
        window.matchMedia('(prefers-color-scheme: dark)').addEventListener(
            "change", update_dark_mode
        );
    }
    initialise_dark_mode();
    initialise_device();
    // update_markdown_colors();
};
