use crate::{
    exit::ExitHandler,
    tray::{buffer::Buffer, event::TrayEvent},
};

pub(crate) struct TrayState {
    connected: bool,
    buffer: Buffer<5, TrayEvent>,
    exit: ExitHandler,
}

impl ksni::Tray for TrayState {
    fn id(&self) -> String {
        "mpclipboard".to_string()
    }

    fn icon_pixmap(&self) -> Vec<ksni::Icon> {
        const GREEN: &[u8] = include_bytes!("../../assets/green-32x32.rgba");
        const RED: &[u8] = include_bytes!("../../assets/red-32x32.rgba");
        let bytes = if self.connected { GREEN } else { RED };

        vec![ksni::Icon {
            width: 32,
            height: 32,
            data: bytes.to_vec(),
        }]
    }

    fn menu(&self) -> Vec<ksni::MenuItem<Self>> {
        use ksni::menu::*;

        let exit = self.exit.clone();
        self.buffer
            .iter()
            .map(MenuItem::from)
            .chain([
                MenuItem::Separator,
                MenuItem::Standard(StandardItem {
                    label: "Quit".to_string(),
                    activate: Box::new(move |_| exit.trigger()),
                    ..Default::default()
                }),
            ])
            .collect()
    }
}

impl TrayState {
    pub(crate) fn new(exit: ExitHandler) -> Self {
        Self {
            connected: false,
            buffer: Buffer::new(),
            exit,
        }
    }

    pub(crate) fn set_connectivity(&mut self, connectivity: bool) {
        self.connected = connectivity;
    }

    pub(crate) fn push_local(&mut self, text: &str) {
        self.buffer
            .push(TrayEvent::PushedFromLocal(text.to_string()))
    }
    pub(crate) fn push_received(&mut self, text: &str) {
        self.buffer
            .push(TrayEvent::ReceivedFromServer(text.to_string()))
    }
}
