# grana

Track your subscriptions and who owes you money, from the terminal.

## Install

```sh
cargo install --path .
```

## Usage

```sh
grana subs add netflix 44,90          # monthly
grana subs add icloud 120 --yearly
grana subs list                       # totals per month and year
grana subs price netflix 49,90        # keeps price history
grana subs history netflix
grana subs cancel netflix
```

Data is stored in `~/.local/share/grana/grana.json`.
