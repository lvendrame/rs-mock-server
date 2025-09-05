# JGD (JSON Generator Definition) Files

Generate dynamic JSON responses using JGD files for complex data structures, randomization, and realistic mock data generation.

## Overview

JGD (JSON Generator Definition) files allow you to define templates that generate dynamic JSON data instead of serving static responses. This uses the [JGD-rs library](https://github.com/lvendrame/jgd-rs) for realistic data generation with faker patterns and cross-references.

## Basic JGD Structure

### File Extension

JGD files use the `.jgd` extension and follow the JGD v1 format:

```
mocks/
├── users.jgd              # GET /users returns generated JSON
├── products.jgd           # GET /products returns generated JSON
└── api/
    ├── customers.jgd      # GET /api/customers returns generated JSON
    └── orders.jgd         # GET /api/orders returns generated JSON
```

### Simple Object Generation (Root Mode)

**File:** `mocks/user.jgd`

```json
{
    "$format": "jgd/v1",
    "version": "1.0.0",
    "root": {
        "fields": {
            "id": "${ulid}",
            "name": "${name.name}",
            "email": "${internet.safeEmail}",
            "age": {
                "number": {
                    "min": 18,
                    "max": 65,
                    "integer": true
                }
            },
            "active": true
        }
    }
}
```

**Request:** `GET /user`
**Response:**

```json
{
    "id": "01ARZ3NDEKTSV4RRFFQ69G5FAV",
    "name": "Sarah Johnson",
    "email": "sarah.johnson@example.com",
    "age": 32,
    "active": true
}
```

## Schema Structure

### Required Fields

-   `$format`: Always "jgd/v1"
-   `version`: User-defined schema version
-   Either `root` OR `entities` (mutually exclusive)

### Optional Fields

-   `seed`: Random seed for deterministic generation
-   `defaultLocale`: Locale for faker data (EN, FR_FR, DE_DE, etc.)

## Generation Modes

### Root Mode

Generate a single entity (object or array):

**Single Object:**

```json
{
    "$format": "jgd/v1",
    "version": "1.0.0",
    "root": {
        "fields": {
            "id": "${ulid}",
            "name": "${name.firstName}"
        }
    }
}
```

**Array of Objects:**

```json
{
    "$format": "jgd/v1",
    "version": "1.0.0",
    "root": {
        "count": 10,
        "fields": {
            "id": "${uuid.v4}",
            "title": "${lorem.sentence(3, 6)}"
        }
    }
}
```

## Field Types

### Template Strings

Use `${category.method}` format for dynamic data:

```json
{
    "fullName": "${name.firstName} ${name.lastName}",
    "email": "${internet.safeEmail}",
    "id": "${ulid}"
}
```

### Number Generation

```json
{
    "score": {
        "number": {
            "min": 0,
            "max": 100,
            "integer": true
        }
    }
}
```

### Arrays (Primitives Only)

Arrays are only for primitive values (strings, numbers, booleans):

```json
{
    "tags": {
        "array": {
            "count": [1, 5],
            "of": "${lorem.word}"
        }
    },
    "scores": {
        "array": {
            "count": 3,
            "of": {
                "number": {
                    "min": 0,
                    "max": 100,
                    "integer": true
                }
            }
        }
    }
}
```

### Nested Objects

```json
{
    "address": {
        "fields": {
            "street": "${address.streetName}",
            "city": "${address.cityName}",
            "zipCode": "${address.zipCode}"
        }
    }
}
```

### Optional Fields

```json
{
    "middleName": {
        "optional": {
            "of": "${name.firstName}",
            "prob": 0.3
        }
    }
}
```

### Cross-references

```json
{
    "authorId": {
        "ref": "users.id"
    }
}
```

## Count Specifications

### Fixed Count

```json
{
    "users": {
        "count": 5,
        "fields": { "name": "${name.name}" }
    }
}
```

### Range Count

```json
{
    "posts": {
        "count": [10, 20],
        "fields": { "title": "${lorem.sentence}" }
    }
}
```

## Faker Patterns

### Names & Personal Data

-   `${name.firstName}` - First name
-   `${name.lastName}` - Last name
-   `${name.name}` - Full name
-   `${name.nameWithTitle}` - Full name with title

### Internet

-   `${internet.safeEmail}` - Safe email address
-   `${internet.freeEmail}` - Free email address
-   `${internet.username}` - Username
-   `${internet.password(12)}` - Password with length
-   `${internet.IPv4}` - IPv4 address

### Address

-   `${address.cityName}` - City name
-   `${address.countryName}` - Country name
-   `${address.streetName}` - Street name
-   `${address.stateName}` - State name
-   `${address.zipCode}` - ZIP code

### Lorem Text

-   `${lorem.word}` - Single word
-   `${lorem.words(3)}` - Multiple words
-   `${lorem.sentence(5,10)}` - Sentence with word count range
-   `${lorem.paragraph(1,3)}` - Paragraph with sentence count range
-   `${lorem.paragraphs(2)}` - Multiple paragraphs

### Time & Date

-   `${chrono.time}` - Time
-   `${chrono.date}` - Date
-   `${chrono.dateTime}` - Date and time
-   `${chrono.dateTimeBetween(2021-01-01T00:00:00Z,2022-12-31T23:59:59Z)}` - Date between range

### Numbers & Identifiers

-   `${ulid}` - ULID identifier
-   `${uuid.v4}` - UUID v4
-   `${number.digit}` - Single digit

### Boolean

-   `${boolean.boolean}` - Boolean value
-   `${boolean.boolean(80)}` - Boolean with 80% chance of true

### Company

-   `${company.companyName}` - Company name
-   `${company.buzzword}` - Business buzzword
-   `${company.profession}` - Profession

## Context-Aware Keys

### Index and Count

-   `${index}` - Current item index (1-based)
-   `${count}` - Total count of items being generated
-   `${entity.name}` - Name of current entity
-   `${field.name}` - Name of current field

### Multi-level Index

-   `${index(1)}` - Current entity level (default)
-   `${index(2)}` - Parent entity level
-   `${index(3)}` - Grandparent entity level

## Localization

Set the `defaultLocale` field for locale-specific data:

```json
{
    "$format": "jgd/v1",
    "version": "1.0.0",
    "defaultLocale": "FR_FR",
    "root": {
        "fields": {
            "name": "${name.name}",
            "city": "${address.cityName}"
        }
    }
}
```

**Supported Locales:**

-   `EN` - English (default)
-   `FR_FR` - French (France)
-   `DE_DE` - German (Germany)
-   `IT_IT` - Italian (Italy)
-   `PT_BR` - Portuguese (Brazil)
-   `JA_JP` - Japanese (Japan)
-   `AR_SA` - Arabic (Saudi Arabia)
-   `CY_GB` - Welsh (Great Britain)

## Deterministic Generation

Use seeds for reproducible output:

```json
{
    "$format": "jgd/v1",
    "version": "1.0.0",
    "seed": 42,
    "root": {
        "fields": {
            "random_number": {
                "number": {
                    "min": 1,
                    "max": 100,
                    "integer": true
                }
            }
        }
    }
}
```

## Examples from the Repository

### Single Object Example

```json
{
    "$format": "jgd/v1",
    "version": "1.2",
    "root": {
        "fields": {
            "id": "${ulid}",
            "name": "${name.name}",
            "email": "${internet.safeEmail}",
            "city": "${address.cityName}",
            "display": "${name.lastName}, ${name.firstName}"
        }
    }
}
```

### Array of Objects Example

```json
{
    "$format": "jgd/v1",
    "version": "1.1",
    "root": {
        "count": 10,
        "fields": {
            "id": "${uuid.v4}",
            "title": "${lorem.sentence(3, 6)}"
        }
    }
}
```

### Multi-Entity with Cross-References

```json
{
    "$format": "jgd/v1",
    "version": "1.2",
    "entities": {
        "users": {
            "count": 3,
            "fields": {
                "id": "${ulid}",
                "name": "${name.name}",
                "email": "${internet.safeEmail}"
            }
        },
        "posts": {
            "count": 10,
            "fields": {
                "id": "${uuid.v4}",
                "userId": { "ref": "users.id" },
                "title": "${lorem.sentence(3,7)}",
                "content": "${lorem.paragraphs(2,4)}",
                "createdAt": "${chrono.dateTimeBetween(2021-01-01T00:00:00Z,2022-12-31T23:59:59Z)}",
                "tags": {
                    "array": {
                        "count": [1, 5],
                        "of": "${lorem.word}"
                    }
                }
            }
        }
    }
}
```

## Integration with Routing

Combine JGD with routing patterns:

```
mocks/
├── users/
│   ├── get.jgd            # GET /users
│   └── get{id}.jgd        # GET /users/123
├── products/
│   ├── get.jgd            # GET /products
│   └── get{id}.jgd        # GET /products/456
└── jgd-examples/
    ├── single-object-root.jgd
    ├── array-object-root.jgd
    └── user-post-entities.jgd
```

## Hot Reload Support

JGD files support hot reload:

1. Edit your `.jgd` file
2. Save changes
3. Server automatically reloads
4. Next request returns newly generated data

## Documentation Reference

For complete JGD documentation and API reference, see:

-   [JGD-rs Library Documentation](https://github.com/lvendrame/jgd-rs/blob/main/jgd-rs/README.md)
-   [JGD Schema Definition](https://raw.githubusercontent.com/lvendrame/jgd-rs/refs/heads/main/jgd-rs/schema/jgd.schema.json)

## Next Steps

-   Learn about [REST APIs](rest-apis.md) to combine with dynamic generation
-   Explore [Authentication](authentication.md) to protect JGD endpoints
-   Try [Basic Routing](basic-routing.md) for organizing JGD files
-   See [Static Files](static-files.md) for serving static assets alongside dynamic data
