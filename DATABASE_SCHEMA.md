# Database Schema

This is the living reference for the latest supported SQLite schema. It reflects the current schema version 9 working tree and must be updated with every persistence change.

## Migration history

| Version | Responsibility |
| --- | --- |
| 1 | Create `customers`. |
| 2 | Create Motorcycle catalogs and `motorcycles`; seed make and color catalogs. |
| 3 | Add Customer INSERT/UPDATE validation triggers and validate legacy Customer rows. |
| 4 | Rebuild `motorcycles` to add canonical chassis/frame identity support. |
| 5 | Create `service_visits`, skeletal `invoices`, lifecycle/history triggers, and automatic draft invoices. |
| 6 | Create reusable `inventory_units`, archivable `inventory_items`, and the immutable `stock_movements` ledger. |
| 7 | Create historical `service_visit_parts`; rebuild Stock Movements for automatic usage and reversal entries. |
| 8 | Rebuild `motorcycles` around one textual plate number, migrate complete legacy code/number plates, and remove the plate-code relationship. |
| 9 | Add immutable issued-Invoice identity, customer, Motorcycle, labor, Part-line, and integer-fils total snapshots. |

The migration runner rejects databases whose `PRAGMA user_version` is greater than 9. Historical migrations 1–8 are immutable.

## Entity relationships

