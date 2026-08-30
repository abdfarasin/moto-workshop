use rusqlite::{Connection, OptionalExtension};

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
