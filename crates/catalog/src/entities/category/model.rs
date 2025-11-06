use this::prelude::*;

impl_data_entity_validated!(
    Category,
    "category",
    ["name", "slug"],
    {
        slug: String,
        description: Option<String>,
    },
    validate: {
        create: {
            name: [required string_length(2, 100)],
            slug: [required string_length(2, 100)],
            status: [required in_list("active", "inactive")],
        },
        update: {
            name: [optional string_length(2, 100)],
            slug: [optional string_length(2, 100)],
            status: [optional in_list("active", "inactive")],
        },
    },
    filters: {
        create: {
            slug: [trim lowercase],
            status: [trim lowercase],
        },
        update: {
            slug: [trim lowercase],
            status: [trim lowercase],
        },
    }
);

