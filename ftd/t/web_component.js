// Create a class for the element
class WordCount extends HTMLElement {
    constructor() {
        // Always call super first in constructor
        super();

        let data = window.ftd.component_data(this);

        // Create a shadow root
        const shadow = this.attachShadow({mode: 'open'});

        // Create spans
        const wrapper = document.createElement('span');
        wrapper.setAttribute('class', 'wrapper');

        const icon = document.createElement('span');
        icon.setAttribute('class', 'icon');
        icon.setAttribute('tabindex', 0);

        const info = document.createElement('span');
        info.setAttribute('class', 'info');

        // Take attribute content and put it inside the info span
        const text = data.body;
        info.textContent = text.get();

        const count = document.createElement('span');
        info.setAttribute('class', 'info');

        const seperator = data.separator;
        let value = text.get().split(seperator.get()).length;
        count.textContent = value;

        const text_count = data.count;
        text_count.set(value);

        text_count.on_change(function () {
            const text = text_count.get();
            count.textContent = text;
            console.log("Changed");
        });


        // Insert icon
        let imgUrl;
        if(this.hasAttribute('img')) {
            imgUrl = this.getAttribute('img');
            const img = document.createElement('img');
            img.src = imgUrl;
            icon.appendChild(img);
        } else {
            const img = document.createElement('div');
            img.innerText = "ℹ️";
            icon.appendChild(img);
        }



        // Create some CSS to apply to the shadow dom
        const style = document.createElement('style');
        console.log(style.isConnected);

        style.textContent = `
      .wrapper {
        position: relative;
      }

      .info {
        font-size: 0.8rem;
        width: 200px;
        display: inline-block;
        border: 1px solid black;
        padding: 10px;
        background: white;
        border-radius: 10px;
        opacity: 0;
        transition: 0.6s all;
        position: absolute;
        bottom: 20px;
        left: 10px;
        z-index: 3;
      }

      img {
        width: 1.2rem;
      }

      .icon:hover + .info, .icon:focus + .info {
        opacity: 1;
      }
    `;

        // Attach the created elements to the shadow dom
        shadow.appendChild(style);
        console.log(style.isConnected);
        shadow.appendChild(wrapper);
        wrapper.appendChild(icon);
        wrapper.appendChild(info);
        wrapper.appendChild(count);
    }
}

// Define the new element
customElements.define('word-count', WordCount);
