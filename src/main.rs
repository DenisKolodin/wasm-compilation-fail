#![recursion_limit="512"]

#[macro_use]
extern crate stdweb;
#[macro_use]
extern crate yew;
extern crate serde;
#[macro_use]
extern crate serde_derive;
extern crate serde_json;

mod wsproto;

use stdweb::Value;
use yew::prelude::*;

struct Context {
}

#[derive(PartialEq)]
enum Model {
    One,
    Two,
    Three,
}

enum Msg {
    Replace(Model),
}

impl Component<Context> for Model {
    type Msg = Msg;
    type Properties = ();

    fn create(_: Self::Properties, context: &mut Env<Context, Self>) -> Self {
        Model::One
    }

    fn update(&mut self, msg: Self::Msg, context: &mut Env<Context, Self>) -> ShouldRender {
        match msg {
            Msg::Replace(model) => {
                *self = model;
            }
        }
        true
    }
}

impl Renderable<Context, Model> for Model {
    fn view(&self) -> Html<Context, Self> {
        let desk = if *self == Model::Two { "selected" } else { "not-selected" };
        let settings = if *self == Model::Three { "selected" } else { "not-selected" };
        html! {
            <div class="main",>
                <header class="header",>
                    <nav class="nav",>
                        <button class=desk,
                                onclick=|_| Msg::Replace(Model::Two),>{ "Two" }</button>
                        <button class=settings,
                                onclick=|_| Msg::Replace(Model::Three),>{ "Three" }</button>
                    </nav>
                </header>
            </div>
        }
    }
}

fn main() {
    yew::initialize();
    let context = Context { };
    let app: App<_, Model> = App::new(context);
    app.mount_to_body();
    yew::run_loop();
}
