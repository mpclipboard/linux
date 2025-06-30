use crate::exit_handler::ExitHandler;

pub(crate) struct Tray {
    connected: bool,
    exit: ExitHandler,
}

const GREEN: &[u8] = include_bytes!("../assets/green-32x32.rgba");
const RED: &[u8] = include_bytes!("../assets/red-32x32.rgba");

impl ksni::Tray for Tray {
    fn id(&self) -> String {
        "mpclipboard".to_string()
    }

    fn icon_pixmap(&self) -> Vec<ksni::Icon> {
        let bytes = if self.connected { GREEN } else { RED };

        vec![ksni::Icon {
            width: 32,
            height: 32,
            data: bytes.to_vec(),
        }]
    }

    fn menu(&self) -> Vec<ksni::MenuItem<Self>> {
        let exit = self.exit.clone();

        vec![
            ksni::menu::StandardItem {
                label: "Quit".to_string(),
                icon_name: "application-exit".into(),
                activate: Box::new(move |_| exit.trigger_manually()),
                ..Default::default()
            }
            .into(),
        ]
    }
}

impl Tray {
    pub(crate) fn new(exit: ExitHandler) -> Self {
        Self {
            connected: false,
            exit,
        }
    }

    pub(crate) fn set_connectivity(&mut self, connectivity: bool) {
        self.connected = connectivity;
    }
}
