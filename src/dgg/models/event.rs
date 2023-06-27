use crate::dgg::models::user::User;
use anyhow::{anyhow, Context, Result};
use chrono::serde::ts_milliseconds_option;
use chrono::{DateTime, Utc};
use enum_dispatch::enum_dispatch;
use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::collections::HashMap;

const EVENT_ME: &str = "ME";
const EVENT_SERVED_CONNECTIONS: &str = "NAMES";
const EVENT_USER_JOINED: &str = "JOIN";
const EVENT_USER_QUIT: &str = "QUIT";
const EVENT_BROADCAST: &str = "BROADCAST";
const EVENT_CHAT_MESSAGE: &str = "MSG";
const EVENT_WHISPER: &str = "PRIVMSG";
const EVENT_WHISPER_SENT: &str = "PRIVMSGSENT";
const EVENT_MUTE: &str = "MUTE";
const EVENT_UNMUTE: &str = "UNMUTE";
const EVENT_BAN: &str = "BAN";
const EVENT_UNBAN: &str = "UNBAN";
const EVENT_SUB_ONLY: &str = "SUBONLY";
const EVENT_PIN: &str = "PIN";
const EVENT_ERROR_MESSAGE: &str = "ERR";
const EVENT_BEFORE_EVERY_MESSAGE: &str = "BEFORE_EVERY_MESSAGE";
const EVENT_AFTER_EVERY_MESSAGE: &str = "AFTER_EVERY_MESSAGE";
const EVENT_MENTION: &str = "MENTION";
const EVENT_WEBSOCKET_ERROR: &str = "WS_ERROR";
const EVENT_WEBSOCKET_CLOSE: &str = "WS_CLOSE";
const EVENT_HANDLER_ERROR: &str = "HANDLER_ERROR";

#[derive(Debug, Default, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct BaseEventData {
    #[serde(flatten)]
    pub user: Option<User>,
    #[serde(flatten)]
    pub extra: Option<HashMap<String, Value>>,
    #[serde(
        with = "ts_milliseconds_option",
        default,
        skip_serializing_if = "Option::is_none"
    )]
    pub timestamp: Option<DateTime<Utc>>,
}

#[derive(Debug, Default, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct EventData<T> {
    #[serde(flatten)]
    pub data: T,
    #[serde(flatten)]
    pub base: BaseEventData,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[enum_dispatch(EventType)]
#[serde(untagged)]
pub enum Event {
    Connected(BaseEventData),
    ServedConnections(EventData<ServedConnectionsData>),
    UserJoined(BaseEventData),
    UserQuit(BaseEventData),
    Broadcast(BaseEventData),
    ChatMessage(EventData<ChatMessageData>),
    Whisper(BaseEventData),
    WhisperSent(BaseEventData),
    Mute(BaseEventData),
    Unmute(BaseEventData),
    Ban(BaseEventData),
    Unban(BaseEventData),
    SubOnly(BaseEventData),
    Pin(BaseEventData),
    ErrorMessage(BaseEventData),
    BeforeEveryMessage(BaseEventData),
    AfterEveryMessage(BaseEventData),
    Mention(BaseEventData),
    WebSocketError(BaseEventData),
    WebSocketClose(BaseEventData),
    HandlerError(BaseEventData),
    Unknown(EventData<Value>),
}

impl TryFrom<&str> for Event {
    type Error = anyhow::Error;

    fn try_from(value: &str) -> Result<Self, Self::Error> {
        let (event_type, mut event_json) = value
            .split_once(' ')
            .context("Expected a string in the form <event_type> [<event_json>|<\"null\">]")?;

        if event_json.eq("null") {
            event_json = "{}";
        }

        let data = match event_type {
            EVENT_ME => Event::Connected(serde_json::from_str(event_json)?),
            EVENT_SERVED_CONNECTIONS => Event::ServedConnections(serde_json::from_str(event_json)?),
            EVENT_USER_JOINED => Event::UserJoined(serde_json::from_str(event_json)?),
            EVENT_USER_QUIT => Event::UserQuit(serde_json::from_str(event_json)?),
            EVENT_BROADCAST => Event::Broadcast(serde_json::from_str(event_json)?),
            EVENT_CHAT_MESSAGE => Event::ChatMessage(serde_json::from_str(event_json)?),
            EVENT_WHISPER => Event::Whisper(serde_json::from_str(event_json)?),
            EVENT_WHISPER_SENT => Event::WhisperSent(serde_json::from_str(event_json)?),
            EVENT_MUTE => Event::Mute(serde_json::from_str(event_json)?),
            EVENT_UNMUTE => Event::Unmute(serde_json::from_str(event_json)?),
            EVENT_BAN => Event::Ban(serde_json::from_str(event_json)?),
            EVENT_UNBAN => Event::Unban(serde_json::from_str(event_json)?),
            EVENT_SUB_ONLY => Event::SubOnly(serde_json::from_str(event_json)?),
            EVENT_PIN => Event::Pin(serde_json::from_str(event_json)?),
            EVENT_ERROR_MESSAGE => Event::ErrorMessage(serde_json::from_str(event_json)?),
            EVENT_BEFORE_EVERY_MESSAGE => {
                Event::BeforeEveryMessage(serde_json::from_str(event_json)?)
            }
            EVENT_AFTER_EVERY_MESSAGE => {
                Event::AfterEveryMessage(serde_json::from_str(event_json)?)
            }
            EVENT_MENTION => Event::Mention(serde_json::from_str(event_json)?),
            EVENT_WEBSOCKET_ERROR => Event::WebSocketError(serde_json::from_str(event_json)?),
            EVENT_WEBSOCKET_CLOSE => Event::WebSocketClose(serde_json::from_str(event_json)?),
            EVENT_HANDLER_ERROR => Event::HandlerError(serde_json::from_str(event_json)?),
            _ => Event::Unknown(serde_json::from_str(event_json)?),
        };

        Ok(data)
    }
}

