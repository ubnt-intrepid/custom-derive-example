#[macro_use]
extern crate custom_derive;
extern crate clap;

trait Subcommand {
  fn app<'a, 'b: 'a>() -> clap::App<'a, 'b>;
}

trait App {
  fn append<'a, 'b: 'a>(app: clap::App<'a,'b>) -> clap::App<'a,'b>;
}

#[derive(Debug, Subcommand)]
#[clap(name = "myapp", about = "My sample application")]
#[clap(VersionlessSubcommands, SubcommandRequiredElseHelp)]
enum MyApp {
  #[clap(name = "foo", help = "Foo app")]
  Foo(Foo),
  #[clap(name = "hoge", help = "Bar app")]
  Bar(Bar),
}

#[derive(Debug, Default)]
struct Foo;

impl<'a,'b:'a> From<&'b clap::ArgMatches<'a>> for Foo {
  fn from(_: &'b clap::ArgMatches<'a>) -> Foo { Foo::default() }
}

impl App for Foo {
  fn append<'a,'b:'a>(app: clap::App<'a,'b>) -> clap::App<'a,'b> {
    app
  }
}


#[derive(Debug, Default)]
struct Bar;

impl<'a,'b:'a> From<&'b clap::ArgMatches<'a>> for Bar {
  fn from(_: &'b clap::ArgMatches<'a>) -> Bar { Bar::default() }
}

impl App for Bar {
  fn append<'a,'b:'a>(app: clap::App<'a,'b>) -> clap::App<'a,'b> {
    app
  }
}


fn main() {
  let ref matches = MyApp::app().get_matches();
  let app: MyApp = matches.into();

  println!("{:?}", app);
}
