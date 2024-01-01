class Todo extends HTMLElement {
    constructor() {
        super(); // Always call super first in constructor

        // get access to arguments passed to this component
        let data = window.ftd.component_data(this);

        let todo_list = [];

        data.name.on_change(function () {
            const text_name = data.name.get();
            if (text_name === null) {
                return;
            }
            let obj = {"name": text_name, "done": true, "status": "Todo", "description": null};
            todo_list.push(obj);
            let index = todo_list.length -1;

            let todo = todo_item_display(obj, index, data, todo_list);
            data.todo_list.set(todo_list);

            wrapper.appendChild(todo);
        });

        // Create a shadow root
        const shadow = this.attachShadow({mode: 'open'});

        const page = document.createElement('div');
        page.setAttribute('class', 'page');

        const heading = document.createElement('div');
        heading.setAttribute('class', 'heading');
        heading.innerText = "JS World"

        // Create spans
        const wrapper = document.createElement('div');
        wrapper.setAttribute('class', 'wrapper');

        todo_list.map((value, index) => {
            let todo = todo_item_display(value, index, data, todo_list);
            wrapper.appendChild(todo);
        });

        const button = document.createElement('button');
        button.setAttribute('class', 'button');
        button.innerText = "Sync"
        button.onclick = function (e) {
            data.todo_list.set(todo_list);
        }



        // Create some CSS to apply to the shadow dom
        const style = document.createElement('style');
        console.log(style.isConnected);

        style.textContent = `
        .parent-wrap {
            width: 100%;
        }
        .page {
            background-color: #dae6f0;
            display: flex;
            flex-direction: column;
            gap: 10px;
            padding: 20px;
        }
        .heading {
            font-size: 32px;
            font-weight: 600;
            font-family: fifthtry-github-io-inter-font-Inter;
        }
        
      .wrapper {
            gap: 10px;
            display: flex;
            flex-direction: column;
            min-height: 200px;
            background-color: white;
            padding: 20px;
      }
      
      .todo-item {
        flex-direction: row;
        display: flex;
        gap: 10px;
        font-size: 30px;
        width: 100%;
        background-color: cyan;
      }
      
      .button {
        width: fit-content;
        font-size: 20px;
        padding: 5px 8px;
        align-self: end;
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
      .todo-done {
        background-color: #aff5af;
        color: darkgreen;
        display: flex;
        font-family: fifthtry-github-io-inter-font-Inter;
        font-size: 18px;
        font-weight: 700;
        gap: 18px;
      }
      .todo-added {
        background-color: yellow;
        color: #bb7d0c;
        display: flex;
        font-family: fifthtry-github-io-inter-font-Inter;
        font-size: 18px;
        font-weight: 700;
        gap: 18px;
      }
      @media (max-width: 500px) {
            .page {
                background-color: #dae6f0;
                display: flex;
                flex-direction: column;
                gap: 10px;
                padding: 20px;
            }
            .heading {
                font-family: fifthtry-github-io-inter-font-Inter;
                font-size: 24px;
                line-height: 36px;
                font-weight: 500;
                overflow: hidden;
                width: 100%;
            }
            .wrapper {
                gap: 10px;
                display: flex;
                flex-direction: column;
                min-height: 200px;
                background-color: white;
                padding: 12px;
            }
        }
    `;

        // Attach the created elements to the shadow dom
        shadow.appendChild(style);
        console.log(style.isConnected);

        page.appendChild(heading);
        page.appendChild(wrapper);
        //page.appendChild(button);

        shadow.appendChild(page);
        this.setAttribute("style", "width: 100%");
    }
}

// Define the new element
customElements.define('todo-list-display', Todo);



function todo_item_display(obj, index, data, todo_list) {
    const todo = document.createElement('div');
    todo.setAttribute('tabindex', 0);
    todo.setAttribute('id', index.toString());

    const check = document.createElement('input');
    check.id = 'todocheck' + index;
    check.type = "checkbox";
    check.value = obj.name + '<br/>';
    check.onclick = function (event) {
        if (check.checked) {
            todo.setAttribute("class", "todo-done");
            obj.status = "Done";
        } else {
            todo.setAttribute("class", "todo-added");
            obj.status = "Todo";
        }
        obj.done = check.checked;
        data.todo_list.set(todo_list);
    }

    const text = document.createElement('div');
    text.innerText = obj.name

    if (check.checked) {
        todo.setAttribute("class", "todo-done");
    } else {
        todo.setAttribute("class", "todo-added");
    }

    const input = document.createElement('input');
    input.id = 'todoinput' + index;
    input.onchange = function (event) {
        obj.description = !obj.description;
    }

    todo.appendChild(check);
    todo.appendChild(text);

    return todo;
}
