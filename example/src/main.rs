extern crate gtk;
#[macro_use]
extern crate pretty_assertions;
extern crate form_builder;
#[macro_use]
extern crate form_builder_derive;

use gtk::prelude::*;
use form_builder::{Form, NonEmptyString, Password};

#[derive(Debug, Default, Form)]
struct LoginForm {
    username: NonEmptyString,
    password: Password,
}

fn main() {
    gtk::init().unwrap();
    let form = LoginForm {
        username: NonEmptyString("My u\"sername".to_string()),
        password: Password("the_passw0rd".to_string()),
    };
    let html = form.render_html("/login");
    assert_eq!(&*html, "\
<form action=\"/login\">
<label for=\"username\">Username: </label><input name=\"username\" type=\"text\" required value=\"My u\\\"sername\"><br>
<label for=\"password\">Password: </label><input name=\"password\" type=\"password\" required><br>
<button type=\"submit\">Submit</button>
</form>
");
    //let form = LoginForm::from_request(req);

    let form = form.show_gtk();
    println!("{:#?}", form);
}
