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
            console.log("Navigating to:", this.method, this.path, this.param);
        } else {
            console.log("Toggle UL children:", this.path);
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
