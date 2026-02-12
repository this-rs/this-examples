use this::prelude::*;

impl_data_entity_validated!(
    Warehouse,
    "warehouse",
    ["name"],
    {
        location: Option<String>,
        store_id: Uuid,
    },
    validate: {
        create: {
            name: [required string_length(2, 100)],
            status: [required in_list("active", "inactive", "closed")],
        },
        update: {
            name: [optional string_length(2, 100)],
            status: [optional in_list("active", "inactive", "closed")],
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
