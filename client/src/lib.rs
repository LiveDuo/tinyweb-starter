
use std::cell::RefCell;
use std::collections::HashMap;

use json::JsonValue;
use tinyweb::element::{El, Router, Page};
use tinyweb::signals::Signal;

use tinyweb::bindings::{console, dom, http_request, history};
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

fn task_component(index: usize, task: &Task, signal_tasks: Signal<Vec<Task>>) -> El {

    let signal_tasks_clone = signal_tasks.clone();
    let signal_tasks_clone_2 = signal_tasks.clone();
    let signal_tasks_clone_3 = signal_tasks.clone();
    El::new("li")
        .classes(&["border-b", "border-gray-200", "flex", "items-center", "justify-between", "py-4"])
        .child(El::new("div").classes(&["flex", "items-center"])
            .child(El::new("input").attr("id", &format!("checkbox-{}", index)).attr("value", &task.done.to_string()).attr("type", "checkbox").classes(&["mr-2"]))
                .on_change(move |s| {

                    let checkbox_element = dom::query_selector(&format!("#checkbox-{}", index));
                    set_property_bool(&checkbox_element, "checked", s.value == "false");

                    let mut tasks = signal_tasks_clone.get();
                    tasks[index].done = s.value == "false";
                    signal_tasks_clone.set(tasks.clone());

                    console::console_log(&format!("{:?} {:?}", tasks, s.value));
                })
            .child(El::new("span").text(&task.title))
        )
        .child(El::new("div")
            .child(El::new("button").text("Edit").classes(&["text-blue-500", "hover:text-blue-700"]))
                .on_click(move |_s| {
                    let title = dom::prompt("New title", "");
                    let mut tasks = signal_tasks_clone_2.get();
                    tasks[index].title = title;
                    signal_tasks_clone_2.set(tasks);
                })
            .child(El::new("button").text("Delete").classes(&["text-red-500", "hover:text-red-700", "ml-2"]))
                .on_click(move |_s| {
                    let mut tasks = signal_tasks_clone_3.get();
                    tasks.remove(index);
                    signal_tasks_clone_3.set(tasks);
                })
        )
}

fn container_component() -> El {

    // time signal
    let signal_time = Signal::new("-");
    let signal_time_clone = signal_time.clone();

    // tasks signal
    let signal_tasks = Signal::new(vec![Task { title: "title".to_owned(), done: false }]);
    let signal_tasks_clone = signal_tasks.clone();
    let signal_tasks_clone_2 = signal_tasks.clone();
    let signal_tasks_clone_3 = signal_tasks.clone();
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

            let signal_tasks_clone_3 = signal_tasks_clone_3.clone();

            tinyweb::runtime::run(async move {
                let result = fetch_json(HTTPMethod::GET, format!("/api/tasks"), None).await;
                let tasks = result["tasks"].members().map(|s| {
                    Task { title: s["title"].as_str().unwrap().to_string(), done: s["done"].as_bool().unwrap() }
                }).collect::<Vec<_>>();
                signal_tasks_clone_3.set(tasks);

            });
        })
        .classes(&["mx-auto", "my-10", "w-1/2", "bg-white", "shadow-md", "rounded-lg", "p-6"])
        .child(El::new("div").classes(&["flex", "mb-4"])
            .child(El::new("input").attr("id", "title").attr("placeholder", "Add task").classes(&["w-full", "p-2", "mr-2", "rounded", "focus:outline-none"]))
            .child(El::new("button").text("Add").classes(&["bg-blue-500", "hover:bg-blue-700", "text-white", "p-2", "rounded", "m-2"]).on_click(move |_s| {

                let title_element = dom::query_selector("#title");
                let title = get_property_string(&title_element, "value");

                if title == "" {
                    dom::alert("Task can't be empty");
                    return
                }
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
            .classes(&["m-2", "flex"])
            .child(El::new("span").text("-").classes(&["ml-2"]).on_mount(move |el| {
                let el_clone = el.clone();
                signal_time.on(move |v| { dom::element_set_inner_html(&el_clone, &v.to_string()); });
            }))
            .child(El::new("span").text("-").classes(&["ml-auto", "mr-2"]).on_mount(move |el| {
                let el_clone = el.clone();
                signal_tasks.on(move |tasks| {
                    dom::element_set_inner_html(&el_clone, &format!("Total: {}", tasks.len())); }
                );
            }))
        )
}

fn tasks_page() -> El {
    let body = El::new("div")
        .child(El::new("button").text("about").classes(&["underline", "hover:opacity-50", "m-2"]).on_click(move |_| {
            ROUTER.with(|s| { s.borrow().navigate("/about"); });
        }))
        .child(container_component());
    layout_component(&[body])
}

fn about_page() -> El {
    let body = El::new("div")
        .child(El::new("button").text("tasks").classes(&["underline", "hover:opacity-50", "m-2"]).on_click(move |_| {
            ROUTER.with(|s| { s.borrow().navigate("/tasks"); });
        }))
        .child(El::new("div").text("This is the about page").classes(&["m-2"]));
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
    let pages = [
        ("/tasks".to_owned(), Page { element: tasks_page(), title: None }),
        ("/about".to_owned(), Page { element: about_page(), title: None })
    ];

    // load page
    let body = dom::query_selector("body");
    let (_, page) = pages.iter().find(|&(s, _)| *s == history::location_pathname()).unwrap_or(&pages[0]);
    page.element.mount(&body);

    // init router
    ROUTER.with(|s| {
        let pages_map = HashMap::<String, Page>::from_iter(pages);
        *s.borrow_mut() = Router { pages: HashMap::from_iter(pages_map), root: Some(body) };
    });
}
