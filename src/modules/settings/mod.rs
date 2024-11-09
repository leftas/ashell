use self::{
    audio::AudioMessage, bluetooth::BluetoothMessage, network::NetworkMessage, power::PowerMessage,
};
use crate::{
    components::icons::{icon, Icons},
    config::SettingsModuleConfig,
    menu::{Menu, MenuType},
    modules::settings::power::power_menu,
    password_dialog,
    services::{
        audio::{AudioCommand, AudioService},
        bluetooth::{BluetoothCommand, BluetoothService, BluetoothState},
        brightness::{BrightnessCommand, BrightnessService},
        idle_inhibitor::IdleInhibitorManager,
        network::{NetworkCommand, NetworkEvent, NetworkService},
        upower::{PowerProfileCommand, UPowerService},
        ReadOnlyService, Service, ServiceEvent,
    },
    style::{
        HeaderButtonStyle, QuickSettingsButtonStyle, QuickSettingsSubMenuButtonStyle,
        SettingsButtonStyle,
    },
};
use brightness::BrightnessMessage;
use iced::{
    alignment::{Horizontal, Vertical},
    theme::Button,
    widget::{
        button, column, container, horizontal_space, row, text, vertical_rule, Column, Row, Space,
    },
    Alignment, Background, Border, Command, Element, Length, Subscription, Theme,
};
use upower::UPowerMessage;

pub mod audio;
pub mod bluetooth;
pub mod brightness;
pub mod network;
mod power;
mod upower;

pub struct Settings {
    audio: Option<AudioService>,
    brightness: Option<BrightnessService>,
    network: Option<NetworkService>,
    bluetooth: Option<BluetoothService>,
    idle_inhibitor: Option<IdleInhibitorManager>,
    sub_menu: Option<SubMenu>,
    upower: Option<UPowerService>,
    pub password_dialog: Option<(String, String)>,
}

impl Default for Settings {
    fn default() -> Self {
        Settings {
            audio: None,
            brightness: None,
            network: None,
            bluetooth: None,
            idle_inhibitor: IdleInhibitorManager::new(),
            sub_menu: None,
            upower: None,
            password_dialog: None,
        }
    }
}

#[derive(Debug, Clone)]
pub enum Message {
    ToggleMenu,
    UPower(UPowerMessage),
    Network(NetworkMessage),
    Bluetooth(BluetoothMessage),
    Audio(AudioMessage),
    Brightness(BrightnessMessage),
    ToggleInhibitIdle,
    Lock,
    Power(PowerMessage),
    ToggleSubMenu(SubMenu),
    PasswordDialog(password_dialog::Message),
}

#[derive(Debug, PartialEq, Eq, Clone, Copy)]
pub enum SubMenu {
    Power,
    Sinks,
    Sources,
    Wifi,
    Vpn,
    Bluetooth,
}

