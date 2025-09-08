const mock_routes = [
    {
        method: "GET",
        route: "/auth/users",
        options: [],
    },
    {
        method: "POST",
        route: "/auth/users",
        options: [],
    },
    {
        method: "GET",
        route: "/auth/users/{id}",
        options: [],
    },
    {
        method: "PUT",
        route: "/auth/users/{id}",
        options: [],
    },
    {
        method: "PATCH",
        route: "/auth/users/{id}",
        options: [],
    },
    {
        method: "DELETE",
        route: "/auth/users/{id}",
        options: [],
    },
    {
        method: "POST",
        route: "/auth/login",
        options: [],
    },
    {
        method: "POST",
        route: "/auth/logout",
        options: [],
    },
    {
        method: "POST",
        route: "/account/login",
        options: [],
    },
    {
        method: "POST",
        route: "/account/logout",
        options: [],
    },
    {
        method: "POST",
        route: "/api/auth/activate",
        options: [],
    },
    {
        method: "POST",
        route: "/api/auth/change-password",
        options: [],
    },
    {
        method: "POST",
        route: "/api/auth/forget-password",
        options: [],
    },
    {
        method: "GET",
        route: "/api/auth/login/auth",
        options: [],
    },
    {
        method: "POST",
        route: "/api/auth/login",
        options: [],
    },
    {
        method: "POST",
        route: "/api/auth/register",
        options: [],
    },
    {
        method: "GET",
        route: "/api/exercises",
        options: [],
    },
    {
        method: "POST",
        route: "/api/exercises",
        options: [],
    },
    {
        method: "DELETE",
        route: "/api/workout/{id}",
        options: [],
    },
    {
        method: "GET",
        route: "/api/workout",
        options: [],
    },
    {
        method: "GET",
        route: "/api/workout/{id}",
        options: [],
    },
    {
        method: "GET",
        route: "/api/workout/list",
        options: [],
    },
    {
        method: "POST",
        route: "/api/workout",
        options: [],
    },
    {
        method: "PUT",
        route: "/api/workout",
        options: [],
    },
    {
        method: "GET",
        route: "/jgd-examples/array-object-root",
        options: [],
    },
    {
        method: "GET",
        route: "/jgd-examples/customers-orders",
        options: [],
    },
    {
        method: "GET",
        route: "/jgd-examples/entities-blog-system",
        options: [],
    },
    {
        method: "GET",
        route: "/jgd-examples/ranged-array-object-root",
        options: [],
    },
    {
        method: "GET",
        route: "/jgd-examples/root-address-fr-fr",
        options: [],
    },
    {
        method: "GET",
        route: "/jgd-examples/root-e-commerce",
        options: [],
    },
    {
        method: "GET",
        route: "/jgd-examples/root-user",
        options: [],
    },
    {
        method: "GET",
        route: "/jgd-examples/single-object-root",
        options: [],
    },
    {
        method: "GET",
        route: "/jgd-examples/user-post-entities",
        options: [],
    },
    {
        method: "GET",
        route: "/reports/companies",
        options: [],
    },
    {
        method: "GET",
        route: "/reports/companies/{id}",
        options: [],
    },
    {
        method: "PUT",
        route: "/users",
        options: [],
    },
    {
        method: "GET",
        route: "/users",
        options: [],
    },
    {
        method: "GET",
        route: "/users/2",
        options: [],
    },
    {
        method: "GET",
        route: "/users/3",
        options: [],
    },
    {
        method: "GET",
        route: "/users/4",
        options: [],
    },
    {
        method: "GET",
        route: "/users/5",
        options: [],
    },
    {
        method: "GET",
        route: "/users/{id}",
        options: [],
    },
    {
        method: "GET",
        route: "/users/luis",
        options: [],
    },
    {
        method: "GET",
        route: "/users/images",
        options: [],
    },
    {
        method: "GET",
        route: "/users/images/{id}",
        options: [],
    },
    {
        method: "GET",
        route: "/users/images/other/animal-eye-staring-close",
        options: [],
    },
    {
        method: "GET",
        route: "/users/images/other/animal-portrait-close-up",
        options: [],
    },
    {
        method: "POST",
        route: "/users",
        options: [],
    },
    {
        method: "GET",
        route: "/cities",
        options: [],
    },
    {
        method: "POST",
        route: "/cities",
        options: [],
    },
    {
        method: "GET",
        route: "/cities/{id}",
        options: [],
    },
    {
        method: "PUT",
        route: "/cities/{id}",
        options: [],
    },
    {
        method: "PATCH",
        route: "/cities/{id}",
        options: [],
    },
    {
        method: "DELETE",
        route: "/cities/{id}",
        options: [],
    },
    {
        method: "GET",
        route: "/companies",
        options: [],
    },
    {
        method: "POST",
        route: "/companies",
        options: [],
    },
    {
        method: "GET",
        route: "/companies/{id}",
        options: [],
    },
    {
        method: "PUT",
        route: "/companies/{id}",
        options: [],
    },
    {
        method: "PATCH",
        route: "/companies/{id}",
        options: [],
    },
    {
        method: "DELETE",
        route: "/companies/{id}",
        options: [],
    },
    {
        method: "GET",
        route: "/jgd-examples/e-commerce",
        options: [],
    },
    {
        method: "POST",
        route: "/jgd-examples/e-commerce",
        options: [],
    },
    {
        method: "GET",
        route: "/jgd-examples/e-commerce/{id}",
        options: [],
    },
    {
        method: "PUT",
        route: "/jgd-examples/e-commerce/{id}",
        options: [],
    },
    {
        method: "PATCH",
        route: "/jgd-examples/e-commerce/{id}",
        options: [],
    },
    {
        method: "DELETE",
        route: "/jgd-examples/e-commerce/{id}",
        options: [],
    },
    {
        method: "GET",
        route: "/products",
        options: [],
    },
    {
        method: "POST",
        route: "/products",
        options: [],
    },
    {
        method: "GET",
        route: "/products/{_id}",
        options: [],
    },
    {
        method: "PUT",
        route: "/products/{_id}",
        options: [],
    },
    {
        method: "PATCH",
        route: "/products/{_id}",
        options: [],
    },
    {
        method: "DELETE",
        route: "/products/{_id}",
        options: [],
    },
    {
        method: "POST",
        route: "/upload",
        options: ["upload"],
    },
    {
        method: "GET",
        route: "/upload/{file_name}",
        options: ["download"],
    },
    {
        method: "GET",
        route: "/upload",
        options: [],
    },
    {
        method: "POST",
        route: "/docs",
        options: ["upload"],
    },
    {
        method: "GET",
        route: "/docs/{file_name}",
        options: ["download"],
    },
    {
        method: "GET",
        route: "/docs",
        options: [],
    },
    {
        method: "GET",
        route: "/mock-server/collections",
        options: [],
    },
    {
        method: "POST",
        route: "/mock-server/collections",
        options: ["upload"],
    },
    {
        method: "GET",
        route: "/mock-server/collections/download",
        options: ["download"],
    },
    {
        method: "GET",
        route: "/mock-server/collections/{name}",
        options: [],
    },
    {
        method: "POST",
        route: "/mock-server/collections/{name}",
        options: ["upload"],
    },
    {
        method: "GET",
        route: "/mock-server/collections/{name}/download",
        options: ["download"],
    },
];
