// Define the web component using the standard Web Components API
class HelloWorld extends HTMLElement {
    constructor() {
        super();
        const shadow = this.attachShadow({ mode: 'open' });
        const div = document.createElement('div');
        div.classList.add('hello-world');
        div.textContent = 'Hello World!';
        div.style.color = 'orange';
        shadow.appendChild(div);
    }
}

// Register the web component
customElements.define('hello-world', HelloWorld);









// Define the web component using the standard Web Components API
class NumToWords extends HTMLElement {
    constructor() {
        super();
        let data = window.ftd.component_data(this);
        let num = data.num.get();

        const shadow = this.attachShadow({ mode: 'open' });
        const div = document.createElement('div');
        div.textContent = numberToWords(num);
        div.style.color = 'orange';

        data.num.on_change(function () {
            const changed_value = data.num.get();
            div.textContent = numberToWords(changed_value);
        })

        shadow.appendChild(div);
    }
}

// Register the web component
customElements.define('num-to-words', NumToWords);









function numberToWords(num) {
    const ones = ['', 'one', 'two', 'three', 'four', 'five', 'six', 'seven', 'eight', 'nine'];
    const tens = ['', '', 'twenty', 'thirty', 'forty', 'fifty', 'sixty', 'seventy', 'eighty', 'ninety'];
    const teens = ['ten', 'eleven', 'twelve', 'thirteen', 'fourteen', 'fifteen', 'sixteen', 'seventeen', 'eighteen', 'nineteen'];

    if (num == 0) {
        return 'zero';
    }

    if (num < 0) {
        return 'minus ' + numberToWords(Math.abs(num));
    }

    let words = '';

    if (Math.floor(num / 1000) > 0) {
        words += numberToWords(Math.floor(num / 1000)) + ' thousand ';
        num %= 1000;
    }

    if (Math.floor(num / 100) > 0) {
        words += numberToWords(Math.floor(num / 100)) + ' hundred ';
        num %= 100;
    }

    if (num >= 10 && num <= 19) {
        words += teens[num - 10] + ' ';
        num = 0;
    } else if (num >= 20 || num === 0) {
        words += tens[Math.floor(num / 10)] + ' ';
        num %= 10;
    }

    if (num > 0) {
        words += ones[num] + ' ';
    }

    return words.trim();
}
