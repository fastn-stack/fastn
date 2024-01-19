function qrCodeGenerate(id, url) {
    id = fastn_utils.getStaticValue(id);
    url = fastn_utils.getStaticValue(url);
    new QRCode(document.getElementById(id), url);
}



function qrCodeGenerateWithConfig(id, url, width, height, color, bgColor, darkmode, level) {
    id = fastn_utils.getStaticValue(id);
    url = fastn_utils.getStaticValue(url);
    width = fastn_utils.getStaticValue(width);
    height = fastn_utils.getStaticValue(height);
    color = fastn_utils.getStaticValue(color);
    bgColor = fastn_utils.getStaticValue(bgColor);
    darkmode = fastn_utils.getStaticValue(darkmode);
    level = fastn_utils.getStaticValue(level);

    if (darkmode) {
        if (color instanceof fastn.recordInstanceClass) {
            color = fastn_utils.getStaticValue(color.get("dark"));
        }
        if (bgColor instanceof fastn.recordInstanceClass) {
            bgColor = fastn_utils.getStaticValue(bgColor.get("dark"));
        }
    } else {
        if (color instanceof fastn.recordInstanceClass) {
            color = fastn_utils.getStaticValue(color.get("light"));
        }
        if (bgColor instanceof fastn.recordInstanceClass) {
            bgColor = fastn_utils.getStaticValue(bgColor.get("light"));
        }
    }

    const container = document.getElementById(id); // Replace 'containerId' with the ID of your container element
    while (container.firstChild) {
        container.firstChild.remove();
    }

    new QRCode(container, {
        text: url,
        width: width ?? 256,
        height: height ?? 256 ,
        colorDark : color ?? "#000",
        colorLight : bgColor ?? "#ffffff",
        correctLevel : !fastn_utils.isNull(level) ? (level > 3 ? 3: (level < 0 ? 0 : level)) : QRCode.CorrectLevel.H
    });
}
