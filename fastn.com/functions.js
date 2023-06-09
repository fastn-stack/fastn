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
