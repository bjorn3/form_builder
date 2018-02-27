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
    let mut render_fields = Vec::new();
    for field in fields_named.named.iter() {
        let field_name = field.ident.unwrap();
        let mut field_label = field_name.as_ref().to_string();
        field_label[0..1].make_ascii_uppercase();
        field_label.push_str(": ");
        render_fields.push(quote! {
            self.#field_name.render_field_html(&mut buf, stringify!(#field_name), #field_label);
        });
    }
    let expanded = quote! {
        impl Form for #name {
            fn render_html(&self, action: &str) -> String {
                use std::io::Write;
                use form_builder::FormField;
                let mut buf = ::std::io::Cursor::new(Vec::new());
                writeln!(buf, "<form action=\"{}\">", action).unwrap();

                #(#render_fields);*

                writeln!(buf, "<button type=\"submit\">Submit</button>\n</form>").unwrap();

                String::from_utf8(buf.into_inner()).unwrap()
            }
        }
    };
    expanded.into()
}