impl TryFrom<Event> for String {
    type Error = anyhow::Error;

    fn try_from(value: Event) -> std::result::Result<Self, Self::Error> {
        let event_type = match value {
            Event::Connected(_) => EVENT_ME,
            Event::ServedConnections(_) => EVENT_SERVED_CONNECTIONS,
            Event::UserJoined(_) => EVENT_USER_JOINED,
            Event::UserQuit(_) => EVENT_USER_QUIT,
            Event::Broadcast(_) => EVENT_BROADCAST,
            Event::ChatMessage(_) => EVENT_CHAT_MESSAGE,
            Event::Whisper(_) => EVENT_WHISPER,
            Event::WhisperSent(_) => EVENT_WHISPER_SENT,
            Event::Mute(_) => EVENT_MUTE,
            Event::Unmute(_) => EVENT_UNMUTE,
            Event::Ban(_) => EVENT_BAN,
            Event::Unban(_) => EVENT_UNBAN,
            Event::SubOnly(_) => EVENT_SUB_ONLY,
            Event::Pin(_) => EVENT_PIN,
            Event::ErrorMessage(_) => EVENT_ERROR_MESSAGE,
            Event::BeforeEveryMessage(_) => EVENT_BEFORE_EVERY_MESSAGE,
            Event::AfterEveryMessage(_) => EVENT_AFTER_EVERY_MESSAGE,
            Event::Mention(_) => EVENT_MENTION,
            Event::WebSocketError(_) => EVENT_WEBSOCKET_ERROR,
            Event::WebSocketClose(_) => EVENT_WEBSOCKET_CLOSE,
            Event::HandlerError(_) => EVENT_HANDLER_ERROR,
            Event::Unknown(_) => return Err(anyhow!("Unknown event type")),
        };

        let event_json = serde_json::to_string(&value)?;

        Ok(format!("{} {}", event_type, event_json))
    }
}

#[derive(Debug, Serialize, Deserialize, Clone, PartialEq, Eq, Hash)]
pub struct ServedConnectionsData {
    #[serde(rename = "users")]
    pub users: Vec<User>,
    #[serde(rename = "connectioncount")]
    pub connection_count: u32,
}

#[derive(Debug, Serialize, Deserialize, Clone, PartialEq, Eq, Hash)]
pub struct ChatMessageData {
    pub data: String,
}

#[cfg(test)]
mod tests {
    use crate::dgg::models::event::Event;
    use anyhow::Result;

    const SAMPLE_EVENT_NAMES: &str = include_resource!("test_samples", "events", "NAMES");

    #[test]
    fn parse_event_names() -> Result<()> {
        let event = Event::try_from(SAMPLE_EVENT_NAMES)?;
        debug!("{:?}", event);

        assert!(matches!(event, Event::ServedConnections(_)));
        Ok(())
    }

    #[test]
    fn parse_event_quit() -> Result<()> {
        let event = Event::try_from(include_resource!("test_samples", "events", "QUIT"))?;
        debug!("{:?}", event);

        assert!(matches!(event, Event::UserQuit(_)));
        Ok(())
    }

    #[test]
    fn parse_event_msg() -> Result<()> {
        let event = Event::try_from(include_resource!("test_samples", "events", "MSG"))?;
        debug!("{:?}", event);

        assert!(matches!(event, Event::ChatMessage(_)));
        Ok(())
    }

    #[test]
    fn parse_event_pin() -> Result<()> {
        let event = Event::try_from(include_resource!("test_samples", "events", "PIN"))?;
        debug!("{:?}", event);

        assert!(matches!(event, Event::Pin(_)));
        Ok(())
    }
}
