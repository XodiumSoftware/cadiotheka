//! `WebAuthn` (passkey) ceremony helpers.
//!
//! Wraps `navigator.credentials.create` / `navigator.credentials.get` and converts
//! between the browser's `ArrayBuffer` wire format and the base64url-encoded
//! JSON the backend expects.
//!
//! Both ceremonies fetch options from the backend first, then call the
//! authenticator, then POST the result back. The functions here are designed to
//! be called from `leptos::task::spawn_local` and never block the UI thread.

use base64::Engine as _;
use base64::engine::general_purpose::URL_SAFE_NO_PAD as B64;
use js_sys::{Array, Object, Reflect, Uint8Array};
use serde::Deserialize;
use wasm_bindgen::{JsCast, JsValue};
use wasm_bindgen_futures::JsFuture;
use web_sys::{CredentialsContainer, Navigator, PublicKeyCredential};

use crate::data::error::RequestError;
use crate::utils::{
    passkey_login_begin_url, passkey_login_finish_url, passkey_register_begin_url,
    passkey_register_finish_url,
};
use gloo_net::http::Request;
use leptos::web_sys::RequestCredentials;

/// Decodes a base64url (no-pad) string into a `Uint8Array` suitable for passing
/// to the `WebAuthn` API.
fn b64_to_array(encoded: &str) -> Result<Uint8Array, String> {
    let bytes = B64
        .decode(encoded)
        .map_err(|err| format!("invalid base64url: {err}"))?;
    let len = u32::try_from(bytes.len()).map_err(|_| "input too large".to_string())?;
    let array = Uint8Array::new_with_length(len);
    array.copy_from(&bytes);
    Ok(array)
}

/// Encodes a `Uint8Array`/`ArrayBuffer` into base64url (no-pad) for transport
/// to the backend.
fn array_to_b64(value: &JsValue) -> Result<String, String> {
    let uint8 = if let Ok(buf) = value.clone().dyn_into::<js_sys::ArrayBuffer>() {
        Uint8Array::new(&buf)
    } else if let Ok(view) = value.clone().dyn_into::<Uint8Array>() {
        view
    } else {
        return Err("expected ArrayBuffer or Uint8Array".into());
    };
    let mut bytes = vec![0u8; uint8.length() as usize];
    uint8.copy_to(&mut bytes);
    Ok(B64.encode(bytes))
}

fn credentials_container() -> Option<CredentialsContainer> {
    let window = leptos::web_sys::window()?;
    let navigator: Navigator = window.navigator();
    Some(navigator.credentials())
}

/// Returns `true` when the browser exposes the `WebAuthn` APIs needed to run a
/// ceremony. Used to hide the passkey buttons entirely on unsupported clients.
pub fn passkey_supported() -> bool {
    let Some(window) = leptos::web_sys::window() else {
        return false;
    };
    let has_pkc = matches!(
        Reflect::get(&window, &"PublicKeyCredential".into()),
        Ok(v) if !v.is_undefined()
    );
    has_pkc && credentials_container().is_some()
}

/// JSON payload sent by the backend at the start of registration.
#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
struct RegisterBeginResponse {
    challenge: String,
    rp: RpField,
    user: UserField,
    pub_key_cred_params: Vec<PubKeyCredParam>,
    timeout: u32,
    attestation: String,
    authenticator_selection: AuthenticatorSelectionField,
    exclude_credentials: Vec<CredentialDescriptorField>,
}

