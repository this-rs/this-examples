#![allow(clippy::too_many_arguments)]

use this::prelude::*;

impl_data_entity_validated!(
    StockItem,
    "stock_item",
    ["name"],
    {
        product_id: Option<Uuid>,
        quantity: i32,
        warehouse_id: Uuid,
        reserved_quantity: Option<i32>,
    },
    validate: {
        create: {
            quantity: [required],
            status: [required in_list("available", "reserved", "out_of_stock")],
        },
        update: {
            quantity: [optional],
            status: [optional in_list("available", "reserved", "out_of_stock")],
        },
    },
    filters: {
        create: {
            status: [trim lowercase],
        },
        update: {
            status: [trim lowercase],
        },
    }
);
