use crate::{prelude::*, NetworkId};

#[cfg(feature = "cw-storage-plus")]
use cw_storage_plus::{Key, Prefixer};

use crate::shared::Displayed;
use core::ops::Add;
use cosmwasm_std::{Uint128, Uint256};
use num::Zero;
use serde::{Deserialize, Serialize};

/// Newtype for CVM assets ID. Must be unique for each asset and must never change.
/// This ID is an opaque, arbitrary type from the CVM protocol and no assumption must be made on
/// how it is computed.
#[cfg_attr(
    feature = "json-schema", // all(feature = "json-schema", not(target_arch = "wasm32")),
    derive(schemars::JsonSchema)
)]
#[derive(Copy, Clone, PartialEq, Eq, PartialOrd, Ord, Debug, Hash, Serialize, Deserialize)]
#[repr(transparent)]
pub struct AssetId(
    #[cfg_attr(
        feature = "json-schema", // all(feature = "json-schema", not(target_arch = "wasm32")),
        schemars(with = "String")
    )]
    pub Displayed<u128>,
);

impl FromStr for AssetId {
    type Err = <u128 as FromStr>::Err;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Ok(AssetId(Displayed(u128::from_str(s)?)))
    }
}

impl core::fmt::Display for AssetId {
    fn fmt(&self, fmtr: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        self.0 .0.fmt(fmtr)
    }
}

impl From<AssetId> for u128 {
    fn from(val: AssetId) -> Self {
        val.0 .0
    }
}

impl From<u128> for AssetId {
    fn from(asset: u128) -> Self {
        AssetId(Displayed(asset))
    }
}

#[cfg(feature = "cw-storage-plus")]
impl<'a> cw_storage_plus::PrimaryKey<'a> for AssetId {
    type Prefix = ();
    type SubPrefix = ();
    type Suffix = u128;
    type SuperSuffix = u128;

    fn key(&self) -> Vec<cw_storage_plus::Key> {
        use cw_storage_plus::IntKey;
        vec![cw_storage_plus::Key::Val128(self.0 .0.to_cw_bytes())]
    }
}

#[cfg(feature = "cw-storage-plus")]
impl<'a> Prefixer<'a> for AssetId {
    fn prefix(&self) -> Vec<Key> {
        use cw_storage_plus::IntKey;
        vec![Key::Val128(self.0 .0.to_cw_bytes())]
    }
}

#[cfg(feature = "cw-storage-plus")]
impl cw_storage_plus::KeyDeserialize for AssetId {
    type Output = <u128 as cw_storage_plus::KeyDeserialize>::Output;

    fn from_vec(value: Vec<u8>) -> cosmwasm_std::StdResult<Self::Output> {
        <u128 as cw_storage_plus::KeyDeserialize>::from_vec(value)
    }

    fn from_slice(value: &[u8]) -> cosmwasm_std::StdResult<Self::Output> {
        <u128 as cw_storage_plus::KeyDeserialize>::from_slice(value)
    }
}

/// See https://en.wikipedia.org/wiki/Linear_equation#Slope%E2%80%93intercept_form_or_Gradient-intercept_form
#[cfg_attr(
    feature = "json-schema", // all(feature = "json-schema", not(target_arch = "wasm32")),
    derive(schemars::JsonSchema)
)]
#[derive(Clone, PartialEq, Eq, PartialOrd, Ord, Debug, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub struct Amount {
    /// absolute amount, optional, default is 0
    #[serde(skip_serializing_if = "is_default", default)]
    pub intercept: Displayed<u128>,
    /// part of `MAX_PARTS` from remaining after intercept subtraction, optional, default is 0
    #[serde(skip_serializing_if = "is_default", default)]
    pub slope: Displayed<u64>,
}

impl TryFrom<i64> for Amount {
    type Error = ArithmeticError;

    fn try_from(value: i64) -> Result<Self, Self::Error> {
        if value < 0 {
            return Err(ArithmeticError::Underflow);
        }
        Ok(Amount::absolute(value as u64 as u128))
    }
}

impl FromStr for Amount {
    type Err = <u128 as FromStr>::Err;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        u128::from_str(s).map(Amount::absolute)
    }
}

/// analog of `Coin`s in IBC/CW, but with CVM numeric id and amount
/// requires registry to map id back and forth as needed
#[cfg_attr(
    feature = "json-schema", // all(feature = "json-schema", not(target_arch = "wasm32")),
    derive(schemars::JsonSchema)
)]
#[derive(Clone, PartialEq, Eq, PartialOrd, Ord, Debug, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub struct AbsoluteAmount {
    pub amount: Displayed<u128>,
    pub asset_id: AssetId,
}

