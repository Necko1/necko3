use utoipa::{IntoParams, IntoResponses, ToSchema};
use serde::Serialize;
use std::collections::HashSet;
use chrono::{DateTime, Utc};
use serde_json::json;
use uuid::Uuid;

#[derive(Serialize, ToSchema)]
#[schema(title = "ErrorObject")]
pub struct ErrorObjectSchema {
    /// The type of error returned.
    #[schema(example = "invalid_request_error")]
    pub err_type: String,

    /// A short code indicating the specific error.
    #[schema(example = "resource_missing")]
    pub code: Option<String>,

    /// A human-readable message providing more details about the error.
    #[schema(example = "Chain 'polygon' not found")]
    pub message: String,

    /// The parameter that caused the error, if applicable.
    #[schema(example = "network")]
    pub param: Option<String>,
}

#[derive(IntoResponses)]
pub enum CommonErrors {
    /// Bad request - Validation error
    #[response(
        status = 400,
        example = json!({
            "error": {
                "type": "invalid_request_error",
                "code": "parameter_invalid",
                "message": "Invalid value 'zzz123' for parameter 'id': UUID parsing failed: invalid character: found `z` at 0",
                "param": "id"
            }
        })
    )]
    BadRequest(ErrorResponseSchema),

    /// Unauthorized - Invalid or missing API key
    #[response(
        status = 401,
        example = json!({
            "error": {
                "type": "invalid_request_error",
                "code": "api_key_invalid",
                "message": "The provided API key is invalid."
            }
        })
    )]
    Unauthorized(ErrorResponseSchema),

    /// Forbidden - Insufficient permissions or API key disabled
    #[response(
        status = 403,
        example = json!({
            "error": {
                "type": "invalid_request_error",
                "code": "permission_denied",
                "message": "You do not have the required permissions to access this resource."
            }
        })
    )]
    Forbidden(ErrorResponseSchema),

    /// Resource not found
    #[response(
        status = 404,
        example = json!({
            "error": {
                "type": "invalid_request_error",
                "code": "resource_missing",
                "message": "Resource '123e4567-e89b-12d3-a456-426614174000' not found."
            }
        })
    )]
    NotFound(ErrorResponseSchema),

    /// Internal Server Error
    #[response(
        status = 500,
        example = json!({
            "error": {
                "type": "api_error",
                "message": "An internal server error occurred."
            }
        })
    )]
    Internal(ErrorResponseSchema),
}

#[derive(Serialize, ToSchema)]
#[schema(title = "ErrorResponse")]
pub struct ErrorResponseSchema {
    /// The error object containing details about the failure.
    pub error: ErrorObjectSchema,
}

#[derive(Serialize, ToSchema)]
pub struct VecResponseSchema<T> {
    /// Array containing the requested resources.
    pub items: Vec<T>,
}

#[derive(Serialize, ToSchema)]
pub struct PaginatedVecPageSchema<T> {
    /// Array containing the requested resources for the current page.
    pub items: Vec<T>,
    /// Total number of items available across all pages.
    #[schema(example = 150)]
    pub total: u64,
    /// Number of items returned per page.
    #[schema(example = 20)]
    pub page_size: u32,
    /// Current page number (1-indexed).
    #[schema(example = 1)]
    pub page: u64,
}

#[derive(Serialize, ToSchema)]
#[schema(title = "ChainType")]
pub enum ChainTypeSchema {
    EVM
}

#[derive(Serialize, ToSchema)]
#[schema(title = "ChainData")]
pub struct ChainDataSchema {
    /// Internal system ID for the network.
    #[schema(example = 1)]
    pub id: i32,

    /// Unique canonical identifier of the network.
    #[schema(example = "ethereum")]
    pub name: String,

    /// Indicates whether the network is currently active and processing events.
    #[schema(example = true)]
    pub active: bool,

    /// List of RPC endpoint URLs used to interact with the network.
    #[schema(example = json!(["https://eth-mainnet.g.alchemy.com/v2/...", "https://rpc.ankr.com/eth"]))]
    pub rpc_urls: Vec<String>,

    pub chain_type: ChainTypeSchema,

    /// Extended public key used to derive HD wallet addresses for this network.
    #[schema(example = "xpub6CUGRUonZSQ4TWtTMmzXdrZWMi...")]
    pub xpub: String,

