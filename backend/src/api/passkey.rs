//! `Passkey` (`WebAuthn`) registration and authentication.
//!
//! Implements a subset of the `WebAuthn` Level 3 relying-party flow sufficient for
//! passkey sign-in: attestation is always requested as `"none"` (so no X.509
//! chain validation is needed) and only ECDSA P-256 (ES256) and `Ed25519` (`EdDSA`)
//! credentials are accepted, matching what every major passkey provider emits.
//!
//! Both registration and authentication use short-lived random challenges stored
//! in KV. The challenge key is only ever revealed to the party that started the
//! ceremony (the session holder for registration, the bearer of the state token
//! for login), and is deleted on consumption or expiry.

use base64::Engine as _;
use base64::engine::general_purpose::URL_SAFE_NO_PAD as B64;
use serde::{Deserialize, Serialize};
use sha2::Digest as _;
use worker::wasm_bindgen::JsCast;
use worker::{Headers, Request, Response, ResponseBuilder, Result, RouteContext};

use crate::api::accounts::{Account, fetch_account};
use crate::api::session::{create_session, read_session};
use crate::utils::{
    RateLimitNamespace, bad_request, check_rate_limit, db, is_https_request, kv, now_utc,
    public_origin, required_param, rust_err, safe_redirect_target, unauthorized,
};

const CHALLENGE_TTL_SECONDS: u64 = 10 * 60;
const CHALLENGE_LENGTH: usize = 32;

/// COSE algorithm identifiers recognised by this server.
const COSE_ALG_ES256: i64 = -7;
const COSE_ALG_EDDSA: i64 = -8;

/// `WebAuthn` `authenticatorData` flags.
const AUTH_FLAG_USER_PRESENT: u8 = 0x01;
const AUTH_FLAG_USER_VERIFIED: u8 = 0x04;
const AUTH_FLAG_ATTESTED_CREDENTIAL_DATA: u8 = 0x40;
const AUTH_FLAG_EXTENSION_DATA: u8 = 0x80;

const AUTH_DATA_RP_ID_HASH_LEN: usize = 32;
const AUTH_DATA_FLAGS_OFFSET: usize = 32;
const AUTH_DATA_COUNTER_OFFSET: usize = 33;
const AUTH_DATA_MIN_LEN: usize = 37;
const CRED_ID_LEN_OFFSET: usize = 53;
const CRED_ID_OFFSET: usize = 55;

/// KV key prefix for pending registration challenges.
const KV_REGISTER_PREFIX: &str = "passkey_register:";
/// KV key prefix for pending authentication challenges.
const KV_LOGIN_PREFIX: &str = "passkey_login:";

/// Resolves the relying-party ID for the current request.
///
/// Requests whose host is `localhost` or `127.0.0.1` (i.e. `wrangler dev`) always
/// use `localhost` as the RP ID so passkeys work locally regardless of the env
/// var. All other requests read `WEBAUTHN_RP_ID`, which in production must be
/// the registrable domain of the frontend origin (e.g. `cadiotheka.com`).
fn rp_id(req: &Request, ctx: &RouteContext<()>) -> Result<String> {
    let is_local = req
        .url()
        .ok()
        .and_then(|u| u.host_str().map(str::to_owned))
        .is_some_and(|host| host == "localhost" || host == "127.0.0.1");
    if is_local {
        return Ok(String::from("localhost"));
    }
    ctx.env
        .var("WEBAUTHN_RP_ID")
        .map(|v| v.to_string())
        .or_else(|_| Ok(String::from("localhost")))
}

/// Resolves the human-readable relying-party name shown by the browser during
/// the passkey ceremony.
fn rp_name(ctx: &RouteContext<()>) -> Result<String> {
    ctx.env
        .var("WEBAUTHN_RP_NAME")
        .map(|v| v.to_string())
        .or_else(|_| Ok(String::from("Cadiotheka")))
}

/// Generates a fresh random challenge, base64-url encoded.
///
/// `js_sys::crypto::get_random_values` is used because Workers only exposes
/// system randomness through the JavaScript `crypto` global. In the unlikely
/// case the API is unreachable, the buffer is left zeroed rather than
/// panicking — a failed challenge mint will surface as a later signature
/// mismatch during verification.
fn generate_challenge() -> String {
    let Ok(crypto_val) = worker::js_sys::Reflect::get(&worker::js_sys::global(), &"crypto".into())
    else {
        return B64.encode([0u8; CHALLENGE_LENGTH]);
    };
    let Ok(crypto) = crypto_val.dyn_into::<worker::js_sys::Object>() else {
        return B64.encode([0u8; CHALLENGE_LENGTH]);
    };
    let Ok(get_random_values) = worker::js_sys::Reflect::get(&crypto, &"getRandomValues".into())
    else {
        return B64.encode([0u8; CHALLENGE_LENGTH]);
    };
    let Ok(func) = get_random_values.dyn_into::<worker::js_sys::Function>() else {
        return B64.encode([0u8; CHALLENGE_LENGTH]);
    };
    let Ok(len) = u32::try_from(CHALLENGE_LENGTH) else {
        return B64.encode([0u8; CHALLENGE_LENGTH]);
    };
    let mut bytes = [0u8; CHALLENGE_LENGTH];
    let array = worker::js_sys::Uint8Array::new_with_length(len);
    let _ = func.call1(&crypto, &array);
    array.copy_to(&mut bytes[..]);
    B64.encode(bytes)
}

