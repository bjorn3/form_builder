#![recursion_limit = "256"]

extern crate proc_macro;
extern crate syn;
#[macro_use]
extern crate quote;

use proc_macro::TokenStream;
use syn::{DeriveInput, Data, DataEnum, Fields, Ident};

#[proc_macro_derive(Form)]
pub fn derive_form(input: TokenStream) -> TokenStream {
    let input: DeriveInput = syn::parse(input).unwrap();
    let name = input.ident;
    let data_struct = match input.data {
        Data::Struct(data_struct) => data_struct,
        Data::Enum(data_enum) => return derive_form_enum(name, data_enum),
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

fn derive_form_enum(name: Ident, data: DataEnum) -> TokenStream {
    let mut render_fields_html = Vec::new();
    let mut build_combobox_variants = Vec::new();
    let mut variant_to_variant_name = Vec::new();
    let mut variant_name_to_variant = Vec::new();
    for variant in data.variants.iter() {
        let variant_name = variant.ident;
        let mut variant_label = variant_name.as_ref().to_string();
        match variant.fields {
            syn::Fields::Unit => {}
            _ => panic!("Can't derive Form for enums with fields")
        }
        variant_label[0..1].make_ascii_uppercase();
        render_fields_html.push(quote! {
            let variant_name = &format!("{}__{}", name, stringify!(#variant_name));
            let is_default = match *self {
                #name::#variant_name => " selected=\"selected\"",
                _ => "",
            };
            writeln!(buf, "<option value=\"{n}\"{default}>{l}</option>", n=&variant_name, l=#variant_label, default=is_default).unwrap();
            //self.#field_name.render_html_inner(buf, &field_name);
        });
        build_combobox_variants.push(quote! {
            combo_box.append(Some(stringify!(#variant_name)), #variant_label);
        });
        variant_to_variant_name.push(quote! {
            #name::#variant_name => stringify!(#variant_name),
        });
        variant_name_to_variant.push(quote! {
            stringify!(#variant_name) => #name::#variant_name,
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
            use self::gtk::{Object, Window, Dialog, DialogFlags, Widget, Button, Label, ComboBoxText};
            use self::form_builder::*;

            impl Form for #name {
                fn render_html_inner(&self, buf: &mut Cursor<Vec<u8>>, name: &str) {
                    writeln!(buf, "<select name=\"{}\">", name).unwrap();
                    #(#render_fields_html);*
                    writeln!(buf, "</select><br>").unwrap();
                }

                fn render_gtk_inner(&self, _submit_button: Button, _validity_label: Option<Label>) -> (Widget, Box<Any>) {
                    let combo_box = ComboBoxText::new();
                    #(#build_combobox_variants)*
                    let variant_name = match *self {
                        #(#variant_to_variant_name)*
                    };
                    println!("{}", variant_name);
                    combo_box.set_active_id(Some(variant_name));
                    (combo_box.clone().upcast(), Box::new(combo_box))
                }

                fn from_gtk_widget(fields: Box<Any>) -> Self {
                    let combo_box = fields.downcast::<ComboBoxText>().unwrap();
                    match &*combo_box.get_active_id().unwrap() {
                        #(#variant_name_to_variant)*
                        invalid_id => unreachable!("Somehow we ended up with an invalid gtk ComboBoxText identifier :( ({})", invalid_id),
                    }
                }
            }
        }
    };

    //println!("{:#?}", expanded);
    expanded.into()
}
