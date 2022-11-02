#![cfg_attr(not(feature = "std"), no_std)]
use ink::env::chain_extension::FromStatusCode;
use ink::env::Environment;
use scale::Decode;
pub use xcm::{VersionedMultiAsset, VersionedMultiLocation, VersionedResponse, VersionedXcm};

#[derive(Decode)]
pub enum Error {
    NoResponse = 1,
}

impl FromStatusCode for Error {
    fn from_status_code(status_code: u32) -> Result<(), Self> {
        match status_code {
            0 => Ok(()),
            1 => Err(Self::NoResponse),
            _ => panic!("Unknown status code"),
        }
    }
}

#[ink::chain_extension]
pub trait XCMExtension {
    type ErrorCode = Error;

    #[ink(extension = 0x00010000, handle_status = false, returns_result = false)]
    fn prepare_execute(xcm: VersionedXcm<()>) -> u64;

    #[ink(extension = 0x00010001, handle_status = false, returns_result = false)]
    fn execute();

    #[ink(extension = 0x00010002, handle_status = false, returns_result = false)]
    fn prepare_send(dest: VersionedMultiLocation, xcm: VersionedXcm<()>) -> VersionedMultiAsset;

    #[ink(extension = 0x00010003, handle_status = false, returns_result = false)]
    fn send();

    #[ink(extension = 0x00010004, handle_status = false, returns_result = false)]
    fn new_query() -> u64;

    #[ink(extension = 0x00010005, handle_status = true, returns_result = false)]
    fn take_response(query_id: u64) -> Result<VersionedResponse, Error>;
}

pub enum CustomEnvironment {}

impl Environment for CustomEnvironment {
    const MAX_EVENT_TOPICS: usize = <ink::env::DefaultEnvironment as Environment>::MAX_EVENT_TOPICS;

    type AccountId = <ink::env::DefaultEnvironment as Environment>::AccountId;
    type Balance = <ink::env::DefaultEnvironment as Environment>::Balance;
    type Hash = <ink::env::DefaultEnvironment as Environment>::Hash;
    type BlockNumber = <ink::env::DefaultEnvironment as Environment>::BlockNumber;
    type Timestamp = <ink::env::DefaultEnvironment as Environment>::Timestamp;

    type ChainExtension = XCMExtension;
}


#[ink::contract(env = crate::CustomEnvironment)]
mod xcm_playground {
    pub use xcm::opaque::latest::prelude::{
        Junction, Junctions, MultiLocation,
        Transact, Xcm, OriginKind
    };
    //pub use xcm::opaque::latest::prelude::*;
    pub use xcm::{VersionedMultiAsset, VersionedMultiLocation, VersionedResponse, VersionedXcm, v3::{WeightLimit,Fungibility,AssetId,Parent,WildMultiAsset,MultiAsset,MultiAssets,MultiAssetFilter,Instruction::{DepositReserveAsset,InitiateReserveWithdraw,BuyExecution,DepositAsset,WithdrawAsset}}};

    /// Defines the storage of your contract.
    /// Add new fields to the below struct in order
    /// to add new static storage fields to your contract.
    #[ink(storage)]
    pub struct XcmPlayground {
        /// Stores a single `bool` value on the storage.
        value: bool,
    }

    impl XcmPlayground {
        /// Constructor that initializes the `bool` value to the given `init_value`.
        #[ink(constructor)]
        pub fn new(init_value: bool) -> Self {
            Self { value: init_value }
        }

        /// Constructor that initializes the `bool` value to `false`.
        ///
        /// Constructors can delegate to other constructors.
        #[ink(constructor)]
        pub fn default() -> Self {
            Self::new(Default::default())
        }

        #[ink(message)]
        pub fn send_message(&mut self, account: [u8; 32]) {
            let para_1000 = Junctions::X1(Junction::Parachain(1000));
            let para_3000 = Junctions::X1(Junction::Parachain(3000));
            let account_dest = Junctions::X1(Junction::AccountId32 { network: None, id: account });
            let reserved_asset = Junctions::X3(Junction::Parachain(1000), Junction::PalletInstance(50), Junction::GeneralIndex(1));
            let buy_execution_asset = Junctions::X2(Junction::PalletInstance(50), Junction::GeneralIndex(1));

            let reserve =MultiLocation {
                parents: 1,
                interior: para_1000,
            };
            let dest = MultiLocation {
                parents: 1,
                interior: para_3000,
            };
            let beneficiary = MultiLocation {
                parents: 0,
                interior: account_dest,
            };
            let reserved_location = MultiLocation {
                parents: 1,
                interior: reserved_asset,
            };

            let buy_asset_location = MultiLocation {
                parents: 0,
                interior: buy_execution_asset,
            };
            


            let fees = MultiAsset {
                id: AssetId::Concrete(buy_asset_location),
                fun: Fungibility::Fungible(1000000000000_u128)
            };
            let assets = MultiAssetFilter::Wild(WildMultiAsset::All);
            let mut multi_assets = MultiAssets::new();
            multi_assets.push(
                MultiAsset {
                    id: AssetId::Concrete(reserved_location),
                    fun: Fungibility::Fungible(5000000000000_u128)
                }
            );
            let xcm = Xcm([
                WithdrawAsset(multi_assets),
                InitiateReserveWithdraw {
                    assets: assets.clone(),
                    reserve,
                    xcm: Xcm([
                        BuyExecution { fees, weight_limit: WeightLimit::Unlimited},
                        DepositReserveAsset {
                            assets: assets.clone(),
                            dest,
                            xcm: Xcm([DepositAsset { assets: MultiAssetFilter::Wild(WildMultiAsset::All), beneficiary } ].into())
                        }
                    ].to_vec())
                }].to_vec());
            let versioned_xcm = VersionedXcm::from(xcm);

            self
                .env()
                .extension()
                .prepare_execute(versioned_xcm);
            self.env().extension().execute();
        }
    }

    /// Unit tests in Rust are normally defined within such a `#[cfg(test)]`
    /// module and test functions are marked with a `#[test]` attribute.
    /// The below code is technically just normal Rust code.
    #[cfg(test)]
    mod tests {
    }
}
