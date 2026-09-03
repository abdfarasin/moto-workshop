use rusqlite::{params, Connection, OptionalExtension};

use crate::domain::inventory::{InventoryItem, StockMovement, StockMovementType};

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct InventoryItemRow {
    pub id: i64,
    pub name: String,
    pub sku: Option<String>,
    pub unit_id: i64,
    pub unit_name: String,
    pub quantity_scale: i64,
    pub default_selling_price_fils: i64,
    pub current_quantity: i64,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct InventoryManagementItemRow {
    pub id: i64,
    pub name: String,
    pub sku: Option<String>,
    pub unit_id: i64,
    pub unit_name: String,
    pub quantity_scale: i64,
    pub default_purchase_price_fils: Option<i64>,
    pub default_selling_price_fils: i64,
    pub minimum_stock_quantity: i64,
    pub notes: Option<String>,
    pub current_quantity: i64,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct InventoryUnitRow {
    pub id: i64,
    pub name: String,
    pub quantity_scale: i64,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct StockMovementRow {
    pub id: i64,
    pub movement_type: StockMovementType,
    pub quantity_delta: i64,
    pub notes: Option<String>,
    pub service_visit_part_id: Option<i64>,
    pub created_at: i64,
}

pub(crate) struct InventoryRepository<'connection> {
    connection: &'connection Connection,
}

impl<'connection> InventoryRepository<'connection> {
    pub fn new(connection: &'connection Connection) -> Self {
        Self { connection }
    }

    pub fn list_usable(&self) -> rusqlite::Result<Vec<InventoryItemRow>> {
        let mut statement = self.connection.prepare(
            "SELECT i.id, i.name, i.sku, u.id, u.name, u.quantity_scale,
                    i.default_selling_price_fils,
                    COALESCE(SUM(m.quantity_delta), 0)
             FROM inventory_items i
             JOIN inventory_units u ON u.id = i.unit_id
             LEFT JOIN stock_movements m ON m.inventory_item_id = i.id
             WHERE i.archived_at IS NULL AND u.active = 1
             GROUP BY i.id, i.name, i.sku, u.id, u.name, u.quantity_scale,
                      i.default_selling_price_fils
             ORDER BY i.name COLLATE NOCASE, i.id",
        )?;
        let items = statement
            .query_map([], map_inventory_item)?
            .collect::<rusqlite::Result<_>>()?;
        Ok(items)
    }

    pub fn find_usable(&self, item_id: i64) -> rusqlite::Result<Option<InventoryItemRow>> {
        self.connection
            .query_row(
                "SELECT i.id, i.name, i.sku, u.id, u.name, u.quantity_scale,
                        i.default_selling_price_fils,
                        COALESCE(SUM(m.quantity_delta), 0)
                 FROM inventory_items i
                 JOIN inventory_units u ON u.id = i.unit_id
                 LEFT JOIN stock_movements m ON m.inventory_item_id = i.id
                 WHERE i.id = ?1 AND i.archived_at IS NULL AND u.active = 1
                 GROUP BY i.id, i.name, i.sku, u.id, u.name, u.quantity_scale,
                          i.default_selling_price_fils",
                [item_id],
                map_inventory_item,
            )
            .optional()
    }

    pub fn search(
        &self,
        query: &str,
        limit: i64,
    ) -> rusqlite::Result<Vec<InventoryManagementItemRow>> {
        let pattern = format!(
            "%{}%",
            query
                .replace('\\', "\\\\")
                .replace('%', "\\%")
                .replace('_', "\\_")
        );
        let mut statement = self.connection.prepare(
            "SELECT i.id,i.name,i.sku,u.id,u.name,u.quantity_scale,i.default_purchase_price_fils,
                    i.default_selling_price_fils,i.minimum_stock_quantity,i.notes,
                    COALESCE(SUM(m.quantity_delta),0)
             FROM inventory_items i JOIN inventory_units u ON u.id=i.unit_id
             LEFT JOIN stock_movements m ON m.inventory_item_id=i.id
             WHERE i.archived_at IS NULL AND u.active=1
               AND (?1='' OR i.name LIKE ?2 ESCAPE '\\' COLLATE NOCASE OR COALESCE(i.sku,'') LIKE ?2 ESCAPE '\\' COLLATE NOCASE)
             GROUP BY i.id,i.name,i.sku,u.id,u.name,u.quantity_scale,i.default_purchase_price_fils,
                      i.default_selling_price_fils,i.minimum_stock_quantity,i.notes
             ORDER BY i.name COLLATE NOCASE,i.id LIMIT ?3",
        )?;
        let rows = statement
            .query_map(params![query, pattern, limit], map_management_item)?
            .collect();
        rows
    }

    pub fn find_management_item(
        &self,
        item_id: i64,
    ) -> rusqlite::Result<Option<InventoryManagementItemRow>> {
        self.connection.query_row(
            "SELECT i.id,i.name,i.sku,u.id,u.name,u.quantity_scale,i.default_purchase_price_fils,
                    i.default_selling_price_fils,i.minimum_stock_quantity,i.notes,
                    COALESCE(SUM(m.quantity_delta),0)
             FROM inventory_items i JOIN inventory_units u ON u.id=i.unit_id
             LEFT JOIN stock_movements m ON m.inventory_item_id=i.id
             WHERE i.id=?1 AND i.archived_at IS NULL AND u.active=1
             GROUP BY i.id,i.name,i.sku,u.id,u.name,u.quantity_scale,i.default_purchase_price_fils,
                      i.default_selling_price_fils,i.minimum_stock_quantity,i.notes",
            [item_id],map_management_item).optional()
    }

    pub fn list_units(&self) -> rusqlite::Result<Vec<InventoryUnitRow>> {
        let mut statement=self.connection.prepare("SELECT id,name,quantity_scale FROM inventory_units WHERE active=1 ORDER BY name COLLATE NOCASE,id")?;
        let rows = statement
            .query_map([], |row| {
                Ok(InventoryUnitRow {
                    id: row.get(0)?,
                    name: row.get(1)?,
                    quantity_scale: row.get(2)?,
                })
            })?
            .collect();
        rows
    }

    pub fn active_unit_exists(&self, unit_id: i64) -> rusqlite::Result<bool> {
        self.connection.query_row(
            "SELECT EXISTS(SELECT 1 FROM inventory_units WHERE id=?1 AND active=1)",
            [unit_id],
            |row| row.get(0),
        )
    }

    pub fn list_movements(
        &self,
        item_id: i64,
        limit: i64,
    ) -> rusqlite::Result<Vec<StockMovementRow>> {
        let mut statement=self.connection.prepare("SELECT id,movement_type,quantity_delta,notes,service_visit_part_id,created_at FROM stock_movements WHERE inventory_item_id=?1 ORDER BY created_at DESC,id DESC LIMIT ?2")?;
        let rows = statement
            .query_map(params![item_id, limit], |row| {
                Ok(StockMovementRow {
                    id: row.get(0)?,
                    movement_type: parse_movement_type(row.get::<_, String>(1)?)?,
                    quantity_delta: row.get(2)?,
                    notes: row.get(3)?,
                    service_visit_part_id: row.get(4)?,
                    created_at: row.get(5)?,
                })
            })?
            .collect();
        rows
    }

    pub fn insert_item(&self, item: &InventoryItem, created_at: i64) -> rusqlite::Result<i64> {
        self.connection.execute("INSERT INTO inventory_items(name,sku,unit_id,default_purchase_price_fils,default_selling_price_fils,minimum_stock_quantity,notes,created_at,updated_at) VALUES (?1,?2,?3,?4,?5,?6,?7,?8,?8)",params![item.name(),item.sku(),item.unit_id(),item.default_purchase_price_fils(),item.default_selling_price_fils(),item.minimum_stock_quantity(),item.notes(),created_at])?;
        Ok(self.connection.last_insert_rowid())
    }

    pub fn update_item(
        &self,
        item_id: i64,
        item: &InventoryItem,
        updated_at: i64,
    ) -> rusqlite::Result<usize> {
        self.connection.execute("UPDATE inventory_items SET name=?1,sku=?2,default_purchase_price_fils=?3,default_selling_price_fils=?4,minimum_stock_quantity=?5,notes=?6,updated_at=?7 WHERE id=?8 AND archived_at IS NULL",params![item.name(),item.sku(),item.default_purchase_price_fils(),item.default_selling_price_fils(),item.minimum_stock_quantity(),item.notes(),updated_at,item_id])
    }

    pub fn insert_movement(&self, movement: &StockMovement) -> rusqlite::Result<i64> {
        self.connection.execute("INSERT INTO stock_movements(inventory_item_id,service_visit_part_id,movement_type,quantity_delta,notes,created_at) VALUES (?1,?2,?3,?4,?5,?6)",params![movement.inventory_item_id(),movement.service_visit_part_id(),movement_type_name(movement.movement_type()),movement.quantity_delta(),movement.notes(),movement.created_at()])?;
        Ok(self.connection.last_insert_rowid())
    }
}

fn map_inventory_item(row: &rusqlite::Row<'_>) -> rusqlite::Result<InventoryItemRow> {
    Ok(InventoryItemRow {
        id: row.get(0)?,
        name: row.get(1)?,
        sku: row.get(2)?,
        unit_id: row.get(3)?,
        unit_name: row.get(4)?,
        quantity_scale: row.get(5)?,
        default_selling_price_fils: row.get(6)?,
        current_quantity: row.get(7)?,
    })
}

fn map_management_item(row: &rusqlite::Row<'_>) -> rusqlite::Result<InventoryManagementItemRow> {
    Ok(InventoryManagementItemRow {
        id: row.get(0)?,
        name: row.get(1)?,
        sku: row.get(2)?,
        unit_id: row.get(3)?,
        unit_name: row.get(4)?,
        quantity_scale: row.get(5)?,
        default_purchase_price_fils: row.get(6)?,
        default_selling_price_fils: row.get(7)?,
        minimum_stock_quantity: row.get(8)?,
        notes: row.get(9)?,
        current_quantity: row.get(10)?,
    })
}

fn movement_type_name(value: StockMovementType) -> &'static str {
    match value {
        StockMovementType::OpeningStock => "OPENING_STOCK",
        StockMovementType::Purchase => "PURCHASE",
        StockMovementType::AdjustmentIn => "ADJUSTMENT_IN",
        StockMovementType::AdjustmentOut => "ADJUSTMENT_OUT",
        StockMovementType::ServiceUsage => "SERVICE_USAGE",
        StockMovementType::ServiceUsageReversal => "SERVICE_USAGE_REVERSAL",
    }
}
fn parse_movement_type(value: String) -> rusqlite::Result<StockMovementType> {
    match value.as_str() {
        "OPENING_STOCK" => Ok(StockMovementType::OpeningStock),
        "PURCHASE" => Ok(StockMovementType::Purchase),
        "ADJUSTMENT_IN" => Ok(StockMovementType::AdjustmentIn),
        "ADJUSTMENT_OUT" => Ok(StockMovementType::AdjustmentOut),
        "SERVICE_USAGE" => Ok(StockMovementType::ServiceUsage),
        "SERVICE_USAGE_REVERSAL" => Ok(StockMovementType::ServiceUsageReversal),
        _ => Err(rusqlite::Error::InvalidQuery),
    }
}
