// SPDX-License-Identifier: GPL-3.0-or-later

use once_cell::sync::Lazy;

use adw::subclass::prelude::*;
use gtk::prelude::*;
use gtk::{gio, glib};

use crate::config::{APP_ID, PROFILE};

mod imp {
    use glib::subclass::Signal;

    use super::*;

    #[derive(Debug, gtk::CompositeTemplate)]
    #[template(resource = "/io/github/fkinoshita/Wildcard/ui/flags_dialog.ui")]
    pub struct FlagsDialog {
        pub settings: gio::Settings,

        #[template_child]
        pub multiline_switch: TemplateChild<gtk::Switch>,
        #[template_child]
        pub case_insensitive_switch: TemplateChild<gtk::Switch>,
        #[template_child]
        pub ignore_whitespace_switch: TemplateChild<gtk::Switch>,
        #[template_child]
        pub dot_matches_newline_switch: TemplateChild<gtk::Switch>,
        #[template_child]
        pub unicode_switch: TemplateChild<gtk::Switch>,
        #[template_child]
        pub greed_switch: TemplateChild<gtk::Switch>,
    }

    impl Default for FlagsDialog {
        fn default() -> Self {
            Self {
                settings: gio::Settings::new(APP_ID),

                multiline_switch: TemplateChild::default(),
                case_insensitive_switch: TemplateChild::default(),
                ignore_whitespace_switch: TemplateChild::default(),
                dot_matches_newline_switch: TemplateChild::default(),
                unicode_switch: TemplateChild::default(),
                greed_switch: TemplateChild::default(),
            }
        }
    }

    #[glib::object_subclass]
    impl ObjectSubclass for FlagsDialog {
        const NAME: &'static str = "FlagsDialog";
        type Type = super::FlagsDialog;
        type ParentType = adw::Window;

        fn class_init(klass: &mut Self::Class) {
            klass.bind_template();
        }

        fn instance_init(obj: &glib::subclass::InitializingObject<Self>) {
            obj.init_template();
        }
    }

    impl ObjectImpl for FlagsDialog {
        fn signals() -> &'static [Signal] {
            static SIGNALS: Lazy<Vec<Signal>> =
                Lazy::new(|| vec![Signal::builder("flags-changed").action().build()]);
            SIGNALS.as_ref()
        }

        fn constructed(&self) {
            self.parent_constructed();
            let obj = self.obj();

            // Devel profile
            if PROFILE == "Devel" {
                obj.add_css_class("devel");
            }

            obj.load_flags();
        }
    }

    impl WidgetImpl for FlagsDialog {}
    impl WindowImpl for FlagsDialog {}
    impl AdwWindowImpl for FlagsDialog {}
}

glib::wrapper! {
    pub struct FlagsDialog(ObjectSubclass<imp::FlagsDialog>)
        @extends gtk::Widget, gtk::Window, adw::Window;
}

impl FlagsDialog {
    #[allow(clippy::new_without_default)]
    pub fn new() -> Self {
        glib::Object::builder().build()
    }

    fn load_flags(&self) {
        let imp = self.imp();

        self.bind_flag("multiline-flag", &imp.multiline_switch);
        self.bind_flag("case-insensitive-flag", &imp.case_insensitive_switch);
        self.bind_flag("ignore-whitespace-flag", &imp.ignore_whitespace_switch);
        self.bind_flag("dot-matches-newline-flag", &imp.dot_matches_newline_switch);
        self.bind_flag("unicode-flag", &imp.unicode_switch);
        self.bind_flag("greed-flag", &imp.greed_switch);
    }

    fn bind_flag(&self, setting_name: &str, switch: &TemplateChild<gtk::Switch>) {
        let imp = self.imp();

        imp.settings.bind(setting_name, &**switch, "active").build();
        imp.settings.connect_changed(
            Some(setting_name),
            glib::clone!(@weak self as dialog => move |_setting, _value| {
                dialog.emit_by_name::<()>("flags-changed", &[]);
            }),
        );
    }
}
