function calculateDate(date) {
    date = fastn_utils.getStaticValue(date);
    const currentDate = new Date();
    const givenDate = new Date(date);
    // Calculate the time difference in milliseconds
    const timeDifference = currentDate.getTime() - givenDate.getTime();

    // Check if the difference is within the same day (less than 24 hours)
    if (timeDifference < 0 && timeDifference * -1 < 86400000) {
        const hoursDifference = Math.floor((timeDifference * -1) / (1000 * 60 * 60));
        return `Coming in ${hoursDifference} hours`;
    } else if (timeDifference < 0) {
        const daysDifference = Math.floor((timeDifference * -1) / (1000 * 60 * 60 * 24));
        return `Coming in ${daysDifference} days`;
    } else if (timeDifference < 86400000) { // 86400000 milliseconds = 24 hours
        // Calculate the number of hours
        const hoursDifference = Math.floor(timeDifference / (1000 * 60 * 60));
        return `${hoursDifference} hours ago`;
    } else {
        // Calculate the number of days
        const daysDifference = Math.floor(timeDifference / (1000 * 60 * 60 * 24));
        return `${daysDifference} days ago`;
    }
}


function checkNotification(notification) {
    let notificationLength = notification.getLength();
    let numberOfNotificationsRead = parseInt(localStorage.getItem("fastn_com_number_of_notifications_read") ?? 0);
    if (numberOfNotificationsRead < notificationLength) {
        return notificationLength - numberOfNotificationsRead;
    }
    return 0;
}


function setNotification(notification) {
    // let notificationLength = notification.getLength();
    // let numberOfNotificationsRead = ftd.local_storage.get("number_of_notifications_read") ?? 0;
    // if (numberOfNotificationsRead < notificationLength) {
    //     return notificationLength - numberOfNotificationsRead;
    // }
    // return null;
    let notificationLength = notification.getLength();
    localStorage.setItem("fastn_com_number_of_notifications_read", notificationLength);
    window.document.getElementById("notification-number")?.remove();
}

// window.onload = () => {
//     let g = window.document.getElementById("notification-number");
//     console.log(g.innerText);
// }


document.addEventListener('DOMContentLoaded', function() {
    removeNotificationNumber();
});

function removeNotificationNumber() {
    let element = window.document.getElementById("notification-number");
    if (element !== null && element.innerText === "0") {
        element.remove()
    }
}
