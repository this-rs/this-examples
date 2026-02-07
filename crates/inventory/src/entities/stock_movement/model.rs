#![allow(clippy::too_many_arguments)]

use this::prelude::*;

impl_data_entity_validated!(
    StockMovement,
    "stock_movement",
    ["name"],
    {
        stock_item_id: Uuid,
        movement_type: String,
        quantity: i32,
        reason: Option<String>,
        activity_id: Option<Uuid>,
    },
    validate: {
        create: {
            movement_type: [required in_list("in", "out", "transfer", "adjustment")],
            quantity: [required],
            status: [required in_list("pending", "completed", "cancelled")],
        },
        update: {
            movement_type: [optional in_list("in", "out", "transfer", "adjustment")],
            quantity: [optional],
            status: [optional in_list("pending", "completed", "cancelled")],
        },
    },
    filters: {
        create: {
            movement_type: [trim lowercase],
            status: [trim lowercase],
        },
        update: {
            movement_type: [trim lowercase],
            status: [trim lowercase],
        },
    }
);
