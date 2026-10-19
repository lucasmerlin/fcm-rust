pub use chrono::{DateTime, Duration, FixedOffset};
use serde::Deserialize;
use std::{error::Error, fmt, str::FromStr};

/// A description of what went wrong with the push notification.
/// Referred from [Firebase documentation](https://firebase.google.com/docs/cloud-messaging/http-server-ref#table9)
#[derive(Deserialize, Debug, PartialEq, Copy, Clone)]
pub enum ErrorReason {
    /// Check that the request contains a registration token (in the `to` or
    /// `registration_ids` field).
    MissingRegistration,

    /// Check the format of the registration token you pass to the server. Make
    /// sure it matches the registration token the client app receives from
    /// registering with Firebase Notifications. Do not truncate or add
    /// additional characters.
    InvalidRegistration,

    /// An existing registration token may cease to be valid in a number of
    /// scenarios, including:
    ///
    /// * If the client app unregisters with FCM.
    /// * If the client app is automatically unregistered, which can happen if
    ///   the user uninstalls the application. For example, on iOS, if the APNS
    ///   Feedback Service reported the APNS token as invalid.
    /// * If the registration token expires (for example, Google might decide to
    ///   refresh registration tokens, or the APNS token has expired for iOS
    ///   devices).
    /// * If the client app is updated but the new version is not configured to
    ///   receive messages.
    ///
    /// For all these cases, remove this registration token from the app server
    /// and stop using it to send messages.
    NotRegistered,

    /// Make sure the message was addressed to a registration token whose
    /// package name matches the value passed in the request.
    InvalidPackageName,

    /// A registration token is tied to a certain group of senders. When a
    /// client app registers for FCM, it must specify which senders are allowed
    /// to send messages. You should use one of those sender IDs when sending
    /// messages to the client app. If you switch to a different sender, the
    /// existing registration tokens won't work.
    MismatchSenderId,

    /// Check that the provided parameters have the right name and type.
    InvalidParameters,

    /// Check that the total size of the payload data included in a message does
    /// not exceed FCM limits: 4096 bytes for most messages, or 2048 bytes in
    /// the case of messages to topics. This includes both the keys and the
    /// values.
    MessageTooBig,

    /// Check that the custom payload data does not contain a key (such as
    /// `from`, or `gcm`, or any value prefixed by google) that is used
    /// internally by FCM. Note that some words (such as `collapse_key`) are
    /// also used by FCM but are allowed in the payload, in which case the
    /// payload value will be overridden by the FCM value.
    InvalidDataKey,

    /// Check that the value used in `time_to_live` is an integer representing a
    /// duration in seconds between 0 and 2,419,200 (4 weeks).
    InvalidTtl,

    /// In internal use only. Check
    /// [FcmError::ServerError](enum.FcmError.html#variant.ServerError).
    Unavailable,

    /// In internal use only. Check
    /// [FcmError::ServerError](enum.FcmError.html#variant.ServerError).
    InternalServerError,

    /// The rate of messages to a particular device is too high. If an iOS app
    /// sends messages at a rate exceeding APNs limits, it may receive this
    /// error message
    ///
    /// Reduce the number of messages sent to this device and use exponential
    /// backoff to retry sending.
    DeviceMessageRateExceeded,

    /// The rate of messages to subscribers to a particular topic is too high.
    /// Reduce the number of messages sent for this topic and use exponential
    /// backoff to retry sending.
    TopicsMessageRateExceeded,

    /// A message targeted to an iOS device could not be sent because the
    /// required APNs authentication key was not uploaded or has expired. Check
    /// the validity of your development and production credentials.
    InvalidApnsCredential,
}

#[derive(Deserialize, Debug)]
pub struct FcmResponse {
    pub message_id: Option<u64>,
    pub error: Option<ErrorReason>,
    pub multicast_id: Option<i64>,
    pub success: Option<u64>,
    pub failure: Option<u64>,
    pub canonical_ids: Option<u64>,
    pub results: Option<Vec<MessageResult>>,
}

#[derive(Deserialize, Debug)]
pub struct MessageResult {
    pub message_id: Option<String>,
    pub registration_id: Option<String>,
    pub error: Option<ErrorReason>,
}

