#![cfg_attr(not(feature = "std"), no_std)]

use ink_lang as ink;

#[ink::contract]
mod a1Token {
    use ink_storage::{traits::SpreadAllocate, Mapping};

    #[ink(storage)]
    #[derive(SpreadAllocate)]
    pub struct A1Token {
        total_supply: u32,
        balances: Mapping<AccountId, u32>,
        // (Spender => Recipient) => amount 
        allowances: Mapping<(AccountId, AccountId), u32>,
        mint_authority: AccountId,
    }

    #[ink(event)]
    pub struct Transfer {
        #[ink(topic)]
        sender: Option<AccountId>,
        #[ink(topic)]
        recipient: Option<AccountId>,
        amount: u32,
    }

    #[ink(event)]
    pub struct Approval {
        #[ink(topic)]
        owner: AccountId,
        #[ink(topic)]
        spender: AccountId,
        amount: u32,
    }

    /// Specify ERC-20 error type.
    #[derive(Debug, PartialEq, Eq, scale::Encode, scale::Decode)]
    #[cfg_attr(feature = "std", derive(scale_info::TypeInfo))]
    pub enum Error {
        /// Return if the balance cannot fulfill a request.
        InsufficientBalance,
        InsufficientAllowance,
        Unauthorized
    }

    /// Specify the ERC-20 result type.
    pub type Result<T> = core::result::Result<T, Error>;

    use ink_lang::utils::initialize_contract;
    impl A1Token {
        /// Creates a token contract with the given initial supply belonging to the contract creator.
        #[ink(constructor)]
        pub fn new_token(initial_supply: u32) -> Self {
            initialize_contract(|contract: &mut Self| {
                let caller = Self::env().caller();
                contract.balances.insert(&caller, &initial_supply);
                contract.total_supply = initial_supply;
                contract.mint_authority = caller;
                Self::env().emit_event(Transfer {
                    sender: None,
                    recipient: Some(caller),
                    amount: initial_supply,
                });
            })
        }

        /// Returns the total token supply.
        #[ink(message)]
        pub fn total_supply(&self) -> u32 {
            self.total_supply
        }

        /// Checks the current balance of the chosen account.
        #[ink(message)]
        pub fn balance_of(&self, account: AccountId) -> u32 {
            match self.balances.get(&account) {
                Some(value) => value,
                None => 0,
            }
        }

        /// Checks the current mint authority.
        #[ink(message)]
        pub fn get_current_authority(&self) -> AccountId {
            self.mint_authority
        }

        /// Transfers an amount of tokens to the chosen recipient.
        #[ink(message)]
        pub fn transfer(&mut self, recipient: AccountId, amount: u32) -> Result<()> {
            let sender = self.env().caller();
            self.transfer_from_to(sender, recipient, amount)
        }

        /// Transfers an amount of tokens to the chosen recipient.
        #[ink(message)]
        pub fn transfer_from_to(&mut self, sender: AccountId, recipient: AccountId, amount: u32) -> Result<()> {
            let sender_balance = self.balance_of(sender);
            if sender_balance < amount {
                return Err(Error::InsufficientBalance);
            }
            self.balances.insert(sender, &(sender_balance - amount));
            let recipient_balance = self.balance_of(recipient);
            self.balances.insert(recipient, &(recipient_balance + amount));
            self.env().emit_event(Transfer {
                sender: Some(sender),
                recipient: Some(recipient),
                amount: amount,
            });
            Ok(())
        }

        #[ink(message)]
        pub fn approve(&mut self, spender: AccountId, amount: u32) -> Result<()>{
            let owner = self.env().caller();
            self.allowances.insert((owner, spender), &amount);
            self.env().emit_event(Approval {
                owner,
                spender,
                amount,
            });
            Ok(())
        }

        #[ink(message)]
        pub fn allowance(&self, owner: AccountId, spender: AccountId) -> u32 {
            match self.allowances.get((&owner, &spender)){
                Some(value) => value,
                None => 0,
            }
        }

        #[ink(message)]
        pub fn transfer_from(&mut self, sender: AccountId, recipient: AccountId, amount: u32) -> Result<()> {
            let caller = self.env().caller();
            let allowance = self.allowance(sender, caller);
            if allowance < amount {
                return Err(Error::InsufficientAllowance)
            }
            self.transfer_from_to(sender, recipient, amount)?;
            self.allowances.insert((sender, caller), &(allowance - amount));
            Ok(())
        }
        
        /// Mints more tokens if they are mint authority
        #[ink(message)]
        pub fn mint(&mut self, amount: u32) -> Result<()> {
            let sender = self.env().caller();
            if sender != self.mint_authority {
                return Err(Error::Unauthorized);
            }
            let sender_balance = self.balance_of(sender);
            self.balances.insert(sender, &(sender_balance + amount));
            self.total_supply += amount;
            Ok(())
        }

        // Burns tokens
        #[ink(message)]
        pub fn burn(&mut self, amount: u32) -> Result<()> {
            let sender = self.env().caller();
            let sender_balance = self.balance_of(sender);
            if sender_balance < amount {
                return Err(Error::InsufficientBalance);
            }
            self.balances.insert(sender, &(sender_balance - amount));
            self.total_supply -= amount;
            Ok(())
        }

