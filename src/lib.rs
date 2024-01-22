#![doc(html_root_url = "https://panicbit.github.io/fcm-rust/fcm/")]
//! fcm
//! ===
//!
//! A client for asynchronous sending of Firebase Cloud Messages, or Push Notifications.
//!
//! # Examples:
//!
//! To send out a FCM Message with some custom data:
//!
//! ```no_run
//! # use std::collections::HashMap;
//! # #[tokio::main]
//! # async fn main() -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
//! use fcm::Target;
//! let client = fcm::Client::new();
//!
//! let mut map = HashMap::new();
//! map.insert("message", "Howdy!");
//!
//! let mut builder = fcm::MessageBuilder::new(Target::Token("token".to_string()));
//! builder.data(&map);
//!
//! let response = client.send(builder.finalize()).await?;
//! println!("Sent: {:?}", response);
//! # Ok(())
//! # }
//! ```
//!
//! To send a message using FCM Notifications, we first build the notification:
//!
//! ```rust
//! # fn main() {
//! let mut builder = fcm::NotificationBuilder::new();
//! builder.title("Hey!".to_string());
//! builder.body("Do you want to catch up later?".to_string());
//! let notification = builder.finalize();
//! # }
//! ```
//!
//! And then set it in the message, before sending it:
//!
//! ```no_run
//! # #[tokio::main]
//! # async fn main() -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
//! use fcm::Target;
//! let client = fcm::Client::new();
//!
//! let mut notification_builder = fcm::NotificationBuilder::new();
//! notification_builder.title("Hey!".to_string());
//! notification_builder.body("Do you want to catch up later?".to_string());
//!
//! let notification = notification_builder.finalize();
//! let mut message_builder = fcm::MessageBuilder::new(Target::Token("token".to_string()));
//! message_builder.notification(notification);
//!
//! let response = client.send(message_builder.finalize()).await?;
//! println!("Sent: {:?}", response);
//! # Ok(())
//! # }
//! ```

mod message;
pub use crate::message::fcm_options::*;
pub use crate::message::target::*;
pub use crate::message::*;

mod notification;
pub use crate::notification::*;

mod android;
pub use crate::android::android_config::*;
pub use crate::android::android_fcm_options::*;
pub use crate::android::android_message_priority::*;
pub use crate::android::android_notification::*;
pub use crate::android::color::*;
pub use crate::android::light_settings::*;
pub use crate::android::notification_priority::*;
pub use crate::android::visibility::*;

mod apns;
pub use crate::apns::apns_config::*;
pub use crate::apns::apns_fcm_options::*;

mod web;
pub use crate::web::webpush_config::*;
pub use crate::web::webpush_fcm_options::*;

mod client;
pub use crate::client::response::FcmError as Error;
pub use crate::client::*;
