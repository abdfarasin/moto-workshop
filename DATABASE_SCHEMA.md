# Database Schema

This is the living reference for the latest supported SQLite schema. It reflects the current schema version 5 working tree and must be updated with every persistence change.

## Migration history

| Version | Responsibility |
| --- | --- |
| 1 | Create `customers`. |
| 2 | Create Motorcycle catalogs and `motorcycles`; seed make and color catalogs. |
| 3 | Add Customer INSERT/UPDATE validation triggers and validate legacy Customer rows. |
| 4 | Rebuild `motorcycles` to add canonical chassis/frame identity support. |
| 5 | Create `service_visits`, skeletal `invoices`, lifecycle/history triggers, and automatic draft invoices. |

The migration runner rejects databases whose `PRAGMA user_version` is greater than 5. Historical migrations 1–4 are immutable.

## Entity relationships

```mermaid
erDiagram
    CUSTOMERS ||--o{ MOTORCYCLES : owns
    MOTORCYCLE_MAKES ||--o{ MOTORCYCLES : classifies
    MOTORCYCLE_COLORS ||--o{ MOTORCYCLES : classifies
    JORDAN_PLATE_CODES ||--o{ MOTORCYCLES : identifies
    MOTORCYCLES ||--o{ SERVICE_VISITS : receives
    CUSTOMERS ||--o{ SERVICE_VISITS : owner_snapshot
    SERVICE_VISITS ||--|| INVOICES : creates

    CUSTOMERS {
        INTEGER id PK
        TEXT name
        TEXT phone UK
        TEXT notes NULL
        INTEGER created_at
        INTEGER updated_at
        INTEGER archived_at NULL
    }

    MOTORCYCLES {
        INTEGER id PK
        INTEGER customer_id FK
        INTEGER make_id FK
        TEXT model
        INTEGER year NULL
        INTEGER plate_code_id FK_NULL
        INTEGER plate_number NULL
        TEXT vin UK_NULL
        TEXT chassis_number UK_NULL
        INTEGER color_id FK
        TEXT notes NULL
        INTEGER created_at
        INTEGER updated_at
        INTEGER archived_at NULL
    }

    MOTORCYCLE_MAKES {
        INTEGER id PK
        TEXT name UK
        INTEGER active
    }

    MOTORCYCLE_COLORS {
        INTEGER id PK
        TEXT name UK
        INTEGER active
    }

    JORDAN_PLATE_CODES {
        INTEGER id PK
        TEXT code UK
        INTEGER active
    }

    SERVICE_VISITS {
        INTEGER id PK
        INTEGER motorcycle_id FK
        INTEGER owner_customer_id FK
        TEXT status
        INTEGER opened_at
        INTEGER completed_at NULL
        INTEGER closed_at NULL
        INTEGER cancelled_at NULL
        INTEGER odometer_km NULL
        TEXT customer_complaint
        TEXT diagnosis NULL
        TEXT work_performed NULL
        INTEGER labor_charge_fils
        TEXT cancellation_reason NULL
        TEXT notes NULL
        INTEGER created_at
        INTEGER updated_at
    }

    INVOICES {
        INTEGER id PK
        INTEGER service_visit_id FK_UK
        TEXT status
        TEXT invoice_number UK_NULL
        INTEGER issued_at NULL
        INTEGER cancelled_at NULL
        TEXT notes NULL
        INTEGER created_at
        INTEGER updated_at
    }
```

`UK_NULL` means unique when a non-NULL value is present. Multiple NULL values are allowed by SQLite.

## `customers`

| Column | SQLite declaration | Rules |
| --- | --- | --- |
| `id` | `INTEGER PRIMARY KEY AUTOINCREMENT` | Internal identity. |
| `name` | `TEXT NOT NULL` | Trimmed using SQLite `trim()` semantics; length 1–100. |
| `phone` | `TEXT NOT NULL UNIQUE` | Canonical `+962` followed by exactly nine ASCII digits. |
| `notes` | `TEXT NULL` | NULL or text up to 2,000 characters. |
| `created_at` | `INTEGER NOT NULL` | Application-supplied timestamp. |
| `updated_at` | `INTEGER NOT NULL` | Application-supplied timestamp. |
| `archived_at` | `INTEGER NULL` | Optional archive timestamp. |