fn is_default<T: Default + PartialEq>(value: &T) -> bool {
    *value == T::default()
}

/// Arithmetic errors.
#[derive(Eq, PartialEq, Clone, Copy, Debug, Serialize, Deserialize)]
pub enum ArithmeticError {
    /// Underflow.
    Underflow,
    /// Overflow.
    Overflow,
    /// Division by zero.
    DivisionByZero,
}

impl From<(u64, u64)> for Amount {
    fn from(value: (u64, u64)) -> Self {
        Self::new(
            0,
            (value.0 as u128 * Self::MAX_PARTS as u128 / value.1 as u128) as u64,
        )
    }
}

impl Amount {
    /// idiotic idea fom KO, it should be
    /// u64/u64 ratio
    //  with rounding to reduce or reduce down part up to some seven bit parts
    pub const MAX_PARTS: u64 = 1_000_000_000_000_000_000;

    pub fn one() -> Self {
        Self::absolute(1)
    }

    pub fn try_floor_f64(value: f64) -> Result<Self, ArithmeticError> {
        if value < 0.0 || value.is_nan() {
            Err(ArithmeticError::Underflow)
        } else if value > u128::MAX as f64 {
            Err(ArithmeticError::Underflow)
        } else {
            Ok((value as u128).into())
        }
    }

    pub const fn new(intercept: u128, slope: u64) -> Self {
        Self {
            intercept: Displayed(intercept),
            slope: Displayed(slope),
        }
    }

    /// An absolute amount
    pub const fn absolute(value: u128) -> Self {
        Self {
            intercept: Displayed(value),
            slope: Displayed(0),
        }
    }

    /// A ratio amount, expressed in parts (x / MAX_PARTS)
    pub const fn ratio(parts: u64) -> Self {
        Self {
            intercept: Displayed(0),
            slope: Displayed(parts),
        }
    }

    /// Helper function to see if the amount is absolute
    pub const fn is_absolute(&self) -> bool {
        self.slope.0 == 0
    }

    /// Helper function to see if the amount is ratio
    pub const fn is_ratio(&self) -> bool {
        self.intercept.0 == 0
    }

    /// Everything mean that we move 100% of whats left.
    pub const fn everything() -> Self {
        Self::ratio(Self::MAX_PARTS)
    }

    /// `f(x) = a(x - b) + b where a = slope / MAX_PARTS, b = intercept`
    pub fn apply(&self, value: u128) -> Result<u128, ArithmeticError> {
        if value.is_zero() {
            return Ok(0);
        }
        let amount = if self.slope.0.is_zero() {
            self.intercept.0
        } else if self.slope.0 == Self::MAX_PARTS {
            value
        } else {
            let value = Uint256::from(value)
                .checked_sub(self.intercept.0.into())
                .map_err(|_| ArithmeticError::Underflow)?;
            let value = value
                .checked_multiply_ratio(self.slope.0, Self::MAX_PARTS)
                .map_err(|_| ArithmeticError::Overflow)?;
            let value = value
                .checked_add(self.intercept.0.into())
                .map_err(|_| ArithmeticError::Overflow)?;
            Uint128::try_from(value)
                .map_err(|_| ArithmeticError::Overflow)?
                .u128()
        };
        Ok(u128::min(value, amount))
    }

    /// `f(x) = (a + b) * 10 ^ decimals where a = intercept, b = slope / MAX_PARTS`
    pub fn apply_with_decimals(&self, decimals: u8, value: u128) -> Result<u128, ArithmeticError> {
        if value.is_zero() {
            return Ok(0);
        }
        let unit = 10_u128
            .checked_pow(decimals as u32)
            .ok_or(ArithmeticError::Overflow)?;
        let amount = if self.slope.0.is_zero() {
            self.intercept
                .0
                .checked_mul(unit)
                .ok_or(ArithmeticError::Overflow)?
        } else if self.slope.0 == Self::MAX_PARTS {
            value
        } else {
            let value = Uint256::from(self.intercept.0);
            let value = value
                .checked_add(
                    Uint256::one()
                        .checked_multiply_ratio(self.slope.0, Self::MAX_PARTS)
                        .map_err(|_| ArithmeticError::Overflow)?,
                )
                .map_err(|_| ArithmeticError::Overflow)?;
            let value = value
                .checked_mul(Uint256::from(10_u128.pow(decimals as u32)))
                .map_err(|_| ArithmeticError::Overflow)?;
            Uint128::try_from(value)
                .map_err(|_| ArithmeticError::Overflow)?
                .u128()
        };
        Ok(u128::min(value, amount))
    }
}

impl Add for Amount {
    type Output = Self;

