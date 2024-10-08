
use std::cell::RefCell;
use std::collections::HashMap;

use json::JsonValue;
use tinyweb::element::{El, Router};
use tinyweb::signals::{Signal, SignalAsync};

use tinyweb::bindings::{console, dom, http_request};
use tinyweb::bindings::http_request::*;
use tinyweb::bindings::utils::*;

#[derive(Clone, Debug)]
struct Task { title: String, done: bool }

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

fn task_component(_index: usize, task: &Task, _signal_tasks: Signal<Vec<Task>>) -> El {

    El::new("li")
        .classes(&["border-b", "border-gray-200", "flex", "items-center", "justify-between", "py-4"])
        .child(El::new("div").classes(&["flex", "items-center"])
            .child(El::new("input").attr("value", &task.done.to_string()).attr("type", "checkbox").classes(&["mr-2"]))
            .child(El::new("span").text(&task.title))
        )
        .child(El::new("div")
            .child(El::new("button").text("Edit").classes(&["text-blue-500", "hover:text-blue-700"]))
            .child(El::new("button").text("Delete").classes(&["text-red-500", "hover:text-red-700", "ml-2"]))
        )
}

fn container_component() -> El {

    // time signal
    let signal_time = SignalAsync::new("-");
    let signal_time_clone = signal_time.clone();

    // tasks signal
    let signal_tasks = Signal::new(vec![Task { title: "title".to_owned(), done: false }]);
    let signal_tasks_clone = signal_tasks.clone();
    let signal_tasks_clone_2 = signal_tasks.clone();
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

            tinyweb::runtime::run(async move {
                let result = fetch_json(HTTPMethod::GET, format!("/api/tasks"), None).await;
                let tasks = result["tasks"].members().map(|s| {
                    Task { title: s["title"].as_str().unwrap().to_string(), done: s["done"].as_bool().unwrap() }
                }).collect::<Vec<_>>();

                console::console_log(&format!("{:?}", tasks));
                // TODO signal_tasks_clone_3.set(tasks);

            });
        })
        .classes(&["mx-auto", "my-10", "w-1/2", "bg-white", "shadow-md", "rounded-lg", "p-6"])
        .child(El::new("div").classes(&["flex", "mb-4"])
            .child(El::new("input").attr("id", "title").attr("placeholder", "Add task").classes(&["w-full", "p-2", "mr-2", "rounded", "focus:outline-none"]))
            .child(El::new("button").text("Add").classes(&["bg-blue-500", "hover:bg-blue-700", "text-white", "p-2", "rounded", "m-2"]).on_click(move |_s| {

                let title_element = dom::query_selector("#title");
                let title = get_property_string(&title_element, "value");

                let mut tasks = signal_tasks_clone.get();
                tasks.push(Task { title: title.clone(), done: false });
                signal_tasks_clone.set(tasks);

                tinyweb::runtime::run(async move {
                    let body = json::object!{ title: title, done: false };
                    let result = fetch_json(HTTPMethod::POST, format!("/api/tasks"), Some(body)).await;
                    let success = result["success"].as_bool().unwrap();
                    assert!(success);
                });

                set_property_string(&title_element, "value", "");
            }))
        )
        .child(El::new("div")
            .on_mount(move |el| {
                let el_clone = el.clone();
                let signal_clone = signal_tasks_clone_2.clone();
                signal_tasks_clone_2.on(move |v| {
                    el_clone.children(&v.iter().enumerate()
                        .map(|(i, t)| task_component(i, t, signal_clone.clone()))
                        .collect::<Vec<_>>());
                });
            })
        )
        .child(El::new("div")
            .classes(&["m-2", "text-center"])
            .child(El::new("span").text("-").on_mount(move |el| {
                let el_clone = el.clone();
                signal_tasks.on(move |tasks| {
                    dom::element_set_inner_html(&el_clone, &format!("Total: {}", tasks.len())); }
                );
            }))
            .child(El::new("span").text("-").classes(&["ml-2"]).on_mount(move |el| {
                let el_clone = el.clone();
                signal_time.on(move |v| { dom::element_set_inner_html(&el_clone, &v.to_string()); });
            }))
        )
}

fn tasks_page() -> El {
    let body = El::new("div")
        .child(El::new("button").text("about").classes(&["underline", "hover:opacity-50", "m-2"]).on_click(move |_| {
            ROUTER.with(|s| { s.borrow().navigate("about"); });
        }))
        .child(container_component());
    layout_component(&[body])
}

fn about_page() -> El {
    let body = El::new("div")
        .child(El::new("button").text("tasks").classes(&["underline", "hover:opacity-50", "m-2"]).on_click(move |_| {
            ROUTER.with(|s| { s.borrow().navigate("tasks"); });
        }));
    layout_component(&[body])
}

fn layout_component(children: &[El]) -> El {
    El::new("div")
        .classes(&["m-2", "mt-8", "text-center"])
        .children(children)
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
