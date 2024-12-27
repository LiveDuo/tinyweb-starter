
use std::cell::RefCell;

use json::JsonValue;

use tinyweb::callbacks::{create_async_callback, promise};
use tinyweb::router::{Router, Page};
use tinyweb::runtime::Runtime;
use tinyweb::signals::Signal;
use tinyweb::element::El;

use tinyweb::invoke::*;

#[derive(Clone, Debug)]
struct Task { title: String, done: bool }

thread_local! {
    pub static ROUTER: RefCell<Router> = RefCell::new(Router::default());
}

async fn fetch_json(method: &str, url: &str, body: Option<JsonValue>) -> Result<JsonValue, String> {
    let body = body.map(|s| s.dump()).unwrap_or_default();
    let (callback_ref, future) = create_async_callback();
    let request = r#"
        const options = { method: {}, headers: { 'Content-Type': 'application/json' }, body: p0 !== 'GET' ? {} : null };
        fetch({}, options).then(r => r.json()).then(r => { {}(r) })
    "#;
    Js::invoke(request, &[method.into(), body.into(), url.into(), callback_ref.into()]);
    let result_ref = future.await;
    let result = Js::invoke("return JSON.stringify({})", &[result_ref.into()]).to_str().unwrap();
    json::parse(&result).map_err(|_| "Parse error".to_owned())
}

fn task_component(index: usize, task: &Task, signal_tasks: &'static Signal<Vec<Task>>) -> El {

    let is_done = task.done.clone();
    let text_classes = if is_done { vec!["line-through"] } else { vec![] };

    El::new("li")
        .classes(&["border-b", "border-gray-200", "flex", "items-center", "justify-between", "py-4"])
        .child(El::new("div").classes(&["flex", "items-center"])
            .child(El::new("input").attr("id", &format!("checkbox-{}", index)).attr_fn("checked", "", move || is_done).attr("type", "checkbox").classes(&["mr-2"]))
                .on_event("change", move |_s| {

                    let checkbox_id = format!("#checkbox-{}", index);
                    let checkbox_element = Js::invoke("return document.querySelector({})", &[checkbox_id.into()]).to_ref().unwrap();
                    let checked = Js::invoke("return {}[{}]", &[checkbox_element.into(), "checked".into()]).to_bool().unwrap();

                    let mut tasks = signal_tasks.get();
                    tasks[index].done = checked;
                    signal_tasks.set(tasks.clone());

                    let task = tasks[index].clone();
                    Runtime::block_on(async move {
                        let body = json::object!{ title: task.title, done: task.done };
                        let url = format!("/api/tasks/{}", index);
                        let result = fetch_json("PUT", &url, Some(body)).await.unwrap();
                        let success = result["success"].as_bool().unwrap();
                        assert!(success);
                    });
                })
            .child(El::new("span").classes(&text_classes).text(&task.title))
        )
        .child(El::new("div")
            .child(El::new("button").text("Edit").classes(&["text-blue-500", "hover:text-blue-700"])
                .on_event("click", move |_s| {
                    let title = Js::invoke("return prompt({},{})", &["New title".into(), "".into()]).to_str().unwrap();
                    let mut tasks = signal_tasks.get();
                    tasks[index].title = title;
                    signal_tasks.set(tasks.clone());

                    let task = tasks[index].clone();
                    Runtime::block_on(async move {
                        let body = json::object!{ title: task.title, done: task.done };
                        let url = format!("/api/tasks/{}", index);
                        let result = fetch_json("PUT", &url, Some(body)).await.unwrap();
                        let success = result["success"].as_bool().unwrap();
                        assert!(success);
                    });
                }))
            .child(El::new("button").text("Delete").classes(&["text-red-500", "hover:text-red-700", "ml-2"])
                .on_event("click", move |_s| {
                    let mut tasks = signal_tasks.get();
                    tasks.remove(index);
                    signal_tasks.set(tasks.clone());

                    Runtime::block_on(async move {
                        let url = format!("/api/tasks/{}", index);
                        let result = fetch_json("DELETE", &url, None).await.unwrap();
                        let success = result["success"].as_bool().unwrap();
                        assert!(success);
                    });
                }))
        )
}

