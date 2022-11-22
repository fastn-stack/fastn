
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

    function handle_function(evt: Event, id: string, action: Action, obj: Element, function_arguments: (FunctionArgument | any)[]) {
        console.log(id, action);
        console.log(action.name);

        let argument: keyof typeof action.values;
        for (argument in action.values) {
            if (action.values.hasOwnProperty(argument)) {
                if (typeof action.values[argument] === 'object') {
                    let function_argument = <FunctionArgument>action.values[argument];
                    if (!!function_argument.reference) {
                        let value = resolve_reference(function_argument.reference, ftd_data[id]);
                        if (!!function_argument.mutable) {
                            function_argument.value = value;
                            function_arguments.push(function_argument);
                        } else {
                            function_arguments.push(deepCopy(value));
                        }
                    }
                } else {
                    function_arguments.push(action.values[argument]);
                }
            }
        }

        return window[action.name](...function_arguments);
    }


    function handle_event(evt: Event, id: string, action: Action, obj: Element) {
        let function_arguments: (FunctionArgument | any)[] = [];
        handle_function(evt, id, action, obj, function_arguments);
        change_value(function_arguments, ftd_data[id], id);
    }

    exports.handle_event = function (evt: Event, id: string, event: string, obj: Element) {
        console_log(id, event);
        let actions = JSON.parse(event);
        for (const action in actions) {
            handle_event(evt, id, actions[action], obj);
        }
    };

    exports.handle_function = function (evt: Event, id: string, event: string, obj: Element) {
        console_log(id, event);
        let actions = JSON.parse(event);
        let function_arguments: (FunctionArgument | any)[] = [];
        return handle_function(evt, id, actions, obj, function_arguments);
    };

    return exports;
})();
