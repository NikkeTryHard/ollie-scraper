use serde::{Deserialize, Serialize};

/// Discord Gateway message wrapper
#[derive(Debug, Deserialize, Serialize)]
pub struct GatewayMessage {
    pub op: u8,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub s: Option<u64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub t: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub d: Option<serde_json::Value>,
}

/// Hello payload (op 10)
#[derive(Debug, Deserialize)]
pub struct HelloPayload {
    pub heartbeat_interval: u64,
}

/// Identify payload (op 2)
#[derive(Debug, Serialize)]
pub struct IdentifyPayload {
    pub token: String,
    pub properties: IdentifyProperties,
}

/// Properties block for Identify payload
#[derive(Debug, Serialize)]
pub struct IdentifyProperties {
    pub os: String,
    pub browser: String,
    pub device: String,
}

/// Channel object
#[derive(Debug, Deserialize)]
pub struct Channel {
    pub id: String,
    pub name: Option<String>,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_deserialize_hello_message() {
        let json = r#"{
            "op": 10,
            "d": {
                "heartbeat_interval": 41250
            }
        }"#;

        let msg: GatewayMessage = serde_json::from_str(json).expect("Failed to parse Hello message");
        assert_eq!(msg.op, 10);
        assert!(msg.t.is_none());
        assert!(msg.d.is_some());

        let hello: HelloPayload = serde_json::from_value(msg.d.unwrap()).expect("Failed to parse Hello payload");
        assert_eq!(hello.heartbeat_interval, 41250);
    }

    #[test]
    fn test_deserialize_channel_update_message() {
        let json = r#"{
            "op": 0,
            "t": "CHANNEL_UPDATE",
            "d": {
                "id": "123456789",
                "name": "general-chat"
            }
        }"#;

        let msg: GatewayMessage = serde_json::from_str(json).expect("Failed to parse CHANNEL_UPDATE message");
        assert_eq!(msg.op, 0);
        assert_eq!(msg.t, Some("CHANNEL_UPDATE".to_string()));
        assert!(msg.d.is_some());

        let channel: Channel = serde_json::from_value(msg.d.unwrap()).expect("Failed to parse Channel");
        assert_eq!(channel.id, "123456789");
        assert_eq!(channel.name, Some("general-chat".to_string()));
    }

    #[test]
    fn test_serialize_identify_payload() {
        let identify = IdentifyPayload {
            token: "my_secret_token".to_string(),
            properties: IdentifyProperties {
                os: "linux".to_string(),
                browser: "rust".to_string(),
                device: "rust".to_string(),
            },
        };

        let json = serde_json::to_string(&identify).expect("Failed to serialize Identify payload");
        assert!(json.contains("my_secret_token"));
        assert!(json.contains("linux"));
        assert!(json.contains("rust"));

        // Verify it can be parsed back as valid JSON
        let value: serde_json::Value = serde_json::from_str(&json).expect("Serialized JSON is invalid");
        assert_eq!(value["token"], "my_secret_token");
        assert_eq!(value["properties"]["os"], "linux");
    }
}