    /// The native currency symbol of the network.
    #[schema(example = "ETH")]
    pub native_symbol: String,

    /// Number of decimal places for the native currency
    #[schema(example = 18)]
    pub decimals: u8,

    /// The block number that was last processed by the indexer.
    #[schema(example = 19432890)]
    pub last_processed_block: u64,

    /// Current delay in blocks between the node and the tip.
    #[schema(example = 0)]
    pub block_lag: u8,

    /// Minimum block depth required to trust empty log responses from RPC (prevents false-empty logs)
    #[schema(example = 12)]
    pub safe_lag: u8,

    /// Number of blocks required to consider a transaction fully confirmed.
    #[schema(example = 64)]
    pub required_confirmations: u64,

    /// URL pointing to the network's logo icon.
    #[schema(example = "https://cryptologos.cc/logos/ethereum-eth-logo.png")]
    pub logo_url: Option<String>,

    /// List of explicitly watched addresses on this network.
    #[schema(example = json!(["0x1234567890abcdef1234567890abcdef12345678"]))]
    pub watch_addresses: HashSet<String>,
}

#[derive(Serialize, ToSchema)]
#[schema(title = "PartialChainUpdate")]
pub struct PartialChainUpdateSchema {
    /// Set to true/false to enable or disable the network.
    #[schema(example = false)]
    pub active: Option<bool>,

    /// Overwrite the list of RPC URLs.
    #[schema(example = json!(["https://cloudflare-eth.com"]))]
    pub rpc_urls: Option<Vec<String>>,

    /// Forcefully override the last processed block pointer.
    #[schema(example = 19432000)]
    pub last_processed_block: Option<u64>,

    /// Update the extended public key.
    #[schema(example = "xpub...")]
    pub xpub: Option<String>,

    /// Update the block lag threshold.
    #[schema(example = 2)]
    pub block_lag: Option<u8>,

    /// Update the safe lag depth.
    #[schema(example = 12)]
    pub safe_lag: Option<u8>,

    /// Update the required block confirmations.
    #[schema(example = 32)]
    pub required_confirmations: Option<u64>,

    /// Update the network logo URL.
    #[schema(example = "https://example.com/logo.png")]
    pub logo_url: Option<String>,
}

#[derive(Serialize, ToSchema)]
#[schema(title = "ChainConfig")]
pub struct ChainConfigSchema {
    /// Unique canonical identifier for the new network.
    #[schema(example = "arbitrum")]
    pub name: String,

    /// Indicates whether the network should be active immediately upon creation.
    #[schema(example = true)]
    pub active: bool,

    /// List of RPC endpoint URLs used to interact with the network.
    #[schema(example = json!(["https://arb1.arbitrum.io/rpc"]))]
    pub rpc_urls: Vec<String>,

    pub chain_type: ChainTypeSchema,

    /// Extended public key used to derive HD wallet addresses.
    #[schema(example = "xpub6CUGRUonZSQ4TWtTMmzXdrZWMi...")]
    pub xpub: String,

    /// The native currency symbol.
    #[schema(example = "ETH")]
    pub native_symbol: String,

    /// Number of decimal places for the native currency.
    #[schema(example = 18)]
    pub decimals: u8,

    /// The starting block number for the indexer. Use 0 to start from the current tip.
    #[schema(example = 0)]
    pub last_processed_block: u64,

    /// Delay in blocks between the node and the tip.
    #[schema(example = 3)]
    pub block_lag: u8,

    /// Minimum block depth required to trust empty log responses from RPC (prevents false-empty logs)
    #[schema(example = 15)]
    pub safe_lag: u8,

    /// Number of blocks required to consider a transaction fully confirmed.
    #[schema(example = 5)]
    pub required_confirmations: u64,

    /// Optional URL pointing to the network's logo icon.
    #[schema(example = "https://cryptologos.cc/logos/arbitrum-arb-logo.png")]
    pub logo_url: Option<String>,

    /// Initial list of tokens (e.g., ERC20) to track on this network.
    pub tokens: Vec<TokenConfigSchema>,
}