        /// transfers authority
        #[ink(message)]
        pub fn transfer_authority(&mut self, new_owner: AccountId) -> Result<()> {
            let sender = self.env().caller();
            if sender != self.mint_authority {
                return Err(Error::Unauthorized);
            }
            self.mint_authority = new_owner;
            Ok(())
        }   
    }

    /// Unit tests in Rust are normally defined within such a `#[cfg(test)]`
    /// module and test functions are marked with a `#[test]` attribute.
    /// The below code is technically just normal Rust code.
    #[cfg(test)]
    mod tests {
        /// Imports all the definitions from the outer scope so we can use them here.
        use super::*;

        /// Imports `ink_lang` so we can use `#[ink::test]`.
        use ink_lang as ink;

        /// We test if the default constructor does its job.
        #[ink::test]
        fn should_initialize_with_correct_supply() {
            let A1Token = A1Token::new_token(1000);
            assert_eq!(A1Token.total_supply, 1000);
        }

        #[ink::test]
        fn should_allow_transfers() {
            let mut A1Token = A1Token::new_token(1000);
            let alice = AccountId::from([0x1; 32]);
            let bob = AccountId::from([0x2; 32]);

            let initial_bob_balance : u32 = A1Token.balance_of(bob);
            assert_eq!(initial_bob_balance, 0);

            ink_env::test::set_caller::<ink_env::DefaultEnvironment>(alice);
            let initial_alice_balance : u32 = A1Token.balance_of(alice);

            let amount_to_transfer : u32 = 250;
            let success = A1Token.transfer(bob, amount_to_transfer);

            let alice_balance_after : u32 = A1Token.balance_of(alice);
            let bob_balance_after : u32 = A1Token.balance_of(bob);
            
            assert_eq!(success, Ok(()));
            assert_eq!(alice_balance_after, initial_alice_balance - amount_to_transfer);
            assert_eq!(bob_balance_after, initial_bob_balance + amount_to_transfer);
        }

        #[ink::test]
        fn should_mint_more_supply() {
            let amount_to_mint : u32 = 1000;
            let mut A1Token = A1Token::new_token(amount_to_mint);
            assert_eq!(A1Token.total_supply, amount_to_mint);
            assert_eq!(A1Token.mint(amount_to_mint), Ok(()));
            assert_eq!(A1Token.total_supply, amount_to_mint + amount_to_mint);

            let bob = AccountId::from([0x2; 32]);
            ink_env::test::set_caller::<ink_env::DefaultEnvironment>(bob);
            assert_eq!(A1Token.mint(amount_to_mint), Err(Error::Unauthorized));
        }

        #[ink::test]
        fn should_burn_token(){
            let initial_supply : u32 = 1000;
            let mut A1Token = A1Token::new_token(initial_supply);

            let amount_to_burn : u32 = 250;
            assert_eq!(A1Token.burn(amount_to_burn), Ok(()));
            assert_eq!(A1Token.total_supply, initial_supply - amount_to_burn);

            assert_eq!(A1Token.burn(initial_supply), Err(Error::InsufficientBalance));
            assert_eq!(A1Token.total_supply, initial_supply - amount_to_burn);
        }

        #[ink::test]
        fn should_transfer_authority(){
            let alice = AccountId::from([0x1; 32]);
            let bob = AccountId::from([0x2; 32]);
            let jake = AccountId::from([0x3; 32]);

            ink_env::test::set_caller::<ink_env::DefaultEnvironment>(alice);

            let initial_supply : u32 = 1000;
            let mut A1Token = A1Token::new_token(initial_supply);

            assert_eq!(A1Token.mint_authority, alice);
            assert_eq!(A1Token.transfer_authority(bob), Ok(()));
            assert_eq!(A1Token.mint_authority, bob);

            ink_env::test::set_caller::<ink_env::DefaultEnvironment>(jake);
            assert_eq!(A1Token.transfer_authority(alice), Err(Error::Unauthorized));
            assert_eq!(A1Token.mint_authority, bob);
        }

        #[ink::test]
        fn should_approve_allowances(){
            let alice = AccountId::from([0x1; 32]);
            let bob = AccountId::from([0x2; 32]);
            ink_env::test::set_caller::<ink_env::DefaultEnvironment>(alice);

            let initial_supply : u32 = 1000;
            let mut A1Token = A1Token::new_token(initial_supply);

            let amount_to_approve : u32 = 250;
            assert_eq!(A1Token.approve(bob, amount_to_approve), Ok(())); 
            assert_eq!(A1Token.allowance(alice, bob), amount_to_approve);           
        }

        #[ink::test]
        fn should_transfer_from(){
            let alice = AccountId::from([0x1; 32]);
            let bob = AccountId::from([0x2; 32]);
            let jake = AccountId::from([0x2; 32]);
            ink_env::test::set_caller::<ink_env::DefaultEnvironment>(alice);

            let initial_supply : u32 = 1000;
            let mut A1Token = A1Token::new_token(initial_supply);
            assert_eq!(A1Token.balance_of(alice), initial_supply);

            let amount_to_transfer : u32 = 100;
            A1Token.approve(bob, amount_to_transfer);

            ink_env::test::set_caller::<ink_env::DefaultEnvironment>(bob);
            A1Token.transfer_from(alice, jake, amount_to_transfer);
            assert_eq!(A1Token.balance_of(jake), amount_to_transfer);
        }

    }
}
