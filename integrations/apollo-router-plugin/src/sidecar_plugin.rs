use std::ops::ControlFlow;
use std::str::FromStr;
use apollo_router::graphql;
use apollo_router::layers::ServiceBuilderExt;
use apollo_router::plugin::Plugin;
use apollo_router::plugin::PluginInit;
use apollo_router::register_plugin;
use apollo_router::services::supergraph;
use http::StatusCode;
use reqwest::Url;
use schemars::JsonSchema;
use serde::Deserialize;
use serde_json_bytes::Value;
use tower::BoxError;
use tower::ServiceBuilder;
use tower::ServiceExt;

#[derive(Deserialize, JsonSchema)]
struct SidecarProxyConfig {
    addr: String,
}

struct SidecarProxy {
    addr: reqwest::Url,
}

#[async_trait::async_trait]
impl Plugin for SidecarProxy {
    type Config = SidecarProxyConfig;

    async fn new(init: PluginInit<Self::Config>) -> Result<Self, BoxError> {
        let SidecarProxyConfig { addr } = init.config;
        let url = Url::from_str(&addr)?;
        Ok(Self { addr: url })
    }

    fn supergraph_service(&self, service: supergraph::BoxService) -> supergraph::BoxService {
        let addr = self.addr.clone();

        let handler = move |req: supergraph::Request| {
            let addr = addr.clone();
            async {
                // If we set a res, then we are going to break execution
                // If not, we are continuing
                let res = if !req
                    .supergraph_request
                    .headers()
                    .contains_key("Authorization")
                {
                    // Prepare an HTTP 401 response with a GraphQL error message
                    Some(
                        supergraph::Response::error_builder()
                            .error(
                                graphql::Error::builder()
                                    .message("Missing 'Authorization' header")
                                    .extension_code("AUTH_ERROR")
                                    .build(),
                            )
                            .status_code(StatusCode::UNAUTHORIZED)
                            .context(req.context.clone())
                            .build()
                            .expect("response is valid"),
                    )
                } else {
                    // It is best practice to perform checks before we unwrap,
                    // And to use `expect()` instead of `unwrap()`, with a message
                    // that explains why the use of `expect()` is safe
                    let token = req
                        .supergraph_request
                        .headers()
                        .get("Authorization")
                        .expect("this cannot fail; we checked for header presence above")
                        .to_str();

                    match token {
                        Ok(token) => {
                            // TODO: Should actually split for "Bearer" in token
                            let response = reqwest::Client::new()
                                .post(addr)
                                .form(&[("token", token)])
                                .send()
                                .await;
                            match response {
                                Ok(response) => match response.status() {
                                    StatusCode::OK => None,
                                    // Passthrough the response code from sidecar, consider adding specific cases for individual scenarios later
                                    other_code => Some(
                                        supergraph::Response::builder()
                                            .data(Value::default())
                                            .error(
                                                graphql::Error::builder()
                                                    .message("Request is not allowed")
                                                    .extension_code("UNAUTHORIZED_REQUEST")
                                                    .build(),
                                            )
                                            .status_code(other_code)
                                            .context(req.context.clone())
                                            .build()
                                            .expect("response is valid"),
                                    ),
                                },
                                Err(_) => Some(
                                    supergraph::Response::error_builder()
                                        .error(
                                            graphql::Error::builder()
                                                .message("Auth sidecar is unreachable")
                                                .extension_code("SERVICE_ERROR")
                                                .build(),
                                        )
                                        .status_code(StatusCode::SERVICE_UNAVAILABLE)
                                        .context(req.context.clone())
                                        .build()
                                        .expect("response is valid"),
                                ),
                            }
                        }
                        Err(_not_a_string_error) => {
                            // Prepare an HTTP 400 response with a GraphQL error message
                            Some(
                                supergraph::Response::error_builder()
                                    .error(
                                        graphql::Error::builder()
                                            .message(format!(
                                                "'Authorization' value is not a string"
                                            ))
                                            .extension_code("BAD_CLIENT_ID")
                                            .build(),
                                    )
                                    .status_code(StatusCode::BAD_REQUEST)
                                    .context(req.context.clone())
                                    .build()
                                    .expect("response is valid"),
                            )
                        }
                    }
                };
                // Check to see if we built a response. If we did, we need to Break.
                match res {
                    Some(res) => Ok(ControlFlow::Break(res)),
                    None => Ok(ControlFlow::Continue(req)),
                }
            }
        };
        ServiceBuilder::new()
            .checkpoint_async(handler)
            .buffer(20_000)
            .service(service)
            .boxed()
    }
}

register_plugin!("alyti", "sidecar", SidecarProxy);
