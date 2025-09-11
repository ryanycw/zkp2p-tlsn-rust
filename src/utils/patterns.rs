use crate::domain::Provider;

const EMPTY_PATTERNS: &[(&str, &str)] = &[];

pub const WISE_FIELD_PATTERNS: &[(&str, &str)] = &[
    (r#""id":([0-9]+)"#, "paymentId"),
    (r#""state":"([^"]+)""#, "state"),
    (
        r#""state":"OUTGOING_PAYMENT_SENT","date":([0-9]+)"#,
        "timestamp",
    ),
    (r#""targetAmount":([0-9\.]+)"#, "targetAmount"),
    (r#""targetCurrency":"([^"]+)""#, "targetCurrency"),
    (r#""targetRecipientId":([0-9]+)"#, "targetRecipientId"),
];

pub const HOST_HEADER_PATTERN: &str = r"host: [^\r\n]+";

pub fn get_field_patterns(provider: &Provider) -> &'static [(&'static str, &'static str)] {
    match provider {
        Provider::Wise => WISE_FIELD_PATTERNS,
        Provider::PayPal => EMPTY_PATTERNS,
    }
}
