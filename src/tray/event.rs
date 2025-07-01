use crate::tray::TrayState;

pub(crate) enum TrayEvent {
    PushedFromLocal(String),
    ReceivedFromServer(String),
}

impl From<&TrayEvent> for ksni::menu::MenuItem<TrayState> {
    fn from(event: &TrayEvent) -> Self {
        let label = match event {
            TrayEvent::PushedFromLocal(text) => format!("-> {text}"),
            TrayEvent::ReceivedFromServer(text) => format!("<- {text}"),
        };
        Self::Standard(ksni::menu::StandardItem {
            label,
            enabled: false,
            ..Default::default()
        })
    }
}
