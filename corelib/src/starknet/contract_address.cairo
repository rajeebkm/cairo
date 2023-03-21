use zeroable::Zeroable;
use serde::Serde;
use array::SpanTrait;
use array::ArrayTrait;

#[derive(Copy, Drop)]
extern type ContractAddress;
type NonZeroContractAddress = NonZero<ContractAddress>;


extern fn contract_address_const<const address>() -> ContractAddress nopanic;
extern fn contract_address_to_felt252(address: ContractAddress) -> felt252 nopanic;

/// Checks whether the given `ContractAddress` is zero.
extern fn contract_address_is_zero(address: ContractAddress) -> IsZeroResult<ContractAddress> nopanic;

extern fn contract_address_try_from_felt252(
    address: felt252
) -> Option<ContractAddress> implicits(RangeCheck) nopanic;

impl Felt252TryIntoContractAddress of TryInto::<felt252, ContractAddress> {
    fn try_into(self: felt252) -> Option<ContractAddress> {
        contract_address_try_from_felt252(self)
    }
}
impl ContractAddressIntoFelt252 of Into::<ContractAddress, felt252> {
    fn into(self: ContractAddress) -> felt252 {
        contract_address_to_felt252(self)
    }
}


/// Converts `address` to `NonZeroContractAddress`. Panics if `address` is zero.
fn contract_address_non_zero(address: ContractAddress) -> NonZeroContractAddress {
    match contract_address_is_zero(address) {
        IsZeroResult::Zero(()) => {
            let mut data = ArrayTrait::new();
            data.append('Zero contract address');
            panic(data)
        },
        IsZeroResult::NonZero(address_nz) => address_nz,
    }
}

impl ContractAddressZeroable of Zeroable::<ContractAddress> {
    fn zero() -> ContractAddress {
        contract_address_const::<0>()
    }

    #[inline(always)]
    fn is_zero(self: ContractAddress) -> bool {
        //TODO(yg): change to use contract_address_is_zero.
        contract_address_to_felt252(self).is_zero()
    }

    #[inline(always)]
    fn is_non_zero(self: ContractAddress) -> bool {
        !self.is_zero()
    }
}

impl ContractAddressSerde of serde::Serde::<ContractAddress> {
    fn serialize(ref serialized: Array<felt252>, input: ContractAddress) {
        serde::Serde::serialize(ref serialized, contract_address_to_felt252(input));
    }
    fn deserialize(ref serialized: Span<felt252>) -> Option<ContractAddress> {
        Option::Some(contract_address_try_from_felt252(serde::Serde::deserialize(ref serialized)?)?)
    }
}

impl ContractAddressPartialEq of PartialEq::<ContractAddress> {
    #[inline(always)]
    fn eq(a: ContractAddress, b: ContractAddress) -> bool {
        contract_address_to_felt252(a) == contract_address_to_felt252(b)
    }
    #[inline(always)]
    fn ne(a: ContractAddress, b: ContractAddress) -> bool {
        !(a == b)
    }
}

impl NonZeroContractAddressSerde of Serde::<NonZero<ContractAddress>> {
    fn serialize(ref serialized: Array<felt252>, input: NonZero<ContractAddress>) {
        // TODO(yg): change `contract_address_to_felt252` to `.into()`.
        Serde::<felt252>::serialize(ref serialized, contract_address_to_felt252(unwrap_non_zero(input)));
    }
    fn deserialize(ref serialized: Span<felt252>) -> Option<NonZero<ContractAddress>> {
        let f = *serialized.pop_front()?;
        let ca = contract_address_try_from_felt252(f)?;
        match contract_address_is_zero(ca) {
            IsZeroResult::Zero(()) => Option::None(()),
            IsZeroResult::NonZero(nz_ca) => Option::Some(nz_ca),
        }
    }
}
