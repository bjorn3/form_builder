#[macro_use]
extern crate pretty_assertions;
extern crate form_builder;
#[macro_use]
extern crate form_builder_derive;

use form_builder::{Form, Password};

#[derive(Default, Form)]
struct LoginForm {
    username: String,
    password: Password,
}

fn main() {
    let html = LoginForm {
        username: "My u\"sername".to_string(),
        password: Password("the_passw0rd".to_string()),
    }.render_html("/login");
    assert_eq!(&*html, "\
<form action=\"/login\">
<label for=\"username\">Username: </label><input name=\"username\" type=\"text\" required value=\"My u\\\"sername\"><br>
<label for=\"password\">Password: </label><input name=\"password\" type=\"password\" required><br>
<button type=\"submit\">Submit</button>
</form>
");
    //let form = LoginForm::from_request(req);
}
