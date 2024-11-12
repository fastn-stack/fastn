function findNow(search, sitemap, appendIn, limit) {
    let sectionList = fastn_utils.getStaticValue(sitemap.get("sections"));
    let searchValue = fastn_utils.getStaticValue(search).toLowerCase();
    appendIn.clearAll();
    if (searchValue.length === 0) {
        return;
    }
    findInSections(sectionList, searchValue, appendIn, limit);
}

function findInSections(sectionList, search, appendIn, limit) {
    if (appendIn.getList().length >= limit) {
        return;
    }

    for(let item of sectionList) {
        let tocItem = item.item;
        let title = fastn_utils.getStaticValue(tocItem.get("title"));
        let description = fastn_utils.getStaticValue(tocItem.get("description"));
        let url = fastn_utils.getStaticValue(tocItem.get("url"));
        if (fastn_utils.isNull(url) || url == "") {
            let children = fastn_utils.getStaticValue(tocItem.get("children"));
            findInSections(children, search, appendIn, limit);
            continue;
        }
        let alreadyInList =  appendIn.getList().some(
            existingItem =>
                fastn_utils.getStaticValue(existingItem.item.get("url")) === url
        );
        if (
            (!fastn_utils.isNull(title) && title.toLowerCase().includes(search))
            || (!fastn_utils.isNull(description) && description.toLowerCase().includes(search))
            || url.toLowerCase().includes(search)
            && !alreadyInList
        ) {
            if (appendIn.getList().length >= limit) {
                return;
            }
            appendIn.push(
                fastn.recordInstance({
                    title: title,
                    description: description,
                    url: url
                }));
        }
        let children = fastn_utils.getStaticValue(tocItem.get("children"));
        findInSections(children, search, appendIn, limit);
    }
}


function goBack() {
    const currentURL = new URL(window.location.href);
    let nextPage = currentURL.searchParams.get("next");
    if (nextPage !== null) {
        window.location.href = nextPage;
    } else {
        window.location.href = "/";
    }
}



function openSearch() {
    const currentURL = document.location.pathname + document.location.search;
    window.location.href = `/search/?next=${encodeURIComponent(currentURL)}`
}


function goToUrl(a, l) {
    let index = fastn_utils.getStaticValue(a);
    let list = fastn_utils.getStaticValue(l);
    if (list.length === 0 || index >= list.length) {
        return;
    }
    window.location.href = fastn_utils.getStaticValue(list[index].item.get("url"));
}


function clampDecrement(a,n) {
    let newValue = (a.get() - 1) ;
    if (newValue < 0) {
        newValue = n.get() - 1;
    }
    a.set(newValue);
}
