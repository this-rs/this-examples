//! Icônes inline (24×24, trait 1.7). Injectées en `inner_html` : aucune dépendance,
//! aucune requête, et elles suivent `currentColor`.

macro_rules! icon {
    ($name:ident, $body:expr) => {
        pub const $name: &str = concat!(
            r#"<svg viewBox="0 0 24 24" width="18" height="18" fill="none" stroke="currentColor" stroke-width="1.7" stroke-linecap="round" stroke-linejoin="round" aria-hidden="true" focusable="false">"#,
            $body,
            "</svg>"
        );
    };
}

icon!(
    DASHBOARD,
    r#"<rect x="3" y="3" width="7" height="7" rx="2"/><rect x="14" y="3" width="7" height="7" rx="2"/><rect x="3" y="14" width="7" height="7" rx="2"/><rect x="14" y="14" width="7" height="7" rx="2"/>"#
);
icon!(
    ORDERS,
    r#"<path d="M6 3h9l4 4v14a1 1 0 0 1-1 1H6a1 1 0 0 1-1-1V4a1 1 0 0 1 1-1Z"/><path d="M9 8h6M9 12h6M9 16h3"/>"#
);
icon!(
    CUSTOMERS,
    r#"<path d="M16 20v-1.5c0-2-1.8-3.5-4-3.5H7c-2.2 0-4 1.5-4 3.5V20"/><circle cx="9.5" cy="8" r="3.2"/><path d="M17 11.2a3 3 0 1 0 0-6"/><path d="M21 20v-1.3c0-1.7-1.3-3-3-3.4"/>"#
);
icon!(
    PRODUCTS,
    r#"<path d="M12 3 20 7v10l-8 4-8-4V7l8-4Z"/><path d="m4 7 8 4 8-4M12 11v10"/>"#
);
icon!(
    LINES,
    r#"<path d="M8 6h13M8 12h13M8 18h13"/><circle cx="3.5" cy="6" r="1.2"/><circle cx="3.5" cy="12" r="1.2"/><circle cx="3.5" cy="18" r="1.2"/>"#
);
icon!(PLUS, r#"<path d="M12 5v14M5 12h14"/>"#);
icon!(
    SEARCH,
    r#"<circle cx="11" cy="11" r="7"/><path d="m20 20-3.6-3.6"/>"#
);
icon!(CLOSE, r#"<path d="M6 6l12 12M18 6 6 18"/>"#);
icon!(
    TRASH,
    r#"<path d="M4 7h16M9 7V5a1 1 0 0 1 1-1h4a1 1 0 0 1 1 1v2M6 7l1 13a1 1 0 0 0 1 1h8a1 1 0 0 0 1-1l1-13"/>"#
);
icon!(
    CART,
    r#"<circle cx="9" cy="20" r="1.6"/><circle cx="18" cy="20" r="1.6"/><path d="M2 3h2.5l2.2 11.2a2 2 0 0 0 2 1.6h8.6a2 2 0 0 0 2-1.6L21 7H6"/>"#
);
icon!(
    MODULE,
    r#"<path d="M12 2 4 6.5v11L12 22l8-4.5v-11L12 2Z"/><path d="M8 12h8"/>"#
);
icon!(CHECK, r#"<path d="m4.5 12.5 5 5 10-11"/>"#);
icon!(ARROW, r#"<path d="M9 6l6 6-6 6"/>"#);
icon!(
    GEAR,
    r#"<circle cx="12" cy="12" r="3"/><path d="M19.4 15a1.7 1.7 0 0 0 .3 1.9l.1.1a2 2 0 1 1-2.8 2.8l-.1-.1a1.7 1.7 0 0 0-1.9-.3 1.7 1.7 0 0 0-1 1.5V21a2 2 0 1 1-4 0v-.1a1.7 1.7 0 0 0-1.1-1.5 1.7 1.7 0 0 0-1.9.3l-.1.1a2 2 0 1 1-2.8-2.8l.1-.1a1.7 1.7 0 0 0 .3-1.9 1.7 1.7 0 0 0-1.5-1H3a2 2 0 1 1 0-4h.1a1.7 1.7 0 0 0 1.5-1.1 1.7 1.7 0 0 0-.3-1.9l-.1-.1a2 2 0 1 1 2.8-2.8l.1.1a1.7 1.7 0 0 0 1.9.3H9a1.7 1.7 0 0 0 1-1.5V3a2 2 0 1 1 4 0v.1a1.7 1.7 0 0 0 1 1.5 1.7 1.7 0 0 0 1.9-.3l.1-.1a2 2 0 1 1 2.8 2.8l-.1.1a1.7 1.7 0 0 0-.3 1.9V9a1.7 1.7 0 0 0 1.5 1H21a2 2 0 1 1 0 4h-.1a1.7 1.7 0 0 0-1.5 1Z"/>"#
);
