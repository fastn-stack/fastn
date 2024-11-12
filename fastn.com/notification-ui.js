class Notification extends HTMLElement {
    constructor() {
        super();
        let data = window.ftd.component_data(this);
        let marginLeft = data.margin_left.get();
        let marginTop = data.margin_top.get();
        let remove = data.remove.get();
        const shadow = this.attachShadow({ mode: 'open' });
        const div = document.createElement('div');
        div.style.backgroundColor = 'red';
        div.style.padding = '10px';
        div.style.borderRadius = '50%';
        div.style.padding = '5px';
        div.style.marginLeft = marginLeft;
        div.style.marginTop = marginTop;
        let [checkNotification, lastPublished] = showNotificationAndGetLastPublish(data.notifications);
        if (checkNotification) {
            shadow.appendChild(div);
        }
        data.remove.on_change(() => {
            remove = data.remove.get();
            if (remove) {
                div.remove();
                setLastPublished(lastPublished)
            }
        })
    }
}

customElements.define('notification-ui', Notification);


function showNotificationAndGetLastPublish(notification) {
    let notificationList = notification.get()
    if (notificationList.length > 0) {
        let lastPublished = notificationList[0].item.getAllFields().published_on.get();
        let lastNotificationReadTime = localStorage.getItem("fastn_com_last_notification_read_time");
        if (!lastNotificationReadTime || (!!lastNotificationReadTime && new Date(lastNotificationReadTime) < new Date(lastPublished))) {
            return [true, lastPublished];
        }
        return [false, lastPublished]
    }
    return [false, undefined];
}


function setLastPublished(lastPublished) {
    if (!!lastPublished) {
        localStorage.setItem("fastn_com_last_notification_read_time", lastPublished);
    }
}
