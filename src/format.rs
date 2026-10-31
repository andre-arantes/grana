//! Brazilian money and date conventions: `R$ 1.234,56` and `dd/mm/yyyy`.

use anyhow::{Result, bail};
use chrono::NaiveDate;

/// Parses `44`, `44.9`, `44.90` or `44,90` into centavos.
/// Thousands separators are rejected because `1.234` would be ambiguous.
pub fn parse_money(input: &str) -> Result<i64> {
    let s = input.trim().trim_start_matches("R$").trim();
    let (whole, frac, has_sep) = match s.find([',', '.']) {
        Some(i) => (&s[..i], &s[i + 1..], true),
        None => (s, "", false),
    };
    let all_digits = |p: &str| p.chars().all(|c| c.is_ascii_digit());
    let frac_ok = all_digits(frac) && frac.len() <= 2 && (!has_sep || !frac.is_empty());
    if whole.is_empty() || !all_digits(whole) || !frac_ok {
        bail!("invalid amount '{input}' (use e.g. 44,90 or 44.90)");
    }
    let whole: i64 = whole.parse()?;
    let frac: i64 = match frac.len() {
        0 => 0,
        1 => frac.parse::<i64>()? * 10,
        _ => frac.parse()?,
    };
    Ok(whole * 100 + frac)
}

pub fn format_money(cents: i64) -> String {
    let sign = if cents < 0 { "-" } else { "" };
    let abs = cents.unsigned_abs();
    let whole = (abs / 100).to_string();
    let mut grouped = String::new();
    for (i, c) in whole.chars().enumerate() {
        if i > 0 && (whole.len() - i) % 3 == 0 {
            grouped.push('.');
        }
        grouped.push(c);
    }
    format!("{sign}R$ {grouped},{:02}", abs % 100)
}

pub fn parse_date(input: &str) -> Result<NaiveDate> {
    NaiveDate::parse_from_str(input.trim(), "%d/%m/%Y")
        .map_err(|_| anyhow::anyhow!("invalid date '{input}' (use dd/mm/yyyy)"))
}

pub fn format_date(date: NaiveDate) -> String {
    date.format("%d/%m/%Y").to_string()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_money() {
        assert_eq!(parse_money("44").unwrap(), 4400);
        assert_eq!(parse_money("44.90").unwrap(), 4490);
        assert_eq!(parse_money("44,90").unwrap(), 4490);
        assert_eq!(parse_money("44,9").unwrap(), 4490);
        assert_eq!(parse_money("0,05").unwrap(), 5);
        assert_eq!(parse_money("R$ 12,50").unwrap(), 1250);
    }

    #[test]
    fn rejects_bad_money() {
        for bad in ["", "abc", "1.234,56", "4,999", "-5", ",50", "12,"] {
            assert!(parse_money(bad).is_err(), "accepted {bad:?}");
        }
    }

    #[test]
    fn formats_money() {
        assert_eq!(format_money(4490), "R$ 44,90");
        assert_eq!(format_money(5), "R$ 0,05");
        assert_eq!(format_money(123456), "R$ 1.234,56");
        assert_eq!(format_money(123456789), "R$ 1.234.567,89");
        assert_eq!(format_money(-4000), "-R$ 40,00");
    }

    #[test]
    fn parses_and_formats_dates() {
        let d = parse_date("05/11/2026").unwrap();
        assert_eq!(d, NaiveDate::from_ymd_opt(2026, 11, 5).unwrap());
        assert_eq!(format_date(d), "05/11/2026");
        assert!(parse_date("2026-11-05").is_err());
        assert!(parse_date("31/02/2026").is_err());
    }
}