#[derive(Serialize, ToSchema)]
#[schema(title = "TokenConfig")]
pub struct TokenConfigSchema {
    /// The ticker symbol of the token (e.g., 'USDT').
    #[schema(example = "USDT")]
    pub symbol: String,

    /// The smart contract address of the token on the network.
    #[schema(example = "0xdAC17F958D2ee523a2206206994597C13D831ec7")]
    pub contract: String,

    /// Number of decimal places the token uses.
    #[schema(example = 6)]
    pub decimals: u8,

    /// Optional URL pointing to the token's logo icon.
    #[schema(example = "https://cryptologos.cc/logos/tether-usdt-logo.png")]
    pub logo_url: Option<String>,
}

#[derive(Serialize, ToSchema)]
#[schema(title = "TokenData")]
pub struct TokenDataSchema {
    /// Internal system ID for the token.
    #[schema(example = 1)]
    pub id: i32,

    /// Internal system ID of the chain this token belongs to.
    #[schema(example = 1)]
    pub chain_id: i32,

    /// The ticker symbol of the token.
    #[schema(example = "USDT")]
    pub symbol: String,

    /// The smart contract address of the token.
    #[schema(example = "0xdAC17F958D2ee523a2206206994597C13D831ec7")]
    pub contract: String,

    /// Number of decimal places the token uses.
    #[schema(example = 6)]
    pub decimals: u8,

    /// URL pointing to the token's logo icon.
    #[schema(example = "https://cryptologos.cc/logos/tether-usdt-logo.png")]
    pub logo_url: Option<String>,
}

#[derive(Serialize, ToSchema)]
#[schema(title = "Permission")]
pub enum PermissionSchema {
    FullAccess,
    WriteInvoices,
    ReadInvoices,
    PublicRead,
}

#[derive(Serialize, ToSchema)]
#[schema(title = "ApiKeyRecord")]
pub struct ApiKeyRecordSchema {
    /// The unique identifier (UUID) of the API key record.
    #[schema(example = "123e4567-e89b-12d3-a456-426614174000")]
    pub id: Uuid,

    /// Human-readable identifier for this key.
    #[schema(example = "Payment Gateway Key")]
    pub name: String,

    /// The environment prefix for the key.
    #[schema(example = "sk_live")]
    pub prefix: String,

    /// List of permissions granted to this key.
    #[schema(example = json!(["write_invoices", "read_invoices"]))]
    pub permissions: Vec<PermissionSchema>,

    /// Indicates whether the key is currently active and can be used for authentication.
    #[schema(example = true)]
    pub is_active: bool,

    /// Timestamp indicating when the key was created.
    #[schema(example = "2026-06-26T15:00:00Z")]
    pub created_at: DateTime<Utc>,
}

#[derive(Serialize, ToSchema)]
#[schema(title = "ApiKeyPrefix")]
pub enum ApiKeyPrefixSchema {
    #[serde(rename = "sk_live")]
    SkLive,
    #[serde(rename = "pk_live")]
    PkLive,
}

#[derive(Serialize, ToSchema)]
#[schema(title = "CreateKeyReq")]
pub struct CreateKeyReqSchema {
    /// Human-readable identifier for this key.
    #[schema(example = "Payment Gateway Key")]
    pub name: String,

    /// The environment prefix for the key (secret or publishable).
    pub prefix: ApiKeyPrefixSchema,

    /// List of permissions granted to this key. Publishable keys (`pk_live`) can only have `public_read`.
    #[schema(example = json!(["write_invoices", "read_invoices"]))]
    pub permissions: Vec<PermissionSchema>,
}

#[derive(Serialize, ToSchema)]
#[schema(title = "CreateKeyRes")]
pub struct CreateKeyResSchema {
    /// The raw secret API key. **Store this securely, it will not be shown again.**
    #[schema(example = "sk_live_abc123def456ghi789")]
    pub raw_key: String,

    /// The UUID of the generated key record.
    #[schema(example = "123e4567-e89b-12d3-a456-426614174000")]
    pub key_id: Uuid,
}

#[derive(Serialize, ToSchema)]
#[schema(title = "RevokeKeyRes")]
pub struct RevokeKeyResSchema {
    /// The API key record that was just revoked.
    #[serde(flatten)]
    pub record: ApiKeyRecordSchema,

