use anyhow::{Result, bail};
use chrono::NaiveDate;

use crate::format::{format_date, format_money};
use crate::model::{Cycle, Data, PriceChange, Subscription};

impl Subscription {
    pub fn price(&self) -> i64 {
        self.prices.last().expect("subscription has no prices").cents
    }

    /// Price per month in centavos; yearly plans are divided by 12, rounded.
    pub fn monthly_cents(&self) -> i64 {
        match self.cycle {
            Cycle::Monthly => self.price(),
            Cycle::Yearly => (self.price() + 6) / 12,
        }
    }

    pub fn yearly_cents(&self) -> i64 {
        match self.cycle {
            Cycle::Monthly => self.price() * 12,
            Cycle::Yearly => self.price(),
        }
    }
}

fn same_name(a: &str, b: &str) -> bool {
    a.eq_ignore_ascii_case(b)
}

fn find_active<'a>(data: &'a mut Data, name: &str) -> Result<&'a mut Subscription> {
    match data.subscriptions.iter_mut().find(|s| !s.archived && same_name(&s.name, name)) {
        Some(sub) => Ok(sub),
        None => bail!("no active subscription named '{name}'"),
    }
}

/// Active subscription if there is one, otherwise the most recently added archived one.
fn find_any<'a>(data: &'a Data, name: &str) -> Result<&'a Subscription> {
    let all: Vec<_> = data.subscriptions.iter().filter(|s| same_name(&s.name, name)).collect();
    match all.iter().find(|s| !s.archived).or(all.last()) {
        Some(sub) => Ok(sub),
        None => bail!("no subscription named '{name}'"),
    }
}

pub fn add(data: &mut Data, name: &str, cents: i64, cycle: Cycle, today: NaiveDate) -> Result<String> {
    if data.subscriptions.iter().any(|s| !s.archived && same_name(&s.name, name)) {
        bail!("'{name}' already exists (use `grana subs price` to change its price)");
    }
    data.subscriptions.push(Subscription {
        name: name.to_string(),
        cycle,
        prices: vec![PriceChange { cents, date: today }],
        archived: false,
    });
    let per = match cycle {
        Cycle::Monthly => "month",
        Cycle::Yearly => "year",
    };
    Ok(format!("Added {name}: {} per {per}", format_money(cents)))
}

pub fn change_price(data: &mut Data, name: &str, cents: i64, today: NaiveDate) -> Result<String> {
    let sub = find_active(data, name)?;
    let old = sub.price();
    if old == cents {
        bail!("{} already costs {}", sub.name, format_money(cents));
    }
    sub.prices.push(PriceChange { cents, date: today });
    Ok(format!(
        "{}: {} → {} ({}{})",
        sub.name,
        format_money(old),
        format_money(cents),
        if cents > old { "+" } else { "" },
        format_money(cents - old)
    ))
}

pub fn cancel(data: &mut Data, name: &str) -> Result<String> {
    let sub = find_active(data, name)?;
    sub.archived = true;
    Ok(format!(
        "Cancelled {}. You save {} per month.",
        sub.name,
        format_money(sub.monthly_cents())
    ))
}

pub fn list(data: &Data, include_archived: bool) -> String {
    let subs: Vec<_> = data
        .subscriptions
        .iter()
        .filter(|s| include_archived || !s.archived)
        .collect();
    if subs.is_empty() {
        return "No subscriptions yet. Add one with `grana subs add <name> <price>`.".into();
    }

    let name_w = subs.iter().map(|s| s.name.chars().count()).max().unwrap().max(4);
    let mut out = format!("{:<name_w$}  {:<7}  {:>12}  {:>12}\n", "NAME", "CYCLE", "PRICE", "PER MONTH");
    for s in &subs {
        let cycle = match s.cycle {
            Cycle::Monthly => "monthly",
            Cycle::Yearly => "yearly",
        };
        let note = if s.archived { "  (cancelled)" } else { "" };
        out += &format!(
            "{:<name_w$}  {:<7}  {:>12}  {:>12}{note}\n",
            s.name,
            cycle,
            format_money(s.price()),
            format_money(s.monthly_cents())
        );
    }

    let active = subs.iter().filter(|s| !s.archived);
    let yearly: i64 = active.map(|s| s.yearly_cents()).sum();
    out += &format!(
        "\nTotal: {} per month, {} per year",
        format_money((yearly + 6) / 12),
        format_money(yearly)
    );
    out
}

