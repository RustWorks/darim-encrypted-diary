use chrono::Utc;
use diesel::prelude::*;

use crate::models::{db_connection, error::ServiceError, post::*};
use crate::schema::posts;

pub fn get_list() -> Result<Vec<Post>, ServiceError> {
    let conn = db_connection::connect();
    let post_list: Vec<Post> = posts::table
        .order(posts::created_at.desc())
        .load::<Post>(&conn)?;
    Ok(post_list)
}

pub fn create(args: CreateArgs) -> Result<bool, ServiceError> {
    if args.author.trim().is_empty() || args.content.trim().is_empty() {
        println!("{}", ServiceError::InvalidArgument);
        return Err(ServiceError::InvalidArgument);
    }

    let conn = db_connection::connect();

    let post = PostToCreate {
        author: args.author,
        content: args.content,
    };
    let count = diesel::insert_into(posts::table)
        .values(post)
        .execute(&conn)?;

    if count < 1 {
        println!("{}", ServiceError::QueryExecutionFailure);
        Err(ServiceError::QueryExecutionFailure)
    } else {
        Ok(true)
    }
}

pub fn delete(id: u64) -> Result<bool, ServiceError> {
    let conn = db_connection::connect();

    let target_post = posts::table.find(id);
    let count = diesel::delete(target_post).execute(&conn)?;

    if count < 1 {
        println!("{}", ServiceError::NotFound(id));
        Err(ServiceError::NotFound(id))
    } else {
        Ok(true)
    }
}

pub fn update(id: u64, args: UpdateArgs) -> Result<bool, ServiceError> {
    if args.author.is_none() && args.content.is_none() {
        println!("{}", ServiceError::InvalidArgument);
        return Err(ServiceError::InvalidArgument);
    }

    if let (Some(author), Some(content)) = (&args.author, &args.content) {
        if author.trim().is_empty() || content.trim().is_empty() {
            println!("{}", ServiceError::InvalidArgument);
            return Err(ServiceError::InvalidArgument);
        }
    }

    let conn = db_connection::connect();

    let post = PostToUpdate {
        author: args.author,
        content: args.content,
        updated_at: Some(Utc::now().naive_utc()),
    };

    let target_post = posts::table.find(id);
    let count = diesel::update(target_post).set(post).execute(&conn)?;

    if count < 1 {
        println!("{}", ServiceError::NotFound(id));
        Err(ServiceError::NotFound(id))
    } else {
        Ok(true)
    }
}
