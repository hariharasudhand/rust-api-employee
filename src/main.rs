#[macro_use] extern crate rocket;

use rocket::serde::{json::Json, Serialize, Deserialize};
use rocket::response::status;
use rocket::State;
use rocket::tokio::fs::{self, File};
use rocket::tokio::io::AsyncWriteExt;
use rocket::tokio::sync::Mutex;
use uuid::Uuid;
use std::collections::HashMap;
use std::sync::Arc;

#[derive(Debug, Serialize, Deserialize, Clone)]
struct Employee {
    id: String,
    name: String,
    age: u8,
    position: String,
}

type Db = Arc<Mutex<HashMap<String, Employee>>>;

#[derive(Deserialize)]
struct NewEmployee {
    name: String,
    age: u8,
    position: String,
}

#[post("/employees", data = "<new_employee>")]
async fn add_employee(state: &State<Db>, new_employee: Json<NewEmployee>) -> status::Created<String> {
    let mut db = state.lock().await;

    let id = Uuid::new_v4().to_string();
    let employee = Employee {
        id: id.clone(),
        name: new_employee.name.clone(),
        age: new_employee.age,
        position: new_employee.position.clone(),
    };

    db.insert(id.clone(), employee);

    save_to_file(&db).await;

    status::Created::new("/employees").body(format!("Employee added with ID: {}", id))
}

#[get("/employees")]
async fn list_employees(state: &State<Db>) -> String {
    let db = state.lock().await;
    serde_json::to_string(&db.values().collect::<Vec<&Employee>>()).unwrap()
}

#[get("/employees/<id>")]
async fn get_employee(state: &State<Db>, id: String) -> Option<String> {
    let db = state.lock().await;
    db.get(&id).map(|employee| serde_json::to_string(employee).unwrap())
}

#[delete("/employees/<id>")]
async fn delete_employee(state: &State<Db>, id: String) -> status::NoContent {
    let mut db = state.lock().await;
    db.remove(&id);
    save_to_file(&db).await;
    status::NoContent
}

async fn save_to_file(db: &HashMap<String, Employee>) {
    let data = serde_json::to_string(db).unwrap();
    let mut file = File::create("db.json").await.unwrap();
    file.write_all(data.as_bytes()).await.unwrap();
}

async fn load_from_file() -> HashMap<String, Employee> {
    if let Ok(data) = fs::read_to_string("db.json").await {
        serde_json::from_str(&data).unwrap_or_else(|_| HashMap::new())
    } else {
        HashMap::new()
    }
}

#[launch]
async fn rocket() -> _ {
    let db = load_from_file().await;
    let db = Arc::new(Mutex::new(db));

    rocket::build()
        .manage(db)
        .mount("/", routes![add_employee, list_employees, get_employee, delete_employee])
}
