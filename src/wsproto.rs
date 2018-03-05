use serde::{Serialize, Deserialize};
use serde_json;
use stdweb::Value;
use stdweb::unstable::TryInto;
use yew::services::Task;
use yew::html::Callback;

pub type ProtoResult<OUT> = Result<OUT, String>;

pub type ProtoCallback<OUT> = Callback<ProtoResult<OUT>>;

pub enum ProtoStatus {
    Connected,
    Disconnected,
}

pub struct ProtoTask(Option<Value>);

pub struct ProtoService {
    reference: Option<Value>,
}

impl ProtoService {
    pub fn new() -> Self {
        ProtoService {
            reference: None,
        }
    }

    pub fn connect(&mut self, url: &str, notification: Callback<ProtoStatus>) {
        let notify = move |code: u32| {
            let status = {
                match code {
                    0 => ProtoStatus::Disconnected,
                    1 => ProtoStatus::Connected,
                    x => panic!("unknown status code for mould runner: {}", x),
                }
            };
            println!("NOTIFIED!");
            notification.emit(status);
            println!("AFTER NOTIFIED!");
        };
        let handle = js! {
            var task = {
                active: false,
                error: null,
                callback: function() { },
            };
            var socket = new WebSocket(@{url});
            var notify = @{notify};
            var send_and_drop = function(success, data) {
                if (task.active) {
                    task.active = false;
                    task.callback(success, data);
                    task.callback.drop();
                }
            };
            socket.onopen = function(event) {
                console.log("ON_OPEN");
                notify(1);
            };
            socket.close = function(event) {
                console.log("ON_CLOSE");
                notify(0);
                notify.drop();
                send_and_drop(false, "connection closed");
            };
            socket.onmessage = function(event) {
                console.log("ON_MESSAGE");
                send_and_drop(true, event.data);
            };
            socket.onerror = function(reason) {
                console.log("ON_ERROR");
                send_and_drop(false, reason);
            };
            var reserve_task = function(new_task) {
                console.log("NEW TASK: ", new_task);
                if (!task.active) {
                    task = new_task;
                }
            };
            var execute_task = function(the_task, request) {
                console.log("EXECUTING: ", the_task);
                if (task == the_task) {
                    if (task.active) {
                        console.log("INTERNAL!");
                        try {
                            socket.send(request);
                            console.log("SENT!!!");
                        } catch (e) {
                            console.log("ERROR: ", e.message, callback);
                            send_and_drop(false, e.message);
                            console.log("Callback called^^^^");
                        }
                    } else {
                        console.log("TASK ALREADY CANCELED");
                        send_and_drop(false, "Task already canceled!");
                    }
                } else {
                    console.log("HAS ACTIVE TASK!");
                    send_and_drop(false, "Has active task!");
                }
            };
            return {
                reserve_task,
                execute_task,
                socket,
            };
        };
        self.reference = Some(handle);
    }

    pub fn request<IN, OUT: 'static>(&mut self, service: &str, action: &str, input: &IN, handler: ProtoCallback<OUT>) -> ProtoTask
    where
        IN: Serialize,
        OUT: for <'de> Deserialize<'de>,
    {
        println!("Request Mould Task: {} - {}", service, action);
        if self.reference.is_none() {
            panic!("Mould Client not initialized!");
        }
        let payload = serde_json::to_value(input).unwrap();
        let request = Input {
            service: service.into(),
            action: action.into(),
            payload,
        };
        let request = serde_json::to_string(&request).unwrap();
        let callback = move |success: bool, data: String| {
            use std::error::Error;
            let result = {
                if success {
                    let output = serde_json::from_str::<Output>(&data);
                    match output {
                        Ok(Output::Item(item)) => {
                            serde_json::from_value(item)
                                .or_else(|e| Err(e.description().into()))
                        }
                        Ok(Output::Fail(reason)) => {
                            Err(reason)
                        }
                        Err(e) => {
                            Err(e.description().into())
                        }
                    }
                } else {
                    Err(data)
                }
            };
            handler.emit(result);
        };
        println!("HERE!");
        if let Some(ref handle) = self.reference {
            let task_ref = js! {
                var request = @{request};
                var active = true;
                var callback = @{callback};
                var handle = @{handle};
                var task = { active, callback };
                handle.reserve_task(task);
                // Important! We need to use timeout to prevent `deep call` when
                // messages pool is empty during loop call.
                var bind = {
                    "loop": function() {
                        handle.execute_task(task, request);
                    }
                };
                _yew_schedule_(bind);
                return task;
            };
            println!("REF: {:?}", task_ref);
            ProtoTask(Some(task_ref))
        } else {
            panic!("no reference to a connection (maybe closed)");
        }
    }
}

// -> Get them from `mould` crate

#[derive(Serialize, Deserialize)]
pub struct Input {
    pub service: String,
    pub action: String,
    pub payload: serde_json::Value,
}

#[derive(Serialize, Deserialize)]
#[serde(tag = "event", content = "data", rename_all = "lowercase")]
pub enum Output {
    Item(serde_json::Value),
    Fail(String),
}

// <-

impl Task for ProtoTask {
    fn is_active(&self) -> bool {
        if let Some(ref task) = self.0 {
            let result = js! {
                var the_task = @{task};
                return the_task.active;
            };
            result.try_into().unwrap_or(false)
        } else {
            false
        }
    }
    fn cancel(&mut self) {
        let task = self.0.take().expect("tried to cancel mould task twice");
        js! {
            var the_task = @{task};
            console.log("Cancelling mould task...", the_task);
            if (the_task.active) {
                the_task.active = false;
                the_task.callback.drop();
            }
        }
    }
}

impl Drop for ProtoTask {
    fn drop(&mut self) {
        if self.is_active() {
            println!("Dropping mould task");
            self.cancel();
        }
    }
}
