function findnow(search, sitemap, appendIn) {
    console.log(search, sitemap, appendIn);
    let sectionList = fastn_utils.getStaticValue(sitemap.get("sections"));
    let searchValue = fastn_utils.getStaticValue(search).toLowerCase();
    appendIn.clearAll();
    if (searchValue.length === 0) {
        return;
    }
    findInSections(sectionList, searchValue, appendIn);
}

function findInSections(sectionList, search, appendIn) {
    for(let item of sectionList) {
        let tocItem = item.item;
        let title = fastn_utils.getStaticValue(tocItem.get("title"));
        let description = fastn_utils.getStaticValue(tocItem.get("description"));
        let url = fastn_utils.getStaticValue(tocItem.get("url"));
        if (fastn_utils.isNull(url)) {
            continue;
        }
        let alreadyInList =  appendIn.getList().some(
            existingItem =>
                existingItem.item.get() === url
        );
        if ((!fastn_utils.isNull(title) && title.toLowerCase().includes(search)) ||
            (!fastn_utils.isNull(description) && description.toLowerCase().includes(search))
            && !alreadyInList) {
            appendIn.push(
                fastn.recordInstance({
                    title: title,
                    description: description,
                    url: url
                }));
        }
        let children = fastn_utils.getStaticValue(tocItem.get("children"));
        findInSections(children, search, appendIn)
    }
}
