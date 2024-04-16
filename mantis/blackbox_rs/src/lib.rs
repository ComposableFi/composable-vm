#[allow(unused_imports)]
use progenitor_client::{encode_path, RequestBuilderExt};
#[allow(unused_imports)]
pub use progenitor_client::{ByteStream, Error, ResponseValue};
#[allow(unused_imports)]
use reqwest::header::{HeaderMap, HeaderValue};
/// Types used as operation parameters and responses.
#[allow(clippy::all)]
pub mod types {
    use serde::{Deserialize, Serialize};
    #[allow(unused_imports)]
    use std::convert::TryFrom;
    /// Error types.
    pub mod error {
        /// Error from a TryFrom or FromStr implementation.
        pub struct ConversionError(std::borrow::Cow<'static, str>);
        impl std::error::Error for ConversionError {}
        impl std::fmt::Display for ConversionError {
            fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> Result<(), std::fmt::Error> {
                std::fmt::Display::fmt(&self.0, f)
            }
        }

        impl std::fmt::Debug for ConversionError {
            fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> Result<(), std::fmt::Error> {
                std::fmt::Debug::fmt(&self.0, f)
            }
        }

        impl From<&'static str> for ConversionError {
            fn from(value: &'static str) -> Self {
                Self(value.into())
            }
        }

        impl From<String> for ConversionError {
            fn from(value: String) -> Self {
                Self(value.into())
            }
        }
    }

    ///ExchangeStrStr
    ///
    /// <details><summary>JSON schema</summary>
    ///
    /// ```json
    ///{
    ///  "title": "Exchange[str, str]",
    ///  "type": "object",
    ///  "required": [
    ///    "in_asset_amount",
    ///    "in_asset_id",
    ///    "next",
    ///    "out_asset_amount",
    ///    "out_asset_id",
    ///    "pool_id"
    ///  ],
    ///  "properties": {
    ///    "in_asset_amount": {
    ///      "title": "In Asset Amount",
    ///      "type": "string"
    ///    },
    ///    "in_asset_id": {
    ///      "title": "In Asset Id",
    ///      "type": "string"
    ///    },
    ///    "next": {
    ///      "title": "Next",
    ///      "type": "array",
    ///      "items": {
    ///        "anyOf": [
    ///          {
    ///            "$ref": "#/components/schemas/Exchange_str_str_"
    ///          },
    ///          {
    ///            "$ref": "#/components/schemas/Spawn_str_str_"
    ///          }
    ///        ]
    ///      }
    ///    },
    ///    "out_asset_amount": {
    ///      "title": "Out Asset Amount",
    ///      "type": "string"
    ///    },
    ///    "out_asset_id": {
    ///      "title": "Out Asset Id",
    ///      "type": "string"
    ///    },
    ///    "pool_id": {
    ///      "title": "Pool Id",
    ///      "type": "string"
    ///    }
    ///  }
    ///}
    /// ```
    /// </details>
    #[derive(Clone, Debug, Deserialize, Serialize)]
    pub struct ExchangeStrStr {
        pub in_asset_amount: String,
        pub in_asset_id: String,
        pub next: Vec<NextItem>,
        pub out_asset_amount: String,
        pub out_asset_id: String,
        pub pool_id: String,
    }

    impl From<&ExchangeStrStr> for ExchangeStrStr {
        fn from(value: &ExchangeStrStr) -> Self {
            value.clone()
        }
    }

    ///HttpValidationError
    ///
    /// <details><summary>JSON schema</summary>
    ///
    /// ```json
    ///{
    ///  "title": "HTTPValidationError",
    ///  "type": "object",
    ///  "properties": {
    ///    "detail": {
    ///      "title": "Detail",
    ///      "type": "array",
    ///      "items": {
    ///        "$ref": "#/components/schemas/ValidationError"
    ///      }
    ///    }
    ///  }
    ///}
    /// ```
    /// </details>
    #[derive(Clone, Debug, Deserialize, Serialize)]
    pub struct HttpValidationError {
        #[serde(default, skip_serializing_if = "Vec::is_empty")]
        pub detail: Vec<ValidationError>,
    }

