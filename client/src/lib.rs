
use std::collections::HashMap;
use std::future::Future;
use std::cell::RefCell;

use json::JsonValue;

use tinyweb::handlers::create_future_callback;
use tinyweb::runtime::{Runtime, RuntimeFuture};
use tinyweb::router::{Router, Page};
use tinyweb::signals::Signal;
use tinyweb::element::El;

use tinyweb::http::*;
use tinyweb::invoke::*;

#[derive(Clone, Debug)]
struct Task { title: String, done: bool }

thread_local! {
    pub static ROUTER: RefCell<Router> = RefCell::new(Router::default());
}

async fn fetch_json(method: HttpMethod, url: &str, body: Option<JsonValue>) -> Result<JsonValue, String> {
    let body_temp = body.map(|s| s.dump());
    let body = body_temp.as_ref().map(|s| s.as_str());
    let fetch_options = FetchOptions { method, url, body, ..Default::default() };
    match fetch(fetch_options).await {
        FetchResponse::Text(_, result) => json::parse(&result).map_err(|_| "Parse error".to_owned()),
        _ => Err("Fetch error".to_owned())
    }
}

pub fn sleep(ms: impl Into<f64>) -> impl Future<Output = ()> {
    let future = RuntimeFuture::new();
    let callback_ref = create_future_callback(future.id());
    Js::invoke("window.setTimeout({},{})", &[Ref(callback_ref), Number(ms.into())]);
    future
}

fn task_component(index: usize, task: &Task, signal_tasks: Signal<Vec<Task>>) -> El {

    let signal_tasks_clone = signal_tasks.clone();
    let signal_tasks_clone_2 = signal_tasks.clone();
    let signal_tasks_clone_3 = signal_tasks.clone();

    let is_done = task.done.clone();
    let text_classes = if is_done { vec!["line-through"] } else { vec![] };

    El::new("li")
        .classes(&["border-b", "border-gray-200", "flex", "items-center", "justify-between", "py-4"])
        .child(El::new("div").classes(&["flex", "items-center"])
            .child(El::new("input").attr("id", &format!("checkbox-{}", index)).attr_fn("checked", "", move || is_done).attr("type", "checkbox").classes(&["mr-2"]))
                .on_event("change", move |_s| {

                    let checkbox_id = &format!("#checkbox-{}", index);
                    let checkbox_element = Js::invoke("return document.querySelector({})", &[Str(checkbox_id.into())]).to_ref().unwrap();
                    let checked = Js::invoke("return {}[{}]", &[Ref(checkbox_element), Str("checked".into())]).to_bool().unwrap();

                    let mut tasks = signal_tasks_clone.get();
                    tasks[index].done = checked;
                    signal_tasks_clone.set(tasks.clone());

                    let task = tasks[index].clone();
                    Runtime::block_on(async move {
                        let body = json::object!{ title: task.title, done: task.done };
                        let url = format!("/api/tasks/{}", index);
                        let result = fetch_json(HttpMethod::PUT, &url, Some(body)).await.unwrap();
                        let success = result["success"].as_bool().unwrap();
                        assert!(success);
                    });
                })
            .child(El::new("span").classes(&text_classes).text(&task.title))
        )
        .child(El::new("div")
            .child(El::new("button").text("Edit").classes(&["text-blue-500", "hover:text-blue-700"])
                .on_event("click", move |_s| {
                    let title = Js::invoke("return prompt({},{})", &[Str("New title".into()), Str("".into())]).to_str().unwrap();
                    let mut tasks = signal_tasks_clone_2.get();
                    tasks[index].title = title;
                    signal_tasks_clone_2.set(tasks.clone());

                    let task = tasks[index].clone();
                    Runtime::block_on(async move {
                        let body = json::object!{ title: task.title, done: task.done };
                        let url = format!("/api/tasks/{}", index);
                        let result = fetch_json(HttpMethod::PUT, &url, Some(body)).await.unwrap();
                        let success = result["success"].as_bool().unwrap();
                        assert!(success);
                    });
                }))
            .child(El::new("button").text("Delete").classes(&["text-red-500", "hover:text-red-700", "ml-2"])
                .on_event("click", move |_s| {
                    let mut tasks = signal_tasks_clone_3.get();
                    tasks.remove(index);
                    signal_tasks_clone_3.set(tasks.clone());

                    Runtime::block_on(async move {
                        let url = format!("/api/tasks/{}", index);
                        let result = fetch_json(HttpMethod::DELETE, &url, None).await.unwrap();
                        let success = result["success"].as_bool().unwrap();
                        assert!(success);
                    });
                }))
        )
}

