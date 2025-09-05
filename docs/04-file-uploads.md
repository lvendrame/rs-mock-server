# File Uploads & Downloads

Create file upload and download endpoints with automatic file handling using special `{upload}` folders.

## Overview

When the server detects a specially named `{upload}` folder, it automatically creates endpoints for uploading and downloading files with proper Content-Type detection and file management.

## Basic Upload Folder

Create a folder named `{upload}` to enable file operations:

```
mocks/
└── {upload}/
```

This creates three endpoints:

-   **POST** `/upload` - Upload files (multipart/form-data)
-   **GET** `/upload` - List all uploaded files
-   **GET** `/upload/{filename}` - Download files by name

## Upload Folder Configuration

The `{upload}` folder supports additional configuration through special naming patterns:

| Folder Pattern        | Upload Route   | List Route    | Download Route           | Temporary Files | Description                  |
| :-------------------- | :------------- | :------------ | :----------------------- | :-------------- | :--------------------------- |
| `{upload}`            | `POST /upload` | `GET /upload` | `GET /upload/{filename}` | No              | Basic upload/download        |
| `{upload}{temp}`      | `POST /upload` | `GET /upload` | `GET /upload/{filename}` | **Yes**         | Files deleted on server stop |
| `{upload}-files`      | `POST /files`  | `GET /files`  | `GET /files/{filename}`  | No              | Custom route name            |
| `{upload}{temp}-docs` | `POST /docs`   | `GET /docs`   | `GET /docs/{filename}`   | **Yes**         | Custom route + temporary     |

## Configuration Examples

### Basic Upload

```
mocks/
└── {upload}/
```

-   **Routes**: `/upload`
-   **Persistence**: Files persist until manually deleted

### Temporary Files

```
mocks/
└── {upload}{temp}/
```

-   **Routes**: `/upload`
-   **Persistence**: Files automatically deleted when server stops

### Custom Route Name

```
mocks/
└── {upload}-attachments/
```

-   **Routes**: `/attachments`
-   **Persistence**: Files persist until manually deleted

### Custom + Temporary

```
mocks/
└── {upload}{temp}-documents/
```

-   **Routes**: `/documents`
-   **Persistence**: Files automatically deleted when server stops

## Upload Endpoint

### Single File Upload

**Request:**

```bash
curl -X POST http://localhost:4520/upload \
  -F "file=@document.pdf"
```

**Response:**

```
File uploaded successfully: document.pdf
```

### Multiple File Upload

**Request:**

```bash
curl -X POST http://localhost:4520/upload \
  -F "file1=@image.jpg" \
  -F "file2=@document.pdf" \
  -F "file3=@data.csv"
```

**Response:**

```
Files uploaded successfully: image.jpg, document.pdf, data.csv
```

### Upload with Custom Field Name

**Request:**

```bash
curl -X POST http://localhost:4520/upload \
  -F "attachment=@report.xlsx"
```

**Response:**

```
File uploaded successfully: report.xlsx
```

## List Files Endpoint

### List All Uploaded Files

**Request:**

```bash
curl http://localhost:4520/upload
```

**Response:**

```json
{
    "files": [
        {
            "name": "/upload/document.pdf",
            "size": 1048576,
            "uploaded_at": "2024-01-15T10:30:00Z"
        },
        {
            "name": "/upload/image.jpg",
            "size": 524288,
            "uploaded_at": "2024-01-15T10:25:00Z"
        },
        {
            "name": "/upload/data.csv",
            "size": 2048,
            "uploaded_at": "2024-01-15T10:20:00Z"
        }
    ],
    "total": 3
}
```

## Download Endpoint

### Download File

**Request:**

```bash
curl http://localhost:4520/upload/document.pdf \
  -o downloaded_document.pdf
```

**Response Headers:**

```
Content-Type: application/pdf
Content-Disposition: attachment; filename="document.pdf"
Content-Length: 1048576
```

### Download with Browser

Navigate to `http://localhost:4520/upload/image.jpg` in your browser to download or view the file directly.

## Content-Type Detection

rs-mock-server automatically detects and sets appropriate Content-Type headers:

