# Form builder

Build forms for HTML or GTK3 using a derive proc-macro.

## Gtk example

```rust
extern crate gtk;
extern crate form_builder;
#[macro_use]
extern crate form_builder_derive;

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

    let form = form.show_gtk();
    println!("{:#?}", form);
}
```
