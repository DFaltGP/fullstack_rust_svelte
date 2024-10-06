use sqlx::{prelude::FromRow, postgres::PgPool, Pool};
use std::net::{TcpListener, TcpStream};
use std::io::{Read, Write};
use dotenv::dotenv;
use std::env;

type DatabaseError = sqlx::Error;

#[macro_use]
extern crate serde_derive;

#[derive(Serialize, Deserialize, FromRow)]
struct User {
  id: Option<i32>,
  name: String,
  email: String,
}

const OK_RESPONSE: &str = "HTTP/1.1 200 OK\r\nContent-Type: application/json\r\nAccess-Control-Allow-Origin: *\r\nAccess-Control-Allow-Methods: GET, POST, PUT, DELETE\r\nAccess-Control-Allow-Headers: Content-Type\r\n\r\n";
const NOT_FOUND: &str = "HTTP/1.1 404 NOT FOUND\r\n\r\n";
const INTERNAL_ERROR: &str = "HTTP/1.1 500 INTERNAL ERROR\r\n\r\n";

#[tokio::main]
async fn main() {
  dotenv().ok();

  match set_database().await {
    Ok(pool) => {
      let listener: TcpListener = TcpListener::bind("0.0.0.0:8080").unwrap();
      println!("Server listening on port 8080");
    
      for stream in listener.incoming() {
        match stream {
          Ok(stream) => {
            handle_client(stream, &pool).await;
          },
          Err(e) => eprintln!("Unable to connect: {e}")
        }
      }
    },
    Err(e) => eprintln!("Error starting database: {e}")
  }
}

async fn set_database() -> Result<Pool<sqlx::Postgres>, DatabaseError> {
  let db_url = env::var("DATABASE_URL").expect("DATABASE_URL must be setted");

  let pool = PgPool::connect(&db_url).await?;

  // Default migration path './migrations'
  if let Err(e) = sqlx::migrate!().run(&pool).await {
    eprintln!("Error migrating database: {e}")
  };
  
  Ok(pool)
}

fn get_user_id(request: &str) -> &str {
  request.split("/").nth(4).unwrap_or_default().split_whitespace().next().unwrap_or_default()
}

fn get_user_request_body(request: &str) -> Result<User, serde_json::Error> {
  serde_json::from_str(request.split("\r\n\r\n").last().unwrap_or_default())
}

// Controller
async fn handle_client(mut stream: TcpStream, pool: &Pool<sqlx::Postgres>) {
  let mut buffer = [0; 1024];
  let mut request: String = String::new();

  match stream.read(&mut buffer) {
    Ok(size) => {
      request.push_str(String::from_utf8_lossy(&buffer[..size]).as_ref());

      let (status_line, content) = match &*request {
        r if r.starts_with("OPTIONS") => (OK_RESPONSE.to_string(), "".to_string()),
        r if r.starts_with("POST /api/rust/users") => handle_post_request(r, pool).await,
        r if r.starts_with("GET /api/rust/users/") => handle_get_request(r, pool).await,
        r if r.starts_with("GET /api/rust/users") => handle_get_all_request(r, pool).await,
        r if r.starts_with("PUT /api/rust/users/") => handle_put_request(r, pool).await,
        r if r.starts_with("DELETE /api/rust/users/") => handle_delete_request(r, pool).await,
        _ => (NOT_FOUND.to_string(), "This route does not exists on our service".to_string()),
      };

      stream.write_all(format!("{}{}", status_line, content).as_bytes()).unwrap();
    }, 
    Err(e) => eprintln!("Unable to read stream: {e}"),
  }
}

async fn handle_post_request(request: &str, pool: &Pool<sqlx::Postgres>) -> (String, String) {
  let body = match get_user_request_body(request) {
      Ok(body) => body,
      Err(_) => return (INTERNAL_ERROR.to_string(), "Internal Error".to_string()),
  };

  let new_user_id: (i32,) = match sqlx::query_as("INSERT INTO users (name, email) VALUES ($1, $2) RETURNING id")
      .bind(&body.name)
      .bind(&body.email)
      .fetch_one(pool)
      .await {
          Ok(id) => id,
          Err(e) => return (INTERNAL_ERROR.to_string(), format!("Error: {e}")),
      };

  let new_user: User = sqlx::query_as("SELECT * FROM users WHERE id = $1")
      .bind(new_user_id.0)
      .fetch_one(pool)
      .await
      .unwrap(); 

  (OK_RESPONSE.to_string(), serde_json::to_string(&new_user).unwrap_or_default())
}

