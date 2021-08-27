use borsh::{BorshDeserialize, BorshSchema, BorshSerialize};

#[derive(Clone, Debug, Default, BorshSerialize, BorshDeserialize, BorshSchema, PartialEq)]
pub struct Balance {
    pub base: u128,
    pub quote: u128,
}

impl Balance {
    pub fn calculate_fee_of(&self, quote_amount: u128) -> u128 {
        30 * quote_amount / 10000
    }

    pub fn calculate_base_for_quote_amount(&self, quote_amount: u128) -> u128 {
        (self.base * quote_amount) / (self.quote + quote_amount)
    }

    pub fn calculate_quote_for_base_amount(&self, base_amount: u128) -> u128 {
        (self.quote * base_amount) / (self.base + base_amount)
    }

    pub fn apply_buy_base(&mut self, base_amount: u128) {
        self.apply_buy_base_for_quote(self.calculate_quote_for_base_amount(base_amount))
    }

    pub fn apply_buy_base_for_quote(&mut self, quote_amount: u128) {
        let base_amount = self.calculate_base_for_quote_amount(quote_amount);

        self.quote += quote_amount;
        self.base -= base_amount;
    }

    pub fn apply_sell_base(&mut self, base_amount: u128) {
        let quote_amount = self.calculate_quote_for_base_amount(base_amount);

        self.base += base_amount;
        self.quote -= quote_amount;
    }

    pub fn apply_sell_base_for_quote(&mut self, quote_amount: u128) {
        self.apply_sell_base(self.calculate_base_for_quote_amount(quote_amount))
    }

    pub fn has_enough_quote(&self, quote: u128) -> bool {
        self.quote >= quote
    }

    pub fn has_enough_base(&self, base: u128) -> bool {
        self.base >= base
    }
}

mod tests {
    use super::*;

    #[test]
    fn test_calculations() {
        let mut balance = Balance {
            base: 1000000_000000000000000000u128,
            quote: 1_000000000u128,
        };

        let one_whole_quote = 1000000000u128;
        let one_whole_base = 1000000000000000000u128;

        /*
        let to_buy = one_whole_quote / 7;

        let purchased_base = balance.calculate_base_for_quote_amount(to_buy);
        balance.process_purchase_for_quote_amount(to_buy);
        println!("{:#?}", balance);
        println!("purchased_base = {:#?}", purchased_base);

        let purchased_quote = balance.calculate_quote_for_base_amount(purchased_base);
        balance.process_purchase_for_base_amount(purchased_base);
        println!("{:#?}", balance);
        println!("purchased_quote = {:#?}", purchased_quote);
         */

        println!(
            "\n=============================================================================\n"
        );

        println!(
            "initial balance: base={} ({}) quote={} ({})\n",
            balance.base,
            balance.base / one_whole_base,
            balance.quote,
            balance.quote / one_whole_quote,
        );

        let mut purchases: Vec<u128> = vec![];
        for _ in 0..25 {
            let purchased_base = balance.calculate_base_for_quote_amount(one_whole_quote);
            purchases.push(purchased_base);

            println!(
                "base={} ({}) quote={} ({})   |   buying {} for {} (fee: {})",
                balance.base,
                balance.base / one_whole_base,
                balance.quote,
                balance.quote / one_whole_quote,
                purchased_base,
                one_whole_quote,
                balance.calculate_fee_of(one_whole_quote)
            );

            balance.apply_buy_base_for_quote(one_whole_quote);
        }

        println!(
            "\nbalance after sell-out: base={} ({}) quote={} ({})\n",
            balance.base,
            balance.base / one_whole_base,
            balance.quote,
            balance.quote / one_whole_quote,
        );

        println!(
            "\n=============================================================================\n"
        );

        for p in purchases.iter().rev() {
            println!(
                "base={} ({}) quote={} ({})   |   selling {} for {}",
                balance.base,
                balance.base / one_whole_base,
                balance.quote,
                balance.quote / one_whole_quote,
                *p,
                balance.calculate_quote_for_base_amount(*p),
            );

            balance.apply_sell_base(*p);
        }

        println!(
            "\nbalance after buy-back: base={} ({}) quote={} ({})\n",
            balance.base,
            balance.base / one_whole_base,
            balance.quote,
            balance.quote / one_whole_quote,
        );
    }
}
