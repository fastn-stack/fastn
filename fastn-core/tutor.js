(function() {
    document.addEventListener("DOMContentLoaded", function() {
        let sidebarWidth = "300px";

        let iframe = document.createElement('iframe');
        // iframe.src = "/-/tutor/";
        iframe.src = "https://fastn.com/";
        iframe.style.position = "fixed";
        iframe.style.top = "0";
        iframe.style.left = "0";
        iframe.style.width = sidebarWidth;
        iframe.style.height = "100vh";

        document.body.style.paddingLeft = sidebarWidth;
        document.body.insertBefore(iframe, document.body.firstChild);
    });

    window.onmessage = function(e) {
        if (e.kind === 'navigate') {
            document.location.href = e.url;
            return;
        }

        console.warn('Unknown message', e);
    };
})();