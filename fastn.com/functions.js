function show_alert(a) {
    alert(a);
}

function add_sub(checked, a, answer) {
    if (a) {
        return checked +answer;
    } else {
        return checked -answer;
    }
}

function submit_correct_answer(correct, number_correct, answers) {
    if (number_correct == answers) {
        return correct + 1;
    } else {
        return correct;
    }
}


function submit_wrong_answer(wrong, number_correct, answers) {
    if (number_correct == answers) {
        return wrong;
    } else {
        return wrong + 1;
    }
}


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
