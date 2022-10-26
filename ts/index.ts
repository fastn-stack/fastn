
window.ftd = (function() {
    let ftd_data: any = {};
    let exports: Partial<Export> = {};

    exports.init = function (id: string, data: string) {
        let element = document.getElementById(data);
        if (!!element) {
            ftd_data[id] = JSON.parse(element.innerText);
            // window.ftd.post_init();
        }
    };

    function handle_event(evt: Event, id: string, action: Action, obj: Element) {
        console.log(id, action);
        console.log(action.name);
        console.log("ftd_data",ftd_data[id]);
        let argument: keyof typeof action.values;
        for (argument in action.values) {
            if (action.values.hasOwnProperty(argument)) {
                if(typeof(action.values[argument]) === 'object') {

                }
                console.log(argument, action.values[argument], typeof(action.values[argument]), typeof(action.values[argument]) === 'object');
            }

        }
    }

    exports.handle_event = function (evt: Event, id: string, event: string, obj: Element) {
        console_log(id, event);
        let actions = JSON.parse(event);
        for (const action in actions) {
            handle_event(evt, id, actions[action], obj);
        }
    };

    return exports;
})();
