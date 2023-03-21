use starknet::ContractAddress;
// TODO(yg): move.
#[abi]
trait IMintableToken {
    // TODO(yg): account (in both) should be NonZero<ContractAddress>?
    fn permissioned_mint(account: ContractAddress, amount: u256);
    fn permissioned_burn(account: ContractAddress, amount: u256);
}

#[contract]
mod TokenBridge {
    // TODO(yg): sort.
    use array::ArrayTrait;
    use starknet::ContractAddress;
    use starknet::get_caller_address;
    use zeroable::Zeroable;
    use integer::Felt252IntoU256;
    // TODO(yg): remove after changing ETH_ADDRESS_BOUND to u256.
    use integer::Felt252TryIntoU128;
    use starknet::contract_address::NonZeroContractAddressSerde;
    use starknet::contract_address::contract_address_non_zero;
    use integer::u128_try_from_felt252;
    use option::OptionTrait;
    use starknet::contract_address::ContractAddressZeroable;
    use super::IMintableTokenDispatcherTrait;
    use super::IMintableTokenDispatcher;
    use super::IMintableTokenLibraryDispatcher;
    use integer::u128_to_felt252;
    use starknet::syscalls::send_message_to_l1_syscall;

    const WITHDRAW_MESSAGE: felt252 = 0;
    // TODO(yg): change to 0x10000000000000000000000000000000000000000_u256. Add in semantic in validate_literal and in literal_to_semantic.
    // TODO(yg): remove explicit type.
    const ETH_ADDRESS_BOUND: u128 = 0x100_u128; // 2 ** 160
    const CONTRACT_IDENTITY: felt252 = 'STARKGATE';
    const CONTRACT_VERSION: felt252 = 1;

    struct Storage {
        // TODO(Yg): nonzero? doc
        governor: ContractAddress,
        // The L1 bridge address. Zero when unset.
        l1_bridge: felt252,
        // The L2 token contract address. Zero when unset.
        l2_token: ContractAddress,
    }

    // An event that is emitted when set_l1_bridge is called.
    // * l1_bridge_address is the new l1 bridge address.
    #[event]
    fn l1_bridge_set(l1_bridge_address: NonZero<felt252>) {}

    // An event that is emitted when set_l2_token is called.
    // * l2_token_address is the new l2 token address.
    #[event]
    fn l2_token_set(l2_token_address: NonZero<ContractAddress>) {}

    // An event that is emitted when initiate_withdraw is called.
    // * l1_recipient is the l1 recipient address.
    // * amount is the amount to withdraw.
    // * caller_address is the address from which the call was made.
    #[event]
    fn withdraw_initiated(l1_recipient: NonZero<felt252>, amount: u256, caller_address: NonZero<ContractAddress>) {}

    // An event that is emitted when handle_deposit is called.
    // * account is the recipient address.
    // * amount is the amount to deposit.
    #[event]
    fn deposit_handled(account: NonZero<ContractAddress>, amount: u256) {}

    #[view]
    fn get_version() -> felt252 {
        CONTRACT_VERSION
    }

    #[view]
    fn get_identity() -> felt252 {
        CONTRACT_IDENTITY
    }

    #[constructor]
    fn constructor(
        governor_address: NonZero<ContractAddress>,
    ) {
        // TODO(yg): change to .unwrap() with a NonZero trait?
        governor::write(unwrap_non_zero(governor_address));
    }

    // TODO(Yg): inline stuff... Not only here.
    #[external]
    fn set_l1_bridge(l1_bridge_address: NonZero<felt252>) {
        // The call is restricted to the governor.
        let caller_address = get_caller_address();
        let governor = governor::read();
        assert(caller_address == governor, 'GOVERNOR_ONLY');

        assert(l1_bridge::read().is_zero(), 'BRIDGE_ALREADY_INITIALIZED');
        // TODO(yg): change to .unwrap() with a NonZero trait?
        let l1_bridge_address_felt = unwrap_non_zero(l1_bridge_address);
        // TODO(yg): change to `.into()` after changing ETH_ADDRESS_BOUND to u256.
        assert(u128_try_from_felt252(l1_bridge_address_felt).unwrap() < ETH_ADDRESS_BOUND, 'BRIDGE_ADDRESS_OUT_OF_RANGE');

        l1_bridge::write(l1_bridge_address_felt);
        l1_bridge_set(l1_bridge_address);
    }

