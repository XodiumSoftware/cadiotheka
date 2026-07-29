/// Error returned when a frontend data helper fails to talk to the backend.
#[derive(Debug, Clone)]
pub enum RequestError {
    /// Failed to serialize the request body.
    Serialize(String),
    /// Failed to build the HTTP request.
    BuildRequest(String),
    /// The network request failed.
    Network(String),
    /// The server returned a non-success status.
    Server { status: u16, body: String },
    /// The response body could not be parsed.
    Parse(String),
}

impl RequestError {
    /// Returns a human-readable message suitable for toast or inline error UI.
    pub fn message(&self) -> String {
        match self {
            Self::Serialize(err) => format!("Failed to prepare request data: {err}"),
            Self::BuildRequest(err) => format!("Could not start the request: {err}"),
            Self::Network(err) => format!("Network error: {err}"),
            Self::Server { status, body } => format!("Server error (HTTP {status}): {body}"),
            Self::Parse(err) => format!("Failed to read response: {err}"),
        }
    }
}

impl core::fmt::Display for RequestError {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        f.write_str(&self.message())
    }
}