fn container_component() -> El {

    // signals
    let signal_time = Signal::new("-");
    let signal_tasks = Signal::new(vec![]);

    El::new("div")
        .on_mount(move |_| {

            // start timer
            Runtime::block_on(async move {
                loop {
                    signal_time.set("⏰ tik");
                    promise("window.setTimeout({},{})", move |c| vec![c.into(), 1_000.into()]).await;

                    signal_time.set("⏰ tok");
                    promise("window.setTimeout({},{})", move |c| vec![c.into(), 1_000.into()]).await;
                }
            });

            Runtime::block_on(async move {
                let result = fetch_json("GET", "/api/tasks", None).await.unwrap();
                let tasks = result["tasks"].members().map(|s| {
                    Task { title: s["title"].as_str().unwrap().to_string(), done: s["done"].as_bool().unwrap() }
                }).collect::<Vec<_>>();
                signal_tasks.set(tasks);

            });
        })
        .classes(&["mx-auto", "my-10", "w-1/2", "bg-white", "shadow-md", "rounded-lg", "p-6"])
        .child(El::new("div").classes(&["flex", "mb-4"])
            .child(El::new("input").attr("id", "title").attr("placeholder", "Add task").classes(&["w-full", "p-2", "mr-2", "rounded", "focus:outline-none"]))
            .child(El::new("button").text("Add").classes(&["bg-blue-500", "hover:bg-blue-700", "text-white", "p-2", "rounded", "m-2"]).on_event("click", move |_s| {

                let title_element = Js::invoke("return document.querySelector({})", &["#title".into()]).to_ref().unwrap();
                let title = Js::invoke("return {}[{}]", &[title_element.into(), "value".into()]).to_str().unwrap();

                if title == "" {
                    Js::invoke("alert({})", &["Task can't be empty".into()]);
                    return
                }
                let mut tasks = signal_tasks.get();
                tasks.push(Task { title: title.clone(), done: false });
                signal_tasks.set(tasks);

                Runtime::block_on(async move {
                    let body = json::object!{ title: title, done: false };
                    let result = fetch_json("POST", "/api/tasks", Some(body)).await.unwrap();
                    let success = result["success"].as_bool().unwrap();
                    assert!(success);
                });

                Js::invoke("{}[{}] = {}", &[title_element.into(), "value".into(), "".into()]);
            }))
        )
        .child(El::new("div")
            .on_mount(move |el| {
                let el_clone = el.clone();
                signal_tasks.on(move |v| {
                    let el_clone = el_clone.clone();
                    el_clone.children(&v.iter().enumerate()
                        .map(|(i, t)| task_component(i, t, signal_tasks))
                        .collect::<Vec<_>>());
                });
            })
        )
        .child(El::new("div")
            .classes(&["m-2", "flex"])
            .child(El::new("span").text("-").classes(&["ml-2"]).on_mount(move |el| {
                let el_clone = el.clone();
                signal_time.on(move |v| { Js::invoke("{}.innerHTML = {}", &[el_clone.element.into(), v.to_string().into()]); });
            }))
            .child(El::new("span").text("-").classes(&["ml-auto", "mr-2"]).on_mount(move |el| {
                let el_clone = el.clone();
                signal_tasks.on(move |tasks| {
                    let message = format!("Total: {}", tasks.len());
                    Js::invoke("{}.innerHTML = {}", &[el_clone.element.into(), message.into()]);
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

    std::panic::set_hook(Box::new(|e| { Js::invoke("console.log({})", &[e.to_string().into()]); }));

    // init router
    let pages = &[Page::new("/tasks", tasks_page(), None), Page::new("/about", about_page(), None)];
    ROUTER.with(|s| { *s.borrow_mut() = Router::new("body", pages); });
}
