// Benchmarkable event system with isolated testing capabilities
const fastn_events = {
    handlers: {
        resize: [],
        click: [],
        clickOutside: [],
        keydown: [],
        keyup: [],
        globalKey: [],
        globalKeySeq: [],
    },

    stats: {
        triggeredEvents: {},
        handlerExecutions: 0,
    },

    register(type, handler, metadata = {}) {
        if (typeof fastn_perf !== "undefined")
            fastn_perf.count(`event-register-${type}`);

        if (!this.handlers[type]) {
            this.handlers[type] = [];
        }

        this.handlers[type].push({
            handler,
            metadata,
            registeredAt: Date.now(),
        });
    },

    trigger(type, event, targetHandlers = null) {
        if (typeof fastn_perf !== "undefined")
            fastn_perf.mark(`events-${type}`);
        if (typeof fastn_perf !== "undefined")
            fastn_perf.count(`event-trigger-${type}`);

        const handlers = targetHandlers || this.handlers[type] || [];

        handlers.forEach((handlerObj) => {
            try {
                if (typeof handlerObj === "function") {
                    handlerObj(event);
                } else if (handlerObj.handler) {
                    handlerObj.handler(event);
                }
                this.stats.handlerExecutions++;
            } catch (error) {
                console.error(`Error in ${type} event handler:`, error);
            }
        });

        this.stats.triggeredEvents[type] =
            (this.stats.triggeredEvents[type] || 0) + 1;

        if (typeof fastn_perf !== "undefined")
            fastn_perf.measure(`events-${type}`);
    },

    // For benchmarking
    clear(type) {
        if (type) {
            this.handlers[type] = [];
        } else {
            Object.keys(this.handlers).forEach((key) => {
                this.handlers[key] = [];
            });
        }
        this.stats = { triggeredEvents: {}, handlerExecutions: 0 };
    },

    getHandlerCount(type) {
        return this.handlers[type] ? this.handlers[type].length : 0;
    },

    getStats() {
        return { ...this.stats };
    },
};

// Maintain backward compatibility
ftd.clickOutsideEvents = [];
ftd.globalKeyEvents = [];
ftd.globalKeySeqEvents = [];

ftd.get_device = function () {
    const MOBILE_CLASS = "mobile";
    // not at all sure about this function logic.
    let width = window.innerWidth;
    // In the future, we may want to have more than one break points, and
    // then we may also want the theme builders to decide where the
    // breakpoints should go. we should be able to fetch fpm variables
    // here, or maybe simply pass the width, user agent etc. to fpm and
    // let people put the checks on width user agent etc., but it would
    // be good if we can standardize few breakpoints. or maybe we should
    // do both, some standard breakpoints and pass the raw data.
    // we would then rename this function to detect_device() which will
    // return one of "desktop", "mobile". and also maybe have another
    // function detect_orientation(), "landscape" and "portrait" etc.,
    // and instead of setting `ftd#mobile: boolean` we set `ftd#device`
    // and `ftd#view-port-orientation` etc.
    let mobile_breakpoint = fastn_utils.getStaticValue(
        ftd.breakpoint_width.get("mobile"),
    );
    if (width <= mobile_breakpoint) {
        document.body.classList.add(MOBILE_CLASS);
        return fastn_dom.DeviceData.Mobile;
    }
    if (document.body.classList.contains(MOBILE_CLASS)) {
        document.body.classList.remove(MOBILE_CLASS);
    }
    return fastn_dom.DeviceData.Desktop;
};

