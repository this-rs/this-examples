#![allow(clippy::too_many_arguments)]

use this::prelude::*;

impl_data_entity_validated!(
    Usage,
    "usage",
    ["name"],
    {
        activity_id: Uuid,
        usage_type: String,
        quantity: f64,
        unit: Option<String>,
        from_activity_id: Option<Uuid>,
        date: Option<String>,
    },
    validate: {
        create: {
            usage_type: [required in_list("espace_utilise", "consommation", "service")],
            quantity: [required positive],
            status: [required in_list("pending", "recorded", "billed")],
        },
        update: {
            usage_type: [optional in_list("espace_utilise", "consommation", "service")],
            quantity: [optional positive],
            status: [optional in_list("pending", "recorded", "billed")],
        },
    },
    filters: {
        create: {
            usage_type: [trim lowercase],
            status: [trim lowercase],
        },
        update: {
            usage_type: [trim lowercase],
            status: [trim lowercase],
        },
    }
);
