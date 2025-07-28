# rs-mock-server 🦀

A simple, zero-configuration mock server built in Rust. Spin up a realistic REST API for local development or testing just by creating folders and files.

It works by scanning a directory and mapping its structure directly to API routes, with clever filename conventions for handling HTTP methods, dynamic parameters, and static assets.

---

## Features

-   🚀 **File-System Routing**: Your folder structure defines your API routes. No config files needed.
-   🧩 **Dynamic Path Generation**: Create routes with parameters (`{id}`), specific values (`{admin}`), and even numeric ranges (`{1-10}`) right from the filename.
-   ⚙️ **Full HTTP Method Support**: Define `GET`, `POST`, `PUT`, `DELETE`, `PATCH`, and `OPTIONS` endpoints.
-   🖼️ **Static File Serving**: Automatically serves any file (like images, CSS, or JS) with its correct `Content-Type` if the filename doesn't match a method pattern.
-   🔧 **Configurable**: Easily change the port and mock directory via command-line arguments.
-   ⚡ **Lightweight & Fast**: Built with Rust for minimal resource usage and maximum performance.

---

## How It Works

The server recursively scans a root directory (defaults to `./mocks`) and translates the file and folder paths into API endpoints.

### Folder Structure → URL Path

The path of each folder becomes the base URL for the routes within it.

-   A folder at `./mocks/api/users` creates the base route `/api/users`.
-   A nested folder at `./mocks/api/users/profiles` creates the base route `/api/users/profiles`.

### Filename Conventions → Endpoints

The name of a file determines the **HTTP method** and the **final URL segment**. The content of the file is served as the response body.

The following table shows how different filename patterns are mapped to routes, assuming they are inside a `./mocks/api/users` directory:

| Filename Pattern      | Example File      | Generated Route(s)                                                    | Description                                                                                                                                                                    |
| :-------------------- | :---------------- | :-------------------------------------------------------------------- | :----------------------------------------------------------------------------------------------------------------------------------------------------------------------------- |
| `[method]`            | `get.json`        | `GET /api/users`                                                      | Creates a route for a standard HTTP method.                                                                                                                                    |
| `[method]{param}`     | `get{id}.json`    | `GET /api/users/{id}`                                                 | A dynamic segment that accepts any value in that position.                                                                                                                     |
| `[method]{value}`     | `get{admin}.json` | `GET /api/users/admin`                                                | Matches a specific, hardcoded value.                                                                                                                                           |
| `[method]{start-end}` | `get{1-5}.json`   | `GET /api/users/1`<br>`GET /api/users/2`<br>...<br>`GET /api/users/5` | A numeric range that generates multiple distinct routes.                                                                                                                       |
| `[filename].[ext]`    | `avatar.png`      | `GET /api/users/avatar`                                               | **Static File**. Any filename that doesn't match the patterns above is served as a static asset. The `Content-Type` header is automatically set based on the file's extension. |

---

## Installation

### With Cargo

If the crate is published on [crates.io](https://crates.io), you can install it directly:

```sh
cargo install rs-mock-server
```

### From Source

Alternatively, you can clone the repository and build it yourself:

```sh
# Clone the repository
git clone https://github.com/lvendrame/rs-mock-server.git

# Navigate into the project directory
cd rs-mock-server

# Build the project for release
cargo build --release

# The executable will be at ./target/release/rs-mock-server
./target/release/rs-mock-server --help
```

---

## Usage

You can run the server using the `rs-mock-server` executable.

**To start the server with default settings:**
(This will use the `./mocks` folder and run on port `4520`)

```sh
rs-mock-server
```

**To specify a custom port and mock directory:**

```sh
rs-mock-server --port 8080 --folder ./my-api-mocks
```

### Command-Line Options

```sh
Usage: rs-mock-server [OPTIONS]

Options:
  -p, --port <PORT>      Port to run the server on [default: 4520]
  -f, --folder <FOLDER>  Directory to load mock files from [default: mocks]
  -h, --help             Print help
  -V, --version          Print version
```

---

## Example Walkthrough

Imagine you have the following directory structure:

```
mocks/
├── api/
│   ├── users/
│   │   ├── get.json         # Contains a JSON array of all users
│   │   ├── post.json        # Contains a success message for user creation
│   │   └── get{id}.json    # Contains a single user object template
│   ├── products/
│   │   ├── get{1-3}.json   # Contains a product template for IDs 1, 2, 3
│   │   └── get{special}.json# Contains a specific "special" product
│   └── status.txt           # Contains the plain text "API is running"
└── assets/
    └── logo.svg             # An SVG image file
```

Running `rs-mock-server` in the same directory will create the following endpoints:

| Method   | Route                   | Response Body From...                  | `Content-Type`     |
| :------- | :---------------------- | :------------------------------------- | :----------------- |
| **GET**  | `/api/users`            | `mocks/api/users/get.json`             | `application/json` |
| **POST** | `/api/users`            | `mocks/api/users/post.json`            | `application/json` |
| **GET**  | `/api/users/{id}`       | `mocks/api/users/get{id}.json`         | `application/json` |
| **GET**  | `/api/products/1`       | `mocks/api/products/get{1-3}.json`     | `application/json` |
| **GET**  | `/api/products/2`       | `mocks/api/products/get{1-3}.json`     | `application/json` |
| **GET**  | `/api/products/3`       | `mocks/api/products/get{1-3}.json`     | `application/json` |
| **GET**  | `/api/products/special` | `mocks/api/products/get{special}.json` | `application/json` |
| **GET**  | `/api/status.txt`       | `mocks/api/status.txt`                 | `text/plain`       |
| **GET**  | `/assets/logo`.         | `mocks/assets/logo.svg`                | `image/svg+xml`    |
