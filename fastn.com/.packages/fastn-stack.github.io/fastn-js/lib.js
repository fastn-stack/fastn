function clampDecrement(a, by, min, max) {
    let newValue = (a.get() - by.get()) ;
    if (newValue < min.get()) {
        newValue = max.get() - 1;
    } else if (newValue >= min.get()) {
        newValue = min.get();
    }
    a.set(newValue);
}

function getRange(min, max) {
    const result = [];
    for (let i = min.get(); i < max.get(); i++) {
        result.push(i);
    }
    return fastn.mutableList(result);
}