impl Settings {
    pub fn update(
        &mut self,
        message: Message,
        config: &SettingsModuleConfig,
        menu: &mut Menu,
    ) -> Command<crate::app::Message> {
        match message {
            Message::ToggleMenu => {
                self.sub_menu = None;
                self.password_dialog = None;
                Command::batch(vec![
                    menu.unset_keyboard_interactivity(),
                    menu.toggle(MenuType::Settings),
                ])
            }
            Message::Audio(msg) => match msg {
                AudioMessage::Event(event) => match event {
                    ServiceEvent::Init(service) => {
                        self.audio = Some(service);
                        Command::none()
                    }
                    ServiceEvent::Update(data) => {
                        if let Some(audio) = self.audio.as_mut() {
                            audio.update(data);
                        }
                        Command::none()
                    }
                    ServiceEvent::Error(_) => Command::none(),
                },
                AudioMessage::ToggleSinkMute => {
                    if let Some(audio) = self.audio.as_mut() {
                        let _ = audio.command(AudioCommand::ToggleSinkMute);
                    }
                    Command::none()
                }
                AudioMessage::SinkVolumeChanged(value) => {
                    if let Some(audio) = self.audio.as_mut() {
                        let _ = audio.command(AudioCommand::SinkVolume(value));
                    }
                    Command::none()
                }
                AudioMessage::DefaultSinkChanged(name, port) => {
                    if let Some(audio) = self.audio.as_mut() {
                        let _ = audio.command(AudioCommand::DefaultSink(name, port));
                    }
                    Command::none()
                }
                AudioMessage::ToggleSourceMute => {
                    if let Some(audio) = self.audio.as_mut() {
                        let _ = audio.command(AudioCommand::ToggleSourceMute);
                    }
                    Command::none()
                }
                AudioMessage::SourceVolumeChanged(value) => {
                    if let Some(audio) = self.audio.as_mut() {
                        let _ = audio.command(AudioCommand::SourceVolume(value));
                    }
                    Command::none()
                }
                AudioMessage::DefaultSourceChanged(name, port) => {
                    if let Some(audio) = self.audio.as_mut() {
                        let _ = audio.command(AudioCommand::DefaultSource(name, port));
                    }
                    Command::none()
                }
                AudioMessage::SinksMore => {
                    if let Some(cmd) = &config.audio_sinks_more_cmd {
                        crate::utils::launcher::execute_command(cmd.to_string());
                        menu.close()
                    } else {
                        Command::none()
                    }
                }
                AudioMessage::SourcesMore => {
                    if let Some(cmd) = &config.audio_sources_more_cmd {
                        crate::utils::launcher::execute_command(cmd.to_string());
                        menu.close()
                    } else {
                        Command::none()
                    }
                }
            },
            Message::UPower(msg) => match msg {
                UPowerMessage::Event(event) => match event {
                    ServiceEvent::Init(service) => {
                        self.upower = Some(service);
                        Command::none()
                    }
                    ServiceEvent::Update(data) => {
                        if let Some(upower) = self.upower.as_mut() {
                            upower.update(data);
                        }
                        Command::none()
                    }
                    ServiceEvent::Error(_) => Command::none(),
                },
                UPowerMessage::TogglePowerProfile => {
                    if let Some(upower) = self.upower.as_mut() {
                        upower.command(PowerProfileCommand::Toggle).map(|event| {
                            crate::app::Message::Settings(Message::UPower(UPowerMessage::Event(
                                event,
                            )))
                        })
                    } else {
                        Command::none()
                    }
                }
            },
            Message::Network(msg) => match msg {
                NetworkMessage::Event(event) => match event {
                    ServiceEvent::Init(service) => {
                        self.network = Some(service);
                        Command::none()
                    }
                    ServiceEvent::Update(NetworkEvent::RequestPasswordForSSID(ssid)) => {
                        self.password_dialog = Some((ssid, "".to_string()));
                        menu.set_keyboard_interactivity()
                    }
                    ServiceEvent::Update(data) => {
                        if let Some(network) = self.network.as_mut() {
                            network.update(data);
                        }
                        Command::none()
                    }
                    _ => Command::none(),
                },
                NetworkMessage::ToggleAirplaneMode => {
                    if let Some(network) = self.network.as_mut() {
                        network
                            .command(NetworkCommand::ToggleAirplaneMode)
                            .map(|event| {
                                crate::app::Message::Settings(Message::Network(
                                    NetworkMessage::Event(event),
                                ))
                            })
                    } else {
                        Command::none()
                    }
                }
                NetworkMessage::ToggleWiFi => {
                    if let Some(network) = self.network.as_mut() {
                        network.command(NetworkCommand::ToggleWiFi).map(|event| {
                            crate::app::Message::Settings(Message::Network(NetworkMessage::Event(
                                event,
                            )))
                        })
                    } else {
                        Command::none()
                    }
                }
                NetworkMessage::SelectAccessPoint(ac) => {
                    if let Some(network) = self.network.as_mut() {
                        network
                            .command(NetworkCommand::SelectAccessPoint((ac, None)))
                            .map(|event| {
                                crate::app::Message::Settings(Message::Network(
                                    NetworkMessage::Event(event),
                                ))
                            })
                    } else {
                        Command::none()
                    }
                }
                NetworkMessage::RequestWiFiPassword(ssid) => {
                    self.password_dialog = Some((ssid, "".to_string()));
                    menu.set_keyboard_interactivity()
                }
                NetworkMessage::ScanNearByWiFi => {
                    if let Some(network) = self.network.as_mut() {
                        network
                            .command(NetworkCommand::ScanNearByWiFi)
                            .map(|event| {
                                crate::app::Message::Settings(Message::Network(
                                    NetworkMessage::Event(event),
                                ))
                            })
                    } else {
                        Command::none()
                    }
                }
                NetworkMessage::WiFiMore => {
                    if let Some(cmd) = &config.wifi_more_cmd {
                        crate::utils::launcher::execute_command(cmd.to_string());
                        menu.close()
                    } else {
                        Command::none()
                    }
                }
                NetworkMessage::VpnMore => {
                    if let Some(cmd) = &config.vpn_more_cmd {
                        crate::utils::launcher::execute_command(cmd.to_string());
                        menu.close()
                    } else {
                        Command::none()
                    }
                }
                NetworkMessage::ToggleVpn(vpn) => {
                    if let Some(network) = self.network.as_mut() {
                        network
                            .command(NetworkCommand::ToggleVpn(vpn))
                            .map(|event| {
                                crate::app::Message::Settings(Message::Network(
                                    NetworkMessage::Event(event),
                                ))
                            })
                    } else {
                        Command::none()
                    }
                }
            },
            Message::Bluetooth(msg) => match msg {
                BluetoothMessage::Event(event) => match event {
                    ServiceEvent::Init(service) => {
                        self.bluetooth = Some(service);
                        Command::none()
                    }
                    ServiceEvent::Update(data) => {
                        if let Some(bluetooth) = self.bluetooth.as_mut() {
                            bluetooth.update(data);
                        }
                        Command::none()
                    }
                    _ => Command::none(),
                },
                BluetoothMessage::Toggle => {
                    if let Some(bluetooth) = self.bluetooth.as_mut() {
                        bluetooth.command(BluetoothCommand::Toggle).map(|event| {
                            crate::app::Message::Settings(Message::Bluetooth(
                                BluetoothMessage::Event(event),
                            ))
                        })
                    } else {
                        Command::none()
                    }
                }
                BluetoothMessage::More => {
                    if let Some(cmd) = &config.bluetooth_more_cmd {
                        crate::utils::launcher::execute_command(cmd.to_string());
                        menu.close()
                    } else {
                        Command::none()
                    }
                }
            },
            Message::Brightness(msg) => match msg {
                BrightnessMessage::Event(event) => match event {
                    ServiceEvent::Init(service) => {
                        self.brightness = Some(service);
                        Command::none()
                    }
                    ServiceEvent::Update(data) => {
                        if let Some(brightness) = self.brightness.as_mut() {
                            brightness.update(data);
                        }
                        Command::none()
                    }
                    _ => Command::none(),
                },
                BrightnessMessage::Change(value) => {
                    if let Some(brightness) = self.brightness.as_mut() {
                        brightness
                            .command(BrightnessCommand::Set(value))
                            .map(|event| {
                                crate::app::Message::Settings(Message::Brightness(
                                    BrightnessMessage::Event(event),
                                ))
                            })
                    } else {
                        Command::none()
                    }
                }
            },
            Message::ToggleSubMenu(menu_type) => {
                if self.sub_menu == Some(menu_type) {
                    self.sub_menu.take();
                } else {
                    self.sub_menu.replace(menu_type);

                    if menu_type == SubMenu::Wifi {
                        if let Some(network) = self.network.as_mut() {
                            return network
                                .command(NetworkCommand::ScanNearByWiFi)
                                .map(|event| {
                                    crate::app::Message::Settings(Message::Network(
                                        NetworkMessage::Event(event),
                                    ))
                                });
                        }
                    }
                }

                Command::none()
            }
            Message::ToggleInhibitIdle => {
                if let Some(idle_inhibitor) = &mut self.idle_inhibitor {
                    idle_inhibitor.toggle();
                }
                Command::none()
            }
            Message::Lock => {
                if let Some(lock_cmd) = &config.lock_cmd {
                    crate::utils::launcher::execute_command(lock_cmd.to_string());
                }
                Command::none()
            }
            Message::Power(msg) => {
                msg.update();
                Command::none()
            }
            Message::PasswordDialog(msg) => match msg {
                password_dialog::Message::PasswordChanged(password) => {
                    if let Some((_, current_password)) = &mut self.password_dialog {
                        *current_password = password;
                    }

                    Command::none()
                }
                password_dialog::Message::DialogConfirmed => {
                    if let Some((ssid, password)) = self.password_dialog.take() {
                        let network_command = if let Some(network) = self.network.as_mut() {
                            let ap = network
                                .wireless_access_points
                                .iter()
                                .find(|ap| ap.ssid == ssid)
                                .cloned();
                            if let Some(ap) = ap {
                                network
                                    .command(NetworkCommand::SelectAccessPoint((
                                        ap,
                                        Some(password),
                                    )))
                                    .map(|event| {
                                        crate::app::Message::Settings(Message::Network(
                                            NetworkMessage::Event(event),
                                        ))
                                    })
                            } else {
                                Command::none()
                            }
                        } else {
                            Command::none()
                        };
                        Command::batch(vec![menu.unset_keyboard_interactivity(), network_command])
                    } else {
                        Command::none()
                    }
                }
                password_dialog::Message::DialogCancelled => {
                    if let Some((_, _)) = self.password_dialog.take() {
                        menu.unset_keyboard_interactivity()
                    } else {
                        Command::none()
                    }
                }
            },
        }
    }

