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

const REGEX_PARAM = /^{(.+)}$/;

function isArg(key) {
    return REGEX_PARAM.test(key);
}

function createRouteTree(routes) {
    const root = {};

    routes.forEach((route) => {
        const parts = route.route.split("/").filter(Boolean);
        let current = root;

        let params = [];

        parts.forEach((part, index) => {
            if (!current[part]) {
                current[part] = {};
            }

            current = current[part];
            if (isArg(part) && index < parts.length - 1) {
                params.push(part);
            }
        });

        current.routeConfigs = current.routeConfigs || { methods: [] };
        current.routeConfigs.methods.push({
            method: route.method,
            options: route.options || [],
            params,
        });
    });

    return root;
}

function createRouteNavBar(navElement, routes) {
    const routeTree = createRouteTree(routes);
    buildNavList(navElement, routeTree, "");
}

function isEndArgKey(leaf, key) {
    return (
        isArg(key) && leaf && Object.keys(leaf).length == 1 && leaf.routeConfigs
    );
}

function buildNavList(navList, leaf, path, param, ulParent) {
    const ul = ulParent ?? document.createElement("ul");
    Object.keys(leaf)
        .sort((a, b) => {
            if (isArg(a)) return 1;
            if (isArg(b)) return -1;
            return 0;
        })
        .forEach((key) => {
            if (key === "routeConfigs") {
                leaf.routeConfigs.methods.forEach((methodInfo) => {
                    const item = document.createElement("li", {
                        is: "route-item",
                    });
                    item.route = methodInfo.method;
                    item.path = path;
                    item.method = methodInfo.method;
                    item.param = param;
                    item.params = methodInfo.params;
                    item.options = methodInfo.options;
                    ul.appendChild(item);
                });
                return;
            }

            const current = leaf[key];

            let el = navList;
            let newPath = path;
            if (isEndArgKey(current, key)) {
                buildNavList(navList, current, path, key, ul);
                return;
            } else {
                newPath = `${path}/${key}`;

                el = document.createElement("li", { is: "route-item" });
                el.route = key;
                el.path = newPath;

                ul.appendChild(el);
            }

            if (Object.keys(current).length > 0) {
                let param = isArg(key) ? key : undefined;
                buildNavList(el, current, newPath, param);
            }
        });
    navList.appendChild(ul);
}

class RouteItem extends HTMLLIElement {
    constructor() {
        super();
        this._route = "";
        this._path = "";
        this._method = "";
        this._param = "";
        this._params = [];
        this._options = [];
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

    set params(value) {
        this._params = value;
        this.render();
    }

    get params() {
        return this._params;
    }

    set options(value) {
        this._options = value || [];
        this.render();
    }

    get options() {
        return this._options;
    }

    onLinkClick(event) {
        event.preventDefault();
        if (this.method) {
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

            if (this.params && this.params.length > 0) {
                apiRequestSender.setAttribute("params", this.params.join(","));
            }

            if (this.options && this.options.length > 0) {
                apiRequestSender.setAttribute(
                    "options",
                    this.options.join(",")
                );
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
        return ["method", "route", "param", "options"];
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
        const params = (this.getAttribute("params") || "")
            .split(",")
            .map((p) => p.trim())
            .filter((p) => p);
        const optionsAttr = this.getAttribute("options");
        const options = optionsAttr
            ? optionsAttr.split(",").map((opt) => opt.trim())
            : [];
        const isDownload = options.includes("download");
        const isUpload = options.includes("upload");

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
            ${params.map((param, index) => {
                return `<input type="text" id="param-input-${index}" placeholder="${param}" />`;
            })}
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
                </div>
            `;
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
        const params = (this.getAttribute("params") || "")
            .split(",")
            .map((p) => p.trim())
            .filter((p) => p);
        const optionsAttr = this.getAttribute("options");
        const options = optionsAttr
            ? optionsAttr.split(",").map((opt) => opt.trim())
            : [];
        const isDownload = options.includes("download");

        const resultsDiv = this.shadowRoot.getElementById("results");
        resultsDiv.textContent = "Loading...";

        // 1. Construct URL
        if (param) {
            const paramInput = this.shadowRoot.getElementById("param-input");
            route = `${route}/${paramInput.value || ""}`;
        }

        params.forEach((param, index) => {
            const paramInput = this.shadowRoot.getElementById(
                `param-input-${index}`
            );
            route = route.replace(param, paramInput.value || "unknown");
        });

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
        const fetchOptions = { method };

        if (["POST", "PUT", "PATCH"].includes(method)) {
            const fileInput = this.shadowRoot.getElementById("file-input");
            if (fileInput && fileInput.files.length > 0) {
                const formData = new FormData();
                const bodyContent =
                    this.shadowRoot.getElementById("body-input").value;
                formData.append("file", fileInput.files[0]);
                fetchOptions.body = formData;
            } else {
                fetchOptions.headers = { "Content-Type": "application/json" };
                fetchOptions.body =
                    this.shadowRoot.getElementById("body-input")?.value || "{}";
            }
        }

        // 3. Execute Fetch
        try {
            // const mockUrl = `http://localhost:4520${route}`; //for dev
            const mockUrl = `${route}`;

            const response = await fetch(mockUrl, {
                // mode: "cors",
                ...fetchOptions,
            });

            if (isDownload) {
                let filename = "";
                const filenameInput =
                    this.shadowRoot.getElementById("param-input");

                if (filenameInput) {
                    filenameInput.value || "download.json";
                } else if (params.length > 0) {
                    filename =
                        params
                            .map((_, index) => {
                                return this.shadowRoot.getElementById(
                                    `param-input-${index}`
                                ).value;
                            })
                            .join("_") + ".json";
                } else {
                    filename = "download.json";
                }

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

            const contentType = response.headers.get("content-type");

            // Check if response is an image
            if (contentType && contentType.startsWith("image/")) {
                const blob = await response.blob();
                const imageUrl = URL.createObjectURL(blob);
                resultsDiv.innerHTML = `<img src="${imageUrl}" alt="Response image" style="max-width: 100%; height: auto;" />`;
                return;
            }

            // Check if response is JSON
            if (contentType && contentType.includes("application/json")) {
                const data = await response.json();
                resultsDiv.textContent = JSON.stringify(data, null, 2);
            } else {
                // For other content types, display as text
                const text = await response.text();
                resultsDiv.textContent = text;
            }
        } catch (error) {
            resultsDiv.textContent = `Error: ${error.message}`;
        }
    }
}
customElements.define("api-request-sender", ApiRequestSender);
