use crate::models::customer::{Customer, CustomerDocument, CustomerInput};
use chrono::Utc;
use futures::stream::TryStreamExt;
use mongodb::{
    bson::{doc, oid::ObjectId, DateTime, Document},
    options::{FindOneAndUpdateOptions, FindOptions, ReturnDocument},
    Database,
};
use rocket::serde::json::Json;

pub async fn find_customer(
    db: &Database,
    limit: i64,
    page: i64,
) -> mongodb::error::Result<Vec<Customer>> {
    let collection = db.collection::<CustomerDocument>("customer");

    let find_options = FindOptions::builder()
        .limit(limit)
        .skip(u64::try_from((page - 1) * limit).unwrap())
        .build();

    let mut cursor = collection.find(None, find_options).await?;

    let mut customers: Vec<Customer> = vec![];
    while let Some(result) = cursor.try_next().await? {
        let _id = result.id;
        let name = result.name;
        let created_at = result.created_at;
        // transform ObjectId to String
        let customer_json = Customer {
            id: _id.to_string(),
            name: name.to_string(),
            created_at: created_at.to_string(),
        };
        customers.push(customer_json);
    }

    Ok(customers)
}

pub async fn find_customer_by_id(
    db: &Database,
    oid: ObjectId,
) -> mongodb::error::Result<Option<Customer>> {
    let collection = db.collection::<CustomerDocument>("customer");

    let Some(customer_doc) = collection.find_one(doc! {"_id":oid }, None).await? else {
        return Ok(None);
    };

    // transform ObjectId to String
    let customer_json = Customer {
        id: customer_doc.id.to_string(),
        name: customer_doc.name.to_string(),
        created_at: customer_doc.created_at.to_string(),
    };

    Ok(Some(customer_json))
}

pub async fn insert_customer(
    db: &Database,
    input: Json<CustomerInput>,
) -> mongodb::error::Result<String> {
    let collection = db.collection::<Document>("customer");

    let created_at = Utc::now();

    let insert_one_result = collection
        .insert_one(
            doc! {"name": input.name.clone(), "createdAt": created_at},
            None,
        )
        .await?;

    Ok(insert_one_result.inserted_id.to_string())
}

pub async fn update_customer_by_id(
    db: &Database,
    oid: ObjectId,
    input: Json<CustomerInput>,
) -> mongodb::error::Result<Option<Customer>> {
    let collection = db.collection::<CustomerDocument>("customer");
    let find_one_and_update_options = FindOneAndUpdateOptions::builder()
        .return_document(ReturnDocument::After)
        .build();

    let created_at: DateTime = DateTime::now();

    let Some(customer_doc) = collection
        .find_one_and_update(
            doc! {"_id":oid },
            doc! {"name": input.name.clone(), "createdAt": created_at},
            find_one_and_update_options,
        )
        .await?
    else {
        return Ok(None);
    };

    // transform ObjectId to String
    let customer_json = Customer {
        id: customer_doc.id.to_string(),
        name: customer_doc.name.to_string(),
        created_at: customer_doc.created_at.to_string(),
    };

    Ok(Some(customer_json))
}

pub async fn delete_customer_by_id(
    db: &Database,
    oid: ObjectId,
) -> mongodb::error::Result<Option<Customer>> {
    let collection = db.collection::<CustomerDocument>("customer");

    // if you just unwrap,, when there is no document it results in 500 error.
    let Some(customer_doc) = collection
        .find_one_and_delete(doc! {"_id":oid }, None)
        .await?
    else {
        return Ok(None);
    };

    // transform ObjectId to String
    let customer_json = Customer {
        id: customer_doc.id.to_string(),
        name: customer_doc.name.to_string(),
        created_at: customer_doc.created_at.to_string(),
    };

    Ok(Some(customer_json))
}
