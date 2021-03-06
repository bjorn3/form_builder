extern crate gtk;
#[macro_use]
extern crate pretty_assertions;
extern crate form_builder;
#[macro_use]
extern crate form_builder_derive;

use form_builder::{Form, NonEmptyString, Password};

#[derive(Debug, Default, Form)]
struct LoginForm {
    username: NonEmptyString,
    password: Password,
    remember_username: bool,
}

#[derive(Debug, Default, Form)]
struct FormWrapper {
    login_form_1: LoginForm,
    login_form_2: LoginForm,
}

#[derive(Debug, Form)]
enum Color {
    Red,
    Green,
    Blue,
}

fn main() {
    gtk::init().unwrap();
    let login_form = LoginForm {
        username: NonEmptyString("My u\"sername".to_string()),
        password: Password("the_passw0rd".to_string()),
        remember_username: true,
    };
    let login_form_html = login_form.render_html("/login");
    assert_eq!(&*login_form_html, "\
<form action=\"/login\">
<label for=\"__username\">Username: </label>
<input name=\"__username\" type=\"text\" required value=\"My u\\\"sername\"><br>
<label for=\"__password\">Password: </label>
<input name=\"__password\" type=\"password\" required><br>
<label for=\"__remember_username\">Remember_username: </label>
<input name=\"__remember_username\" type=\"text\" checked><br>
<button type=\"submit\">Submit</button>
</form>
");

    let form_wrapper = FormWrapper::default();
    let form_wrapper_html = form_wrapper.render_html("/login");
    assert_eq!(&*form_wrapper_html, "\
<form action=\"/login\">
<label for=\"__login_form_1\">Login_form_1: </label>
<div class=\"__login_form_1\">
<label for=\"__login_form_1__username\">Username: </label>
<input name=\"__login_form_1__username\" type=\"text\" required value=\"\"><br>
<label for=\"__login_form_1__password\">Password: </label>
<input name=\"__login_form_1__password\" type=\"password\" required><br>
<label for=\"__login_form_1__remember_username\">Remember_username: </label>
<input name=\"__login_form_1__remember_username\" type=\"text\"><br>
</div>
<label for=\"__login_form_2\">Login_form_2: </label>
<div class=\"__login_form_2\">
<label for=\"__login_form_2__username\">Username: </label>
<input name=\"__login_form_2__username\" type=\"text\" required value=\"\"><br>
<label for=\"__login_form_2__password\">Password: </label>
<input name=\"__login_form_2__password\" type=\"password\" required><br>
<label for=\"__login_form_2__remember_username\">Remember_username: </label>
<input name=\"__login_form_2__remember_username\" type=\"text\"><br>
</div>
<button type=\"submit\">Submit</button>
</form>
");

    let color_form = Color::Green;
    let color_form_html = color_form.render_html("/login");
    assert_eq!(&*color_form_html, "\
<form action=\"/login\">
<select name=\"\">
<option value=\"__Red\">Red</option>
<option value=\"__Green\" selected=\"selected\">Green</option>
<option value=\"__Blue\">Blue</option>
</select><br>
<button type=\"submit\">Submit</button>
</form>
");

    println!("{:#?}", form_wrapper.show_gtk());
}