    #[external]
    fn set_l2_token(l2_token_address: NonZero<ContractAddress>) {
        // The call is restricted to the governor.
        let caller_address = get_caller_address();
        let governor = governor::read();
        assert(caller_address == governor, 'GOVERNOR_ONLY');

        assert(l2_token::read().is_zero(), 'L2_TOKEN_ALREADY_INITIALIZED');

        // TODO(yg): change to .unwrap() with a NonZero trait?
        let l2_token_address_felt = unwrap_non_zero(l2_token_address);
        l2_token::write(l2_token_address_felt);
        l2_token_set(l2_token_address);
    }

    #[external]
    fn initiate_withdraw(l1_recipient: NonZero<felt252>, amount: u256) {
        let l1_recipient_felt = unwrap_non_zero(l1_recipient);
        // TODO(yg): change to `.into()` after changing ETH_ADDRESS_BOUND to u256.
        assert(u128_try_from_felt252(l1_recipient_felt).unwrap() < ETH_ADDRESS_BOUND, 'RECIPIENT_ADDRESS_OUT_OF_RANGE');

        // Check address is valid.
        let l2_token = l2_token::read();
        assert(!l2_token.is_zero(), 'UNINITIALIZED_TOKEN');

        // Call burn on l2_token contract.
        let caller_address = get_caller_address();
        IMintableTokenDispatcher{ contract_address: l2_token }.permissioned_burn(
            account: caller_address, :amount
        );

        // Send the message.
        let mut message_payload: Array<felt252> = ArrayTrait::new();
        message_payload.append(WITHDRAW_MESSAGE);
        message_payload.append(unwrap_non_zero(l1_recipient));
        // TODO(yg): change to `.into()`.
        message_payload.append(u128_to_felt252(amount.low));
        // TODO(yg): change to `.into()`.
        message_payload.append(u128_to_felt252(amount.high));

        // Check address is valid.
        // TODO(yg): consider exporting a function to read+verify nz (both for l1_bridge and l2_token).
        let l1_bridge_address = l1_bridge::read();
        assert(!l1_bridge_address.is_zero(), 'L1_BRIDGE_ADDRESS_UNINITIALIZED');
        // TODO(yg): isn't this checked in the set function?
        // TODO(yg): change to `.into()` after changing ETH_ADDRESS_BOUND to u256.
        assert(u128_try_from_felt252(l1_bridge_address).unwrap() < ETH_ADDRESS_BOUND, 'TO_ADDRESS_OUT_OF_RANGE');

        send_message_to_l1_syscall(to_address: l1_bridge_address, payload: message_payload.span());
        withdraw_initiated(:l1_recipient, :amount, caller_address: contract_address_non_zero(caller_address));
    }

    #[l1_handler]
    // TODO(yg): change from_address to nz? Or at least verify it's !.is_zero()...?
    fn handle_deposit(from_address: felt252, account: NonZero<ContractAddress>, amount: u256) {
        assert(from_address == l1_bridge::read(), 'EXPECTED_FROM_BRIDGE_ONLY');

        // Call mint on l2_token contract.
        let l2_token = l2_token::read();
        assert(!l2_token.is_zero(), 'UNINITIALIZED_TOKEN');
        // TODO(yg): change to .unwrap() with a NonZero trait?
        IMintableTokenDispatcher{ contract_address: l2_token }.permissioned_mint(
            account: unwrap_non_zero(account), :amount
        );

        deposit_handled(:account, :amount);
    }

    // TODO(yg): capital letters for events?
    // TODO(yg): revisit error strings.
    // TODO(yg): change all NonZero<ContractAddress> to NonZeroContractAddress? If not, remove NonZeroContractAddress.
}
