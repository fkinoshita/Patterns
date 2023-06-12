// SPDX-License-Identifier: GPL-3.0-or-later

use gtk::prelude::*;
use gtk::subclass::prelude::*;
use gtk::{gio, glib};

use crate::config::{APP_ID, PKGDATADIR, PROFILE, VERSION};
use crate::window::Window;

mod imp {
    use super::*;

    use adw::subclass::prelude::*;

    #[derive(Debug, Default)]
    pub struct Application {}

    #[glib::object_subclass]
    impl ObjectSubclass for Application {
        const NAME: &'static str = "Application";
        type Type = super::Application;
        type ParentType = adw::Application;
    }

    impl ObjectImpl for Application {}

    impl ApplicationImpl for Application {
        fn activate(&self) {
            self.parent_activate();

            let application = self.obj();

            let window = if let Some(window) = application.active_window() {
                window
            } else {
                let window = Window::new(&application);
                window.upcast()
            };

            window.present();
        }

        fn startup(&self) {
            self.parent_startup();
            let app = self.obj();

            gtk::Window::set_default_icon_name(APP_ID);

            app.setup_gactions();
            app.setup_accels();
        }
    }

    impl GtkApplicationImpl for Application {}
    impl AdwApplicationImpl for Application {}
}

glib::wrapper! {
    pub struct Application(ObjectSubclass<imp::Application>)
        @extends gio::Application, gtk::Application, adw::Application,
        @implements gio::ActionGroup, gio::ActionMap;
}

impl Default for Application {
    fn default() -> Self {
        glib::Object::builder()
            .property("application-id", APP_ID)
            .property("resource-base-path", "/io/github/fkinoshita/Wildcard/")
            .build()
    }
}

impl Application {
    fn setup_gactions(&self) {
        let actions = [gio::ActionEntryBuilder::new("quit")
            .activate(|app: &Self, _, _| {
                if let Some(window) = app.active_window() {
                    window.close();
                }
                app.quit();
            })
            .build()];

        self.add_action_entries(actions);
    }

    fn setup_accels(&self) {
        self.set_accels_for_action("app.quit", &["<Control>q"]);
        self.set_accels_for_action("window.close", &["<Control>w"]);
    }

    pub fn run(&self) -> glib::ExitCode {
        log::info!("Wildcard ({})", APP_ID);
        log::info!("Version: {} ({})", VERSION, PROFILE);
        log::info!("Datadir: {}", PKGDATADIR);

        ApplicationExtManual::run(self)
    }
}