Migration 3 supplies matching `BEFORE INSERT` and `BEFORE UPDATE` validation triggers. It also rejects migration when pre-v3 Customer rows violate these persistent rules. Phone uniqueness comes from the table constraint.

## Motorcycle catalogs

### `motorcycle_makes`

- `id INTEGER PRIMARY KEY AUTOINCREMENT`
- `name TEXT NOT NULL COLLATE NOCASE UNIQUE`, trimmed and 1–80 characters
- `active INTEGER NOT NULL DEFAULT 1`, restricted to `0` or `1`
- Migration 2 seeds the current general manufacturer list.

### `motorcycle_colors`

- `id INTEGER PRIMARY KEY AUTOINCREMENT`
- `name TEXT NOT NULL COLLATE NOCASE UNIQUE`, trimmed and 1–40 characters
- `active INTEGER NOT NULL DEFAULT 1`, restricted to `0` or `1`
- Migration 2 seeds the current general color list.

### `jordan_plate_codes`

- `id INTEGER PRIMARY KEY AUTOINCREMENT`
- `code TEXT NOT NULL COLLATE NOCASE UNIQUE`, trimmed and 1–20 characters
- `active INTEGER NOT NULL DEFAULT 1`, restricted to `0` or `1`
- No official Jordanian plate-code data is seeded yet.

## `motorcycles`

| Column | SQLite declaration | Rules |
| --- | --- | --- |
| `id` | `INTEGER PRIMARY KEY AUTOINCREMENT` | Internal identity. |
| `customer_id` | `INTEGER NOT NULL` | FK to `customers(id)`, `ON DELETE RESTRICT`. |
| `make_id` | `INTEGER NOT NULL` | FK to `motorcycle_makes(id)`, `ON DELETE RESTRICT`. |
| `model` | `TEXT NOT NULL` | SQLite text, trimmed, length 1–80. |
| `year` | `INTEGER NULL` | NULL or integer at least 1885. Domain adds a current-year upper bound. |
| `plate_code_id` | `INTEGER NULL` | FK to `jordan_plate_codes(id)`, `ON DELETE RESTRICT`. |
| `plate_number` | `INTEGER NULL` | Integer 1–99,999. Must be present exactly when `plate_code_id` is present. |
| `vin` | `TEXT NULL UNIQUE` | Exactly 17 uppercase ASCII alphanumeric characters; `I`, `O`, and `Q` forbidden. |
| `chassis_number` | `TEXT NULL UNIQUE` | Length 1–64; uppercase ASCII `A-Z`, digits, `-`, `/`, or `.` only. |
| `color_id` | `INTEGER NOT NULL` | FK to `motorcycle_colors(id)`, `ON DELETE RESTRICT`. |
| `notes` | `TEXT NULL` | NULL or text up to 2,000 characters. |
| `created_at` | `INTEGER NOT NULL` | Application-supplied timestamp. |
| `updated_at` | `INTEGER NOT NULL` | Application-supplied timestamp. |
| `archived_at` | `INTEGER NULL` | Optional archive timestamp. |

Identity requires at least one of a complete plate, strict VIN, or chassis number. The database also enforces unique plate-code/number pairs.

Indexes:

- `idx_motorcycles_customer_id` on `customer_id`
- `idx_motorcycles_make_id` on `make_id`
- SQLite-generated unique indexes for VIN, chassis number, and plate pair

## `service_visits`

A Service Visit is one historical occasion when one Motorcycle enters the workshop. `owner_customer_id` snapshots the owner at opening time and is not rewritten after later Motorcycle ownership changes.

