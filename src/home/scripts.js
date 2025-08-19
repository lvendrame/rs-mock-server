let logo = `                                  ___     ___
                                 (o o)   (o o)
 _____                          (  V  ) (  V  )                         _____
( ___ )------------------------ /--m-m- /--m-m-------------------------( ___ )
 |   |                                                                  |   |
 |   |                                                                  |   |
 |   |     ░█▀▄░█▀▀░░░░░█▄█░█▀█░█▀▀░█░█░░░░░█▀▀░█▀▀░█▀▄░█░█░█▀▀░█▀▄     |   |
 |   |     ░█▀▄░▀▀█░▄▄▄░█░█░█░█░█░░░█▀▄░▄▄▄░▀▀█░█▀▀░█▀▄░▀▄▀░█▀▀░█▀▄     |   |
 |   |     ░▀░▀░▀▀▀░░░░░▀░▀░▀▀▀░▀▀▀░▀░▀░░░░░▀▀▀░▀▀▀░▀░▀░░▀░░▀▀▀░▀░▀     |   |
 |   |                                                                  |   |
 |___|                                                                  |___|
(_____)----------------------------------------------------------------(_____)`;

window.addEventListener("DOMContentLoaded", () => {
    document.getElementById("logo").appendChild(document.createTextNode(logo));

    const navElement = document.getElementById("routes");
    createRouteNavBar(navElement, mock_routes);
});

function createRouteTree(routes) {
    const root = {};

    routes.forEach((route) => {
        const parts = route.route.split("/").filter(Boolean);
        let current = root;

        parts.forEach((part) => {
            if (!current[part]) {
                current[part] = {};
            }
            current = current[part];
        });

        current.routeConfigs = current.routeConfigs || { methods: [] };
        current.routeConfigs.methods.push(route.method);
    });

    return root;
}

function createRouteNavBar(navElement, routes) {
    const routeTree = createRouteTree(routes);
    // Build the navigation bar using the routeTree
    buildNavList(navElement, routeTree, "");
}

function buildNavList(navList, leaf, path, param, ulParent) {
    const ul = ulParent ?? document.createElement("ul");
    Object.keys(leaf)
        .sort((a, b) => {
            if (a.startsWith(":")) return 1;
            if (b.startsWith(":")) return -1;
            return 0;
        })
        .forEach((key) => {
            if (key === "routeConfigs") {
                leaf.routeConfigs.methods.forEach((method) => {
                    const item = document.createElement("li", {
                        is: "route-item",
                    });
                    item.route = method;
                    item.path = path;
                    item.method = method;
                    item.param = param;
                    ul.appendChild(item);
                });
                return;
            }
            const current = leaf[key];

            if (key.startsWith(":")) {
                buildNavList(navList, current, path, key, ul);
                return;
            }

            const newPath = `${path}/${key}`;

            const item = document.createElement("li", { is: "route-item" });
            item.route = key;
            item.path = newPath;

            ul.appendChild(item);

            if (
                (Object.keys(current).length > 0 && current.routeConfigs) ||
                (Object.keys(current).length === 1 && !current.routeConfigs)
            ) {
                buildNavList(item, current, newPath);
            }
        });
    navList.appendChild(ul);
}

const mock_routes = [
    {
        route: "/api/products",
        method: "GET",
    },
    {
        route: "/api/products/:id",
        method: "GET",
    },
    {
        route: "/api/products",
        method: "POST",
    },
    {
        route: "/api/products/:id",
        method: "PUT",
    },
    {
        route: "/api/products/:id",
        method: "DELETE",
    },
];

class RouteItem extends HTMLLIElement {
    constructor() {
        super();
        this._route = "";
        this._path = "";
        this._method = "";
        this._param = "";
    }

    connectedCallback() {
        this.render();
    }

    set route(value) {
        this._route = value;
        this.render();
    }

    get route() {
        return this._route;
    }

    set path(value) {
        this._path = value;
        this.render();
    }

    get path() {
        return this._path;
    }

    set method(value) {
        this._method = value;
        this.render();
    }

