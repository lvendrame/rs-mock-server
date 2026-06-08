# SQL Routes

Use `.sql` files to create GET endpoints that execute SQL queries and return the results.

## Overview

When the server detects a file with a `.sql` extension in the `mocks` directory, it automatically creates a `GET` endpoint matching the file's path. The SQL in the file is executed against the Fosk in-memory database, and the results are returned as JSON.

## Basic SQL Route

Place a file named `companies.sql` under `mocks/reports`:

```
mocks/
└── reports/
    └── companies.sql      # contains: select * from companies;
```

This creates:

-   **GET** `/reports/companies`
    Executes the SQL and returns all companies.

## Parameterized SQL Routes

You can include parameters in the filename (e.g., `{id}`, `{start-end}`, `{value}`) to create dynamic routes. Parameters are bound to `?` placeholders in the SQL:

```
mocks/
└── reports/
    ├── companies{id}.sql  # select * from companies where id = ?
    └── sales{2020-2022}.sql  # select * from sales where year > ?
```

Generates:

-   **GET** `/reports/companies/{id}`
    Binds `id` to the SQL placeholder.
-   **GET** `/reports/sales/{2020}` to `/reports/sales/{2022}`
    Binds param value to the SQL placeholder.

## Report Queries

Fosk supports more than simple selects. SQL route files can use joins,
aggregates, grouping, `HAVING`, ordering, limits, offsets, and aliases.

Example:

```sql
select
  c.name as customer,
  count(distinct o.id) as orders,
  count(*) as lines,
  sum(oi.quantity) as units
from warehouse_customers c
join warehouse_orders o on o.customer_id = c.id
join warehouse_order_items oi on oi.order_id = o.id
group by c.name
having sum(oi.quantity) > 20
order by units desc, customer asc
limit 10
```

See `mocks/warehouse-reports` for executable examples covering joins,
`group by`, `having`, `order by`, `limit`, aggregate functions, and
parameterized detail reports.

## Internal Collections

SQL routes share the same in-memory database as REST APIs and startup collection
files. Available collections correspond to REST route names and files loaded
from `{collections}`.

### Listing Collections

-   **GET** `/mock-server/collections`
    Returns a JSON object mapping collection names to their schema definitions.

Example:

```json
{
    "products": {
        "id": { "type": "Int", "nullable": false }
        // ...
    },
    "companies": {
        "id": { "type": "String", "nullable": false }
        // ...
    }
}
```

### Collection Schema

-   **GET** `/mock-server/collections/{collection-name}`
    Returns the schema for the specified collection.

Example:

```json
{
    "id": { "type": "String", "nullable": false },
    "name": { "type": "String", "nullable": false }
    // ...
}
```

## Fosk In-Memory Database

Under the hood, SQL routes use the [Fosk](https://github.com/lvendrame/fosk)
crate for in-memory data storage and query execution. Fosk 0.1.15 supports
`SELECT`, `WHERE`, `GROUP BY`, `HAVING`, inner/left/right/full joins,
`ORDER BY`, `LIMIT`, `OFFSET`, positional `?` parameters, `IN (?)`, and
aggregates such as `COUNT`, `COUNT(DISTINCT ...)`, `SUM`, `AVG`, `MIN`, and
`MAX`.
