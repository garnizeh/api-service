use axum::{
    extract::{Path, State},
    http::StatusCode,
    response::IntoResponse,
    Json,
};
use chrono::NaiveDateTime;
use serde::Serialize;
use serde_json::json;
use sqlx::SqlitePool;

use crate::todo::{CreateTodo, Todo, UpdateTodo};

#[derive(Serialize, Clone)]
pub struct TodoResponse {
    pub id: i64,
    pub body: String,
    pub completed: bool,
    pub created_at: NaiveDateTime,
    pub updated_at: NaiveDateTime,
}

// Convert DB Model to Response
fn to_todo_response(todo: &Todo) -> TodoResponse {
    TodoResponse {
        id: todo.id.to_owned(),
        body: todo.body.to_owned(),
        completed: todo.completed.to_owned(),
        created_at: todo.created_at.to_owned(),
        updated_at: todo.updated_at.to_owned(),
    }
}

pub async fn ping(
    State(dbpool): State<SqlitePool>,
) -> Result<impl IntoResponse, (StatusCode, Json<serde_json::Value>)> {
    use sqlx::Connection;

    let mut conn = dbpool.acquire().await.map_err(|e| {
        let error_response = serde_json::json!({
            "status": "error",
            "message": format!("Pool acquire error: { }", e),
        });
        (StatusCode::INTERNAL_SERVER_ERROR, Json(error_response))
    })?;

    conn.ping().await.map_err(|e| {
        let error_response = serde_json::json!({
            "status": "error",
            "message": format!("Database error: { }", e),
        });
        (StatusCode::INTERNAL_SERVER_ERROR, Json(error_response))
    })?;

    let json_response = serde_json::json!({
        "status": "healthy",
    });

    Ok(Json(json_response))
}

pub async fn todo_list(
    State(dbpool): State<SqlitePool>,
) -> Result<impl IntoResponse, (StatusCode, Json<serde_json::Value>)> {
    let query_list_todos = Todo::list(dbpool).await.map_err(|e| {
        let error_response = serde_json::json!({
            "status": "error",
            "message": format!("Database error: { }", e),
        });
        (StatusCode::INTERNAL_SERVER_ERROR, Json(error_response))
    })?;

    let todo_responses = query_list_todos
        .iter()
        .map(|todo| to_todo_response(&todo))
        .collect::<Vec<TodoResponse>>();

    let json_response = serde_json::json!({
        "status": "ok",
        "count": todo_responses.len(),
        "notes": todo_responses
    });

    Ok(Json(json_response))
}

pub async fn todo_read(
    State(dbpool): State<SqlitePool>,
    Path(id): Path<i64>,
) -> Result<impl IntoResponse, (StatusCode, Json<serde_json::Value>)> {
    let query_todo = Todo::read(dbpool, id).await;

    match query_todo {
        Ok(todo) => {
            let todo_response = serde_json::json!({
                "status": "success",
                "data": serde_json::json!({
                    "todo": to_todo_response(&todo)
                })
            });

            return Ok(Json(todo_response));
        }
        Err(sqlx::Error::RowNotFound) => {
            let error_response = serde_json::json!({
                "status": "fail",
                "message": format!("todo with ID: {} not found", id)
            });
            return Err((StatusCode::NOT_FOUND, Json(error_response)));
        }
        Err(e) => {
            return Err((
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(json!({"status": "error","message": format!("{:?}", e)})),
            ));
        }
    };
}

pub async fn todo_create(
    State(dbpool): State<SqlitePool>,
    Json(new_todo): Json<CreateTodo>,
) -> Result<impl IntoResponse, (StatusCode, Json<serde_json::Value>)> {
    let create_todo = Todo::create(dbpool, new_todo).await;

    match create_todo {
        Ok(todo) => {
            let todo_response = serde_json::json!({
                "status": "success",
                "data": serde_json::json!({
                    "todo": to_todo_response(&todo)
                })
            });

            return Ok(Json(todo_response));
        }
        Err(e) => {
            return Err((
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(json!({"status": "error","message": format!("{:?}", e)})),
            ));
        }
    };
}

pub async fn todo_update(
    State(dbpool): State<SqlitePool>,
    Path(id): Path<i64>,
    Json(updated_todo): Json<UpdateTodo>,
) -> Result<impl IntoResponse, (StatusCode, Json<serde_json::Value>)> {
    let update_todo = Todo::update(dbpool, id, updated_todo).await;

    match update_todo {
        Ok(todo) => {
            let todo_response = serde_json::json!({
                "status": "success",
                "data": serde_json::json!({
                    "todo": to_todo_response(&todo)
                })
            });

            return Ok(Json(todo_response));
        }
        Err(e) => {
            return Err((
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(json!({"status": "error","message": format!("{:?}", e)})),
            ));
        }
    };
}

pub async fn todo_delete(
    State(dbpool): State<SqlitePool>,
    Path(id): Path<i64>,
) -> Result<impl IntoResponse, (StatusCode, Json<serde_json::Value>)> {
    let delete_todo = Todo::delete(dbpool, id).await;

    match delete_todo {
        Ok(_) => {
            return Ok(StatusCode::NO_CONTENT);
        }
        Err(e) => {
            return Err((
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(json!({"status": "error","message": format!("{:?}", e)})),
            ));
        }
    };
}