fn container_component() -> El {

    // time signal
    let signal_time = Signal::new("-");
    let signal_time_clone = signal_time.clone();

    // tasks signal
    let signal_tasks = Signal::new(vec![]);
    let signal_tasks_clone = signal_tasks.clone();
    let signal_tasks_clone_2 = signal_tasks.clone();
    let signal_tasks_clone_3 = signal_tasks.clone();
    El::new("div")
        .on_mount(move |_| {

            // start timer
            let signal_time_clone = signal_time_clone.clone();
            Runtime::block_on(async move {
                loop {
                    signal_time_clone.set("⏰ tik");
                    sleep(1_000).await;
                    signal_time_clone.set("⏰ tok");
                    sleep(1_000).await;
                }
            });

            let signal_tasks_clone_3 = signal_tasks_clone_3.clone();

            Runtime::block_on(async move {
                let result = fetch_json(HttpMethod::GET, "/api/tasks", None).await.unwrap();
                let tasks = result["tasks"].members().map(|s| {
                    Task { title: s["title"].as_str().unwrap().to_string(), done: s["done"].as_bool().unwrap() }
                }).collect::<Vec<_>>();
                signal_tasks_clone_3.set(tasks);

            });
        })
        .classes(&["mx-auto", "my-10", "w-1/2", "bg-white", "shadow-md", "rounded-lg", "p-6"])
        .child(El::new("div").classes(&["flex", "mb-4"])
            .child(El::new("input").attr("id", "title").attr("placeholder", "Add task").classes(&["w-full", "p-2", "mr-2", "rounded", "focus:outline-none"]))
            .child(El::new("button").text("Add").classes(&["bg-blue-500", "hover:bg-blue-700", "text-white", "p-2", "rounded", "m-2"]).on_event("click", move |_s| {

                let title_element = Js::invoke("return document.querySelector({})", &[Str("#title".into())]).to_ref().unwrap();
                let title = Js::invoke("return {}[{}]", &[Ref(title_element), Str("value".into())]).to_str().unwrap();

                if title == "" {
                    Js::invoke("alert({})", &[Str("Task can't be empty".into())]);
                    return
                }
                let mut tasks = signal_tasks_clone.get();
                tasks.push(Task { title: title.clone(), done: false });
                signal_tasks_clone.set(tasks);

                Runtime::block_on(async move {
                    let body = json::object!{ title: title, done: false };
                    let result = fetch_json(HttpMethod::POST, "/api/tasks", Some(body)).await.unwrap();
                    let success = result["success"].as_bool().unwrap();
                    assert!(success);
                });

                Js::invoke("{}[{}] = {}", &[Ref(title_element), Str("value".into()), Str("".into())]);
            }))
        )
        .child(El::new("div")
            .on_mount(move |el| {
                let el_clone = el.clone();
                let signal_clone = signal_tasks_clone_2.clone();
                signal_tasks_clone_2.on(move |v| {
                    el_clone.clone().children(&v.iter().enumerate()
                        .map(|(i, t)| task_component(i, t, signal_clone.clone()))
                        .collect::<Vec<_>>());
                });
            })
        )
        .child(El::new("div")
            .classes(&["m-2", "flex"])
            .child(El::new("span").text("-").classes(&["ml-2"]).on_mount(move |el| {
                let el_clone = el.clone();
                signal_time.on(move |v| { Js::invoke("{}.innerHTML = {}", &[Ref(*el_clone), Str(v.to_string())]); });
            }))
            .child(El::new("span").text("-").classes(&["ml-auto", "mr-2"]).on_mount(move |el| {
                let el_clone = el.clone();
                signal_tasks.on(move |tasks| {
                    let message = format!("Total: {}", tasks.len());
                    Js::invoke("{}.innerHTML = {}", &[Ref(*el_clone), Str(message.into())]);
                });
            }))
        )
}

fn tasks_page() -> El {
    let body = El::new("div")
        .child(El::new("button").text("about").classes(&["underline", "hover:opacity-50", "m-2"]).on_event("click", move |_| {
            ROUTER.with(|s| { s.borrow().navigate("/about"); });
        }))
        .child(container_component());
    layout_component(&[body])
}

fn about_page() -> El {
    let body = El::new("div")
        .child(El::new("button").text("tasks").classes(&["underline", "hover:opacity-50", "m-2"]).on_event("click", move |_| {
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

    std::panic::set_hook(Box::new(|e| { Js::invoke("console.log({})", &[Str(e.to_string())]); }));

    // get pages
    let pages = [
        ("/tasks".to_owned(), Page { element: tasks_page(), title: None }),
        ("/about".to_owned(), Page { element: about_page(), title: None }),
        ("/".to_owned(), Page { element: tasks_page(), title: None })
    ];

    // load page
    let body = Js::invoke("return document.querySelector({})", &[Str("body".into())]).to_ref().unwrap();
    let pathname = Js::invoke("return window.location.pathname", &[]).to_str().unwrap();
    let (_, page) = pages.iter().find(|&(s, _)| *s == pathname).unwrap_or(&pages[0]);
    page.element.mount(&body);

    // init router
    ROUTER.with(|s| {
        let pages_map = HashMap::<String, Page>::from_iter(pages);
        *s.borrow_mut() = Router { pages: HashMap::from_iter(pages_map), root: Some(body) };
    });
}
