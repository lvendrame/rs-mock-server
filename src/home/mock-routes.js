const mock_routes = [
    {
        route: "/auth/users",
        method: "GET",
    },
    {
        route: "/auth/users",
        method: "POST",
    },
    {
        route: "/auth/users/:username",
        method: "GET",
    },
    {
        route: "/auth/users/:username",
        method: "PUT",
    },
    {
        route: "/auth/users/:username",
        method: "PATCH",
    },
    {
        route: "/auth/users/:username",
        method: "DELETE",
    },
    {
        route: "/auth/login",
        method: "POST",
    },
    {
        route: "/auth/logout",
        method: "POST",
    },
    {
        route: "/account/login",
        method: "POST",
    },
    {
        route: "/account/logout",
        method: "POST",
    },
    {
        route: "/api/auth/activate",
        method: "POST",
    },
    {
        route: "/api/auth/change-password",
        method: "POST",
    },
    {
        route: "/api/auth/forget-password",
        method: "POST",
    },
    {
        route: "/api/auth/login/auth",
        method: "GET",
    },
    {
        route: "/api/auth/login",
        method: "POST",
    },
    {
        route: "/api/auth/register",
        method: "POST",
    },
    {
        route: "/api/exercises",
        method: "GET",
    },
    {
        route: "/api/exercises",
        method: "POST",
    },
    {
        route: "/api/workout/:id",
        method: "DELETE",
    },
    {
        route: "/api/workout",
        method: "GET",
    },
    {
        route: "/api/workout/:id",
        method: "GET",
    },
    {
        route: "/api/workout/list",
        method: "GET",
    },
    {
        route: "/api/workout",
        method: "POST",
    },
    {
        route: "/api/workout",
        method: "PUT",
    },
    {
        route: "/jgd-examples/array-object-root",
        method: "GET",
    },
    {
        route: "/jgd-examples/customers-orders",
        method: "GET",
    },
    {
        route: "/jgd-examples/entities-blog-system",
        method: "GET",
    },
    {
        route: "/jgd-examples/ranged-array-object-root",
        method: "GET",
    },
    {
        route: "/jgd-examples/root-address-fr-fr",
        method: "GET",
    },
    {
        route: "/jgd-examples/root-ecommerce",
        method: "GET",
    },
    {
        route: "/jgd-examples/root-user",
        method: "GET",
    },
    {
        route: "/jgd-examples/single-object-root",
        method: "GET",
    },
    {
        route: "/jgd-examples/user-post-entities",
        method: "GET",
    },
    {
        route: "/users",
        method: "PUT",
    },
    {
        route: "/users",
        method: "GET",
    },
    {
        route: "/users/2",
        method: "GET",
    },
    {
        route: "/users/3",
        method: "GET",
    },
    {
        route: "/users/4",
        method: "GET",
    },
    {
        route: "/users/5",
        method: "GET",
    },
    {
        route: "/users/:id",
        method: "GET",
    },
    {
        route: "/users/luis",
        method: "GET",
    },
    {
        route: "/users/images",
        method: "GET",
    },
    {
        route: "/users/images/:id",
        method: "GET",
    },
    {
        route: "/users/images/other/animal-eye-staring-close",
        method: "GET",
    },
    {
        route: "/users/images/other/animal-portrait-close-up",
        method: "GET",
    },
    {
        route: "/users",
        method: "POST",
    },
    {
        route: "/cities",
        method: "GET",
    },
    {
        route: "/cities",
        method: "POST",
    },
    {
        route: "/cities/:id",
        method: "GET",
    },
    {
        route: "/cities/:id",
        method: "PUT",
    },
    {
        route: "/cities/:id",
        method: "PATCH",
    },
    {
        route: "/cities/:id",
        method: "DELETE",
    },
    {
        route: "/companies",
        method: "GET",
    },
    {
        route: "/companies",
        method: "POST",
    },
    {
        route: "/companies/:id",
        method: "GET",
    },
    {
        route: "/companies/:id",
        method: "PUT",
    },
    {
        route: "/companies/:id",
        method: "PATCH",
    },
    {
        route: "/companies/:id",
        method: "DELETE",
    },
    {
        route: "/jgd-examples/ecommerce",
        method: "GET",
    },
    {
        route: "/jgd-examples/ecommerce",
        method: "POST",
    },
    {
        route: "/jgd-examples/ecommerce/:id",
        method: "GET",
    },
    {
        route: "/jgd-examples/ecommerce/:id",
        method: "PUT",
    },
    {
        route: "/jgd-examples/ecommerce/:id",
        method: "PATCH",
    },
    {
        route: "/jgd-examples/ecommerce/:id",
        method: "DELETE",
    },
    {
        route: "/products",
        method: "GET",
    },
    {
        route: "/products",
        method: "POST",
    },
    {
        route: "/products/:_id",
        method: "GET",
    },
    {
        route: "/products/:_id",
        method: "PUT",
    },
    {
        route: "/products/:_id",
        method: "PATCH",
    },
    {
        route: "/products/:_id",
        method: "DELETE",
    },
    {
        route: "/upload",
        method: "POST",
        options: ["upload"],
    },
    {
        route: "/upload/:file_name",
        method: "GET",
        options: ["download"],
    },
    {
        route: "/upload",
        method: "GET",
    },
    {
        route: "/docs",
        method: "POST",
        options: ["upload"],
    },
    {
        route: "/docs/:file_name",
        method: "GET",
        options: ["download"],
    },
    {
        route: "/docs",
        method: "GET",
    },
];