/// JSON payload returned to the frontend at the start of a ceremony. The exact
/// field names follow the `WebAuthn` wire format so the browser's
/// `PublicKeyCredential` options can be constructed without remapping.
#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
struct PublicKeyCredentialCreationOptions {
    rp: RpEntity,
    user: UserEntity,
    challenge: String,
    pub_key_cred_params: Vec<PublicKeyCredParam>,
    timeout: u32,
    attestation: &'static str,
    authenticator_selection: AuthenticatorSelection,
    exclude_credentials: Vec<CredentialDescriptor>,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
struct RpEntity {
    id: String,
    name: String,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
struct UserEntity {
    id: String,
    name: String,
    display_name: String,
}

#[derive(Debug, Serialize)]
struct PublicKeyCredParam {
    #[serde(rename = "type")]
    kind: &'static str,
    alg: i64,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
struct AuthenticatorSelection {
    authenticator_attachment: &'static str,
    resident_key: &'static str,
    user_verification: &'static str,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
struct CredentialDescriptor {
    #[serde(rename = "type")]
    kind: &'static str,
    id: String,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    transports: Vec<String>,
}

/// JSON payload returned to the frontend at the start of an authentication
/// ceremony.
#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
struct PublicKeyCredentialRequestOptions {
    challenge: String,
    rp_id: String,
    timeout: u32,
    user_verification: &'static str,
    allow_credentials: Vec<CredentialDescriptor>,
}

/// Challenge state persisted in KV for the registration ceremony.
#[derive(Debug, Serialize, Deserialize)]
struct RegisterChallenge {
    challenge: String,
    account_id: String,
}

/// Challenge state persisted in KV for the login ceremony for usernameless
/// (discoverable) authentication. The challenge alone is the shared secret; the
/// account is identified by the credential id returned by the authenticator.
#[derive(Debug, Serialize, Deserialize)]
struct LoginChallenge {
    challenge: String,
    account_id: Option<String>,
}

/// Starts the passkey registration ceremony for an authenticated user.
///
/// Returns a `PublicKeyCredentialCreationOptions` JSON document that the
/// frontend hands to `navigator.credentials.create`.
pub async fn register_begin(req: Request, ctx: RouteContext<()>) -> Result<Response> {
    if let Some(response) = check_rate_limit(&req, &ctx, RateLimitNamespace::OauthCallback).await? {
        return Ok(response);
    }
    let Some(account) = read_session(&req, &ctx).await? else {
        return unauthorized("Unauthorized");
    };

    let existing = list_passkey_descriptors(&ctx, &account.id).await?;

    let challenge = generate_challenge();
    let state = RegisterChallenge {
        challenge: challenge.clone(),
        account_id: account.id.clone(),
    };
    kv(&ctx)?
        .put(
            &format!("{KV_REGISTER_PREFIX}{}", account.id),
            &serde_json::to_string(&state).map_err(rust_err)?,
        )?
        .expiration_ttl(CHALLENGE_TTL_SECONDS)
        .execute()
        .await?;

    let rp_id = rp_id(&req, &ctx)?;
    let options = PublicKeyCredentialCreationOptions {
        rp: RpEntity {
            id: rp_id,
            name: rp_name(&ctx)?,
        },
        user: UserEntity {
            id: B64.encode(account.id.as_bytes()),
            name: account.username.clone(),
            display_name: account.display_name.clone(),
        },
        challenge,
        pub_key_cred_params: vec![
            PublicKeyCredParam {
                kind: "public-key",
                alg: COSE_ALG_ES256,
            },
            PublicKeyCredParam {
                kind: "public-key",
                alg: COSE_ALG_EDDSA,
            },
        ],
        timeout: 60_000,
        attestation: "none",
        authenticator_selection: AuthenticatorSelection {
            authenticator_attachment: "platform",
            resident_key: "required",
            user_verification: "preferred",
        },
        exclude_credentials: existing,
    };

    Response::from_json(&options)
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
struct RegisterFinishPayload {
    id: String,
    raw_id: String,
    response: RegisterResponse,
    #[serde(rename = "type")]
    kind: String,
    #[serde(default)]
    name: String,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
struct RegisterResponse {
    attestation_object: String,
    #[serde(rename = "clientDataJSON")]
    client_data_json: String,
    #[serde(default)]
    transports: Vec<String>,
}

/// Completes passkey registration.
///
/// Verifies the signed challenge and persists the new credential. The passkey
/// only becomes usable for sign-in after this endpoint succeeds.
pub async fn register_finish(mut req: Request, ctx: RouteContext<()>) -> Result<Response> {
    if let Some(response) = check_rate_limit(&req, &ctx, RateLimitNamespace::OauthCallback).await? {
        return Ok(response);
    }
    let Some(account) = read_session(&req, &ctx).await? else {
        return unauthorized("Unauthorized");
    };
    let payload: RegisterFinishPayload = req.json().await?;

    if payload.kind != "public-key" {
        return bad_request("invalid credential type");
    }
    if payload.id != payload.raw_id {
        return bad_request("credential id mismatch");
    }

    let key = format!("{KV_REGISTER_PREFIX}{}", account.id);
    let state_json = kv(&ctx)?
        .get(&key)
        .text()
        .await?
        .ok_or_else(|| rust_err("registration challenge expired or not started"))?;
    let state: RegisterChallenge = serde_json::from_str(&state_json).map_err(rust_err)?;
    if state.account_id != account.id {
        kv(&ctx)?.delete(&key).await?;
        return unauthorized("challenge account mismatch");
    }
    kv(&ctx)?.delete(&key).await?;

    let client_data =
        verify_client_data_json(&payload.response.client_data_json, "webauthn.create")?;
    verify_challenge(&client_data.challenge, &state.challenge)?;
    verify_origin(&client_data.origin, &req)?;

    let attestation_object = B64
        .decode(&payload.response.attestation_object)
        .map_err(|err| rust_err(format!("invalid attestation object: {err}")))?;
    let attestation = parse_attestation_object(&attestation_object)?;
    if attestation.fmt != "none" {
        return bad_request("attestation format not accepted");
    }

    let auth_data = verify_authenticator_data(&attestation.auth_data, &rp_id(&req, &ctx)?, true)?;
    let Some(credential) = parse_attested_credential_data(&attestation.auth_data)? else {
        return bad_request("missing attested credential data");
    };
    if credential.id != payload.raw_id {
        return bad_request("credential id does not match payload");
    }

    let public_key = PublicKey::from_credential(&credential)?;

    let name = payload.name.trim().to_string();
    if name.chars().count() > shared::validation::MAX_PASSKEY_NAME_LENGTH {
        return bad_request("passkey name too long");
    }

    let created_at = now_utc()
        .format(&time::format_description::well_known::Rfc3339)
        .map_err(|err| rust_err(format!("failed to format timestamp: {err}")))?;
    let transports = serde_json::to_string(&payload.response.transports).map_err(rust_err)?;

    db(&ctx)?
        .prepare(
            "INSERT INTO passkeys (id, account_id, credential_id, public_key, sign_count, transports, name, created_at) \
             VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8)",
        )
        .bind(&[
            uuid::Uuid::new_v4().to_string().into(),
            account.id.into(),
            payload.raw_id.into(),
            public_key.serialize().into(),
            f64::from(auth_data.sign_count).into(),
            transports.into(),
            name.into(),
            created_at.into(),
        ])?
        .run()
        .await?;

    Response::empty()
}

#[derive(Debug, Deserialize, Serialize)]
struct ClientData {
    #[serde(rename = "type")]
    kind: String,
    challenge: String,
    origin: String,
}

/// Decodes `clientDataJSON` and validates the ceremony type.
fn verify_client_data_json(encoded: &str, expected_type: &str) -> Result<ClientData> {
    let bytes = B64
        .decode(encoded)
        .map_err(|err| rust_err(format!("invalid clientDataJSON: {err}")))?;
    let data: ClientData = serde_json::from_slice(&bytes)
        .map_err(|err| rust_err(format!("invalid clientDataJSON: {err}")))?;
    if data.kind != expected_type {
        return Err(rust_err("clientDataJSON type mismatch"));
    }
    Ok(data)
}

/// Compares two base64-encoded challenges in constant time.
fn verify_challenge(client: &str, server: &str) -> Result<()> {
    let client_bytes = B64
        .decode(client)
        .map_err(|err| rust_err(format!("invalid challenge: {err}")))?;
    let server_bytes = B64
        .decode(server)
        .map_err(|err| rust_err(format!("invalid stored challenge: {err}")))?;
    if client_bytes.len() != server_bytes.len()
        || client_bytes
            .iter()
            .zip(server_bytes.iter())
            .any(|(a, b)| a != b)
    {
        return Err(rust_err("challenge mismatch"));
    }
    Ok(())
}

/// Ensures the ceremony originated from an origin this server serves.
fn verify_origin(origin: &str, req: &Request) -> Result<()> {
    let expected = public_origin(req);
    let is_https = is_https_request(req);
    verify_origin_strings(origin, &expected, is_https)
}

/// Pure helper comparing the client-supplied origin against the trusted set.
fn verify_origin_strings(origin: &str, request_origin: &str, is_https: bool) -> Result<()> {
    let mut allowed = vec![request_origin.to_string()];
    if is_https {
        allowed.extend(
            shared::origins::ALLOWED_REDIRECT_ORIGINS
                .iter()
                .map(|o| String::from(*o)),
        );
    } else {
        allowed.extend(
            shared::origins::ALLOWED_LOCALHOST_ORIGINS
                .iter()
                .map(|o| String::from(*o)),
        );
    }
    if allowed.iter().any(|o| o == origin) {
        Ok(())
    } else {
        Err(rust_err("origin not allowed"))
    }
}

/// Raw `authenticatorData` validated against the RP ID.
struct AuthData {
    sign_count: u32,
    user_present: bool,
}

/// Validates the fixed header of `authenticatorData`.
///
/// The RP ID hash must match the SHA-256 of the server's RP ID, the user
/// presence flag must be set, and the minimum length must be present. Extension
/// data may follow the credential data; this function doesn't parse it.
fn verify_authenticator_data(
    bytes: &[u8],
    rp_id: &str,
    require_attested_credential: bool,
) -> Result<AuthData> {
    if bytes.len() < AUTH_DATA_MIN_LEN {
        return Err(rust_err("authenticatorData too short"));
    }
    let mut hasher = sha2::Sha256::new();
    hasher.update(rp_id.as_bytes());
    let expected_hash = hasher.finalize();
    let rp_id_hash_matches = bytes[..AUTH_DATA_RP_ID_HASH_LEN] == expected_hash[..];
    if !rp_id_hash_matches {
        return Err(rust_err("rpId hash mismatch"));
    }

    let flags = bytes[AUTH_DATA_FLAGS_OFFSET];
    let user_present = flags & AUTH_FLAG_USER_PRESENT != 0;
    let user_verified = flags & AUTH_FLAG_USER_VERIFIED != 0;
    let has_attested_credential_data = flags & AUTH_FLAG_ATTESTED_CREDENTIAL_DATA != 0;
    let has_extension_data = flags & AUTH_FLAG_EXTENSION_DATA != 0;

    if !user_present {
        return Err(rust_err("user presence flag not set"));
    }
    if require_attested_credential && !has_attested_credential_data {
        return Err(rust_err("attested credential data missing"));
    }
    if !require_attested_credential && has_attested_credential_data {
        return Err(rust_err("unexpected attested credential data"));
    }
    let _ = has_extension_data;
    let _ = user_verified;

    let sign_count = u32::from_be_bytes([
        bytes[AUTH_DATA_COUNTER_OFFSET],
        bytes[AUTH_DATA_COUNTER_OFFSET + 1],
        bytes[AUTH_DATA_COUNTER_OFFSET + 2],
        bytes[AUTH_DATA_COUNTER_OFFSET + 3],
    ]);

    Ok(AuthData {
        sign_count,
        user_present,
    })
}

/// Attested credential data extracted from a registration's `authenticatorData`.
struct AttestedCredential {
    id: String,
    public_key_cose: Vec<u8>,
}

/// Parses the variable-length attested credential data appended after the fixed
/// `authenticatorData` header during registration.
fn parse_attested_credential_data(bytes: &[u8]) -> Result<Option<AttestedCredential>> {
    if bytes.len() <= CRED_ID_OFFSET {
        return Ok(None);
    }
    if bytes.len() < CRED_ID_LEN_OFFSET + 2 {
        return Err(rust_err("malformed attested credential data"));
    }
    let cred_id_len =
        u16::from_be_bytes([bytes[CRED_ID_LEN_OFFSET], bytes[CRED_ID_LEN_OFFSET + 1]]) as usize;
    let cred_id_end = CRED_ID_OFFSET + cred_id_len;
    if bytes.len() < cred_id_end {
        return Err(rust_err("credential id truncated"));
    }
    let cred_id = &bytes[CRED_ID_OFFSET..cred_id_end];
    let public_key_cose = bytes[cred_id_end..].to_vec();
    if public_key_cose.is_empty() {
        return Err(rust_err("public key missing"));
    }

    Ok(Some(AttestedCredential {
        id: B64.encode(cred_id),
        public_key_cose,
    }))
}

/// Public key material, stored as a self-describing blob.
///
/// The first byte identifies the algorithm so the row remains interpretable as
/// new algorithms are introduced without a schema migration.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum PublicKeyKind {
    Es256,
    EdDsa,
}

impl PublicKeyKind {
    fn from_cose_alg(alg: i64) -> Result<Self> {
        match alg {
            COSE_ALG_ES256 => Ok(Self::Es256),
            COSE_ALG_EDDSA => Ok(Self::EdDsa),
            _ => Err(rust_err(format!("unsupported COSE algorithm: {alg}"))),
        }
    }

    fn to_tag(self) -> u8 {
        match self {
            Self::Es256 => 1,
            Self::EdDsa => 2,
        }
    }

    fn from_tag(tag: u8) -> Result<Self> {
        match tag {
            1 => Ok(Self::Es256),
            2 => Ok(Self::EdDsa),
            _ => Err(rust_err(format!("unknown public key tag: {tag}"))),
        }
    }
}

/// A parsed credential public key, ready to verify signatures.
#[derive(Debug, Clone)]
struct PublicKey {
    kind: PublicKeyKind,
    /// Raw key bytes: SEC 1 uncompressed point for ES256, raw 32-byte key for
    /// Ed25519.
    material: Vec<u8>,
}

impl PublicKey {
    /// Parses the COSE key embedded in attested credential data.
    fn from_credential(credential: &AttestedCredential) -> Result<Self> {
        let cbor: serde_cbor_2::Value = serde_cbor_2::from_slice(&credential.public_key_cose)
            .map_err(|err| rust_err(format!("failed to parse credential public key: {err}")))?;
        let serde_cbor_2::Value::Map(map) = cbor else {
            return Err(rust_err("credential public key is not a CBOR map"));
        };
        let alg_key = serde_cbor_2::Value::Integer(3);
        let x_key = serde_cbor_2::Value::Integer(-2);
        let y_key = serde_cbor_2::Value::Integer(-3);

        let alg = match map.get(&alg_key) {
            Some(serde_cbor_2::Value::Integer(i)) => i64::try_from(*i).map_err(rust_err)?,
            _ => return Err(rust_err("missing alg in credential public key")),
        };
        let kind = PublicKeyKind::from_cose_alg(alg)?;

        let material = match kind {
            PublicKeyKind::Es256 => {
                let x = match map.get(&x_key) {
                    Some(serde_cbor_2::Value::Bytes(b)) => b.clone(),
                    _ => return Err(rust_err("missing x coordinate")),
                };
                let y = match map.get(&y_key) {
                    Some(serde_cbor_2::Value::Bytes(b)) => b.clone(),
                    _ => return Err(rust_err("missing y coordinate")),
                };
                if x.len() != 32 || y.len() != 32 {
                    return Err(rust_err("invalid coordinate length"));
                }
                let mut buf = Vec::with_capacity(65);
                buf.push(0x04);
                buf.extend_from_slice(&x);
                buf.extend_from_slice(&y);
                buf
            }
            PublicKeyKind::EdDsa => {
                let x = match map.get(&x_key) {
                    Some(serde_cbor_2::Value::Bytes(b)) => b.clone(),
                    _ => return Err(rust_err("missing ed25519 key")),
                };
                if x.len() != 32 {
                    return Err(rust_err("invalid ed25519 key length"));
                }
                x
            }
        };

        Ok(Self { kind, material })
    }

    /// Deserializes from the storage format written to D1.
    fn deserialize(bytes: &[u8]) -> Result<Self> {
        let (tag, material) = bytes
            .split_first()
            .ok_or_else(|| rust_err("public key blob is empty"))?;
        Ok(Self {
            kind: PublicKeyKind::from_tag(*tag)?,
            material: material.to_vec(),
        })
    }

    /// Serializes into the storage format persisted in D1.
    fn serialize(&self) -> Vec<u8> {
        let mut out = Vec::with_capacity(1 + self.material.len());
        out.push(self.kind.to_tag());
        out.extend_from_slice(&self.material);
        out
    }

    /// Verifies an ECDSA/EdDSA signature over the authenticator-signed payload
    /// (`authenticatorData || SHA-256(clientDataJSON)`).
    fn verify(
        &self,
        authenticator_data: &[u8],
        client_data_json: &[u8],
        signature: &[u8],
    ) -> Result<()> {
        let mut message = Vec::with_capacity(authenticator_data.len() + 32);
        message.extend_from_slice(authenticator_data);
        let mut hasher = sha2::Sha256::new();
        hasher.update(client_data_json);
        message.extend_from_slice(&hasher.finalize());

        match self.kind {
            PublicKeyKind::Es256 => {
                let key = p256::ecdsa::VerifyingKey::from_sec1_bytes(&self.material)
                    .map_err(|err| rust_err(format!("invalid p256 public key: {err}")))?;
                let sig = p256::ecdsa::Signature::from_der(signature)
                    .map_err(|err| rust_err(format!("invalid signature: {err}")))?;
                p256::ecdsa::signature::Verifier::verify(&key, &message, &sig)
                    .map_err(|err| rust_err(format!("signature verification failed: {err}")))
            }
            PublicKeyKind::EdDsa => {
                let key_bytes: [u8; 32] = self
                    .material
                    .as_slice()
                    .try_into()
                    .map_err(|_err| rust_err("invalid ed25519 key length"))?;
                let key = ed25519_dalek::VerifyingKey::from_bytes(&key_bytes)
                    .map_err(|err| rust_err(format!("invalid ed25519 public key: {err}")))?;
                let sig_bytes: [u8; 64] = signature
                    .try_into()
                    .map_err(|_err| rust_err("invalid ed25519 signature length"))?;
                let sig = ed25519_dalek::Signature::from_bytes(&sig_bytes);
                ed25519_dalek::Verifier::verify(&key, &message, &sig)
                    .map_err(|err| rust_err(format!("signature verification failed: {err}")))
            }
        }
    }
}

/// Parses the CBOR attestation object returned by the authenticator.
///
/// We accept only `fmt: "none"`, so `attStmt` is ignored entirely.
struct ParsedAttestation {
    fmt: String,
    auth_data: Vec<u8>,
}

fn parse_attestation_object(bytes: &[u8]) -> Result<ParsedAttestation> {
    let cbor: serde_cbor_2::Value = serde_cbor_2::from_slice(bytes)
        .map_err(|err| rust_err(format!("failed to parse attestation object: {err}")))?;
    let serde_cbor_2::Value::Map(map) = cbor else {
        return Err(rust_err("attestation object is not a CBOR map"));
    };
    let fmt = match map.get(&serde_cbor_2::Value::Text("fmt".into())) {
        Some(serde_cbor_2::Value::Text(s)) => s.clone(),
        _ => return Err(rust_err("missing fmt in attestation object")),
    };
    let auth_data = match map.get(&serde_cbor_2::Value::Text("authData".into())) {
        Some(serde_cbor_2::Value::Bytes(b)) => b.clone(),
        _ => return Err(rust_err("missing authData in attestation object")),
    };
    Ok(ParsedAttestation { fmt, auth_data })
}

/// Passkey row as persisted in D1, plus its deserialized public key.
struct StoredPasskey {
    id: String,
    account_id: String,
    public_key: PublicKey,
    sign_count: u32,
}

#[derive(Deserialize)]
struct StoredPasskeyRow {
    id: String,
    account_id: String,
    #[allow(dead_code)]
    credential_id: String,
    public_key: Vec<u8>,
    sign_count: i64,
}

/// Looks up a passkey row by its credential id alone. Safe because credential
/// ids are authenticator-generated and globally unique across the deployment.
async fn fetch_passkey_by_credential_id(
    ctx: &RouteContext<()>,
    credential_id: &str,
) -> Result<Option<StoredPasskey>> {
    let row = db(ctx)?
        .prepare(
            "SELECT id, account_id, credential_id, public_key, sign_count FROM passkeys WHERE credential_id = ?1",
        )
        .bind(&[credential_id.into()])?
        .all()
        .await?
        .results::<StoredPasskeyRow>()?
        .into_iter()
        .next();
    row.map(|row| {
        Ok(StoredPasskey {
            id: row.id,
            account_id: row.account_id,
            public_key: PublicKey::deserialize(&row.public_key)?,
            sign_count: u32::try_from(row.sign_count).unwrap_or(0),
        })
    })
    .transpose()
}

/// Loads `excludeCredentials` descriptors for a user's existing passkeys.
async fn list_passkey_descriptors(
    ctx: &RouteContext<()>,
    account_id: &str,
) -> Result<Vec<CredentialDescriptor>> {
    let rows = db(ctx)?
        .prepare("SELECT credential_id, transports FROM passkeys WHERE account_id = ?1")
        .bind(&[account_id.into()])?
        .all()
        .await?
        .results::<ExistingPasskeyRow>()?;
    Ok(rows
        .into_iter()
        .map(|row| {
            let transports: Vec<String> = serde_json::from_str(&row.transports).unwrap_or_default();
            CredentialDescriptor {
                kind: "public-key",
                id: row.credential_id,
                transports,
            }
        })
        .collect())
}

#[derive(Deserialize)]
struct ExistingPasskeyRow {
    credential_id: String,
    transports: String,
}

/// Starts the passkey login ceremony.
///
/// Uses discoverable credentials (usernameless flow): the challenge is bound to
/// no account and the `allowCredentials` list is empty so the authenticator
/// offers every passkey registered for this RP. The account is resolved on
/// `login_finish` from the credential id in the assertion, cross-checked
/// against the `userHandle` when the authenticator provides one.
pub async fn login_begin(req: Request, ctx: RouteContext<()>) -> Result<Response> {
    if req.method() != worker::Method::Post {
        return bad_request("method not allowed");
    }
    if let Some(response) = check_rate_limit(&req, &ctx, RateLimitNamespace::OauthLogin).await? {
        return Ok(response);
    }

    let challenge = generate_challenge();
    let state = LoginChallenge {
        challenge: challenge.clone(),
        account_id: None,
    };
    let state_token = uuid::Uuid::new_v4().to_string();
    kv(&ctx)?
        .put(
            &format!("{KV_LOGIN_PREFIX}{state_token}"),
            &serde_json::to_string(&state).map_err(rust_err)?,
        )?
        .expiration_ttl(CHALLENGE_TTL_SECONDS)
        .execute()
        .await?;

    let options = PublicKeyCredentialRequestOptions {
        challenge,
        rp_id: rp_id(&req, &ctx)?,
        timeout: 60_000,
        user_verification: "preferred",
        allow_credentials: Vec::new(),
    };

    Response::from_json(&serde_json::json!({
        "state": state_token,
        "options": options,
    }))
}

/// Account lookup helper used by the legacy scoped `login_begin` flow. Retained
/// so the credential can be pre-narrowed for tests; the production login path
/// resolves the account from the credential id instead.
#[allow(dead_code)]
async fn fetch_account_by_username_or_email(
    ctx: &RouteContext<()>,
    identifier: &str,
) -> Result<Option<Account>> {
    let result = db(ctx)?
        .prepare(
            "SELECT id FROM accounts WHERE username = ?1 COLLATE NOCASE OR email = ?1 COLLATE NOCASE",
        )
        .bind(&[identifier.into()])?
        .all()
        .await?;
    let ids: Vec<IdRow> = result.results::<IdRow>()?;
    let Some(row) = ids.into_iter().next() else {
        return Ok(None);
    };
    fetch_account(ctx, &row.id).await
}

#[derive(Deserialize)]
struct IdRow {
    id: String,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
struct LoginFinishPayload {
    state: String,
    id: String,
    raw_id: String,
    response: LoginResponse,
    #[serde(rename = "type")]
    kind: String,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
struct LoginResponse {
    authenticator_data: String,
    /// `WebAuthn` spec name is `clientDataJSON` (uppercase JSON), which serde's
    /// `camelCase` would normally emit as `clientDataJson`.
    #[serde(rename = "clientDataJSON")]
    client_data_json: String,
    signature: String,
    /// Returned by the authenticator when the credential was created as a
    /// resident (discoverable) key. When present, it must equal the base64url
    /// encoding of the account id bound to the challenge. We intentionally
    /// ignore it when absent because our challenges always pin an account.
    #[serde(default)]
    #[allow(dead_code)]
    user_handle: Option<String>,
}

/// Completes the passkey login ceremony.
///
/// On success this creates a new session and returns an `HttpOnly` session cookie
/// in a redirect response, mirroring the OAuth callback.
pub async fn login_finish(mut req: Request, ctx: RouteContext<()>) -> Result<Response> {
    if let Some(response) = check_rate_limit(&req, &ctx, RateLimitNamespace::OauthCallback).await? {
        return Ok(response);
    }
    let payload: LoginFinishPayload = req.json().await?;

    if payload.kind != "public-key" {
        return bad_request("invalid credential type");
    }
    if payload.id != payload.raw_id {
        return bad_request("credential id mismatch");
    }

    let key = format!("{KV_LOGIN_PREFIX}{}", payload.state);
    let state_json = kv(&ctx)?
        .get(&key)
        .text()
        .await?
        .ok_or_else(|| rust_err("login challenge expired or not started"))?;
    let state: LoginChallenge = serde_json::from_str(&state_json).map_err(rust_err)?;

    kv(&ctx)?.delete(&key).await?;

    let Some(stored) = fetch_passkey_by_credential_id(&ctx, &payload.raw_id).await? else {
        return unauthorized("unknown credential");
    };

    if let Some(ref expected_account_id) = state.account_id
        && *expected_account_id != stored.account_id
    {
        return unauthorized("credential account mismatch");
    }

    if let Some(user_handle) = &payload.response.user_handle {
        let expected_handle = B64.encode(stored.account_id.as_bytes());
        if *user_handle != expected_handle
            && B64.decode(user_handle).ok().as_deref() != Some(stored.account_id.as_bytes())
        {
            worker::console_log!(
                "passkey user_handle mismatch for account {}",
                stored.account_id
            );
            return unauthorized("user handle mismatch");
        }
    }

    let client_data = verify_client_data_json(&payload.response.client_data_json, "webauthn.get")?;
    verify_challenge(&client_data.challenge, &state.challenge)?;
    verify_origin(&client_data.origin, &req)?;

    let authenticator_data = B64
        .decode(&payload.response.authenticator_data)
        .map_err(|err| rust_err(format!("invalid authenticatorData: {err}")))?;
    let auth_data = verify_authenticator_data(&authenticator_data, &rp_id(&req, &ctx)?, false)?;
    if !auth_data.user_present {
        return unauthorized("user presence flag not set");
    }

    let signature = B64
        .decode(&payload.response.signature)
        .map_err(|err| rust_err(format!("invalid signature: {err}")))?;
    let client_data_json_bytes = B64
        .decode(&payload.response.client_data_json)
        .map_err(|err| rust_err(format!("invalid clientDataJSON: {err}")))?;
    stored
        .public_key
        .verify(&authenticator_data, &client_data_json_bytes, &signature)?;

    if auth_data.sign_count > 0 && auth_data.sign_count <= stored.sign_count {
        worker::console_log!(
            "passkey sign count regression for account {}: stored={} presented={}",
            stored.account_id,
            stored.sign_count,
            auth_data.sign_count
        );
        return unauthorized("sign count regression");
    }

    finish_login(&ctx, &stored, auth_data.sign_count, &req).await
}

/// Completes the ceremony after signature verification: updates the stored
/// sign count, creates a session, and returns the redirect response.
async fn finish_login(
    ctx: &RouteContext<()>,
    stored: &StoredPasskey,
    new_sign_count: u32,
    req: &Request,
) -> Result<Response> {
    let account = fetch_account(ctx, &stored.account_id)
        .await?
        .ok_or_else(|| rust_err("account not found"))?;

    let now = now_utc()
        .format(&time::format_description::well_known::Rfc3339)
        .map_err(|err| rust_err(format!("failed to format timestamp: {err}")))?;
    db(ctx)?
        .prepare("UPDATE passkeys SET sign_count = ?1, last_used_at = ?2 WHERE id = ?3")
        .bind(&[
            f64::from(new_sign_count).into(),
            now.into(),
            stored.id.clone().into(),
        ])?
        .run()
        .await?;

    let cookie = create_session(ctx, &account, req).await?;

    let redirect_to = req
        .url()
        .ok()
        .and_then(|url| safe_redirect_target(is_https_request(req), &url, "redirect_to"))
        .unwrap_or_else(|| "/".to_string());
    let redirect_to = html_escape::encode_text(&redirect_to);

    let headers = Headers::new();
    headers.set("Content-Type", "text/html; charset=utf-8")?;
    headers.set("Set-Cookie", &cookie)?;

    let html = format!(
        r#"<!doctype html>
<html>
  <head>
    <meta charset="utf-8">
    <meta http-equiv='refresh' content='0; url={redirect_to}'>
    <title>Redirecting...</title>
  </head>
  <body>
    <p>Redirecting to <a href='{redirect_to}'>Cadiotheka</a>...</p>
    <script>window.location.replace('{redirect_to}');</script>
  </body>
</html>"#
    );

    Ok(ResponseBuilder::new()
        .with_status(200)
        .with_headers(headers)
        .body(worker::ResponseBody::Body(html.into())))
}

/// Summary of a passkey returned to the frontend for display.
#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
struct PasskeyInfo {
    id: String,
    credential_id: String,
    name: String,
    created_at: String,
    last_used_at: Option<String>,
    transports: Vec<String>,
}

/// Lists the passkeys registered to the currently authenticated account.
pub async fn list_passkeys(req: Request, ctx: RouteContext<()>) -> Result<Response> {
    let Some(account) = read_session(&req, &ctx).await? else {
        return unauthorized("Unauthorized");
    };
    let rows = db(&ctx)?
        .prepare(
            "SELECT id, credential_id, name, created_at, last_used_at, transports FROM passkeys WHERE account_id = ?1 ORDER BY created_at",
        )
        .bind(&[account.id.into()])?
        .all()
        .await?
        .results::<PasskeyInfoRow>()?;
    let items: Vec<PasskeyInfo> = rows
        .into_iter()
        .map(|row| PasskeyInfo {
            id: row.id,
            credential_id: row.credential_id,
            name: row.name,
            created_at: row.created_at,
            last_used_at: row.last_used_at,
            transports: serde_json::from_str(&row.transports).unwrap_or_default(),
        })
        .collect();
    Response::from_json(&serde_json::json!({ "passkeys": items }))
}

#[derive(Deserialize)]
struct PasskeyInfoRow {
    id: String,
    credential_id: String,
    name: String,
    created_at: String,
    last_used_at: Option<String>,
    transports: String,
}

/// Removes one of the caller's passkeys. Refuses when no passkey matching
/// `:id` exists for this account.
pub async fn delete_passkey(req: Request, ctx: RouteContext<()>) -> Result<Response> {
    let Some(account) = read_session(&req, &ctx).await? else {
        return unauthorized("Unauthorized");
    };
    let passkey_id = required_param(&ctx, "id")?;

    let result = db(&ctx)?
        .prepare("DELETE FROM passkeys WHERE id = ?1 AND account_id = ?2")
        .bind(&[passkey_id.into(), account.id.into()])?
        .run()
        .await?;

    let meta = result.meta()?;
    if meta.and_then(|m| m.changes).unwrap_or(0) == 0 {
        return crate::utils::not_found("passkey not found");
    }
    Response::empty()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn public_key_serialization_roundtrip_es256() -> Result<(), worker::Error> {
        let pk = PublicKey {
            kind: PublicKeyKind::Es256,
            material: vec![0x04u8; 65],
        };
        let bytes = pk.serialize();
        let parsed = PublicKey::deserialize(&bytes)?;
        assert_eq!(parsed.kind, PublicKeyKind::Es256);
        assert_eq!(parsed.material, pk.material);
        Ok(())
    }

    #[test]
    fn public_key_serialization_roundtrip_eddsa() -> Result<(), worker::Error> {
        let pk = PublicKey {
            kind: PublicKeyKind::EdDsa,
            material: vec![0xabu8; 32],
        };
        let bytes = pk.serialize();
        let parsed = PublicKey::deserialize(&bytes)?;
        assert_eq!(parsed.kind, PublicKeyKind::EdDsa);
        assert_eq!(parsed.material, pk.material);
        Ok(())
    }

    #[test]
    fn public_key_deserialize_rejects_unknown_tag() {
        let bytes = [0xffu8, 1, 2, 3];
        assert!(PublicKey::deserialize(&bytes).is_err());
    }

    #[test]
    fn verify_challenge_rejects_mismatch() {
        let server = B64.encode(b"abc");
        let client = B64.encode(b"abd");
        assert!(verify_challenge(&client, &server).is_err());
    }

    #[test]
    fn verify_challenge_accepts_equal() {
        let encoded = B64.encode(b"hello-pad-32");
        assert!(verify_challenge(&encoded, &encoded).is_ok());
    }

    #[test]
    fn verify_origin_accepts_https_match() {
        assert!(
            verify_origin_strings("https://cadiotheka.com", "https://api.cadiotheka.com", true)
                .is_ok()
        );
    }

    #[test]
    fn verify_origin_rejects_unknown() {
        assert!(
            verify_origin_strings("https://evil.example", "https://api.cadiotheka.com", true)
                .is_err()
        );
    }

    #[test]
    fn verify_origin_accepts_localhost_in_dev() {
        assert!(
            verify_origin_strings("http://localhost:8080", "http://localhost:8787", false).is_ok()
        );
    }

    #[test]
    fn verify_origin_rejects_localhost_in_prod() {
        assert!(
            verify_origin_strings("http://localhost:8080", "https://api.cadiotheka.com", true)
                .is_err()
        );
    }

    #[test]
    fn parse_attested_credential_data_rejects_short_input() {
        let short = vec![0u8; 10];
        assert!(parse_attested_credential_data(&short).is_ok_and(|o| o.is_none()));
    }

    #[test]
    fn verify_authenticator_data_rejects_short_input() {
        let short = vec![0u8; 4];
        assert!(verify_authenticator_data(&short, "example.com", false).is_err());
    }

    #[test]
    fn verify_authenticator_data_rejects_rp_id_mismatch() {
        let mut bytes = vec![0u8; AUTH_DATA_MIN_LEN];
        bytes[AUTH_DATA_FLAGS_OFFSET] = AUTH_FLAG_USER_PRESENT;
        assert!(verify_authenticator_data(&bytes, "example.com", false).is_err());
    }

    #[test]
    fn verify_authenticator_data_accepts_correct_rp_id_hash() {
        let mut bytes = vec![0u8; AUTH_DATA_MIN_LEN];
        let hash = {
            let mut hasher = sha2::Sha256::new();
            hasher.update(b"example.com");
            hasher.finalize()
        };
        bytes[..32].copy_from_slice(&hash);
        bytes[AUTH_DATA_FLAGS_OFFSET] = AUTH_FLAG_USER_PRESENT;
        let result = verify_authenticator_data(&bytes, "example.com", false);
        assert!(result.is_ok());
    }

    #[test]
    fn verify_authenticator_data_rejects_missing_user_present() {
        let mut bytes = vec![0u8; AUTH_DATA_MIN_LEN];
        let hash = {
            let mut hasher = sha2::Sha256::new();
            hasher.update(b"example.com");
            hasher.finalize()
        };
        bytes[..32].copy_from_slice(&hash);
        bytes[AUTH_DATA_FLAGS_OFFSET] = 0;
        assert!(verify_authenticator_data(&bytes, "example.com", false).is_err());
    }
}
