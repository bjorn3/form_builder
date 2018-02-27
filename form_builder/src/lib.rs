use std::io::{self, Write};

pub trait Form {
    fn render_html(&self, action: &str) -> String;
}

#[derive(Default)]
pub struct Password(pub String);

pub trait FormField {
    fn render_field_html(&self, buf: &mut io::Cursor<Vec<u8>>, name: &str, label: &str);
}

impl FormField for String {
    fn render_field_html(&self, buf: &mut io::Cursor<Vec<u8>>, name: &str, label: &str) {
        writeln!(buf, "<label for=\"{n}\">{l}</label><input name=\"{n}\" type=\"text\" required value={val:?}><br>", n=name, l=label, val=self).unwrap();
    }
}

impl FormField for Password {
    fn render_field_html(&self, buf: &mut io::Cursor<Vec<u8>>, name: &str, label: &str) {
        writeln!(buf, "<label for=\"{n}\">{l}</label><input name=\"{n}\" type=\"password\" required><br>", n=name, l=label).unwrap();
    }
}
