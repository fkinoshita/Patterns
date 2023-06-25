// SPDX-License-Identifier: GPL-3.0-or-later

use regex::{Regex, RegexBuilder};

use adw::subclass::prelude::*;
use gtk::prelude::*;
use gtk::{gio, glib};

use gettextrs::gettext;

use crate::i18n::ngettext_f;

use crate::application::Application;
use crate::config::{APP_ID, PROFILE, VERSION};
use crate::flags_dialog::FlagsDialog;

mod imp {
    use super::*;

    #[derive(Debug, gtk::CompositeTemplate)]
    #[template(resource = "/io/github/fkinoshita/Wildcard/ui/window.ui")]
    pub struct Window {
        pub settings: gio::Settings,

        #[template_child]
        pub test_buffer: TemplateChild<gtk::TextBuffer>,
        #[template_child]
        pub regex_row: TemplateChild<adw::EntryRow>,

        #[template_child]
        pub matches_label: TemplateChild<gtk::Label>,
    }

    impl Default for Window {
        fn default() -> Self {
            Self {
                settings: gio::Settings::new(APP_ID),

                test_buffer: TemplateChild::default(),
                regex_row: TemplateChild::default(),

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
        fn on_buffer_changed(&self) {
            let regex_string = self.regex_row.text();
            let test_string = self.test_buffer.text(
                &self.test_buffer.start_iter(),
                &self.test_buffer.end_iter(),
                false,
            );

            let re: Regex = RegexBuilder::new(&regex_string)
                .multi_line(self.settings.boolean("multiline-flag"))
                .case_insensitive(self.settings.boolean("case-insensitive-flag"))
                .ignore_whitespace(self.settings.boolean("ignore-whitespace-flag"))
                .dot_matches_new_line(self.settings.boolean("dot-matches-newline-flag"))
                .unicode(self.settings.boolean("unicode-flag"))
                .swap_greed(self.settings.boolean("greed-flag"))
                .build()
                .unwrap_or(Regex::new(r"").unwrap());

            self.test_buffer
                .remove_all_tags(&self.test_buffer.start_iter(), &self.test_buffer.end_iter());

            let mut captures = 0;

            for (index, caps) in re.captures_iter(&test_string).enumerate() {
                let m = caps.get(0).unwrap();

                if m.is_empty() {
                    continue;
                }

                let mut start_iter = self.test_buffer.start_iter();
                start_iter.set_offset(m.start() as i32);

                let mut end_iter = self.test_buffer.start_iter();
                end_iter.set_offset(m.end() as i32);

                let marker = if index % 2 == 0 {
                    "marked_first"
                } else {
                    "marked_second"
                };
                self.test_buffer
                    .apply_tag_by_name(marker, &start_iter, &end_iter);

                captures += 1;
            }

            if regex_string.is_empty() || captures == 0 {
                self.matches_label.set_label(&gettext("no matches"));
                return;
            }

            self.matches_label.set_label(&ngettext_f(
                "{matches} match",
                "{matches} matches",
                captures,
                &[("matches", &captures.to_string())],
            ));
        }

        #[template_callback]
        fn on_flags_row_activated(&self) {
            self.obj().show_flags_dialog();
        }
    }

    impl WidgetImpl for Window {}
    impl WindowImpl for Window {
        fn close_request(&self) -> gtk::Inhibit {
            let window = self.obj();

            if let Err(err) = window.save_regex_state() {
                log::error!("Failed to save regex state, {}", &err);
            }

            if let Err(err) = window.save_window_size() {
                log::error!("Failed to save window state, {}", &err);
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

        imp.regex_row.grab_focus();

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

        let regex_string = imp.regex_row.text();
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

        imp.regex_row.set_text(&regex_string);
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

    fn show_flags_dialog(&self) {
        let dialog = FlagsDialog::new();

        dialog.connect_local(
            "flags-changed",
            false,
            glib::clone!(@strong self as this => move |_| {
                    this.imp().test_buffer.emit_by_name::<()>("changed", &[]);

                    None
                }
            ),
        );

        dialog.set_transient_for(Some(self));
        dialog.set_modal(true);

        dialog.present();
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
            .artists(vec!["kramo https://kramo.hu"])
            .copyright("© 2023 Felipe Kinoshita.")
            .release_notes(gettext("
                <p>This is minor release of Wildcard brings much needed fixes and improvements:</p>
                <ul>
                  <li>Fixed scrolling issues on the text views</li>
                  <li>Add label to text views to make their purpose clearer</li>
                  <li>French translation</li>
                </ul>
                <p>Wildcard is made possible by volunteer developers, designers, and translators. Thank you for your contributions!</p>
                <p>Feel free to report issues and ask for new features.</p>"
            ))
            .build();

        dialog.present();
    }
}
