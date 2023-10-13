const createGlobalRef = () => {
    let ref = null;

    return {
        get: () => ref,
        set: global => ref = global,
    }
};

const __fastn_legacy_global_ref__ = createGlobalRef();

(function(fastn, ftd, global) {
    const GLOBAL_VARIABLE_MAP = "global";

    const LOCAL_VARIABLE_MAP = "__args__";

    function getDocNameAndRemaining(s) {
        let part1 = "";
        let patternToSplitAt = s;
        
        const split1 = s.split('#');
        if (split1.length === 2) {
            part1 = split1[0] + '#';
            patternToSplitAt = split1[1];
        }
    
        const split2 = patternToSplitAt.split('.');
        if (split2.length === 2) {
            return [part1 + split2[0], split2[1]];
        } else {
            return [s, null];
        }
    }
    
    function getPrefix(s) {
        let prefix = null;
    
        s = s.toString();
    
        if (s.startsWith(GLOBAL_VARIABLE_MAP + '.')) {
            prefix = GLOBAL_VARIABLE_MAP;
            s = s.substring(GLOBAL_VARIABLE_MAP.length + 1);
        } else if (s.startsWith(LOCAL_VARIABLE_MAP + '.')) {
            prefix = LOCAL_VARIABLE_MAP;
            s = s.substring(LOCAL_VARIABLE_MAP.length + 1);
        } else if (s.startsWith('ftd.') || s.startsWith('ftd#')) {
            prefix = 'ftd';
            s = s.substring(4);
        } else if (s.startsWith('fastn_utils.')) {
            prefix = 'fastn_utils';
            s = s.substring(12);
        }
    
        return [prefix, s];
    }
    
    function nameToJs(s) {
        const [prefix, name] = getPrefix(s);
        return `${prefix ? prefix + '.' : ''}${nameToJs_(name)}`;
    }
    
    function nameToJs_(s) {
        let name = s.toString();
    
        if (name[0].charCodeAt(0) >= 48 && name[0].charCodeAt(0) <= 57) {
            name = '_' + name;
        }
    
        return name.replace(/#/g, '__')
            .replace(/-/g, '_')
            .replace(/:/g, '___')
            .replace(/,/g, '$')
            .replace(/\\\\/g, '/')
            .replace(/\\/g, '/')
            .replace(/[\/.]/g, '_');
    }
    
    function kebabToSnakeCase(s) {
        return s.replace(/-/g, '_');
    }
    
    function variableToJs(variableName) {
        const [docName, remaining] = getDocNameAndRemaining(variableName);
        let name = nameToJs(docName);
    
        if (remaining) {
            name = `${name}.${kebabToSnakeCase(remaining)}`;
        }
    
        return [name, remaining];
    }
    
    ftd.set_value = function(variable, value) {
        const data = global.get();
        const [var_name, remaining] = variableToJs(variable);
        console.log(var_name, remaining)
        if(data[var_name] === undefined) {
            console.log(`[ftd-legacy]: ${variable} is not in global map, ignoring`);
            return;
        }
        const mutable = data[var_name];
        if(!(mutable instanceof fastn.mutableClass || mutable instanceof fastn.mutableListClass || mutable instanceof fastn.recordInstanceClass)) {
            console.log(`[ftd-legacy]: ${variable} is not a mutable, ignoring`);
            return;
        }
        if(remaining) {
            mutable.get(remaining).set(value);
        } else {
            mutable.set(value);
        }
    }    
})(fastn, ftd, __fastn_legacy_global_ref__);
