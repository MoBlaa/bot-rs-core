extern crate proc_macro;

use proc_macro::TokenStream;
use quote::quote;

#[proc_macro_derive(StreamablePlugin)]
pub fn piped_command_derive(input: TokenStream) -> TokenStream {
    let ast = syn::parse(input).unwrap();

    impl_piped_command(&ast)
}

fn impl_piped_command(ast: &syn::DeriveInput) -> TokenStream {
    let name = &ast.ident;
    let gen = quote! {
        #[async_trait]
        impl StreamablePlugin for #name {
            async fn stream(
                &self,
                mut input: futures::channel::mpsc::UnboundedReceiver<Message>,
                mut output: futures::channel::mpsc::UnboundedSender<Vec<Message>>)
            -> Result<(), InvocationError> {
                    while let Some(msg) = futures::StreamExt::next(&mut input).await {
                        let results = self.call(msg).await?;
                        futures::SinkExt::send(&mut output, results).await.expect("failed to send results to output");
                    }
                    Ok(())
            }

            fn info(&self) -> PluginInfo {
                Plugin::info(self)
            }
        }
    };
    gen.into()
}