    /// Timestamp indicating exactly when the key was deactivated.
    #[schema(example = "2026-06-26T15:00:00Z")]
    pub revoked_at: DateTime<Utc>,
}

#[derive(Serialize, ToSchema)]
#[schema(title = "InvoiceStatus")]
pub enum InvoiceStatusSchema {
    Pending,
    Paid,
    Expired,
    Cancelled,
}

#[derive(Serialize, ToSchema)]
#[schema(title = "PaymentStatus")]
pub enum PaymentStatusSchema {
    Pending,
    Confirming,
    Confirmed,
    Failed,
}

#[derive(Serialize, ToSchema)]
#[schema(title = "PublicInvoiceModel")]
pub struct PublicInvoiceModelSchema {
    #[schema(example = "123e4567-e89b-12d3-a456-426614174000")]
    pub id: Uuid,
    #[schema(example = "0x742d35Cc6634C0532925a3b844Bc454e4438f44e")]
    pub address: String,
    #[schema(example = "150.50")]
    pub amount: String,
    #[schema(example = "0.00")]
    pub paid: String,
    #[schema(example = "USDT")]
    pub token: String,
    #[schema(example = "ethereum")]
    pub network: String,
    #[schema(example = "2026-06-26T15:00:00Z")]
    pub created_at: DateTime<Utc>,
    #[schema(example = "2026-06-26T16:00:00Z")]
    pub expires_at: DateTime<Utc>,
    pub status: InvoiceStatusSchema,
}

#[derive(Serialize, ToSchema)]
#[schema(title = "PublicPaymentModel")]
pub struct PublicPaymentModelSchema {
    #[schema(example = "f47ac10b-58cc-4372-a567-0e02b2c3d479")]
    pub id: Uuid,
    #[schema(example = "0x123...abc")]
    pub from: String,
    #[schema(example = "0x742d35Cc6634C0532925a3b844Bc454e4438f44e")]
    pub to: String,
    #[schema(example = "ethereum")]
    pub network: String,
    #[schema(example = "USDT")]
    pub token: String,
    #[schema(example = "0x987...def")]
    pub tx_hash: String,
    #[schema(example = "150.50")]
    pub amount: String,
    #[schema(example = 19432890)]
    pub block_number: u64,
    #[schema(example = "0xabc...123")]
    pub block_hash: String,
    pub status: PaymentStatusSchema,
    #[schema(example = "2026-06-26T15:10:00Z")]
    pub created_at: DateTime<Utc>,
}

/// Query parameters for pagination.
#[derive(IntoParams)]
#[into_params(parameter_in = Query)]
pub struct QueryPaginationParams {
    /// Number of items to return per page.
    #[param(default = 20, minimum = 1, maximum = 100)]
    pub page_size: Option<u32>,

    /// Page number to retrieve (1-indexed).
    #[param(default = 1, minimum = 1)]
    pub page: Option<u64>,
}

#[derive(Serialize, ToSchema)]
#[schema(title = "WebhookConfig")]
pub struct WebhookConfigSchema {
    /// The URL that will receive POST requests for invoice events.
    #[schema(example = "https://your-domain.com/webhooks/crypto")]
    pub url: String,

    /// Optional secret key used to sign the webhook payload (HMAC SHA256).
    #[schema(example = "whsec_abc123...")]
    pub secret: Option<String>,

    /// Maximum number of retry attempts if the delivery fails.
    #[schema(example = 3)]
    pub max_retries: Option<u32>,
}

#[derive(Serialize, ToSchema)]
#[schema(title = "CreateInvoiceReq")]
pub struct CreateInvoiceReqSchema {
    /// The target blockchain network.
    #[schema(example = "ethereum")]
    pub network: String,

    /// The asset to be paid (native coin symbol or token ticker).
    #[schema(example = "USDT")]
    pub asset: String,

    /// The exact amount of the asset expected to be paid.
    #[schema(example = 150.50)]
    pub amount: f64,

    /// Duration in seconds before the invoice expires (default is 900 seconds / 15 minutes).
    #[schema(example = 3600)]
    pub duration: u64,

    /// Configuration for delivering real-time webhook updates about this specific invoice.
    pub webhook_config: Option<WebhookConfigSchema>,
}

