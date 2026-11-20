use std::cell::RefCell;
use wasm_bindgen::prelude::*;
use wasm_bindgen::JsCast;
use web_sys::{Document, HtmlInputElement};

struct Todo {
    text: String,
    done: bool,
}

thread_local! {
    static TODOS: RefCell<Vec<Todo>> = RefCell::new(Vec::new());
}

fn window() -> web_sys::Window {
    web_sys::window().expect("no window")
}

fn document() -> Document {
    window().document().expect("no document")
}

fn save() {
    let json = TODOS.with(|todos| {
        let todos = todos.borrow();
        let items: Vec<String> = todos
            .iter()
            .map(|t| {
                format!(
                    "{{\"text\":\"{}\",\"done\":{}}}",
                    t.text.replace('\\', "\\\\").replace('"', "\\\""),
                    t.done
                )
            })
            .collect();
        format!("[{}]", items.join(","))
    });
    if let Ok(Some(storage)) = window().local_storage() {
        let _ = storage.set_item("todos", &json);
    }
}

fn load() {
    if let Ok(Some(storage)) = window().local_storage() {
        if let Ok(Some(data)) = storage.get_item("todos") {
            let mut parsed = Vec::new();
            for entry in data.trim_matches(|c| c == '[' || c == ']').split("},{") {
                let entry = entry.trim_matches(|c| c == '{' || c == '}');
                let text = entry
                    .split("\"text\":\"")
                    .nth(1)
                    .and_then(|s| s.split("\",\"done\":").next())
                    .unwrap_or("")
                    .replace("\\\"", "\"")
                    .replace("\\\\", "\\");
                let done = entry.ends_with("\"done\":true");
                if !text.is_empty() {
                    parsed.push(Todo { text, done });
                }
            }
            TODOS.with(|todos| *todos.borrow_mut() = parsed);
        }
    }
}

fn render() {
    let doc = document();
    let list = doc.get_element_by_id("todo-list").expect("no #todo-list");
    list.set_inner_html("");

    TODOS.with(|todos| {
        for (i, todo) in todos.borrow().iter().enumerate() {
            let li = doc.create_element("li").unwrap();
            li.set_class_name(if todo.done { "done" } else { "" });

            let span = doc.create_element("span").unwrap();
            span.set_text_content(Some(&todo.text));
            li.append_child(&span).unwrap();

            // Toggle on click
            let toggle = Closure::wrap(Box::new(move || {
                TODOS.with(|todos| {
                    if let Some(t) = todos.borrow_mut().get_mut(i) {
                        t.done = !t.done;
                    }
                });
                save();
                render();
            }) as Box<dyn FnMut()>);
            li.add_event_listener_with_callback("click", toggle.as_ref().unchecked_ref())
                .unwrap();
            toggle.forget();

            let btn = doc.create_element("button").unwrap();
            btn.set_text_content(Some("✕"));
            btn.set_class_name("delete");
            let remove = Closure::wrap(Box::new(move |e: web_sys::Event| {
                e.stop_propagation();
                TODOS.with(|todos| {
                    let mut todos = todos.borrow_mut();
                    if i < todos.len() {
                        todos.remove(i);
                    }
                });
                save();
                render();
            }) as Box<dyn FnMut(_)>);
            btn.add_event_listener_with_callback("click", remove.as_ref().unchecked_ref())
                .unwrap();
            remove.forget();

            li.append_child(&btn).unwrap();
            list.append_child(&li).unwrap();
        }
    });

    let count = TODOS.with(|todos| todos.borrow().iter().filter(|t| !t.done).count());
    doc.get_element_by_id("count")
        .unwrap()
        .set_text_content(Some(&format!("{} offen", count)));
}

fn add_todo() {
    let input = document()
        .get_element_by_id("new-todo")
        .unwrap()
        .dyn_into::<HtmlInputElement>()
        .unwrap();
    let text = input.value().trim().to_string();
    if !text.is_empty() {
        TODOS.with(|todos| todos.borrow_mut().push(Todo { text, done: false }));
        input.set_value("");
        save();
        render();
    }
}

#[wasm_bindgen(start)]
pub fn main() -> Result<(), JsValue> {
    load();
    render();

    let doc = document();
    let add_btn = doc.get_element_by_id("add-btn").unwrap();
    let on_add = Closure::wrap(Box::new(add_todo) as Box<dyn FnMut()>);
    add_btn
        .add_event_listener_with_callback("click", on_add.as_ref().unchecked_ref())?;
    on_add.forget();

    let input = doc.get_element_by_id("new-todo").unwrap();
    let on_key = Closure::wrap(Box::new(move |e: web_sys::Event| {
        if let Ok(ke) = e.dyn_into::<web_sys::KeyboardEvent>() {
            if ke.key() == "Enter" {
                add_todo();
            }
        }
    }) as Box<dyn FnMut(_)>);
    input.add_event_listener_with_callback("keydown", on_key.as_ref().unchecked_ref())?;
    on_key.forget();

    Ok(())
}