| File Extension  | Content-Type                                                              |
| --------------- | ------------------------------------------------------------------------- |
| `.pdf`          | `application/pdf`                                                         |
| `.jpg`, `.jpeg` | `image/jpeg`                                                              |
| `.png`          | `image/png`                                                               |
| `.gif`          | `image/gif`                                                               |
| `.svg`          | `image/svg+xml`                                                           |
| `.txt`          | `text/plain`                                                              |
| `.csv`          | `text/csv`                                                                |
| `.json`         | `application/json`                                                        |
| `.xml`          | `application/xml`                                                         |
| `.zip`          | `application/zip`                                                         |
| `.doc`          | `application/msword`                                                      |
| `.docx`         | `application/vnd.openxmlformats-officedocument.wordprocessingml.document` |
| `.xls`          | `application/vnd.ms-excel`                                                |
| `.xlsx`         | `application/vnd.openxmlformats-officedocument.spreadsheetml.sheet`       |

## File Organization

### Directory Structure After Uploads

```
mocks/
├── {upload}/
│   ├── document.pdf       # Uploaded file
│   ├── image.jpg          # Uploaded file
│   └── data.csv           # Uploaded file
├── {upload}{temp}-docs/
│   ├── temp_report.xlsx   # Temporary file
│   └── draft.docx         # Temporary file
└── api/
    └── get.json
```

### File Naming

-   **Original Names**: Files keep their original names
-   **Overwrites**: Uploading a file with the same name overwrites the existing file
-   **No Conflicts**: No automatic renaming or versioning

## Error Handling

### File Not Found

**Request:** `GET /upload/nonexistent.pdf`
**Response:** `404 Not Found`

```json
{
    "error": "File not found: nonexistent.pdf"
}
```

### No Files in Upload

**Request:** `POST /upload` (without files)
**Response:** `400 Bad Request`

```json
{
    "error": "No files provided"
}
```

### Invalid Upload Format

**Request:** `POST /upload` (not multipart/form-data)
**Response:** `400 Bad Request`

```json
{
    "error": "Invalid upload format. Use multipart/form-data"
}
```

## Web Interface Integration

The web interface provides a user-friendly upload experience:

1. **Drag & Drop**: Drag files directly onto the upload area
2. **File Browser**: Click to browse and select files
3. **Progress Indicator**: Visual upload progress
4. **File List**: View all uploaded files with download links
5. **Delete Option**: Remove files through the interface

## Authentication Integration

Protect upload endpoints using the `$` prefix:

### Protected Upload Folder

```
mocks/
├── auth/
│   └── {auth}.json         # Authentication system
├── ${upload}/              # Protected uploads
└── {upload}-public/        # Public uploads
```

**Usage:**

```bash
# Login first
curl -X POST http://localhost:4520/auth/login \
  -H "Content-Type: application/json" \
  -d '{"username": "admin", "password": "admin123"}'

# Upload with authentication
curl -X POST http://localhost:4520/upload \
  -H "Authorization: Bearer <jwt_token>" \
  -F "file=@secure_document.pdf"
```

## Temporary File Management

### Automatic Cleanup

Files in `{temp}` folders are automatically deleted when:

-   **Server Stops**: All temporary files removed on shutdown
-   **Server Restart**: Temporary files don't persist across restarts

### Manual Cleanup

For non-temporary uploads, files persist until manually deleted:

```bash
# Delete specific file (not implemented by default)
# You would need to implement this or delete files directly from the filesystem
rm mocks/{upload}/unwanted_file.pdf
```

## Upload Limits

### Current Limits

-   **File Size**: No built-in limit (depends on system resources)
-   **File Count**: No built-in limit per upload
-   **Storage**: Limited by available disk space

### Best Practices

-   Monitor disk usage for large file uploads
-   Use temporary folders for test files to prevent accumulation

## Integration Examples

### With REST APIs

```
mocks/
├── api/
│   ├── products/
│   │   └── rest.json       # Product catalog
│   └── attachments/
│       └── {upload}/       # Product attachments
```

### With Authentication

```
mocks/
├── auth/
│   └── {auth}.json         # Authentication
├── $private/
│   └── {upload}/           # Private file uploads
└── public/
    └── {upload}-assets/    # Public asset uploads
```

### Multiple Upload Areas

```
mocks/
├── {upload}-documents/     # General documents
├── {upload}-images/        # Image uploads
├── {upload}{temp}-temp/    # Temporary files
└── $secure/
    └── {upload}/           # Secure uploads
```

## Hot Reload Behavior

**Note**: Upload folders (containing `{upload}` in the name) are excluded from hot reload monitoring to prevent server restarts when files are uploaded during testing.

## Next Steps

-   Learn about [Authentication](03-authentication.md) to protect upload endpoints
-   Explore [Static File Serving](05-static-files.md) for serving uploaded assets
-   Try the [Web Interface](07-web-interface.md) for interactive file management
-   See [Basic Routing](01-basic-routing.md) for organizing upload endpoints