ftd.post_init = function () {
    const DARK_MODE_COOKIE = "fastn-dark-mode";
    const COOKIE_SYSTEM_LIGHT = "system-light";
    const COOKIE_SYSTEM_DARK = "system-dark";
    const COOKIE_DARK_MODE = "dark";
    const COOKIE_LIGHT_MODE = "light";
    const DARK_MODE_CLASS = "dark";
    let last_device = ftd.device.get();

    // Benchmarkable resize handler
    const optimizedResizeHandler = function () {
        if (typeof fastn_perf !== "undefined")
            fastn_perf.mark("resize-handler");
        if (typeof fastn_perf !== "undefined")
            fastn_perf.count("resize-events");

        initialise_device();
        fastn_events.trigger("resize", { timestamp: Date.now() });

        if (typeof fastn_perf !== "undefined")
            fastn_perf.measure("resize-handler");
    };

    window.onresize = optimizedResizeHandler;
    function initialise_click_outside_events() {
        document.addEventListener("click", function (event) {
            ftd.clickOutsideEvents.forEach(([ftdNode, func]) => {
                let node = ftdNode.getNode();
                if (
                    !!node &&
                    node.style.display !== "none" &&
                    !node.contains(event.target)
                ) {
                    func();
                }
            });
        });
    }
    function initialise_global_key_events() {
        let globalKeys = {};
        let buffer = [];
        let lastKeyTime = Date.now();

        document.addEventListener("keydown", function (event) {
            let eventKey = fastn_utils.getEventKey(event);
            globalKeys[eventKey] = true;
            const currentTime = Date.now();
            if (currentTime - lastKeyTime > 1000) {
                buffer = [];
            }
            lastKeyTime = currentTime;
            if (
                (event.target.nodeName === "INPUT" ||
                    event.target.nodeName === "TEXTAREA") &&
                eventKey !== "ArrowDown" &&
                eventKey !== "ArrowUp" &&
                eventKey !== "ArrowRight" &&
                eventKey !== "ArrowLeft" &&
                event.target.nodeName === "INPUT" &&
                eventKey !== "Enter"
            ) {
                return;
            }
            buffer.push(eventKey);

            ftd.globalKeyEvents.forEach(([_ftdNode, func, array]) => {
                let globalKeysPresent = array.reduce(
                    (accumulator, currentValue) =>
                        accumulator && !!globalKeys[currentValue],
                    true,
                );
                if (
                    globalKeysPresent &&
                    buffer.join(",").includes(array.join(","))
                ) {
                    func();
                    globalKeys[eventKey] = false;
                    buffer = [];
                }
                return;
            });

            ftd.globalKeySeqEvents.forEach(([_ftdNode, func, array]) => {
                if (buffer.join(",").includes(array.join(","))) {
                    func();
                    globalKeys[eventKey] = false;
                    buffer = [];
                }
                return;
            });
        });

        document.addEventListener("keyup", function (event) {
            globalKeys[fastn_utils.getEventKey(event)] = false;
        });
    }
    function initialise_device() {
        let current = ftd.get_device();
        if (current === last_device) {
            return;
        }
        console.log("last_device", last_device, "current_device", current);
        ftd.device.set(current);
        last_device = current;
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
        ftd.dark_mode.set(true);
        ftd.follow_system_dark_mode.set(false);
        ftd.system_dark_mode.set(system_dark_mode());
        document.body.classList.add(DARK_MODE_CLASS);
        set_cookie(DARK_MODE_COOKIE, COOKIE_DARK_MODE);
    };
    window.enable_light_mode = function () {
        // TODO: coalesce the two set_bool-s into one so there is only one DOM
        //       update
        ftd.dark_mode.set(false);
        ftd.follow_system_dark_mode.set(false);
        ftd.system_dark_mode.set(system_dark_mode());
        if (document.body.classList.contains(DARK_MODE_CLASS)) {
            document.body.classList.remove(DARK_MODE_CLASS);
        }
        set_cookie(DARK_MODE_COOKIE, COOKIE_LIGHT_MODE);
    };
    window.enable_system_mode = function () {
        // TODO: coalesce the two set_bool-s into one so there is only one DOM
        //       update
        let systemMode = system_dark_mode();
        ftd.follow_system_dark_mode.set(true);
        ftd.system_dark_mode.set(systemMode);
        if (systemMode) {
            ftd.dark_mode.set(true);
            document.body.classList.add(DARK_MODE_CLASS);
            set_cookie(DARK_MODE_COOKIE, COOKIE_SYSTEM_DARK);
        } else {
            ftd.dark_mode.set(false);
            if (document.body.classList.contains(DARK_MODE_CLASS)) {
                document.body.classList.remove(DARK_MODE_CLASS);
            }
            set_cookie(DARK_MODE_COOKIE, COOKIE_SYSTEM_LIGHT);
        }
    };
    function set_cookie(name, value) {
        document.cookie = name + "=" + value + "; path=/";
    }
    function system_dark_mode() {
        return !!(
            window.matchMedia &&
            window.matchMedia("(prefers-color-scheme: dark)").matches
        );
    }
    function initialise_dark_mode() {
        update_dark_mode();
        start_watching_dark_mode_system_preference();
    }
    function get_cookie(name, def) {
        // source: https://stackoverflow.com/questions/5639346/
        let regex = document.cookie.match(
            "(^|;)\\s*" + name + "\\s*=\\s*([^;]+)",
        );
        return regex !== null ? regex.pop() : def;
    }
    function update_dark_mode() {
        let current_dark_mode_cookie = get_cookie(
            DARK_MODE_COOKIE,
            COOKIE_SYSTEM_LIGHT,
        );
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
        window
            .matchMedia("(prefers-color-scheme: dark)")
            .addEventListener("change", update_dark_mode);
    }
    initialise_device();
    initialise_dark_mode();
    initialise_click_outside_events();
    initialise_global_key_events();
    fastn_utils.resetFullHeight();
    fastn_utils.setFullHeight();
};