/// The `errorCode` of an FCM v1 error, from the
/// `google.firebase.fcm.v1.FcmError` entry in the error's `details`.
/// See [ErrorCode](https://firebase.google.com/docs/reference/fcm/rest/v1/ErrorCode).
#[derive(Deserialize, Debug, PartialEq, Eq, Copy, Clone)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum FcmErrorCode {
    /// No more information is available about this error.
    UnspecifiedError,
    /// Request parameters were invalid: a malformed registration token, an
    /// invalid field or a payload over the size limit. The message says which.
    InvalidArgument,
    /// The registration token is no longer valid (app uninstalled, token
    /// expired, ...). Stop sending to it. Usually HTTP 404.
    Unregistered,
    /// The token belongs to a different sender. Usually HTTP 403.
    SenderIdMismatch,
    /// Rate limit for the device, topic or project exceeded. Back off and
    /// honor `Retry-After`. Usually HTTP 429.
    QuotaExceeded,
    /// The server is overloaded. Usually HTTP 503.
    Unavailable,
    /// Unknown internal error. Usually HTTP 500.
    Internal,
    /// The APNs certificate / auth key or web push auth was invalid or missing.
    ThirdPartyAuthError,
    /// A code this crate doesn't know yet.
    #[serde(other)]
    Unknown,
}

/// A non-success response from the FCM v1 API, with what it said about why.
#[derive(PartialEq, Debug)]
pub struct ApiError {
    /// The HTTP status code.
    pub status: u16,
    /// FCM's own error code, if the body carried one.
    pub error_code: Option<FcmErrorCode>,
    /// The human readable message, or the raw body if it wasn't the usual
    /// JSON error shape.
    pub message: String,
    pub retry_after: Option<RetryAfter>,
}

#[derive(Deserialize)]
struct ErrorBody {
    error: ErrorBodyInner,
}

#[derive(Deserialize)]
struct ErrorBodyInner {
    #[serde(default)]
    message: String,
    #[serde(default)]
    details: Vec<ErrorDetail>,
}

#[derive(Deserialize)]
struct ErrorDetail {
    #[serde(rename = "errorCode")]
    error_code: Option<FcmErrorCode>,
}

impl ApiError {
    pub(crate) fn from_response(status: u16, body: &str, retry_after: Option<RetryAfter>) -> Self {
        let (error_code, message) = match serde_json::from_str::<ErrorBody>(body) {
            Ok(ErrorBody { error }) => (error.details.iter().find_map(|detail| detail.error_code), error.message),
            Err(_) => (None, body.to_string()),
        };
        ApiError {
            status,
            error_code,
            message,
            retry_after,
        }
    }
}

impl fmt::Display for ApiError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "HTTP {}", self.status)?;
        if let Some(code) = self.error_code {
            write!(f, " {:?}", code)?;
        }
        write!(f, ": {}", self.message)
    }
}

/// Fatal errors. Referred from [Firebase
/// documentation](https://firebase.google.com/docs/cloud-messaging/http-server-ref#table9)
#[derive(PartialEq, Debug)]
pub enum FcmError {
    /// The sender account used to send a message couldn't be authenticated. Possible causes are:
    ///
    /// Authorization header missing or with invalid syntax in HTTP request.
    ///
    /// * The Firebase project that the specified server key belongs to is
    ///   incorrect.
    /// * Legacy server keys only—the request originated from a server not
    ///   whitelisted in the Server key IPs.
    ///
    /// Check that the token you're sending inside the Authentication header is
    /// the correct Server key associated with your project. See Checking the
    /// validity of a Server key for details. If you are using a legacy server
    /// key, you're recommended to upgrade to a new key that has no IP
    /// restrictions.
    Unauthorized,

    /// Check that the JSON message is properly formatted and contains valid
    /// fields (for instance, making sure the right data type is passed in).
    InvalidMessage(String),

    /// The server couldn't process the request. Retry the same request, but you must:
    ///
    /// * Honor the [RetryAfter](enum.RetryAfter.html) value if included.
    /// * Implement exponential back-off in your retry mechanism. (e.g. if you
    ///   waited one second before the first retry, wait at least two second
    ///   before the next one, then 4 seconds and so on). If you're sending
    ///   multiple messages, delay each one independently by an additional random
    ///   amount to avoid issuing a new request for all messages at the same time.
    ///
    /// Senders that cause problems risk being blacklisted.
    ServerError(Option<RetryAfter>),

    ProjectIdError(String),

    AuthToken(String),

    /// Any other non-success response (400, 403, 404, 429, ...). Check
    /// [ApiError::error_code]: `Unregistered` means the token is dead.
    Api(ApiError),
}

impl Error for FcmError {}

impl fmt::Display for FcmError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            FcmError::Unauthorized => write!(f, "authorization header missing or with invalid syntax in HTTP request"),
            FcmError::InvalidMessage(ref s) => write!(f, "invalid message {}", s),
            FcmError::ServerError(_) => write!(f, "the server couldn't process the request"),
            FcmError::ProjectIdError(error) => write!(f, "error getting project_id: {error}"),
            FcmError::AuthToken(error) => write!(f, "error getting auth token: {error}"),
            FcmError::Api(error) => write!(f, "{error}"),
        }
    }
}

