use crate::prelude::*;

/// A wrapper around a type which is serde-serialised as a string.
///
/// For serde-serialisation to be implemented for the type `T` must implement
/// `Display` and `FromStr` traits.
///
/// ```rust
/// use cvm::shared::Displayed;
///
/// #[derive(serde::Serialize, serde::Deserialize)]
/// struct Foo {
///     value: Displayed<u64>
/// }
///
/// let encoded = serde_json_wasm::to_string(&Foo { value: Displayed(42) }).unwrap();
/// assert_eq!(r#"{"value":"42"}"#, encoded);
///
/// let decoded = serde_json_wasm::from_str::<Foo>(r#"{"value":"42"}"#).unwrap();
/// assert_eq!(Displayed(42), decoded.value);
/// ```
#[cfg_attr(
    feature = "json-schema", // all(feature = "json-schema", not(target_arch = "wasm32")),
    derive(schemars::JsonSchema)
)]
#[derive(
    Copy,
    Clone,
    Default,
    PartialEq,
    Eq,
    PartialOrd,
    Ord,
    Hash,
    derive_more::Deref,
    derive_more::From,
)]
#[repr(transparent)]
pub struct Displayed<T>(
    #[cfg_attr(
        feature = "json-schema", // all(feature = "json-schema", not(target_arch = "wasm32")),
        schemars(with = "String")
    )]
    pub T,
);

impl<T: FromStr> FromStr for Displayed<T> {
    type Err = <T as FromStr>::Err;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        T::from_str(s).map(Displayed)
    }
}

#[cfg(feature = "parity-scale-codec")]
impl<T> parity_scale_codec::WrapperTypeEncode for Displayed<T> {}
#[cfg(feature = "parity-scale-codec")]
impl<T> parity_scale_codec::WrapperTypeDecode for Displayed<T> {
    type Wrapped = T;
}

impl<T: core::fmt::Display> core::fmt::Display for Displayed<T> {
    fn fmt(&self, fmtr: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        core::fmt::Display::fmt(&self.0, fmtr)
    }
}

impl<T: core::fmt::Display> core::fmt::Debug for Displayed<T> {
    fn fmt(&self, fmtr: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        core::fmt::Display::fmt(&self.0, fmtr)
    }
}

impl<T: core::fmt::Display> serde::Serialize for Displayed<T> {
    fn serialize<S: serde::Serializer>(&self, ser: S) -> Result<S::Ok, S::Error> {
        ser.collect_str(&self.0)
    }
}

impl<'de, T> serde::Deserialize<'de> for Displayed<T>
where
    T: core::str::FromStr,
    <T as core::str::FromStr>::Err: core::fmt::Display,
{
    fn deserialize<D: serde::Deserializer<'de>>(de: D) -> Result<Self, D::Error> {
        de.deserialize_str(DisplayedVisitor::<T>(Default::default()))
    }
}

/// Serde Visitor helper for deserialising [`Displayed`] type.
struct DisplayedVisitor<V>(core::marker::PhantomData<V>);

impl<'de, T> serde::de::Visitor<'de> for DisplayedVisitor<T>
where
    T: core::str::FromStr,
    <T as core::str::FromStr>::Err: core::fmt::Display,
{
    type Value = Displayed<T>;

    fn expecting(&self, fmt: &mut core::fmt::Formatter) -> core::fmt::Result {
        fmt.write_str("a string")
    }

    fn visit_str<E: serde::de::Error>(self, s: &str) -> Result<Self::Value, E> {
        T::from_str(s).map(Displayed).map_err(E::custom)
    }
}

macro_rules! impl_conversions {
	($(Displayed<$inner:ty> => $other:ty),*) => {
		$(
		impl From<Displayed<$inner>> for $other {
			fn from(value: Displayed<$inner>) -> Self {
				<$other>::from(value.0)
			}
		}
			)*
	};

	($($other:ty = Displayed<$inner:ty>),*) => {
		$(
		impl From<$other> for Displayed<$inner> {
			fn from(value: $other) -> Self {
				Self(<$inner>::from(value))
			}
		}

		impl From<Displayed<$inner>> for $other {
			fn from(value: Displayed<$inner>) -> Self {
				<$other>::from(value.0)
			}
		}
			)*
	};
}

// impl prost::Message for Displayed<u64> {
//     fn encoded_len(&self) -> usize {
//         self.0.encoded_len()
//     }

//     fn clear(&mut self) {
//         self.0.clear()
//     }

// 	fn encode_raw<B>(&self, buf: &mut B)
// 		where
// 			B: prost::bytes::BufMut,
// 			Self: Sized {
// 		self.0.encode_raw(buf)
// 	}

// 	fn merge_field<B>(
// 			&mut self,
// 			tag: u32,
// 			wire_type: prost::encoding::WireType,
// 			buf: &mut B,
// 			ctx: prost::encoding::DecodeContext,
// 		) -> Result<(), prost::DecodeError>
// 		where
// 			B: prost::bytes::Buf,
// 			Self: Sized {
// 		self.0.merge_field(tag, wire_type, buf, ctx)
// 	}
// }

// Due to Rust orphan rules it’s not possible to make generic `impl<T>
// From<Displayed<T>> for T` so we’re defining common conversions explicitly.
impl_conversions!(Displayed<u128> => u128, Displayed<u64> => u64);

#[cfg(feature = "cosmwasm")]
impl_conversions!(cosmwasm_std::Uint128 = Displayed<u128>,
                  cosmwasm_std::Uint64 = Displayed<u64>);
impl_conversions!(crate::proto::Uint128 = Displayed<u128>);

impl<T: core::cmp::PartialEq> core::cmp::PartialEq<T> for Displayed<T> {
    fn eq(&self, rhs: &T) -> bool {
        self.0 == *rhs
    }
}
