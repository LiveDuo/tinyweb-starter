
use std::cell::RefCell;
use std::collections::HashMap;

use tinyweb::element::El;
use tinyweb::js::ExternRef;
use tinyweb::signals::{Signal, SignalAsync};

use tinyweb::bindings::{console, dom, history, http_request};
use tinyweb::bindings::http_request::{FetchOptions, FetchResponse, FetchResponseType};

const BUTTON_CLASSES: &[&str] = &["bg-blue-500", "hover:bg-blue-700", "text-white", "p-2", "rounded", "m-2"];

#[derive(Debug, Default)]
struct State { root: Option<ExternRef>, pages: HashMap::<String, El> }

impl State {
    fn navigate(&self, page: &str) {

        history::history_push_state("test", page);

        let body = self.root.as_ref().unwrap();
        dom::element_set_inner_html(&body, "");
        
        let el = self.pages.get(page).unwrap();
        el.mount(&body);
    }
}

thread_local! {
    pub static STATE: RefCell<State> = Default::default();
}

async fn fetch_array_buffer(url: &str) -> Result<Vec<u8>, String> {
    let fetch_options = FetchOptions { url, response_type: FetchResponseType::ArrayBuffer, ..Default::default()};
    match http_request::fetch(fetch_options).await {
        FetchResponse::ArrayBuffer(_, ab) => Ok(ab),
        FetchResponse::Text(_, _) => Err("Invalid response".to_owned()),
    }
}

fn get_pokemon() {
    tinyweb::runtime::run(async move {
        let result = fetch_array_buffer("/api/ping").await.unwrap();
        let string = String::from_utf8(result).unwrap();
        let value = json::parse(&string).unwrap();
        dom::alert(&format!("{}", value["pong"].as_bool().unwrap()));
    });
}

fn page1() -> El {

    // count signal
    let signal_count = Signal::new(0);
    let signal_count_clone = signal_count.clone();
    
    // time signal
    let signal_time = SignalAsync::new("-");
    let signal_time_clone = signal_time.clone();

    El::new("div")
        .on_mount(move |_| {

            // start timer
            let signal_time_clone = signal_time_clone.clone();
            tinyweb::runtime::coroutine(async move {
                loop {
                    signal_time_clone.set("⏰ tik");
                    tinyweb::bindings::utils::sleep(1_000).await;
                    signal_time_clone.set("⏰ tok");
                    tinyweb::bindings::utils::sleep(1_000).await;
                }
            });

        })
        .classes(&["m-2"])
        .child(El::new("button").text("api").classes(&BUTTON_CLASSES).on_click(|_| { get_pokemon(); }))
        .child(El::new("button").text("page 2").classes(&BUTTON_CLASSES).on_click(move |_| {
            STATE.with(|s| { s.borrow().navigate("page2"); });
        }))
        .child(El::new("br"))
        .child(El::new("button").text("add").classes(&BUTTON_CLASSES).on_click(move |_| {
            let count = signal_count_clone.get() + 1;
            signal_count_clone.set(count);
        }))
        .child(El::new("div").text("0").on_mount(move |el| {
            let el_clone = el.clone();
            signal_count.on(move |v| { dom::element_set_inner_html(&el_clone, &v.to_string()); });
        }))
        .child(El::new("div").text("-").on_mount(move |el| {
            let el_clone = el.clone();
            signal_time.on(move |v| { dom::element_set_inner_html(&el_clone, &v.to_string()); });
        }))
}

fn page2() -> El {
    El::new("div")
        .classes(&["m-2"])
        .child(El::new("button").text("page 1").classes(&BUTTON_CLASSES).on_click(move |_| {
            STATE.with(|s| { s.borrow().navigate("page1"); });
        }))
}

#[no_mangle]
pub fn main() {

    std::panic::set_hook(Box::new(|e| console::console_log(&e.to_string())));

    // get pages
    let page1 = page1();
    let page2 = page2();
    
    // mount page
    let body = dom::query_selector("body");
    page1.mount(&body);
    
    // set state
    let pages_iter = [("page1".to_owned(), page1), ("page2".to_owned(), page2)];
    let pages = HashMap::<String, El>::from_iter(pages_iter);
    let state = State { pages, root: Some(body) };
    STATE.with(|s| { *s.borrow_mut() = state; });

}
