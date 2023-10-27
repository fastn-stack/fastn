(function() {
    document.addEventListener("DOMContentLoaded", function() {
        console.log("onload 1");

        let iframe = document.createElement('iframe');
        // iframe.src = "/-/tutor/";
        iframe.src = "https://fastn.com/";
        iframe.style.position = "fixed";
        iframe.style.top = "0";
        iframe.style.left = "0";
        iframe.style.width = "100px";
        iframe.style.height = "100vh";

        document.body.style.paddingLeft = "100px";
        document.body.insertBefore(iframe, document.body.firstChild);
        console.log("onload");
    });
    console.log("registered");
})();