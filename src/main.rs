#[macro_use]
extern crate stdweb;

use stdweb::js_export;
use stdweb::web::FileReader;

extern crate mavlink;

use mavlink::MavConnection;

use std::sync::Arc;

extern crate serde_json;
use serde_json::json;

mod filereader;

#[js_export]
fn run(file: FileReader) -> String {
    console!(log, "Started!");
    let mut mavconn = filereader::open(&file).unwrap();

    // TODO: Let the user choose
    mavconn.set_protocol_version(mavlink::MavlinkVersion::V1);

    let vehicle = Arc::new(mavconn);

    let mut json = json!({"mavlink":{}});
    loop {
        match vehicle.recv() {
            Ok((_header, msg)) => {
                let value = serde_json::to_value(&msg).unwrap();
                let msg_type = value["type"].to_string().replace("\"", "");
                match json["mavlink"][&msg_type].as_array_mut() {
                    Some(array) => {
                        array.push(value);
                    }
                    _ => {
                        json["mavlink"][&msg_type] = serde_json::Value::Array(Vec::new());
                        json["mavlink"][&msg_type]
                            .as_array_mut()
                            .unwrap()
                            .push(value);
                    }
                }
            }
            Err(e) => match e.kind() {
                std::io::ErrorKind::WouldBlock => {
                    return format!("WouldBlock");
                }
                _ => {
                    js! {
                        download(@{json.to_string()}, "log.json");
                    }
                    console!(log, "recv error: {:?}", format!("{:?}", e));
                    break format!("{:?}", e);
                }
            },
        }
    }
}

fn main() {
    stdweb::initialize();
    stdweb::event_loop();
}
