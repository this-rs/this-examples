use this::prelude::*;

impl_data_entity_validated!(
    Tag,
    "tag",
    ["name"],
    {
        color: Option<String>,
        description: Option<String>,
    },
    validate: {
        create: {
            name: [required string_length(2, 50)],
        },
        update: {
            name: [optional string_length(2, 50)],
        },
    },
    filters: {
        create: {
            name: [trim lowercase],
            color: [trim uppercase],
        },
        update: {
            name: [trim lowercase],
            color: [trim uppercase],
        },
    }
);

