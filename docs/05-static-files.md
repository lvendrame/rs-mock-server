# Static Files

## Overview

🌐 **Public Directory Serving**: Serve a directory of static files (e.g., a frontend build) from a root public folder, or map a folder like public-assets to a custom /assets route.

## Special "Public" Folder for Static Serving

To serve a directory of static assets (like a frontend app), you can use a specially named public folder in your mock directory root.

### Public Folder

If you create a folder named `public`, all its contents will be served from the `/public` route.

**Example:**

```
./mocks/public/home.html → GET /public/home.html
```

### Public-Alias Folder

You can customize the URL path by adding a dash. A folder named `public-static` will serve its files from the `/static` route.

**Example:**

```
./mocks/public-static/style.css → GET /static/style.css
```