    impl From<&HttpValidationError> for HttpValidationError {
        fn from(value: &HttpValidationError) -> Self {
            value.clone()
        }
    }

    ///LocationItem
    ///
    /// <details><summary>JSON schema</summary>
    ///
    /// ```json
    ///{
    ///  "anyOf": [
    ///    {
    ///      "type": "string"
    ///    },
    ///    {
    ///      "type": "integer"
    ///    }
    ///  ]
    ///}
    /// ```
    /// </details>
    #[derive(Clone, Debug, Deserialize, Serialize)]
    #[serde(untagged)]
    pub enum LocationItem {
        Variant0(String),
        Variant1(i64),
    }

    impl From<&LocationItem> for LocationItem {
        fn from(value: &LocationItem) -> Self {
            value.clone()
        }
    }

    impl std::str::FromStr for LocationItem {
        type Err = self::error::ConversionError;
        fn from_str(value: &str) -> Result<Self, self::error::ConversionError> {
            if let Ok(v) = value.parse() {
                Ok(Self::Variant0(v))
            } else if let Ok(v) = value.parse() {
                Ok(Self::Variant1(v))
            } else {
                Err("string conversion failed for all variants".into())
            }
        }
    }

    impl std::convert::TryFrom<&str> for LocationItem {
        type Error = self::error::ConversionError;
        fn try_from(value: &str) -> Result<Self, self::error::ConversionError> {
            value.parse()
        }
    }

    impl std::convert::TryFrom<&String> for LocationItem {
        type Error = self::error::ConversionError;
        fn try_from(value: &String) -> Result<Self, self::error::ConversionError> {
            value.parse()
        }
    }

    impl std::convert::TryFrom<String> for LocationItem {
        type Error = self::error::ConversionError;
        fn try_from(value: String) -> Result<Self, self::error::ConversionError> {
            value.parse()
        }
    }

    impl ToString for LocationItem {
        fn to_string(&self) -> String {
            match self {
                Self::Variant0(x) => x.to_string(),
                Self::Variant1(x) => x.to_string(),
            }
        }
    }

    impl From<i64> for LocationItem {
        fn from(value: i64) -> Self {
            Self::Variant1(value)
        }
    }

    ///NextItem
    ///
    /// <details><summary>JSON schema</summary>
    ///
    /// ```json
    ///{
    ///  "anyOf": [
    ///    {
    ///      "$ref": "#/components/schemas/Exchange_str_str_"
    ///    },
    ///    {
    ///      "$ref": "#/components/schemas/Spawn_str_str_"
    ///    }
    ///  ]
    ///}
    /// ```
    /// </details>
    #[derive(Clone, Debug, Deserialize, Serialize)]
    #[serde(untagged)]
    pub enum NextItem {
        ExchangeStrStr(ExchangeStrStr),
        SpawnStrStr(SpawnStrStr),
    }

    impl From<&NextItem> for NextItem {
        fn from(value: &NextItem) -> Self {
            value.clone()
        }
    }

    impl From<ExchangeStrStr> for NextItem {
        fn from(value: ExchangeStrStr) -> Self {
            Self::ExchangeStrStr(value)
        }
    }

    impl From<SpawnStrStr> for NextItem {
        fn from(value: SpawnStrStr) -> Self {
            Self::SpawnStrStr(value)
        }
    }