#[derive(Serialize, ToSchema)]
#[schema(title = "Invoice")]
pub struct InvoiceSchema {
    #[schema(example = "123e4567-e89b-12d3-a456-426614174000")]
    pub id: Uuid,

    /// Internal index used for HD wallet address derivation.
    #[schema(example = 42)]
    pub address_index: u32,

    /// The unique cryptocurrency address generated for this invoice.
    #[schema(example = "0x742d35Cc6634C0532925a3b844Bc454e4438f44e")]
    pub address: String,

    /// Human-readable target amount.
    #[schema(example = "150.50")]
    pub amount: String,

    /// Raw target amount in minimal indivisible units (e.g., Wei). Passed as a string to prevent precision loss.
    #[schema(value_type = String, example = "150500000")]
    pub amount_raw: String,

    /// Human-readable amount that has been paid so far.
    #[schema(example = "0.00")]
    pub paid: String,

    /// Raw paid amount in minimal units. Passed as a string.
    #[schema(value_type = String, example = "0")]
    pub paid_raw: String,

    #[schema(example = "USDT")]
    pub token: String,

    #[schema(example = "ethereum")]
    pub network: String,

    #[schema(example = 6)]
    pub decimals: u8,

    #[schema(example = "https://your-domain.com/webhooks/crypto")]
    pub webhook_url: Option<String>,

    #[schema(example = "whsec_abc123...")]
    pub webhook_secret: Option<String>,

    #[schema(example = 3)]
    pub webhook_max_retries: Option<u32>,

    #[schema(example = "2026-06-26T15:00:00Z")]
    pub created_at: DateTime<Utc>,

    #[schema(example = "2026-06-26T16:00:00Z")]
    pub expires_at: DateTime<Utc>,

    pub status: InvoiceStatusSchema, // Ссылка на статус, который мы делали ранее
}

/// Query parameters for filtering invoices.
#[derive(IntoParams)]
#[into_params(parameter_in = Query)]
pub struct QueryInvoiceFilterParams {
    /// Filter by current invoice status.
    pub status: Option<InvoiceStatusSchema>,

    /// Filter by the destination payment address.
    pub address: Option<String>,

    /// Filter by the target blockchain network.
    pub network: Option<String>,

    /// Filter by the requested token.
    pub token: Option<String>,
}

#[derive(Serialize, ToSchema)]
#[schema(title = "Payment")]
pub struct PaymentSchema {
    #[schema(example = "f47ac10b-58cc-4372-a567-0e02b2c3d479")]
    pub id: Uuid,

    /// The sender's wallet address.
    #[schema(example = "0x123...abc")]
    pub from: String,

    /// The receiver's wallet address (usually the invoice address).
    #[schema(example = "0x742d35Cc6634C0532925a3b844Bc454e4438f44e")]
    pub to: String,

    /// The canonical name of the blockchain network.
    #[schema(example = "ethereum")]
    pub network: String,

    /// The ticker symbol of the transferred token.
    #[schema(example = "USDT")]
    pub token: String,

    /// The transaction hash on the blockchain.
    #[schema(example = "0x987...def")]
    pub tx_hash: String,

    /// Raw transferred amount in minimal indivisible units (e.g., Wei). Passed as a string to prevent precision loss.
    #[schema(value_type = String, example = "150500000")]
    pub amount_raw: String,

    /// The block number in which the transaction was included.
    #[schema(example = 19432890)]
    pub block_number: u64,

    /// The hash of the block containing the transaction.
    #[schema(example = "0xabc...123")]
    pub block_hash: String,

    /// The log index of the event within the block (for EVM-compatible chains).
    #[schema(example = 12)]
    pub log_index: Option<u64>,

    pub status: PaymentStatusSchema,

    #[schema(example = "2026-06-26T15:10:00Z")]
    pub created_at: DateTime<Utc>,
}

/// Query parameters for filtering payments.
#[derive(IntoParams)]
#[into_params(parameter_in = Query)]
pub struct QueryPaymentFilterParams {
    /// Filter by the sender's address.
    pub from: Option<String>,

    /// Filter by the receiver's address.
    pub to: Option<String>,

    /// Filter by the target blockchain network.
    pub network: Option<String>,

    /// Filter by the transferred token.
    pub token: Option<String>,