    fn add(
        self,
        Self {
            intercept: Displayed(i_1),
            slope: Displayed(s_1),
        }: Self,
    ) -> Self::Output {
        let Self {
            intercept: Displayed(i_0),
            slope: Displayed(s_0),
        } = self;
        Self {
            intercept: Displayed(i_0.saturating_add(i_1)),
            slope: Displayed(s_0.saturating_add(s_1)),
        }
    }
}

impl Zero for Amount {
    fn zero() -> Self {
        Self {
            intercept: Displayed(0),
            slope: Displayed(0),
        }
    }

    fn is_zero(&self) -> bool {
        self == &Self::zero()
    }
}

impl From<u128> for Amount {
    fn from(x: u128) -> Self {
        Self::absolute(x)
    }
}

/// a set of assets with non zero balances
#[cfg_attr(
    feature = "json-schema", // all(feature = "json-schema", not(target_arch = "wasm32")),
    derive(schemars::JsonSchema)
)]
#[derive(Clone, PartialEq, Eq, PartialOrd, Ord, Debug, Serialize, Deserialize)]
#[repr(transparent)]
pub struct Funds<T = Amount>(pub Vec<(AssetId, T)>);

impl<T> Funds<T> {
    pub fn one<A: Into<T>>(id: AssetId, amount: A) -> Self {
        Self(vec![(id, amount.into())])
    }
}

impl<T> Default for Funds<T> {
    fn default() -> Self {
        Self(Vec::new())
    }
}

impl<T> IntoIterator for Funds<T> {
    type Item = <Vec<(AssetId, T)> as IntoIterator>::Item;
    type IntoIter = <Vec<(AssetId, T)> as IntoIterator>::IntoIter;
    fn into_iter(self) -> Self::IntoIter {
        self.0.into_iter()
    }
}

impl<T, U, V> From<Vec<(U, V)>> for Funds<T>
where
    U: Into<AssetId>,
    V: Into<T>,
{
    fn from(assets: Vec<(U, V)>) -> Self {
        Funds(
            assets
                .into_iter()
                .map(|(asset, amount)| (asset.into(), amount.into()))
                .collect(),
        )
    }
}

impl<T, U, V, const K: usize> From<[(U, V); K]> for Funds<T>
where
    U: Into<AssetId>,
    V: Into<T>,
{
    #[inline]
    fn from(x: [(U, V); K]) -> Self {
        Funds(
            x.into_iter()
                .map(|(asset, amount)| (asset.into(), amount.into()))
                .collect(),
        )
    }
}

impl<T> From<Funds<T>> for Vec<(AssetId, T)> {
    fn from(Funds(assets): Funds<T>) -> Self {
        assets
    }
}

impl<T> From<Funds<T>> for Vec<(u128, T)> {
    fn from(Funds(assets): Funds<T>) -> Self {
        assets
            .into_iter()
            .map(|(AssetId(Displayed(asset)), amount)| (asset, amount))
            .collect()
    }
}

/// see `generate_network_prefixed_id`
///```rust
/// use cvm::generate_asset_id;
/// let pica_on_picasso = generate_asset_id(0.into(), 0, 1);
/// assert_eq!(pica_on_picasso, 1.into());
/// let pica_on_composable = generate_asset_id(1.into(), 0, 1);
/// assert_eq!(pica_on_composable, 79228162514264337593543950337.into());
///```
pub fn generate_asset_id(network_id: NetworkId, protocol_id: u32, nonce: u64) -> AssetId {
    AssetId::from(generate_network_prefixed_id(network_id, protocol_id, nonce))
}

// `protocol_id` - namespace like thing, default is 0, but can be used for example other consensus
// to create known ahead
/// `nonce` - local consensus atomic number, usually increasing monotonic increment
pub fn generate_network_prefixed_id(network_id: NetworkId, protocol_id: u32, nonce: u64) -> u128 {
    (u128::from(network_id.0) << 96) | (u128::from(protocol_id) << 64) | u128::from(nonce)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn amounts() {
        let amount = Amount::new(0, Amount::MAX_PARTS);
        let result = amount.apply(100).unwrap();
        assert_eq!(result, 100);

        let amount = Amount::new(42, Amount::MAX_PARTS);
        let result = amount.apply(100).unwrap();
        assert_eq!(result, 100);

        let amount = Amount::new(123, 0);
        let result = amount.apply(100).unwrap();
        assert_eq!(
            result, 100,
            "seems this is feature to ask more but return what is here"
        );

        let amount = Amount::new(42, 0);
        let result = amount.apply(100).unwrap();
        assert_eq!(result, 42);

        let amount = Amount::new(50, Amount::MAX_PARTS / 10);
        let result = amount.apply(100).unwrap();
        assert_eq!(result, 50 + 5, "percentage of remaining");
    }
}
