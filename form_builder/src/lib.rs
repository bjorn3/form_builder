extern crate gtk;

use std::io::{self, Write};
use gtk::prelude::*;
use gtk::{Object, Dialog, Widget, Label, Entry, Button, Orientation, Inhibit};
use gtk::Box as GtkBox;

pub trait Form {
    fn render_html(&self, action: &str) -> String;

    fn show_gtk(&self) -> Self where Self: Sized{
        let (dialog, fields) = self.render_gtk();
        dialog.run();
        Self::from_gtk_dialog(fields)
    }
    fn render_gtk(&self) -> (Dialog, Vec<Object>);
    fn from_gtk_dialog(fields: Vec<Object>) -> Self;
}

pub trait FormField {
    fn render_field_html(&self, buf: &mut io::Cursor<Vec<u8>>, name: &str, label: &str);

    fn render_field_gtk(&self, submit_button: Button, label: &str) -> (Widget, Object);
    fn from_gtk_widget(object: Object) -> Self;
}

impl FormField for String {
    fn render_field_html(&self, buf: &mut io::Cursor<Vec<u8>>, name: &str, label: &str) {
        writeln!(buf, "<label for=\"{n}\">{l}</label><input name=\"{n}\" type=\"text\" value={val:?}><br>", n=name, l=label, val=self).unwrap();
    }

    fn render_field_gtk(&self, _submit_button: Button, label: &str) -> (Widget, Object) {
        let box_ = GtkBox::new(Orientation::Horizontal, 0);
        box_.pack_start(&Label::new(Some(label)), true, false, 0);
        let text_entry = Entry::new();
        text_entry.set_text(self);
        box_.pack_start(&text_entry, true, true, 0);
        (box_.upcast(), text_entry.upcast())
    }

    fn from_gtk_widget(object: Object) -> Self {
        let text_entry: Entry = object.downcast().unwrap();
        text_entry.get_text().unwrap_or_else(|| String::new())
    }
}

#[derive(Debug, Default)]
pub struct NonEmptyString(pub String);

impl FormField for NonEmptyString {
    fn render_field_html(&self, buf: &mut io::Cursor<Vec<u8>>, name: &str, label: &str) {
        writeln!(buf, "<label for=\"{n}\">{l}</label><input name=\"{n}\" type=\"text\" required value={val:?}><br>", n=name, l=label, val=&self.0).unwrap();
    }

    fn render_field_gtk(&self, submit_button: Button, label: &str) -> (Widget, Object) {
        let (box_, validity_label) = create_basic_layout(label);
        let text_entry = Entry::new();
        text_entry.set_text(&self.0);
        text_entry.set_vexpand(true);
        box_.pack_start(&text_entry, true, true, 0);

        validate_entry(&text_entry, validity_label, submit_button, |entry| {
            !entry.get_text().as_ref().map(|t|&**t).unwrap_or("").is_empty()
        });

        (box_.upcast(), text_entry.upcast())
    }

    fn from_gtk_widget(object: Object) -> Self {
        let text_entry: Entry = object.downcast().unwrap();
        NonEmptyString(text_entry.get_text().unwrap_or_else(|| String::new()))
    }
}

#[derive(Debug, Default)]
pub struct Password(pub String);

impl FormField for Password {
    fn render_field_html(&self, buf: &mut io::Cursor<Vec<u8>>, name: &str, label: &str) {
        writeln!(buf, "<label for=\"{n}\">{l}</label><input name=\"{n}\" type=\"password\" required><br>", n=name, l=label).unwrap();
    }

    fn render_field_gtk(&self, submit_button: Button, label: &str) -> (Widget, Object) {
        let (box_, validity_label) = create_basic_layout(label);
        let pass_entry = Entry::new();
        pass_entry.set_visibility(false);
        box_.pack_start(&pass_entry, true, true, 0);

        validate_entry(&pass_entry, validity_label, submit_button, |entry| {
            !entry.get_text().as_ref().map(|t|&**t).unwrap_or("").is_empty()
        });

        (box_.upcast(), pass_entry.upcast())
    }

    fn from_gtk_widget(object: Object) -> Self {
        let pass_entry: Entry = object.downcast().unwrap();
        Password(pass_entry.get_text().unwrap_or_else(|| String::new()))
    }
}

fn create_basic_layout(label: &str) -> (GtkBox, Label) {
    let box_ = GtkBox::new(Orientation::Horizontal, 0);
    box_.pack_start(&Label::new(Some(label)), false, false, 0);
    let validity_label = Label::new(None);
    box_.pack_start(&validity_label, false, false, 0);
    (box_, validity_label)
}

fn validate_entry<F: Fn(&Entry) -> bool + 'static>(entry: &Entry, validity_label: Label, submit_button: Button, f: F) {
    if !f(entry) {
        validity_label.set_markup("<span color=\"red\">Must not be empty</span>");
        submit_button.set_sensitive(false);
    }

    entry.connect_property_text_notify(move |entry| {
        if f(entry) {
            submit_button.set_sensitive(true);
            validity_label.set_text("");
        } else {
            submit_button.set_sensitive(false);
            validity_label.set_markup("<span color=\"red\">Must not be empty</span>");
        }
    });
}
