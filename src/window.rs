// SPDX-License-Identifier: GPL-3.0-or-later

use regex::{Regex, RegexBuilder};

use adw::subclass::prelude::*;
use gtk::prelude::*;
use gtk::{gio, glib};

use gettextrs::gettext;

use crate::i18n::ngettext_f;

use crate::application::Application;
use crate::config::{APP_ID, PROFILE, VERSION};

mod imp {
    use super::*;

    #[derive(Debug, gtk::CompositeTemplate)]
    #[template(resource = "/io/github/fkinoshita/Wildcard/ui/window.ui")]
    pub struct Window {
        pub settings: gio::Settings,

        #[template_child]
        pub regex_text_view: TemplateChild<gtk::TextView>,
        #[template_child]
        pub regex_placeholder: TemplateChild<gtk::Label>,
        #[template_child]
        pub test_placeholder: TemplateChild<gtk::Label>,
        #[template_child]
        pub regex_buffer: TemplateChild<gtk::TextBuffer>,
        #[template_child]
        pub test_buffer: TemplateChild<gtk::TextBuffer>,
        #[template_child]
        pub matches_label: TemplateChild<gtk::Label>,
    }

    impl Default for Window {
        fn default() -> Self {
            Self {
                settings: gio::Settings::new(APP_ID),

                regex_text_view: TemplateChild::default(),
                regex_placeholder: TemplateChild::default(),
                test_placeholder: TemplateChild::default(),
                regex_buffer: TemplateChild::default(),
                test_buffer: TemplateChild::default(),
                matches_label: TemplateChild::default(),
            }
        }
    }

    #[glib::object_subclass]
    impl ObjectSubclass for Window {
        const NAME: &'static str = "Window";
        type Type = super::Window;
        type ParentType = adw::ApplicationWindow;

        fn class_init(klass: &mut Self::Class) {
            klass.bind_template();
            klass.bind_template_callbacks();

            klass.install_action("win.about", None, move |obj, _, _| {
                obj.show_about_dialog();
            });
        }

        fn instance_init(obj: &glib::subclass::InitializingObject<Self>) {
            obj.init_template();
        }
    }

    impl ObjectImpl for Window {
        fn constructed(&self) {
            self.parent_constructed();
            let obj = self.obj();

            // Devel profile
            if PROFILE == "Devel" {
                obj.add_css_class("devel");
            }

            obj.setup_text_views();
            obj.load_regex_state();
            obj.load_window_size();
        }
    }

    #[gtk::template_callbacks]
    impl Window {
        #[template_callback]
        fn on_buffer_changed(&self, _text_buffer: &gtk::TextBuffer) {
            let regex_string = self.regex_buffer.text(
                &self.regex_buffer.start_iter(),
                &self.regex_buffer.end_iter(),
                false,
            );
            let test_string = self.test_buffer.text(
                &self.test_buffer.start_iter(),
                &self.test_buffer.end_iter(),
                false,
            );

            self.regex_placeholder.set_visible(regex_string.is_empty());
            self.test_placeholder.set_visible(test_string.is_empty());

            let mut regex_parts = regex_string.split('/');
            let regex = regex_parts.next().unwrap_or_default();
            let flags = regex_parts.next().unwrap_or_default();

            let re: Regex = RegexBuilder::new(regex)
                .multi_line(flags.contains('m'))
                .case_insensitive(flags.contains('i'))
                .ignore_whitespace(flags.contains('x'))
                .dot_matches_new_line(flags.contains('s'))
                .unicode(flags.contains('u'))
                .swap_greed(flags.contains('U'))
                .build()
                .unwrap_or(Regex::new(r"").unwrap());

            self.test_buffer
                .remove_all_tags(&self.test_buffer.start_iter(), &self.test_buffer.end_iter());

            let mut captures = 0;

            for (index, caps) in re.captures_iter(&test_string).enumerate() {
                let m = caps.get(0).unwrap();

                let mut start_iter = self.test_buffer.start_iter();
                start_iter.set_offset(m.start() as i32);

                let mut end_iter = self.test_buffer.start_iter();
                end_iter.set_offset(m.end() as i32);

                if index % 2 == 0 {
                    self.test_buffer
                        .apply_tag_by_name("marked_first", &start_iter, &end_iter);
                } else {
                    self.test_buffer
                        .apply_tag_by_name("marked_second", &start_iter, &end_iter);
                }

                captures += 1;
            }

            if regex.is_empty() || captures == 0 {
                self.matches_label.set_label(&gettext("no matches"));
                return;
            }

            self.matches_label.set_label(&ngettext_f(
                "{matches} match",
                "{matches} matches",
                captures,
                &[("matches", &format!("{}", captures))],
            ));
        }
    }