pub fn history(data: &Data, name: &str) -> Result<String> {
    let sub = find_any(data, name)?;
    let mut out = format!("{}\n", sub.name);
    let mut prev: Option<i64> = None;
    for p in &sub.prices {
        let change = match prev {
            None => "(started)".to_string(),
            Some(old) => {
                let pct = (p.cents - old) as f64 / old as f64 * 100.0;
                let pct = format!("{pct:+.1}").replace('.', ",");
                format!("({}{}, {pct}%)", if p.cents > old { "+" } else { "" }, format_money(p.cents - old))
            }
        };
        out += &format!("  {}  {:>12}  {change}\n", format_date(p.date), format_money(p.cents));
        prev = Some(p.cents);
    }
    if sub.archived {
        out += "  (cancelled)\n";
    }
    Ok(out.trim_end().to_string())
}

#[cfg(test)]
mod tests {
    use super::*;

    fn day(d: u32) -> NaiveDate {
        NaiveDate::from_ymd_opt(2026, 10, d).unwrap()
    }

    #[test]
    fn totals_mix_monthly_and_yearly() {
        let mut data = Data::default();
        add(&mut data, "netflix", 4490, Cycle::Monthly, day(1)).unwrap();
        add(&mut data, "icloud", 12000, Cycle::Yearly, day(1)).unwrap();
        let out = list(&data, false);
        // 44,90 * 12 + 120,00 = 658,80 per year; / 12 = 54,90 per month
        assert!(out.contains("R$ 54,90 per month, R$ 658,80 per year"), "{out}");
    }

    #[test]
    fn duplicate_active_name_is_rejected_case_insensitively() {
        let mut data = Data::default();
        add(&mut data, "Netflix", 4490, Cycle::Monthly, day(1)).unwrap();
        assert!(add(&mut data, "netflix", 3990, Cycle::Monthly, day(2)).is_err());
    }

    #[test]
    fn price_change_is_recorded() {
        let mut data = Data::default();
        add(&mut data, "netflix", 3990, Cycle::Monthly, day(1)).unwrap();
        change_price(&mut data, "NETFLIX", 4490, day(15)).unwrap();
        let sub = &data.subscriptions[0];
        assert_eq!(sub.price(), 4490);
        assert_eq!(sub.prices.len(), 2);
        assert_eq!(sub.prices[1].date, day(15));
        assert!(change_price(&mut data, "netflix", 4490, day(16)).is_err());
    }

    #[test]
    fn cancelled_subs_leave_totals_but_keep_history() {
        let mut data = Data::default();
        add(&mut data, "netflix", 4490, Cycle::Monthly, day(1)).unwrap();
        add(&mut data, "spotify", 2190, Cycle::Monthly, day(1)).unwrap();
        cancel(&mut data, "netflix").unwrap();

        assert!(list(&data, false).contains("R$ 21,90 per month"));
        assert!(!list(&data, false).contains("netflix"));
        assert!(list(&data, true).contains("(cancelled)"));
        assert!(history(&data, "netflix").unwrap().contains("(cancelled)"));
        assert!(change_price(&mut data, "netflix", 5000, day(2)).is_err());

        // Re-subscribing creates a fresh entry; history shows the active one.
        add(&mut data, "netflix", 5990, Cycle::Monthly, day(20)).unwrap();
        assert!(history(&data, "netflix").unwrap().contains("R$ 59,90"));
    }
}