async fn handle_get_request(request: &str, pool: &Pool<sqlx::Postgres>) -> (String, String) {
  let user_id: &str = get_user_id(request);
  let user_id: i32 = match user_id.parse::<i32>() {
    Ok(id) => id,
    Err(_) => {
      eprintln!("Error parsing ID");
      return (INTERNAL_ERROR.to_string(), "Internal Error".to_string())
    }
  }; 

  let user: User = match sqlx::query_as("SELECT * FROM users WHERE id = $1")
    .bind(user_id)
    .fetch_one(pool).await {
      Ok(user) => user,
      Err(e) => {
        eprintln!("Unable to find user: {e}");
        return (NOT_FOUND.to_string(), "404 NOT FOUND".to_string());
      }
    };

  let user_json = match serde_json::to_string(&user) {
    Ok(user_json) => user_json,
    Err(_) => {
      eprintln!("Error serializing user to JSON");
      return (INTERNAL_ERROR.to_string(), "Internal Error".to_string());
    }
  };

  (OK_RESPONSE.to_string(), user_json)

}

async fn handle_get_all_request(_request: &str, pool: &Pool<sqlx::Postgres>) -> (String, String) {
  let users: Vec<User> = match sqlx::query_as("SELECT * FROM users").fetch_all(pool).await {
    Ok(users) => users,
    Err(_) => {
      eprintln!("Error serializing user to JSON");
      return (INTERNAL_ERROR.to_string(), "Internal Error".to_string());
    }
  };

  let users_json = match serde_json::to_string(&users) {
    Ok(users_json) => users_json,
    Err(_) => {
      eprintln!("Error serializing user to JSON");
      return (INTERNAL_ERROR.to_string(), "Internal Error".to_string());
    }
  };

  (OK_RESPONSE.to_string(), users_json)
}

async fn handle_put_request(request: &str, pool: &Pool<sqlx::Postgres>) -> (String, String) {
  let (id, body): (i32, User) = match (get_user_id(request).parse::<i32>(), get_user_request_body(request)) {
    (Ok(id), Ok(user)) => (id, user),
    (Err(e), _) => {
      eprintln!("Unable to parse ID: {e}");
      return (INTERNAL_ERROR.to_string(), "Internal Error".to_string())
    },
    (_, Err(e)) => {
      eprintln!("Unable to get user info from body: {e}");
      return (INTERNAL_ERROR.to_string(), "Internal Error".to_string())
    },
  };

  match sqlx::query("UPDATE users SET name = $1, email = $2 WHERE id = $3")
    .bind(body.name)
    .bind(body.email)
    .bind(id)
    .execute(pool).await {
      Ok(_) => return (OK_RESPONSE.to_string(), "User updated successfully".to_string()),
      Err(e) => {
        eprintln!("Unable to update user: {e}");
        return (INTERNAL_ERROR.to_string(), "Internal Error".to_string())
      }
    };
}

async fn handle_delete_request(request: &str, pool: &Pool<sqlx::Postgres>) -> (String, String) {
  let user_id = match get_user_id(request).parse::<i32>() {
    Ok(user_id) => user_id,
    Err(e) => {
      eprintln!("Unable to parse ID: {e}");
      return (INTERNAL_ERROR.to_string(), "Unable to delete user".to_string())
    }
  };

  match sqlx::query("DELETE FROM users WHERE id = $1")
    .bind(user_id)
    .execute(pool)
    .await {
      Ok(_) => return (OK_RESPONSE.to_string(), "User deleted successfully".to_string()),
      Err(_) => {
        eprintln!("Unable to delete user");
        return (INTERNAL_ERROR.to_string(), "Internal Error".to_string())
      }
    } 
}