    pub fn view(&self) -> Element<Message> {
        button(
            Row::new()
                .push_maybe(
                    self.idle_inhibitor
                        .as_ref()
                        .filter(|i| i.is_inhibited())
                        .map(|_| {
                            container(icon(Icons::EyeOpened)).style(|theme: &Theme| {
                                container::Appearance {
                                    text_color: Some(theme.palette().danger),
                                    ..Default::default()
                                }
                            })
                        }),
                )
                .push_maybe(
                    self.upower
                        .as_ref()
                        .and_then(|p| p.power_profile.indicator()),
                )
                .push_maybe(self.audio.as_ref().and_then(|a| a.sink_indicator()))
                .push(
                    Row::new()
                        .push_maybe(
                            self.network
                                .as_ref()
                                .and_then(|n| n.get_connection_indicator()),
                        )
                        .push_maybe(self.network.as_ref().and_then(|n| n.get_vpn_indicator()))
                        .spacing(4),
                )
                .push_maybe(
                    self.upower
                        .as_ref()
                        .and_then(|upower| upower.battery)
                        .map(|battery| battery.indicator()),
                )
                .spacing(8),
        )
        .style(Button::custom(HeaderButtonStyle::Full))
        .padding([2, 8])
        .on_press(Message::ToggleMenu)
        .into()
    }

