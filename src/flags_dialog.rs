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
            klass.bind_template_callbacks();
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

    #[gtk::template_callbacks]
    impl FlagsDialog {
        #[template_callback]
        fn on_multiline_changed(&self) {
            if let Err(err) = self
                .obj()
                .set_flag_setting("multiline-flag", self.multiline_switch.is_active())
            {
                log::error!("Failed to save multiline flag, {}", &err);
            }
        }

        #[template_callback]
        fn on_case_insensitive_changed(&self) {
            if let Err(err) = self.obj().set_flag_setting(
                "case-insensitive-flag",
                self.case_insensitive_switch.is_active(),
            ) {
                log::error!("Failed to save case insensitive flag, {}", &err);
            }
        }

        #[template_callback]
        fn on_ignore_whitespace_changed(&self) {
            if let Err(err) = self.obj().set_flag_setting(
                "ignore-whitespace-flag",
                self.ignore_whitespace_switch.is_active(),
            ) {
                log::error!("Failed to save ignore whitespace flag, {}", &err);
            }
        }

        #[template_callback]
        fn on_dot_matches_newline_changed(&self) {
            if let Err(err) = self.obj().set_flag_setting(
                "dot-matches-newline-flag",
                self.dot_matches_newline_switch.is_active(),
            ) {
                log::error!("Failed to save dot matches newline flag, {}", &err);
            }
        }

        #[template_callback]
        fn on_unicode_changed(&self) {
            if let Err(err) = self
                .obj()
                .set_flag_setting("unicode-flag", self.unicode_switch.is_active())
            {
                log::error!("Failed to save unicode flag, {}", &err);
            }
        }

        #[template_callback]
        fn on_greed_changed(&self) {
            if let Err(err) = self
                .obj()
                .set_flag_setting("greed-flag", self.greed_switch.is_active())
            {
                log::error!("Failed to save greed flag, {}", &err);
            }
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
    #[warn(clippy::new_without_default)]
    pub fn new() -> Self {
        glib::Object::builder().build()
    }

    fn load_flags(&self) {
        let imp = self.imp();

        imp.multiline_switch
            .set_active(imp.settings.boolean("multiline-flag"));
        imp.case_insensitive_switch
            .set_active(imp.settings.boolean("case-insensitive-flag"));
        imp.ignore_whitespace_switch
            .set_active(imp.settings.boolean("ignore-whitespace-flag"));
        imp.dot_matches_newline_switch
            .set_active(imp.settings.boolean("dot-matches-newline-flag"));
        imp.unicode_switch
            .set_active(imp.settings.boolean("unicode-flag"));
        imp.greed_switch
            .set_active(imp.settings.boolean("greed-flag"));
    }

    fn set_flag_setting(
        &self,
        setting_name: &str,
        setting_value: bool,
    ) -> Result<(), glib::BoolError> {
        let imp = self.imp();

        imp.settings.set_boolean(setting_name, setting_value)?;
        self.emit_by_name::<()>("flags-changed", &[]);

        Ok(())
    }
}