    get method() {
        return this._method;
    }

    set param(value) {
        this._param = value;
        this.render();
    }

    get param() {
        return this._param;
    }

    onLinkClick(event) {
        event.preventDefault();
        if (this.method) {
            // document.getElementById("content").textContent = `Navigating to: ${
            //     this.method
            // } ${this.path}${this.param ? "/" + this.param : ""}`;
            const contentDiv = document.getElementById("content");

            // 1. Clear the previous content
            contentDiv.innerHTML = "";

            // 2. Create the new component instance
            const apiRequestSender =
                document.createElement("api-request-sender");

            // 3. Set its attributes from the clicked item's properties
            apiRequestSender.setAttribute("method", this.method);
            apiRequestSender.setAttribute("route", this.path);

            if (this.param) {
                apiRequestSender.setAttribute("param", this.param);
            }

            // 4. Append the new component to the content div
            contentDiv.appendChild(apiRequestSender);
        } else {
            this.classList.toggle("expanded");
        }
    }

    render() {
        if (!this.isConnected || !this.route || !this.path) {
            return;
        }

        if (!this.method) {
            this.classList.add("collapsible");
            this.classList.add("expanded");
        }

        // Find a direct child 'a' tag, if one already exists
        let link = this.querySelector(":scope > a");

        if (!link) {
            link = document.createElement("a");
            link.addEventListener("click", this.onLinkClick.bind(this));

            this.prepend(link);
        }

        link.href = this.param
            ? `#${this.path}/${this.param}`
            : `#${this.path}`;
        link.textContent = this.param
            ? `${this.route} ${this.param}`
            : this.route;
    }
}

customElements.define("route-item", RouteItem, { extends: "li" });

class ApiRequestSender extends HTMLElement {
    constructor() {
        super();
        this.attachShadow({ mode: "open" });
        const template = document.getElementById("api-request-sender-template");
        this.shadowRoot.appendChild(template.content.cloneNode(true));
    }

    static get observedAttributes() {
        return ["method", "route", "param"];
    }

    attributeChangedCallback() {
        this._render();
    }

    connectedCallback() {
        this._render();
    }

    _render() {
        const method = this.getAttribute("method")?.toUpperCase() || "GET";
        const route = this.getAttribute("route") || "/";
        const param = this.getAttribute("param");
        const isDownload = route.endsWith("/download");
        const isUpload = method === "POST" && route.endsWith("/upload");

        // === Render First Row ===
        const firstRow = this.shadowRoot.getElementById("first-row");
        firstRow.innerHTML = `
            <span class="method method-${method}">${method}</span>
            <span class="route">${route}</span>
            ${
                param
                    ? `<input type="text" id="param-input" placeholder="${param}" />`
                    : ""
            }
            <button id="send-btn">Send</button>
        `;

        // === Render Conditional Content (Second Row) ===
        const conditionalContent = this.shadowRoot.getElementById(
            "conditional-content"
        );
        let conditionalHTML = "";

        if (isUpload) {
            conditionalHTML += `
                <div class="file-upload-wrapper">
                    <input type="file" id="file-input" />
                    <span id="file-name">No file selected</span>
                </div>
            `;
        }

        if (isDownload && method === "GET") {
            conditionalHTML += `<input type="text" id="filename-input" placeholder="filename.txt" />`;
        }

        switch (method) {
            case "GET":
                if (!isDownload) {
                    conditionalHTML += `
                        <table id="query-params-table">
                            <thead><tr><th>Key</th><th>Value</th></tr></thead>
                            <tbody>
                                <tr>
                                    <td><input type="text" placeholder="key"></td>
                                    <td><input type="text" placeholder="value"></td>
                                </tr>
                            </tbody>
                        </table>
                        <button id="add-query-btn" class="add-btn">+</button>
                    `;
                }
                break;
            case "POST":
            case "PUT":
            case "PATCH":
                conditionalHTML += `<textarea id="body-input" placeholder="Enter JSON body..."></textarea>`;
                break;
        }
        conditionalContent.innerHTML = conditionalHTML;

        // === Handle Results Display ===
        const resultsContainer =
            this.shadowRoot.getElementById("results-container");
        resultsContainer.style.display = isDownload ? "none" : "block";

        // === Add Event Listeners ===
        this._addEventListeners();
    }

