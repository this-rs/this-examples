use this::prelude::*;

impl_data_entity_validated!(
    Product,
    "product",
    ["name", "sku"],
    {
        sku: String,
        price: f64,
        stock_quantity: i32,
        description: Option<String>,
    },
    validate: {
        create: {
            sku: [required string_length(3, 50)],
            price: [required positive max_value(1_000_000.0)],
            status: [required in_list("active", "inactive", "discontinued")],
        },
        update: {
            price: [optional positive max_value(1_000_000.0)],
            status: [optional in_list("active", "inactive", "discontinued")],
        },
    },
    filters: {
        create: {
            sku: [trim uppercase],
            status: [trim lowercase],
            price: [round_decimals(2)],
        },
        update: {
            status: [trim lowercase],
            price: [round_decimals(2)],
        },
    }
);

