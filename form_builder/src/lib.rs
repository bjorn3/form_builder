extern crate gtk;

use std::any::Any;
use std::io::{self, Write};
use gtk::prelude::*;
use gtk::{Object, Window, Dialog, DialogFlags, Widget, Label, Entry, Button, Orientation};
use gtk::Box as GtkBox;

pub struct GtkFormBuilder(Button, GtkBox, Vec<Box<Any>>);

impl GtkFormBuilder {
    pub fn new(submit_button: Button) -> Self {
        GtkFormBuilder(submit_button, GtkBox::new(Orientation::Vertical, 0), vec![])
    }

    pub fn add_field<T: Form>(&mut self, field: &T, label: &str) {
        let box_ = GtkBox::new(Orientation::Horizontal, 0);

        box_.pack_start(&Label::new(Some(label)), false, false, 0);

        let validity_label = Label::new(None);
        box_.pack_start(&validity_label, false, false, 0);

        let (widget, data) = field.render_gtk_inner(self.0.clone(), Some(validity_label));
        box_.pack_start(&widget, true, true, 0);

        self.1.pack_start(&box_, false, false, 0);
        self.2.push(data);
    }

    pub fn build(self) -> (GtkBox, Vec<Box<Any>>) {
        (self.1, self.2)
    }
}

pub trait Form {
    fn render_html(&self, action: &str) -> String {
        use std::io::Write;
        let mut buf = ::std::io::Cursor::new(Vec::new());
        writeln!(buf, "<form action=\"{}\">", action).unwrap();

        self.render_html_inner(&mut buf, "");

        writeln!(buf, "<button type=\"submit\">Submit</button>\n</form>").unwrap();

        String::from_utf8(buf.into_inner()).unwrap()
    }

    fn render_html_inner(&self, buf: &mut io::Cursor<Vec<u8>>, name: &str);

    fn show_gtk(&self) -> Self where Self: Sized{
        let (dialog, fields) = self.render_gtk();
        dialog.run();
        Self::from_gtk_widget(fields)
    }
    fn render_gtk(&self) -> (Dialog, Box<Any>) {
        let dialog = Dialog::new_with_buttons::<Window>(
            Some("form"),
            None,
            DialogFlags::empty(),
            &[("Submit", 0)]
        );
        let submit_button: Button = dialog.get_widget_for_response(0).unwrap().downcast().unwrap();
        submit_button.set_size_request(200, 0);

        let (box_, fields) = self.render_gtk_inner(submit_button, None);
        dialog.get_content_area().add(&box_);

        dialog.show_all();
        (dialog, fields)
    }
    fn render_gtk_inner(&self, submit_button: Button, validity_label: Option<Label>) -> (Widget, Box<Any>);
    fn from_gtk_widget(fields: Box<Any>) -> Self;
}

impl Form for String {
    fn render_html_inner(&self, buf: &mut io::Cursor<Vec<u8>>, name: &str) {
        writeln!(buf, "<input name=\"{n}\" type=\"text\" value={val:?}><br>", n=name, val=self).unwrap();
    }

    fn render_gtk_inner(&self, _submit_button: Button, _validity_label: Option<Label>) -> (Widget, Box<Any>) {
        let text_entry = Entry::new();
        text_entry.set_text(self);
        (text_entry.clone().upcast(), Box::new(text_entry))
    }

    fn from_gtk_widget(object: Box<Any>) -> Self {
        let text_entry: Entry = *object.downcast().unwrap();
        text_entry.get_text().unwrap_or_else(|| String::new())
    }
}

#[derive(Debug, Default)]
pub struct NonEmptyString(pub String);

impl Form for NonEmptyString {
    fn render_html_inner(&self, buf: &mut io::Cursor<Vec<u8>>, name: &str) {
        writeln!(buf, "<input name=\"{n}\" type=\"text\" required value={val:?}><br>", n=name, val=&self.0).unwrap();
    }

    fn render_gtk_inner(&self, submit_button: Button, validity_label: Option<Label>) -> (Widget, Box<Any>) {
        let text_entry = Entry::new();
        text_entry.set_text(&self.0);
        validate_entry(&text_entry, validity_label, submit_button, |entry| {
            !entry.get_text().as_ref().map(|t|&**t).unwrap_or("").is_empty()
        });
        (text_entry.clone().upcast(), Box::new(text_entry))
    }

    fn from_gtk_widget(object: Box<Any>) -> Self {
        let text_entry: Entry = *object.downcast().unwrap();
        NonEmptyString(text_entry.get_text().unwrap_or_else(|| String::new()))
    }
}

#[derive(Debug, Default)]
pub struct Password(pub String);

impl Form for Password {
    fn render_html_inner(&self, buf: &mut io::Cursor<Vec<u8>>, name: &str) {
        writeln!(buf, "<input name=\"{n}\" type=\"password\" required><br>", n=name).unwrap();
    }

    fn render_gtk_inner(&self, submit_button: Button, validity_label: Option<Label>) -> (Widget, Box<Any>) {
        let pass_entry = Entry::new();
        pass_entry.set_visibility(false);
        validate_entry(&pass_entry, validity_label, submit_button, |entry| {
            !entry.get_text().as_ref().map(|t|&**t).unwrap_or("").is_empty()
        });
        (pass_entry.clone().upcast(), Box::new(pass_entry))
    }

    fn from_gtk_widget(object: Box<Any>) -> Self {
        let pass_entry: Entry = *object.downcast().unwrap();
        Password(pass_entry.get_text().unwrap_or_else(|| String::new()))
    }
}

fn validate_entry<F: Fn(&Entry) -> bool + 'static>(entry: &Entry, validity_label: Option<Label>, submit_button: Button, f: F) {
    if !f(entry) {
        if let Some(ref validity_label) = validity_label {
            validity_label.set_markup("<span color=\"red\">Must not be empty</span>");
        }
        submit_button.set_sensitive(false);
    }

    entry.connect_property_text_notify(move |entry| {
        if f(entry) {
            submit_button.set_sensitive(true);
            if let Some(ref validity_label) = validity_label {
                validity_label.set_text("");
            }
        } else {
            submit_button.set_sensitive(false);
            if let Some(ref validity_label) = validity_label {
                validity_label.set_markup("<span color=\"red\">Must not be empty</span>");
            }
        }
    });
}
