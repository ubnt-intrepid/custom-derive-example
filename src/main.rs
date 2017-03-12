#[macro_use]
extern crate custom_derive;
extern crate clap;

trait App {
  fn app<'a, 'b: 'a>() -> clap::App<'a, 'b>;
  fn append<'a, 'b: 'a>(app: clap::App<'a, 'b>) -> clap::App<'a, 'b>;
}

#[derive(Debug, App)]
#[clap(name = "myapp", about = "My sample application")]
#[clap(VersionlessSubcommands, SubcommandRequiredElseHelp)]
enum MyApp {
  #[clap(name = "foo")]
  Foo(Foo),
  #[clap(name = "hoge")]
  Bar(Bar),
}

#[derive(Debug, Default, App)]
#[clap(name = "foo", about = "Foo app")]
struct Foo;

#[derive(Debug, Default, App)]
#[clap(name = "bar", about = "Bar app")]
struct Bar;

fn main() {
  let ref matches = MyApp::app().get_matches();
  let app: MyApp = matches.into();

  println!("{:?}", app);
}