| Column | SQLite declaration | Rules |
| --- | --- | --- |
| `id` | `INTEGER PRIMARY KEY AUTOINCREMENT` | Internal visit identity. |
| `motorcycle_id` | `INTEGER NOT NULL` | FK to `motorcycles(id)`, `ON DELETE RESTRICT`; immutable after insert. |
| `owner_customer_id` | `INTEGER NOT NULL` | FK to `customers(id)`, `ON DELETE RESTRICT`; must match the Motorcycle owner at insert and is then immutable. |
| `status` | `TEXT NOT NULL` | Exactly `OPEN`, `READY_FOR_PICKUP`, `CLOSED`, or `CANCELLED`. |
| `opened_at` | `INTEGER NOT NULL` | Nonnegative caller-supplied UTC timestamp; immutable after insert. |
| `completed_at` | `INTEGER NULL` | Required for READY/CLOSED; at least `opened_at`. |
| `closed_at` | `INTEGER NULL` | Required only for CLOSED; at least `completed_at`. |
| `cancelled_at` | `INTEGER NULL` | Required only for CANCELLED; at least `opened_at`. |
| `odometer_km` | `INTEGER NULL` | Integer 0–9,999,999; no monotonic-history rule. |
| `customer_complaint` | `TEXT NOT NULL` | SQLite-trimmed canonical text, length 1–4,000. |
| `diagnosis` | `TEXT NULL` | NULL or trimmed text, length 1–4,000. |
| `work_performed` | `TEXT NULL` | NULL or trimmed text, length 1–4,000; required for READY/CLOSED. |
| `labor_charge_fils` | `INTEGER NOT NULL DEFAULT 0` | Nonnegative integer fils; never SQLite REAL. |
| `cancellation_reason` | `TEXT NULL` | Required only for CANCELLED; trimmed length 1–1,000. |
| `notes` | `TEXT NULL` | NULL or trimmed text, length 1–4,000. |
| `created_at` | `INTEGER NOT NULL` | Caller-supplied timestamp. |
| `updated_at` | `INTEGER NOT NULL` | Caller-supplied timestamp. |

Lifecycle transitions enforced by trigger:

```text
OPEN -> READY_FOR_PICKUP -> CLOSED
  |             |
  +-> CANCELLED +-> OPEN
```

`CLOSED` and `CANCELLED` are terminal and cannot be updated. Hard deletion is prohibited. A partial unique index permits at most one `OPEN` or `READY_FOR_PICKUP` visit per Motorcycle, while a normal `motorcycle_id` index supports complete history queries.

## `invoices`

| Column | SQLite declaration | Rules |
| --- | --- | --- |
| `id` | `INTEGER PRIMARY KEY AUTOINCREMENT` | Internal invoice identity; not a customer-facing invoice number. |
| `service_visit_id` | `INTEGER NOT NULL UNIQUE` | FK to `service_visits(id)`, `ON DELETE RESTRICT`; immutable. |
| `status` | `TEXT NOT NULL DEFAULT 'DRAFT'` | Reserved values: `DRAFT`, `ISSUED`, `CANCELLED`. Workflow is not implemented yet. |
| `invoice_number` | `TEXT NULL UNIQUE` | NULL for drafts; number format deferred. |
| `issued_at` | `INTEGER NULL` | Reserved for later issuance workflow. |
| `cancelled_at` | `INTEGER NULL` | Reserved for later cancellation workflow. |
| `notes` | `TEXT NULL` | Reserved skeletal invoice notes. |
| `created_at` | `INTEGER NOT NULL` | Copied from the Service Visit creation timestamp. |
| `updated_at` | `INTEGER NOT NULL` | Initially copied from the Service Visit creation timestamp. |

An `AFTER INSERT` Service Visit trigger creates exactly one draft Invoice atomically. Uniqueness prevents a second Invoice; focused triggers prevent changing `service_visit_id` or deleting the Invoice.

## Deferred schema

There are currently no inventory items, stock movements, or structured parts-usage records. `ServiceVisitPart`, invoice issuance/numbering, payments, and financial snapshot totals remain deferred and must be added through later migrations rather than editing versions 1–5.
