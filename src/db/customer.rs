use crate::models::customer::{Customer, CustomerDocument, CustomerInput};
use chrono::Utc;
use futures::stream::TryStreamExt;
use mongodb::{
    bson::{doc, oid::ObjectId, DateTime, Document},
    Database,
};
use rocket::serde::json::Json;

pub async fn find_customer(
    db: &Database,
    limit: i64,
    page: i64,
) -> mongodb::error::Result<Vec<Customer>> {
    let collection = db.collection::<CustomerDocument>("customer");

    let mut cursor = collection
        .find(doc! { "name": doc! { "$exists": true } })
        .limit(limit)
        .skip(u64::try_from((page - 1) * limit).unwrap())
        .await?;

    let mut customers: Vec<Customer> = vec![];
    while let Some(result) = cursor.try_next().await? {
        let _id = result.id;
        let name = result.name;
        let created_at = result.created_at;
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

    let Some(customer_doc) = collection.find_one(doc! {"_id":oid }).await? else {
        return Ok(None);
    };

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
        .insert_one(doc! {"name": input.name.clone(), "createdAt": created_at})
        .await?;

    Ok(insert_one_result.inserted_id.to_string())
}

pub async fn update_customer_by_id(
    db: &Database,
    oid: ObjectId,
    input: Json<CustomerInput>,
) -> mongodb::error::Result<Option<Customer>> {
    let collection = db.collection::<CustomerDocument>("customer");
    let created_at: DateTime = DateTime::now();

    let find = collection
        .find_one_and_update(
            doc! { "_id": oid },
            doc! { "$set": doc! { "name": input.name.clone(), "createdAt": created_at } },
        )
        .return_document(mongodb::options::ReturnDocument::After)
        .await;

    let Some(customer_doc) = find? else {
        return Ok(None);
    };

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
    let Some(customer_doc) = collection.find_one_and_delete(doc! {"_id":oid }).await? else {
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

#[cfg(test)]
mod tests {
    use super::*;

    #[rocket::async_test]
    async fn test_find_customer() {
        let db = initialize_test_database().await;

        insert_test_customers(&db).await;
        let result = find_customer(&db, 10, 1).await;

        assert!(result.is_ok());

        let customers = result.unwrap();
        assert_eq!(customers.len(), 10);

        cleanup_test_database(&db).await;
    }

    #[rocket::async_test]
    async fn test_find_customer_by_id() {
        let db = initialize_test_database().await;

        let customer_id = insert_test_customer(&db).await;

        let result = find_customer_by_id(&db, customer_id).await;

        assert!(result.is_ok());

        let customer = result.unwrap();
        assert_eq!(customer.is_some(), true);

        cleanup_test_database(&db).await;
    }

    #[rocket::async_test]
    async fn test_insert_customer() {
        let db = initialize_test_database().await;

        let input = Json(CustomerInput {
            name: "John Doe".to_string(),
        });

        let result = insert_customer(&db, input).await;

        assert!(result.is_ok());

        let inserted_id = result.unwrap();
        assert_eq!(inserted_id.is_empty(), false);

        cleanup_test_database(&db).await;
    }

    #[rocket::async_test]
    async fn test_update_customer_by_id() {
        let db = initialize_test_database().await;

        let customer_id = insert_test_customer(&db).await;

        let input = Json(CustomerInput {
            name: "Updated Name".to_string(),
        });

        let result = update_customer_by_id(&db, customer_id, input).await;

        assert!(result.is_ok());

        let customer = result.unwrap();
        assert_eq!(customer.is_some(), true);
        assert_eq!(customer.unwrap().name, "Updated Name");

        cleanup_test_database(&db).await;
    }

    #[rocket::async_test]
    async fn test_delete_customer_by_id() {
        let db = initialize_test_database().await;

        let customer_id = insert_test_customer(&db).await;

        let result = delete_customer_by_id(&db, customer_id).await;

        assert!(result.is_ok());

        let customer = result.unwrap();
        assert_eq!(customer.is_some(), true);

        cleanup_test_database(&db).await;
    }

    // Helper functions for test setup and cleanup

    async fn initialize_test_database() -> Database {
        let client = mongodb::Client::with_uri_str("mongodb://localhost:27017")
            .await
            .unwrap();
        let db = client.database("test_db");

        db.collection::<CustomerDocument>("customer")
            .drop()
            .await
            .unwrap();

        db
    }

    async fn cleanup_test_database(db: &Database) {
        db.collection::<CustomerDocument>("customer")
            .drop()
            .await
            .unwrap();
    }

    async fn insert_test_customers(db: &Database) {
        let collection = db.collection::<CustomerDocument>("customer");

        for i in 1..=20 {
            let customer = CustomerDocument {
                id: ObjectId::new(),
                name: format!("Customer {}", i),
                created_at: Utc::now(),
            };

            collection.insert_one(customer).await.unwrap();
        }
    }

    async fn insert_test_customer(db: &Database) -> ObjectId {
        let collection = db.collection::<CustomerDocument>("customer");

        let customer = CustomerDocument {
            id: ObjectId::new(),
            name: "Test Customer".to_string(),
            created_at: Utc::now(),
        };

        let id_cloned = customer.id.clone();
        collection.insert_one(customer).await.unwrap();

        id_cloned
    }
}