```mermaid
erDiagram
    CUSTOMERS ||--o{ MOTORCYCLES : owns
    MOTORCYCLE_MAKES ||--o{ MOTORCYCLES : classifies
    MOTORCYCLE_COLORS ||--o{ MOTORCYCLES : classifies
    MOTORCYCLES ||--o{ SERVICE_VISITS : receives
    CUSTOMERS ||--o{ SERVICE_VISITS : owner_snapshot
    SERVICE_VISITS ||--|| INVOICES : creates
    INVOICES ||--o{ INVOICE_LINES : snapshots
    INVENTORY_UNITS ||--o{ INVENTORY_ITEMS : measures
    INVENTORY_ITEMS ||--o{ STOCK_MOVEMENTS : ledger
    SERVICE_VISITS ||--o{ SERVICE_VISIT_PARTS : uses
    INVENTORY_ITEMS ||--o{ SERVICE_VISIT_PARTS : snapshots
    SERVICE_VISIT_PARTS ||--o{ STOCK_MOVEMENTS : effects
    SERVICE_VISIT_PARTS ||--o| INVOICE_LINES : issued_snapshot

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
        TEXT plate_number UK_NULL
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
        TEXT customer_name NULL
        TEXT customer_phone NULL
        TEXT motorcycle_make_name NULL
        TEXT motorcycle_model NULL
        TEXT motorcycle_plate_number NULL
        TEXT motorcycle_vin NULL
        TEXT motorcycle_chassis_number NULL
        INTEGER labor_charge_fils NULL
        INTEGER parts_total_fils NULL
        INTEGER total_fils NULL
        INTEGER created_at
        INTEGER updated_at
    }

    INVOICE_LINES {
        INTEGER id PK
        INTEGER invoice_id FK
        INTEGER service_visit_part_id FK_UK
        TEXT item_name
        TEXT unit_name
        INTEGER quantity
        INTEGER quantity_scale
        INTEGER unit_price_fils
        INTEGER line_total_fils
        INTEGER created_at
    }

    INVENTORY_UNITS {
        INTEGER id PK
        TEXT name UK
        INTEGER quantity_scale
        INTEGER active
    }

    INVENTORY_ITEMS {
        INTEGER id PK
        TEXT name
        TEXT sku UK_NULL
        INTEGER unit_id FK
        INTEGER default_purchase_price_fils NULL
        INTEGER default_selling_price_fils
        INTEGER minimum_stock_quantity
        TEXT notes NULL
        INTEGER created_at
        INTEGER updated_at
        INTEGER archived_at NULL
    }

    STOCK_MOVEMENTS {
        INTEGER id PK
        INTEGER inventory_item_id FK
        INTEGER service_visit_part_id FK_NULL
        TEXT movement_type
        INTEGER quantity_delta
        TEXT notes NULL
        INTEGER created_at
    }

    SERVICE_VISIT_PARTS {
        INTEGER id PK
        INTEGER service_visit_id FK
        INTEGER inventory_item_id FK
        TEXT item_name
        TEXT unit_name
        INTEGER quantity
        INTEGER quantity_scale
        INTEGER unit_price_fils
        INTEGER line_total_fils
        TEXT status
        INTEGER voided_at NULL
        TEXT void_reason NULL
        INTEGER created_at
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
- No official Jordanian plate-code data is seeded.
- Schema v8 retains this table only as an unused historical catalog. Current application, command, repository, and UI behavior does not read or write it.

## `motorcycles`

| Column | SQLite declaration | Rules |
| --- | --- | --- |
| `id` | `INTEGER PRIMARY KEY AUTOINCREMENT` | Internal identity. |
| `customer_id` | `INTEGER NOT NULL` | FK to `customers(id)`, `ON DELETE RESTRICT`. |
| `make_id` | `INTEGER NOT NULL` | FK to `motorcycle_makes(id)`, `ON DELETE RESTRICT`. |
| `model` | `TEXT NOT NULL` | SQLite text, trimmed, length 1–80. |
| `year` | `INTEGER NULL` | NULL or integer at least 1885. Domain adds a current-year upper bound. |
| `plate_number` | `TEXT NULL UNIQUE` | New inserts require a trimmed, nonempty string containing only ASCII digits and `-`; dashes may appear only between digit groups. Migrated VIN/chassis-only rows may remain NULL. No arbitrary length limit is defined. |
| `vin` | `TEXT NULL UNIQUE` | Exactly 17 uppercase ASCII alphanumeric characters; `I`, `O`, and `Q` forbidden. |
| `chassis_number` | `TEXT NULL UNIQUE` | Length 1–64; uppercase ASCII `A-Z`, digits, `-`, `/`, or `.` only. |
| `color_id` | `INTEGER NOT NULL` | FK to `motorcycle_colors(id)`, `ON DELETE RESTRICT`. |
| `notes` | `TEXT NULL` | NULL or text up to 2,000 characters. |
| `created_at` | `INTEGER NOT NULL` | Application-supplied timestamp. |
| `updated_at` | `INTEGER NOT NULL` | Application-supplied timestamp. |
| `archived_at` | `INTEGER NULL` | Optional archive timestamp. |

New Motorcycle registration requires a unique plate number; VIN and chassis number are optional. The v8 table-level identity check still permits migrated legacy rows whose plate is NULL when VIN or chassis identity exists. The `validate_motorcycle_plate_insert_v8` and `validate_motorcycle_plate_update_v8` triggers reject NULL, blank, non-text, untrimmed, non-digit/dash, leading-dash, trailing-dash, and consecutive-dash values for new writes.

Migration 8 converts each complete legacy plate to `plate_code.code || '-' || old plate_number` while preserving Motorcycle IDs and dependent Service Visit foreign keys. Legacy VIN/chassis-only Motorcycles keep a NULL plate. The migration temporarily removes and then recreates `validate_service_visit_owner_v5` while replacing the table so ownership protection remains active.

Indexes:

- `idx_motorcycles_customer_id` on `customer_id`
- `idx_motorcycles_make_id` on `make_id`
- SQLite-generated unique indexes for plate number, VIN, and chassis number

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
| `status` | `TEXT NOT NULL DEFAULT 'DRAFT'` | `DRAFT`, `ISSUED`, or reserved `CANCELLED`. Only a DRAFT can be issued. |
| `invoice_number` | `TEXT NULL UNIQUE` | NULL for drafts; issuance sets deterministic `INV-######` from the internal ID. |
| `issued_at` | `INTEGER NULL` | Required for issued rows and not before Service Visit completion. |
| `cancelled_at` | `INTEGER NULL` | Reserved for later cancellation workflow. |
| `notes` | `TEXT NULL` | Reserved skeletal invoice notes. |
| `customer_name`, `customer_phone` | `TEXT NULL` | NULL in a draft; frozen current Customer identity at issuance. |
| `motorcycle_make_name`, `motorcycle_model` | `TEXT NULL` | NULL in a draft; frozen current presentation at issuance. |
| `motorcycle_plate_number`, `motorcycle_vin`, `motorcycle_chassis_number` | `TEXT NULL` | Optional frozen Motorcycle identity values. |
| `labor_charge_fils` | `INTEGER NULL` | Frozen nonnegative Service Visit labor at issuance. |
| `parts_total_fils` | `INTEGER NULL` | Frozen checked sum of ACTIVE issued Part line totals. |
| `total_fils` | `INTEGER NULL` | Exact checked `labor_charge_fils + parts_total_fils`. |
| `created_at` | `INTEGER NOT NULL` | Copied from the Service Visit creation timestamp. |
| `updated_at` | `INTEGER NOT NULL` | Initially copied from the Service Visit creation timestamp. |

