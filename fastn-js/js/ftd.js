let ftd = {
    // source: https://stackoverflow.com/questions/400212/ (cc-by-sa)
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
        let rive_const = node.getExtraData().rive;
        const stateMachineName = rive_const.stateMachineNames[0];
        const inputs = rive_const.stateMachineInputs(stateMachineName);
        const bumpTrigger = inputs.find(i => i.name === args.input);
        bumpTrigger.value = args.value;
    }

};

// ftd.append($a = $people, v = Tom)
ftd.append = function (a, v) { a.push(v) }
