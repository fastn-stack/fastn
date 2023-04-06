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
