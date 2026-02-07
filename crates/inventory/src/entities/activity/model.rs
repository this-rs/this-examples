use this::prelude::*;

impl_data_entity_validated!(
    Activity,
    "activity",
    ["name"],
    {
        activity_type: Option<String>,
        description: Option<String>,
    },
    validate: {
        create: {
            name: [required string_length(2, 100)],
            status: [required in_list("active", "inactive")],
        },
        update: {
            name: [optional string_length(2, 100)],
            status: [optional in_list("active", "inactive")],
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





