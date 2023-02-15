fn main() {
    let raw = "\
    "
    let out = String::from_utf8(output.stdout).map_err(|_|PARSE_FAILED)?;
    let out: Value = serde_json::from_str(&out).map_err(|_|PARSE_FAILED)?;
    let out = out.as_object().ok_or(PARSE_FAILED)?;
    let sender_bandwidth = out.get("sum_sent").ok_or(PARSE_FAILED)?
        .as_object().ok_or(PARSE_FAILED)?.get("bits_per_second").ok_or(PARSE_FAILED)?
        .as_f64().ok_or(PARSE_FAILED)?;
    let receiver_bandwidth = out.get("sum_received").ok_or(PARSE_FAILED)?
        .as_object().ok_or(PARSE_FAILED)?.get("bits_per_second").ok_or(PARSE_FAILED)?
        .as_f64().ok_or(PARSE_FAILED)?;
}