#[ink::contract]
mod erc721 {
    use ink::storage::Mapping;

    #[ink(storage)]
    pub struct Erc721 {
        owner_of: Mapping<u128, AccountId>,
        balances: Mapping<AccountId, u128>,
        token_approvals: Mapping<u128, AccountId>,
        operator_approvals: Mapping<(AccountId, AccountId), bool>,
        next_token_id: u128,
    }

    impl Erc721 {
        #[ink(constructor)]
        pub fn new() -> Self {
            Self {
                owner_of: Mapping::new(),
                balances: Mapping::new(),
                token_approvals: Mapping::new(),
                operator_approvals: Mapping::new(),
                next_token_id: 0,
            }
        }

        #[ink(message)]
        pub fn balance_of(&self, owner: AccountId) -> u128 {
            self.balances.get(owner).unwrap_or(0)
        }

        #[ink(message)]
        pub fn owner_of(&self, token_id: u128) -> Option<AccountId> {
            self.owner_of.get(token_id)
        }

        #[ink(message)]
        pub fn mint(&mut self) -> u128 {
            let caller = self.env().caller();
            let token_id = self.next_token_id;
            self.owner_of.insert(token_id, &caller);
            let balance = self.balance_of(caller);
            self.balances.insert(caller, &(balance + 1));
            self.next_token_id += 1;
            token_id
        }

        #[ink(message)]
        pub fn transfer_from(&mut self, from: AccountId, to: AccountId, token_id: u128) -> bool {
            let caller = self.env().caller();
            let owner = match self.owner_of(token_id) {
                Some(owner) => owner,
                None => return false,
            };
            if owner != from {
                return false;
            }
            if caller != from && !self.is_approved_or_owner(caller, token_id) {
                return false;
            }
            self.owner_of.insert(token_id, &to);
            let from_balance = self.balance_of(from);
            self.balances.insert(from, &(from_balance - 1));
            let to_balance = self.balance_of(to);
            self.balances.insert(to, &(to_balance + 1));
            self.token_approvals.remove(token_id);
            true
        }

        fn is_approved_or_owner(&self, spender: AccountId, token_id: u128) -> bool {
            let owner = match self.owner_of(token_id) {
                Some(owner) => owner,
                None => return false,
            };
            if spender == owner {
                return true;
            }
            if let Some(approved) = self.token_approvals.get(token_id) {
                if approved == spender {
                    return true;
                }
            }
            if let Some(is_approved) = self.operator_approvals.get((owner, spender)) {
                return is_approved;
            }
            false
        }
    }
}
            