    impl WidgetImpl for Window {}
    impl WindowImpl for Window {
        fn close_request(&self) -> gtk::Inhibit {
            let window = self.obj();

            if let Err(err) = window.save_regex_state() {
                println!("Failed to save regex state, {}", &err);
            }

            if let Err(err) = window.save_window_size() {
                println!("Failed to save window state, {}", &err);
            }

            self.parent_close_request()
        }
    }

    impl ApplicationWindowImpl for Window {}
    impl AdwApplicationWindowImpl for Window {}
}

glib::wrapper! {
    pub struct Window(ObjectSubclass<imp::Window>)
        @extends gtk::Widget, gtk::Window, gtk::ApplicationWindow, adw::ApplicationWindow,
        @implements gio::ActionGroup, gio::ActionMap, gtk::Root;
}

impl Window {
    pub fn new(application: &Application) -> Self {
        glib::Object::builder()
            .property("application", application)
            .build()
    }

    fn setup_text_views(&self) {
        let imp = self.imp();

        imp.regex_text_view.grab_focus();

        imp.test_buffer.create_tag(
            Some("marked_first"),
            &[("background", &"#99c1f1"), ("foreground", &"#000000")],
        );
        imp.test_buffer.create_tag(
            Some("marked_second"),
            &[("background", &"#62a0ea"), ("foreground", &"#000000")],
        );
        imp.test_buffer
            .create_tag(Some("marked_highlight"), &[("background", &"#f9f06b")]);
    }

    fn save_regex_state(&self) -> Result<(), glib::BoolError> {
        let imp = self.imp();

        let regex_string = imp.regex_buffer.text(
            &imp.regex_buffer.start_iter(),
            &imp.regex_buffer.end_iter(),
            false,
        );
        let test_string = imp.test_buffer.text(
            &imp.test_buffer.start_iter(),
            &imp.test_buffer.end_iter(),
            false,
        );

        imp.settings.set_string("last-regex", &regex_string)?;
        imp.settings.set_string("last-test", &test_string)?;

        Ok(())
    }

    fn load_regex_state(&self) {
        let imp = self.imp();

        let regex_string = imp.settings.string("last-regex");
        let test_string = imp.settings.string("last-test");

        imp.regex_buffer.set_text(&regex_string);
        imp.test_buffer.set_text(&test_string);
    }

    fn save_window_size(&self) -> Result<(), glib::BoolError> {
        let imp = self.imp();

        let (width, height) = self.default_size();

        imp.settings.set_int("window-width", width)?;
        imp.settings.set_int("window-height", height)?;

        Ok(())
    }

    fn load_window_size(&self) {
        let imp = self.imp();

        let width = imp.settings.int("window-width");
        let height = imp.settings.int("window-height");

        self.set_default_size(width, height);
    }

    fn show_about_dialog(&self) {
        let dialog = adw::AboutWindow::builder()
            .application_icon(APP_ID)
            .application_name(gettext("Wildcard"))
            .license_type(gtk::License::Gpl30)
            .comments(gettext("Test your regular expressions"))
            .website("https://github.com/fkinoshita/Wildcard")
            .issue_url("https://github.com/fkinoshita/Wildcard/issues/new")
            .version(VERSION)
            .transient_for(self)
            .translator_credits(gettext("translator-credits"))
            .developer_name("Felipe Kinoshita")
            .developers(vec!["Felipe Kinoshita <fkinoshita@gnome.org>"])
            .copyright("© 2023 Felipe Kinoshita.")
            .build();

        dialog.present();
    }
}
