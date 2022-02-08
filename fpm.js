(function () {
    const FPM_MOBILE = "fpm#mobile";
    const FPM_MOBILE_BREAKPOINT = "fpm#mobile-breakpoint";
    const FPM_THEME_COLOR = "fpm#theme-color";
    const FPM_DARK_MODE = "fpm#dark-mode"
    const SYSTEM_DARK_MODE = "fpm#system-dark-mode"
    const FPM_FOLLOW_SYSTEM_DARK_MODE = "fpm#follow-system-dark-mode"
    const DARK_MODE_COOKIE = "fpm-dark-mode";
    const COOKIE_SYSTEM_LIGHT = "system-light";
    const COOKIE_SYSTEM_DARK = "system-dark";
    const COOKIE_DARK_MODE = "dark";
    const COOKIE_LIGHT_MODE = "light";
    const DARK_MODE_CLASS = "fpm-dark";
    const THEME_COLOR_META = "theme-color";

    let last_device;

    function initialise_device() {
        last_device = is_mobile();
        console.log("is_mobile", last_device);
        window.ftd.set_bool_for_all(FPM_MOBILE, last_device);
    }

    window.onresize = function () {
        let current = is_mobile();
        if (current === last_device) {
            return;
        }

        window.ftd.set_bool_for_all(FPM_MOBILE, current);
        last_device = current;
        console.log("is_mobile", last_device);
    }

    function is_mobile() {
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
        // and instead of setting `fpm-ui#mobile: boolean` we set `fpm-ui#device`
        // and `fpm-ui#view-port-orientation` etc.
        let breakpoint = parseInt(window.ftd.get_value("main", FPM_MOBILE_BREAKPOINT));
        return width <= breakpoint;
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


    window.enable_dark_mode = function () {
        // TODO: coalesce the two set_bool-s into one so there is only one DOM
        //       update
        window.ftd.set_bool_for_all(FPM_DARK_MODE, true);
        window.ftd.set_bool_for_all(FPM_FOLLOW_SYSTEM_DARK_MODE, false);
        window.ftd.set_bool_for_all(SYSTEM_DARK_MODE, system_dark_mode());
        document.body.classList.add(DARK_MODE_CLASS);
        set_cookie(DARK_MODE_COOKIE, COOKIE_DARK_MODE);
        update_theme_color();
    }

    window.enable_light_mode = function () {
        // TODO: coalesce the two set_bool-s into one so there is only one DOM
        //       update
        window.ftd.set_bool_for_all(FPM_DARK_MODE, false);
        window.ftd.set_bool_for_all(FPM_FOLLOW_SYSTEM_DARK_MODE, false);
        window.ftd.set_bool_for_all(SYSTEM_DARK_MODE, system_dark_mode());
        document.body.classList.remove(DARK_MODE_CLASS);
        set_cookie(DARK_MODE_COOKIE, COOKIE_LIGHT_MODE);
        update_theme_color();
    }

    window.enable_system_mode = function () {
        // TODO: coalesce the two set_bool-s into one so there is only one DOM
        //       update
        window.ftd.set_bool_for_all(FPM_FOLLOW_SYSTEM_DARK_MODE, true);
        window.ftd.set_bool_for_all(SYSTEM_DARK_MODE, system_dark_mode());
        if (system_dark_mode()) {
            window.ftd.set_bool_for_all(FPM_DARK_MODE, true);
            document.body.classList.add(DARK_MODE_CLASS);
            set_cookie(DARK_MODE_COOKIE, COOKIE_SYSTEM_DARK)
        } else {
            window.ftd.set_bool_for_all(FPM_DARK_MODE, false);
            document.body.classList.remove(DARK_MODE_CLASS);
            set_cookie(DARK_MODE_COOKIE, COOKIE_SYSTEM_LIGHT)
        }
        update_theme_color();
    }

    function update_theme_color() {
        let theme_color = window.ftd.get_value("main", FPM_THEME_COLOR);
        if (!!theme_color) {
            set_meta(THEME_COLOR_META, theme_color);
        } else {
            delete_meta(THEME_COLOR_META);
        }
    }

    function set_meta(name, value) {
        let meta = document.querySelector("meta[name=" + name + "]");
        if (!!meta) {
            meta.content = value;
        } else {
            meta = document.createElement('meta');
            meta.name = name;
            meta.content = value;
            document.getElementsByTagName('head')[0].appendChild(meta);
        }
    }

    function delete_meta(name) {
        let meta = document.querySelector("meta[name=" + name + "]")
        if (!!meta) {
            meta.remove();
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
        let regex = document.cookie.match('(^|;)\\s*' + name + '\\s*=\\s*([^;]+)');
        return regex !== null ? regex.pop() : def;
    }

    function set_cookie(name, value) {
        document.cookie = name + "=" + value + "; path=/";
    }

    function system_dark_mode() {
        return !!(window.matchMedia && window.matchMedia('(prefers-color-scheme: dark)').matches)
    }

    function start_watching_dark_mode_system_preference() {
        window.matchMedia && window.matchMedia('(prefers-color-scheme: dark)').addEventListener(
            "change", update_dark_mode
        );
    }

    initialise_device();
    initialise_dark_mode();
})();

/*! instant.page v5.1.0 - (C) 2019-2020 Alexandre Dieulot - https://instant.page/license */
let t,e;const n=new Set,o=document.createElement("link"),i=o.relList&&o.relList.supports&&o.relList.supports("prefetch")&&window.IntersectionObserver&&"isIntersecting"in IntersectionObserverEntry.prototype,s="instantAllowQueryString"in document.body.dataset,a="instantAllowExternalLinks"in document.body.dataset,r="instantWhitelist"in document.body.dataset,c="instantMousedownShortcut"in document.body.dataset,d=1111;let l=65,u=!1,f=!1,m=!1;if("instantIntensity"in document.body.dataset){const t=document.body.dataset.instantIntensity;if("mousedown"==t.substr(0,"mousedown".length))u=!0,"mousedown-only"==t&&(f=!0);else if("viewport"==t.substr(0,"viewport".length))navigator.connection&&(navigator.connection.saveData||navigator.connection.effectiveType&&navigator.connection.effectiveType.includes("2g"))||("viewport"==t?document.documentElement.clientWidth*document.documentElement.clientHeight<45e4&&(m=!0):"viewport-all"==t&&(m=!0));else{const e=parseInt(t);isNaN(e)||(l=e)}}if(i){const n={capture:!0,passive:!0};if(f||document.addEventListener("touchstart",function(t){e=performance.now();const n=t.target.closest("a");if(!h(n))return;v(n.href)},n),u?c||document.addEventListener("mousedown",function(t){const e=t.target.closest("a");if(!h(e))return;v(e.href)},n):document.addEventListener("mouseover",function(n){if(performance.now()-e<d)return;const o=n.target.closest("a");if(!h(o))return;o.addEventListener("mouseout",p,{passive:!0}),t=setTimeout(()=>{v(o.href),t=void 0},l)},n),c&&document.addEventListener("mousedown",function(t){if(performance.now()-e<d)return;const n=t.target.closest("a");if(t.which>1||t.metaKey||t.ctrlKey)return;if(!n)return;n.addEventListener("click",function(t){1337!=t.detail&&t.preventDefault()},{capture:!0,passive:!1,once:!0});const o=new MouseEvent("click",{view:window,bubbles:!0,cancelable:!1,detail:1337});n.dispatchEvent(o)},n),m){let t;(t=window.requestIdleCallback?t=>{requestIdleCallback(t,{timeout:1500})}:t=>{t()})(()=>{const t=new IntersectionObserver(e=>{e.forEach(e=>{if(e.isIntersecting){const n=e.target;t.unobserve(n),v(n.href)}})});document.querySelectorAll("a").forEach(e=>{h(e)&&t.observe(e)})})}}function p(e){e.relatedTarget&&e.target.closest("a")==e.relatedTarget.closest("a")||t&&(clearTimeout(t),t=void 0)}function h(t){if(t&&t.href&&(!r||"instant"in t.dataset)&&(a||t.origin==location.origin||"instant"in t.dataset)&&["http:","https:"].includes(t.protocol)&&("http:"!=t.protocol||"https:"!=location.protocol)&&(s||!t.search||"instant"in t.dataset)&&!(t.hash&&t.pathname+t.search==location.pathname+location.search||"noInstant"in t.dataset))return!0}function v(t){if(n.has(t))return;const e=document.createElement("link");e.rel="prefetch",e.href=t,document.head.appendChild(e),n.add(t)}