#[derive(Debug, Deserialize)]
struct RpField {
    id: String,
    name: String,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
struct UserField {
    id: String,
    name: String,
    display_name: String,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
struct PubKeyCredParam {
    #[serde(rename = "type")]
    kind: String,
    alg: i64,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
struct AuthenticatorSelectionField {
    authenticator_attachment: String,
    resident_key: String,
    user_verification: String,
}

#[derive(Debug, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
struct CredentialDescriptorField {
    #[serde(rename = "type")]
    kind: String,
    id: String,
    #[serde(default)]
    transports: Vec<String>,
}

/// Sets a JS property on the given object, returning the error as a string for
/// `?`-propagation. Centralized so the rest of the module doesn't need to wrap
/// every call site with a `format!` helper.
fn set(target: &Object, key: &str, value: &JsValue) -> Result<(), String> {
    Reflect::set(target, &JsValue::from_str(key), value).map_err(|err| format!("{err:?}"))?;
    Ok(())
}

/// Starts passkey registration for the currently authenticated account.
///
/// Fetches a challenge from the backend, calls `navigator.credentials.create`,
/// then submits the resulting attestation back. On success the new passkey is
/// immediately usable for sign-in. `name` is sent to the backend and stored on
/// the credential row as the user-facing label.
///
/// # Errors
///
/// Returns a user-friendly message when the browser is unsupported, the
/// ceremony is cancelled, the backend rejects the credential, or the network
/// fails.
pub async fn register_passkey(name: &str) -> Result<(), String> {
    let container = credentials_container()
        .ok_or_else(|| "WebAuthn is not available in this context".to_string())?;

    let resp = Request::post(&passkey_register_begin_url())
        .credentials(RequestCredentials::Include)
        .send()
        .await
        .map_err(|err| format!("failed to start passkey registration: {err}"))?;
    if !resp.ok() {
        return Err(format!("server returned status {}", resp.status()));
    }
    let begin: RegisterBeginResponse = resp
        .json()
        .await
        .map_err(|err| format!("invalid registration options: {err}"))?;

    let rp = Object::new();
    set(&rp, "id", &JsValue::from_str(&begin.rp.id))?;
    set(&rp, "name", &JsValue::from_str(&begin.rp.name))?;

    let user = Object::new();
    set(&user, "id", &b64_to_array(&begin.user.id)?.into())?;
    set(&user, "name", &JsValue::from_str(&begin.user.name))?;
    set(
        &user,
        "displayName",
        &JsValue::from_str(&begin.user.display_name),
    )?;

    let params = build_cred_params(&begin.pub_key_cred_params)?;
    let exclude = build_descriptor_array(&begin.exclude_credentials)?;

    let authenticator_selection = Object::new();
    set(
        &authenticator_selection,
        "authenticatorAttachment",
        &JsValue::from_str(&begin.authenticator_selection.authenticator_attachment),
    )?;
    set(
        &authenticator_selection,
        "residentKey",
        &JsValue::from_str(&begin.authenticator_selection.resident_key),
    )?;
    set(
        &authenticator_selection,
        "userVerification",
        &JsValue::from_str(&begin.authenticator_selection.user_verification),
    )?;

    let public_key = Object::new();
    set(&public_key, "rp", &rp)?;
    set(&public_key, "user", &user)?;
    set(
        &public_key,
        "challenge",
        &b64_to_array(&begin.challenge)?.into(),
    )?;
    set(&public_key, "pubKeyCredParams", &params.into())?;
    set(
        &public_key,
        "timeout",
        &JsValue::from_f64(f64::from(begin.timeout)),
    )?;
    set(
        &public_key,
        "attestation",
        &JsValue::from_str(&begin.attestation),
    )?;
    set(
        &public_key,
        "authenticatorSelection",
        &authenticator_selection.into(),
    )?;
    set(&public_key, "excludeCredentials", &exclude.into())?;

    let options = Object::new();
    set(&options, "publicKey", &public_key.into())?;

    let credential = invoke_create(&container, &options).await?;

    let raw_id = array_to_b64(&credential.raw_id())?;
    let credential_id = credential.id();

    let response = Reflect::get(&credential, &JsValue::from_str("response"))
        .map_err(|err| format!("failed to read credential response: {err:?}"))?;
    let attestation_object_b64 = array_to_b64(
        &Reflect::get(&response, &JsValue::from_str("attestationObject"))
            .map_err(|err| format!("failed to read attestationObject: {err:?}"))?,
    )?;
    let client_data_json_b64 = array_to_b64(
        &Reflect::get(&response, &JsValue::from_str("clientDataJSON"))
            .map_err(|err| format!("failed to read clientDataJSON: {err:?}"))?,
    )?;

    let transports = parse_transports(&response);

    let body = serde_json::json!({
        "id": credential_id,
        "rawId": raw_id,
        "type": "public-key",
        "name": name,
        "response": {
            "attestationObject": attestation_object_b64,
            "clientDataJSON": client_data_json_b64,
            "transports": transports,
        },
    });

    let finish = Request::post(&passkey_register_finish_url())
        .credentials(RequestCredentials::Include)
        .header("Content-Type", "application/json")
        .body(body.to_string())
        .map_err(|err| format!("failed to build finish request: {err}"))?
        .send()
        .await
        .map_err(|err| format!("failed to finish registration: {err}"))?;
    if !finish.ok() {
        let status = finish.status();
        let text = finish.text().await.unwrap_or_default();
        return Err(format!("server rejected passkey: {status} {text}"));
    }
    Ok(())
}

/// Builds the JS array of `pubKeyCredParams` from the backend response.
fn build_cred_params(params: &[PubKeyCredParam]) -> Result<Array, String> {
    let arr = Array::new();
    for param in params {
        let o = Object::new();
        set(&o, "type", &JsValue::from_str(&param.kind))?;
        let alg = i32::try_from(param.alg).map_err(|_| "alg out of i32 range".to_string())?;
        set(&o, "alg", &JsValue::from(alg))?;
        arr.push(&o);
    }
    Ok(arr)
}

/// Builds the JS array of credential descriptors (used for both
/// `excludeCredentials` and `allowCredentials`).
fn build_descriptor_array(descriptors: &[CredentialDescriptorField]) -> Result<Array, String> {
    let arr = Array::new();
    for descriptor in descriptors {
        let o = Object::new();
        set(&o, "type", &JsValue::from_str(&descriptor.kind))?;
        set(&o, "id", &b64_to_array(&descriptor.id)?.into())?;
        if !descriptor.transports.is_empty() {
            let transports: Array = descriptor
                .transports
                .iter()
                .map(|t| JsValue::from_str(t))
                .collect();
            set(&o, "transports", &transports.into())?;
        }
        arr.push(&o);
    }
    Ok(arr)
}

/// Calls `credential.response.getTransports()` if available.
fn parse_transports(response: &JsValue) -> Vec<String> {
    let Ok(method_val) = Reflect::get(response, &JsValue::from_str("getTransports")) else {
        return Vec::new();
    };
    let Ok(method) = method_val.dyn_into::<js_sys::Function>() else {
        return Vec::new();
    };
    let Ok(arr_val) = method.call0(response) else {
        return Vec::new();
    };
    let Ok(arr) = arr_val.dyn_into::<Array>() else {
        return Vec::new();
    };
    arr.iter().filter_map(|v| v.as_string()).collect()
}

/// Awaits `navigator.credentials.create()` with the given options, returning a
/// cleanly typed `PublicKeyCredential`.
async fn invoke_create(
    container: &CredentialsContainer,
    options: &Object,
) -> Result<PublicKeyCredential, String> {
    let promise = container
        .create_with_options(&options.clone().unchecked_into())
        .map_err(|err| format!("failed to invoke credential creation: {err:?}"))?;
    let credential = JsFuture::from(promise)
        .await
        .map_err(|err| match err.as_string() {
            Some(msg) if msg.contains("NotAllowedError") => "Registration cancelled".to_string(),
            _ => format!("passkey creation failed: {err:?}"),
        })?;
    credential
        .dyn_into()
        .map_err(|_err| "unexpected credential type".to_string())
}

/// Awaits `navigator.credentials.get()` with the given options, returning a
/// cleanly typed `PublicKeyCredential`.
async fn invoke_get(
    container: &CredentialsContainer,
    options: &Object,
) -> Result<PublicKeyCredential, String> {
    let promise = container
        .get_with_options(&options.clone().unchecked_into())
        .map_err(|err| format!("failed to invoke credential assertion: {err:?}"))?;
    let credential = JsFuture::from(promise)
        .await
        .map_err(|err| match err.as_string() {
            Some(msg) if msg.contains("NotAllowedError") => "Sign-in cancelled".to_string(),
            _ => format!("passkey sign-in failed: {err:?}"),
        })?;
    credential
        .dyn_into()
        .map_err(|_err| "unexpected credential type".to_string())
}

#[derive(Debug, Deserialize)]
struct LoginBeginResponse {
    state: String,
    options: LoginOptionsField,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
struct LoginOptionsField {
    challenge: String,
    rp_id: String,
    timeout: u32,
    user_verification: String,
    allow_credentials: Vec<CredentialDescriptorField>,
}

/// Starts a usernameless passkey login. On success triggers a full page
/// reload so the new session cookie is picked up by the app state.
///
/// The authenticator shows every passkey it has for this site (the
/// backend's `allowCredentials` is intentionally empty); the account is
/// resolved from the credential the user picks.
///
/// # Errors
///
/// Returns a user-friendly message when the browser ceremony is cancelled or
/// the backend rejects the assertion.
pub async fn login_with_passkey(redirect_to: &str) -> Result<(), String> {
    let container = credentials_container()
        .ok_or_else(|| "WebAuthn is not available in this context".to_string())?;

    let begin_resp = Request::post(&passkey_login_begin_url())
        .credentials(RequestCredentials::Include)
        .header("Content-Type", "application/json")
        .body("{}")
        .map_err(|err| format!("failed to build login start request: {err}"))?
        .send()
        .await
        .map_err(|err| format!("failed to start passkey login: {err}"))?;
    if !begin_resp.ok() {
        return Err("Couldn't start passkey sign-in".to_string());
    }
    let begin: LoginBeginResponse = begin_resp
        .json()
        .await
        .map_err(|err| format!("invalid login options: {err}"))?;

    let public_key = Object::new();
    set(
        &public_key,
        "challenge",
        &b64_to_array(&begin.options.challenge)?.into(),
    )?;
    set(
        &public_key,
        "rpId",
        &JsValue::from_str(&begin.options.rp_id),
    )?;
    set(
        &public_key,
        "timeout",
        &JsValue::from_f64(f64::from(begin.options.timeout)),
    )?;
    set(
        &public_key,
        "userVerification",
        &JsValue::from_str(&begin.options.user_verification),
    )?;
    set(
        &public_key,
        "allowCredentials",
        &build_descriptor_array(&begin.options.allow_credentials)?.into(),
    )?;

    let options = Object::new();
    set(&options, "publicKey", &public_key.into())?;

    let credential = invoke_get(&container, &options).await?;

    let raw_id = array_to_b64(&credential.raw_id())?;
    let credential_id = credential.id();

    let response = Reflect::get(&credential, &JsValue::from_str("response"))
        .map_err(|err| format!("failed to read credential response: {err:?}"))?;
    let authenticator_data_b64 = array_to_b64(
        &Reflect::get(&response, &JsValue::from_str("authenticatorData"))
            .map_err(|err| format!("failed to read authenticatorData: {err:?}"))?,
    )?;
    let client_data_json_b64 = array_to_b64(
        &Reflect::get(&response, &JsValue::from_str("clientDataJSON"))
            .map_err(|err| format!("failed to read clientDataJSON: {err:?}"))?,
    )?;
    let signature_b64 = array_to_b64(
        &Reflect::get(&response, &JsValue::from_str("signature"))
            .map_err(|err| format!("failed to read signature: {err:?}"))?,
    )?;
    let user_handle_b64 = Reflect::get(&response, &JsValue::from_str("userHandle"))
        .ok()
        .filter(|v| !v.is_null() && !v.is_undefined())
        .map(|v| array_to_b64(&v))
        .transpose()?;

    let mut response_obj = serde_json::json!({
        "authenticatorData": authenticator_data_b64,
        "clientDataJSON": client_data_json_b64,
        "signature": signature_b64,
    });
    if let Some(handle) = user_handle_b64 {
        response_obj["userHandle"] = serde_json::json!(handle);
    }

    let body = serde_json::json!({
        "state": begin.state,
        "id": credential_id,
        "rawId": raw_id,
        "type": "public-key",
        "response": response_obj,
    });

    let encoded = urlencoding::encode(redirect_to);
    let finish_url = format!("{}?redirect_to={}", passkey_login_finish_url(), encoded);

    let finish = Request::post(&finish_url)
        .credentials(RequestCredentials::Include)
        .header("Content-Type", "application/json")
        .body(body.to_string())
        .map_err(|err| format!("failed to build login finish request: {err}"))?
        .send()
        .await
        .map_err(|err| format!("failed to finish login: {err}"))?;

    if !finish.ok() {
        let status = finish.status();
        let text = finish.text().await.unwrap_or_default();
        return Err(format!("sign-in rejected: {status} {text}"));
    }

    if let Some(window) = leptos::web_sys::window() {
        let _ = window.location().reload();
    }
    Ok(())
}

/// Lists the passkeys registered to the currently authenticated account.
///
/// # Errors
///
/// Returns [`RequestError`] when the user is unauthenticated or the backend
/// rejects the request.
pub async fn fetch_passkeys() -> Result<Vec<PasskeyInfo>, RequestError> {
    let url = crate::utils::passkeys_url();
    match Request::get(&url)
        .credentials(RequestCredentials::Include)
        .send()
        .await
    {
        Ok(response) if response.ok() => {
            let body = response.text().await.unwrap_or_default();
            let parsed: PasskeyListResponse = serde_json::from_str(&body).map_err(|err| {
                RequestError::Parse(format!(
                    "Failed to parse passkey list from {url}: {err} (body={body:?})"
                ))
            })?;
            Ok(parsed.passkeys)
        }
        Ok(response) if response.status() == 401 => Ok(Vec::new()),
        Ok(response) => {
            let status = response.status();
            let body = response.text().await.unwrap_or_default();
            Err(RequestError::Server {
                status,
                body: format!("Failed to fetch passkeys from {url}: {body}"),
            })
        }
        Err(err) => Err(RequestError::Network(format!(
            "Failed to fetch passkeys from {url}: {err}"
        ))),
    }
}

/// Deletes a single passkey belonging to the current account.
///
/// # Errors
///
/// Returns [`RequestError`] when the passkey does not exist or the backend
/// rejects the request.
pub async fn delete_passkey(id: &str) -> Result<(), RequestError> {
    let url = crate::utils::passkey_url(id);
    match Request::delete(&url)
        .credentials(RequestCredentials::Include)
        .send()
        .await
    {
        Ok(response) if response.ok() => Ok(()),
        Ok(response) => {
            let status = response.status();
            let body = response.text().await.unwrap_or_default();
            Err(RequestError::Server {
                status,
                body: format!("Failed to delete passkey at {url}: {body}"),
            })
        }
        Err(err) => Err(RequestError::Network(format!(
            "Failed to delete passkey at {url}: {err}"
        ))),
    }
}

/// Single passkey entry in the list returned by `GET /auth/passkeys`.
#[derive(Debug, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct PasskeyInfo {
    pub id: String,
    pub credential_id: String,
    pub name: String,
    #[serde(with = "time::serde::rfc3339")]
    pub created_at: time::OffsetDateTime,
    #[serde(default, with = "time::serde::rfc3339::option")]
    pub last_used_at: Option<time::OffsetDateTime>,
    pub transports: Vec<String>,
}

#[derive(Debug, Deserialize)]
struct PasskeyListResponse {
    passkeys: Vec<PasskeyInfo>,
}
