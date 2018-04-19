#![recursion_limit = "256"]

extern crate proc_macro;
extern crate syn;
#[macro_use]
extern crate quote;

use proc_macro::TokenStream;
use syn::{DeriveInput, Data, Fields, Ident};

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
    let mut from_gtk_dialog = Vec::new();
    let mut build_form_fields = Vec::new();
    for (i, field) in fields_named.named.iter().enumerate() {
        let field_name = field.ident.unwrap();
        let mut field_label = field_name.as_ref().to_string();
        field_label[0..1].make_ascii_uppercase();
        field_label.push_str(": ");
        render_fields_html.push(quote! {
            let field_name = &format!("{}__{}", name, stringify!(#field_name));
            writeln!(buf, "<label for=\"{n}\">{l}</label>", n=&field_name, l=#field_label).unwrap();
            self.#field_name.render_html_inner(buf, &field_name);
        });
        from_gtk_dialog.push(quote! {
            #field_name: Form::from_gtk_widget(fields[#i].take().unwrap())
        });
        build_form_fields.push(quote! {
            form_builder.add_field(&self.#field_name, #field_label);
        });
    }
    let some_random_wrapper_mod = Ident::from(format!("some_random_wrapper_mod_for_{}", name));
    let expanded = quote! {
        #[allow(non_snake_case)]
        mod #some_random_wrapper_mod {
            extern crate gtk;
            extern crate form_builder;
            use std::any::Any;
            use std::io::{Write, Cursor};
            use self::gtk::prelude::*;
            use self::gtk::{Object, Window, Dialog, DialogFlags, Widget, Button, Label};
            use self::form_builder::*;

            impl Form for #name {
                fn render_html_inner(&self, buf: &mut Cursor<Vec<u8>>, name: &str) {
                    if !name.is_empty() {
                        writeln!(buf, "<div class=\"{}\">", name).unwrap();
                    }
                    #(#render_fields_html);*
                    if !name.is_empty() {
                        writeln!(buf, "</div>").unwrap();
                    }
                }

                fn render_gtk_inner(&self, submit_button: Button, _validity_label: Option<Label>) -> (Widget, Box<Any>) {
                    let mut form_builder = GtkFormBuilder::new(submit_button);
                    #(#build_form_fields)*
                    let (box_, fields) = form_builder.build();
                    (box_.upcast(), Box::new(fields))
                }

                fn from_gtk_widget(fields: Box<Any>) -> Self {
                    let fields = fields.downcast::<Vec<Box<Any>>>().unwrap();
                    let mut fields = fields.into_iter().map(Option::Some).collect::<Vec<_>>();
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