An `AFTER INSERT` Service Visit trigger creates exactly one draft Invoice atomically. A draft is a live projection of current persisted Customer, Motorcycle, labor, and ACTIVE Parts. Issuance is a single immediate transaction: Rust validates the lifecycle and checked totals, copies ACTIVE Part snapshots into `invoice_lines`, and freezes Invoice identity and totals. VOIDED Parts are excluded. Database triggers reject incomplete totals, snapshot mutation, line mutation/deletion, Invoice deletion, and changed Service Visit identity.

## `invoice_lines`

Each row is an immutable issuance-time copy of one ACTIVE `service_visit_parts` row. `service_visit_part_id` is unique, all quantities/prices/totals are integer snapshots, and the line is inserted only while its Invoice is still DRAFT. The Invoice can move to ISSUED only when `parts_total_fils` exactly equals the sum of its line snapshots. Issued history therefore does not change when later catalog, Customer, Motorcycle, or Service Visit data changes.

## Inventory quantity and price representation

Inventory quantities and money are stored only as SQLite `INTEGER` values. Each reusable Inventory Unit defines a `quantity_scale` of `1`, `10`, `100`, or `1000`, meaning that many stored subunits equal one displayed unit. For example, a Piece uses scale `1`; a Liter uses scale `1000`, so stored quantity `3750` means `3.750` liters.

InventoryItem prices are integer fils per one displayed unit. Thus a selling price of `7000` on a Liter item means `7.000 JOD` per `1.000` liter. Prices and stored quantities are bounded at `1,000,000,000`; no SQLite REAL value is authoritative.

## `inventory_units`

| Column | SQLite declaration | Rules |
| --- | --- | --- |
| `id` | `INTEGER PRIMARY KEY AUTOINCREMENT` | Internal unit identity. |
| `name` | `TEXT NOT NULL COLLATE NOCASE UNIQUE` | SQLite-trimmed canonical name, length 1–40; case-insensitive uniqueness. |
| `quantity_scale` | `INTEGER NOT NULL` | Exactly `1`, `10`, `100`, or `1000`. |
| `active` | `INTEGER NOT NULL DEFAULT 1` | Exactly `0` or `1`; deactivation is the lifecycle mechanism. |

Migration 6 seeds only `Piece` with scale `1` and `Liter` with scale `1000`. Hard deletion is prohibited. Once an Inventory Unit is referenced by any Inventory Item, a focused trigger prevents changing its scale so historical quantities cannot be reinterpreted.

## `inventory_items`

| Column | SQLite declaration | Rules |
| --- | --- | --- |
| `id` | `INTEGER PRIMARY KEY AUTOINCREMENT` | Internal item identity. |
| `name` | `TEXT NOT NULL` | SQLite-trimmed canonical text, length 1–150; deliberately not unique. |
| `sku` | `TEXT NULL COLLATE NOCASE UNIQUE` | NULL or trimmed text, length 1–64; uniqueness includes archived items. |
| `unit_id` | `INTEGER NOT NULL` | FK to `inventory_units(id)`, `ON DELETE RESTRICT`. |
| `default_purchase_price_fils` | `INTEGER NULL` | NULL or integer `0..=1,000,000,000` fils per displayed unit. |
| `default_selling_price_fils` | `INTEGER NOT NULL` | Integer `0..=1,000,000,000` fils per displayed unit. |
| `minimum_stock_quantity` | `INTEGER NOT NULL DEFAULT 0` | Scaled integer warning threshold `0..=1,000,000,000`; never blocks outgoing stock. |
| `notes` | `TEXT NULL` | NULL or trimmed text, length 1–2,000. |
| `created_at` | `INTEGER NOT NULL` | Caller-supplied timestamp. |
| `updated_at` | `INTEGER NOT NULL` | Caller-supplied timestamp. |
| `archived_at` | `INTEGER NULL` | Optional archive timestamp; archived rows and SKUs remain historical. |

