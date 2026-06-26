pub mod schemas;

use utoipa::{OpenApi, Modify};
use utoipa::openapi::security::{SecurityScheme, HttpAuthScheme, HttpBuilder};

use crate::http::v1::*;

#[derive(OpenApi)]
#[openapi(
    paths(
        chains::add_chain,
        chains::list_chains,
        chains::get_chain,
        chains::update_chain,
        chains::delete_chain,

        chains::tokens::add_token,
        chains::tokens::list_tokens,
        chains::tokens::get_token,
        chains::tokens::delete_token,

        api_keys::create_key,
        api_keys::list_api_keys,
        api_keys::get_key_record,
        api_keys::revoke_key,

        checkout::invoice_ws_handler,
        checkout::get_checkout_invoice,
        checkout::get_checkout_invoice_payments,
        external::image_proxy,

        invoices::create_invoice,
        invoices::list_invoices,
        invoices::get_invoice,
        invoices::cancel_invoice,

        payments::list_payments,
        payments::get_payment,

        webhooks::list_webhooks,
        webhooks::get_webhook,
    ),
    components(
        schemas(
            schemas::ErrorObjectSchema,
            schemas::ErrorResponseSchema,

            schemas::ChainTypeSchema,
            schemas::ChainDataSchema,
            schemas::PartialChainUpdateSchema,
            schemas::ChainConfigSchema,
            schemas::VecResponseSchema<schemas::ChainDataSchema>,

            schemas::TokenConfigSchema,
            schemas::TokenDataSchema,
            schemas::VecResponseSchema<schemas::TokenDataSchema>,

            schemas::PermissionSchema,
            schemas::ApiKeyRecordSchema,
            schemas::CreateKeyReqSchema,
            schemas::CreateKeyResSchema,
            schemas::RevokeKeyResSchema,

            schemas::InvoiceStatusSchema,
            schemas::PaymentStatusSchema,
            schemas::PublicInvoiceModelSchema,
            schemas::PublicPaymentModelSchema,
            schemas::PaginatedVecPageSchema<schemas::PublicPaymentModelSchema>,

            schemas::WebhookConfigSchema,
            schemas::CreateInvoiceReqSchema,
            schemas::InvoiceSchema,
            schemas::PaginatedVecPageSchema<schemas::InvoiceSchema>,

            schemas::PaymentSchema,
            schemas::PaginatedVecPageSchema<schemas::PaymentSchema>,

            schemas::WebhookStatusSchema,
            schemas::WebhookEventSchema,
            schemas::WebhookSchema,
            schemas::PaginatedVecPageSchema<schemas::WebhookSchema>,
        )
    ),
    modifiers(&SecurityAddon),
    tags(
        (name = "checkout", description = "Public endpoints for checkout pages and customers. Safe to expose."),
        (name = "auth", description = "Management of API Keys and permissions"),
        (name = "chains", description = "Configuration of blockchain networks and supported tokens"),
        (name = "invoices", description = "Creation and management of cryptocurrency invoices"),
        (name = "payments", description = "Internal ledger of all detected blockchain transactions"),
        (name = "webhooks", description = "Delivery logs and statuses for outgoing webhook events")
    )
)]
pub struct ApiDoc;

struct SecurityAddon;

impl Modify for SecurityAddon {
    fn modify(&self, openapi: &mut utoipa::openapi::OpenApi) {
        if let Some(components) = openapi.components.as_mut() {
            components.add_security_scheme(
                "bearer_auth",
                SecurityScheme::Http(
                    HttpBuilder::new()
                        .scheme(HttpAuthScheme::Bearer)
                        .bearer_format("sk_live_ / pk_live_")
                        .description(Some("Authenticate requests using your API key. Prefix it with `Bearer `, e.g., `Bearer sk_live_123`"))
                        .build(),
                ),
            );
        }
    }
}