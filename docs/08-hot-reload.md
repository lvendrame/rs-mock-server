# Hot Reload & Development

Automatic server restart and file monitoring for rapid development with rs-mock-server.

## Overview

Hot reload automatically restarts the mock server when you modify files, enabling rapid development cycles. Changes to mock files are detected and applied instantly.

## How Hot Reload Works

rs-mock-server monitors the mock directory for file changes and automatically restarts the server when changes are detected. This allows you to modify your mock responses and see the changes immediately without manually restarting the server.

### File Monitoring

-   Monitors the folder specified with `--folder` flag (default: `mocks/`)
-   All subdirectories are monitored recursively
-   Uses a 300ms debounce delay to prevent excessive restarts during rapid file changes

### Upload Directory Handling

Upload directories (folders containing `{upload}` in the name) have special handling:

-   Only directory-level changes trigger reloads
-   Individual file changes within upload folders are ignored to prevent reload loops during file uploads

## Development Workflow

1. **Start the server:**

    ```bash
    rs-mock-server
    ```

2. **Edit mock files** in your `mocks/` directory

3. **Server automatically restarts** when files are saved

4. **Test your changes** - endpoints are immediately updated

## Example

```bash
# Terminal 1: Start server
$ rs-mock-server

# Terminal 2: Create/edit a mock file
$ echo '{"message": "Hello World"}' > mocks/hello.json

# Server automatically restarts and the new endpoint is available
$ curl http://localhost:4520/hello
{"message": "Hello World"}
```

Hot reload is always enabled and cannot be disabled or configured.

## Next Steps

-   Learn about [Web Interface](web-interface.md) for visual development
-   Explore [Basic Routing](basic-routing.md) for organizing files efficiently
-   Try [REST APIs](rest-apis.md) for comprehensive API development
-   See [JGD Files](jgd-files.md) for dynamic data generation
