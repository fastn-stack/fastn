let ftd = {};

ftd.device = fastn.mutable("desktop");
ftd["breakpoint-width"] = fastn.recordInstance({mobile: 768})
// ftd.append($a = $people, v = Tom)

ftd.append = function (a, v) { a.push(v)}