There is no `current_quantity`, `stock`, `low_stock`, or fractional-quantity flag. The Unit scale is the single precision definition. An Item's Unit can be corrected before its first Stock Movement, but a focused trigger prevents changing it after any ledger history exists. Hard deletion is prohibited; archiving preserves history.

## `stock_movements`

| Column | SQLite declaration | Rules |
| --- | --- | --- |
| `id` | `INTEGER PRIMARY KEY AUTOINCREMENT` | Internal movement identity. |
| `inventory_item_id` | `INTEGER NOT NULL` | FK to `inventory_items(id)`, `ON DELETE RESTRICT`. |
| `service_visit_part_id` | `INTEGER NULL` | NULL for manual movements; FK to `service_visit_parts(id)` for usage/reversal. |
| `movement_type` | `TEXT NOT NULL` | Manual types plus `SERVICE_USAGE` and `SERVICE_USAGE_REVERSAL`. |
| `quantity_delta` | `INTEGER NOT NULL` | Incoming: `1..=1,000,000,000`; outgoing: `-1,000,000,000..=-1`. |
| `notes` | `TEXT NULL` | NULL or trimmed text, length 1–2,000. |
| `created_at` | `INTEGER NOT NULL` | Nonnegative caller-supplied timestamp. |

Stock Movements are an immutable ledger: focused triggers reject every UPDATE and DELETE. Corrections use compensating movements, preserving both the mistaken entry and its correction. `idx_stock_movements_inventory_item_id` supports per-item aggregation.

Authoritative current stock is always derived:

```sql
SELECT COALESCE(SUM(quantity_delta), 0)
FROM stock_movements
WHERE inventory_item_id = ?;
```

No movements means zero. Negative stock is intentionally valid and records operational reality; neither minimum stock nor the current aggregate blocks outgoing adjustments.

## `service_visit_parts`

A ServiceVisitPart is an immutable historical part/material line. It references its Service Visit and Inventory Item while snapshotting the Item name, Unit name, Unit scale, positive scaled quantity, charged unit price, and calculated line total. Catalog renames, Unit renames, archiving, and default-price changes never rewrite these snapshots.

| Column | Rules |
| --- | --- |
| `service_visit_id` | Existing OPEN or READY_FOR_PICKUP Service Visit; CLOSED/CANCELLED reject add or void. |
| `inventory_item_id` | Existing, non-archived Inventory Item. |
| `item_name` | Must equal current Item name at insertion; trimmed length 1–150, then immutable. |
| `unit_name` | Must equal current Unit name; trimmed length 1–40, then immutable. |
| `quantity` | Integer `1..=1,000,000,000` stored subunits. |
| `quantity_scale` | Exactly 1, 10, 100, or 1000 and must match the current Unit at insertion. |
| `unit_price_fils` | Actual charged price per displayed Unit, integer `0..=1,000,000,000`; it may differ from the Item default. |
| `line_total_fils` | Calculated snapshot using integer half-up rounding per line. |
| `status` | `ACTIVE` or terminal `VOIDED`. New rows must be ACTIVE. |
| `voided_at` | Required for VOIDED and not before `created_at`. |
| `void_reason` | Optional; when present, canonical text length 1–1,000. |

The single line-total formula in Rust and SQLite is `(quantity * unit_price_fils + quantity_scale / 2) / quantity_scale`. This rounds to the nearest fil with exact halves upward, per line. No floating-point arithmetic is used.

Inserting a part atomically creates exactly one immutable `SERVICE_USAGE` movement for the same Item and part with `-quantity`. Voiding it atomically appends exactly one `SERVICE_USAGE_REVERSAL` with `+quantity`; the original usage remains. Partial unique indexes prevent duplicate usage or reversal. Direct linked movements must match their Part's Item, quantity, and status. Negative stock remains valid.

Business snapshots cannot be edited in place or deleted. Correction is void-and-replace: ACTIVE becomes VOIDED with an optional reason, its reversal restores stock, and a replacement Part creates a new usage entry.

Indexes support Service Visit parts display, Inventory Item usage history, and Item ledger aggregation.

## Deferred schema

Invoice cancellation actions, payments/settlement, discounts, taxes, printing/export, suppliers, purchasing workflows, stock valuation, and reporting remain deferred. No payment model exists in schema v9. They must be added through later migrations rather than editing versions 1–9.
