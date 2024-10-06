
use std::cell::RefCell;
use std::collections::HashMap;

use json::JsonValue;
use tinyweb::element::{El, Router};
use tinyweb::signals::{Signal, SignalAsync};

use tinyweb::bindings::{console, dom, http_request};
use tinyweb::bindings::http_request::*;

#[derive(Clone)]
struct Task { title: String, done: bool }

const BUTTON_CLASSES: &[&str] = &["bg-blue-500", "hover:bg-blue-700", "text-white", "p-2", "rounded", "m-2"];

thread_local! {
    pub static ROUTER: RefCell<Router> = RefCell::new(Router::default());
}

async fn fetch_json(method: HTTPMethod, url: String, body: Option<JsonValue>) -> JsonValue {
    let body_temp = body.map(|s| s.dump());
    let body = body_temp.as_ref().map(|s| s.as_str());
    let fetch_options = FetchOptions { action: method, url: &url, body, ..Default::default()};
    let fetch_res = http_request::fetch(fetch_options).await;
    let result = match fetch_res { FetchResponse::Text(_, d) => Ok(d), _ => Err(()), };
    json::parse(&result.unwrap()).unwrap()
}

fn task(title: &str) -> El {
    El::new("li")
        .classes(&["border-b", "border-gray-200", "flex", "items-center", "justify-between", "py-4"])
        .child(El::new("div").classes(&["flex", "items-center"])
            .child(El::new("input").attr("type", "checkbox").classes(&["mr-2"]))
            .child(El::new("span").text(title))
        )
        .child(El::new("div")
            .child(El::new("button").text("Edit").classes(&["text-blue-500", "hover:text-blue-700", "mr-2"]))
            .child(El::new("button").text("Delete").classes(&["text-red-500", "hover:text-red-700"]))
        )
}

fn container() -> El {

    let tasks = ["Item 1", "Item 2"];
    let mut children = El::new("div");
    for _task in tasks {
        children = children.child(task(_task));
    }

    El::new("div")
        .classes(&["mx-auto", "my-10", "w-1/2", "bg-white", "shadow-md", "rounded-lg", "p-6"])
        .child(El::new("div").classes(&["flex", "mb-4"])
            .child(El::new("input").attr("placeholder", "Add task").classes(&["w-full", "px-4" ,"py-2", "mr-2", "rounded", "focus:outline-none"]))
            .child(El::new("button").text("Add").classes(BUTTON_CLASSES))
        )
        .child(children)
}

fn tasks_page() -> El {

    // tasks signal
    let signal_tasks = Signal::new(vec![]);
    let signal_tasks_clone = signal_tasks.clone();
    
    // time signal
    let signal_time = SignalAsync::new("-");
    let signal_time_clone = signal_time.clone();

    El::new("div")
        .on_mount(move |_| {

            // start timer
            let signal_time_clone = signal_time_clone.clone();
            tinyweb::runtime::run(async move {
                loop {
                    signal_time_clone.set("⏰ tik");
                    tinyweb::bindings::utils::sleep(1_000).await;
                    signal_time_clone.set("⏰ tok");
                    tinyweb::bindings::utils::sleep(1_000).await;
                }
            });

        })
        .classes(&["m-2"])
        .child(El::new("button").text("api").classes(BUTTON_CLASSES).on_click(|_| {
            tinyweb::runtime::run(async move {
                let result = fetch_json(HTTPMethod::GET, format!("/api/ping"), None).await;
                dom::alert(&format!("{}", result["pong"].as_bool().unwrap()));
            });
        }))
        .child(El::new("button").text("update").classes(BUTTON_CLASSES).on_click(move |_| {
            let mut tasks = signal_tasks_clone.get();
            tasks.push(Task { title: "title".to_owned(), done: false });

            signal_tasks_clone.set(tasks);
        }))
        .child(El::new("button").text("about").classes(BUTTON_CLASSES).on_click(move |_| {
            ROUTER.with(|s| { s.borrow().navigate("about"); });
        }))
        .child(El::new("div").text("-").on_mount(move |el| {
            let el_clone = el.clone();
            signal_tasks.on(move |v| {
                if let Some(task) = v.last() {
                    dom::element_set_inner_html(&el_clone, &format!("{} - {}", task.title, task.done)); }
                }
            );
        }))
        .child(El::new("div").text("-").on_mount(move |el| {
            let el_clone = el.clone();
            signal_time.on(move |v| { dom::element_set_inner_html(&el_clone, &v.to_string()); });
        }))
        .child(container())
}

fn about_page() -> El {
    El::new("div")
        .classes(&["m-2"])
        .child(El::new("button").text("tasks").classes(BUTTON_CLASSES).on_click(move |_| {
            ROUTER.with(|s| { s.borrow().navigate("tasks"); });
        }))
}

#[no_mangle]
pub fn main() {

    std::panic::set_hook(Box::new(|e| console::console_log(&e.to_string())));

    // get pages
    let tasks_page = tasks_page();
    let about_page = about_page();
    
    // mount page
    let body = dom::query_selector("body");
    tasks_page.mount(&body);
    
    // set state
    let pages_iter = [("tasks".to_owned(), (tasks_page, None)), ("about".to_owned(), (about_page, None))];
    let pages = HashMap::<String, (El, Option<String>)>::from_iter(pages_iter);
    ROUTER.with(|s| {
        *s.borrow_mut() = Router { pages: HashMap::from_iter(pages), root: Some(body) };
    });

}