    /// Filter by a specific block number.
    pub block_number: Option<u64>,

    /// Filter by a specific block hash.
    pub block_hash: Option<String>,

    /// Filter by current payment status (e.g., Confirming, Confirmed).
    pub status: Option<PaymentStatusSchema>,
}

#[derive(Serialize, ToSchema)]
#[schema(title = "WebhookStatus")]
pub enum WebhookStatusSchema {
    Pending,
    Processing,
    Delivered,
    Failed,
}

/// The specific event payload that will be sent to the webhook URL.
#[derive(Serialize, ToSchema)]
#[serde(tag = "event_type", content = "data", rename_all = "snake_case")]
#[schema(title = "WebhookEvent")]
pub enum WebhookEventSchema {
    /// Fired immediately when a transaction to the invoice address is detected on the network (0 confirmations).
    TxDetected {
        #[schema(example = "123e4567-e89b-12d3-a456-426614174000")]
        invoice_id: Uuid,
        #[schema(example = "0x987...def")]
        tx_hash: String,
        #[schema(example = "150.50")]
        amount: String,
        #[schema(example = "USDT")]
        currency: String,
    },
    /// Fired when a transaction reaches the required number of confirmations.
    TxConfirmed {
        #[schema(example = "123e4567-e89b-12d3-a456-426614174000")]
        invoice_id: Uuid,
        #[schema(example = "0x987...def")]
        tx_hash: String,
        #[schema(example = 5)]
        confirmations: u64,
    },
    /// Fired when the invoice is fully paid (total confirmed payments >= target amount).
    InvoicePaid {
        #[schema(example = "123e4567-e89b-12d3-a456-426614174000")]
        invoice_id: Uuid,
        #[schema(example = "150.50")]
        paid_amount: String,
    },
    /// Fired when the invoice's time-to-live expires before being fully paid.
    InvoiceExpired {
        #[schema(example = "123e4567-e89b-12d3-a456-426614174000")]
        invoice_id: Uuid,
    },
    /// Fired when the invoice is manually cancelled via the API.
    InvoiceCancelled {
        #[schema(example = "123e4567-e89b-12d3-a456-426614174000")]
        invoice_id: Uuid,
    }
}

#[derive(Serialize, ToSchema)]
#[schema(title = "Webhook")]
pub struct WebhookSchema {
    #[schema(example = "987e6543-e21b-34d3-b567-526614174999")]
    pub id: Uuid,

    /// The ID of the invoice that triggered this webhook.
    #[schema(example = "123e4567-e89b-12d3-a456-426614174000")]
    pub invoice_id: Uuid,

    /// The destination URL where the webhook payload is sent.
    #[schema(example = "https://your-domain.com/webhooks/crypto")]
    pub url: String,

    /// The event data payload.
    pub payload: WebhookEventSchema,

    pub status: WebhookStatusSchema,

    /// The current number of delivery attempts made.
    #[schema(example = 1)]
    pub attempts: u32,

    /// The maximum number of retries allowed before permanently failing.
    #[schema(example = 3)]
    pub max_retries: u32,

    /// Timestamp indicating when the next delivery attempt will occur (if pending/processing).
    #[schema(example = "2026-06-26T15:05:00Z")]
    pub next_retry: DateTime<Utc>,

    #[schema(example = "2026-06-26T15:00:00Z")]
    pub created_at: DateTime<Utc>,
}

/// Query parameters for filtering webhook delivery logs.
#[derive(IntoParams)]
#[into_params(parameter_in = Query)]
pub struct QueryWebhookFilterParams {
    /// Filter webhooks by the associated invoice ID.
    pub invoice_id: Option<Uuid>,

    /// Filter by the type of event (e.g., 'tx_detected', 'invoice_paid').
    pub event_type: Option<String>,

    /// Filter by the destination URL.
    pub url: Option<String>,

    /// Filter by the current delivery status.
    pub status: Option<WebhookStatusSchema>,
}

#[derive(IntoParams)]
#[into_params(parameter_in = Query)]
pub struct ImageProxyParamsSchema {
    /// The absolute HTTP(S) URL of the external image to be proxied.
    #[param(example = "https://cryptologos.cc/logos/ethereum-eth-logo.png")]
    pub url: String,
}