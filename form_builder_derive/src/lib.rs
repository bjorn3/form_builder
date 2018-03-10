#![recursion_limit = "256"]

extern crate proc_macro;
extern crate syn;
#[macro_use]
extern crate quote;

use proc_macro::TokenStream;
use syn::{DeriveInput, Data, Fields};

#[proc_macro_derive(Form)]
pub fn derive_form(input: TokenStream) -> TokenStream {
    let input: DeriveInput = syn::parse(input).unwrap();
    let name = input.ident;
    let data_struct = match input.data {
        Data::Struct(data_struct) => data_struct,
        Data::Enum(_) => panic!("Can't derive Form for enum"),
        Data::Union(_) => panic!("Can't derive Form for union"),
    };
    let fields_named = match data_struct.fields {
        Fields::Named(fields_named) => fields_named,
        Fields::Unnamed(_) => panic!("Can't derive Form for tuple struct"),
        Fields::Unit => panic!("Can't derive Form for unit struct"),
    };
    let mut render_fields_html = Vec::new();
    let mut render_fields_gtk = Vec::new();
    let mut from_gtk_dialog = Vec::new();
    for (i, field) in fields_named.named.iter().enumerate() {
        let field_name = field.ident.unwrap();
        let mut field_label = field_name.as_ref().to_string();
        field_label[0..1].make_ascii_uppercase();
        field_label.push_str(": ");
        render_fields_html.push(quote! {
            self.#field_name.render_field_html(&mut buf, stringify!(#field_name), #field_label);
        });
        render_fields_gtk.push(quote! {
            self.#field_name.render_field_gtk(submit_button.clone(), #field_label)
        });
        from_gtk_dialog.push(quote! {
            #field_name: FormField::from_gtk_widget(fields[#i].clone())
        });
    }
    let expanded = quote! {
        mod _form_builder_impl {
            extern crate gtk;
            extern crate form_builder;
            use self::gtk::prelude::*;
            use self::gtk::{Object, Window, Dialog, DialogFlags, Button, Label};
            use self::form_builder::{Form, FormField};

            impl Form for #name {
                fn render_html(&self, action: &str) -> String {
                    use std::io::Write;
                    let mut buf = ::std::io::Cursor::new(Vec::new());
                    writeln!(buf, "<form action=\"{}\">", action).unwrap();

                    #(#render_fields_html);*

                    writeln!(buf, "<button type=\"submit\">Submit</button>\n</form>").unwrap();

                    String::from_utf8(buf.into_inner()).unwrap()
                }

                fn render_gtk(&self) -> (Dialog, Vec<Object>) {
                    let dialog = Dialog::new_with_buttons::<Window>(
                        Some("form"),
                        None,
                        DialogFlags::empty(),
                        &[("Submit", 0)]
                    );
                    let submit_button: Button = dialog.get_widget_for_response(0).unwrap().downcast().unwrap();
                    submit_button.set_size_request(200, 0);

                    let mut fields = Vec::new();
                    let content = dialog.get_content_area();
                    #(
                        let field = #render_fields_gtk;
                        fields.push(field.1);
                        content.add(&field.0);
                    )*

                    dialog.show_all();
                    (dialog, fields)
                }

                fn from_gtk_dialog(fields: Vec<Object>) -> Self {
                    #name {
                        #(#from_gtk_dialog),*
                    }
                }
            }
        }
    };

    //println!("{:#?}", expanded);
    expanded.into()
}
