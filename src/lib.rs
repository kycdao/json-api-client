#[macro_use]
extern crate log;

pub mod error;
pub mod types;

use reqwest::header::*;
use reqwest::Method;
use serde::de::DeserializeOwned;

use error::*;
pub use oauth2::{AccessToken, AuthType, AuthorizationCode, RefreshToken, StandardToken, Token};
use std::fmt::Debug;
use url::Url;

pub type Queries<'a> = &'a [(&'a str, &'a str)];

pub struct AuthorizationHeaderConfig {
    pub token: String,
}

pub struct OAuth2Config {
    pub client_id: String,
    pub client_secret: String,
    pub authorize_path: String,
    pub auth_type: AuthType,
    pub token_path: String,
    pub refresh_path: String,
    pub redirect_url: String,
    pub scopes: Vec<String>,
}

pub enum AuthConfig {
    NoAuth,
    AuthorizationHeader(AuthorizationHeaderConfig),
    OAuth2(OAuth2Config),
}

pub trait JsonResponse: DeserializeOwned + Debug {}

impl<T> JsonResponse for T where T: DeserializeOwned + Debug {}

pub struct ApiClient {
    client: reqwest::Client,
    oauth_client: Option<oauth2::Client>,
    base_url: reqwest::Url,
}

impl ApiClient {
    pub fn new(api_url: &str, auth: AuthConfig, default_headers: Option<HeaderMap>) -> Result<ApiClient> {
        let base_url = reqwest::Url::parse(api_url)?;

        let mut headers = default_headers.unwrap_or_default();

        let oauth_client: Option<oauth2::Client> = match auth {
            AuthConfig::AuthorizationHeader(c) => {
                let mut auth_value = HeaderValue::from_str(&c.token).expect("Invalid API token value");
                auth_value.set_sensitive(true);
                headers.insert(AUTHORIZATION, auth_value);
                None
            },
            AuthConfig::OAuth2(c) => {
                let authorize_url = base_url.join(&c.authorize_path)?;
                let token_url = base_url.join(&c.token_path)?;
                let refresh_url = base_url.join(&c.refresh_path)?;

                let mut oauth2_client = oauth2::Client::new(c.client_id, authorize_url, token_url);

                oauth2_client.set_refresh_url(refresh_url);
                oauth2_client.set_client_secret(c.client_secret);
                oauth2_client.set_auth_type(c.auth_type);

                // Set the URL the user will be redirected to after the authorization process
                oauth2_client.set_redirect_url(Url::parse(&c.redirect_url)?);

                // Set the desired scopes
                c.scopes.iter().for_each(|scope| oauth2_client.add_scope(scope));
                Some(oauth2_client)
            },
            AuthConfig::NoAuth => None,
        };

        let client = reqwest::Client::builder().default_headers(headers).build()?;

        Ok(ApiClient {
            client,
            base_url,
            oauth_client,
        })
    }

    async fn parse_response<T>(resp: reqwest::Response) -> Result<T>
    where
        T: JsonResponse,
    {
        let text = resp.text().await?;
        trace!("Raw API Response: {}", text);
        match serde_json::from_str(&text) {
            Ok(r) => {
                debug!("API Response: {:?}", r);

                // TODO KYC-136 check response code and decode error if present

                Ok(r)
            },
            Err(e) => {
                error!("API response parsing failed! Raw response: {}", text);
                Err(e.into())
            },
        }
    }

    async fn handle_request<T>(
        &self,
        method: Method,
        path: &str,
        query: Option<Queries<'_>>,
        data: Option<&serde_json::Value>,
        headers: Option<HeaderMap>,
    ) -> Result<T>
    where
        T: JsonResponse,
    {
        let url = self.base_url.join(path)?;
        let mut builder = self.client.request(method, url);

        if let Some(q) = query {
            builder = builder.query(q);
        }
        if let Some(d) = data {
            builder = builder.json(d);
        }
        if let Some(h) = headers {
            builder = builder.headers(h);
        }

        let resp = builder.send().await?;
        ApiClient::parse_response(resp).await
    }

    pub async fn get<T>(&self, path: &str, query: Option<Queries<'_>>, headers: Option<HeaderMap>) -> Result<T>
    where
        T: JsonResponse,
    {
        self.handle_request(Method::GET, path, query, None, headers).await
    }

    pub async fn post<T>(&self, path: &str, data: Option<&serde_json::Value>, headers: Option<HeaderMap>) -> Result<T>
    where
        T: JsonResponse,
    {
        self.handle_request(Method::POST, path, None, data, headers).await
    }

    pub async fn put<T>(&self, path: &str, data: Option<&serde_json::Value>, headers: Option<HeaderMap>) -> Result<T>
    where
        T: JsonResponse,
    {
        self.handle_request(Method::PUT, path, None, data, headers).await
    }

    pub async fn patch<T>(&self, path: &str, data: Option<&serde_json::Value>, headers: Option<HeaderMap>) -> Result<T>
    where
        T: JsonResponse,
    {
        self.handle_request(Method::PATCH, path, None, data, headers).await
    }

    pub async fn delete<T>(&self, path: &str, headers: Option<HeaderMap>) -> Result<T>
    where
        T: JsonResponse,
    {
        self.handle_request(Method::DELETE, path, None, None, headers).await
    }

    // ******
    // OAuth2
    // ******

    fn ensure_oauth(&self) -> Result<&oauth2::Client> {
        self.oauth_client.as_ref().ok_or(Error::ClientError("OAuth2 not in use".to_owned()))
    }

    pub async fn exchange_code(&self, code: AuthorizationCode) -> Result<StandardToken> {
        let oauth = self.ensure_oauth()?;
        oauth
            .exchange_code(code)
            .with_client(&self.client)
            .execute::<StandardToken>()
            .await
            .map_err(Error::from)
    }

    pub async fn refresh(&self, refresh_token: &RefreshToken) -> Result<StandardToken> {
        let oauth = self.ensure_oauth()?;
        oauth
            .exchange_refresh_token(refresh_token)
            .with_client(&self.client)
            .execute::<StandardToken>()
            .await
            .map_err(Error::from)
    }
}

#[cfg(test)]
mod tests {
    //use super::*;

    // put unittests here
}