    pub fn menu_view(&self, config: &SettingsModuleConfig) -> Element<Message> {
        if let Some((ssid, current_password)) = &self.password_dialog {
            password_dialog::view(ssid, current_password).map(Message::PasswordDialog)
        } else {
            let battery_data = self
                .upower
                .as_ref()
                .and_then(|upower| upower.battery)
                .map(|battery| battery.settings_indicator());
            let right_buttons = Row::new()
                .push_maybe(config.lock_cmd.as_ref().map(|_| {
                    button(icon(Icons::Lock))
                        .padding([8, 13])
                        .on_press(Message::Lock)
                        .style(Button::custom(SettingsButtonStyle))
                }))
                .push(
                    button(icon(if self.sub_menu == Some(SubMenu::Power) {
                        Icons::Close
                    } else {
                        Icons::Power
                    }))
                    .padding([8, 13])
                    .on_press(Message::ToggleSubMenu(SubMenu::Power))
                    .style(Button::custom(SettingsButtonStyle)),
                )
                .spacing(8);

            let header = Row::new()
                .push_maybe(battery_data)
                .push(Space::with_width(Length::Fill))
                .push(right_buttons)
                .spacing(8)
                .width(Length::Fill);

            let (sink_slider, source_slider) = self
                .audio
                .as_ref()
                .map(|a| a.audio_sliders(self.sub_menu))
                .unwrap_or((None, None));

            let wifi_setting_button = self.network.as_ref().and_then(|n| {
                n.get_wifi_quick_setting_button(self.sub_menu, config.wifi_more_cmd.is_some())
            });
            let quick_settings = quick_settings_section(
                vec![
                    wifi_setting_button,
                    self.bluetooth
                        .as_ref()
                        .filter(|b| b.state != BluetoothState::Unavailable)
                        .and_then(|b| {
                            b.get_quick_setting_button(
                                self.sub_menu,
                                config.bluetooth_more_cmd.is_some(),
                            )
                        }),
                    self.network.as_ref().map(|n| {
                        n.get_vpn_quick_setting_button(self.sub_menu, config.vpn_more_cmd.is_some())
                    }),
                    self.network
                        .as_ref()
                        .map(|n| n.get_airplane_mode_quick_setting_button()),
                    self.idle_inhibitor.as_ref().map(|idle_inhibitor| {
                        (
                            quick_setting_button(
                                if idle_inhibitor.is_inhibited() {
                                    Icons::EyeOpened
                                } else {
                                    Icons::EyeClosed
                                },
                                "Idle Inhibitor".to_string(),
                                None,
                                idle_inhibitor.is_inhibited(),
                                Message::ToggleInhibitIdle,
                                None,
                            ),
                            None,
                        )
                    }),
                    self.upower
                        .as_ref()
                        .and_then(|u| u.power_profile.get_quick_setting_button()),
                ]
                .into_iter()
                .flatten()
                .collect::<Vec<_>>(),
            );

            Column::new()
                .push(header)
                .push_maybe(
                    self.sub_menu
                        .filter(|menu_type| *menu_type == SubMenu::Power)
                        .map(|_| sub_menu_wrapper(power_menu().map(Message::Power))),
                )
                .push_maybe(sink_slider)
                .push_maybe(
                    self.sub_menu
                        .filter(|menu_type| *menu_type == SubMenu::Sinks)
                        .and_then(|_| {
                            self.audio.as_ref().map(|a| {
                                sub_menu_wrapper(
                                    a.sinks_submenu(config.audio_sinks_more_cmd.is_some()),
                                )
                            })
                        }),
                )
                .push_maybe(source_slider)
                .push_maybe(
                    self.sub_menu
                        .filter(|menu_type| *menu_type == SubMenu::Sources)
                        .and_then(|_| {
                            self.audio.as_ref().map(|a| {
                                sub_menu_wrapper(
                                    a.sources_submenu(config.audio_sources_more_cmd.is_some()),
                                )
                            })
                        }),
                )
                .push_maybe(self.brightness.as_ref().map(|b| b.brightness_slider()))
                .push(quick_settings)
                .spacing(16)
                .padding(16)
                .max_width(350.)
                .into()
        }
    }

