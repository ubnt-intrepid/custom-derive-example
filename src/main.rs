#[macro_use]
extern crate custom_derive;
extern crate clap;

trait Subcommand {
  fn app<'a, 'b: 'a>(name: &str) -> clap::App<'a, 'b>;
}

#[derive(Debug, Subcommand)]
enum MyApp {
  #[clap(name = "foo", help = "Foo app")]
  Foo(Foo),
  #[clap(name = "hoge", help = "Bar app")]
  Bar(Bar),
}

#[derive(Debug, Default)]
struct Foo;

#[derive(Debug, Default)]
struct Bar;


fn main() {
  let ref matches = MyApp::app("myapp").get_matches();
  let app: MyApp = matches.into();

  println!("{:?}", app);
}
