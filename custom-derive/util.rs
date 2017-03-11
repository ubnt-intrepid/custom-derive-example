use quote;

pub fn join_tokens<I, T>(iter: I) -> quote::Tokens
  where I: IntoIterator<Item = T>,
        T: quote::ToTokens
{
  let mut tokens = quote::Tokens::new();
  tokens.append_all(iter);
  tokens
}
