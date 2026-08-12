# Collection Loading

rs-mock-server can initialize Fosk collections from standalone JSON, JGD, CSV, or Parquet files
at startup. This is useful when SQL, GraphQL, schema validation, or other
runtime features need shared data that is not tied to one REST route file.

## Startup Folder

By default, collections are loaded from:

```text
mocks/{collections}
```

Each `.json`, `.jgd`, `.csv`, or `.parquet` file in that folder is loaded as one Fosk collection. The
collection name comes from the file stem:

| File                                      | Collection          |
| ----------------------------------------- | ------------------- |
| `mocks/{collections}/warehouse.json`      | `warehouse`         |
| `mocks/{collections}/warehouse_assets.jgd`| `warehouse_assets`  |
| `mocks/{collections}/people.csv`           | `people`            |
| `mocks/{collections}/events.parquet`       | `events`            |

`{collections}` is reserved for collection loading and is not exposed as normal
mock routes.

## JSON Format

JSON collection files use the same array format accepted by REST route files:

```json
[
  {
    "id": "wh-lisbon",
    "name": "Lisbon Fulfillment Hub",
    "capacity": 24000
  },
  {
    "id": "wh-porto",
    "name": "Porto Returns Center",
    "capacity": 8500
  }
]
```

Save that file as `mocks/{collections}/warehouse_locations.json` to load the
`warehouse_locations` collection.

## JGD Format

JGD collection files use the same generated data format accepted by `rest.jgd`:

```json
{
  "$format": "jgd/v1",
  "version": "1.1",
  "root": {
    "count": 6,
    "fields": {
      "id": "${uuid.v4}",
      "warehouse_id": "${number.numberWithFormat(WH-####)}",
      "status": "${lorem.word}"
    }
  }
}
```

Save that file as `mocks/{collections}/warehouse_assets.jgd` to generate and
load the `warehouse_assets` collection.

## CSV and Parquet Formats

CSV files use the first row as headers and Fosk's default scalar type inference.
Parquet files use their embedded schema. Both formats otherwise use Fosk's
default import options and replace the destination collection at startup.

## Configuration

Override the collection folder in `rs-mock-server.toml`:

```toml
[server]
folder = "./mocks"

[collections]
folder = "{collections}"
```

Relative collection folders are resolved under `[server].folder`. Absolute
collection folders are used as provided.

## Load Order

Route files are discovered first, collection files are loaded next, and schema
files are loaded after collections. Keep collection file names distinct from
REST route collection names when you want to avoid replacing route-seeded data.