impl From<reqwest::Error> for FcmError {
    fn from(_: reqwest::Error) -> Self {
        Self::ServerError(None)
    }
}

#[derive(PartialEq, Debug)]
pub enum RetryAfter {
    /// Amount of time to wait until retrying the message is allowed.
    Delay(Duration),

    /// A point in time until retrying the message is allowed.
    DateTime(DateTime<FixedOffset>),
}

impl FromStr for RetryAfter {
    type Err = crate::Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        s.parse::<i64>()
            .map(Duration::seconds)
            .map(RetryAfter::Delay)
            .or_else(|_| DateTime::parse_from_rfc2822(s).map(RetryAfter::DateTime))
            .map_err(|e| crate::Error::InvalidMessage(format!("{}", e)))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::{DateTime, Duration};
    use serde_json::json;

    #[test]
    fn test_some_errors() {
        let errors = vec![
            ("MissingRegistration", ErrorReason::MissingRegistration),
            ("InvalidRegistration", ErrorReason::InvalidRegistration),
            ("NotRegistered", ErrorReason::NotRegistered),
            ("InvalidPackageName", ErrorReason::InvalidPackageName),
            ("MismatchSenderId", ErrorReason::MismatchSenderId),
            ("InvalidParameters", ErrorReason::InvalidParameters),
            ("MessageTooBig", ErrorReason::MessageTooBig),
            ("InvalidDataKey", ErrorReason::InvalidDataKey),
            ("InvalidTtl", ErrorReason::InvalidTtl),
            ("Unavailable", ErrorReason::Unavailable),
            ("InternalServerError", ErrorReason::InternalServerError),
            ("DeviceMessageRateExceeded", ErrorReason::DeviceMessageRateExceeded),
            ("TopicsMessageRateExceeded", ErrorReason::TopicsMessageRateExceeded),
            ("InvalidApnsCredential", ErrorReason::InvalidApnsCredential),
        ];

        for (error_str, error_enum) in errors.into_iter() {
            let response_data = json!({
                "error": error_str,
                "results": [
                    {"error": error_str}
                ]
            });

            let response_string = serde_json::to_string(&response_data).unwrap();
            let fcm_response: FcmResponse = serde_json::from_str(&response_string).unwrap();

            assert_eq!(Some(error_enum.clone()), fcm_response.results.unwrap()[0].error,);

            assert_eq!(Some(error_enum), fcm_response.error,)
        }
    }

    #[test]
    fn test_api_error_from_v1_body() {
        let body = json!({
            "error": {
                "code": 404,
                "message": "Requested entity was not found.",
                "status": "NOT_FOUND",
                "details": [
                    {
                        "@type": "type.googleapis.com/google.firebase.fcm.v1.FcmError",
                        "errorCode": "UNREGISTERED"
                    }
                ]
            }
        });
        let error = ApiError::from_response(404, &body.to_string(), None);
        assert_eq!(error.status, 404);
        assert_eq!(error.error_code, Some(FcmErrorCode::Unregistered));
        assert_eq!(error.message, "Requested entity was not found.");

        let body = json!({
            "error": {
                "code": 400,
                "message": "The registration token is not a valid FCM registration token",
                "status": "INVALID_ARGUMENT",
                "details": [
                    {"@type": "type.googleapis.com/google.rpc.BadRequest", "fieldViolations": []},
                    {"@type": "type.googleapis.com/google.firebase.fcm.v1.FcmError", "errorCode": "SOMETHING_NEW"}
                ]
            }
        });
        let error = ApiError::from_response(400, &body.to_string(), None);
        assert_eq!(error.error_code, Some(FcmErrorCode::Unknown));

        let retry_after = Some(RetryAfter::Delay(Duration::seconds(3)));
        let error = ApiError::from_response(429, "Too Many Requests", retry_after);
        assert_eq!(error.error_code, None);
        assert_eq!(error.message, "Too Many Requests");
        assert_eq!(error.retry_after, Some(RetryAfter::Delay(Duration::seconds(3))));
    }

    #[test]
    fn test_retry_after_from_seconds() {
        assert_eq!(RetryAfter::Delay(Duration::seconds(420)), "420".parse().unwrap());
    }

    #[test]
    fn test_retry_after_from_date() {
        let date = "Sun, 06 Nov 1994 08:49:37 GMT";
        let retry_after = RetryAfter::from_str(date).unwrap();

        assert_eq!(
            RetryAfter::DateTime(DateTime::parse_from_rfc2822(date).unwrap()),
            retry_after,
        );
    }
}
