extern crate proc_macro;

use proc_macro::TokenStream;
use quote::quote;

#[proc_macro_derive(PipedCommand)]
pub fn piped_command_derive(input: TokenStream) -> TokenStream {
    let ast = syn::parse(input).unwrap();

    impl_piped_command(&ast)
}

fn impl_piped_command(ast: &syn::DeriveInput) -> TokenStream {
    let name = &ast.ident;
    let gen = quote! {
        use futures::{SinkExt, StreamExt};

        #[async_trait]
        impl PipedCommand for #name {
            async fn stream(
                &self,
                mut input: futures::channel::mpsc::UnboundedReceiver<Message>,
                mut output: futures::channel::mpsc::UnboundedSender<Vec<Message>>)
            -> Result<(), InvocationError> {
                    while let Some(msg) = input.next().await {
                        let results = self.call(msg).await?;
                        output.send(results).await.expect("failed to send results to output");
                    }
                    Ok(())
            }

            fn info(&self) -> String {
                Command::info(self)
            }
        }
    };
    gen.into()
}
