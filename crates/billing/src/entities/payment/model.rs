use this::prelude::*;

impl_data_entity_validated!(
    Payment,
    "payment",
    ["name", "number"],
    {
        number: String,
        amount: f64,
        method: String,
        transaction_id: Option<String>,
    },
    validate: {
        create: {
            number: [required string_length(3, 50)],
            amount: [required positive max_value(2_000_000.0)],
            method: [required in_list("credit_card", "bank_transfer", "cash")],
            status: [required in_list("pending", "completed", "failed")],
        },
        update: {
            amount: [optional positive max_value(2_000_000.0)],
            method: [optional in_list("credit_card", "bank_transfer", "cash")],
            status: [optional in_list("pending", "completed", "failed")],
        },
    },
    filters: {
        create: {
            number: [trim uppercase],
            method: [trim lowercase],
            status: [trim lowercase],
            amount: [round_decimals(2)],
        },
        update: {
            method: [trim lowercase],
            status: [trim lowercase],
            amount: [round_decimals(2)],
        },
    }
);