    ///SingleInputAssetCvmRouteStrStr
    ///
    /// <details><summary>JSON schema</summary>
    ///
    /// ```json
    ///{
    ///  "title": "SingleInputAssetCvmRoute[str, str]",
    ///  "type": "object",
    ///  "required": [
    ///    "in_asset_amount",
    ///    "in_asset_id",
    ///    "next",
    ///    "out_asset_amount",
    ///    "out_asset_id"
    ///  ],
    ///  "properties": {
    ///    "in_asset_amount": {
    ///      "title": "In Asset Amount",
    ///      "type": "string"
    ///    },
    ///    "in_asset_id": {
    ///      "title": "In Asset Id",
    ///      "type": "string"
    ///    },
    ///    "next": {
    ///      "title": "Next",
    ///      "type": "array",
    ///      "items": {
    ///        "anyOf": [
    ///          {
    ///            "$ref": "#/components/schemas/Exchange_str_str_"
    ///          },
    ///          {
    ///            "$ref": "#/components/schemas/Spawn_str_str_"
    ///          }
    ///        ]
    ///      }
    ///    },
    ///    "out_asset_amount": {
    ///      "title": "Out Asset Amount",
    ///      "type": "string"
    ///    },
    ///    "out_asset_id": {
    ///      "title": "Out Asset Id",
    ///      "type": "string"
    ///    }
    ///  }
    ///}
    /// ```
    /// </details>
    #[derive(Clone, Debug, Deserialize, Serialize)]
    pub struct SingleInputAssetCvmRouteStrStr {
        pub in_asset_amount: String,
        pub in_asset_id: String,
        pub next: Vec<NextItem>,
        pub out_asset_amount: String,
        pub out_asset_id: String,
    }

    impl From<&SingleInputAssetCvmRouteStrStr> for SingleInputAssetCvmRouteStrStr {
        fn from(value: &SingleInputAssetCvmRouteStrStr) -> Self {
            value.clone()
        }
    }

    ///SpawnStrStr
    ///
    /// <details><summary>JSON schema</summary>
    ///
    /// ```json
    ///{
    ///  "title": "Spawn[str, str]",
    ///  "type": "object",
    ///  "required": [
    ///    "in_asset_amount",
    ///    "in_asset_id",
    ///    "next",
    ///    "out_asset_amount",
    ///    "out_asset_id"
    ///  ],
    ///  "properties": {
    ///    "in_asset_amount": {
    ///      "title": "In Asset Amount",
    ///      "type": "string"
    ///    },
    ///    "in_asset_id": {
    ///      "title": "In Asset Id",
    ///      "type": "string"
    ///    },
    ///    "next": {
    ///      "title": "Next",
    ///      "type": "array",
    ///      "items": {
    ///        "anyOf": [
    ///          {
    ///            "$ref": "#/components/schemas/Exchange_str_str_"
    ///          },
    ///          {
    ///            "$ref": "#/components/schemas/Spawn_str_str_"
    ///          }
    ///        ]
    ///      }
    ///    },
    ///    "out_asset_amount": {
    ///      "title": "Out Asset Amount",
    ///      "type": "string"
    ///    },
    ///    "out_asset_id": {
    ///      "title": "Out Asset Id",
    ///      "type": "string"
    ///    }
    ///  }
    ///}
    /// ```
    /// </details>
    #[derive(Clone, Debug, Deserialize, Serialize)]
    pub struct SpawnStrStr {
        pub in_asset_amount: String,
        pub in_asset_id: String,
        pub next: Vec<NextItem>,
        pub out_asset_amount: String,
        pub out_asset_id: String,
    }

    impl From<&SpawnStrStr> for SpawnStrStr {
        fn from(value: &SpawnStrStr) -> Self {
            value.clone()
        }
    }

    ///ValidationError
    ///
    /// <details><summary>JSON schema</summary>
    ///
    /// ```json
    ///{
    ///  "title": "ValidationError",
    ///  "type": "object",
    ///  "required": [
    ///    "loc",
    ///    "msg",
    ///    "type"
    ///  ],
    ///  "properties": {
    ///    "loc": {
    ///      "title": "Location",
    ///      "type": "array",
    ///      "items": {
    ///        "anyOf": [
    ///          {
    ///            "type": "string"
    ///          },
    ///          {
    ///            "type": "integer"
    ///          }
    ///        ]
    ///      }
    ///    },
    ///    "msg": {
    ///      "title": "Message",
    ///      "type": "string"
    ///    },
    ///    "type": {
    ///      "title": "Error Type",
    ///      "type": "string"
    ///    }
    ///  }
    ///}
    /// ```
    /// </details>
    #[derive(Clone, Debug, Deserialize, Serialize)]
    pub struct ValidationError {
        pub loc: Vec<LocationItem>,
        pub msg: String,
        #[serde(rename = "type")]
        pub type_: String,
    }

