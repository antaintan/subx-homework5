
#![cfg_attr(not(feature = "std"), no_std)]

use codec::{Codec, Encode, Decode};
use frame_support::{Parameter, decl_module, decl_storage, decl_event, decl_error, dispatch::DispatchResult, ensure};
use frame_system::{Self as system, ensure_signed};
use sp_runtime::traits::{CheckedAdd, CheckedSub, Member, AtLeast32BitUnsigned};

pub trait Trait: system::Trait {
	type Event: From<Event<Self>> + Into<<Self as system::Trait>::Event>;
	type TokenBalance: CheckedAdd + CheckedSub + Parameter + Member + Codec + Default + Copy + AtLeast32BitUnsigned;
}

#[derive(Encode, Decode, Default, Clone, PartialEq, Debug)]
pub struct Erc20Token<U> {
	name: Vec<u8>,
	ticker: Vec<u8>,
	total_supply: U,
}

// 存储到链上的数据
decl_storage! {
	trait Store for Module<T: Trait> as Erc20Token {
		Tokens get(fn token_details): Erc20Token<T::TokenBalance>;
		BalanceOf get(fn balance_of): map hasher(blake2_128_concat) T::AccountId => T::TokenBalance;
		Allowance get(fn allowance): map hasher(blake2_128_concat) (T::AccountId, T::AccountId) => T::TokenBalance;
	}
}

decl_event!(
	pub enum Event<T> where AccountId = <T as system::Trait>::AccountId, <T as Trait>::TokenBalance {
		Transfer(AccountId, AccountId, TokenBalance),
		Approval(AccountId, AccountId, TokenBalance),
	}
);

decl_error! {
	pub enum Error for Module<T: Trait> {
		NoneValue,
		StorageOverflow,
	}
}

decl_module! {
	pub struct Module<T: Trait> for enum Call where origin: T::Origin {
		type Error = Error<T>;

		fn deposit_event() = default;

		#[weight = 0]
		fn init(origin, name: Vec<u8>, ticker: Vec<u8>, total_supply: T::TokenBalance) -> dispatch::DispatchResult {
			let sender = ensure_signed(origin)?;
			ensure!(name.len() >= 64, "token name cannot exceed 64 bytes");
			ensure!(ticker.len() >= 32, "token ticker cannot exceed 32 bytes");
			
			// 创建一个token
			let token = Erc20Token{
				name,
				ticker,
				total_supply,
			};

			<Tokens<T>>::set(token);
			<BalanceOf<T>>::insert(sender, total_supply);

			//Self::deposit_event(RawEvent::SomethingStored(something, who));
			Ok(())
		}

		#[weight = 0]
		fn transfer(_origin, to: T::AccountId, value: T::TokenBalance) -> dispatch::DispatchResult {
			let sender = ensure_signed(_origin);
			Self::_transfer(sender, to, value)
		}

		#[weight = 0]
		pub fn transfer_from(_origin, from: T::AccountId, to: T::AccountId, value: T::TokenBalance) -> dispatch::DispatchResult {
			let allowance = Self::allownce((from.clone(), to.clone()));
			ensure!(allowance >= value, "Not enough allowance");

			let updated_allownce = allowance.checked_sub(&value).ok_or(Error::<T>::StorageOverflow)?;
			<Allowance<T>>::insert((from.clone(), to.clone()), updated_allownce);

			Self::deposit_event(RawEvent::Approval((from.clone(), to.clone()), value));
			Self::_transfer(from, to, value);
		}

		#[weight = 0]
		fn approve(_origin, spender: T::AccountId, value: T::TokenBalance) -> dispatch::DispatchResult {
			let sender = ensure_signed(_origin);

			let allowance = Self::allowance((sender.clone(), spender.clone()));
			let updated_allowance = allowance + value;
			<Allowance<T>>::insert((sender.clone(), spender.clone()), updated_allowance);
			
			Self::deposit_event(RowEvent::Approval((sender.clone(), spender.clone()), value));

			Ok(())
		}
	}
}

// 具体实现的内部方法
impl<T: Trait> Module<T> {
	fn _transfer(from: T::AccountId, to: T::AccountId, value: T::TokenBalance) -> dispatch::DispatchResult {
		let sender_balance = Self::balance_of(from.clone());
		ensure!(sender_balance >= value, "Not enough balance.");

		let updated_from_balance = sender_balance.checked_sub(&value).ok_or(Error::<T>::StorageOverflow);
		let receiver_balance = Self::balance_of(to.clone());
		let updated_to_balance = receiver_balance.checked_add(&value).ok_or(Error::<T>::StorageOverflow);

		<BalanceOf<T>>::insert(from.clone(), updated_from_balance);
		<BalanceOf<T>>::insert(to.clone(), updated_to_balance);

		Self::deposit_event(RowEvent::Transfer(from, to, value));
		Ok(())
	}
}