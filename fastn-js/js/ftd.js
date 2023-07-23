let ftd = {
    // source: https://stackoverflow.com/questions/400212/ (cc-by-sa)
    riveNodes: {},
    copy_to_clipboard(args) {
        let text = args.a;
        if (text.startsWith("\\", 0)) {
            text = text.substring(1);
        }
        if (!navigator.clipboard) {
            fallbackCopyTextToClipboard(text);
            return;
        }
        navigator.clipboard.writeText(text).then(function() {
            console.log('Async: Copying to clipboard was successful!');
        }, function(err) {
            console.error('Async: Could not copy text: ', err);
        });
    },

    set_rive_boolean(args, node) {
        if (!!args.rive) {
            let riveNode = ftd.riveNodes[`${args.rive}__${ftd.device.get()}`];
            node = riveNode ? riveNode: node;
        }
        let riveConst = node.getExtraData().rive;
        const stateMachineName = riveConst.stateMachineNames[0];
        const inputs = riveConst.stateMachineInputs(stateMachineName);
        const bumpTrigger = inputs.find(i => i.name === args.input);
        bumpTrigger.value = args.value;
    },

    toggle_rive_boolean(args, node) {
        if (!!args.rive) {
            let riveNode = ftd.riveNodes[`${args.rive}__${ftd.device.get()}`];
            node = riveNode ? riveNode: node;
        }
        let riveConst = node.getExtraData().rive;
        const stateMachineName = riveConst.stateMachineNames[0];
        const inputs = riveConst.stateMachineInputs(stateMachineName);
        const trigger = inputs.find(i => i.name === args.input);
        trigger.value = !trigger.value;
    }

};

// ftd.append($a = $people, v = Tom)
ftd.append = function (a, v) { a.push(v) }
