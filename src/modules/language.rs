use crate::style::header_pills;
use hyprland::{data::Devices, event_listener::AsyncEventListener, shared::HyprData};
use iced::{
    subscription::channel,
    widget::{container, text},
    Element, Subscription,
};
use log::{debug, error};
use std::{
    any::TypeId,
    sync::{Arc, RwLock},
};

pub struct Language {
    value: Option<String>,
}

#[derive(Debug, Clone)]
pub enum Message {
    LanguageChanged(Option<String>),
}

impl Default for Language {
    fn default() -> Self {
        let init = Some(match Devices::get() {
            Ok(x) => match x.keyboards.into_iter().find(|x| x.main) {
                Some(x) => x.active_keymap,
                None => "Unknown".to_string(),
            },
            Err(_) => "Unknwon".to_string(),
        });

        Self { value: init }
    }
}

impl Language {
    pub fn update(&mut self, message: Message) {
        match message {
            Message::LanguageChanged(value) => {
                self.value = value;
            }
        }
    }

    pub fn view(&self) -> Option<Element<Message>> {
        self.value.as_ref().map(|value| {
            container(text(value).size(12))
                .padding([4, 8])
                .style(header_pills)
                .into()
        })
    }

    pub fn subscription(&self) -> Subscription<Message> {
        let id = TypeId::of::<Self>();
        channel(id, 10, |output| async move {
            let output = Arc::new(RwLock::new(output));
            loop {
                let mut event_listener = AsyncEventListener::new();

                event_listener.add_layout_changed_handler({
                    let output = output.clone();
                    move |_| {
                        let output = output.clone();
                        Box::pin(async move {
                            if let Ok(mut output) = output.write() {
                                let current = Some(match Devices::get() {
                                    Ok(x) => match x.keyboards.into_iter().find(|x| x.main) {
                                        Some(x) => x.active_keymap,
                                        None => "Unknown".to_string(),
                                    },
                                    Err(_) => "Unknwon".to_string(),
                                });

                                debug!("Sending language changed message");
                                output.try_send(Message::LanguageChanged(current)).unwrap();
                            }
                        })
                    }
                });

                debug!("Starting language listener");

                let res = event_listener.start_listener_async().await;

                if let Err(e) = res {
                    error!("restarting active window listener due to error: {:?}", e);
                }
            }
        })
    }
}