    pub fn subscription(&self) -> Subscription<Message> {
        Subscription::batch(vec![
            UPowerService::subscribe().map(|event| Message::UPower(UPowerMessage::Event(event))),
            AudioService::subscribe().map(|evenet| Message::Audio(AudioMessage::Event(evenet))),
            BrightnessService::subscribe()
                .map(|event| Message::Brightness(BrightnessMessage::Event(event))),
            NetworkService::subscribe().map(|event| Message::Network(NetworkMessage::Event(event))),
            BluetoothService::subscribe()
                .map(|event| Message::Bluetooth(BluetoothMessage::Event(event))),
        ])
    }
}

fn quick_settings_section<'a>(
    buttons: Vec<(Element<'a, Message>, Option<Element<'a, Message>>)>,
) -> Element<'a, Message> {
    let mut section = column!().spacing(8);

    let mut before: Option<(Element<'a, Message>, Option<Element<'a, Message>>)> = None;

    for (button, menu) in buttons.into_iter() {
        if let Some((before_button, before_menu)) = before.take() {
            section = section.push(row![before_button, button].width(Length::Fill).spacing(8));

            if let Some(menu) = before_menu {
                section = section.push(sub_menu_wrapper(menu));
            }

            if let Some(menu) = menu {
                section = section.push(sub_menu_wrapper(menu));
            }
        } else {
            before = Some((button, menu));
        }
    }

    if let Some((before_button, before_menu)) = before.take() {
        section = section.push(
            row![before_button, horizontal_space()]
                .width(Length::Fill)
                .spacing(8),
        );

        if let Some(menu) = before_menu {
            section = section.push(sub_menu_wrapper(menu));
        }
    }

    section.into()
}

fn sub_menu_wrapper<Msg: 'static>(content: Element<Msg>) -> Element<Msg> {
    container(content)
        .style(|theme: &Theme| container::Appearance {
            background: Background::Color(theme.extended_palette().secondary.strong.color).into(),
            border: Border::with_radius(16),
            ..container::Appearance::default()
        })
        .padding(8)
        .width(Length::Fill)
        .into()
}

fn quick_setting_button<'a, Msg: Clone + 'static>(
    icon_type: Icons,
    title: String,
    subtitle: Option<String>,
    active: bool,
    on_press: Msg,
    with_submenu: Option<(SubMenu, Option<SubMenu>, Msg)>,
) -> Element<'a, Msg> {
    let main_content = row!(
        icon(icon_type).size(20),
        Column::new()
            .push(text(title).size(12))
            .push_maybe(subtitle.map(|s| text(s).size(10)))
            .spacing(4)
    )
    .spacing(8)
    .padding([0, 0, 0, 4])
    .width(Length::Fill)
    .align_items(Alignment::Center);

    button(
        Row::new()
            .push(main_content)
            .push_maybe(with_submenu.as_ref().map(|_| vertical_rule(1)))
            .push_maybe(with_submenu.map(|(menu_type, submenu, msg)| {
                button(
                    container(icon(if Some(menu_type) == submenu {
                        Icons::Close
                    } else {
                        Icons::VerticalDots
                    }))
                    .align_y(Vertical::Center)
                    .align_x(Horizontal::Center),
                )
                .padding([4, if Some(menu_type) == submenu { 9 } else { 12 }])
                .style(Button::custom(QuickSettingsSubMenuButtonStyle(active)))
                .width(Length::Shrink)
                .height(Length::Shrink)
                .on_press(msg)
            }))
            .spacing(4)
            .align_items(Alignment::Center)
            .height(Length::Fill),
    )
    .padding([4, 8])
    .on_press(on_press)
    .height(Length::Fill)
    .width(Length::Fill)
    .style(Button::custom(QuickSettingsButtonStyle(active)))
    .width(Length::Fill)
    .height(Length::Fixed(50.))
    .into()
}