    impl From<&ValidationError> for ValidationError {
        fn from(value: &ValidationError) -> Self {
            value.clone()
        }
    }
}

#[derive(Clone, Debug)]
///Client for FastAPI
///
///Version: 0.1.0
pub struct Client {
    pub(crate) baseurl: String,
    pub(crate) client: reqwest::Client,
}

impl Client {
    /// Create a new client.
    ///
    /// `baseurl` is the base URL provided to the internal
    /// `reqwest::Client`, and should include a scheme and hostname,
    /// as well as port and a path stem if applicable.
    pub fn new(baseurl: &str) -> Self {
        #[cfg(not(target_arch = "wasm32"))]
        let client = {
            let dur = std::time::Duration::from_secs(15);
            reqwest::ClientBuilder::new()
                .connect_timeout(dur)
                .timeout(dur)
        };
        #[cfg(target_arch = "wasm32")]
        let client = reqwest::ClientBuilder::new();
        Self::new_with_client(baseurl, client.build().unwrap())
    }

    /// Construct a new client with an existing `reqwest::Client`,
    /// allowing more control over its configuration.
    ///
    /// `baseurl` is the base URL provided to the internal
    /// `reqwest::Client`, and should include a scheme and hostname,
    /// as well as port and a path stem if applicable.
    pub fn new_with_client(baseurl: &str, client: reqwest::Client) -> Self {
        Self {
            baseurl: baseurl.to_string(),
            client,
        }
    }

    /// Get the base URL to which requests are made.
    pub fn baseurl(&self) -> &String {
        &self.baseurl
    }

    /// Get the internal `reqwest::Client` used to make requests.
    pub fn client(&self) -> &reqwest::Client {
        &self.client
    }

    /// Get the version of this API.
    ///
    /// This string is pulled directly from the source OpenAPI
    /// document and may be in any format the API selects.
    pub fn api_version(&self) -> &'static str {
        "0.1.0"
    }
}

#[allow(clippy::all)]
impl Client {
    ///Simulator Router
    ///
    ///_summary_
    ///Given input, find and return route.
    ///
    ///Sends a `GET` request to `/simulator/router`
    pub async fn simulator_router_simulator_router_get<'a>(
        &'a self,
        in_asset_amount: Option<&'a str>,
        in_asset_id: Option<&'a str>,
        max: Option<bool>,
        out_asset_amount: &'a str,
        out_asset_id: Option<&'a str>,
    ) -> Result<
        ResponseValue<Vec<types::SingleInputAssetCvmRouteStrStr>>,
        Error<types::HttpValidationError>,
    > {
        let url = format!("{}/simulator/router", self.baseurl,);
        let mut query = Vec::with_capacity(5usize);
        if let Some(v) = &in_asset_amount {
            query.push(("in_asset_amount", v.to_string()));
        }

        if let Some(v) = &in_asset_id {
            query.push(("in_asset_id", v.to_string()));
        }

        if let Some(v) = &max {
            query.push(("max", v.to_string()));
        }

        query.push(("out_asset_amount", out_asset_amount.to_string()));
        if let Some(v) = &out_asset_id {
            query.push(("out_asset_id", v.to_string()));
        }

        #[allow(unused_mut)]
        let mut request = self
            .client
            .get(url)
            .header(
                reqwest::header::ACCEPT,
                reqwest::header::HeaderValue::from_static("application/json"),
            )
            .query(&query)
            .build()?;
        let result = self.client.execute(request).await;
        let response = result?;
        match response.status().as_u16() {
            200u16 => ResponseValue::from_response(response).await,
            422u16 => Err(Error::ErrorResponse(
                ResponseValue::from_response(response).await?,
            )),
            _ => Err(Error::UnexpectedResponse(response)),
        }
    }
}

/// Items consumers will typically use such as the Client.
pub mod prelude {
    #[allow(unused_imports)]
    pub use super::Client;
}
