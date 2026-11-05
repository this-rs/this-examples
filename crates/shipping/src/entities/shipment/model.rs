use this::prelude::*;

impl_data_entity_validated!(
    Shipment,
    "shipment",
    ["name", "number"],
    {
        number: String,
        carrier: String,
        tracking: Option<String>,
    },
    validate: {
        create: {
            number: [required string_length(3, 50)],
            carrier: [required string_length(2, 50)],
            status: [required in_list("preparing", "shipped", "delivered")],
        },
        update: {
            carrier: [optional string_length(2, 50)],
            status: [optional in_list("preparing", "shipped", "delivered")],
        },
    },
    filters: {
        create: {
            number: [trim uppercase],
            status: [trim lowercase],
        },
        update: {
            status: [trim lowercase],
        },
    }
);