    _addEventListeners() {
        const sendBtn = this.shadowRoot.getElementById("send-btn");
        sendBtn.onclick = () => this._handleSend();

        const addQueryBtn = this.shadowRoot.getElementById("add-query-btn");
        if (addQueryBtn) {
            addQueryBtn.onclick = () => this._addQueryParamRow();
        }

        const fileInput = this.shadowRoot.getElementById("file-input");
        if (fileInput) {
            fileInput.onchange = (e) => {
                const fileNameSpan =
                    this.shadowRoot.getElementById("file-name");
                fileNameSpan.textContent =
                    e.target.files.length > 0
                        ? e.target.files[0].name
                        : "No file selected";
            };
        }
    }

    _addQueryParamRow() {
        const tableBody = this.shadowRoot.querySelector(
            "#query-params-table tbody"
        );
        const newRow = document.createElement("tr");
        newRow.innerHTML = `
            <td><input type="text" placeholder="key"></td>
            <td><input type="text" placeholder="value"></td>
        `;
        tableBody.appendChild(newRow);
    }

    async _handleSend() {
        const method = this.getAttribute("method")?.toUpperCase() || "GET";
        let route = this.getAttribute("route") || "/";
        const param = this.getAttribute("param");
        const isDownload = route.endsWith("/download");

        const resultsDiv = this.shadowRoot.getElementById("results");
        resultsDiv.textContent = "Loading...";

        // 1. Construct URL
        if (param) {
            const paramInput = this.shadowRoot.getElementById("param-input");
            route = route.replace(param, paramInput.value || "");
        }

        if (method === "GET") {
            const table = this.shadowRoot.getElementById("query-params-table");
            if (table) {
                const params = new URLSearchParams();
                const rows = table.querySelectorAll("tbody tr");
                rows.forEach((row) => {
                    const key = row.cells[0].querySelector("input").value;
                    const value = row.cells[1].querySelector("input").value;
                    if (key) params.append(key, value);
                });
                const queryString = params.toString();
                if (queryString) route += `?${queryString}`;
            }
        }

        // 2. Prepare Fetch Options
        const options = { method };

        if (["POST", "PUT", "PATCH"].includes(method)) {
            const fileInput = this.shadowRoot.getElementById("file-input");
            if (fileInput && fileInput.files.length > 0) {
                const formData = new FormData();
                const bodyContent =
                    this.shadowRoot.getElementById("body-input").value;
                formData.append("file", fileInput.files[0]);
                formData.append("jsonData", bodyContent); // Also send JSON data if present
                options.body = formData;
            } else {
                options.headers = { "Content-Type": "application/json" };
                options.body =
                    this.shadowRoot.getElementById("body-input")?.value || "{}";
            }
        }

        // 3. Execute Fetch
        try {
            // For demo, we'll use a placeholder API
            const mockUrl = `https://jsonplaceholder.typicode.com/posts/1`; // Using a mock API endpoint
            console.log("Requesting:", method, route, options);

            const response = await fetch(mockUrl, { mode: "cors" }); // Using mock URL for demo

            if (isDownload) {
                const filenameInput =
                    this.shadowRoot.getElementById("filename-input");
                const filename = filenameInput.value || "download.json";
                const blob = await response.blob();
                const url = window.URL.createObjectURL(blob);
                const a = document.createElement("a");
                a.style.display = "none";
                a.href = url;
                a.download = filename;
                document.body.appendChild(a);
                a.click();
                window.URL.revokeObjectURL(url);
                resultsDiv.textContent = `Download initiated for ${filename}.`;
                return;
            }

            const data = await response.json();
            resultsDiv.textContent = JSON.stringify(data, null, 2);
        } catch (error) {
            resultsDiv.textContent = `Error: ${error.message}`;
        }
    }
}
customElements.define("api-request-sender", ApiRequestSender);
